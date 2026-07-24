//! Deterministic `CombatCatalog` construction and cross-reference validation.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::ProgramId;

use super::{
    CatalogDigest, CatalogRevision, CombatCatalog,
    definition::{
        AbilityDefinition, AbilityParameterDefinition, EffectDefinition, EncounterDefinition,
        EnemyDefinition, ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition,
        UnitDefinition,
    },
    encounter::AiGraphDefinition,
    table::{DefinitionTable, DuplicateId},
};
use crate::modifier::{
    model::{ModifierDefinition, ModifierStackingGroup},
    registry::ModifierRegistry,
};

mod composition;
mod lifecycle_validate;
mod parameter_validate;

/// Foundational definition family named by catalog diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DefinitionKind {
    /// Generic combat unit form.
    Unit,
    /// Complete linked-unit runtime template.
    LinkedUnit,
    /// Timeline-only countdown template.
    Countdown,
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
    /// Finite enemy AI graph.
    AiGraph,
    /// Encounter definition.
    Encounter,
}

impl DefinitionKind {
    const fn name(self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::LinkedUnit => "linked unit",
            Self::Countdown => "countdown",
            Self::Ability => "ability",
            Self::Effect => "effect",
            Self::Rule => "rule",
            Self::Program => "program",
            Self::Selector => "selector",
            Self::RuleBundle => "rule bundle",
            Self::Modifier => "modifier",
            Self::Enemy => "enemy",
            Self::AiGraph => "AI graph",
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
    linked_units: Vec<crate::LinkedUnitCatalogDefinition>,
    countdowns: Vec<crate::CountdownCatalogDefinition>,
    abilities: Vec<AbilityDefinition>,
    ability_parameters: Vec<AbilityParameterDefinition>,
    effects: Vec<EffectDefinition>,
    rules: Vec<RuleDefinition>,
    programs: Vec<ProgramDefinition>,
    selectors: Vec<SelectorDefinition>,
    rule_bundles: Vec<RuleBundle>,
    modifiers: Vec<ModifierDefinition>,
    modifier_groups: Vec<ModifierStackingGroup>,
    ai_graphs: Vec<AiGraphDefinition>,
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
            linked_units: Vec::new(),
            countdowns: Vec::new(),
            abilities: Vec::new(),
            ability_parameters: Vec::new(),
            effects: Vec::new(),
            rules: Vec::new(),
            programs: Vec::new(),
            selectors: Vec::new(),
            rule_bundles: Vec::new(),
            modifiers: Vec::new(),
            modifier_groups: Vec::new(),
            ai_graphs: Vec::new(),
            enemies: Vec::new(),
            encounters: Vec::new(),
        }
    }

    /// Adds a unit definition.
    pub fn add_unit(&mut self, definition: UnitDefinition) {
        self.units.push(definition);
    }
    /// Adds one complete Rule IR summon template.
    pub fn add_linked_unit(&mut self, definition: crate::LinkedUnitCatalogDefinition) {
        self.linked_units.push(definition);
    }
    /// Adds one timeline-only Rule IR countdown template.
    pub fn add_countdown(&mut self, definition: crate::CountdownCatalogDefinition) {
        self.countdowns.push(definition);
    }
    /// Adds an ability definition.
    pub fn add_ability(&mut self, definition: AbilityDefinition) {
        self.abilities.push(definition);
    }
    /// Adds one value for an exact effective-level ability definition.
    pub fn add_ability_parameter(&mut self, definition: AbilityParameterDefinition) {
        self.ability_parameters.push(definition);
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
    /// Adds a finite enemy AI graph.
    pub fn add_ai_graph(&mut self, definition: AiGraphDefinition) {
        self.ai_graphs.push(definition);
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
            linked_units: table(self.linked_units, DefinitionKind::LinkedUnit)?,
            countdowns: table(self.countdowns, DefinitionKind::Countdown)?,
            abilities: table(self.abilities, DefinitionKind::Ability)?,
            ability_parameters: parameter_validate::table(self.ability_parameters)?,
            effects: table(self.effects, DefinitionKind::Effect)?,
            rules: table(self.rules, DefinitionKind::Rule)?,
            programs: table(self.programs, DefinitionKind::Program)?,
            selectors: table(self.selectors, DefinitionKind::Selector)?,
            rule_bundles: table(self.rule_bundles, DefinitionKind::RuleBundle)?,
            modifiers,
            ai_graphs: table(self.ai_graphs, DefinitionKind::AiGraph)?,
            enemies: table(self.enemies, DefinitionKind::Enemy)?,
            encounters: table(self.encounters, DefinitionKind::Encounter)?,
            trigger_index: super::index::TriggerDefinitionIndex::default(),
        };
        validate_references(&catalog)?;
        validate_ai_graphs(&catalog)?;
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
    for ability in catalog.ability_parameters.keys() {
        if catalog.abilities.get(*ability).is_none() {
            return Err(error(
                CatalogBuildErrorKind::MissingReference,
                format!(
                    "ability parameters refer to missing ability {}",
                    ability.get()
                ),
            ));
        }
    }
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
        if unit
            .resources()
            .windows(2)
            .any(|pair| pair[0].stable_key() >= pair[1].stable_key())
        {
            return Err(error(
                CatalogBuildErrorKind::NonCanonicalReferences,
                format!("unit {} resources are not strictly key-ordered", id.get()),
            ));
        }
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
    for id in catalog.linked_units.ids() {
        let linked = catalog
            .linked_units
            .get(id)
            .expect("ID originated from this table")
            .definition();
        if !lifecycle_validate::valid_linked_definition(catalog, linked) {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!("linked-unit definition {} is invalid", id.get()),
            ));
        }
    }
    let mut countdown_abilities = BTreeSet::new();
    for code in catalog.countdowns.ids() {
        let countdown = catalog
            .countdowns
            .get(code)
            .expect("ID originated from this table")
            .definition();
        if catalog
            .abilities
            .get(countdown.ability())
            .and_then(super::definition::AbilityDefinition::action)
            .is_none_or(|action| action.kind() != super::action::AbilityKind::Countdown)
        {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!("countdown definition {code} has an invalid ability"),
            ));
        }
        if !countdown_abilities.insert(countdown.ability()) {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!(
                    "countdown definition {code} reuses ability {}",
                    countdown.ability().get()
                ),
            ));
        }
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
        if ability
            .programs()
            .windows(2)
            .any(|pair| pair[0].sequence() >= pair[1].sequence())
        {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!(
                    "ability definition {} has unordered phase programs",
                    id.get()
                ),
            ));
        }
        require_all(
            &ability
                .programs()
                .iter()
                .map(|binding| binding.program())
                .collect::<Vec<_>>(),
            |value| catalog.programs.get(value).is_some(),
            DefinitionKind::Ability,
            id.get(),
            DefinitionKind::Program,
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
                && !action.resources().has_payable_cost()
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
                            if !lifecycle_validate::valid_linked_definition(catalog, linked) {
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
                    && queued.tags().supports_forced_skill());
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
        let grants = effect.granted_abilities();
        let kind = DefinitionKind::Effect;
        let raw = id.get();
        canonical(effect.rules(), kind, raw, "rules")?;
        canonical(effect.modifiers(), kind, raw, "modifiers")?;
        canonical(grants, kind, raw, "granted abilities")?;
        require_all(
            effect.rules(),
            |value| catalog.rules.get(value).is_some(),
            kind,
            raw,
            DefinitionKind::Rule,
        )?;
        require_all(
            effect.modifiers(),
            |value| catalog.modifiers.definition(value).is_some(),
            kind,
            raw,
            DefinitionKind::Modifier,
        )?;
        require_all(
            grants,
            |value| catalog.abilities.get(value).is_some(),
            kind,
            raw,
            DefinitionKind::Ability,
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
        if let Some(graph_id) = enemy.ai_graph() {
            require(
                catalog.ai_graphs.get(graph_id).is_some(),
                DefinitionKind::Enemy,
                id.get(),
                DefinitionKind::AiGraph,
                graph_id.get(),
            )?;
            let graph = catalog
                .ai_graphs
                .get(graph_id)
                .expect("reference was just validated");
            for state in graph.states() {
                let abilities = state
                    .candidates()
                    .iter()
                    .map(|candidate| candidate.ability())
                    .chain(core::iter::once(state.mandatory_fallback()));
                for ability in abilities {
                    if enemy.abilities().binary_search(&ability).is_err() {
                        return Err(error(
                            CatalogBuildErrorKind::InvalidDefinition,
                            format!(
                                "enemy definition {} AI graph {} uses unbound ability {}",
                                id.get(),
                                graph_id.get(),
                                ability.get()
                            ),
                        ));
                    }
                }
            }
        }
        for phase in enemy.phases() {
            require(
                catalog.ai_graphs.get(phase.ai_graph()).is_some(),
                DefinitionKind::Enemy,
                id.get(),
                DefinitionKind::AiGraph,
                phase.ai_graph().get(),
            )?;
            if let Some(program) = phase.entry_program() {
                require(
                    catalog.programs.get(program).is_some(),
                    DefinitionKind::Enemy,
                    id.get(),
                    DefinitionKind::Program,
                    program.get(),
                )?;
            }
            if let super::encounter::EnemyPhaseTransitionModel::ReplaceLinkedVariant(replacement) =
                phase.transition()
            {
                require(
                    catalog.enemies.get(replacement).is_some(),
                    DefinitionKind::Enemy,
                    id.get(),
                    DefinitionKind::Enemy,
                    replacement.get(),
                )?;
            }
        }
        for link in enemy.links() {
            require(
                catalog.enemies.get(link.linked_enemy()).is_some(),
                DefinitionKind::Enemy,
                id.get(),
                DefinitionKind::Enemy,
                link.linked_enemy().get(),
            )?;
        }
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
        if encounter.waves().is_empty()
            || encounter.waves().iter().any(|wave| wave.slots().is_empty())
        {
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
                &wave
                    .slots()
                    .iter()
                    .map(|slot| slot.enemy())
                    .collect::<Vec<_>>(),
                |value| catalog.enemies.get(value).is_some(),
                DefinitionKind::Encounter,
                id.get(),
                DefinitionKind::Enemy,
            )?;
            for program in [wave.entry_program(), wave.exit_program()]
                .into_iter()
                .flatten()
            {
                require(
                    catalog.programs.get(program).is_some(),
                    DefinitionKind::Encounter,
                    id.get(),
                    DefinitionKind::Program,
                    program.get(),
                )?;
            }
            for slot in wave.slots() {
                if let Some(phase) = slot.initial_phase() {
                    let valid = catalog
                        .enemies
                        .get(slot.enemy())
                        .is_some_and(|enemy| enemy.phases().iter().any(|item| item.id() == phase));
                    if !valid {
                        return Err(error(
                            CatalogBuildErrorKind::InvalidDefinition,
                            format!(
                                "encounter definition {} wave {} slot {} has invalid phase {}",
                                id.get(),
                                wave.sequence(),
                                slot.spawn_sequence(),
                                phase.get()
                            ),
                        ));
                    }
                }
            }
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

fn validate_ai_graphs(catalog: &CombatCatalog) -> Result<(), CatalogBuildError> {
    for graph_id in catalog.ai_graphs.ids() {
        let graph = catalog
            .ai_graphs
            .get(graph_id)
            .expect("ID originated from table");
        let state_ids = graph
            .states()
            .iter()
            .map(|state| state.id())
            .collect::<BTreeSet<_>>();
        let mut candidate_ids = BTreeSet::new();
        let mut transition_ids = BTreeSet::new();
        let mut edges = BTreeMap::new();
        for state in graph.states() {
            let candidates_unique = state
                .candidates()
                .iter()
                .all(|item| candidate_ids.insert(item.id()));
            let transitions_unique = state
                .transitions()
                .iter()
                .all(|item| transition_ids.insert(item.id()));
            if state.candidates().is_empty() || !candidates_unique || !transitions_unique {
                return Err(error(
                    CatalogBuildErrorKind::InvalidDefinition,
                    format!(
                        "AI graph {} has an empty state or duplicate child ID",
                        graph_id.get()
                    ),
                ));
            }
            require(
                catalog.abilities.get(state.mandatory_fallback()).is_some(),
                DefinitionKind::AiGraph,
                graph_id.get(),
                DefinitionKind::Ability,
                state.mandatory_fallback().get(),
            )?;
            if let Some(program) = state.entry_program() {
                require(
                    catalog.programs.get(program).is_some(),
                    DefinitionKind::AiGraph,
                    graph_id.get(),
                    DefinitionKind::Program,
                    program.get(),
                )?;
            }
            for candidate in state.candidates() {
                require(
                    catalog.abilities.get(candidate.ability()).is_some(),
                    DefinitionKind::AiGraph,
                    graph_id.get(),
                    DefinitionKind::Ability,
                    candidate.ability().get(),
                )?;
                require(
                    catalog.selectors.get(candidate.target_selector()).is_some(),
                    DefinitionKind::AiGraph,
                    graph_id.get(),
                    DefinitionKind::Selector,
                    candidate.target_selector().get(),
                )?;
                if matches!(
                    candidate.selection(),
                    super::encounter::AiCandidateSelection::WeightedDraw { weight: 0, .. }
                ) {
                    return Err(error(
                        CatalogBuildErrorKind::InvalidDefinition,
                        format!("AI graph {} has a zero-weight candidate", graph_id.get()),
                    ));
                }
                if let super::encounter::AiNoTargetFallback::Transition(target) =
                    candidate.no_target()
                    && !state_ids.contains(&target)
                {
                    return Err(error(
                        CatalogBuildErrorKind::MissingReference,
                        format!(
                            "AI graph {} no-target transition refers outside the graph",
                            graph_id.get()
                        ),
                    ));
                }
                if let super::encounter::AiNoTargetFallback::UseFallbackAbility(ability) =
                    candidate.no_target()
                {
                    require(
                        catalog.abilities.get(ability).is_some(),
                        DefinitionKind::AiGraph,
                        graph_id.get(),
                        DefinitionKind::Ability,
                        ability.get(),
                    )?;
                }
            }
            let targets = state
                .transitions()
                .iter()
                .map(|item| item.target())
                .collect::<Vec<_>>();
            if targets.iter().any(|target| !state_ids.contains(target)) {
                return Err(error(
                    CatalogBuildErrorKind::MissingReference,
                    format!(
                        "AI graph {} transition refers outside the graph",
                        graph_id.get()
                    ),
                ));
            }
            edges.insert(state.id(), targets);
        }
        let mut reachable = BTreeSet::new();
        let mut stack = vec![graph.initial_state()];
        while let Some(state) = stack.pop() {
            if reachable.insert(state)
                && let Some(targets) = edges.get(&state)
            {
                stack.extend(targets.iter().rev().copied());
            }
        }
        if reachable != state_ids {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!("AI graph {} contains unreachable states", graph_id.get()),
            ));
        }
        let automatic = graph
            .states()
            .iter()
            .map(|state| {
                (
                    state.id(),
                    state
                        .transitions()
                        .iter()
                        .filter(|item| {
                            item.timing()
                                == super::encounter::AiTransitionTiming::AutomaticBeforeDecision
                        })
                        .map(|item| item.target())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        if has_cycle(
            graph.initial_state(),
            &automatic,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
        ) {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!(
                    "AI graph {} has an automatic transition cycle",
                    graph_id.get()
                ),
            ));
        }
    }
    Ok(())
}

fn has_cycle(
    state: crate::AiStateId,
    edges: &BTreeMap<crate::AiStateId, Vec<crate::AiStateId>>,
    visiting: &mut BTreeSet<crate::AiStateId>,
    visited: &mut BTreeSet<crate::AiStateId>,
) -> bool {
    if visited.contains(&state) {
        return false;
    }
    if !visiting.insert(state) {
        return true;
    }
    if edges.get(&state).is_some_and(|targets| {
        targets
            .iter()
            .any(|target| has_cycle(*target, edges, visiting, visited))
    }) {
        return true;
    }
    visiting.remove(&state);
    visited.insert(state);
    false
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
        for step in program.steps() {
            let crate::rule::model::ProgramStep::Operation(operation) = step else {
                continue;
            };
            lifecycle_validate::validate_program_operation(catalog, id, operation)?;
        }
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
