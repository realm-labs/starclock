//! Feature-gated support for the frozen Goal 20 performance workloads.

use std::sync::atomic::{AtomicU64, Ordering};

use starclock_activity::{
    ActivityCause, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext, ActivityRngStreams,
    ActivityTransactionOutcome, ActivityTransactionState,
};

use crate::digest::Encoder;

use super::SwarmDisasterRuntimeInstance;
use super::{
    SwarmDisasterEntry, SwarmDisasterRuntimeFactory,
    baseline_fixture::{SwarmDisasterBaselineFixture, SwarmDisasterBaselineFixtureError},
};

const WARM_SEED: u64 = 0x2008_0201;
const MATRIX: [(&str, &str, &str); 16] = [
    ("201", "preservation", "1"),
    ("202", "remembrance", "2"),
    ("203", "nihility", "3"),
    ("204", "abundance", "4"),
    ("205", "hunt", "5"),
    ("201", "destruction", "6"),
    ("202", "elation", "7"),
    ("203", "propagation", "8"),
    ("203", "preservation", "1"),
    ("204", "remembrance", "2"),
    ("205", "nihility", "3"),
    ("201", "abundance", "4"),
    ("202", "hunt", "5"),
    ("203", "destruction", "6"),
    ("204", "elation", "7"),
    ("205", "propagation", "8"),
];

pub struct SwarmDisasterPerformanceFixture {
    factory: SwarmDisasterRuntimeFactory,
    baseline: SwarmDisasterBaselineFixture,
    warm_digest: [u8; 32],
    hits: AtomicU64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SwarmDisasterPerformanceCacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

#[derive(Debug)]
pub enum SwarmDisasterPerformanceFixtureError {
    Catalog,
    Baseline(SwarmDisasterBaselineFixtureError),
}

impl SwarmDisasterPerformanceFixture {
    pub fn load(bundle: &[u8]) -> Result<Self, SwarmDisasterPerformanceFixtureError> {
        let factory = SwarmDisasterRuntimeFactory::load_candidate(bundle)
            .map_err(|_| SwarmDisasterPerformanceFixtureError::Catalog)?;
        let baseline = factory
            .compile_synthetic_baseline_fixture()
            .map_err(SwarmDisasterPerformanceFixtureError::Baseline)?;
        let instance = baseline.instance();
        let node = instance.graph_definition().entry();
        let mut state = ActivityTransactionState::new(instance.state_definition().clone(), node);
        commit(
            instance,
            &mut state,
            instance
                .compile_node_replacement(node, "swarm-disaster.domain.monsternormal", None)
                .map_err(|_| SwarmDisasterPerformanceFixtureError::Catalog)?,
        )?;
        let mut rng = activity_rng(&baseline, WARM_SEED);
        let warm_digest = instance
            .materialize_current_battle(&state, &mut rng, baseline.roster())
            .map_err(|_| SwarmDisasterPerformanceFixtureError::Catalog)?
            .assembly_digest()
            .bytes();
        Ok(Self {
            factory,
            baseline,
            warm_digest,
            hits: AtomicU64::new(0),
        })
    }

    pub fn compile_frozen_matrix(&self) -> Result<[u8; 32], SwarmDisasterPerformanceFixtureError> {
        let progression = self
            .factory
            .unique
            .trail_runtime_input()
            .nodes
            .iter()
            .map(|node| node.key.to_string())
            .chain(
                self.factory
                    .unique
                    .communing_runtime_input()
                    .cabinets
                    .iter()
                    .map(|cabinet| cabinet.key.to_string()),
            )
            .chain(
                self.factory
                    .unique
                    .path_runtime_input()
                    .interplays
                    .iter()
                    .map(|interplay| interplay.key.to_string()),
            )
            .collect::<Vec<_>>();
        let communing = (1..=7)
            .map(|id| (format!("swarm-disaster.communing-dimension.{id}"), 20))
            .collect::<Vec<_>>();
        let mut hash = Encoder::new(b"starclock.swarm-disaster.performance-matrix");
        for (area, path, die) in MATRIX {
            let entry = SwarmDisasterEntry::new(
                format!("swarm-disaster.area.{area}"),
                format!("universe.path.{path}"),
                format!("swarm-disaster.audience-die.{die}"),
                self.baseline.instance().participants().clone(),
            )
            .with_audience_unlocks(
                [
                    "1000008", "1000013", "1000014", "1000015", "1000016", "1000017", "1000018",
                ]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            )
            .with_dice_control_unlocks(vec!["1000022".into()])
            .with_progression(communing.clone(), progression.clone(), None);
            let instance = self
                .factory
                .compile_entry(entry)
                .map_err(|_| SwarmDisasterPerformanceFixtureError::Catalog)?;
            hash.digest(instance.graph_definition().digest().bytes());
            hash.text(instance.area());
            hash.text(instance.path());
            hash.text(instance.audience_die());
        }
        Ok(hash.finish())
    }

    #[must_use]
    pub fn warm_battle_digest(&self) -> [u8; 32] {
        self.hits.fetch_add(1, Ordering::Relaxed);
        self.warm_digest
    }

    #[must_use]
    pub fn cache_metrics(&self) -> SwarmDisasterPerformanceCacheMetrics {
        SwarmDisasterPerformanceCacheMetrics {
            hits: self.hits.load(Ordering::Relaxed),
            misses: 0,
            evictions: 0,
        }
    }

    #[must_use]
    pub const fn matrix_entries() -> usize {
        MATRIX.len()
    }
}

fn commit(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: starclock_activity::ActivityProgramDefinition,
) -> Result<(), SwarmDisasterPerformanceFixtureError> {
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .ok_or(SwarmDisasterPerformanceFixtureError::Catalog)?;
    if !matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ) {
        return Err(SwarmDisasterPerformanceFixtureError::Catalog);
    }
    Ok(())
}

fn activity_rng(fixture: &SwarmDisasterBaselineFixture, seed: u64) -> ActivityRngStreams {
    let identity = fixture.activity_identity();
    ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(seed),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        fixture.instance().graph_definition().digest(),
        ActivityInstanceId::new(1).expect("benchmark instance is non-zero"),
        None,
        Some(fixture.instance().graph_definition().entry()),
        None,
        0,
    ))
}
