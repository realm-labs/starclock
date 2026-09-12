//! Immutable caller, save and terminal-settlement snapshots.

use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityStateHash, ActivityTerminalOutcome,
    GraphActivity, ParticipantLock, ParticipantLockDigest,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseFinishConditionId;
use starclock_data::divergent_universe_progression_catalog::DivergentUniverseDivisionId;

use super::entry_flow::DivergentUniverseFlowInstance;

macro_rules! digest_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name([u8; 32]);

        impl $name {
            pub const fn new(bytes: [u8; 32]) -> Result<Self, DivergentUniverseSnapshotError> {
                if all_zero(&bytes) {
                    Err(DivergentUniverseSnapshotError::ZeroDigest)
                } else {
                    Ok(Self(bytes))
                }
            }

            #[must_use]
            pub const fn bytes(self) -> [u8; 32] {
                self.0
            }
        }
    };
}

digest_type!(DivergentUniverseAccountSnapshotDigest);
digest_type!(DivergentUniverseLoadoutSnapshotDigest);
digest_type!(DivergentUniverseInputSnapshotDigest);
digest_type!(DivergentUniverseSaveSnapshotDigest);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseInputSnapshot {
    account: DivergentUniverseAccountSnapshotDigest,
    loadout: DivergentUniverseLoadoutSnapshotDigest,
    participants: ParticipantLockDigest,
    digest: DivergentUniverseInputSnapshotDigest,
}

impl DivergentUniverseInputSnapshot {
    #[must_use]
    pub fn seal(
        account: DivergentUniverseAccountSnapshotDigest,
        loadout: DivergentUniverseLoadoutSnapshotDigest,
        participants: &ParticipantLock,
    ) -> Self {
        let digest = hash(&[
            b"starclock.divergent-universe.input-snapshot.v1",
            &account.bytes(),
            &loadout.bytes(),
            &participants.digest().bytes(),
        ]);
        Self {
            account,
            loadout,
            participants: participants.digest(),
            digest: DivergentUniverseInputSnapshotDigest::new(digest)
                .expect("domain-separated SHA-256 is non-zero"),
        }
    }

    #[must_use]
    pub const fn account(&self) -> DivergentUniverseAccountSnapshotDigest {
        self.account
    }

    #[must_use]
    pub const fn loadout(&self) -> DivergentUniverseLoadoutSnapshotDigest {
        self.loadout
    }

    #[must_use]
    pub const fn participant_lock(&self) -> ParticipantLockDigest {
        self.participants
    }

    #[must_use]
    pub const fn digest(&self) -> DivergentUniverseInputSnapshotDigest {
        self.digest
    }

    pub(super) fn validate_participants(
        &self,
        participants: &ParticipantLock,
    ) -> Result<(), DivergentUniverseSnapshotError> {
        if self.participants != participants.digest() {
            return Err(DivergentUniverseSnapshotError::ParticipantSnapshotMismatch);
        }
        let expected = Self::seal(self.account, self.loadout, participants);
        if expected.digest != self.digest {
            return Err(DivergentUniverseSnapshotError::InputDigestMismatch);
        }
        Ok(())
    }

    pub(super) fn stable_value(&self) -> u64 {
        stable_digest(self.digest.bytes())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseSaveSnapshot {
    component_digest: [u8; 32],
    definition: ActivityDefinitionIdentity,
    input: DivergentUniverseInputSnapshotDigest,
    participants: ParticipantLockDigest,
    instance: ActivityInstanceId,
    state_hash: ActivityStateHash,
    canonical_state: Box<[u8]>,
    digest: DivergentUniverseSaveSnapshotDigest,
}

impl DivergentUniverseSaveSnapshot {
    #[must_use]
    pub const fn definition(&self) -> ActivityDefinitionIdentity {
        self.definition
    }

    #[must_use]
    pub const fn input(&self) -> DivergentUniverseInputSnapshotDigest {
        self.input
    }

    #[must_use]
    pub const fn participant_lock(&self) -> ParticipantLockDigest {
        self.participants
    }

    #[must_use]
    pub const fn instance(&self) -> ActivityInstanceId {
        self.instance
    }

    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }

    #[must_use]
    pub fn canonical_state_bytes(&self) -> &[u8] {
        &self.canonical_state
    }

    #[must_use]
    pub const fn digest(&self) -> DivergentUniverseSaveSnapshotDigest {
        self.digest
    }

    fn compute_digest(&self) -> DivergentUniverseSaveSnapshotDigest {
        DivergentUniverseSaveSnapshotDigest::new(hash(&[
            b"starclock.divergent-universe.save-snapshot.v1",
            &self.component_digest,
            &self.definition.id().get().to_le_bytes(),
            &self.definition.definition_digest().bytes(),
            &self.definition.config_digest().bytes(),
            &self.input.bytes(),
            &self.participants.bytes(),
            &self.instance.get().to_le_bytes(),
            &self.state_hash.bytes(),
            &self.canonical_state,
        ]))
        .expect("domain-separated SHA-256 is non-zero")
    }

    #[cfg(test)]
    pub(super) fn corrupt_state_for_test(&mut self) {
        if let Some(first) = self.canonical_state.first_mut() {
            *first ^= 1;
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseProgressionSettlement {
    input: DivergentUniverseInputSnapshotDigest,
    participant_lock: ParticipantLockDigest,
    permanent_unlocks: Box<[u64]>,
    division: Option<DivergentUniverseDivisionId>,
    cognoculi: u16,
    finish_conditions: Box<[DivergentUniverseFinishConditionId]>,
}

impl DivergentUniverseProgressionSettlement {
    #[must_use]
    pub const fn input(&self) -> DivergentUniverseInputSnapshotDigest {
        self.input
    }

    #[must_use]
    pub const fn participant_lock(&self) -> ParticipantLockDigest {
        self.participant_lock
    }

    #[must_use]
    pub fn permanent_unlocks(&self) -> &[u64] {
        &self.permanent_unlocks
    }

    #[must_use]
    pub const fn division(&self) -> Option<&DivergentUniverseDivisionId> {
        self.division.as_ref()
    }

    #[must_use]
    pub const fn cognoculi(&self) -> u16 {
        self.cognoculi
    }

    #[must_use]
    pub fn finish_conditions(&self) -> &[DivergentUniverseFinishConditionId] {
        &self.finish_conditions
    }
}

impl DivergentUniverseFlowInstance {
    /// Captures the exact command-boundary state without advancing the graph or RNG.
    pub fn capture_save_snapshot(
        &self,
        activity: &GraphActivity,
    ) -> Result<DivergentUniverseSaveSnapshot, DivergentUniverseSnapshotError> {
        self.validate_activity(activity)?;
        let canonical_state = activity.canonical_state_bytes();
        let state_hash = activity.state_hash();
        if CanonicalDigestBuilder::digest(&canonical_state).as_slice() != state_hash.bytes() {
            return Err(DivergentUniverseSnapshotError::StateHashMismatch);
        }
        let mut snapshot = DivergentUniverseSaveSnapshot {
            component_digest: self.component_digest,
            definition: self.definition.identity(),
            input: self.input_snapshot.digest(),
            participants: self.definition.participants().digest(),
            instance: activity.instance(),
            state_hash,
            canonical_state,
            digest: DivergentUniverseSaveSnapshotDigest::new([1; 32])
                .expect("non-zero placeholder"),
        };
        snapshot.digest = snapshot.compute_digest();
        Ok(snapshot)
    }

    /// Validates a captured snapshot against fresh current-tree inputs. Actual
    /// reconstruction from canonical bytes is owned by G22-P7-B5.
    pub fn validate_save_snapshot(
        &self,
        snapshot: &DivergentUniverseSaveSnapshot,
    ) -> Result<(), DivergentUniverseSnapshotError> {
        if snapshot.component_digest != self.component_digest {
            return Err(DivergentUniverseSnapshotError::ComponentMismatch);
        }
        if snapshot.definition != self.definition.identity() {
            return Err(DivergentUniverseSnapshotError::DefinitionMismatch);
        }
        if snapshot.input != self.input_snapshot.digest() {
            return Err(DivergentUniverseSnapshotError::InputDigestMismatch);
        }
        if snapshot.participants != self.definition.participants().digest() {
            return Err(DivergentUniverseSnapshotError::ParticipantSnapshotMismatch);
        }
        let actual_state_hash =
            ActivityStateHash::new(CanonicalDigestBuilder::digest(&snapshot.canonical_state))
                .expect("Activity state hash accepts all SHA-256 outputs");
        if actual_state_hash != snapshot.state_hash {
            return Err(DivergentUniverseSnapshotError::StateHashMismatch);
        }
        if snapshot.compute_digest() != snapshot.digest {
            return Err(DivergentUniverseSnapshotError::SaveDigestMismatch);
        }
        Ok(())
    }

    /// Produces immutable account-consumer input only after successful terminal
    /// settlement; it does not query or mutate account state.
    pub fn progression_settlement(
        &self,
        activity: &GraphActivity,
    ) -> Result<DivergentUniverseProgressionSettlement, DivergentUniverseSnapshotError> {
        self.validate_activity(activity)?;
        if activity.player_view().terminal() != Some(ActivityTerminalOutcome::Completed) {
            return Err(DivergentUniverseSnapshotError::ActivityNotCompleted);
        }
        Ok(DivergentUniverseProgressionSettlement {
            input: self.input_snapshot.digest(),
            participant_lock: self.definition.participants().digest(),
            permanent_unlocks: self.permanent_unlocks.clone(),
            division: self.progression.settled_division().cloned(),
            cognoculi: self.progression.settled_cognoculi(),
            finish_conditions: self.finish_conditions.clone(),
        })
    }

    fn validate_activity(
        &self,
        activity: &GraphActivity,
    ) -> Result<(), DivergentUniverseSnapshotError> {
        if activity.definition().identity() != self.definition.identity() {
            return Err(DivergentUniverseSnapshotError::DefinitionMismatch);
        }
        Ok(())
    }
}

pub(super) fn participant_stable_value(participants: &ParticipantLock) -> u64 {
    stable_digest(participants.digest().bytes())
}

fn stable_digest(bytes: [u8; 32]) -> u64 {
    let mut raw = [0_u8; 8];
    raw.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(raw).max(1)
}

fn hash(parts: &[&[u8]]) -> [u8; 32] {
    let mut hash = CanonicalDigestBuilder::new();
    for part in parts {
        hash.update(
            u64::try_from(part.len())
                .expect("slice length fits u64")
                .to_le_bytes(),
        );
        hash.update(part);
    }
    hash.finalize()
}

const fn all_zero(bytes: &[u8; 32]) -> bool {
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != 0 {
            return false;
        }
        index += 1;
    }
    true
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseSnapshotError {
    ZeroDigest,
    ParticipantSnapshotMismatch,
    InputDigestMismatch,
    DefinitionMismatch,
    ComponentMismatch,
    StateHashMismatch,
    SaveDigestMismatch,
    ActivityNotCompleted,
}

impl std::fmt::Display for DivergentUniverseSnapshotError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid Divergent Universe snapshot: {self:?}")
    }
}

impl std::error::Error for DivergentUniverseSnapshotError {}
