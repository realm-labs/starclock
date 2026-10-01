//! Resolved effect magnitudes feed real modifier formulas, not identity-only state.
use super::{action, catalog, combatant, definition};
use crate::combat_decision::advance_boundary_if_offered;
use starclock_combat::{
    AssemblyDigest, Battle, BattleBuildErrorKind, BattleEvent, BattleEventKind, BattleSeed,
    BattleSpec, Command, ConcedePolicy, DecisionId, DispelCategory, DurationClock,
    EffectApplicationDefinition, EffectCategory, EffectChancePolicy, EffectRuntimeDefinition,
    EffectRuntimeTemplate, EffectStackPolicy, EffectTickPhase, FormationIndex, ParticipantSource,
    ParticipantSpec, ResolvedCombatantSpec, ResolvedDefinitionBindings, ResolvedModifierBinding,
    Rounding, Scalar, TeamResourceSpec, TeamSide,
    catalog::{
        CombatCatalog,
        action::{AbilityProgramBinding, AbilityProgramTiming, HitOperationDefinition},
        builder::{CatalogBuildErrorKind, CombatCatalogBuilder},
        definition::{AbilityDefinition, EffectDefinition, ProgramDefinition, SelectorDefinition},
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
        ProgramStep, RuleEffectChancePolicy, RuleOperationTemplate, RuleSource, RuleValue,
        RuleValueKind, SourceClass, ValueExpr,
    },
};
use std::sync::Arc;

fn builder(policy: EffectStackPolicy, snapshot: SnapshotPolicy) -> CombatCatalogBuilder {
    let mut builder = CombatCatalogBuilder::from_catalog(&catalog(), [0x64; 32]);
    builder.add_modifier_group(ModifierStackingGroup {
        id: definition(3),
        aggregation: ModifierAggregation::Sum,
        comparator: None,
    });
    builder.add_modifier(ModifierDefinition {
        id: definition(3),
        stat: StatKind::Atk,
        stage: FormulaStage::Vulnerability,
        purpose: FormulaPurpose::OrdinaryDamage,
        value: ValueExpr::Slot(definition(30)),
        stacking_group: definition(3),
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Vulnerability,
        snapshot,
        source_stack_slot: None,
        filters: Box::new([]),
    });
    builder.add_effect(effect(3, vec![3], vec![(3, 30)], policy));
    builder.add_selector(
        SelectorDefinition::new(definition(3)).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::EventTargets,
                RuleSelectorSide::Opposing,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Present,
                RuleSelectorReference::CurrentState,
                RuleSelectorOrdering::Formation,
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
    let apply = ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector: definition(3),
        effect: definition(3),
        stacks: ValueExpr::Literal(RuleValue::Integer(1)),
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    });
    let damage = ProgramStep::Operation(RuleOperationTemplate::Damage {
        selector: definition(3),
        amount: ValueExpr::Literal(RuleValue::Scalar(
            Scalar::checked_from_integer(100).unwrap(),
        )),
        class: DamageClass::Direct,
        element: CombatElement::Physical,
        can_crit: false,
        can_defeat: true,
    });
    builder.add_program(
        ProgramDefinition::new(
            definition(3),
            vec![],
            vec![definition(3)],
            vec![definition(3)],
            vec![],
        )
        .with_steps(vec![apply, damage]),
    );
    assert!(
        builder.replace_ability(
            AbilityDefinition::new(
                definition(1),
                definition(1),
                definition(1),
                vec![definition(3)]
            )
            .with_action(action(vec![]))
            .with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, definition(3)).unwrap(),
                AbilityProgramBinding::new(2, AbilityProgramTiming::AfterHits, definition(3))
                    .unwrap(),
            ]),
        )
    );
    builder
}

fn effect(
    raw: u32,
    modifiers: Vec<u32>,
    captures: Vec<(u32, u32)>,
    policy: EffectStackPolicy,
) -> EffectDefinition {
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Debuff,
        DispelCategory::DispellableDebuff,
        2,
        Some(ValueExpr::Literal(RuleValue::Integer(1))),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        policy,
    )
    .unwrap()
    .with_comparison(
        Some(ValueExpr::Divide {
            lhs: Box::new(ValueExpr::QueryHp {
                subject: StatQuerySubject::CurrentTarget,
            }),
            rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(
                Scalar::checked_from_integer(1000).unwrap(),
            ))),
            rounding: Rounding::NearestTiesEven,
        }),
        0,
    );
    EffectDefinition::new(
        definition(raw),
        vec![],
        modifiers.into_iter().map(definition).collect(),
    )
    .with_runtime_template(runtime)
    .with_modifier_magnitude_slots(
        captures
            .into_iter()
            .map(|(modifier, slot)| (definition(modifier), definition(slot)))
            .collect(),
    )
}

fn battle(catalog: Arc<CombatCatalog>) -> Battle {
    Battle::create(catalog, spec(false), BattleSeed::new([0x66; 32])).unwrap()
}

fn spec(innate_modifier: bool) -> BattleSpec {
    let base = combatant(1, 1, 200_000_000, 1);
    let player = if innate_modifier {
        ResolvedCombatantSpec::new(
            base.form(),
            base.level(),
            base.maximum_hp(),
            base.speed(),
            ResolvedDefinitionBindings::new(
                base.abilities().to_vec(),
                base.rule_bundles().to_vec(),
                vec![definition(3)],
            )
            .unwrap(),
            base.digest(),
        )
        .unwrap()
        .with_sources(vec![RuleSource::new(
            definition(3),
            SourceClass::Ability,
            vec![],
            [0x63; 32],
        )])
        .unwrap()
        .with_modifier_bindings(vec![ResolvedModifierBinding::new(
            definition(3),
            definition(3),
        )])
        .unwrap()
    } else {
        base
    };
    BattleSpec::new(
        AssemblyDigest::new([0x65; 32]).unwrap(),
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
    .unwrap()
}

fn apply(running: &mut Battle, fresh: &mut Battle, command: Command) -> Vec<BattleEvent> {
    let actual = running.apply(command.clone()).unwrap();
    let reconstructed = fresh.apply(command).unwrap();
    assert!(actual.fault().is_none(), "{:?}", actual.fault());
    assert_eq!(actual.events(), reconstructed.events());
    assert_eq!(running.state_hash(), fresh.state_hash());
    assert_eq!(
        running.view().rng_draw_count(),
        fresh.view().rng_draw_count()
    );
    actual.events().to_vec()
}

#[test]
fn magnitude_capture_executes_actual_damage_and_follows_retained_refresh_policy() {
    for policy in [
        EffectStackPolicy::Refresh,
        EffectStackPolicy::RefreshAndAddStacks,
        EffectStackPolicy::IndependentBySource,
        EffectStackPolicy::Replace,
    ] {
        for snapshot in [
            SnapshotPolicy::Dynamic,
            SnapshotPolicy::OnApplication,
            SnapshotPolicy::RecomputeOnStackChange,
        ] {
            let catalog = builder(policy, snapshot).build().unwrap();
            let mut running = battle(Arc::clone(&catalog));
            let mut fresh = battle(catalog);
            let before = running.state_hash();
            let rng = running.view().rng_draw_count();
            assert!(
                running
                    .apply(Command::StartBattle {
                        decision: DecisionId::new(900).unwrap()
                    })
                    .is_err()
            );
            assert_eq!(running.state_hash(), before);
            assert_eq!(running.view().rng_draw_count(), rng);
            let command = Command::StartBattle {
                decision: running.decision().unwrap().id(),
            };
            apply(&mut running, &mut fresh, command);
            advance_boundary_if_offered(&mut running);
            advance_boundary_if_offered(&mut fresh);
            let command = running.decision().unwrap().legal_commands().iter()
                .find(|command| matches!(command, Command::UseAbility { ability, .. } if ability.get() == 1)).unwrap().clone();
            let events = apply(&mut running, &mut fresh, command);
            let damages = events
                .iter()
                .filter_map(|event| match event.kind() {
                    BattleEventKind::Damage(data) => Some(data.calculated.get()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let refresh_preserves = matches!(
                policy,
                EffectStackPolicy::Refresh | EffectStackPolicy::RefreshAndAddStacks
            );
            assert_eq!(damages, [200, if refresh_preserves { 200 } else { 180 }]);
            let captured = if refresh_preserves {
                Scalar::ONE
            } else {
                Scalar::from_scaled(800_000)
            };
            let modifier = running
                .view()
                .modifier_instances_by_id()
                .find(|modifier| modifier.definition().get() == 3)
                .unwrap();
            assert_eq!(
                modifier.slots().collect::<Vec<_>>(),
                [(definition(30), &RuleValue::Scalar(captured))]
            );
            assert_eq!(
                modifier.captured_value(),
                (snapshot != SnapshotPolicy::Dynamic).then_some(captured)
            );
            let effect = running
                .view()
                .effects_by_id()
                .find(|effect| effect.definition().get() == 3)
                .unwrap();
            assert_eq!(effect.magnitude(), captured);
            assert_eq!(modifier.source_effect(), Some(effect.id()));
            for _ in 0..4 {
                let command = if let Some(command) = running.advance_command() {
                    command
                } else {
                    running.decision().unwrap().legal_commands().iter()
                        .find(|command| matches!(command, Command::UseAbility { ability, .. } if ability.get() == 2)).unwrap().clone()
                };
                apply(&mut running, &mut fresh, command);
                if running
                    .view()
                    .effects_by_id()
                    .all(|effect| effect.definition().get() != 3)
                {
                    break;
                }
            }
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
                    .all(|modifier| modifier.definition().get() != 3)
            );
        }
    }
}

#[test]
fn magnitude_bindings_reject_duplicates_unattached_missing_and_shared_owners() {
    for (modifiers, captures, expected) in [
        (
            vec![3],
            vec![(3, 30), (3, 31)],
            CatalogBuildErrorKind::InvalidDefinition,
        ),
        (
            vec![],
            vec![(3, 30)],
            CatalogBuildErrorKind::InvalidDefinition,
        ),
        (
            vec![],
            vec![(99, 30)],
            CatalogBuildErrorKind::MissingReference,
        ),
        (
            vec![3],
            vec![(1, 2)],
            CatalogBuildErrorKind::InvalidDefinition,
        ),
        (
            vec![3],
            vec![(3, 30)],
            CatalogBuildErrorKind::InvalidDefinition,
        ),
        (
            vec![3],
            vec![(3, 30), (1, 31)],
            CatalogBuildErrorKind::InvalidDefinition,
        ),
    ] {
        let mut builder = builder(EffectStackPolicy::Replace, SnapshotPolicy::Dynamic);
        builder.add_effect(effect(4, modifiers, captures, EffectStackPolicy::Replace));
        assert_eq!(builder.build().unwrap_err().kind(), expected);
    }
}

fn stack_and_magnitude_builder(capture_slot: u32) -> CombatCatalogBuilder {
    let mut builder = CombatCatalogBuilder::from_catalog(&catalog(), [0x67; 32]);
    builder.add_modifier_group(ModifierStackingGroup {
        id: definition(4),
        aggregation: ModifierAggregation::Sum,
        comparator: None,
    });
    builder.add_modifier(ModifierDefinition {
        id: definition(4),
        stat: StatKind::Atk,
        stage: FormulaStage::Vulnerability,
        purpose: FormulaPurpose::OrdinaryDamage,
        value: ValueExpr::Add(
            Box::new(ValueExpr::Slot(definition(30))),
            Box::new(ValueExpr::Convert {
                value: Box::new(ValueExpr::Slot(definition(31))),
                target: RuleValueKind::Scalar,
                rounding: Rounding::NearestTiesEven,
            }),
        ),
        stacking_group: definition(4),
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Vulnerability,
        snapshot: SnapshotPolicy::RecomputeOnStackChange,
        source_stack_slot: Some(definition(31)),
        filters: Box::new([]),
    });
    builder.add_effect(
        EffectDefinition::new(definition(4), vec![], vec![definition(4)])
            .with_runtime(
                EffectRuntimeDefinition::new(
                    EffectCategory::Debuff,
                    DispelCategory::DispellableDebuff,
                    2,
                    Some(1),
                    DurationClock::TargetTurnEnd,
                    EffectTickPhase::None,
                    EffectStackPolicy::RefreshAndAddStacks,
                )
                .unwrap()
                .with_comparison(Scalar::from_scaled(700_001), 0),
            )
            .with_modifier_magnitude_slots(vec![(definition(4), definition(capture_slot))]),
    );
    let apply = HitOperationDefinition::ApplyEffect(
        EffectApplicationDefinition::new(definition(4), EffectChancePolicy::Guaranteed, 1).unwrap(),
    );
    assert!(
        builder.replace_ability(
            AbilityDefinition::new(
                definition(1),
                definition(1),
                definition(1),
                vec![definition(4)],
            )
            .with_action(action(vec![apply.clone(), apply])),
        )
    );
    builder
}

#[test]
fn magnitude_and_stack_slots_keep_distinct_types_and_refresh_before_recapture() {
    let catalog = stack_and_magnitude_builder(30).build().unwrap();
    let mut running = battle(Arc::clone(&catalog));
    let mut fresh = battle(catalog);
    let command = Command::StartBattle {
        decision: running.decision().unwrap().id(),
    };
    apply(&mut running, &mut fresh, command);
    advance_boundary_if_offered(&mut running);
    advance_boundary_if_offered(&mut fresh);
    let command = running
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(
            |command| matches!(command, Command::UseAbility { ability, .. } if ability.get() == 1),
        )
        .unwrap()
        .clone();
    apply(&mut running, &mut fresh, command);
    let modifier = running
        .view()
        .modifier_instances_by_id()
        .find(|modifier| modifier.definition().get() == 4)
        .unwrap();
    assert_eq!(
        modifier.slots().collect::<Vec<_>>(),
        [
            (
                definition(30),
                &RuleValue::Scalar(Scalar::from_scaled(700_001))
            ),
            (definition(31), &RuleValue::Integer(2)),
        ]
    );
    assert_eq!(
        modifier.captured_value(),
        Some(Scalar::from_scaled(2_700_001))
    );
    let effect = running
        .view()
        .effects_by_id()
        .find(|effect| effect.definition().get() == 4)
        .unwrap();
    assert_eq!(effect.stacks(), 2);
    assert_eq!(effect.magnitude(), Scalar::from_scaled(700_001));
    assert_eq!(modifier.source_effect(), Some(effect.id()));
}

#[test]
fn magnitude_capture_cannot_alias_its_integer_stack_slot() {
    assert_eq!(
        stack_and_magnitude_builder(31).build().unwrap_err().kind(),
        CatalogBuildErrorKind::InvalidDefinition
    );
}

#[test]
fn effect_magnitude_modifiers_cannot_be_attached_as_innate_modifiers() {
    let catalog = builder(EffectStackPolicy::Replace, SnapshotPolicy::Dynamic)
        .build()
        .unwrap();
    let error = Battle::create(catalog, spec(true), BattleSeed::new([0x66; 32])).unwrap_err();
    assert_eq!(
        error.kind(),
        BattleBuildErrorKind::InvalidModifierAttachment
    );
    assert_eq!(error.definition_id(), Some(3));
}
