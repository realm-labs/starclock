//! Immutable Universe mechanic-rule contributions.

use crate::definition::LocalizedText;
use crate::id::MechanicRuleId;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum MechanicRuleKind {
    PathResonance = 0,
    BlessingDefinition = 1,
    BlessingLevel = 2,
    CurioDefinition = 3,
    CurioState = 4,
    RunService = 5,
    AbilityTreeContribution = 6,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MechanicParameter {
    index: Option<u32>,
    key: Option<Box<str>>,
    value: Box<str>,
}
impl MechanicParameter {
    pub(crate) fn indexed(index: u32, value: &str) -> Self {
        Self {
            index: Some(index),
            key: None,
            value: value.into(),
        }
    }
    pub(crate) fn named(key: &str, value: &str) -> Self {
        Self {
            index: None,
            key: Some(key.into()),
            value: value.into(),
        }
    }
    #[must_use]
    pub const fn index(&self) -> Option<u32> {
        self.index
    }
    #[must_use]
    pub fn key(&self) -> Option<&str> {
        self.key.as_deref()
    }
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MechanicRuleDefinition {
    id: MechanicRuleId,
    stable_key: Box<str>,
    source_record_key: Box<str>,
    source_file: Box<str>,
    kind: MechanicRuleKind,
    native_handler_key: Option<Box<str>>,
    source_binding_key: Option<Box<str>>,
    parameters: Box<[MechanicParameter]>,
    mechanic_tags: Box<[Box<str>]>,
    approximation_replacement_condition: Option<Box<str>>,
    text: LocalizedText,
}

impl MechanicRuleDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: MechanicRuleId,
        stable_key: &str,
        source_record_key: &str,
        source_file: &str,
        kind: MechanicRuleKind,
        native_handler_key: Option<Box<str>>,
        source_binding_key: Option<Box<str>>,
        parameters: Box<[MechanicParameter]>,
        mechanic_tags: Box<[Box<str>]>,
        approximation_replacement_condition: Option<Box<str>>,
        text: LocalizedText,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            source_record_key: source_record_key.into(),
            source_file: source_file.into(),
            kind,
            native_handler_key,
            source_binding_key,
            parameters,
            mechanic_tags,
            approximation_replacement_condition,
            text,
        }
    }
    #[must_use]
    pub const fn id(&self) -> MechanicRuleId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub fn source_record_key(&self) -> &str {
        &self.source_record_key
    }
    #[must_use]
    pub fn source_file(&self) -> &str {
        &self.source_file
    }
    #[must_use]
    pub const fn kind(&self) -> MechanicRuleKind {
        self.kind
    }
    #[must_use]
    pub fn native_handler_key(&self) -> Option<&str> {
        self.native_handler_key.as_deref()
    }
    #[must_use]
    pub fn source_binding_key(&self) -> Option<&str> {
        self.source_binding_key.as_deref()
    }
    #[must_use]
    pub fn parameters(&self) -> &[MechanicParameter] {
        &self.parameters
    }
    #[must_use]
    pub fn mechanic_tags(&self) -> &[Box<str>] {
        &self.mechanic_tags
    }
    #[must_use]
    pub fn approximation_replacement_condition(&self) -> Option<&str> {
        self.approximation_replacement_condition.as_deref()
    }
    #[must_use]
    pub const fn text(&self) -> &LocalizedText {
        &self.text
    }
}
