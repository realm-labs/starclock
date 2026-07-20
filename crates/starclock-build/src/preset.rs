//! Named exact build selections expanded before compilation.

use crate::{digest::CombatantBuildDigest, id::BuildPresetId, spec::CombatantBuildSpec};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildPreset {
    id: BuildPresetId,
    name: Box<str>,
    spec: CombatantBuildSpec,
    expected_build_digest: Option<CombatantBuildDigest>,
}

impl BuildPreset {
    #[must_use]
    pub fn new(id: BuildPresetId, name: &str, spec: CombatantBuildSpec) -> Option<Self> {
        (!name.trim().is_empty()).then(|| Self {
            id,
            name: name.into(),
            spec,
            expected_build_digest: None,
        })
    }
    #[must_use]
    pub fn with_expected_build_digest(mut self, digest: CombatantBuildDigest) -> Self {
        self.expected_build_digest = Some(digest);
        self
    }
    #[must_use]
    pub const fn id(&self) -> BuildPresetId {
        self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn spec(&self) -> &CombatantBuildSpec {
        &self.spec
    }
    #[must_use]
    pub const fn expected_build_digest(&self) -> Option<CombatantBuildDigest> {
        self.expected_build_digest
    }
}
