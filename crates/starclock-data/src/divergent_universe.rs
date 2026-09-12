//! Private generated-reader boundary for the Divergent Universe production bundle.
//!
//! Generated Sora rows never leave `starclock-data`. This module validates the
//! complete transport inventory and exposes only immutable current-tree identity.

use std::collections::{BTreeMap, BTreeSet};

use self::persona_obligations::validate_persona_obligations;
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[path = "divergent_universe_persona.rs"]
mod persona_obligations;

use crate::divergent_universe_blessing::lower_divergent_universe_blessings;
use crate::divergent_universe_blessing_catalog::DivergentUniverseBlessingCatalog;
use crate::divergent_universe_catalog::DivergentUniverseFlowCatalog;
use crate::divergent_universe_curio::lower_divergent_universe_curios;
use crate::divergent_universe_curio_catalog::DivergentUniverseCurioCatalog;
use crate::divergent_universe_encounter::lower_divergent_universe_encounters;
use crate::divergent_universe_encounter_catalog::DivergentUniverseEncounterCatalog;
use crate::divergent_universe_equation::lower_divergent_universe_equations;
use crate::divergent_universe_equation_catalog::DivergentUniverseEquationCatalog;
use crate::divergent_universe_flow::lower_divergent_universe_flow;
use crate::divergent_universe_generated::{
    SCHEMA_FINGERPRINT, SoraConfig,
    runtime::{SoraBundle, SoraTableSource},
};
use crate::divergent_universe_mapping::lower_divergent_universe_mapping;
use crate::divergent_universe_mapping_catalog::DivergentUniverseMappingCatalog;
use crate::divergent_universe_mechanic::lower_divergent_universe_mechanics;
use crate::divergent_universe_mechanic_catalog::DivergentUniverseMechanicCatalog;
use crate::divergent_universe_progression::lower_divergent_universe_progression;
use crate::divergent_universe_progression_catalog::DivergentUniverseProgressionCatalog;
use crate::divergent_universe_service::lower_divergent_universe_services;
use crate::divergent_universe_service_catalog::DivergentUniverseServiceCatalog;
use crate::divergent_universe_titan::lower_divergent_universe_titans;
use crate::divergent_universe_titan_catalog::DivergentUniverseTitanCatalog;

const PRODUCTION_BUNDLE: &[u8] =
    include_bytes!("../../../config/divergent-universe-generated/config.sora");
const PRODUCTION_SCHEMA_LOCK: &[u8] =
    include_bytes!("../../../config/divergent-universe-generated/schema.lock");
const EXPECTED_TABLES: u32 = 81;
const EXPECTED_ROWS: u32 = 28_732;
const EXPECTED_SOURCES: u32 = 8_171;
const EXPECTED_EMPTY_TABLES: [&str; 2] = [
    "DivergentUniverseLayerRooms",
    "DivergentUniverseOccurrenceChoices",
];
const ROW_SCHEMA_REVISION: &str = "starclock.divergent-universe-row.v1";

macro_rules! digest_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name([u8; 32]);

        impl $name {
            /// Returns the canonical current-tree SHA-256 bytes.
            #[must_use]
            pub const fn bytes(self) -> [u8; 32] {
                self.0
            }
        }
    };
}

digest_type!(DivergentUniverseSchemaDigest);
digest_type!(DivergentUniverseConfigurationDigest);
digest_type!(DivergentUniverseContentDigest);
digest_type!(DivergentUniverseSourceDigest);
digest_type!(DivergentUniverseComponentDigest);

/// Immutable identity of one completely decoded Divergent Universe bundle.
///
/// The identity describes only the current schema and inputs. It is not a
/// compatibility promise and exposes no generated Sora row or lookup surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBundleIdentity {
    schema_fingerprint: Box<str>,
    schema_digest: DivergentUniverseSchemaDigest,
    configuration_digest: DivergentUniverseConfigurationDigest,
    content_digest: DivergentUniverseContentDigest,
    source_digest: DivergentUniverseSourceDigest,
    component_digest: DivergentUniverseComponentDigest,
    table_count: u32,
    row_count: u32,
    source_count: u32,
    source_obligation_count: u32,
    empty_table_count: u32,
}

impl DivergentUniverseBundleIdentity {
    /// Returns the fingerprint required by the generated Sora reader.
    #[must_use]
    pub fn schema_fingerprint(&self) -> &str {
        &self.schema_fingerprint
    }

    /// Returns the exact generated schema-lock digest.
    #[must_use]
    pub const fn schema_digest(&self) -> DivergentUniverseSchemaDigest {
        self.schema_digest
    }

    /// Returns the exact binary bundle digest.
    #[must_use]
    pub const fn configuration_digest(&self) -> DivergentUniverseConfigurationDigest {
        self.configuration_digest
    }

    /// Returns the canonical bundle/table/pack content digest.
    #[must_use]
    pub const fn content_digest(&self) -> DivergentUniverseContentDigest {
        self.content_digest
    }

    /// Returns the canonical digest of all ordered source-provenance rows.
    #[must_use]
    pub const fn source_digest(&self) -> DivergentUniverseSourceDigest {
        self.source_digest
    }

    /// Returns the digest that binds schema, configuration, content and source inputs.
    #[must_use]
    pub const fn component_digest(&self) -> DivergentUniverseComponentDigest {
        self.component_digest
    }

    /// Returns the exact decoded table denominator.
    #[must_use]
    pub const fn table_count(&self) -> u32 {
        self.table_count
    }

    /// Returns the exact decoded row denominator, including metadata tables.
    #[must_use]
    pub const fn row_count(&self) -> u32 {
        self.row_count
    }

    /// Returns the exact source-provenance row denominator.
    #[must_use]
    pub const fn source_count(&self) -> u32 {
        self.source_count
    }

    /// Returns the decoded coverage-row denominator, including unimplemented candidates.
    /// This count is not runtime completion or confirmed profile membership.
    #[must_use]
    pub const fn source_obligation_count(&self) -> u32 {
        self.source_obligation_count
    }

    /// Returns the number of explicitly verified-empty tables.
    #[must_use]
    pub const fn empty_table_count(&self) -> u32 {
        self.empty_table_count
    }
}

/// A validated bundle candidate with no public generated-row access.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBundleCandidate {
    identity: DivergentUniverseBundleIdentity,
    catalog: DivergentUniverseFlowCatalog,
    mapping_catalog: DivergentUniverseMappingCatalog,
    equation_catalog: DivergentUniverseEquationCatalog,
    blessing_catalog: DivergentUniverseBlessingCatalog,
    curio_catalog: DivergentUniverseCurioCatalog,
    titan_catalog: DivergentUniverseTitanCatalog,
    progression_catalog: DivergentUniverseProgressionCatalog,
    mechanic_catalog: DivergentUniverseMechanicCatalog,
    service_catalog: DivergentUniverseServiceCatalog,
    encounter_catalog: DivergentUniverseEncounterCatalog,
}

impl DivergentUniverseBundleCandidate {
    /// Returns the immutable current-tree bundle identity.
    #[must_use]
    pub const fn identity(&self) -> &DivergentUniverseBundleIdentity {
        &self.identity
    }

    /// Returns the validated immutable entry/topology catalog.
    #[must_use]
    pub const fn catalog(&self) -> &DivergentUniverseFlowCatalog {
        &self.catalog
    }

    /// Returns the validated immutable Arithmetic Mapping catalog.
    #[must_use]
    pub const fn mapping_catalog(&self) -> &DivergentUniverseMappingCatalog {
        &self.mapping_catalog
    }

    /// Returns the validated immutable Equation catalog.
    #[must_use]
    pub const fn equation_catalog(&self) -> &DivergentUniverseEquationCatalog {
        &self.equation_catalog
    }

    /// Returns the validated immutable Blessing and Equation-contribution catalog.
    #[must_use]
    pub const fn blessing_catalog(&self) -> &DivergentUniverseBlessingCatalog {
        &self.blessing_catalog
    }

    /// Returns the validated immutable Curio and Grand Miracle catalog.
    #[must_use]
    pub const fn curio_catalog(&self) -> &DivergentUniverseCurioCatalog {
        &self.curio_catalog
    }

    /// Returns the validated immutable Titan catalog.
    #[must_use]
    pub const fn titan_catalog(&self) -> &DivergentUniverseTitanCatalog {
        &self.titan_catalog
    }

    /// Returns the validated immutable Protocol and progression catalog.
    #[must_use]
    pub const fn progression_catalog(&self) -> &DivergentUniverseProgressionCatalog {
        &self.progression_catalog
    }

    /// Returns reference-only mechanic identities without executable handlers.
    #[must_use]
    pub const fn mechanic_catalog(&self) -> &DivergentUniverseMechanicCatalog {
        &self.mechanic_catalog
    }

    /// Returns the immutable fail-closed economy, service and Occurrence catalog.
    #[must_use]
    pub const fn service_catalog(&self) -> &DivergentUniverseServiceCatalog {
        &self.service_catalog
    }

    /// Returns the immutable unpromoted weekly encounter candidate closure.
    #[must_use]
    pub const fn encounter_catalog(&self) -> &DivergentUniverseEncounterCatalog {
        &self.encounter_catalog
    }
}

/// Typed configuration or validation failure at the generated-reader boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseDataError {
    message: Box<str>,
}

impl std::fmt::Display for DivergentUniverseDataError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for DivergentUniverseDataError {}

/// Loads and validates the repository-owned production bundle.
///
/// All 81 tables and 28,732 rows are decoded before success. No partial
/// candidate is returned on format, schema, row, provenance or digest failure.
pub fn load_divergent_universe_bundle()
-> Result<DivergentUniverseBundleCandidate, DivergentUniverseDataError> {
    load_divergent_universe_bundle_from_bytes(PRODUCTION_BUNDLE)
}

/// Loads and validates a binary Sora bundle using the current generated reader.
///
/// JSON, workbooks, old schemas and partially decoded candidates are rejected.
pub fn load_divergent_universe_bundle_from_bytes(
    bytes: &[u8],
) -> Result<DivergentUniverseBundleCandidate, DivergentUniverseDataError> {
    let bundle = SoraBundle::parse(bytes).map_err(debug_error)?;
    let config = SoraConfig::from_source(&bundle).map_err(debug_error)?;
    let identity = validate_and_identify(bytes, &bundle, &config)?;
    let catalog = lower_divergent_universe_flow(&config)?;
    let mapping_catalog = lower_divergent_universe_mapping(&config)?;
    let equation_catalog = lower_divergent_universe_equations(&config)?;
    let blessing_catalog = lower_divergent_universe_blessings(&config, &equation_catalog)?;
    let curio_catalog = lower_divergent_universe_curios(&config)?;
    let titan_catalog = lower_divergent_universe_titans(&config)?;
    let progression_catalog = lower_divergent_universe_progression(&config, &catalog)?;
    let mechanic_catalog = lower_divergent_universe_mechanics(&config)?;
    let service_catalog = lower_divergent_universe_services(&config)?;
    let encounter_catalog =
        lower_divergent_universe_encounters(&config, &catalog, &progression_catalog)?;
    Ok(DivergentUniverseBundleCandidate {
        identity,
        catalog,
        mapping_catalog,
        equation_catalog,
        blessing_catalog,
        curio_catalog,
        titan_catalog,
        progression_catalog,
        mechanic_catalog,
        service_catalog,
        encounter_catalog,
    })
}

fn validate_and_identify(
    bytes: &[u8],
    bundle: &SoraBundle<'_>,
    config: &SoraConfig,
) -> Result<DivergentUniverseBundleIdentity, DivergentUniverseDataError> {
    let fingerprint = bundle.schema_fingerprint().map_err(debug_error)?;
    if fingerprint != SCHEMA_FINGERPRINT {
        return Err(error("Divergent Universe schema fingerprint mismatch"));
    }
    let tables = table_inventory(config)?;
    validate_manifest_inventory(config, &tables)?;
    validate_all_rows(config)?;
    validate_persona_obligations(config)?;
    let source_digest = source_digest(config)?;
    let (pack_digest, authoring_digest) = validate_pack_index(config)?;

    let schema_digest = DivergentUniverseSchemaDigest(sha256(PRODUCTION_SCHEMA_LOCK));
    let configuration_digest = DivergentUniverseConfigurationDigest(sha256(bytes));
    let mut content = Sha256::new();
    content.update(b"starclock.divergent-universe.mode-content\0");
    content.update(schema_digest.bytes());
    content.update(configuration_digest.bytes());
    content.update(pack_digest);
    hash_text(&mut content, fingerprint)?;
    for (name, rows) in &tables {
        hash_text(&mut content, name)?;
        content.update(rows.to_be_bytes());
    }
    let content_digest = DivergentUniverseContentDigest(content.finalize().into());
    let mut component = Sha256::new();
    component.update(b"starclock.divergent-universe.component.mode-content\0");
    component.update(schema_digest.bytes());
    component.update(configuration_digest.bytes());
    component.update(content_digest.bytes());
    component.update(source_digest.bytes());
    component.update(authoring_digest);

    Ok(DivergentUniverseBundleIdentity {
        schema_fingerprint: fingerprint.into(),
        schema_digest,
        configuration_digest,
        content_digest,
        source_digest,
        component_digest: DivergentUniverseComponentDigest(component.finalize().into()),
        table_count: EXPECTED_TABLES,
        row_count: EXPECTED_ROWS,
        source_count: EXPECTED_SOURCES,
        source_obligation_count: u32::try_from(config.divergent_universe_coverage().len())
            .map_err(debug_error)?,
        empty_table_count: u32::try_from(EXPECTED_EMPTY_TABLES.len()).map_err(debug_error)?,
    })
}

fn table_inventory(config: &SoraConfig) -> Result<BTreeMap<&str, u32>, DivergentUniverseDataError> {
    let mut tables = BTreeMap::new();
    let mut total = 0_u32;
    for table in config.tables() {
        let rows = u32::try_from(table.len()).map_err(debug_error)?;
        if tables.insert(table.info().name, rows).is_some() {
            return Err(error(
                "Divergent Universe generated table identity is duplicated",
            ));
        }
        total = total
            .checked_add(rows)
            .ok_or_else(|| error("Divergent Universe row count overflow"))?;
    }
    if tables.len() != usize::try_from(EXPECTED_TABLES).map_err(debug_error)?
        || total != EXPECTED_ROWS
    {
        return Err(error(&format!(
            "Divergent Universe inventory mismatch: expected {EXPECTED_TABLES} tables/{EXPECTED_ROWS} rows, got {}/{} rows",
            tables.len(),
            total,
        )));
    }
    let empty = tables
        .iter()
        .filter_map(|(name, rows)| (*rows == 0).then_some(*name))
        .collect::<Vec<_>>();
    if empty != EXPECTED_EMPTY_TABLES {
        return Err(error(
            "Divergent Universe verified-empty table boundary drift",
        ));
    }
    Ok(tables)
}

fn validate_manifest_inventory(
    config: &SoraConfig,
    tables: &BTreeMap<&str, u32>,
) -> Result<(), DivergentUniverseDataError> {
    let manifest = exactly_one(
        config.divergent_universe_manifest().ordered_rows(),
        "manifest",
    )?;
    let files: Vec<String> = parse_required_json(
        manifest.normalized_files.as_deref(),
        "manifest normalized files",
    )?;
    let counts: BTreeMap<String, u32> =
        parse_required_json(manifest.record_counts.as_deref(), "manifest record counts")?;
    if files.len() != usize::try_from(EXPECTED_TABLES).map_err(debug_error)?
        || counts.len() != files.len()
        || files.iter().any(|file| !counts.contains_key(file))
    {
        return Err(error(
            "Divergent Universe manifest file/count closure drift",
        ));
    }
    for (file, rows) in counts {
        let table = normalized_file_table_name(&file)?;
        if tables.get(table.as_str()).copied() != Some(rows) {
            return Err(error(&format!(
                "Divergent Universe manifest count mismatch for {file}",
            )));
        }
    }
    let manifest_digest = required_text(
        manifest.content_manifest_sha256.as_deref(),
        "manifest content digest",
    )?;
    validate_hex_digest(manifest_digest, "manifest content digest")?;
    Ok(())
}

macro_rules! validate_table_rows {
    ($config:expr, $sources:expr, $method:ident) => {{
        for row in $config.$method().ordered_rows() {
            validate_common_row(
                row.id,
                &row.stable_key,
                &row.schema_revision,
                &row.kind,
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                row.source_refs.as_deref(),
                &row.payload_json,
                $sources,
            )?;
        }
    }};
}

fn validate_all_rows(config: &SoraConfig) -> Result<(), DivergentUniverseDataError> {
    let sources = config
        .divergent_universe_sources()
        .keys()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if sources.len() != usize::try_from(EXPECTED_SOURCES).map_err(debug_error)? {
        return Err(error("Divergent Universe source row denominator drift"));
    }
    validate_table_rows!(config, &sources, divergent_universe_profiles);
    validate_table_rows!(config, &sources, divergent_universe_modules);
    validate_table_rows!(config, &sources, divergent_universe_entries);
    validate_table_rows!(config, &sources, divergent_universe_finish_conditions);
    validate_table_rows!(config, &sources, divergent_universe_areas);
    validate_table_rows!(config, &sources, divergent_universe_difficulties);
    validate_table_rows!(config, &sources, divergent_universe_layers);
    validate_table_rows!(config, &sources, divergent_universe_layer_rooms);
    validate_table_rows!(config, &sources, divergent_universe_rooms);
    validate_table_rows!(config, &sources, divergent_universe_stage_flow);
    validate_table_rows!(config, &sources, divergent_universe_cyclical_challenges);
    validate_table_rows!(config, &sources, divergent_universe_protocols);
    validate_table_rows!(config, &sources, divergent_universe_astronomical_divisions);
    validate_table_rows!(config, &sources, divergent_universe_star_pioneer_practice);
    validate_table_rows!(config, &sources, divergent_universe_cognoculi);
    validate_table_rows!(
        config,
        &sources,
        divergent_universe_arithmetic_mapping_eligibility
    );
    validate_table_rows!(
        config,
        &sources,
        divergent_universe_arithmetic_mapping_builds
    );
    validate_table_rows!(
        config,
        &sources,
        divergent_universe_arithmetic_mapping_rules
    );
    validate_table_rows!(config, &sources, divergent_universe_equations);
    validate_table_rows!(config, &sources, divergent_universe_equation_recipes);
    validate_table_rows!(config, &sources, divergent_universe_equation_categories);
    validate_table_rows!(config, &sources, divergent_universe_equation_offers);
    validate_table_rows!(config, &sources, divergent_universe_equation_progress);
    validate_table_rows!(
        config,
        &sources,
        divergent_universe_equation_expansion_states
    );
    validate_table_rows!(config, &sources, divergent_universe_equation_effects);
    validate_table_rows!(
        config,
        &sources,
        divergent_universe_equation_replacement_rules
    );
    validate_table_rows!(config, &sources, divergent_universe_blessing_paths);
    validate_table_rows!(config, &sources, divergent_universe_blessings);
    validate_table_rows!(config, &sources, divergent_universe_blessing_levels);
    validate_table_rows!(config, &sources, divergent_universe_blessing_groups);
    validate_table_rows!(config, &sources, divergent_universe_blessing_rewrite_rules);
    validate_table_rows!(
        config,
        &sources,
        divergent_universe_blessing_equation_contributions
    );
    validate_table_rows!(config, &sources, divergent_universe_curios);
    validate_table_rows!(config, &sources, divergent_universe_curio_states);
    validate_table_rows!(config, &sources, divergent_universe_curio_groups);
    validate_table_rows!(config, &sources, divergent_universe_curio_lifecycle_rules);
    validate_table_rows!(config, &sources, divergent_universe_grand_miracles);
    validate_table_rows!(
        config,
        &sources,
        divergent_universe_grand_miracle_eligibility
    );
    validate_table_rows!(config, &sources, divergent_universe_grand_miracle_states);
    validate_table_rows!(config, &sources, divergent_universe_titan_types);
    validate_table_rows!(config, &sources, divergent_universe_titan_boons);
    validate_table_rows!(config, &sources, divergent_universe_titan_talents);
    validate_table_rows!(config, &sources, divergent_universe_titan_choices);
    validate_table_rows!(config, &sources, divergent_universe_titan_contributions);
    validate_table_rows!(config, &sources, divergent_universe_workbenches);
    validate_table_rows!(config, &sources, divergent_universe_workbench_functions);
    validate_table_rows!(config, &sources, divergent_universe_gamble_groups);
    validate_table_rows!(config, &sources, divergent_universe_gamble_units);
    validate_table_rows!(config, &sources, divergent_universe_curse_chests);
    validate_table_rows!(config, &sources, divergent_universe_currencies);
    validate_table_rows!(config, &sources, divergent_universe_service_rules);
    validate_table_rows!(config, &sources, divergent_universe_service_offer_rules);
    validate_table_rows!(config, &sources, divergent_universe_permanent_talents);
    validate_table_rows!(config, &sources, divergent_universe_unlocks);
    validate_table_rows!(config, &sources, divergent_universe_common_constants);
    validate_table_rows!(config, &sources, divergent_universe_weekly_modifiers);
    validate_table_rows!(config, &sources, divergent_universe_room_marks);
    validate_table_rows!(config, &sources, divergent_universe_progression_effects);
    validate_table_rows!(config, &sources, divergent_universe_pool_membership);
    validate_table_rows!(config, &sources, divergent_universe_curio_pool_membership);
    validate_table_rows!(config, &sources, divergent_universe_occurrences);
    validate_table_rows!(config, &sources, divergent_universe_occurrence_variants);
    validate_table_rows!(config, &sources, divergent_universe_occurrence_choices);
    validate_table_rows!(config, &sources, divergent_universe_mode_service_npcs);
    validate_table_rows!(config, &sources, divergent_universe_adventure_outcomes);
    validate_table_rows!(
        config,
        &sources,
        divergent_universe_encounter_source_obligations
    );
    validate_table_rows!(config, &sources, divergent_universe_encounter_groups);
    validate_table_rows!(config, &sources, divergent_universe_encounter_waves);
    validate_table_rows!(config, &sources, divergent_universe_enemy_slots);
    validate_table_rows!(config, &sources, divergent_universe_boss_pools);
    validate_table_rows!(
        config,
        &sources,
        divergent_universe_persona_source_obligations
    );
    validate_table_rows!(config, &sources, divergent_universe_mechanic_source_files);
    validate_table_rows!(config, &sources, divergent_universe_mechanic_rules);
    validate_table_rows!(config, &sources, divergent_universe_sources);
    validate_table_rows!(config, &sources, divergent_universe_coverage);
    validate_table_rows!(config, &sources, divergent_universe_research_gaps);
    validate_table_rows!(
        config,
        &sources,
        divergent_universe_semantic_fixture_families
    );
    validate_table_rows!(config, &sources, divergent_universe_review_fixtures);
    validate_table_rows!(config, &sources, divergent_universe_reconciliation_receipts);
    validate_table_rows!(config, &sources, divergent_universe_manifest);
    validate_table_rows!(config, &sources, divergent_universe_pack_index);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_common_row(
    id: i32,
    stable_key: &str,
    schema_revision: &str,
    kind: &str,
    name_en: &str,
    name_zh_cn: &str,
    summary_en: &str,
    summary_zh_cn: &str,
    source_refs: Option<&[i32]>,
    payload_json: &str,
    sources: &BTreeSet<i32>,
) -> Result<(), DivergentUniverseDataError> {
    if id <= 0
        || !stable_key.starts_with("divergent-universe.")
        || schema_revision != ROW_SCHEMA_REVISION
        || !kind.starts_with("DivergentUniverse")
        || name_en.is_empty()
        || name_zh_cn.is_empty()
        || summary_en.is_empty()
        || summary_zh_cn.is_empty()
    {
        return Err(error(&format!(
            "invalid common row contract for {stable_key}"
        )));
    }
    if source_refs
        .unwrap_or_default()
        .iter()
        .any(|source| !sources.contains(source))
    {
        return Err(error(&format!("unknown source reference on {stable_key}")));
    }
    let payload: serde_json::Value = serde_json::from_str(payload_json).map_err(debug_error)?;
    if !payload.is_object() {
        return Err(error(&format!("non-object payload on {stable_key}")));
    }
    Ok(())
}

fn source_digest(
    config: &SoraConfig,
) -> Result<DivergentUniverseSourceDigest, DivergentUniverseDataError> {
    let mut rows = config
        .divergent_universe_sources()
        .ordered_rows()
        .collect::<Vec<_>>();
    rows.sort_unstable_by(|left, right| left.stable_key.cmp(&right.stable_key));
    let mut digest = Sha256::new();
    digest.update(b"starclock.divergent-universe.sources\0");
    for row in rows {
        hash_text(&mut digest, &row.stable_key)?;
        for (value, label) in [
            (row.source_id.as_deref(), "source ID"),
            (row.repository.as_deref(), "source repository"),
            (row.revision.as_deref(), "source revision"),
            (row.path.as_deref(), "source path"),
            (row.locator.as_deref(), "source locator"),
            (row.sha256.as_deref(), "source SHA-256"),
            (row.mechanism_quality.as_deref(), "source mechanism quality"),
        ] {
            hash_text(&mut digest, required_text(value, label)?)?;
        }
        validate_hex_digest(
            required_text(row.sha256.as_deref(), "source SHA-256")?,
            "source SHA-256",
        )?;
    }
    Ok(DivergentUniverseSourceDigest(digest.finalize().into()))
}

fn validate_pack_index(
    config: &SoraConfig,
) -> Result<([u8; 32], [u8; 32]), DivergentUniverseDataError> {
    let pack = exactly_one(
        config.divergent_universe_pack_index().ordered_rows(),
        "pack index",
    )?;
    let pack_digest = parse_hex_digest(
        required_text(pack.pack_digest.as_deref(), "pack digest")?,
        "pack digest",
    )?;
    let mut files: Vec<PackFileDigest> =
        parse_required_json(pack.file_digests.as_deref(), "pack file digests")?;
    let identity_index: StableIdentityIndexProjection = parse_required_json(
        pack.stable_id_index.as_deref(),
        "pack stable identity index",
    )?;
    if files.len() != 80
        || identity_index.item_count != 28_731
        || identity_index.projection != "ExcelCellLimit"
    {
        return Err(error("Divergent Universe pack-index denominator drift"));
    }
    files.sort_unstable_by(|left, right| left.file.cmp(&right.file));
    if files.windows(2).any(|pair| pair[0].file == pair[1].file) {
        return Err(error("Divergent Universe pack file identity is duplicated"));
    }
    validate_hex_digest(&identity_index.sha256, "pack stable-ID index digest")?;
    let mut digest = Sha256::new();
    digest.update(b"starclock.divergent-universe.authoring-components\0");
    for file in files {
        hash_text(&mut digest, &file.file)?;
        digest.update(file.rows.to_be_bytes());
        digest.update(file.bytes.to_be_bytes());
        digest.update(parse_hex_digest(&file.sha256, "pack file digest")?);
    }
    digest.update(identity_index.canonical_json_bytes.to_be_bytes());
    digest.update(identity_index.item_count.to_be_bytes());
    hash_text(&mut digest, &identity_index.projection)?;
    digest.update(parse_hex_digest(
        &identity_index.sha256,
        "pack stable-ID index digest",
    )?);
    Ok((pack_digest, digest.finalize().into()))
}

#[derive(Deserialize)]
struct PackFileDigest {
    file: String,
    rows: u32,
    bytes: u64,
    sha256: String,
}

#[derive(Deserialize)]
struct StableIdentityIndexProjection {
    canonical_json_bytes: u64,
    item_count: u32,
    projection: String,
    sha256: String,
}

fn normalized_file_table_name(file: &str) -> Result<String, DivergentUniverseDataError> {
    let stem = file
        .strip_suffix(".json")
        .ok_or_else(|| error("Divergent Universe normalized file is not JSON"))?;
    let mut result = String::from("DivergentUniverse");
    for part in stem.split('-') {
        let mut characters = part.chars();
        let first = characters
            .next()
            .ok_or_else(|| error("Divergent Universe normalized file has an empty segment"))?;
        result.extend(first.to_uppercase());
        result.extend(characters);
    }
    Ok(result)
}

fn exactly_one<T>(
    mut values: impl Iterator<Item = T>,
    label: &str,
) -> Result<T, DivergentUniverseDataError> {
    let value = values
        .next()
        .ok_or_else(|| error(&format!("missing {label}")))?;
    if values.next().is_some() {
        return Err(error(&format!("duplicate {label}")));
    }
    Ok(value)
}

fn parse_required_json<T: for<'de> Deserialize<'de>>(
    value: Option<&str>,
    label: &str,
) -> Result<T, DivergentUniverseDataError> {
    serde_json::from_str(required_text(value, label)?).map_err(debug_error)
}

fn required_text<'a>(
    value: Option<&'a str>,
    label: &str,
) -> Result<&'a str, DivergentUniverseDataError> {
    value
        .filter(|text| !text.is_empty())
        .ok_or_else(|| error(&format!("missing {label}")))
}

fn parse_hex_digest(value: &str, label: &str) -> Result<[u8; 32], DivergentUniverseDataError> {
    validate_hex_digest(value, label)?;
    let mut result = [0_u8; 32];
    for (index, byte) in result.iter_mut().enumerate() {
        let offset = index * 2;
        *byte = u8::from_str_radix(&value[offset..offset + 2], 16).map_err(debug_error)?;
    }
    Ok(result)
}

fn validate_hex_digest(value: &str, label: &str) -> Result<(), DivergentUniverseDataError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(error(&format!("invalid {label}")));
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn hash_text(digest: &mut Sha256, value: &str) -> Result<(), DivergentUniverseDataError> {
    let length = u32::try_from(value.len())
        .map_err(|_| error("Divergent Universe identity text exceeds u32"))?;
    digest.update(length.to_be_bytes());
    digest.update(value.as_bytes());
    Ok(())
}

pub(super) fn error(message: &str) -> DivergentUniverseDataError {
    DivergentUniverseDataError {
        message: message.into(),
    }
}

pub(super) fn debug_error(value: impl std::fmt::Debug) -> DivergentUniverseDataError {
    DivergentUniverseDataError {
        message: format!("{value:?}").into_boxed_str(),
    }
}

#[cfg(test)]
#[path = "divergent_universe_tests.rs"]
mod tests;
