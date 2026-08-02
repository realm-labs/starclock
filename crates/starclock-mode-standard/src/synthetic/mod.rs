//! Deterministic Standard-shaped battle fixture used by the Phase 3 vertical slice.

mod catalog;
mod spec;

use std::sync::Arc;

use starclock_combat::{
    AssemblyDigest, Battle, BattleBuildError, BattleSeed, BattleSpec, EncounterId,
    catalog::CombatCatalog,
};

/// Scenario key reserved for the Phase 3 deterministic vertical slice.
pub const SYNTHETIC_STANDARD_SCENARIO_ID: &str = "synthetic-standard";
/// Configuration digest of the immutable synthetic definitions below.
pub const SYNTHETIC_STANDARD_CONFIG_DIGEST: [u8; 32] = [0xa1; 32];
/// Exact synthetic battle-spec digest.
pub const SYNTHETIC_STANDARD_SPEC_DIGEST: [u8; 32] = [0xb1; 32];

/// Phase 3 smoke-only Standard profile with no hidden clock, score or mode rule.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SyntheticStandardProfile;

impl SyntheticStandardProfile {
    /// Instantiates the immutable one-wave fixture for one master seed.
    #[must_use]
    pub fn instantiate(self, master_seed: u64) -> SyntheticStandardBattle {
        SyntheticStandardBattle {
            catalog: catalog::catalog(),
            spec: spec::battle_spec(),
            seed: spec::battle_seed(master_seed),
            master_seed,
        }
    }
}

/// Complete low-level battle handoff produced by the synthetic profile.
#[derive(Debug)]
pub struct SyntheticStandardBattle {
    catalog: Arc<CombatCatalog>,
    spec: BattleSpec,
    seed: BattleSeed,
    master_seed: u64,
}

impl SyntheticStandardBattle {
    /// Builds a fresh isolated aggregate for run or one-shot replay.
    pub fn create_battle(&self) -> Result<Battle, BattleBuildError> {
        Battle::create(Arc::clone(&self.catalog), self.spec.clone(), self.seed)
    }

    /// Returns the exact CLI/replay master seed.
    #[must_use]
    pub const fn master_seed(&self) -> u64 {
        self.master_seed
    }

    /// Returns the immutable encounter definition selected by the profile.
    #[must_use]
    pub const fn encounter(&self) -> EncounterId {
        self.spec.encounter()
    }

    /// Returns the exact battle-spec digest.
    #[must_use]
    pub const fn assembly_digest(&self) -> AssemblyDigest {
        self.spec.assembly_digest()
    }

    /// Returns the exact configuration digest represented by the catalog.
    #[must_use]
    pub fn config_digest(&self) -> [u8; 32] {
        self.catalog.digest().bytes()
    }
}

#[cfg(test)]
mod tests {
    use starclock_combat::TeamSide;

    use super::*;

    #[test]
    fn profile_has_only_explicit_standard_defaults_and_rebuilds_identically() {
        let left = SyntheticStandardProfile.instantiate(7);
        let right = SyntheticStandardProfile.instantiate(7);
        let left_battle = left.create_battle().unwrap();
        let right_battle = right.create_battle().unwrap();
        assert_eq!(left.master_seed(), 7);
        assert_eq!(left.encounter().get(), 1);
        assert_eq!(left.assembly_digest(), right.assembly_digest());
        assert_eq!(left.config_digest(), SYNTHETIC_STANDARD_CONFIG_DIGEST);
        assert_eq!(left_battle.state_hash(), right_battle.state_hash());
        assert_eq!(left_battle.view().encounter().total_waves(), 1);
        assert_eq!(left_battle.view().team(TeamSide::Player).skill_points(), 0);
    }
}
