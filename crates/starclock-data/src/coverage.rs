//! Goal-aware coverage projection over validated production Sora identities.

use crate::{
    catalog::SimulationCatalog,
    generated::{content_kind::ContentKind, coverage_state::CoverageState},
};

/// Frozen Goal 01 denominator category.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GoalCoverageCategory {
    ReleasedCharacterCombatForms,
    ReleasedLightCones,
    StandardEnemyVariants,
    StandardEncounters,
    StandardScenarios,
    StandardProfile,
}

impl GoalCoverageCategory {
    pub const ALL: [Self; 6] = [
        Self::ReleasedCharacterCombatForms,
        Self::ReleasedLightCones,
        Self::StandardEnemyVariants,
        Self::StandardEncounters,
        Self::StandardScenarios,
        Self::StandardProfile,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ReleasedCharacterCombatForms => "released-character-combat-forms",
            Self::ReleasedLightCones => "released-light-cones",
            Self::StandardEnemyVariants => "standard-v1-enemy-variants",
            Self::StandardEncounters => "standard-v1-encounters",
            Self::StandardScenarios => "standard-v1-scenarios",
            Self::StandardProfile => "standard-v1-profile",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.name() == value)
    }

    pub(crate) const fn from_content_kind(kind: ContentKind) -> Option<Self> {
        match kind {
            ContentKind::CharacterForm => Some(Self::ReleasedCharacterCombatForms),
            ContentKind::LightCone => Some(Self::ReleasedLightCones),
            ContentKind::EnemyVariant => Some(Self::StandardEnemyVariants),
            ContentKind::Encounter => Some(Self::StandardEncounters),
            ContentKind::Scenario => Some(Self::StandardScenarios),
            ContentKind::StandardProfile => Some(Self::StandardProfile),
            _ => None,
        }
    }
}

/// Terminal coverage state carried by one Sora identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GoalCoverageState {
    Cataloged,
    Documented,
    Researching,
    DataReady,
    GoldenVerified,
    Disabled,
}

impl GoalCoverageState {
    pub(crate) const fn from_generated(value: CoverageState) -> Self {
        match value {
            CoverageState::Cataloged => Self::Cataloged,
            CoverageState::Documented => Self::Documented,
            CoverageState::Researching => Self::Researching,
            CoverageState::DataReady => Self::DataReady,
            CoverageState::GoldenVerified => Self::GoldenVerified,
            CoverageState::Disabled => Self::Disabled,
        }
    }
}

/// Counts for one exact frozen category.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoalCoverageCategorySummary {
    category: GoalCoverageCategory,
    required: usize,
    enabled: usize,
    data_ready: usize,
    golden_verified: usize,
}

impl GoalCoverageCategorySummary {
    #[must_use]
    pub const fn category(self) -> GoalCoverageCategory {
        self.category
    }
    #[must_use]
    pub const fn required(self) -> usize {
        self.required
    }
    #[must_use]
    pub const fn enabled(self) -> usize {
        self.enabled
    }
    #[must_use]
    pub const fn data_ready(self) -> usize {
        self.data_ready
    }
    #[must_use]
    pub const fn golden_verified(self) -> usize {
        self.golden_verified
    }
}

/// Complete frozen-denominator view derived only from validated Sora rows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoalCoverageReport {
    manifest_digest: Box<str>,
    categories: Box<[GoalCoverageCategorySummary]>,
}

impl GoalCoverageReport {
    #[must_use]
    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }
    #[must_use]
    pub fn categories(&self) -> &[GoalCoverageCategorySummary] {
        &self.categories
    }
    #[must_use]
    pub fn category(&self, category: GoalCoverageCategory) -> GoalCoverageCategorySummary {
        self.categories[category as usize]
    }
    #[must_use]
    pub fn required(&self) -> usize {
        self.categories.iter().map(|row| row.required).sum()
    }
    #[must_use]
    pub fn enabled(&self) -> usize {
        self.categories.iter().map(|row| row.enabled).sum()
    }
    #[must_use]
    pub fn data_ready(&self) -> usize {
        self.categories.iter().map(|row| row.data_ready).sum()
    }
    #[must_use]
    pub fn golden_verified(&self) -> usize {
        self.categories.iter().map(|row| row.golden_verified).sum()
    }
}

impl SimulationCatalog {
    /// Computes frozen Goal 01 coverage from production Sora identities.
    #[must_use]
    pub fn goal_coverage(&self) -> GoalCoverageReport {
        let mut categories =
            GoalCoverageCategory::ALL.map(|category| GoalCoverageCategorySummary {
                category,
                required: 0,
                enabled: 0,
                data_ready: 0,
                golden_verified: 0,
            });
        for identity in &self.identities {
            let Some(category) = identity.goal_category else {
                continue;
            };
            let row = &mut categories[category as usize];
            row.required += 1;
            row.enabled += usize::from(identity.enabled);
            row.data_ready += usize::from(matches!(
                identity.coverage_state,
                GoalCoverageState::DataReady | GoalCoverageState::GoldenVerified
            ));
            row.golden_verified += usize::from(matches!(
                identity.coverage_state,
                GoalCoverageState::GoldenVerified
            ));
        }
        GoalCoverageReport {
            manifest_digest: self
                .manifest()
                .coverage_manifest_sha256
                .clone()
                .into_boxed_str(),
            categories: Box::new(categories),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRODUCTION_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");

    #[test]
    fn production_bundle_matches_the_frozen_goal_denominator() {
        let catalog = crate::catalog::load(PRODUCTION_BUNDLE).unwrap();
        let report = catalog.goal_coverage();
        assert_eq!(report.required(), 283);
        assert_eq!(report.enabled(), 166);
        assert_eq!(report.data_ready(), 166);
        assert_eq!(report.golden_verified(), 166);
        assert_eq!(
            report
                .category(GoalCoverageCategory::ReleasedCharacterCombatForms)
                .required(),
            88
        );
        assert_eq!(
            report
                .category(GoalCoverageCategory::ReleasedCharacterCombatForms)
                .data_ready(),
            88
        );
        assert_eq!(
            report
                .category(GoalCoverageCategory::ReleasedLightCones)
                .required(),
            165
        );
        assert_eq!(
            report
                .category(GoalCoverageCategory::ReleasedLightCones)
                .data_ready(),
            48
        );
        assert_eq!(
            report
                .category(GoalCoverageCategory::StandardEnemyVariants)
                .required(),
            17
        );
        assert_eq!(
            report
                .category(GoalCoverageCategory::StandardEncounters)
                .required(),
            6
        );
        assert_eq!(
            report
                .category(GoalCoverageCategory::StandardScenarios)
                .required(),
            6
        );
        assert_eq!(
            report
                .category(GoalCoverageCategory::StandardProfile)
                .required(),
            1
        );
        for category in [
            GoalCoverageCategory::StandardEnemyVariants,
            GoalCoverageCategory::StandardEncounters,
            GoalCoverageCategory::StandardScenarios,
            GoalCoverageCategory::StandardProfile,
        ] {
            let summary = report.category(category);
            assert_eq!(summary.enabled(), summary.required());
            assert_eq!(summary.data_ready(), summary.required());
            assert_eq!(summary.golden_verified(), summary.required());
        }
    }
}
