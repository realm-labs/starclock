//! Fail-closed production coverage for every frozen Gold and Gears obligation.

use crate::{
    digest::Encoder, gold_gears_content::GoldAndGearsContentCatalog,
    gold_gears_unique::GoldAndGearsUniqueCatalog,
};

use super::{
    GoldAndGearsEntryError, api::GoldAndGearsRuntimeFactory,
    content_link_runtime::GoldAndGearsContentRuntimeCatalog,
};

pub const GOLD_AND_GEARS_RUNTIME_COVERAGE_REVISION: &str = "gold-and-gears-runtime-coverage-v1";

const SOURCE_CATEGORY_COUNT: usize = 42;
const SOURCE_OBLIGATION_COUNT: u32 = 7_913;
const MECHANIC_RULE_COUNT: usize = 1_224;
const SEMANTIC_FIXTURE_COUNT: usize = 18;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(super) enum GoldAndGearsSourceRuntimeDisposition {
    Integrated = 0,
    SharedIntegrated = 1,
    ExternalOutcome = 2,
    Metadata = 3,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(super) enum GoldAndGearsSourceRuntimeOwner {
    Entry = 0,
    Topology = 1,
    Cognition = 2,
    DiceLoadout = 3,
    DiceResolution = 4,
    DiceFace = 5,
    Knowledge = 6,
    NeuralNetwork = 7,
    Conundrum = 8,
    Progression = 9,
    Content = 10,
    PlaneTransition = 11,
    SemanticFixture = 12,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GoldAndGearsSourceRuntimeBinding {
    category: Box<str>,
    required: u32,
    owner: GoldAndGearsSourceRuntimeOwner,
    disposition: GoldAndGearsSourceRuntimeDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldAndGearsRuntimeCoverageSummary {
    source_categories: u16,
    source_runtime_slices: u16,
    source_obligations: u32,
    integrated_obligations: u32,
    shared_integrated_obligations: u32,
    external_outcomes: u32,
    metadata_obligations: u32,
    mechanic_rules: u16,
    semantic_fixtures: u8,
    native_handlers: u8,
    digest: [u8; 32],
}

/// Stable public name for the exact runtime coverage snapshot.
pub type GoldAndGearsCoverage = GoldAndGearsRuntimeCoverageSummary;

impl GoldAndGearsRuntimeCoverageSummary {
    #[must_use]
    pub const fn source_categories(self) -> u16 {
        self.source_categories
    }

    #[must_use]
    pub const fn source_runtime_slices(self) -> u16 {
        self.source_runtime_slices
    }

    #[must_use]
    pub const fn source_obligations(self) -> u32 {
        self.source_obligations
    }

    #[must_use]
    pub const fn integrated_obligations(self) -> u32 {
        self.integrated_obligations
    }

    #[must_use]
    pub const fn shared_integrated_obligations(self) -> u32 {
        self.shared_integrated_obligations
    }

    #[must_use]
    pub const fn external_outcomes(self) -> u32 {
        self.external_outcomes
    }

    #[must_use]
    pub const fn metadata_obligations(self) -> u32 {
        self.metadata_obligations
    }

    #[must_use]
    pub const fn mechanic_rules(self) -> u16 {
        self.mechanic_rules
    }

    #[must_use]
    pub const fn semantic_fixtures(self) -> u8 {
        self.semantic_fixtures
    }

    #[must_use]
    pub const fn native_handlers(self) -> u8 {
        self.native_handlers
    }

    #[must_use]
    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone, Debug)]
pub(super) struct RuntimeCoverageCatalog {
    summary: GoldAndGearsRuntimeCoverageSummary,
}

impl RuntimeCoverageCatalog {
    pub(super) fn compile(
        content: &GoldAndGearsContentCatalog,
        unique: &GoldAndGearsUniqueCatalog,
        runtime: &GoldAndGearsContentRuntimeCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let mut source_bindings = Vec::new();
        for (category, required, accounted, data_ready, no_blocking_gaps) in
            content.runtime_coverage_rows()
        {
            if required <= 0 || required != accounted || required != data_ready || !no_blocking_gaps
            {
                return Err(GoldAndGearsEntryError::InvalidRuntimeCoverage);
            }
            let required = u32::try_from(required)
                .map_err(|_| GoldAndGearsEntryError::InvalidRuntimeCoverage)?;
            source_bindings.extend(
                category_contract(category, required)
                    .ok_or(GoldAndGearsEntryError::InvalidRuntimeCoverage)?,
            );
        }
        source_bindings.sort_unstable_by(|left, right| {
            left.category
                .cmp(&right.category)
                .then(left.disposition.cmp(&right.disposition))
        });
        let distinct_categories = source_bindings
            .iter()
            .enumerate()
            .filter(|(index, binding)| {
                *index == 0 || source_bindings[*index - 1].category != binding.category
            })
            .count();
        if source_bindings.len() != SOURCE_CATEGORY_COUNT + 2
            || distinct_categories != SOURCE_CATEGORY_COUNT
            || source_bindings.windows(2).any(|pair| {
                pair[0].category > pair[1].category
                    || (pair[0].category == pair[1].category
                        && pair[0].disposition >= pair[1].disposition)
            })
            || source_bindings
                .iter()
                .map(|binding| binding.required)
                .sum::<u32>()
                != SOURCE_OBLIGATION_COUNT
            || disposition_total(
                &source_bindings,
                GoldAndGearsSourceRuntimeDisposition::Integrated,
            ) != 7_181
            || disposition_total(
                &source_bindings,
                GoldAndGearsSourceRuntimeDisposition::SharedIntegrated,
            ) != 706
            || disposition_total(
                &source_bindings,
                GoldAndGearsSourceRuntimeDisposition::ExternalOutcome,
            ) != 8
            || disposition_total(
                &source_bindings,
                GoldAndGearsSourceRuntimeDisposition::Metadata,
            ) != 18
        {
            return Err(GoldAndGearsEntryError::InvalidRuntimeCoverage);
        }

        let mut authored_rule_bindings = content.mechanic_rule_bindings().collect::<Vec<_>>();
        authored_rule_bindings.sort_unstable();
        let mut runtime_rule_bindings = unique
            .trailblaze_bonuses
            .iter()
            .map(|row| {
                (
                    row.rule_contribution.as_ref(),
                    row.identity.stable_key.as_ref(),
                )
            })
            .chain(unique.conundrum_levels.iter().map(|row| {
                (
                    row.rule_contribution.as_ref(),
                    row.identity.stable_key.as_ref(),
                )
            }))
            .chain(unique.neural_nodes.iter().map(|row| {
                (
                    row.rule_contribution.as_ref(),
                    row.identity.stable_key.as_ref(),
                )
            }))
            .chain(
                runtime
                    .curios
                    .rule_bindings()
                    .iter()
                    .map(|binding| (binding.rule_id(), binding.owner_id())),
            )
            .chain(
                runtime
                    .occurrences
                    .rule_bindings()
                    .iter()
                    .map(|binding| (binding.rule_id(), binding.owner_id())),
            )
            .chain(
                runtime
                    .service_adventure
                    .rule_bindings()
                    .iter()
                    .map(|binding| (binding.rule_id(), binding.owner_id())),
            )
            .chain(
                runtime
                    .path_boost_rules
                    .bindings()
                    .iter()
                    .map(|binding| (binding.rule_id(), binding.owner_id())),
            )
            .chain(
                runtime
                    .resonance_rules
                    .bindings()
                    .iter()
                    .map(|binding| (binding.rule_id(), binding.owner_id())),
            )
            .collect::<Vec<_>>();
        runtime_rule_bindings.sort_unstable();
        validate_exact_rule_bindings(
            &authored_rule_bindings,
            &runtime_rule_bindings,
            MECHANIC_RULE_COUNT,
        )?;

        let mut authored_fixture_ids = content.review_fixture_keys().collect::<Vec<_>>();
        authored_fixture_ids.sort_unstable();
        let mut runtime_fixture_ids = runtime
            .semantic_fixtures
            .bindings()
            .iter()
            .map(|binding| binding.fixture_id())
            .collect::<Vec<_>>();
        runtime_fixture_ids.sort_unstable();
        validate_exact_ids(
            &authored_fixture_ids,
            &runtime_fixture_ids,
            SEMANTIC_FIXTURE_COUNT,
        )?;
        let digest = coverage_digest(
            &source_bindings,
            &runtime_rule_bindings,
            &runtime_fixture_ids,
        );
        let summary = GoldAndGearsRuntimeCoverageSummary {
            source_categories: SOURCE_CATEGORY_COUNT as u16,
            source_runtime_slices: source_bindings.len() as u16,
            source_obligations: SOURCE_OBLIGATION_COUNT,
            integrated_obligations: 7_181,
            shared_integrated_obligations: 706,
            external_outcomes: 8,
            metadata_obligations: 18,
            mechanic_rules: MECHANIC_RULE_COUNT as u16,
            semantic_fixtures: SEMANTIC_FIXTURE_COUNT as u8,
            native_handlers: 0,
            digest,
        };
        Ok(Self { summary })
    }

    pub(super) const fn summary(&self) -> GoldAndGearsRuntimeCoverageSummary {
        self.summary
    }
}

impl GoldAndGearsRuntimeFactory {
    #[must_use]
    pub fn runtime_coverage_summary(&self) -> GoldAndGearsRuntimeCoverageSummary {
        self.runtime_coverage.summary()
    }
}

pub(super) fn validate_exact_ids(
    authored: &[&str],
    runtime: &[&str],
    expected: usize,
) -> Result<(), GoldAndGearsEntryError> {
    if authored.len() != expected
        || runtime.len() != expected
        || authored.windows(2).any(|pair| pair[0] >= pair[1])
        || runtime.windows(2).any(|pair| pair[0] >= pair[1])
        || authored != runtime
    {
        return Err(GoldAndGearsEntryError::InvalidRuntimeCoverage);
    }
    Ok(())
}

pub(super) fn validate_exact_rule_bindings(
    authored: &[(&str, &str)],
    runtime: &[(&str, &str)],
    expected: usize,
) -> Result<(), GoldAndGearsEntryError> {
    if authored.len() != expected
        || runtime.len() != expected
        || authored.windows(2).any(|pair| pair[0].0 >= pair[1].0)
        || runtime.windows(2).any(|pair| pair[0].0 >= pair[1].0)
        || authored != runtime
    {
        return Err(GoldAndGearsEntryError::InvalidRuntimeCoverage);
    }
    Ok(())
}

pub(super) fn category_contract(
    category: &str,
    required: u32,
) -> Option<Vec<GoldAndGearsSourceRuntimeBinding>> {
    use GoldAndGearsSourceRuntimeDisposition::{
        ExternalOutcome, Integrated, Metadata, SharedIntegrated,
    };
    use GoldAndGearsSourceRuntimeOwner::{
        Cognition, Content, Conundrum, DiceFace, DiceLoadout, DiceResolution, Entry, Knowledge,
        NeuralNetwork, PlaneTransition, Progression, SemanticFixture, Topology,
    };
    if category == "curios" {
        return (required == 80).then(|| {
            vec![
                source_binding(category, 19, Content, Integrated),
                source_binding(category, 61, Content, SharedIntegrated),
            ]
        });
    }
    if category == "occurrences" {
        return (required == 62).then(|| {
            vec![
                source_binding(category, 11, Content, Integrated),
                source_binding(category, 51, Content, SharedIntegrated),
            ]
        });
    }
    let (expected, owner, disposition) = match category {
        "adventure_outcomes" => (8, Content, ExternalOutcome),
        "beacons" => (6, Topology, SharedIntegrated),
        "blessing_levels" => (324, Content, SharedIntegrated),
        "blessings" => (162, Content, SharedIntegrated),
        "block_create_rules" => (1_091, Topology, Integrated),
        "boss_choices" => (6, PlaneTransition, SharedIntegrated),
        "chessboards" => (115, Topology, Integrated),
        "cognition_ranges" => (13, Cognition, Integrated),
        "conundrum_levels" => (12, Conundrum, Integrated),
        "curio_states" => (80, Content, Integrated),
        "dice_categories" => (4, DiceLoadout, Integrated),
        "dice_definitions" => (12, DiceResolution, Integrated),
        "dice_face_tags" => (10, DiceFace, Integrated),
        "dice_faces" => (80, DiceFace, Integrated),
        "dice_path_values" => (108, DiceResolution, Integrated),
        "dice_slots" => (6, DiceLoadout, Integrated),
        "difficulty_segments" => (16, Entry, SharedIntegrated),
        "domains" => (12, Topology, SharedIntegrated),
        "entry_points" => (3, Entry, Integrated),
        "formal_difficulties" => (5, Entry, Integrated),
        "guide_areas" => (3, Entry, Integrated),
        "knowledge_bindings" => (22, Knowledge, Integrated),
        "map_columns" => (1_313, Topology, Integrated),
        "map_events" => (332, Topology, Integrated),
        "map_nodes" => (2_502, Topology, Integrated),
        "mode_constants" => (22, Cognition, Integrated),
        "neural_network_nodes" => (40, NeuralNetwork, Integrated),
        "occurrence_variants" => (65, Content, Integrated),
        "path_boosts" => (9, Progression, Integrated),
        "paths" => (9, Progression, SharedIntegrated),
        "planes" => (8, Topology, SharedIntegrated),
        "profiles" => (1, Entry, Integrated),
        "resonance_extrapolations" => (36, Progression, Integrated),
        "resonance_interplays" => (18, Progression, Integrated),
        "resonances" => (36, Progression, SharedIntegrated),
        "room_bindings" => (1_224, Topology, Integrated),
        "secret_conditions" => (20, Cognition, Integrated),
        "semantic_fixture_families" => (18, SemanticFixture, Metadata),
        "shared_services" => (15, Content, SharedIntegrated),
        "trailblaze_bonuses" => (5, Progression, Integrated),
        _ => return None,
    };
    if required != expected {
        return None;
    }
    Some(vec![source_binding(category, required, owner, disposition)])
}

fn source_binding(
    category: &str,
    required: u32,
    owner: GoldAndGearsSourceRuntimeOwner,
    disposition: GoldAndGearsSourceRuntimeDisposition,
) -> GoldAndGearsSourceRuntimeBinding {
    GoldAndGearsSourceRuntimeBinding {
        category: category.into(),
        required,
        owner,
        disposition,
    }
}

fn disposition_total(
    bindings: &[GoldAndGearsSourceRuntimeBinding],
    disposition: GoldAndGearsSourceRuntimeDisposition,
) -> u32 {
    bindings
        .iter()
        .filter(|binding| binding.disposition == disposition)
        .map(|binding| binding.required)
        .sum()
}

fn coverage_digest(
    source_bindings: &[GoldAndGearsSourceRuntimeBinding],
    rule_bindings: &[(&str, &str)],
    fixture_ids: &[&str],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-gold-gears-runtime-coverage-v1");
    encoder.text(GOLD_AND_GEARS_RUNTIME_COVERAGE_REVISION);
    encoder.u32(source_bindings.len() as u32);
    for binding in source_bindings {
        encoder.text(&binding.category);
        encoder.u32(binding.required);
        encoder.u8(binding.owner as u8);
        encoder.u8(binding.disposition as u8);
    }
    encoder.u32(rule_bindings.len() as u32);
    for (rule_id, owner_id) in rule_bindings {
        encoder.text(rule_id);
        encoder.text(owner_id);
    }
    encoder.u32(fixture_ids.len() as u32);
    for fixture_id in fixture_ids {
        encoder.text(fixture_id);
    }
    encoder.finish()
}
