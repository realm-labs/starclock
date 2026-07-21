use starclock_combat::AbilityId;

use super::{CatalogLoadError, contiguous, positive, positive_u16};
use crate::generated::SoraConfig;

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
