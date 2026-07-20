use std::sync::Arc;

use starclock_combat::{
    Battle, BattleEventKind, BattleSeed, BattleSpec, BattleSpecDigest, CombatantSpecDigest,
    Command, ConcedePolicy, EncounterWaveId, FormationIndex, Hp, ParticipantSource,
    ParticipantSpec, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    ResolvedModifierBinding, Scalar, SourceDefinitionId, Speed, StatValue, TeamResourceSpec,
    TeamSide, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            ActionHitDefinition, ActionResourcePolicy, HitTargetGroup, TargetInvalidationPolicy,
            TargetPattern, TargetRelation, UnitTargetSelector,
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
    formula::model::{CombatElement, DamageClass},
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind, StatQuerySubject,
    },
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleEventKind, RuleOperationTemplate, RuleSource, RuleValue, SourceClass, TriggerDef,
        TriggerPhase, ValueExpr,
    },
};

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn empty_action() -> AbilityActionDefinition {
    AbilityActionDefinition::new(
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
}

fn catalog(
    program: ProgramDefinition,
    with_modifier: bool,
    with_rule: bool,
    recursive_rule: bool,
) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new("ability-program-v1", [0x43; 32]);
    builder.add_selector(SelectorDefinition::new(id(1)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_selector(
        SelectorDefinition::new(id(2)).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::PrimaryTarget,
                RuleSelectorSide::Opposing,
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
    builder.add_selector(SelectorDefinition::new(id(3)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_program(program);
    if with_rule {
        builder.add_program(
            ProgramDefinition::new(id(3), vec![], vec![id(2)], vec![], vec![]).with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::Damage {
                    selector: id(2),
                    amount: ValueExpr::Literal(RuleValue::Scalar(
                        Scalar::checked_from_integer(if recursive_rule { 0 } else { 50 }).unwrap(),
                    )),
                    class: DamageClass::Additional,
                    element: CombatElement::Physical,
                    can_crit: false,
                }),
            ]),
        );
        let source = RuleSource::new(
            SourceDefinitionId::new(60).unwrap(),
            SourceClass::Progression,
            vec![],
            [0x60; 32],
        );
        builder.add_rule(
            RuleDefinition::new(id(1), vec![id(3)], vec![id(2)]).with_runtime(
                BattleRuleDefinition::new(
                    source,
                    vec![],
                    vec![TriggerDef {
                        id: id(1),
                        event: if recursive_rule {
                            RuleEventKind::Damage
                        } else {
                            RuleEventKind::Hit
                        },
                        phase: TriggerPhase::AfterEvent,
                        filter: EventFilter::default(),
                        condition: ConditionExpr::Literal(true),
                        once_scope: if recursive_rule {
                            OnceScope::Event
                        } else {
                            OnceScope::Action
                        },
                        priority: ReactionPriority::new(0),
                        program: id(3),
                    }],
                    None,
                ),
            ),
        );
        builder.add_rule_bundle(RuleBundle::new(id(1), vec![id(1)]));
    }
    if with_modifier {
        builder.add_modifier_group(ModifierStackingGroup {
            id: id(1),
            aggregation: ModifierAggregation::Sum,
        });
        builder.add_modifier(ModifierDefinition {
            id: id(1),
            stat: StatKind::Atk,
            stage: FormulaStage::Flat,
            purpose: FormulaPurpose::Stat,
            value: ValueExpr::Literal(RuleValue::Scalar(
                Scalar::checked_from_integer(200).unwrap(),
            )),
            stacking_group: id(1),
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::Flat,
            snapshot: SnapshotPolicy::Dynamic,
            filters: Box::new([]),
        });
    }
    builder.add_program(ProgramDefinition::new(
        id(2),
        vec![],
        vec![],
        vec![],
        vec![],
    ));
    let hits = vec![
        ActionHitDefinition::new(vec![]).with_profile(
            HitTargetGroup::Primary,
            Ratio::from_scaled(250_000),
            Ratio::ONE,
            starclock_combat::catalog::action::HitCritPolicy::Never,
        ),
        ActionHitDefinition::new(vec![]).with_profile(
            HitTargetGroup::Primary,
            Ratio::from_scaled(750_000),
            Ratio::ONE,
            starclock_combat::catalog::action::HitCritPolicy::Never,
        ),
    ];
    let action = AbilityActionDefinition::new(
        AbilityKind::Basic,
        2,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(
            0,
            0,
            starclock_combat::Energy::ZERO,
            starclock_combat::Energy::ZERO,
        ),
    )
    .unwrap()
    .with_hits(hits)
    .unwrap();
    builder.add_ability(
        AbilityDefinition::new(id(1), id(1), id(1), vec![])
            .with_action(action)
            .with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, id(1)).unwrap(),
            ]),
    );
    builder.add_ability(
        AbilityDefinition::new(id(2), id(2), id(3), vec![]).with_action(empty_action()),
    );
    builder.add_unit(UnitDefinition::new(
        id(1),
        vec![id(1)],
        with_rule.then(|| id(1)).into_iter().collect(),
    ));
    builder.add_unit(UnitDefinition::new(id(2), vec![id(2)], vec![]));
    builder.add_enemy(EnemyDefinition::new(id(1), id(2), vec![id(2)]));
    builder.add_encounter(
        EncounterDefinition::new(id(1), vec![id(1)], vec![])
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

fn combatant(
    form: u32,
    ability: u32,
    digest: u8,
    with_modifier: bool,
    with_rule: bool,
) -> ResolvedCombatantSpec {
    let modifiers = with_modifier.then(|| id(1)).into_iter().collect();
    let mut combatant = ResolvedCombatantSpec::new(
        id(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(if form == 1 { 100_000_000 } else { 1_000_000 }).unwrap(),
        ResolvedDefinitionBindings::new(
            vec![id(ability)],
            with_rule.then(|| id(1)).into_iter().collect(),
            modifiers,
        )
        .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
    .with_base_attack_defense(
        StatValue::from_scaled(200_000_000).unwrap(),
        StatValue::from_scaled(100_000_000).unwrap(),
    );
    if with_modifier {
        let source = SourceDefinitionId::new(50).unwrap();
        combatant = combatant
            .with_sources(vec![RuleSource::new(
                source,
                SourceClass::Progression,
                vec![],
                [0x50; 32],
            )])
            .unwrap()
            .with_modifier_bindings(vec![ResolvedModifierBinding::new(id(1), source)])
            .unwrap();
    }
    combatant
}

fn battle(catalog: Arc<CombatCatalog>, with_modifier: bool, with_rule: bool) -> Battle {
    let spec = BattleSpec::new(
        "ability-program-rules-v1",
        BattleSpecDigest::new([0x44; 32]).unwrap(),
        id(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 1, 0x45, with_modifier, with_rule),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(id(1)),
                combatant(2, 2, 0x46, false, false),
            ),
        ],
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog, spec, BattleSeed::new([0x47; 32])).unwrap()
}

fn start_and_use(
    battle: &mut Battle,
) -> Result<starclock_combat::Resolution, starclock_combat::CommandError> {
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
    let use_ability = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(
            |command| matches!(command, Command::UseAbility { ability, .. } if ability.get() == 1),
        )
        .unwrap()
        .clone();
    battle.apply(use_ability)
}

#[test]
fn hit_programs_use_authored_selector_order_and_exact_hit_shares() {
    let program =
        ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::Damage {
                selector: id(2),
                amount: ValueExpr::Literal(RuleValue::Scalar(
                    Scalar::checked_from_integer(200).unwrap(),
                )),
                class: DamageClass::Direct,
                element: CombatElement::Physical,
                can_crit: false,
            }),
        ]);
    let mut battle = battle(catalog(program, false, false, false), false, false);
    let resolution = start_and_use(&mut battle).unwrap();
    let damage = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(value) => Some(value.applied.get()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(damage, [50, 150]);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .nth(1)
            .unwrap()
            .current_hp()
            .get(),
        800
    );
}

#[test]
fn selected_build_modifier_changes_rule_ir_stat_query_inside_transaction() {
    let program =
        ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::Damage {
                selector: id(2),
                amount: ValueExpr::QueryStat {
                    subject: StatQuerySubject::Actor,
                    stat: StatKind::Atk,
                    purpose: FormulaPurpose::Stat,
                },
                class: DamageClass::Direct,
                element: CombatElement::Physical,
                can_crit: false,
            }),
        ]);
    let mut battle = battle(catalog(program, true, false, false), true, false);
    let resolution = start_and_use(&mut battle).unwrap();
    let damage = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(value) => Some(value.applied.get()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(damage, [100, 300]);
    assert!(resolution.fault().is_none());
    let actor = battle.view().units_by_id().next().unwrap();
    assert_eq!(actor.base_attack().scaled(), 200_000_000);
    assert_eq!(actor.base_defense().scaled(), 100_000_000);
    assert_eq!(actor.base_speed().scaled(), 100_000_000);
    let modifier = battle.view().modifier_instances_by_id().next().unwrap();
    assert_eq!(modifier.id().get(), 1);
    assert_eq!(modifier.definition().get(), 1);
    assert_eq!(modifier.owner(), modifier.subject());
    assert_eq!(modifier.source().get(), 50);
    assert_eq!(modifier.source_class(), SourceClass::Progression);
}

#[test]
fn selected_rule_bundle_dispatches_once_after_the_authored_hit_event() {
    let program = ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![], vec![]);
    let mut battle = battle(catalog(program, false, true, false), false, true);
    let resolution = start_and_use(&mut battle).unwrap();
    let damage = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(value) => Some(value.applied.get()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(damage, [50]);
    assert_eq!(battle.view().rule_instances_by_id().count(), 1);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .nth(1)
            .unwrap()
            .current_hp()
            .get(),
        950
    );
}

#[test]
fn recursively_emitting_rule_faults_at_the_dispatch_budget_and_rolls_back() {
    let program =
        ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::Damage {
                selector: id(2),
                amount: ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO)),
                class: DamageClass::Direct,
                element: CombatElement::Physical,
                can_crit: false,
            }),
        ]);
    let mut battle = battle(catalog(program, false, true, true), false, true);
    let resolution = start_and_use(&mut battle).unwrap();

    assert!(resolution.fault().is_some());
    assert_eq!(
        battle.view().phase(),
        starclock_combat::BattlePhase::Faulted
    );
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .nth(1)
            .unwrap()
            .current_hp()
            .get(),
        1_000
    );
}

#[test]
fn unsupported_program_emission_rolls_back_the_whole_command() {
    let program =
        ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::TrueDamage {
                selector: id(2),
                amount: ValueExpr::Literal(RuleValue::Scalar(
                    Scalar::checked_from_integer(200).unwrap(),
                )),
            }),
        ]);
    let mut battle = battle(catalog(program, false, false, false), false, false);
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
    let before_revision = battle.view().committed_revision();
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(
            |command| matches!(command, Command::UseAbility { ability, .. } if ability.get() == 1),
        )
        .unwrap()
        .clone();
    let resolution = battle.apply(command).unwrap();
    assert!(resolution.fault().is_some());
    assert_eq!(
        battle.view().phase(),
        starclock_combat::BattlePhase::Faulted
    );
    assert_eq!(battle.view().committed_revision(), before_revision + 1);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .nth(1)
            .unwrap()
            .current_hp()
            .get(),
        1_000
    );
}
