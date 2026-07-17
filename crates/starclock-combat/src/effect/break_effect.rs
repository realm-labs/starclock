use crate::{
    EffectInstanceId, OperationId, SourceDefinitionId, UnitId,
    formula::toughness::{BaseBreakEffect, BreakDamageDefinition},
};

/// Dedicated core state for the seven base Break effects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BreakEffectState {
    pub(crate) id: EffectInstanceId,
    pub(crate) owner: UnitId,
    pub(crate) applier: UnitId,
    pub(crate) source_operation: OperationId,
    pub(crate) source_definition: SourceDefinitionId,
    pub(crate) plan: BaseBreakEffect,
    pub(crate) damage: BreakDamageDefinition,
    pub(crate) remaining_turns: u8,
    pub(crate) stacks: u8,
    pub(crate) speed_before: Option<crate::Speed>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct BreakEffectStore {
    entries: Vec<BreakEffectState>,
}

impl BreakEffectStore {
    pub(crate) fn insert(&mut self, state: BreakEffectState) {
        assert_eq!(state.id.get(), self.entries.len() as u64 + 1);
        self.entries.push(state);
    }
    pub(crate) fn iter_by_id(&self) -> impl Iterator<Item = &BreakEffectState> {
        self.entries
            .iter()
            .filter(|effect| effect.remaining_turns > 0)
    }
    pub(crate) fn canonical_entries(&self) -> &[BreakEffectState] {
        &self.entries
    }
    pub(crate) fn active_for(&self, owner: UnitId) -> Vec<BreakEffectState> {
        self.entries
            .iter()
            .copied()
            .filter(|effect| effect.owner == owner && effect.remaining_turns > 0)
            .collect()
    }
    pub(crate) fn get_mut(&mut self, id: EffectInstanceId) -> Option<&mut BreakEffectState> {
        self.entries.iter_mut().find(|effect| effect.id == id)
    }
}
