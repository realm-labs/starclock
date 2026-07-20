use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};

use crate::{BuildDigest, ParticipantId, ParticipantLockDigest, codec::CanonicalWriter};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ParticipantUniquenessScope {
    Team = 0,
    Node = 1,
    Section = 2,
    Activity = 3,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum LoadoutLockScope {
    Activity = 0,
    Section = 1,
    Node = 2,
    Attempt = 3,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ParticipantSourceKind {
    CompiledBuild = 0,
    FixedResolved = 1,
    Trial = 2,
    Synthetic = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParticipantPolicy {
    team_count: u8,
    minimum_team_size: u8,
    maximum_team_size: u8,
    uniqueness: ParticipantUniquenessScope,
    lock_scope: LoadoutLockScope,
}

impl ParticipantPolicy {
    #[must_use]
    pub const fn new(
        team_count: u8,
        minimum_team_size: u8,
        maximum_team_size: u8,
        uniqueness: ParticipantUniquenessScope,
        lock_scope: LoadoutLockScope,
    ) -> Option<Self> {
        if team_count >= 1
            && team_count <= 8
            && minimum_team_size >= 1
            && minimum_team_size <= maximum_team_size
            && maximum_team_size <= 8
        {
            Some(Self {
                team_count,
                minimum_team_size,
                maximum_team_size,
                uniqueness,
                lock_scope,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn loadout_lock_scope(self) -> LoadoutLockScope {
        self.lock_scope
    }
}

/// Opaque upstream build/resolved-spec identity retained only for locking.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpaqueParticipantBuild {
    resolved_spec: CombatantSpecDigest,
    build: BuildDigest,
    build_catalog_revision: Box<str>,
    source: ParticipantSourceKind,
}

impl OpaqueParticipantBuild {
    pub fn new(
        resolved_spec: CombatantSpecDigest,
        build: BuildDigest,
        build_catalog_revision: impl Into<Box<str>>,
        source: ParticipantSourceKind,
    ) -> Result<Self, ParticipantLockError> {
        let revision = build_catalog_revision.into();
        if revision.is_empty()
            || revision.len() > 80
            || !revision.bytes().all(|byte| byte.is_ascii_graphic())
        {
            return Err(ParticipantLockError::InvalidCatalogRevision);
        }
        Ok(Self {
            resolved_spec,
            build,
            build_catalog_revision: revision,
            source,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantLockEntry {
    participant: ParticipantId,
    team_index: u8,
    formation_index: u8,
    character: UnitDefinitionId,
    build: OpaqueParticipantBuild,
}

impl ParticipantLockEntry {
    pub fn new(
        participant: ParticipantId,
        team_index: u8,
        formation_index: u8,
        character: UnitDefinitionId,
        build: OpaqueParticipantBuild,
    ) -> Result<Self, ParticipantLockError> {
        if team_index > 7 || formation_index > 7 {
            return Err(ParticipantLockError::FormationOutOfRange);
        }
        Ok(Self {
            participant,
            team_index,
            formation_index,
            character,
            build,
        })
    }

    pub(crate) fn encode(&self, writer: &mut CanonicalWriter) {
        writer.u32(self.participant.get());
        writer.byte(self.team_index);
        writer.byte(self.formation_index);
        writer.u32(self.character.get());
        writer.digest(self.build.resolved_spec.bytes());
        writer.digest(self.build.build.bytes());
        writer.text(&self.build.build_catalog_revision);
        writer.byte(self.build.source as u8);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantLock {
    policy: ParticipantPolicy,
    entries: Box<[ParticipantLockEntry]>,
    digest: ParticipantLockDigest,
}

impl ParticipantLock {
    pub fn seal(
        policy: ParticipantPolicy,
        entries: Vec<ParticipantLockEntry>,
    ) -> Result<Self, ParticipantLockError> {
        let digest = Self::digest_for(policy, &entries)?;
        Ok(Self {
            policy,
            entries: canonical_entries(entries)?.into_boxed_slice(),
            digest,
        })
    }

    pub fn new(
        policy: ParticipantPolicy,
        entries: Vec<ParticipantLockEntry>,
        claimed_digest: ParticipantLockDigest,
    ) -> Result<Self, ParticipantLockError> {
        let actual = Self::digest_for(policy, &entries)?;
        if actual != claimed_digest {
            return Err(ParticipantLockError::DigestMismatch);
        }
        Ok(Self {
            policy,
            entries: canonical_entries(entries)?.into_boxed_slice(),
            digest: actual,
        })
    }

    pub fn digest_for(
        policy: ParticipantPolicy,
        entries: &[ParticipantLockEntry],
    ) -> Result<ParticipantLockDigest, ParticipantLockError> {
        let entries = canonical_entries(entries.to_vec())?;
        validate_policy(policy, &entries)?;
        let mut writer = CanonicalWriter::new(b"starclock-participant-lock-v1");
        encode_policy(policy, &mut writer);
        writer.u64(entries.len() as u64);
        for entry in &entries {
            entry.encode(&mut writer);
        }
        Ok(ParticipantLockDigest::new(writer.finish()).expect("SHA-256 output is non-zero"))
    }

    #[must_use]
    pub const fn policy(&self) -> ParticipantPolicy {
        self.policy
    }
    #[must_use]
    pub fn entries(&self) -> &[ParticipantLockEntry] {
        &self.entries
    }
    #[must_use]
    pub const fn digest(&self) -> ParticipantLockDigest {
        self.digest
    }

    pub(crate) fn encode(&self, writer: &mut CanonicalWriter) {
        encode_policy(self.policy, writer);
        writer.digest(self.digest.bytes());
        writer.u64(self.entries.len() as u64);
        for entry in &self.entries {
            entry.encode(writer);
        }
    }
}

fn canonical_entries(
    mut entries: Vec<ParticipantLockEntry>,
) -> Result<Vec<ParticipantLockEntry>, ParticipantLockError> {
    if entries.is_empty() || entries.len() > 64 {
        return Err(ParticipantLockError::InvalidParticipantCount);
    }
    entries.sort_by_key(|entry| (entry.team_index, entry.formation_index, entry.participant));
    if entries.windows(2).any(|pair| {
        pair[0].team_index == pair[1].team_index
            && pair[0].formation_index == pair[1].formation_index
    }) {
        return Err(ParticipantLockError::DuplicateFormation);
    }
    if entries.iter().enumerate().any(|(index, entry)| {
        entries[..index]
            .iter()
            .any(|prior| prior.participant == entry.participant)
    }) {
        return Err(ParticipantLockError::DuplicateParticipant);
    }
    Ok(entries)
}

fn validate_policy(
    policy: ParticipantPolicy,
    entries: &[ParticipantLockEntry],
) -> Result<(), ParticipantLockError> {
    for team in 0..policy.team_count {
        let count = entries
            .iter()
            .filter(|entry| entry.team_index == team)
            .count();
        if count < usize::from(policy.minimum_team_size)
            || count > usize::from(policy.maximum_team_size)
        {
            return Err(ParticipantLockError::TeamSizeOutsidePolicy);
        }
    }
    if entries
        .iter()
        .any(|entry| entry.team_index >= policy.team_count)
    {
        return Err(ParticipantLockError::TeamOutsidePolicy);
    }
    let duplicate_character = entries.iter().enumerate().any(|(index, entry)| {
        entries[..index].iter().any(|prior| {
            prior.character == entry.character
                && (policy.uniqueness != ParticipantUniquenessScope::Team
                    || prior.team_index == entry.team_index)
        })
    });
    if duplicate_character {
        return Err(ParticipantLockError::DuplicateCharacter);
    }
    Ok(())
}

fn encode_policy(policy: ParticipantPolicy, writer: &mut CanonicalWriter) {
    writer.byte(policy.team_count);
    writer.byte(policy.minimum_team_size);
    writer.byte(policy.maximum_team_size);
    writer.byte(policy.uniqueness as u8);
    writer.byte(policy.lock_scope as u8);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParticipantLockError {
    FormationOutOfRange,
    InvalidCatalogRevision,
    InvalidParticipantCount,
    DuplicateFormation,
    DuplicateParticipant,
    DuplicateCharacter,
    TeamSizeOutsidePolicy,
    TeamOutsidePolicy,
    DigestMismatch,
}
