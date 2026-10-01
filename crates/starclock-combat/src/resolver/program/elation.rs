//! Lower an evaluated dedicated proposal using the enclosing hit's metadata.
use super::{AbilityProgramContext, emission_targets, program_fault};
use crate::{
    OperationId, SelectorId, UnitId,
    battle::fault::BattleFault,
    catalog::{CombatCatalog, action::HitCritPolicy},
    operation::{ElationDamageOp, Operation},
    rule::model::RuleEmission,
};

pub(super) fn operation(
    catalog: &CombatCatalog,
    context: &AbilityProgramContext,
    resolved: &[(SelectorId, Box<[UnitId]>)],
    id: OperationId,
    emission: RuleEmission,
) -> Result<Operation, BattleFault> {
    let RuleEmission::ElationDamage {
        selector,
        definition,
        can_crit,
        current_target,
    } = emission
    else {
        return Err(program_fault(90, 0));
    };
    Ok(Operation::ElationDamage(ElationDamageOp {
        id,
        targets: emission_targets(catalog, resolved, selector, current_target)?,
        definition: definition
            .with_share(context.damage_share)
            .map_err(|_| program_fault(42, context.damage_share.scaled()))?,
        crit_policy: if can_crit {
            context.crit_policy
        } else {
            HitCritPolicy::Never
        },
    }))
}
