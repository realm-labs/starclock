use core::fmt::Debug;

use super::definition::{
    AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
    RuleBundle, RuleDefinition, SelectorDefinition, UnitDefinition,
};
use super::encounter::AiGraphDefinition;
use crate::{
    AbilityId, EffectDefinitionId, EncounterId, EnemyDefinitionId, ProgramId, RuleBundleId, RuleId,
    SelectorId, UnitDefinitionId,
};

pub(super) trait Identified<I> {
    fn id(&self) -> I;
}

macro_rules! identified {
    ($definition:ty, $id:ty) => {
        impl Identified<$id> for $definition {
            fn id(&self) -> $id {
                self.id()
            }
        }
    };
}

identified!(UnitDefinition, UnitDefinitionId);
identified!(AbilityDefinition, AbilityId);
identified!(EffectDefinition, EffectDefinitionId);
identified!(RuleDefinition, RuleId);
identified!(ProgramDefinition, ProgramId);
identified!(SelectorDefinition, SelectorId);
identified!(RuleBundle, RuleBundleId);
identified!(EnemyDefinition, EnemyDefinitionId);
identified!(EncounterDefinition, EncounterId);
identified!(AiGraphDefinition, crate::AiGraphId);

#[derive(Debug)]
pub(super) struct DuplicateId<I>(pub I);

#[derive(Debug)]
pub(super) struct DefinitionTable<I, D> {
    rows: Box<[D]>,
    marker: core::marker::PhantomData<I>,
}

impl<I: Copy + Debug + Ord, D: Identified<I>> DefinitionTable<I, D> {
    pub(super) fn from_unsorted(mut rows: Vec<D>) -> Result<Self, DuplicateId<I>> {
        rows.sort_unstable_by_key(Identified::id);
        if let Some(pair) = rows.windows(2).find(|pair| pair[0].id() == pair[1].id()) {
            return Err(DuplicateId(pair[0].id()));
        }
        Ok(Self {
            rows: rows.into_boxed_slice(),
            marker: core::marker::PhantomData,
        })
    }

    pub(super) fn get(&self, id: I) -> Option<&D> {
        self.rows
            .binary_search_by_key(&id, Identified::id)
            .ok()
            .map(|index| &self.rows[index])
    }

    pub(super) fn ids(&self) -> impl ExactSizeIterator<Item = I> + '_ {
        self.rows.iter().map(Identified::id)
    }

    pub(super) const fn len(&self) -> usize {
        self.rows.len()
    }
}
