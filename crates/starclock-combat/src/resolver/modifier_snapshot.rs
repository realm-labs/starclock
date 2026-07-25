//! Runtime modifier snapshot capture at explicit lifecycle boundaries.

use std::collections::BTreeSet;

use crate::{
    battle::fault::BattleFault,
    modifier::model::{ActiveModifier, SnapshotPolicy, StatQuery, StatQuerySubject},
    rule::model::{ConditionExpr, ValueExpr},
};

use super::{
    journal::MutationField,
    transaction::{Transaction, action_fault},
};

pub(crate) fn initialize_battle(
    catalog: &crate::catalog::CombatCatalog,
    state: &mut crate::battle::state::BattleState,
) -> Result<(), u32> {
    let bases = stat_bases(state).map_err(|_| 0_u32)?;
    let shields = state_shield_values(state);
    let active = state.modifiers.iter_by_id().cloned().collect::<Vec<_>>();
    for source in &active {
        let definition = catalog
            .modifier(source.definition)
            .ok_or_else(|| source.definition.get())?;
        if !matches!(
            definition.snapshot,
            SnapshotPolicy::OnApplication
                | SnapshotPolicy::RecomputeOnStackChange
                | SnapshotPolicy::SourceSnapshotTargetDynamic
                | SnapshotPolicy::SourceDynamicTargetSnapshot
                | SnapshotPolicy::ExplicitFields
        ) {
            continue;
        }
        let peers = active
            .iter()
            .filter(|candidate| candidate.instance != source.instance)
            .cloned()
            .collect::<Vec<_>>();
        let resolver = crate::modifier::resolve::StatResolver::new(
            catalog.modifier_registry(),
            &bases,
            &peers,
        )
        .with_shields(&shields);
        let mut captured_value = None;
        let mut captured_stats = None;
        match definition.snapshot {
            SnapshotPolicy::OnApplication | SnapshotPolicy::RecomputeOnStackChange => {
                captured_value = Some(
                    resolver
                        .capture_value(source, definition)
                        .map_err(|_| source.definition.get())?,
                );
            }
            SnapshotPolicy::SourceSnapshotTargetDynamic
            | SnapshotPolicy::SourceDynamicTargetSnapshot
            | SnapshotPolicy::ExplicitFields => {
                captured_stats = Some(
                    capture_stats(&resolver, source, definition)
                        .map_err(|_| source.definition.get())?
                        .into_boxed_slice(),
                );
            }
            _ => {}
        }
        let target = state
            .modifiers
            .get_mut(source.instance)
            .ok_or_else(|| source.definition.get())?;
        if let Some(value) = captured_value {
            target.captured_value = Some(value);
        }
        if let Some(values) = captured_stats {
            target.captured_stats = values;
        }
    }
    Ok(())
}

pub(super) fn initialize(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    instance: &mut ActiveModifier,
) -> Result<(), BattleFault> {
    let definition = catalog
        .modifier(instance.definition)
        .ok_or_else(|| action_fault(135))?;
    let bases = super::program::stat_bases(txn)?;
    let shields = super::stat_input::shield_values(txn);
    let active = txn
        .state
        .modifiers
        .iter_by_id()
        .cloned()
        .collect::<Vec<_>>();
    let resolver =
        crate::modifier::resolve::StatResolver::new(catalog.modifier_registry(), &bases, &active)
            .with_shields(&shields);
    match definition.snapshot {
        SnapshotPolicy::OnApplication | SnapshotPolicy::RecomputeOnStackChange => {
            instance.captured_value = Some(
                resolver
                    .capture_value(instance, definition)
                    .map_err(|_| action_fault(136))?,
            );
        }
        SnapshotPolicy::SourceSnapshotTargetDynamic
        | SnapshotPolicy::SourceDynamicTargetSnapshot
        | SnapshotPolicy::ExplicitFields => {
            instance.captured_stats =
                capture_stats(&resolver, instance, definition)?.into_boxed_slice();
        }
        SnapshotPolicy::Dynamic
        | SnapshotPolicy::OnActionStart
        | SnapshotPolicy::OnPhaseStart
        | SnapshotPolicy::OnHitStart => {}
    }
    Ok(())
}

pub(super) fn refresh(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    boundary: SnapshotPolicy,
) -> Result<(), BattleFault> {
    debug_assert!(matches!(
        boundary,
        SnapshotPolicy::OnActionStart | SnapshotPolicy::OnPhaseStart | SnapshotPolicy::OnHitStart
    ));
    let bases = super::program::stat_bases(txn)?;
    let shields = super::stat_input::shield_values(txn);
    let active = txn
        .state
        .modifiers
        .iter_by_id()
        .cloned()
        .collect::<Vec<_>>();
    let resolver =
        crate::modifier::resolve::StatResolver::new(catalog.modifier_registry(), &bases, &active)
            .with_shields(&shields);
    let mut updates = Vec::new();
    for instance in &active {
        let definition = catalog
            .modifier(instance.definition)
            .ok_or_else(|| action_fault(137))?;
        if definition.snapshot == boundary {
            updates.push((
                instance.instance,
                resolver
                    .capture_value(instance, definition)
                    .map_err(|_| action_fault(138))?,
            ));
        }
    }
    for (id, value) in updates {
        let instance = txn
            .state
            .modifiers
            .get_mut(id)
            .ok_or_else(|| action_fault(139))?;
        if instance.captured_value != Some(value) {
            instance.captured_value = Some(value);
            txn.journal.mutation(
                MutationField::ModifierStore,
                id.get(),
                id.get().rotate_left(1),
            );
        }
    }
    Ok(())
}

pub(super) fn refresh_effect_stacks(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    effect: crate::EffectInstanceId,
    stacks: u16,
) -> Result<(), BattleFault> {
    let bindings = txn
        .state
        .modifiers
        .iter_by_id()
        .filter(|instance| instance.source_effect == Some(effect))
        .filter_map(|instance| {
            catalog
                .modifier(instance.definition)
                .and_then(|definition| definition.source_stack_slot)
                .map(|slot| (instance.instance, slot))
        })
        .collect::<Vec<_>>();
    for (instance, slot) in &bindings {
        let modifier = txn
            .state
            .modifiers
            .get_mut(*instance)
            .ok_or_else(|| action_fault(141))?;
        if !modifier.set_slot(
            slot.to_owned(),
            crate::rule::model::RuleValue::Integer(i64::from(stacks)),
        ) {
            return Err(action_fault(142));
        }
        txn.journal.mutation(
            MutationField::ModifierStore,
            instance.get(),
            u64::from(stacks),
        );
    }
    let bases = super::program::stat_bases(txn)?;
    let shields = super::stat_input::shield_values(txn);
    let active = txn
        .state
        .modifiers
        .iter_by_id()
        .cloned()
        .collect::<Vec<_>>();
    for (instance, _) in bindings {
        let current = active
            .iter()
            .find(|candidate| candidate.instance == instance)
            .ok_or_else(|| action_fault(143))?;
        let definition = catalog
            .modifier(current.definition)
            .ok_or_else(|| action_fault(144))?;
        if definition.snapshot != SnapshotPolicy::RecomputeOnStackChange {
            continue;
        }
        let peers = active
            .iter()
            .filter(|candidate| candidate.instance != instance)
            .cloned()
            .collect::<Vec<_>>();
        let resolver = crate::modifier::resolve::StatResolver::new(
            catalog.modifier_registry(),
            &bases,
            &peers,
        )
        .with_shields(&shields);
        let value = resolver
            .capture_value(current, definition)
            .map_err(|_| action_fault(145))?;
        txn.state
            .modifiers
            .get_mut(instance)
            .ok_or_else(|| action_fault(146))?
            .captured_value = Some(value);
    }
    Ok(())
}

fn capture_stats(
    resolver: &crate::modifier::resolve::StatResolver<'_>,
    instance: &ActiveModifier,
    definition: &crate::modifier::model::ModifierDefinition,
) -> Result<Vec<(StatQuery, crate::Scalar)>, BattleFault> {
    let mut queries = BTreeSet::new();
    collect_value_queries(
        &definition.value,
        definition.snapshot,
        instance,
        &mut queries,
    );
    queries
        .into_iter()
        .map(|query| {
            resolver
                .query(
                    query,
                    &crate::modifier::model::ModifierQueryContext::default(),
                )
                .map(|value| (query, value))
                .map_err(|_| action_fault(140))
        })
        .collect()
}

fn collect_value_queries(
    expression: &ValueExpr,
    policy: SnapshotPolicy,
    instance: &ActiveModifier,
    output: &mut BTreeSet<StatQuery>,
) {
    match expression {
        ValueExpr::QueryStat {
            subject,
            stat,
            purpose,
        } if captures_subject(policy, *subject) => {
            output.insert(StatQuery {
                subject: concrete_subject(instance, *subject),
                stat: *stat,
                purpose: *purpose,
            });
        }
        ValueExpr::QueryBaseStat { .. } => {}
        ValueExpr::SelectorSum { value, .. }
        | ValueExpr::Negate(value)
        | ValueExpr::Convert { value, .. } => {
            collect_value_queries(value, policy, instance, output);
        }
        ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs)
        | ValueExpr::Multiply { lhs, rhs, .. }
        | ValueExpr::Divide { lhs, rhs, .. } => {
            collect_value_queries(lhs, policy, instance, output);
            collect_value_queries(rhs, policy, instance, output);
        }
        ValueExpr::Clamp {
            value,
            minimum,
            maximum,
        } => {
            collect_value_queries(value, policy, instance, output);
            collect_value_queries(minimum, policy, instance, output);
            collect_value_queries(maximum, policy, instance, output);
        }
        ValueExpr::Choose {
            condition,
            when_true,
            when_false,
        } => {
            collect_condition_queries(condition, policy, instance, output);
            collect_value_queries(when_true, policy, instance, output);
            collect_value_queries(when_false, policy, instance, output);
        }
        ValueExpr::QueryStat { .. }
        | ValueExpr::QueryShield { .. }
        | ValueExpr::QueryEffectStacks { .. }
        | ValueExpr::QueryEffectCategoryStacks { .. }
        | ValueExpr::Literal(_)
        | ValueExpr::Slot(_)
        | ValueExpr::AbilityParameter { .. }
        | ValueExpr::ReadResource { .. }
        | ValueExpr::ReadEventProperty(_)
        | ValueExpr::SelectorCount(_)
        | ValueExpr::EventId
        | ValueExpr::EventOwner
        | ValueExpr::EventActor
        | ValueExpr::EventApplier
        | ValueExpr::EventTarget
        | ValueExpr::CurrentTarget => {}
    }
}

fn collect_condition_queries(
    condition: &ConditionExpr,
    policy: SnapshotPolicy,
    instance: &ActiveModifier,
    output: &mut BTreeSet<StatQuery>,
) {
    match condition {
        ConditionExpr::Not(value) => collect_condition_queries(value, policy, instance, output),
        ConditionExpr::All(values) | ConditionExpr::Any(values) => {
            for value in values {
                collect_condition_queries(value, policy, instance, output);
            }
        }
        ConditionExpr::Compare { lhs, rhs, .. } => {
            collect_value_queries(lhs, policy, instance, output);
            collect_value_queries(rhs, policy, instance, output);
        }
        ConditionExpr::Literal(_)
        | ConditionExpr::EventKind(_)
        | ConditionExpr::SourceTag(_)
        | ConditionExpr::LifePresence { .. }
        | ConditionExpr::EffectExists { .. }
        | ConditionExpr::IsFrozen(_)
        | ConditionExpr::HasWeakness { .. }
        | ConditionExpr::IsBroken(_)
        | ConditionExpr::SelectorCardinality { .. } => {}
    }
}

const fn captures_subject(policy: SnapshotPolicy, subject: StatQuerySubject) -> bool {
    use StatQuerySubject::{Actor, Applier, CurrentTarget, EventTarget, Owner};

    match policy {
        SnapshotPolicy::SourceSnapshotTargetDynamic => {
            matches!(subject, Owner | Actor | Applier)
        }
        SnapshotPolicy::SourceDynamicTargetSnapshot => {
            matches!(subject, EventTarget | CurrentTarget)
        }
        SnapshotPolicy::ExplicitFields => true,
        _ => false,
    }
}

const fn concrete_subject(instance: &ActiveModifier, subject: StatQuerySubject) -> crate::UnitId {
    use StatQuerySubject::{Actor, Applier, CurrentTarget, EventTarget, Owner};

    match subject {
        Owner | Actor | Applier => instance.owner,
        EventTarget | CurrentTarget => instance.subject,
    }
}

fn stat_bases(
    state: &crate::battle::state::BattleState,
) -> Result<
    std::collections::BTreeMap<(crate::UnitId, crate::modifier::model::StatKind), crate::Scalar>,
    crate::NumericError,
> {
    use crate::modifier::model::StatKind::{
        Atk, BreakBaseDamage, DebuffDurationMultiplier, Def, DotDurationAddition, FreezeResistance,
        Hp, Spd, ToughnessDamage,
    };

    let mut bases = std::collections::BTreeMap::new();
    for unit in state.units.iter_by_id() {
        bases.insert(
            (unit.id, Hp),
            crate::Scalar::checked_from_integer(unit.maximum_hp.get())?,
        );
        bases.insert(
            (unit.id, Atk),
            crate::Scalar::from_scaled(unit.base_attack.scaled()),
        );
        bases.insert(
            (unit.id, Def),
            crate::Scalar::from_scaled(unit.base_defense.scaled()),
        );
        bases.insert(
            (unit.id, Spd),
            crate::Scalar::from_scaled(unit.base_speed.scaled()),
        );
        bases.insert((unit.id, FreezeResistance), crate::Scalar::ZERO);
        bases.insert((unit.id, ToughnessDamage), crate::Scalar::ZERO);
        if let Some(value) = crate::formula::toughness::attacker_level_multiplier(unit.level) {
            bases.insert((unit.id, BreakBaseDamage), value);
        }
        bases.insert((unit.id, DotDurationAddition), crate::Scalar::ZERO);
        bases.insert((unit.id, DebuffDurationMultiplier), crate::Scalar::ONE);
    }
    Ok(bases)
}

fn state_shield_values(
    state: &crate::battle::state::BattleState,
) -> std::collections::BTreeMap<crate::UnitId, crate::Scalar> {
    state
        .units
        .iter_by_id()
        .map(|unit| {
            let value = state
                .shields
                .effective_remaining(unit.id)
                .ok()
                .and_then(|value| crate::Scalar::checked_from_integer(value.get()).ok())
                .unwrap_or(crate::Scalar::ZERO);
            (unit.id, value)
        })
        .collect()
}
