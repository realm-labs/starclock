use crate::{
    battle::spec::{CombatantSpecDigest, FormationIndex, ParticipantSource, TeamSide, UnitLevel},
    id::{
        AbilityId, ModifierDefinitionId, RuleBundleId, SpawnSequence, TimelineActorId,
        UnitDefinitionId, UnitId,
    },
    numeric::domain::{ActionGauge, Energy, Hp, Speed},
};

use super::link::{
    LinkedEntity, LinkedEntityKind, OwnerLinkPolicy, TransformEndPolicy, WaveLinkPolicy,
};
use super::model::{LifeState, PresenceState};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnitState {
    pub(crate) id: UnitId,
    pub(crate) spawn: SpawnSequence,
    pub(crate) form: UnitDefinitionId,
    pub(crate) source: ParticipantSource,
    pub(crate) side: TeamSide,
    pub(crate) formation: FormationIndex,
    pub(crate) entry_wave: u16,
    pub(crate) level: UnitLevel,
    pub(crate) life: LifeState,
    pub(crate) presence: PresenceState,
    pub(crate) current_hp: Hp,
    pub(crate) maximum_hp: Hp,
    pub(crate) current_energy: Energy,
    pub(crate) maximum_energy: Energy,
    pub(crate) rank: crate::formula::toughness::EnemyRank,
    pub(crate) weaknesses: Vec<crate::formula::model::CombatElement>,
    pub(crate) permanent_weaknesses: Box<[crate::formula::model::CombatElement]>,
    pub(crate) temporary_weaknesses: Vec<TemporaryWeaknessState>,
    pub(crate) toughness_layers: Vec<crate::toughness::state::ToughnessLayerState>,
    pub(crate) weakness_broken: bool,
    pub(crate) abilities: Box<[AbilityId]>,
    pub(crate) rule_bundles: Box<[RuleBundleId]>,
    pub(crate) modifiers: Box<[ModifierDefinitionId]>,
    pub(crate) digest: CombatantSpecDigest,
    pub(crate) transformation: Option<TransformationState>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransformationState {
    pub(crate) source_operation: crate::OperationId,
    pub(crate) original_form: UnitDefinitionId,
    pub(crate) original_abilities: Box<[AbilityId]>,
    pub(crate) original_presence: PresenceState,
    pub(crate) countdown_actor: Option<TimelineActorId>,
    pub(crate) defeat: TransformEndPolicy,
    pub(crate) wave: TransformEndPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TemporaryWeaknessState {
    pub(crate) element: crate::formula::model::CombatElement,
    pub(crate) applier: UnitId,
    pub(crate) source_operation: crate::OperationId,
    pub(crate) remaining_turns: u8,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct UnitStore {
    slots: Vec<Option<UnitState>>,
}

impl UnitStore {
    pub(crate) fn insert(&mut self, state: UnitState) {
        assert_eq!(
            state.id.get(),
            self.slots.len() as u64 + 1,
            "unit IDs are allocated monotonically by the central sequence state"
        );
        self.slots.push(Some(state));
    }

    pub(crate) fn iter_by_id(&self) -> impl Iterator<Item = &UnitState> {
        self.slots.iter().filter_map(Option::as_ref)
    }

    pub(crate) fn get(&self, id: UnitId) -> Option<&UnitState> {
        let index = usize::try_from(id.get().checked_sub(1)?).ok()?;
        self.slots.get(index)?.as_ref()
    }

    pub(crate) fn get_mut(&mut self, id: UnitId) -> Option<&mut UnitState> {
        let index = usize::try_from(id.get().checked_sub(1)?).ok()?;
        self.slots.get_mut(index)?.as_mut()
    }

    pub(crate) fn canonical_slots(&self) -> &[Option<UnitState>] {
        &self.slots
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TimelineActorState {
    pub(crate) id: TimelineActorId,
    pub(crate) owner: UnitId,
    pub(crate) unit: Option<UnitId>,
    pub(crate) kind: Option<LinkedEntityKind>,
    pub(crate) automatic_ability: Option<AbilityId>,
    pub(crate) active: bool,
    pub(crate) gauge: ActionGauge,
    pub(crate) speed: Speed,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TimelineActorStore {
    slots: Vec<Option<TimelineActorState>>,
}

impl TimelineActorStore {
    pub(crate) fn insert(&mut self, state: TimelineActorState) {
        assert_eq!(
            state.id.get(),
            self.slots.len() as u64 + 1,
            "timeline actor IDs are allocated monotonically by the central sequence state"
        );
        self.slots.push(Some(state));
    }

    pub(crate) fn iter_by_id(&self) -> impl Iterator<Item = &TimelineActorState> {
        self.slots.iter().filter_map(Option::as_ref)
    }

    pub(crate) fn get_mut(&mut self, id: TimelineActorId) -> Option<&mut TimelineActorState> {
        let index = usize::try_from(id.get().checked_sub(1)?).ok()?;
        self.slots.get_mut(index)?.as_mut()
    }

    pub(crate) fn get(&self, id: TimelineActorId) -> Option<&TimelineActorState> {
        let index = usize::try_from(id.get().checked_sub(1)?).ok()?;
        self.slots.get(index)?.as_ref()
    }

    pub(crate) fn id_for_owner(&self, owner: UnitId) -> Option<TimelineActorId> {
        self.iter_by_id()
            .find(|actor| actor.active && actor.unit == Some(owner))
            .map(|actor| actor.id)
    }

    pub(crate) fn any_id_for_unit(&self, unit: UnitId) -> Option<TimelineActorId> {
        self.iter_by_id()
            .find(|actor| actor.unit == Some(unit))
            .map(|actor| actor.id)
    }

    pub(crate) fn canonical_slots(&self) -> &[Option<TimelineActorState>] {
        &self.slots
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LinkState {
    pub(crate) owner: UnitId,
    pub(crate) entity: LinkedEntity,
    pub(crate) kind: LinkedEntityKind,
    pub(crate) owner_defeat: OwnerLinkPolicy,
    pub(crate) owner_departure: OwnerLinkPolicy,
    pub(crate) wave: WaveLinkPolicy,
    pub(crate) active: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct LinkStore {
    entries: Vec<LinkState>,
}

impl LinkStore {
    pub(crate) fn insert(&mut self, state: LinkState) -> bool {
        if self
            .entries
            .iter()
            .any(|entry| entry.entity == state.entity)
        {
            return false;
        }
        self.entries.push(state);
        true
    }

    pub(crate) fn get_mut(&mut self, entity: LinkedEntity) -> Option<&mut LinkState> {
        self.entries.iter_mut().find(|entry| entry.entity == entity)
    }

    pub(crate) fn active_for_owner(&self, owner: UnitId) -> impl Iterator<Item = LinkState> + '_ {
        self.entries
            .iter()
            .copied()
            .filter(move |entry| entry.active && entry.owner == owner)
    }

    pub(crate) fn for_unit(&self, unit: UnitId) -> Option<LinkState> {
        self.entries
            .iter()
            .copied()
            .find(|entry| entry.entity == LinkedEntity::Unit(unit))
    }

    pub(crate) fn canonical_entries(&self) -> &[LinkState] {
        &self.entries
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FormationEntry {
    pub(crate) side: TeamSide,
    pub(crate) index: FormationIndex,
    pub(crate) unit: UnitId,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FormationState {
    entries: Vec<FormationEntry>,
}

impl FormationState {
    pub(crate) fn push(&mut self, entry: FormationEntry) {
        self.entries.push(entry);
    }

    pub(crate) fn on_side(&self, side: TeamSide) -> impl Iterator<Item = FormationEntry> + '_ {
        self.entries
            .iter()
            .copied()
            .filter(move |entry| entry.side == side)
    }

    pub(crate) fn canonical_entries(&self) -> &[FormationEntry] {
        &self.entries
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TeamState {
    pub(crate) side: TeamSide,
    pub(crate) skill_points: u16,
    pub(crate) maximum_skill_points: u16,
    pub(crate) keyed_resources: Box<[KeyedTeamResourceState]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KeyedTeamResourceState {
    pub(crate) id: crate::SourceDefinitionId,
    pub(crate) initial: u16,
    pub(crate) current: u16,
    pub(crate) maximum: u16,
    pub(crate) wave: crate::battle::spec::TeamResourceWavePolicy,
}

impl TeamState {
    pub(crate) fn keyed(&self, id: crate::SourceDefinitionId) -> Option<&KeyedTeamResourceState> {
        self.keyed_resources
            .binary_search_by_key(&id, |entry| entry.id)
            .ok()
            .map(|index| &self.keyed_resources[index])
    }
    pub(crate) fn keyed_mut(
        &mut self,
        id: crate::SourceDefinitionId,
    ) -> Option<&mut KeyedTeamResourceState> {
        self.keyed_resources
            .binary_search_by_key(&id, |entry| entry.id)
            .ok()
            .map(|index| &mut self.keyed_resources[index])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TeamStateStore {
    teams: [TeamState; 2],
}

impl TeamStateStore {
    pub(crate) const fn new(player: TeamState, enemy: TeamState) -> Self {
        Self {
            teams: [player, enemy],
        }
    }

    pub(crate) fn get(&self, side: TeamSide) -> &TeamState {
        &self.teams[side.canonical_index()]
    }

    pub(crate) fn get_mut(&mut self, side: TeamSide) -> &mut TeamState {
        &mut self.teams[side.canonical_index()]
    }
}
