//! Cross-definition validation for generic effect runtime ownership.

use crate::{
    DotDetonationFilter, EffectCategory, EffectRuntimeDefinition, EffectRuntimeTemplate,
    EffectTickPhase,
    catalog::{CombatCatalog, definition::EffectDefinition},
    damage::DamageSemantics,
    formula::model::DamageClass,
};

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
        validate_magnitude_slots(catalog, effect)?;
        let classifications = effect.damage_classifications();
        if !classifications.is_empty()
            && ((effect.runtime().is_none() && effect.runtime_template().is_none())
                || classifications
                    .windows(2)
                    .any(|pair| pair[0].class >= pair[1].class)
                || classifications.iter().any(|entry| {
                    entry.class == DamageClass::Dot || entry.semantics == DamageSemantics::NONE
                }))
        {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                "damage classifications require a runtime and unique ordered nonempty non-DoT entries",
            ));
        }
        if effect.dot_family().is_some() {
            let damaging_dot = effect.runtime().is_some_and(|runtime| {
                runtime.category() == EffectCategory::Dot && runtime.dot().is_some()
            }) || effect.runtime_template().is_some_and(|template| {
                template.category() == EffectCategory::Dot && template.has_dot()
            });
            if !damaging_dot {
                return Err(error(
                    CatalogBuildErrorKind::InvalidDefinition,
                    format!(
                        "effect {} classifies a family without a damaging DoT runtime",
                        id.get()
                    ),
                ));
            }
        }
        let tick_phase = effect
            .runtime()
            .map(EffectRuntimeDefinition::tick_phase)
            .or_else(|| {
                effect
                    .runtime_template()
                    .map(EffectRuntimeTemplate::tick_phase)
            });
        if tick_phase == Some(EffectTickPhase::AfterEvent) && effect.rules().is_empty() {
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

fn validate_magnitude_slots(
    catalog: &CombatCatalog,
    effect: &EffectDefinition,
) -> Result<(), CatalogBuildError> {
    let bindings = effect.modifier_magnitude_slots();
    if bindings.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
        return Err(error(
            CatalogBuildErrorKind::InvalidDefinition,
            "effect magnitude bindings must have unique ordered modifier IDs",
        ));
    }
    for (modifier, slot) in bindings {
        let definition = catalog.modifier(*modifier).ok_or_else(|| {
            error(
                CatalogBuildErrorKind::MissingReference,
                "missing magnitude modifier",
            )
        })?;
        if effect.modifiers().binary_search(modifier).is_err()
            || definition.source_stack_slot == Some(*slot)
            || catalog
                .effects
                .ids()
                .filter(|id| {
                    catalog
                        .effects
                        .get(*id)
                        .is_some_and(|owner| owner.modifiers().binary_search(modifier).is_ok())
                })
                .count()
                != 1
        {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                "effect magnitude modifier needs one attached owner and a distinct Scalar slot",
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_detonation_filter(
    catalog: &CombatCatalog,
    filter: DotDetonationFilter,
) -> Result<(), CatalogBuildError> {
    if let Some(effect) = filter.excluded_effect()
        && catalog.effect(effect).is_none()
    {
        return Err(error(
            CatalogBuildErrorKind::MissingReference,
            format!("detonation excludes missing effect {}", effect.get()),
        ));
    }
    Ok(())
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
