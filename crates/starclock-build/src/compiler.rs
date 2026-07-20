//! Pure build-validation and combat-boundary compilation pipeline.

use std::collections::{BTreeMap, BTreeSet};

use starclock_combat::{
    AbilityId, CombatantSpecError, ModifierDefinitionId, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, RuleBundleId, catalog::CombatCatalog,
};

use crate::{
    ability::{AbilityInvestment, AbilityLevel},
    catalog::BuildCatalog,
    output::CompiledBuild,
    patch::BuildPatch,
    report::{BuildCompilationReport, BuildValidationEntry, BuildValidationStage},
    spec::CombatantBuildSpec,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildCompileErrorKind {
    IncompatibleCatalogs,
    UnknownCharacter,
    UnsupportedLevel,
    InvalidAbilitySelection,
    InvalidTraceSelection,
    InvalidEidolonSelection,
    PatchConflict,
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
        let mut entries = Vec::with_capacity(8);
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

        let Some(stat_row) = definition.stat_row(spec.level(), spec.promotion()) else {
            return Err(failure(
                spec,
                entries,
                BuildValidationStage::LevelSelection,
                BuildCompileErrorKind::UnsupportedLevel,
            ));
        };
        entries.push(BuildValidationEntry::passed(
            BuildValidationStage::LevelSelection,
        ));

        if !valid_ability_input(definition.ability_levels(), spec.ability_levels()) {
            return Err(failure(
                spec,
                entries,
                BuildValidationStage::AbilitySelection,
                BuildCompileErrorKind::InvalidAbilitySelection,
            ));
        }
        entries.push(BuildValidationEntry::passed(
            BuildValidationStage::AbilitySelection,
        ));

        let mut workspace = CompilationWorkspace::new(definition);
        if apply_traces(definition, spec, &mut workspace).is_err() {
            return Err(failure(
                spec,
                entries,
                BuildValidationStage::TraceSelection,
                BuildCompileErrorKind::InvalidTraceSelection,
            ));
        }
        entries.push(BuildValidationEntry::passed(
            BuildValidationStage::TraceSelection,
        ));

        if apply_eidolons(definition, spec, &mut workspace).is_err() {
            return Err(failure(
                spec,
                entries,
                BuildValidationStage::EidolonSelection,
                BuildCompileErrorKind::InvalidEidolonSelection,
            ));
        }
        if resolve_ability_levels(definition, spec.ability_levels(), &mut workspace).is_err() {
            return Err(failure(
                spec,
                entries,
                BuildValidationStage::EidolonSelection,
                BuildCompileErrorKind::PatchConflict,
            ));
        }
        entries.push(BuildValidationEntry::passed(
            BuildValidationStage::EidolonSelection,
        ));

        if combat_catalog.unit(definition.form()).is_none()
            || workspace
                .abilities
                .iter()
                .any(|id| combat_catalog.ability(*id).is_none())
            || workspace
                .rule_bundles
                .iter()
                .any(|id| combat_catalog.rule_bundle(*id).is_none())
            || workspace
                .modifiers
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
            workspace.abilities.into_iter().collect(),
            workspace.rule_bundles.into_iter().collect(),
            workspace.modifiers.into_iter().collect(),
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
            stat_row.level(),
            stat_row.maximum_hp(),
            stat_row.speed(),
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

fn valid_ability_input(
    tables: &[crate::ability::AbilityLevelTable],
    investments: &[AbilityInvestment],
) -> bool {
    tables.len() == investments.len()
        && tables.iter().zip(investments).all(|(table, investment)| {
            table.family() == investment.family() && investment.invested() <= table.invested_cap()
        })
}

struct CompilationWorkspace {
    abilities: BTreeSet<AbilityId>,
    rule_bundles: BTreeSet<RuleBundleId>,
    modifiers: BTreeSet<ModifierDefinitionId>,
    ability_adjustments: BTreeMap<AbilityId, (i16, i16)>,
}

impl CompilationWorkspace {
    fn new(definition: &crate::catalog::CharacterBuildDefinition) -> Self {
        let mut abilities = definition
            .abilities()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        for row in definition
            .ability_levels()
            .iter()
            .flat_map(|table| table.rows())
        {
            abilities.remove(&row.resolved_ability());
        }
        Self {
            abilities,
            rule_bundles: definition.rule_bundles().iter().copied().collect(),
            modifiers: definition.modifiers().iter().copied().collect(),
            ability_adjustments: BTreeMap::new(),
        }
    }

    fn apply_patch(&mut self, patch: BuildPatch) -> Result<(), ()> {
        match patch {
            BuildPatch::AddAbility(id) if !self.abilities.insert(id) => Err(()),
            BuildPatch::AddRuleBundle(id) if !self.rule_bundles.insert(id) => Err(()),
            BuildPatch::RemoveRuleBundle(id) if !self.rule_bundles.remove(&id) => Err(()),
            BuildPatch::AddModifier(id) if !self.modifiers.insert(id) => Err(()),
            BuildPatch::ReplaceAbility { old, new }
                if old == new || !self.abilities.remove(&old) || !self.abilities.insert(new) =>
            {
                Err(())
            }
            BuildPatch::AdjustAbilityLevel {
                family,
                bonus,
                cap_delta,
            } => {
                let adjustment = self.ability_adjustments.entry(family).or_default();
                adjustment.0 = adjustment.0.checked_add(i16::from(bonus)).ok_or(())?;
                adjustment.1 = adjustment.1.checked_add(i16::from(cap_delta)).ok_or(())?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

fn apply_traces(
    definition: &crate::catalog::CharacterBuildDefinition,
    spec: &CombatantBuildSpec,
    workspace: &mut CompilationWorkspace,
) -> Result<(), ()> {
    let selected = spec.traces().iter().copied().collect::<BTreeSet<_>>();
    let Some(graph) = definition.trace_graph() else {
        return if selected.is_empty() { Ok(()) } else { Err(()) };
    };
    for id in &selected {
        let node = graph.node(*id).ok_or(())?;
        if node.promotion_requirement() > spec.promotion()
            || node
                .prerequisites()
                .iter()
                .any(|required| !selected.contains(required))
        {
            return Err(());
        }
    }
    for id in graph
        .canonical_order()
        .iter()
        .filter(|id| selected.contains(id))
    {
        let node = graph.node(*id).expect("canonical Trace ID resolves");
        for patch in node.patches() {
            workspace.apply_patch(*patch)?;
        }
    }
    Ok(())
}

fn apply_eidolons(
    definition: &crate::catalog::CharacterBuildDefinition,
    spec: &CombatantBuildSpec,
    workspace: &mut CompilationWorkspace,
) -> Result<(), ()> {
    for raw_rank in 1..=spec.eidolon().get() {
        let rank = crate::spec::EidolonLevel::new(raw_rank).ok_or(())?;
        let definition = definition.eidolons().rank(rank).ok_or(())?;
        for patch in definition.patches() {
            workspace.apply_patch(*patch)?;
        }
    }
    Ok(())
}

fn resolve_ability_levels(
    definition: &crate::catalog::CharacterBuildDefinition,
    investments: &[AbilityInvestment],
    workspace: &mut CompilationWorkspace,
) -> Result<(), ()> {
    for (table, investment) in definition.ability_levels().iter().zip(investments) {
        let (bonus, cap_delta) = workspace
            .ability_adjustments
            .get(&table.family())
            .copied()
            .unwrap_or_default();
        let effective = i16::from(investment.invested().get())
            .checked_add(bonus)
            .ok_or(())?;
        let cap = i16::from(table.invested_cap().get())
            .checked_add(cap_delta)
            .ok_or(())?;
        if effective < 1 || effective > cap {
            return Err(());
        }
        let effective = AbilityLevel::new(u8::try_from(effective).map_err(|_| ())?).ok_or(())?;
        let resolved = table.resolve(effective).ok_or(())?;
        if !workspace.abilities.insert(resolved) {
            return Err(());
        }
    }
    Ok(())
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
