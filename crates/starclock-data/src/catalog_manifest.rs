use crate::{
    catalog::{CatalogLoadError, CatalogLoadErrorKind, CatalogManifest, fail, valid_date},
    generated::SoraConfig,
};

pub(super) fn convert_manifest(config: &SoraConfig) -> Result<CatalogManifest, CatalogLoadError> {
    let row = config.config_manifest();
    if row.sora_cli_version != "0.6.1" {
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
    for (name, value) in [("game_version", row.game_version.as_str())] {
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
        sora_cli_version: row.sora_cli_version.clone(),
    })
}
