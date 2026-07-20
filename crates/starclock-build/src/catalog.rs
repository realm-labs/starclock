//! Immutable build-domain catalog and validated construction.

use starclock_combat::{
    AbilityId, CombatantSpecDigest, Hp, ModifierDefinitionId, ResolvedDefinitionBindings,
    RuleBundleId, Speed, StatValue, UnitDefinitionId, UnitLevel,
    catalog::{CatalogDigest, CombatCatalog},
};

use crate::{
    ability::AbilityLevelTable,
    eidolon::{EidolonSetDefinition, EidolonSetError},
    id::LightConeId,
    light_cone::{CombatPath, LightConeDefinition, LightConeDefinitionError},
    patch::BuildPatch,
    spec::PromotionStage,
    trace::{TraceGraphDefinition, TraceGraphError},
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
    path: CombatPath,
    stats: Box<[CharacterStatRow]>,
    bindings: ResolvedDefinitionBindings,
    ability_levels: Box<[AbilityLevelTable]>,
    trace_graph: Option<TraceGraphDefinition>,
    eidolons: Option<EidolonSetDefinition>,
    combatant_digest: CombatantSpecDigest,
}

impl CharacterBuildDefinition {
    #[must_use]
    pub fn new(
        form: UnitDefinitionId,
        path: CombatPath,
        stat: CharacterStatRow,
        bindings: ResolvedDefinitionBindings,
        combatant_digest: CombatantSpecDigest,
    ) -> Self {
        Self {
            form,
            path,
            stats: vec![stat].into_boxed_slice(),
            bindings,
            ability_levels: Box::new([]),
            trace_graph: None,
            eidolons: None,
            combatant_digest,
        }
    }

    #[must_use]
    pub fn with_stat_rows(mut self, stats: Vec<CharacterStatRow>) -> Self {
        self.stats = stats.into_boxed_slice();
        self
    }

    #[must_use]
    pub fn with_ability_levels(mut self, tables: Vec<AbilityLevelTable>) -> Self {
        self.ability_levels = tables.into_boxed_slice();
        self
    }

    #[must_use]
    pub fn with_trace_graph(mut self, graph: TraceGraphDefinition) -> Self {
        self.trace_graph = Some(graph);
        self
    }

    #[must_use]
    pub fn with_eidolons(mut self, eidolons: EidolonSetDefinition) -> Self {
        self.eidolons = Some(eidolons);
        self
    }

    #[must_use]
    pub const fn form(&self) -> UnitDefinitionId {
        self.form
    }
    #[must_use]
    pub const fn path(&self) -> CombatPath {
        self.path
    }
    #[must_use]
    pub fn stat_row(
        &self,
        level: UnitLevel,
        promotion: PromotionStage,
    ) -> Option<&CharacterStatRow> {
        self.stats
            .binary_search_by_key(&(level, promotion), |row| (row.level, row.promotion))
            .ok()
            .map(|index| &self.stats[index])
    }
    #[must_use]
    pub fn stat_rows(&self) -> &[CharacterStatRow] {
        &self.stats
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
    #[must_use]
    pub fn ability_levels(&self) -> &[AbilityLevelTable] {
        &self.ability_levels
    }
    #[must_use]
    pub fn ability_level_table(&self, family: AbilityId) -> Option<&AbilityLevelTable> {
        self.ability_levels
            .binary_search_by_key(&family, AbilityLevelTable::family)
            .ok()
            .map(|index| &self.ability_levels[index])
    }
    #[must_use]
    pub const fn trace_graph(&self) -> Option<&TraceGraphDefinition> {
        self.trace_graph.as_ref()
    }
    #[must_use]
    pub fn eidolons(&self) -> &EidolonSetDefinition {
        self.eidolons
            .as_ref()
            .expect("validated character has an E1-E6 set")
    }
}

/// Exact fixed-point/integer base row at one level and promotion boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CharacterStatRow {
    level: UnitLevel,
    promotion: PromotionStage,
    maximum_hp: Hp,
    attack: StatValue,
    defense: StatValue,
    speed: Speed,
}

impl CharacterStatRow {
    #[must_use]
    pub fn new(level: UnitLevel, promotion: PromotionStage, maximum_hp: Hp, speed: Speed) -> Self {
        Self {
            level,
            promotion,
            maximum_hp,
            attack: StatValue::from_scaled(0).expect("zero is a valid stat"),
            defense: StatValue::from_scaled(0).expect("zero is a valid stat"),
            speed,
        }
    }
    #[must_use]
    pub const fn with_attack_defense(mut self, attack: StatValue, defense: StatValue) -> Self {
        self.attack = attack;
        self.defense = defense;
        self
    }
    #[must_use]
    pub const fn level(self) -> UnitLevel {
        self.level
    }
    #[must_use]
    pub const fn promotion(self) -> PromotionStage {
        self.promotion
    }
    #[must_use]
    pub const fn maximum_hp(self) -> Hp {
        self.maximum_hp
    }
    #[must_use]
    pub const fn attack(self) -> StatValue {
        self.attack
    }
    #[must_use]
    pub const fn defense(self) -> StatValue {
        self.defense
    }
    #[must_use]
    pub const fn speed(self) -> Speed {
        self.speed
    }
}

/// Immutable validated build definitions compatible with one combat revision.
#[derive(Debug)]
pub struct BuildCatalog {
    revision: BuildCatalogRevision,
    compatible_combat_revision: Box<str>,
    compatible_combat_digest: CatalogDigest,
    characters: Box<[CharacterBuildDefinition]>,
    light_cones: Box<[LightConeDefinition]>,
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
    #[must_use]
    pub fn light_cone(&self, id: LightConeId) -> Option<&LightConeDefinition> {
        self.light_cones
            .binary_search_by_key(&id, LightConeDefinition::id)
            .ok()
            .map(|index| &self.light_cones[index])
    }
    pub fn light_cone_ids(&self) -> impl ExactSizeIterator<Item = LightConeId> + '_ {
        self.light_cones.iter().map(LightConeDefinition::id)
    }
}

/// Validated catalog builder; input order is never retained as semantics.
#[derive(Debug)]
pub struct BuildCatalogBuilder {
    revision: BuildCatalogRevision,
    compatible_combat_revision: Box<str>,
    characters: Vec<CharacterBuildDefinition>,
    light_cones: Vec<LightConeDefinition>,
}

impl BuildCatalogBuilder {
    #[must_use]
    pub fn new(revision: BuildCatalogRevision, compatible_combat_revision: &str) -> Option<Self> {
        (!compatible_combat_revision.trim().is_empty()).then(|| Self {
            revision,
            compatible_combat_revision: compatible_combat_revision.into(),
            characters: Vec::new(),
            light_cones: Vec::new(),
        })
    }

    pub fn add_character(&mut self, definition: CharacterBuildDefinition) {
        self.characters.push(definition);
    }

    pub fn add_light_cone(&mut self, definition: LightConeDefinition) {
        self.light_cones.push(definition);
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
        for definition in &mut self.characters {
            validate_character(definition, combat)?;
        }
        self.light_cones
            .sort_unstable_by_key(LightConeDefinition::id);
        if let Some(pair) = self
            .light_cones
            .windows(2)
            .find(|pair| pair[0].id() == pair[1].id())
        {
            return Err(light_cone_error(
                BuildCatalogErrorKind::DuplicateLightCone,
                pair[0].id(),
            ));
        }
        for definition in &mut self.light_cones {
            validate_light_cone(definition, combat)?;
        }
        Ok(BuildCatalog {
            revision: self.revision,
            compatible_combat_revision: self.compatible_combat_revision,
            compatible_combat_digest: combat.digest(),
            characters: self.characters.into_boxed_slice(),
            light_cones: self.light_cones.into_boxed_slice(),
        })
    }
}

fn validate_character(
    definition: &mut CharacterBuildDefinition,
    combat: &CombatCatalog,
) -> Result<(), BuildCatalogError> {
    let form = definition.form;
    let unit = combat.unit(form).ok_or_else(|| {
        BuildCatalogError::new(BuildCatalogErrorKind::MissingCombatForm, Some(form))
    })?;
    definition
        .stats
        .sort_unstable_by_key(|row| (row.level, row.promotion));
    if definition.stats.is_empty()
        || definition
            .stats
            .windows(2)
            .any(|pair| (pair[0].level, pair[0].promotion) == (pair[1].level, pair[1].promotion))
        || definition.stats.iter().any(|row| row.maximum_hp.get() == 0)
    {
        return Err(BuildCatalogError::new(
            BuildCatalogErrorKind::InvalidStatCurve,
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
    definition
        .ability_levels
        .sort_unstable_by_key(AbilityLevelTable::family);
    if definition
        .ability_levels
        .windows(2)
        .any(|pair| pair[0].family() == pair[1].family())
    {
        return Err(character_error(
            BuildCatalogErrorKind::DuplicateAbilityFamily,
            form,
        ));
    }
    let mut resolved_curve_abilities = std::collections::BTreeSet::new();
    for table in &mut definition.ability_levels {
        table.canonicalize();
        if !table.is_complete()
            || table.rows().iter().any(|row| {
                combat.ability(row.resolved_ability()).is_none()
                    || unit
                        .abilities()
                        .binary_search(&row.resolved_ability())
                        .is_err()
                    || !resolved_curve_abilities.insert(row.resolved_ability())
            })
        {
            return Err(character_error(
                BuildCatalogErrorKind::InvalidAbilityCurve,
                form,
            ));
        }
    }
    if let Some(graph) = &mut definition.trace_graph {
        if graph.form() != form {
            return Err(character_error(
                BuildCatalogErrorKind::InvalidTraceGraph,
                form,
            ));
        }
        graph.canonicalize().map_err(|error| {
            character_error(
                match error {
                    TraceGraphError::DuplicateNode => BuildCatalogErrorKind::DuplicateTraceNode,
                    TraceGraphError::InvalidPrerequisite | TraceGraphError::Cycle => {
                        BuildCatalogErrorKind::InvalidTraceGraph
                    }
                },
                form,
            )
        })?;
        validate_trace_patches(definition, combat, unit)?;
    }
    let Some(eidolons) = &mut definition.eidolons else {
        return Err(character_error(
            BuildCatalogErrorKind::MissingEidolonSet,
            form,
        ));
    };
    if eidolons.form() != form {
        return Err(character_error(
            BuildCatalogErrorKind::InvalidEidolonSet,
            form,
        ));
    }
    eidolons.canonicalize().map_err(|error| {
        character_error(
            match error {
                EidolonSetError::IncompleteRankSet => BuildCatalogErrorKind::IncompleteEidolonSet,
                EidolonSetError::DuplicateDefinition => BuildCatalogErrorKind::InvalidEidolonSet,
            },
            form,
        )
    })?;
    validate_eidolon_patches(definition, combat, unit)?;
    validate_level_adjustments(definition)?;
    Ok(())
}

fn validate_trace_patches(
    definition: &CharacterBuildDefinition,
    combat: &CombatCatalog,
    unit: &starclock_combat::catalog::definition::UnitDefinition,
) -> Result<(), BuildCatalogError> {
    let form = definition.form;
    for patch in definition
        .trace_graph
        .iter()
        .flat_map(|graph| graph.nodes())
        .flat_map(|node| node.patches())
    {
        match *patch {
            BuildPatch::AddAbility(id)
                if combat.ability(id).is_none() || unit.abilities().binary_search(&id).is_err() =>
            {
                return Err(character_error(
                    BuildCatalogErrorKind::InvalidTracePatch,
                    form,
                ));
            }
            BuildPatch::AddRuleBundle(id)
                if combat.rule_bundle(id).is_none()
                    || unit.rule_bundles().binary_search(&id).is_err() =>
            {
                return Err(character_error(
                    BuildCatalogErrorKind::InvalidTracePatch,
                    form,
                ));
            }
            BuildPatch::AddModifier(id) if combat.modifier(id).is_none() => {
                return Err(character_error(
                    BuildCatalogErrorKind::InvalidTracePatch,
                    form,
                ));
            }
            BuildPatch::RemoveRuleBundle(id)
                if combat.rule_bundle(id).is_none()
                    || unit.rule_bundles().binary_search(&id).is_err() =>
            {
                return Err(character_error(
                    BuildCatalogErrorKind::InvalidTracePatch,
                    form,
                ));
            }
            BuildPatch::ReplaceAbility { old, new }
                if old == new
                    || combat.ability(old).is_none()
                    || combat.ability(new).is_none()
                    || unit.abilities().binary_search(&old).is_err()
                    || unit.abilities().binary_search(&new).is_err() =>
            {
                return Err(character_error(
                    BuildCatalogErrorKind::InvalidTracePatch,
                    form,
                ));
            }
            BuildPatch::AdjustAbilityLevel { family, .. }
                if definition.ability_level_table(family).is_none() =>
            {
                return Err(character_error(
                    BuildCatalogErrorKind::InvalidTracePatch,
                    form,
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_eidolon_patches(
    definition: &CharacterBuildDefinition,
    combat: &CombatCatalog,
    unit: &starclock_combat::catalog::definition::UnitDefinition,
) -> Result<(), BuildCatalogError> {
    let form = definition.form;
    let mut abilities = definition
        .abilities()
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let mut rules = definition
        .rule_bundles()
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let mut modifiers = definition
        .modifiers()
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    for patch in definition
        .eidolons()
        .ranks()
        .iter()
        .flat_map(|rank| rank.patches())
    {
        let valid = match *patch {
            BuildPatch::AddAbility(id) => {
                combat.ability(id).is_some()
                    && unit.abilities().binary_search(&id).is_ok()
                    && abilities.insert(id)
            }
            BuildPatch::AddRuleBundle(id) => {
                combat.rule_bundle(id).is_some()
                    && unit.rule_bundles().binary_search(&id).is_ok()
                    && rules.insert(id)
            }
            BuildPatch::RemoveRuleBundle(id) => rules.remove(&id),
            BuildPatch::AddModifier(id) => combat.modifier(id).is_some() && modifiers.insert(id),
            BuildPatch::ReplaceAbility { old, new } => {
                old != new
                    && combat.ability(new).is_some()
                    && unit.abilities().binary_search(&new).is_ok()
                    && abilities.remove(&old)
                    && abilities.insert(new)
            }
            BuildPatch::AdjustAbilityLevel { family, .. } => {
                definition.ability_level_table(family).is_some()
            }
        };
        if !valid {
            return Err(character_error(
                BuildCatalogErrorKind::InvalidEidolonPatch,
                form,
            ));
        }
    }
    Ok(())
}

fn validate_level_adjustments(
    definition: &CharacterBuildDefinition,
) -> Result<(), BuildCatalogError> {
    let mut adjustments = std::collections::BTreeMap::<AbilityId, (i16, i16)>::new();
    let patches = definition
        .trace_graph
        .iter()
        .flat_map(|graph| graph.nodes())
        .flat_map(|node| node.patches())
        .chain(
            definition
                .eidolons()
                .ranks()
                .iter()
                .flat_map(|rank| rank.patches()),
        );
    for patch in patches {
        if let BuildPatch::AdjustAbilityLevel {
            family,
            bonus,
            cap_delta,
        } = *patch
        {
            let entry = adjustments.entry(family).or_default();
            entry.0 = entry.0.checked_add(i16::from(bonus)).ok_or_else(|| {
                character_error(
                    BuildCatalogErrorKind::InvalidAbilityAdjustment,
                    definition.form,
                )
            })?;
            entry.1 = entry.1.checked_add(i16::from(cap_delta)).ok_or_else(|| {
                character_error(
                    BuildCatalogErrorKind::InvalidAbilityAdjustment,
                    definition.form,
                )
            })?;
        }
    }
    for (family, (bonus, cap_delta)) in adjustments {
        let table = definition
            .ability_level_table(family)
            .expect("adjustment family was checked");
        let invested_cap = i16::from(table.invested_cap().get());
        let effective_min = 1_i16 + bonus;
        let effective_max = invested_cap + bonus;
        let adjusted_cap = invested_cap + cap_delta;
        let table_max = i16::from(table.maximum_effective_level().get());
        if effective_min < 1
            || effective_max > adjusted_cap
            || adjusted_cap < 1
            || adjusted_cap > table_max
        {
            return Err(character_error(
                BuildCatalogErrorKind::InvalidAbilityAdjustment,
                definition.form,
            ));
        }
    }
    Ok(())
}

fn validate_light_cone(
    definition: &mut LightConeDefinition,
    combat: &CombatCatalog,
) -> Result<(), BuildCatalogError> {
    let id = definition.id();
    definition.canonicalize().map_err(|error| {
        light_cone_error(
            match error {
                LightConeDefinitionError::InvalidStatCurve => {
                    BuildCatalogErrorKind::InvalidLightConeStatCurve
                }
                LightConeDefinitionError::IncompletePassiveRanks => {
                    BuildCatalogErrorKind::IncompleteLightConePassive
                }
            },
            id,
        )
    })?;
    for rank in definition.passive_ranks() {
        if rank.patches().is_empty() {
            return Err(light_cone_error(
                BuildCatalogErrorKind::InvalidLightConePassive,
                id,
            ));
        }
        let mut rules = std::collections::BTreeSet::new();
        let mut modifiers = std::collections::BTreeSet::new();
        for patch in rank.patches() {
            let valid = match *patch {
                BuildPatch::AddRuleBundle(rule) => {
                    combat.rule_bundle(rule).is_some() && rules.insert(rule)
                }
                BuildPatch::AddModifier(modifier) => {
                    combat.modifier(modifier).is_some() && modifiers.insert(modifier)
                }
                _ => false,
            };
            if !valid {
                return Err(light_cone_error(
                    BuildCatalogErrorKind::InvalidLightConePassive,
                    id,
                ));
            }
        }
    }
    Ok(())
}

const fn character_error(kind: BuildCatalogErrorKind, form: UnitDefinitionId) -> BuildCatalogError {
    BuildCatalogError::new(kind, Some(form))
}

const fn light_cone_error(kind: BuildCatalogErrorKind, id: LightConeId) -> BuildCatalogError {
    BuildCatalogError::new_light_cone(kind, id)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildCatalogErrorKind {
    IncompatibleCombatRevision,
    DuplicateCharacter,
    MissingCombatForm,
    InvalidStatCurve,
    NonCanonicalReferences,
    InvalidAbilityBinding,
    InvalidRuleBinding,
    InvalidModifierBinding,
    DuplicateAbilityFamily,
    InvalidAbilityCurve,
    DuplicateTraceNode,
    InvalidTraceGraph,
    InvalidTracePatch,
    MissingEidolonSet,
    IncompleteEidolonSet,
    InvalidEidolonSet,
    InvalidEidolonPatch,
    InvalidAbilityAdjustment,
    DuplicateLightCone,
    InvalidLightConeStatCurve,
    IncompleteLightConePassive,
    InvalidLightConePassive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildCatalogError {
    kind: BuildCatalogErrorKind,
    form: Option<UnitDefinitionId>,
    light_cone: Option<LightConeId>,
}

impl BuildCatalogError {
    const fn new(kind: BuildCatalogErrorKind, form: Option<UnitDefinitionId>) -> Self {
        Self {
            kind,
            form,
            light_cone: None,
        }
    }
    const fn new_light_cone(kind: BuildCatalogErrorKind, light_cone: LightConeId) -> Self {
        Self {
            kind,
            form: None,
            light_cone: Some(light_cone),
        }
    }
    #[must_use]
    pub const fn kind(self) -> BuildCatalogErrorKind {
        self.kind
    }
    #[must_use]
    pub const fn form(self) -> Option<UnitDefinitionId> {
        self.form
    }
    #[must_use]
    pub const fn light_cone(self) -> Option<LightConeId> {
        self.light_cone
    }
}

impl std::fmt::Display for BuildCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid build catalog: {:?}", self.kind)
    }
}

impl std::error::Error for BuildCatalogError {}
