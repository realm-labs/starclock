use crate::{
    catalog::{CatalogDigest, CatalogRevision},
    id::{DecisionId, EncounterId, SpawnSequence, TimelineActorId, UnitId, WaveInstanceId},
    rng::{engine::DeterministicRng, types::RngSeed},
};

use crate::{
    actor::store::{FormationState, TeamStateStore, TimelineActorStore, UnitStore},
    command::model::DecisionPoint,
};

use super::{
    model::BattlePhase,
    spec::{BattleSeed, BattleSpecDigest, ConcedePolicy},
};

#[derive(Debug)]
pub(crate) struct BattleIdentity {
    pub(crate) catalog_revision: CatalogRevision,
    pub(crate) catalog_digest: CatalogDigest,
    pub(crate) rules_revision: Box<str>,
    pub(crate) spec_digest: BattleSpecDigest,
    pub(crate) seed: BattleSeed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EncounterState {
    pub(crate) definition: EncounterId,
    pub(crate) wave: WaveInstanceId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SequenceState {
    next_unit: u64,
    next_actor: u64,
    next_spawn: u64,
    next_wave: u64,
    next_decision: u64,
}

impl SequenceState {
    pub(crate) const fn new() -> Self {
        Self {
            next_unit: 1,
            next_actor: 1,
            next_spawn: 1,
            next_wave: 1,
            next_decision: 1,
        }
    }

    pub(crate) fn unit(&mut self) -> UnitId {
        allocate(&mut self.next_unit, UnitId::new)
    }

    pub(crate) fn actor(&mut self) -> TimelineActorId {
        allocate(&mut self.next_actor, TimelineActorId::new)
    }

    pub(crate) fn spawn(&mut self) -> SpawnSequence {
        allocate(&mut self.next_spawn, SpawnSequence::new)
    }

    pub(crate) fn wave(&mut self) -> WaveInstanceId {
        allocate(&mut self.next_wave, WaveInstanceId::new)
    }

    pub(crate) fn decision(&mut self) -> DecisionId {
        allocate(&mut self.next_decision, DecisionId::new)
    }
}

fn allocate<I>(next: &mut u64, constructor: impl FnOnce(u64) -> Option<I>) -> I {
    let raw = *next;
    *next = next
        .checked_add(1)
        .expect("validated initial battle bounds cannot exhaust u64 sequences");
    constructor(raw).expect("sequence starts at one and never wraps")
}

#[derive(Debug)]
pub(crate) struct BattleState {
    pub(crate) identity: BattleIdentity,
    pub(crate) phase: BattlePhase,
    pub(crate) decision: Option<DecisionPoint>,
    pub(crate) units: UnitStore,
    pub(crate) actors: TimelineActorStore,
    pub(crate) formations: FormationState,
    pub(crate) teams: TeamStateStore,
    pub(crate) encounter: EncounterState,
    pub(crate) concede: ConcedePolicy,
    pub(crate) rng: DeterministicRng,
    pub(crate) sequences: SequenceState,
    pub(crate) committed_revision: u64,
}

impl BattleState {
    pub(crate) fn rng_from_seed(seed: BattleSeed) -> DeterministicRng {
        DeterministicRng::from_seed(RngSeed::new(seed.bytes()))
    }
}
