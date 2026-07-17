use std::sync::Arc;

use starclock_combat::{
    AbilityId, EffectDefinitionId, EncounterId, EnemyDefinitionId, Energy, ModifierDefinitionId,
    ProgramId, RuleBundleId, RuleId, SelectorId, UnitDefinitionId,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionResourcePolicy, TargetInvalidationPolicy,
            TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::{CatalogBuildErrorKind, CombatCatalogBuilder},
        definition::{
            AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition,
            ModifierDefinition, ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition,
            UnitDefinition,
        },
    },
};

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).expect("test IDs are non-zero")
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
    builder.add_modifier(ModifierDefinition::new(id(1)));
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
    assert_eq!(forward.definition_count(), 15);
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
