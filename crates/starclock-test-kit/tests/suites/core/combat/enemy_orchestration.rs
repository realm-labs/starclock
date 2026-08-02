use std::sync::Arc;

use starclock_combat::{
    AssemblyDigest, Battle, BattleEventKind, BattleSeed, BattleSpec, CombatantSpecDigest, Command,
    ConcedePolicy, DispelCategory, DurationClock, EffectApplicationDefinition, EffectCategory,
    EffectChancePolicy, EffectDefinitionId, EffectRuntimeDefinition, EffectStackPolicy,
    EffectTickPhase, EncounterWaveId, EnemyPhaseEventData, EnemyPhaseId, FormationIndex, Hp,
    ParticipantSource, ParticipantSpec, PresenceState, Ratio, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, Scalar, Speed, TeamResourceSpec, TeamSide, ToughnessLayerSpec,
    UnitLevel,
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
            AiStateDefinition, EncounterWaveDefinition, EnemyPhaseCarry, EnemyPhaseDefinition,
            EnemyPhaseTransitionModel, PhaseCarryPolicy, WaveCarry, WaveSlotDefinition,
            WaveTransitionPolicy,
        },
    },
    formula::{model::CombatElement, toughness::EnemyRank},
    rule::model::{ConditionExpr, ProgramStep, RuleOperationTemplate},
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

fn damage(amount: i64) -> OrdinaryDamageDefinition {
    OrdinaryDamageDefinition::new(
        Scalar::checked_from_integer(amount).unwrap(),
        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
    )
    .unwrap()
}

fn phase(raw: u32, targetable: bool, carry: EnemyPhaseCarry) -> EnemyPhaseDefinition {
    EnemyPhaseDefinition::new(
        id::<EnemyPhaseId>(raw),
        u16::try_from(raw).unwrap(),
        ConditionExpr::Literal(true),
        ConditionExpr::Literal(false),
        0,
        id(1),
        targetable,
        EnemyPhaseTransitionModel::TransformSameUnit,
        (raw == 2).then(|| id(4)),
        carry,
    )
}

fn catalog() -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new([0xb9; 32]);
    for (raw, relation) in [(1, TargetRelation::Opposing), (2, TargetRelation::Opposing)] {
        builder.add_selector(
            SelectorDefinition::new(id(raw)).with_unit_targets(
                UnitTargetSelector::new(relation, TargetPattern::Single).unwrap(),
            ),
        );
        builder.add_program(ProgramDefinition::new(
            id(raw),
            vec![],
            vec![id(raw)],
            if raw == 1 { vec![id(1)] } else { vec![] },
            vec![],
        ));
    }
    builder.add_program(
        ProgramDefinition::new(id(4), vec![], vec![], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
                code: 704,
                value: None,
            }),
        ]),
    );
    builder.add_program(ProgramDefinition::new(
        id(5),
        vec![],
        vec![],
        vec![],
        vec![],
    ));
    let effect_runtime = EffectRuntimeDefinition::new(
        EffectCategory::Debuff,
        DispelCategory::DispellableDebuff,
        1,
        Some(3),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::TurnEnd,
        EffectStackPolicy::Refresh,
    )
    .unwrap();
    builder.add_effect(EffectDefinition::new(id(1), vec![], vec![]).with_runtime(effect_runtime));
    let effect = EffectApplicationDefinition::new(
        id::<EffectDefinitionId>(1),
        EffectChancePolicy::Guaranteed,
        1,
    )
    .unwrap();
    builder.add_ability(
        AbilityDefinition::new(id(1), id(1), id(1), vec![id(1)]).with_action(action(vec![
            HitOperationDefinition::Damage(damage(400)),
            HitOperationDefinition::ApplyEffect(effect),
            HitOperationDefinition::TransitionEnemyPhase(id(2)),
        ])),
    );
    builder.add_ability(
        AbilityDefinition::new(id(2), id(2), id(2), vec![]).with_action(action(vec![])),
    );
    builder.add_ability(
        AbilityDefinition::new(id(3), id(1), id(1), vec![])
            .with_action(action(vec![HitOperationDefinition::Damage(damage(1_000))])),
    );
    builder.add_unit(UnitDefinition::new(id(1), vec![id(1), id(3)], vec![]));
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
    let carry_all = EnemyPhaseCarry {
        hp: PhaseCarryPolicy::CarryExact,
        action_gauge: PhaseCarryPolicy::CarryExact,
        effects: PhaseCarryPolicy::CarryExact,
        toughness: PhaseCarryPolicy::CarryExact,
        summons: PhaseCarryPolicy::CarryExact,
    };
    let replacement = EnemyPhaseCarry {
        hp: PhaseCarryPolicy::Reset,
        action_gauge: PhaseCarryPolicy::Reset,
        effects: PhaseCarryPolicy::Clear,
        toughness: PhaseCarryPolicy::Clear,
        summons: PhaseCarryPolicy::ExplicitProgram(id(5)),
    };
    builder.add_enemy(
        EnemyDefinition::new(id(1), id(2), vec![id(2)])
            .with_orchestration(
                id(1),
                vec![phase(1, true, carry_all), phase(2, false, replacement)],
            )
            .unwrap(),
    );
    let wave = EncounterWaveDefinition::new(
        id::<EncounterWaveId>(1),
        1,
        None,
        None,
        WaveCarry::CARRY_ALL,
        vec![
            WaveSlotDefinition::new(
                1,
                FormationIndex::new(4).unwrap(),
                id(1),
                None,
                Some(id(1)),
                true,
            )
            .unwrap(),
        ],
    )
    .unwrap();
    builder.add_encounter(
        EncounterDefinition::new(id(1), vec![id(1)], vec![])
            .with_authored_waves(WaveTransitionPolicy::AfterAction, vec![wave])
            .unwrap(),
    );
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
    battle_with_player_ability(1)
}

fn battle_with_player_ability(ability: u32) -> Battle {
    let enemy = combatant(2, 2, 1_000_000, 0x22)
        .with_toughness(
            EnemyRank::EliteOrBoss,
            vec![CombatElement::Fire],
            vec![
                ToughnessLayerSpec::ordinary(1, starclock_combat::RawToughness::new(60).unwrap())
                    .unwrap(),
            ],
        )
        .unwrap();
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x31; 32]).unwrap(),
        id(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, ability, 2_000_000, 0x11),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(id(1)),
                enemy,
            ),
        ],
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog(), spec, BattleSeed::new([0x41; 32])).unwrap()
}

#[test]
fn lethal_damage_advances_a_multi_phase_enemy_without_defeat_events() {
    let mut battle = battle_with_player_ability(3);
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
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { ability, .. } if *ability == id(3)))
        .unwrap()
        .clone();
    let resolution = battle.apply(command).unwrap();
    assert_eq!(resolution.fault(), None);
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::EnemyPhase(EnemyPhaseEventData::Transitioned { from, to, .. })
            if *from == Some(id(1)) && *to == id(2)
    )));
    assert!(!resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Unit(
            starclock_combat::UnitEventData::Downed { .. }
                | starclock_combat::UnitEventData::Defeated { .. }
        )
    )));
    let enemy = battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Enemy)
        .unwrap();
    assert_eq!(enemy.enemy_phase(), Some(id(2)));
    assert_eq!(enemy.current_hp(), enemy.maximum_hp());
}

#[test]
fn phase_transition_is_transactional_and_applies_every_carry_family() {
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
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { ability, .. } if *ability == id(1)))
        .unwrap()
        .clone();
    let resolution = battle.apply(command).unwrap();
    assert_eq!(
        resolution.fault(),
        None,
        "events: {:?}",
        resolution.events()
    );
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::EnemyPhase(EnemyPhaseEventData::Transitioned { to, .. }) if *to == id(2)
    )));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::RuleSignal(starclock_combat::RuleSignalEventData { code: 704, .. })
    )));
    let enemy = battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Enemy)
        .unwrap();
    assert_eq!(enemy.enemy_phase(), Some(id(2)));
    assert_eq!(enemy.enemy_ai_state().unwrap().1, id(1));
    assert_eq!(enemy.current_hp(), enemy.maximum_hp());
    assert_eq!(enemy.presence(), PresenceState::Untargetable);
    assert!(enemy.weakness_broken());
    assert!(
        enemy
            .toughness_layers()
            .all(|layer| layer.current().get() == 0)
    );
    assert_eq!(battle.view().effects_by_id().count(), 0);
    let gauge = battle
        .view()
        .timeline_actors()
        .find(|actor| actor.owner() == enemy.id())
        .unwrap()
        .action_gauge();
    assert!(gauge.scaled() > 0);
    assert_eq!(battle.view().links().count(), 0);
}
