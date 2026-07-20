//! Immutable lookup and enabled-authoring audit.

use starclock_combat::NativeHandlerId;

use crate::model::{
    BattleHandlerRegistration, HandlerDomain, NativeHandlerRequirement, RegistryError,
    RegistryErrorKind,
};

pub const PRODUCTION_REGISTRY_REVISION: &str = "native-registry-v1";
static PRODUCTION_BATTLE_HANDLERS: [BattleHandlerRegistration; 0] = [];

/// Returns the immutable registry compiled into this Starclock build.
///
/// Goal 01 V1a admits no production handler: all reviewed probes lower to the
/// typed Rule IR. Future registrations must be added here and to the committed
/// native-handler audit in the same batch.
pub fn production() -> NativeHandlerRegistry {
    NativeHandlerRegistry::new(PRODUCTION_REGISTRY_REVISION, &PRODUCTION_BATTLE_HANDLERS)
        .expect("the static production native-handler registry is valid")
}

#[derive(Clone, Copy, Debug)]
pub struct NativeHandlerRegistry {
    revision: &'static str,
    battle: &'static [BattleHandlerRegistration],
}

impl NativeHandlerRegistry {
    pub fn new(
        revision: &'static str,
        battle: &'static [BattleHandlerRegistration],
    ) -> Result<Self, RegistryError> {
        if revision.is_empty()
            || revision.len() > 128
            || !revision.bytes().all(|byte| byte.is_ascii_graphic())
        {
            return Err(registry_error(RegistryErrorKind::InvalidRevision, None));
        }
        for (index, registration) in battle.iter().enumerate() {
            if index > 0 && battle[index - 1].id >= registration.id {
                return Err(registry_error(
                    RegistryErrorKind::NonCanonicalRegistration,
                    Some(registration.id),
                ));
            }
            if !metadata_present(registration.stable_key)
                || !registration
                    .stable_key
                    .bytes()
                    .all(|byte| byte.is_ascii_graphic())
                || !metadata_present(registration.version)
                || registration
                    .argument_schema_digest
                    .iter()
                    .all(|byte| *byte == 0)
                || !metadata_present(registration.determinism_note)
                || !metadata_present(registration.owner)
                || !metadata_present(registration.ir_insufficiency)
                || !metadata_present(registration.removal_condition)
            {
                return Err(registry_error(
                    RegistryErrorKind::InvalidRegistration,
                    Some(registration.id),
                ));
            }
        }
        Ok(Self { revision, battle })
    }

    #[must_use]
    pub const fn revision(self) -> &'static str {
        self.revision
    }

    #[must_use]
    pub fn battle(self, id: NativeHandlerId) -> Option<&'static BattleHandlerRegistration> {
        self.battle
            .binary_search_by_key(&id, |registration| registration.id)
            .ok()
            .map(|index| &self.battle[index])
    }

    pub fn audit(self, requirements: &[NativeHandlerRequirement<'_>]) -> Result<(), RegistryError> {
        for requirement in requirements {
            if !requirement.enabled {
                continue;
            }
            if !metadata_present(requirement.ir_insufficiency) {
                return Err(registry_error(
                    RegistryErrorKind::MissingIrInsufficiencyDecision,
                    Some(requirement.id),
                ));
            }
            if !metadata_present(requirement.stable_key)
                || !metadata_present(requirement.version)
                || requirement
                    .argument_schema_digest
                    .iter()
                    .all(|byte| *byte == 0)
                || !metadata_present(requirement.determinism_note)
                || !metadata_present(requirement.owner)
                || !metadata_present(requirement.removal_condition)
            {
                return Err(registry_error(
                    RegistryErrorKind::InvalidRequirement,
                    Some(requirement.id),
                ));
            }
            if requirement.domain != HandlerDomain::Battle {
                return Err(registry_error(
                    RegistryErrorKind::UnsupportedDomain,
                    Some(requirement.id),
                ));
            }
            let registration = self.battle(requirement.id).ok_or_else(|| {
                registry_error(RegistryErrorKind::MissingRegistration, Some(requirement.id))
            })?;
            if registration.stable_key != requirement.stable_key {
                return Err(registry_error(
                    RegistryErrorKind::StableKeyMismatch,
                    Some(requirement.id),
                ));
            }
            if registration.version != requirement.version {
                return Err(registry_error(
                    RegistryErrorKind::VersionMismatch,
                    Some(requirement.id),
                ));
            }
            if registration.argument_schema_digest != requirement.argument_schema_digest {
                return Err(registry_error(
                    RegistryErrorKind::ArgumentSchemaMismatch,
                    Some(requirement.id),
                ));
            }
            if registration.determinism_note != requirement.determinism_note {
                return Err(registry_error(
                    RegistryErrorKind::DeterminismNoteMismatch,
                    Some(requirement.id),
                ));
            }
            if registration.owner != requirement.owner {
                return Err(registry_error(
                    RegistryErrorKind::OwnerMismatch,
                    Some(requirement.id),
                ));
            }
            if registration.ir_insufficiency != requirement.ir_insufficiency {
                return Err(registry_error(
                    RegistryErrorKind::IrInsufficiencyMismatch,
                    Some(requirement.id),
                ));
            }
            if registration.removal_condition != requirement.removal_condition {
                return Err(registry_error(
                    RegistryErrorKind::RemovalConditionMismatch,
                    Some(requirement.id),
                ));
            }
        }
        Ok(())
    }
}

fn registry_error(kind: RegistryErrorKind, handler: Option<NativeHandlerId>) -> RegistryError {
    RegistryError { kind, handler }
}

fn metadata_present(value: &str) -> bool {
    !value.trim().is_empty()
}
