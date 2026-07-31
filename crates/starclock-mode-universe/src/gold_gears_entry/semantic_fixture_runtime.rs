//! Production execution bindings for the 18 frozen semantic fixture families.

use crate::{digest::Encoder, gold_gears_content::GoldAndGearsContentCatalog};

use super::{
    GoldAndGearsEntryError,
    api::{GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance},
};

pub const GOLD_AND_GEARS_SEMANTIC_FIXTURE_EXECUTION_REVISION: &str =
    "gold-and-gears-semantic-fixture-execution-v1";

const ENCOUNTER_FIXTURE_GROUP: &str = "gold-gears.encounter-group.223003";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsSemanticFixtureExecutionKind {
    ProductionRuntime = 0,
    ProductionCatalogProbe = 1,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsSemanticFixtureProbe {
    CognitionLifecycle = 0,
    ConundrumAuxiliary = 1,
    ConundrumStats = 2,
    CurioLifecycle = 3,
    CustomDicePassive = 4,
    DiceFaceTargeting = 5,
    DiceRerollAndCheat = 6,
    EncounterSelection = 7,
    KnowledgeLifecycle = 8,
    NeuralNetworkEffect = 9,
    OccurrenceChoice = 10,
    PathBoost = 11,
    ProfileEntry = 12,
    ResonanceExtrapolation = 13,
    SecretThreshold = 14,
    ServiceAndAdventure = 15,
    TopologyEventOrder = 16,
    TopologyGeneration = 17,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsSemanticFixtureBinding {
    fixture_id: Box<str>,
    family_id: Box<str>,
    production_regression: Box<str>,
    probe: GoldAndGearsSemanticFixtureProbe,
    execution_kind: GoldAndGearsSemanticFixtureExecutionKind,
}

impl GoldAndGearsSemanticFixtureBinding {
    #[must_use]
    pub fn fixture_id(&self) -> &str {
        &self.fixture_id
    }

    #[must_use]
    pub fn family_id(&self) -> &str {
        &self.family_id
    }

    #[must_use]
    pub fn production_regression(&self) -> &str {
        &self.production_regression
    }

    #[must_use]
    pub const fn probe(&self) -> GoldAndGearsSemanticFixtureProbe {
        self.probe
    }

    #[must_use]
    pub const fn execution_kind(&self) -> GoldAndGearsSemanticFixtureExecutionKind {
        self.execution_kind
    }
}

#[derive(Clone, Debug)]
pub(super) struct GoldAndGearsSemanticFixtureRuntimeCatalog {
    bindings: Box<[GoldAndGearsSemanticFixtureBinding]>,
    encounter_shape: (usize, usize),
    digest: [u8; 32],
}

impl GoldAndGearsSemanticFixtureRuntimeCatalog {
    pub(super) fn compile(
        content: &GoldAndGearsContentCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let expected = expected_bindings();
        let mut fixture_keys = content.review_fixture_keys().collect::<Vec<_>>();
        fixture_keys.sort_unstable();
        if fixture_keys.len() != expected.len()
            || !fixture_keys
                .iter()
                .zip(&expected)
                .all(|(key, binding)| **key == format!("gold-gears.fixture.{}", binding.family_id))
        {
            return Err(GoldAndGearsEntryError::InvalidSharedContentRuntime);
        }
        let encounter_shape = content
            .encounter_fixture_shape(ENCOUNTER_FIXTURE_GROUP)
            .ok_or(GoldAndGearsEntryError::InvalidSharedContentRuntime)?;
        if encounter_shape != (2, 2) {
            return Err(GoldAndGearsEntryError::InvalidSharedContentRuntime);
        }
        let bindings = expected
            .iter()
            .map(|expected| GoldAndGearsSemanticFixtureBinding {
                fixture_id: format!("gold-gears.fixture.{}", expected.family_id).into(),
                family_id: expected.family_id.into(),
                production_regression: expected.production_regression.into(),
                probe: expected.probe,
                execution_kind: expected.execution_kind,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let digest = execution_digest(&bindings, encounter_shape);
        Ok(Self {
            bindings,
            encounter_shape,
            digest,
        })
    }

    pub(super) fn bindings(&self) -> &[GoldAndGearsSemanticFixtureBinding] {
        &self.bindings
    }

    pub(super) const fn encounter_shape(&self) -> (usize, usize) {
        self.encounter_shape
    }

    pub(super) const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

impl GoldAndGearsRuntimeFactory {
    #[must_use]
    pub fn semantic_fixture_bindings(&self) -> &[GoldAndGearsSemanticFixtureBinding] {
        self.content_runtime.semantic_fixtures.bindings()
    }

    #[must_use]
    pub fn semantic_fixture_execution_digest(&self) -> [u8; 32] {
        self.content_runtime.semantic_fixtures.digest()
    }

    #[must_use]
    pub fn encounter_selection_fixture_shape(&self) -> (usize, usize) {
        self.content_runtime.semantic_fixtures.encounter_shape()
    }
}

impl GoldAndGearsRuntimeInstance {
    #[must_use]
    pub fn semantic_fixture_bindings(&self) -> &[GoldAndGearsSemanticFixtureBinding] {
        self.content_runtime.semantic_fixtures.bindings()
    }

    #[must_use]
    pub fn semantic_fixture_execution_digest(&self) -> [u8; 32] {
        self.content_runtime.semantic_fixtures.digest()
    }
}

#[derive(Clone, Copy)]
struct ExpectedBinding {
    family_id: &'static str,
    production_regression: &'static str,
    probe: GoldAndGearsSemanticFixtureProbe,
    execution_kind: GoldAndGearsSemanticFixtureExecutionKind,
}

fn expected_bindings() -> [ExpectedBinding; 18] {
    use GoldAndGearsSemanticFixtureExecutionKind::{ProductionCatalogProbe, ProductionRuntime};
    use GoldAndGearsSemanticFixtureProbe::{
        CognitionLifecycle, ConundrumAuxiliary, ConundrumStats, CurioLifecycle, CustomDicePassive,
        DiceFaceTargeting, DiceRerollAndCheat, EncounterSelection, KnowledgeLifecycle,
        NeuralNetworkEffect, OccurrenceChoice, PathBoost, ProfileEntry, ResonanceExtrapolation,
        SecretThreshold, ServiceAndAdventure, TopologyEventOrder, TopologyGeneration,
    };
    [
        expected(
            "cognition-lifecycle",
            "cognition_adjustment_clamps_and_carries_without_rng",
            CognitionLifecycle,
            ProductionRuntime,
        ),
        expected(
            "conundrum-auxiliary",
            "cumulative_start_program_executes_all_six_rule_payloads_without_rng",
            ConundrumAuxiliary,
            ProductionRuntime,
        ),
        expected(
            "conundrum-stats",
            "stats_fixture_executes_all_active_modifiers_through_combat_resolver",
            ConundrumStats,
            ProductionRuntime,
        ),
        expected(
            "curio-lifecycle",
            "all_160_curio_rules_execute_through_the_production_fixture",
            CurioLifecycle,
            ProductionRuntime,
        ),
        expected(
            "custom-dice-passive",
            "all_twelve_passives_emit_typed_operations_and_exact_immediate_values",
            CustomDicePassive,
            ProductionRuntime,
        ),
        expected(
            "dice-face-targeting",
            "authored_empty_content_face_commits_no_effect_without_rng",
            DiceFaceTargeting,
            ProductionRuntime,
        ),
        expected(
            "dice-reroll-and-cheat",
            "empty_reroll_candidates_keep_previous_consume_attempt_and_draw_nothing",
            DiceRerollAndCheat,
            ProductionRuntime,
        ),
        expected(
            "encounter-selection",
            "encounter_selection_fixture_is_catalog_bound_pending_p6_execution",
            EncounterSelection,
            ProductionCatalogProbe,
        ),
        expected(
            "knowledge-lifecycle",
            "production_programs_match_the_knowledge_lifecycle_semantic_fixture",
            KnowledgeLifecycle,
            ProductionRuntime,
        ),
        expected(
            "neural-network-effect",
            "production_program_matches_the_neural_network_effect_semantic_fixture",
            NeuralNetworkEffect,
            ProductionRuntime,
        ),
        expected(
            "occurrence-choice",
            "all_384_occurrence_rules_execute_through_the_production_fixture",
            OccurrenceChoice,
            ProductionRuntime,
        ),
        expected(
            "path-boost",
            "all_nine_path_boost_rules_execute_through_combat_modifiers",
            PathBoost,
            ProductionRuntime,
        ),
        expected(
            "profile-entry",
            "profile_entry_fixture_executes_all_five_rules_against_production_state",
            ProfileEntry,
            ProductionRuntime,
        ),
        expected(
            "resonance-extrapolation",
            "all_36_extrapolation_rules_project_with_seeded_enemy_attachment",
            ResonanceExtrapolation,
            ProductionRuntime,
        ),
        expected(
            "secret-threshold",
            "every_authored_secret_can_enter_a_valid_runtime_frontier",
            SecretThreshold,
            ProductionRuntime,
        ),
        expected(
            "service-and-adventure",
            "all_38_service_adventure_rules_execute_through_the_production_fixture",
            ServiceAndAdventure,
            ProductionRuntime,
        ),
        expected(
            "topology-event-order",
            "selected_map_event_executes_before_block_creation_and_is_rng_isolated",
            TopologyEventOrder,
            ProductionRuntime,
        ),
        expected(
            "topology-generation",
            "formal_entry_compiles_canonical_three_plane_activity_graph",
            TopologyGeneration,
            ProductionRuntime,
        ),
    ]
}

const fn expected(
    family_id: &'static str,
    production_regression: &'static str,
    probe: GoldAndGearsSemanticFixtureProbe,
    execution_kind: GoldAndGearsSemanticFixtureExecutionKind,
) -> ExpectedBinding {
    ExpectedBinding {
        family_id,
        production_regression,
        probe,
        execution_kind,
    }
}

fn execution_digest(
    bindings: &[GoldAndGearsSemanticFixtureBinding],
    encounter_shape: (usize, usize),
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-gold-gears-semantic-fixture-execution-v1");
    encoder.text(GOLD_AND_GEARS_SEMANTIC_FIXTURE_EXECUTION_REVISION);
    encoder.u32(bindings.len() as u32);
    for binding in bindings {
        encoder.text(&binding.fixture_id);
        encoder.text(&binding.family_id);
        encoder.text(&binding.production_regression);
        encoder.u8(binding.probe as u8);
        encoder.u8(binding.execution_kind as u8);
    }
    encoder.u32(encounter_shape.0 as u32);
    encoder.u32(encounter_shape.1 as u32);
    encoder.finish()
}
