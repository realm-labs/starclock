//! Immutable battle-domain definition inputs accepted by the catalog builder.

use super::action::{AbilityActionDefinition, UnitTargetSelector};
use crate::rule::model::{BattleRuleDefinition, ProgramStep};
use crate::{
    AbilityId, EffectDefinitionId, EncounterId, EnemyDefinitionId, ModifierDefinitionId, ProgramId,
    RuleBundleId, RuleId, SelectorId, UnitDefinitionId,
};

/// Deterministic selector definition with an optional executable unit-target plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectorDefinition {
    id: SelectorId,
    unit_targets: Option<UnitTargetSelector>,
}

impl SelectorDefinition {
    /// Creates an identity-only selector that cannot execute until configured.
    #[must_use]
    pub const fn new(id: SelectorId) -> Self {
        Self {
            id,
            unit_targets: None,
        }
    }
    /// Attaches deterministic unit-target semantics.
    #[must_use]
    pub const fn with_unit_targets(mut self, selector: UnitTargetSelector) -> Self {
        self.unit_targets = Some(selector);
        self
    }
    /// Returns the stable selector definition ID.
    #[must_use]
    pub const fn id(&self) -> SelectorId {
        self.id
    }
    /// Returns executable unit-target semantics, if configured.
    #[must_use]
    pub const fn unit_targets(&self) -> Option<UnitTargetSelector> {
        self.unit_targets
    }
}
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
    action: Option<AbilityActionDefinition>,
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
            action: None,
        }
    }
    /// Attaches a validated finite action definition.
    #[must_use]
    pub fn with_action(mut self, action: AbilityActionDefinition) -> Self {
        self.action = Some(action);
        self
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
    /// Returns the executable action definition, if this ability owns one.
    #[must_use]
    pub const fn action(&self) -> Option<&AbilityActionDefinition> {
        self.action.as_ref()
    }
}

/// Effect definition referencing attached rules and modifiers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectDefinition {
    id: EffectDefinitionId,
    rules: Box<[RuleId]>,
    modifiers: Box<[ModifierDefinitionId]>,
    runtime: Option<crate::effect::model::EffectRuntimeDefinition>,
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
            runtime: None,
        }
    }
    /// Attaches the validated generic runtime behavior.
    #[must_use]
    pub fn with_runtime(mut self, runtime: crate::effect::model::EffectRuntimeDefinition) -> Self {
        self.runtime = Some(runtime);
        self
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
    /// Returns executable effect semantics when authored.
    #[must_use]
    pub const fn runtime(&self) -> Option<&crate::effect::model::EffectRuntimeDefinition> {
        self.runtime.as_ref()
    }
}

/// Rule definition referencing typed programs and selectors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleDefinition {
    id: RuleId,
    programs: Box<[ProgramId]>,
    selectors: Box<[SelectorId]>,
    runtime: Option<BattleRuleDefinition>,
}

impl RuleDefinition {
    /// Creates an unvalidated battle-rule definition.
    #[must_use]
    pub fn new(id: RuleId, programs: Vec<ProgramId>, selectors: Vec<SelectorId>) -> Self {
        Self {
            id,
            programs: programs.into_boxed_slice(),
            selectors: selectors.into_boxed_slice(),
            runtime: None,
        }
    }
    /// Attaches the executable typed battle-rule definition.
    #[must_use]
    pub fn with_runtime(mut self, runtime: BattleRuleDefinition) -> Self {
        self.runtime = Some(runtime);
        self
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
    /// Returns executable rule semantics when this is not an identity-only fixture.
    #[must_use]
    pub const fn runtime(&self) -> Option<&BattleRuleDefinition> {
        self.runtime.as_ref()
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
    steps: Box<[ProgramStep]>,
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
            steps: Box::new([]),
        }
    }
    /// Attaches a finite ordered typed program body.
    #[must_use]
    pub fn with_steps(mut self, steps: Vec<ProgramStep>) -> Self {
        self.called_programs = steps
            .iter()
            .flat_map(|step| match step {
                ProgramStep::Operation(_) => [None, None],
                ProgramStep::If {
                    then_program,
                    else_program,
                    ..
                } => [Some(*then_program), *else_program],
                ProgramStep::ForEach { body, .. } => [Some(*body), None],
            })
            .flatten()
            .collect::<Vec<_>>()
            .into_boxed_slice();
        self.steps = steps.into_boxed_slice();
        self
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
    /// Returns the finite ordered typed program body.
    #[must_use]
    pub fn steps(&self) -> &[ProgramStep] {
        &self.steps
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
    waves: Box<[Box<[EnemyDefinitionId]>]>,
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
            waves: vec![enemies.clone().into_boxed_slice()].into_boxed_slice(),
            enemies: enemies.into_boxed_slice(),
            rule_bundles: rule_bundles.into_boxed_slice(),
        }
    }
    /// Replaces the default one-wave layout with ordered non-empty waves.
    #[must_use]
    pub fn with_waves(mut self, waves: Vec<Vec<EnemyDefinitionId>>) -> Option<Self> {
        if waves.is_empty()
            || waves.len() > usize::from(u16::MAX)
            || waves.iter().any(Vec::is_empty)
        {
            return None;
        }
        let mut enemies = waves.iter().flatten().copied().collect::<Vec<_>>();
        enemies.sort_unstable();
        enemies.dedup();
        self.enemies = enemies.into_boxed_slice();
        self.waves = waves
            .into_iter()
            .map(Vec::into_boxed_slice)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Some(self)
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
    /// Returns ordered waves; each wave preserves authored enemy occurrence order.
    #[must_use]
    pub fn waves(&self) -> &[Box<[EnemyDefinitionId]>] {
        &self.waves
    }
    /// Returns exact enemy occurrences for one-based wave number.
    #[must_use]
    pub fn wave_enemies(&self, number: u16) -> Option<&[EnemyDefinitionId]> {
        self.waves
            .get(usize::from(number.checked_sub(1)?))
            .map(AsRef::as_ref)
    }
    /// Returns the canonical encounter-rule set.
    #[must_use]
    pub fn rule_bundles(&self) -> &[RuleBundleId] {
        &self.rule_bundles
    }
}
