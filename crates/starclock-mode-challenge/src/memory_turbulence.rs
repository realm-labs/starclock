//! Executable active Memory Turbulence projected from MazeBuff 3030146.

use starclock_combat::{
    ModifierDefinitionId, ModifierStackingGroupId, ProgramId, RuleBundleId, RuleId, Scalar,
    SelectorId, SourceDefinitionId, StateSlotDefinitionId, TriggerId,
    catalog::{
        action::AbilityTag,
        definition::{ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition},
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
    rng::types::DrawPurpose,
    rule::model::{
        BattleRuleDefinition, BattleRuleScope, Comparison, ConditionExpr, EventFilter, OnceScope,
        ProgramStep, ReactionPriority, RuleEventPoint, RuleOperationTemplate, RuleSource,
        RuleValue, RuleValueKind, SlotPersistence, SlotVisibility, SourceClass, StateSlotDef,
        TriggerDef, TriggerPhase, ValueExpr,
    },
};

pub const TURBULENCE_BUNDLE: RuleBundleId = id_rule_bundle(0x7d46_0001);
pub const TURBULENCE_RULE: RuleId = id_rule(0x7d46_0002);
pub const TURBULENCE_SOURCE: SourceDefinitionId = id_source(30_301_460);
pub const ULTIMATE_BOOST: ModifierDefinitionId = id_modifier(0x7d46_0003);
pub const FOLLOW_UP_BOOST: ModifierDefinitionId = id_modifier(0x7d46_0004);

const BOOST_GROUP: ModifierStackingGroupId = id_modifier_group(0x7d46_0005);
const ALL_ALLIES: SelectorId = id_selector(0x7d46_0006);
const ALL_ENEMIES: SelectorId = id_selector(0x7d46_0007);
const STORED_HITS: StateSlotDefinitionId = id_slot(0x7d46_0008);
const ACCUMULATE: ProgramId = id_program(0x7d46_0009);
const DISCHARGE: ProgramId = id_program(0x7d46_000a);
const ULTIMATE_TRIGGER: TriggerId = id_trigger(0x7d46_000b);
const FOLLOW_UP_TRIGGER: TriggerId = id_trigger(0x7d46_000c);
const CYCLE_TRIGGER: TriggerId = id_trigger(0x7d46_000d);
const TARGET_DRAW: DrawPurpose =
    DrawPurpose::new(0x3046).expect("reserved draw purpose is nonzero");

/// Complete catalog contribution for the released 3030146/30146 Turbulence.
pub struct MemoryTurbulenceDefinitions {
    pub modifier_group: ModifierStackingGroup,
    pub modifiers: [ModifierDefinition; 2],
    pub selectors: [SelectorDefinition; 2],
    pub programs: [ProgramDefinition; 2],
    pub rule: RuleDefinition,
    pub bundle: RuleBundle,
    pub source: RuleSource,
}

impl MemoryTurbulenceDefinitions {
    #[must_use]
    pub fn active() -> Self {
        let source = RuleSource::new(
            TURBULENCE_SOURCE,
            SourceClass::Mode,
            Vec::new(),
            [
                0x7e, 0xfe, 0x06, 0xaa, 0x8f, 0x5f, 0x30, 0x51, 0x3a, 0x81, 0x82, 0x40, 0x60, 0x02,
                0x74, 0xf9, 0x34, 0xfc, 0x0e, 0x0f, 0x27, 0x9d, 0x81, 0x70, 0x5f, 0x67, 0xf2, 0xa6,
                0xd2, 0x57, 0x35, 0x50,
            ],
        );
        let selectors = [
            SelectorDefinition::new(ALL_ALLIES).with_rule_units(team_selector(
                RuleSelectorSide::Same,
                RuleLifePredicate::Alive,
            )),
            SelectorDefinition::new(ALL_ENEMIES).with_rule_units(team_selector(
                RuleSelectorSide::Opposing,
                RuleLifePredicate::Any,
            )),
        ];
        let programs = [
            ProgramDefinition::new(
                ACCUMULATE,
                Vec::new(),
                vec![ALL_ALLIES],
                Vec::new(),
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::AddSlot {
                    slot: STORED_HITS,
                    value: integer(1),
                },
            )]),
            ProgramDefinition::new(
                DISCHARGE,
                Vec::new(),
                vec![ALL_ENEMIES],
                Vec::new(),
                Vec::new(),
            )
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::RandomRepeatedTrueDamage {
                    selector: ALL_ENEMIES,
                    repetitions: ValueExpr::Slot(STORED_HITS),
                    maximum_repetitions: 15,
                    normal_coefficient: Scalar::from_scaled(120_000),
                    elite_coefficient: Scalar::from_scaled(20_000),
                    boss_coefficient: Scalar::from_scaled(12_000),
                    target_rng_purpose: TARGET_DRAW,
                }),
                ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                    slot: STORED_HITS,
                    value: integer(0),
                }),
            ]),
        ];
        let triggers = vec![
            action_trigger(ULTIMATE_TRIGGER, AbilityTag::Ultimate),
            action_trigger(FOLLOW_UP_TRIGGER, AbilityTag::FollowUp),
            TriggerDef {
                id: CYCLE_TRIGGER,
                event: RuleEventPoint::CycleStarted.kind(),
                event_point: RuleEventPoint::CycleStarted,
                phase: TriggerPhase::AfterEvent,
                filter: EventFilter::default(),
                condition: ConditionExpr::Compare {
                    lhs: Box::new(ValueExpr::Slot(STORED_HITS)),
                    operator: Comparison::GreaterOrEqual,
                    rhs: Box::new(integer(1)),
                },
                once_scope: OnceScope::Event,
                priority: ReactionPriority::new(0),
                program: DISCHARGE,
            },
        ];
        let runtime = BattleRuleDefinition::new(
            source.clone(),
            vec![
                StateSlotDef::new(
                    STORED_HITS,
                    RuleValueKind::Integer,
                    BattleRuleScope::Battle,
                    RuleValue::Integer(0),
                )
                .with_bounds(RuleValue::Integer(0), RuleValue::Integer(15))
                .with_policy(SlotVisibility::Public, SlotPersistence::ScopeLifetime),
            ],
            triggers,
            None,
        );
        Self {
            modifier_group: ModifierStackingGroup {
                id: BOOST_GROUP,
                aggregation: ModifierAggregation::Sum,
                comparator: None,
            },
            modifiers: [
                boost_modifier(ULTIMATE_BOOST, "ultimate"),
                boost_modifier(FOLLOW_UP_BOOST, "follow_up"),
            ],
            selectors,
            programs,
            rule: RuleDefinition::new(
                TURBULENCE_RULE,
                vec![ACCUMULATE, DISCHARGE],
                vec![ALL_ALLIES, ALL_ENEMIES],
            )
            .with_runtime(runtime),
            bundle: RuleBundle::new(TURBULENCE_BUNDLE, vec![TURBULENCE_RULE]),
            source,
        }
    }
}

fn team_selector(side: RuleSelectorSide, life: RuleLifePredicate) -> RuleUnitSelector {
    RuleUnitSelector::new(
        RuleSelectorOrigin::Team,
        side,
        life,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StableId,
        0,
        32,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::All,
        None,
        false,
    )
    .expect("bounded team selector is valid")
}

fn action_trigger(id: TriggerId, ability_tag: AbilityTag) -> TriggerDef {
    TriggerDef {
        id,
        event: RuleEventPoint::ActionResolved.kind(),
        event_point: RuleEventPoint::ActionResolved,
        phase: TriggerPhase::AfterAction,
        filter: EventFilter {
            actor_selector: Some(ALL_ALLIES),
            ability_tag: Some(ability_tag),
            ..EventFilter::default()
        },
        condition: ConditionExpr::Literal(true),
        once_scope: OnceScope::Action,
        priority: ReactionPriority::new(0),
        program: ACCUMULATE,
    }
}

fn boost_modifier(id: ModifierDefinitionId, ability_tag: &str) -> ModifierDefinition {
    ModifierDefinition {
        id,
        stat: StatKind::Hp,
        stage: FormulaStage::DamageBoost,
        purpose: FormulaPurpose::OrdinaryDamage,
        value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(500_000))),
        stacking_group: BOOST_GROUP,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::DamageBoost,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: vec![ModifierFilter::AbilityTag(ability_tag.into())].into_boxed_slice(),
    }
}

fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
}

const fn id_rule_bundle(raw: u32) -> RuleBundleId {
    RuleBundleId::new(raw).expect("reserved ID is nonzero")
}
const fn id_rule(raw: u32) -> RuleId {
    RuleId::new(raw).expect("reserved ID is nonzero")
}
const fn id_source(raw: u32) -> SourceDefinitionId {
    SourceDefinitionId::new(raw).expect("reserved ID is nonzero")
}
const fn id_modifier(raw: u32) -> ModifierDefinitionId {
    ModifierDefinitionId::new(raw).expect("reserved ID is nonzero")
}
const fn id_modifier_group(raw: u32) -> ModifierStackingGroupId {
    ModifierStackingGroupId::new(raw).expect("reserved ID is nonzero")
}
const fn id_selector(raw: u32) -> SelectorId {
    SelectorId::new(raw).expect("reserved ID is nonzero")
}
const fn id_slot(raw: u32) -> StateSlotDefinitionId {
    StateSlotDefinitionId::new(raw).expect("reserved ID is nonzero")
}
const fn id_program(raw: u32) -> ProgramId {
    ProgramId::new(raw).expect("reserved ID is nonzero")
}
const fn id_trigger(raw: u32) -> TriggerId {
    TriggerId::new(raw).expect("reserved ID is nonzero")
}
