use starclock_combat::{AbilityId, EffectCategory, EffectRuntimeTemplate, ForcedNormalAction};

use super::{CatalogLoadError, CatalogLoadErrorKind, contiguous, fail, positive, positive_u16};
use crate::generated::SoraConfig;

pub(super) fn tags(config: &SoraConfig, effect_id: i32) -> Result<Vec<&str>, CatalogLoadError> {
    let mut bindings = config
        .effect_tag()
        .iter()
        .filter(|tag| tag.effect_id == effect_id)
        .collect::<Vec<_>>();
    bindings.sort_unstable_by_key(|tag| tag.sequence);
    contiguous(
        bindings
            .iter()
            .map(|tag| positive_u16(tag.sequence, "EffectTag.sequence"))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter(),
        "effect tags",
    )?;
    Ok(bindings.iter().map(|tag| tag.tag.as_str()).collect())
}

pub(super) fn apply_runtime_tags(
    effect_id: i32,
    category: EffectCategory,
    tags: &[&str],
    mut template: EffectRuntimeTemplate,
) -> Result<EffectRuntimeTemplate, CatalogLoadError> {
    if tags.contains(&"prevents-toughness-reduction") {
        template = template.with_toughness_protection();
    }
    if tags.contains(&"forced-basic-attack-random-ally") {
        if category != EffectCategory::Control {
            return Err(fail(
                CatalogLoadErrorKind::Domain,
                format!("effect {effect_id} declares a forced normal action outside Control"),
            ));
        }
        template = template
            .with_forced_normal_action(ForcedNormalAction::BasicAttackRandomAlly)
            .expect("Control category accepts a forced normal action");
    }
    if tags.contains(&"forced-basic-attack-applier") {
        if category != EffectCategory::Control {
            return Err(fail(
                CatalogLoadErrorKind::Domain,
                format!("effect {effect_id} declares a forced normal action outside Control"),
            ));
        }
        template = template
            .with_forced_normal_action(ForcedNormalAction::BasicAttackApplier)
            .expect("Control category accepts a forced normal action");
    }
    Ok(template)
}

pub(super) fn granted_abilities(
    config: &SoraConfig,
    effect_id: i32,
) -> Result<Vec<AbilityId>, CatalogLoadError> {
    let mut bindings = config
        .effect_granted_ability()
        .iter()
        .filter(|binding| binding.effect_id == effect_id)
        .collect::<Vec<_>>();
    bindings.sort_unstable_by_key(|binding| binding.sequence);
    contiguous(
        bindings
            .iter()
            .map(|binding| positive_u16(binding.sequence, "EffectGrantedAbility.sequence"))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter(),
        "effect granted abilities",
    )?;
    bindings
        .into_iter()
        .map(|binding| {
            positive(binding.ability_id, "EffectGrantedAbility.ability_id")
                .map(|id| AbilityId::new(id).expect("positive ability ID"))
        })
        .collect()
}
