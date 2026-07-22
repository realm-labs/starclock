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
    use crate::generated::{SoraConfig, runtime::SoraBundle};

    const PRODUCTION_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");

    #[test]
    fn production_bundle_exposes_the_standard_v1_metadata_boundary() {
        let metadata = inspect(PRODUCTION_BUNDLE).expect("production bundle must load");
        assert_eq!(metadata.game_version, "4.4");
        assert_eq!(metadata.data_revision, "core-combat-v1-phase7-l11");
        assert_eq!(metadata.identity_count, 5278);
        assert_eq!(metadata.enabled_identity_count, 5278);
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

    #[test]
    fn goal03_fixture_bundle_loads_all_universe_tables() {
        let Ok(path) = std::env::var("STARCLOCK_G03_FIXTURE_BUNDLE") else {
            return;
        };
        let bytes = std::fs::read(path).expect("Goal 03 fixture bundle must be readable");
        let bundle = SoraBundle::parse(&bytes).expect("Goal 03 fixture must be a Sora bundle");
        let config = SoraConfig::from_source(&bundle).expect("all generated readers must load");
        let expected = std::env::var("STARCLOCK_G03_EXPECTED_PROFILES")
            .expect("fixture profile expectation must be provided")
            .parse::<usize>()
            .expect("fixture profile expectation must be an integer");
        assert_eq!(config.universe_profile().len(), expected);
        if expected > 0 {
            assert_eq!(config.universe_world().len(), 1);
            assert_eq!(config.universe_domain().len(), 1);
            assert_eq!(config.universe_activity_binding().len(), 1);
            assert_eq!(config.universe_source_record().len(), 1);
        }
    }
}
