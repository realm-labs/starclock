//! Payload-direct canonical encoding for authoritative battle events.
//!
//! Verification compares these bytes directly. Decoding them back into combat
//! domain values is intentionally unnecessary: replay execution produces the
//! authoritative value and this codec proves every retained field.

use starclock_combat::{
    ActionEventData, BattleEvent, BattleEventData, BattleEventKind, BreakDamageEventData,
    BreakDamageKind, Cause, CauseActor, DamageEventData, DamageKind, DecisionEventData,
    DecisionKind, DecisionOwner, EffectEventData, EnemyPhaseEventData, FaultEventData,
    HealEventData, HitEventData, HpConsumptionEventData, LinkedEntity, PhaseEventData,
    ResourceEventData, RuleSignalEventData, RuleStateEventData, ShieldEventData, SkillPointPayer,
    TeamSide, ToughnessEventData, TurnEventData, UnitEventData, WaveEventData,
    catalog::{action::AbilityTags, encounter::EnemyPhaseTransitionModel},
    formula::model::{CombatElement, DamageClass},
    rule::model::RuleValue,
};

use crate::codec::{CodecError, Encoder};

/// Version of one complete event byte payload.
pub const BATTLE_EVENT_PAYLOAD_VERSION: u16 = 1;

/// Canonically encodes one event identity, cause chain and complete typed data.
pub fn encode_battle_event_payload(
    event: &BattleEvent,
) -> Result<Vec<u8>, BattleEventPayloadError> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(BATTLE_EVENT_PAYLOAD_VERSION);
    encoder.u64(event.id().get());
    encode_cause(&mut encoder, event.cause());
    encode_kind(&mut encoder, event.kind())?;
    Ok(encoder.into_inner())
}

fn encode_cause(encoder: &mut Encoder<Vec<u8>>, cause: Cause) {
    optional_u64(encoder, cause.parent_event().map(|value| value.get()));
    encoder.u64(cause.root_command().get());
    optional_u64(encoder, cause.action().map(|value| value.get()));
    optional_u64(encoder, cause.phase().map(|value| value.get()));
    optional_u64(encoder, cause.hit().map(|value| value.get()));
    optional_u64(encoder, cause.owner().map(|value| value.get()));
    match cause.actor() {
        None => encoder.u8(0),
        Some(CauseActor::Unit(value)) => {
            encoder.u8(1);
            encoder.u64(value.get());
        }
        Some(CauseActor::TimelineActor(value)) => {
            encoder.u8(2);
            encoder.u64(value.get());
        }
    }
    optional_u64(encoder, cause.applier().map(|value| value.get()));
    optional_u32(encoder, cause.source_definition().map(|value| value.get()));
    optional_u64(encoder, cause.primary_target().map(|value| value.get()));
    optional_u32(encoder, cause.activity_source().map(|value| value.get()));
}

fn encode_kind(
    encoder: &mut Encoder<Vec<u8>>,
    kind: &BattleEventKind,
) -> Result<(), BattleEventPayloadError> {
    match kind {
        BattleEventKind::Battle(value) => {
            encoder.u8(0);
            encode_battle(encoder, *value);
        }
        BattleEventKind::Decision(value) => {
            encoder.u8(1);
            encode_decision(encoder, *value);
        }
        BattleEventKind::Turn(value) => {
            encoder.u8(2);
            encode_turn(encoder, *value);
        }
        BattleEventKind::Action(value) => {
            encoder.u8(3);
            encode_action(encoder, *value);
        }
        BattleEventKind::Phase(value) => {
            encoder.u8(4);
            encode_phase(encoder, *value);
        }
        BattleEventKind::Hit(value) => {
            encoder.u8(5);
            encode_hit(encoder, value)?;
        }
        BattleEventKind::Damage(value) => {
            encoder.u8(6);
            encode_damage(encoder, *value);
        }
        BattleEventKind::Heal(value) => {
            encoder.u8(7);
            encode_heal(encoder, *value);
        }
        BattleEventKind::HpConsumption(value) => {
            encoder.u8(8);
            encode_hp_consumption(encoder, *value);
        }
        BattleEventKind::Shield(value) => {
            encoder.u8(9);
            encode_shield(encoder, *value);
        }
        BattleEventKind::Toughness(value) => {
            encoder.u8(10);
            encode_toughness(encoder, *value);
        }
        BattleEventKind::BreakDamage(value) => {
            encoder.u8(11);
            encode_break_damage(encoder, *value);
        }
        BattleEventKind::Unit(value) => {
            encoder.u8(12);
            encode_unit(encoder, *value);
        }
        BattleEventKind::Wave(value) => {
            encoder.u8(13);
            encode_wave(encoder, *value);
        }
        BattleEventKind::EnemyPhase(value) => {
            encoder.u8(14);
            encode_enemy_phase(encoder, *value);
        }
        BattleEventKind::Resource(value) => {
            encoder.u8(15);
            encode_resource(encoder, value)?;
        }
        BattleEventKind::Effect(value) => {
            encoder.u8(16);
            encode_effect(encoder, *value);
        }
        BattleEventKind::RuleState(value) => {
            encoder.u8(17);
            encode_rule_state(encoder, value)?;
        }
        BattleEventKind::RuleSignal(value) => {
            encoder.u8(18);
            encode_rule_signal(encoder, value)?;
        }
        BattleEventKind::Fault(value) => {
            encoder.u8(19);
            encode_fault(encoder, *value);
        }
        _ => return Err(BattleEventPayloadError::UnsupportedEventFamily),
    }
    Ok(())
}

fn encode_battle(encoder: &mut Encoder<Vec<u8>>, value: BattleEventData) {
    match value {
        BattleEventData::Started => encoder.u8(0),
        BattleEventData::Conceded { side } => {
            encoder.u8(1);
            team_side(encoder, side);
        }
        BattleEventData::Won => encoder.u8(2),
        BattleEventData::Lost => encoder.u8(3),
    }
}

fn encode_decision(encoder: &mut Encoder<Vec<u8>>, value: DecisionEventData) {
    match value {
        DecisionEventData::Offered {
            decision,
            kind,
            owner,
        } => {
            encoder.u8(0);
            encoder.u64(decision.get());
            decision_kind(encoder, kind);
            decision_owner(encoder, owner);
        }
        DecisionEventData::Closed { decision } => {
            encoder.u8(1);
            encoder.u64(decision.get());
        }
    }
}

fn encode_turn(encoder: &mut Encoder<Vec<u8>>, value: TurnEventData) {
    let (kind, actor, owner) = match value {
        TurnEventData::Started { actor, owner } => (0, actor, owner),
        TurnEventData::Ended { actor, owner } => (1, actor, owner),
    };
    encoder.u8(kind);
    encoder.u64(actor.get());
    encoder.u64(owner.get());
}

fn encode_action(encoder: &mut Encoder<Vec<u8>>, value: ActionEventData) {
    match value {
        ActionEventData::Queued {
            insertion,
            actor,
            ability,
            origin,
            boundary,
        } => {
            encoder.u8(0);
            encoder.u64(insertion);
            encoder.u64(actor.get());
            encoder.u32(ability.get());
            encoder.u8(origin as u8);
            encoder.u8(boundary as u8);
        }
        ActionEventData::Declared {
            action,
            actor,
            ability,
            origin,
            tags,
        } => action_lifecycle(
            encoder,
            1,
            action.get(),
            actor.get(),
            ability.get(),
            origin as u8,
            tags,
        ),
        ActionEventData::Started {
            action,
            actor,
            ability,
            origin,
            tags,
        } => action_lifecycle(
            encoder,
            2,
            action.get(),
            actor.get(),
            ability.get(),
            origin as u8,
            tags,
        ),
        ActionEventData::Resolved {
            action,
            actor,
            ability,
            origin,
            tags,
        } => action_lifecycle(
            encoder,
            3,
            action.get(),
            actor.get(),
            ability.get(),
            origin as u8,
            tags,
        ),
        ActionEventData::Cancelled {
            insertion,
            actor,
            ability,
            origin,
        } => {
            encoder.u8(4);
            encoder.u64(insertion);
            encoder.u64(actor.get());
            encoder.u32(ability.get());
            encoder.u8(origin as u8);
        }
    }
}

fn action_lifecycle(
    encoder: &mut Encoder<Vec<u8>>,
    kind: u8,
    action: u64,
    actor: u64,
    ability: u32,
    origin: u8,
    tags: AbilityTags,
) {
    encoder.u8(kind);
    encoder.u64(action);
    encoder.u64(actor);
    encoder.u32(ability);
    encoder.u8(origin);
    encoder.u32(tags.bits());
}

fn encode_phase(encoder: &mut Encoder<Vec<u8>>, value: PhaseEventData) {
    let (kind, action, phase) = match value {
        PhaseEventData::Started { action, phase } => (0, action, phase),
        PhaseEventData::Ended { action, phase } => (1, action, phase),
    };
    encoder.u8(kind);
    encoder.u64(action.get());
    encoder.u64(phase.get());
}

fn encode_hit(
    encoder: &mut Encoder<Vec<u8>>,
    value: &HitEventData,
) -> Result<(), BattleEventPayloadError> {
    let (kind, action, phase, hit, targets) = match value {
        HitEventData::Started {
            action,
            phase,
            hit,
            targets,
        } => (0, action, phase, hit, targets),
        HitEventData::Ended {
            action,
            phase,
            hit,
            targets,
        } => (1, action, phase, hit, targets),
    };
    encoder.u8(kind);
    encoder.u64(action.get());
    encoder.u64(phase.get());
    encoder.u64(hit.get());
    encoder.u32(u32::try_from(targets.len()).map_err(|_| CodecError::LengthOverflow)?);
    for target in targets {
        encoder.u64(target.get());
    }
    Ok(())
}

fn encode_damage(encoder: &mut Encoder<Vec<u8>>, value: DamageEventData) {
    encoder.u64(value.operation.get());
    damage_kind(encoder, value.kind);
    damage_class(encoder, value.class);
    optional_element(encoder, value.element);
    optional_u64(encoder, value.source_effect.map(|item| item.get()));
    encoder.u64(value.target.get());
    encoder.i64(value.raw.scaled());
    encoder.i64(value.calculated.get());
    encoder.i64(value.absorbed.get());
    encoder.i64(value.applied.get());
    encoder.i64(value.hp_before.get());
    encoder.i64(value.hp_after.get());
}

fn encode_heal(encoder: &mut Encoder<Vec<u8>>, value: HealEventData) {
    encoder.u64(value.operation.get());
    encoder.u64(value.target.get());
    encoder.i64(value.raw.scaled());
    encoder.i64(value.calculated.get());
    encoder.i64(value.effective.get());
    encoder.i64(value.overheal.get());
    encoder.i64(value.hp_before.get());
    encoder.i64(value.hp_after.get());
}

fn encode_hp_consumption(encoder: &mut Encoder<Vec<u8>>, value: HpConsumptionEventData) {
    encoder.u64(value.operation.get());
    encoder.u64(value.target.get());
    encoder.i64(value.requested.get());
    encoder.i64(value.effective.get());
    encoder.i64(value.overflow.get());
    encoder.i64(value.hp_before.get());
    encoder.i64(value.hp_after.get());
}

fn encode_shield(encoder: &mut Encoder<Vec<u8>>, value: ShieldEventData) {
    match value {
        ShieldEventData::Applied {
            operation,
            shield,
            target,
            raw,
            amount,
        } => {
            encoder.u8(0);
            encoder.u64(operation.get());
            encoder.u64(shield.get());
            encoder.u64(target.get());
            encoder.i64(raw.scaled());
            encoder.i64(amount.get());
        }
        ShieldEventData::Absorbed {
            shield,
            target,
            before,
            after,
        } => {
            encoder.u8(1);
            encoder.u64(shield.get());
            encoder.u64(target.get());
            encoder.i64(before.get());
            encoder.i64(after.get());
        }
    }
}

fn encode_break_damage(encoder: &mut Encoder<Vec<u8>>, value: BreakDamageEventData) {
    encoder.u64(value.operation.get());
    encoder.u64(value.target.get());
    encoder.u8(match value.kind {
        BreakDamageKind::Initial => 0,
        BreakDamageKind::Effect => 1,
        BreakDamageKind::SuperBreak => 2,
    });
    element(encoder, value.element);
    encoder.i64(value.raw.scaled());
    encoder.i64(value.calculated.get());
    encoder.i64(value.absorbed.get());
    encoder.i64(value.applied.get());
    encoder.i64(value.hp_before.get());
    encoder.i64(value.hp_after.get());
}

fn encode_effect(encoder: &mut Encoder<Vec<u8>>, value: EffectEventData) {
    match value {
        EffectEventData::Applied {
            operation,
            effect,
            definition,
            target,
            stacks,
            remaining,
        } => {
            encoder.u8(0);
            operation_effect_target(encoder, operation.get(), effect.get(), target.get());
            encoder.u32(definition.get());
            encoder.u16(stacks);
            optional_u16(encoder, remaining);
        }
        EffectEventData::Resisted {
            operation,
            definition,
            target,
            pre_clamp_chance,
        } => {
            encoder.u8(1);
            encoder.u64(operation.get());
            encoder.u32(definition.get());
            encoder.u64(target.get());
            encoder.i64(pre_clamp_chance.scaled());
        }
        EffectEventData::Refreshed {
            operation,
            effect,
            target,
            stacks_before,
            stacks_after,
            remaining,
        } => {
            encoder.u8(2);
            operation_effect_target(encoder, operation.get(), effect.get(), target.get());
            encoder.u16(stacks_before);
            encoder.u16(stacks_after);
            optional_u16(encoder, remaining);
        }
        EffectEventData::Removed {
            operation,
            effect,
            target,
        } => {
            encoder.u8(3);
            operation_effect_target(encoder, operation.get(), effect.get(), target.get());
        }
        EffectEventData::Ticked {
            operation,
            effect,
            target,
            remaining,
        } => {
            encoder.u8(4);
            operation_effect_target(encoder, operation.get(), effect.get(), target.get());
            optional_u16(encoder, remaining);
        }
        EffectEventData::Detonated {
            operation,
            effect,
            target,
            fraction,
        } => {
            encoder.u8(5);
            operation_effect_target(encoder, operation.get(), effect.get(), target.get());
            encoder.i64(fraction.scaled());
        }
    }
}

fn encode_toughness(encoder: &mut Encoder<Vec<u8>>, value: ToughnessEventData) {
    match value {
        ToughnessEventData::WeaknessAdded {
            operation,
            target,
            element: value,
            already_present,
            duration_turns,
        } => {
            encoder.u8(0);
            encoder.u64(operation.get());
            encoder.u64(target.get());
            element(encoder, value);
            encoder.boolean(already_present);
            optional_u8(encoder, duration_turns);
        }
        ToughnessEventData::WeaknessRemoved {
            operation,
            target,
            element: value,
        } => {
            encoder.u8(1);
            encoder.u64(operation.get());
            encoder.u64(target.get());
            element(encoder, value);
        }
        ToughnessEventData::Reduced {
            operation,
            target,
            layer_key,
            attempted,
            effective,
            before,
            after,
        } => {
            encoder.u8(2);
            encoder.u64(operation.get());
            encoder.u64(target.get());
            optional_u32(encoder, layer_key);
            encoder.i64(attempted.get());
            encoder.i64(effective.get());
            encoder.i64(before.get());
            encoder.i64(after.get());
        }
        ToughnessEventData::LayerDepleted {
            operation,
            target,
            layer_key,
            changed_global_broken,
        } => {
            encoder.u8(3);
            encoder.u64(operation.get());
            encoder.u64(target.get());
            encoder.u32(layer_key);
            encoder.boolean(changed_global_broken);
        }
        ToughnessEventData::BaseEffectApplied {
            operation,
            target,
            effect,
            element: value,
            duration_turns,
            stacks,
        } => {
            encoder.u8(4);
            encoder.u64(operation.get());
            encoder.u64(target.get());
            encoder.u64(effect.get());
            element(encoder, value);
            encoder.u8(duration_turns);
            encoder.u8(stacks);
        }
        ToughnessEventData::BaseEffectResisted {
            operation,
            target,
            element: value,
        } => {
            encoder.u8(5);
            encoder.u64(operation.get());
            encoder.u64(target.get());
            element(encoder, value);
        }
        ToughnessEventData::BaseEffectTicked {
            operation,
            target,
            effect,
            remaining_turns,
            stacks,
        } => {
            encoder.u8(6);
            encoder.u64(operation.get());
            encoder.u64(target.get());
            encoder.u64(effect.get());
            encoder.u8(remaining_turns);
            encoder.u8(stacks);
        }
        ToughnessEventData::BaseEffectExpired {
            target,
            effect,
            element: value,
        } => {
            encoder.u8(7);
            encoder.u64(target.get());
            encoder.u64(effect.get());
            element(encoder, value);
        }
        ToughnessEventData::Recovered {
            target,
            layer_key,
            before,
            after,
            exited_global_broken,
        } => {
            encoder.u8(8);
            encoder.u64(target.get());
            encoder.u32(layer_key);
            encoder.i64(before.get());
            encoder.i64(after.get());
            encoder.boolean(exited_global_broken);
        }
        ToughnessEventData::SuperBreakSkipped {
            operation,
            target,
            effective_reduction,
        } => {
            encoder.u8(9);
            encoder.u64(operation.get());
            encoder.u64(target.get());
            encoder.i64(effective_reduction.get());
        }
    }
}

fn encode_unit(encoder: &mut Encoder<Vec<u8>>, value: UnitEventData) {
    match value {
        UnitEventData::Downed { unit } => {
            encoder.u8(0);
            encoder.u64(unit.get());
        }
        UnitEventData::Defeated { unit, credited_to } => {
            encoder.u8(1);
            encoder.u64(unit.get());
            encoder.u64(credited_to.get());
        }
        UnitEventData::Summoned {
            unit,
            owner,
            actor,
            kind,
        } => {
            encoder.u8(2);
            encoder.u64(unit.get());
            encoder.u64(owner.get());
            optional_u64(encoder, actor.map(|value| value.get()));
            encoder.u8(kind as u8);
        }
        UnitEventData::CountdownCreated {
            owner,
            actor,
            ability,
        } => {
            encoder.u8(3);
            encoder.u64(owner.get());
            encoder.u64(actor.get());
            encoder.u32(ability.get());
        }
        UnitEventData::PresenceChanged {
            unit,
            before,
            after,
        } => {
            encoder.u8(4);
            encoder.u64(unit.get());
            encoder.u8(before as u8);
            encoder.u8(after as u8);
        }
        UnitEventData::Transformed {
            unit,
            from,
            to,
            countdown,
        } => {
            encoder.u8(5);
            encoder.u64(unit.get());
            encoder.u32(from.get());
            encoder.u32(to.get());
            optional_u64(encoder, countdown.map(|value| value.get()));
        }
        UnitEventData::TransformationEnded {
            unit,
            restored_form,
        } => {
            encoder.u8(6);
            encoder.u64(unit.get());
            encoder.u32(restored_form.get());
        }
        UnitEventData::Revived { unit, hp, presence } => {
            encoder.u8(7);
            encoder.u64(unit.get());
            encoder.i64(hp.get());
            encoder.u8(presence as u8);
        }
        UnitEventData::Despawned { unit } => {
            encoder.u8(8);
            encoder.u64(unit.get());
        }
        UnitEventData::LinkSettled {
            owner,
            entity,
            policy,
        } => {
            encoder.u8(9);
            encoder.u64(owner.get());
            linked_entity(encoder, entity);
            encoder.u8(policy as u8);
        }
    }
}

fn linked_entity(encoder: &mut Encoder<Vec<u8>>, value: LinkedEntity) {
    match value {
        LinkedEntity::Unit(value) => {
            encoder.u8(0);
            encoder.u64(value.get());
        }
        LinkedEntity::TimelineActor(value) => {
            encoder.u8(1);
            encoder.u64(value.get());
        }
    }
}

fn encode_wave(encoder: &mut Encoder<Vec<u8>>, value: WaveEventData) {
    let (kind, wave, number) = match value {
        WaveEventData::Ended { wave, number } => (0, wave, number),
        WaveEventData::Started { wave, number } => (1, wave, number),
    };
    encoder.u8(kind);
    encoder.u64(wave.get());
    encoder.u16(number);
}

fn encode_enemy_phase(encoder: &mut Encoder<Vec<u8>>, value: EnemyPhaseEventData) {
    match value {
        EnemyPhaseEventData::Transitioned {
            unit,
            from,
            to,
            model,
            graph,
            state,
        } => {
            encoder.u8(0);
            encoder.u64(unit.get());
            optional_u32(encoder, from.map(|value| value.get()));
            encoder.u32(to.get());
            enemy_phase_model(encoder, model);
            encoder.u32(graph.get());
            encoder.u32(state.get());
        }
    }
}

fn enemy_phase_model(encoder: &mut Encoder<Vec<u8>>, value: EnemyPhaseTransitionModel) {
    match value {
        EnemyPhaseTransitionModel::TransformSameUnit => encoder.u8(0),
        EnemyPhaseTransitionModel::ReplaceLinkedVariant(definition) => {
            encoder.u8(1);
            encoder.u32(definition.get());
        }
        EnemyPhaseTransitionModel::ExplicitWave => encoder.u8(2),
    }
}

fn encode_resource(
    encoder: &mut Encoder<Vec<u8>>,
    value: &ResourceEventData,
) -> Result<(), BattleEventPayloadError> {
    match value {
        ResourceEventData::SkillPoints {
            side,
            attempted,
            payer,
            effective,
            before,
            after,
            overflow,
        } => {
            encoder.u8(0);
            team_side(encoder, *side);
            encoder.u16(*attempted);
            skill_point_payer(encoder, *payer);
            encoder.u16(*effective);
            encoder.u16(*before);
            encoder.u16(*after);
            encoder.u16(*overflow);
        }
        ResourceEventData::Energy {
            unit,
            before,
            after,
            overflow,
        } => {
            encoder.u8(1);
            encoder.u64(unit.get());
            encoder.i64(before.scaled());
            encoder.i64(after.scaled());
            encoder.i64(overflow.scaled());
        }
        ResourceEventData::CharacterResource {
            unit,
            resource,
            before,
            after,
            maximum,
        } => {
            encoder.u8(2);
            encoder.u64(unit.get());
            encoder.string(resource)?;
            encoder.i64(before.scaled());
            encoder.i64(after.scaled());
            encoder.i64(maximum.scaled());
        }
        ResourceEventData::TeamResource {
            side,
            resource,
            attempted,
            effective,
            before,
            after,
            overflow,
        } => {
            encoder.u8(3);
            team_side(encoder, *side);
            encoder.u32(resource.get());
            encoder.u16(*attempted);
            encoder.u16(*effective);
            encoder.u16(*before);
            encoder.u16(*after);
            encoder.u16(*overflow);
        }
    }
    Ok(())
}

fn skill_point_payer(encoder: &mut Encoder<Vec<u8>>, value: SkillPointPayer) {
    match value {
        SkillPointPayer::TeamSkillPoints => encoder.u8(0),
        SkillPointPayer::TeamResource(resource) => {
            encoder.u8(1);
            encoder.u32(resource.get());
        }
        SkillPointPayer::Suppressed => encoder.u8(2),
    }
}

fn encode_rule_state(
    encoder: &mut Encoder<Vec<u8>>,
    value: &RuleStateEventData,
) -> Result<(), BattleEventPayloadError> {
    encoder.u64(value.operation.get());
    encoder.u64(value.instance.get());
    encoder.u32(value.slot.get());
    rule_value(encoder, &value.before)?;
    rule_value(encoder, &value.after)
}

fn encode_rule_signal(
    encoder: &mut Encoder<Vec<u8>>,
    value: &RuleSignalEventData,
) -> Result<(), BattleEventPayloadError> {
    encoder.u64(value.operation.get());
    encoder.u32(value.code);
    encoder.boolean(value.value.is_some());
    if let Some(value) = &value.value {
        rule_value(encoder, value)?;
    }
    Ok(())
}

fn rule_value(
    encoder: &mut Encoder<Vec<u8>>,
    value: &RuleValue,
) -> Result<(), BattleEventPayloadError> {
    match value {
        RuleValue::Integer(value) => {
            encoder.u8(0);
            encoder.i64(*value);
        }
        RuleValue::Scalar(value) => {
            encoder.u8(1);
            encoder.i64(value.scaled());
        }
        RuleValue::Boolean(value) => {
            encoder.u8(2);
            encoder.boolean(*value);
        }
        RuleValue::StableId(value) => {
            encoder.u8(3);
            encoder.u64(*value);
        }
        RuleValue::OptionalStableId(value) => {
            encoder.u8(4);
            optional_u64(encoder, *value);
        }
        RuleValue::OrderedStableIdSet(values) => {
            encoder.u8(5);
            encoder.u32(u32::try_from(values.len()).map_err(|_| CodecError::LengthOverflow)?);
            for value in values {
                encoder.u64(*value);
            }
        }
    }
    Ok(())
}

fn encode_fault(encoder: &mut Encoder<Vec<u8>>, value: FaultEventData) {
    let fault = value.fault();
    encoder.u8(fault.kind() as u8);
    encoder.u8(fault.boundary() as u8);
    encoder.u8(fault.policy() as u8);
    encoder.u32(fault.context_code());
    optional_i64(encoder, fault.numeric_context());
}

fn operation_effect_target(
    encoder: &mut Encoder<Vec<u8>>,
    operation: u64,
    effect: u64,
    target: u64,
) {
    encoder.u64(operation);
    encoder.u64(effect);
    encoder.u64(target);
}

/// Stable payload encoding failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleEventPayloadError {
    Codec(CodecError),
    UnsupportedEventFamily,
}

impl From<CodecError> for BattleEventPayloadError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}

fn optional_u16(encoder: &mut Encoder<Vec<u8>>, value: Option<u16>) {
    encoder.boolean(value.is_some());
    if let Some(value) = value {
        encoder.u16(value);
    }
}

fn optional_u8(encoder: &mut Encoder<Vec<u8>>, value: Option<u8>) {
    encoder.boolean(value.is_some());
    if let Some(value) = value {
        encoder.u8(value);
    }
}

fn optional_u32(encoder: &mut Encoder<Vec<u8>>, value: Option<u32>) {
    encoder.boolean(value.is_some());
    if let Some(value) = value {
        encoder.u32(value);
    }
}

fn optional_u64(encoder: &mut Encoder<Vec<u8>>, value: Option<u64>) {
    encoder.boolean(value.is_some());
    if let Some(value) = value {
        encoder.u64(value);
    }
}

fn optional_i64(encoder: &mut Encoder<Vec<u8>>, value: Option<i64>) {
    encoder.boolean(value.is_some());
    if let Some(value) = value {
        encoder.i64(value);
    }
}

fn team_side(encoder: &mut Encoder<Vec<u8>>, value: TeamSide) {
    encoder.u8(match value {
        TeamSide::Player => 0,
        TeamSide::Enemy => 1,
    });
}

fn decision_kind(encoder: &mut Encoder<Vec<u8>>, value: DecisionKind) {
    encoder.u8(match value {
        DecisionKind::BattleStart => 0,
        DecisionKind::NormalAction => 1,
        DecisionKind::InterruptWindow => 2,
        DecisionKind::BattleChoice => 3,
    });
}

fn decision_owner(encoder: &mut Encoder<Vec<u8>>, value: DecisionOwner) {
    match value {
        DecisionOwner::System => encoder.u8(0),
        DecisionOwner::Team(side) => {
            encoder.u8(1);
            team_side(encoder, side);
        }
    }
}

fn damage_kind(encoder: &mut Encoder<Vec<u8>>, value: DamageKind) {
    encoder.u8(match value {
        DamageKind::Direct => 0,
        DamageKind::DotTick => 1,
        DamageKind::DotDetonation => 2,
    });
}

fn damage_class(encoder: &mut Encoder<Vec<u8>>, value: DamageClass) {
    encoder.u8(match value {
        DamageClass::Direct => 0,
        DamageClass::Dot => 1,
        DamageClass::Additional => 2,
        DamageClass::Elation => 3,
    });
}

fn optional_element(encoder: &mut Encoder<Vec<u8>>, value: Option<CombatElement>) {
    encoder.boolean(value.is_some());
    if let Some(value) = value {
        element(encoder, value);
    }
}

fn element(encoder: &mut Encoder<Vec<u8>>, value: CombatElement) {
    encoder.u8(value as u8);
}
