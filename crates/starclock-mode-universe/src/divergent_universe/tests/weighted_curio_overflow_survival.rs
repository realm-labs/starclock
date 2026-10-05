//! Nonterminal settlements must not become confirmed-death overflow credit.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    tests::weighted_curio_overflow_fixture::{Probe, cast, id, scenario, start},
};
use starclock_combat::{
    Battle, BattleEvent, BattleEventKind, Command, DamageEventData, DecisionId, EffectDamageGuard,
    EnemyPhaseEventData, LifeState, Ratio, TEAM_DEFEAT_GUARDED_SIGNAL, UnitEventData, UnitId,
    formula::hp::damage_overflow,
};
use starclock_replay::battle_event::encode_battle_event_payload;

fn damage(events: &[BattleEvent]) -> Vec<DamageEventData> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(*data),
            _ => None,
        })
        .collect()
}

fn defeated(events: &[BattleEvent]) -> Vec<u64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Unit(UnitEventData::Defeated { unit, .. }) => Some(unit.get()),
            _ => None,
        })
        .collect()
}

fn overflow(data: &DamageEventData) -> i64 {
    damage_overflow(data.calculated, data.absorbed, data.hp_before)
        .unwrap()
        .get()
}

fn hp(battle: &Battle) -> Vec<i64> {
    battle
        .view()
        .units_by_id()
        .map(|u| u.current_hp().get())
        .collect()
}

fn clean(battle: &Battle) {
    assert!(battle.view().effects_by_id().all(|effect| {
        effect.definition().get() < 0x7f05_0000 || effect.definition().get() >= 0x7f06_0000
    }));
}

#[test]
fn weighted_curio_overflow_phase_reset_preserves_positive_event_overflow_without_conversion() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let probe = Probe {
        phase_targets: 2,
        ..Probe::default()
    };
    let mut a = scenario(&fixture, &probe);
    let mut b = scenario(&fixture, &probe);
    start(&mut a);
    start(&mut b);
    let before = a.state_hash();
    assert!(
        a.apply(Command::UseAbility {
            decision: DecisionId::new(999_999).unwrap(),
            actor: UnitId::new(1).unwrap(),
            ability: id(1),
            primary_target: Some(UnitId::new(3).unwrap()),
        })
        .is_err()
    );
    assert_eq!(a.state_hash(), before);
    assert_eq!(a.view().rng_draw_count(), 0);
    let x = cast(&mut a, 1, Some(3));
    let y = cast(&mut b, 1, Some(3));
    assert_eq!(x, y);
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(
        x.iter()
            .map(|event| encode_battle_event_payload(event).unwrap())
            .collect::<Vec<_>>(),
        y.iter()
            .map(|event| encode_battle_event_payload(event).unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        damage(&x).iter().map(overflow).collect::<Vec<_>>(),
        [150, 170, 0]
    );
    assert!(defeated(&x).is_empty());
    let transitioned =
        x.iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::EnemyPhase(EnemyPhaseEventData::Transitioned {
                    unit, to, ..
                }) if to.get() == 2 => Some(unit.get()),
                _ => None,
            })
            .collect::<Vec<_>>();
    assert_eq!(transitioned, [2, 3]);
    assert_eq!(hp(&a), [10_000, 100, 80, 2_750, 9_000]);
    assert!(
        a.view()
            .units_by_id()
            .all(|unit| unit.life() == LifeState::Alive)
    );
    assert_eq!(a.view().rng_draw_count(), 0);
    clean(&a);
    // The final phase can really die on the next attack. The previous phase's
    // positive DamageOverflow must not leak into this fresh accumulator.
    let events = cast(&mut a, 1, Some(3));
    let replayed = cast(&mut b, 1, Some(3));
    assert_eq!(events, replayed);
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(
        damage(&events)
            .iter()
            .map(|d| d.calculated.get())
            .collect::<Vec<_>>(),
        [250, 250, 250, 2_220]
    );
    assert_eq!(defeated(&events), [2, 3]);
    assert_eq!(hp(&a), [10_000, 0, 0, 280, 9_000]);
    assert_eq!(a.view().rng_draw_count(), 1);
    clean(&a);
}

#[test]
fn weighted_curio_overflow_mixed_phase_and_confirmed_death_counts_only_the_death() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let probe = Probe {
        phase_targets: 1,
        ..Probe::default()
    };
    let mut battle = scenario(&fixture, &probe);
    start(&mut battle);
    let events = cast(&mut battle, 1, Some(3));
    assert!(events.iter().any(|event| matches!(event.kind(),
        BattleEventKind::EnemyPhase(EnemyPhaseEventData::Transitioned { unit, to, .. })
            if unit.get() == 2 && to.get() == 2)));
    assert_eq!(defeated(&events), [3]);
    assert_eq!(
        damage(&events)
            .iter()
            .map(|d| d.calculated.get())
            .collect::<Vec<_>>(),
        [250, 250, 250, 2_070]
    );
    assert_eq!(hp(&battle), [10_000, 100, 0, 680, 9_000]);
    assert_eq!(battle.view().rng_draw_count(), 1);
    clean(&battle);
}

#[test]
fn weighted_curio_overflow_persistent_hp_floor_keeps_targets_alive_and_unready() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let probe = Probe {
        hp_floor: Some(Ratio::from_scaled(250_000)),
        ..Probe::default()
    };
    let mut battle = scenario(&fixture, &probe);
    start(&mut battle);
    let events = cast(&mut battle, 1, Some(3));
    assert_eq!(
        damage(&events)
            .iter()
            .map(|d| d.calculated.get())
            .collect::<Vec<_>>(),
        [75, 60, 250]
    );
    assert!(damage(&events).iter().all(|d| overflow(d) == 0));
    assert!(defeated(&events).is_empty());
    assert_eq!(hp(&battle), [10_000, 25, 20, 2_750, 9_000]);
    assert_eq!(battle.view().rng_draw_count(), 0);
    // Only the two authored HP floors survive, not bridge marks/readiness.
    assert_eq!(battle.view().effects_by_id().count(), 2);
    clean(&battle);
}

#[test]
fn weighted_curio_overflow_team_guard_then_real_death_does_not_credit_prevented_damage() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let probe = Probe {
        guard: Some(EffectDamageGuard::TeamDefeatOnce),
        ..Probe::default()
    };
    let mut battle = scenario(&fixture, &probe);
    start(&mut battle);
    let events = cast(&mut battle, 1, Some(3));
    assert!(events.iter().any(|event| matches!(event.kind(),
        BattleEventKind::RuleSignal(data) if data.code == TEAM_DEFEAT_GUARDED_SIGNAL)));
    assert_eq!(defeated(&events), [3]);
    assert_eq!(
        damage(&events)
            .iter()
            .map(|d| d.calculated.get())
            .collect::<Vec<_>>(),
        [99, 250, 250, 2_070]
    );
    assert_eq!(overflow(&damage(&events)[0]), 0);
    assert_eq!(hp(&battle), [10_000, 1, 0, 680, 9_000]);
    assert_eq!(battle.view().rng_draw_count(), 1);
    assert_eq!(battle.view().effects_by_id().count(), 0);
    clean(&battle);
}
