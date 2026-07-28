//! Generated Sora operation/program rows to typed Rule IR proposals.

use std::collections::BTreeSet;

use starclock_combat::{
    AbilityId, EffectDefinitionId, NativeHandlerId, PresenceState, ProgramId, Rounding, SelectorId,
    UnitDefinitionId,
    catalog::action::ReactionBoundary,
    formula::model::{CombatElement, DamageClass},
    rule::{
        evaluate::ProgramLookup,
        model::{
            ProgramStep, ReactionPriority, ResourceUpdateKind, RuleActionOwner,
            RuleActionPaymentPolicy, RuleEffectChancePolicy, RuleOperationTemplate,
            RuleResourceKind, RuleValue, StateSlotUpdateKind, ValueExpr,
        },
    },
};

use crate::{
    catalog::{CatalogLoadError, SimulationCatalog, domain_fail},
    generated::{
        self, SoraConfig, combat_element, damage_class, effect_chance_policy, operation_payload,
        presence_state, program_step_node, queued_action_owner_policy,
        queued_action_payment_policy, reaction_boundary, resource_kind, resource_update_kind,
        rounding_policy, state_slot_update_kind,
    },
};

#[derive(Debug)]
pub(super) struct RuleProgramDefinition {
    pub(super) id: ProgramId,
    pub(super) steps: Box<[ProgramStep]>,
    pub(super) selectors: Box<[SelectorId]>,
    pub(super) effects: Box<[EffectDefinitionId]>,
}

impl SimulationCatalog {
    /// Returns one validated Rule IR program lowered from generated Sora rows.
    #[must_use]
    pub fn program_steps(&self, id: ProgramId) -> Option<&[ProgramStep]> {
        self.combat
            .programs
            .binary_search_by_key(&id, |program| program.id)
            .ok()
            .map(|index| self.combat.programs[index].steps.as_ref())
    }
}

impl ProgramLookup for SimulationCatalog {
    fn program_steps(&self, id: ProgramId) -> Option<&[ProgramStep]> {
        self.program_steps(id)
    }
}

pub(super) fn convert(
    config: &SoraConfig,
    native_handlers: &BTreeSet<NativeHandlerId>,
) -> Result<Vec<RuleProgramDefinition>, CatalogLoadError> {
    let mut programs = config
        .program()
        .ordered_rows()
        .map(|program| {
            let mut rows = config
                .program_step()
                .iter()
                .filter(|step| step.program_id == program.id)
                .collect::<Vec<_>>();
            rows.sort_unstable_by_key(|step| step.sequence);
            for (offset, row) in rows.iter().enumerate() {
                if row.sequence != i32::try_from(offset + 1).expect("program bound fits i32") {
                    return Err(domain_fail(format!(
                        "program {} has noncontiguous operation order",
                        program.id
                    )));
                }
            }
            let steps = rows
                .into_iter()
                .map(|step| match &step.step {
                    program_step_node::ProgramStepNode::Operation { operation_id } => config
                        .operation()
                        .get(operation_id)
                        .ok_or_else(|| domain_fail(format!("missing operation {operation_id}")))
                        .and_then(|operation| lower_operation(config, operation, native_handlers))
                        .map(ProgramStep::Operation),
                    program_step_node::ProgramStepNode::If {
                        condition_id,
                        then_program_id,
                        else_program_id,
                    } => Ok(ProgramStep::If {
                        condition: crate::rule_lower::lower_condition(
                            config,
                            *condition_id,
                            &mut BTreeSet::new(),
                        )?,
                        then_program: program_id(*then_program_id)?,
                        else_program: else_program_id.map(program_id).transpose()?,
                    }),
                    program_step_node::ProgramStepNode::ForEach {
                        selector_id: raw_selector,
                        body_program_id,
                        maximum_iterations,
                    } => Ok(ProgramStep::ForEach {
                        selector: selector_id(*raw_selector)?,
                        body: program_id(*body_program_id)?,
                        maximum: u16::try_from(*maximum_iterations)
                            .map_err(|_| domain_fail("ForEach maximum does not fit u16"))?,
                    }),
                })
                .collect::<Result<Vec<_>, _>>()?;
            let (selectors, effects) = program_references(&steps);
            Ok(RuleProgramDefinition {
                id: ProgramId::new(positive(program.id)?).expect("positive program ID"),
                steps: steps.into_boxed_slice(),
                selectors,
                effects,
            })
        })
        .collect::<Result<Vec<_>, CatalogLoadError>>()?;
    programs.sort_unstable_by_key(|program| program.id);
    Ok(programs)
}

fn program_references(steps: &[ProgramStep]) -> (Box<[SelectorId]>, Box<[EffectDefinitionId]>) {
    use starclock_combat::rule::model::RuleOperationTemplate as O;
    let mut selectors = BTreeSet::new();
    let mut effects = BTreeSet::new();
    for step in steps {
        if let ProgramStep::ForEach { selector, .. } = step {
            selectors.insert(*selector);
            continue;
        }
        let ProgramStep::Operation(operation) = step else {
            continue;
        };
        match operation {
            O::Damage { selector, .. }
            | O::UnboostedDamage { selector, .. }
            | O::UnboostedDamageFromEventElement { selector, .. }
            | O::DamageFromEventElement { selector, .. }
            | O::DamageFromActorBasicElement { selector, .. }
            | O::UltimateDamageFromActorBasicElement { selector, .. }
            | O::RandomRepeatedDamage { selector, .. }
            | O::TrueDamage { selector, .. }
            | O::Heal { selector, .. }
            | O::ConsumeHp { selector, .. }
            | O::ReduceToughness { selector, .. }
            | O::Break { selector, .. }
            | O::SuperBreak { selector, .. }
            | O::AddWeakness { selector, .. }
            | O::AddWeaknessFromAlliedElements { selector, .. }
            | O::RemoveWeakness { selector, .. }
            | O::CreateToughnessLayer { selector, .. }
            | O::RemoveToughnessLayer { selector, .. }
            | O::ModifyResource { selector, .. }
            | O::RemoveEffect { selector, .. }
            | O::Cleanse { selector, .. }
            | O::DetonateDot { selector, .. }
            | O::AdvanceAction { selector, .. }
            | O::DelayAction { selector, .. }
            | O::Despawn { selector }
            | O::Transform { selector, .. }
            | O::ReplaceAbility { selector, .. }
            | O::ChangePresence { selector, .. } => {
                selectors.insert(*selector);
            }
            O::Shield {
                selector, effect, ..
            }
            | O::RemoveShield { selector, effect }
            | O::ApplyEffect {
                selector, effect, ..
            }
            | O::RandomGroupedEffect {
                selector, effect, ..
            }
            | O::AdjustEffectStacks {
                selector, effect, ..
            } => {
                selectors.insert(*selector);
                effects.insert(*effect);
            }
            O::ApplyRandomEffect {
                selector,
                effects: candidates,
                ..
            } => {
                selectors.insert(*selector);
                effects.extend(candidates.iter().copied());
            }
            O::QueueAction {
                actor_selector,
                target_selector,
                ..
            } => {
                selectors.insert(*actor_selector);
                selectors.insert(*target_selector);
            }
            O::GrantExtraTurn { actor_selector } => {
                selectors.insert(*actor_selector);
            }
            O::Summon { owner_selector, .. } => {
                selectors.insert(*owner_selector);
            }
            O::SetSlot { .. }
            | O::AddSlot { .. }
            | O::ModifyStateSlot { .. }
            | O::CreateCountdown { .. }
            | O::EmitRuleEvent { .. }
            | O::ProposeReplacement { .. }
            | O::InvokeNative { .. } => {}
        }
    }
    (
        selectors.into_iter().collect::<Vec<_>>().into_boxed_slice(),
        effects.into_iter().collect::<Vec<_>>().into_boxed_slice(),
    )
}

fn lower_operation(
    config: &SoraConfig,
    row: &generated::operation::Operation,
    native_handlers: &BTreeSet<NativeHandlerId>,
) -> Result<RuleOperationTemplate, CatalogLoadError> {
    if row.condition_id.is_some() {
        return Err(domain_fail(format!(
            "probe operation {} has an unsupported condition",
            row.id
        )));
    }
    let selector = || {
        row.target_selector_id
            .ok_or_else(|| domain_fail(format!("operation {} lacks a target selector", row.id)))
            .and_then(selector_id)
    };
    let expression = |id| crate::modifier_lower::expression(config, id, &mut BTreeSet::new());
    use operation_payload::OperationPayload as Payload;
    Ok(match &row.payload {
        Payload::Damage {
            amount_expression_id,
            damage_class,
            element,
            can_crit,
        } => RuleOperationTemplate::Damage {
            selector: selector()?,
            amount: expression(*amount_expression_id)?,
            class: lower_damage_class(*damage_class)?,
            element: lower_element(*element),
            can_crit: *can_crit,
            can_defeat: true,
        },
        Payload::TrueDamage {
            amount_expression_id,
        } => RuleOperationTemplate::TrueDamage {
            selector: selector()?,
            amount: expression(*amount_expression_id)?,
        },
        Payload::Heal {
            amount_expression_id,
        } => RuleOperationTemplate::Heal {
            selector: selector()?,
            amount: expression(*amount_expression_id)?,
            apply_formula_modifiers: true,
        },
        Payload::Shield {
            amount_expression_id,
            effect_id,
        } => RuleOperationTemplate::Shield {
            selector: selector()?,
            amount: expression(*amount_expression_id)?,
            effect: effect(*effect_id)?,
        },
        Payload::ConsumeHp {
            amount_expression_id,
            floor_expression_id,
        } => RuleOperationTemplate::ConsumeHp {
            selector: selector()?,
            amount: expression(*amount_expression_id)?,
            floor: expression(*floor_expression_id)?,
        },
        Payload::ReduceToughness {
            amount_expression_id,
            element,
        } => RuleOperationTemplate::ReduceToughness {
            selector: selector()?,
            amount: expression(*amount_expression_id)?,
            element: lower_element(*element),
        },
        Payload::Break { element } => RuleOperationTemplate::Break {
            selector: selector()?,
            element: lower_element(*element),
        },
        Payload::SuperBreak {
            multiplier_expression_id,
        } => RuleOperationTemplate::SuperBreak {
            selector: selector()?,
            multiplier: expression(*multiplier_expression_id)?,
        },
        Payload::AddWeakness { element, .. } => RuleOperationTemplate::AddWeakness {
            selector: selector()?,
            element: lower_element(*element),
            duration_turns: None,
        },
        Payload::RemoveWeakness { element } => RuleOperationTemplate::RemoveWeakness {
            selector: selector()?,
            element: lower_element(*element),
        },
        Payload::CreateToughnessLayer {
            layer_key,
            maximum_expression_id,
        } => RuleOperationTemplate::CreateToughnessLayer {
            selector: selector()?,
            layer_key: layer_key.clone().into_boxed_str(),
            maximum: expression(*maximum_expression_id)?,
        },
        Payload::RemoveToughnessLayer { layer_key } => {
            RuleOperationTemplate::RemoveToughnessLayer {
                selector: selector()?,
                layer_key: layer_key.clone().into_boxed_str(),
            }
        }
        Payload::ModifyResource {
            resource_kind,
            character_resource_key,
            update_kind,
            amount_expression_id,
            scales_with_energy_regeneration,
            rounding,
        } => {
            let resource = match resource_kind {
                resource_kind::ResourceKind::Energy if character_resource_key.is_none() => {
                    RuleResourceKind::Energy
                }
                resource_kind::ResourceKind::SkillPoints if character_resource_key.is_none() => {
                    RuleResourceKind::SkillPoints
                }
                resource_kind::ResourceKind::CharacterResource => RuleResourceKind::Character(
                    character_resource_key
                        .as_deref()
                        .filter(|key| !key.trim().is_empty())
                        .ok_or_else(|| domain_fail("character resource lacks its authored key"))?
                        .into(),
                ),
                resource_kind::ResourceKind::TeamResource => RuleResourceKind::Team(
                    character_resource_key
                        .as_deref()
                        .filter(|key| !key.trim().is_empty())
                        .ok_or_else(|| domain_fail("team resource lacks its authored key"))?
                        .into(),
                ),
                resource_kind::ResourceKind::Hp => {
                    return Err(domain_fail("HP changes must use ConsumeHp, Heal or Damage"));
                }
                _ => return Err(domain_fail("resource kind/key combination is invalid")),
            };
            if *scales_with_energy_regeneration && resource != RuleResourceKind::Energy {
                return Err(domain_fail(
                    "only Energy can scale with energy regeneration",
                ));
            }
            RuleOperationTemplate::ModifyResource {
                selector: selector()?,
                resource,
                update: lower_update(*update_kind),
                amount: expression(*amount_expression_id)?,
                scales_with_regeneration: *scales_with_energy_regeneration,
                rounding: lower_rounding(*rounding),
            }
        }
        Payload::ApplyEffect {
            effect_id,
            stacks_expression_id,
            chance_policy,
            base_chance_expression_id,
            rng_purpose_key,
        } => RuleOperationTemplate::ApplyEffect {
            selector: selector()?,
            effect: effect(*effect_id)?,
            stacks: stacks_expression_id
                .map(expression)
                .transpose()?
                .unwrap_or(ValueExpr::Literal(RuleValue::Integer(1))),
            chance: lower_effect_chance(*chance_policy),
            base_chance: base_chance_expression_id.map(expression).transpose()?,
            rng_purpose: lower_rng_purpose(rng_purpose_key.as_deref())?,
        },
        Payload::ApplyRandomEffect {
            effect_ids,
            stacks_expression_id,
            choice_rng_purpose_key,
            chance_policy,
            base_chance_expression_id,
            chance_rng_purpose_key,
        } => RuleOperationTemplate::ApplyRandomEffect {
            selector: selector()?,
            effects: effect_ids
                .iter()
                .copied()
                .map(effect)
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(),
            stacks: stacks_expression_id
                .map(expression)
                .transpose()?
                .unwrap_or(ValueExpr::Literal(RuleValue::Integer(1))),
            choice_rng_purpose: lower_rng_purpose(Some(choice_rng_purpose_key))?
                .expect("authored choice RNG purpose is required"),
            chance: lower_effect_chance(*chance_policy),
            base_chance: base_chance_expression_id.map(expression).transpose()?,
            chance_rng_purpose: lower_rng_purpose(chance_rng_purpose_key.as_deref())?,
        },
        Payload::RemoveEffect { effect_id } => RuleOperationTemplate::RemoveEffect {
            selector: selector()?,
            effect: effect(*effect_id)?,
        },
        Payload::DetonateDot {
            fraction_expression_id,
            required_effect_tag,
        } => {
            if required_effect_tag.is_some() {
                return Err(domain_fail(
                    "string effect tags require a compiled tag registry",
                ));
            }
            RuleOperationTemplate::DetonateDot {
                selector: selector()?,
                fraction: expression(*fraction_expression_id)?,
                required_tag: None,
                selection: starclock_combat::rule::model::RuleDotSelection::All,
            }
        }
        Payload::ModifyStateSlot {
            state_slot_id,
            update_kind,
            value_expression_id,
        } => RuleOperationTemplate::ModifyStateSlot {
            slot: starclock_combat::StateSlotDefinitionId::new(positive(*state_slot_id)?)
                .expect("positive state-slot ID"),
            update: lower_slot_update(*update_kind),
            value: expression(*value_expression_id)?,
        },
        Payload::AdvanceAction {
            amount_expression_id,
        } => RuleOperationTemplate::AdvanceAction {
            selector: selector()?,
            amount: expression(*amount_expression_id)?,
        },
        Payload::DelayAction {
            amount_expression_id,
        } => RuleOperationTemplate::DelayAction {
            selector: selector()?,
            amount: expression(*amount_expression_id)?,
        },
        Payload::QueueAction {
            ability_id,
            actor_selector_id,
            priority,
            forced_use,
            reaction_boundary,
            owner_policy,
            payment_policy,
            payment_resource_key,
        } => RuleOperationTemplate::QueueAction {
            actor_selector: selector_id(*actor_selector_id)?,
            target_selector: selector()?,
            ability: AbilityId::new(positive(*ability_id)?).expect("positive queued ability ID"),
            priority: ReactionPriority::new(i16::try_from(*priority).map_err(|_| {
                domain_fail(format!("operation {} priority does not fit i16", row.id))
            })?),
            forced_use: *forced_use,
            boundary: lower_reaction_boundary(*reaction_boundary)?,
            owner: lower_queue_owner(*owner_policy),
            payment: lower_queue_payment(*payment_policy, payment_resource_key.as_deref())?,
        },
        Payload::GrantExtraTurn { actor_selector_id } => RuleOperationTemplate::GrantExtraTurn {
            actor_selector: selector_id(*actor_selector_id)?,
        },
        Payload::Summon {
            unit_definition_identity_id,
            owner_selector_id,
        } => RuleOperationTemplate::Summon {
            owner_selector: selector_id(*owner_selector_id)?,
            unit_definition: UnitDefinitionId::new(positive(*unit_definition_identity_id)?)
                .expect("positive summoned unit-definition ID"),
        },
        Payload::Despawn {} => RuleOperationTemplate::Despawn {
            selector: selector()?,
        },
        Payload::Transform {
            replacement_definition_identity_id,
        } => RuleOperationTemplate::Transform {
            selector: selector()?,
            replacement_definition: UnitDefinitionId::new(positive(
                *replacement_definition_identity_id,
            )?)
            .expect("positive replacement unit-definition ID"),
        },
        Payload::ReplaceAbility {
            old_ability_id,
            new_ability_id,
        } => RuleOperationTemplate::ReplaceAbility {
            selector: selector()?,
            old_ability: AbilityId::new(positive(*old_ability_id)?)
                .expect("positive old ability ID"),
            new_ability: AbilityId::new(positive(*new_ability_id)?)
                .expect("positive new ability ID"),
        },
        Payload::ChangePresence { presence } => RuleOperationTemplate::ChangePresence {
            selector: selector()?,
            presence: lower_presence(*presence),
        },
        Payload::EmitRuleEvent { .. } if row.target_selector_id.is_none() => {
            RuleOperationTemplate::CreateCountdown {
                code: positive(row.id)?,
            }
        }
        Payload::InvokeNativeHandler { native_handler_id } => {
            let handler = crate::native_handler_lower::handler_id(*native_handler_id)?;
            if !native_handlers.contains(&handler) {
                return Err(domain_fail(format!(
                    "operation {} invokes an unregistered native handler {}",
                    row.id, native_handler_id
                )));
            }
            RuleOperationTemplate::InvokeNative {
                handler,
                arguments: Box::new([]),
            }
        }
        _ => {
            return Err(domain_fail(format!(
                "operation {} uses a payload outside the current lowering boundary",
                row.id
            )));
        }
    })
}

fn lower_presence(value: presence_state::PresenceState) -> PresenceState {
    match value {
        presence_state::PresenceState::Present => PresenceState::Present,
        presence_state::PresenceState::Reserved => PresenceState::Reserved,
        presence_state::PresenceState::Departed => PresenceState::Departed,
        presence_state::PresenceState::Untargetable => PresenceState::Untargetable,
        presence_state::PresenceState::Linked => PresenceState::Linked,
        presence_state::PresenceState::Transformed => PresenceState::Transformed,
    }
}

fn lower_effect_chance(value: effect_chance_policy::EffectChancePolicy) -> RuleEffectChancePolicy {
    match value {
        effect_chance_policy::EffectChancePolicy::Guaranteed => RuleEffectChancePolicy::Guaranteed,
        effect_chance_policy::EffectChancePolicy::Fixed => RuleEffectChancePolicy::Fixed,
        effect_chance_policy::EffectChancePolicy::Resistible => RuleEffectChancePolicy::Resistible,
    }
}

fn lower_rng_purpose(
    value: Option<&str>,
) -> Result<Option<starclock_combat::rng::types::DrawPurpose>, CatalogLoadError> {
    match value {
        None => Ok(None),
        Some("effect-application") => Ok(Some(
            starclock_combat::rng::types::DrawPurpose::EFFECT_CHANCE,
        )),
        Some("behavior-choice") => Ok(Some(
            starclock_combat::rng::types::DrawPurpose::BEHAVIOR_CHOICE,
        )),
        Some("weakness-element") => Ok(Some(
            starclock_combat::rng::types::DrawPurpose::WEAKNESS_ELEMENT,
        )),
        Some(_) => Err(domain_fail("unknown RNG purpose key")),
    }
}

fn lower_slot_update(value: state_slot_update_kind::StateSlotUpdateKind) -> StateSlotUpdateKind {
    match value {
        state_slot_update_kind::StateSlotUpdateKind::Set => StateSlotUpdateKind::Set,
        state_slot_update_kind::StateSlotUpdateKind::Add => StateSlotUpdateKind::Add,
        state_slot_update_kind::StateSlotUpdateKind::Subtract => StateSlotUpdateKind::Subtract,
        state_slot_update_kind::StateSlotUpdateKind::Minimum => StateSlotUpdateKind::Minimum,
        state_slot_update_kind::StateSlotUpdateKind::Maximum => StateSlotUpdateKind::Maximum,
    }
}

fn lower_damage_class(value: damage_class::DamageClass) -> Result<DamageClass, CatalogLoadError> {
    match value {
        damage_class::DamageClass::Ordinary => Ok(DamageClass::Direct),
        damage_class::DamageClass::Dot => Ok(DamageClass::Dot),
        damage_class::DamageClass::Additional => Ok(DamageClass::Additional),
        damage_class::DamageClass::Elation => Ok(DamageClass::Elation),
        _ => Err(domain_fail(
            "damage class belongs to another formula family",
        )),
    }
}

fn lower_element(value: combat_element::CombatElement) -> CombatElement {
    match value {
        combat_element::CombatElement::Physical => CombatElement::Physical,
        combat_element::CombatElement::Fire => CombatElement::Fire,
        combat_element::CombatElement::Ice => CombatElement::Ice,
        combat_element::CombatElement::Lightning => CombatElement::Lightning,
        combat_element::CombatElement::Wind => CombatElement::Wind,
        combat_element::CombatElement::Quantum => CombatElement::Quantum,
        combat_element::CombatElement::Imaginary => CombatElement::Imaginary,
    }
}

fn lower_update(value: resource_update_kind::ResourceUpdateKind) -> ResourceUpdateKind {
    match value {
        resource_update_kind::ResourceUpdateKind::Spend => ResourceUpdateKind::Spend,
        resource_update_kind::ResourceUpdateKind::Reserve => ResourceUpdateKind::Reserve,
        resource_update_kind::ResourceUpdateKind::Gain => ResourceUpdateKind::Gain,
        resource_update_kind::ResourceUpdateKind::Set => ResourceUpdateKind::Set,
    }
}

fn lower_queue_owner(
    value: Option<queued_action_owner_policy::QueuedActionOwnerPolicy>,
) -> RuleActionOwner {
    match value {
        None | Some(queued_action_owner_policy::QueuedActionOwnerPolicy::Actor) => {
            RuleActionOwner::Actor
        }
        Some(queued_action_owner_policy::QueuedActionOwnerPolicy::CauseOwner) => {
            RuleActionOwner::CauseOwner
        }
        Some(queued_action_owner_policy::QueuedActionOwnerPolicy::CauseApplier) => {
            RuleActionOwner::CauseApplier
        }
    }
}

fn lower_queue_payment(
    value: Option<queued_action_payment_policy::QueuedActionPaymentPolicy>,
    resource_key: Option<&str>,
) -> Result<Option<RuleActionPaymentPolicy>, CatalogLoadError> {
    match value {
        None if resource_key.is_none() => Ok(None),
        Some(queued_action_payment_policy::QueuedActionPaymentPolicy::TeamSkillPoints)
            if resource_key.is_none() =>
        {
            Ok(Some(RuleActionPaymentPolicy::TeamSkillPoints))
        }
        Some(queued_action_payment_policy::QueuedActionPaymentPolicy::Suppressed)
            if resource_key.is_none() =>
        {
            Ok(Some(RuleActionPaymentPolicy::Suppressed))
        }
        Some(queued_action_payment_policy::QueuedActionPaymentPolicy::TeamResource) => resource_key
            .filter(|key| !key.trim().is_empty())
            .map(|key| Some(RuleActionPaymentPolicy::TeamResource(key.into())))
            .ok_or_else(|| domain_fail("queued team-resource payment lacks its authored key")),
        _ => Err(domain_fail(
            "queued payment policy/key combination is invalid",
        )),
    }
}

fn lower_reaction_boundary(
    value: Option<reaction_boundary::ReactionBoundary>,
) -> Result<ReactionBoundary, CatalogLoadError> {
    match value {
        Some(reaction_boundary::ReactionBoundary::AfterHit) => Ok(ReactionBoundary::AfterHit),
        Some(reaction_boundary::ReactionBoundary::AfterPhase) => Ok(ReactionBoundary::AfterPhase),
        Some(reaction_boundary::ReactionBoundary::AfterAction) => Ok(ReactionBoundary::AfterAction),
        Some(reaction_boundary::ReactionBoundary::BeforeTimeline) => {
            Ok(ReactionBoundary::BeforeTimeline)
        }
        None => Err(domain_fail(
            "queued action lacks its explicit reaction boundary",
        )),
    }
}

fn lower_rounding(value: rounding_policy::RoundingPolicy) -> Rounding {
    match value {
        rounding_policy::RoundingPolicy::Floor => Rounding::Floor,
        rounding_policy::RoundingPolicy::Ceil => Rounding::Ceil,
        rounding_policy::RoundingPolicy::TowardZero => Rounding::TowardZero,
        rounding_policy::RoundingPolicy::AwayFromZero => Rounding::AwayFromZero,
        rounding_policy::RoundingPolicy::NearestTiesAway => Rounding::NearestTiesAway,
        rounding_policy::RoundingPolicy::NearestTiesEven => Rounding::NearestTiesEven,
    }
}

fn positive(value: i32) -> Result<u32, CatalogLoadError> {
    u32::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| domain_fail("operation-domain ID must be positive"))
}

fn selector_id(value: i32) -> Result<SelectorId, CatalogLoadError> {
    Ok(SelectorId::new(positive(value)?).expect("positive selector ID"))
}

fn program_id(value: i32) -> Result<ProgramId, CatalogLoadError> {
    Ok(ProgramId::new(positive(value)?).expect("positive program ID"))
}

fn effect(value: i32) -> Result<EffectDefinitionId, CatalogLoadError> {
    Ok(EffectDefinitionId::new(positive(value)?).expect("positive effect ID"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::runtime::SoraBundle;

    const PRODUCTION: &[u8] = include_bytes!("../../../config/generated/config.sora");

    #[test]
    fn goal07_action_break_probes_survive_excel_sora_and_typed_lowering() {
        let bundle = SoraBundle::parse(PRODUCTION).expect("production Sora bundle");
        let config = SoraConfig::from_source(&bundle).expect("generated production config");
        let handlers = BTreeSet::new();
        let lowered = [24_701, 24_702, 24_703].map(|id| {
            lower_operation(
                &config,
                config.operation().get(&id).expect("Goal 07 probe row"),
                &handlers,
            )
            .expect("typed operation lowering")
        });
        assert!(matches!(
            lowered[0],
            RuleOperationTemplate::Break {
                element: CombatElement::Fire,
                ..
            }
        ));
        assert!(matches!(
            lowered[1],
            RuleOperationTemplate::DelayAction { .. }
        ));
        assert!(matches!(
            lowered[2],
            RuleOperationTemplate::GrantExtraTurn { .. }
        ));
    }
}
