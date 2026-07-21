//! Static validation for executable battle Rule IR.

use std::collections::BTreeSet;

use crate::{
    ProgramId, SelectorId, StateSlotDefinitionId,
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventValueProperty, OnceScope, ProgramStep,
        RuleOperationTemplate, RuleValue, RuleValueKind, SlotPersistence, TriggerDef, TriggerPhase,
        ValueExpr,
    },
};

use super::{CombatCatalog, builder::CatalogBuildErrorKind};

pub(super) fn validate(catalog: &CombatCatalog) -> Result<(), super::builder::CatalogBuildError> {
    for rule_id in catalog.rules.ids() {
        let rule = catalog.rules.get(rule_id).expect("ID came from table");
        let Some(runtime) = rule.runtime() else {
            continue;
        };
        validate_runtime(runtime).map_err(invalid)?;
        for trigger in runtime.triggers() {
            if rule.programs().binary_search(&trigger.program).is_err() {
                return Err(invalid(format!(
                    "rule {} trigger {} program {} is absent from its declared program set",
                    rule_id.get(),
                    trigger.id.get(),
                    trigger.program.get()
                )));
            }
            if catalog.programs.get(trigger.program).is_none() {
                return Err(invalid(format!(
                    "rule {} trigger {} refers to missing program {}",
                    rule_id.get(),
                    trigger.id.get(),
                    trigger.program.get()
                )));
            }
            let mut visiting = BTreeSet::new();
            validate_program_for_rule(
                catalog,
                runtime,
                rule.programs(),
                trigger,
                trigger.program,
                &mut visiting,
            )
            .map_err(invalid)?;
        }
    }
    Ok(())
}

fn validate_runtime(runtime: &BattleRuleDefinition) -> Result<(), String> {
    if runtime.source().digest().iter().all(|byte| *byte == 0) {
        return Err("runtime rule source digest cannot be zero".into());
    }
    strictly_ordered(
        runtime.source().tags().iter().map(|id| id.get()),
        "source tags",
    )?;
    strictly_ordered(
        runtime.state_slots().iter().map(|slot| slot.id().get()),
        "state slot IDs",
    )?;
    strictly_ordered(
        runtime.triggers().iter().map(|trigger| trigger.id.get()),
        "trigger IDs",
    )?;
    for slot in runtime.state_slots() {
        if slot.initial().kind() != slot.kind()
            || slot
                .minimum()
                .is_some_and(|value| value.kind() != slot.kind())
            || slot
                .maximum()
                .is_some_and(|value| value.kind() != slot.kind())
        {
            return Err(format!(
                "slot {} has a value-kind mismatch",
                slot.id().get()
            ));
        }
        if let (Some(minimum), Some(maximum)) = (slot.minimum(), slot.maximum())
            && compare_static(minimum, maximum).is_some_and(|ordering| ordering.is_gt())
        {
            return Err(format!("slot {} has reversed bounds", slot.id().get()));
        }
        strictly_ordered(
            slot.reset_points().iter().map(|point| *point as u8),
            "slot reset points",
        )?;
        if slot.persistence() == SlotPersistence::ExplicitReset && slot.reset_points().is_empty() {
            return Err(format!(
                "slot {} uses ExplicitReset without a reset point",
                slot.id().get()
            ));
        }
        validate_static_value(slot.initial())?;
        if let Some(value) = slot.minimum() {
            validate_static_value(value)?;
        }
        if let Some(value) = slot.maximum() {
            validate_static_value(value)?;
        }
    }
    for trigger in runtime.triggers() {
        if trigger.event_point.kind() != trigger.event {
            return Err(format!(
                "trigger {} event point does not belong to its indexed event family",
                trigger.id.get()
            ));
        }
        if !once_scope_constructible(trigger) {
            return Err(format!(
                "trigger {} once scope cannot be constructed from its event family",
                trigger.id.get()
            ));
        }
    }
    Ok(())
}

fn validate_static_value(value: &RuleValue) -> Result<(), String> {
    match value {
        RuleValue::StableId(0) | RuleValue::OptionalStableId(Some(0)) => {
            return Err("stable rule values must be non-zero".into());
        }
        RuleValue::OrderedStableIdSet(values) => {
            if values.first() == Some(&0) {
                return Err("ordered stable-ID slot values must be non-zero".into());
            }
            strictly_ordered(values.iter().copied(), "ordered stable-ID slot value")?;
        }
        _ => {}
    }
    Ok(())
}

fn validate_program_for_rule(
    catalog: &CombatCatalog,
    runtime: &BattleRuleDefinition,
    declared_programs: &[ProgramId],
    trigger: &TriggerDef,
    program_id: ProgramId,
    visiting: &mut BTreeSet<ProgramId>,
) -> Result<(), String> {
    if declared_programs.binary_search(&program_id).is_err() {
        return Err(format!(
            "program {} is absent from the owning rule's declared program set",
            program_id.get()
        ));
    }
    if !visiting.insert(program_id) {
        return Err(format!("program {} is cyclic", program_id.get()));
    }
    let program = catalog
        .programs
        .get(program_id)
        .ok_or_else(|| format!("missing program {}", program_id.get()))?;
    for step in program.steps() {
        match step {
            ProgramStep::Operation(operation) => {
                validate_operation(catalog, runtime, operation)?;
                let replacement =
                    matches!(operation, RuleOperationTemplate::ProposeReplacement { .. });
                if (trigger.phase == TriggerPhase::Replace) != replacement {
                    return Err(format!(
                        "trigger {} replacement phase/program mismatch",
                        trigger.id.get()
                    ));
                }
            }
            ProgramStep::If {
                condition,
                then_program,
                else_program,
            } => {
                validate_condition(catalog, runtime, condition, 0)?;
                validate_program_for_rule(
                    catalog,
                    runtime,
                    declared_programs,
                    trigger,
                    *then_program,
                    visiting,
                )?;
                if let Some(program) = else_program {
                    validate_program_for_rule(
                        catalog,
                        runtime,
                        declared_programs,
                        trigger,
                        *program,
                        visiting,
                    )?;
                }
            }
            ProgramStep::ForEach {
                selector,
                body,
                maximum,
            } => {
                require_selector(catalog, *selector)?;
                if !(1..=64).contains(maximum) {
                    return Err(format!("ForEach maximum {maximum} is outside 1..=64"));
                }
                validate_program_for_rule(
                    catalog,
                    runtime,
                    declared_programs,
                    trigger,
                    *body,
                    visiting,
                )?;
            }
        }
    }
    visiting.remove(&program_id);
    Ok(())
}

fn validate_operation(
    catalog: &CombatCatalog,
    runtime: &BattleRuleDefinition,
    operation: &RuleOperationTemplate,
) -> Result<(), String> {
    match operation {
        RuleOperationTemplate::SetSlot { slot, value } => {
            let kind = slot_kind(runtime, *slot)?;
            if infer_value(catalog, runtime, value, 0)? != kind {
                return Err(format!("SetSlot {} type mismatch", slot.get()));
            }
        }
        RuleOperationTemplate::AddSlot { slot, value } => {
            let kind = slot_kind(runtime, *slot)?;
            if !matches!(kind, RuleValueKind::Integer | RuleValueKind::Scalar)
                || infer_value(catalog, runtime, value, 0)? != kind
            {
                return Err(format!(
                    "AddSlot {} requires matching numeric values",
                    slot.get()
                ));
            }
        }
        RuleOperationTemplate::Damage {
            selector, amount, ..
        }
        | RuleOperationTemplate::TrueDamage { selector, amount }
        | RuleOperationTemplate::Heal { selector, amount }
        | RuleOperationTemplate::ReduceToughness {
            selector, amount, ..
        }
        | RuleOperationTemplate::SuperBreak {
            selector,
            multiplier: amount,
        }
        | RuleOperationTemplate::CreateToughnessLayer {
            selector,
            maximum: amount,
            ..
        }
        | RuleOperationTemplate::AdvanceAction { selector, amount }
        | RuleOperationTemplate::DelayAction { selector, amount } => {
            require_selector(catalog, *selector)?;
            require_scalar(catalog, runtime, amount)?;
        }
        RuleOperationTemplate::QueueAction {
            actor_selector,
            target_selector,
            ability,
            forced_use,
            payment,
            ..
        } => {
            require_selector(catalog, *actor_selector)?;
            require_selector(catalog, *target_selector)?;
            let action = catalog
                .ability(*ability)
                .and_then(|ability| ability.action())
                .ok_or_else(|| format!("queued ability {} is not executable", ability.get()))?;
            let forced_skill = *forced_use
                && action.kind() == crate::catalog::action::AbilityKind::Skill
                && action.tags().supports_forced_skill();
            if action.kind().is_normal_turn() && !forced_skill {
                return Err(format!(
                    "queued ability {} must declare a queued action kind or an explicitly tagged forced Skill",
                    ability.get()
                ));
            }
            if !*forced_use && payment.is_some() {
                return Err("only forced queued actions can override action payment".into());
            }
        }
        RuleOperationTemplate::GrantExtraTurn { actor_selector } => {
            require_selector(catalog, *actor_selector)?;
        }
        RuleOperationTemplate::Summon {
            owner_selector,
            unit_definition,
        } => {
            require_selector(catalog, *owner_selector)?;
            if catalog.linked_unit(*unit_definition).is_none() {
                return Err(format!(
                    "summon refers to missing linked-unit definition {}",
                    unit_definition.get()
                ));
            }
        }
        RuleOperationTemplate::Despawn { selector }
        | RuleOperationTemplate::ChangePresence { selector, .. } => {
            require_selector(catalog, *selector)?;
        }
        RuleOperationTemplate::Transform {
            selector,
            replacement_definition,
        } => {
            require_selector(catalog, *selector)?;
            if catalog.unit(*replacement_definition).is_none() {
                return Err(format!(
                    "transform refers to missing unit {}",
                    replacement_definition.get()
                ));
            }
        }
        RuleOperationTemplate::ReplaceAbility {
            selector,
            old_ability,
            new_ability,
        } => {
            require_selector(catalog, *selector)?;
            if catalog.ability(*old_ability).is_none() || catalog.ability(*new_ability).is_none() {
                return Err("ability replacement refers to a missing ability".into());
            }
        }
        RuleOperationTemplate::Break { selector, .. }
        | RuleOperationTemplate::AddWeakness { selector, .. }
        | RuleOperationTemplate::RemoveWeakness { selector, .. }
        | RuleOperationTemplate::RemoveToughnessLayer { selector, .. } => {
            require_selector(catalog, *selector)?;
        }
        RuleOperationTemplate::Shield {
            selector,
            amount,
            effect,
        } => {
            require_selector(catalog, *selector)?;
            require_scalar(catalog, runtime, amount)?;
            if catalog.effect(*effect).is_none() {
                return Err(format!("shield refers to missing effect {}", effect.get()));
            }
        }
        RuleOperationTemplate::ConsumeHp {
            selector,
            amount,
            floor,
        } => {
            require_selector(catalog, *selector)?;
            require_scalar(catalog, runtime, amount)?;
            require_scalar(catalog, runtime, floor)?;
        }
        RuleOperationTemplate::ModifyResource {
            selector,
            resource,
            amount,
            scales_with_regeneration,
            ..
        } => {
            require_selector(catalog, *selector)?;
            require_scalar(catalog, runtime, amount)?;
            if *scales_with_regeneration
                && !matches!(resource, crate::rule::model::RuleResourceKind::Energy)
            {
                return Err("only Energy can scale with energy regeneration".into());
            }
        }
        RuleOperationTemplate::ApplyEffect {
            selector,
            effect,
            chance,
            base_chance,
            rng_purpose,
        } => {
            require_selector(catalog, *selector)?;
            if catalog.effect(*effect).is_none() {
                return Err(format!(
                    "operation refers to missing effect {}",
                    effect.get()
                ));
            }
            match chance {
                crate::rule::model::RuleEffectChancePolicy::Guaranteed => {
                    if base_chance.is_some() || rng_purpose.is_some() {
                        return Err("guaranteed effect cannot declare chance RNG".into());
                    }
                }
                _ => {
                    require_scalar(
                        catalog,
                        runtime,
                        base_chance
                            .as_ref()
                            .ok_or("chance operation requires base chance")?,
                    )?;
                    if rng_purpose.is_none() {
                        return Err("chance operation requires RNG purpose".into());
                    }
                }
            }
        }
        RuleOperationTemplate::RemoveEffect { selector, effect } => {
            require_selector(catalog, *selector)?;
            if catalog.effect(*effect).is_none() {
                return Err(format!(
                    "operation refers to missing effect {}",
                    effect.get()
                ));
            }
        }
        RuleOperationTemplate::DetonateDot {
            selector, fraction, ..
        } => {
            require_selector(catalog, *selector)?;
            require_scalar(catalog, runtime, fraction)?;
        }
        RuleOperationTemplate::ModifyStateSlot { slot, value, .. } => {
            let definition = runtime
                .state_slots()
                .iter()
                .find(|definition| definition.id() == *slot)
                .ok_or_else(|| {
                    format!("operation refers to undeclared state slot {}", slot.get())
                })?;
            if infer_value(catalog, runtime, value, 0)? != definition.kind() {
                return Err("state-slot update type differs from its definition".into());
            }
        }
        RuleOperationTemplate::CreateCountdown { code } => {
            if catalog.countdown(*code).is_none() {
                return Err(format!("countdown code {code} has no catalog definition"));
            }
        }
        RuleOperationTemplate::EmitRuleEvent { value, .. }
        | RuleOperationTemplate::ProposeReplacement { value, .. } => {
            if let Some(value) = value {
                let _ = infer_value(catalog, runtime, value, 0)?;
            }
        }
        RuleOperationTemplate::InvokeNative { handler, arguments } => {
            if runtime.native_handler() != Some(*handler) {
                return Err(format!(
                    "native handler {} is not declared by the owning rule",
                    handler.get()
                ));
            }
            for argument in arguments {
                let _ = infer_value(catalog, runtime, argument, 0)?;
            }
        }
    }
    Ok(())
}

fn require_scalar(
    catalog: &CombatCatalog,
    runtime: &BattleRuleDefinition,
    value: &ValueExpr,
) -> Result<(), String> {
    if infer_value(catalog, runtime, value, 0)? != RuleValueKind::Scalar {
        Err("formula operation requires a scalar expression".into())
    } else {
        Ok(())
    }
}

fn validate_condition(
    catalog: &CombatCatalog,
    runtime: &BattleRuleDefinition,
    condition: &ConditionExpr,
    depth: u16,
) -> Result<(), String> {
    check_depth(depth)?;
    match condition {
        ConditionExpr::Literal(_) | ConditionExpr::EventKind(_) => {}
        ConditionExpr::Not(value) => validate_condition(catalog, runtime, value, depth + 1)?,
        ConditionExpr::All(values) | ConditionExpr::Any(values) => {
            if values.is_empty() {
                return Err("boolean composition cannot be empty".into());
            }
            for value in values {
                validate_condition(catalog, runtime, value, depth + 1)?;
            }
        }
        ConditionExpr::Compare { lhs, rhs, .. } => {
            if infer_value(catalog, runtime, lhs, depth + 1)?
                != infer_value(catalog, runtime, rhs, depth + 1)?
            {
                return Err("comparison operands have different types".into());
            }
        }
        ConditionExpr::SourceTag(_) => {}
        ConditionExpr::SelectorCardinality { selector, .. } => {
            require_selector(catalog, *selector)?;
        }
        ConditionExpr::LifePresence { selector, .. }
        | ConditionExpr::HasWeakness { selector, .. }
        | ConditionExpr::IsBroken(selector) => require_selector(catalog, *selector)?,
        ConditionExpr::EffectExists { selector, effect } => {
            require_selector(catalog, *selector)?;
            if catalog.effect(*effect).is_none() {
                return Err("effect predicate refers to a missing effect".into());
            }
        }
    }
    Ok(())
}

fn infer_value(
    catalog: &CombatCatalog,
    runtime: &BattleRuleDefinition,
    expression: &ValueExpr,
    depth: u16,
) -> Result<RuleValueKind, String> {
    check_depth(depth)?;
    Ok(match expression {
        ValueExpr::Literal(value) => {
            validate_static_value(value)?;
            value.kind()
        }
        ValueExpr::Slot(slot) => slot_kind(runtime, *slot)?,
        ValueExpr::AbilityParameter { kind, .. } => *kind,
        ValueExpr::ReadResource { selector, .. } => {
            require_selector(catalog, *selector)?;
            RuleValueKind::Scalar
        }
        ValueExpr::ReadEventProperty(property) => match property {
            EventValueProperty::OwnerId
            | EventValueProperty::ActorId
            | EventValueProperty::ApplierId
            | EventValueProperty::SourceDefinitionId
            | EventValueProperty::PrimaryTargetId => RuleValueKind::OptionalStableId,
            EventValueProperty::DamageAmount
            | EventValueProperty::HpChangeAmount
            | EventValueProperty::ResourceDelta => RuleValueKind::Scalar,
            EventValueProperty::StackCount | EventValueProperty::HitIndex => RuleValueKind::Integer,
        },
        ValueExpr::SelectorCount(selector) => {
            require_selector(catalog, *selector)?;
            RuleValueKind::Integer
        }
        ValueExpr::SelectorSum { selector, value } => {
            require_selector(catalog, *selector)?;
            let kind = infer_value(catalog, runtime, value, depth + 1)?;
            if !matches!(kind, RuleValueKind::Integer | RuleValueKind::Scalar) {
                return Err("selector sum requires a numeric value".into());
            }
            kind
        }
        ValueExpr::EventId => RuleValueKind::StableId,
        ValueExpr::EventOwner
        | ValueExpr::EventActor
        | ValueExpr::EventApplier
        | ValueExpr::EventTarget
        | ValueExpr::CurrentTarget => RuleValueKind::OptionalStableId,
        ValueExpr::QueryStat { .. } => RuleValueKind::Scalar,
        ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => matching_numeric(catalog, runtime, lhs, rhs, depth)?,
        ValueExpr::Multiply { lhs, rhs, .. } | ValueExpr::Divide { lhs, rhs, .. } => {
            matching_numeric(catalog, runtime, lhs, rhs, depth)?
        }
        ValueExpr::Clamp {
            value,
            minimum,
            maximum,
        } => {
            let kind = infer_value(catalog, runtime, value, depth + 1)?;
            if kind != infer_value(catalog, runtime, minimum, depth + 1)?
                || kind != infer_value(catalog, runtime, maximum, depth + 1)?
            {
                return Err("Clamp operands have different types".into());
            }
            kind
        }
        ValueExpr::Negate(value) => {
            let kind = infer_value(catalog, runtime, value, depth + 1)?;
            if !matches!(kind, RuleValueKind::Integer | RuleValueKind::Scalar) {
                return Err("Negate requires a numeric value".into());
            }
            kind
        }
        ValueExpr::Choose {
            condition,
            when_true,
            when_false,
        } => {
            validate_condition(catalog, runtime, condition, depth + 1)?;
            let kind = infer_value(catalog, runtime, when_true, depth + 1)?;
            if kind != infer_value(catalog, runtime, when_false, depth + 1)? {
                return Err("Choose branches have different types".into());
            }
            kind
        }
        ValueExpr::Convert { value, target, .. } => {
            let source = infer_value(catalog, runtime, value, depth + 1)?;
            if source != *target
                && !matches!(
                    (source, *target),
                    (RuleValueKind::Integer, RuleValueKind::Scalar)
                        | (RuleValueKind::Scalar, RuleValueKind::Integer)
                )
            {
                return Err("unsupported explicit conversion".into());
            }
            *target
        }
    })
}

fn matching_numeric(
    catalog: &CombatCatalog,
    runtime: &BattleRuleDefinition,
    lhs: &ValueExpr,
    rhs: &ValueExpr,
    depth: u16,
) -> Result<RuleValueKind, String> {
    let lhs = infer_value(catalog, runtime, lhs, depth + 1)?;
    let rhs = infer_value(catalog, runtime, rhs, depth + 1)?;
    if lhs != rhs || !matches!(lhs, RuleValueKind::Integer | RuleValueKind::Scalar) {
        return Err("arithmetic operands require the same numeric type".into());
    }
    Ok(lhs)
}

fn slot_kind(
    runtime: &BattleRuleDefinition,
    id: StateSlotDefinitionId,
) -> Result<RuleValueKind, String> {
    runtime
        .state_slots()
        .binary_search_by_key(&id, |slot| slot.id())
        .ok()
        .map(|index| runtime.state_slots()[index].kind())
        .ok_or_else(|| format!("missing state slot {}", id.get()))
}

fn require_selector(catalog: &CombatCatalog, id: SelectorId) -> Result<(), String> {
    if catalog.selectors.get(id).is_none() {
        return Err(format!("missing selector {}", id.get()));
    }
    Ok(())
}

fn once_scope_constructible(trigger: &TriggerDef) -> bool {
    match trigger.once_scope {
        OnceScope::Event | OnceScope::Wave | OnceScope::Battle => true,
        OnceScope::Hit | OnceScope::TargetWithinHit => matches!(
            trigger.event,
            crate::rule::model::RuleEventKind::Hit
                | crate::rule::model::RuleEventKind::Damage
                | crate::rule::model::RuleEventKind::Heal
        ),
        OnceScope::Ability | OnceScope::Action => matches!(
            trigger.event,
            crate::rule::model::RuleEventKind::Action
                | crate::rule::model::RuleEventKind::Phase
                | crate::rule::model::RuleEventKind::Hit
                | crate::rule::model::RuleEventKind::Damage
                | crate::rule::model::RuleEventKind::Heal
        ),
        OnceScope::Turn => matches!(
            trigger.event,
            crate::rule::model::RuleEventKind::Turn
                | crate::rule::model::RuleEventKind::Action
                | crate::rule::model::RuleEventKind::Phase
                | crate::rule::model::RuleEventKind::Hit
                | crate::rule::model::RuleEventKind::Damage
                | crate::rule::model::RuleEventKind::Heal
        ),
    }
}

fn compare_static(lhs: &RuleValue, rhs: &RuleValue) -> Option<core::cmp::Ordering> {
    match (lhs, rhs) {
        (RuleValue::Integer(lhs), RuleValue::Integer(rhs)) => Some(lhs.cmp(rhs)),
        (RuleValue::Scalar(lhs), RuleValue::Scalar(rhs)) => Some(lhs.cmp(rhs)),
        (RuleValue::Boolean(lhs), RuleValue::Boolean(rhs)) => Some(lhs.cmp(rhs)),
        (RuleValue::StableId(lhs), RuleValue::StableId(rhs)) => Some(lhs.cmp(rhs)),
        (RuleValue::OptionalStableId(lhs), RuleValue::OptionalStableId(rhs)) => Some(lhs.cmp(rhs)),
        (RuleValue::OrderedStableIdSet(lhs), RuleValue::OrderedStableIdSet(rhs)) => {
            Some(lhs.cmp(rhs))
        }
        _ => None,
    }
}

fn strictly_ordered<I: Ord>(values: impl Iterator<Item = I>, label: &str) -> Result<(), String> {
    let mut previous = None;
    for value in values {
        if previous.as_ref().is_some_and(|previous| previous >= &value) {
            return Err(format!("{label} must be strictly ordered and unique"));
        }
        previous = Some(value);
    }
    Ok(())
}

fn check_depth(depth: u16) -> Result<(), String> {
    if depth > 64 {
        return Err("expression depth exceeds 64".into());
    }
    Ok(())
}

fn invalid(message: String) -> super::builder::CatalogBuildError {
    super::builder::catalog_error(CatalogBuildErrorKind::InvalidDefinition, message)
}
