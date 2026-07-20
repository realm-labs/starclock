use std::sync::Arc;

use starclock_build::{
    catalog::{
        BuildCatalog, BuildCatalogBuilder, BuildCatalogErrorKind, BuildCatalogRevision,
        CharacterBuildDefinition,
    },
    compiler::{BuildCompileErrorKind, LoadoutCompiler},
    report::{BuildValidationOutcome, BuildValidationStage},
    spec::CombatantBuildSpec,
};
use starclock_combat::{
    AbilityId, CombatantSpecDigest, Energy, Hp, ResolvedDefinitionBindings, Speed,
    UnitDefinitionId, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{AbilityDefinition, ProgramDefinition, SelectorDefinition, UnitDefinition},
    },
};

#[test]
fn catalog_canonicalizes_definition_order_and_compiles_only_combat_types() {
    let combat = combat_catalog("combat-v1");
    let mut builder = build_builder("combat-v1");
    builder.add_character(character(2, 2));
    builder.add_character(character(1, 1));
    let catalog = builder.build(&combat).unwrap();

    assert_eq!(
        catalog.character_ids().collect::<Vec<_>>(),
        vec![form(1), form(2)]
    );
    let spec = CombatantBuildSpec::new(form(1), UnitLevel::new(80).unwrap());
    let compiled = LoadoutCompiler.compile(&catalog, &combat, &spec).unwrap();

    assert_eq!(compiled.combatant().form(), form(1));
    assert_eq!(compiled.combatant().level(), UnitLevel::new(80).unwrap());
    assert_eq!(compiled.combatant().abilities(), &[ability(1)]);
    assert!(compiled.report().is_valid());
    assert_eq!(
        compiled
            .report()
            .entries()
            .iter()
            .map(|entry| entry.stage())
            .collect::<Vec<_>>(),
        vec![
            BuildValidationStage::CatalogCompatibility,
            BuildValidationStage::CharacterLookup,
            BuildValidationStage::LevelSelection,
            BuildValidationStage::CombatBindings,
            BuildValidationStage::CombatantConstruction,
        ]
    );
}

#[test]
fn catalog_rejects_duplicate_forms_and_cross_catalog_bindings() {
    let combat = combat_catalog("combat-v1");
    let mut duplicates = build_builder("combat-v1");
    duplicates.add_character(character(1, 1));
    duplicates.add_character(character(1, 1));
    let error = duplicates.build(&combat).unwrap_err();
    assert_eq!(error.kind(), BuildCatalogErrorKind::DuplicateCharacter);
    assert_eq!(error.form(), Some(form(1)));

    let mut invalid = build_builder("combat-v1");
    invalid.add_character(character(1, 2));
    let error = invalid.build(&combat).unwrap_err();
    assert_eq!(error.kind(), BuildCatalogErrorKind::InvalidAbilityBinding);
    assert_eq!(error.form(), Some(form(1)));
}

#[test]
fn invalid_builds_return_ordered_typed_validation_reports() {
    let combat = combat_catalog("combat-v1");
    let catalog = build_catalog(&combat, "combat-v1");
    let unknown = CombatantBuildSpec::new(form(3), UnitLevel::new(80).unwrap());
    let error = LoadoutCompiler
        .compile(&catalog, &combat, &unknown)
        .unwrap_err();
    assert_eq!(error.kind(), BuildCompileErrorKind::UnknownCharacter);
    assert_eq!(error.report().entries().len(), 2);
    assert_eq!(
        error.report().entries()[1].outcome(),
        BuildValidationOutcome::Failed
    );
    assert_eq!(
        error.report().entries()[1].stage(),
        BuildValidationStage::CharacterLookup
    );

    let wrong_level = CombatantBuildSpec::new(form(1), UnitLevel::new(79).unwrap());
    let error = LoadoutCompiler
        .compile(&catalog, &combat, &wrong_level)
        .unwrap_err();
    assert_eq!(error.kind(), BuildCompileErrorKind::UnsupportedLevel);
    assert_eq!(
        error.report().entries().last().unwrap().stage(),
        BuildValidationStage::LevelSelection
    );
}

#[test]
fn compile_rechecks_catalog_compatibility_without_mutating_either_catalog() {
    let combat_v1 = combat_catalog("combat-v1");
    let catalog = build_catalog(&combat_v1, "combat-v1");
    let combat_v2 = combat_catalog_with_digest("combat-v2", 0x72);
    let spec = CombatantBuildSpec::new(form(1), UnitLevel::new(80).unwrap());

    let error = LoadoutCompiler
        .compile(&catalog, &combat_v2, &spec)
        .unwrap_err();
    assert_eq!(error.kind(), BuildCompileErrorKind::IncompatibleCatalogs);
    assert_eq!(error.report().entries().len(), 1);
    assert_eq!(
        error.report().entries()[0].stage(),
        BuildValidationStage::CatalogCompatibility
    );
    assert_eq!(catalog.character_ids().count(), 1);
    assert_eq!(combat_v1.unit_ids().count(), 2);

    let same_revision_different_digest = combat_catalog_with_digest("combat-v1", 0x73);
    let error = LoadoutCompiler
        .compile(&catalog, &same_revision_different_digest, &spec)
        .unwrap_err();
    assert_eq!(error.kind(), BuildCompileErrorKind::IncompatibleCatalogs);
}

fn build_catalog(combat: &CombatCatalog, compatible: &str) -> BuildCatalog {
    let mut builder = build_builder(compatible);
    builder.add_character(character(1, 1));
    builder.build(combat).unwrap()
}

fn build_builder(compatible: &str) -> BuildCatalogBuilder {
    BuildCatalogBuilder::new(BuildCatalogRevision::new("build-v1").unwrap(), compatible).unwrap()
}

fn character(form: u32, bound_ability: u32) -> CharacterBuildDefinition {
    CharacterBuildDefinition::new(
        self::form(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(10_000).unwrap(),
        Speed::from_scaled(100_000_000).unwrap(),
        ResolvedDefinitionBindings::new(vec![ability(bound_ability)], vec![], vec![]).unwrap(),
        CombatantSpecDigest::new([u8::try_from(form).unwrap(); 32]).unwrap(),
    )
}

fn combat_catalog(revision: &str) -> Arc<CombatCatalog> {
    combat_catalog_with_digest(revision, 0x71)
}

fn combat_catalog_with_digest(revision: &str, digest: u8) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new(revision, [digest; 32]);
    for raw in 1..=2 {
        builder.add_selector(SelectorDefinition::new(definition(raw)).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single).unwrap(),
        ));
        builder.add_program(ProgramDefinition::new(
            definition(raw),
            vec![],
            vec![definition(raw)],
            vec![],
            vec![],
        ));
        builder.add_ability(
            AbilityDefinition::new(ability(raw), definition(raw), definition(raw), vec![])
                .with_action(
                    AbilityActionDefinition::new(
                        AbilityKind::Basic,
                        1,
                        TargetInvalidationPolicy::CancelRemainingForTarget,
                        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
                    )
                    .unwrap()
                    .with_hits(vec![ActionHitDefinition::new(vec![])])
                    .unwrap(),
                ),
        );
        builder.add_unit(UnitDefinition::new(form(raw), vec![ability(raw)], vec![]));
    }
    builder.build().unwrap()
}

fn form(raw: u32) -> UnitDefinitionId {
    UnitDefinitionId::new(raw).unwrap()
}

fn ability(raw: u32) -> AbilityId {
    AbilityId::new(raw).unwrap()
}

fn definition<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}
