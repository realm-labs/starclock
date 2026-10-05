//! Real HP-loss commands prove the native policy, not complete equipment parity.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    tests::weighted_curio_footstep_fixture::{Probe, begin, cast, concede, remainder, scenario},
    weighted_curio_footstep::{HpLossPointPolicy, HpLossPointPolicyError},
};
use starclock_build::light_cone::CombatPath;
use starclock_combat::{
    Battle, BattleEvent, BattleEventKind, Command, ResourceEventData, Scalar, TeamSide,
};
use starclock_replay::battle_event::encode_battle_event_payload;

fn points(battle: &Battle) -> u16 {
    battle.view().team(TeamSide::Player).skill_points()
}
fn grants(events: &[BattleEvent]) -> Vec<(u16, u16, u16)> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Resource(ResourceEventData::SkillPoints {
                attempted,
                effective,
                overflow,
                ..
            }) => Some((*attempted, *effective, *overflow)),
            _ => None,
        })
        .collect()
}
fn assert_remainder(battle: &Battle, owner: u64, expected: i64) {
    assert_eq!(
        remainder(battle, owner),
        Scalar::checked_from_integer(expected).unwrap()
    );
}

#[test]
fn weighted_curio_footstep_hp_loss_accumulates_across_healing_and_each_committed_operation() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let run = || {
        let mut battle = scenario(&fixture, &Probe::default());
        begin(&mut battle);
        let mut events = cast(&mut battle, 1, 1);
        assert_remainder(&battle, 1, 30);
        assert_eq!(points(&battle), 0);
        assert_eq!(grants(&events), [(0, 0, 0)]);
        events.extend(cast(&mut battle, 2, 1));
        assert_remainder(&battle, 1, 30);
        events.extend(cast(&mut battle, 1, 1));
        assert_remainder(&battle, 1, 10);
        assert_eq!(points(&battle), 1);
        // Two separate HP events within one action, with a heal between them.
        events.extend(cast(&mut battle, 7, 1));
        assert_remainder(&battle, 1, 20);
        assert_eq!(points(&battle), 2);
        assert_eq!(battle.view().rng_draw_count(), 0);
        let payloads = events
            .iter()
            .map(|event| encode_battle_event_payload(event).unwrap())
            .collect::<Vec<_>>();
        (events, payloads, battle.state_hash())
    };
    assert_eq!(run(), run());
}

#[test]
fn weighted_curio_footstep_hp_loss_uses_exact_original_paths_not_unrelated_or_inherited_rosters() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for path in [
        CombatPath::Destruction,
        CombatPath::Remembrance,
        CombatPath::Hunt,
        CombatPath::Erudition,
        CombatPath::Harmony,
        CombatPath::Nihility,
        CombatPath::Preservation,
        CombatPath::Abundance,
        CombatPath::Elation,
    ] {
        let mut battle = scenario(
            &fixture,
            &Probe {
                path,
                loss: 60,
                inherit_buddy: true,
                ..Probe::default()
            },
        );
        begin(&mut battle);
        let eligible = matches!(path, CombatPath::Destruction | CombatPath::Remembrance);
        let first = cast(&mut battle, 1, 1);
        assert_eq!(points(&battle), u16::from(eligible));
        assert_remainder(&battle, 1, if eligible { 10 } else { 0 });
        assert_eq!(
            grants(&first),
            if eligible { vec![(1, 1, 0)] } else { vec![] }
        );
        let second = cast(&mut battle, 1, 2);
        assert!(grants(&second).is_empty());
        assert_remainder(&battle, 2, 0);
        assert_eq!(points(&battle), u16::from(eligible));
    }
}

#[test]
fn weighted_curio_footstep_hp_loss_independent_second_owner_is_not_formation_zero() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut battle = scenario(
        &fixture,
        &Probe {
            bind_buddy: true,
            ..Probe::default()
        },
    );
    begin(&mut battle);
    cast(&mut battle, 1, 1);
    cast(&mut battle, 1, 2);
    assert_remainder(&battle, 1, 30);
    assert_remainder(&battle, 2, 30);
    cast(&mut battle, 1, 2);
    assert_eq!(points(&battle), 1);
    assert_remainder(&battle, 1, 30);
    assert_remainder(&battle, 2, 10);
}

#[test]
fn weighted_curio_footstep_hp_loss_excludes_absorbed_damage_and_preserves_effective_hp_floor() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut battle = scenario(&fixture, &Probe::default());
    begin(&mut battle);
    cast(&mut battle, 4, 1);
    cast(&mut battle, 3, 1);
    assert_remainder(&battle, 1, 0);
    cast(&mut battle, 3, 1);
    assert_remainder(&battle, 1, 20);
    cast(&mut battle, 2, 1);
    cast(&mut battle, 1, 1);
    assert_remainder(&battle, 1, 0);
    assert_eq!(points(&battle), 1);
    // The request is 80 but consumption must respect the one-HP floor.
    let mut floor = scenario(
        &fixture,
        &Probe {
            loss: 80,
            ..Probe::default()
        },
    );
    begin(&mut floor);
    cast(&mut floor, 1, 1);
    assert_remainder(&floor, 1, 30);
    let events = cast(&mut floor, 1, 1);
    assert!(events.iter().any(|event|matches!(event.kind(),BattleEventKind::HpConsumption(data) if data.effective.get()==19)));
    assert_remainder(&floor, 1, 49);
    assert_eq!(points(&floor), 1);
}

#[test]
fn weighted_curio_footstep_hp_loss_cap_overflow_consumes_thresholds_without_banking_points() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut battle = scenario(
        &fixture,
        &Probe {
            initial_points: 5,
            loss: 60,
            ..Probe::default()
        },
    );
    begin(&mut battle);
    assert_eq!(grants(&cast(&mut battle, 1, 1)), [(1, 0, 1)]);
    assert_remainder(&battle, 1, 10);
    cast(&mut battle, 8, 1);
    cast(&mut battle, 1, 1); // Only 39 HP can actually be consumed now.
    assert_remainder(&battle, 1, 49);
    assert_eq!(points(&battle), 0);
    cast(&mut battle, 2, 1);
    assert_eq!(grants(&cast(&mut battle, 1, 1)), [(2, 2, 0)]);
    assert_remainder(&battle, 1, 9);
}

#[test]
fn weighted_curio_footstep_hp_loss_uses_live_capacity_and_exact_fractional_thresholds() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut battle = scenario(&fixture, &Probe::default());
    begin(&mut battle);
    cast(&mut battle, 1, 1);
    cast(&mut battle, 5, 1); // Max HP becomes 60; the capacity clamp itself adds no loss.
    assert_remainder(&battle, 1, 30);
    assert_eq!(points(&battle), 0);
    assert_eq!(grants(&cast(&mut battle, 1, 1)), [(2, 2, 0)]);
    assert_remainder(&battle, 1, 0);
    let mut fractional = scenario(
        &fixture,
        &Probe {
            hp: 101,
            loss: 50,
            ..Probe::default()
        },
    );
    begin(&mut fractional);
    cast(&mut fractional, 1, 1);
    assert_eq!(points(&fractional), 0);
    cast(&mut fractional, 2, 1);
    cast(&mut fractional, 1, 1);
    assert_eq!(points(&fractional), 1);
    assert_eq!(remainder(&fractional, 1), Scalar::from_scaled(49_500_000));
}

#[test]
fn weighted_curio_footstep_hp_loss_lethal_event_and_terminal_cleanup_use_bounded_actual_loss() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut battle = scenario(&fixture, &Probe::default());
    begin(&mut battle);
    let events = cast(&mut battle, 9, 1);
    assert_eq!(grants(&events), [(2, 2, 0)]);
    assert_remainder(&battle, 1, 0);
    // Surviving unbound party member can still concede after original defeat.
    concede(&mut battle);
    let mut other = scenario(&fixture, &Probe::default());
    begin(&mut other);
    cast(&mut other, 1, 1);
    concede(&mut other);
    assert_remainder(&other, 1, 0);
    let fresh = scenario(&fixture, &Probe::default());
    assert_remainder(&fresh, 1, 0);
}

#[test]
fn weighted_curio_footstep_hp_loss_stale_commands_are_inert_and_arithmetic_faults_roll_back() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut battle = scenario(&fixture, &Probe::default());
    let stale = Command::StartBattle {
        decision: battle.decision().unwrap().id(),
    };
    begin(&mut battle);
    cast(&mut battle, 1, 1);
    let before = battle.state_hash();
    assert!(battle.apply(stale).is_err());
    assert_eq!(battle.state_hash(), before);
    assert_remainder(&battle, 1, 30);
    assert_eq!(battle.view().rng_draw_count(), 0);
    // A valid tiny threshold generates a point request outside the event's u16 domain.
    let mut bad = scenario(
        &fixture,
        &Probe {
            fraction: Scalar::from_scaled(1),
            ..Probe::default()
        },
    );
    begin(&mut bad);
    let command=bad.decision().unwrap().legal_commands().iter().find(|command|matches!(command,
        Command::UseAbility {ability,primary_target,..} if ability.get()==1 && primary_target.is_some_and(|target|target.get()==1))).unwrap().clone();
    let result = bad.apply(command).unwrap();
    assert!(result.fault().is_some());
    assert_remainder(&bad, 1, 0);
    assert_eq!(points(&bad), 0);
    assert!(!result.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::HpConsumption(_)
            | BattleEventKind::Resource(_)
            | BattleEventKind::RuleState(_)
    )));
    assert_eq!(bad.view().rng_draw_count(), 0);
}

#[test]
fn weighted_curio_footstep_hp_loss_rejects_invalid_fraction_and_binds_policy_identity() {
    for fraction in [
        Scalar::from_scaled(-1),
        Scalar::ZERO,
        Scalar::from_scaled(1_000_001),
    ] {
        assert_eq!(
            HpLossPointPolicy::new(fraction, [0; 32]).unwrap_err(),
            HpLossPointPolicyError::InvalidLossFraction
        );
    }
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let a = scenario(&fixture, &Probe::default());
    let b = scenario(
        &fixture,
        &Probe {
            identity: [0x96; 32],
            ..Probe::default()
        },
    );
    assert_ne!(a.state_hash(), b.state_hash());
}
