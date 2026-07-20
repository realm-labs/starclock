use crate::{
    catalog::{CatalogDigest, CatalogRevision},
    id::{
        ActionId, CommandId, DecisionId, EffectInstanceId, EncounterId, EventId, HitId,
        OperationId, PhaseId, ShieldInstanceId, SpawnSequence, TimelineActorId, UnitId,
        WaveInstanceId,
    },
    rng::{engine::DeterministicRng, types::RngSeed},
};

use crate::{
    actor::store::{FormationState, TeamStateStore, TimelineActorStore, UnitStore},
    command::model::DecisionPoint,
};

use super::{
    fault::BattleFault,
    model::BattlePhase,
    spec::{BattleSeed, BattleSpecDigest, ConcedePolicy},
};

#[derive(Clone, Debug)]
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
    pub(crate) number: u16,
    pub(crate) total_waves: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SequenceState {
    next_unit: u64,
    next_actor: u64,
    next_spawn: u64,
    next_wave: u64,
    next_decision: u64,
    next_command: u64,
    next_event: u64,
    next_action: u64,
    next_phase: u64,
    next_hit: u64,
    next_operation: u64,
    next_shield: u64,
    next_effect: u64,
    next_rule: u64,
    next_modifier: u64,
}

impl SequenceState {
    pub(crate) const fn new() -> Self {
        Self {
            next_unit: 1,
            next_actor: 1,
            next_spawn: 1,
            next_wave: 1,
            next_decision: 1,
            next_command: 1,
            next_event: 1,
            next_action: 1,
            next_phase: 1,
            next_hit: 1,
            next_operation: 1,
            next_shield: 1,
            next_effect: 1,
            next_rule: 1,
            next_modifier: 1,
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

    pub(crate) fn try_wave(&mut self) -> Option<WaveInstanceId> {
        try_allocate(&mut self.next_wave, WaveInstanceId::new)
    }

    pub(crate) fn decision(&mut self) -> DecisionId {
        allocate(&mut self.next_decision, DecisionId::new)
    }

    pub(crate) fn try_decision(&mut self) -> Option<DecisionId> {
        try_allocate(&mut self.next_decision, DecisionId::new)
    }

    pub(crate) fn try_command(&mut self) -> Option<CommandId> {
        try_allocate(&mut self.next_command, CommandId::new)
    }

    pub(crate) fn try_event(&mut self) -> Option<EventId> {
        try_allocate(&mut self.next_event, EventId::new)
    }

    pub(crate) fn try_action(&mut self) -> Option<ActionId> {
        try_allocate(&mut self.next_action, ActionId::new)
    }

    pub(crate) fn try_phase(&mut self) -> Option<PhaseId> {
        try_allocate(&mut self.next_phase, PhaseId::new)
    }

    pub(crate) fn try_hit(&mut self) -> Option<HitId> {
        try_allocate(&mut self.next_hit, HitId::new)
    }

    pub(crate) fn try_operation(&mut self) -> Option<OperationId> {
        try_allocate(&mut self.next_operation, OperationId::new)
    }

    pub(crate) fn try_shield(&mut self) -> Option<ShieldInstanceId> {
        try_allocate(&mut self.next_shield, ShieldInstanceId::new)
    }

    pub(crate) fn try_effect(&mut self) -> Option<EffectInstanceId> {
        try_allocate(&mut self.next_effect, EffectInstanceId::new)
    }

    pub(crate) fn rule(&mut self) -> crate::RuleInstanceId {
        allocate(&mut self.next_rule, crate::RuleInstanceId::new)
    }

    pub(crate) fn modifier(&mut self) -> crate::ModifierInstanceId {
        allocate(&mut self.next_modifier, crate::ModifierInstanceId::new)
    }

    pub(crate) const fn canonical_next_values(&self) -> [u64; 15] {
        [
            self.next_unit,
            self.next_actor,
            self.next_spawn,
            self.next_wave,
            self.next_decision,
            self.next_command,
            self.next_event,
            self.next_action,
            self.next_phase,
            self.next_hit,
            self.next_operation,
            self.next_shield,
            self.next_effect,
            self.next_rule,
            self.next_modifier,
        ]
    }
}

fn allocate<I>(next: &mut u64, constructor: impl FnOnce(u64) -> Option<I>) -> I {
    let raw = *next;
    *next = next
        .checked_add(1)
        .expect("validated initial battle bounds cannot exhaust u64 sequences");
    constructor(raw).expect("sequence starts at one and never wraps")
}

fn try_allocate<I>(next: &mut u64, constructor: impl FnOnce(u64) -> Option<I>) -> Option<I> {
    let raw = *next;
    let advanced = raw.checked_add(1)?;
    let value = constructor(raw)?;
    *next = advanced;
    Some(value)
}

#[derive(Debug)]
pub(crate) struct BattleState {
    pub(crate) identity: BattleIdentity,
    pub(crate) phase: BattlePhase,
    pub(crate) fault: Option<BattleFault>,
    pub(crate) decision: Option<DecisionPoint>,
    pub(crate) units: UnitStore,
    pub(crate) actors: TimelineActorStore,
    pub(crate) links: crate::actor::store::LinkStore,
    pub(crate) formations: FormationState,
    pub(crate) teams: TeamStateStore,
    pub(crate) shields: crate::effect::shield::ShieldStore,
    pub(crate) break_effects: crate::effect::break_effect::BreakEffectStore,
    pub(crate) effects: crate::effect::state::EffectStore,
    pub(crate) rules: crate::rule::state::RuleStateStore,
    pub(crate) modifiers: crate::modifier::state::ModifierStore,
    pub(crate) encounter: EncounterState,
    pub(crate) timeline: crate::timeline::state::TimelineState,
    pub(crate) concede: ConcedePolicy,
    pub(crate) rng: DeterministicRng,
    pub(crate) sequences: SequenceState,
    pub(crate) committed_revision: u64,
}

impl BattleState {
    pub(crate) fn rng_from_seed(seed: BattleSeed) -> DeterministicRng {
        DeterministicRng::from_seed(RngSeed::new(seed.bytes()))
    }

    pub(crate) fn semantic_clone(&self) -> Self {
        let mut cloned = Self {
            identity: self.identity.clone(),
            phase: self.phase,
            fault: self.fault,
            decision: self.decision.clone(),
            units: self.units.clone(),
            actors: self.actors.clone(),
            links: self.links.clone(),
            formations: self.formations.clone(),
            teams: self.teams.clone(),
            shields: self.shields.clone(),
            break_effects: self.break_effects.clone(),
            effects: self.effects.clone(),
            rules: self.rules.clone(),
            modifiers: self.modifiers.clone(),
            encounter: self.encounter,
            timeline: self.timeline.clone(),
            concede: self.concede,
            rng: Self::rng_from_seed(self.identity.seed),
            sequences: self.sequences,
            committed_revision: self.committed_revision,
        };
        cloned.rng.clone_from_authoritative(&self.rng);
        cloned
    }

    pub(crate) fn clone_from_semantics(&mut self, source: &Self) {
        self.identity.clone_from(&source.identity);
        self.phase = source.phase;
        self.fault = source.fault;
        self.decision.clone_from(&source.decision);
        self.units.clone_from(&source.units);
        self.actors.clone_from(&source.actors);
        self.links.clone_from(&source.links);
        self.formations.clone_from(&source.formations);
        self.teams.clone_from(&source.teams);
        self.shields.clone_from(&source.shields);
        self.break_effects.clone_from(&source.break_effects);
        self.effects.clone_from(&source.effects);
        self.rules.clone_from(&source.rules);
        self.modifiers.clone_from(&source.modifiers);
        self.encounter = source.encounter;
        self.timeline.clone_from(&source.timeline);
        self.concede = source.concede;
        self.rng.clone_from_authoritative(&source.rng);
        self.sequences = source.sequences;
        self.committed_revision = source.committed_revision;
    }
}
