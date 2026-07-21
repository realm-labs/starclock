//! Private generated-reader to stable Starclock metadata boundary.

use crate::generated::{SoraConfig, runtime::SoraBundle};

/// Stable metadata inspected before domain catalog construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleMetadata {
    /// Authored game-version snapshot.
    pub game_version: String,
    /// Stable data revision from the singleton manifest.
    pub data_revision: String,
    /// Rules compatibility revision required by the bundle.
    pub required_rules_revision: String,
    /// Frozen goal-coverage manifest digest.
    pub coverage_manifest_sha256: String,
    /// Number of transport identities present in the bundle.
    pub identity_count: usize,
    /// Number of identities currently enabled for domain conversion.
    pub enabled_identity_count: usize,
}

/// Bundle-format or generated-reader failure without exposing a Sora type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleLoadError {
    message: String,
}

impl std::fmt::Display for BundleLoadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for BundleLoadError {}

/// Loads the Sora binary envelope and returns only stable boundary metadata.
///
/// Domain catalog construction and cross-row validation belong to
/// `G01-P1-B11`; diagnostic JSON is intentionally not accepted here.
pub fn inspect(bytes: &[u8]) -> Result<BundleMetadata, BundleLoadError> {
    let bundle = SoraBundle::parse(bytes).map_err(load_error)?;
    let config = SoraConfig::from_source(&bundle).map_err(load_error)?;
    let manifest = config.config_manifest();
    let identities = config.content_identity();
    Ok(BundleMetadata {
        game_version: manifest.game_version.clone(),
        data_revision: manifest.data_revision.clone(),
        required_rules_revision: manifest.required_rules_revision.clone(),
        coverage_manifest_sha256: manifest.coverage_manifest_sha256.clone(),
        identity_count: identities.len(),
        enabled_identity_count: identities.values().filter(|row| row.enabled).count(),
    })
}

fn load_error(error: impl std::fmt::Display) -> BundleLoadError {
    BundleLoadError {
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::inspect;

    const PRODUCTION_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");

    #[test]
    fn production_bundle_exposes_the_standard_v1_metadata_boundary() {
        let metadata = inspect(PRODUCTION_BUNDLE).expect("production bundle must load");
        assert_eq!(metadata.game_version, "4.4");
        assert_eq!(metadata.data_revision, "core-combat-v1-phase7-l02");
        assert_eq!(metadata.identity_count, 4577);
        assert_eq!(metadata.enabled_identity_count, 4444);
        assert_eq!(
            metadata.coverage_manifest_sha256,
            "e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19"
        );
    }

    #[test]
    fn diagnostic_json_is_not_a_runtime_input() {
        let error = inspect(br#"{"table":{"name":"ContentIdentity"}}"#)
            .expect_err("JSON must not be accepted as a Sora bundle");
        assert!(error.to_string().contains("magic"));
    }
}
