use core::fmt;

use crate::{
    codec::{CanonicalEncode, CanonicalSink, CodecError, Encoder},
    digest::{ComponentDigest, ComponentRootDigest, Sha256Sink},
};

/// Component-set canonical encoding revision.
pub const COMPONENT_SET_REVISION: u32 = 1;
/// Maximum stable component-ID or revision bytes.
pub const MAX_COMPONENT_TEXT_BYTES: usize = 128;
/// Maximum components consumed by one replay.
pub const MAX_REPLAY_COMPONENTS: usize = 256;

/// Closed component families in canonical compatibility order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ConfigurationComponentKind {
    CombatCatalog = 1,
    BuildCatalog = 2,
    ActivityCore = 3,
    ModeProfile = 4,
    ModeContent = 5,
    ActivityHandlerRegistry = 6,
    CombatRuleRegistry = 7,
    EncounterOverlay = 8,
    Controller = 9,
}

impl TryFrom<u8> for ConfigurationComponentKind {
    type Error = ComponentIdentityError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::CombatCatalog),
            2 => Ok(Self::BuildCatalog),
            3 => Ok(Self::ActivityCore),
            4 => Ok(Self::ModeProfile),
            5 => Ok(Self::ModeContent),
            6 => Ok(Self::ActivityHandlerRegistry),
            7 => Ok(Self::CombatRuleRegistry),
            8 => Ok(Self::EncounterOverlay),
            9 => Ok(Self::Controller),
            other => Err(ComponentIdentityError::UnknownKind(other)),
        }
    }
}

/// Stable identity for one component actually consumed by a simulation.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConfigurationComponentIdentity {
    kind: ConfigurationComponentKind,
    id: Box<str>,
    revision: Box<str>,
    digest: ComponentDigest,
}

impl ConfigurationComponentIdentity {
    pub fn new(
        kind: ConfigurationComponentKind,
        id: impl Into<Box<str>>,
        revision: impl Into<Box<str>>,
        digest: ComponentDigest,
    ) -> Result<Self, ComponentIdentityError> {
        let value = Self {
            kind,
            id: id.into(),
            revision: revision.into(),
            digest,
        };
        validate_text(&value.id)?;
        validate_text(&value.revision)?;
        Ok(value)
    }

    #[must_use]
    pub const fn kind(&self) -> ConfigurationComponentKind {
        self.kind
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    #[must_use]
    pub const fn digest(&self) -> ComponentDigest {
        self.digest
    }
}

impl CanonicalEncode for ConfigurationComponentIdentity {
    fn encode<S: CanonicalSink>(&self, e: &mut Encoder<S>) -> Result<(), CodecError> {
        e.u8(self.kind as u8);
        e.string(&self.id)?;
        e.string(&self.revision)?;
        e.raw(&self.digest.bytes());
        Ok(())
    }
}

/// Strictly ordered, duplicate-free component compatibility manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationComponentSet {
    components: Box<[ConfigurationComponentIdentity]>,
    root: ComponentRootDigest,
}

impl ConfigurationComponentSet {
    pub fn new(
        components: Vec<ConfigurationComponentIdentity>,
    ) -> Result<Self, ComponentIdentityError> {
        if components.is_empty() || components.len() > MAX_REPLAY_COMPONENTS {
            return Err(ComponentIdentityError::ComponentCount);
        }
        if components
            .windows(2)
            .any(|pair| component_key(&pair[0]) >= component_key(&pair[1]))
        {
            return Err(ComponentIdentityError::NonCanonicalOrder);
        }
        let root = calculate_root(&components)?;
        Ok(Self {
            components: components.into_boxed_slice(),
            root,
        })
    }

    #[must_use]
    pub fn components(&self) -> &[ConfigurationComponentIdentity] {
        &self.components
    }

    #[must_use]
    pub const fn root(&self) -> ComponentRootDigest {
        self.root
    }

    pub fn verify_exact(&self, actual: &Self) -> Result<(), Box<ConfigurationComponentDivergence>> {
        let shared = self.components.len().min(actual.components.len());
        for index in 0..shared {
            if self.components[index] != actual.components[index] {
                return Err(Box::new(ConfigurationComponentDivergence {
                    index,
                    expected: Some(self.components[index].clone()),
                    actual: Some(actual.components[index].clone()),
                }));
            }
        }
        if self.components.len() != actual.components.len() {
            return Err(Box::new(ConfigurationComponentDivergence {
                index: shared,
                expected: self.components.get(shared).cloned(),
                actual: actual.components.get(shared).cloned(),
            }));
        }
        Ok(())
    }
}

impl CanonicalEncode for ConfigurationComponentSet {
    fn encode<S: CanonicalSink>(&self, e: &mut Encoder<S>) -> Result<(), CodecError> {
        e.u32(COMPONENT_SET_REVISION);
        e.u32(u32::try_from(self.components.len()).map_err(|_| CodecError::LengthOverflow)?);
        for component in &self.components {
            component.encode(e)?;
        }
        e.raw(&self.root.bytes());
        Ok(())
    }
}

/// First exact component mismatch encountered during compatibility verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationComponentDivergence {
    pub index: usize,
    pub expected: Option<ConfigurationComponentIdentity>,
    pub actual: Option<ConfigurationComponentIdentity>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentIdentityError {
    InvalidText,
    ComponentCount,
    NonCanonicalOrder,
    UnknownKind(u8),
    RootMismatch,
    Codec(CodecError),
}

impl From<CodecError> for ComponentIdentityError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}

impl fmt::Display for ComponentIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "configuration component identity error: {self:?}"
        )
    }
}

impl std::error::Error for ComponentIdentityError {}

fn component_key(component: &ConfigurationComponentIdentity) -> (ConfigurationComponentKind, &str) {
    (component.kind, &component.id)
}

fn calculate_root(
    components: &[ConfigurationComponentIdentity],
) -> Result<ComponentRootDigest, CodecError> {
    let mut encoder = Encoder::new(Sha256Sink::new());
    encoder.raw(b"starclock.configuration-components");
    encoder.u32(COMPONENT_SET_REVISION);
    encoder.u32(u32::try_from(components.len()).map_err(|_| CodecError::LengthOverflow)?);
    for component in components {
        component.encode(&mut encoder)?;
    }
    Ok(ComponentRootDigest::new(
        encoder.into_inner().finalize().bytes(),
    ))
}

fn validate_text(value: &str) -> Result<(), ComponentIdentityError> {
    if value.is_empty()
        || value.len() > MAX_COMPONENT_TEXT_BYTES
        || !value.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(ComponentIdentityError::InvalidText);
    }
    Ok(())
}
