//! Arithmetic Mapping over immutable caller and temporary-minimum builds.

use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{ParticipantId, ParticipantLock, ParticipantLockDigest};
use starclock_build::{
    compiler::{BuildCompileError, LoadoutCompiler},
    output::CompiledBuild,
    spec::CombatantBuildSpec,
    substitution::{
        BuildFieldSource, BuildSubstitutionError, BuildSubstitutionKind, BuildSubstitutionReceipt,
        OwnedBuildMinimumFacts, substitute_owned_or_trial,
    },
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_mapping_catalog::{
        DivergentUniverseAvatarLocator, DivergentUniverseMappingBuildId,
        DivergentUniverseMappingCatalog, DivergentUniverseMappingDecimal,
        DivergentUniversePublicIdentityResolution,
    },
};

use super::entry_flow::DivergentUniverseRuntimeFactory;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseTemporaryMinimumAccuracy {
    VersionedProjectPolicyCallerProvided,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseAvatarBindingAccuracy {
    CoreCatalogResolved,
    VersionedProjectPolicyCallerProvided,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMappingInput {
    participant: ParticipantId,
    avatar: DivergentUniverseAvatarLocator,
    owned: Option<(CombatantBuildSpec, OwnedBuildMinimumFacts)>,
    temporary_minimum: CombatantBuildSpec,
    accuracy: DivergentUniverseTemporaryMinimumAccuracy,
}

impl DivergentUniverseMappingInput {
    #[must_use]
    pub fn new(
        participant: ParticipantId,
        avatar: DivergentUniverseAvatarLocator,
        owned: Option<(CombatantBuildSpec, OwnedBuildMinimumFacts)>,
        temporary_minimum: CombatantBuildSpec,
    ) -> Self {
        Self {
            participant,
            avatar,
            owned,
            temporary_minimum,
            accuracy:
                DivergentUniverseTemporaryMinimumAccuracy::VersionedProjectPolicyCallerProvided,
        }
    }

    #[must_use]
    pub const fn participant(&self) -> ParticipantId {
        self.participant
    }

    #[must_use]
    pub const fn avatar(&self) -> &DivergentUniverseAvatarLocator {
        &self.avatar
    }

    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseTemporaryMinimumAccuracy {
        self.accuracy
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMappedParticipant {
    participant: ParticipantId,
    avatar: DivergentUniverseAvatarLocator,
    mapping_build: DivergentUniverseMappingBuildId,
    role_buff_id: Box<str>,
    role_buff_parameters: Box<[DivergentUniverseMappingDecimal]>,
    avatar_binding_accuracy: DivergentUniverseAvatarBindingAccuracy,
    compiled: CompiledBuild,
    receipt: BuildSubstitutionReceipt,
}

impl DivergentUniverseMappedParticipant {
    #[must_use]
    pub const fn participant(&self) -> ParticipantId {
        self.participant
    }
    #[must_use]
    pub const fn avatar(&self) -> &DivergentUniverseAvatarLocator {
        &self.avatar
    }
    #[must_use]
    pub const fn mapping_build(&self) -> &DivergentUniverseMappingBuildId {
        &self.mapping_build
    }
    #[must_use]
    pub fn role_buff_id(&self) -> &str {
        &self.role_buff_id
    }
    #[must_use]
    pub fn role_buff_parameters(&self) -> &[DivergentUniverseMappingDecimal] {
        &self.role_buff_parameters
    }
    #[must_use]
    pub const fn avatar_binding_accuracy(&self) -> DivergentUniverseAvatarBindingAccuracy {
        self.avatar_binding_accuracy
    }
    #[must_use]
    pub const fn compiled(&self) -> &CompiledBuild {
        &self.compiled
    }
    #[must_use]
    pub const fn receipt(&self) -> BuildSubstitutionReceipt {
        self.receipt
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DivergentUniverseMappingSnapshotDigest([u8; 32]);

impl DivergentUniverseMappingSnapshotDigest {
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMappingSnapshot {
    component_digest: [u8; 32],
    participant_lock: ParticipantLockDigest,
    participants: Box<[DivergentUniverseMappedParticipant]>,
    digest: DivergentUniverseMappingSnapshotDigest,
}

impl DivergentUniverseMappingSnapshot {
    #[must_use]
    pub const fn participant_lock(&self) -> ParticipantLockDigest {
        self.participant_lock
    }
    #[must_use]
    pub fn participants(&self) -> &[DivergentUniverseMappedParticipant] {
        &self.participants
    }
    #[must_use]
    pub const fn digest(&self) -> DivergentUniverseMappingSnapshotDigest {
        self.digest
    }

    #[must_use]
    pub fn teardown(self) -> DivergentUniverseMappingTeardown {
        DivergentUniverseMappingTeardown {
            participant_lock: self.participant_lock,
            removed_snapshot: self.digest,
            removed_participants: self.participants.len(),
        }
    }

    pub(super) fn validate(
        &self,
        component_digest: [u8; 32],
        participants: &ParticipantLock,
    ) -> Result<(), DivergentUniverseMappingRuntimeError> {
        if self.component_digest != component_digest {
            return Err(DivergentUniverseMappingRuntimeError::ComponentMismatch);
        }
        if self.participant_lock != participants.digest() {
            return Err(DivergentUniverseMappingRuntimeError::ParticipantLockMismatch);
        }
        Ok(())
    }

    pub(super) fn state_values(&self) -> Box<[(u64, i64)]> {
        self.participants
            .iter()
            .map(|entry| {
                let bytes = entry.compiled.build_digest().bytes();
                let mut raw = [0_u8; 8];
                raw.copy_from_slice(&bytes[..8]);
                (u64::from(entry.participant.get()), stable_i64(raw))
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMappingTeardown {
    participant_lock: ParticipantLockDigest,
    removed_snapshot: DivergentUniverseMappingSnapshotDigest,
    removed_participants: usize,
}

impl DivergentUniverseMappingTeardown {
    #[must_use]
    pub const fn participant_lock(self) -> ParticipantLockDigest {
        self.participant_lock
    }
    #[must_use]
    pub const fn removed_snapshot(self) -> DivergentUniverseMappingSnapshotDigest {
        self.removed_snapshot
    }
    #[must_use]
    pub const fn removed_participants(self) -> usize {
        self.removed_participants
    }
}

impl DivergentUniverseRuntimeFactory {
    pub fn compile_mapping(
        &self,
        participants: &ParticipantLock,
        core: &SimulationCatalog,
        mut inputs: Vec<DivergentUniverseMappingInput>,
    ) -> Result<DivergentUniverseMappingSnapshot, DivergentUniverseMappingRuntimeError> {
        inputs.sort_unstable_by_key(DivergentUniverseMappingInput::participant);
        if inputs.len() != participants.entries().len()
            || inputs
                .windows(2)
                .any(|pair| pair[0].participant == pair[1].participant)
        {
            return Err(DivergentUniverseMappingRuntimeError::InputClosureMismatch);
        }
        let catalog = self.bundle.mapping_catalog();
        let mut mapped = Vec::with_capacity(inputs.len());
        for input in inputs {
            mapped.push(compile_participant(catalog, participants, core, input)?);
        }
        let component_digest = self.bundle.identity().component_digest().bytes();
        let digest = mapping_digest(component_digest, participants.digest(), &mapped);
        Ok(DivergentUniverseMappingSnapshot {
            component_digest,
            participant_lock: participants.digest(),
            participants: mapped.into_boxed_slice(),
            digest,
        })
    }

    pub fn refresh_mapping(
        &self,
        previous: &DivergentUniverseMappingSnapshot,
        participants: &ParticipantLock,
        core: &SimulationCatalog,
        inputs: Vec<DivergentUniverseMappingInput>,
    ) -> Result<DivergentUniverseMappingSnapshot, DivergentUniverseMappingRuntimeError> {
        if previous.component_digest != self.bundle.identity().component_digest().bytes() {
            return Err(DivergentUniverseMappingRuntimeError::ComponentMismatch);
        }
        self.compile_mapping(participants, core, inputs)
    }
}

fn compile_participant(
    catalog: &DivergentUniverseMappingCatalog,
    participants: &ParticipantLock,
    core: &SimulationCatalog,
    input: DivergentUniverseMappingInput,
) -> Result<DivergentUniverseMappedParticipant, DivergentUniverseMappingRuntimeError> {
    let locked = participants
        .entries()
        .iter()
        .find(|entry| entry.participant() == input.participant)
        .ok_or(DivergentUniverseMappingRuntimeError::UnknownParticipant)?;
    let definition = catalog
        .build(&input.avatar)
        .ok_or(DivergentUniverseMappingRuntimeError::UnknownAvatar)?;
    if !definition.eligible_catalog_entry {
        return Err(DivergentUniverseMappingRuntimeError::IneligibleAvatar);
    }
    if definition.public_identity != DivergentUniversePublicIdentityResolution::ResolvedAvatarConfig
    {
        return Err(DivergentUniverseMappingRuntimeError::MissingReleasedAvatarIdentity);
    }
    let source_avatar = input
        .avatar
        .as_str()
        .parse::<u32>()
        .map_err(|_| DivergentUniverseMappingRuntimeError::InvalidSourceAvatarLocator)?;
    let avatar_binding_accuracy = match core.character_form_for_source_avatar(source_avatar) {
        Some(resolved_form) if resolved_form == locked.character() => {
            DivergentUniverseAvatarBindingAccuracy::CoreCatalogResolved
        }
        Some(_) => return Err(DivergentUniverseMappingRuntimeError::ParticipantAvatarMismatch),
        None => DivergentUniverseAvatarBindingAccuracy::VersionedProjectPolicyCallerProvided,
    };
    if input.temporary_minimum.form() != locked.character()
        || input
            .owned
            .as_ref()
            .is_some_and(|(owned, _)| owned.form() != locked.character())
    {
        return Err(DivergentUniverseMappingRuntimeError::ParticipantAvatarMismatch);
    }
    if let Some((owned, _)) = &input.owned {
        let compiled =
            LoadoutCompiler.compile(core.build_catalog(), core.combat_catalog(), owned)?;
        if compiled.build_digest().bytes() != locked.build().build_digest().bytes()
            || compiled.combatant().digest() != locked.build().resolved_spec_digest()
        {
            return Err(DivergentUniverseMappingRuntimeError::OwnedBuildLockMismatch);
        }
    }
    let substituted = substitute_owned_or_trial(
        input.owned.as_ref().map(|(spec, facts)| (spec, *facts)),
        &input.temporary_minimum,
    )?;
    let receipt = substituted.receipt();
    let compiled = LoadoutCompiler.compile(
        core.build_catalog(),
        core.combat_catalog(),
        substituted.spec(),
    )?;
    Ok(DivergentUniverseMappedParticipant {
        participant: input.participant,
        avatar: input.avatar,
        mapping_build: definition.id.clone(),
        role_buff_id: definition.role_buff_id.clone(),
        role_buff_parameters: definition.role_buff_parameters.clone(),
        avatar_binding_accuracy,
        compiled,
        receipt,
    })
}

fn mapping_digest(
    component: [u8; 32],
    participants: ParticipantLockDigest,
    mapped: &[DivergentUniverseMappedParticipant],
) -> DivergentUniverseMappingSnapshotDigest {
    let mut hash = CanonicalDigestBuilder::new();
    digest_part(
        &mut hash,
        b"starclock.divergent-universe.mapping-snapshot.v1",
    );
    digest_part(&mut hash, &component);
    digest_part(&mut hash, &participants.bytes());
    for entry in mapped {
        digest_part(&mut hash, &entry.participant.get().to_le_bytes());
        digest_part(&mut hash, entry.avatar.as_str().as_bytes());
        digest_part(&mut hash, entry.mapping_build.as_str().as_bytes());
        digest_part(
            &mut hash,
            &[match entry.avatar_binding_accuracy {
                DivergentUniverseAvatarBindingAccuracy::CoreCatalogResolved => 0,
                DivergentUniverseAvatarBindingAccuracy::VersionedProjectPolicyCallerProvided => 1,
            }],
        );
        digest_part(&mut hash, &entry.compiled.build_digest().bytes());
        digest_part(&mut hash, &entry.compiled.combatant().digest().bytes());
        digest_part(&mut hash, &receipt_bytes(entry.receipt));
    }
    DivergentUniverseMappingSnapshotDigest(hash.finalize())
}

fn receipt_bytes(receipt: BuildSubstitutionReceipt) -> [u8; 7] {
    [
        match receipt.kind() {
            BuildSubstitutionKind::Trial => 0,
            BuildSubstitutionKind::StrengthenedOwned => 1,
        },
        field_source_byte(receipt.progression()),
        field_source_byte(receipt.abilities()),
        field_source_byte(receipt.traces()),
        field_source_byte(receipt.eidolon()),
        field_source_byte(receipt.light_cone()),
        field_source_byte(receipt.contributions()),
    ]
}

const fn field_source_byte(source: BuildFieldSource) -> u8 {
    match source {
        BuildFieldSource::Owned => 0,
        BuildFieldSource::MappedMinimum => 1,
        BuildFieldSource::Combined => 2,
    }
}

fn digest_part(hash: &mut CanonicalDigestBuilder, value: &[u8]) {
    hash.update(
        u64::try_from(value.len())
            .expect("slice length fits u64")
            .to_le_bytes(),
    );
    hash.update(value);
}

fn stable_i64(bytes: [u8; 8]) -> i64 {
    let raw = u64::from_le_bytes(bytes) & i64::MAX as u64;
    i64::try_from(raw)
        .expect("masked stable digest fits i64")
        .max(1)
}

#[derive(Debug)]
pub enum DivergentUniverseMappingRuntimeError {
    InputClosureMismatch,
    UnknownParticipant,
    InvalidSourceAvatarLocator,
    ParticipantAvatarMismatch,
    UnknownAvatar,
    IneligibleAvatar,
    MissingReleasedAvatarIdentity,
    OwnedBuildLockMismatch,
    ParticipantLockMismatch,
    ComponentMismatch,
    BuildSubstitution(BuildSubstitutionError),
    BuildCompile(BuildCompileError),
}

impl From<BuildSubstitutionError> for DivergentUniverseMappingRuntimeError {
    fn from(value: BuildSubstitutionError) -> Self {
        Self::BuildSubstitution(value)
    }
}

impl From<BuildCompileError> for DivergentUniverseMappingRuntimeError {
    fn from(value: BuildCompileError) -> Self {
        Self::BuildCompile(value)
    }
}

impl std::fmt::Display for DivergentUniverseMappingRuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid Divergent Universe Mapping runtime: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseMappingRuntimeError {}
