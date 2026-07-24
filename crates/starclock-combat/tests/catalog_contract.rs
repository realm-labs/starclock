use std::sync::Arc;

use starclock_combat::{
    AbilityId, AiCandidateId, AiGraphId, AiStateId, AiTransitionId, EffectDefinitionId,
    EncounterId, EncounterWaveId, EnemyDefinitionId, EnemyPhaseId, Energy, FormationIndex,
    ModifierDefinitionId, ModifierStackingGroupId, OwnerLinkPolicy, ProgramId, RuleBundleId,
    RuleId, Scalar, SelectorId, UnitDefinitionId, WaveLinkPolicy,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::{CatalogBuildErrorKind, CombatCatalogBuilder},
        definition::{
            AbilityDefinition, AbilityParameterDefinition, EffectDefinition, EncounterDefinition,
            EnemyDefinition, ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition,
            UnitDefinition,
        },
        encounter::{
            AiCandidateDefinition, AiCandidateSelection, AiGraphDefinition, AiNoTargetFallback,
            AiStateDefinition, AiTransitionDefinition, AiTransitionTiming, EncounterWaveDefinition,
            EnemyLinkDefinition, EnemyLinkKind, EnemyPhaseCarry, EnemyPhaseDefinition,
            EnemyPhaseTransitionModel, LinkOverflowPolicy, LinkedFormationPolicy, PhaseCarryPolicy,
            WaveCarry, WaveSlotDefinition, WaveTransitionPolicy,
        },
    },
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rng::types::DrawPurpose,
    rule::model::{ConditionExpr, RuleValue, ValueExpr},
};

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).expect("test IDs are non-zero")
}

fn ai_catalog(states: Vec<AiStateDefinition>) -> Result<Arc<CombatCatalog>, CatalogBuildErrorKind> {
    let mut builder = CombatCatalogBuilder::new("ai-contract-v1", [0xa1; 32]);
    builder.add_selector(SelectorDefinition::new(id(1)));
    builder.add_program(ProgramDefinition::new(
        id(1),
        vec![],
        vec![],
        vec![],
        vec![],
    ));
    builder.add_ability(AbilityDefinition::new(id(1), id(1), id(1), vec![]));
    builder.add_unit(UnitDefinition::new(id(1), vec![id(1)], vec![]));
    builder.add_ai_graph(AiGraphDefinition::new(id(1), id(1), 8, states).unwrap());
    let phase = EnemyPhaseDefinition::new(
        id::<EnemyPhaseId>(1),
        1,
        ConditionExpr::Literal(true),
        ConditionExpr::Literal(false),
        0,
        id(1),
        true,
        EnemyPhaseTransitionModel::TransformSameUnit,
        None,
        EnemyPhaseCarry {
            hp: PhaseCarryPolicy::CarryExact,
            action_gauge: PhaseCarryPolicy::CarryExact,
            effects: PhaseCarryPolicy::CarryExact,
            toughness: PhaseCarryPolicy::CarryExact,
            summons: PhaseCarryPolicy::CarryExact,
        },
    );
    let link = EnemyLinkDefinition::new(
        1,
        id(1),
        EnemyLinkKind::Part,
        1,
        LinkOverflowPolicy::Reject,
        OwnerLinkPolicy::Persist,
        WaveLinkPolicy::Persist,
        false,
        LinkedFormationPolicy::NoFormationSlot,
    )
    .unwrap();
    builder.add_enemy(
        EnemyDefinition::new(id(1), id(1), vec![id(1)])
            .with_orchestration(id(1), vec![phase])
            .unwrap()
            .with_links(vec![link])
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
                FormationIndex::new(0).unwrap(),
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
    builder.build().map_err(|error| error.kind())
}

fn ai_candidate(raw: u32, priority: i32) -> AiCandidateDefinition {
    AiCandidateDefinition::new(
        id::<AiCandidateId>(raw),
        id(1),
        ConditionExpr::Literal(true),
        id(1),
        priority,
        AiCandidateSelection::WeightedDraw {
            weight: raw,
            purpose: DrawPurpose::BEHAVIOR_CHOICE,
        },
        AiNoTargetFallback::StayInState,
    )
}

#[test]
fn ai_graphs_canonicalize_candidates_and_reject_unreachable_or_cyclic_states() {
    let state_two =
        AiStateDefinition::new(id(2), None, id(1), false, vec![ai_candidate(3, 0)], vec![]);
    let state_one = AiStateDefinition::new(
        id(1),
        None,
        id(1),
        false,
        vec![ai_candidate(2, 5), ai_candidate(1, -5)],
        vec![AiTransitionDefinition::new(
            id::<AiTransitionId>(1),
            id(2),
            ConditionExpr::Literal(true),
            0,
            AiTransitionTiming::AfterAction,
        )],
    );
    let catalog = ai_catalog(vec![state_two.clone(), state_one.clone()]).unwrap();
    let graph = catalog.ai_graph(id::<AiGraphId>(1)).unwrap();
    assert_eq!(
        graph
            .states()
            .iter()
            .map(|state| state.id().get())
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(
        graph
            .state(id::<AiStateId>(1))
            .unwrap()
            .candidates()
            .iter()
            .map(|item| item.id().get())
            .collect::<Vec<_>>(),
        [1, 2]
    );

    let unreachable =
        AiStateDefinition::new(id(1), None, id(1), false, vec![ai_candidate(1, 0)], vec![]);
    assert_eq!(
        ai_catalog(vec![unreachable, state_two]).unwrap_err(),
        CatalogBuildErrorKind::InvalidDefinition
    );

    let cyclic_one = AiStateDefinition::new(
        id(1),
        None,
        id(1),
        false,
        vec![ai_candidate(1, 0)],
        vec![AiTransitionDefinition::new(
            id(1),
            id(2),
            ConditionExpr::Literal(true),
            0,
            AiTransitionTiming::AutomaticBeforeDecision,
        )],
    );
    let cyclic_two = AiStateDefinition::new(
        id(2),
        None,
        id(1),
        false,
        vec![ai_candidate(2, 0)],
        vec![AiTransitionDefinition::new(
            id(2),
            id(1),
            ConditionExpr::Literal(true),
            0,
            AiTransitionTiming::AutomaticBeforeDecision,
        )],
    );
    assert_eq!(
        ai_catalog(vec![cyclic_one, cyclic_two]).unwrap_err(),
        CatalogBuildErrorKind::InvalidDefinition
    );
}

fn complete_catalog(reverse_insertion: bool) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new("catalog-contract-v1", [0x5a; 32]);

    let mut units = vec![
        UnitDefinition::new(id(1), vec![id(1), id(2)], vec![id(1)]),
        UnitDefinition::new(id(2), vec![], vec![]),
    ];
    let mut abilities = vec![
        AbilityDefinition::new(id(1), id(1), id(1), vec![id(1)]),
        AbilityDefinition::new(id(2), id(2), id(2), vec![]),
    ];
    let mut programs = vec![
        ProgramDefinition::new(id(1), vec![id(2)], vec![id(1)], vec![id(1)], vec![id(1)]),
        ProgramDefinition::new(id(2), vec![], vec![id(2)], vec![], vec![]),
    ];
    let mut selectors = vec![
        SelectorDefinition::new(id(1)),
        SelectorDefinition::new(id(2)),
    ];
    let mut enemies = vec![
        EnemyDefinition::new(id(1), id(1), vec![id(1)]),
        EnemyDefinition::new(id(2), id(2), vec![]),
    ];
    if reverse_insertion {
        units.reverse();
        abilities.reverse();
        programs.reverse();
        selectors.reverse();
        enemies.reverse();
    }

    for definition in units {
        builder.add_unit(definition);
    }
    for definition in abilities {
        builder.add_ability(definition);
    }
    builder.add_ability_parameter(
        AbilityParameterDefinition::new(
            id(1),
            "coefficient",
            RuleValue::Scalar(Scalar::checked_from_integer(2).unwrap()),
        )
        .unwrap(),
    );
    builder.add_effect(EffectDefinition::new(id(1), vec![id(1)], vec![id(1)]));
    builder.add_rule(RuleDefinition::new(
        id(1),
        vec![id(1), id(2)],
        vec![id(1), id(2)],
    ));
    for definition in programs {
        builder.add_program(definition);
    }
    for definition in selectors {
        builder.add_selector(definition);
    }
    builder.add_rule_bundle(RuleBundle::new(id(1), vec![id(1)]));
    builder.add_modifier_group(ModifierStackingGroup {
        id: id::<ModifierStackingGroupId>(1),
        aggregation: ModifierAggregation::Sum,
    });
    builder.add_modifier(ModifierDefinition {
        id: id(1),
        stat: StatKind::Atk,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::Stat,
        value: ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO)),
        stacking_group: id(1),
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Flat,
        snapshot: SnapshotPolicy::Dynamic,
        filters: Box::new([]),
    });
    for definition in enemies {
        builder.add_enemy(definition);
    }
    builder.add_encounter(EncounterDefinition::new(
        id(1),
        vec![id(2), id(1)],
        vec![id(1)],
    ));

    builder.build().expect("complete graph is valid")
}

#[test]
fn insertion_order_cannot_change_canonical_catalog_indexes() {
    let forward = complete_catalog(false);
    let reverse = complete_catalog(true);

    assert_eq!(forward.revision().as_str(), "catalog-contract-v1");
    assert_eq!(forward.digest().bytes(), [0x5a; 32]);
    assert_eq!(forward.definition_count(), 17);
    assert_eq!(
        forward
            .unit_ids()
            .map(UnitDefinitionId::get)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(
        forward
            .program_ids()
            .map(ProgramId::get)
            .collect::<Vec<_>>(),
        reverse
            .program_ids()
            .map(ProgramId::get)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        forward
            .encounter(id(1))
            .expect("encounter exists")
            .enemies(),
        reverse
            .encounter(id(1))
            .expect("encounter exists")
            .enemies()
    );
    assert_eq!(
        forward
            .encounter(id(1))
            .expect("encounter exists")
            .enemies(),
        [id::<EnemyDefinitionId>(2), id(1)]
    );
    assert!(Arc::ptr_eq(&forward, &Arc::clone(&forward)));
}

#[test]
fn validated_catalog_can_be_composed_without_exposing_private_tables() {
    let base = complete_catalog(false);
    let mut builder =
        CombatCatalogBuilder::from_catalog(&base, "catalog-contract-composed-v1", [0x6b; 32]);
    builder.add_encounter(EncounterDefinition::new(id(2), vec![id(2)], vec![]));
    let composed = builder.build().expect("composed catalog validates again");

    assert_eq!(composed.revision().as_str(), "catalog-contract-composed-v1");
    assert_eq!(composed.digest().bytes(), [0x6b; 32]);
    assert_eq!(composed.definition_count(), base.definition_count() + 1);
    assert_eq!(
        composed.ability_parameter(id(1), "coefficient"),
        base.ability_parameter(id(1), "coefficient")
    );
    assert!(composed.encounter(id(1)).is_some());
    assert!(composed.encounter(id(2)).is_some());
}

#[test]
fn duplicate_ids_are_rejected_per_definition_family() {
    let mut builder = CombatCatalogBuilder::new("duplicate-v1", [1; 32]);
    builder.add_selector(SelectorDefinition::new(id(7)));
    builder.add_selector(SelectorDefinition::new(id(7)));

    let error = builder.build().expect_err("duplicate must fail");
    assert_eq!(error.kind(), CatalogBuildErrorKind::DuplicateDefinition);
}

#[test]
fn executable_abilities_require_target_semantics_and_payable_ultimates() {
    assert!(UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::All).is_none());
    let basic = AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap();
    let mut missing_targets = CombatCatalogBuilder::new("missing-targets-v1", [2; 32]);
    missing_targets.add_selector(SelectorDefinition::new(id(1)));
    missing_targets.add_program(ProgramDefinition::new(
        id(1),
        vec![],
        vec![id(1)],
        vec![],
        vec![],
    ));
    missing_targets
        .add_ability(AbilityDefinition::new(id(1), id(1), id(1), vec![]).with_action(basic));
    assert_eq!(
        missing_targets.build().unwrap_err().kind(),
        CatalogBuildErrorKind::InvalidDefinition
    );

    let mut free_ultimate = CombatCatalogBuilder::new("free-ultimate-v1", [3; 32]);
    free_ultimate.add_selector(SelectorDefinition::new(id(1)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::All).unwrap(),
    ));
    free_ultimate.add_program(ProgramDefinition::new(
        id(1),
        vec![],
        vec![id(1)],
        vec![],
        vec![],
    ));
    free_ultimate.add_ability(
        AbilityDefinition::new(id(1), id(1), id(1), vec![]).with_action(
            AbilityActionDefinition::new(
                AbilityKind::Ultimate,
                1,
                TargetInvalidationPolicy::FailAction,
                ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
            )
            .unwrap(),
        ),
    );
    assert_eq!(
        free_ultimate.build().unwrap_err().kind(),
        CatalogBuildErrorKind::InvalidDefinition
    );
}

#[test]
fn missing_typed_references_are_rejected() {
    let mut builder = CombatCatalogBuilder::new("missing-v1", [2; 32]);
    builder.add_program(ProgramDefinition::new(
        id(1),
        vec![id(9)],
        vec![],
        vec![],
        vec![],
    ));

    let error = builder.build().expect_err("missing program must fail");
    assert_eq!(error.kind(), CatalogBuildErrorKind::MissingReference);
}

#[test]
fn set_like_references_must_be_strictly_ordered_and_unique() {
    let mut builder = CombatCatalogBuilder::new("canonical-v1", [3; 32]);
    builder.add_unit(UnitDefinition::new(
        id(1),
        vec![id::<AbilityId>(2), id(1)],
        vec![],
    ));

    let error = builder.build().expect_err("unsorted set must fail");
    assert_eq!(error.kind(), CatalogBuildErrorKind::NonCanonicalReferences);
}

#[test]
fn effect_granted_abilities_require_existing_canonical_references() {
    let mut missing = CombatCatalogBuilder::new("missing-effect-grant-v1", [0x31; 32]);
    missing.add_effect(
        EffectDefinition::new(id(1), vec![], vec![])
            .with_granted_abilities(vec![id::<AbilityId>(9)]),
    );
    assert_eq!(
        missing.build().unwrap_err().kind(),
        CatalogBuildErrorKind::MissingReference
    );

    let mut unsorted = CombatCatalogBuilder::new("unsorted-effect-grant-v1", [0x32; 32]);
    for raw in [1, 2] {
        unsorted.add_selector(SelectorDefinition::new(id(raw)));
        unsorted.add_program(ProgramDefinition::new(
            id(raw),
            vec![],
            vec![],
            vec![],
            vec![],
        ));
        unsorted.add_ability(AbilityDefinition::new(id(raw), id(raw), id(raw), vec![]));
    }
    unsorted.add_effect(
        EffectDefinition::new(id(1), vec![], vec![])
            .with_granted_abilities(vec![id::<AbilityId>(2), id(1)]),
    );
    assert_eq!(
        unsorted.build().unwrap_err().kind(),
        CatalogBuildErrorKind::NonCanonicalReferences
    );
}

#[test]
fn program_cycles_report_a_canonical_closed_path() {
    let mut builder = CombatCatalogBuilder::new("cycle-v1", [4; 32]);
    builder.add_program(ProgramDefinition::new(
        id(2),
        vec![id(1)],
        vec![],
        vec![],
        vec![],
    ));
    builder.add_program(ProgramDefinition::new(
        id(1),
        vec![id(2)],
        vec![],
        vec![],
        vec![],
    ));

    let error = builder.build().expect_err("cycle must fail");
    assert_eq!(error.kind(), CatalogBuildErrorKind::ProgramCycle);
    assert_eq!(
        error
            .program_cycle()
            .iter()
            .map(|value| value.get())
            .collect::<Vec<_>>(),
        vec![1, 2, 1]
    );
}

#[test]
fn malformed_catalog_identity_is_rejected_before_graph_validation() {
    let mut builder = CombatCatalogBuilder::new("", [0; 32]);
    builder.add_effect(EffectDefinition::new(
        id::<EffectDefinitionId>(1),
        vec![id::<RuleId>(9)],
        vec![id::<ModifierDefinitionId>(9)],
    ));

    let error = builder.build().expect_err("identity must fail first");
    assert_eq!(error.kind(), CatalogBuildErrorKind::InvalidCatalogIdentity);
}

#[test]
fn definition_id_types_are_not_interchangeable_at_catalog_boundaries() {
    let _: UnitDefinitionId = id(1);
    let _: AbilityId = id(1);
    let _: EncounterId = id(1);
    let _: RuleBundleId = id(1);
    let _: SelectorId = id(1);
}

#[test]
fn phased_actions_accept_the_released_101_hit_envelope_but_remain_finite() {
    let base = AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap();
    let hits = |count| {
        (0..count)
            .map(|_| ActionHitDefinition::new(Vec::new()))
            .collect::<Vec<_>>()
    };

    assert_eq!(base.clone().with_hits(hits(101)).unwrap().hit_count(), 101);
    assert_eq!(base.clone().with_hits(hits(256)).unwrap().hit_count(), 256);
    assert!(base.with_hits(hits(257)).is_none());
    assert!(
        AbilityActionDefinition::new(
            AbilityKind::Basic,
            257,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
        )
        .is_none()
    );
}
