//! Production execution bindings for the 23 frozen semantic fixture families.

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::{
        SwarmDisasterContentCatalog, semantic_access::SemanticFixtureRuntimeInput,
        types::ReviewFixtureQuality,
    },
};

use super::SwarmDisasterRuntimeFactory;

const REVISION: &str = "swarm-disaster-semantic-fixture-execution-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum ExecutionKind {
    ProductionRuntime = 0,
    ProductionCatalogProbe = 1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FixtureBinding {
    fixture_id: Box<str>,
    family_id: Box<str>,
    production_regression: Box<str>,
    execution_kind: ExecutionKind,
    quality: ReviewFixtureQuality,
    source_record_count: usize,
    ordered_operation_count: usize,
    expected_fact_count: usize,
}

type CompiledFixtureBindings = (
    Box<[FixtureBinding]>,
    Vec<SemanticFixtureRuntimeInput>,
    (usize, usize, usize, usize),
);

#[derive(Clone, Debug)]
pub(super) struct SemanticFixtureRuntimeCatalog {
    digest: [u8; 32],
}

impl SemanticFixtureRuntimeCatalog {
    pub(super) fn compile(
        content: &SwarmDisasterContentCatalog,
    ) -> Result<Self, UniverseCatalogLoadError> {
        let (bindings, inputs, encounter_shape) = fixture_bindings(content)?;
        let digest = execution_digest(&bindings, &inputs, encounter_shape);
        Ok(Self { digest })
    }
}

fn fixture_bindings(
    content: &SwarmDisasterContentCatalog,
) -> Result<CompiledFixtureBindings, UniverseCatalogLoadError> {
    let mut inputs = content.semantic_fixture_runtime_inputs().into_vec();
    inputs.sort_unstable_by(|left, right| left.key.cmp(&right.key));
    let expected = expected_bindings();
    if inputs.len() != expected.len() {
        return Err(reference("semantic fixture denominator drift"));
    }
    let bindings = inputs
        .iter()
        .zip(expected)
        .map(|(input, expected)| bind(input, expected))
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    let encounter_shape = content
        .encounter_fixture_shape()
        .ok_or_else(|| reference("encounter semantic fixture source drift"))?;
    Ok((bindings, inputs, encounter_shape))
}

impl SwarmDisasterRuntimeFactory {
    /// Digest binding every frozen semantic fixture to its production regression.
    #[must_use]
    pub fn semantic_fixture_execution_digest(&self) -> [u8; 32] {
        self.semantic_fixtures.digest
    }
}

#[derive(Clone, Copy)]
struct ExpectedBinding {
    family_id: &'static str,
    production_regression: &'static str,
    execution_kind: ExecutionKind,
    quality: ReviewFixtureQuality,
    source_records: usize,
    operations: usize,
    facts: usize,
}

const fn expected_bindings() -> [ExpectedBinding; 23] {
    use ExecutionKind::{ProductionCatalogProbe, ProductionRuntime};
    use ReviewFixtureQuality::{ExactStructured, ProjectPolicy};
    [
        expected(
            "audience-die-passive",
            "selected_path_passive_initializes_once_through_activity_state",
            ProductionRuntime,
            ProjectPolicy,
            2,
            4,
            5,
        ),
        expected(
            "beacon-copy-and-blanking",
            "domain_beacon_copy_and_blanking_execute_atomically",
            ProductionRuntime,
            ProjectPolicy,
            7,
            4,
            5,
        ),
        expected(
            "boss-choice-consequence",
            "released_boss_choices_remain_stable_and_fixture_choice_compiles",
            ProductionRuntime,
            ProjectPolicy,
            2,
            3,
            4,
        ),
        expected(
            "boss-decay-stack",
            "boss_decay_thresholds_stack_once_and_gate_plane_completion",
            ProductionRuntime,
            ProjectPolicy,
            3,
            4,
            5,
        ),
        expected(
            "communing-choice",
            "choice_execution_updates_one_aeon_once_and_rejects_stale_program",
            ProductionRuntime,
            ProjectPolicy,
            2,
            4,
            5,
        ),
        expected(
            "communing-dimension-points",
            "dimension_adjustments_clamp_in_order_and_unlock_cabinet_edges_once",
            ProductionRuntime,
            ProjectPolicy,
            2,
            4,
            5,
        ),
        expected(
            "communing-trail-effect",
            "exact_trail_selection_routes_activity_and_battle_contributions_once",
            ProductionRuntime,
            ExactStructured,
            2,
            4,
            5,
        ),
        expected(
            "countdown-lifecycle",
            "countdown_enters_disarray_once_and_projects_capped_modifiers",
            ProductionRuntime,
            ProjectPolicy,
            1,
            4,
            5,
        ),
        expected(
            "curio-lifecycle",
            "charged_curio_transitions_once_and_rejects_stale_use",
            ProductionRuntime,
            ProjectPolicy,
            3,
            3,
            4,
        ),
        expected(
            "dice-face-targeting",
            "seeded_random_activation_freezes_state_and_rng_hash",
            ProductionRuntime,
            ProjectPolicy,
            2,
            4,
            5,
        ),
        expected(
            "dice-roll-reroll-cheat",
            "seeded_control_sequence_freezes_state_and_rng_hash",
            ProductionRuntime,
            ProjectPolicy,
            4,
            4,
            5,
        ),
        expected(
            "domain-replacement",
            "replacement_domain_copy_and_blanking_preserve_explicit_beacon_state",
            ProductionRuntime,
            ProjectPolicy,
            2,
            3,
            4,
        ),
        expected(
            "encounter-selection",
            "all_formal_difficulties_share_one_immutable_encounter_contract",
            ProductionCatalogProbe,
            ProjectPolicy,
            6,
            4,
            5,
        ),
        expected(
            "final-boss-consequence",
            "final_boss_inputs_reuse_decay_selection_and_explicit_choice_programs",
            ProductionRuntime,
            ProjectPolicy,
            6,
            4,
            5,
        ),
        expected(
            "occurrence-choice",
            "exact_fixture_variant_and_choice_route_through_the_existing_catalog",
            ProductionRuntime,
            ExactStructured,
            2,
            4,
            5,
        ),
        expected(
            "path-and-propagation-unlock",
            "propagation_selection_requires_the_exact_released_unlock_binding",
            ProductionRuntime,
            ProjectPolicy,
            3,
            3,
            4,
        ),
        expected(
            "pathstrider-progress",
            "pathstrider_progress_routes_nondecreasing_unlocks_exactly_once",
            ProductionRuntime,
            ProjectPolicy,
            3,
            4,
            5,
        ),
        expected(
            "planar-disarray-transition",
            "accepted_moves_enter_disarray_and_cap_level_twenty_modifiers",
            ProductionRuntime,
            ProjectPolicy,
            1,
            4,
            5,
        ),
        expected(
            "profile-entry",
            "all_five_formal_entries_execute_once_and_stale_programs_reject_atomically",
            ProductionRuntime,
            ProjectPolicy,
            12,
            3,
            4,
        ),
        expected(
            "resonance-interplay",
            "interplays_activate_in_stable_order_once_and_reject_stale_programs",
            ProductionRuntime,
            ProjectPolicy,
            2,
            4,
            5,
        ),
        expected(
            "service-and-adventure",
            "fixture_service_beacon_and_external_adventure_bind_to_production_catalogs",
            ProductionRuntime,
            ProjectPolicy,
            3,
            3,
            4,
        ),
        expected(
            "topology-event-order",
            "topology_generation_and_exact_event_order_delegate_to_existing_runtime",
            ProductionRuntime,
            ExactStructured,
            2,
            3,
            4,
        ),
        expected(
            "topology-generation",
            "compiles_canonical_bounded_three_plane_topology",
            ProductionRuntime,
            ProjectPolicy,
            4,
            4,
            5,
        ),
    ]
}

const fn expected(
    family_id: &'static str,
    production_regression: &'static str,
    execution_kind: ExecutionKind,
    quality: ReviewFixtureQuality,
    source_records: usize,
    operations: usize,
    facts: usize,
) -> ExpectedBinding {
    ExpectedBinding {
        family_id,
        production_regression,
        execution_kind,
        quality,
        source_records,
        operations,
        facts,
    }
}

fn bind(
    input: &SemanticFixtureRuntimeInput,
    expected: ExpectedBinding,
) -> Result<FixtureBinding, UniverseCatalogLoadError> {
    let operation_count = array_len(&input.ordered_operations)?;
    let fact_count = array_len(&input.expected_facts)?;
    let fixture_id = format!("swarm-disaster.fixture.{}", expected.family_id);
    if input.key.as_ref() != fixture_id
        || input.family.as_ref() != expected.family_id
        || input.quality != expected.quality
        || input.source_record_keys.len() != expected.source_records
        || operation_count != expected.operations
        || fact_count != expected.facts
    {
        return Err(reference("semantic fixture binding drift"));
    }
    Ok(FixtureBinding {
        fixture_id: input.key.clone(),
        family_id: input.family.clone(),
        production_regression: expected.production_regression.into(),
        execution_kind: expected.execution_kind,
        quality: expected.quality,
        source_record_count: expected.source_records,
        ordered_operation_count: expected.operations,
        expected_fact_count: expected.facts,
    })
}

fn array_len(value: &str) -> Result<usize, UniverseCatalogLoadError> {
    serde_json::from_str::<Vec<serde_json::Value>>(value)
        .map(|values| values.len())
        .map_err(|_| reference("invalid semantic fixture array"))
}

fn execution_digest(
    bindings: &[FixtureBinding],
    inputs: &[SemanticFixtureRuntimeInput],
    encounter_shape: (usize, usize, usize, usize),
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster-semantic-fixture-execution.v1");
    encoder.text(REVISION);
    encoder.u32(bindings.len() as u32);
    for (binding, input) in bindings.iter().zip(inputs) {
        encoder.text(&binding.fixture_id);
        encoder.text(&binding.family_id);
        encoder.text(&binding.production_regression);
        encoder.u8(binding.execution_kind as u8);
        encoder.u8(match binding.quality {
            ReviewFixtureQuality::ExactStructured => 0,
            ReviewFixtureQuality::ProjectPolicy => 1,
        });
        for source in &input.source_record_keys {
            encoder.text(source);
        }
        encoder.text(&input.preconditions);
        encoder.text(&input.input);
        encoder.text(&input.ordered_operations);
        encoder.text(&input.expected_facts);
    }
    encoder.u32(encounter_shape.0 as u32);
    encoder.u32(encounter_shape.1 as u32);
    encoder.u32(encounter_shape.2 as u32);
    encoder.u32(encounter_shape.3 as u32);
    encoder.finish()
}

fn reference(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}

#[cfg(test)]
#[path = "semantic_fixture_runtime_tests.rs"]
mod tests;
