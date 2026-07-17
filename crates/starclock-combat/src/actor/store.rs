use crate::{
    battle::spec::{CombatantSpecDigest, FormationIndex, ParticipantSource, TeamSide, UnitLevel},
    id::{
        AbilityId, ModifierDefinitionId, RuleBundleId, SpawnSequence, TimelineActorId,
        UnitDefinitionId, UnitId,
    },
    numeric::domain::{ActionGauge, Energy, Hp, Speed},
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
            .find(|actor| actor.owner == owner)
            .map(|actor| actor.id)
    }

    pub(crate) fn canonical_slots(&self) -> &[Option<TimelineActorState>] {
        &self.slots
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TeamState {
    pub(crate) side: TeamSide,
    pub(crate) skill_points: u16,
    pub(crate) maximum_skill_points: u16,
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
