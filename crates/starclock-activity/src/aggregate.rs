use starclock_combat::{BattleSeed, BattleSpec};

use crate::{
    ActivityDefinitionIdentity, ActivityEdgeCondition, ActivityInstanceId, ActivityMasterSeed,
    ActivitySlotId, ActivitySpec, ActivityStateHash, ActivityValue, BattleOutcome, BattleResult,
    BattleResultDigest, BattleResultIdentity, BattleSequence, NodeId, ScopeIdentity,
    SlotResetPoint, TerminalOutcome, codec::CanonicalWriter, slot::ScopedSlots,
};

/// Stable lifecycle state of the minimum one-Battle aggregate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityPhase {
    ReadyToStartBattle,
    AwaitingBattleResult,
    Terminal(TerminalOutcome),
}

/// The only decisions exposed by the Goal 01 activity surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityDecision {
    StartBattle(BattleResultIdentity),
    SubmitBattleResult(BattleResultIdentity),
    Terminal(TerminalOutcome),
}

/// Command surface for starting and completing exactly one battle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivityCommand {
    StartBattle {
        expected_state_hash: ActivityStateHash,
    },
    SubmitBattleResult {
        expected_state_hash: ActivityStateHash,
        result: Box<BattleResult>,
    },
}

/// Typed audit facts emitted by accepted Activity commands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivityEvent {
    SlotReset {
        slot: ActivitySlotId,
        point: SlotResetPoint,
    },
    BattleRequested(BattleResultIdentity),
    BattleResultAccepted(BattleResultDigest),
    Terminal(TerminalOutcome),
}

/// Opaque immutable request returned at the only Activity-to-combat boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleHandoff {
    identity: BattleResultIdentity,
    spec: BattleSpec,
    seed: BattleSeed,
}

impl BattleHandoff {
    #[must_use]
    pub const fn identity(&self) -> BattleResultIdentity {
        self.identity
    }
    #[must_use]
    pub const fn battle_spec(&self) -> &BattleSpec {
        &self.spec
    }
    #[must_use]
    pub const fn seed(&self) -> BattleSeed {
        self.seed
    }
}

/// Accepted command output at a stable handoff/terminal boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityResolution {
    events: Vec<ActivityEvent>,
    next_decision: ActivityDecision,
    battle_handoff: Option<BattleHandoff>,
    state_hash: ActivityStateHash,
}

impl ActivityResolution {
    #[must_use]
    pub fn events(&self) -> &[ActivityEvent] {
        &self.events
    }
    #[must_use]
    pub const fn next_decision(&self) -> ActivityDecision {
        self.next_decision
    }
    #[must_use]
    pub const fn battle_handoff(&self) -> Option<&BattleHandoff> {
        self.battle_handoff.as_ref()
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
}

/// Authoritative one-Battle Activity aggregate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Activity {
    spec: ActivitySpec,
    scope: ScopeIdentity,
    battle_sequence: BattleSequence,
    seed: BattleSeed,
    slots: ScopedSlots,
    phase: ActivityPhase,
    current_node: NodeId,
    completed_result: Option<BattleResultDigest>,
}

impl Activity {
    pub fn new(
        spec: ActivitySpec,
        instance: ActivityInstanceId,
        master_seed: ActivityMasterSeed,
    ) -> Self {
        let attempt = crate::AttemptId::new(1).expect("one is a valid attempt ID");
        let battle_sequence = BattleSequence::new(1).expect("one is a valid sequence");
        let entry = spec.graph().entry();
        let section = spec
            .graph()
            .node(entry)
            .expect("validated graph contains its entry")
            .section();
        let scope = ScopeIdentity::new(instance, section, entry, attempt);
        let seed = spec
            .binding()
            .derive_seed(master_seed, spec.identity(), scope, battle_sequence);
        let mut slots = ScopedSlots::new(spec.slots().to_vec())
            .expect("ActivitySpec rejects duplicate slot IDs");
        for point in [
            SlotResetPoint::ActivityStart,
            SlotResetPoint::SectionStart,
            SlotResetPoint::NodeStart,
            SlotResetPoint::AttemptStart,
        ] {
            let _ = slots.reset(point);
        }
        let current_node = entry;
        Self {
            spec,
            scope,
            battle_sequence,
            seed,
            slots,
            phase: ActivityPhase::ReadyToStartBattle,
            current_node,
            completed_result: None,
        }
    }

    #[must_use]
    pub const fn phase(&self) -> ActivityPhase {
        self.phase
    }
    #[must_use]
    pub const fn scope(&self) -> ScopeIdentity {
        self.scope
    }
    #[must_use]
    pub const fn definition_identity(&self) -> ActivityDefinitionIdentity {
        self.spec.identity()
    }
    #[must_use]
    pub const fn current_node(&self) -> NodeId {
        self.current_node
    }
    #[must_use]
    pub fn slot_value(&self, id: ActivitySlotId) -> Option<&ActivityValue> {
        self.slots.value(id)
    }
    #[must_use]
    pub fn state_hash(&self) -> ActivityStateHash {
        self.hash_state()
    }

    #[must_use]
    pub fn decision(&self) -> ActivityDecision {
        match self.phase {
            ActivityPhase::ReadyToStartBattle => {
                ActivityDecision::StartBattle(self.expected_identity())
            }
            ActivityPhase::AwaitingBattleResult => {
                ActivityDecision::SubmitBattleResult(self.expected_identity())
            }
            ActivityPhase::Terminal(outcome) => ActivityDecision::Terminal(outcome),
        }
    }

    pub fn apply(
        &mut self,
        command: ActivityCommand,
    ) -> Result<ActivityResolution, ActivityCommandError> {
        let current_hash = self.hash_state();
        match command {
            ActivityCommand::StartBattle {
                expected_state_hash,
            } => {
                if expected_state_hash != current_hash {
                    return Err(ActivityCommandError::new(
                        ActivityCommandErrorKind::StaleStateHash,
                    ));
                }
                if self.phase != ActivityPhase::ReadyToStartBattle {
                    return Err(ActivityCommandError::new(
                        ActivityCommandErrorKind::CommandNotOffered,
                    ));
                }
                let identity = self.expected_identity();
                let mut events = self
                    .slots
                    .reset(SlotResetPoint::BattleStart)
                    .into_iter()
                    .map(|slot| ActivityEvent::SlotReset {
                        slot,
                        point: SlotResetPoint::BattleStart,
                    })
                    .collect::<Vec<_>>();
                events.push(ActivityEvent::BattleRequested(identity));
                self.phase = ActivityPhase::AwaitingBattleResult;
                let handoff = BattleHandoff {
                    identity,
                    spec: self.spec.binding().battle_spec().clone(),
                    seed: self.seed,
                };
                Ok(self.resolution(events, Some(handoff)))
            }
            ActivityCommand::SubmitBattleResult {
                expected_state_hash,
                result,
            } => {
                if expected_state_hash != current_hash {
                    return Err(ActivityCommandError::new(
                        ActivityCommandErrorKind::StaleStateHash,
                    ));
                }
                if self.phase != ActivityPhase::AwaitingBattleResult {
                    return Err(ActivityCommandError::new(
                        ActivityCommandErrorKind::CommandNotOffered,
                    ));
                }
                self.validate_result(&result)?;
                let digest = result.actual_digest();
                let outcome: TerminalOutcome = result
                    .outcome()
                    .expect("validated projection has outcome")
                    .into();
                let mut events = vec![ActivityEvent::BattleResultAccepted(digest)];
                events.extend(self.slots.reset(SlotResetPoint::BattleEnd).into_iter().map(
                    |slot| ActivityEvent::SlotReset {
                        slot,
                        point: SlotResetPoint::BattleEnd,
                    },
                ));
                events.push(ActivityEvent::Terminal(outcome));
                self.phase = ActivityPhase::Terminal(outcome);
                self.current_node = self
                    .spec
                    .graph()
                    .outgoing(self.current_node)
                    .find(|edge| edge.condition() == ActivityEdgeCondition::BattleOutcome(outcome))
                    .expect("one-battle graph has one edge for every battle outcome")
                    .to();
                self.completed_result = Some(digest);
                Ok(self.resolution(events, None))
            }
        }
    }

    fn validate_result(&self, result: &BattleResult) -> Result<(), ActivityCommandError> {
        let expected = self.expected_identity();
        let actual = result.identity();
        macro_rules! identity {
            ($getter:ident, $field:ident) => {
                if actual.$getter() != expected.$getter() {
                    return Err(ActivityCommandError::new(
                        ActivityCommandErrorKind::ResultIdentityMismatch(
                            ResultIdentityField::$field,
                        ),
                    ));
                }
            };
        }
        identity!(activity, Activity);
        if actual.scope().section() != expected.scope().section() {
            return Err(identity_error(ResultIdentityField::Section));
        }
        if actual.scope().node() != expected.scope().node() {
            return Err(identity_error(ResultIdentityField::Node));
        }
        if actual.scope().attempt() != expected.scope().attempt() {
            return Err(identity_error(ResultIdentityField::Attempt));
        }
        identity!(battle_sequence, BattleSequence);
        identity!(definition_digest, DefinitionDigest);
        identity!(config_digest, ConfigDigest);
        identity!(participant_lock_digest, ParticipantLockDigest);
        identity!(combat_input_digest, CombatInputDigest);
        identity!(assembly_digest, AssemblyDigest);
        identity!(seed, Seed);
        if result.actual_digest() != result.claimed_digest() {
            return Err(ActivityCommandError::new(
                ActivityCommandErrorKind::ResultDigestMismatch,
            ));
        }
        if !self.spec.projection().matches(result.values()) {
            return Err(ActivityCommandError::new(
                ActivityCommandErrorKind::ProjectionMismatch,
            ));
        }
        let outcome = result.outcome().expect("matching required projection");
        let fault = result
            .terminal_fault()
            .expect("matching required projection");
        if (outcome == BattleOutcome::Faulted) != fault.is_some() {
            return Err(ActivityCommandError::new(
                ActivityCommandErrorKind::OutcomeFaultMismatch,
            ));
        }
        Ok(())
    }

    fn expected_identity(&self) -> BattleResultIdentity {
        self.spec
            .result_identity(self.scope, self.battle_sequence, self.seed)
    }

    fn resolution(
        &self,
        events: Vec<ActivityEvent>,
        battle_handoff: Option<BattleHandoff>,
    ) -> ActivityResolution {
        ActivityResolution {
            events,
            next_decision: self.decision(),
            battle_handoff,
            state_hash: self.hash_state(),
        }
    }

    fn hash_state(&self) -> ActivityStateHash {
        let mut writer = CanonicalWriter::new(b"starclock-activity-state-v2");
        self.spec.encode(&mut writer);
        writer.u64(self.scope.activity().get());
        writer.u32(self.scope.section().get());
        writer.u32(self.scope.node().get());
        writer.u32(self.scope.attempt().get());
        writer.u32(self.battle_sequence.get());
        writer.digest(self.seed.bytes());
        writer.byte(match self.phase {
            ActivityPhase::ReadyToStartBattle => 0,
            ActivityPhase::AwaitingBattleResult => 1,
            ActivityPhase::Terminal(_) => 2,
        });
        if let ActivityPhase::Terminal(outcome) = self.phase {
            writer.byte(outcome as u8);
        }
        writer.u32(self.current_node.get());
        self.slots.encode(&mut writer);
        writer.bool(self.completed_result.is_some());
        if let Some(digest) = self.completed_result {
            writer.digest(digest.bytes());
        }
        ActivityStateHash::new(writer.finish()).expect("activity hashes accept every byte sequence")
    }
}

fn identity_error(field: ResultIdentityField) -> ActivityCommandError {
    ActivityCommandError::new(ActivityCommandErrorKind::ResultIdentityMismatch(field))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultIdentityField {
    Activity,
    Section,
    Node,
    Attempt,
    BattleSequence,
    DefinitionDigest,
    ConfigDigest,
    ParticipantLockDigest,
    CombatInputDigest,
    AssemblyDigest,
    Seed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityCommandErrorKind {
    StaleStateHash,
    CommandNotOffered,
    ResultIdentityMismatch(ResultIdentityField),
    ResultDigestMismatch,
    ProjectionMismatch,
    OutcomeFaultMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityCommandError {
    kind: ActivityCommandErrorKind,
}

impl ActivityCommandError {
    const fn new(kind: ActivityCommandErrorKind) -> Self {
        Self { kind }
    }
    #[must_use]
    pub const fn kind(self) -> ActivityCommandErrorKind {
        self.kind
    }
}

impl core::fmt::Display for ActivityCommandError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "activity command rejected: {:?}", self.kind)
    }
}

impl std::error::Error for ActivityCommandError {}
