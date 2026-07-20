use std::sync::Arc;

use starclock_combat::{
    ActionEventData, ActionOrigin, Battle, BattleEventKind, BattleSeed, BattleSpec,
    BattleSpecDigest, CombatantSpecDigest, Command, ConcedePolicy, ControlledAction,
    DispelCategory, DurationClock, EffectApplicationDefinition, EffectCategory, EffectChancePolicy,
    EffectRuntimeDefinition, EffectStackPolicy, EffectTickPhase, Energy, FormationIndex, Hp,
    ParticipantSource, ParticipantSpec, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    Scalar, Speed, TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HitOperationDefinition, OrdinaryDamageDefinition, OrdinaryDamageMultipliers,
            QueueActionDefinition, QueuedActor, QueuedTarget, ReactionBoundary,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition,
            ProgramDefinition, SelectorDefinition, UnitDefinition,
        },
    },
};

fn definition<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn action(kind: AbilityKind, operations: Vec<HitOperationDefinition>) -> AbilityActionDefinition {
    AbilityActionDefinition::new(
        kind,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap()
    .with_hits(vec![ActionHitDefinition::new(operations)])
    .unwrap()
}

fn catalog(
    control_counter: bool,
    boundary: ReactionBoundary,
    origin: ActionOrigin,
    queued_kind: AbilityKind,
    invalidate_attacker: bool,
) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new("reaction-scheduler-v1", [0xb6; 32]);
    for (raw, relation) in [
        (1, TargetRelation::Opposing),
        (2, TargetRelation::Opposing),
        (3, TargetRelation::Opposing),
        (4, TargetRelation::SelfUnit),
    ] {
        builder.add_selector(
            SelectorDefinition::new(definition(raw)).with_unit_targets(
                UnitTargetSelector::new(relation, TargetPattern::Single).unwrap(),
            ),
        );
        builder.add_program(ProgramDefinition::new(
            definition(raw),
            vec![],
            vec![definition(raw)],
            if raw == 3 && control_counter {
                vec![definition(1)]
            } else {
                vec![]
            },
            vec![],
        ));
    }

    let mut enemy_operations = Vec::new();
    if control_counter {
        let runtime = EffectRuntimeDefinition::new(
            EffectCategory::Control,
            DispelCategory::DispellableDebuff,
            1,
            Some(1),
            DurationClock::TargetTurnEnd,
            EffectTickPhase::None,
            EffectStackPolicy::Refresh,
        )
        .unwrap()
        .with_control(vec![ControlledAction::FollowUp])
        .unwrap();
        builder
            .add_effect(EffectDefinition::new(definition(1), vec![], vec![]).with_runtime(runtime));
        enemy_operations.push(HitOperationDefinition::ApplyEffect(
            EffectApplicationDefinition::new(definition(1), EffectChancePolicy::Guaranteed, 1)
                .unwrap(),
        ));
    }
    if invalidate_attacker {
        enemy_operations.push(HitOperationDefinition::QueueAction(
            QueueActionDefinition::new(
                definition(4),
                ActionOrigin::ExtraAction,
                QueuedActor::CauseApplier,
                QueuedTarget::None,
                ReactionBoundary::AfterAction,
                -200,
            ),
        ));
    }
    enemy_operations.push(HitOperationDefinition::QueueAction(
        QueueActionDefinition::new(
            definition(2),
            origin,
            QueuedActor::PrimaryTarget,
            QueuedTarget::CauseActor,
            boundary,
            -100,
        ),
    ));

    builder.add_ability(
        AbilityDefinition::new(definition(1), definition(1), definition(1), vec![])
            .with_action(action(AbilityKind::Basic, vec![])),
    );
    builder.add_ability(
        AbilityDefinition::new(definition(2), definition(2), definition(2), vec![])
            .with_action(action(queued_kind, vec![])),
    );
    builder.add_ability(
        AbilityDefinition::new(
            definition(3),
            definition(3),
            definition(3),
            if control_counter {
                vec![definition(1)]
            } else {
                vec![]
            },
        )
        .with_action(action(AbilityKind::Basic, enemy_operations)),
    );
    builder.add_ability(
        AbilityDefinition::new(definition(4), definition(4), definition(4), vec![]).with_action(
            action(
                AbilityKind::ExtraAction,
                vec![HitOperationDefinition::Damage(
                    OrdinaryDamageDefinition::new(
                        Scalar::checked_from_integer(2_000).unwrap(),
                        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                    )
                    .unwrap(),
                )],
            ),
        ),
    );
    builder.add_unit(UnitDefinition::new(
        definition(1),
        vec![definition(1), definition(2)],
        vec![],
    ));
    builder.add_unit(UnitDefinition::new(
        definition(2),
        vec![definition(3), definition(4)],
        vec![],
    ));
    builder.add_enemy(EnemyDefinition::new(
        definition(1),
        definition(2),
        vec![definition(3), definition(4)],
    ));
    builder.add_encounter(EncounterDefinition::new(
        definition(1),
        vec![definition(1)],
        vec![],
    ));
    builder.build().unwrap()
}

fn combatant(form: u32, abilities: Vec<u32>, speed: i64, digest: u8) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        definition(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(speed).unwrap(),
        ResolvedDefinitionBindings::new(
            abilities.into_iter().map(definition).collect(),
            vec![],
            vec![],
        )
        .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
}

fn battle(
    control_counter: bool,
    boundary: ReactionBoundary,
    origin: ActionOrigin,
    queued_kind: AbilityKind,
    invalidate_attacker: bool,
) -> Battle {
    let spec = BattleSpec::new(
        "reaction-scheduler-rules-v1",
        BattleSpecDigest::new([0xc6; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, vec![1, 2], 100_000_000, 1),
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
    Battle::create(
        catalog(
            control_counter,
            boundary,
            origin,
            queued_kind,
            invalidate_attacker,
        ),
        spec,
        BattleSeed::new([0xd6; 32]),
    )
    .unwrap()
}

fn execute_enemy_attack(
    control_counter: bool,
    boundary: ReactionBoundary,
    origin: ActionOrigin,
    queued_kind: AbilityKind,
    invalidate_attacker: bool,
) -> (Battle, starclock_combat::Resolution) {
    let mut battle = battle(
        control_counter,
        boundary,
        origin,
        queued_kind,
        invalidate_attacker,
    );
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    battle
        .apply(Command::PassInterruptWindow {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    let attack = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(
            |command| matches!(command, Command::UseAbility { ability, .. } if ability.get() == 3),
        )
        .unwrap()
        .clone();
    let resolution = battle.apply(attack).unwrap();
    (battle, resolution)
}

#[test]
fn counter_preserves_cause_and_does_not_own_the_normal_timeline_turn() {
    let (_battle, resolution) = execute_enemy_attack(
        false,
        ReactionBoundary::AfterHit,
        ActionOrigin::Counter,
        AbilityKind::Counter,
        false,
    );
    let actions = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Action(data) => Some((event, data)),
            _ => None,
        })
        .collect::<Vec<_>>();
    let queued = actions
        .iter()
        .find(|(_, data)| matches!(data, ActionEventData::Queued { .. }))
        .unwrap()
        .0;
    let declared = actions
        .iter()
        .find(|(_, data)| {
            matches!(
                data,
                ActionEventData::Declared {
                    ability,
                    origin: ActionOrigin::Counter,
                    ..
                } if ability.get() == 2
            )
        })
        .unwrap()
        .0;
    assert_eq!(declared.cause().parent_event(), Some(queued.id()));
    assert_eq!(
        declared.cause().root_command(),
        queued.cause().root_command()
    );
    assert_eq!(
        declared.cause().primary_target(),
        queued.cause().actor().and_then(|actor| match actor {
            starclock_combat::CauseActor::Unit(unit) => Some(unit),
            starclock_combat::CauseActor::TimelineActor(_) => None,
        })
    );
    assert_eq!(queued.cause().owner(), queued.cause().applier());
    assert_eq!(declared.cause().owner(), queued.cause().primary_target());
    assert_eq!(
        declared.cause().actor(),
        declared
            .cause()
            .owner()
            .map(starclock_combat::CauseActor::Unit)
    );
    assert_eq!(declared.cause().source_definition().unwrap().get(), 2);

    let ended_turns = resolution
        .events()
        .iter()
        .filter(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Turn(starclock_combat::TurnEventData::Ended { .. })
            )
        })
        .count();
    assert_eq!(
        ended_turns, 1,
        "the counter does not create or end a normal turn"
    );
}

#[test]
fn crowd_control_cancels_a_counter_without_consuming_rng() {
    let (battle, resolution) = execute_enemy_attack(
        true,
        ReactionBoundary::AfterHit,
        ActionOrigin::Counter,
        AbilityKind::Counter,
        false,
    );
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Action(ActionEventData::Cancelled {
            ability,
            origin: ActionOrigin::Counter,
            ..
        }) if ability.get() == 2
    )));
    assert!(!resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Action(ActionEventData::Declared {
            origin: ActionOrigin::Counter,
            ..
        })
    )));
    assert_eq!(battle.view().rng_draw_count(), 0);
}

#[test]
fn a_delayed_action_waits_until_the_declared_boundary() {
    let (_, resolution) = execute_enemy_attack(
        false,
        ReactionBoundary::BeforeTimeline,
        ActionOrigin::DelayedAction,
        AbilityKind::DelayedAction,
        false,
    );
    let events = resolution.events();
    let turn_ended = events
        .iter()
        .position(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Turn(starclock_combat::TurnEventData::Ended { .. })
            )
        })
        .unwrap();
    let counter_started = events
        .iter()
        .position(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Action(ActionEventData::Started {
                    origin: ActionOrigin::DelayedAction,
                    ..
                })
            )
        })
        .unwrap();
    assert!(turn_ended < counter_started);
}

#[test]
fn every_automatic_action_family_uses_the_common_envelope_without_owning_a_turn() {
    for (origin, kind) in [
        (ActionOrigin::FollowUp, AbilityKind::FollowUp),
        (ActionOrigin::ExtraAction, AbilityKind::ExtraAction),
        (ActionOrigin::ExtraTurn, AbilityKind::ExtraTurn),
    ] {
        let (_, resolution) =
            execute_enemy_attack(false, ReactionBoundary::AfterAction, origin, kind, false);
        assert!(resolution.events().iter().any(|event| matches!(
            event.kind(),
            BattleEventKind::Action(ActionEventData::Resolved {
                ability,
                origin: resolved_origin,
                ..
            }) if ability.get() == 2 && *resolved_origin == origin
        )));
        assert_eq!(
            resolution
                .events()
                .iter()
                .filter(|event| matches!(
                    event.kind(),
                    BattleEventKind::Turn(starclock_combat::TurnEventData::Ended { .. })
                ))
                .count(),
            1
        );
    }
}

#[test]
fn invalidated_queued_target_cancels_without_an_implicit_fallback() {
    let (battle, resolution) = execute_enemy_attack(
        false,
        ReactionBoundary::AfterAction,
        ActionOrigin::Counter,
        AbilityKind::Counter,
        true,
    );
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Action(ActionEventData::Cancelled {
            ability,
            origin: ActionOrigin::Counter,
            ..
        }) if ability.get() == 2
    )));
    assert!(!resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Action(ActionEventData::Declared {
            ability,
            origin: ActionOrigin::Counter,
            ..
        }) if ability.get() == 2
    )));
    assert_eq!(battle.view().rng_draw_count(), 0);
}
