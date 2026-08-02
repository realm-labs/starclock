//! Immutable projection of every current Activity contribution visible to battle assembly.

use starclock_activity::{
    ActivityParticipantCarryState, ActivityStateHash, ParticipantLock, ParticipantLockDigest,
};

use crate::{
    ability_runtime::{AbilityExecutionContext, AbilityRuntimeProjection},
    battle_contribution::UniverseBattleContributionSet,
    blessing_runtime::BlessingContributionSet,
    curio_runtime::CurioContributionSet,
    digest::Encoder,
    path_runtime::PathContributionSet,
    run_runtime::AbilityTreeContributionSet,
};

/// One self-contained, immutable read of battle-relevant Activity state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardUniverseBattleSnapshot {
    source_state_hash: ActivityStateHash,
    participant_lock: ParticipantLockDigest,
    context: AbilityExecutionContext,
    path: PathContributionSet,
    blessings: BlessingContributionSet,
    curios: CurioContributionSet,
    ability_tree: AbilityTreeContributionSet,
    ability_projection: AbilityRuntimeProjection,
    contributions: UniverseBattleContributionSet,
    participant_carry: Box<[ActivityParticipantCarryState]>,
    carry_digest: [u8; 32],
    digest: [u8; 32],
}

impl StandardUniverseBattleSnapshot {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        source_state_hash: ActivityStateHash,
        participant_lock: &ParticipantLock,
        context: AbilityExecutionContext,
        path: PathContributionSet,
        blessings: BlessingContributionSet,
        curios: CurioContributionSet,
        ability_tree: AbilityTreeContributionSet,
        ability_projection: AbilityRuntimeProjection,
        contributions: UniverseBattleContributionSet,
        participant_carry: &[ActivityParticipantCarryState],
    ) -> Result<Self, StandardUniverseBattleSnapshotError> {
        if context.chosen_path_blessings() != path.selected_path_blessings()
            || participant_carry
                .windows(2)
                .any(|pair| pair[0].participant() >= pair[1].participant())
            || participant_carry.iter().any(|carry| {
                participant_lock
                    .entries()
                    .binary_search_by_key(&carry.participant(), |entry| entry.participant())
                    .is_err()
                    || carry.current_hp() > carry.maximum_hp()
                    || carry.current_energy() > carry.maximum_energy()
            })
        {
            return Err(StandardUniverseBattleSnapshotError::InvalidProjection);
        }
        let carry_digest = encode_carry(participant_carry);
        let digest = snapshot_digest(
            source_state_hash,
            participant_lock.digest(),
            context,
            &path,
            &blessings,
            &curios,
            &ability_tree,
            &ability_projection,
            &contributions,
            carry_digest,
        );
        Ok(Self {
            source_state_hash,
            participant_lock: participant_lock.digest(),
            context,
            path,
            blessings,
            curios,
            ability_tree,
            ability_projection,
            contributions,
            participant_carry: participant_carry.to_vec().into_boxed_slice(),
            carry_digest,
            digest,
        })
    }

    #[must_use]
    pub const fn source_state_hash(&self) -> ActivityStateHash {
        self.source_state_hash
    }

    #[must_use]
    pub const fn participant_lock(&self) -> ParticipantLockDigest {
        self.participant_lock
    }

    #[must_use]
    pub const fn context(&self) -> AbilityExecutionContext {
        self.context
    }

    #[must_use]
    pub const fn path(&self) -> &PathContributionSet {
        &self.path
    }

    #[must_use]
    pub const fn blessings(&self) -> &BlessingContributionSet {
        &self.blessings
    }

    #[must_use]
    pub const fn curios(&self) -> &CurioContributionSet {
        &self.curios
    }

    #[must_use]
    pub const fn ability_tree(&self) -> &AbilityTreeContributionSet {
        &self.ability_tree
    }

    #[must_use]
    pub const fn ability_projection(&self) -> &AbilityRuntimeProjection {
        &self.ability_projection
    }

    #[must_use]
    pub const fn contributions(&self) -> &UniverseBattleContributionSet {
        &self.contributions
    }

    #[must_use]
    pub fn participant_carry(&self) -> &[ActivityParticipantCarryState] {
        &self.participant_carry
    }

    #[must_use]
    pub const fn carry_digest(&self) -> [u8; 32] {
        self.carry_digest
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[must_use]
    pub fn into_contributions(self) -> UniverseBattleContributionSet {
        self.contributions
    }
}

#[allow(clippy::too_many_arguments)]
fn snapshot_digest(
    source_state_hash: ActivityStateHash,
    participant_lock: ParticipantLockDigest,
    context: AbilityExecutionContext,
    path: &PathContributionSet,
    blessings: &BlessingContributionSet,
    curios: &CurioContributionSet,
    ability_tree: &AbilityTreeContributionSet,
    ability_projection: &AbilityRuntimeProjection,
    contributions: &UniverseBattleContributionSet,
    carry_digest: [u8; 32],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.standard-universe.battle-contribution-snapshot");
    encoder.digest(source_state_hash.bytes());
    encoder.digest(participant_lock.bytes());
    encoder.u8(context.scope() as u8);
    encoder.u8(context.boundary() as u8);
    encoder.u8(context.chosen_path_blessings());
    encoder.u8(u8::from(context.first_battle_won()));
    encoder.digest(path.digest());
    encoder.digest(blessings.digest());
    encoder.digest(curios.digest());
    encoder.digest(ability_tree.digest());
    encoder.digest(ability_projection.digest());
    encoder.digest(contributions.digest());
    encoder.digest(carry_digest);
    encoder.finish()
}

fn encode_carry(carry: &[ActivityParticipantCarryState]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.standard-universe.battle-carry.v1");
    encoder.u32(u32::try_from(carry.len()).expect("participant roster is bounded"));
    for state in carry {
        encoder.u32(state.participant().get());
        encoder.i64(state.current_hp().get());
        encoder.i64(state.maximum_hp().get());
        encoder.i64(state.current_energy().scaled());
        encoder.i64(state.maximum_energy().scaled());
        encoder.u8(state.life() as u8);
        encoder.u8(state.presence() as u8);
    }
    encoder.finish()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseBattleSnapshotError {
    InvalidProjection,
}

impl core::fmt::Display for StandardUniverseBattleSnapshotError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Standard Universe battle snapshot error: {self:?}"
        )
    }
}

impl std::error::Error for StandardUniverseBattleSnapshotError {}
