//! Immutable battle-domain catalog and validated construction API.

pub mod action;
pub mod builder;
pub mod definition;
pub mod encounter;
mod index;
mod lifecycle;
mod rule_validate;
pub mod selector;
mod table;

use crate::modifier::{model::ModifierDefinition, registry::ModifierRegistry};
use crate::{
    AbilityId, AiGraphId, EffectDefinitionId, EncounterId, EnemyDefinitionId, ModifierDefinitionId,
    ProgramId, RuleBundleId, RuleId, SelectorId, UnitDefinitionId,
};
use definition::{
    AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
    RuleBundle, RuleDefinition, SelectorDefinition, UnitDefinition,
};
use index::TriggerDefinitionIndex;
use table::DefinitionTable;

use crate::{
    TriggerId,
    rule::model::{RuleEventKind, TriggerPhase},
};

/// Human-readable immutable catalog revision.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CatalogRevision(Box<str>);

impl CatalogRevision {
    /// Returns the validated revision string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Exact digest of the configuration input represented by this catalog.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CatalogDigest([u8; 32]);

impl CatalogDigest {
    /// Returns the canonical 32-byte digest.
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

/// Immutable validated definitions shared by isolated battles.
#[derive(Debug)]
pub struct CombatCatalog {
    revision: CatalogRevision,
    digest: CatalogDigest,
    units: DefinitionTable<UnitDefinitionId, UnitDefinition>,
    linked_units: DefinitionTable<UnitDefinitionId, crate::LinkedUnitCatalogDefinition>,
    countdowns: DefinitionTable<u32, crate::CountdownCatalogDefinition>,
    abilities: DefinitionTable<AbilityId, AbilityDefinition>,
    effects: DefinitionTable<EffectDefinitionId, EffectDefinition>,
    rules: DefinitionTable<RuleId, RuleDefinition>,
    programs: DefinitionTable<ProgramId, ProgramDefinition>,
    selectors: DefinitionTable<SelectorId, SelectorDefinition>,
    rule_bundles: DefinitionTable<RuleBundleId, RuleBundle>,
    modifiers: ModifierRegistry,
    ai_graphs: DefinitionTable<AiGraphId, encounter::AiGraphDefinition>,
    enemies: DefinitionTable<EnemyDefinitionId, EnemyDefinition>,
    encounters: DefinitionTable<EncounterId, EncounterDefinition>,
    trigger_index: TriggerDefinitionIndex,
}

impl CombatCatalog {
    /// Returns the catalog revision.
    #[must_use]
    pub const fn revision(&self) -> &CatalogRevision {
        &self.revision
    }
    /// Returns the exact configuration digest.
    #[must_use]
    pub const fn digest(&self) -> CatalogDigest {
        self.digest
    }
    /// Returns total definition count across foundational tables.
    #[must_use]
    pub fn definition_count(&self) -> usize {
        self.units.len()
            + self.linked_units.len()
            + self.countdowns.len()
            + self.abilities.len()
            + self.effects.len()
            + self.rules.len()
            + self.programs.len()
            + self.selectors.len()
            + self.rule_bundles.len()
            + self.modifiers.len()
            + self.modifiers.group_count()
            + self.ai_graphs.len()
            + self.enemies.len()
            + self.encounters.len()
    }

    /// Returns the number of executable triggers in the compiled immutable index.
    #[must_use]
    pub fn trigger_count(&self) -> usize {
        self.trigger_index.len()
    }

    /// Returns `(rule, trigger)` identities in deterministic definition order.
    pub fn trigger_ids(
        &self,
        event: RuleEventKind,
        phase: TriggerPhase,
    ) -> impl ExactSizeIterator<Item = (RuleId, TriggerId)> + '_ {
        self.trigger_index
            .get(event, phase)
            .iter()
            .map(|entry| (entry.rule, entry.trigger))
    }

    /// Looks up a unit definition by stable ID.
    #[must_use]
    pub fn unit(&self, id: UnitDefinitionId) -> Option<&UnitDefinition> {
        self.units.get(id)
    }
    /// Looks up an ability definition by stable ID.
    #[must_use]
    pub fn ability(&self, id: AbilityId) -> Option<&AbilityDefinition> {
        self.abilities.get(id)
    }
    /// Looks up an effect definition by stable ID.
    #[must_use]
    pub fn effect(&self, id: EffectDefinitionId) -> Option<&EffectDefinition> {
        self.effects.get(id)
    }
    /// Looks up a rule definition by stable ID.
    #[must_use]
    pub fn rule(&self, id: RuleId) -> Option<&RuleDefinition> {
        self.rules.get(id)
    }
    /// Looks up a program definition by stable ID.
    #[must_use]
    pub fn program(&self, id: ProgramId) -> Option<&ProgramDefinition> {
        self.programs.get(id)
    }
    /// Looks up a selector definition by stable ID.
    #[must_use]
    pub fn selector(&self, id: SelectorId) -> Option<&SelectorDefinition> {
        self.selectors.get(id)
    }
    /// Looks up a rule bundle by stable ID.
    #[must_use]
    pub fn rule_bundle(&self, id: RuleBundleId) -> Option<&RuleBundle> {
        self.rule_bundles.get(id)
    }
    /// Looks up a modifier definition by stable ID.
    #[must_use]
    pub fn modifier(&self, id: ModifierDefinitionId) -> Option<&ModifierDefinition> {
        self.modifiers.definition(id)
    }
    pub(crate) const fn modifier_registry(&self) -> &ModifierRegistry {
        &self.modifiers
    }
    /// Looks up a validated finite enemy AI graph by stable ID.
    #[must_use]
    pub fn ai_graph(&self, id: AiGraphId) -> Option<&encounter::AiGraphDefinition> {
        self.ai_graphs.get(id)
    }
    /// Looks up an enemy definition by stable ID.
    #[must_use]
    pub fn enemy(&self, id: EnemyDefinitionId) -> Option<&EnemyDefinition> {
        self.enemies.get(id)
    }
    /// Looks up an encounter definition by stable ID.
    #[must_use]
    pub fn encounter(&self, id: EncounterId) -> Option<&EncounterDefinition> {
        self.encounters.get(id)
    }

    /// Iterates unit IDs in canonical numeric order.
    pub fn unit_ids(&self) -> impl ExactSizeIterator<Item = UnitDefinitionId> + '_ {
        self.units.ids()
    }
    /// Iterates program IDs in canonical numeric order.
    pub fn program_ids(&self) -> impl ExactSizeIterator<Item = ProgramId> + '_ {
        self.programs.ids()
    }
    /// Iterates selector IDs in canonical numeric order.
    pub fn selector_ids(&self) -> impl ExactSizeIterator<Item = SelectorId> + '_ {
        self.selectors.ids()
    }
}

impl crate::rule::evaluate::ProgramLookup for CombatCatalog {
    fn program_steps(&self, id: ProgramId) -> Option<&[crate::rule::model::ProgramStep]> {
        self.program(id).map(ProgramDefinition::steps)
    }
}
