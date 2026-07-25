//! Cross-definition validation for generic effect runtime ownership.

use crate::catalog::CombatCatalog;

use super::{CatalogBuildError, CatalogBuildErrorKind, error};

pub(super) fn validate(catalog: &CombatCatalog) -> Result<(), CatalogBuildError> {
    for id in catalog.effects.ids() {
        let effect = catalog
            .effects
            .get(id)
            .expect("ID originated from this table");
        if effect.runtime().is_some() && effect.runtime_template().is_some() {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!("effect {} declares two runtime representations", id.get()),
            ));
        }
        let tick_phase = effect
            .runtime()
            .map(crate::EffectRuntimeDefinition::tick_phase)
            .or_else(|| {
                effect
                    .runtime_template()
                    .map(crate::EffectRuntimeTemplate::tick_phase)
            });
        if tick_phase == Some(crate::EffectTickPhase::AfterEvent) && effect.rules().is_empty() {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!(
                    "effect {} requires an attached rule for its event-driven tick",
                    id.get()
                ),
            ));
        }
    }
    validate_stack_modifier_owners(catalog)
}

fn validate_stack_modifier_owners(catalog: &CombatCatalog) -> Result<(), CatalogBuildError> {
    for modifier in catalog.modifiers.definitions() {
        if modifier.source_stack_slot.is_none() {
            continue;
        }
        let owners = catalog
            .effects
            .ids()
            .filter(|effect| {
                catalog.effects.get(*effect).is_some_and(|definition| {
                    definition.modifiers().binary_search(&modifier.id).is_ok()
                })
            })
            .count();
        if owners != 1 {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!(
                    "source-stack modifier {} must belong to exactly one effect",
                    modifier.id.get()
                ),
            ));
        }
    }
    Ok(())
}
