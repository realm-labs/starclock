//! Exact six-rank Eidolon definitions and canonical rank ordering.

use starclock_combat::{UnitDefinitionId, rule::model::RuleSource};

use crate::{id::EidolonDefinitionId, patch::BuildPatch, spec::EidolonLevel};

/// One authored Eidolon rank and its explicitly ordered patches.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EidolonDefinition {
    id: EidolonDefinitionId,
    source: RuleSource,
    rank: EidolonLevel,
    patches: Box<[BuildPatch]>,
}

impl EidolonDefinition {
    #[must_use]
    pub fn new(
        id: EidolonDefinitionId,
        source: RuleSource,
        rank: EidolonLevel,
        patches: Vec<BuildPatch>,
    ) -> Self {
        Self {
            id,
            source,
            rank,
            patches: patches.into_boxed_slice(),
        }
    }
    #[must_use]
    pub const fn id(&self) -> EidolonDefinitionId {
        self.id
    }
    #[must_use]
    pub const fn source(&self) -> &RuleSource {
        &self.source
    }
    #[must_use]
    pub const fn rank(&self) -> EidolonLevel {
        self.rank
    }
    #[must_use]
    pub fn patches(&self) -> &[BuildPatch] {
        &self.patches
    }
}

/// Complete E1-E6 definition set for one combat form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EidolonSetDefinition {
    form: UnitDefinitionId,
    ranks: Box<[EidolonDefinition]>,
}

impl EidolonSetDefinition {
    #[must_use]
    pub fn new(form: UnitDefinitionId, ranks: Vec<EidolonDefinition>) -> Self {
        Self {
            form,
            ranks: ranks.into_boxed_slice(),
        }
    }
    pub(crate) fn canonicalize(&mut self) -> Result<(), EidolonSetError> {
        self.ranks.sort_unstable_by_key(EidolonDefinition::rank);
        if self.ranks.len() != usize::from(EidolonLevel::MAX) {
            return Err(EidolonSetError::IncompleteRankSet);
        }
        for (index, definition) in self.ranks.iter().enumerate() {
            let expected = u8::try_from(index + 1).expect("six Eidolon ranks fit u8");
            if definition.rank().get() != expected {
                return Err(EidolonSetError::IncompleteRankSet);
            }
        }
        let mut ids = self
            .ranks
            .iter()
            .map(EidolonDefinition::id)
            .collect::<Vec<_>>();
        ids.sort_unstable();
        if ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(EidolonSetError::DuplicateDefinition);
        }
        Ok(())
    }
    #[must_use]
    pub const fn form(&self) -> UnitDefinitionId {
        self.form
    }
    #[must_use]
    pub fn ranks(&self) -> &[EidolonDefinition] {
        &self.ranks
    }
    #[must_use]
    pub fn rank(&self, rank: EidolonLevel) -> Option<&EidolonDefinition> {
        if rank == EidolonLevel::E0 {
            return None;
        }
        self.ranks
            .get(usize::from(rank.get() - 1))
            .filter(|definition| definition.rank() == rank)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EidolonSetError {
    IncompleteRankSet,
    DuplicateDefinition,
}
