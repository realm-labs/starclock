//! Static handler function, metadata, output and audit value types.

use starclock_combat::{
    NativeHandlerId,
    rule::model::{RuleEmission, RuleEvaluationInput, RuleValue},
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum HandlerDomain {
    Battle,
    Activity,
}

#[derive(Clone, Copy, Debug)]
pub struct BattleHandlerInput<'a> {
    pub rule: RuleEvaluationInput<'a>,
    pub arguments: &'a [RuleValue],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleHandlerOutput {
    emissions: Box<[RuleEmission]>,
}

impl BattleHandlerOutput {
    #[must_use]
    pub fn new(emissions: Vec<RuleEmission>) -> Self {
        Self {
            emissions: emissions.into_boxed_slice(),
        }
    }
    #[must_use]
    pub fn emissions(&self) -> &[RuleEmission] {
        &self.emissions
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeHandlerFault {
    code: u32,
    context: Option<i64>,
}

impl NativeHandlerFault {
    #[must_use]
    pub const fn new(code: u32, context: Option<i64>) -> Option<Self> {
        if code == 0 {
            None
        } else {
            Some(Self { code, context })
        }
    }
    #[must_use]
    pub const fn code(self) -> u32 {
        self.code
    }
    #[must_use]
    pub const fn context(self) -> Option<i64> {
        self.context
    }
}

impl core::fmt::Display for NativeHandlerFault {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "native handler fault {}", self.code)
    }
}

impl std::error::Error for NativeHandlerFault {}

pub type BattleHandler =
    for<'a> fn(BattleHandlerInput<'a>) -> Result<BattleHandlerOutput, NativeHandlerFault>;

#[derive(Clone, Copy)]
pub struct BattleHandlerRegistration {
    pub id: NativeHandlerId,
    pub stable_key: &'static str,
    pub version: &'static str,
    pub argument_schema_digest: [u8; 32],
    pub determinism_note: &'static str,
    pub owner: &'static str,
    pub ir_insufficiency: &'static str,
    pub removal_condition: &'static str,
    pub handler: BattleHandler,
}

impl core::fmt::Debug for BattleHandlerRegistration {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("BattleHandlerRegistration")
            .field("id", &self.id)
            .field("stable_key", &self.stable_key)
            .field("version", &self.version)
            .field("argument_schema_digest", &self.argument_schema_digest)
            .field("determinism_note", &self.determinism_note)
            .field("owner", &self.owner)
            .field("ir_insufficiency", &self.ir_insufficiency)
            .field("removal_condition", &self.removal_condition)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeHandlerRequirement<'a> {
    pub id: NativeHandlerId,
    pub stable_key: &'a str,
    pub domain: HandlerDomain,
    pub version: &'a str,
    pub argument_schema_digest: [u8; 32],
    pub determinism_note: &'a str,
    pub owner: &'a str,
    pub ir_insufficiency: &'a str,
    pub removal_condition: &'a str,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RegistryErrorKind {
    InvalidRevision,
    NonCanonicalRegistration,
    InvalidRegistration,
    InvalidRequirement,
    MissingRegistration,
    UnsupportedDomain,
    StableKeyMismatch,
    VersionMismatch,
    ArgumentSchemaMismatch,
    MissingIrInsufficiencyDecision,
    DeterminismNoteMismatch,
    OwnerMismatch,
    IrInsufficiencyMismatch,
    RemovalConditionMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegistryError {
    pub(crate) kind: RegistryErrorKind,
    pub(crate) handler: Option<NativeHandlerId>,
}

impl RegistryError {
    #[must_use]
    pub const fn kind(self) -> RegistryErrorKind {
        self.kind
    }
    #[must_use]
    pub const fn handler(self) -> Option<NativeHandlerId> {
        self.handler
    }
}

impl core::fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "native registry {:?}", self.kind)
    }
}

impl std::error::Error for RegistryError {}
