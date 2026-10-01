//! Charge admission is safe; the unbound Counter action remains a recorded gap.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::ready,
        curio_battle_stats::assemble,
        weighted_curio_retaliation::{Attack, equipped, first_attack, probe, retaliations},
    },
};
use starclock_combat::{
    ActionEventData, BattleEventKind, Command, Resolution, rule::model::RuleValue,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;

#[test]
fn production_clara_counter_exhaustion_does_not_fault_with_weighted_curio_retaliation() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1107, 1105, 1001, 1002])
            .unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        for with_curio in [false, true] {
            let assembled = if with_curio {
                equipped(&fixture, family)
            } else {
                let (flow, activity) = ready(&fixture, family);
                assemble(&fixture, &flow, &activity)
            };
            for hits in [3, 9, 1] {
                let attack = Attack::with_hits(hits);
                let mut battle = probe(&assembled, attack, 1);
                let mut fresh = probe(&assembled, attack, 1);
                let result = first_attack(&mut battle);
                let reconstructed = first_attack(&mut fresh);
                assert_eq!(result.events(), reconstructed.events());
                assert_eq!(battle.state_hash(), fresh.state_hash());
                assert_eq!(
                    battle.view().rng_draw_count(),
                    fresh.view().rng_draw_count()
                );
                let counters = result.events().iter().filter(|event| matches!(
                    event.kind(), BattleEventKind::Action(ActionEventData::Queued { ability, .. })
                    if ability.get() == 24201
                )).count();
                assert_eq!(
                    counters,
                    usize::from(hits.min(2)),
                    "admission cannot exceed the initial two charges"
                );
                assert_eq!(
                    retaliations(result.events()).len(),
                    if with_curio { 2 } else { 0 }
                );
                let mut drained = Vec::new();
                let mut next_enemy_attack = false;
                for _ in 0..64 {
                    let command = battle
                        .decision()
                        .and_then(|decision| {
                            decision
                                .legal_commands()
                                .iter()
                                .find(|command| !matches!(command, Command::Concede { .. }))
                        })
                        .cloned()
                        .unwrap_or_else(|| battle.advance_command().unwrap());
                    let resolution = battle.apply(command.clone()).unwrap();
                    let rebuilt = fresh.apply(command).unwrap();
                    assert!(resolution.fault().is_none());
                    assert_eq!(resolution.events(), rebuilt.events());
                    assert_eq!(battle.state_hash(), fresh.state_hash());
                    assert_eq!(
                        battle.view().rng_draw_count(),
                        fresh.view().rng_draw_count()
                    );
                    next_enemy_attack = resolution.events().iter().any(|event| matches!(
                        event.kind(), BattleEventKind::Action(ActionEventData::Declared { ability, .. })
                        if ability.get() == 0x7d33_0001
                    ));
                    drained.push(resolution);
                    if next_enemy_attack {
                        break;
                    }
                }
                assert!(next_enemy_attack);
                let drained_events = drained
                    .iter()
                    .flat_map(Resolution::events)
                    .collect::<Vec<_>>();
                let all_events = result
                    .events()
                    .iter()
                    .chain(drained_events.iter().copied())
                    .collect::<Vec<_>>();
                // The internal Counter is not in the production character's
                // ability bindings. Assert this gap, never credit cancellation
                // as execution of Clara's released Counter.
                assert_eq!(all_events.iter().filter(|event| matches!(
                    event.kind(), BattleEventKind::Action(ActionEventData::Declared { ability, .. })
                    if ability.get() == 24201
                )).count(), 0);
                assert_eq!(all_events.iter().filter(|event| matches!(
                    event.kind(), BattleEventKind::Action(ActionEventData::Cancelled { ability, .. })
                    if ability.get() == 24201
                )).count(), 2, "hits {hits}, curio {with_curio}");
                let changes = result
                    .events()
                    .iter()
                    .chain(drained_events)
                    .filter_map(|event| match event.kind() {
                        BattleEventKind::RuleState(data) if data.slot.get() == 24206 => {
                            Some((&data.before, &data.after))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    changes,
                    vec![
                        (&RuleValue::Integer(2), &RuleValue::Integer(1)),
                        (&RuleValue::Integer(1), &RuleValue::Integer(0))
                    ]
                );
            }
        }
    }
}
