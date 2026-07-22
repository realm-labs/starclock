//! Immutable Path, Blessing and Resonance definitions.

use crate::definition::LocalizedText;
use crate::digest::UniversePathDefinitionsDigest;
use crate::id::{BlessingId, BlessingLevelId, PathId, ResonanceId};

/// Exact authored decimal atom. Formula compilation owns later six-place rounding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExactParameter {
    coefficient: i64,
    scale: u8,
}

impl ExactParameter {
    pub(crate) const fn new(coefficient: i64, scale: u8) -> Self {
        Self { coefficient, scale }
    }
    #[must_use]
    pub const fn coefficient(self) -> i64 {
        self.coefficient
    }
    #[must_use]
    pub const fn scale(self) -> u8 {
        self.scale
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ResonanceKind {
    Resonance = 0,
    Formation = 1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathDefinition {
    id: PathId,
    stable_key: Box<str>,
    buff_type: u32,
    text: LocalizedText,
    unlock_policy_key: Box<str>,
    resonance: ResonanceId,
    formations: Box<[ResonanceId]>,
    blessings: Box<[BlessingId]>,
}

impl PathDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: PathId,
        stable_key: &str,
        buff_type: u32,
        text: LocalizedText,
        unlock_policy_key: &str,
        resonance: ResonanceId,
        formations: Box<[ResonanceId]>,
        blessings: Box<[BlessingId]>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            buff_type,
            text,
            unlock_policy_key: unlock_policy_key.into(),
            resonance,
            formations,
            blessings,
        }
    }
    #[must_use]
    pub const fn id(&self) -> PathId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn buff_type(&self) -> u32 {
        self.buff_type
    }
    #[must_use]
    pub const fn text(&self) -> &LocalizedText {
        &self.text
    }
    #[must_use]
    pub fn unlock_policy_key(&self) -> &str {
        &self.unlock_policy_key
    }
    #[must_use]
    pub const fn resonance(&self) -> ResonanceId {
        self.resonance
    }
    #[must_use]
    pub fn formations(&self) -> &[ResonanceId] {
        &self.formations
    }
    #[must_use]
    pub fn blessings(&self) -> &[BlessingId] {
        &self.blessings
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlessingDefinition {
    id: BlessingId,
    stable_key: Box<str>,
    path: PathId,
    rarity: u8,
    text: LocalizedText,
    pool_tags: Box<[Box<str>]>,
    mechanic_tags: Box<[Box<str>]>,
    rule_key: Box<str>,
    source_description_en: [u8; 32],
    source_description_zh_cn: [u8; 32],
    levels: Box<[BlessingLevelId]>,
    prerequisite_keys: Box<[Box<str>]>,
}

impl BlessingDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: BlessingId,
        stable_key: &str,
        path: PathId,
        rarity: u8,
        text: LocalizedText,
        pool_tags: Box<[Box<str>]>,
        mechanic_tags: Box<[Box<str>]>,
        rule_key: &str,
        source_description_en: [u8; 32],
        source_description_zh_cn: [u8; 32],
        levels: Box<[BlessingLevelId]>,
        prerequisite_keys: Box<[Box<str>]>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            path,
            rarity,
            text,
            pool_tags,
            mechanic_tags,
            rule_key: rule_key.into(),
            source_description_en,
            source_description_zh_cn,
            levels,
            prerequisite_keys,
        }
    }
    #[must_use]
    pub const fn id(&self) -> BlessingId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn path(&self) -> PathId {
        self.path
    }
    #[must_use]
    pub const fn rarity(&self) -> u8 {
        self.rarity
    }
    #[must_use]
    pub const fn text(&self) -> &LocalizedText {
        &self.text
    }
    #[must_use]
    pub fn pool_tags(&self) -> &[Box<str>] {
        &self.pool_tags
    }
    #[must_use]
    pub fn mechanic_tags(&self) -> &[Box<str>] {
        &self.mechanic_tags
    }
    #[must_use]
    pub fn rule_key(&self) -> &str {
        &self.rule_key
    }
    #[must_use]
    pub const fn source_description_en(&self) -> [u8; 32] {
        self.source_description_en
    }
    #[must_use]
    pub const fn source_description_zh_cn(&self) -> [u8; 32] {
        self.source_description_zh_cn
    }
    #[must_use]
    pub fn levels(&self) -> &[BlessingLevelId] {
        &self.levels
    }
    #[must_use]
    pub fn prerequisite_keys(&self) -> &[Box<str>] {
        &self.prerequisite_keys
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlessingLevelDefinition {
    id: BlessingLevelId,
    stable_key: Box<str>,
    blessing: BlessingId,
    level: u8,
    source_binding_key: Box<str>,
    rule_key: Box<str>,
    text: LocalizedText,
    parameters: Box<[ExactParameter]>,
}

impl BlessingLevelDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: BlessingLevelId,
        stable_key: &str,
        blessing: BlessingId,
        level: u8,
        source_binding_key: &str,
        rule_key: &str,
        text: LocalizedText,
        parameters: Box<[ExactParameter]>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            blessing,
            level,
            source_binding_key: source_binding_key.into(),
            rule_key: rule_key.into(),
            text,
            parameters,
        }
    }
    #[must_use]
    pub const fn id(&self) -> BlessingLevelId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn blessing(&self) -> BlessingId {
        self.blessing
    }
    #[must_use]
    pub const fn level(&self) -> u8 {
        self.level
    }
    #[must_use]
    pub fn source_binding_key(&self) -> &str {
        &self.source_binding_key
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResonanceDefinition {
    id: ResonanceId,
    stable_key: Box<str>,
    path: PathId,
    kind: ResonanceKind,
    threshold: u8,
    energy_max: ExactParameter,
    initial_energy: ExactParameter,
    text: LocalizedText,
    mechanic_tags: Box<[Box<str>]>,
    source_binding_key: Box<str>,
    rule_key: Box<str>,
    parameters: Box<[ExactParameter]>,
}

#[derive(Debug)]
pub(crate) struct PathDefinitions {
    pub(crate) digest: UniversePathDefinitionsDigest,
    pub(crate) paths: Box<[PathDefinition]>,
    pub(crate) blessings: Box<[BlessingDefinition]>,
    pub(crate) levels: Box<[BlessingLevelDefinition]>,
    pub(crate) resonances: Box<[ResonanceDefinition]>,
}

impl ResonanceDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: ResonanceId,
        stable_key: &str,
        path: PathId,
        kind: ResonanceKind,
        threshold: u8,
        energy_max: ExactParameter,
        initial_energy: ExactParameter,
        text: LocalizedText,
        mechanic_tags: Box<[Box<str>]>,
        source_binding_key: &str,
        rule_key: &str,
        parameters: Box<[ExactParameter]>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            path,
            kind,
            threshold,
            energy_max,
            initial_energy,
            text,
            mechanic_tags,
            source_binding_key: source_binding_key.into(),
            rule_key: rule_key.into(),
            parameters,
        }
    }
    #[must_use]
    pub const fn id(&self) -> ResonanceId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn path(&self) -> PathId {
        self.path
    }
    #[must_use]
    pub const fn kind(&self) -> ResonanceKind {
        self.kind
    }
    #[must_use]
    pub const fn threshold(&self) -> u8 {
        self.threshold
    }
    #[must_use]
    pub const fn energy_max(&self) -> ExactParameter {
        self.energy_max
    }
    #[must_use]
    pub const fn initial_energy(&self) -> ExactParameter {
        self.initial_energy
    }
    #[must_use]
    pub const fn text(&self) -> &LocalizedText {
        &self.text
    }
    #[must_use]
    pub fn mechanic_tags(&self) -> &[Box<str>] {
        &self.mechanic_tags
    }
    #[must_use]
    pub fn source_binding_key(&self) -> &str {
        &self.source_binding_key
    }
    #[must_use]
    pub fn rule_key(&self) -> &str {
        &self.rule_key
    }
    #[must_use]
    pub fn parameters(&self) -> &[ExactParameter] {
        &self.parameters
    }
}
