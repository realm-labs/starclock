//! Executable Standard Universe combat slices lowered from validated contributions.

use starclock_combat::{
    AbilityId, ProgramId, Ratio, SelectorId, SourceDefinitionId, TriggerId,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityTag, ActionHitDefinition,
            ActionResourcePolicy, HitOperationDefinition, ScalingDamageDefinition,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, TeamResourceCost,
            UnitTargetSelector,
        },
        definition::{
            AbilityDefinition, ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    formula::model::{CombatElement, DamageClass},
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleEventKind, RuleEventPoint, RuleOperationTemplate, RuleValue, TriggerDef, TriggerPhase,
        ValueExpr,
    },
};

use crate::{
    battle_contribution::{UniverseBattleRuleBinding, UniverseBattleRuleRole},
    blessing_runtime::BlessingContributionSet,
    catalog::UniverseCatalog,
    curio_runtime::CurioContributionSet,
    path::ExactParameter,
};

const PROGRAM_ID_BASE: u32 = 0x7600_0000;
const BODY_PROGRAM_ID_BASE: u32 = 0x7601_0000;
const OWNER_SELECTOR_ID_BASE: u32 = 0x7610_0000;
const TARGET_SELECTOR_ID_BASE: u32 = 0x7611_0000;
const ALL_TARGET_SELECTOR_ID_BASE: u32 = 0x7612_0000;
const CURRENT_TARGET_SELECTOR_ID_BASE: u32 = 0x7613_0000;
const TRIGGER_ID_BASE: u32 = 0x7620_0000;

pub(crate) const RESONANCE_ABILITY_ID: AbilityId =
    AbilityId::new(0x7630_0001).expect("reserved ability ID is non-zero");
pub(crate) const RESONANCE_PROGRAM_ID: ProgramId =
    ProgramId::new(0x7630_0002).expect("reserved program ID is non-zero");
pub(crate) const RESONANCE_SELECTOR_ID: SelectorId =
    SelectorId::new(0x7630_0003).expect("reserved selector ID is non-zero");
pub(crate) const RESONANCE_RESOURCE_ID: SourceDefinitionId =
    SourceDefinitionId::new(0x7630_0004).expect("reserved resource ID is non-zero");
pub(crate) const RESONANCE_RESOURCE_KEY: &str = "standard-universe.path-resonance-energy";

const ABUNDANCE_ADDITIONAL_DAMAGE_BINDING: &str = "StageAbility_612344";
const ENTRY_ENEMY_DAMAGE_BINDING: &str = "8";
const HUNT_RESONANCE_BINDING: &str = "StageAbility_612420";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RuleAttachment {
    EveryPlayer,
    FirstPlayer,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExecutableBattleRule {
    attachment: RuleAttachment,
    selectors: Box<[SelectorDefinition]>,
    programs: Box<[ProgramDefinition]>,
    definition: RuleDefinition,
    bundle: RuleBundle,
}

impl ExecutableBattleRule {
    pub(crate) const fn attachment(&self) -> RuleAttachment {
        self.attachment
    }
    pub(crate) fn selectors(&self) -> &[SelectorDefinition] {
        &self.selectors
    }
    pub(crate) fn programs(&self) -> &[ProgramDefinition] {
        &self.programs
    }
    pub(crate) const fn definition(&self) -> &RuleDefinition {
        &self.definition
    }
    pub(crate) const fn bundle(&self) -> &RuleBundle {
        &self.bundle
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExecutableResonance {
    selector: SelectorDefinition,
    program: ProgramDefinition,
    ability: AbilityDefinition,
    initial_energy: u16,
    maximum_energy: u16,
}

impl ExecutableResonance {
    pub(crate) const fn selector(&self) -> &SelectorDefinition {
        &self.selector
    }
    pub(crate) const fn program(&self) -> &ProgramDefinition {
        &self.program
    }
    pub(crate) const fn ability(&self) -> &AbilityDefinition {
        &self.ability
    }
    pub(crate) const fn initial_energy(&self) -> u16 {
        self.initial_energy
    }
    pub(crate) const fn maximum_energy(&self) -> u16 {
        self.maximum_energy
    }
}

pub(crate) fn lower_rules(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
    curios: &CurioContributionSet,
    initial_resonance_energy: u16,
) -> Result<(Vec<ExecutableBattleRule>, Option<ExecutableResonance>), BattleRuleLoweringError> {
    let mut output = Vec::new();
    if let Some(binding) = bindings.iter().find(|binding| {
        binding.role() == UniverseBattleRuleRole::BlessingLevel
            && binding.source_binding_key() == Some(ABUNDANCE_ADDITIONAL_DAMAGE_BINDING)
    }) {
        let contribution = blessings
            .entries()
            .iter()
            .find(|entry| entry.level().source_binding_key() == ABUNDANCE_ADDITIONAL_DAMAGE_BINDING)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        if contribution.level().level() == 2 {
            let ratio = parameter(contribution.level().parameters(), 0)?;
            output.push(abundance_additional_damage(binding, ratio)?);
        }
    }
    if let Some(binding) = bindings.iter().find(|binding| {
        binding.role() == UniverseBattleRuleRole::CurioState
            && binding.source_binding_key() == Some(ENTRY_ENEMY_DAMAGE_BINDING)
    }) {
        let contribution = curios
            .entries()
            .iter()
            .find(|entry| entry.state().source_effect_id() == ENTRY_ENEMY_DAMAGE_BINDING)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        let ratio = parameter(contribution.state().parameters(), 0)?;
        output.push(entry_enemy_damage(binding, ratio)?);
    }
    output.sort_unstable_by_key(|rule| rule.bundle().id());

    let resonance = bindings
        .iter()
        .find(|binding| {
            binding.role() == UniverseBattleRuleRole::Resonance
                && binding.source_binding_key() == Some(HUNT_RESONANCE_BINDING)
        })
        .map(|binding| hunt_resonance(catalog, binding, initial_resonance_energy))
        .transpose()?;
    Ok((output, resonance))
}

fn abundance_additional_damage(
    binding: &UniverseBattleRuleBinding,
    ratio: i64,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let trigger = id::<TriggerId>(TRIGGER_ID_BASE, raw)?;
    let selectors = vec![
        SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
        SelectorDefinition::new(target).with_rule_units(primary_target_selector()?),
    ];
    let amount = ValueExpr::Multiply {
        lhs: Box::new(ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        }),
        rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(
            starclock_combat::Scalar::from_scaled(ratio),
        ))),
        rounding: starclock_combat::Rounding::NearestTiesEven,
    };
    let program_definition = ProgramDefinition::new(
        program,
        Vec::new(),
        vec![owner, target],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::Damage {
            selector: target,
            amount,
            class: DamageClass::Additional,
            element: CombatElement::Physical,
            can_crit: false,
        },
    )]);
    let definition = RuleDefinition::new(binding.rule(), vec![program], vec![owner, target])
        .with_runtime(BattleRuleDefinition::new(
            binding.source().clone(),
            Vec::new(),
            vec![TriggerDef {
                id: trigger,
                event: RuleEventKind::Damage,
                event_point: RuleEventPoint::DamageApplied,
                phase: TriggerPhase::AfterEvent,
                filter: EventFilter {
                    actor_selector: Some(owner),
                    ability_tag: Some(AbilityTag::Attack),
                    ..EventFilter::default()
                },
                condition: ConditionExpr::Literal(true),
                once_scope: OnceScope::Action,
                priority: ReactionPriority::new(0),
                program,
            }],
            None,
        ));
    Ok(ExecutableBattleRule {
        attachment: RuleAttachment::EveryPlayer,
        selectors: selectors.into_boxed_slice(),
        programs: vec![program_definition].into_boxed_slice(),
        definition,
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}

fn entry_enemy_damage(
    binding: &UniverseBattleRuleBinding,
    ratio: i64,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let all_targets = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let current_target = id::<SelectorId>(CURRENT_TARGET_SELECTOR_ID_BASE, raw)?;
    let trigger = id::<TriggerId>(TRIGGER_ID_BASE, raw)?;
    let selectors = vec![
        SelectorDefinition::new(all_targets).with_rule_units(all_enemy_selector()?),
        SelectorDefinition::new(current_target).with_rule_units(current_subject_selector()?),
    ];
    let root_definition =
        ProgramDefinition::new(root, Vec::new(), vec![all_targets], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::ForEach {
                selector: all_targets,
                body,
                maximum: 16,
            }]);
    let amount = ValueExpr::Multiply {
        lhs: Box::new(ValueExpr::QueryStat {
            subject: StatQuerySubject::CurrentTarget,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        }),
        rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(
            starclock_combat::Scalar::from_scaled(ratio),
        ))),
        rounding: starclock_combat::Rounding::NearestTiesEven,
    };
    let body_definition = ProgramDefinition::new(
        body,
        Vec::new(),
        vec![current_target],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::TrueDamage {
            selector: current_target,
            amount,
        },
    )]);
    let definition = RuleDefinition::new(
        binding.rule(),
        vec![root, body],
        vec![all_targets, current_target],
    )
    .with_runtime(BattleRuleDefinition::new(
        binding.source().clone(),
        Vec::new(),
        vec![TriggerDef {
            id: trigger,
            event: RuleEventKind::Battle,
            event_point: RuleEventPoint::BattleStarted,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter::default(),
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Battle,
            priority: ReactionPriority::new(-100),
            program: root,
        }],
        None,
    ));
    Ok(ExecutableBattleRule {
        attachment: RuleAttachment::FirstPlayer,
        selectors: selectors.into_boxed_slice(),
        programs: vec![root_definition, body_definition].into_boxed_slice(),
        definition,
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}

fn hunt_resonance(
    catalog: &UniverseCatalog,
    binding: &UniverseBattleRuleBinding,
    initial_energy: u16,
) -> Result<ExecutableResonance, BattleRuleLoweringError> {
    let resonance = catalog
        .resonances()
        .iter()
        .find(|definition| definition.stable_key() == binding.source_record_key())
        .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
    let ratio = parameter(resonance.parameters(), 1)?;
    let action = AbilityActionDefinition::new(
        AbilityKind::Ultimate,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(
            0,
            0,
            starclock_combat::Energy::ZERO,
            starclock_combat::Energy::ZERO,
        )
        .with_team_resource_costs(vec![
            TeamResourceCost::new(RESONANCE_RESOURCE_KEY, 100)
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
        ])
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?
    .with_tags(&[AbilityTag::Attack, AbilityTag::Ultimate, AbilityTag::Assist])
    .with_hits(vec![ActionHitDefinition::new(vec![
        HitOperationDefinition::ScalingDamage(
            ScalingDamageDefinition::new(
                StatKind::Atk,
                Ratio::from_scaled(ratio),
                DamageClass::Additional,
                CombatElement::Wind,
            )
            .map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
        ),
    ])])
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let selector = SelectorDefinition::new(RESONANCE_SELECTOR_ID).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::All)
            .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
    );
    let program = ProgramDefinition::new(
        RESONANCE_PROGRAM_ID,
        Vec::new(),
        vec![RESONANCE_SELECTOR_ID],
        Vec::new(),
        Vec::new(),
    );
    let ability = AbilityDefinition::new(
        RESONANCE_ABILITY_ID,
        RESONANCE_PROGRAM_ID,
        RESONANCE_SELECTOR_ID,
        Vec::new(),
    )
    .with_action(action);
    Ok(ExecutableResonance {
        selector,
        program,
        ability,
        initial_energy: initial_energy.min(100),
        maximum_energy: 100,
    })
}

fn owner_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleSelectorChoice::First,
        1,
    )
}

fn primary_target_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Opposing,
        RuleSelectorChoice::First,
        1,
    )
}

fn all_enemy_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Encounter,
        RuleSelectorSide::Opposing,
        RuleSelectorChoice::All,
        16,
    )
}

fn current_subject_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::CurrentSubject,
        RuleSelectorSide::Opposing,
        RuleSelectorChoice::First,
        1,
    )
}

fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    choice: RuleSelectorChoice,
    maximum: u16,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        origin,
        side,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        1,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        choice,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn parameter(parameters: &[ExactParameter], index: usize) -> Result<i64, BattleRuleLoweringError> {
    let parameter = parameters
        .get(index)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let exponent = 6_u8
        .checked_sub(parameter.scale())
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    parameter
        .coefficient()
        .checked_mul(10_i64.pow(u32::from(exponent)))
        .ok_or(BattleRuleLoweringError::InvalidParameter)
}

fn id<T>(base: u32, raw: u32) -> Result<T, BattleRuleLoweringError>
where
    T: TryFrom<u32>,
{
    base.checked_add(raw)
        .and_then(|value| T::try_from(value).ok())
        .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleRuleLoweringError {
    SnapshotMismatch,
    InvalidParameter,
    InvalidDefinition,
}
