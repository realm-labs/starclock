//! Versioned Standard-shaped benchmark fixtures with no production content claim.

mod catalog;
mod spec;

use std::sync::Arc;

use starclock_combat::{
    Battle, BattleBuildError, BattleSeed, BattleSpec, BattleSpecDigest, EncounterId,
    catalog::CombatCatalog,
};

/// Exact workload definition revision bound into reports and ceilings.
pub const BENCHMARK_WORKLOAD_REVISION: &str = "g01-phase4-full-kernel-v1";
/// Catalog revision shared by every isolated job.
pub const BENCHMARK_CATALOG_REVISION: &str = "g01-phase4-benchmark-catalog-v1";
/// Rules revision for the synthetic benchmark battles.
pub const BENCHMARK_RULES_REVISION: &str = "g01-phase4-benchmark-rules-v1";
/// Configuration digest of the fixed benchmark catalog.
pub const BENCHMARK_CONFIG_DIGEST: [u8; 32] = [0xd2; 32];

/// Fixed scenario shapes exercised by the harness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BenchmarkScenario {
    /// One player and one enemy with a structural no-op Basic.
    Ordinary,
    /// One player executes eight one-damage hits as a pre-trigger-kernel proxy.
    TriggerHeavyProxy,
    /// Formula, HP, effect and resource operations in one retained action.
    FullKernel,
    /// Two-combatant state-hash input.
    HashSmall,
    /// Four-combatant state-hash input.
    HashMedium,
    /// Eight-combatant state-hash input.
    HashLarge,
}

impl BenchmarkScenario {
    /// Stable report key.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Ordinary => "ordinary-apply-v1",
            Self::TriggerHeavyProxy => "trigger-heavy-proxy-v1",
            Self::FullKernel => "full-kernel-apply-v1",
            Self::HashSmall => "hash-small-v1",
            Self::HashMedium => "hash-medium-v1",
            Self::HashLarge => "hash-large-v1",
        }
    }

    const fn code(self) -> u8 {
        match self {
            Self::Ordinary => 1,
            Self::TriggerHeavyProxy => 2,
            Self::HashSmall => 3,
            Self::HashMedium => 4,
            Self::HashLarge => 5,
            Self::FullKernel => 6,
        }
    }
}

/// Immutable factory whose catalog `Arc` is shared across isolated battles.
#[derive(Clone, Debug)]
pub struct BenchmarkFactory {
    catalog: Arc<CombatCatalog>,
}

impl Default for BenchmarkFactory {
    fn default() -> Self {
        Self {
            catalog: catalog::catalog(),
        }
    }
}

impl BenchmarkFactory {
    /// Creates one deterministic isolated scenario handoff.
    #[must_use]
    pub fn instantiate(&self, scenario: BenchmarkScenario, master_seed: u64) -> BenchmarkBattle {
        BenchmarkBattle {
            catalog: Arc::clone(&self.catalog),
            spec: spec::battle_spec(scenario),
            seed: spec::battle_seed(scenario, master_seed),
            master_seed,
            scenario,
        }
    }
}

/// Complete low-level benchmark battle handoff.
#[derive(Debug)]
pub struct BenchmarkBattle {
    catalog: Arc<CombatCatalog>,
    spec: BattleSpec,
    seed: BattleSeed,
    master_seed: u64,
    scenario: BenchmarkScenario,
}

impl BenchmarkBattle {
    /// Builds a fresh job while retaining the factory's immutable catalog.
    pub fn create_battle(&self) -> Result<Battle, BattleBuildError> {
        Battle::create(Arc::clone(&self.catalog), self.spec.clone(), self.seed)
    }

    /// Returns the scenario shape.
    #[must_use]
    pub const fn scenario(&self) -> BenchmarkScenario {
        self.scenario
    }

    /// Returns the master seed recorded by replay headers.
    #[must_use]
    pub const fn master_seed(&self) -> u64 {
        self.master_seed
    }

    /// Returns the encounter identity.
    #[must_use]
    pub const fn encounter(&self) -> EncounterId {
        self.spec.encounter()
    }

    /// Returns the battle-spec digest.
    #[must_use]
    pub const fn spec_digest(&self) -> BattleSpecDigest {
        self.spec.digest()
    }
}
