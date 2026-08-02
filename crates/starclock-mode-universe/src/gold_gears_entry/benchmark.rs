//! Feature-gated support for the frozen Goal 14 performance workloads.

use starclock_activity::{
    ActivityCause, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext, ActivityRngStreams,
    ActivityStateHash, ActivityTransactionOutcome, ActivityTransactionState,
};

use crate::{digest::Encoder, gold_gears_identity::GoldAndGearsCatalogIdentity};

use super::{
    GoldAndGearsBattleAssemblyCacheMetrics, GoldAndGearsBattleAssemblyContext,
    GoldAndGearsEncounterSelection, GoldAndGearsEntry, GoldAndGearsEntryError,
    GoldAndGearsRuntimeFactory,
    baseline_fixture::{GoldAndGearsBaselineFixture, GoldAndGearsBaselineFixtureError},
};

const WARM_SEED: u64 = 0x1408_0201;
const CONUNDRUM_AREA: &str = "gold-gears.area.405";

#[derive(Clone, Copy)]
struct MatrixEntry {
    area: &'static str,
    path: &'static str,
    dice: &'static str,
    stats: u8,
    auxiliary: u8,
}

const MATRIX: [MatrixEntry; 25] = [
    row("401", "abundance", "101", 0, 0),
    row("402", "destruction", "102", 0, 0),
    row("403", "elation", "103", 0, 0),
    row("404", "erudition", "201", 0, 0),
    row("405", "hunt", "202", 0, 0),
    row("401", "nihility", "203", 0, 0),
    row("402", "preservation", "301", 0, 0),
    row("403", "propagation", "302", 0, 0),
    row("404", "remembrance", "303", 0, 0),
    row("405", "abundance", "401", 0, 0),
    row("401", "destruction", "402", 0, 0),
    row("402", "elation", "403", 0, 0),
    row("405", "erudition", "203", 0, 1),
    row("405", "hunt", "301", 0, 2),
    row("405", "nihility", "302", 0, 3),
    row("405", "preservation", "303", 0, 4),
    row("405", "propagation", "401", 0, 5),
    row("405", "remembrance", "402", 0, 6),
    row("405", "abundance", "403", 1, 0),
    row("405", "destruction", "101", 2, 0),
    row("405", "elation", "102", 3, 0),
    row("405", "erudition", "103", 4, 0),
    row("405", "hunt", "201", 5, 0),
    row("405", "nihility", "202", 6, 0),
    row("405", "remembrance", "403", 6, 6),
];

const fn row(
    area: &'static str,
    path: &'static str,
    dice: &'static str,
    stats: u8,
    auxiliary: u8,
) -> MatrixEntry {
    MatrixEntry {
        area,
        path,
        dice,
        stats,
        auxiliary,
    }
}

pub struct GoldAndGearsPerformanceFixture {
    factory: GoldAndGearsRuntimeFactory,
    baseline: GoldAndGearsBaselineFixture,
    state: ActivityTransactionState,
    selection: GoldAndGearsEncounterSelection,
    context: GoldAndGearsBattleAssemblyContext,
    expected_state_hash: ActivityStateHash,
}

#[derive(Debug)]
pub enum GoldAndGearsPerformanceFixtureError {
    Catalog,
    Baseline(GoldAndGearsBaselineFixtureError),
    Runtime(GoldAndGearsEntryError),
}

impl GoldAndGearsPerformanceFixture {
    pub fn load(bundle: &[u8]) -> Result<Self, GoldAndGearsPerformanceFixtureError> {
        let identity = GoldAndGearsCatalogIdentity::load(bundle)
            .map_err(|_| GoldAndGearsPerformanceFixtureError::Catalog)?;
        let factory = GoldAndGearsRuntimeFactory::load_candidate(bundle)
            .map_err(GoldAndGearsPerformanceFixtureError::Runtime)?;
        let baseline = factory
            .compile_synthetic_baseline_fixture(&identity)
            .map_err(GoldAndGearsPerformanceFixtureError::Baseline)?;
        let instance = baseline.instance();
        let node = instance
            .encounter_runtime
            .node_at(1, 2)
            .ok_or(GoldAndGearsPerformanceFixtureError::Catalog)?;
        let mut state = ActivityTransactionState::new(instance.state_definition().clone(), node);
        commit(
            instance,
            &mut state,
            instance
                .compile_node_replacement(node, "gold-gears.domain.monsternormal", None)
                .map_err(GoldAndGearsPerformanceFixtureError::Runtime)?,
        )?;
        let mut rng = activity_rng(&baseline, WARM_SEED);
        let selection = instance
            .select_current_encounter(&state, &mut rng)
            .map_err(GoldAndGearsPerformanceFixtureError::Runtime)?;
        let expected_state_hash = state.state_hash(
            baseline.activity_identity(),
            instance.graph_definition(),
            ActivityInstanceId::new(1).expect("benchmark instance is non-zero"),
            &rng,
        );
        let context = GoldAndGearsBattleAssemblyContext::new(Vec::new(), false);
        let fixture = Self {
            factory,
            baseline,
            state,
            selection,
            context,
            expected_state_hash,
        };
        fixture.warm_battle_digest()?;
        Ok(fixture)
    }

    pub fn compile_frozen_matrix(&self) -> Result<[u8; 32], GoldAndGearsEntryError> {
        let mut hash = Encoder::new(b"starclock.gold-and-gears.performance-matrix");
        for entry in MATRIX {
            let dice = self
                .factory
                .unique
                .dice
                .iter()
                .find(|candidate| candidate.identity.stable_key.ends_with(entry.dice))
                .ok_or(GoldAndGearsEntryError::InvalidCatalog)?;
            let faces = dice
                .default_face_sources
                .iter()
                .map(|source| {
                    self.factory
                        .unique
                        .dice_faces
                        .iter()
                        .find(|face| face.identity.source_id == *source)
                        .map(|face| face.identity.stable_key.to_string())
                        .ok_or(GoldAndGearsEntryError::InvalidCatalog)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let unlocked = self
                .factory
                .unique
                .dice
                .iter()
                .map(|candidate| candidate.identity.stable_key.to_string())
                .collect();
            let area = format!("gold-gears.area.{}", entry.area);
            let path = format!("universe.path.{}", entry.path);
            let mut request = GoldAndGearsEntry::new(
                area,
                path,
                dice.identity.stable_key.to_string(),
                faces,
                self.baseline.instance().participants().as_ref().clone(),
            )
            .with_unlocked_dice(unlocked);
            if entry.stats != 0 || entry.auxiliary != 0 {
                request = request.with_conundrum(
                    entry.stats,
                    entry.auxiliary,
                    vec![CONUNDRUM_AREA.to_owned()],
                );
            }
            let compiled = self.factory.compile_entry(request)?;
            hash.digest(compiled.graph_definition().digest().bytes());
            hash.text(compiled.area());
            hash.text(compiled.path());
            hash.text(compiled.custom_dice());
            hash.u8(compiled.stats_conundrum());
            hash.u8(compiled.auxiliary_conundrum());
        }
        Ok(hash.finish())
    }

    pub fn warm_battle_digest(&self) -> Result<[u8; 32], GoldAndGearsPerformanceFixtureError> {
        self.baseline
            .instance()
            .resolve_current_battle(
                self.expected_state_hash,
                &self.state,
                &self.selection,
                self.baseline.roster(),
                &self.context,
            )
            .map(|materialization| materialization.digest())
            .map_err(GoldAndGearsPerformanceFixtureError::Runtime)
    }

    #[must_use]
    pub fn cache_metrics(&self) -> GoldAndGearsBattleAssemblyCacheMetrics {
        self.baseline.instance().battle_assembly_cache_metrics()
    }

    #[must_use]
    pub const fn matrix_entries() -> usize {
        MATRIX.len()
    }
}

fn commit(
    instance: &super::GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: starclock_activity::ActivityProgramDefinition,
) -> Result<(), GoldAndGearsPerformanceFixtureError> {
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .map_err(|_| GoldAndGearsPerformanceFixtureError::Catalog)?;
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .ok_or(GoldAndGearsPerformanceFixtureError::Catalog)?;
    if !matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ) {
        return Err(GoldAndGearsPerformanceFixtureError::Catalog);
    }
    Ok(())
}

fn activity_rng(fixture: &GoldAndGearsBaselineFixture, seed: u64) -> ActivityRngStreams {
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
