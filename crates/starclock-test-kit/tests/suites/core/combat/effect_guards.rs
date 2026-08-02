use std::sync::Arc;

use starclock_combat::{
    AssemblyDigest, Battle, BattleEventKind, BattleSeed, BattleSpec, CombatantSpecDigest, Command,
    ConcedePolicy, DispelCategory, DurationClock, EffectApplicationDefinition,
    EffectApplicationGuard, EffectCategory, EffectChancePolicy, EffectDamageGuard,
    EffectDefinitionId, EffectEventData, EffectRuntimeDefinition, EffectStackPolicy,
    EffectTickPhase, FormationIndex, Hp, LifeState, NEGATIVE_EFFECT_GUARDED_SIGNAL,
    ParticipantSource, ParticipantSpec, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    Scalar, Speed, TEAM_DEFEAT_GUARDED_SIGNAL, TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HitOperationDefinition, OrdinaryDamageDefinition, OrdinaryDamageMultipliers,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition,
            ProgramDefinition, SelectorDefinition, UnitDefinition,
        },
        encounter::{
            AiCandidateDefinition, AiCandidateSelection, AiGraphDefinition, AiNoTargetFallback,
            AiStateDefinition,
        },
    },
    rule::model::ConditionExpr,
};

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn action(operations: Vec<HitOperationDefinition>) -> AbilityActionDefinition {
    AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::KeepIfPresent,
        ActionResourcePolicy::new(
            0,
            0,
            starclock_combat::Energy::ZERO,
            starclock_combat::Energy::ZERO,
        ),
    )
    .unwrap()
    .with_hits(vec![ActionHitDefinition::new(operations)])
    .unwrap()
}

fn permanent_guard() -> EffectRuntimeDefinition {
    EffectRuntimeDefinition::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::TurnEnd,
        EffectStackPolicy::Refresh,
    )
    .unwrap()
}

fn catalog() -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new([0x62; 32]);
    for (raw, relation) in [(1, TargetRelation::SelfUnit), (2, TargetRelation::Opposing)] {
        builder.add_selector(
            SelectorDefinition::new(id(raw)).with_unit_targets(
                UnitTargetSelector::new(relation, TargetPattern::Single).unwrap(),
            ),
        );
        builder.add_program(ProgramDefinition::new(
            id(raw),
            vec![],
            vec![id(raw)],
            vec![],
            vec![],
        ));
    }

    builder.add_effect(
        EffectDefinition::new(id(10), vec![], vec![])
            .with_runtime(permanent_guard().with_damage_guard(EffectDamageGuard::TeamDefeatOnce)),
    );
    builder.add_effect(EffectDefinition::new(id(11), vec![], vec![]).with_runtime(
        permanent_guard().with_application_guard(EffectApplicationGuard::NegativeEffectOnce),
    ));
    builder.add_effect(
        EffectDefinition::new(id(12), vec![], vec![]).with_runtime(
            EffectRuntimeDefinition::new(
                EffectCategory::Debuff,
                DispelCategory::DispellableDebuff,
                1,
                Some(2),
                DurationClock::TargetTurnEnd,
                EffectTickPhase::TurnEnd,
                EffectStackPolicy::Refresh,
            )
            .unwrap(),
        ),
    );

    let apply = |effect: u32| {
        HitOperationDefinition::ApplyEffect(
            EffectApplicationDefinition::new(
                id::<EffectDefinitionId>(effect),
                EffectChancePolicy::Guaranteed,
                1,
            )
            .unwrap(),
        )
    };
    builder.add_ability(
        AbilityDefinition::new(id(1), id(1), id(1), vec![])
            .with_action(action(vec![apply(10), apply(11)])),
    );
    let damage = OrdinaryDamageDefinition::new(
        Scalar::checked_from_integer(5_000).unwrap(),
        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
    )
    .unwrap();
    builder.add_ability(
        AbilityDefinition::new(id(2), id(2), id(2), vec![]).with_action(action(vec![
            apply(12),
            HitOperationDefinition::Damage(damage),
        ])),
    );

    builder.add_unit(UnitDefinition::new(id(1), vec![id(1)], vec![]));
    builder.add_unit(UnitDefinition::new(id(2), vec![id(2)], vec![]));
    let candidate = AiCandidateDefinition::new(
        id(1),
        id(2),
        ConditionExpr::Literal(true),
        id(2),
        0,
        AiCandidateSelection::FirstLegal,
        AiNoTargetFallback::StayInState,
    );
    builder.add_ai_graph(
        AiGraphDefinition::new(
            id(1),
            id(1),
            4,
            vec![AiStateDefinition::new(
                id(1),
                None,
                id(2),
                false,
                vec![candidate],
                vec![],
            )],
        )
        .unwrap(),
    );
    builder.add_enemy(
        EnemyDefinition::new(id(1), id(2), vec![id(2)])
            .with_orchestration(id(1), vec![])
            .unwrap(),
    );
    builder.add_encounter(EncounterDefinition::new(id(1), vec![id(1)], vec![]));
    builder.build().unwrap()
}

fn combatant(form: u32, ability: u32, speed: i64, digest: u8) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        id(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(speed).unwrap(),
        ResolvedDefinitionBindings::new(vec![id(ability)], vec![], vec![]).unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
}

fn battle() -> Battle {
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x72; 32]).unwrap(),
        id(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 1, 101_000_000, 0x11),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(id(1)),
                combatant(2, 2, 100_000_000, 0x22),
            ),
        ],
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog(), spec, BattleSeed::new([0x82; 32])).unwrap()
}

#[test]
fn one_shot_effect_guards_reject_a_debuff_and_prevent_team_defeat() {
    let mut battle = battle();
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    let pass = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::PassInterruptWindow { .. }))
        .unwrap()
        .clone();
    battle.apply(pass).unwrap();
    let use_guard = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { ability, .. } if *ability == id(1)))
        .unwrap()
        .clone();
    let mut events = battle.apply(use_guard).unwrap().events().to_vec();
    for _ in 0..4 {
        let Some(command) = battle.decision().and_then(|decision| {
            decision
                .legal_commands()
                .iter()
                .find(|command| match command {
                    Command::PassInterruptWindow { .. } => true,
                    Command::UseAbility { ability, .. } => *ability == id(2),
                    _ => false,
                })
                .cloned()
        }) else {
            break;
        };
        events.extend_from_slice(battle.apply(command).unwrap().events());
    }

    let player = battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Player)
        .unwrap();
    assert_eq!(
        player.current_hp().get(),
        1,
        "next decision: {:?}; events: {events:#?}",
        battle.decision()
    );
    assert_eq!(player.life(), LifeState::Alive);
    assert_eq!(battle.view().effects_by_id().count(), 0);
    assert!(events.iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::RuleSignal(signal)
            if signal.code == NEGATIVE_EFFECT_GUARDED_SIGNAL
    )));
    assert!(events.iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::RuleSignal(signal)
                if signal.code == TEAM_DEFEAT_GUARDED_SIGNAL
                    && event.cause().primary_target() == Some(player.id())
        )
    }));
    assert!(events.iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Effect(EffectEventData::Resisted { definition, .. })
            if *definition == id(12)
    )));
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                event.kind(),
                BattleEventKind::Effect(EffectEventData::Removed { definition, .. })
                    if *definition == id(10) || *definition == id(11)
            ))
            .count(),
        2
    );
}
