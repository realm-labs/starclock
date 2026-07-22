use std::{collections::BTreeMap, sync::Arc};

use sha2::{Digest, Sha256};
use starclock_combat::{BattleSeed, BattleSpec, Energy, Hp, LifeState, PresenceState};

use crate::{
    ActivityDefinitionIdentity, ActivityEdgeCondition, ActivityEdgeId, ActivityFault,
    ActivityGraphDefinition, ActivityInstanceId, ActivityRngLabel, ActivityRngStreams,
    ActivitySlotId, ActivityStateHash, ActivityTerminalOutcome, ActivityValue, BattleOutcome,
    BattleResult, BattleResultConfiguration, BattleResultDigest, BattleResultIdentity,
    BattleResultProjection, BattleSettlementContractDigest, MetricValue, MetricValueKind, NodeId,
    ParticipantBattleState, ParticipantId, ProjectionField, ScopeIdentity, TerminalOutcome,
    codec::{ActivityStateEncoder, CanonicalWriter},
};

pub const MAX_COMPLETED_ACTIVITY_BATTLES: usize = 4_096;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum HpCarryPolicy {
    CarryExact = 0,
    CarryClamped = 1,
    RestoreFull = 2,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum EnergyCarryPolicy {
    CarryExact = 0,
    CarryClamped = 1,
    ResetZero = 2,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum LifeCarryPolicy {
    CarryExact = 0,
    DefeatOnZero = 1,
    RestoreAlive = 2,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PresenceCarryPolicy {
    CarryExact = 0,
    DepartIfDefeated = 1,
    RestorePresent = 2,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum MetricSettlementPolicy {
    Replace = 0,
    Sum = 1,
    Minimum = 2,
    Maximum = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityParticipantCarryDefinition {
    participant: ParticipantId,
    hp: HpCarryPolicy,
    energy: EnergyCarryPolicy,
    life: LifeCarryPolicy,
    presence: PresenceCarryPolicy,
}

impl ActivityParticipantCarryDefinition {
    #[must_use]
    pub const fn new(
        participant: ParticipantId,
        hp: HpCarryPolicy,
        energy: EnergyCarryPolicy,
        life: LifeCarryPolicy,
        presence: PresenceCarryPolicy,
    ) -> Self {
        Self {
            participant,
            hp,
            energy,
            life,
            presence,
        }
    }

    #[must_use]
    pub const fn participant(self) -> ParticipantId {
        self.participant
    }
    #[must_use]
    pub const fn hp(self) -> HpCarryPolicy {
        self.hp
    }
    #[must_use]
    pub const fn energy(self) -> EnergyCarryPolicy {
        self.energy
    }
    #[must_use]
    pub const fn life(self) -> LifeCarryPolicy {
        self.life
    }
    #[must_use]
    pub const fn presence(self) -> PresenceCarryPolicy {
        self.presence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityMetricProjectionBinding {
    key: Box<str>,
    kind: MetricValueKind,
    slot: ActivitySlotId,
    policy: MetricSettlementPolicy,
}

impl ActivityMetricProjectionBinding {
    #[must_use]
    pub fn new(
        key: impl Into<Box<str>>,
        kind: MetricValueKind,
        slot: ActivitySlotId,
        policy: MetricSettlementPolicy,
    ) -> Option<Self> {
        let key = key.into();
        (!key.is_empty() && key.len() <= 120).then_some(Self {
            key,
            kind,
            slot,
            policy,
        })
    }

    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }
    #[must_use]
    pub const fn kind(&self) -> MetricValueKind {
        self.kind
    }
    #[must_use]
    pub const fn slot(&self) -> ActivitySlotId {
        self.slot
    }
    #[must_use]
    pub const fn policy(&self) -> MetricSettlementPolicy {
        self.policy
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityBattleResultContract {
    projection: Arc<BattleResultProjection>,
    carry: Box<[ActivityParticipantCarryDefinition]>,
    metrics: Box<[ActivityMetricProjectionBinding]>,
    digest: BattleSettlementContractDigest,
}

impl ActivityBattleResultContract {
    pub fn new(
        projection: Arc<BattleResultProjection>,
        mut carry: Vec<ActivityParticipantCarryDefinition>,
        mut metrics: Vec<ActivityMetricProjectionBinding>,
    ) -> Result<Self, ActivityBattleResultContractError> {
        carry.sort_by_key(|item| item.participant);
        metrics.sort_by(|left, right| left.key.cmp(&right.key));
        if carry
            .windows(2)
            .any(|pair| pair[0].participant == pair[1].participant)
        {
            return Err(ActivityBattleResultContractError::DuplicateParticipant);
        }
        if carry.iter().any(|item| {
            (item.hp == HpCarryPolicy::RestoreFull && item.life == LifeCarryPolicy::CarryExact)
                || (item.life == LifeCarryPolicy::RestoreAlive
                    && item.hp != HpCarryPolicy::RestoreFull)
        }) {
            return Err(ActivityBattleResultContractError::InvalidCarryPolicy);
        }
        if metrics.windows(2).any(|pair| pair[0].key == pair[1].key) {
            return Err(ActivityBattleResultContractError::DuplicateMetric);
        }
        let projected_participants = projection
            .fields()
            .iter()
            .filter_map(|field| match field {
                ProjectionField::ParticipantState(participant) => Some(*participant),
                _ => None,
            })
            .collect::<Vec<_>>();
        let projected_metrics = projection
            .fields()
            .iter()
            .filter_map(|field| match field {
                ProjectionField::Metric { key, kind } => Some((key.as_ref(), *kind)),
                _ => None,
            })
            .collect::<Vec<_>>();
        if projected_participants.len() != carry.len()
            || carry
                .iter()
                .any(|item| !projected_participants.contains(&item.participant))
        {
            return Err(ActivityBattleResultContractError::ParticipantProjectionMismatch);
        }
        if projected_metrics.len() != metrics.len()
            || metrics
                .iter()
                .any(|item| !projected_metrics.contains(&(item.key(), item.kind)))
        {
            return Err(ActivityBattleResultContractError::MetricProjectionMismatch);
        }
        let digest = contract_digest(&projection, &carry, &metrics);
        Ok(Self {
            projection,
            carry: carry.into_boxed_slice(),
            metrics: metrics.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub fn projection(&self) -> &BattleResultProjection {
        &self.projection
    }
    #[must_use]
    pub fn participant_carry(&self) -> &[ActivityParticipantCarryDefinition] {
        &self.carry
    }
    #[must_use]
    pub fn metrics(&self) -> &[ActivityMetricProjectionBinding] {
        &self.metrics
    }
    #[must_use]
    pub const fn digest(&self) -> BattleSettlementContractDigest {
        self.digest
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityBattleResultContractError {
    DuplicateParticipant,
    DuplicateMetric,
    InvalidCarryPolicy,
    ParticipantProjectionMismatch,
    MetricProjectionMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityParticipantCarryState {
    participant: ParticipantId,
    current_hp: Hp,
    maximum_hp: Hp,
    current_energy: Energy,
    maximum_energy: Energy,
    life: LifeState,
    presence: PresenceState,
}

impl ActivityParticipantCarryState {
    #[must_use]
    pub const fn participant(self) -> ParticipantId {
        self.participant
    }
    #[must_use]
    pub const fn current_hp(self) -> Hp {
        self.current_hp
    }
    #[must_use]
    pub const fn maximum_hp(self) -> Hp {
        self.maximum_hp
    }
    #[must_use]
    pub const fn current_energy(self) -> Energy {
        self.current_energy
    }
    #[must_use]
    pub const fn maximum_energy(self) -> Energy {
        self.maximum_energy
    }
    #[must_use]
    pub const fn life(self) -> LifeState {
        self.life
    }
    #[must_use]
    pub const fn presence(self) -> PresenceState {
        self.presence
    }

    fn encode(self, writer: &mut ActivityStateEncoder) {
        writer.u32(self.participant.get());
        writer.i64(self.current_hp.get());
        writer.i64(self.maximum_hp.get());
        writer.i64(self.current_energy.scaled());
        writer.i64(self.maximum_energy.scaled());
        writer.byte(self.life as u8);
        writer.byte(self.presence as u8);
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ActivityCarryLedger(BTreeMap<ParticipantId, ActivityParticipantCarryState>);

impl ActivityCarryLedger {
    pub(crate) fn encode(&self, writer: &mut ActivityStateEncoder) {
        writer.u32(self.0.len() as u32);
        for state in self.0.values().copied() {
            state.encode(writer);
        }
    }

    pub(crate) fn view(&self) -> Box<[ActivityParticipantCarryState]> {
        self.0
            .values()
            .copied()
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn insert(&mut self, state: ActivityParticipantCarryState) {
        self.0.insert(state.participant, state);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActivityAwaitingBattle {
    identity: BattleResultIdentity,
    contract: Arc<ActivityBattleResultContract>,
}

impl ActivityAwaitingBattle {
    pub(crate) fn encode(&self, writer: &mut ActivityStateEncoder) {
        encode_result_identity(writer, self.identity);
        writer.digest(self.contract.digest().bytes());
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityBattleHandoff {
    identity: BattleResultIdentity,
    battle_spec: BattleSpec,
    carry: Box<[ActivityParticipantCarryState]>,
    contract_digest: BattleSettlementContractDigest,
}

impl ActivityBattleHandoff {
    #[must_use]
    pub const fn identity(&self) -> BattleResultIdentity {
        self.identity
    }
    #[must_use]
    pub const fn battle_spec(&self) -> &BattleSpec {
        &self.battle_spec
    }
    #[must_use]
    pub fn participant_carry(&self) -> &[ActivityParticipantCarryState] {
        &self.carry
    }
    #[must_use]
    pub const fn contract_digest(&self) -> BattleSettlementContractDigest {
        self.contract_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityBattleStartRequest {
    expected_state_hash: ActivityStateHash,
    definition_identity: ActivityDefinitionIdentity,
    instance: ActivityInstanceId,
    contract: Arc<ActivityBattleResultContract>,
}

impl ActivityBattleStartRequest {
    #[must_use]
    pub fn new(
        expected_state_hash: ActivityStateHash,
        definition_identity: ActivityDefinitionIdentity,
        instance: ActivityInstanceId,
        contract: Arc<ActivityBattleResultContract>,
    ) -> Self {
        Self {
            expected_state_hash,
            definition_identity,
            instance,
            contract,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityBattleResultSubmission {
    expected_state_hash: ActivityStateHash,
    result: Box<BattleResult>,
}

impl ActivityBattleResultSubmission {
    #[must_use]
    pub fn new(expected_state_hash: ActivityStateHash, result: BattleResult) -> Self {
        Self {
            expected_state_hash,
            result: Box::new(result),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityBattleSettlement {
    outcome: BattleOutcome,
    traversed_edge: ActivityEdgeId,
    target: NodeId,
    result_digest: BattleResultDigest,
    terminal: Option<ActivityTerminalOutcome>,
    state_hash: ActivityStateHash,
}

impl ActivityBattleSettlement {
    #[must_use]
    pub const fn outcome(self) -> BattleOutcome {
        self.outcome
    }
    #[must_use]
    pub const fn traversed_edge(self) -> ActivityEdgeId {
        self.traversed_edge
    }
    #[must_use]
    pub const fn target(self) -> NodeId {
        self.target
    }
    #[must_use]
    pub const fn result_digest(self) -> BattleResultDigest {
        self.result_digest
    }
    #[must_use]
    pub const fn terminal(self) -> Option<ActivityTerminalOutcome> {
        self.terminal
    }
    #[must_use]
    pub const fn state_hash(self) -> ActivityStateHash {
        self.state_hash
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityBattleSettlementError {
    StaleState,
    MissingPendingBattle,
    BattleAlreadyStarted,
    BattleNotStarted,
    ParticipantContractMismatch,
    MetricSlotMismatch,
    ResultIdentityMismatch,
    ResultDigestMismatch,
    ResultProjectionMismatch,
    FaultOutcomeMismatch,
    ParticipantResultMismatch,
    ParticipantMaximumMismatch,
    CarryInvariant,
    MissingOutcomeEdge,
    AmbiguousOutcomeEdge,
    CompletedBattleLimit,
    ActivityFault(ActivityFault),
}

impl crate::ActivityTransactionState {
    pub fn start_pending_battle(
        &mut self,
        graph: &ActivityGraphDefinition,
        rng: &ActivityRngStreams,
        request: ActivityBattleStartRequest,
    ) -> Result<ActivityBattleHandoff, ActivityBattleSettlementError> {
        if self.state_hash(request.definition_identity, graph, request.instance, rng)
            != request.expected_state_hash
        {
            return Err(ActivityBattleSettlementError::StaleState);
        }
        if self.awaiting_battle.is_some() {
            return Err(ActivityBattleSettlementError::BattleAlreadyStarted);
        }
        let attempt = self
            .attempt
            .as_ref()
            .ok_or(ActivityBattleSettlementError::MissingPendingBattle)?;
        let pending = attempt
            .pending()
            .ok_or(ActivityBattleSettlementError::MissingPendingBattle)?;
        validate_contract(attempt, &request.contract)?;
        if request.contract.metrics.iter().any(|binding| {
            self.slot(binding.slot)
                .is_none_or(|value| value.kind() != metric_slot_kind(binding.kind))
        }) {
            return Err(ActivityBattleSettlementError::MetricSlotMismatch);
        }
        let path = pending.path();
        let scope = ScopeIdentity::new(
            path.activity(),
            path.section()
                .ok_or(ActivityBattleSettlementError::MissingPendingBattle)?,
            path.node()
                .ok_or(ActivityBattleSettlementError::MissingPendingBattle)?,
            path.attempt()
                .ok_or(ActivityBattleSettlementError::MissingPendingBattle)?,
        );
        let seed = derive_battle_seed(
            rng,
            request.definition_identity,
            pending,
            request.contract.digest(),
        )?;
        let identity = BattleResultIdentity::new(
            scope,
            pending.battle_sequence(),
            BattleResultConfiguration::new(
                request.definition_identity.definition_digest(),
                request.definition_identity.config_digest(),
                pending.participant_lock_digest(),
            ),
            pending.battle_spec_digest(),
            seed,
        );
        self.awaiting_battle = Some(ActivityAwaitingBattle {
            identity,
            contract: Arc::clone(&request.contract),
        });
        Ok(ActivityBattleHandoff {
            identity,
            battle_spec: pending.battle_spec().clone(),
            carry: self.carry.view(),
            contract_digest: request.contract.digest(),
        })
    }

    pub fn submit_pending_battle_result(
        &mut self,
        identity: ActivityDefinitionIdentity,
        graph: &ActivityGraphDefinition,
        instance: ActivityInstanceId,
        rng: &ActivityRngStreams,
        submission: ActivityBattleResultSubmission,
    ) -> Result<ActivityBattleSettlement, ActivityBattleSettlementError> {
        if self.state_hash(identity, graph, instance, rng) != submission.expected_state_hash {
            return Err(ActivityBattleSettlementError::StaleState);
        }
        let awaiting = self
            .awaiting_battle
            .as_ref()
            .ok_or(ActivityBattleSettlementError::BattleNotStarted)?;
        let result = submission.result.as_ref();
        if result.identity() != awaiting.identity {
            return Err(ActivityBattleSettlementError::ResultIdentityMismatch);
        }
        let actual_digest = result.actual_digest();
        if actual_digest != result.claimed_digest() {
            return Err(ActivityBattleSettlementError::ResultDigestMismatch);
        }
        if !awaiting.contract.projection.matches(result.values()) {
            return Err(ActivityBattleSettlementError::ResultProjectionMismatch);
        }
        let outcome = result
            .outcome()
            .ok_or(ActivityBattleSettlementError::ResultProjectionMismatch)?;
        let fault = result
            .terminal_fault()
            .ok_or(ActivityBattleSettlementError::ResultProjectionMismatch)?;
        if (outcome == BattleOutcome::Faulted) != fault.is_some() {
            return Err(ActivityBattleSettlementError::FaultOutcomeMismatch);
        }
        let attempt = self
            .attempt
            .as_ref()
            .ok_or(ActivityBattleSettlementError::MissingPendingBattle)?;
        validate_participant_results(attempt, result)?;
        let edge = outcome_edge(graph, self.current_node(), outcome)?;
        if self.completed_battles.len() >= MAX_COMPLETED_ACTIVITY_BATTLES {
            return Err(ActivityBattleSettlementError::CompletedBattleLimit);
        }
        let contract = Arc::clone(&awaiting.contract);
        let mut working = self.transaction_copy();
        for definition in contract.carry.iter().copied() {
            let state = result
                .participant_states()
                .find(|state| state.participant() == definition.participant)
                .ok_or(ActivityBattleSettlementError::ParticipantResultMismatch)?;
            working.carry.insert(apply_carry(definition, state)?);
        }
        for binding in contract.metrics.iter() {
            let value = result
                .metrics()
                .find(|(key, _)| *key == binding.key())
                .map(|(_, value)| value)
                .ok_or(ActivityBattleSettlementError::ResultProjectionMismatch)?;
            working
                .settle_metric(binding.slot, metric_value(value), binding.policy)
                .map_err(ActivityBattleSettlementError::ActivityFault)?;
        }
        let target = working
            .traverse_edge(edge, graph)
            .map_err(ActivityBattleSettlementError::ActivityFault)?;
        let terminal = graph.node(target).and_then(|node| node.kind().terminal());
        if let Some(terminal) = terminal {
            working.settle_terminal(terminal);
        }
        working.awaiting_battle = None;
        working
            .attempt
            .as_mut()
            .expect("validated attempt exists")
            .mark_settled();
        working.completed_battles.push(actual_digest);
        *self = working;
        Ok(ActivityBattleSettlement {
            outcome,
            traversed_edge: edge,
            target,
            result_digest: actual_digest,
            terminal,
            state_hash: self.state_hash(identity, graph, instance, rng),
        })
    }
}

fn validate_contract(
    attempt: &crate::battle_preparation::ActivityAttemptState,
    contract: &ActivityBattleResultContract,
) -> Result<(), ActivityBattleSettlementError> {
    let participants = attempt.participant_specs();
    if participants.len() != contract.carry.len()
        || participants.iter().any(|(participant, _)| {
            contract
                .carry
                .binary_search_by_key(participant, |item| item.participant)
                .is_err()
        })
    {
        return Err(ActivityBattleSettlementError::ParticipantContractMismatch);
    }
    Ok(())
}

fn validate_participant_results(
    attempt: &crate::battle_preparation::ActivityAttemptState,
    result: &BattleResult,
) -> Result<(), ActivityBattleSettlementError> {
    let participants = attempt.participant_specs();
    let states = result.participant_states().collect::<Vec<_>>();
    if participants.len() != states.len() {
        return Err(ActivityBattleSettlementError::ParticipantResultMismatch);
    }
    for (participant, spec) in participants {
        let state = states
            .iter()
            .find(|state| state.participant() == participant)
            .ok_or(ActivityBattleSettlementError::ParticipantResultMismatch)?;
        if state.maximum_hp() != spec.combatant().maximum_hp()
            || state.maximum_energy() != spec.combatant().maximum_energy()
        {
            return Err(ActivityBattleSettlementError::ParticipantMaximumMismatch);
        }
    }
    Ok(())
}

fn apply_carry(
    definition: ActivityParticipantCarryDefinition,
    state: ParticipantBattleState,
) -> Result<ActivityParticipantCarryState, ActivityBattleSettlementError> {
    let current_hp = match definition.hp {
        HpCarryPolicy::CarryExact => state.current_hp(),
        HpCarryPolicy::CarryClamped => {
            Hp::new(state.current_hp().get().min(state.maximum_hp().get()))
                .expect("minimum of non-negative HP remains non-negative")
        }
        HpCarryPolicy::RestoreFull => state.maximum_hp(),
    };
    let current_energy = match definition.energy {
        EnergyCarryPolicy::CarryExact => state.current_energy(),
        EnergyCarryPolicy::CarryClamped => Energy::from_scaled(
            state
                .current_energy()
                .scaled()
                .min(state.maximum_energy().scaled()),
        )
        .expect("minimum of non-negative Energy remains non-negative"),
        EnergyCarryPolicy::ResetZero => Energy::ZERO,
    };
    let life = match definition.life {
        LifeCarryPolicy::CarryExact => state.life(),
        LifeCarryPolicy::DefeatOnZero => {
            if current_hp.get() == 0 {
                LifeState::Defeated
            } else {
                LifeState::Alive
            }
        }
        LifeCarryPolicy::RestoreAlive => LifeState::Alive,
    };
    let presence = match definition.presence {
        PresenceCarryPolicy::CarryExact => state.presence(),
        PresenceCarryPolicy::DepartIfDefeated if life == LifeState::Defeated => {
            PresenceState::Departed
        }
        PresenceCarryPolicy::DepartIfDefeated => state.presence(),
        PresenceCarryPolicy::RestorePresent => PresenceState::Present,
    };
    if (life == LifeState::Alive) != (current_hp.get() > 0) {
        return Err(ActivityBattleSettlementError::CarryInvariant);
    }
    Ok(ActivityParticipantCarryState {
        participant: definition.participant,
        current_hp,
        maximum_hp: state.maximum_hp(),
        current_energy,
        maximum_energy: state.maximum_energy(),
        life,
        presence,
    })
}

fn metric_value(value: MetricValue) -> ActivityValue {
    match value {
        MetricValue::BoundedInteger(value) => ActivityValue::BoundedInteger(value),
        MetricValue::FixedScalar(value)
        | MetricValue::Ratio(value)
        | MetricValue::Probability(value)
        | MetricValue::ActionValue(value) => ActivityValue::FixedScalar(value),
    }
}

const fn metric_slot_kind(kind: MetricValueKind) -> crate::SlotValueKind {
    match kind {
        MetricValueKind::BoundedInteger => crate::SlotValueKind::BoundedInteger,
        MetricValueKind::FixedScalar
        | MetricValueKind::Ratio
        | MetricValueKind::Probability
        | MetricValueKind::ActionValue => crate::SlotValueKind::FixedScalar,
    }
}

fn outcome_edge(
    graph: &ActivityGraphDefinition,
    node: NodeId,
    outcome: BattleOutcome,
) -> Result<ActivityEdgeId, ActivityBattleSettlementError> {
    let expected: TerminalOutcome = outcome.into();
    let mut edges = graph
        .outgoing(node)
        .filter(|edge| edge.condition() == ActivityEdgeCondition::BattleOutcome(expected));
    let edge = edges
        .next()
        .ok_or(ActivityBattleSettlementError::MissingOutcomeEdge)?
        .id();
    if edges.next().is_some() {
        return Err(ActivityBattleSettlementError::AmbiguousOutcomeEdge);
    }
    Ok(edge)
}

fn derive_battle_seed(
    rng: &ActivityRngStreams,
    identity: ActivityDefinitionIdentity,
    pending: &crate::PendingBattleSpec,
    contract: BattleSettlementContractDigest,
) -> Result<BattleSeed, ActivityBattleSettlementError> {
    let battle_seed = rng
        .snapshots()
        .iter()
        .find(|snapshot| snapshot.label() == ActivityRngLabel::Battle)
        .map(|snapshot| snapshot.seed())
        .ok_or(ActivityBattleSettlementError::MissingPendingBattle)?;
    let path = pending.path();
    let mut hash = Sha256::new();
    hash.update(b"SCBH");
    hash.update(1_u32.to_le_bytes());
    hash.update(battle_seed);
    hash.update(identity.id().get().to_le_bytes());
    hash.update(identity.definition_digest().bytes());
    hash.update(identity.config_digest().bytes());
    hash.update(path.activity().get().to_le_bytes());
    hash.update(path.section().expect("validated path").get().to_le_bytes());
    hash.update(path.node().expect("validated path").get().to_le_bytes());
    hash.update(path.attempt().expect("validated path").get().to_le_bytes());
    hash.update(pending.battle_sequence().get().to_le_bytes());
    hash.update(pending.participant_lock_digest().bytes());
    hash.update(pending.battle_spec_digest().bytes());
    hash.update(contract.bytes());
    Ok(BattleSeed::new(hash.finalize().into()))
}

fn contract_digest(
    projection: &BattleResultProjection,
    carry: &[ActivityParticipantCarryDefinition],
    metrics: &[ActivityMetricProjectionBinding],
) -> BattleSettlementContractDigest {
    let mut writer = CanonicalWriter::new(b"starclock-battle-settlement-contract-v1");
    writer.digest(projection.digest().bytes());
    writer.u64(carry.len() as u64);
    for item in carry {
        writer.u32(item.participant.get());
        writer.byte(item.hp as u8);
        writer.byte(item.energy as u8);
        writer.byte(item.life as u8);
        writer.byte(item.presence as u8);
    }
    writer.u64(metrics.len() as u64);
    for item in metrics {
        writer.text(&item.key);
        writer.byte(item.kind as u8);
        writer.u32(item.slot.get());
        writer.byte(item.policy as u8);
    }
    BattleSettlementContractDigest::new(writer.finish()).expect("SHA-256 output is non-zero")
}

fn encode_result_identity(writer: &mut ActivityStateEncoder, identity: BattleResultIdentity) {
    let scope = identity.scope();
    writer.u64(scope.activity().get());
    writer.u32(scope.section().get());
    writer.u32(scope.node().get());
    writer.u32(scope.attempt().get());
    writer.u32(identity.battle_sequence().get());
    writer.digest(identity.definition_digest().bytes());
    writer.digest(identity.config_digest().bytes());
    writer.digest(identity.participant_lock_digest().bytes());
    writer.digest(identity.spec_digest().bytes());
    writer.digest(identity.seed().bytes());
}
