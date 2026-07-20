//! Pure build-validation and combat-boundary compilation pipeline.

use starclock_combat::{
    CombatantSpecError, ResolvedCombatantSpec, ResolvedDefinitionBindings, catalog::CombatCatalog,
};

use crate::{
    catalog::BuildCatalog,
    output::CompiledBuild,
    report::{BuildCompilationReport, BuildValidationEntry, BuildValidationStage},
    spec::CombatantBuildSpec,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildCompileErrorKind {
    IncompatibleCatalogs,
    UnknownCharacter,
    UnsupportedLevel,
    InvalidCombatBindings,
    InvalidCombatant,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildCompileError {
    kind: BuildCompileErrorKind,
    report: BuildCompilationReport,
}

impl BuildCompileError {
    #[must_use]
    pub const fn kind(&self) -> BuildCompileErrorKind {
        self.kind
    }
    #[must_use]
    pub const fn report(&self) -> &BuildCompilationReport {
        &self.report
    }
}

impl std::fmt::Display for BuildCompileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "build compilation failed: {:?}", self.kind)
    }
}

impl std::error::Error for BuildCompileError {}

#[derive(Clone, Copy, Debug, Default)]
pub struct LoadoutCompiler;

impl LoadoutCompiler {
    pub fn compile(
        self,
        build_catalog: &BuildCatalog,
        combat_catalog: &CombatCatalog,
        spec: &CombatantBuildSpec,
    ) -> Result<CompiledBuild, BuildCompileError> {
        let mut entries = Vec::with_capacity(5);
        if build_catalog.compatible_combat_revision() != combat_catalog.revision().as_str()
            || build_catalog.compatible_combat_digest() != combat_catalog.digest()
        {
            return Err(failure(
                spec,
                entries,
                BuildValidationStage::CatalogCompatibility,
                BuildCompileErrorKind::IncompatibleCatalogs,
            ));
        }
        entries.push(BuildValidationEntry::passed(
            BuildValidationStage::CatalogCompatibility,
        ));

        let Some(definition) = build_catalog.character(spec.form()) else {
            return Err(failure(
                spec,
                entries,
                BuildValidationStage::CharacterLookup,
                BuildCompileErrorKind::UnknownCharacter,
            ));
        };
        entries.push(BuildValidationEntry::passed(
            BuildValidationStage::CharacterLookup,
        ));

        if definition.level() != spec.level() {
            return Err(failure(
                spec,
                entries,
                BuildValidationStage::LevelSelection,
                BuildCompileErrorKind::UnsupportedLevel,
            ));
        }
        entries.push(BuildValidationEntry::passed(
            BuildValidationStage::LevelSelection,
        ));

        if combat_catalog.unit(definition.form()).is_none()
            || definition
                .abilities()
                .iter()
                .any(|id| combat_catalog.ability(*id).is_none())
            || definition
                .rule_bundles()
                .iter()
                .any(|id| combat_catalog.rule_bundle(*id).is_none())
            || definition
                .modifiers()
                .iter()
                .any(|id| combat_catalog.modifier(*id).is_none())
        {
            return Err(failure(
                spec,
                entries,
                BuildValidationStage::CombatBindings,
                BuildCompileErrorKind::InvalidCombatBindings,
            ));
        }
        let bindings = ResolvedDefinitionBindings::new(
            definition.abilities().to_vec(),
            definition.rule_bundles().to_vec(),
            definition.modifiers().to_vec(),
        )
        .map_err(|_| {
            failure(
                spec,
                entries.clone(),
                BuildValidationStage::CombatBindings,
                BuildCompileErrorKind::InvalidCombatBindings,
            )
        })?;
        entries.push(BuildValidationEntry::passed(
            BuildValidationStage::CombatBindings,
        ));

        let combatant = ResolvedCombatantSpec::new(
            definition.form(),
            definition.level(),
            definition.maximum_hp(),
            definition.speed(),
            bindings,
            definition.combatant_digest(),
        )
        .map_err(|error| combatant_failure(spec, entries.clone(), error))?;
        entries.push(BuildValidationEntry::passed(
            BuildValidationStage::CombatantConstruction,
        ));
        let report = BuildCompilationReport::new(spec.form(), spec.level(), entries);
        Ok(CompiledBuild::new(combatant, report))
    }
}

fn combatant_failure(
    spec: &CombatantBuildSpec,
    entries: Vec<BuildValidationEntry>,
    _error: CombatantSpecError,
) -> BuildCompileError {
    failure(
        spec,
        entries,
        BuildValidationStage::CombatantConstruction,
        BuildCompileErrorKind::InvalidCombatant,
    )
}

fn failure(
    spec: &CombatantBuildSpec,
    mut entries: Vec<BuildValidationEntry>,
    stage: BuildValidationStage,
    kind: BuildCompileErrorKind,
) -> BuildCompileError {
    entries.push(BuildValidationEntry::failed(stage));
    BuildCompileError {
        kind,
        report: BuildCompilationReport::new(spec.form(), spec.level(), entries),
    }
}
