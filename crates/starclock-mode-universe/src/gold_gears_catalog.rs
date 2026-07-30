//! Generated-row-free validation boundary for the frozen Gold and Gears bundle.

use crate::digest::bundle_digest;
use crate::gold_gears_generated::{
    ErasedSoraTable, SCHEMA_FINGERPRINT, SoraConfig,
    gold_gears_manifest::GoldGearsManifest,
    runtime::{SoraBundle, SoraTableSource},
};

const EXPECTED_BUNDLE_DIGEST: [u8; 32] = [
    0x97, 0xee, 0xfe, 0x25, 0x95, 0x4b, 0x16, 0xdf, 0x3b, 0x96, 0xc7, 0x13, 0x10, 0x1e, 0xd2, 0x8b,
    0xf2, 0x88, 0x06, 0xd0, 0xbd, 0xff, 0x0d, 0x89, 0x25, 0xb0, 0x73, 0x4a, 0x75, 0x6b, 0xfe, 0x7b,
];
const EXPECTED_SCHEMA_FINGERPRINT: &str = "5d5e76d3dbe1afca";
const EXPECTED_TABLES: usize = 52;
const EXPECTED_ROWS: usize = 29_140;
const EXPECTED_GOAL_ID: &str = "gold-and-gears-reference-v1";
const EXPECTED_PROFILE_ID: &str = "gold-gears.profile.v1";
const EXPECTED_MANIFEST_REVISION: &str = "starclock.gold-and-gears-pack-manifest.v1";
const EXPECTED_MANIFEST_KEY: &str = "gold-gears.manifest.v1";
const EXPECTED_CONTENT_MANIFEST_DIGEST: &str =
    "88885b409da0037b4db6a41fcfc6adbbb1bc15a681c519e192251e7fef476085";
const EXPECTED_RUNTIME_MARKER: &str = "ForbiddenReferenceOnly";
const SECTION_KIND_TABLE: usize = 2;
const HEADER_LEN: usize = 24;
const SECTION_ENTRY_LEN: usize = 28;

/// Generated-row-free facts proven while loading the immutable Candidate bundle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldAndGearsBundleSummary {
    bundle_digest: [u8; 32],
    table_count: usize,
    row_count: usize,
    source_obligations: usize,
    mechanic_rules: usize,
    semantic_fixtures: usize,
    policy_boundaries: usize,
}

impl GoldAndGearsBundleSummary {
    #[must_use]
    pub const fn bundle_digest(self) -> [u8; 32] {
        self.bundle_digest
    }
    #[must_use]
    pub const fn table_count(self) -> usize {
        self.table_count
    }
    #[must_use]
    pub const fn row_count(self) -> usize {
        self.row_count
    }
    #[must_use]
    pub const fn source_obligations(self) -> usize {
        self.source_obligations
    }
    #[must_use]
    pub const fn mechanic_rules(self) -> usize {
        self.mechanic_rules
    }
    #[must_use]
    pub const fn semantic_fixtures(self) -> usize {
        self.semantic_fixtures
    }
    #[must_use]
    pub const fn policy_boundaries(self) -> usize {
        self.policy_boundaries
    }
}

/// Stable pre-domain-lowering failure family. Generated Sora errors stay private.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoldAndGearsBundleLoadError {
    BundleDigest,
    BundleFormat,
    SchemaFingerprint,
    TableClosure,
    ManifestRevision,
    RowDenominator,
}

impl core::fmt::Display for GoldAndGearsBundleLoadError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::BundleDigest => "Gold and Gears bundle digest mismatch",
            Self::BundleFormat => "Gold and Gears bundle format is invalid",
            Self::SchemaFingerprint => "Gold and Gears schema fingerprint mismatch",
            Self::TableClosure => "Gold and Gears table closure mismatch",
            Self::ManifestRevision => "Gold and Gears manifest revision mismatch",
            Self::RowDenominator => "Gold and Gears row denominator mismatch",
        })
    }
}

impl std::error::Error for GoldAndGearsBundleLoadError {}

/// Loads all 52 generated tables and returns only Starclock-owned summary values.
pub fn validate_gold_and_gears_bundle(
    bytes: &[u8],
) -> Result<GoldAndGearsBundleSummary, GoldAndGearsBundleLoadError> {
    let digest = bundle_digest(bytes).bytes();
    if digest != EXPECTED_BUNDLE_DIGEST {
        return Err(GoldAndGearsBundleLoadError::BundleDigest);
    }
    let raw_tables = table_section_names(bytes)?;
    let bundle = SoraBundle::parse(bytes).map_err(|_| GoldAndGearsBundleLoadError::BundleFormat)?;
    validate_schema_fingerprint(
        bundle
            .schema_fingerprint()
            .map_err(|_| GoldAndGearsBundleLoadError::BundleFormat)?,
    )?;
    let config =
        SoraConfig::from_source(&bundle).map_err(|_| GoldAndGearsBundleLoadError::BundleFormat)?;
    let mut loaded_tables = config
        .tables()
        .map(|table| table.info().name)
        .collect::<Vec<_>>();
    loaded_tables.sort_unstable();
    validate_table_closure(&raw_tables, &loaded_tables)?;
    let row_count = config.tables().map(ErasedSoraTable::len).sum::<usize>();
    if loaded_tables.len() != EXPECTED_TABLES || row_count != EXPECTED_ROWS {
        return Err(GoldAndGearsBundleLoadError::RowDenominator);
    }
    let manifest = config
        .gold_gears_manifest()
        .get(&1)
        .ok_or(GoldAndGearsBundleLoadError::ManifestRevision)?;
    validate_manifest(manifest)?;
    Ok(GoldAndGearsBundleSummary {
        bundle_digest: digest,
        table_count: loaded_tables.len(),
        row_count,
        source_obligations: usize::try_from(manifest.frozen_source_obligations)
            .map_err(|_| GoldAndGearsBundleLoadError::RowDenominator)?,
        mechanic_rules: usize::try_from(manifest.mechanic_rule_count)
            .map_err(|_| GoldAndGearsBundleLoadError::RowDenominator)?,
        semantic_fixtures: usize::try_from(manifest.semantic_fixture_family_count)
            .map_err(|_| GoldAndGearsBundleLoadError::RowDenominator)?,
        policy_boundaries: usize::try_from(manifest.research_gap_count)
            .map_err(|_| GoldAndGearsBundleLoadError::RowDenominator)?,
    })
}

fn validate_schema_fingerprint(value: &str) -> Result<(), GoldAndGearsBundleLoadError> {
    if value != EXPECTED_SCHEMA_FINGERPRINT || value != SCHEMA_FINGERPRINT {
        return Err(GoldAndGearsBundleLoadError::SchemaFingerprint);
    }
    Ok(())
}

fn validate_table_closure(
    raw: &[String],
    loaded: &[&str],
) -> Result<(), GoldAndGearsBundleLoadError> {
    if raw.len() != EXPECTED_TABLES
        || loaded.len() != EXPECTED_TABLES
        || !raw.iter().map(String::as_str).eq(loaded.iter().copied())
    {
        return Err(GoldAndGearsBundleLoadError::TableClosure);
    }
    Ok(())
}

fn validate_manifest(manifest: &GoldGearsManifest) -> Result<(), GoldAndGearsBundleLoadError> {
    if manifest.stable_key != EXPECTED_MANIFEST_KEY
        || manifest.schema_revision != EXPECTED_MANIFEST_REVISION
        || manifest.goal_id != EXPECTED_GOAL_ID
        || manifest.profile_id != EXPECTED_PROFILE_ID
        || manifest.content_manifest_sha256 != EXPECTED_CONTENT_MANIFEST_DIGEST
        || manifest.runtime_loading != EXPECTED_RUNTIME_MARKER
        || !manifest.candidate_quality
    {
        return Err(GoldAndGearsBundleLoadError::ManifestRevision);
    }
    if manifest.frozen_source_obligations != 7_913
        || manifest.data_ready_source_obligations != 7_913
        || manifest.mechanic_rule_count != 1_224
        || manifest.semantic_fixture_family_count != 18
        || manifest.research_gap_count != 16
        || manifest.blocking_research_gap_count != 0
    {
        return Err(GoldAndGearsBundleLoadError::RowDenominator);
    }
    Ok(())
}

fn table_section_names(bytes: &[u8]) -> Result<Vec<String>, GoldAndGearsBundleLoadError> {
    if bytes.len() < HEADER_LEN || &bytes[..4] != b"SORA" {
        return Err(GoldAndGearsBundleLoadError::BundleFormat);
    }
    let header_len = read_u32(bytes, 8)?;
    let directory_len = read_u32(bytes, 12)?;
    let section_count = read_u32(bytes, 16)?;
    if header_len != HEADER_LEN {
        return Err(GoldAndGearsBundleLoadError::BundleFormat);
    }
    let directory_end = header_len
        .checked_add(directory_len)
        .filter(|end| *end <= bytes.len())
        .ok_or(GoldAndGearsBundleLoadError::BundleFormat)?;
    let mut cursor = header_len;
    let mut names = Vec::new();
    for _ in 0..section_count {
        let entry_end = cursor
            .checked_add(SECTION_ENTRY_LEN)
            .filter(|end| *end <= directory_end)
            .ok_or(GoldAndGearsBundleLoadError::BundleFormat)?;
        let kind = read_u32(bytes, cursor)?;
        let name_len = read_u32(bytes, cursor + 8)?;
        let name_end = entry_end
            .checked_add(name_len)
            .filter(|end| *end <= directory_end)
            .ok_or(GoldAndGearsBundleLoadError::BundleFormat)?;
        if kind == SECTION_KIND_TABLE {
            names.push(
                std::str::from_utf8(&bytes[entry_end..name_end])
                    .map_err(|_| GoldAndGearsBundleLoadError::BundleFormat)?
                    .to_owned(),
            );
        }
        cursor = name_end;
    }
    if cursor != directory_end {
        return Err(GoldAndGearsBundleLoadError::BundleFormat);
    }
    names.sort_unstable();
    Ok(names)
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<usize, GoldAndGearsBundleLoadError> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or(GoldAndGearsBundleLoadError::BundleFormat)?;
    Ok(u32::from_le_bytes(
        value
            .try_into()
            .map_err(|_| GoldAndGearsBundleLoadError::BundleFormat)?,
    ) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUNDLE: &[u8] = include_bytes!("../../../config/gold-and-gears-generated/config.sora");

    #[test]
    fn loads_exact_candidate_bundle_without_exposing_generated_rows() {
        let summary = validate_gold_and_gears_bundle(BUNDLE).unwrap();
        assert_eq!(summary.bundle_digest(), EXPECTED_BUNDLE_DIGEST);
        assert_eq!(summary.table_count(), 52);
        assert_eq!(summary.row_count(), 29_140);
        assert_eq!(summary.source_obligations(), 7_913);
        assert_eq!(summary.mechanic_rules(), 1_224);
        assert_eq!(summary.semantic_fixtures(), 18);
        assert_eq!(summary.policy_boundaries(), 16);
    }

    #[test]
    fn rejects_digest_format_schema_and_table_closure_drift() {
        let mut tampered = BUNDLE.to_vec();
        tampered[HEADER_LEN] ^= 1;
        assert_eq!(
            validate_gold_and_gears_bundle(&tampered),
            Err(GoldAndGearsBundleLoadError::BundleDigest)
        );
        assert_eq!(
            validate_gold_and_gears_bundle(b"not-sora"),
            Err(GoldAndGearsBundleLoadError::BundleDigest)
        );
        assert_eq!(
            validate_schema_fingerprint("wrong"),
            Err(GoldAndGearsBundleLoadError::SchemaFingerprint)
        );
        assert_eq!(
            validate_table_closure(&["unexpected".to_owned()], &["expected"]),
            Err(GoldAndGearsBundleLoadError::TableClosure)
        );
    }

    #[test]
    fn rejects_manifest_revision_and_denominator_drift() {
        let bundle = SoraBundle::parse(BUNDLE).unwrap();
        let config = SoraConfig::from_source(&bundle).unwrap();
        let mut manifest = config.gold_gears_manifest().get(&1).unwrap().clone();
        manifest.schema_revision = "wrong".to_owned();
        assert_eq!(
            validate_manifest(&manifest),
            Err(GoldAndGearsBundleLoadError::ManifestRevision)
        );
        manifest.schema_revision = EXPECTED_MANIFEST_REVISION.to_owned();
        manifest.mechanic_rule_count = 1_223;
        assert_eq!(
            validate_manifest(&manifest),
            Err(GoldAndGearsBundleLoadError::RowDenominator)
        );
    }
}
