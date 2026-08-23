//! Catalog-owned contributions selected by modes before combat assembly.

use starclock_combat::{UnitDefinitionId, rule::model::RuleSource};

use crate::{id::BuildContributionId, light_cone::CombatPath, patch::BuildPatch};

/// Generic build-side applicability retained by an authored contribution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildContributionApplicability {
    Any,
    Form(UnitDefinitionId),
    Path(CombatPath),
}

/// One immutable equipment, progression, or mode contribution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildContributionDefinition {
    id: BuildContributionId,
    source: RuleSource,
    applicability: BuildContributionApplicability,
    patches: Box<[BuildPatch]>,
}

impl BuildContributionDefinition {
    #[must_use]
    pub fn new(
        id: BuildContributionId,
        source: RuleSource,
        applicability: BuildContributionApplicability,
        patches: Vec<BuildPatch>,
    ) -> Option<Self> {
        if patches.is_empty() {
            return None;
        }
        Some(Self {
            id,
            source,
            applicability,
            patches: patches.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn id(&self) -> BuildContributionId {
        self.id
    }
    #[must_use]
    pub const fn source(&self) -> &RuleSource {
        &self.source
    }
    #[must_use]
    pub const fn applicability(&self) -> BuildContributionApplicability {
        self.applicability
    }
    #[must_use]
    pub fn patches(&self) -> &[BuildPatch] {
        &self.patches
    }

    #[must_use]
    pub fn applies_to(&self, form: UnitDefinitionId, path: CombatPath) -> bool {
        match self.applicability {
            BuildContributionApplicability::Any => true,
            BuildContributionApplicability::Form(required) => required == form,
            BuildContributionApplicability::Path(required) => required == path,
        }
    }
}
