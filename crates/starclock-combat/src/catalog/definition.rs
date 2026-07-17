//! Immutable battle-domain definition inputs accepted by the catalog builder.

use crate::{
    AbilityId, EffectDefinitionId, EncounterId, EnemyDefinitionId, ModifierDefinitionId, ProgramId,
    RuleBundleId, RuleId, SelectorId, UnitDefinitionId,
};

macro_rules! leaf_definition {
    ($name:ident, $id:ty, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name {
            id: $id,
        }

        impl $name {
            /// Creates a leaf definition with a stable typed ID.
            #[must_use]
            pub const fn new(id: $id) -> Self {
                Self { id }
            }

            /// Returns the stable definition ID.
            #[must_use]
            pub const fn id(&self) -> $id {
                self.id
            }
        }
    };
}

leaf_definition!(
    SelectorDefinition,
    SelectorId,
    "Foundational typed selector definition; selector semantics are added by the Rule IR batch."
);
leaf_definition!(
    ModifierDefinition,
    ModifierDefinitionId,
    "Foundational modifier definition identity used by validated references."
);

/// Generic unit-form definition referencing combat abilities and rule bundles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnitDefinition {
    id: UnitDefinitionId,
    abilities: Box<[AbilityId]>,
    rule_bundles: Box<[RuleBundleId]>,
}

impl UnitDefinition {
    /// Creates an unvalidated builder input. Reference sets must be strictly ID-ordered.
    #[must_use]
    pub fn new(
        id: UnitDefinitionId,
        abilities: Vec<AbilityId>,
        rule_bundles: Vec<RuleBundleId>,
    ) -> Self {
        Self {
            id,
            abilities: abilities.into_boxed_slice(),
            rule_bundles: rule_bundles.into_boxed_slice(),
        }
    }

    /// Returns the stable definition ID.
    #[must_use]
    pub const fn id(&self) -> UnitDefinitionId {
        self.id
    }
    /// Returns the canonical ability-reference set.
    #[must_use]
    pub fn abilities(&self) -> &[AbilityId] {
        &self.abilities
    }
    /// Returns the canonical innate rule-bundle set.
    #[must_use]
    pub fn rule_bundles(&self) -> &[RuleBundleId] {
        &self.rule_bundles
    }
}

/// Ability entry point referencing one program, selector and applied effects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityDefinition {
    id: AbilityId,
    program: ProgramId,
    selector: SelectorId,
    effects: Box<[EffectDefinitionId]>,
}

impl AbilityDefinition {
    /// Creates an unvalidated ability definition.
    #[must_use]
    pub fn new(
        id: AbilityId,
        program: ProgramId,
        selector: SelectorId,
        effects: Vec<EffectDefinitionId>,
    ) -> Self {
        Self {
            id,
            program,
            selector,
            effects: effects.into_boxed_slice(),
        }
    }
    /// Returns the stable definition ID.
    #[must_use]
    pub const fn id(&self) -> AbilityId {
        self.id
    }
    /// Returns the authored program reference.
    #[must_use]
    pub const fn program(&self) -> ProgramId {
        self.program
    }
    /// Returns the targeting selector reference.
    #[must_use]
    pub const fn selector(&self) -> SelectorId {
        self.selector
    }
    /// Returns the canonical set of effects this ability may apply.
    #[must_use]
    pub fn effects(&self) -> &[EffectDefinitionId] {
        &self.effects
    }
}

/// Effect definition referencing attached rules and modifiers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectDefinition {
    id: EffectDefinitionId,
    rules: Box<[RuleId]>,
    modifiers: Box<[ModifierDefinitionId]>,
}

impl EffectDefinition {
    /// Creates an unvalidated effect definition.
    #[must_use]
    pub fn new(
        id: EffectDefinitionId,
        rules: Vec<RuleId>,
        modifiers: Vec<ModifierDefinitionId>,
    ) -> Self {
        Self {
            id,
            rules: rules.into_boxed_slice(),
            modifiers: modifiers.into_boxed_slice(),
        }
    }
    /// Returns the stable definition ID.
    #[must_use]
    pub const fn id(&self) -> EffectDefinitionId {
        self.id
    }
    /// Returns the canonical attached-rule set.
    #[must_use]
    pub fn rules(&self) -> &[RuleId] {
        &self.rules
    }
    /// Returns the canonical attached-modifier set.
    #[must_use]
    pub fn modifiers(&self) -> &[ModifierDefinitionId] {
        &self.modifiers
    }
}

/// Rule definition referencing typed programs and selectors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleDefinition {
    id: RuleId,
    programs: Box<[ProgramId]>,
    selectors: Box<[SelectorId]>,
}

impl RuleDefinition {
    /// Creates an unvalidated battle-rule definition.
    #[must_use]
    pub fn new(id: RuleId, programs: Vec<ProgramId>, selectors: Vec<SelectorId>) -> Self {
        Self {
            id,
            programs: programs.into_boxed_slice(),
            selectors: selectors.into_boxed_slice(),
        }
    }
    /// Returns the stable definition ID.
    #[must_use]
    pub const fn id(&self) -> RuleId {
        self.id
    }
    /// Returns the canonical program-reference set.
    #[must_use]
    pub fn programs(&self) -> &[ProgramId] {
        &self.programs
    }
    /// Returns the canonical selector-reference set.
    #[must_use]
    pub fn selectors(&self) -> &[SelectorId] {
        &self.selectors
    }
}

/// Finite typed-program graph node and its referenced domain definitions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramDefinition {
    id: ProgramId,
    called_programs: Box<[ProgramId]>,
    selectors: Box<[SelectorId]>,
    effects: Box<[EffectDefinitionId]>,
    modifiers: Box<[ModifierDefinitionId]>,
}

impl ProgramDefinition {
    /// Creates an unvalidated program definition. Call order is authored and preserved.
    #[must_use]
    pub fn new(
        id: ProgramId,
        called_programs: Vec<ProgramId>,
        selectors: Vec<SelectorId>,
        effects: Vec<EffectDefinitionId>,
        modifiers: Vec<ModifierDefinitionId>,
    ) -> Self {
        Self {
            id,
            called_programs: called_programs.into_boxed_slice(),
            selectors: selectors.into_boxed_slice(),
            effects: effects.into_boxed_slice(),
            modifiers: modifiers.into_boxed_slice(),
        }
    }
    /// Returns the stable definition ID.
    #[must_use]
    pub const fn id(&self) -> ProgramId {
        self.id
    }
    /// Returns nested program calls in authored execution order.
    #[must_use]
    pub fn called_programs(&self) -> &[ProgramId] {
        &self.called_programs
    }
    /// Returns the canonical selector-reference set.
    #[must_use]
    pub fn selectors(&self) -> &[SelectorId] {
        &self.selectors
    }
    /// Returns the canonical effect-reference set.
    #[must_use]
    pub fn effects(&self) -> &[EffectDefinitionId] {
        &self.effects
    }
    /// Returns the canonical modifier-reference set.
    #[must_use]
    pub fn modifiers(&self) -> &[ModifierDefinitionId] {
        &self.modifiers
    }
}

/// Ordered rule composition selected by a unit, encounter or resolved build.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleBundle {
    id: RuleBundleId,
    rules: Box<[RuleId]>,
}

impl RuleBundle {
    /// Creates an unvalidated rule bundle. Rule order is executable and preserved.
    #[must_use]
    pub fn new(id: RuleBundleId, rules: Vec<RuleId>) -> Self {
        Self {
            id,
            rules: rules.into_boxed_slice(),
        }
    }
    /// Returns the stable definition ID.
    #[must_use]
    pub const fn id(&self) -> RuleBundleId {
        self.id
    }
    /// Returns rules in authored binding order.
    #[must_use]
    pub fn rules(&self) -> &[RuleId] {
        &self.rules
    }
}

/// Enemy definition referencing its generic unit form and available abilities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnemyDefinition {
    id: EnemyDefinitionId,
    unit: UnitDefinitionId,
    abilities: Box<[AbilityId]>,
}

impl EnemyDefinition {
    /// Creates an unvalidated enemy definition.
    #[must_use]
    pub fn new(id: EnemyDefinitionId, unit: UnitDefinitionId, abilities: Vec<AbilityId>) -> Self {
        Self {
            id,
            unit,
            abilities: abilities.into_boxed_slice(),
        }
    }
    /// Returns the stable definition ID.
    #[must_use]
    pub const fn id(&self) -> EnemyDefinitionId {
        self.id
    }
    /// Returns the generic unit-form reference.
    #[must_use]
    pub const fn unit(&self) -> UnitDefinitionId {
        self.unit
    }
    /// Returns the canonical ability-reference set.
    #[must_use]
    pub fn abilities(&self) -> &[AbilityId] {
        &self.abilities
    }
}

/// Encounter definition referencing enemy definitions and encounter rule bundles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterDefinition {
    id: EncounterId,
    enemies: Box<[EnemyDefinitionId]>,
    rule_bundles: Box<[RuleBundleId]>,
}

impl EncounterDefinition {
    /// Creates an unvalidated encounter definition.
    #[must_use]
    pub fn new(
        id: EncounterId,
        enemies: Vec<EnemyDefinitionId>,
        rule_bundles: Vec<RuleBundleId>,
    ) -> Self {
        Self {
            id,
            enemies: enemies.into_boxed_slice(),
            rule_bundles: rule_bundles.into_boxed_slice(),
        }
    }
    /// Returns the stable definition ID.
    #[must_use]
    pub const fn id(&self) -> EncounterId {
        self.id
    }
    /// Returns enemies in authored encounter order.
    #[must_use]
    pub fn enemies(&self) -> &[EnemyDefinitionId] {
        &self.enemies
    }
    /// Returns the canonical encounter-rule set.
    #[must_use]
    pub fn rule_bundles(&self) -> &[RuleBundleId] {
        &self.rule_bundles
    }
}
