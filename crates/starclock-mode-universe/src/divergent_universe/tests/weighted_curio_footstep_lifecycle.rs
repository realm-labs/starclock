//! Genuine transforms and linked/timeline actions over production equipment.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::ready,
        curio_battle_stats::assemble,
        weighted_curio_footstep_equipment_fixture::{
            DAMAGE, HEAL, LOSS, cast, damage, probe_with_setup, progress, stacks,
        },
        weighted_curio_footstep_fixture::{concede, remainder},
        weighted_curio_footstep_lifecycle_fixture::{
            COUNTDOWN, LINKED_SKILL, RESTORE, SETUP, Setup,
        },
    },
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_combat::{
    ActionEventData, Battle, BattleEvent, BattleEventKind, CauseActor, Command, DecisionId,
    LinkedEntityKind, PresenceState, Scalar, TeamSide, UnitEventData, rule::model::RuleValue,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_replay::battle_event::encode_battle_event_payload;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
fn equipped(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> DivergentUniverseAssembledBattle {
    let (flow, mut activity) = ready(fixture, family);
    let definition = &fixture
        .factory()
        .decision_catalog()
        .weighted_curio_footsteps()[0];
    let hash = activity.state_hash();
    fixture
        .factory()
        .weighted_curio_runtime()
        .unwrap()
        .replace_accepted_loadout(
            &flow,
            &mut activity,
            hash,
            WeightedCurioSlotLimit::new(1).unwrap(),
            std::slice::from_ref(&definition.weighted_curio),
        )
        .unwrap();
    assemble(fixture, &flow, &activity)
}
fn finish_loss(battle: &mut Battle) {
    for _ in 0..64 {
        if battle.decision().is_some() {
            concede(battle);
            return;
        }
        progress(battle);
    }
    panic!("manual concede boundary not reached");
}
fn repeated(events: &[BattleEvent]) -> Vec<Vec<u8>> {
    events
        .iter()
        .map(|event| encode_battle_event_payload(event).unwrap())
        .collect()
}
fn stale_is_inert(battle: &mut Battle) {
    let hash = battle.state_hash();
    let draws = battle.view().rng_draw_count();
    assert!(
        battle
            .apply(Command::Concede {
                decision: DecisionId::new(999_999).unwrap()
            })
            .is_err()
    );
    assert_eq!(battle.state_hash(), hash);
    assert_eq!(battle.view().rng_draw_count(), draws);
}

fn representative_counter_charges(battle: &Battle, owner: u64) -> Option<RuleValue> {
    battle
        .view()
        .rule_instances_by_id()
        .filter(|rule| {
            rule.rule().get() == 24205 && rule.owner().is_some_and(|unit| unit.get() == owner)
        })
        .flat_map(|rule| rule.slots())
        .find(|(slot, _)| slot.get() == 24206)
        .map(|(_, value)| value.clone())
}

#[test]
fn weighted_curio_footstep_equipment_transform_preserves_unavailable_representative_counter_charges()
 {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1107, 1402, 1009, 1002])
            .unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let form = source
            .battle_spec()
            .participants()
            .iter()
            .find(|participant| {
                participant.side() == TeamSide::Player && participant.formation().get() == 3
            })
            .unwrap()
            .combatant()
            .form();
        let run = || {
            let mut battle = probe_with_setup(
                &source,
                Some((
                    0,
                    Setup::Transform {
                        form,
                        countdown: false,
                    },
                )),
            );
            // Transform before any other probe action can exhaust the broad
            // representative HitEnded rule. Do not hide the missing-owner case
            // by observing only an already empty charge slot.
            assert_eq!(
                representative_counter_charges(&battle, 1),
                Some(RuleValue::Integer(2))
            );
            let mut events = cast(&mut battle, 1, SETUP);
            assert_eq!(
                representative_counter_charges(&battle, 1),
                Some(RuleValue::Integer(2))
            );
            let hit = cast(&mut battle, 1, DAMAGE);
            assert_eq!(damage(&hit), [108]);
            events.extend(hit);
            assert_eq!(
                representative_counter_charges(&battle, 1),
                Some(RuleValue::Integer(2))
            );
            assert!(!events.iter().any(|event| matches!(event.kind(),
                BattleEventKind::RuleState(data) if data.slot.get() == 24206)));
            assert!(!events.iter().any(|event| matches!(event.kind(),
                BattleEventKind::Action(ActionEventData::Cancelled { ability, .. }) if ability.get() == 24201)));
            let restore = cast(&mut battle, 1, RESTORE);
            assert_eq!(
                representative_counter_charges(&battle, 1),
                Some(RuleValue::Integer(1))
            );
            // Binding/executing the internal Counter remains a separate gap.
            assert_eq!(restore.iter().filter(|event| matches!(event.kind(),
                BattleEventKind::Action(ActionEventData::Cancelled { ability, .. }) if ability.get() == 24201)).count(), 1);
            events.extend(restore);
            assert_eq!(stacks(&battle, 1), 2);
            assert_eq!(battle.view().rng_draw_count(), 0);
            stale_is_inert(&mut battle);
            let payload = repeated(&events);
            finish_loss(&mut battle);
            assert_eq!(stacks(&battle, 1), 0);
            assert_eq!(remainder(&battle, 1), Scalar::ZERO);
            (events, payload, battle.state_hash())
        };
        assert_eq!(run(), run());
    }
}

#[test]
fn weighted_curio_footstep_equipment_real_transform_anchors_entry_path_and_retains_both_clauses() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1107, 1402, 1009, 1002])
            .unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for formation in [0, 1, 3] {
            let original = source
                .battle_spec()
                .participants()
                .iter()
                .find(|p| p.side() == TeamSide::Player && p.formation().get() == formation)
                .unwrap()
                .combatant()
                .form();
            // Eligible originals become Hunt; the Hunt original becomes Destruction.
            let replacement = source
                .battle_spec()
                .participants()
                .iter()
                .find(|p| {
                    p.side() == TeamSide::Player
                        && p.formation().get() == if formation < 2 { 3 } else { 0 }
                })
                .unwrap()
                .combatant()
                .form();
            let owner = u64::from(formation) + 1;
            let run = || {
                let mut battle = probe_with_setup(
                    &source,
                    Some((
                        formation,
                        Setup::Transform {
                            form: replacement,
                            countdown: false,
                        },
                    )),
                );
                let eligible = formation < 2;
                let mut events = cast(&mut battle, owner, LOSS);
                assert_eq!(
                    remainder(&battle, owner),
                    if eligible {
                        Scalar::checked_from_integer(30).unwrap()
                    } else {
                        Scalar::ZERO
                    }
                );
                events.extend(cast(&mut battle, owner, HEAL));
                let transform = cast(&mut battle, owner, SETUP);
                assert!(transform.iter().any(|event| matches!(event.kind(), BattleEventKind::Unit(UnitEventData::Transformed {unit, from, to, ..})
                    if unit.get() == owner && *from == original && *to == replacement)));
                events.extend(transform);
                assert_eq!(
                    battle
                        .view()
                        .units_by_id()
                        .find(|unit| unit.id().get() == owner)
                        .unwrap()
                        .presence(),
                    PresenceState::Transformed
                );
                assert_eq!(stacks(&battle, owner), if eligible { 3 } else { 0 });
                let hit = cast(&mut battle, owner, DAMAGE);
                assert_eq!(damage(&hit), [if eligible { 124 } else { 100 }]);
                events.extend(hit);
                events.extend(cast(&mut battle, owner, LOSS));
                assert_eq!(
                    battle.view().team(TeamSide::Player).skill_points(),
                    u16::from(eligible)
                );
                assert_eq!(
                    remainder(&battle, owner),
                    if eligible {
                        Scalar::checked_from_integer(10).unwrap()
                    } else {
                        Scalar::ZERO
                    }
                );
                let restore = cast(&mut battle, owner, RESTORE);
                assert!(restore.iter().any(|event| matches!(event.kind(), BattleEventKind::Unit(UnitEventData::TransformationEnded {unit, restored_form})
                    if unit.get() == owner && *restored_form == original)));
                events.extend(restore);
                assert_eq!(stacks(&battle, owner), if eligible { 5 } else { 0 });
                let hit = cast(&mut battle, owner, DAMAGE);
                assert_eq!(damage(&hit), [if eligible { 140 } else { 100 }]);
                events.extend(hit);
                assert_eq!(battle.view().rng_draw_count(), 0);
                stale_is_inert(&mut battle);
                let payload = repeated(&events);
                finish_loss(&mut battle);
                assert_eq!(stacks(&battle, owner), 0);
                assert_eq!(remainder(&battle, owner), Scalar::ZERO);
                (events, payload, battle.state_hash())
            };
            assert_eq!(run(), run());
        }
    }
}

#[test]
fn weighted_curio_footstep_equipment_real_inherited_linked_units_cannot_borrow_loss_or_skill_bonus()
{
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1107, 1402, 1009, 1002])
            .unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for kind in [
            LinkedEntityKind::Summon,
            LinkedEntityKind::Memosprite,
            LinkedEntityKind::SharedActor,
        ] {
            for formation in [0, 4] {
                for force_effect in [false, true] {
                    let run = || {
                        let mut battle = probe_with_setup(
                            &source,
                            Some((
                                0,
                                Setup::Linked {
                                    kind,
                                    formation,
                                    force_effect,
                                },
                            )),
                        );
                        cast(&mut battle, 1, LOSS);
                        let mut events = cast(&mut battle, 1, SETUP);
                        let linked = events
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
                        for _ in 0..64 {
                            if events.iter().any(|event| {
                                matches!(event.kind(), BattleEventKind::Damage(_))
                                    && event
                                        .cause()
                                        .source_definition()
                                        .is_some_and(|source| source.get() == LINKED_SKILL)
                            }) {
                                break;
                            }
                            events.extend(progress(&mut battle));
                        }
                        let packets = events
                            .iter()
                            .filter(|event| {
                                matches!(event.kind(), BattleEventKind::Damage(_))
                                    && event
                                        .cause()
                                        .source_definition()
                                        .is_some_and(|source| source.get() == LINKED_SKILL)
                            })
                            .collect::<Vec<_>>();
                        assert_eq!(packets.len(), 1);
                        assert_eq!(packets[0].cause().actor(), Some(CauseActor::Unit(linked)));
                        assert!(
                            matches!(packets[0].kind(), BattleEventKind::Damage(data) if data.applied.get() == 100)
                        );
                        assert_eq!(
                            battle
                                .view()
                                .units_by_id()
                                .find(|unit| unit.id() == linked)
                                .unwrap()
                                .current_hp()
                                .get(),
                            70
                        );
                        assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 0);
                        assert_eq!(
                            remainder(&battle, 1),
                            Scalar::checked_from_integer(30).unwrap()
                        );
                        assert_eq!(remainder(&battle, linked.get()), Scalar::ZERO);
                        assert_eq!(stacks(&battle, 1), 2);
                        assert_eq!(
                            stacks(&battle, linked.get()),
                            if force_effect { 10 } else { 0 }
                        );
                        assert_eq!(battle.view().rng_draw_count(), 0);
                        stale_is_inert(&mut battle);
                        let payload = repeated(&events);
                        finish_loss(&mut battle);
                        assert_eq!(stacks(&battle, 1), 0);
                        assert_eq!(stacks(&battle, linked.get()), 0);
                        assert_eq!(remainder(&battle, 1), Scalar::ZERO);
                        (events, payload, battle.state_hash())
                    };
                    assert_eq!(
                        run(),
                        run(),
                        "{family:?}/{kind:?}/{formation}/{force_effect}"
                    );
                }
            }
        }
    }
}

#[test]
fn weighted_curio_footstep_equipment_unitless_countdown_cannot_borrow_transformed_original_bonus() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1107, 1402, 1009, 1002])
            .unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let form = source
            .battle_spec()
            .participants()
            .iter()
            .find(|p| p.side() == TeamSide::Player && p.formation().get() == 3)
            .unwrap()
            .combatant()
            .form();
        let run = || {
            let mut battle = probe_with_setup(
                &source,
                Some((
                    0,
                    Setup::Transform {
                        form,
                        countdown: true,
                    },
                )),
            );
            cast(&mut battle, 1, LOSS);
            let mut events = cast(&mut battle, 1, SETUP);
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
            for _ in 0..64 {
                if events.iter().any(|event| {
                    matches!(event.kind(), BattleEventKind::Damage(_))
                        && event.cause().actor() == Some(CauseActor::TimelineActor(actor))
                }) {
                    break;
                }
                events.extend(progress(&mut battle));
            }
            let packets = events
                .iter()
                .filter(|event| {
                    matches!(event.kind(), BattleEventKind::Damage(_))
                        && event
                            .cause()
                            .source_definition()
                            .is_some_and(|source| source.get() == COUNTDOWN)
                })
                .collect::<Vec<_>>();
            assert_eq!(packets.len(), 1);
            assert_eq!(
                packets[0].cause().actor(),
                Some(CauseActor::TimelineActor(actor))
            );
            assert!(
                matches!(packets[0].kind(), BattleEventKind::Damage(data) if data.applied.get() == 100)
            );
            assert_eq!(stacks(&battle, 1), 2);
            assert_eq!(
                remainder(&battle, 1),
                Scalar::checked_from_integer(30).unwrap()
            );
            assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 0);
            assert_eq!(battle.view().rng_draw_count(), 0);
            stale_is_inert(&mut battle);
            let payload = repeated(&events);
            finish_loss(&mut battle);
            assert_eq!(stacks(&battle, 1), 0);
            assert_eq!(remainder(&battle, 1), Scalar::ZERO);
            (events, payload, battle.state_hash())
        };
        assert_eq!(run(), run());
    }
}
