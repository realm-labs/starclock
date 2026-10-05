//! Real lifecycle commands retain normal equipment assembly's authored bindings.
use crate::divergent_universe::tests::{
    weighted_curio_burn_fixture::{
        CLEAR_WAVE, Probe, RESTORE, SEED, Scenario, attack, command, idle_step, until_source_tick,
    },
    weighted_curio_burn_lifecycle_fixture::{
        COUNTDOWN, END_TRANSFORM, LINKED_ATTACK, SETUP, Setup,
    },
    weighted_curio_deflagration_native::{
        EFFECT, FAMILIES, applications, fixture, parts, payloads, scenario, source,
    },
};
use starclock_combat::{
    BattleEvent, BattleEventKind, BattlePhase, CauseActor, Command, DamageKind, DecisionId,
    LinkedEntityKind, PresenceState, TeamSide, UnitEventData, UnitId, rule::model::RuleValue,
};

fn cast_self(test: &mut Scenario, raw: u32) -> Vec<BattleEvent> {
    let offered = command(test, test.actor, raw.try_into().unwrap(), Some(test.actor));
    accept(test, offered)
}
fn accept(test: &mut Scenario, command: Command) -> Vec<BattleEvent> {
    let outcome = test.battle.apply(command).unwrap();
    assert!(outcome.fault().is_none(), "{:?}", outcome.fault());
    outcome.events().to_vec()
}
fn cast_all_seeder(test: &mut Scenario, raw: u32) -> Vec<BattleEvent> {
    let seeder = test.seeder;
    let offered = command(test, seeder, raw.try_into().unwrap(), None);
    accept(test, offered)
}
fn entry_count(test: &Scenario, owner: UnitId) -> i64 {
    test.battle
        .view()
        .rule_instances_by_id()
        .filter(|rule| rule.owner() == Some(owner))
        .flat_map(|rule| rule.slots())
        .find_map(|(id, value)| {
            (id.get() == 0x7f76_0020).then(|| match value {
                RuleValue::Integer(count) => *count,
                other => panic!("invalid count: {other:?}"),
            })
        })
        .unwrap()
}
fn invariance(test: &mut Scenario) {
    let before = test.battle.state_hash();
    let decision = test.battle.decision().cloned();
    let draws = test.battle.view().rng_draw_count();
    assert!(
        test.battle
            .apply(Command::Concede {
                decision: DecisionId::new(999_999).unwrap(),
            })
            .is_err()
    );
    assert_eq!(test.battle.state_hash(), before);
    assert_eq!(test.battle.decision(), decision.as_ref());
    assert_eq!(test.battle.view().rng_draw_count(), draws);
    assert_eq!(draws, 0);
}

#[test]
fn deflagration_lifecycle_real_transform_and_restore_retain_original_fire_membership() {
    let fire_fixture = fixture(1);
    let other_fixture = fixture(0);
    for family in FAMILIES {
        let fire = parts(&fire_fixture, family);
        let other = parts(&other_fixture, family);
        for eligible in [false, true] {
            let input = if eligible { &fire } else { &other };
            let replacement = if eligible {
                fire.1[1].combatant().form()
            } else {
                fire.1[0].combatant().form()
            };
            let original = input.1[0].combatant().form();
            assert_ne!(original, replacement);
            let run = || {
                let mut test = scenario(
                    input,
                    Probe {
                        hits: 1,
                        burns: true,
                        lifecycle: Some(Setup::Transform {
                            form: replacement,
                            countdown: false,
                        }),
                        ..Probe::default()
                    },
                );
                let mut events = cast_self(&mut test, SETUP);
                assert!(events.iter().any(|event| matches!(event.kind(),
                    BattleEventKind::Unit(UnitEventData::Transformed { unit, from, to, .. })
                        if *unit == test.actor && *from == original && *to == replacement)));
                assert_eq!(applications(&events), 0);
                assert_eq!(
                    test.battle
                        .view()
                        .units_by_id()
                        .find(|unit| unit.id() == test.actor)
                        .unwrap()
                        .presence(),
                    PresenceState::Transformed
                );
                let hit = attack(&mut test);
                assert_eq!(applications(&hit), usize::from(eligible));
                events.extend(hit);
                if eligible {
                    let tick = until_source_tick(&mut test, source(0));
                    assert!(tick.iter().any(|event| matches!(event.kind(),
                        BattleEventKind::Damage(data) if data.kind == DamageKind::DotDetonation
                            && data.raw.scaled() == 50_000_000)));
                    assert_eq!(
                        test.battle
                            .view()
                            .units_by_id()
                            .find(|unit| unit.id() == test.actor)
                            .unwrap()
                            .presence(),
                        PresenceState::Transformed
                    );
                    events.extend(tick);
                }
                let restored = cast_self(&mut test, END_TRANSFORM);
                assert!(restored.iter().any(|event| matches!(event.kind(),
                    BattleEventKind::Unit(UnitEventData::TransformationEnded { unit, restored_form })
                        if *unit == test.actor && *restored_form == original)));
                assert_eq!(applications(&restored), 0);
                events.extend(restored);
                let hit = attack(&mut test);
                assert_eq!(applications(&hit), usize::from(eligible));
                events.extend(hit);
                if eligible {
                    assert_eq!(entry_count(&test, test.actor), 1);
                    assert!(
                        test.battle
                            .view()
                            .effects_by_id()
                            .any(|effect| effect.definition().get() == EFFECT
                                && effect.applier() == test.actor)
                    );
                } else {
                    assert!(
                        !test
                            .battle
                            .view()
                            .effects_by_id()
                            .any(|effect| effect.definition().get() == EFFECT)
                    );
                }
                invariance(&mut test);
                (payloads(&events), test.battle.state_hash())
            };
            assert_eq!(run(), run(), "{family:?}/{eligible}");
        }
    }
}

#[test]
fn deflagration_lifecycle_inherited_linked_attacks_do_not_borrow_original_qualification() {
    let fixture = fixture(4);
    for family in FAMILIES {
        let parts = parts(&fixture, family);
        for kind in [
            LinkedEntityKind::Summon,
            LinkedEntityKind::Memosprite,
            LinkedEntityKind::SharedActor,
        ] {
            for formation in [0, 4] {
                let run = || {
                    let mut test = scenario(
                        &parts,
                        Probe {
                            hits: 1,
                            absent_formation: Some(3),
                            lifecycle: Some(Setup::Linked { kind, formation }),
                            ..Probe::default()
                        },
                    );
                    assert_eq!(entry_count(&test, test.actor), 3);
                    let mut events = cast_self(&mut test, SETUP);
                    let linked = events
                        .iter()
                        .find_map(|event| match event.kind() {
                            BattleEventKind::Unit(UnitEventData::Summoned {
                                unit,
                                owner,
                                kind: actual,
                                ..
                            }) if *owner == test.actor && *actual == kind => Some(*unit),
                            _ => None,
                        })
                        .unwrap();
                    for _ in 0..64 {
                        if events.iter().any(|event| {
                            matches!(event.kind(), BattleEventKind::Damage(_))
                                && event
                                    .cause()
                                    .source_definition()
                                    .is_some_and(|source| source.get() == LINKED_ATTACK)
                        }) {
                            break;
                        }
                        events.extend(idle_step(&mut test));
                    }
                    let packets = events
                        .iter()
                        .filter(|event| {
                            matches!(event.kind(), BattleEventKind::Damage(_))
                                && event
                                    .cause()
                                    .source_definition()
                                    .is_some_and(|source| source.get() == LINKED_ATTACK)
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(packets.len(), 1);
                    assert_eq!(packets[0].cause().actor(), Some(CauseActor::Unit(linked)));
                    assert!(
                        matches!(packets[0].kind(), BattleEventKind::Damage(data) if data.applied.get() == 100)
                    );
                    assert_eq!(applications(&events), 0);
                    assert!(
                        !test
                            .battle
                            .view()
                            .effects_by_id()
                            .any(|effect| effect.definition().get() == EFFECT)
                    );
                    assert_eq!(entry_count(&test, linked), 0);
                    assert_eq!(entry_count(&test, test.actor), 3);
                    let hit = attack(&mut test);
                    assert_eq!(applications(&hit), 1);
                    events.extend(hit);
                    invariance(&mut test);
                    (payloads(&events), test.battle.state_hash())
                };
                assert_eq!(run(), run(), "{family:?}/{kind:?}/{formation}");
            }
        }
    }
}

#[test]
fn deflagration_lifecycle_unitless_attack_countdown_cannot_borrow_transformed_fire_owner() {
    let fixture = fixture(1);
    for family in FAMILIES {
        let parts = parts(&fixture, family);
        let form = parts.1[1].combatant().form();
        let run = || {
            let mut test = scenario(
                &parts,
                Probe {
                    hits: 1,
                    lifecycle: Some(Setup::Transform {
                        form,
                        countdown: true,
                    }),
                    ..Probe::default()
                },
            );
            let mut events = cast_self(&mut test, SETUP);
            let countdown = events
                .iter()
                .find_map(|event| match event.kind() {
                    BattleEventKind::Unit(UnitEventData::Transformed {
                        unit,
                        countdown: Some(actor),
                        ..
                    }) if *unit == test.actor => Some(*actor),
                    _ => None,
                })
                .unwrap();
            for _ in 0..64 {
                if events.iter().any(|event| {
                    matches!(event.kind(), BattleEventKind::Damage(_))
                        && event.cause().actor() == Some(CauseActor::TimelineActor(countdown))
                }) {
                    break;
                }
                events.extend(idle_step(&mut test));
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
                Some(CauseActor::TimelineActor(countdown))
            );
            assert_eq!(packets[0].cause().applier(), Some(test.actor));
            assert_eq!(applications(&events), 0);
            assert!(
                !test
                    .battle
                    .view()
                    .effects_by_id()
                    .any(|effect| effect.definition().get() == EFFECT)
            );
            assert_eq!(entry_count(&test, test.actor), 1);
            let hit = attack(&mut test);
            assert_eq!(applications(&hit), 1);
            events.extend(hit);
            events.extend(cast_self(&mut test, END_TRANSFORM));
            invariance(&mut test);
            (payloads(&events), test.battle.state_hash())
        };
        assert_eq!(run(), run(), "{family:?}");
    }
}

#[test]
fn deflagration_lifecycle_real_wave_retains_entry_count_and_final_win_cleans_captures() {
    let fixture = fixture(4);
    for family in FAMILIES {
        let parts = parts(&fixture, family);
        let run = || {
            let mut test = scenario(
                &parts,
                Probe {
                    hits: 1,
                    waves: true,
                    burns: true,
                    absent_formation: Some(3),
                    ..Probe::default()
                },
            );
            assert_eq!(entry_count(&test, test.actor), 3);
            let mut events = attack(&mut test);
            assert_eq!(applications(&events), 1);
            events.extend(cast_all_seeder(&mut test, RESTORE));
            events.extend(cast_all_seeder(&mut test, CLEAR_WAVE));
            for _ in 0..16 {
                if test.battle.view().encounter().number() == 2 {
                    break;
                }
                events.extend(idle_step(&mut test));
            }
            assert_eq!(test.battle.view().encounter().number(), 2);
            assert_eq!(entry_count(&test, test.actor), 3);
            test.targets = test
                .battle
                .view()
                .units_by_id()
                .filter(|unit| {
                    unit.side() == TeamSide::Enemy && unit.presence() == PresenceState::Present
                })
                .map(|unit| unit.id())
                .collect();
            assert_eq!(test.targets.len(), 3);
            // CarryExact retains records on departed old-wave enemies; it
            // neither migrates them to arriving targets nor recaptures counts.
            for effect in test
                .battle
                .view()
                .effects_by_id()
                .filter(|effect| effect.definition().get() == EFFECT)
            {
                assert!(!test.targets.contains(&effect.target()));
                assert_eq!(
                    test.battle
                        .view()
                        .units_by_id()
                        .find(|unit| unit.id() == effect.target())
                        .unwrap()
                        .presence(),
                    PresenceState::Departed
                );
                assert_eq!(effect.applier(), test.actor);
            }
            events.extend(cast_all_seeder(&mut test, SEED));
            let hit = attack(&mut test);
            assert_eq!(applications(&hit), 1);
            events.extend(hit);
            let tick = until_source_tick(&mut test, source(0));
            assert!(
                tick.iter()
                    .any(|event| matches!(event.kind(), BattleEventKind::Damage(data)
                if data.kind == DamageKind::DotDetonation && data.raw.scaled() == 150_000_000))
            );
            events.extend(tick);
            invariance(&mut test);
            events.extend(cast_all_seeder(&mut test, CLEAR_WAVE));
            for _ in 0..16 {
                if test.battle.view().phase().is_terminal() {
                    break;
                }
                events.extend(idle_step(&mut test));
            }
            assert_eq!(test.battle.view().phase(), BattlePhase::Won);
            assert_eq!(entry_count(&test, test.actor), 0);
            assert!(
                !test
                    .battle
                    .view()
                    .effects_by_id()
                    .any(|effect| effect.definition().get() == EFFECT)
            );
            (payloads(&events), test.battle.state_hash())
        };
        assert_eq!(run(), run(), "{family:?}");
    }
}
