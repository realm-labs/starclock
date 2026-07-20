//! Deterministic `CombatCatalog` construction and cross-reference validation.

use std::sync::Arc;

use crate::ProgramId;

use super::{
    CatalogDigest, CatalogRevision, CombatCatalog,
    definition::{
        AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition,
        ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition, UnitDefinition,
    },
    table::{DefinitionTable, DuplicateId},
};
use crate::modifier::{
    model::{ModifierDefinition, ModifierStackingGroup},
    registry::ModifierRegistry,
};

/// Foundational definition family named by catalog diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DefinitionKind {
    /// Generic combat unit form.
    Unit,
    /// Ability entry point.
    Ability,
    /// Effect definition.
    Effect,
    /// Typed battle rule.
    Rule,
    /// Finite typed program.
    Program,
    /// Deterministic selector.
    Selector,
    /// Ordered rule bundle.
    RuleBundle,
    /// Modifier definition.
    Modifier,
    /// Enemy definition.
    Enemy,
    /// Encounter definition.
    Encounter,
}

impl DefinitionKind {
    const fn name(self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::Ability => "ability",
            Self::Effect => "effect",
            Self::Rule => "rule",
            Self::Program => "program",
            Self::Selector => "selector",
            Self::RuleBundle => "rule bundle",
            Self::Modifier => "modifier",
            Self::Enemy => "enemy",
            Self::Encounter => "encounter",
        }
    }
}

/// Stable category for catalog-construction failure.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CatalogBuildErrorKind {
    /// Revision/digest identity is missing or malformed.
    InvalidCatalogIdentity,
    /// Two definitions in one typed table share an ID.
    DuplicateDefinition,
    /// A set-like reference list is not strictly ordered and unique.
    NonCanonicalReferences,
    /// A typed reference points to no definition.
    MissingReference,
    /// The static program call graph contains a cycle.
    ProgramCycle,
    /// A definition's local executable shape violates its domain contract.
    InvalidDefinition,
}

/// Typed, deterministic catalog-construction error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogBuildError {
    kind: CatalogBuildErrorKind,
    message: String,
    program_cycle: Box<[ProgramId]>,
}

impl CatalogBuildError {
    /// Returns the stable error category.
    #[must_use]
    pub const fn kind(&self) -> CatalogBuildErrorKind {
        self.kind
    }
    /// Returns a canonical cycle path when `kind` is `ProgramCycle`.
    #[must_use]
    pub fn program_cycle(&self) -> &[ProgramId] {
        &self.program_cycle
    }
}

impl std::fmt::Display for CatalogBuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CatalogBuildError {}

/// Public integration builder accepting only Starclock-owned domain definitions.
#[derive(Debug)]
pub struct CombatCatalogBuilder {
    revision: String,
    digest: [u8; 32],
    units: Vec<UnitDefinition>,
    abilities: Vec<AbilityDefinition>,
    effects: Vec<EffectDefinition>,
    rules: Vec<RuleDefinition>,
    programs: Vec<ProgramDefinition>,
    selectors: Vec<SelectorDefinition>,
    rule_bundles: Vec<RuleBundle>,
    modifiers: Vec<ModifierDefinition>,
    modifier_groups: Vec<ModifierStackingGroup>,
    enemies: Vec<EnemyDefinition>,
    encounters: Vec<EncounterDefinition>,
}

impl CombatCatalogBuilder {
    /// Starts a builder for an exact revision and configuration digest.
    #[must_use]
    pub fn new(revision: impl Into<String>, digest: [u8; 32]) -> Self {
        Self {
            revision: revision.into(),
            digest,
            units: Vec::new(),
            abilities: Vec::new(),
            effects: Vec::new(),
            rules: Vec::new(),
            programs: Vec::new(),
            selectors: Vec::new(),
            rule_bundles: Vec::new(),
            modifiers: Vec::new(),
            modifier_groups: Vec::new(),
            enemies: Vec::new(),
            encounters: Vec::new(),
        }
    }

    /// Adds a unit definition.
    pub fn add_unit(&mut self, definition: UnitDefinition) {
        self.units.push(definition);
    }
    /// Adds an ability definition.
    pub fn add_ability(&mut self, definition: AbilityDefinition) {
        self.abilities.push(definition);
    }
    /// Adds an effect definition.
    pub fn add_effect(&mut self, definition: EffectDefinition) {
        self.effects.push(definition);
    }
    /// Adds a rule definition.
    pub fn add_rule(&mut self, definition: RuleDefinition) {
        self.rules.push(definition);
    }
    /// Adds a program definition.
    pub fn add_program(&mut self, definition: ProgramDefinition) {
        self.programs.push(definition);
    }
    /// Adds a selector definition.
    pub fn add_selector(&mut self, definition: SelectorDefinition) {
        self.selectors.push(definition);
    }
    /// Adds an ordered rule bundle.
    pub fn add_rule_bundle(&mut self, definition: RuleBundle) {
        self.rule_bundles.push(definition);
    }
    /// Adds a modifier definition.
    pub fn add_modifier(&mut self, definition: ModifierDefinition) {
        self.modifiers.push(definition);
    }
    /// Adds a modifier stacking group.
    pub fn add_modifier_group(&mut self, definition: ModifierStackingGroup) {
        self.modifier_groups.push(definition);
    }
    /// Adds an enemy definition.
    pub fn add_enemy(&mut self, definition: EnemyDefinition) {
        self.enemies.push(definition);
    }
    /// Adds an encounter definition.
    pub fn add_encounter(&mut self, definition: EncounterDefinition) {
        self.encounters.push(definition);
    }

    /// Validates all inputs and returns an immutable catalog shared by battles.
    pub fn build(self) -> Result<Arc<CombatCatalog>, CatalogBuildError> {
        validate_identity(&self.revision, &self.digest)?;
        let modifiers =
            ModifierRegistry::new(self.modifier_groups, self.modifiers).map_err(|source| {
                error(CatalogBuildErrorKind::InvalidDefinition, source.to_string())
            })?;
        let catalog = CombatCatalog {
            revision: CatalogRevision(self.revision.into_boxed_str()),
            digest: CatalogDigest(self.digest),
            units: table(self.units, DefinitionKind::Unit)?,
            abilities: table(self.abilities, DefinitionKind::Ability)?,
            effects: table(self.effects, DefinitionKind::Effect)?,
            rules: table(self.rules, DefinitionKind::Rule)?,
            programs: table(self.programs, DefinitionKind::Program)?,
            selectors: table(self.selectors, DefinitionKind::Selector)?,
            rule_bundles: table(self.rule_bundles, DefinitionKind::RuleBundle)?,
            modifiers,
            enemies: table(self.enemies, DefinitionKind::Enemy)?,
            encounters: table(self.encounters, DefinitionKind::Encounter)?,
            trigger_index: super::index::TriggerDefinitionIndex::default(),
        };
        validate_references(&catalog)?;
        validate_program_cycles(&catalog)?;
        super::rule_validate::validate(&catalog)?;
        let mut catalog = catalog;
        catalog.trigger_index = super::index::TriggerDefinitionIndex::compile(&catalog.rules);
        Ok(Arc::new(catalog))
    }
}

fn table<I, D>(
    definitions: Vec<D>,
    kind: DefinitionKind,
) -> Result<DefinitionTable<I, D>, CatalogBuildError>
where
    I: Copy + core::fmt::Debug + Ord,
    D: super::table::Identified<I>,
{
    DefinitionTable::from_unsorted(definitions).map_err(|DuplicateId(id)| {
        error(
            CatalogBuildErrorKind::DuplicateDefinition,
            format!("duplicate {} definition ID {id:?}", kind.name()),
        )
    })
}

fn validate_identity(revision: &str, digest: &[u8; 32]) -> Result<(), CatalogBuildError> {
    if revision.is_empty()
        || revision.len() > 128
        || !revision.bytes().all(|byte| byte.is_ascii_graphic())
        || digest.iter().all(|byte| *byte == 0)
    {
        return Err(error(
            CatalogBuildErrorKind::InvalidCatalogIdentity,
            "invalid catalog revision or zero digest",
        ));
    }
    Ok(())
}

fn validate_references(catalog: &CombatCatalog) -> Result<(), CatalogBuildError> {
    for id in catalog.units.ids() {
        let unit = catalog
            .units
            .get(id)
            .expect("ID originated from this table");
        canonical(
            unit.abilities(),
            DefinitionKind::Unit,
            id.get(),
            "abilities",
        )?;
        canonical(
            unit.rule_bundles(),
            DefinitionKind::Unit,
            id.get(),
            "rule bundles",
        )?;
        require_all(
            unit.abilities(),
            |value| catalog.abilities.get(value).is_some(),
            DefinitionKind::Unit,
            id.get(),
            DefinitionKind::Ability,
        )?;
        require_all(
            unit.rule_bundles(),
            |value| catalog.rule_bundles.get(value).is_some(),
            DefinitionKind::Unit,
            id.get(),
            DefinitionKind::RuleBundle,
        )?;
    }
    for id in catalog.abilities.ids() {
        let ability = catalog
            .abilities
            .get(id)
            .expect("ID originated from this table");
        canonical(
            ability.effects(),
            DefinitionKind::Ability,
            id.get(),
            "effects",
        )?;
        require(
            catalog.programs.get(ability.program()).is_some(),
            DefinitionKind::Ability,
            id.get(),
            DefinitionKind::Program,
            ability.program().get(),
        )?;
        require(
            catalog.selectors.get(ability.selector()).is_some(),
            DefinitionKind::Ability,
            id.get(),
            DefinitionKind::Selector,
            ability.selector().get(),
        )?;
        if ability.action().is_some()
            && catalog
                .selectors
                .get(ability.selector())
                .is_some_and(|selector| selector.unit_targets().is_none())
        {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!(
                    "ability definition {} requires executable unit-target selector {}",
                    id.get(),
                    ability.selector().get()
                ),
            ));
        }
        if ability.action().is_some_and(|action| {
            action.kind() == super::action::AbilityKind::Ultimate
                && action.resources().skill_point_cost() == 0
                && action.resources().energy_cost() == crate::Energy::ZERO
        }) {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!(
                    "ultimate ability definition {} requires a payable current resource cost",
                    id.get()
                ),
            ));
        }
        if let Some(action) = ability.action() {
            for operation in action
                .hits()
                .iter()
                .flat_map(super::action::ActionHitDefinition::operations)
            {
                let super::action::HitOperationDefinition::QueueAction(queue) = operation else {
                    match operation {
                        super::action::HitOperationDefinition::SummonLinked(linked) => {
                            let combatant = linked.combatant();
                            if catalog.units.get(combatant.form()).is_none()
                                || combatant
                                    .abilities()
                                    .iter()
                                    .any(|ability| catalog.abilities.get(*ability).is_none())
                                || combatant
                                    .rule_bundles()
                                    .iter()
                                    .any(|bundle| catalog.rule_bundles.get(*bundle).is_none())
                                || combatant.modifiers().iter().any(|modifier| {
                                    catalog.modifiers.definition(*modifier).is_none()
                                })
                                || linked.action_ability().is_some_and(|ability| {
                                    catalog
                                        .abilities
                                        .get(ability)
                                        .and_then(super::definition::AbilityDefinition::action)
                                        .is_none_or(|action| {
                                            !matches!(
                                                (linked.kind(), action.kind()),
                                                (
                                                    crate::LinkedEntityKind::Summon,
                                                    super::action::AbilityKind::Summon
                                                ) | (
                                                    crate::LinkedEntityKind::Memosprite,
                                                    super::action::AbilityKind::Memosprite
                                                )
                                            )
                                        })
                                })
                            {
                                return Err(error(
                                    CatalogBuildErrorKind::InvalidDefinition,
                                    format!(
                                        "ability definition {} has an invalid linked-unit definition",
                                        id.get()
                                    ),
                                ));
                            }
                        }
                        super::action::HitOperationDefinition::Transform(transform) => {
                            let valid =
                                catalog.units.get(transform.replacement_form()).is_some_and(
                                    |unit| {
                                        transform.replacement_abilities().iter().all(|ability| {
                                            unit.abilities().binary_search(ability).is_ok()
                                                && catalog.abilities.get(*ability).is_some()
                                        })
                                    },
                                ) && transform.countdown().is_none_or(|countdown| {
                                    catalog
                                        .abilities
                                        .get(countdown.ability())
                                        .and_then(super::definition::AbilityDefinition::action)
                                        .is_some_and(|action| {
                                            action.kind() == super::action::AbilityKind::Countdown
                                        })
                                });
                            if !valid {
                                return Err(error(
                                    CatalogBuildErrorKind::InvalidDefinition,
                                    format!(
                                        "ability definition {} has an invalid transformation definition",
                                        id.get()
                                    ),
                                ));
                            }
                        }
                        _ => {}
                    }
                    continue;
                };
                let Some(queued) = catalog
                    .abilities
                    .get(queue.ability())
                    .and_then(super::definition::AbilityDefinition::action)
                else {
                    return Err(error(
                        CatalogBuildErrorKind::InvalidDefinition,
                        format!(
                            "ability definition {} queues missing executable ability {}",
                            id.get(),
                            queue.ability().get()
                        ),
                    ));
                };
                let compatible = matches!(
                    (queue.origin(), queued.kind()),
                    (
                        crate::ActionOrigin::FollowUp,
                        super::action::AbilityKind::FollowUp
                    ) | (
                        crate::ActionOrigin::UltimateInterrupt,
                        super::action::AbilityKind::Ultimate
                    ) | (
                        crate::ActionOrigin::Counter,
                        super::action::AbilityKind::Counter
                    ) | (
                        crate::ActionOrigin::ExtraTurn,
                        super::action::AbilityKind::ExtraTurn
                    ) | (
                        crate::ActionOrigin::ExtraAction | crate::ActionOrigin::Forced,
                        super::action::AbilityKind::ExtraAction
                    ) | (
                        crate::ActionOrigin::DelayedAction,
                        super::action::AbilityKind::DelayedAction
                    )
                ) || (queue.origin() == crate::ActionOrigin::Forced
                    && queued.kind() == super::action::AbilityKind::Skill
                    && queued
                        .tags()
                        .contains(super::action::AbilityTag::ElationSkill));
                if !compatible {
                    return Err(error(
                        CatalogBuildErrorKind::InvalidDefinition,
                        format!(
                            "ability definition {} queues ability {} with an incompatible origin",
                            id.get(),
                            queue.ability().get()
                        ),
                    ));
                }
            }
        }
        require_all(
            ability.effects(),
            |value| catalog.effects.get(value).is_some(),
            DefinitionKind::Ability,
            id.get(),
            DefinitionKind::Effect,
        )?;
    }
    for id in catalog.effects.ids() {
        let effect = catalog
            .effects
            .get(id)
            .expect("ID originated from this table");
        canonical(effect.rules(), DefinitionKind::Effect, id.get(), "rules")?;
        canonical(
            effect.modifiers(),
            DefinitionKind::Effect,
            id.get(),
            "modifiers",
        )?;
        require_all(
            effect.rules(),
            |value| catalog.rules.get(value).is_some(),
            DefinitionKind::Effect,
            id.get(),
            DefinitionKind::Rule,
        )?;
        require_all(
            effect.modifiers(),
            |value| catalog.modifiers.definition(value).is_some(),
            DefinitionKind::Effect,
            id.get(),
            DefinitionKind::Modifier,
        )?;
    }
    for id in catalog.rules.ids() {
        let rule = catalog
            .rules
            .get(id)
            .expect("ID originated from this table");
        canonical(rule.programs(), DefinitionKind::Rule, id.get(), "programs")?;
        canonical(
            rule.selectors(),
            DefinitionKind::Rule,
            id.get(),
            "selectors",
        )?;
        require_all(
            rule.programs(),
            |value| catalog.programs.get(value).is_some(),
            DefinitionKind::Rule,
            id.get(),
            DefinitionKind::Program,
        )?;
        require_all(
            rule.selectors(),
            |value| catalog.selectors.get(value).is_some(),
            DefinitionKind::Rule,
            id.get(),
            DefinitionKind::Selector,
        )?;
    }
    validate_program_references(catalog)?;
    for id in catalog.rule_bundles.ids() {
        let bundle = catalog
            .rule_bundles
            .get(id)
            .expect("ID originated from this table");
        require_all(
            bundle.rules(),
            |value| catalog.rules.get(value).is_some(),
            DefinitionKind::RuleBundle,
            id.get(),
            DefinitionKind::Rule,
        )?;
    }
    for id in catalog.enemies.ids() {
        let enemy = catalog
            .enemies
            .get(id)
            .expect("ID originated from this table");
        canonical(
            enemy.abilities(),
            DefinitionKind::Enemy,
            id.get(),
            "abilities",
        )?;
        require(
            catalog.units.get(enemy.unit()).is_some(),
            DefinitionKind::Enemy,
            id.get(),
            DefinitionKind::Unit,
            enemy.unit().get(),
        )?;
        require_all(
            enemy.abilities(),
            |value| catalog.abilities.get(value).is_some(),
            DefinitionKind::Enemy,
            id.get(),
            DefinitionKind::Ability,
        )?;
    }
    for id in catalog.encounters.ids() {
        let encounter = catalog
            .encounters
            .get(id)
            .expect("ID originated from this table");
        canonical(
            encounter.rule_bundles(),
            DefinitionKind::Encounter,
            id.get(),
            "rule bundles",
        )?;
        if encounter.waves().is_empty() || encounter.waves().iter().any(|wave| wave.is_empty()) {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!(
                    "encounter definition {} requires non-empty ordered waves",
                    id.get()
                ),
            ));
        }
        for wave in encounter.waves() {
            require_all(
                wave,
                |value| catalog.enemies.get(value).is_some(),
                DefinitionKind::Encounter,
                id.get(),
                DefinitionKind::Enemy,
            )?;
        }
        require_all(
            encounter.enemies(),
            |value| catalog.enemies.get(value).is_some(),
            DefinitionKind::Encounter,
            id.get(),
            DefinitionKind::Enemy,
        )?;
        require_all(
            encounter.rule_bundles(),
            |value| catalog.rule_bundles.get(value).is_some(),
            DefinitionKind::Encounter,
            id.get(),
            DefinitionKind::RuleBundle,
        )?;
    }
    Ok(())
}

fn validate_program_references(catalog: &CombatCatalog) -> Result<(), CatalogBuildError> {
    for id in catalog.programs.ids() {
        let program = catalog
            .programs
            .get(id)
            .expect("ID originated from this table");
        canonical(
            program.selectors(),
            DefinitionKind::Program,
            id.get(),
            "selectors",
        )?;
        canonical(
            program.effects(),
            DefinitionKind::Program,
            id.get(),
            "effects",
        )?;
        canonical(
            program.modifiers(),
            DefinitionKind::Program,
            id.get(),
            "modifiers",
        )?;
        require_all(
            program.called_programs(),
            |value| catalog.programs.get(value).is_some(),
            DefinitionKind::Program,
            id.get(),
            DefinitionKind::Program,
        )?;
        require_all(
            program.selectors(),
            |value| catalog.selectors.get(value).is_some(),
            DefinitionKind::Program,
            id.get(),
            DefinitionKind::Selector,
        )?;
        require_all(
            program.effects(),
            |value| catalog.effects.get(value).is_some(),
            DefinitionKind::Program,
            id.get(),
            DefinitionKind::Effect,
        )?;
        require_all(
            program.modifiers(),
            |value| catalog.modifiers.definition(value).is_some(),
            DefinitionKind::Program,
            id.get(),
            DefinitionKind::Modifier,
        )?;
    }
    Ok(())
}

fn canonical<I: Copy + Ord>(
    values: &[I],
    owner: DefinitionKind,
    owner_id: u32,
    field: &str,
) -> Result<(), CatalogBuildError> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(error(
            CatalogBuildErrorKind::NonCanonicalReferences,
            format!("{} {owner_id} has non-canonical {field}", owner.name()),
        ));
    }
    Ok(())
}

fn require_all<I: Copy + Into<u32>>(
    values: &[I],
    mut exists: impl FnMut(I) -> bool,
    owner: DefinitionKind,
    owner_id: u32,
    target: DefinitionKind,
) -> Result<(), CatalogBuildError> {
    for value in values {
        if !exists(*value) {
            return require(false, owner, owner_id, target, (*value).into());
        }
    }
    Ok(())
}

fn require(
    exists: bool,
    owner: DefinitionKind,
    owner_id: u32,
    target: DefinitionKind,
    target_id: u32,
) -> Result<(), CatalogBuildError> {
    if !exists {
        return Err(error(
            CatalogBuildErrorKind::MissingReference,
            format!(
                "{} {owner_id} refers to missing {} {target_id}",
                owner.name(),
                target.name()
            ),
        ));
    }
    Ok(())
}

fn validate_program_cycles(catalog: &CombatCatalog) -> Result<(), CatalogBuildError> {
    let mut marks = vec![0_u8; catalog.programs.len()];
    let ids = catalog.programs.ids().collect::<Vec<_>>();
    let mut path = Vec::new();
    for index in 0..ids.len() {
        visit_program(index, &ids, &mut marks, &mut path, catalog)?;
    }
    Ok(())
}

fn visit_program(
    index: usize,
    ids: &[ProgramId],
    marks: &mut [u8],
    path: &mut Vec<ProgramId>,
    catalog: &CombatCatalog,
) -> Result<(), CatalogBuildError> {
    if marks[index] == 2 {
        return Ok(());
    }
    if marks[index] == 1 {
        let start = path
            .iter()
            .position(|value| *value == ids[index])
            .unwrap_or(0);
        let mut cycle = path[start..].to_vec();
        cycle.push(ids[index]);
        return Err(CatalogBuildError {
            kind: CatalogBuildErrorKind::ProgramCycle,
            message: format!("program call cycle starts at {}", ids[index].get()),
            program_cycle: cycle.into_boxed_slice(),
        });
    }
    marks[index] = 1;
    path.push(ids[index]);
    let program = catalog
        .programs
        .get(ids[index])
        .expect("ID originated from this table");
    for called in program.called_programs() {
        let called_index = ids
            .binary_search(called)
            .expect("references were validated");
        visit_program(called_index, ids, marks, path, catalog)?;
    }
    path.pop();
    marks[index] = 2;
    Ok(())
}

pub(super) fn catalog_error(
    kind: CatalogBuildErrorKind,
    message: impl Into<String>,
) -> CatalogBuildError {
    CatalogBuildError {
        kind,
        message: message.into(),
        program_cycle: Box::new([]),
    }
}

fn error(kind: CatalogBuildErrorKind, message: impl Into<String>) -> CatalogBuildError {
    catalog_error(kind, message)
}
