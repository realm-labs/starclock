//! Fail-closed production coverage for every frozen Swarm obligation.

use crate::{
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::{SwarmDisasterContentCatalog, coverage_access::MechanicCoverageInput},
};

const SOURCE_OBLIGATIONS: u32 = 6_963;
const MECHANIC_RULES: usize = 23;
const SEMANTIC_FIXTURES: usize = 23;
const SOURCE_CATEGORIES: [(&str, u32); 42] = [
    ("adventure_outcomes", 6),
    ("audience_dice", 8),
    ("audience_paths", 8),
    ("beacons", 4),
    ("blessing_levels", 288),
    ("blessings", 144),
    ("block_create_rules", 1_212),
    ("boss_choices", 2),
    ("boss_decay_levels", 42),
    ("chessboards", 101),
    ("communing_choices", 21),
    ("communing_dimensions", 7),
    ("communing_trail_nodes", 63),
    ("curio_states", 66),
    ("curios", 66),
    ("dice_faces", 42),
    ("dice_rarities", 3),
    ("difficulty_segments", 20),
    ("domains", 12),
    ("entry_points", 3),
    ("formal_difficulties", 5),
    ("guide_areas", 3),
    ("map_columns", 1_109),
    ("map_events", 349),
    ("map_nodes", 1_991),
    ("mechanical_chapter_locators", 13),
    ("mode_constants", 19),
    ("occurrence_variants", 57),
    ("occurrences", 75),
    ("path_boosts", 8),
    ("paths", 8),
    ("pathstrider_cabinets", 31),
    ("pathstrider_finish_conditions", 102),
    ("pathstrider_unlocks", 110),
    ("planes", 11),
    ("profiles", 1),
    ("resonance_interplays", 16),
    ("resonances", 32),
    ("room_bindings", 861),
    ("semantic_fixture_families", 23),
    ("shared_services", 15),
    ("trailblaze_bonuses", 6),
];

#[derive(Clone, Debug)]
pub(super) struct RuntimeCoverageCatalog;

impl RuntimeCoverageCatalog {
    pub(super) fn compile(
        content: &SwarmDisasterContentCatalog,
        semantic_fixture_digest: [u8; 32],
    ) -> Result<Self, UniverseCatalogLoadError> {
        validate_coverage(&coverage_inputs(content), semantic_fixture_digest)?;
        Ok(Self)
    }
}

#[derive(Clone, Debug)]
struct CoverageInputs {
    categories: Box<[(Box<str>, u32)]>,
    rules: Box<[MechanicCoverageInput]>,
    fixture_ids: Box<[Box<str>]>,
}

fn coverage_inputs(content: &SwarmDisasterContentCatalog) -> CoverageInputs {
    let categories = content
        .source_coverage_categories()
        .map(|(key, count)| (key.into(), count))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let mut rules = content.mechanic_coverage_inputs().into_vec();
    rules.sort_unstable_by(|left, right| left.key.cmp(&right.key));
    let mut fixture_ids = content
        .semantic_fixture_runtime_inputs()
        .iter()
        .map(|input| input.key.clone())
        .collect::<Vec<_>>();
    fixture_ids.sort_unstable();
    CoverageInputs {
        categories,
        rules: rules.into_boxed_slice(),
        fixture_ids: fixture_ids.into_boxed_slice(),
    }
}

fn validate_coverage(
    inputs: &CoverageInputs,
    semantic_fixture_digest: [u8; 32],
) -> Result<(), UniverseCatalogLoadError> {
    validate_categories(&inputs.categories)?;
    validate_rule_fixture_ids(&inputs.rules, &inputs.fixture_ids)?;
    if semantic_fixture_digest == [0; 32] {
        return Err(reference("semantic fixture execution digest is empty"));
    }
    Ok(())
}

#[cfg(test)]
fn coverage_snapshot(
    content: &SwarmDisasterContentCatalog,
    semantic_fixture_digest: [u8; 32],
) -> Result<CoverageInputs, UniverseCatalogLoadError> {
    let inputs = coverage_inputs(content);
    validate_coverage(&inputs, semantic_fixture_digest)?;
    Ok(inputs)
}

fn validate_categories(categories: &[(Box<str>, u32)]) -> Result<(), UniverseCatalogLoadError> {
    if categories.len() != SOURCE_CATEGORIES.len()
        || categories
            .iter()
            .zip(SOURCE_CATEGORIES)
            .any(|((key, count), expected)| key.as_ref() != expected.0 || *count != expected.1)
        || categories.iter().map(|(_, count)| count).sum::<u32>() != SOURCE_OBLIGATIONS
    {
        return Err(reference("source coverage category drift"));
    }
    Ok(())
}

fn validate_rule_fixture_ids(
    rules: &[MechanicCoverageInput],
    fixture_ids: &[Box<str>],
) -> Result<(), UniverseCatalogLoadError> {
    if rules.len() != MECHANIC_RULES
        || fixture_ids.len() != SEMANTIC_FIXTURES
        || rules.windows(2).any(|pair| pair[0].key >= pair[1].key)
        || fixture_ids.windows(2).any(|pair| pair[0] >= pair[1])
        || rules.iter().zip(fixture_ids).any(|(rule, fixture)| {
            let expected_rule = format!("swarm-disaster.mechanic-rule.{}", rule.family);
            let expected_fixture = format!("swarm-disaster.fixture.{}", rule.family);
            rule.key.as_ref() != expected_rule
                || rule.fixture_keys.len() != 1
                || rule.fixture_keys[0].as_ref() != expected_fixture
                || fixture.as_ref() != expected_fixture
        })
    {
        return Err(reference(
            "mechanic rule or semantic fixture coverage drift",
        ));
    }
    Ok(())
}

fn reference(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}

#[cfg(test)]
#[path = "runtime_coverage_tests.rs"]
mod tests;
