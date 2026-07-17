//! Immutable lookup and enabled-authoring audit.

use starclock_combat::NativeHandlerId;

use crate::model::{
    BattleHandlerRegistration, HandlerDomain, NativeHandlerRequirement, RegistryError,
    RegistryErrorKind,
};

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
            if registration.version == 0
                || registration
                    .argument_schema_digest
                    .iter()
                    .all(|byte| *byte == 0)
                || registration.determinism_note.is_empty()
                || registration.owner.is_empty()
                || registration.ir_insufficiency.is_empty()
                || registration.removal_condition.is_empty()
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

    pub fn audit(self, requirements: &[NativeHandlerRequirement]) -> Result<(), RegistryError> {
        for requirement in requirements {
            if !requirement.enabled {
                continue;
            }
            if !requirement.has_ir_insufficiency_decision {
                return Err(registry_error(
                    RegistryErrorKind::MissingIrInsufficiencyDecision,
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
        }
        Ok(())
    }
}

fn registry_error(kind: RegistryErrorKind, handler: Option<NativeHandlerId>) -> RegistryError {
    RegistryError { kind, handler }
}
