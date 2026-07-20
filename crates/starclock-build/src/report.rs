//! Stable typed build-validation evidence.

use starclock_combat::{UnitDefinitionId, UnitLevel};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BuildValidationStage {
    CatalogCompatibility = 0,
    CharacterLookup = 1,
    LevelSelection = 2,
    AbilitySelection = 3,
    TraceSelection = 4,
    CombatBindings = 5,
    CombatantConstruction = 6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildValidationOutcome {
    Passed,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildValidationEntry {
    stage: BuildValidationStage,
    outcome: BuildValidationOutcome,
}

impl BuildValidationEntry {
    pub(crate) const fn passed(stage: BuildValidationStage) -> Self {
        Self {
            stage,
            outcome: BuildValidationOutcome::Passed,
        }
    }
    pub(crate) const fn failed(stage: BuildValidationStage) -> Self {
        Self {
            stage,
            outcome: BuildValidationOutcome::Failed,
        }
    }
    #[must_use]
    pub const fn stage(self) -> BuildValidationStage {
        self.stage
    }
    #[must_use]
    pub const fn outcome(self) -> BuildValidationOutcome {
        self.outcome
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildCompilationReport {
    form: UnitDefinitionId,
    level: UnitLevel,
    entries: Box<[BuildValidationEntry]>,
}

impl BuildCompilationReport {
    pub(crate) fn new(
        form: UnitDefinitionId,
        level: UnitLevel,
        entries: Vec<BuildValidationEntry>,
    ) -> Self {
        debug_assert!(entries.windows(2).all(|pair| pair[0].stage < pair[1].stage));
        Self {
            form,
            level,
            entries: entries.into_boxed_slice(),
        }
    }
    #[must_use]
    pub const fn form(&self) -> UnitDefinitionId {
        self.form
    }
    #[must_use]
    pub const fn level(&self) -> UnitLevel {
        self.level
    }
    #[must_use]
    pub fn entries(&self) -> &[BuildValidationEntry] {
        &self.entries
    }
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.entries
            .iter()
            .all(|entry| entry.outcome == BuildValidationOutcome::Passed)
    }
}
