//! Real production commands plus controlled boundaries over Sora-lowered rules.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::ready,
        curio_battle_stats::assemble,
        weighted_curio_elation_fixture::{
            BASIC, CAP, DISPEL, EFFECT, KILL_ALLY, KILL_ENEMY, LINKED_ACTION, Probe, QUEUED, READ,
            SKILL, ULTIMATE, accept, balance, cast, id, idle, observed, scenario,
        },
    },
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_combat::{
    ActionEventData, Battle, BattleEvent, BattleEventKind, BattleSeed, CauseActor, Command,
    DecisionId, EffectEventData, LinkedEntityKind, PresenceState, ResourceEventData, TeamSide,
    UnitId,
    catalog::action::{AbilityKind, AbilityTag},
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
    let curio = fixture
        .factory()
        .decision_catalog()
        .weighted_curio_elations()[0]
        .weighted_curio
        .clone();
    let hash = activity.state_hash();
    runtime
        .replace_accepted_loadout(
            &flow,
            &mut activity,
            hash,
            WeightedCurioSlotLimit::new(1).unwrap(),
            &[curio],
        )
        .unwrap();
    let bytes = activity.canonical_state_bytes();
    let result = assemble(fixture, &flow, &activity);
    assert_eq!(bytes, activity.canonical_state_bytes());
    result
}
fn effects(battle: &Battle) -> Vec<(u8, Option<u16>, u64)> {
    battle
        .view()
        .effects_by_id()
        .filter(|effect| effect.definition().get() == EFFECT)
        .map(|effect| {
            (
                battle
                    .view()
                    .units_by_id()
                    .find(|unit| unit.id() == effect.target())
                    .unwrap()
                    .formation()
                    .get(),
                effect.remaining(),
                effect.applier().get(),
            )
        })
        .collect()
}
fn gains(events: &[BattleEvent]) -> Vec<(u16, u16)> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Resource(ResourceEventData::TeamResource {
                attempted,
                effective,
                ..
            }) if event
                .cause()
                .source_definition()
                .is_some_and(|id| id.get() == 0x7ec3_0001) =>
            {
                Some((*attempted, *effective))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn weighted_curio_elation_production_basic_and_nonattack_skill_execute_both_families() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1502, 1005, 1009, 1202])
            .unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for kind in [AbilityKind::Basic, AbilityKind::Skill] {
            let mut battle = Battle::create(
                Arc::clone(source.combat_catalog()),
                source.battle_spec().clone(),
                BattleSeed::new([0xe9; 32]),
            )
            .unwrap();
            let mut replay = Battle::create(
                Arc::clone(source.combat_catalog()),
                source.battle_spec().clone(),
                BattleSeed::new([0xe9; 32]),
            )
            .unwrap();
            let start = Command::StartBattle {
                decision: battle.decision().unwrap().id(),
            };
            assert_eq!(
                accept(&mut battle, start.clone()),
                accept(&mut replay, start)
            );
            let mut found = false;
            for _ in 0..256 {
                // Formation three is the released Harmony form Tingyun. Her
                // support Skill has no Attack tag; no test ability replaces it.
                let chosen = battle.decision().and_then(|decision| decision.legal_commands().iter().find(|command|
                    matches!(command, Command::UseAbility {actor, ability, ..} if actor.get() == 4
                        && source.combat_catalog().ability(*ability).and_then(|a| a.action()).is_some_and(|a| a.kind() == kind))).cloned());
                let selected = chosen.is_some();
                let command = chosen.unwrap_or_else(|| {
                    if let Some(decision) = battle.decision() {
                        decision
                            .legal_commands()
                            .iter()
                            .find(|c| {
                                matches!(
                                    c,
                                    Command::UseAbility { .. }
                                        | Command::CommitPreparedAction { .. }
                                        | Command::CommitActionFrame { .. }
                                )
                            })
                            .unwrap()
                            .clone()
                    } else {
                        Command::Advance {
                            boundary: battle.view().action_boundary().unwrap().id(),
                        }
                    }
                });
                if selected && kind == AbilityKind::Skill {
                    let Command::UseAbility { ability, .. } = command else {
                        panic!("selected production command must be a Skill");
                    };
                    assert!(
                        !source
                            .combat_catalog()
                            .ability(ability)
                            .unwrap()
                            .action()
                            .unwrap()
                            .tags()
                            .contains(AbilityTag::Attack)
                    );
                }
                let before = balance(&battle);
                let events = accept(&mut battle, command.clone());
                let reconstructed = accept(&mut replay, command);
                assert_eq!(
                    events
                        .iter()
                        .map(encode_battle_event_payload)
                        .collect::<Vec<_>>(),
                    reconstructed
                        .iter()
                        .map(encode_battle_event_payload)
                        .collect::<Vec<_>>()
                );
                assert_eq!(battle.state_hash(), replay.state_hash());
                assert_eq!(
                    battle.view().rng_draw_count(),
                    replay.view().rng_draw_count()
                );
                if selected {
                    assert_eq!(gains(&events), [(2, 2)]);
                    assert_eq!(balance(&battle), before + 2);
                    assert_eq!(effects(&battle), [(0, Some(2), 4)]);
                    found = true;
                    break;
                }
            }
            assert!(found, "real mapped action must be executed");
        }
    }
}

#[test]
fn weighted_curio_elation_multihit_refresh_stat_query_and_overflow_are_real_mutations() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1502, 1501, 1009, 1105])
            .unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let mut battle = scenario(&source, Probe::default());
        let enemy = battle
            .view()
            .units_by_id()
            .find(|unit| unit.side() == TeamSide::Enemy)
            .unwrap()
            .id();
        let events = cast(&mut battle, 2, BASIC, Some(enemy));
        assert_eq!(gains(&events), [(2, 2)]);
        assert_eq!(effects(&battle), [(0, Some(2), 3), (1, Some(2), 3)]);
        let events = cast(&mut battle, 3, SKILL, None);
        assert_eq!(gains(&events), [(2, 2)]);
        assert_eq!(effects(&battle), [(0, Some(2), 4), (1, Some(2), 4)]);
        cast(&mut battle, 0, READ, None);
        assert_eq!(
            observed(&battle),
            50,
            "live Elation query must execute the additive modifier"
        );
        assert!(
            effects(&battle)
                .iter()
                .all(|effect| effect.1 == Some(2) || effect.1 == Some(1))
        );
        cast(&mut battle, 0, CAP, None);
        assert_eq!(balance(&battle), 9999);
        let events = cast(&mut battle, 2, SKILL, None);
        assert_eq!(gains(&events), [(2, 0)]);
        assert!(events.iter().any(|event| matches!(event.kind(), BattleEventKind::Resource(ResourceEventData::TeamResource {overflow, ..}) if *overflow == 2)));
        assert_eq!(effects(&battle).len(), 2);
        assert_eq!(battle.view().rng_draw_count(), 0);
    }
}

#[test]
fn weighted_curio_elation_recipient_turn_expiry_survives_caster_defeat() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let mut battle = scenario(
            &source,
            Probe {
                sole_owner: true,
                ..Probe::default()
            },
        );
        cast(&mut battle, 1, SKILL, None);
        assert_eq!(effects(&battle), [(0, Some(2), 2)]);
        cast(&mut battle, 0, KILL_ALLY, Some(UnitId::new(2).unwrap()));
        assert_eq!(effects(&battle), [(0, Some(2), 2)]);
        let mut saw_one = false;
        for _ in 0..128 {
            idle(&mut battle);
            let active = effects(&battle);
            if active == [(0, Some(1), 2)] {
                saw_one = true;
            }
            if active.is_empty() {
                break;
            }
        }
        assert!(saw_one);
        assert!(effects(&battle).is_empty());
        cast(&mut battle, 0, READ, None);
        assert_eq!(observed(&battle), 0);
        assert_eq!(
            balance(&battle),
            2,
            "defeated provider does not retrigger or erase the meter"
        );
    }
}

#[test]
fn weighted_curio_elation_presence_and_excluded_owners_do_not_invent_recipients() {
    for party in [PARTY, [1308, 1005, 1009, 1105], [1502, 1501, 1009, 1105]] {
        let fixture = DivergentUniverseBaselineFixture::production_for_source_party(party).unwrap();
        for family in FAMILIES {
            let source = equipped(&fixture, family);
            let mut battle = scenario(&source, Probe::default());
            let events = cast(&mut battle, 0, SKILL, None);
            assert!(gains(&events).is_empty());
            let events = cast(&mut battle, 1, ULTIMATE, None);
            assert!(gains(&events).is_empty());
            if party[1] == 1501 {
                let mut no_owners = scenario(
                    &source,
                    Probe {
                        sole_owner: true,
                        ..Probe::default()
                    },
                );
                assert!(gains(&cast(&mut no_owners, 0, SKILL, None)).is_empty());
                assert!(effects(&no_owners).is_empty());
                assert_eq!(balance(&no_owners), 0);
            }
            if party == PARTY {
                for probe in [
                    Probe {
                        defeated: Some(0),
                        ..Probe::default()
                    },
                    Probe {
                        absent: Some(0),
                        ..Probe::default()
                    },
                ] {
                    let mut battle = scenario(&source, probe);
                    let events = cast(&mut battle, 1, SKILL, None);
                    assert_eq!(gains(&events), [(2, 2)]);
                    assert!(effects(&battle).is_empty());
                }
            }
        }
    }
}

#[test]
fn weighted_curio_elation_fresh_reconstruction_rejections_and_unequip_are_inert() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let fresh = equipped(
            &DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap(),
            family,
        );
        assert_eq!(source.battle_spec(), fresh.battle_spec());
        let mut first = scenario(&source, Probe::default());
        let mut second = scenario(&fresh, Probe::default());
        let events = cast(&mut first, 1, SKILL, None);
        assert_eq!(events, cast(&mut second, 1, SKILL, None));
        assert_eq!(first.state_hash(), second.state_hash());
        let hash = first.state_hash();
        let draws = first.view().rng_draw_count();
        assert!(
            first
                .apply(Command::UseAbility {
                    decision: DecisionId::new(u64::MAX).unwrap(),
                    actor: UnitId::new(2).unwrap(),
                    ability: id(SKILL),
                    primary_target: None
                })
                .is_err()
        );
        assert_eq!(first.state_hash(), hash);
        assert_eq!(first.view().rng_draw_count(), draws);
        assert_eq!(balance(&scenario(&source, Probe::default())), 0);
        let (flow, mut activity) = ready(&fixture, family);
        let runtime = fixture.factory().weighted_curio_runtime().unwrap();
        let curio = fixture
            .factory()
            .decision_catalog()
            .weighted_curio_elations()[0]
            .weighted_curio
            .clone();
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                &[curio],
            )
            .unwrap();
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
        let no_curio = assemble(&fixture, &flow, &activity);
        let mut plain = scenario(&no_curio, Probe::default());
        assert!(gains(&cast(&mut plain, 1, SKILL, None)).is_empty());
        assert!(effects(&plain).is_empty());
        assert_eq!(balance(&plain), 0);
        assert!(events.iter().any(|event| matches!(
            event.kind(),
            BattleEventKind::Effect(EffectEventData::Applied { .. })
        ) && event.cause().actor()
            == Some(CauseActor::Unit(UnitId::new(2).unwrap()))));
    }
}

#[test]
fn weighted_curio_elation_dispel_and_wave_transition_retain_recipient_scope_and_meter() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let mut battle = scenario(&source, Probe::default());
        cast(&mut battle, 1, SKILL, None);
        let before = balance(&battle);
        let enemy = battle
            .view()
            .units_by_id()
            .find(|unit| unit.side() == TeamSide::Enemy)
            .unwrap()
            .id();
        cast(&mut battle, 0, KILL_ENEMY, Some(enemy));
        assert_eq!(balance(&battle), before);
        assert_eq!(effects(&battle), [(0, Some(2), 2)]);
        assert_eq!(battle.view().encounter().number(), 2);
        let events = cast(&mut battle, 0, DISPEL, Some(UnitId::new(1).unwrap()));
        assert!(events.iter().any(|event| matches!(
            event.kind(),
            BattleEventKind::Effect(EffectEventData::Removed { .. })
        )));
        assert!(effects(&battle).is_empty());
    }
}

#[test]
fn weighted_curio_elation_queued_non_basic_skill_actions_never_trigger() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for kind in [
            AbilityKind::FollowUp,
            AbilityKind::Counter,
            AbilityKind::ExtraAction,
        ] {
            let mut battle = scenario(
                &source,
                Probe {
                    queued_kind: Some(kind),
                    ..Probe::default()
                },
            );
            let events = cast(&mut battle, 1, ULTIMATE, None);
            assert!(gains(&events).is_empty());
            let mut queued_resolved = events.iter().any(|event| {
                matches!(
                    event.kind(),
                    BattleEventKind::Action(ActionEventData::Resolved { .. })
                ) && event
                    .cause()
                    .source_definition()
                    .is_some_and(|id| id.get() == QUEUED)
            });
            for _ in 0..32 {
                if queued_resolved {
                    break;
                }
                let events = idle(&mut battle);
                if events.iter().any(|event| {
                    matches!(
                        event.kind(),
                        BattleEventKind::Action(ActionEventData::Resolved { .. })
                    ) && event
                        .cause()
                        .source_definition()
                        .is_some_and(|id| id.get() == QUEUED)
                }) {
                    assert!(gains(&events).is_empty());
                    queued_resolved = true;
                }
            }
            assert!(queued_resolved, "the excluded action must actually execute");
        }
    }
}

#[test]
fn weighted_curio_elation_actual_linked_and_shared_actors_cannot_inherit_original_owner_trigger() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for (kind, presence) in [
            LinkedEntityKind::Summon,
            LinkedEntityKind::Memosprite,
            LinkedEntityKind::SharedActor,
        ]
        .into_iter()
        .flat_map(|kind| {
            [PresenceState::Linked, PresenceState::Present].map(|presence| (kind, presence))
        }) {
            let mut battle = scenario(
                &source,
                Probe {
                    linked_kind: Some(kind),
                    linked_presence: Some(presence),
                    ..Probe::default()
                },
            );
            let events = cast(&mut battle, 1, ULTIMATE, None);
            assert!(gains(&events).is_empty());
            let resolved = |events: &[BattleEvent]| {
                events.iter().any(|event| {
                    matches!(
                        event.kind(),
                        BattleEventKind::Action(ActionEventData::Resolved { .. })
                    ) && event
                        .cause()
                        .source_definition()
                        .is_some_and(|id| id.get() == LINKED_ACTION)
                })
            };
            let mut executed = resolved(&events);
            for _ in 0..64 {
                if executed {
                    break;
                }
                let events = idle(&mut battle);
                if resolved(&events) {
                    assert!(gains(&events).is_empty());
                    executed = true;
                }
            }
            assert!(
                executed,
                "the excluded linked actor must actually act: {kind:?}"
            );
            let linked = battle
                .view()
                .units_by_id()
                .find(|unit| unit.formation().get() == 4 && unit.side() == TeamSide::Player)
                .unwrap()
                .id();
            assert!(
                !battle
                    .view()
                    .effects_by_id()
                    .any(|effect| effect.definition().get() == EFFECT && effect.target() == linked)
            );
        }
    }
}
