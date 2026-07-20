//! Successful generic combat compilation output.

use starclock_combat::{CombatantSpecDigest, ResolvedCombatantSpec};

use crate::{
    catalog::BuildCatalog,
    digest::{BuildCatalogDigest, CombatantBuildDigest},
    report::BuildCompilationReport,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledBuild {
    combatant: ResolvedCombatantSpec,
    report: BuildCompilationReport,
    build_digest: CombatantBuildDigest,
    lock: BuildLock,
}

impl CompiledBuild {
    pub(crate) fn new(
        combatant: ResolvedCombatantSpec,
        report: BuildCompilationReport,
        build_digest: CombatantBuildDigest,
        catalog_revision: &str,
        catalog_digest: BuildCatalogDigest,
    ) -> Self {
        let lock = BuildLock {
            catalog_revision: catalog_revision.into(),
            catalog_digest,
            build_digest,
            combatant_digest: combatant.digest(),
        };
        Self {
            combatant,
            report,
            build_digest,
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
    #[must_use]
    pub const fn lock(&self) -> &BuildLock {
        &self.lock
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildLock {
    catalog_revision: Box<str>,
    catalog_digest: BuildCatalogDigest,
    build_digest: CombatantBuildDigest,
    combatant_digest: CombatantSpecDigest,
}

impl BuildLock {
    #[must_use]
    pub fn catalog_revision(&self) -> &str {
        &self.catalog_revision
    }
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
        if self.catalog_revision.as_ref() != catalog.revision().as_str()
            || self.catalog_digest != catalog.digest()
        {
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
