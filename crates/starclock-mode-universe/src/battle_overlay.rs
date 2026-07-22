//! Resolved roster-specific battle inputs kept outside generated Universe rows.

use std::sync::Arc;

use crate::{
    digest::{Encoder, UniverseEncounterOverlayDigest},
    id::EncounterMemberId,
};
use starclock_activity::{
    ActivityBattleResultContract, EncounterPreparationDefinition, ParticipantLockDigest,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniverseEncounterBattleBinding {
    member: EncounterMemberId,
    preparation: Arc<EncounterPreparationDefinition>,
    contract: Arc<ActivityBattleResultContract>,
}

impl UniverseEncounterBattleBinding {
    #[must_use]
    pub fn new(
        member: EncounterMemberId,
        preparation: Arc<EncounterPreparationDefinition>,
        contract: Arc<ActivityBattleResultContract>,
    ) -> Self {
        Self {
            member,
            preparation,
            contract,
        }
    }
    #[must_use]
    pub const fn member(&self) -> EncounterMemberId {
        self.member
    }
    #[must_use]
    pub const fn preparation(&self) -> &Arc<EncounterPreparationDefinition> {
        &self.preparation
    }
    #[must_use]
    pub const fn contract(&self) -> &Arc<ActivityBattleResultContract> {
        &self.contract
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniverseEncounterOverlay {
    bindings: Box<[UniverseEncounterBattleBinding]>,
    digest: UniverseEncounterOverlayDigest,
}

impl UniverseEncounterOverlay {
    pub fn new(
        mut bindings: Vec<UniverseEncounterBattleBinding>,
    ) -> Result<Self, UniverseEncounterOverlayError> {
        bindings.sort_by_key(UniverseEncounterBattleBinding::member);
        if bindings.is_empty() {
            return Err(UniverseEncounterOverlayError::Empty);
        }
        if bindings
            .windows(2)
            .any(|pair| pair[0].member == pair[1].member)
        {
            return Err(UniverseEncounterOverlayError::DuplicateMember);
        }
        let mut specs = Vec::new();
        for binding in &bindings {
            for variant in binding.preparation.variants() {
                let digest = variant.battle_spec().digest();
                if specs.contains(&digest) {
                    return Err(UniverseEncounterOverlayError::DuplicateBattleSpec);
                }
                specs.push(digest);
            }
        }
        let digest = overlay_digest(&bindings);
        Ok(Self {
            bindings: bindings.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub fn bindings(&self) -> &[UniverseEncounterBattleBinding] {
        &self.bindings
    }
    #[must_use]
    pub const fn digest(&self) -> UniverseEncounterOverlayDigest {
        self.digest
    }
    #[must_use]
    pub fn binding(&self, member: EncounterMemberId) -> Option<&UniverseEncounterBattleBinding> {
        self.bindings
            .binary_search_by_key(&member, UniverseEncounterBattleBinding::member)
            .ok()
            .map(|index| &self.bindings[index])
    }
    #[must_use]
    pub fn binding_for_spec(&self, digest: [u8; 32]) -> Option<&UniverseEncounterBattleBinding> {
        self.bindings.iter().find(|binding| {
            binding
                .preparation
                .variants()
                .iter()
                .any(|variant| variant.battle_spec().digest().bytes() == digest)
        })
    }
    #[must_use]
    pub fn participant_lock_digest(&self) -> Option<ParticipantLockDigest> {
        let first = self.bindings.first()?.preparation.participant_lock_digest();
        self.bindings
            .iter()
            .all(|binding| binding.preparation.participant_lock_digest() == first)
            .then_some(first)
    }
}

fn overlay_digest(bindings: &[UniverseEncounterBattleBinding]) -> UniverseEncounterOverlayDigest {
    let mut encoder = Encoder::new(b"starclock-universe-encounter-overlay-v1");
    encoder.u32(bindings.len() as u32);
    for binding in bindings {
        encoder.u32(binding.member.get());
        encoder.digest(binding.preparation.digest().bytes());
        encoder.digest(binding.contract.digest().bytes());
    }
    UniverseEncounterOverlayDigest::new(encoder.finish())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UniverseEncounterOverlayError {
    Empty,
    DuplicateMember,
    DuplicateBattleSpec,
}
