//! Owner attribution is local to one operation; the trigger snapshot is unchanged.

use crate::{
    EventId, Ratio, SelectorId, UnitId,
    battle::fault::BattleFault,
    catalog::{
        CombatCatalog,
        action::{HitCritPolicy, OrdinaryDamageDefinition, OrdinaryDamageMultipliers},
    },
    event::cause::Cause,
    operation::{DamageOp, HitOperationScratch, Operation},
    resolver::{
        operation::execute_operation,
        program::{
            AbilityProgramContext,
            emission::emission_targets,
            fault::{emission_code, program_fault},
            value::{non_negative_scalar, scale},
        },
        transaction::Transaction,
    },
    rule::model::RuleEmission,
};

pub(super) fn execute(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    context: &AbilityProgramContext,
    resolved: &[(SelectorId, Box<[UnitId]>)],
    emission: RuleEmission,
) -> Result<EventId, BattleFault> {
    let RuleEmission::DamageFromOwner {
        selector,
        amount,
        class,
        element,
        can_crit,
        can_defeat,
        current_target,
    } = emission
    else {
        return Err(program_fault(12, emission_code(&emission)));
    };
    let amount = scale(non_negative_scalar(amount)?, context.damage_share)?;
    let formula = OrdinaryDamageDefinition::new(
        amount,
        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).expect("neutral multipliers are valid"),
    )
    .map_err(|_| program_fault(2, amount.scaled()))?
    .with_class(class);
    let targets = emission_targets(catalog, resolved, selector, current_target)?;
    let operation = Operation::Damage(DamageOp {
        id: txn.allocate_operation(),
        targets,
        formula,
        element: Some(element),
        crit_policy: if can_crit {
            context.crit_policy
        } else {
            HitCritPolicy::Never
        },
        apply_source_modifiers: true,
        ultimate_semantics: false,
        minimum_hp: i64::from(!can_defeat),
    });
    // Neither the observed actor's Crit cache nor an earlier owner's draw group
    // can determine this operation's result. Sibling emissions retain theirs.
    execute_operation(
        catalog,
        txn,
        cause.with_damage_source(context.owner),
        parent,
        operation,
        &mut HitOperationScratch::default(),
    )
}
