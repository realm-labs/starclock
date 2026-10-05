//! Original roster identity survives transformation; inherited actors never admit it.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    tests::{
        weighted_curio_overflow_fixture::{Probe, accept, cast, cast_ability, scenario, start},
        weighted_curio_overflow_lifecycle_fixture::Setup,
    },
};
use starclock_combat::{
    Battle, BattleEvent, BattleEventData, BattleEventKind, CauseActor, LinkedEntityKind,
    UnitEventData, catalog::action::TargetPattern,
};
use starclock_replay::battle_event::encode_battle_event_payload;

fn damages(events: &[BattleEvent]) -> Vec<i64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(data.calculated.get()),
            _ => None,
        })
        .collect()
}

fn clean(battle: &Battle) {
    assert!(battle.view().effects_by_id().all(|effect| {
        effect.definition().get() < 0x7f05_0000 || effect.definition().get() >= 0x7f06_0000
    }));
}

#[test]
fn weighted_curio_overflow_original_transform_preserves_admission_and_replays_exactly() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let probe = Probe {
        setup: Some(Setup::Transform),
        hp: vec![100, 80, 4_000, 100],
        ..Probe::default()
    };
    let mut a = scenario(&fixture, &probe);
    let mut b = scenario(&fixture, &probe);
    start(&mut a);
    start(&mut b);
    for battle in [&mut a, &mut b] {
        let events = cast_ability(battle, 1, 3, None);
        assert!(events.iter().any(|event| matches!(event.kind(),
            BattleEventKind::Unit(UnitEventData::Transformed { unit, from, to, .. })
                if unit.get() == 1 && from.get() == 1 && to.get() == 3)));
        assert_eq!(battle.view().rng_draw_count(), 0);
        clean(battle);
    }
    let x = cast(&mut a, 1, Some(3));
    let y = cast(&mut b, 1, Some(3));
    assert_eq!(damages(&x), [250, 250, 250, 2220]);
    assert_eq!(x, y);
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(
        x.iter()
            .map(|e| encode_battle_event_payload(e).unwrap())
            .collect::<Vec<_>>(),
        y.iter()
            .map(|e| encode_battle_event_payload(e).unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(a.view().rng_draw_count(), 1);
    clean(&a);
    let restored = cast_ability(&mut a, 1, 4, None);
    assert!(restored.iter().any(|event| matches!(event.kind(),
        BattleEventKind::Unit(UnitEventData::TransformationEnded { unit, restored_form })
            if unit.get() == 1 && restored_form.get() == 1)));
    assert_eq!(a.view().rng_draw_count(), 1);
    clean(&a);
    let after_restore = cast(&mut a, 1, Some(4));
    assert_eq!(damages(&after_restore), [250, 250, 2050]);
    assert!(
        after_restore
            .iter()
            .any(|event| matches!(event.kind(), BattleEventKind::Battle(BattleEventData::Won)))
    );
    assert_eq!(a.view().rng_draw_count(), 2);
    clean(&a);
}

#[test]
fn weighted_curio_overflow_real_inherited_summon_memosprite_and_shared_actor_do_not_collect() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for (kind, formation) in [
        LinkedEntityKind::Summon,
        LinkedEntityKind::Memosprite,
        LinkedEntityKind::SharedActor,
    ]
    .into_iter()
    .flat_map(|kind| [0, 4].map(|formation| (kind, formation)))
    {
        let probe = Probe {
            setup: Some(Setup::Linked(kind, formation)),
            pattern: TargetPattern::All,
            ..Probe::default()
        };
        let mut battle = scenario(&fixture, &probe);
        start(&mut battle);
        let mut events = cast_ability(&mut battle, 1, 3, None);
        let unit = events
            .iter()
            .find_map(|event| match event.kind() {
                BattleEventKind::Unit(UnitEventData::Summoned {
                    unit,
                    owner,
                    kind: observed,
                    ..
                }) if owner.get() == 1 && *observed == kind => Some(*unit),
                _ => None,
            })
            .unwrap();
        for _ in 0..32 {
            if !damages(&events).is_empty() {
                break;
            }
            let command = battle.advance_command().unwrap();
            events.extend(accept(&mut battle, command));
        }
        let linked_damage = events
            .iter()
            .filter(|event| {
                matches!(event.kind(), BattleEventKind::Damage(_))
                    && event
                        .cause()
                        .source_definition()
                        .is_some_and(|source| source.get() == 6)
            })
            .collect::<Vec<_>>();
        assert_eq!(linked_damage.len(), 4, "linked kind {kind:?}");
        assert!(
            linked_damage
                .iter()
                .all(|event| event.cause().actor() == Some(CauseActor::Unit(unit)))
        );
        assert!(linked_damage.iter().all(|event| match event.kind() {
            BattleEventKind::Damage(data) => data.calculated.get() == 250,
            _ => false,
        }));
        assert_eq!(battle.view().rng_draw_count(), 0);
        clean(&battle);
    }
}

#[test]
fn weighted_curio_overflow_unitless_countdown_cannot_borrow_original_owner() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let probe = Probe {
        setup: Some(Setup::Countdown),
        pattern: TargetPattern::All,
        ..Probe::default()
    };
    let mut battle = scenario(&fixture, &probe);
    start(&mut battle);
    let mut events = cast_ability(&mut battle, 1, 3, None);
    let actor = events
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Unit(UnitEventData::Transformed {
                unit,
                countdown: Some(actor),
                ..
            }) if unit.get() == 1 => Some(*actor),
            _ => None,
        })
        .unwrap();
    for _ in 0..32 {
        if !damages(&events).is_empty() {
            break;
        }
        let command = battle.advance_command().unwrap();
        events.extend(accept(&mut battle, command));
    }
    assert_eq!(damages(&events), [250, 250, 250, 250]);
    assert!(
        events
            .iter()
            .filter(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
            .all(|event| event.cause().actor() == Some(CauseActor::TimelineActor(actor)))
    );
    assert_eq!(battle.view().rng_draw_count(), 0);
    clean(&battle);
}
