use starclock_combat::{
    ProgramId, SelectorId, SourceDefinitionId, StateSlotDefinitionId, UnitId,
    catalog::{
        action::AbilityTag,
        builder::{CatalogBuildErrorKind, CombatCatalogBuilder},
        definition::{
            AbilityDefinition, AbilityParameterDefinition, ProgramDefinition, RuleDefinition,
            SelectorDefinition,
        },
    },
    rule::{
        evaluate::{
            EvaluationBudget, ResourceQueryReader, RuleEvaluationErrorKind, TriggerLedger,
            evaluate_condition, evaluate_program, evaluate_value, matches_filter,
        },
        model::{
            BattleRuleDefinition, BattleRuleScope, CauseAncestry, Comparison, ConditionExpr,
            EventFilter, EventValueProperty, OnceScope, ProgramStep, ReactionPriority,
            RuleActionKind, RuleCause, RuleEmission, RuleEvaluationInput, RuleEventFacts,
            RuleEventKind, RuleEventPoint, RuleOccurrence, RuleOperationTemplate, RuleResourceKind,
            RuleSource, RuleValue, RuleValueKind, SelectorResult, SourceClass, StateSlotDef,
            TriggerDef, TriggerPhase, ValueExpr, once_key,
        },
    },
};

fn definition<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn runtime<I: TryFrom<u64>>(raw: u64) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn source(raw: u32) -> RuleSource {
    RuleSource::new(
        definition(raw),
        SourceClass::Synthetic,
        vec![definition::<SourceDefinitionId>(100 + raw)],
        [u8::try_from(raw).unwrap(); 32],
    )
}

fn trigger(id: u32, program: ProgramId) -> TriggerDef {
    TriggerDef {
        id: definition(id),
        event: RuleEventKind::Action,
        event_point: RuleEventPoint::ActionResolved,
        phase: TriggerPhase::AfterEvent,
        filter: EventFilter::default(),
        condition: ConditionExpr::Literal(true),
        once_scope: OnceScope::Action,
        priority: ReactionPriority::new(0),
        program,
    }
}

fn input<'a>(
    units: &'a [UnitId],
    selector: SelectorId,
    slots: &'a [(StateSlotDefinitionId, RuleValue)],
) -> RuleEvaluationInput<'a> {
    let selectors = Box::leak(vec![SelectorResult { selector, units }].into_boxed_slice());
    let event_facts = Box::leak(Box::new(RuleEventFacts {
        point: Some(RuleEventPoint::ActionResolved),
        ..RuleEventFacts::default()
    }));
    RuleEvaluationInput {
        event_kind: RuleEventKind::Action,
        event_facts,
        cause: RuleCause {
            owner: Some(runtime(1)),
            actor: Some(runtime(2)),
            applier: Some(runtime(3)),
            target: Some(runtime(4)),
            source: Some(definition(1)),
        },
        occurrence: occurrence(),
        source_tags: &[],
        slots,
        selectors,
        stat_reader: None,
        ability_parameter_reader: None,
        resource_reader: None,
        battle_query_reader: None,
    }
}

fn occurrence() -> RuleOccurrence {
    RuleOccurrence {
        rule_instance: runtime(1),
        event: runtime(10),
        hit: Some(runtime(11)),
        target: Some(runtime(12)),
        ability: Some(definition(13)),
        action: Some(runtime(14)),
        turn_event: Some(runtime(15)),
        wave: runtime(16),
    }
}

#[test]
fn ability_parameter_leaf_reads_the_exact_resolved_ability_and_fails_closed() {
    let ability = definition(13);
    let program = definition(1);
    let selector = definition(1);
    let mut builder = CombatCatalogBuilder::new("ability-parameter-v1", [0x28; 32]);
    builder.add_selector(SelectorDefinition::new(selector));
    builder.add_program(ProgramDefinition::new(
        program,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ));
    builder.add_ability(AbilityDefinition::new(
        ability,
        program,
        selector,
        Vec::new(),
    ));
    builder.add_ability_parameter(
        AbilityParameterDefinition::new(ability, "parameter.01", RuleValue::Integer(37)).unwrap(),
    );
    let catalog = builder.build().unwrap();
    let mut context = input(&[], selector, &[]);
    context.ability_parameter_reader = Some(&*catalog);

    assert_eq!(
        evaluate_value(
            &ValueExpr::AbilityParameter {
                key: "parameter.01".into(),
                kind: RuleValueKind::Integer,
            },
            context,
            None,
        )
        .unwrap(),
        RuleValue::Integer(37)
    );
    let error = evaluate_value(
        &ValueExpr::AbilityParameter {
            key: "parameter.missing".into(),
            kind: RuleValueKind::Integer,
        },
        context,
        None,
    )
    .unwrap_err();
    assert_eq!(error.kind(), RuleEvaluationErrorKind::MissingValue);
}

struct FixedSkillPoints;

impl ResourceQueryReader for FixedSkillPoints {
    fn query_resource(&self, subject: UnitId, resource: &RuleResourceKind) -> Option<RuleValue> {
        (subject == runtime(2) && resource == &RuleResourceKind::SkillPoints)
            .then_some(RuleValue::Integer(4))
    }
}

#[test]
fn exact_event_points_filters_and_observed_values_fail_closed() {
    let program = definition(1);
    let selector = definition(1);
    let mut builder = CombatCatalogBuilder::new("event-observation-v1", [0x29; 32]);
    builder.add_selector(SelectorDefinition::new(selector));
    builder.add_program(
        ProgramDefinition::new(program, vec![], vec![selector], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
                code: 29,
                value: Some(ValueExpr::ReadResource {
                    selector,
                    resource: RuleResourceKind::SkillPoints,
                }),
            }),
        ]),
    );
    let mut observed = trigger(1, program);
    observed.filter = EventFilter {
        actor_selector: Some(selector),
        action_kind: Some(RuleActionKind::Skill),
        ability_tag: Some(AbilityTag::Basic),
        resource: Some(RuleResourceKind::SkillPoints),
        cause_ancestry: CauseAncestry::SameAction,
        ..EventFilter::default()
    };
    builder.add_rule(
        RuleDefinition::new(definition(1), vec![program], vec![selector]).with_runtime(
            BattleRuleDefinition::new(source(1), vec![], vec![observed.clone()], None),
        ),
    );
    let catalog = builder.build().unwrap();
    let units = [runtime(2)];
    let facts = RuleEventFacts {
        point: Some(RuleEventPoint::ActionStarted),
        action_kind: Some(RuleActionKind::Skill),
        ability_tags: starclock_combat::catalog::action::AbilityTags::new(&[AbilityTag::Basic]),
        resource: Some(RuleResourceKind::SkillPoints),
        resource_delta: Some(starclock_combat::Scalar::from_scaled(-1_000_000)),
        has_action: true,
        ..RuleEventFacts::default()
    };
    let resources = FixedSkillPoints;
    let mut context = input(&units, selector, &[]);
    context.event_facts = &facts;
    context.resource_reader = Some(&resources);

    let mut ledger = TriggerLedger::default();
    assert!(
        ledger
            .evaluate(&*catalog, &observed, context, EvaluationBudget::STANDARD, 1,)
            .unwrap()
            .is_empty()
    );
    assert!(ledger.is_empty());

    observed.event_point = RuleEventPoint::ActionStarted;
    assert_eq!(
        ledger
            .evaluate(&*catalog, &observed, context, EvaluationBudget::STANDARD, 1,)
            .unwrap(),
        vec![RuleEmission::Informational {
            code: 29,
            value: Some(RuleValue::Integer(4)),
            current_target: None,
        }]
    );
    assert_eq!(
        evaluate_value(
            &ValueExpr::ReadEventProperty(EventValueProperty::ResourceDelta),
            context,
            None,
        )
        .unwrap(),
        RuleValue::Scalar(starclock_combat::Scalar::from_scaled(-1_000_000))
    );
}

#[test]
fn finite_program_evaluation_preserves_selector_order_and_faults_on_budget() {
    let root = definition(1);
    let body = definition(2);
    let selector = definition(1);
    let rule = definition(1);
    let mut builder = CombatCatalogBuilder::new("rule-ir-fixture-v1", [1; 32]);
    builder.add_selector(SelectorDefinition::new(selector));
    builder.add_program(
        ProgramDefinition::new(root, vec![], vec![selector], vec![], vec![]).with_steps(vec![
            ProgramStep::ForEach {
                selector,
                body,
                maximum: 4,
            },
        ]),
    );
    builder.add_program(
        ProgramDefinition::new(body, vec![], vec![], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
                code: 7,
                value: Some(ValueExpr::CurrentTarget),
            }),
        ]),
    );
    builder.add_rule(
        RuleDefinition::new(rule, vec![root, body], vec![selector]).with_runtime(
            BattleRuleDefinition::new(source(1), vec![], vec![trigger(1, root)], None),
        ),
    );
    let catalog = builder.build().unwrap();
    let units = [runtime(9), runtime(3)];
    let context = input(&units, selector, &[]);
    let emissions = evaluate_program(&*catalog, root, context, EvaluationBudget::STANDARD).unwrap();
    assert_eq!(
        emissions,
        vec![
            RuleEmission::Informational {
                code: 7,
                value: Some(RuleValue::OptionalStableId(Some(9))),
                current_target: Some(runtime(9)),
            },
            RuleEmission::Informational {
                code: 7,
                value: Some(RuleValue::OptionalStableId(Some(3))),
                current_target: Some(runtime(3)),
            },
        ]
    );
    let error = evaluate_program(
        &*catalog,
        root,
        context,
        EvaluationBudget {
            maximum_steps: 10,
            maximum_emissions: 10,
            maximum_iterations: 1,
        },
    )
    .unwrap_err();
    assert_eq!(error.kind(), RuleEvaluationErrorKind::BudgetExceeded);

    let trigger = &catalog.rule(rule).unwrap().runtime().unwrap().triggers()[0];
    let mut ledger = TriggerLedger::default();
    let first = ledger
        .evaluate(&*catalog, trigger, context, EvaluationBudget::STANDARD, 1)
        .unwrap();
    assert_eq!(first, emissions);
    assert!(
        ledger
            .evaluate(&*catalog, trigger, context, EvaluationBudget::STANDARD, 1,)
            .unwrap()
            .is_empty()
    );
    assert_eq!(ledger.len(), 1);

    let mut bounded = TriggerLedger::default();
    let error = bounded
        .evaluate(&*catalog, trigger, context, EvaluationBudget::STANDARD, 0)
        .unwrap_err();
    assert_eq!(error.kind(), RuleEvaluationErrorKind::BudgetExceeded);
    assert_eq!(bounded.len(), 0);
}

#[test]
fn catalog_rejects_mistyped_slots_and_mutating_replacement_programs() {
    let program = definition(1);
    let rule = definition(1);
    let slot = definition(1);
    let mut builder = CombatCatalogBuilder::new("invalid-rule-v1", [2; 32]);
    builder.add_program(
        ProgramDefinition::new(program, vec![], vec![], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                slot,
                value: ValueExpr::Literal(RuleValue::Integer(2)),
            }),
        ]),
    );
    let mut replacement = trigger(1, program);
    replacement.phase = TriggerPhase::Replace;
    builder.add_rule(
        RuleDefinition::new(rule, vec![program], vec![]).with_runtime(BattleRuleDefinition::new(
            source(1),
            vec![StateSlotDef::new(
                slot,
                RuleValueKind::Integer,
                BattleRuleScope::Battle,
                RuleValue::Integer(0),
            )],
            vec![replacement],
            None,
        )),
    );
    assert_eq!(
        builder.build().unwrap_err().kind(),
        CatalogBuildErrorKind::InvalidDefinition
    );
}

#[test]
fn catalog_rejects_unresolved_linked_and_countdown_emissions() {
    for (revision, operation, selectors) in [
        (
            "missing-linked-unit-v1",
            RuleOperationTemplate::Summon {
                owner_selector: definition(1),
                unit_definition: definition(9),
            },
            vec![definition(1)],
        ),
        (
            "missing-countdown-v1",
            RuleOperationTemplate::CreateCountdown { code: 9 },
            vec![],
        ),
    ] {
        let program = definition(1);
        let mut builder = CombatCatalogBuilder::new(revision, [0x26; 32]);
        builder.add_selector(SelectorDefinition::new(definition(1)));
        builder.add_program(
            ProgramDefinition::new(program, vec![], selectors, vec![], vec![])
                .with_steps(vec![ProgramStep::Operation(operation)]),
        );
        builder.add_rule(
            RuleDefinition::new(definition(1), vec![program], vec![]).with_runtime(
                BattleRuleDefinition::new(source(1), vec![], vec![trigger(1, program)], None),
            ),
        );

        assert_eq!(
            builder.build().unwrap_err().kind(),
            CatalogBuildErrorKind::MissingReference
        );
    }
}

#[test]
fn ability_owned_programs_reject_unresolved_lifecycle_emissions() {
    for (revision, operation, selectors) in [
        (
            "ability-missing-linked-unit-v1",
            RuleOperationTemplate::Summon {
                owner_selector: definition(1),
                unit_definition: definition(9),
            },
            vec![definition(1)],
        ),
        (
            "ability-missing-countdown-v1",
            RuleOperationTemplate::CreateCountdown { code: 9 },
            vec![],
        ),
    ] {
        let program = definition(1);
        let mut builder = CombatCatalogBuilder::new(revision, [0x27; 32]);
        builder.add_selector(SelectorDefinition::new(definition(1)));
        builder.add_program(
            ProgramDefinition::new(program, vec![], selectors, vec![], vec![])
                .with_steps(vec![ProgramStep::Operation(operation)]),
        );
        builder.add_ability(AbilityDefinition::new(
            definition(1),
            program,
            definition(1),
            vec![],
        ));

        assert_eq!(
            builder.build().unwrap_err().kind(),
            CatalogBuildErrorKind::MissingReference
        );
    }
}

#[test]
fn trigger_index_uses_phase_priority_source_rule_and_trigger_order() {
    let mut builder = CombatCatalogBuilder::new("trigger-index-v1", [3; 32]);
    for raw in 1..=3 {
        let program = definition(raw);
        builder.add_program(
            ProgramDefinition::new(program, vec![], vec![], vec![], vec![]).with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
                    code: raw,
                    value: None,
                }),
            ]),
        );
        let mut trigger = trigger(raw, program);
        trigger.priority = ReactionPriority::new(if raw == 3 { -1 } else { 0 });
        builder.add_rule(
            RuleDefinition::new(definition(raw), vec![program], vec![]).with_runtime(
                BattleRuleDefinition::new(source(4 - raw), vec![], vec![trigger], None),
            ),
        );
    }
    let catalog = builder.build().unwrap();
    assert_eq!(catalog.trigger_count(), 3);
    assert_eq!(
        catalog
            .trigger_ids(RuleEventKind::Action, TriggerPhase::AfterEvent)
            .map(|(rule, trigger)| (rule.get(), trigger.get()))
            .collect::<Vec<_>>(),
        vec![(3, 3), (2, 2), (1, 1)]
    );
}

#[test]
fn every_once_scope_builds_only_from_its_declared_identity() {
    let original = occurrence();
    for scope in [
        OnceScope::Event,
        OnceScope::Hit,
        OnceScope::TargetWithinHit,
        OnceScope::Ability,
        OnceScope::Action,
        OnceScope::Turn,
        OnceScope::Wave,
        OnceScope::Battle,
    ] {
        assert_eq!(
            once_key(definition(1), scope, original),
            once_key(definition(1), scope, original)
        );
    }
    assert_ne!(
        once_key(definition(1), OnceScope::TargetWithinHit, original),
        once_key(
            definition(1),
            OnceScope::TargetWithinHit,
            RuleOccurrence {
                target: Some(runtime(99)),
                ..original
            }
        )
    );
    assert_eq!(
        once_key(definition(1), OnceScope::Hit, original),
        once_key(
            definition(1),
            OnceScope::Hit,
            RuleOccurrence {
                target: Some(runtime(99)),
                ..original
            }
        )
    );
    assert_eq!(
        once_key(definition(1), OnceScope::Battle, original),
        once_key(
            definition(1),
            OnceScope::Battle,
            RuleOccurrence {
                event: runtime(99),
                hit: Some(runtime(98)),
                target: Some(runtime(97)),
                action: Some(runtime(96)),
                turn_event: Some(runtime(95)),
                wave: runtime(94),
                ..original
            }
        )
    );
}

#[test]
fn conditions_and_filters_keep_owner_actor_applier_target_distinct() {
    let selector = definition(1);
    let units = [runtime(1), runtime(2)];
    let context = input(&units, selector, &[]);
    assert!(matches_filter(
        &EventFilter {
            owner: Some(runtime(1)),
            actor: Some(runtime(2)),
            applier: Some(runtime(3)),
            target: Some(runtime(4)),
            source: Some(definition(1)),
            ..EventFilter::default()
        },
        context
    ));
    assert!(!matches_filter(
        &EventFilter {
            actor: Some(runtime(1)),
            ..EventFilter::default()
        },
        context
    ));
    assert!(
        evaluate_condition(
            &ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::SelectorCount(selector)),
                operator: Comparison::Equal,
                rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(2))),
            },
            context,
            None,
        )
        .unwrap()
    );
}

#[test]
fn missing_scope_identity_is_rejected_instead_of_coalesced() {
    let occurrence = RuleOccurrence {
        hit: None,
        target: None,
        ..occurrence()
    };
    assert_eq!(
        once_key(definition(1), OnceScope::TargetWithinHit, occurrence),
        None
    );
}
