//! Executable released Pure Fiction continuous-spawn mechanics.

use starclock_combat::{
    AbilityId, ActionGauge, CountdownCatalogDefinition, CountdownDefinition, DispelCategory,
    DurationClock, EffectCategory, EffectDefinitionId, EffectRuntimeTemplate, EffectStackPolicy,
    EffectTickPhase, Energy, ModifierDefinitionId, ModifierStackingGroupId, OwnerLinkPolicy,
    ProgramId, Ratio, Rounding, RuleBundleId, RuleId, Scalar, SelectorId, SourceDefinitionId,
    Speed, StateSlotDefinitionId, TriggerId, WaveLinkPolicy,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            ActionHitDefinition, ActionResourcePolicy, HitCritPolicy, HitTargetGroup,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        definition::{
            AbilityDefinition, EffectDefinition, ProgramDefinition, RuleBundle, RuleDefinition,
            SelectorDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    formula::toughness::EnemyRank,
    modifier::model::{
        FormulaPurpose, FormulaStage, FormulaSubject, ModifierAggregation, ModifierDefinition,
        ModifierFilter, ModifierStackingGroup, SnapshotPolicy, StatKind, StatQuerySubject,
    },
    rule::model::{
        BattleRuleDefinition, BattleRuleScope, Comparison, ConditionExpr, EventFilter,
        EventValueProperty, OnceScope, ProgramStep, ReactionPriority, RuleEventPoint,
        RuleOperationTemplate, RuleSource, RuleValue, RuleValueKind, SlotPersistence,
        SlotVisibility, SourceClass, StateSlotDef, TriggerDef, TriggerPhase, ValueExpr,
    },
};

pub const PURE_FICTION_SPAWN_BUNDLE: RuleBundleId = id_bundle(0x7f10_0001);
pub const PURE_FICTION_SPAWN_RULE: RuleId = id_rule(0x7f10_0002);
pub const PURE_FICTION_SPAWN_SOURCE: SourceDefinitionId = id_source(0x7f10_0003);

const DEFEATED_NORMAL: SelectorId = id_selector(0x7f10_0004);
const REQUIRED_TARGETS: SelectorId = id_selector(0x7f10_0005);
const CURRENT_TARGET: SelectorId = id_selector(0x7f10_0006);
const ROOT_PROGRAM: ProgramId = id_program(0x7f10_0007);
const BODY_PROGRAM: ProgramId = id_program(0x7f10_0008);
const DAMAGE_PROGRAM: ProgramId = id_program(0x7f10_0009);
const NORMAL_DEFEAT_TRIGGER: TriggerId = id_trigger(0x7f10_000a);
const GRIT: StateSlotDefinitionId = id_slot(0x7f10_000b);
const SURGING: StateSlotDefinitionId = id_slot(0x7f10_000c);
const TIDE: StateSlotDefinitionId = id_slot(0x7f10_000d);
const GRIT_ROUTE: ProgramId = id_program(0x7f10_000e);
const ADD_GRIT: ProgramId = id_program(0x7f10_000f);
const ADD_TIDE: ProgramId = id_program(0x7f10_0010);
const ACTIVATE_SURGING: ProgramId = id_program(0x7f10_0011);
const END_SURGING: ProgramId = id_program(0x7f10_0012);
const COUNTDOWN_PROGRAM: ProgramId = id_program(0x7f10_0013);
const GRIT_CHANGED_TRIGGER: TriggerId = id_trigger(0x7f10_0014);
const COUNTDOWN_SIGNAL_TRIGGER: TriggerId = id_trigger(0x7f10_0015);
const COUNTDOWN_OWNER: SelectorId = id_selector(0x7f10_0016);
const COUNTDOWN_ABILITY: AbilityId = id_ability(0x7f10_0017);
const SURGING_COUNTDOWN_CODE: u32 = 0x7f10_0018;
const SURGING_END_SIGNAL: u32 = 0x7f10_0019;
pub(crate) const CACOPHONY_GRIT_ONE_SIGNAL: u32 = 0x7f10_001a;
pub(crate) const CACOPHONY_GRIT_TWO_SIGNAL: u32 = 0x7f10_001b;
pub(crate) const CACOPHONY_GRIT_THREE_SIGNAL: u32 = 0x7f10_001c;
const ROUTE_ONE: ProgramId = id_program(0x7f10_001d);
const ADD_GRIT_ONE: ProgramId = id_program(0x7f10_001e);
const ADD_TIDE_ONE: ProgramId = id_program(0x7f10_001f);
const ROUTE_TWO: ProgramId = id_program(0x7f10_0020);
const ADD_GRIT_TWO: ProgramId = id_program(0x7f10_0021);
const ADD_TIDE_TWO: ProgramId = id_program(0x7f10_0022);
const ROUTE_THREE: ProgramId = id_program(0x7f10_0023);
const ADD_GRIT_THREE: ProgramId = id_program(0x7f10_0024);
const ADD_TIDE_THREE: ProgramId = id_program(0x7f10_0025);
const SIGNAL_ONE_TRIGGER: TriggerId = id_trigger(0x7f10_0026);
const SIGNAL_TWO_TRIGGER: TriggerId = id_trigger(0x7f10_0027);
const SIGNAL_THREE_TRIGGER: TriggerId = id_trigger(0x7f10_0028);
const ALL_ENEMIES: SelectorId = id_selector(0x7f10_0029);
const REFILLED_ENEMY: SelectorId = id_selector(0x7f10_002a);
const APPLY_SURGING_TO_REFILL: ProgramId = id_program(0x7f10_002b);
const REFILL_TRIGGER: TriggerId = id_trigger(0x7f10_002c);
const SURGING_EFFECT: EffectDefinitionId = id_effect(0x7f10_002d);
const SURGING_GROUP: ModifierStackingGroupId = id_modifier_group(0x7f10_002e);
pub const PURE_FICTION_CONCORDANT_EFFECT: EffectDefinitionId = id_effect(0x7f10_0037);
const CONCORDANT_GROUP: ModifierStackingGroupId = id_modifier_group(0x7f10_0038);
const APPLY_CONCORDANT: ProgramId = id_program(0x7f10_0039);
const SURGING_REFILL_BODY: ProgramId = id_program(0x7f10_003a);
const BATTLE_START_TRIGGER: TriggerId = id_trigger(0x7f10_003b);
const NEGATIVE_APPLICATION_MARKER: EffectDefinitionId = id_effect(0x7f10_0050);
const DEJECTION_EFFECT: EffectDefinitionId = id_effect(0x7f10_0051);
const DEJECTION_GROUP: ModifierStackingGroupId = id_modifier_group(0x7f10_0052);
const NEGATIVE_EFFECT_PROGRAM: ProgramId = id_program(0x7f10_0053);
const CAPPED_GRIT_PROGRAM: ProgramId = id_program(0x7f10_0054);
const APPLY_DEJECTION_PROGRAM: ProgramId = id_program(0x7f10_0055);
const SURGING_ENTRY_BODY: ProgramId = id_program(0x7f10_0056);
const SURGING_ENTRY_DAMAGE: ProgramId = id_program(0x7f10_0057);
const NEGATIVE_TRIGGER_BASE: u32 = 0x7f10_0058;
const PLAYERS: SelectorId = id_selector(0x7f10_0067);
pub(crate) const PURE_FICTION_SURGING_STARTED_SIGNAL: u32 = 0x7f10_0068;
const DEJECTION_MODIFIER_BASE: u32 = 0x7f10_0070;

pub struct PureFictionMechanicsDefinitions {
    pub modifier_groups: Vec<ModifierStackingGroup>,
    pub modifiers: Vec<ModifierDefinition>,
    pub effects: Vec<EffectDefinition>,
    pub selectors: Vec<SelectorDefinition>,
    pub programs: Vec<ProgramDefinition>,
    pub ability: AbilityDefinition,
    pub countdown: CountdownCatalogDefinition,
    pub rule: RuleDefinition,
    pub bundle: RuleBundle,
}

impl PureFictionMechanicsDefinitions {
    #[must_use]
    pub fn active() -> Self {
        let source = RuleSource::new(
            PURE_FICTION_SPAWN_SOURCE,
            SourceClass::Mode,
            Vec::new(),
            [
                0xb8, 0x92, 0x34, 0x87, 0x4f, 0x14, 0x97, 0x4e, 0xda, 0x47, 0xac, 0x30, 0xeb, 0x2a,
                0xcf, 0x47, 0xa7, 0xa6, 0x19, 0xe2, 0x9a, 0x1a, 0xd0, 0xee, 0x82, 0xd5, 0x42, 0x2b,
                0x3b, 0xe8, 0x9c, 0x17,
            ],
        );
        let selectors = [
            SelectorDefinition::new(DEFEATED_NORMAL).with_rule_units(selector(
                RuleSelectorOrigin::PrimaryTarget,
                RuleSelectorSide::Opposing,
                RuleLifePredicate::Any,
                RuleSelectorOrdering::EventOrder,
                RuleSelectorChoice::First,
                1,
                Vec::new(),
            )),
            SelectorDefinition::new(REQUIRED_TARGETS).with_rule_units(selector(
                RuleSelectorOrigin::Encounter,
                RuleSelectorSide::Opposing,
                RuleLifePredicate::Alive,
                RuleSelectorOrdering::Formation,
                RuleSelectorChoice::All,
                8,
                vec![RuleSelectorPredicate::FormationRange {
                    minimum: 4,
                    maximum: 4,
                }],
            )),
            SelectorDefinition::new(CURRENT_TARGET).with_rule_units(selector(
                RuleSelectorOrigin::CurrentSubject,
                RuleSelectorSide::Opposing,
                RuleLifePredicate::Alive,
                RuleSelectorOrdering::StableId,
                RuleSelectorChoice::First,
                1,
                Vec::new(),
            )),
            SelectorDefinition::new(COUNTDOWN_OWNER)
                .with_unit_targets(
                    UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single)
                        .expect("self countdown target is valid"),
                )
                .with_rule_units(selector(
                    RuleSelectorOrigin::Owner,
                    RuleSelectorSide::Same,
                    RuleLifePredicate::Alive,
                    RuleSelectorOrdering::StableId,
                    RuleSelectorChoice::First,
                    1,
                    Vec::new(),
                )),
            SelectorDefinition::new(ALL_ENEMIES).with_rule_units(selector(
                RuleSelectorOrigin::Encounter,
                RuleSelectorSide::Opposing,
                RuleLifePredicate::Alive,
                RuleSelectorOrdering::Formation,
                RuleSelectorChoice::All,
                32,
                Vec::new(),
            )),
            SelectorDefinition::new(REFILLED_ENEMY).with_rule_units(selector(
                RuleSelectorOrigin::PrimaryTarget,
                RuleSelectorSide::Opposing,
                RuleLifePredicate::Alive,
                RuleSelectorOrdering::EventOrder,
                RuleSelectorChoice::First,
                1,
                Vec::new(),
            )),
            SelectorDefinition::new(PLAYERS).with_rule_units(selector(
                RuleSelectorOrigin::Team,
                RuleSelectorSide::Same,
                RuleLifePredicate::Alive,
                RuleSelectorOrdering::Formation,
                RuleSelectorChoice::All,
                8,
                Vec::new(),
            )),
        ];
        let programs = [
            ProgramDefinition::new(
                ROOT_PROGRAM,
                vec![BODY_PROGRAM, GRIT_ROUTE],
                vec![REQUIRED_TARGETS],
                Vec::new(),
                Vec::new(),
            )
            .with_steps(vec![
                ProgramStep::ForEach {
                    selector: REQUIRED_TARGETS,
                    body: BODY_PROGRAM,
                    maximum: 1,
                },
                ProgramStep::If {
                    condition: ConditionExpr::Literal(true),
                    then_program: GRIT_ROUTE,
                    else_program: None,
                },
            ]),
            ProgramDefinition::new(
                BODY_PROGRAM,
                vec![DAMAGE_PROGRAM],
                vec![CURRENT_TARGET],
                Vec::new(),
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::If {
                condition: ConditionExpr::EnemyRankEliteOrBoss(CURRENT_TARGET),
                then_program: DAMAGE_PROGRAM,
                else_program: None,
            }]),
            ProgramDefinition::new(
                DAMAGE_PROGRAM,
                Vec::new(),
                vec![CURRENT_TARGET],
                Vec::new(),
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::TrueDamage {
                    selector: CURRENT_TARGET,
                    amount: ValueExpr::Multiply {
                        lhs: Box::new(ValueExpr::QueryBaseStat {
                            subject: StatQuerySubject::CurrentTarget,
                            stat: StatKind::Hp,
                        }),
                        rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                            30_000,
                        )))),
                        rounding: Rounding::NearestTiesEven,
                    },
                },
            )]),
            ProgramDefinition::new(
                GRIT_ROUTE,
                vec![ADD_GRIT, ADD_TIDE],
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::If {
                condition: slot_equals(SURGING, RuleValue::Boolean(true)),
                then_program: ADD_TIDE,
                else_program: Some(ADD_GRIT),
            }]),
            ProgramDefinition::new(ADD_GRIT, Vec::new(), Vec::new(), Vec::new(), Vec::new())
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::AddSlot {
                        slot: GRIT,
                        value: integer(5),
                    },
                )]),
            ProgramDefinition::new(ADD_TIDE, Vec::new(), Vec::new(), Vec::new(), Vec::new())
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::AddSlot {
                        slot: TIDE,
                        value: integer(5),
                    },
                )]),
            ProgramDefinition::new(
                ACTIVATE_SURGING,
                vec![SURGING_ENTRY_BODY],
                vec![ALL_ENEMIES],
                vec![SURGING_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                    slot: GRIT,
                    value: integer(0),
                }),
                ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                    slot: SURGING,
                    value: boolean(true),
                }),
                ProgramStep::Operation(RuleOperationTemplate::CreateCountdown {
                    code: SURGING_COUNTDOWN_CODE,
                }),
                ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
                    code: PURE_FICTION_SURGING_STARTED_SIGNAL,
                    value: None,
                }),
                apply_effect(ALL_ENEMIES, SURGING_EFFECT),
                ProgramStep::ForEach {
                    selector: ALL_ENEMIES,
                    body: SURGING_ENTRY_BODY,
                    maximum: 5,
                },
            ]),
            ProgramDefinition::new(
                END_SURGING,
                Vec::new(),
                vec![ALL_ENEMIES],
                vec![SURGING_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                    selector: ALL_ENEMIES,
                    effect: SURGING_EFFECT,
                }),
                ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                    slot: SURGING,
                    value: boolean(false),
                }),
                ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                    slot: GRIT,
                    value: ValueExpr::Slot(TIDE),
                }),
                ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                    slot: TIDE,
                    value: integer(0),
                }),
            ]),
            ProgramDefinition::new(
                COUNTDOWN_PROGRAM,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::EmitRuleEvent {
                    code: SURGING_END_SIGNAL,
                    value: None,
                },
            )]),
            route_program(ROUTE_ONE, ADD_GRIT_ONE, ADD_TIDE_ONE),
            add_program(ADD_GRIT_ONE, GRIT, 1),
            add_program(ADD_TIDE_ONE, TIDE, 1),
            route_program(ROUTE_TWO, ADD_GRIT_TWO, ADD_TIDE_TWO),
            add_program(ADD_GRIT_TWO, GRIT, 2),
            add_program(ADD_TIDE_TWO, TIDE, 2),
            route_program(ROUTE_THREE, ADD_GRIT_THREE, ADD_TIDE_THREE),
            add_program(ADD_GRIT_THREE, GRIT, 3),
            add_program(ADD_TIDE_THREE, TIDE, 3),
            ProgramDefinition::new(
                APPLY_SURGING_TO_REFILL,
                vec![SURGING_REFILL_BODY],
                vec![REFILLED_ENEMY],
                vec![SURGING_EFFECT, PURE_FICTION_CONCORDANT_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![
                apply_effect(REFILLED_ENEMY, PURE_FICTION_CONCORDANT_EFFECT),
                ProgramStep::If {
                    condition: slot_equals(SURGING, RuleValue::Boolean(true)),
                    then_program: SURGING_REFILL_BODY,
                    else_program: None,
                },
            ]),
            ProgramDefinition::new(
                SURGING_REFILL_BODY,
                Vec::new(),
                vec![REFILLED_ENEMY],
                vec![SURGING_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![apply_effect(REFILLED_ENEMY, SURGING_EFFECT)]),
            ProgramDefinition::new(
                APPLY_CONCORDANT,
                Vec::new(),
                vec![ALL_ENEMIES],
                vec![PURE_FICTION_CONCORDANT_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![apply_effect(
                ALL_ENEMIES,
                PURE_FICTION_CONCORDANT_EFFECT,
            )]),
            ProgramDefinition::new(
                NEGATIVE_EFFECT_PROGRAM,
                vec![CAPPED_GRIT_PROGRAM, APPLY_DEJECTION_PROGRAM],
                vec![COUNTDOWN_OWNER, REFILLED_ENEMY],
                vec![NEGATIVE_APPLICATION_MARKER, DEJECTION_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![
                ProgramStep::If {
                    condition: ConditionExpr::Compare {
                        lhs: Box::new(ValueExpr::QueryEffectStacks {
                            subject: StatQuerySubject::EventTarget,
                            effect: NEGATIVE_APPLICATION_MARKER,
                        }),
                        operator: Comparison::Less,
                        rhs: Box::new(integer(10)),
                    },
                    then_program: CAPPED_GRIT_PROGRAM,
                    else_program: None,
                },
                ProgramStep::If {
                    condition: ConditionExpr::All(
                        vec![
                            slot_equals(SURGING, RuleValue::Boolean(true)),
                            ConditionExpr::Compare {
                                lhs: Box::new(ValueExpr::QueryEffectStacks {
                                    subject: StatQuerySubject::EventTarget,
                                    effect: DEJECTION_EFFECT,
                                }),
                                operator: Comparison::Less,
                                rhs: Box::new(ValueExpr::Choose {
                                    condition: Box::new(ConditionExpr::EffectExists {
                                        selector: COUNTDOWN_OWNER,
                                        effect: crate::pure_fiction_cacophony::TOCCATA_HOST_EFFECT,
                                    }),
                                    when_true: Box::new(integer(50)),
                                    when_false: Box::new(integer(20)),
                                }),
                            },
                        ]
                        .into_boxed_slice(),
                    ),
                    then_program: APPLY_DEJECTION_PROGRAM,
                    else_program: None,
                },
            ]),
            ProgramDefinition::new(
                CAPPED_GRIT_PROGRAM,
                vec![ROUTE_ONE],
                vec![REFILLED_ENEMY],
                vec![NEGATIVE_APPLICATION_MARKER],
                Vec::new(),
            )
            .with_steps(vec![
                apply_effect(REFILLED_ENEMY, NEGATIVE_APPLICATION_MARKER),
                ProgramStep::If {
                    condition: ConditionExpr::Literal(true),
                    then_program: ROUTE_ONE,
                    else_program: None,
                },
            ]),
            ProgramDefinition::new(
                APPLY_DEJECTION_PROGRAM,
                Vec::new(),
                vec![COUNTDOWN_OWNER, REFILLED_ENEMY],
                vec![
                    DEJECTION_EFFECT,
                    crate::pure_fiction_cacophony::TOCCATA_HOST_EFFECT,
                ],
                Vec::new(),
            )
            .with_steps(vec![apply_effect_with_stacks(
                REFILLED_ENEMY,
                DEJECTION_EFFECT,
                ValueExpr::Choose {
                    condition: Box::new(ConditionExpr::EffectExists {
                        selector: COUNTDOWN_OWNER,
                        effect: crate::pure_fiction_cacophony::TOCCATA_HOST_EFFECT,
                    }),
                    when_true: Box::new(integer(2)),
                    when_false: Box::new(integer(1)),
                },
            )]),
            ProgramDefinition::new(
                SURGING_ENTRY_BODY,
                vec![SURGING_ENTRY_DAMAGE],
                vec![CURRENT_TARGET],
                Vec::new(),
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::If {
                condition: ConditionExpr::Compare {
                    lhs: Box::new(negative_effect_stacks(StatQuerySubject::CurrentTarget)),
                    operator: Comparison::Greater,
                    rhs: Box::new(integer(0)),
                },
                then_program: SURGING_ENTRY_DAMAGE,
                else_program: None,
            }]),
            ProgramDefinition::new(
                SURGING_ENTRY_DAMAGE,
                Vec::new(),
                vec![CURRENT_TARGET],
                Vec::new(),
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::TrueDamage {
                    selector: CURRENT_TARGET,
                    amount: ValueExpr::Multiply {
                        lhs: Box::new(ValueExpr::Multiply {
                            lhs: Box::new(ValueExpr::QueryBaseStat {
                                subject: StatQuerySubject::Owner,
                                stat: StatKind::Atk,
                            }),
                            rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(
                                Scalar::from_scaled(800_000),
                            ))),
                            rounding: Rounding::NearestTiesEven,
                        }),
                        rhs: Box::new(ValueExpr::Convert {
                            value: Box::new(negative_effect_stacks(
                                StatQuerySubject::CurrentTarget,
                            )),
                            target: RuleValueKind::Scalar,
                            rounding: Rounding::NearestTiesEven,
                        }),
                        rounding: Rounding::NearestTiesEven,
                    },
                },
            )]),
        ];
        let mut triggers = vec![
            TriggerDef {
                id: NORMAL_DEFEAT_TRIGGER,
                event: RuleEventPoint::UnitDefeated.kind(),
                event_point: RuleEventPoint::UnitDefeated,
                phase: TriggerPhase::AfterEvent,
                filter: EventFilter {
                    target_selector: Some(DEFEATED_NORMAL),
                    ..EventFilter::default()
                },
                condition: ConditionExpr::EnemyRank(DEFEATED_NORMAL, EnemyRank::Normal),
                once_scope: OnceScope::Event,
                priority: ReactionPriority::new(0),
                program: ROOT_PROGRAM,
            },
            TriggerDef {
                id: GRIT_CHANGED_TRIGGER,
                event: RuleEventPoint::RuleStateChanged.kind(),
                event_point: RuleEventPoint::RuleStateChanged,
                phase: TriggerPhase::AfterEvent,
                filter: EventFilter::default(),
                condition: ConditionExpr::All(
                    vec![
                        compare_slot(GRIT, Comparison::GreaterOrEqual, integer(100)),
                        slot_equals(SURGING, RuleValue::Boolean(false)),
                    ]
                    .into_boxed_slice(),
                ),
                once_scope: OnceScope::Event,
                priority: ReactionPriority::new(0),
                program: ACTIVATE_SURGING,
            },
            TriggerDef {
                id: COUNTDOWN_SIGNAL_TRIGGER,
                event: RuleEventPoint::InformationalRule.kind(),
                event_point: RuleEventPoint::InformationalRule,
                phase: TriggerPhase::AfterEvent,
                filter: EventFilter::default(),
                condition: ConditionExpr::All(
                    vec![
                        ConditionExpr::Compare {
                            lhs: Box::new(ValueExpr::ReadEventProperty(
                                EventValueProperty::RuleSignalCode,
                            )),
                            operator: Comparison::Equal,
                            rhs: Box::new(integer(i64::from(SURGING_END_SIGNAL))),
                        },
                        slot_equals(SURGING, RuleValue::Boolean(true)),
                    ]
                    .into_boxed_slice(),
                ),
                once_scope: OnceScope::Event,
                priority: ReactionPriority::new(0),
                program: END_SURGING,
            },
            signal_trigger(SIGNAL_ONE_TRIGGER, CACOPHONY_GRIT_ONE_SIGNAL, ROUTE_ONE),
            signal_trigger(SIGNAL_TWO_TRIGGER, CACOPHONY_GRIT_TWO_SIGNAL, ROUTE_TWO),
            signal_trigger(
                SIGNAL_THREE_TRIGGER,
                CACOPHONY_GRIT_THREE_SIGNAL,
                ROUTE_THREE,
            ),
            TriggerDef {
                id: REFILL_TRIGGER,
                event: RuleEventPoint::UnitSummoned.kind(),
                event_point: RuleEventPoint::UnitSummoned,
                phase: TriggerPhase::AfterEvent,
                filter: EventFilter {
                    target_selector: Some(REFILLED_ENEMY),
                    ..EventFilter::default()
                },
                condition: ConditionExpr::Literal(true),
                once_scope: OnceScope::Event,
                priority: ReactionPriority::new(0),
                program: APPLY_SURGING_TO_REFILL,
            },
            TriggerDef {
                id: BATTLE_START_TRIGGER,
                event: RuleEventPoint::BattleStarted.kind(),
                event_point: RuleEventPoint::BattleStarted,
                phase: TriggerPhase::AfterEvent,
                filter: EventFilter::default(),
                condition: ConditionExpr::Literal(true),
                once_scope: OnceScope::Battle,
                priority: ReactionPriority::new(0),
                program: APPLY_CONCORDANT,
            },
        ];
        triggers.extend(negative_effect_triggers());
        let runtime = BattleRuleDefinition::new(
            source,
            vec![
                public_integer_slot(GRIT, 100),
                StateSlotDef::new(
                    SURGING,
                    RuleValueKind::Boolean,
                    BattleRuleScope::Battle,
                    RuleValue::Boolean(false),
                )
                .with_policy(SlotVisibility::Public, SlotPersistence::ScopeLifetime),
                public_integer_slot(TIDE, 30),
            ],
            triggers,
            None,
        );
        let ability = AbilityDefinition::new(
            COUNTDOWN_ABILITY,
            COUNTDOWN_PROGRAM,
            COUNTDOWN_OWNER,
            Vec::new(),
        )
        .with_action(countdown_action())
        .with_programs(vec![
            AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, COUNTDOWN_PROGRAM)
                .expect("countdown binding sequence is non-zero"),
        ]);
        let countdown = CountdownCatalogDefinition::new(
            SURGING_COUNTDOWN_CODE,
            CountdownDefinition::new(
                COUNTDOWN_ABILITY,
                ActionGauge::from_scaled(10_000_000_000).expect("100-AV countdown gauge is valid"),
                Speed::from_scaled(100_000_000).expect("countdown speed 100 is valid"),
                OwnerLinkPolicy::Persist,
                OwnerLinkPolicy::Persist,
                WaveLinkPolicy::Persist,
            ),
        )
        .expect("non-zero Pure Fiction countdown code is valid");
        Self {
            selectors: selectors.into(),
            programs: programs.into(),
            ability,
            countdown,
            rule: RuleDefinition::new(
                PURE_FICTION_SPAWN_RULE,
                vec![
                    ROOT_PROGRAM,
                    BODY_PROGRAM,
                    DAMAGE_PROGRAM,
                    GRIT_ROUTE,
                    ADD_GRIT,
                    ADD_TIDE,
                    ACTIVATE_SURGING,
                    END_SURGING,
                    COUNTDOWN_PROGRAM,
                    ROUTE_ONE,
                    ADD_GRIT_ONE,
                    ADD_TIDE_ONE,
                    ROUTE_TWO,
                    ADD_GRIT_TWO,
                    ADD_TIDE_TWO,
                    ROUTE_THREE,
                    ADD_GRIT_THREE,
                    ADD_TIDE_THREE,
                    APPLY_SURGING_TO_REFILL,
                    APPLY_CONCORDANT,
                    SURGING_REFILL_BODY,
                    NEGATIVE_EFFECT_PROGRAM,
                    CAPPED_GRIT_PROGRAM,
                    APPLY_DEJECTION_PROGRAM,
                    SURGING_ENTRY_BODY,
                    SURGING_ENTRY_DAMAGE,
                ],
                vec![
                    DEFEATED_NORMAL,
                    REQUIRED_TARGETS,
                    CURRENT_TARGET,
                    COUNTDOWN_OWNER,
                    ALL_ENEMIES,
                    REFILLED_ENEMY,
                    PLAYERS,
                ],
            )
            .with_runtime(runtime),
            bundle: RuleBundle::new(PURE_FICTION_SPAWN_BUNDLE, vec![PURE_FICTION_SPAWN_RULE]),
            modifier_groups: vec![
                modifier_group(SURGING_GROUP),
                modifier_group(CONCORDANT_GROUP),
                modifier_group(DEJECTION_GROUP),
            ],
            modifiers: all_mechanics_modifiers(),
            effects: vec![
                surging_effect(),
                concordant_effect(),
                negative_application_marker(),
                dejection_effect(),
            ],
        }
    }
}

fn apply_effect(selector: SelectorId, effect: EffectDefinitionId) -> ProgramStep {
    apply_effect_with_stacks(selector, effect, integer(1))
}

fn apply_effect_with_stacks(
    selector: SelectorId,
    effect: EffectDefinitionId,
    stacks: ValueExpr,
) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector,
        effect,
        stacks,
        chance: starclock_combat::rule::model::RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })
}

fn negative_effect_triggers() -> Vec<TriggerDef> {
    let mut output = Vec::with_capacity(9);
    let points = [
        RuleEventPoint::EffectApplied,
        RuleEventPoint::EffectStacksChanged,
        RuleEventPoint::EffectRefreshed,
    ];
    let categories = [
        EffectCategory::Debuff,
        EffectCategory::Control,
        EffectCategory::Dot,
    ];
    for (point_index, point) in points.into_iter().enumerate() {
        for (category_index, category) in categories.into_iter().enumerate() {
            let offset = u32::try_from(point_index * categories.len() + category_index)
                .expect("nine negative-effect triggers fit u32");
            output.push(TriggerDef {
                id: id_trigger(NEGATIVE_TRIGGER_BASE + offset),
                event: point.kind(),
                event_point: point,
                phase: TriggerPhase::AfterEvent,
                filter: EventFilter {
                    applier_selector: Some(PLAYERS),
                    target_selector: Some(REFILLED_ENEMY),
                    effect_category: Some(category),
                    ..EventFilter::default()
                },
                condition: ConditionExpr::Literal(true),
                once_scope: OnceScope::Event,
                priority: ReactionPriority::new(0),
                program: NEGATIVE_EFFECT_PROGRAM,
            });
        }
    }
    output
}

fn negative_effect_stacks(subject: StatQuerySubject) -> ValueExpr {
    ValueExpr::Add(
        Box::new(ValueExpr::QueryEffectCategoryStacks {
            subject,
            category: EffectCategory::Debuff,
        }),
        Box::new(ValueExpr::Add(
            Box::new(ValueExpr::QueryEffectCategoryStacks {
                subject,
                category: EffectCategory::Control,
            }),
            Box::new(ValueExpr::QueryEffectCategoryStacks {
                subject,
                category: EffectCategory::Dot,
            }),
        )),
    )
}

fn surging_effect() -> EffectDefinition {
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::NeutralState,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .expect("permanent Surging Grit vulnerability is valid");
    EffectDefinition::new(
        SURGING_EFFECT,
        Vec::new(),
        surging_modifiers()
            .iter()
            .map(|modifier| modifier.id)
            .collect(),
    )
    .with_runtime_template(runtime)
}

fn concordant_effect() -> EffectDefinition {
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::NeutralState,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .expect("permanent Concordant effect is valid");
    EffectDefinition::new(
        PURE_FICTION_CONCORDANT_EFFECT,
        Vec::new(),
        concordant_modifiers()
            .iter()
            .map(|modifier| modifier.id)
            .collect(),
    )
    .with_runtime_template(runtime)
}

fn negative_application_marker() -> EffectDefinition {
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::NeutralState,
        DispelCategory::NonDispellable,
        10,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .expect("per-enemy Grit application counter is valid");
    EffectDefinition::new(NEGATIVE_APPLICATION_MARKER, Vec::new(), Vec::new())
        .with_runtime_template(runtime)
}

fn dejection_effect() -> EffectDefinition {
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::NeutralState,
        DispelCategory::NonDispellable,
        50,
        Some(integer(2)),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .expect("two-turn Dejection stack is valid");
    EffectDefinition::new(
        DEJECTION_EFFECT,
        Vec::new(),
        dejection_modifiers()
            .iter()
            .map(|modifier| modifier.id)
            .collect(),
    )
    .with_runtime_template(runtime)
}

fn all_mechanics_modifiers() -> Vec<ModifierDefinition> {
    let mut output = surging_modifiers();
    output.extend(concordant_modifiers());
    output.extend(dejection_modifiers());
    output
}

fn dejection_modifiers() -> Vec<ModifierDefinition> {
    damage_purposes()
        .into_iter()
        .enumerate()
        .map(|(index, purpose)| ModifierDefinition {
            id: id_modifier(
                DEJECTION_MODIFIER_BASE
                    + u32::try_from(index).expect("seven damage purposes fit u32"),
            ),
            stat: StatKind::Hp,
            stage: FormulaStage::Resistance,
            purpose,
            value: ValueExpr::Multiply {
                lhs: Box::new(ValueExpr::Convert {
                    value: Box::new(ValueExpr::QueryEffectStacks {
                        subject: StatQuerySubject::CurrentTarget,
                        effect: DEJECTION_EFFECT,
                    }),
                    target: RuleValueKind::Scalar,
                    rounding: Rounding::NearestTiesEven,
                }),
                rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                    10_000,
                )))),
                rounding: Rounding::NearestTiesEven,
            },
            stacking_group: DEJECTION_GROUP,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::Resistance,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)]
                .into_boxed_slice(),
        })
        .collect()
}

fn concordant_modifiers() -> Vec<ModifierDefinition> {
    let mut output = vec![ModifierDefinition {
        id: id_modifier(0x7f10_003c),
        stat: StatKind::EffectResistance,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::EffectChance,
        value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(-300_000))),
        stacking_group: CONCORDANT_GROUP,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Flat,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::new([]),
    }];
    let negative_stacks = ValueExpr::Add(
        Box::new(ValueExpr::QueryEffectCategoryStacks {
            subject: StatQuerySubject::CurrentTarget,
            category: EffectCategory::Debuff,
        }),
        Box::new(ValueExpr::Add(
            Box::new(ValueExpr::QueryEffectCategoryStacks {
                subject: StatQuerySubject::CurrentTarget,
                category: EffectCategory::Control,
            }),
            Box::new(ValueExpr::QueryEffectCategoryStacks {
                subject: StatQuerySubject::CurrentTarget,
                category: EffectCategory::Dot,
            }),
        )),
    );
    for (index, purpose) in damage_purposes().into_iter().enumerate() {
        output.push(ModifierDefinition {
            id: id_modifier(
                0x7f10_003d + u32::try_from(index).expect("seven damage purposes fit u32"),
            ),
            stat: StatKind::Hp,
            stage: FormulaStage::Vulnerability,
            purpose,
            value: ValueExpr::Choose {
                condition: Box::new(ConditionExpr::Compare {
                    lhs: Box::new(negative_stacks.clone()),
                    operator: Comparison::GreaterOrEqual,
                    rhs: Box::new(integer(4)),
                }),
                when_true: Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                    200_000,
                )))),
                when_false: Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO))),
            },
            stacking_group: CONCORDANT_GROUP,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::Vulnerability,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)]
                .into_boxed_slice(),
        });
    }
    output
}

fn modifier_group(id: ModifierStackingGroupId) -> ModifierStackingGroup {
    ModifierStackingGroup {
        id,
        aggregation: ModifierAggregation::Sum,
        comparator: None,
    }
}

fn surging_modifiers() -> Vec<ModifierDefinition> {
    damage_purposes()
        .into_iter()
        .enumerate()
        .map(|(index, purpose)| ModifierDefinition {
            id: id_modifier(
                0x7f10_0030 + u32::try_from(index).expect("seven damage purposes fit u32"),
            ),
            stat: StatKind::Hp,
            stage: FormulaStage::Vulnerability,
            purpose,
            value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(500_000))),
            stacking_group: SURGING_GROUP,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::Vulnerability,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)]
                .into_boxed_slice(),
        })
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

fn route_program(id: ProgramId, grit: ProgramId, tide: ProgramId) -> ProgramDefinition {
    ProgramDefinition::new(id, vec![grit, tide], Vec::new(), Vec::new(), Vec::new()).with_steps(
        vec![ProgramStep::If {
            condition: slot_equals(SURGING, RuleValue::Boolean(true)),
            then_program: tide,
            else_program: Some(grit),
        }],
    )
}

fn add_program(id: ProgramId, slot: StateSlotDefinitionId, amount: i64) -> ProgramDefinition {
    ProgramDefinition::new(id, Vec::new(), Vec::new(), Vec::new(), Vec::new()).with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::AddSlot {
            slot,
            value: integer(amount),
        }),
    ])
}

fn signal_trigger(id: TriggerId, code: u32, program: ProgramId) -> TriggerDef {
    TriggerDef {
        id,
        event: RuleEventPoint::InformationalRule.kind(),
        event_point: RuleEventPoint::InformationalRule,
        phase: TriggerPhase::AfterEvent,
        filter: EventFilter::default(),
        condition: ConditionExpr::Compare {
            lhs: Box::new(ValueExpr::ReadEventProperty(
                EventValueProperty::RuleSignalCode,
            )),
            operator: Comparison::Equal,
            rhs: Box::new(integer(i64::from(code))),
        },
        once_scope: OnceScope::Event,
        priority: ReactionPriority::new(0),
        program,
    }
}

fn countdown_action() -> AbilityActionDefinition {
    AbilityActionDefinition::new(
        AbilityKind::Countdown,
        1,
        TargetInvalidationPolicy::KeepIfPresent,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .expect("countdown action has a non-zero target cap")
    .with_hits(vec![ActionHitDefinition::new(Vec::new()).with_profile(
        HitTargetGroup::Selected,
        Ratio::ONE,
        Ratio::ONE,
        HitCritPolicy::Never,
    )])
    .expect("countdown hit profile is valid")
}

fn public_integer_slot(id: StateSlotDefinitionId, maximum: i64) -> StateSlotDef {
    StateSlotDef::new(
        id,
        RuleValueKind::Integer,
        BattleRuleScope::Battle,
        RuleValue::Integer(0),
    )
    .with_bounds(RuleValue::Integer(0), RuleValue::Integer(maximum))
    .with_policy(SlotVisibility::Public, SlotPersistence::ScopeLifetime)
}

fn compare_slot(
    slot: StateSlotDefinitionId,
    operator: Comparison,
    rhs: ValueExpr,
) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::Slot(slot)),
        operator,
        rhs: Box::new(rhs),
    }
}

fn slot_equals(slot: StateSlotDefinitionId, value: RuleValue) -> ConditionExpr {
    compare_slot(slot, Comparison::Equal, ValueExpr::Literal(value))
}

fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
}

fn boolean(value: bool) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Boolean(value))
}

fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    life: RuleLifePredicate,
    ordering: RuleSelectorOrdering,
    choice: RuleSelectorChoice,
    maximum: u16,
    predicates: Vec<RuleSelectorPredicate>,
) -> RuleUnitSelector {
    RuleUnitSelector::new(
        origin,
        side,
        life,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        ordering,
        0,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        choice,
        None,
        false,
    )
    .expect("bounded Pure Fiction selector is valid")
    .with_predicates(predicates)
}

const fn id_bundle(raw: u32) -> RuleBundleId {
    RuleBundleId::new(raw).expect("reserved ID is nonzero")
}
const fn id_rule(raw: u32) -> RuleId {
    RuleId::new(raw).expect("reserved ID is nonzero")
}
const fn id_source(raw: u32) -> SourceDefinitionId {
    SourceDefinitionId::new(raw).expect("reserved ID is nonzero")
}
const fn id_selector(raw: u32) -> SelectorId {
    SelectorId::new(raw).expect("reserved ID is nonzero")
}
const fn id_program(raw: u32) -> ProgramId {
    ProgramId::new(raw).expect("reserved ID is nonzero")
}
const fn id_trigger(raw: u32) -> TriggerId {
    TriggerId::new(raw).expect("reserved ID is nonzero")
}
const fn id_slot(raw: u32) -> StateSlotDefinitionId {
    StateSlotDefinitionId::new(raw).expect("reserved ID is nonzero")
}
const fn id_ability(raw: u32) -> AbilityId {
    AbilityId::new(raw).expect("reserved ID is nonzero")
}
const fn id_effect(raw: u32) -> EffectDefinitionId {
    EffectDefinitionId::new(raw).expect("reserved ID is nonzero")
}
const fn id_modifier(raw: u32) -> ModifierDefinitionId {
    ModifierDefinitionId::new(raw).expect("reserved ID is nonzero")
}
const fn id_modifier_group(raw: u32) -> ModifierStackingGroupId {
    ModifierStackingGroupId::new(raw).expect("reserved ID is nonzero")
}
