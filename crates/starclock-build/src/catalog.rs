//! Immutable build-domain catalog and validated construction.

use starclock_combat::{
    AbilityId, CombatantSpecDigest, Hp, ModifierDefinitionId, ResolvedDefinitionBindings,
    RuleBundleId, Speed, UnitDefinitionId, UnitLevel,
    catalog::{CatalogDigest, CombatCatalog},
};

/// Human-readable immutable build-catalog revision.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BuildCatalogRevision(Box<str>);

impl BuildCatalogRevision {
    /// Creates a non-empty revision.
    #[must_use]
    pub fn new(value: &str) -> Option<Self> {
        (!value.trim().is_empty()).then(|| Self(value.into()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One already-resolved character row used by the B1 compilation boundary.
///
/// Later batches replace the fixed level row and bindings with curves and
/// ordered Trace/Eidolon/equipment patches without changing catalog ownership.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharacterBuildDefinition {
    form: UnitDefinitionId,
    level: UnitLevel,
    maximum_hp: Hp,
    speed: Speed,
    bindings: ResolvedDefinitionBindings,
    combatant_digest: CombatantSpecDigest,
}

impl CharacterBuildDefinition {
    #[must_use]
    pub fn new(
        form: UnitDefinitionId,
        level: UnitLevel,
        maximum_hp: Hp,
        speed: Speed,
        bindings: ResolvedDefinitionBindings,
        combatant_digest: CombatantSpecDigest,
    ) -> Self {
        Self {
            form,
            level,
            maximum_hp,
            speed,
            bindings,
            combatant_digest,
        }
    }

    #[must_use]
    pub const fn form(&self) -> UnitDefinitionId {
        self.form
    }
    #[must_use]
    pub const fn level(&self) -> UnitLevel {
        self.level
    }
    #[must_use]
    pub const fn maximum_hp(&self) -> Hp {
        self.maximum_hp
    }
    #[must_use]
    pub const fn speed(&self) -> Speed {
        self.speed
    }
    #[must_use]
    pub fn abilities(&self) -> &[AbilityId] {
        self.bindings.abilities()
    }
    #[must_use]
    pub fn rule_bundles(&self) -> &[RuleBundleId] {
        self.bindings.rule_bundles()
    }
    #[must_use]
    pub fn modifiers(&self) -> &[ModifierDefinitionId] {
        self.bindings.modifiers()
    }
    #[must_use]
    pub const fn combatant_digest(&self) -> CombatantSpecDigest {
        self.combatant_digest
    }
}

/// Immutable validated build definitions compatible with one combat revision.
#[derive(Debug)]
pub struct BuildCatalog {
    revision: BuildCatalogRevision,
    compatible_combat_revision: Box<str>,
    compatible_combat_digest: CatalogDigest,
    characters: Box<[CharacterBuildDefinition]>,
}

impl BuildCatalog {
    #[must_use]
    pub const fn revision(&self) -> &BuildCatalogRevision {
        &self.revision
    }
    #[must_use]
    pub fn compatible_combat_revision(&self) -> &str {
        &self.compatible_combat_revision
    }
    #[must_use]
    pub const fn compatible_combat_digest(&self) -> CatalogDigest {
        self.compatible_combat_digest
    }
    #[must_use]
    pub fn character(&self, form: UnitDefinitionId) -> Option<&CharacterBuildDefinition> {
        self.characters
            .binary_search_by_key(&form, CharacterBuildDefinition::form)
            .ok()
            .map(|index| &self.characters[index])
    }
    pub fn character_ids(&self) -> impl ExactSizeIterator<Item = UnitDefinitionId> + '_ {
        self.characters.iter().map(CharacterBuildDefinition::form)
    }
}

/// Validated catalog builder; input order is never retained as semantics.
#[derive(Debug)]
pub struct BuildCatalogBuilder {
    revision: BuildCatalogRevision,
    compatible_combat_revision: Box<str>,
    characters: Vec<CharacterBuildDefinition>,
}

impl BuildCatalogBuilder {
    #[must_use]
    pub fn new(revision: BuildCatalogRevision, compatible_combat_revision: &str) -> Option<Self> {
        (!compatible_combat_revision.trim().is_empty()).then(|| Self {
            revision,
            compatible_combat_revision: compatible_combat_revision.into(),
            characters: Vec::new(),
        })
    }

    pub fn add_character(&mut self, definition: CharacterBuildDefinition) {
        self.characters.push(definition);
    }

    pub fn build(mut self, combat: &CombatCatalog) -> Result<BuildCatalog, BuildCatalogError> {
        if combat.revision().as_str() != self.compatible_combat_revision.as_ref() {
            return Err(BuildCatalogError::new(
                BuildCatalogErrorKind::IncompatibleCombatRevision,
                None,
            ));
        }
        self.characters
            .sort_unstable_by_key(CharacterBuildDefinition::form);
        if let Some(pair) = self
            .characters
            .windows(2)
            .find(|pair| pair[0].form == pair[1].form)
        {
            return Err(BuildCatalogError::new(
                BuildCatalogErrorKind::DuplicateCharacter,
                Some(pair[0].form),
            ));
        }
        for definition in &self.characters {
            validate_character(definition, combat)?;
        }
        Ok(BuildCatalog {
            revision: self.revision,
            compatible_combat_revision: self.compatible_combat_revision,
            compatible_combat_digest: combat.digest(),
            characters: self.characters.into_boxed_slice(),
        })
    }
}

fn validate_character(
    definition: &CharacterBuildDefinition,
    combat: &CombatCatalog,
) -> Result<(), BuildCatalogError> {
    let form = definition.form;
    let unit = combat.unit(form).ok_or_else(|| {
        BuildCatalogError::new(BuildCatalogErrorKind::MissingCombatForm, Some(form))
    })?;
    if definition.maximum_hp.get() == 0 {
        return Err(BuildCatalogError::new(
            BuildCatalogErrorKind::ZeroMaximumHp,
            Some(form),
        ));
    }
    if definition.abilities().is_empty() {
        return Err(BuildCatalogError::new(
            BuildCatalogErrorKind::NonCanonicalReferences,
            Some(form),
        ));
    }
    if definition
        .abilities()
        .iter()
        .any(|id| combat.ability(*id).is_none() || unit.abilities().binary_search(id).is_err())
    {
        return Err(BuildCatalogError::new(
            BuildCatalogErrorKind::InvalidAbilityBinding,
            Some(form),
        ));
    }
    if definition.rule_bundles().iter().any(|id| {
        combat.rule_bundle(*id).is_none() || unit.rule_bundles().binary_search(id).is_err()
    }) {
        return Err(BuildCatalogError::new(
            BuildCatalogErrorKind::InvalidRuleBinding,
            Some(form),
        ));
    }
    if definition
        .modifiers()
        .iter()
        .any(|id| combat.modifier(*id).is_none())
    {
        return Err(BuildCatalogError::new(
            BuildCatalogErrorKind::InvalidModifierBinding,
            Some(form),
        ));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildCatalogErrorKind {
    IncompatibleCombatRevision,
    DuplicateCharacter,
    MissingCombatForm,
    ZeroMaximumHp,
    NonCanonicalReferences,
    InvalidAbilityBinding,
    InvalidRuleBinding,
    InvalidModifierBinding,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildCatalogError {
    kind: BuildCatalogErrorKind,
    form: Option<UnitDefinitionId>,
}

impl BuildCatalogError {
    const fn new(kind: BuildCatalogErrorKind, form: Option<UnitDefinitionId>) -> Self {
        Self { kind, form }
    }
    #[must_use]
    pub const fn kind(self) -> BuildCatalogErrorKind {
        self.kind
    }
    #[must_use]
    pub const fn form(self) -> Option<UnitDefinitionId> {
        self.form
    }
}

impl std::fmt::Display for BuildCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid build catalog: {:?}", self.kind)
    }
}

impl std::error::Error for BuildCatalogError {}
