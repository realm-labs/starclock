use starclock_combat::{BattleSeed, BattleSpec};

use crate::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId, ActivityGraphDefinition,
    ActivitySlotDefinition, ActivityStateDefinition, ActivityStateDefinitionError,
    BattleResultConfiguration, BattleResultIdentity, BattleResultProjection, BattleSequence,
    OneBattleFlow, ParticipantLock, ParticipantLockDigest, ScopeIdentity, codec::CanonicalWriter,
};

/// Immutable definition/configuration identity carried through every battle result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityDefinitionIdentity {
    id: ActivityDefinitionId,
    definition_digest: ActivityDefinitionDigest,
    config_digest: ActivityConfigDigest,
}

impl ActivityDefinitionIdentity {
    #[must_use]
    pub const fn new(
        id: ActivityDefinitionId,
        definition_digest: ActivityDefinitionDigest,
        config_digest: ActivityConfigDigest,
    ) -> Self {
        Self {
            id,
            definition_digest,
            config_digest,
        }
    }

    #[must_use]
    pub const fn id(self) -> ActivityDefinitionId {
        self.id
    }
    #[must_use]
    pub const fn definition_digest(self) -> ActivityDefinitionDigest {
        self.definition_digest
    }
    #[must_use]
    pub const fn config_digest(self) -> ActivityConfigDigest {
        self.config_digest
    }
}

/// Exact activity master seed; each battle seed is purpose-derived from it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ActivityMasterSeed([u8; 32]);

impl ActivityMasterSeed {
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub fn from_u64(value: u64) -> Self {
        let mut writer = CanonicalWriter::new(b"starclock-activity-master-seed-v1");
        writer.u64(value);
        Self(writer.finish())
    }

    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

/// Immutable opaque battle request plus deterministic handoff policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleBinding {
    spec: BattleSpec,
    seed_stream_label: Box<str>,
    battle_spec_policy_revision: Box<str>,
    participant_lock_digest: ParticipantLockDigest,
}

impl BattleBinding {
    pub fn new(
        spec: BattleSpec,
        seed_stream_label: impl Into<Box<str>>,
        battle_spec_policy_revision: impl Into<Box<str>>,
        participant_lock_digest: ParticipantLockDigest,
    ) -> Result<Self, BattleBindingError> {
        let label = seed_stream_label.into();
        let revision = battle_spec_policy_revision.into();
        if !valid_ascii(&label, 120) {
            return Err(BattleBindingError::InvalidSeedStreamLabel);
        }
        if !valid_ascii(&revision, 80) {
            return Err(BattleBindingError::InvalidPolicyRevision);
        }
        Ok(Self {
            spec,
            seed_stream_label: label,
            battle_spec_policy_revision: revision,
            participant_lock_digest,
        })
    }

    #[must_use]
    pub const fn battle_spec(&self) -> &BattleSpec {
        &self.spec
    }
    #[must_use]
    pub fn seed_stream_label(&self) -> &str {
        &self.seed_stream_label
    }
    #[must_use]
    pub fn battle_spec_policy_revision(&self) -> &str {
        &self.battle_spec_policy_revision
    }
    #[must_use]
    pub const fn participant_lock_digest(&self) -> ParticipantLockDigest {
        self.participant_lock_digest
    }

    pub(crate) fn derive_seed(
        &self,
        master: ActivityMasterSeed,
        identity: ActivityDefinitionIdentity,
        scope: ScopeIdentity,
        sequence: BattleSequence,
    ) -> BattleSeed {
        let mut writer = CanonicalWriter::new(b"starclock-activity-battle-seed-v1");
        writer.digest(master.bytes());
        writer.u32(identity.id().get());
        writer.digest(identity.definition_digest().bytes());
        writer.digest(identity.config_digest().bytes());
        writer.u64(scope.activity().get());
        writer.u32(scope.section().get());
        writer.u32(scope.node().get());
        writer.u32(scope.attempt().get());
        writer.u32(sequence.get());
        writer.text(&self.seed_stream_label);
        writer.text(&self.battle_spec_policy_revision);
        writer.digest(self.participant_lock_digest.bytes());
        writer.text(starclock_combat::COMBAT_INPUT_CODEC_REVISION);
        writer.digest(self.spec.combat_input_digest().bytes());
        writer.digest(self.spec.assembly_digest().bytes());
        BattleSeed::new(writer.finish())
    }

    pub(crate) fn encode(&self, writer: &mut CanonicalWriter) {
        writer.text(&self.seed_stream_label);
        writer.text(&self.battle_spec_policy_revision);
        writer.digest(self.participant_lock_digest.bytes());
        writer.text(starclock_combat::COMBAT_INPUT_CODEC_REVISION);
        writer.digest(self.spec.combat_input_digest().bytes());
        writer.digest(self.spec.assembly_digest().bytes());
    }
}

/// Fully validated minimum Activity input. It intentionally contains no mode extension state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivitySpec {
    identity: ActivityDefinitionIdentity,
    flow: OneBattleFlow,
    graph: ActivityGraphDefinition,
    state: ActivityStateDefinition,
    participants: ParticipantLock,
    projection: BattleResultProjection,
    binding: BattleBinding,
}

impl ActivitySpec {
    pub fn new(
        identity: ActivityDefinitionIdentity,
        flow: OneBattleFlow,
        slots: Vec<ActivitySlotDefinition>,
        participants: ParticipantLock,
        projection: BattleResultProjection,
        binding: BattleBinding,
    ) -> Result<Self, ActivitySpecError> {
        let state =
            ActivityStateDefinition::new(slots, vec![], vec![]).map_err(|error| match error {
                ActivityStateDefinitionError::DuplicateSlot(_) => ActivitySpecError::DuplicateSlot,
                ActivityStateDefinitionError::TooManySlots => ActivitySpecError::TooManySlots,
                _ => unreachable!("empty inventory/modifier collections have no other errors"),
            })?;
        if participants.digest() != binding.participant_lock_digest() {
            return Err(ActivitySpecError::ParticipantLockMismatch);
        }
        let graph = flow.into_graph();
        Ok(Self {
            identity,
            flow,
            graph,
            state,
            participants,
            projection,
            binding,
        })
    }

    #[must_use]
    pub const fn identity(&self) -> ActivityDefinitionIdentity {
        self.identity
    }
    #[must_use]
    pub const fn flow(&self) -> OneBattleFlow {
        self.flow
    }
    /// Generic graph backing the retained Goal 01 one-battle profile.
    #[must_use]
    pub const fn graph(&self) -> &ActivityGraphDefinition {
        &self.graph
    }
    #[must_use]
    pub fn slots(&self) -> &[ActivitySlotDefinition] {
        self.state.slots()
    }
    #[must_use]
    pub const fn state_definition(&self) -> &ActivityStateDefinition {
        &self.state
    }
    #[must_use]
    pub const fn participants(&self) -> &ParticipantLock {
        &self.participants
    }
    #[must_use]
    pub const fn projection(&self) -> &BattleResultProjection {
        &self.projection
    }
    #[must_use]
    pub const fn binding(&self) -> &BattleBinding {
        &self.binding
    }

    pub(crate) fn result_identity(
        &self,
        scope: ScopeIdentity,
        sequence: BattleSequence,
        seed: BattleSeed,
    ) -> BattleResultIdentity {
        BattleResultIdentity::new(
            scope,
            sequence,
            BattleResultConfiguration::new(
                self.identity.definition_digest(),
                self.identity.config_digest(),
                self.participants.digest(),
            ),
            self.binding.spec.combat_input_digest(),
            self.binding.spec.assembly_digest(),
            seed,
        )
    }

    pub(crate) fn encode(&self, writer: &mut CanonicalWriter) {
        writer.u32(self.identity.id().get());
        writer.digest(self.identity.definition_digest().bytes());
        writer.digest(self.identity.config_digest().bytes());
        self.flow.encode(writer);
        writer.u64(self.state.slots().len() as u64);
        for slot in self.state.slots() {
            slot.encode(writer);
        }
        self.participants.encode(writer);
        self.projection.encode(writer);
        self.binding.encode(writer);
    }
}

fn valid_ascii(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum && value.bytes().all(|byte| byte.is_ascii_graphic())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleBindingError {
    InvalidSeedStreamLabel,
    InvalidPolicyRevision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivitySpecError {
    DuplicateSlot,
    TooManySlots,
    ParticipantLockMismatch,
}
