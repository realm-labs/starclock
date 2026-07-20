use crate::{
    ActionGauge, LifeState, LinkedEntity, LinkedEntityKind, OwnerLinkPolicy, PresenceState,
    ReviveGaugePolicy, TransformEndPolicy, WaveLinkPolicy,
    actor::store::{FormationEntry, LinkState, TimelineActorState, TransformationState, UnitState},
    battle::fault::BattleFault,
    event::{
        cause::Cause,
        model::{BattleEventKind, EnemyPhaseEventData, UnitEventData},
    },
    id::{EventId, UnitId},
    operation::{
        ChangePresenceOp, EnemyPhaseOp, ReviveOp, SummonLinkedOp, TransformOp, UnitLifecycleOp,
    },
};

use super::transaction::{Transaction, action_fault};

const BASE_ACTION_GAUGE_SCALED: i64 = 10_000_000_000;
const MAX_LINKED_ENTITIES: usize = 64;

pub(super) fn execute_enemy_phase(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: EnemyPhaseOp,
) -> Result<EventId, BattleFault> {
    for unit in operation.targets {
        let runtime = txn
            .state
            .units
            .get(unit)
            .and_then(|state| state.enemy)
            .ok_or_else(|| action_fault(97))?;
        let enemy = catalog
            .enemy(runtime.definition)
            .ok_or_else(|| action_fault(98))?;
        let phase = enemy
            .phases()
            .iter()
            .find(|phase| phase.id() == operation.phase)
            .ok_or_else(|| action_fault(99))?;
        let expected_sequence = match runtime.phase {
            None => 1,
            Some(current) => enemy
                .phases()
                .iter()
                .find(|phase| phase.id() == current)
                .and_then(|phase| phase.sequence().checked_add(1))
                .ok_or_else(|| action_fault(100))?,
        };
        if phase.sequence() != expected_sequence {
            return Err(action_fault(101));
        }
        parent = apply_phase_carry(txn, cause, parent, unit, phase.carry())?;
        let presence = if phase.targetable() {
            PresenceState::Present
        } else {
            PresenceState::Untargetable
        };
        if let crate::catalog::encounter::EnemyPhaseTransitionModel::ReplaceLinkedVariant(
            replacement,
        ) = phase.transition()
        {
            let replacement = catalog
                .enemy(replacement)
                .ok_or_else(|| action_fault(102))?;
            txn.set_unit_definition(
                unit,
                replacement.unit(),
                replacement.abilities().into(),
                presence,
                None,
            )?;
        } else {
            txn.set_presence(unit, presence)?;
        }
        let graph = phase.ai_graph();
        let state = catalog
            .ai_graph(graph)
            .ok_or_else(|| action_fault(103))?
            .initial_state();
        txn.set_enemy_runtime(
            unit,
            crate::actor::store::EnemyRuntimeState {
                definition: runtime.definition,
                graph,
                state,
                turn_counter: 0,
                phase: Some(operation.phase),
            },
        )?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(unit)),
            BattleEventKind::EnemyPhase(EnemyPhaseEventData::Transitioned {
                unit,
                from: runtime.phase,
                to: operation.phase,
                model: phase.transition(),
                graph,
                state,
            }),
        );
    }
    Ok(parent)
}

fn apply_phase_carry(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    unit: UnitId,
    carry: crate::catalog::encounter::EnemyPhaseCarry,
) -> Result<EventId, BattleFault> {
    use crate::catalog::encounter::PhaseCarryPolicy;
    let maximum_hp = txn
        .state
        .units
        .get(unit)
        .map(|state| state.maximum_hp)
        .ok_or_else(|| action_fault(104))?;
    match carry.hp {
        PhaseCarryPolicy::CarryExact | PhaseCarryPolicy::CarryRatio => {}
        PhaseCarryPolicy::Reset => txn.set_hp(unit, maximum_hp)?,
        PhaseCarryPolicy::Clear => {
            txn.set_hp(unit, crate::Hp::new(1).map_err(|_| action_fault(105))?)?
        }
        PhaseCarryPolicy::ExplicitProgram(_) => return Err(action_fault(106)),
    }
    match carry.action_gauge {
        PhaseCarryPolicy::CarryExact | PhaseCarryPolicy::CarryRatio => {}
        PhaseCarryPolicy::Reset | PhaseCarryPolicy::Clear => {
            let actor = txn
                .state
                .actors
                .any_id_for_unit(unit)
                .ok_or_else(|| action_fault(107))?;
            let gauge = if carry.action_gauge == PhaseCarryPolicy::Reset {
                base_gauge()?
            } else {
                ActionGauge::from_scaled(0).map_err(|_| action_fault(108))?
            };
            txn.set_actor_gauge(actor, gauge)?;
        }
        PhaseCarryPolicy::ExplicitProgram(_) => return Err(action_fault(109)),
    }
    if !matches!(
        carry.effects,
        PhaseCarryPolicy::CarryExact | PhaseCarryPolicy::CarryRatio
    ) {
        if matches!(carry.effects, PhaseCarryPolicy::ExplicitProgram(_)) {
            return Err(action_fault(110));
        }
        let effects = txn
            .state
            .effects
            .iter_by_id()
            .filter(|effect| {
                effect.target == unit
                    && (carry.effects == PhaseCarryPolicy::Clear
                        || effect.duration_clock != crate::DurationClock::Permanent)
            })
            .map(|effect| effect.id)
            .collect::<Vec<_>>();
        for effect in effects {
            if let Some(removed) = txn.state.effects.remove(effect) {
                txn.record_effect_change(effect.get(), 0, effect.get());
                parent = txn.emit(
                    cause.with_parent(parent).with_primary_target(Some(unit)),
                    BattleEventKind::Effect(crate::EffectEventData::Removed {
                        operation: removed.source_operation,
                        effect,
                        target: unit,
                    }),
                );
            }
        }
    }
    match carry.toughness {
        PhaseCarryPolicy::CarryExact | PhaseCarryPolicy::CarryRatio => {}
        PhaseCarryPolicy::Reset | PhaseCarryPolicy::Clear => {
            let layers = txn
                .state
                .units
                .get(unit)
                .ok_or_else(|| action_fault(111))?
                .toughness_layers
                .iter()
                .map(|layer| {
                    let current = if carry.toughness == PhaseCarryPolicy::Reset {
                        layer.spec.maximum()
                    } else {
                        crate::RawToughness::new(0).expect("zero Toughness is valid")
                    };
                    (layer.spec.key(), current)
                })
                .collect::<Vec<_>>();
            for (key, current) in layers {
                txn.set_toughness(unit, key, current)?;
            }
            txn.set_weakness_broken(unit, carry.toughness == PhaseCarryPolicy::Clear)?;
        }
        PhaseCarryPolicy::ExplicitProgram(_) => return Err(action_fault(112)),
    }
    match carry.summons {
        PhaseCarryPolicy::CarryExact | PhaseCarryPolicy::CarryRatio => {}
        PhaseCarryPolicy::Reset | PhaseCarryPolicy::Clear => {
            let entities = txn
                .state
                .links
                .canonical_entries()
                .iter()
                .filter(|link| link.owner == unit && link.active)
                .map(|link| link.entity)
                .collect::<Vec<_>>();
            for entity in entities {
                if let Some(actor) = actor_for_entity(txn, entity) {
                    txn.set_actor_active(actor, false)?;
                }
                if let LinkedEntity::Unit(linked) = entity {
                    txn.set_presence(linked, PresenceState::Departed)?;
                }
                txn.set_link_active(entity, false)?;
                parent = txn.emit(
                    cause.with_parent(parent),
                    BattleEventKind::Unit(UnitEventData::LinkSettled {
                        owner: unit,
                        entity,
                        policy: OwnerLinkPolicy::Depart,
                    }),
                );
            }
        }
        PhaseCarryPolicy::ExplicitProgram(_) => return Err(action_fault(113)),
    }
    Ok(parent)
}

pub(super) fn execute_summon(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: SummonLinkedOp,
) -> Result<EventId, BattleFault> {
    for owner in operation.owners {
        if txn.state.links.canonical_entries().len() >= MAX_LINKED_ENTITIES {
            return Err(budget_fault(1));
        }
        let owner_state = txn.state.units.get(owner).ok_or_else(|| action_fault(80))?;
        if owner_state.life != LifeState::Alive || !owner_state.presence.is_active() {
            return Err(action_fault(81));
        }
        let side = owner_state.side;
        let entry_wave = txn.state.encounter.number;
        let definition = &operation.definition;
        let combatant = definition.combatant();
        validate_combatant(catalog, combatant)?;
        let unit = txn.allocate_unit();
        let spawn = txn.allocate_spawn();
        let actor = definition.action_ability().map(|ability| {
            let actor = txn.allocate_actor();
            txn.insert_actor(TimelineActorState {
                id: actor,
                owner,
                unit: Some(unit),
                kind: Some(definition.kind()),
                automatic_ability: Some(ability),
                active: true,
                gauge: definition.initial_gauge(),
                speed: combatant.speed(),
            });
            actor
        });
        txn.insert_unit(UnitState {
            id: unit,
            spawn,
            form: combatant.form(),
            source: crate::ParticipantSource::Linked(definition.source()),
            side,
            formation: definition.formation(),
            entry_wave,
            level: combatant.level(),
            life: LifeState::Alive,
            presence: definition.presence(),
            current_hp: combatant.maximum_hp(),
            maximum_hp: combatant.maximum_hp(),
            base_attack: combatant.base_attack(),
            base_defense: combatant.base_defense(),
            base_speed: combatant.speed(),
            current_energy: combatant.current_energy(),
            maximum_energy: combatant.maximum_energy(),
            rank: combatant.rank(),
            weaknesses: combatant.weaknesses().to_vec(),
            permanent_weaknesses: combatant.weaknesses().into(),
            temporary_weaknesses: Vec::new(),
            toughness_layers: combatant
                .toughness_layers()
                .iter()
                .cloned()
                .map(crate::toughness::state::ToughnessLayerState::from_spec)
                .collect(),
            weakness_broken: false,
            abilities: combatant.abilities().into(),
            rule_bundles: combatant.rule_bundles().into(),
            modifiers: combatant.modifiers().into(),
            digest: combatant.digest(),
            transformation: None,
            enemy: None,
        });
        txn.insert_formation(FormationEntry {
            side,
            index: definition.formation(),
            unit,
        });
        txn.insert_link(LinkState {
            owner,
            entity: LinkedEntity::Unit(unit),
            kind: definition.kind(),
            owner_defeat: definition.owner_defeat(),
            owner_departure: definition.owner_departure(),
            wave: definition.wave(),
            active: true,
        })?;
        instantiate_rules(catalog, txn, unit, combatant.rule_bundles())?;
        instantiate_modifiers(txn, unit, combatant)?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(unit)),
            BattleEventKind::Unit(UnitEventData::Summoned {
                unit,
                owner,
                actor,
                kind: definition.kind(),
            }),
        );
    }
    Ok(parent)
}

pub(super) fn execute_countdown(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    operation: crate::operation::CreateCountdownOp,
) -> Result<EventId, BattleFault> {
    if txn.state.links.canonical_entries().len() >= MAX_LINKED_ENTITIES {
        return Err(budget_fault(3));
    }
    let owner = txn
        .state
        .units
        .get(operation.owner)
        .ok_or_else(|| action_fault(114))?;
    if owner.life != LifeState::Alive || !owner.presence.is_active() {
        return Err(action_fault(115));
    }
    let definition = operation.definition;
    if catalog
        .ability(definition.ability())
        .and_then(crate::catalog::definition::AbilityDefinition::action)
        .is_none_or(|action| action.kind() != crate::catalog::action::AbilityKind::Countdown)
    {
        return Err(action_fault(116));
    }
    let actor = txn.allocate_actor();
    txn.insert_actor(TimelineActorState {
        id: actor,
        owner: operation.owner,
        unit: None,
        kind: Some(LinkedEntityKind::Countdown),
        automatic_ability: Some(definition.ability()),
        active: true,
        gauge: definition.initial_gauge(),
        speed: definition.speed(),
    });
    txn.insert_link(LinkState {
        owner: operation.owner,
        entity: LinkedEntity::TimelineActor(actor),
        kind: LinkedEntityKind::Countdown,
        owner_defeat: definition.owner_defeat(),
        owner_departure: definition.owner_departure(),
        wave: definition.wave(),
        active: true,
    })?;
    Ok(txn.emit(
        cause.with_parent(parent),
        BattleEventKind::Unit(UnitEventData::CountdownCreated {
            owner: operation.owner,
            actor,
            ability: definition.ability(),
        }),
    ))
}

pub(super) fn execute_presence(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: ChangePresenceOp,
) -> Result<EventId, BattleFault> {
    for unit in operation.targets {
        let before = txn
            .state
            .units
            .get(unit)
            .ok_or_else(|| action_fault(83))?
            .presence;
        txn.set_presence(unit, operation.presence)?;
        if before != operation.presence {
            parent = txn.emit(
                cause.with_parent(parent).with_primary_target(Some(unit)),
                BattleEventKind::Unit(UnitEventData::PresenceChanged {
                    unit,
                    before,
                    after: operation.presence,
                }),
            );
        }
        if !operation.presence.is_active() {
            parent = settle_owner_boundary(txn, cause, parent, unit, false)?;
        }
    }
    Ok(parent)
}

pub(super) fn execute_transform(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: TransformOp,
) -> Result<EventId, BattleFault> {
    for unit in operation.targets {
        let current = txn.state.units.get(unit).ok_or_else(|| action_fault(84))?;
        if current.transformation.is_some()
            || current.life != LifeState::Alive
            || !current.presence.is_active()
        {
            return Err(action_fault(85));
        }
        validate_transform(catalog, &operation.definition)?;
        let original_form = current.form;
        let original_abilities = current.abilities.clone();
        let original_presence = current.presence;
        let countdown_actor = if let Some(countdown) = operation.definition.countdown() {
            if txn.state.links.canonical_entries().len() >= MAX_LINKED_ENTITIES {
                return Err(budget_fault(2));
            }
            let actor = txn.allocate_actor();
            txn.insert_actor(TimelineActorState {
                id: actor,
                owner: unit,
                unit: None,
                kind: Some(LinkedEntityKind::Countdown),
                automatic_ability: Some(countdown.ability()),
                active: true,
                gauge: countdown.initial_gauge(),
                speed: countdown.speed(),
            });
            txn.insert_link(LinkState {
                owner: unit,
                entity: LinkedEntity::TimelineActor(actor),
                kind: LinkedEntityKind::Countdown,
                owner_defeat: countdown.owner_defeat(),
                owner_departure: countdown.owner_departure(),
                wave: countdown.wave(),
                active: true,
            })?;
            Some(actor)
        } else {
            None
        };
        let transform = TransformationState {
            source_operation: operation.id,
            original_form,
            original_abilities,
            original_presence,
            countdown_actor,
            defeat: operation.definition.defeat(),
            wave: operation.definition.wave(),
        };
        txn.set_unit_definition(
            unit,
            operation.definition.replacement_form(),
            operation.definition.replacement_abilities().into(),
            PresenceState::Transformed,
            Some(transform),
        )?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(unit)),
            BattleEventKind::Unit(UnitEventData::Transformed {
                unit,
                from: original_form,
                to: operation.definition.replacement_form(),
                countdown: countdown_actor,
            }),
        );
    }
    Ok(parent)
}

pub(super) fn execute_end_transform(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: UnitLifecycleOp,
) -> Result<EventId, BattleFault> {
    for unit in operation.targets {
        parent = end_transform(txn, cause, parent, unit)?;
    }
    Ok(parent)
}

pub(super) fn execute_revive(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: ReviveOp,
) -> Result<EventId, BattleFault> {
    for unit in operation.targets {
        let state = txn.state.units.get(unit).ok_or_else(|| action_fault(86))?;
        if state.life == LifeState::Alive || operation.definition.restored_hp() > state.maximum_hp {
            return Err(action_fault(87));
        }
        let linked = txn.state.links.for_unit(unit);
        if let Some(link) = linked {
            let owner = txn
                .state
                .units
                .get(link.owner)
                .ok_or_else(|| action_fault(98))?;
            if owner.life != LifeState::Alive || !owner.presence.is_active() {
                return Err(action_fault(99));
            }
        }
        txn.set_hp(unit, operation.definition.restored_hp())?;
        txn.set_life(unit, LifeState::Alive)?;
        txn.set_presence(unit, operation.definition.presence())?;
        if linked.is_some() {
            txn.set_link_active(LinkedEntity::Unit(unit), true)?;
        }
        if let Some(actor) = txn.state.actors.any_id_for_unit(unit) {
            txn.set_actor_active(actor, true)?;
            let gauge = match operation.definition.gauge() {
                ReviveGaugePolicy::Preserve => None,
                ReviveGaugePolicy::Reset => Some(base_gauge()?),
                ReviveGaugePolicy::Immediate => {
                    Some(ActionGauge::from_scaled(0).map_err(|_| action_fault(97))?)
                }
            };
            if let Some(gauge) = gauge {
                txn.set_actor_gauge(actor, gauge)?;
            }
        }
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(unit)),
            BattleEventKind::Unit(UnitEventData::Revived {
                unit,
                hp: operation.definition.restored_hp(),
                presence: operation.definition.presence(),
            }),
        );
    }
    Ok(parent)
}

pub(super) fn execute_despawn(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: UnitLifecycleOp,
) -> Result<EventId, BattleFault> {
    for unit in operation.targets {
        if txn.state.links.for_unit(unit).is_none() {
            return Err(action_fault(88));
        }
        depart_linked_unit(txn, unit)?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(unit)),
            BattleEventKind::Unit(UnitEventData::Despawned { unit }),
        );
    }
    Ok(parent)
}

pub(super) fn settle_owner_defeat(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    owner: UnitId,
) -> Result<EventId, BattleFault> {
    settle_owner_boundary(txn, cause, parent, owner, true)
}

pub(super) fn settle_owner_departure(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    owner: UnitId,
) -> Result<EventId, BattleFault> {
    settle_owner_boundary(txn, cause, parent, owner, false)
}

pub(super) fn settle_wave_links(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
) -> Result<EventId, BattleFault> {
    let transformed = txn
        .state
        .units
        .iter_by_id()
        .filter(|unit| {
            unit.transformation
                .as_ref()
                .is_some_and(|state| state.wave == TransformEndPolicy::End)
        })
        .map(|unit| unit.id)
        .collect::<Vec<_>>();
    for unit in transformed {
        parent = end_transform(txn, cause, parent, unit)?;
    }

    let links = txn
        .state
        .links
        .canonical_entries()
        .iter()
        .copied()
        .filter(|link| link.active)
        .collect::<Vec<_>>();
    for link in links {
        match link.wave {
            WaveLinkPolicy::Persist => {}
            WaveLinkPolicy::ResetGauge => {
                if let Some(actor) = actor_for_entity(txn, link.entity) {
                    txn.set_actor_gauge(actor, base_gauge()?)?;
                }
            }
            WaveLinkPolicy::Depart => {
                apply_link_policy(txn, link.entity, OwnerLinkPolicy::Depart)?;
                txn.set_link_active(link.entity, false)?;
                parent = emit_link(txn, cause, parent, link, OwnerLinkPolicy::Depart);
            }
        }
    }
    Ok(parent)
}

fn settle_owner_boundary(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    owner: UnitId,
    defeated: bool,
) -> Result<EventId, BattleFault> {
    if defeated
        && txn
            .state
            .units
            .get(owner)
            .and_then(|unit| unit.transformation.as_ref())
            .is_some_and(|state| state.defeat == TransformEndPolicy::End)
    {
        parent = end_transform(txn, cause, parent, owner)?;
    }
    let links = txn.state.links.active_for_owner(owner).collect::<Vec<_>>();
    for link in links {
        let policy = if defeated {
            link.owner_defeat
        } else {
            link.owner_departure
        };
        if policy == OwnerLinkPolicy::Persist {
            continue;
        }
        apply_link_policy(txn, link.entity, policy)?;
        txn.set_link_active(link.entity, false)?;
        parent = emit_link(txn, cause, parent, link, policy);
    }
    Ok(parent)
}

fn apply_link_policy(
    txn: &mut Transaction<'_>,
    entity: LinkedEntity,
    policy: OwnerLinkPolicy,
) -> Result<(), BattleFault> {
    match entity {
        LinkedEntity::Unit(unit) => match policy {
            OwnerLinkPolicy::Persist => {}
            OwnerLinkPolicy::Depart => depart_linked_unit(txn, unit)?,
            OwnerLinkPolicy::Defeat => {
                txn.set_hp(unit, crate::Hp::new(0).expect("zero HP is in domain"))?;
                txn.set_life(unit, LifeState::Defeated)?;
                if let Some(actor) = txn.state.actors.any_id_for_unit(unit) {
                    txn.set_actor_active(actor, false)?;
                }
            }
        },
        LinkedEntity::TimelineActor(actor) => {
            if policy != OwnerLinkPolicy::Persist {
                txn.set_actor_active(actor, false)?;
            }
        }
    }
    Ok(())
}

fn depart_linked_unit(txn: &mut Transaction<'_>, unit: UnitId) -> Result<(), BattleFault> {
    txn.set_presence(unit, PresenceState::Departed)?;
    if let Some(actor) = txn.state.actors.any_id_for_unit(unit) {
        txn.set_actor_active(actor, false)?;
    }
    if txn.state.links.for_unit(unit).is_some() {
        txn.set_link_active(LinkedEntity::Unit(unit), false)?;
    }
    Ok(())
}

fn end_transform(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    unit: UnitId,
) -> Result<EventId, BattleFault> {
    let transform = txn
        .state
        .units
        .get(unit)
        .and_then(|unit| unit.transformation.clone())
        .ok_or_else(|| action_fault(89))?;
    if let Some(actor) = transform.countdown_actor {
        txn.set_actor_active(actor, false)?;
        txn.set_link_active(LinkedEntity::TimelineActor(actor), false)?;
    }
    txn.set_unit_definition(
        unit,
        transform.original_form,
        transform.original_abilities,
        transform.original_presence,
        None,
    )?;
    Ok(txn.emit(
        cause.with_parent(parent).with_primary_target(Some(unit)),
        BattleEventKind::Unit(UnitEventData::TransformationEnded {
            unit,
            restored_form: transform.original_form,
        }),
    ))
}

fn actor_for_entity(txn: &Transaction<'_>, entity: LinkedEntity) -> Option<crate::TimelineActorId> {
    match entity {
        LinkedEntity::Unit(unit) => txn.state.actors.any_id_for_unit(unit),
        LinkedEntity::TimelineActor(actor) => Some(actor),
    }
}

fn emit_link(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    link: LinkState,
    policy: OwnerLinkPolicy,
) -> EventId {
    txn.emit(
        cause.with_parent(parent),
        BattleEventKind::Unit(UnitEventData::LinkSettled {
            owner: link.owner,
            entity: link.entity,
            policy,
        }),
    )
}

fn validate_combatant(
    catalog: &crate::catalog::CombatCatalog,
    combatant: &crate::ResolvedCombatantSpec,
) -> Result<(), BattleFault> {
    if catalog.unit(combatant.form()).is_none()
        || combatant
            .abilities()
            .iter()
            .any(|ability| catalog.ability(*ability).is_none())
        || combatant
            .rule_bundles()
            .iter()
            .any(|bundle| catalog.rule_bundle(*bundle).is_none())
        || combatant
            .modifiers()
            .iter()
            .any(|modifier| catalog.modifier(*modifier).is_none())
        || combatant.modifier_bindings().len() != combatant.modifiers().len()
        || combatant.modifier_bindings().iter().any(|binding| {
            combatant
                .sources()
                .binary_search_by_key(&binding.source(), |source| source.definition())
                .is_err()
        })
    {
        return Err(action_fault(90));
    }
    Ok(())
}

fn instantiate_modifiers(
    txn: &mut Transaction<'_>,
    unit: UnitId,
    combatant: &crate::ResolvedCombatantSpec,
) -> Result<(), BattleFault> {
    for binding in combatant.modifier_bindings() {
        let source = combatant
            .sources()
            .binary_search_by_key(&binding.source(), |source| source.definition())
            .ok()
            .map(|index| &combatant.sources()[index])
            .ok_or_else(|| action_fault(95))?;
        let instance = txn.allocate_modifier();
        txn.insert_modifier(crate::modifier::model::ActiveModifier {
            instance,
            definition: binding.definition(),
            owner: unit,
            subject: unit,
            source: binding.source(),
            source_class: source.class(),
            insertion_sequence: instance.get(),
            application_action: None,
            slots: Box::new([]),
            captured_value: None,
            captured_stats: Box::new([]),
        })?;
    }
    Ok(())
}

fn validate_transform(
    catalog: &crate::catalog::CombatCatalog,
    definition: &crate::TransformationDefinition,
) -> Result<(), BattleFault> {
    let unit = catalog
        .unit(definition.replacement_form())
        .ok_or_else(|| action_fault(91))?;
    if definition.replacement_abilities().iter().any(|ability| {
        catalog.ability(*ability).is_none() || unit.abilities().binary_search(ability).is_err()
    }) || definition.countdown().is_some_and(|countdown| {
        catalog
            .ability(countdown.ability())
            .and_then(|ability| ability.action())
            .is_none_or(|action| action.kind() != crate::catalog::action::AbilityKind::Countdown)
    }) {
        return Err(action_fault(92));
    }
    Ok(())
}

fn instantiate_rules(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    unit: UnitId,
    bundles: &[crate::RuleBundleId],
) -> Result<(), BattleFault> {
    for bundle_id in bundles {
        let bundle = catalog
            .rule_bundle(*bundle_id)
            .ok_or_else(|| action_fault(93))?;
        for rule_id in bundle.rules() {
            let definition = catalog.rule(*rule_id).ok_or_else(|| action_fault(94))?;
            if let Some(runtime) = definition.runtime() {
                let id = txn.allocate_rule();
                if !txn.state.rules.insert(id, *rule_id, Some(unit), runtime) {
                    return Err(action_fault(95));
                }
            }
        }
    }
    Ok(())
}

fn base_gauge() -> Result<ActionGauge, BattleFault> {
    ActionGauge::from_scaled(BASE_ACTION_GAUGE_SCALED).map_err(|_| action_fault(96))
}

fn budget_fault(code: u32) -> BattleFault {
    BattleFault::new(
        crate::FaultKind::BudgetExceeded,
        crate::FaultBoundary::Command,
        crate::FaultPolicy::Rollback,
        0x31a0 + code,
        Some(MAX_LINKED_ENTITIES as i64),
    )
}
