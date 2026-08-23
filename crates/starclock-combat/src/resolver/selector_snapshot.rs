//! Compact immutable battlefield projections for authored selector reference points.

use crate::{
    battle::state::BattleState,
    catalog::selector::RuleSelectorReference,
    event::{
        cause::Cause,
        model::{BattleEvent, BattleEventKind},
    },
    formula::toughness::attacker_level_multiplier,
    modifier::model::StatKind::{
        self, Atk, BreakBaseDamage, BreakEffect, CritDamage, CritRate, Def, DotDurationAddition,
        EnergyRegenerationRate, FireDamageBoost, Hp as HpStat, IceDamageBoost,
        ImaginaryDamageBoost, LightningDamageBoost, OutgoingHealing, PhysicalDamageBoost,
        QuantumDamageBoost, Spd, ToughnessDamage, ToughnessRecovery, WindDamageBoost,
    },
};
use std::{collections::BTreeMap, sync::Arc};

use super::transaction;
use crate::{
    ActionGauge, ActionId, EffectDefinitionId, EventId, Hp, LifeState, LinkedEntity, NumericError,
    PresenceState, Scalar, SourceDefinitionId, Speed, StatValue, UnitDefinitionId, UnitId,
    UnitLevel,
    battle::spec::{FormationIndex, ResolvedBuildBonuses, TeamSide},
    formula::model::CombatElement,
    formula::toughness::EnemyRank,
    modifier::model::ActiveModifier,
};

#[derive(Clone)]
pub(super) struct SelectorUnitSnapshot {
    pub(super) form: UnitDefinitionId,
    pub(super) side: TeamSide,
    pub(super) formation: FormationIndex,
    pub(super) life: LifeState,
    pub(super) presence: PresenceState,
    pub(super) current_hp: Hp,
    pub(super) maximum_hp: Hp,
    pub(super) base_attack: StatValue,
    pub(super) base_defense: StatValue,
    pub(super) base_speed: Speed,
    pub(super) build_bonuses: ResolvedBuildBonuses,
    pub(super) level: UnitLevel,
    pub(super) gauge: Option<ActionGauge>,
    pub(super) shield: Scalar,
    pub(super) weaknesses: Box<[CombatElement]>,
    pub(super) rank: EnemyRank,
}

#[derive(Clone)]
pub(super) struct SelectorEffectSnapshot {
    pub(super) definition: EffectDefinitionId,
    pub(super) tags: Box<[SourceDefinitionId]>,
}

#[derive(Clone)]
pub(super) struct RuleSelectorSnapshot {
    pub(super) units: BTreeMap<UnitId, SelectorUnitSnapshot>,
    pub(super) effects: BTreeMap<UnitId, Box<[SelectorEffectSnapshot]>>,
    pub(super) owners: BTreeMap<UnitId, UnitId>,
    pub(super) modifiers: Box<[ActiveModifier]>,
}

impl RuleSelectorSnapshot {
    pub(super) fn capture(state: &BattleState) -> Self {
        let units = state
            .units
            .iter_by_id()
            .map(|unit| {
                (
                    unit.id,
                    SelectorUnitSnapshot {
                        form: unit.form,
                        side: unit.side,
                        formation: unit.formation,
                        life: unit.life,
                        presence: unit.presence,
                        current_hp: unit.current_hp,
                        maximum_hp: unit.maximum_hp,
                        base_attack: unit.base_attack,
                        base_defense: unit.base_defense,
                        base_speed: unit.base_speed,
                        build_bonuses: unit.build_bonuses,
                        level: unit.level,
                        gauge: state
                            .actors
                            .id_for_owner(unit.id)
                            .and_then(|actor| state.actors.get(actor))
                            .map(|actor| actor.gauge),
                        shield: state
                            .shields
                            .effective_remaining(unit.id)
                            .ok()
                            .and_then(|value| Scalar::checked_from_integer(value.get()).ok())
                            .unwrap_or(Scalar::ZERO),
                        weaknesses: unit.weaknesses.clone().into_boxed_slice(),
                        rank: unit.rank,
                    },
                )
            })
            .collect();
        let mut effects = BTreeMap::<UnitId, Vec<SelectorEffectSnapshot>>::new();
        for effect in state.effects.iter_by_id() {
            effects
                .entry(effect.target)
                .or_default()
                .push(SelectorEffectSnapshot {
                    definition: effect.definition,
                    tags: effect.tags.clone(),
                });
        }
        let effects = effects
            .into_iter()
            .map(|(unit, effects)| (unit, effects.into_boxed_slice()))
            .collect();
        let owners = state
            .links
            .canonical_entries()
            .iter()
            .filter_map(|link| {
                if !link.active {
                    return None;
                }
                match link.entity {
                    LinkedEntity::Unit(unit) => Some((unit, link.owner)),
                    LinkedEntity::TimelineActor(_) => None,
                }
            })
            .collect();
        Self {
            units,
            effects,
            owners,
            modifiers: state
                .modifiers
                .iter_by_id()
                .cloned()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    pub(super) fn stat_bases(&self) -> Result<BTreeMap<(UnitId, StatKind), Scalar>, NumericError> {
        let mut bases = BTreeMap::new();
        for (id, unit) in &self.units {
            bases.insert(
                (*id, HpStat),
                Scalar::checked_from_integer(unit.maximum_hp.get())?,
            );
            bases.insert((*id, Atk), Scalar::from_scaled(unit.base_attack.scaled()));
            bases.insert((*id, Def), Scalar::from_scaled(unit.base_defense.scaled()));
            bases.insert((*id, Spd), Scalar::from_scaled(unit.base_speed.scaled()));
            let [
                critical_rate,
                critical_damage,
                break_effect,
                energy_regeneration,
                outgoing,
            ] = unit.build_bonuses.secondary();
            bases.insert(
                (*id, CritRate),
                Scalar::from_scaled(if unit.side == TeamSide::Player {
                    50_000
                } else {
                    0
                })
                .checked_add(critical_rate)?,
            );
            bases.insert(
                (*id, CritDamage),
                Scalar::from_scaled(if unit.side == TeamSide::Player {
                    500_000
                } else {
                    0
                })
                .checked_add(critical_damage)?,
            );
            bases.insert((*id, BreakEffect), break_effect);
            bases.insert((*id, OutgoingHealing), outgoing);
            bases.insert((*id, ToughnessDamage), Scalar::ZERO);
            bases.insert(
                (*id, EnergyRegenerationRate),
                Scalar::ONE.checked_add(energy_regeneration)?,
            );
            for (stat, value) in [
                PhysicalDamageBoost,
                FireDamageBoost,
                IceDamageBoost,
                LightningDamageBoost,
                WindDamageBoost,
                QuantumDamageBoost,
                ImaginaryDamageBoost,
            ]
            .into_iter()
            .zip(unit.build_bonuses.element_damage_boosts())
            {
                bases.insert((*id, stat), value);
            }
            bases.insert((*id, ToughnessRecovery), Scalar::ONE);
            if let Some(value) = attacker_level_multiplier(unit.level) {
                bases.insert((*id, BreakBaseDamage), value);
            }
            bases.insert((*id, DotDurationAddition), Scalar::ZERO);
        }
        Ok(bases)
    }

    pub(super) fn shield_values(&self) -> BTreeMap<UnitId, Scalar> {
        self.units
            .iter()
            .map(|(id, unit)| (*id, unit.shield))
            .collect()
    }
}

impl transaction::Transaction<'_> {
    pub(super) fn selector_snapshot(
        &self,
        reference: RuleSelectorReference,
        event: EventId,
        action: Option<ActionId>,
    ) -> Option<Arc<RuleSelectorSnapshot>> {
        match reference {
            RuleSelectorReference::CurrentState => None,
            RuleSelectorReference::EventSnapshot => {
                self.selector_event_snapshots.get(&event).cloned()
            }
            RuleSelectorReference::ActionSnapshot => action
                .and_then(|action| self.selector_action_snapshots.get(&action))
                .cloned(),
        }
    }

    pub(super) fn emit(&mut self, cause: Cause, kind: BattleEventKind) -> EventId {
        let id = self.allocate_event();
        if self.capture_selector_snapshots {
            let snapshot = Arc::new(RuleSelectorSnapshot::capture(self.state));
            self.selector_event_snapshots.insert(id, snapshot.clone());
            if let Some(action) = cause.action() {
                self.selector_action_snapshots
                    .entry(action)
                    .or_insert(snapshot);
            }
        }
        self.events.push(BattleEvent::new(id, cause, kind));
        self.journal.event(id);
        id
    }
}
