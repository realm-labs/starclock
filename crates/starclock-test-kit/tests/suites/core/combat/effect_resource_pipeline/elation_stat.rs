//! Native Elation queries reach real effects, snapshots and accepted damage.
//! The synthetic damage expressions here are not a released Elation operation.
use super::{action, catalog, combatant, definition};
use crate::combat_decision::advance_boundary_if_offered;
use starclock_combat::{
    AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattleSeed, BattleSpec, Command,
    ConcedePolicy, DecisionId, DispelCategory, DurationClock, EffectCategory, EffectEventData,
    EffectRuntimeDefinition, EffectStackPolicy, EffectTickPhase, FormationIndex, ParticipantSource,
    ParticipantSpec, ResolvedCombatantSpec, ResolvedDefinitionBindings, ResolvedModifierBinding,
    Rounding, Scalar, TeamResourceSpec, TeamSide,
    catalog::{
        CombatCatalog,
        action::{AbilityProgramBinding, AbilityProgramTiming},
        builder::CombatCatalogBuilder,
        definition::{AbilityDefinition, EffectDefinition, ProgramDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    formula::model::{CombatElement, DamageClass},
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind, StatQuerySubject,
    },
    rule::model::{
        Comparison, ProgramStep, RuleEffectChancePolicy, RuleOperationTemplate, RuleSource,
        RuleValue, SourceClass, ValueExpr,
    },
};
use std::sync::Arc;

fn literal(scaled: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(scaled)))
}
fn query() -> ValueExpr {
    ValueExpr::QueryStat {
        subject: StatQuerySubject::Actor,
        stat: StatKind::Elation,
        purpose: FormulaPurpose::ElationDamage,
    }
}
fn probe_damage(extra: ValueExpr) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::Damage {
        selector: definition(5),
        amount: ValueExpr::Multiply {
            lhs: Box::new(literal(100_000_000)),
            rhs: Box::new(ValueExpr::Add(
                Box::new(literal(1_000_000)),
                Box::new(extra),
            )),
            rounding: Rounding::NearestTiesEven,
        },
        class: DamageClass::Direct,
        element: CombatElement::Physical,
        can_crit: false,
        can_defeat: true,
    })
}
fn apply_effect(selector: u32, effect: u32) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector: definition(selector),
        effect: definition(effect),
        stacks: ValueExpr::Literal(RuleValue::Integer(1)),
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })
}

fn fixture(
    reference: RuleSelectorReference,
    snapshot: Option<SnapshotPolicy>,
) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::from_catalog(&catalog(), [0xea; 32]);
    for raw in [3, 4, 5] {
        builder.add_modifier_group(ModifierStackingGroup {
            id: definition(raw),
            aggregation: ModifierAggregation::Sum,
            comparator: None,
        });
        let (stat, stage, value, policy) = match raw {
            3 => (
                StatKind::Elation,
                FormulaStage::Flat,
                literal(500_000),
                SnapshotPolicy::Dynamic,
            ),
            4 => (
                StatKind::Atk,
                FormulaStage::DamageBoost,
                ValueExpr::QueryStat {
                    subject: StatQuerySubject::Owner,
                    stat: StatKind::Elation,
                    purpose: FormulaPurpose::Stat,
                },
                snapshot.unwrap_or(SnapshotPolicy::Dynamic),
            ),
            5 => (
                StatKind::Atk,
                FormulaStage::DamageBoost,
                literal(250_000),
                SnapshotPolicy::Dynamic,
            ),
            _ => unreachable!(),
        };
        builder.add_modifier(ModifierDefinition {
            id: definition(raw),
            stat,
            stage,
            purpose: if raw == 3 {
                FormulaPurpose::Stat
            } else {
                FormulaPurpose::OrdinaryDamage
            },
            value,
            stacking_group: definition(raw),
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: stage,
            snapshot: policy,
            source_stack_slot: None,
            filters: Box::default(),
        });
    }
    for raw in [3, 5] {
        builder.add_effect(
            EffectDefinition::new(definition(raw), vec![], vec![definition(raw)]).with_runtime(
                EffectRuntimeDefinition::new(
                    EffectCategory::Buff,
                    DispelCategory::DispellableBuff,
                    1,
                    Some(1),
                    DurationClock::TargetTurnEnd,
                    EffectTickPhase::None,
                    EffectStackPolicy::Replace,
                )
                .unwrap(),
            ),
        );
    }
    for (raw, origin, side, reference) in [
        (
            3,
            RuleSelectorOrigin::Owner,
            RuleSelectorSide::Same,
            RuleSelectorReference::CurrentState,
        ),
        (
            4,
            RuleSelectorOrigin::Owner,
            RuleSelectorSide::Same,
            reference,
        ),
        (
            5,
            RuleSelectorOrigin::EventTargets,
            RuleSelectorSide::Opposing,
            RuleSelectorReference::CurrentState,
        ),
    ] {
        let mut selector = RuleUnitSelector::new(
            origin,
            side,
            RuleLifePredicate::Alive,
            RulePresencePredicate::Present,
            reference,
            RuleSelectorOrdering::Formation,
            0,
            1,
            RuleEmptyPoolPolicy::NoOp,
            RuleSelectorChoice::All,
            None,
            false,
        )
        .unwrap();
        if raw == 4 {
            selector = selector.with_predicates(vec![RuleSelectorPredicate::StatCompare {
                stat: StatKind::Elation,
                comparison: Comparison::Greater,
                value: literal(0),
            }]);
        }
        builder.add_selector(SelectorDefinition::new(definition(raw)).with_rule_units(selector));
    }
    builder.add_program(
        ProgramDefinition::new(
            definition(3),
            vec![],
            vec![definition(3), definition(5)],
            vec![definition(3)],
            vec![],
        )
        .with_steps(vec![probe_damage(query()), apply_effect(3, 3)]),
    );
    builder.add_program(
        ProgramDefinition::new(
            definition(4),
            vec![],
            vec![definition(3), definition(4), definition(5)],
            vec![definition(5)],
            vec![],
        )
        .with_steps(vec![
            // A Rule IR program evaluates before committing its emissions.
            // Read the new property in the next explicit program phase.
            probe_damage(query()),
            probe_damage(ValueExpr::QueryBaseStat {
                subject: StatQuerySubject::Actor,
                stat: StatKind::Elation,
            }),
            apply_effect(4, 5),
            probe_damage(literal(0)),
            ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                selector: definition(3),
                effect: definition(5),
            }),
        ]),
    );
    assert!(
        builder.replace_ability(
            AbilityDefinition::new(
                definition(1),
                definition(1),
                definition(1),
                vec![definition(3), definition(5)]
            )
            .with_action(action(vec![]))
            .with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, definition(3)).unwrap(),
                AbilityProgramBinding::new(2, AbilityProgramTiming::AfterHits, definition(4))
                    .unwrap(),
            ])
        )
    );
    builder.build().unwrap()
}

fn battle(catalog: Arc<CombatCatalog>, snapshot: Option<SnapshotPolicy>) -> Battle {
    let base = combatant(1, 1, 200_000_000, 1);
    let player = if snapshot.is_some() {
        ResolvedCombatantSpec::new(
            base.form(),
            base.level(),
            base.maximum_hp(),
            base.speed(),
            ResolvedDefinitionBindings::new(
                base.abilities().to_vec(),
                base.rule_bundles().to_vec(),
                vec![definition(4)],
            )
            .unwrap(),
            base.digest(),
        )
        .unwrap()
        .with_sources(vec![RuleSource::new(
            definition(4),
            SourceClass::Ability,
            vec![],
            [0xeb; 32],
        )])
        .unwrap()
        .with_modifier_bindings(vec![ResolvedModifierBinding::new(
            definition(4),
            definition(4),
        )])
        .unwrap()
    } else {
        base
    };
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xec; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                player,
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(2, 2, 101_000_000, 2),
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog, spec, BattleSeed::new([0xed; 32])).unwrap()
}
fn apply(running: &mut Battle, fresh: &mut Battle, command: Command) -> Vec<BattleEvent> {
    let result = running.apply(command.clone()).unwrap();
    let replay = fresh.apply(command).unwrap();
    assert!(result.fault().is_none(), "{:?}", result.fault());
    assert_eq!(result.events(), replay.events());
    assert_eq!(running.state_hash(), fresh.state_hash());
    assert_eq!(
        running.view().rng_draw_count(),
        fresh.view().rng_draw_count()
    );
    result.events().to_vec()
}
fn start(running: &mut Battle, fresh: &mut Battle) {
    let before = running.state_hash();
    let draws = running.view().rng_draw_count();
    assert!(
        running
            .apply(Command::StartBattle {
                decision: DecisionId::new(900).unwrap()
            })
            .is_err()
    );
    assert_eq!(running.state_hash(), before);
    assert_eq!(running.view().rng_draw_count(), draws);
    let command = Command::StartBattle {
        decision: running.decision().unwrap().id(),
    };
    apply(running, fresh, command);
    advance_boundary_if_offered(running);
    advance_boundary_if_offered(fresh);
}
fn attack(running: &mut Battle, fresh: &mut Battle) -> Vec<i64> {
    let command = running
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|c| matches!(c, Command::UseAbility { ability, .. } if ability.get() == 1))
        .unwrap()
        .clone();
    apply(running, fresh, command)
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(data.calculated.get()),
            _ => None,
        })
        .collect()
}

#[test]
fn live_and_selector_snapshot_queries_keep_elation_separate_and_expire_eventfully() {
    for (reference, last) in [
        (RuleSelectorReference::CurrentState, 125),
        (RuleSelectorReference::EventSnapshot, 125),
        (RuleSelectorReference::ActionSnapshot, 100),
    ] {
        let catalog = fixture(reference, None);
        let mut running = battle(Arc::clone(&catalog), None);
        let mut fresh = battle(catalog, None);
        start(&mut running, &mut fresh);
        assert_eq!(attack(&mut running, &mut fresh), [100, 150, 100, last]);
        assert!(
            running
                .view()
                .effects_by_id()
                .any(|effect| effect.definition().get() == 3)
        );
        let command = running.advance_command().unwrap();
        let events = apply(&mut running, &mut fresh, command);
        assert!(
            events
                .iter()
                .any(|e| matches!(e.kind(), BattleEventKind::Effect(EffectEventData::Removed { definition, .. }) if definition.get() == 3))
        );
        assert!(
            running
                .view()
                .effects_by_id()
                .all(|effect| effect.definition().get() != 3)
        );
        assert!(
            running
                .view()
                .modifier_instances_by_id()
                .all(|m| m.definition().get() != 3)
        );
        assert_eq!(running.view().rng_draw_count(), 0);
    }
}

#[test]
fn innate_elation_reads_capture_zero_before_later_effects_and_dynamic_reads_stay_live() {
    for snapshot in [
        SnapshotPolicy::OnApplication,
        SnapshotPolicy::SourceSnapshotTargetDynamic,
        SnapshotPolicy::Dynamic,
    ] {
        let catalog = fixture(RuleSelectorReference::CurrentState, Some(snapshot));
        let mut running = battle(Arc::clone(&catalog), Some(snapshot));
        let mut fresh = battle(catalog, Some(snapshot));
        let modifier = running
            .view()
            .modifier_instances_by_id()
            .find(|m| m.definition().get() == 4)
            .unwrap();
        match snapshot {
            SnapshotPolicy::OnApplication => {
                assert_eq!(modifier.captured_value(), Some(Scalar::ZERO))
            }
            SnapshotPolicy::SourceSnapshotTargetDynamic => {
                let captured = modifier.captured_stats().collect::<Vec<_>>();
                assert_eq!(captured.len(), 1);
                assert_eq!(captured[0].0.stat, StatKind::Elation);
                assert_eq!(captured[0].1, Scalar::ZERO);
            }
            SnapshotPolicy::Dynamic => assert_eq!(modifier.captured_value(), None),
            _ => unreachable!(),
        }
        start(&mut running, &mut fresh);
        let expected = if snapshot == SnapshotPolicy::Dynamic {
            [100, 225, 150, 175]
        } else {
            [100, 150, 100, 125]
        };
        assert_eq!(attack(&mut running, &mut fresh), expected);
    }
}
