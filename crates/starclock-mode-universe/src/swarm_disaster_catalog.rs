//! Private validation boundary for the frozen Swarm Disaster Candidate bundle.

use crate::digest::bundle_digest;
use crate::error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind};
use crate::swarm_disaster_generated::{
    ErasedSoraTable, SCHEMA_FINGERPRINT, SoraConfig,
    runtime::{SoraBundle, SoraTableSource},
    swarm_disaster_manifest::SwarmDisasterManifest,
};

const EXPECTED_BUNDLE_DIGEST: [u8; 32] = [
    0x38, 0x57, 0x27, 0xa8, 0xa5, 0x87, 0x57, 0x95, 0xb2, 0x9c, 0x99, 0x61, 0x02, 0x04, 0x0f, 0x7f,
    0x44, 0x19, 0xc6, 0xad, 0xac, 0x7b, 0x5e, 0x10, 0xee, 0x6b, 0x09, 0xc0, 0x84, 0x40, 0x93, 0x62,
];
const EXPECTED_SCHEMA_FINGERPRINT: &str = "e1a4fc5af6b64ee9";
const EXPECTED_TABLES: usize = 65;
const EXPECTED_ROWS: usize = 33_380;
const EXPECTED_GOAL_ID: &str = "swarm-disaster-reference-v1";
const EXPECTED_PROFILE_ID: &str = "swarm-disaster.profile.v1";
const EXPECTED_MANIFEST_REVISION: &str = "starclock.swarm-disaster-pack-manifest.v1";
const EXPECTED_MANIFEST_KEY: &str = "swarm-disaster.pack-manifest.v1";
const EXPECTED_CONTENT_MANIFEST_DIGEST: &str =
    "e466cae0481d93241eaadf6d894b82898d47c9d4863fea262134cbbac10b8850";
const EXPECTED_RUNTIME_MARKER: &str = "ForbiddenReferenceOnly";
const SECTION_KIND_TABLE: usize = 2;
const HEADER_LEN: usize = 24;
const SECTION_ENTRY_LEN: usize = 28;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SwarmDisasterBundleSummary {
    bundle_digest: [u8; 32],
    table_count: usize,
    row_count: usize,
    source_obligations: usize,
    mechanic_rules: usize,
    semantic_fixtures: usize,
    policy_boundaries: usize,
}

impl SwarmDisasterBundleSummary {
    const fn bundle_digest(self) -> [u8; 32] {
        self.bundle_digest
    }

    const fn table_count(self) -> usize {
        self.table_count
    }

    const fn row_count(self) -> usize {
        self.row_count
    }

    const fn source_obligations(self) -> usize {
        self.source_obligations
    }

    const fn mechanic_rules(self) -> usize {
        self.mechanic_rules
    }

    const fn semantic_fixtures(self) -> usize {
        self.semantic_fixtures
    }

    const fn policy_boundaries(self) -> usize {
        self.policy_boundaries
    }

    fn matches_contract(self) -> bool {
        self.bundle_digest() == EXPECTED_BUNDLE_DIGEST
            && self.table_count() == EXPECTED_TABLES
            && self.row_count() == EXPECTED_ROWS
            && self.source_obligations() == 6_963
            && self.mechanic_rules() == 23
            && self.semantic_fixtures() == 23
            && self.policy_boundaries() == 31
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SwarmDisasterBundleLoadError {
    BundleDigest,
    BundleFormat,
    SchemaFingerprint,
    TableClosure,
    ManifestRevision,
    RowDenominator,
}

impl core::fmt::Display for SwarmDisasterBundleLoadError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::BundleDigest => "Swarm Disaster bundle digest mismatch",
            Self::BundleFormat => "Swarm Disaster bundle format is invalid",
            Self::SchemaFingerprint => "Swarm Disaster schema fingerprint mismatch",
            Self::TableClosure => "Swarm Disaster table closure mismatch",
            Self::ManifestRevision => "Swarm Disaster manifest revision mismatch",
            Self::RowDenominator => "Swarm Disaster row denominator mismatch",
        })
    }
}

impl std::error::Error for SwarmDisasterBundleLoadError {}

/// Validates the immutable Goal 09 Candidate bundle without exposing generated rows.
///
/// Production construction remains owned by the future Swarm Disaster runtime factory;
/// this function is a generated-type-free diagnostic boundary.
pub fn validate_swarm_disaster_bundle(bytes: &[u8]) -> Result<(), UniverseCatalogLoadError> {
    let (summary, _) = load_swarm_disaster_bundle(bytes).map_err(|error| {
        UniverseCatalogLoadError::new(
            UniverseCatalogLoadErrorKind::InvalidEmbeddedData,
            error.to_string(),
        )
    })?;
    if !summary.matches_contract() {
        return Err(UniverseCatalogLoadError::new(
            UniverseCatalogLoadErrorKind::InvalidEmbeddedData,
            "Swarm Disaster bundle summary contract mismatch",
        ));
    }
    Ok(())
}

fn load_swarm_disaster_bundle(
    bytes: &[u8],
) -> Result<(SwarmDisasterBundleSummary, SoraConfig), SwarmDisasterBundleLoadError> {
    let digest = bundle_digest(bytes).bytes();
    if digest != EXPECTED_BUNDLE_DIGEST {
        return Err(SwarmDisasterBundleLoadError::BundleDigest);
    }
    let raw_tables = table_section_names(bytes)?;
    let bundle =
        SoraBundle::parse(bytes).map_err(|_| SwarmDisasterBundleLoadError::BundleFormat)?;
    validate_schema_fingerprint(
        bundle
            .schema_fingerprint()
            .map_err(|_| SwarmDisasterBundleLoadError::BundleFormat)?,
    )?;
    let config =
        SoraConfig::from_source(&bundle).map_err(|_| SwarmDisasterBundleLoadError::BundleFormat)?;
    let mut loaded_tables = config
        .tables()
        .map(|table| table.info().name)
        .collect::<Vec<_>>();
    loaded_tables.sort_unstable();
    validate_table_closure(&raw_tables, &loaded_tables)?;
    let row_count = config.tables().map(ErasedSoraTable::len).sum::<usize>();
    if loaded_tables.len() != EXPECTED_TABLES || row_count != EXPECTED_ROWS {
        return Err(SwarmDisasterBundleLoadError::RowDenominator);
    }
    let manifest = config
        .swarm_disaster_manifest()
        .get(&1)
        .ok_or(SwarmDisasterBundleLoadError::ManifestRevision)?;
    validate_manifest(manifest)?;
    let summary = SwarmDisasterBundleSummary {
        bundle_digest: digest,
        table_count: loaded_tables.len(),
        row_count,
        source_obligations: usize::try_from(manifest.frozen_source_obligations)
            .map_err(|_| SwarmDisasterBundleLoadError::RowDenominator)?,
        mechanic_rules: usize::try_from(manifest.mechanic_rule_count)
            .map_err(|_| SwarmDisasterBundleLoadError::RowDenominator)?,
        semantic_fixtures: usize::try_from(manifest.semantic_fixture_family_count)
            .map_err(|_| SwarmDisasterBundleLoadError::RowDenominator)?,
        policy_boundaries: usize::try_from(manifest.research_gap_count)
            .map_err(|_| SwarmDisasterBundleLoadError::RowDenominator)?,
    };
    Ok((summary, config))
}

fn validate_schema_fingerprint(value: &str) -> Result<(), SwarmDisasterBundleLoadError> {
    if value != EXPECTED_SCHEMA_FINGERPRINT || value != SCHEMA_FINGERPRINT {
        return Err(SwarmDisasterBundleLoadError::SchemaFingerprint);
    }
    Ok(())
}

fn validate_table_closure(
    raw: &[String],
    loaded: &[&str],
) -> Result<(), SwarmDisasterBundleLoadError> {
    if raw.len() != EXPECTED_TABLES
        || loaded.len() != EXPECTED_TABLES
        || !raw.iter().map(String::as_str).eq(loaded.iter().copied())
    {
        return Err(SwarmDisasterBundleLoadError::TableClosure);
    }
    Ok(())
}

fn validate_manifest(manifest: &SwarmDisasterManifest) -> Result<(), SwarmDisasterBundleLoadError> {
    if manifest.stable_key != EXPECTED_MANIFEST_KEY
        || manifest.schema_revision != EXPECTED_MANIFEST_REVISION
        || manifest.goal_id != EXPECTED_GOAL_ID
        || manifest.profile_id != EXPECTED_PROFILE_ID
        || manifest.content_manifest_sha256 != EXPECTED_CONTENT_MANIFEST_DIGEST
        || manifest.runtime_loading != EXPECTED_RUNTIME_MARKER
        || !manifest.candidate_quality
    {
        return Err(SwarmDisasterBundleLoadError::ManifestRevision);
    }
    if manifest.frozen_source_obligations != 6_963
        || manifest.data_ready_source_obligations != 6_963
        || manifest.mechanic_rule_count != 23
        || manifest.semantic_fixture_family_count != 23
        || manifest.research_gap_count != 31
        || manifest.blocking_research_gap_count != 0
    {
        return Err(SwarmDisasterBundleLoadError::RowDenominator);
    }
    Ok(())
}

fn table_section_names(bytes: &[u8]) -> Result<Vec<String>, SwarmDisasterBundleLoadError> {
    if bytes.len() < HEADER_LEN || &bytes[..4] != b"SORA" {
        return Err(SwarmDisasterBundleLoadError::BundleFormat);
    }
    let header_len = read_u32(bytes, 8)?;
    let directory_len = read_u32(bytes, 12)?;
    let section_count = read_u32(bytes, 16)?;
    if header_len != HEADER_LEN {
        return Err(SwarmDisasterBundleLoadError::BundleFormat);
    }
    let directory_end = header_len
        .checked_add(directory_len)
        .filter(|end| *end <= bytes.len())
        .ok_or(SwarmDisasterBundleLoadError::BundleFormat)?;
    let mut cursor = header_len;
    let mut names = Vec::new();
    for _ in 0..section_count {
        let entry_end = cursor
            .checked_add(SECTION_ENTRY_LEN)
            .filter(|end| *end <= directory_end)
            .ok_or(SwarmDisasterBundleLoadError::BundleFormat)?;
        let kind = read_u32(bytes, cursor)?;
        let name_len = read_u32(bytes, cursor + 8)?;
        let name_end = entry_end
            .checked_add(name_len)
            .filter(|end| *end <= directory_end)
            .ok_or(SwarmDisasterBundleLoadError::BundleFormat)?;
        if kind == SECTION_KIND_TABLE {
            names.push(
                std::str::from_utf8(&bytes[entry_end..name_end])
                    .map_err(|_| SwarmDisasterBundleLoadError::BundleFormat)?
                    .to_owned(),
            );
        }
        cursor = name_end;
    }
    if cursor != directory_end {
        return Err(SwarmDisasterBundleLoadError::BundleFormat);
    }
    names.sort_unstable();
    Ok(names)
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<usize, SwarmDisasterBundleLoadError> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or(SwarmDisasterBundleLoadError::BundleFormat)?;
    Ok(u32::from_le_bytes(
        value
            .try_into()
            .map_err(|_| SwarmDisasterBundleLoadError::BundleFormat)?,
    ) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUNDLE: &[u8] = include_bytes!("../../../config/swarm-disaster-generated/config.sora");

    #[test]
    fn loads_exact_candidate_bundle_without_exposing_generated_rows() {
        let (summary, _) = load_swarm_disaster_bundle(BUNDLE).unwrap();
        assert_eq!(summary.bundle_digest(), EXPECTED_BUNDLE_DIGEST);
        assert_eq!(summary.table_count(), 65);
        assert_eq!(summary.row_count(), 33_380);
        assert_eq!(summary.source_obligations(), 6_963);
        assert_eq!(summary.mechanic_rules(), 23);
        assert_eq!(summary.semantic_fixtures(), 23);
        assert_eq!(summary.policy_boundaries(), 31);
    }

    #[test]
    fn rejects_digest_format_schema_and_table_closure_drift() {
        let mut tampered = BUNDLE.to_vec();
        tampered[HEADER_LEN] ^= 1;
        assert_eq!(
            load_swarm_disaster_bundle(&tampered).map(|value| value.0),
            Err(SwarmDisasterBundleLoadError::BundleDigest)
        );
        assert_eq!(
            load_swarm_disaster_bundle(b"not-sora").map(|value| value.0),
            Err(SwarmDisasterBundleLoadError::BundleDigest)
        );
        assert_eq!(
            table_section_names(b"not-sora"),
            Err(SwarmDisasterBundleLoadError::BundleFormat)
        );
        assert_eq!(
            validate_schema_fingerprint("wrong"),
            Err(SwarmDisasterBundleLoadError::SchemaFingerprint)
        );
        assert_eq!(
            validate_table_closure(&["unexpected".to_owned()], &["expected"]),
            Err(SwarmDisasterBundleLoadError::TableClosure)
        );
    }

    #[test]
    fn rejects_manifest_revision_and_denominator_drift() {
        let bundle = SoraBundle::parse(BUNDLE).unwrap();
        let config = SoraConfig::from_source(&bundle).unwrap();
        let mut manifest = config.swarm_disaster_manifest().get(&1).unwrap().clone();
        manifest.schema_revision = "wrong".to_owned();
        assert_eq!(
            validate_manifest(&manifest),
            Err(SwarmDisasterBundleLoadError::ManifestRevision)
        );
        manifest.schema_revision = EXPECTED_MANIFEST_REVISION.to_owned();
        manifest.mechanic_rule_count = 22;
        assert_eq!(
            validate_manifest(&manifest),
            Err(SwarmDisasterBundleLoadError::RowDenominator)
        );
    }
}
