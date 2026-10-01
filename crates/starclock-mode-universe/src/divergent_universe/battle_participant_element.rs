//! Immutable mapped-character Basic element admission for equipment effects.
use crate::divergent_universe::DivergentUniverseBattleAssemblyError;
use starclock_combat::{
    ParticipantSpec,
    catalog::action::{AbilityKind, HitOperationDefinition},
    formula::model::CombatElement,
};
use starclock_data::catalog::SimulationCatalog;

pub(super) fn basic_element(
    core: &SimulationCatalog,
    player: &ParticipantSpec,
) -> Result<CombatElement, DivergentUniverseBattleAssemblyError> {
    let invalid = || DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants;
    let mut element = None;
    for id in player.combatant().abilities() {
        let definition = core.combat_catalog().ability(*id).ok_or_else(invalid)?;
        let Some(action) = definition
            .action()
            .filter(|action| action.kind() == AbilityKind::Basic)
        else {
            continue;
        };
        for operation in action.hits().iter().flat_map(|hit| hit.operations()) {
            if let HitOperationDefinition::ScalingDamage(damage) = operation {
                if element.is_some_and(|previous| previous != damage.element()) {
                    return Err(invalid());
                }
                element = Some(damage.element());
            }
        }
    }
    element.ok_or_else(invalid)
}
