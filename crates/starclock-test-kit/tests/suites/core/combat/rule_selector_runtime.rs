use std::sync::Arc;

use starclock_combat::{
    AssemblyDigest, Battle, BattleEventKind, BattleSeed, BattleSpec, CombatantSpecDigest, Command,
    ConcedePolicy, EncounterWaveId, FormationIndex, Hp, ParticipantSource, ParticipantSpec, Ratio,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, Scalar, SourceDefinitionId, Speed,
    TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HitOperationDefinition, OrdinaryDamageDefinition, OrdinaryDamageMultipliers,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition, RuleBundle,
            RuleDefinition, SelectorDefinition, UnitDefinition,
        },
        encounter::{EncounterWaveDefinition, WaveCarry, WaveSlotDefinition, WaveTransitionPolicy},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleEventKind, RuleEventPoint, RuleOperationTemplate, RuleSource, SourceClass, TriggerDef,
        TriggerPhase, ValueExpr,
    },
};

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn combatant(form: u32, ability: u32, digest: u8, with_rule: bool) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        id(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(if form == 1 { 100_000_000 } else { 1_000_000 }).unwrap(),
        ResolvedDefinitionBindings::new(
            vec![id(ability)],
            with_rule.then(|| id(1)).into_iter().collect(),
            Vec::new(),
        )
        .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
}

fn catalog(
    empty_policy: Option<RuleEmptyPoolPolicy>,
) -> Arc<starclock_combat::catalog::CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new([0x81; 32]);
    builder.add_selector(SelectorDefinition::new(id(1)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_selector(
        SelectorDefinition::new(id(2)).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::Encounter,
                RuleSelectorSide::Opposing,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Present,
                RuleSelectorReference::ActionSnapshot,
                RuleSelectorOrdering::EventOrder,
                0,
                4,
                RuleEmptyPoolPolicy::NoOp,
                RuleSelectorChoice::All,
                None,
                false,
            )
            .unwrap(),
        ),
    );
    if let Some(policy) = empty_policy {
        builder.add_selector(
            SelectorDefinition::new(id(5)).with_rule_units(
                RuleUnitSelector::new(
                    RuleSelectorOrigin::Encounter,
                    RuleSelectorSide::Opposing,
                    RuleLifePredicate::Downed,
                    RulePresencePredicate::Present,
                    RuleSelectorReference::CurrentState,
                    RuleSelectorOrdering::StableId,
                    1,
                    4,
                    policy,
                    RuleSelectorChoice::All,
                    None,
                    false,
                )
                .unwrap(),
            ),
        );
    }
    builder.add_selector(SelectorDefinition::new(id(3)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_selector(
        SelectorDefinition::new(id(4)).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::Encounter,
                RuleSelectorSide::Opposing,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Present,
                RuleSelectorReference::ActionSnapshot,
                RuleSelectorOrdering::StableId,
                1,
                3,
                RuleEmptyPoolPolicy::Fault,
                RuleSelectorChoice::RngWeighted,
                Some("behavior-choice".into()),
                true,
            )
            .unwrap()
            .with_weight(Some(ValueExpr::QueryStat {
                subject: StatQuerySubject::CurrentTarget,
                stat: StatKind::Hp,
                purpose: FormulaPurpose::Stat,
            })),
        ),
    );
    builder.add_program(ProgramDefinition::new(
        id(1),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ));
    builder.add_program(ProgramDefinition::new(
        id(2),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ));
    let (selectors, steps) = if empty_policy.is_some() {
        (
            vec![id(5)],
            vec![ProgramStep::Operation(
                RuleOperationTemplate::EmitRuleEvent {
                    code: 703,
                    value: Some(ValueExpr::Literal(
                        starclock_combat::rule::model::RuleValue::Integer(1),
                    )),
                },
            )],
        )
    } else {
        (
            vec![id(2), id(4)],
            vec![
                ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
                    code: 701,
                    value: Some(ValueExpr::SelectorCount(id(2))),
                }),
                ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
                    code: 702,
                    value: Some(ValueExpr::SelectorCount(id(4))),
                }),
            ],
        )
    };
    builder.add_program(
        ProgramDefinition::new(id(3), Vec::new(), selectors.clone(), Vec::new(), Vec::new())
            .with_steps(steps),
    );
    builder.add_rule(
        RuleDefinition::new(id(1), vec![id(3)], selectors).with_runtime(BattleRuleDefinition::new(
            RuleSource::new(
                SourceDefinitionId::new(70).unwrap(),
                SourceClass::Synthetic,
                Vec::new(),
                [0x70; 32],
            ),
            Vec::new(),
            vec![TriggerDef {
                id: id(1),
                event: RuleEventKind::Hit,
                event_point: RuleEventPoint::HitEnded,
                phase: TriggerPhase::AfterEvent,
                filter: EventFilter::default(),
                condition: ConditionExpr::Literal(true),
                once_scope: OnceScope::Action,
                priority: ReactionPriority::new(0),
                program: id(3),
            }],
            None,
        )),
    );
    let mut rules = vec![id(1)];
    if empty_policy == Some(RuleEmptyPoolPolicy::CancelRemaining) {
        builder.add_program(
            ProgramDefinition::new(id(4), Vec::new(), Vec::new(), Vec::new(), Vec::new())
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::EmitRuleEvent {
                        code: 704,
                        value: None,
                    },
                )]),
        );
        builder.add_rule(
            RuleDefinition::new(id(2), vec![id(4)], Vec::new()).with_runtime(
                BattleRuleDefinition::new(
                    RuleSource::new(
                        SourceDefinitionId::new(71).unwrap(),
                        SourceClass::Synthetic,
                        Vec::new(),
                        [0x71; 32],
                    ),
                    Vec::new(),
                    vec![TriggerDef {
                        id: id(2),
                        event: RuleEventKind::Hit,
                        event_point: RuleEventPoint::HitEnded,
                        phase: TriggerPhase::AfterEvent,
                        filter: EventFilter::default(),
                        condition: ConditionExpr::Literal(true),
                        once_scope: OnceScope::Action,
                        priority: ReactionPriority::new(1),
                        program: id(4),
                    }],
                    None,
                ),
            ),
        );
        rules.push(id(2));
    }
    builder.add_rule_bundle(RuleBundle::new(id(1), rules));
    let lethal = OrdinaryDamageDefinition::new(
        Scalar::checked_from_integer(2_000).unwrap(),
        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
    )
    .unwrap();
    let player_action = AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(
            0,
            0,
            starclock_combat::Energy::ZERO,
            starclock_combat::Energy::ZERO,
        ),
    )
    .unwrap()
    .with_hits(vec![ActionHitDefinition::new(vec![
        HitOperationDefinition::Damage(lethal),
    ])])
    .unwrap();
    let enemy_action = AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(
            0,
            0,
            starclock_combat::Energy::ZERO,
            starclock_combat::Energy::ZERO,
        ),
    )
    .unwrap();
    builder.add_ability(
        AbilityDefinition::new(id(1), id(1), id(1), Vec::new()).with_action(player_action),
    );
    builder.add_ability(
        AbilityDefinition::new(id(2), id(2), id(3), Vec::new()).with_action(enemy_action),
    );
    builder.add_unit(UnitDefinition::new(id(1), vec![id(1)], vec![id(1)]));
    builder.add_unit(UnitDefinition::new(id(2), vec![id(2)], Vec::new()));
    builder.add_enemy(EnemyDefinition::new(id(1), id(2), vec![id(2)]));
    builder.add_encounter(
        EncounterDefinition::new(id(1), vec![id(1)], Vec::new())
            .with_authored_waves(
                WaveTransitionPolicy::AfterAction,
                vec![
                    EncounterWaveDefinition::new(
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
                                None,
                                true,
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
    );
    builder.build().unwrap()
}

fn battle(empty_policy: Option<RuleEmptyPoolPolicy>) -> Battle {
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x82; 32]).unwrap(),
        id(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 1, 0x83, true),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(id(1)),
                combatant(2, 2, 0x84, false),
            ),
        ],
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog(empty_policy), spec, BattleSeed::new([0x85; 32])).unwrap()
}

#[test]
fn action_snapshot_selector_observes_pre_hit_life_after_lethal_damage() {
    let mut battle = battle(None);
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    crate::combat_decision::pass_interrupt_if_offered(&mut battle);
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { .. }))
        .unwrap()
        .clone();
    let resolution = battle.apply(command).unwrap();
    assert!(resolution.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::RuleSignal(signal)
                if signal.code == 701
                    && signal.value
                        == Some(starclock_combat::rule::model::RuleValue::Integer(1))
        )
    }));
    assert!(resolution.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::RuleSignal(signal)
                if signal.code == 702
                    && signal.value
                        == Some(starclock_combat::rule::model::RuleValue::Integer(3))
        )
    }));
    assert!(resolution.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Damage(damage)
                if damage.applied.get() == 1_000
        )
    }));
}

fn execute_player_action(policy: RuleEmptyPoolPolicy) -> starclock_combat::Resolution {
    let mut battle = battle(Some(policy));
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    crate::combat_decision::pass_interrupt_if_offered(&mut battle);
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { .. }))
        .unwrap()
        .clone();
    battle.apply(command).unwrap()
}

#[test]
fn empty_pool_policies_have_distinct_runtime_control_flow() {
    let no_op = execute_player_action(RuleEmptyPoolPolicy::NoOp);
    assert!(no_op.events().iter().any(|event| {
        matches!(event.kind(), BattleEventKind::RuleSignal(signal) if signal.code == 703)
    }));

    let skip = execute_player_action(RuleEmptyPoolPolicy::Skip);
    assert!(!skip.events().iter().any(|event| {
        matches!(event.kind(), BattleEventKind::RuleSignal(signal) if signal.code == 703)
    }));

    let cancel = execute_player_action(RuleEmptyPoolPolicy::CancelRemaining);
    assert!(!cancel.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::RuleSignal(signal) if matches!(signal.code, 703 | 704)
        )
    }));

    let fault = execute_player_action(RuleEmptyPoolPolicy::Fault);
    assert!(fault.fault().is_some());
    assert!(!fault.events().iter().any(|event| {
        matches!(event.kind(), BattleEventKind::RuleSignal(signal) if signal.code == 703)
    }));
}
