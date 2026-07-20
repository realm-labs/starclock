//! Immutable battle-domain definition inputs accepted by the catalog builder.

use super::action::{AbilityActionDefinition, AbilityProgramBinding, UnitTargetSelector};
use super::encounter::{
    EncounterWaveDefinition, EnemyLinkDefinition, EnemyPhaseDefinition, WaveCarry,
    WaveSlotDefinition, WaveTransitionPolicy,
};
use super::selector::RuleUnitSelector;
use crate::rule::model::{BattleRuleDefinition, ProgramStep};
use crate::{
    AbilityId, AiGraphId, EffectDefinitionId, EncounterId, EnemyDefinitionId, ModifierDefinitionId,
    ProgramId, RuleBundleId, RuleId, SelectorId, UnitDefinitionId,
};

/// Deterministic selector definition with an optional executable unit-target plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectorDefinition {
    id: SelectorId,
    unit_targets: Option<UnitTargetSelector>,
    rule_units: Option<RuleUnitSelector>,
}

impl SelectorDefinition {
    /// Creates an identity-only selector that cannot execute until configured.
    #[must_use]
    pub const fn new(id: SelectorId) -> Self {
        Self {
            id,
            unit_targets: None,
            rule_units: None,
        }
    }
    /// Attaches deterministic unit-target semantics.
    #[must_use]
    pub const fn with_unit_targets(mut self, selector: UnitTargetSelector) -> Self {
        self.unit_targets = Some(selector);
        self
    }
    /// Attaches the complete typed Rule IR unit-selector plan.
    #[must_use]
    pub fn with_rule_units(mut self, selector: RuleUnitSelector) -> Self {
        self.rule_units = Some(selector);
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
    /// Returns executable Rule IR unit-selector semantics, if configured.
    #[must_use]
    pub const fn rule_units(&self) -> Option<&RuleUnitSelector> {
        self.rule_units.as_ref()
    }
}
/// Generic unit-form definition referencing combat abilities and rule bundles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnitDefinition {
    id: UnitDefinitionId,
    abilities: Box<[AbilityId]>,
    rule_bundles: Box<[RuleBundleId]>,
    resources: Box<[CharacterResourceDefinition]>,
}

/// One form-scoped named character resource with checked scalar bounds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharacterResourceDefinition {
    stable_key: Box<str>,
    initial: crate::Scalar,
    maximum: crate::Scalar,
}

impl CharacterResourceDefinition {
    /// Creates a non-negative bounded named resource.
    #[must_use]
    pub fn new(
        stable_key: impl Into<Box<str>>,
        initial: crate::Scalar,
        maximum: crate::Scalar,
    ) -> Option<Self> {
        let stable_key = stable_key.into();
        if stable_key.trim().is_empty()
            || initial.scaled() < 0
            || maximum.scaled() < 0
            || initial > maximum
        {
            return None;
        }
        Some(Self {
            stable_key,
            initial,
            maximum,
        })
    }
    /// Returns the exact authored semantic key.
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    /// Returns the initial value.
    #[must_use]
    pub const fn initial(&self) -> crate::Scalar {
        self.initial
    }
    /// Returns the inclusive upper bound.
    #[must_use]
    pub const fn maximum(&self) -> crate::Scalar {
        self.maximum
    }
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
            resources: Box::new([]),
        }
    }

    /// Attaches form-scoped resources in canonical stable-key order.
    #[must_use]
    pub fn with_resources(mut self, resources: Vec<CharacterResourceDefinition>) -> Self {
        self.resources = resources.into_boxed_slice();
        self
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
    /// Returns the canonical form-scoped resource definitions.
    #[must_use]
    pub fn resources(&self) -> &[CharacterResourceDefinition] {
        &self.resources
    }
}

/// One typed value selected for an exact effective-level ability definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityParameterDefinition {
    ability: AbilityId,
    stable_key: Box<str>,
    value: crate::rule::model::RuleValue,
}

impl AbilityParameterDefinition {
    #[must_use]
    pub fn new(
        ability: AbilityId,
        stable_key: impl Into<Box<str>>,
        value: crate::rule::model::RuleValue,
    ) -> Option<Self> {
        let stable_key = stable_key.into();
        if stable_key.trim().is_empty() || stable_key.len() > 128 {
            return None;
        }
        Some(Self {
            ability,
            stable_key,
            value,
        })
    }
    #[must_use]
    pub const fn ability(&self) -> AbilityId {
        self.ability
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn value(&self) -> &crate::rule::model::RuleValue {
        &self.value
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
    programs: Box<[AbilityProgramBinding]>,
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
            programs: Box::new([]),
        }
    }
    /// Attaches a validated finite action definition.
    #[must_use]
    pub fn with_action(mut self, action: AbilityActionDefinition) -> Self {
        self.action = Some(action);
        self
    }
    /// Attaches authored phase programs in strict sequence order.
    #[must_use]
    pub fn with_programs(mut self, programs: Vec<AbilityProgramBinding>) -> Self {
        self.programs = programs.into_boxed_slice();
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
    /// Returns authored phase programs in execution order.
    #[must_use]
    pub fn programs(&self) -> &[AbilityProgramBinding] {
        &self.programs
    }
}

/// Effect definition referencing attached rules and modifiers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectDefinition {
    id: EffectDefinitionId,
    rules: Box<[RuleId]>,
    modifiers: Box<[ModifierDefinitionId]>,
    runtime: Option<crate::effect::model::EffectRuntimeDefinition>,
    runtime_template: Option<crate::effect::model::EffectRuntimeTemplate>,
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
            runtime_template: None,
        }
    }
    /// Attaches the validated generic runtime behavior.
    #[must_use]
    pub fn with_runtime(mut self, runtime: crate::effect::model::EffectRuntimeDefinition) -> Self {
        self.runtime = Some(runtime);
        self
    }
    /// Attaches expression-backed runtime behavior resolved for each application target.
    #[must_use]
    pub fn with_runtime_template(
        mut self,
        runtime: crate::effect::model::EffectRuntimeTemplate,
    ) -> Self {
        self.runtime_template = Some(runtime);
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
    /// Returns expression-backed effect semantics when authored.
    #[must_use]
    pub const fn runtime_template(&self) -> Option<&crate::effect::model::EffectRuntimeTemplate> {
        self.runtime_template.as_ref()
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
    ai_graph: Option<AiGraphId>,
    phases: Box<[EnemyPhaseDefinition]>,
    links: Box<[EnemyLinkDefinition]>,
}

impl EnemyDefinition {
    /// Creates an unvalidated enemy definition.
    #[must_use]
    pub fn new(id: EnemyDefinitionId, unit: UnitDefinitionId, abilities: Vec<AbilityId>) -> Self {
        Self {
            id,
            unit,
            abilities: abilities.into_boxed_slice(),
            ai_graph: None,
            phases: Box::new([]),
            links: Box::new([]),
        }
    }
    /// Adds canonical definition-level summon, part, and countdown links.
    #[must_use]
    pub fn with_links(mut self, mut links: Vec<EnemyLinkDefinition>) -> Option<Self> {
        links.sort_by_key(|link| link.sequence());
        if links
            .iter()
            .enumerate()
            .any(|(index, link)| usize::from(link.sequence()) != index + 1)
        {
            return None;
        }
        self.links = links.into_boxed_slice();
        Some(self)
    }
    /// Binds the default AI graph and ordered boss-phase definitions.
    #[must_use]
    pub fn with_orchestration(
        mut self,
        ai_graph: AiGraphId,
        mut phases: Vec<EnemyPhaseDefinition>,
    ) -> Option<Self> {
        phases.sort_by_key(EnemyPhaseDefinition::sequence);
        if phases
            .iter()
            .enumerate()
            .any(|(index, phase)| usize::from(phase.sequence()) != index + 1)
            || phases.iter().enumerate().any(|(index, phase)| {
                phases[..index]
                    .iter()
                    .any(|earlier| earlier.id() == phase.id())
            })
        {
            return None;
        }
        self.ai_graph = Some(ai_graph);
        self.phases = phases.into_boxed_slice();
        Some(self)
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
    /// Returns the default authored AI graph, when this definition is executable by AI.
    #[must_use]
    pub const fn ai_graph(&self) -> Option<AiGraphId> {
        self.ai_graph
    }
    /// Returns ordered boss phases. Ordinary enemies have an empty slice.
    #[must_use]
    pub fn phases(&self) -> &[EnemyPhaseDefinition] {
        &self.phases
    }
    /// Returns ordered definition-level links.
    #[must_use]
    pub fn links(&self) -> &[EnemyLinkDefinition] {
        &self.links
    }
}

/// Encounter definition referencing enemy definitions and encounter rule bundles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterDefinition {
    id: EncounterId,
    enemies: Box<[EnemyDefinitionId]>,
    waves: Box<[EncounterWaveDefinition]>,
    rule_bundles: Box<[RuleBundleId]>,
    wave_transition: WaveTransitionPolicy,
}

impl EncounterDefinition {
    /// Creates an unvalidated encounter definition.
    #[must_use]
    pub fn new(
        id: EncounterId,
        enemies: Vec<EnemyDefinitionId>,
        rule_bundles: Vec<RuleBundleId>,
    ) -> Self {
        let wave = (1..=32).contains(&enemies.len()).then(|| {
            let slots = enemies
                .iter()
                .copied()
                .enumerate()
                .map(|(index, enemy)| {
                    WaveSlotDefinition::legacy(
                        u16::try_from(index + 1).expect("32 slots fit u16"),
                        enemy,
                    )
                })
                .collect();
            EncounterWaveDefinition::new(
                crate::EncounterWaveId::new(id.get()).expect("encounter ID is non-zero"),
                1,
                None,
                None,
                WaveCarry::CARRY_ALL,
                slots,
            )
            .expect("bounded non-empty default wave is valid")
        });
        Self {
            id,
            waves: wave.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            enemies: enemies.into_boxed_slice(),
            rule_bundles: rule_bundles.into_boxed_slice(),
            wave_transition: WaveTransitionPolicy::AfterAction,
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
            .enumerate()
            .map(|(wave_index, enemies)| {
                let slots = enemies
                    .into_iter()
                    .enumerate()
                    .map(|(slot_index, enemy)| {
                        Some(WaveSlotDefinition::legacy(
                            u16::try_from(slot_index + 1).ok()?,
                            enemy,
                        ))
                    })
                    .collect::<Option<Vec<_>>>()?;
                EncounterWaveDefinition::new(
                    crate::EncounterWaveId::new(u32::try_from(wave_index + 1).ok()?)?,
                    u16::try_from(wave_index + 1).ok()?,
                    None,
                    None,
                    WaveCarry::CARRY_ALL,
                    slots,
                )
            })
            .collect::<Option<Vec<_>>>()?
            .into_boxed_slice();
        Some(self)
    }
    /// Replaces the wave layout with fully authored slot and carry definitions.
    #[must_use]
    pub fn with_authored_waves(
        mut self,
        wave_transition: WaveTransitionPolicy,
        mut waves: Vec<EncounterWaveDefinition>,
    ) -> Option<Self> {
        waves.sort_by_key(EncounterWaveDefinition::sequence);
        if waves.is_empty()
            || waves
                .iter()
                .enumerate()
                .any(|(index, wave)| usize::from(wave.sequence()) != index + 1)
        {
            return None;
        }
        let mut enemies = waves
            .iter()
            .flat_map(|wave| wave.slots().iter().map(|slot| slot.enemy()))
            .collect::<Vec<_>>();
        enemies.sort_unstable();
        enemies.dedup();
        self.enemies = enemies.into_boxed_slice();
        self.waves = waves.into_boxed_slice();
        self.wave_transition = wave_transition;
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
    pub fn waves(&self) -> &[EncounterWaveDefinition] {
        &self.waves
    }
    /// Returns one authored one-based wave.
    #[must_use]
    pub fn wave(&self, number: u16) -> Option<&EncounterWaveDefinition> {
        self.waves.get(usize::from(number.checked_sub(1)?))
    }
    /// Returns the boundary at which pending wave replacement is allowed.
    #[must_use]
    pub const fn wave_transition(&self) -> WaveTransitionPolicy {
        self.wave_transition
    }
    /// Returns the canonical encounter-rule set.
    #[must_use]
    pub fn rule_bundles(&self) -> &[RuleBundleId] {
        &self.rule_bundles
    }
}
