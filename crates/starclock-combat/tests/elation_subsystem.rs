use std::sync::Arc;

use starclock_combat::{
    ActionEventData, ActionGauge, ActionOrigin, Battle, BattleEventKind, BattleSeed, BattleSpec,
    BattleSpecDigest, CombatantSpecDigest, Command, ConcedePolicy, DispelCategory, DurationClock,
    EffectApplicationDefinition, EffectCategory, EffectChancePolicy, EffectRuntimeDefinition,
    EffectStackPolicy, EffectTickPhase, Energy, FormationIndex, Hp, KeyedTeamResourceSpec,
    LinkedEntityKind, LinkedUnitDefinition, OwnerLinkPolicy, ParticipantSource, ParticipantSpec,
    Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings, Scalar, SkillPointPayer, Speed,
    TeamResourceSpec, TeamResourceWavePolicy, TeamSide, UnitLevel, WaveLinkPolicy,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, AbilityTag, ActionHitDefinition,
            ActionResourcePolicy, HitOperationDefinition, OrdinaryDamageDefinition,
            OrdinaryDamageMultipliers, QueueActionDefinition, QueuedActor, QueuedOwner,
            QueuedTarget, ReactionBoundary, SkillPointPaymentPolicy, TargetInvalidationPolicy,
            TargetPattern, TargetRelation, TeamResourceChange, TeamResourceChangeDefinition,
            UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition,
            ProgramDefinition, SelectorDefinition, UnitDefinition,
        },
    },
    formula::model::DamageClass,
};

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn combatant(form: u32, abilities: Vec<u32>, speed: i64, digest: u8) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        id(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(5_000).unwrap(),
        Speed::from_scaled(speed).unwrap(),
        ResolvedDefinitionBindings::new(abilities.into_iter().map(id).collect(), vec![], vec![])
            .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
}

fn action(
    kind: AbilityKind,
    selector_tags: &[AbilityTag],
    resources: ActionResourcePolicy,
    operations: Vec<HitOperationDefinition>,
) -> AbilityActionDefinition {
    AbilityActionDefinition::new(
        kind,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        resources,
    )
    .unwrap()
    .with_tags(selector_tags)
    .with_hits(vec![ActionHitDefinition::new(operations)])
    .unwrap()
}

fn catalog(shared_actor: bool) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new("shared-elation-v1", [0xe1; 32]);
    for (raw, relation, pattern) in [
        (1, TargetRelation::Allied, TargetPattern::Single),
        (2, TargetRelation::Opposing, TargetPattern::All),
        (3, TargetRelation::Opposing, TargetPattern::Single),
    ] {
        builder.add_selector(
            SelectorDefinition::new(id(raw))
                .with_unit_targets(UnitTargetSelector::new(relation, pattern).unwrap()),
        );
        builder.add_program(ProgramDefinition::new(
            id(raw),
            vec![],
            vec![id(raw)],
            vec![],
            vec![],
        ));
    }
    builder.add_program(ProgramDefinition::new(
        id(4),
        vec![],
        vec![id(2)],
        vec![],
        vec![],
    ));

    let banger = EffectRuntimeDefinition::new(
        EffectCategory::NeutralState,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::UniqueGlobal,
    )
    .unwrap();
    builder.add_effect(EffectDefinition::new(id(1), vec![], vec![]).with_runtime(banger));

    let elation_damage = OrdinaryDamageDefinition::new(
        Scalar::checked_from_integer(10).unwrap(),
        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
    )
    .unwrap()
    .with_class(DamageClass::Elation);
    let forced = action(
        AbilityKind::Skill,
        &[
            AbilityTag::Attack,
            AbilityTag::Skill,
            AbilityTag::ElationSkill,
        ],
        ActionResourcePolicy::new(1, 0, Energy::ZERO, Energy::ZERO),
        vec![HitOperationDefinition::Damage(elation_damage)],
    );

    let queue_actor = if shared_actor {
        QueuedActor::SharedEntity(LinkedEntityKind::SharedActor)
    } else {
        QueuedActor::PrimaryTarget
    };
    let payment = if shared_actor {
        SkillPointPaymentPolicy::Suppressed
    } else {
        SkillPointPaymentPolicy::TeamResource(id(90))
    };
    let queue = QueueActionDefinition::new(
        id(2),
        ActionOrigin::Forced,
        queue_actor,
        QueuedTarget::None,
        ReactionBoundary::AfterHit,
        -100,
    )
    .with_envelope(QueuedOwner::CauseOwner, Some(payment));

    let mut provider_ops = vec![HitOperationDefinition::ModifyTeamResource(
        TeamResourceChangeDefinition::new(id(90), TeamResourceChange::Gain(4)),
    )];
    if shared_actor {
        let linked = LinkedUnitDefinition::new(
            combatant(4, vec![2, 4], 100_000_000, 4),
            id(91),
            FormationIndex::new(7).unwrap(),
            LinkedEntityKind::SharedActor,
            starclock_combat::PresenceState::Linked,
            None,
            ActionGauge::from_scaled(10_000_000_000).unwrap(),
            OwnerLinkPolicy::Persist,
            OwnerLinkPolicy::Persist,
            WaveLinkPolicy::Persist,
        )
        .unwrap();
        provider_ops.push(HitOperationDefinition::SummonLinked(linked));
    } else {
        provider_ops.push(HitOperationDefinition::ApplyEffect(
            EffectApplicationDefinition::new(id(1), EffectChancePolicy::Guaranteed, 1).unwrap(),
        ));
    }
    provider_ops.push(HitOperationDefinition::QueueAction(queue));

    builder.add_ability(
        AbilityDefinition::new(
            id(1),
            id(1),
            id(1),
            if shared_actor { vec![] } else { vec![id(1)] },
        )
        .with_action(action(
            AbilityKind::Skill,
            &[AbilityTag::Attack, AbilityTag::Skill],
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
            provider_ops,
        )),
    );
    builder.add_ability(AbilityDefinition::new(id(2), id(2), id(2), vec![]).with_action(forced));
    builder.add_ability(
        AbilityDefinition::new(id(3), id(3), id(3), vec![]).with_action(action(
            AbilityKind::Basic,
            &[AbilityTag::Attack, AbilityTag::Basic],
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
            vec![],
        )),
    );
    builder.add_ability(
        AbilityDefinition::new(id(4), id(4), id(2), vec![]).with_action(action(
            AbilityKind::Basic,
            &[AbilityTag::Attack, AbilityTag::Basic],
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
            vec![],
        )),
    );
    builder.add_unit(UnitDefinition::new(id(1), vec![id(1)], vec![]));
    builder.add_unit(UnitDefinition::new(id(2), vec![id(2), id(4)], vec![]));
    builder.add_unit(UnitDefinition::new(id(3), vec![id(3)], vec![]));
    builder.add_unit(UnitDefinition::new(id(4), vec![id(2), id(4)], vec![]));
    builder.add_enemy(EnemyDefinition::new(id(1), id(3), vec![id(3)]));
    builder.add_encounter(EncounterDefinition::new(id(1), vec![id(1)], vec![]));
    builder.build().unwrap()
}

fn run(shared_actor: bool) -> (Battle, starclock_combat::Resolution) {
    let player_resources = TeamResourceSpec::new(1, 5)
        .unwrap()
        .with_keyed(vec![
            KeyedTeamResourceSpec::new(id(90), 2, 5, TeamResourceWavePolicy::Persist).unwrap(),
        ])
        .unwrap();
    let spec = BattleSpec::new(
        "shared-elation-rules-v1",
        BattleSpecDigest::new([0xe2; 32]).unwrap(),
        id(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, vec![1], 300_000_000, 1),
            ),
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(1).unwrap(),
                ParticipantSource::Player,
                combatant(2, vec![2, 4], 100_000_000, 2),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(id(1)),
                combatant(3, vec![3], 50_000_000, 3),
            ),
        ],
        player_resources,
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle =
        Battle::create(catalog(shared_actor), spec, BattleSeed::new([0xe3; 32])).unwrap();
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
    let command = battle.decision().unwrap().legal_commands().iter().find(|command| {
        matches!(command, Command::UseAbility { ability, primary_target: Some(target), .. } if ability.get() == 1 && target.get() == 2)
    }).unwrap().clone();
    let resolution = battle.apply(command).unwrap();
    (battle, resolution)
}

#[test]
fn forced_elation_skill_keeps_provider_actor_tags_damage_and_substitute_cost_distinct() {
    let (battle, resolution) = run(false);
    let declared = resolution.events().iter().find(|event| matches!(
        event.kind(),
        BattleEventKind::Action(ActionEventData::Declared { actor, ability, origin: ActionOrigin::Forced, tags, .. })
            if actor.get() == 2 && ability.get() == 2 && tags.contains(AbilityTag::ElationSkill)
    )).unwrap();
    assert_eq!(declared.cause().owner().unwrap().get(), 1);
    assert_eq!(
        declared.cause().actor(),
        Some(starclock_combat::CauseActor::Unit(
            starclock_combat::UnitId::new(2).unwrap()
        ))
    );
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Damage(data) if data.class == DamageClass::Elation
    )));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Resource(starclock_combat::ResourceEventData::SkillPoints {
            attempted: 1,
            payer: SkillPointPayer::TeamResource(resource),
            effective: 1,
            before: 5,
            after: 4,
            ..
        }) if resource.get() == 90
    )));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Resource(starclock_combat::ResourceEventData::TeamResource {
            resource, attempted: 4, effective: 3, before: 2, after: 5, overflow: 1, ..
        }) if resource.get() == 90
    )));
    assert_eq!(
        battle.view().team(TeamSide::Player).keyed_resource(id(90)),
        Some((4, 5))
    );
    assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 1);
    let effect = battle.view().effects_by_id().next().unwrap();
    assert_eq!((effect.applier().get(), effect.target().get()), (1, 2));
    assert_eq!(effect.source_definition().get(), 1);
}

#[test]
fn shared_linked_actor_is_selected_by_kind_and_uses_suppressed_forced_cost() {
    let (battle, resolution) = run(true);
    let shared = battle
        .view()
        .links()
        .find(|link| link.kind() == LinkedEntityKind::SharedActor)
        .unwrap();
    assert!(shared.is_active());
    let starclock_combat::LinkedEntity::Unit(shared_unit) = shared.entity() else {
        panic!("shared actor must be a unit")
    };
    let declared = resolution.events().iter().find(|event| matches!(
        event.kind(),
        BattleEventKind::Action(ActionEventData::Declared { actor, origin: ActionOrigin::Forced, .. }) if *actor == shared_unit
    )).unwrap();
    assert_eq!(declared.cause().owner().unwrap().get(), 1);
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Resource(starclock_combat::ResourceEventData::SkillPoints {
            attempted: 1,
            payer: SkillPointPayer::Suppressed,
            effective: 0,
            before: 0,
            after: 0,
            ..
        })
    )));
}
