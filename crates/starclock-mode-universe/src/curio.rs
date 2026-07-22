//! Immutable Curio and lifecycle-state definitions.

use crate::definition::LocalizedText;
use crate::digest::UniverseCurioDefinitionsDigest;
use crate::id::{CurioId, CurioStateId};
use crate::path::ExactParameter;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CurioStateKind {
    Active = 0,
    Repairing = 1,
    Fixed = 2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioDefinition {
    id: CurioId,
    stable_key: Box<str>,
    initial_state: CurioStateId,
    handbook_order: u32,
    text: LocalizedText,
    tags: Box<[Box<str>]>,
    pool_tags: Box<[Box<str>]>,
    rule_key: Box<str>,
    states: Box<[CurioStateId]>,
}

impl CurioDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: CurioId,
        stable_key: &str,
        initial_state: CurioStateId,
        handbook_order: u32,
        text: LocalizedText,
        tags: Box<[Box<str>]>,
        pool_tags: Box<[Box<str>]>,
        rule_key: &str,
        states: Box<[CurioStateId]>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            initial_state,
            handbook_order,
            text,
            tags,
            pool_tags,
            rule_key: rule_key.into(),
            states,
        }
    }

    #[must_use]
    pub const fn id(&self) -> CurioId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn initial_state(&self) -> CurioStateId {
        self.initial_state
    }
    #[must_use]
    pub const fn handbook_order(&self) -> u32 {
        self.handbook_order
    }
    #[must_use]
    pub const fn text(&self) -> &LocalizedText {
        &self.text
    }
    #[must_use]
    pub fn tags(&self) -> &[Box<str>] {
        &self.tags
    }
    #[must_use]
    pub fn pool_tags(&self) -> &[Box<str>] {
        &self.pool_tags
    }
    #[must_use]
    pub fn rule_key(&self) -> &str {
        &self.rule_key
    }
    #[must_use]
    pub fn states(&self) -> &[CurioStateId] {
        &self.states
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioStateDefinition {
    id: CurioStateId,
    stable_key: Box<str>,
    curio: CurioId,
    kind: CurioStateKind,
    charges: Option<ExactParameter>,
    charge_parameter_index: Option<u8>,
    next_state: Option<CurioStateId>,
    repair_state: Option<CurioStateId>,
    replacement_curio: Option<CurioId>,
    source_effect_id: Box<str>,
    rule_key: Box<str>,
    text: LocalizedText,
    parameters: Box<[ExactParameter]>,
}

impl CurioStateDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: CurioStateId,
        stable_key: &str,
        curio: CurioId,
        kind: CurioStateKind,
        charges: Option<ExactParameter>,
        charge_parameter_index: Option<u8>,
        next_state: Option<CurioStateId>,
        repair_state: Option<CurioStateId>,
        replacement_curio: Option<CurioId>,
        source_effect_id: &str,
        rule_key: &str,
        text: LocalizedText,
        parameters: Box<[ExactParameter]>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            curio,
            kind,
            charges,
            charge_parameter_index,
            next_state,
            repair_state,
            replacement_curio,
            source_effect_id: source_effect_id.into(),
            rule_key: rule_key.into(),
            text,
            parameters,
        }
    }

    #[must_use]
    pub const fn id(&self) -> CurioStateId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn curio(&self) -> CurioId {
        self.curio
    }
    #[must_use]
    pub const fn kind(&self) -> CurioStateKind {
        self.kind
    }
    #[must_use]
    pub const fn charges(&self) -> Option<ExactParameter> {
        self.charges
    }
    #[must_use]
    pub const fn charge_parameter_index(&self) -> Option<u8> {
        self.charge_parameter_index
    }
    #[must_use]
    pub const fn next_state(&self) -> Option<CurioStateId> {
        self.next_state
    }
    #[must_use]
    pub const fn repair_state(&self) -> Option<CurioStateId> {
        self.repair_state
    }
    #[must_use]
    pub const fn replacement_curio(&self) -> Option<CurioId> {
        self.replacement_curio
    }
    #[must_use]
    pub fn source_effect_id(&self) -> &str {
        &self.source_effect_id
    }
    #[must_use]
    pub fn rule_key(&self) -> &str {
        &self.rule_key
    }
    #[must_use]
    pub const fn text(&self) -> &LocalizedText {
        &self.text
    }
    #[must_use]
    pub fn parameters(&self) -> &[ExactParameter] {
        &self.parameters
    }
}

#[derive(Debug)]
pub(crate) struct CurioDefinitions {
    pub(crate) digest: UniverseCurioDefinitionsDigest,
    pub(crate) curios: Box<[CurioDefinition]>,
    pub(crate) states: Box<[CurioStateDefinition]>,
}
