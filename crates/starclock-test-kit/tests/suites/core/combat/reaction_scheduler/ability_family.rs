//! Both queue paths select a bound variant, never execute an unbound family root.
use super::{action, catalog, combatant, definition};
use crate::combat_decision::advance_boundary_if_offered;
use starclock_combat::{
    ActionEventData, ActionOrigin, AssemblyDigest, Battle, BattleEvent, BattleEventKind,
    BattleSeed, BattleSpec, Command, ConcedePolicy, DecisionId, DispelCategory, DurationClock,
    EffectApplicationDefinition, EffectCategory, EffectChancePolicy, EffectRuntimeDefinition,
    EffectStackPolicy, EffectTickPhase, FormationIndex, Hp, ParticipantSource, ParticipantSpec,
    Ratio, Scalar, TeamResourceSpec, TeamSide,
    catalog::{
        CombatCatalog,
        action::{
            AbilityKind, AbilityProgramBinding, AbilityProgramTiming, HitOperationDefinition,
            OrdinaryDamageDefinition, OrdinaryDamageMultipliers, QueueActionDefinition,
            QueuedActor, QueuedTarget, ReactionBoundary,
        },
        builder::{CatalogBuildErrorKind, CombatCatalogBuilder},
        definition::{AbilityDefinition, EffectDefinition, ProgramDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    rule::model::{ProgramStep, ReactionPriority, RuleActionOwner, RuleOperationTemplate},
};
use std::sync::Arc;

fn builder(rule_queue: bool, requested: u32) -> CombatCatalogBuilder {
    let base = catalog(
        false,
        ReactionBoundary::AfterAction,
        ActionOrigin::Counter,
        AbilityKind::Counter,
        false,
    );
    let mut builder = CombatCatalogBuilder::from_catalog(&base, [0xee; 32]);
    for (raw, amount) in [(5, 37), (6, 71)] {
        builder.add_ability(
            AbilityDefinition::new(definition(raw), definition(2), definition(2), vec![])
                .with_family(definition(2))
                .with_action(action(
                    AbilityKind::Counter,
                    vec![HitOperationDefinition::Damage(
                        OrdinaryDamageDefinition::new(
                            Scalar::checked_from_integer(amount).unwrap(),
                            OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                        )
                        .unwrap(),
                    )],
                )),
        );
    }
    if rule_queue {
        for (raw, origin, side) in [
            (
                5,
                RuleSelectorOrigin::PrimaryTarget,
                RuleSelectorSide::Opposing,
            ),
            (6, RuleSelectorOrigin::Actor, RuleSelectorSide::Same),
        ] {
            builder.add_selector(
                SelectorDefinition::new(definition(raw)).with_rule_units(
                    RuleUnitSelector::new(
                        origin,
                        side,
                        RuleLifePredicate::Alive,
                        RulePresencePredicate::Present,
                        RuleSelectorReference::CurrentState,
                        RuleSelectorOrdering::StableId,
                        1,
                        1,
                        RuleEmptyPoolPolicy::Fault,
                        RuleSelectorChoice::First,
                        None,
                        false,
                    )
                    .unwrap(),
                ),
            );
        }
        builder.add_program(
            ProgramDefinition::new(
                definition(5),
                vec![],
                vec![definition(5), definition(6)],
                vec![],
                vec![],
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::QueueAction {
                    actor_selector: definition(5),
                    target_selector: definition(6),
                    ability: definition(requested),
                    priority: ReactionPriority::new(-100),
                    forced_use: false,
                    boundary: ReactionBoundary::AfterAction,
                    owner: RuleActionOwner::Actor,
                    payment: None,
                },
            )]),
        );
        assert!(
            builder.replace_ability(
                AbilityDefinition::new(definition(3), definition(3), definition(3), vec![])
                    .with_action(action(AbilityKind::Basic, vec![]))
                    .with_programs(vec![
                        AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, definition(5))
                            .unwrap()
                    ])
            )
        );
    } else if requested != 2 {
        assert!(
            builder.replace_ability(
                AbilityDefinition::new(definition(3), definition(3), definition(3), vec![])
                    .with_action(action(AbilityKind::Basic, vec![queue(requested)]))
            )
        );
    }
    builder
}

fn queue(requested: u32) -> HitOperationDefinition {
    HitOperationDefinition::QueueAction(QueueActionDefinition::new(
        definition(requested),
        ActionOrigin::Counter,
        QueuedActor::PrimaryTarget,
        QueuedTarget::CauseActor,
        ReactionBoundary::AfterAction,
        -100,
    ))
}

fn battle(catalog: Arc<CombatCatalog>, bound: Vec<u32>) -> Battle {
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xec; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, bound, 100_000_000, 1),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(2, vec![3, 4], 200_000_000, 2),
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog, spec, BattleSeed::new([0xed; 32])).unwrap()
}

fn attack(battle: &mut Battle, fresh: &mut Battle) -> Vec<BattleEvent> {
    let before = battle.state_hash();
    let rng = battle.view().rng_draw_count();
    assert!(
        battle
            .apply(Command::StartBattle {
                decision: DecisionId::new(999).unwrap()
            })
            .is_err()
    );
    assert_eq!(battle.state_hash(), before);
    assert_eq!(battle.view().rng_draw_count(), rng);
    let mut events = Vec::new();
    for step in 0..3 {
        let command = if step == 0 {
            Command::StartBattle {
                decision: battle.decision().unwrap().id(),
            }
        } else if step == 1 {
            advance_boundary_if_offered(battle);
            advance_boundary_if_offered(fresh);
            battle.decision().unwrap().legal_commands().iter()
                .find(|command| matches!(command, Command::UseAbility { ability, .. } if ability.get() == 3)).unwrap().clone()
        } else {
            let Some(command) = battle.advance_command() else {
                break;
            };
            command
        };
        let actual = battle.apply(command.clone()).unwrap();
        let reconstructed = fresh.apply(command).unwrap();
        assert_eq!(actual.events(), reconstructed.events());
        assert_eq!(actual.fault(), reconstructed.fault());
        assert_eq!(battle.state_hash(), fresh.state_hash());
        assert_eq!(
            battle.view().rng_draw_count(),
            fresh.view().rng_draw_count()
        );
        events.extend_from_slice(actual.events());
        if actual.fault().is_some() {
            break;
        }
    }
    events
}

#[test]
fn hit_and_rule_queues_execute_the_unique_bound_effective_variant() {
    for rule_queue in [false, true] {
        for (bound, expected, amount) in [
            (vec![1, 5], 5, 37),
            (vec![1, 6], 6, 71),
            (vec![1, 2, 5, 6], 2, 0),
        ] {
            let catalog = builder(rule_queue, 2).build().unwrap();
            let mut running = battle(Arc::clone(&catalog), bound.clone());
            let mut fresh = battle(catalog, bound);
            let events = attack(&mut running, &mut fresh);
            assert!(events.iter().any(|event| matches!(event.kind(), BattleEventKind::Action(ActionEventData::Queued { ability, .. }) if ability.get() == expected)));
            assert!(events.iter().any(|event| matches!(event.kind(), BattleEventKind::Action(ActionEventData::Declared { ability, origin: ActionOrigin::Counter, .. }) if ability.get() == expected)));
            assert!(!events.iter().any(|event| matches!(
                event.kind(),
                BattleEventKind::Action(ActionEventData::Cancelled { .. })
            )));
            if amount != 0 {
                assert!(events.iter().any(|event| matches!(event.kind(), BattleEventKind::Damage(data) if data.raw.scaled() == amount * 1_000_000)));
            }
        }
    }
}

#[test]
fn ambiguous_bound_variants_fault_without_enqueuing_or_choosing_by_id() {
    for rule_queue in [false, true] {
        let catalog = builder(rule_queue, 2).build().unwrap();
        let mut running = battle(Arc::clone(&catalog), vec![1, 5, 6]);
        let mut fresh = battle(catalog, vec![1, 5, 6]);
        let events = attack(&mut running, &mut fresh);
        assert!(events.iter().any(|event| matches!(event.kind(), BattleEventKind::Fault(fault) if fault.fault().context_code() == 0x32f0)));
        assert!(!events.iter().any(|event| matches!(
            event.kind(),
            BattleEventKind::Action(ActionEventData::Queued { .. })
        )));
        assert_eq!(
            running
                .view()
                .units_by_id()
                .find(|unit| unit.side() == TeamSide::Enemy)
                .unwrap()
                .current_hp(),
            Hp::new(1_000).unwrap()
        );
    }
}

#[test]
fn family_catalog_rejects_missing_chained_and_cyclic_roots() {
    for (root, expected) in [
        (99, CatalogBuildErrorKind::MissingReference),
        (6, CatalogBuildErrorKind::InvalidDefinition),
    ] {
        let mut builder = builder(false, 2);
        assert!(
            builder.replace_ability(
                AbilityDefinition::new(definition(5), definition(2), definition(2), vec![])
                    .with_family(definition(root))
                    .with_action(action(AbilityKind::Counter, vec![]))
            )
        );
        assert_eq!(builder.build().unwrap_err().kind(), expected);
    }
    let mut builder = builder(false, 2);
    for (id, family) in [(5, 6), (6, 5)] {
        assert!(
            builder.replace_ability(
                AbilityDefinition::new(definition(id), definition(2), definition(2), vec![])
                    .with_family(definition(family))
                    .with_action(action(AbilityKind::Counter, vec![]))
            )
        );
    }
    assert_eq!(
        builder.build().unwrap_err().kind(),
        CatalogBuildErrorKind::InvalidDefinition
    );
}

#[test]
fn unbound_roots_and_exact_variants_cancel_without_family_retargeting() {
    for rule_queue in [false, true] {
        for (requested, bound) in [(2, vec![1]), (5, vec![1, 6]), (6, vec![1, 5])] {
            let catalog = builder(rule_queue, requested).build().unwrap();
            let mut running = battle(Arc::clone(&catalog), bound.clone());
            let mut fresh = battle(catalog, bound);
            let events = attack(&mut running, &mut fresh);
            assert!(events.iter().any(|event| matches!(event.kind(), BattleEventKind::Action(ActionEventData::Queued { ability, .. }) if ability.get() == requested)));
            assert!(events.iter().any(|event| matches!(event.kind(), BattleEventKind::Action(ActionEventData::Cancelled { ability, .. }) if ability.get() == requested)));
            assert!(!events.iter().any(|event| matches!(
                event.kind(),
                BattleEventKind::Action(ActionEventData::Declared {
                    origin: ActionOrigin::Counter,
                    ..
                })
            )));
            assert!(running.view().fault().is_none());
        }
    }
}

#[test]
fn a_current_effect_grant_can_supply_the_unique_bound_variant() {
    let mut builder = builder(false, 2);
    let runtime = EffectRuntimeDefinition::new(
        EffectCategory::Buff,
        DispelCategory::DispellableBuff,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::UniquePerSource,
    )
    .unwrap();
    builder.add_effect(
        EffectDefinition::new(definition(7), vec![], vec![])
            .with_runtime(runtime)
            .with_granted_abilities(vec![definition(5)]),
    );
    assert!(
        builder.replace_ability(
            AbilityDefinition::new(
                definition(3),
                definition(3),
                definition(3),
                vec![definition(7)]
            )
            .with_action(action(
                AbilityKind::Basic,
                vec![
                    HitOperationDefinition::ApplyEffect(
                        EffectApplicationDefinition::new(
                            definition(7),
                            EffectChancePolicy::Guaranteed,
                            1
                        )
                        .unwrap()
                    ),
                    queue(2)
                ]
            ))
        )
    );
    let catalog = builder.build().unwrap();
    let mut running = battle(Arc::clone(&catalog), vec![1]);
    let mut fresh = battle(catalog, vec![1]);
    let events = attack(&mut running, &mut fresh);
    assert!(events.iter().any(|event| matches!(event.kind(), BattleEventKind::Action(ActionEventData::Declared { ability, origin: ActionOrigin::Counter, .. }) if ability.get() == 5)));
    assert!(events.iter().any(
        |event| matches!(event.kind(), BattleEventKind::Damage(data) if data.applied.get() == 37)
    ));
    assert!(running.view().fault().is_none());
}
