use crate::catalog::{
    CatalogLoadError, CatalogLoadErrorKind, CatalogManifest, fail, valid_date, valid_sha256,
};
use crate::generated::SoraConfig;

pub(super) fn convert_manifest(config: &SoraConfig) -> Result<CatalogManifest, CatalogLoadError> {
    let row = config.config_manifest();
    if row.sora_cli_version != "0.3.0" {
        return Err(fail(
            CatalogLoadErrorKind::Manifest,
            format!(
                "unsupported Sora authoring version {}",
                row.sora_cli_version
            ),
        ));
    }
    if !valid_date(&row.snapshot_date) {
        return Err(fail(
            CatalogLoadErrorKind::Manifest,
            "invalid snapshot date",
        ));
    }
    if !valid_sha256(&row.coverage_manifest_sha256) {
        return Err(fail(
            CatalogLoadErrorKind::Manifest,
            "coverage manifest digest is not lowercase SHA-256",
        ));
    }
    for (name, value) in [
        ("game_version", row.game_version.as_str()),
        ("data_revision", row.data_revision.as_str()),
        (
            "required_rules_revision",
            row.required_rules_revision.as_str(),
        ),
        (
            "numeric_policy_revision",
            row.numeric_policy_revision.as_str(),
        ),
        (
            "rng_algorithm_revision",
            row.rng_algorithm_revision.as_str(),
        ),
        ("state_hash_revision", row.state_hash_revision.as_str()),
        ("replay_format_version", row.replay_format_version.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(fail(
                CatalogLoadErrorKind::Manifest,
                format!("manifest field {name} is empty"),
            ));
        }
    }
    Ok(CatalogManifest {
        game_version: row.game_version.clone(),
        snapshot_date: row.snapshot_date.clone(),
        data_revision: row.data_revision.clone(),
        required_rules_revision: row.required_rules_revision.clone(),
        sora_cli_version: row.sora_cli_version.clone(),
        numeric_policy_revision: row.numeric_policy_revision.clone(),
        rng_algorithm_revision: row.rng_algorithm_revision.clone(),
        state_hash_revision: row.state_hash_revision.clone(),
        replay_format_version: row.replay_format_version.clone(),
        coverage_manifest_sha256: row.coverage_manifest_sha256.clone(),
    })
}
