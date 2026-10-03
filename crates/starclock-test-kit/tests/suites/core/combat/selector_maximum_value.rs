//! Maximum-value ties execute through the ordinary accepted-command pipeline.

use crate::combat_decision::advance_boundary_if_offered;
use starclock_combat::{
    AssemblyDigest, Battle, BattleEventKind, BattleSeed, BattleSpec, CombatantSpecDigest, Command,
    ConcedePolicy, DecisionId, Energy, FaultPolicy, FormationIndex, Hp, LifeState,
    ParticipantInitialState, ParticipantSource, ParticipantSpec, PresenceState, Resolution,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, Rounding, Scalar, Speed, TeamResourceSpec,
    TeamSide, UnitId, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            ActionResourcePolicy, TargetInvalidationPolicy, TargetPattern, TargetRelation,
            UnitTargetSelector,
        },
        builder::{CatalogBuildErrorKind, CombatCatalogBuilder},
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition, RuleBundle,
            RuleDefinition, SelectorDefinition, UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, EventValueProperty, OnceScope,
        ProgramStep, ReactionPriority, RuleEventPoint, RuleOperationTemplate, RuleSource,
        RuleValue, RuleValueKind, SourceClass, TriggerDef, TriggerPhase, ValueExpr,
    },
};
use std::sync::Arc;

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn hp() -> ValueExpr {
    ValueExpr::QueryHp {
        subject: StatQuerySubject::CurrentTarget,
    }
}

fn scalar(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(
        Scalar::checked_from_integer(value).unwrap(),
    ))
}

fn selector(
    predicates: Vec<RuleSelectorPredicate>,
    choice: RuleSelectorChoice,
) -> RuleUnitSelector {
    RuleUnitSelector::new(
        RuleSelectorOrigin::Encounter,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StableId,
        0,
        if choice == RuleSelectorChoice::All {
            3
        } else {
            1
        },
        RuleEmptyPoolPolicy::NoOp,
        choice,
        (choice == RuleSelectorChoice::RngUniform).then(|| "damage-target".into()),
        false,
    )
    .unwrap()
    .with_predicates(predicates)
}

fn catalog(
    plan: RuleUnitSelector,
    point: RuleEventPoint,
) -> Result<Arc<CombatCatalog>, CatalogBuildErrorKind> {
    let mut builder = CombatCatalogBuilder::new([0xe1; 32]);
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
                0,
                1,
                RuleEmptyPoolPolicy::NoOp,
                RuleSelectorChoice::First,
                None,
                false,
            )
            .unwrap(),
        ),
    );
    builder.add_selector(SelectorDefinition::new(id(11)).with_rule_units(plan));
    builder.add_program(ProgramDefinition::new(
        id(1),
        vec![],
        vec![],
        vec![],
        vec![],
    ));
    builder.add_program(
        ProgramDefinition::new(id(12), vec![], vec![id(2)], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::TrueDamage {
                selector: id(2),
                amount: scalar(1),
            }),
        ]),
    );
    builder.add_program(
        ProgramDefinition::new(id(20), vec![], vec![id(11)], vec![], vec![]).with_steps(vec![
            ProgramStep::ForEach {
                selector: id(11),
                body: id(21),
                maximum: 3,
            },
            ProgramStep::Operation(RuleOperationTemplate::TrueDamage {
                selector: id(11),
                amount: scalar(1),
            }),
        ]),
    );
    builder.add_program(
        ProgramDefinition::new(id(21), vec![], vec![], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
                code: 811,
                value: Some(ValueExpr::CurrentTarget),
            }),
        ]),
    );
    builder.add_rule(
        RuleDefinition::new(id(1), vec![id(20), id(21)], vec![id(11)]).with_runtime(
            BattleRuleDefinition::new(
                RuleSource::new(id(81), SourceClass::Synthetic, vec![], [0xe2; 32]),
                vec![],
                vec![TriggerDef {
                    id: id(1),
                    event: point.kind(),
                    event_point: point,
                    phase: TriggerPhase::AfterEvent,
                    filter: EventFilter::default(),
                    condition: ConditionExpr::Literal(true),
                    once_scope: OnceScope::Event,
                    priority: ReactionPriority::new(0),
                    program: id(20),
                }],
                None,
            ),
        ),
    );
    builder.add_rule_bundle(RuleBundle::new(id(1), vec![id(1)]));
    let action = AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap();
    builder.add_ability(
        AbilityDefinition::new(id(1), id(1), id(1), vec![])
            .with_action(action.clone())
            .with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, id(12)).unwrap(),
            ]),
    );
    builder.add_ability(AbilityDefinition::new(id(2), id(1), id(1), vec![]).with_action(action));
    builder.add_unit(UnitDefinition::new(id(1), vec![id(1)], vec![id(1)]));
    builder.add_unit(UnitDefinition::new(id(2), vec![id(2)], vec![]));
    builder.add_enemy(EnemyDefinition::new(id(1), id(2), vec![id(2)]));
    builder.add_encounter(EncounterDefinition::new(id(1), vec![id(1)], vec![]));
    builder.build().map_err(|error| error.kind())
}

fn battle(plan: RuleUnitSelector, point: RuleEventPoint, seed: u8, current: [i64; 3]) -> Battle {
    let mut participants = Vec::new();
    for raw in 1..=4_u32 {
        let player = raw == 1;
        let maximum = Hp::new(if player {
            1_000
        } else {
            i64::from(raw - 1) * 1_000
        })
        .unwrap();
        let combatant = ResolvedCombatantSpec::new(
            id(if player { 1 } else { 2 }),
            UnitLevel::new(80).unwrap(),
            maximum,
            Speed::from_scaled(if player { 100_000_000 } else { 1_000_000 }).unwrap(),
            ResolvedDefinitionBindings::new(
                vec![id(if player { 1 } else { 2 })],
                if player { vec![id(1)] } else { vec![] },
                vec![],
            )
            .unwrap(),
            CombatantSpecDigest::new([u8::try_from(raw).unwrap(); 32]).unwrap(),
        )
        .unwrap();
        let participant = ParticipantSpec::new(
            if player {
                TeamSide::Player
            } else {
                TeamSide::Enemy
            },
            FormationIndex::new(u8::try_from(if player { 0 } else { raw - 2 }).unwrap()).unwrap(),
            if player {
                ParticipantSource::Player
            } else {
                ParticipantSource::EncounterEnemy(id(1))
            },
            combatant,
        )
        .with_initial_state(
            ParticipantInitialState::new(
                if player {
                    maximum
                } else {
                    Hp::new(current[usize::try_from(raw - 2).unwrap()]).unwrap()
                },
                maximum,
                Energy::ZERO,
                Energy::ZERO,
                LifeState::Alive,
                PresenceState::Present,
            )
            .unwrap(),
        )
        .unwrap();
        participants.push(participant);
    }
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xe3; 32]).unwrap(),
        id(1),
        participants,
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(
        catalog(plan, point).unwrap(),
        spec,
        BattleSeed::new([seed; 32]),
    )
    .unwrap()
}

fn start(battle: &mut Battle) -> Resolution {
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap()
}

fn selected(resolution: &Resolution) -> Vec<u64> {
    resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::RuleSignal(signal) if signal.code == 811 => match signal.value {
                Some(RuleValue::OptionalStableId(Some(unit))) => Some(unit),
                _ => panic!("unit signal"),
            },
            _ => None,
        })
        .collect()
}

#[test]
fn maximum_value_keeps_all_exact_current_hp_ties_not_maximum_hp_or_hp_ratio() {
    let plan = selector(
        vec![RuleSelectorPredicate::MaximumValue(hp())],
        RuleSelectorChoice::All,
    );
    let mut first = battle(
        plan.clone(),
        RuleEventPoint::BattleStarted,
        1,
        [100, 900, 900],
    );
    let mut fresh = battle(plan, RuleEventPoint::BattleStarted, 1, [100, 900, 900]);
    let result = start(&mut first);
    let replay = start(&mut fresh);
    assert!(result.fault().is_none());
    assert_eq!(selected(&result), [3, 4]);
    assert_eq!(result.events(), replay.events());
    assert_eq!(first.state_hash(), fresh.state_hash());
    assert_eq!(first.view().rng_draw_count(), 0);
    for raw in [3, 4] {
        assert!(result.events().iter().any(|event| matches!(event.kind(),
            BattleEventKind::Damage(data) if data.target == UnitId::new(raw).unwrap() && data.applied.get() == 1)));
    }
}

#[test]
fn maximum_value_predicate_order_and_negative_numeric_keys_are_exact() {
    let range = RuleSelectorPredicate::FormationRange {
        minimum: 2,
        maximum: 2,
    };
    for (predicates, expected) in [
        (
            vec![range.clone(), RuleSelectorPredicate::MaximumValue(hp())],
            vec![4],
        ),
        (
            vec![RuleSelectorPredicate::MaximumValue(hp()), range],
            vec![],
        ),
        (
            vec![RuleSelectorPredicate::MaximumValue(ValueExpr::Negate(
                Box::new(hp()),
            ))],
            vec![2],
        ),
    ] {
        let mut battle = battle(
            selector(predicates, RuleSelectorChoice::All),
            RuleEventPoint::BattleStarted,
            1,
            [100, 900, 800],
        );
        let result = start(&mut battle);
        assert!(result.fault().is_none());
        assert_eq!(selected(&result), expected);
        assert_eq!(battle.view().rng_draw_count(), 0);
    }
}

#[test]
fn maximum_value_uniform_choice_draws_only_from_ties_and_rejects_inertly() {
    let plan = selector(
        vec![RuleSelectorPredicate::MaximumValue(hp())],
        RuleSelectorChoice::RngUniform,
    );
    let mut vectors = Vec::new();
    for seed in 1..=8 {
        let mut first = battle(
            plan.clone(),
            RuleEventPoint::BattleStarted,
            seed,
            [100, 900, 900],
        );
        let mut fresh = battle(
            plan.clone(),
            RuleEventPoint::BattleStarted,
            seed,
            [100, 900, 900],
        );
        let before = first.state_hash();
        assert!(
            first
                .apply(Command::StartBattle {
                    decision: DecisionId::new(999).unwrap()
                })
                .is_err()
        );
        assert_eq!(first.state_hash(), before);
        assert_eq!(first.view().rng_draw_count(), 0);
        let result = start(&mut first);
        let replay = start(&mut fresh);
        assert!(result.fault().is_none());
        let selected = selected(&result);
        assert_eq!(selected.len(), 1);
        assert!([3, 4].contains(&selected[0]));
        assert_eq!(first.view().rng_draw_count(), 1);
        assert_eq!(result.events(), replay.events());
        assert_eq!(first.state_hash(), fresh.state_hash());
        vectors.push(selected[0]);
    }
    assert_eq!(vectors, [4, 3, 4, 3, 3, 3, 3, 3]);
    let mut singleton = battle(plan, RuleEventPoint::BattleStarted, 1, [100, 900, 800]);
    assert_eq!(selected(&start(&mut singleton)), [3]);
    assert_eq!(singleton.view().rng_draw_count(), 1);
}

#[test]
fn maximum_value_integer_and_fractional_keys_do_not_round_into_false_ties() {
    for expression in [
        ValueExpr::Convert {
            value: Box::new(hp()),
            target: RuleValueKind::Integer,
            rounding: Rounding::Floor,
        },
        ValueExpr::Divide {
            lhs: Box::new(hp()),
            rhs: Box::new(scalar(1_000)),
            rounding: Rounding::Floor,
        },
    ] {
        let mut battle = battle(
            selector(
                vec![RuleSelectorPredicate::MaximumValue(expression)],
                RuleSelectorChoice::All,
            ),
            RuleEventPoint::BattleStarted,
            1,
            [100, 900, 901],
        );
        let result = start(&mut battle);
        assert!(result.fault().is_none());
        assert_eq!(selected(&result), [4]);
    }
}

#[test]
fn maximum_value_empty_pool_does_not_evaluate_or_draw_and_missing_nonempty_read_faults() {
    let missing = RuleSelectorPredicate::MaximumValue(ValueExpr::ReadEventProperty(
        EventValueProperty::DamageOverflow,
    ));
    let plan = selector(
        vec![
            RuleSelectorPredicate::FormationRange {
                minimum: 4,
                maximum: 4,
            },
            missing.clone(),
        ],
        RuleSelectorChoice::RngUniform,
    );
    let mut empty = battle(plan, RuleEventPoint::BattleStarted, 1, [100, 900, 900]);
    let result = start(&mut empty);
    assert!(result.fault().is_none());
    assert!(selected(&result).is_empty());
    assert_eq!(empty.view().rng_draw_count(), 0);
    let mut missing = battle(
        selector(vec![missing], RuleSelectorChoice::RngUniform),
        RuleEventPoint::BattleStarted,
        1,
        [100, 900, 900],
    );
    let result = start(&mut missing);
    assert_eq!(result.fault().unwrap().context_code(), 0x3198);
    assert_eq!(result.fault().unwrap().policy(), FaultPolicy::Rollback);
    assert!(selected(&result).is_empty());
    assert_eq!(missing.view().rng_draw_count(), 0);
    assert!(
        !result
            .events()
            .iter()
            .any(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
    );
}

#[test]
fn maximum_value_current_hp_reselects_after_a_real_action() {
    let plan = selector(
        vec![RuleSelectorPredicate::MaximumValue(hp())],
        RuleSelectorChoice::All,
    );
    let mut battle = battle(plan, RuleEventPoint::ActionResolved, 1, [900, 900, 800]);
    assert!(selected(&start(&mut battle)).is_empty());
    advance_boundary_if_offered(&mut battle);
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(command,
        Command::UseAbility { primary_target: Some(target), .. } if target.get() == 2)
        })
        .unwrap()
        .clone();
    let result = battle.apply(command).unwrap();
    assert!(result.fault().is_none());
    assert_eq!(selected(&result), [3]);
}

#[test]
fn maximum_value_catalog_rejects_non_numeric_missing_cyclic_and_unsafe_historical_inputs() {
    for value in [
        ValueExpr::Literal(RuleValue::Boolean(true)),
        ValueExpr::SelectorCount(id(99)),
        ValueExpr::SelectorCount(id(11)),
    ] {
        assert!(
            catalog(
                selector(
                    vec![RuleSelectorPredicate::MaximumValue(value)],
                    RuleSelectorChoice::All
                ),
                RuleEventPoint::BattleStarted
            )
            .is_err()
        );
    }
    for reference in [
        RuleSelectorReference::EventSnapshot,
        RuleSelectorReference::ActionSnapshot,
    ] {
        let mut plan = selector(
            vec![RuleSelectorPredicate::MaximumValue(hp())],
            RuleSelectorChoice::All,
        );
        // Reconstruct the plan through its public constructor, preserving predicates.
        plan = RuleUnitSelector::new(
            plan.origin(),
            plan.side(),
            plan.life(),
            plan.presence(),
            reference,
            plan.ordering(),
            plan.minimum(),
            plan.maximum(),
            plan.empty_pool(),
            plan.choice(),
            None,
            false,
        )
        .unwrap()
        .with_predicates(plan.predicates().to_vec());
        assert_eq!(
            catalog(plan, RuleEventPoint::ActionResolved).unwrap_err(),
            CatalogBuildErrorKind::InvalidDefinition
        );
    }
    let value = ValueExpr::QueryStat {
        subject: StatQuerySubject::CurrentTarget,
        stat: StatKind::Hp,
        purpose: FormulaPurpose::Stat,
    };
    let plan = RuleUnitSelector::new(
        RuleSelectorOrigin::Encounter,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::ActionSnapshot,
        RuleSelectorOrdering::StableId,
        0,
        3,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::All,
        None,
        false,
    )
    .unwrap()
    .with_predicates(vec![RuleSelectorPredicate::MaximumValue(value)]);
    let mut battle = battle(plan, RuleEventPoint::ActionResolved, 1, [900, 900, 800]);
    start(&mut battle);
    advance_boundary_if_offered(&mut battle);
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { .. }))
        .unwrap()
        .clone();
    let result = battle.apply(command).unwrap();
    assert!(result.fault().is_none());
    assert_eq!(selected(&result), [4]);
}
