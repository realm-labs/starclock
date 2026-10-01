//! Family roots are explicit immutable definitions, never numeric ID inference.
use crate::catalog::{
    CombatCatalog,
    builder::{CatalogBuildError, CatalogBuildErrorKind, error},
};

pub(super) fn validate(catalog: &CombatCatalog) -> Result<(), CatalogBuildError> {
    for ability in catalog.abilities.values() {
        let family = catalog.ability(ability.family()).ok_or_else(|| {
            error(
                CatalogBuildErrorKind::MissingReference,
                format!("ability {} has a missing family root", ability.id().get()),
            )
        })?;
        if family.family() != family.id() {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!(
                    "ability {} has a non-root family reference",
                    ability.id().get()
                ),
            ));
        }
        if family.action().map(|action| action.kind())
            != ability.action().map(|action| action.kind())
        {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!(
                    "ability {} changes its family's action kind",
                    ability.id().get()
                ),
            ));
        }
    }
    Ok(())
}
