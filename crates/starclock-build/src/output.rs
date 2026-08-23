//! Successful generic combat compilation output.

use starclock_combat::{CombatantSpecDigest, ResolvedCombatantSpec};

use crate::{
    ability::AbilityInvestment,
    catalog::BuildCatalog,
    digest::{BuildCatalogDigest, CombatantBuildDigest},
    report::BuildCompilationReport,
    spec::CombatantBuildSpec,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledBuild {
    combatant: ResolvedCombatantSpec,
    report: BuildCompilationReport,
    build_digest: CombatantBuildDigest,
    effective_ability_levels: Box<[AbilityInvestment]>,
    selected_spec: CombatantBuildSpec,
    lock: BuildLock,
}

impl CompiledBuild {
    pub(crate) fn new(
        combatant: ResolvedCombatantSpec,
        report: BuildCompilationReport,
        build_digest: CombatantBuildDigest,
        catalog_digest: BuildCatalogDigest,
        effective_ability_levels: Box<[AbilityInvestment]>,
        selected_spec: CombatantBuildSpec,
    ) -> Self {
        let lock = BuildLock {
            catalog_digest,
            build_digest,
            combatant_digest: combatant.digest(),
        };
        Self {
            combatant,
            report,
            build_digest,
            effective_ability_levels,
            selected_spec,
            lock,
        }
    }
    #[must_use]
    pub const fn combatant(&self) -> &ResolvedCombatantSpec {
        &self.combatant
    }
    #[must_use]
    pub const fn report(&self) -> &BuildCompilationReport {
        &self.report
    }
    #[must_use]
    pub const fn build_digest(&self) -> CombatantBuildDigest {
        self.build_digest
    }
    /// Final selected levels after Trace, Eidolon and contribution adjustments.
    #[must_use]
    pub fn effective_ability_levels(&self) -> &[AbilityInvestment] {
        &self.effective_ability_levels
    }
    /// Exact normalized input selected by this compilation.
    #[must_use]
    pub const fn selected_spec(&self) -> &CombatantBuildSpec {
        &self.selected_spec
    }
    #[must_use]
    pub const fn lock(&self) -> &BuildLock {
        &self.lock
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildLock {
    catalog_digest: BuildCatalogDigest,
    build_digest: CombatantBuildDigest,
    combatant_digest: CombatantSpecDigest,
}

impl BuildLock {
    #[must_use]
    pub const fn catalog_digest(&self) -> BuildCatalogDigest {
        self.catalog_digest
    }
    #[must_use]
    pub const fn build_digest(&self) -> CombatantBuildDigest {
        self.build_digest
    }
    #[must_use]
    pub const fn combatant_digest(&self) -> CombatantSpecDigest {
        self.combatant_digest
    }
    pub fn verify(
        &self,
        catalog: &BuildCatalog,
        compiled: &CompiledBuild,
    ) -> Result<(), BuildLockError> {
        if self.catalog_digest != catalog.digest() {
            return Err(BuildLockError::CatalogMismatch);
        }
        if self.build_digest != compiled.build_digest {
            return Err(BuildLockError::BuildMismatch);
        }
        if self.combatant_digest != compiled.combatant.digest() {
            return Err(BuildLockError::CombatantMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildLockError {
    CatalogMismatch,
    BuildMismatch,
    CombatantMismatch,
}
