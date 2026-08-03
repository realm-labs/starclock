//! Compact immutable battlefield projections for authored selector reference points.

use crate::battle::state::BattleState;
use crate::catalog::selector::RuleSelectorReference;
use crate::event::cause::Cause;
use crate::event::model::BattleEvent;
use crate::event::model::BattleEventKind;
use crate::formula::toughness::attacker_level_multiplier;
use crate::modifier::model::StatKind;
use crate::modifier::model::StatKind::{
    Atk, BreakBaseDamage, Def, DotDurationAddition, EnergyRegenerationRate, Hp as HpStat, Spd,
    ToughnessDamage, ToughnessRecovery,
};
use std::{collections::BTreeMap, sync::Arc};

use crate::{
    ActionGauge, ActionId, EffectDefinitionId, EventId, Hp, LifeState, LinkedEntity, NumericError,
    PresenceState, Scalar, SourceDefinitionId, Speed, StatValue, UnitId, UnitLevel,
    battle::spec::{FormationIndex, TeamSide},
    formula::model::CombatElement,
    modifier::model::ActiveModifier,
};

#[derive(Clone)]
pub(super) struct SelectorUnitSnapshot {
    pub(super) side: TeamSide,
    pub(super) formation: FormationIndex,
    pub(super) life: LifeState,
    pub(super) presence: PresenceState,
    pub(super) current_hp: Hp,
    pub(super) maximum_hp: Hp,
    pub(super) base_attack: StatValue,
    pub(super) base_defense: StatValue,
    pub(super) base_speed: Speed,
    pub(super) level: UnitLevel,
    pub(super) gauge: Option<ActionGauge>,
    pub(super) shield: Scalar,
    pub(super) weaknesses: Box<[CombatElement]>,
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
                        side: unit.side,
                        formation: unit.formation,
                        life: unit.life,
                        presence: unit.presence,
                        current_hp: unit.current_hp,
                        maximum_hp: unit.maximum_hp,
                        base_attack: unit.base_attack,
                        base_defense: unit.base_defense,
                        base_speed: unit.base_speed,
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
            bases.insert((*id, ToughnessDamage), Scalar::ZERO);
            bases.insert((*id, EnergyRegenerationRate), Scalar::ONE);
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

impl super::transaction::Transaction<'_> {
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
