use crate::{
    codec::CodecError,
    digest::{BuildCatalogDigest, CombatantBuildDigest, DefinitionDigest, EntrySpecDigest},
    record::ReplayFormatError,
};

pub const MAX_BUILD_BINDINGS: u32 = 1024;

/// Build identities bound to an activity entry in participant order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildBindings {
    catalog_digest: BuildCatalogDigest,
    combatants: Box<[CombatantBuildDigest]>,
}

impl BuildBindings {
    pub fn new(
        catalog_digest: BuildCatalogDigest,
        combatants: Vec<CombatantBuildDigest>,
    ) -> Result<Self, ReplayFormatError> {
        if combatants.len() > MAX_BUILD_BINDINGS as usize {
            return Err(CodecError::LimitExceeded.into());
        }
        Ok(Self {
            catalog_digest,
            combatants: combatants.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn catalog_digest(&self) -> BuildCatalogDigest {
        self.catalog_digest
    }

    #[must_use]
    pub fn combatants(&self) -> &[CombatantBuildDigest] {
        &self.combatants
    }
}

/// Initial battle or activity identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayEntry {
    Battle {
        definition_id: u32,
        spec_digest: EntrySpecDigest,
    },
    Activity {
        profile_id: Box<str>,
        definition_id: u32,
        definition_digest: DefinitionDigest,
        spec_digest: EntrySpecDigest,
        builds: Option<BuildBindings>,
    },
}
