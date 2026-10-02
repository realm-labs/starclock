//! Both-family accepted-command proof, with original-owner eligibility intact.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::ready,
        curio_battle_stats::assemble,
        weighted_curio_encouragement_fixture::{
            BASIC, COUNTDOWN, EFFECT, FOLLOW_UP, KILL_ENEMY, LINKED_ATTACK, Probe, REFRESH, SPAWN,
            accept, cast, idle, scenario,
        },
    },
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_combat::{
    ActionEventData, Battle, BattleEvent, BattleEventKind, BattleSeed, CauseActor, Command,
    DamageEventData, DecisionId, LifeState, LinkedEntityKind, TeamSide, UnitId,
    catalog::action::{AbilityTag, HitCritPolicy},
    formula::model::DamageClass,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_replay::battle_event::encode_battle_event_payload;
use std::sync::Arc;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
const PARTY: [u32; 4] = [1502, 1005, 1009, 1105];
fn equipped(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> DivergentUniverseAssembledBattle {
    let (flow, mut activity) = ready(fixture, family);
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    let id = fixture
        .factory()
        .decision_catalog()
        .weighted_curio_encouragements()[0]
        .weighted_curio
        .clone();
    let hash = activity.state_hash();
    runtime
        .replace_accepted_loadout(
            &flow,
            &mut activity,
            hash,
            WeightedCurioSlotLimit::new(1).unwrap(),
            &[id],
        )
        .unwrap();
    let before = activity.canonical_state_bytes();
    let result = assemble(fixture, &flow, &activity);
    assert_eq!(activity.canonical_state_bytes(), before);
    result
}
fn damages(events: &[BattleEvent]) -> Vec<DamageEventData> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(*data),
            _ => None,
        })
        .collect()
}
fn amounts(events: &[BattleEvent]) -> Vec<i64> {
    damages(events)
        .iter()
        .map(|data| data.calculated.get())
        .collect()
}
fn labels(events: &[BattleEvent]) -> Vec<u8> {
    damages(events)
        .iter()
        .map(|data| data.semantics.bits())
        .collect()
}
fn target(battle: &Battle) -> UnitId {
    battle
        .view()
        .units_by_id()
        .find(|unit| {
            unit.side() == TeamSide::Enemy
                && unit.life() == LifeState::Alive
                && unit.presence().is_active()
        })
        .unwrap()
        .id()
}
fn owner(battle: &Battle) -> UnitId {
    battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Player && unit.formation().get() == 0)
        .unwrap()
        .id()
}
fn effect_ids(battle: &Battle) -> Vec<u64> {
    battle
        .view()
        .effects_by_id()
        .filter(|effect| effect.definition().get() == EFFECT)
        .map(|effect| effect.id().get())
        .collect()
}

#[test]
fn weighted_curio_encouragement_mixed_calculators_and_native_follow_up_do_not_retag_actions() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for crit in [HitCritPolicy::Never, HitCritPolicy::Shared] {
            let mut battle = scenario(
                &source,
                Probe {
                    crit,
                    ..Probe::default()
                },
            );
            assert_eq!(effect_ids(&battle).len(), 1);
            let enemy = target(&battle);
            let events = cast(&mut battle, 0, BASIC, Some(enemy));
            assert_eq!(
                amounts(&events),
                if crit == HitCritPolicy::Shared {
                    [150, 300, 150]
                } else {
                    [100, 100, 100]
                }
            );
            assert_eq!(labels(&events), [0, 1, 0]);
            assert_eq!(
                damages(&events)
                    .iter()
                    .map(|data| data.class)
                    .collect::<Vec<_>>(),
                [
                    DamageClass::Direct,
                    DamageClass::Elation,
                    DamageClass::Additional
                ]
            );
            assert!(events.iter().any(|event| matches!(event.kind(), BattleEventKind::Action(ActionEventData::Started { tags, .. })
                if tags.contains(AbilityTag::Basic) && !tags.contains(AbilityTag::FollowUp))));
            let events = cast(&mut battle, 0, FOLLOW_UP, Some(enemy));
            assert_eq!(
                amounts(&events),
                if crit == HitCritPolicy::Shared {
                    [300, 300, 150]
                } else {
                    [100, 100, 100]
                }
            );
            assert_eq!(labels(&events), [1, 1, 0]);
            assert_eq!(battle.view().rng_draw_count(), 0);
            let events = cast(&mut battle, 1, FOLLOW_UP, Some(enemy));
            assert_eq!(
                amounts(&events),
                if crit == HitCritPolicy::Shared {
                    [150, 150, 150]
                } else {
                    [100, 100, 100]
                }
            );
            assert_eq!(
                labels(&events),
                [1, 0, 0],
                "an ineligible original retains native labels only"
            );
        }
    }
}

#[test]
fn weighted_curio_encouragement_shared_crit_reuses_decision_not_scoped_critical_damage() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for crit in [
            HitCritPolicy::Shared,
            HitCritPolicy::PerTarget,
            HitCritPolicy::Never,
        ] {
            let mut battle = scenario(
                &source,
                Probe {
                    crit,
                    rate: 500_000,
                    all: true,
                    hits: 3,
                    ..Probe::default()
                },
            );
            let events = cast(&mut battle, 0, BASIC, None);
            let data = damages(&events);
            assert_eq!(data.len(), 18);
            for hit in 0..3 {
                for enemy in [data[0].target, data[1].target] {
                    let group = data[hit * 6..hit * 6 + 6]
                        .iter()
                        .filter(|d| d.target == enemy)
                        .collect::<Vec<_>>();
                    let critical = group[0].calculated.get() == 150;
                    assert_eq!(
                        group.iter().map(|d| d.calculated.get()).collect::<Vec<_>>(),
                        if critical {
                            [150, 300, 150]
                        } else {
                            [100, 100, 100]
                        }
                    );
                    assert_eq!(
                        group.iter().map(|d| d.semantics.bits()).collect::<Vec<_>>(),
                        [0, 1, 0]
                    );
                }
            }
            assert_eq!(
                battle.view().rng_draw_count(),
                match crit {
                    HitCritPolicy::Shared => 3,
                    HitCritPolicy::PerTarget => 6,
                    _ => 0,
                }
            );
        }
    }
}

#[test]
fn weighted_curio_encouragement_real_linked_units_with_inherited_bundles_and_forced_effect_are_excluded()
 {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for kind in [
            LinkedEntityKind::Summon,
            LinkedEntityKind::Memosprite,
            LinkedEntityKind::SharedActor,
        ] {
            let mut battle = scenario(
                &source,
                Probe {
                    linked: Some(kind),
                    ..Probe::default()
                },
            );
            let original = owner(&battle);
            let mut events = cast(&mut battle, 0, SPAWN, None);
            for _ in 0..128 {
                if events.iter().any(|event| {
                    matches!(event.kind(), BattleEventKind::Damage(_))
                        && event
                            .cause()
                            .source_definition()
                            .is_some_and(|id| id.get() == LINKED_ATTACK)
                }) {
                    break;
                }
                events.extend(idle(&mut battle));
            }
            let linked = events
                .iter()
                .filter(|event| {
                    event
                        .cause()
                        .source_definition()
                        .is_some_and(|id| id.get() == LINKED_ATTACK)
                })
                .cloned()
                .collect::<Vec<_>>();
            assert_eq!(amounts(&linked), [150, 150, 150], "{kind:?}");
            assert_eq!(labels(&linked), [1, 0, 0]);
            assert!(
                battle.view().effects_by_id().any(
                    |effect| effect.definition().get() == EFFECT && effect.target() != original
                )
            );
            let enemy = target(&battle);
            assert_eq!(
                amounts(&cast(&mut battle, 0, BASIC, Some(enemy))),
                [150, 300, 150]
            );
        }
    }
}

#[test]
fn weighted_curio_encouragement_unitless_timeline_actor_cannot_borrow_owner_critical_bonus_or_alias()
 {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let mut battle = scenario(
            &source,
            Probe {
                unitless: true,
                ..Probe::default()
            },
        );
        let mut events = cast(&mut battle, 0, SPAWN, None);
        for _ in 0..128 {
            if events.iter().any(|event| {
                matches!(event.kind(), BattleEventKind::Damage(_))
                    && event
                        .cause()
                        .source_definition()
                        .is_some_and(|id| id.get() == COUNTDOWN)
            }) {
                break;
            }
            events.extend(idle(&mut battle));
        }
        let actor_events = events
            .iter()
            .filter(|event| {
                event
                    .cause()
                    .source_definition()
                    .is_some_and(|id| id.get() == COUNTDOWN)
            })
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(amounts(&actor_events), [150, 150, 150]);
        assert_eq!(labels(&actor_events), [1, 0, 0]);
        assert!(
            actor_events
                .iter()
                .filter(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
                .all(|event| matches!(event.cause().actor(), Some(CauseActor::TimelineActor(_))))
        );
        assert_eq!(effect_ids(&battle).len(), 1);
    }
}

#[test]
fn weighted_curio_encouragement_refresh_wave_and_fresh_battle_keep_one_owner_contribution() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let mut battle = scenario(&source, Probe::default());
        cast(&mut battle, 0, REFRESH, None);
        cast(&mut battle, 0, REFRESH, None);
        assert_eq!(effect_ids(&battle).len(), 1);
        let ids = effect_ids(&battle);
        for _ in 0..2 {
            let enemy = target(&battle);
            cast(&mut battle, 0, KILL_ENEMY, Some(enemy));
        }
        for _ in 0..16 {
            if battle.view().encounter().number() == 2 {
                break;
            }
            idle(&mut battle);
        }
        assert_eq!(battle.view().encounter().number(), 2);
        assert_eq!(effect_ids(&battle), ids);
        let enemy = target(&battle);
        assert_eq!(
            amounts(&cast(&mut battle, 0, BASIC, Some(enemy))),
            [150, 300, 150]
        );
        assert_eq!(effect_ids(&scenario(&source, Probe::default())).len(), 1);
    }
}

#[test]
fn weighted_curio_encouragement_unequip_and_empty_eligible_roster_have_no_effect() {
    for party in [PARTY, [1001, 1005, 1009, 1105]] {
        let fixture = DivergentUniverseBaselineFixture::production_for_source_party(party).unwrap();
        for family in FAMILIES {
            let (flow, mut activity) = ready(&fixture, family);
            let runtime = fixture.factory().weighted_curio_runtime().unwrap();
            let id = fixture
                .factory()
                .decision_catalog()
                .weighted_curio_encouragements()[0]
                .weighted_curio
                .clone();
            let hash = activity.state_hash();
            runtime
                .replace_accepted_loadout(
                    &flow,
                    &mut activity,
                    hash,
                    WeightedCurioSlotLimit::new(1).unwrap(),
                    &[id],
                )
                .unwrap();
            if party == PARTY {
                let hash = activity.state_hash();
                runtime
                    .replace_accepted_loadout(
                        &flow,
                        &mut activity,
                        hash,
                        WeightedCurioSlotLimit::new(1).unwrap(),
                        &[],
                    )
                    .unwrap();
            }
            let before = activity.canonical_state_bytes();
            let source = assemble(&fixture, &flow, &activity);
            assert_eq!(activity.canonical_state_bytes(), before);
            let mut battle = scenario(&source, Probe::default());
            assert!(effect_ids(&battle).is_empty());
            let enemy = target(&battle);
            let events = cast(&mut battle, 0, BASIC, Some(enemy));
            assert_eq!(amounts(&events), [150, 150, 150]);
            assert_eq!(labels(&events), [0, 0, 0]);
        }
    }
}

#[test]
fn weighted_curio_encouragement_fresh_reconstruction_and_stale_rejection_preserve_events_state_rng()
{
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    let fresh = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let first_source = equipped(&fixture, family);
        let second_source = equipped(&fresh, family);
        assert_eq!(
            first_source.assembly_digest(),
            second_source.assembly_digest()
        );
        let probe = Probe {
            crit: HitCritPolicy::Shared,
            rate: 500_000,
            ..Probe::default()
        };
        let mut first = scenario(&first_source, probe);
        let mut second = scenario(&second_source, probe);
        for raw in [BASIC, FOLLOW_UP, BASIC] {
            let enemy = target(&first);
            let first_events = cast(&mut first, 0, raw, Some(enemy));
            let second_enemy = target(&second);
            let second_events = cast(&mut second, 0, raw, Some(second_enemy));
            let bytes = |events: &[BattleEvent]| {
                events
                    .iter()
                    .map(|event| encode_battle_event_payload(event).unwrap())
                    .collect::<Vec<_>>()
            };
            assert_eq!(bytes(&first_events), bytes(&second_events));
            assert_eq!(first.state_hash(), second.state_hash());
            assert_eq!(
                first.view().rng_draw_count(),
                second.view().rng_draw_count()
            );
        }
        let hash = first.state_hash();
        let draws = first.view().rng_draw_count();
        assert!(
            first
                .apply(Command::StartBattle {
                    decision: DecisionId::new(1).unwrap()
                })
                .is_err()
        );
        assert_eq!(first.state_hash(), hash);
        assert_eq!(first.view().rng_draw_count(), draws);
        // The unmodified production catalog must construct and execute too.
        let mut production = Battle::create(
            Arc::clone(first_source.combat_catalog()),
            first_source.battle_spec().clone(),
            BattleSeed::new([0xea; 32]),
        )
        .unwrap();
        let command = Command::StartBattle {
            decision: production.decision().unwrap().id(),
        };
        accept(&mut production, command);
        assert_eq!(effect_ids(&production).len(), 1);
        let command = production
            .decision()
            .unwrap()
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::UseAbility { .. }))
            .unwrap()
            .clone();
        accept(&mut production, command);
    }
}
