//! Death-conversion policy executes commands; production admission stays pending.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    tests::weighted_curio_overflow_fixture::{Probe, cast, id, scenario, start},
};
use starclock_combat::{
    Battle, BattleEvent, BattleEventKind, CauseActor, Command, DamageEventData, DecisionId,
    EffectDamageGuard, LifeState, UnitEventData, UnitId, catalog::action::TargetPattern,
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
fn hp(battle: &Battle) -> Vec<i64> {
    battle
        .view()
        .units_by_id()
        .map(|unit| unit.current_hp().get())
        .collect()
}
fn marks(battle: &Battle) -> usize {
    battle
        .view()
        .effects_by_id()
        .filter(|effect| {
            effect.definition().get() >= 0x7f05_0000 && effect.definition().get() < 0x7f06_0000
        })
        .count()
}
fn bytes(events: &[BattleEvent]) -> Vec<Vec<u8>> {
    events
        .iter()
        .map(|event| encode_battle_event_payload(event).unwrap())
        .collect()
}

#[test]
fn weighted_curio_overflow_multiple_deaths_sum_before_conversion_and_exclude_unattacked_maximum() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut battle = scenario(&fixture, &Probe::default());
    start(&mut battle);
    let events = cast(&mut battle, 1, Some(3));
    assert_eq!(
        damage(&events)
            .iter()
            .map(|d| (d.target.get(), d.calculated.get()))
            .collect::<Vec<_>>(),
        [(2, 250), (3, 250), (4, 250), (4, 2220)]
    );
    assert_eq!(hp(&battle), [10_000, 0, 0, 530, 9_000]);
    assert_eq!(
        battle.view().rng_draw_count(),
        1,
        "singleton conversion uses the labeled selector draw"
    );
    assert_eq!(marks(&battle), 0);
    assert_eq!(
        events.last().unwrap().cause().root_command(),
        events[0].cause().root_command()
    );
}

#[test]
fn weighted_curio_overflow_ties_keep_all_maxima_and_fresh_commands_replay_identically() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut winners = Vec::new();
    for seed in 1..=8 {
        let probe = Probe {
            hp: vec![100, 80, 3_000, 3_000],
            levels: vec![1, 81, 95, 95],
            pattern: TargetPattern::All,
            seed,
            ..Probe::default()
        };
        let mut a = scenario(&fixture, &probe);
        let mut b = scenario(&fixture, &probe);
        start(&mut a);
        start(&mut b);
        let before = a.state_hash();
        let draws = a.view().rng_draw_count();
        assert!(
            a.apply(Command::UseAbility {
                decision: DecisionId::new(999_999).unwrap(),
                actor: UnitId::new(1).unwrap(),
                ability: id(1),
                primary_target: None
            })
            .is_err()
        );
        assert_eq!(a.state_hash(), before);
        assert_eq!(a.view().rng_draw_count(), draws);
        let x = cast(&mut a, 1, None);
        let y = cast(&mut b, 1, None);
        assert_eq!(x, y);
        assert_eq!(bytes(&x), bytes(&y));
        assert_eq!(a.state_hash(), b.state_hash());
        let converted = *damage(&x).last().unwrap();
        assert_eq!(converted.calculated.get(), 2220);
        winners.push(converted.target.get());
        assert_eq!(a.view().rng_draw_count(), 1);
        assert_eq!(marks(&a), 0);
    }
    assert_eq!(winners, [5, 4, 5, 4, 4, 4, 4, 4]);
}

#[test]
fn weighted_curio_overflow_empty_pool_and_zero_overflow_do_not_draw_or_leave_marks() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for (hp, expected) in [
        (vec![100, 80], 2),
        (vec![250, 3_000], 2),
        (vec![300, 3_000], 2),
    ] {
        let probe = Probe {
            levels: vec![1, 95],
            hp,
            pattern: TargetPattern::All,
            ..Probe::default()
        };
        let mut battle = scenario(&fixture, &probe);
        start(&mut battle);
        let events = cast(&mut battle, 1, None);
        assert_eq!(damage(&events).len(), expected);
        assert_eq!(battle.view().rng_draw_count(), 0);
        assert_eq!(marks(&battle), 0);
    }
}

#[test]
fn weighted_curio_overflow_explicit_base_expression_uses_selected_targets_level_and_bound_factor() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for (level, factor, expected) in [(1, 1, 330), (81, 2, 1940), (95, 3, 3170)] {
        let probe = Probe {
            levels: vec![1, 81, level, 50],
            base_factor: factor,
            ..Probe::default()
        };
        let mut battle = scenario(&fixture, &probe);
        start(&mut battle);
        let events = cast(&mut battle, 1, Some(3));
        assert_eq!(damage(&events).last().unwrap().calculated.get(), expected);
    }
}

#[test]
fn weighted_curio_overflow_conversion_kill_keeps_credit_without_self_recursion() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let probe = Probe {
        hp: vec![100, 80, 2_000, 9_000],
        ..Probe::default()
    };
    let mut battle = scenario(&fixture, &probe);
    start(&mut battle);
    let events = cast(&mut battle, 1, Some(3));
    assert_eq!(damage(&events).len(), 4);
    assert_eq!(hp(&battle), [10_000, 0, 0, 0, 9_000]);
    let converted = damage(&events).last().unwrap().target;
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .find(|u| u.id() == converted)
            .unwrap()
            .life(),
        LifeState::Defeated
    );
    let credited = events
        .iter()
        .filter(|e| {
            matches!(
                e.kind(),
                BattleEventKind::Unit(UnitEventData::Defeated { .. })
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(credited.len(), 3);
    assert!(
        credited
            .iter()
            .all(|e| e.cause().applier() == Some(UnitId::new(1).unwrap()))
    );
    assert!(credited.iter().all(|event| {
        event.cause().owner() == Some(UnitId::new(1).unwrap())
            && event.cause().actor() == Some(CauseActor::Unit(UnitId::new(1).unwrap()))
    }));
    let converted_event = events
        .iter()
        .rfind(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
        .unwrap();
    assert_eq!(
        converted_event.cause().source_definition().unwrap().get(),
        0x7f04_0020
    );
    assert!(converted_event.cause().action().is_some());
    assert!(converted_event.cause().hit().is_some());
    assert_eq!(battle.view().rng_draw_count(), 1);
    assert_eq!(marks(&battle), 0);
}

#[test]
fn weighted_curio_overflow_policy_identity_binds_immutable_participant_state() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let a = scenario(&fixture, &Probe::default());
    let b = scenario(
        &fixture,
        &Probe {
            base_factor: 3,
            ..Probe::default()
        },
    );
    let c = scenario(&fixture, &Probe::default());
    assert_ne!(a.state_hash(), b.state_hash());
    assert_eq!(a.state_hash(), c.state_hash());
}

#[test]
fn weighted_curio_overflow_multihit_resets_conversion_and_inherited_form_does_not_collect() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let probe = Probe {
        damages: vec![250, 250],
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
        [250, 250, 250, 2220, 250]
    );
    assert_eq!(hp(&battle), [10_000, 0, 0, 280, 9_000]);
    assert_eq!(battle.view().rng_draw_count(), 1);
    assert_eq!(marks(&battle), 0);
    let probe = Probe {
        inherited: true,
        ..Probe::default()
    };
    let mut battle = scenario(&fixture, &probe);
    start(&mut battle);
    // The faster copied form attacks first with the inherited bundle/source.
    let events = cast(&mut battle, 2, Some(4));
    assert_eq!(
        damage(&events).len(),
        3,
        "a copied bundle on another form is not a new eligible original"
    );
    assert_eq!(battle.view().rng_draw_count(), 0);
    assert_eq!(marks(&battle), 0);
}

#[test]
fn weighted_curio_overflow_uses_post_shield_excess_and_guard_prevents_death_confirmation() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let probe = Probe {
        shield: 100,
        ..Probe::default()
    };
    let mut battle = scenario(&fixture, &probe);
    start(&mut battle);
    let events = cast(&mut battle, 1, Some(3));
    let data = damage(&events);
    assert_eq!(
        data.iter().map(|d| d.absorbed.get()).collect::<Vec<_>>(),
        [100, 100, 0, 0]
    );
    assert_eq!(data.last().unwrap().calculated.get(), 2020);
    assert_eq!(hp(&battle), [10_000, 0, 0, 730, 9_000]);
    assert_eq!(battle.view().rng_draw_count(), 1);
    assert_eq!(marks(&battle), 0);
    let probe = Probe {
        shield: 100,
        guard: Some(EffectDamageGuard::ShieldOverflowOnce),
        ..Probe::default()
    };
    let mut battle = scenario(&fixture, &probe);
    start(&mut battle);
    let events = cast(&mut battle, 1, Some(3));
    assert_eq!(damage(&events).len(), 3);
    assert_eq!(hp(&battle), [10_000, 100, 80, 2750, 9000]);
    assert_eq!(battle.view().rng_draw_count(), 0);
    assert_eq!(marks(&battle), 0);
}
