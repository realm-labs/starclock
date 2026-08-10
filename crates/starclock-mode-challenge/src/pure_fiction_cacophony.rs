//! Executable selected Pure Fiction Cacophony contributions.

use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectDefinitionId, EffectRuntimeTemplate,
    EffectStackPolicy, EffectTickPhase, ModifierDefinitionId, ModifierStackingGroupId, ProgramId,
    Rounding, RuleBundleId, RuleId, Scalar, SelectorId, SourceDefinitionId, StateSlotDefinitionId,
    TriggerId,
    catalog::{
        action::AbilityTag,
        definition::{
            EffectDefinition, ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition, ModifierFilter,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{
        BattleRuleDefinition, Comparison, ConditionExpr, EventFilter, EventValueProperty,
        OnceScope, ProgramStep, ReactionPriority, ResourceUpdateKind, RuleEffectChancePolicy,
        RuleEventPoint, RuleOperationTemplate, RuleResourceKind, RuleSource, RuleValue,
        RuleValueKind, SourceClass, TriggerDef, TriggerPhase, ValueExpr,
    },
};

use crate::pure_fiction_mechanics::{
    CACOPHONY_GRIT_ONE_SIGNAL, CACOPHONY_GRIT_THREE_SIGNAL, CACOPHONY_GRIT_TWO_SIGNAL,
    PURE_FICTION_SURGING_STARTED_SIGNAL,
};

pub const TOCCATA_BUNDLE: RuleBundleId = id_bundle(3_031_359);
pub const VARIATION_BUNDLE: RuleBundleId = id_bundle(3_031_361);
pub const MIRTHFUL_CADENCE_BUNDLE: RuleBundleId = id_bundle(3_031_362);
pub const CACOPHONY_SOURCE: SourceDefinitionId = id_source(0x7f20_0001);
pub const TOCCATA_ULTIMATE_BOOST: ModifierDefinitionId = id_modifier(0x7f20_0002);
pub const TOCCATA_FOLLOW_UP_BOOST: ModifierDefinitionId = id_modifier(0x7f20_0003);
pub(crate) const TOCCATA_HOST_EFFECT: EffectDefinitionId = id_effect(0x7f20_001f);

const DAMAGE_GROUP: ModifierStackingGroupId = id_group(0x7f20_0004);
const ALL_ALLIES: SelectorId = id_selector(0x7f20_0005);
const ACTION_TARGETS: SelectorId = id_selector(0x7f20_0006);
const TOCCATA_ROOT: ProgramId = id_program(0x7f20_0007);
const TOCCATA_BODY: ProgramId = id_program(0x7f20_0008);
const VARIATION_SIGNAL: ProgramId = id_program(0x7f20_0009);
const MIRTHFUL_ROOT: ProgramId = id_program(0x7f20_000a);
const MIRTHFUL_BODY: ProgramId = id_program(0x7f20_000b);
const TOCCATA_RULE: RuleId = id_rule(0x7f20_000c);
const VARIATION_RULE: RuleId = id_rule(0x7f20_000d);
const MIRTHFUL_RULE: RuleId = id_rule(0x7f20_000e);
const VARIATION_EFFECT: EffectDefinitionId = id_effect(0x7f20_0015);
const VARIATION_MODIFIER: ModifierDefinitionId = id_modifier(0x7f20_0016);
const VARIATION_GROUP: ModifierStackingGroupId = id_group(0x7f20_0017);
const VARIATION_STACKS: StateSlotDefinitionId = id_slot(0x7f20_0018);
const MIRTHFUL_EFFECT: EffectDefinitionId = id_effect(0x7f20_0019);
const MIRTHFUL_GROUP: ModifierStackingGroupId = id_group(0x7f20_001b);
const MIRTHFUL_EFFECT_PROGRAM: ProgramId = id_program(0x7f20_001d);
const HOST: SelectorId = id_selector(0x7f20_0020);
const TOCCATA_START: ProgramId = id_program(0x7f20_0021);
const MIRTHFUL_SURGING: ProgramId = id_program(0x7f20_0022);
const TOCCATA_START_TRIGGER: TriggerId = id_trigger(0x7f20_0023);
const MIRTHFUL_SURGING_TRIGGER: TriggerId = id_trigger(0x7f20_0024);
const MIRTHFUL_MODIFIER_BASE: u32 = 0x7f20_0025;
const PUNCHLINE_RESOURCE: &str = "shared.punchline";

pub struct PureFictionCacophonyDefinitions {
    pub modifier_groups: Vec<ModifierStackingGroup>,
    pub modifiers: Vec<ModifierDefinition>,
    pub effects: Vec<EffectDefinition>,
    pub selectors: Vec<SelectorDefinition>,
    pub programs: Vec<ProgramDefinition>,
    pub rules: [RuleDefinition; 3],
    pub bundles: [RuleBundle; 3],
    pub source: RuleSource,
}

impl PureFictionCacophonyDefinitions {
    #[must_use]
    pub fn active() -> Self {
        let source = RuleSource::new(
            CACOPHONY_SOURCE,
            SourceClass::Mode,
            Vec::new(),
            [
                0xa5, 0x62, 0xb5, 0xeb, 0xc4, 0x4b, 0x7a, 0xe9, 0x3d, 0x4e, 0x83, 0x1f, 0x7b, 0xcd,
                0x51, 0xe4, 0x61, 0xd5, 0xd8, 0x06, 0xe8, 0x76, 0xe5, 0xeb, 0x22, 0xd3, 0xaf, 0xea,
                0x13, 0x95, 0xaf, 0x38,
            ],
        );
        let selectors = [
            SelectorDefinition::new(ALL_ALLIES).with_rule_units(team_selector()),
            SelectorDefinition::new(ACTION_TARGETS).with_rule_units(event_targets()),
            SelectorDefinition::new(HOST).with_rule_units(host_selector()),
        ];
        let programs = [
            foreach_program(TOCCATA_ROOT, TOCCATA_BODY),
            signal_program(TOCCATA_BODY, CACOPHONY_GRIT_ONE_SIGNAL),
            ProgramDefinition::new(
                VARIATION_SIGNAL,
                Vec::new(),
                vec![ACTION_TARGETS],
                vec![VARIATION_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![
                apply_effect(VARIATION_EFFECT),
                emit_signal(CACOPHONY_GRIT_TWO_SIGNAL),
            ]),
            foreach_program(MIRTHFUL_ROOT, MIRTHFUL_BODY),
            signal_program(MIRTHFUL_BODY, CACOPHONY_GRIT_THREE_SIGNAL),
            ProgramDefinition::new(
                MIRTHFUL_EFFECT_PROGRAM,
                Vec::new(),
                vec![ACTION_TARGETS],
                vec![MIRTHFUL_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![apply_effect(MIRTHFUL_EFFECT)]),
            ProgramDefinition::new(
                TOCCATA_START,
                Vec::new(),
                vec![HOST],
                vec![TOCCATA_HOST_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![apply_effect_to(HOST, TOCCATA_HOST_EFFECT, integer(1))]),
            ProgramDefinition::new(
                MIRTHFUL_SURGING,
                Vec::new(),
                vec![HOST],
                Vec::new(),
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ModifyResource {
                    selector: HOST,
                    resource: RuleResourceKind::Team(PUNCHLINE_RESOURCE.into()),
                    update: ResourceUpdateKind::Gain,
                    amount: ValueExpr::Literal(RuleValue::Scalar(
                        Scalar::checked_from_integer(40).expect("40 Punchline is representable"),
                    )),
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                },
            )]),
        ];
        let rules = [
            rule(
                TOCCATA_RULE,
                &source,
                vec![TOCCATA_ROOT, TOCCATA_BODY, TOCCATA_START],
                vec![
                    action_trigger(0x7f20_0010, AbilityTag::Ultimate, TOCCATA_ROOT),
                    action_trigger(0x7f20_0011, AbilityTag::FollowUp, TOCCATA_ROOT),
                    battle_start_trigger(TOCCATA_START_TRIGGER, TOCCATA_START),
                ],
            ),
            rule(
                VARIATION_RULE,
                &source,
                vec![VARIATION_SIGNAL],
                vec![
                    action_trigger(0x7f20_0012, AbilityTag::Basic, VARIATION_SIGNAL),
                    action_trigger(0x7f20_002c, AbilityTag::Skill, VARIATION_SIGNAL),
                    action_trigger(0x7f20_002d, AbilityTag::Ultimate, VARIATION_SIGNAL),
                ],
            ),
            rule(
                MIRTHFUL_RULE,
                &source,
                vec![
                    MIRTHFUL_ROOT,
                    MIRTHFUL_BODY,
                    MIRTHFUL_EFFECT_PROGRAM,
                    MIRTHFUL_SURGING,
                ],
                vec![
                    action_trigger(0x7f20_0013, AbilityTag::Basic, MIRTHFUL_ROOT),
                    action_trigger(0x7f20_0014, AbilityTag::Skill, MIRTHFUL_ROOT),
                    action_trigger(
                        0x7f20_001e,
                        AbilityTag::ElationSkill,
                        MIRTHFUL_EFFECT_PROGRAM,
                    ),
                    surging_started_trigger(),
                ],
            ),
        ];
        Self {
            modifier_groups: vec![
                sum_group(DAMAGE_GROUP),
                sum_group(VARIATION_GROUP),
                sum_group(MIRTHFUL_GROUP),
            ],
            modifiers: cacophony_modifiers(),
            effects: vec![variation_effect(), mirthful_effect(), host_marker_effect()],
            selectors: selectors.into(),
            programs: programs.into(),
            rules,
            bundles: [
                RuleBundle::new(TOCCATA_BUNDLE, vec![TOCCATA_RULE]),
                RuleBundle::new(VARIATION_BUNDLE, vec![VARIATION_RULE]),
                RuleBundle::new(MIRTHFUL_CADENCE_BUNDLE, vec![MIRTHFUL_RULE]),
            ],
            source,
        }
    }
}

fn rule(
    id: RuleId,
    source: &RuleSource,
    programs: Vec<ProgramId>,
    triggers: Vec<TriggerDef>,
) -> RuleDefinition {
    RuleDefinition::new(id, programs, vec![ALL_ALLIES, ACTION_TARGETS, HOST]).with_runtime(
        BattleRuleDefinition::new(source.clone(), Vec::new(), triggers, None),
    )
}

fn action_trigger(raw: u32, tag: AbilityTag, program: ProgramId) -> TriggerDef {
    TriggerDef {
        id: id_trigger(raw),
        event: RuleEventPoint::ActionResolved.kind(),
        event_point: RuleEventPoint::ActionResolved,
        phase: TriggerPhase::AfterAction,
        filter: EventFilter {
            actor_selector: Some(ALL_ALLIES),
            ability_tag: Some(tag),
            ..EventFilter::default()
        },
        condition: ConditionExpr::Literal(true),
        once_scope: OnceScope::Action,
        priority: ReactionPriority::new(0),
        program,
    }
}

fn battle_start_trigger(id: TriggerId, program: ProgramId) -> TriggerDef {
    TriggerDef {
        id,
        event: RuleEventPoint::BattleStarted.kind(),
        event_point: RuleEventPoint::BattleStarted,
        phase: TriggerPhase::AfterEvent,
        filter: EventFilter::default(),
        condition: ConditionExpr::Literal(true),
        once_scope: OnceScope::Battle,
        priority: ReactionPriority::new(0),
        program,
    }
}

fn surging_started_trigger() -> TriggerDef {
    TriggerDef {
        id: MIRTHFUL_SURGING_TRIGGER,
        event: RuleEventPoint::InformationalRule.kind(),
        event_point: RuleEventPoint::InformationalRule,
        phase: TriggerPhase::AfterEvent,
        filter: EventFilter::default(),
        condition: ConditionExpr::Compare {
            lhs: Box::new(ValueExpr::ReadEventProperty(
                EventValueProperty::RuleSignalCode,
            )),
            operator: Comparison::Equal,
            rhs: Box::new(integer(i64::from(PURE_FICTION_SURGING_STARTED_SIGNAL))),
        },
        once_scope: OnceScope::Event,
        priority: ReactionPriority::new(0),
        program: MIRTHFUL_SURGING,
    }
}

fn foreach_program(id: ProgramId, body: ProgramId) -> ProgramDefinition {
    ProgramDefinition::new(id, vec![body], vec![ACTION_TARGETS], Vec::new(), Vec::new()).with_steps(
        vec![ProgramStep::ForEach {
            selector: ACTION_TARGETS,
            body,
            maximum: 16,
        }],
    )
}

fn signal_program(id: ProgramId, code: u32) -> ProgramDefinition {
    ProgramDefinition::new(id, Vec::new(), Vec::new(), Vec::new(), Vec::new()).with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent { code, value: None }),
    ])
}

fn emit_signal(code: u32) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent { code, value: None })
}

fn apply_effect(effect: EffectDefinitionId) -> ProgramStep {
    apply_effect_to(ACTION_TARGETS, effect, integer(1))
}

fn apply_effect_to(
    selector: SelectorId,
    effect: EffectDefinitionId,
    stacks: ValueExpr,
) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector,
        effect,
        stacks,
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })
}

fn team_selector() -> RuleUnitSelector {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleSelectorOrdering::Formation,
        32,
    )
}

fn event_targets() -> RuleUnitSelector {
    selector(
        RuleSelectorOrigin::EventTargets,
        RuleSelectorSide::Opposing,
        RuleSelectorOrdering::EventOrder,
        16,
    )
}

fn host_selector() -> RuleUnitSelector {
    RuleUnitSelector::new(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StableId,
        0,
        1,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::First,
        None,
        false,
    )
    .expect("Cacophony host selector is valid")
}

fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    ordering: RuleSelectorOrdering,
    maximum: u16,
) -> RuleUnitSelector {
    RuleUnitSelector::new(
        origin,
        side,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::ActionSnapshot,
        ordering,
        0,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::All,
        None,
        false,
    )
    .expect("bounded Cacophony selector is valid")
}

fn damage_modifier(id: ModifierDefinitionId, tag: &str) -> ModifierDefinition {
    ModifierDefinition {
        id,
        stat: StatKind::Hp,
        stage: FormulaStage::DamageBoost,
        purpose: FormulaPurpose::OrdinaryDamage,
        value: starclock_combat::rule::model::ValueExpr::Literal(
            starclock_combat::rule::model::RuleValue::Scalar(Scalar::from_scaled(400_000)),
        ),
        stacking_group: DAMAGE_GROUP,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::DamageBoost,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: vec![ModifierFilter::AbilityTag(tag.into())].into_boxed_slice(),
    }
}

fn sum_group(id: ModifierStackingGroupId) -> ModifierStackingGroup {
    ModifierStackingGroup {
        id,
        aggregation: ModifierAggregation::Sum,
        comparator: None,
    }
}

fn cacophony_modifiers() -> Vec<ModifierDefinition> {
    let mut output = vec![
        damage_modifier(TOCCATA_ULTIMATE_BOOST, "ultimate"),
        damage_modifier(TOCCATA_FOLLOW_UP_BOOST, "follow_up"),
        stack_modifier(
            VARIATION_MODIFIER,
            VARIATION_GROUP,
            VARIATION_STACKS,
            StatKind::Def,
            FormulaStage::PercentOfBase,
            FormulaPurpose::Stat,
            -30_000,
        ),
    ];
    output.extend(
        damage_purposes()
            .into_iter()
            .enumerate()
            .map(|(index, purpose)| ModifierDefinition {
                id: id_modifier(
                    MIRTHFUL_MODIFIER_BASE + u32::try_from(index).expect("seven purposes fit u32"),
                ),
                stat: StatKind::Hp,
                stage: FormulaStage::Vulnerability,
                purpose,
                value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(150_000))),
                stacking_group: MIRTHFUL_GROUP,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::Vulnerability,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: Box::new([]),
            }),
    );
    output
}

fn mirthful_modifier_ids() -> Vec<ModifierDefinitionId> {
    (0..7)
        .map(|index| id_modifier(MIRTHFUL_MODIFIER_BASE + index))
        .collect()
}

fn damage_purposes() -> [FormulaPurpose; 7] {
    [
        FormulaPurpose::OrdinaryDamage,
        FormulaPurpose::Dot,
        FormulaPurpose::Break,
        FormulaPurpose::SuperBreak,
        FormulaPurpose::AdditionalDamage,
        FormulaPurpose::JointDamage,
        FormulaPurpose::ElationDamage,
    ]
}

#[allow(clippy::too_many_arguments)]
fn stack_modifier(
    id: ModifierDefinitionId,
    group: ModifierStackingGroupId,
    stack_slot: StateSlotDefinitionId,
    stat: StatKind,
    stage: FormulaStage,
    purpose: FormulaPurpose,
    per_stack: i64,
) -> ModifierDefinition {
    ModifierDefinition {
        id,
        stat,
        stage,
        purpose,
        value: ValueExpr::Multiply {
            lhs: Box::new(ValueExpr::Convert {
                value: Box::new(ValueExpr::Slot(stack_slot)),
                target: RuleValueKind::Scalar,
                rounding: Rounding::NearestTiesEven,
            }),
            rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                per_stack,
            )))),
            rounding: Rounding::NearestTiesEven,
        },
        stacking_group: group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: stage,
        snapshot: SnapshotPolicy::RecomputeOnStackChange,
        source_stack_slot: Some(stack_slot),
        filters: Box::new([]),
    }
}

fn variation_effect() -> EffectDefinition {
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Debuff,
        DispelCategory::DispellableDebuff,
        10,
        Some(ValueExpr::Literal(RuleValue::Integer(2))),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .expect("two-turn Cacophony debuff is valid");
    EffectDefinition::new(VARIATION_EFFECT, Vec::new(), vec![VARIATION_MODIFIER])
        .with_runtime_template(runtime)
}

fn mirthful_effect() -> EffectDefinition {
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Debuff,
        DispelCategory::DispellableDebuff,
        1,
        Some(integer(2)),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::IndependentInstances,
    )
    .expect("independent two-turn Indulgence is valid");
    EffectDefinition::new(MIRTHFUL_EFFECT, Vec::new(), mirthful_modifier_ids())
        .with_runtime_template(runtime)
}

fn host_marker_effect() -> EffectDefinition {
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::NeutralState,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .expect("permanent Toccata host marker is valid");
    EffectDefinition::new(TOCCATA_HOST_EFFECT, Vec::new(), Vec::new())
        .with_runtime_template(runtime)
}

fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
}

const fn id_bundle(raw: u32) -> RuleBundleId {
    RuleBundleId::new(raw).expect("nonzero ID")
}
const fn id_rule(raw: u32) -> RuleId {
    RuleId::new(raw).expect("nonzero ID")
}
const fn id_source(raw: u32) -> SourceDefinitionId {
    SourceDefinitionId::new(raw).expect("nonzero ID")
}
const fn id_modifier(raw: u32) -> ModifierDefinitionId {
    ModifierDefinitionId::new(raw).expect("nonzero ID")
}
const fn id_group(raw: u32) -> ModifierStackingGroupId {
    ModifierStackingGroupId::new(raw).expect("nonzero ID")
}
const fn id_selector(raw: u32) -> SelectorId {
    SelectorId::new(raw).expect("nonzero ID")
}
const fn id_program(raw: u32) -> ProgramId {
    ProgramId::new(raw).expect("nonzero ID")
}
const fn id_trigger(raw: u32) -> TriggerId {
    TriggerId::new(raw).expect("nonzero ID")
}
const fn id_effect(raw: u32) -> EffectDefinitionId {
    EffectDefinitionId::new(raw).expect("nonzero ID")
}
const fn id_slot(raw: u32) -> StateSlotDefinitionId {
    StateSlotDefinitionId::new(raw).expect("nonzero ID")
}
