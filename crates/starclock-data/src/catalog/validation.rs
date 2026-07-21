use std::collections::BTreeMap;

use super::{
    CatalogLoadError, CatalogLoadErrorKind, IdentityDefinition, IdentityKind, LoadMode, fail,
};
use crate::generated::content_kind::ContentKind;

pub(crate) fn positive(value: i32, field: &str) -> Result<u32, CatalogLoadError> {
    u32::try_from(value)
        .ok()
        .filter(|value| *value != 0)
        .ok_or_else(|| {
            fail(
                CatalogLoadErrorKind::Domain,
                format!("{field} must be positive"),
            )
        })
}

pub(crate) fn positive_u16(value: i32, field: &str) -> Result<u16, CatalogLoadError> {
    let value = bounded_u16(value, field)?;
    if value == 0 {
        return Err(fail(
            CatalogLoadErrorKind::Domain,
            format!("{field} must be positive"),
        ));
    }
    Ok(value)
}

pub(super) fn bounded_u16(value: i32, field: &str) -> Result<u16, CatalogLoadError> {
    u16::try_from(value).map_err(|_| {
        fail(
            CatalogLoadErrorKind::Domain,
            format!("{field} is outside the domain range"),
        )
    })
}

pub(crate) fn contiguous(
    values: impl Iterator<Item = u16>,
    description: &str,
) -> Result<(), CatalogLoadError> {
    for (index, value) in values.enumerate() {
        if value as usize != index + 1 {
            return Err(fail(
                CatalogLoadErrorKind::Domain,
                format!("{description} are not contiguous from one"),
            ));
        }
    }
    Ok(())
}

pub(crate) fn require_identity(
    identities: &BTreeMap<u32, &IdentityDefinition>,
    id: u32,
    kind: IdentityKind,
    mode: LoadMode,
) -> Result<(), CatalogLoadError> {
    let identity = identities.get(&id).ok_or_else(|| {
        fail(
            CatalogLoadErrorKind::Domain,
            format!("definition {id} has no content identity"),
        )
    })?;
    if identity.kind != kind {
        return Err(fail(
            CatalogLoadErrorKind::Domain,
            format!(
                "identity {} ({}) has the wrong content kind",
                id, identity.stable_key
            ),
        ));
    }
    if mode == LoadMode::Production && !identity.enabled {
        return Err(fail(
            CatalogLoadErrorKind::Domain,
            format!(
                "disabled identity {} has executable rows",
                identity.stable_key
            ),
        ));
    }
    Ok(())
}

pub(super) fn identity_kind(kind: ContentKind) -> IdentityKind {
    match kind {
        ContentKind::CharacterForm => IdentityKind::Character,
        ContentKind::LightCone => IdentityKind::LightCone,
        ContentKind::Ability => IdentityKind::Ability,
        ContentKind::Program => IdentityKind::Program,
        _ => IdentityKind::Other,
    }
}
