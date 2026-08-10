//! Released 4.4 Apocalyptic Shadow environment and Finality's Axiom mechanics.

use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectDefinitionId, EffectRemovalOrder,
    EffectRuntimeTemplate, EffectStackPolicy, EffectTickPhase, ModifierDefinitionId,
    ModifierStackingGroupId, ProgramId, Rounding, RuleBundleId, RuleId, Scalar, SelectorId,
    SourceDefinitionId, TriggerId,
    catalog::{
        definition::{
            EffectDefinition, ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition,
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
        BattleRuleDefinition, Comparison, ConditionExpr, EventFilter, EventValueProperty,
        OnceScope, ProgramStep, ReactionPriority, ResourceUpdateKind, RuleDamageClass,
        RuleEffectChancePolicy, RuleEventPoint, RuleOperationTemplate, RuleResourceKind,
        RuleSource, RuleValue, RuleValueKind, SourceClass, TriggerDef, TriggerPhase, ValueExpr,
    },
};

pub const RUINOUS_EMBERS_BUNDLE: RuleBundleId = bundle(3_110_006);
pub const BLIGHTED_TO_BONE_BUNDLE: RuleBundleId = bundle(3_111_058);
pub const UNSTOPPABLE_FORCE_BUNDLE: RuleBundleId = bundle(3_111_065);
pub const SHATTERSTRIKE_BUNDLE: RuleBundleId = bundle(3_111_077);
pub const LINEBREAKER_BUNDLE: RuleBundleId = bundle(3_111_078);
pub const UNTO_APOTHEOSIS_BUNDLE: RuleBundleId = bundle(3_111_079);
pub const WHIRLWIND_TURN_BUNDLE: RuleBundleId = bundle(3_111_081);
pub const KNOWLEDGE_DECORUM_BUNDLE: RuleBundleId = bundle(3_111_082);
pub const OPPOSE_TENDERNESS_BUNDLE: RuleBundleId = bundle(3_111_083);
pub const MOMENT_OPPORTUNITY_BUNDLE: RuleBundleId = bundle(3_111_085);
pub const APOCALYPTIC_SOURCE: SourceDefinitionId = source(0x7f30_0001);
pub const APOCALYPTIC_PUNCHLINE_RESOURCE: SourceDefinitionId = source(0x7f30_0002);
pub const APOCALYPTIC_PUNCHLINE_KEY: &str = "shared.punchline";

const PLAYERS: SelectorId = selector_id(0x7f30_0010);
const FIRST_PLAYER: SelectorId = selector_id(0x7f30_0011);
const ENEMIES: SelectorId = selector_id(0x7f30_0012);
const ACTOR: SelectorId = selector_id(0x7f30_0013);
const EVENT_TARGET: SelectorId = selector_id(0x7f30_0014);
const CURRENT_PLAYER: SelectorId = selector_id(0x7f30_0015);

const EMBER_EFFECT: EffectDefinitionId = effect_id(0x7f30_0100);
const BLIGHTED_EFFECT: EffectDefinitionId = effect_id(0x7f30_0101);
const UNSTOPPABLE_EFFECT: EffectDefinitionId = effect_id(0x7f30_0102);
const SHATTER_EFFECT: EffectDefinitionId = effect_id(0x7f30_0103);
const LINEBREAKER_EFFECT: EffectDefinitionId = effect_id(0x7f30_0104);
const APOTHEOSIS_EFFECT: EffectDefinitionId = effect_id(0x7f30_0105);
const WHIRLWIND_EFFECT: EffectDefinitionId = effect_id(0x7f30_0106);
const KNOWLEDGE_EFFECT: EffectDefinitionId = effect_id(0x7f30_0107);
const OPPOSE_EFFECT: EffectDefinitionId = effect_id(0x7f30_0108);
const MOMENT_EFFECT: EffectDefinitionId = effect_id(0x7f30_0109);

const GROUP_BASE: u32 = 0x7f30_0200;
const MODIFIER_BASE: u32 = 0x7f30_0300;
const PROGRAM_BASE: u32 = 0x7f30_0500;
const RULE_BASE: u32 = 0x7f30_0600;
const TRIGGER_BASE: u32 = 0x7f30_0700;

pub struct ApocalypticMechanicsDefinitions {
    pub modifier_groups: Vec<ModifierStackingGroup>,
    pub modifiers: Vec<ModifierDefinition>,
    pub effects: Vec<EffectDefinition>,
    pub selectors: Vec<SelectorDefinition>,
    pub programs: Vec<ProgramDefinition>,
    pub rules: Vec<RuleDefinition>,
    pub bundles: Vec<RuleBundle>,
    pub source: RuleSource,
}

impl ApocalypticMechanicsDefinitions {
    #[must_use]
    pub fn active() -> Self {
        let source = RuleSource::new(
            APOCALYPTIC_SOURCE,
            SourceClass::Mode,
            Vec::new(),
            [
                0x5d, 0x76, 0x8b, 0x0d, 0x0b, 0xd3, 0x6b, 0xef, 0x27, 0x4d, 0x09, 0x17, 0x36, 0x6d,
                0x91, 0xf9, 0x16, 0xba, 0xb5, 0xe1, 0x84, 0xe4, 0x1a, 0x77, 0xe3, 0xa3, 0xa7, 0x90,
                0x9e, 0x87, 0x70, 0xd1,
            ],
        );
        let selectors = vec![
            SelectorDefinition::new(PLAYERS).with_rule_units(units(
                RuleSelectorOrigin::Owner,
                RuleSelectorSide::Same,
                RuleSelectorOrdering::Formation,
                16,
                Vec::new(),
            )),
            SelectorDefinition::new(FIRST_PLAYER).with_rule_units(units(
                RuleSelectorOrigin::Owner,
                RuleSelectorSide::Same,
                RuleSelectorOrdering::Formation,
                1,
                Vec::new(),
            )),
            SelectorDefinition::new(ENEMIES).with_rule_units(units(
                RuleSelectorOrigin::Encounter,
                RuleSelectorSide::Opposing,
                RuleSelectorOrdering::Formation,
                32,
                Vec::new(),
            )),
            SelectorDefinition::new(ACTOR).with_rule_units(units(
                RuleSelectorOrigin::Actor,
                RuleSelectorSide::Same,
                RuleSelectorOrdering::StableId,
                1,
                Vec::new(),
            )),
            SelectorDefinition::new(EVENT_TARGET).with_rule_units(units(
                RuleSelectorOrigin::EventTargets,
                RuleSelectorSide::Opposing,
                RuleSelectorOrdering::EventOrder,
                16,
                Vec::new(),
            )),
            SelectorDefinition::new(CURRENT_PLAYER).with_rule_units(units(
                RuleSelectorOrigin::CurrentSubject,
                RuleSelectorSide::Same,
                RuleSelectorOrdering::StableId,
                1,
                Vec::new(),
            )),
        ];
        let (modifier_groups, modifiers, effects) = definitions();
        let programs = programs();
        let rules = rules(&source);
        let bundles = released_bundles()
            .into_iter()
            .enumerate()
            .map(|(index, bundle)| {
                RuleBundle::new(
                    bundle,
                    vec![rule_id(RULE_BASE + u32::try_from(index).unwrap())],
                )
            })
            .collect();
        Self {
            modifier_groups,
            modifiers,
            effects,
            selectors,
            programs,
            rules,
            bundles,
            source,
        }
    }
}

#[must_use]
pub const fn released_axioms() -> [RuleBundleId; 9] {
    [
        BLIGHTED_TO_BONE_BUNDLE,
        UNSTOPPABLE_FORCE_BUNDLE,
        SHATTERSTRIKE_BUNDLE,
        LINEBREAKER_BUNDLE,
        UNTO_APOTHEOSIS_BUNDLE,
        WHIRLWIND_TURN_BUNDLE,
        KNOWLEDGE_DECORUM_BUNDLE,
        OPPOSE_TENDERNESS_BUNDLE,
        MOMENT_OPPORTUNITY_BUNDLE,
    ]
}

fn released_bundles() -> [RuleBundleId; 10] {
    [
        RUINOUS_EMBERS_BUNDLE,
        BLIGHTED_TO_BONE_BUNDLE,
        UNSTOPPABLE_FORCE_BUNDLE,
        SHATTERSTRIKE_BUNDLE,
        LINEBREAKER_BUNDLE,
        UNTO_APOTHEOSIS_BUNDLE,
        WHIRLWIND_TURN_BUNDLE,
        KNOWLEDGE_DECORUM_BUNDLE,
        OPPOSE_TENDERNESS_BUNDLE,
        MOMENT_OPPORTUNITY_BUNDLE,
    ]
}

fn definitions() -> (
    Vec<ModifierStackingGroup>,
    Vec<ModifierDefinition>,
    Vec<EffectDefinition>,
) {
    let mut groups = Vec::new();
    let mut modifiers = Vec::new();
    let mut effects = Vec::new();
    let mut add_effect = |effect: EffectDefinitionId,
                          category: EffectCategory,
                          maximum: u16,
                          duration: Option<ValueExpr>,
                          clock: DurationClock,
                          ids: Vec<ModifierDefinitionId>| {
        effects.push(
            EffectDefinition::new(effect, Vec::new(), ids).with_runtime_template(
                EffectRuntimeTemplate::new(
                    category,
                    DispelCategory::NonDispellable,
                    maximum,
                    duration,
                    clock,
                    EffectTickPhase::None,
                    EffectStackPolicy::RefreshAndAddStacks,
                )
                .expect("Apocalyptic effect runtime is valid"),
            ),
        );
    };
    let mut next = 0_u32;
    let mut effect_modifiers = |specs: Vec<ModifierSpec>| {
        specs
            .into_iter()
            .map(|spec| {
                let id = modifier_id(MODIFIER_BASE + next);
                let group = group_id(GROUP_BASE + next);
                next += 1;
                groups.push(ModifierStackingGroup {
                    id: group,
                    aggregation: ModifierAggregation::Sum,
                    comparator: None,
                });
                modifiers.push(spec.build(id, group));
                id
            })
            .collect::<Vec<_>>()
    };

    let ember = effect_modifiers(vec![
        target_damage(250_000, FormulaPurpose::OrdinaryDamage, Some("skill")),
        target_damage(150_000, FormulaPurpose::OrdinaryDamage, Some("ultimate")),
    ]);
    add_effect(
        EMBER_EFFECT,
        EffectCategory::NeutralState,
        1,
        None,
        DurationClock::Permanent,
        ember,
    );
    let blighted = effect_modifiers(vec![target_resistance(400_000, FormulaPurpose::Dot, None)]);
    add_effect(
        BLIGHTED_EFFECT,
        EffectCategory::NeutralState,
        1,
        None,
        DurationClock::Permanent,
        blighted,
    );
    let unstoppable = effect_modifiers(vec![target_defense(200_000, Some("memosprite"))]);
    add_effect(
        UNSTOPPABLE_EFFECT,
        EffectCategory::NeutralState,
        1,
        None,
        DurationClock::Permanent,
        unstoppable,
    );
    let shatter = effect_modifiers(vec![ModifierSpec {
        stat: StatKind::Hp,
        stage: FormulaStage::DamageBoost,
        purpose: FormulaPurpose::Break,
        value: ValueExpr::Add(
            Box::new(scalar(100_000)),
            Box::new(ValueExpr::Multiply {
                lhs: Box::new(ValueExpr::Convert {
                    value: Box::new(ValueExpr::Add(
                        Box::new(ValueExpr::QueryEffectStacks {
                            subject: StatQuerySubject::CurrentTarget,
                            effect: SHATTER_EFFECT,
                        }),
                        Box::new(ValueExpr::Negate(Box::new(integer(1)))),
                    )),
                    target: RuleValueKind::Scalar,
                    rounding: Rounding::NearestTiesEven,
                }),
                rhs: Box::new(scalar(50_000)),
                rounding: Rounding::NearestTiesEven,
            }),
        ),
        filters: vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)],
    }]);
    add_effect(
        SHATTER_EFFECT,
        EffectCategory::NeutralState,
        5,
        None,
        DurationClock::Permanent,
        shatter,
    );
    let linebreaker = effect_modifiers(vec![source_defense(150_000, None)]);
    add_effect(
        LINEBREAKER_EFFECT,
        EffectCategory::NeutralState,
        1,
        None,
        DurationClock::Permanent,
        linebreaker,
    );
    let apotheosis = effect_modifiers(vec![ModifierSpec {
        stat: StatKind::CritDamage,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::Stat,
        value: stack_value(APOTHEOSIS_EFFECT, 60_000, StatQuerySubject::Actor),
        filters: vec![ModifierFilter::FormulaSubject(FormulaSubject::Source)],
    }]);
    add_effect(
        APOTHEOSIS_EFFECT,
        EffectCategory::Buff,
        10,
        None,
        DurationClock::Permanent,
        apotheosis,
    );
    let whirlwind = effect_modifiers(vec![ModifierSpec {
        stat: StatKind::Spd,
        stage: FormulaStage::PercentOfBase,
        purpose: FormulaPurpose::Stat,
        value: scalar(250_000),
        filters: Vec::new(),
    }]);
    add_effect(
        WHIRLWIND_EFFECT,
        EffectCategory::Buff,
        1,
        Some(integer(3)),
        DurationClock::TargetTurnEnd,
        whirlwind,
    );
    let knowledge = effect_modifiers(
        damage_purposes()
            .into_iter()
            .map(|purpose| target_resistance(250_000, purpose, None))
            .collect(),
    );
    add_effect(
        KNOWLEDGE_EFFECT,
        EffectCategory::NeutralState,
        1,
        None,
        DurationClock::Permanent,
        knowledge,
    );
    let oppose = effect_modifiers(vec![target_resistance(
        150_000,
        FormulaPurpose::ElationDamage,
        None,
    )]);
    add_effect(
        OPPOSE_EFFECT,
        EffectCategory::NeutralState,
        1,
        None,
        DurationClock::Permanent,
        oppose,
    );
    let mut moment_specs = Vec::new();
    for tag in ["follow_up", "ultimate"] {
        moment_specs.push(target_damage(
            500_000,
            FormulaPurpose::OrdinaryDamage,
            Some(tag),
        ));
        moment_specs.push(ModifierSpec {
            stat: StatKind::Hp,
            stage: FormulaStage::DamageBoost,
            purpose: FormulaPurpose::OrdinaryDamage,
            value: ValueExpr::Choose {
                condition: Box::new(ConditionExpr::CurrentTargetIsBroken),
                when_true: Box::new(scalar(500_000)),
                when_false: Box::new(scalar(0)),
            },
            filters: vec![
                ModifierFilter::AbilityTag(tag.into()),
                ModifierFilter::FormulaSubject(FormulaSubject::Target),
            ],
        });
    }
    let moment = effect_modifiers(moment_specs);
    add_effect(
        MOMENT_EFFECT,
        EffectCategory::NeutralState,
        1,
        None,
        DurationClock::Permanent,
        moment,
    );
    (groups, modifiers, effects)
}

struct ModifierSpec {
    stat: StatKind,
    stage: FormulaStage,
    purpose: FormulaPurpose,
    value: ValueExpr,
    filters: Vec<ModifierFilter>,
}

impl ModifierSpec {
    fn build(
        self,
        id: ModifierDefinitionId,
        stacking_group: ModifierStackingGroupId,
    ) -> ModifierDefinition {
        ModifierDefinition {
            id,
            stat: self.stat,
            stage: self.stage,
            purpose: self.purpose,
            value: self.value,
            stacking_group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: self.stage,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: self.filters.into_boxed_slice(),
        }
    }
}

fn target_damage(value: i64, purpose: FormulaPurpose, tag: Option<&str>) -> ModifierSpec {
    let mut filters = vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)];
    if let Some(tag) = tag {
        filters.push(ModifierFilter::AbilityTag(tag.into()));
    }
    ModifierSpec {
        stat: StatKind::Hp,
        stage: FormulaStage::DamageBoost,
        purpose,
        value: scalar(value),
        filters,
    }
}

fn target_defense(value: i64, tag: Option<&str>) -> ModifierSpec {
    let mut filters = vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)];
    if let Some(tag) = tag {
        filters.push(ModifierFilter::AbilityTag(tag.into()));
    }
    ModifierSpec {
        stat: StatKind::Def,
        stage: FormulaStage::Defense,
        purpose: FormulaPurpose::OrdinaryDamage,
        value: scalar(value),
        filters,
    }
}

fn source_defense(value: i64, tag: Option<&str>) -> ModifierSpec {
    let mut filters = vec![ModifierFilter::FormulaSubject(FormulaSubject::Source)];
    if let Some(tag) = tag {
        filters.push(ModifierFilter::AbilityTag(tag.into()));
    }
    ModifierSpec {
        stat: StatKind::Def,
        stage: FormulaStage::Defense,
        purpose: FormulaPurpose::OrdinaryDamage,
        value: scalar(value),
        filters,
    }
}

fn target_resistance(value: i64, purpose: FormulaPurpose, tag: Option<&str>) -> ModifierSpec {
    let mut filters = vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)];
    if let Some(tag) = tag {
        filters.push(ModifierFilter::AbilityTag(tag.into()));
    }
    ModifierSpec {
        stat: StatKind::Hp,
        stage: FormulaStage::Resistance,
        purpose,
        value: scalar(value),
        filters,
    }
}

fn stack_value(
    effect: EffectDefinitionId,
    coefficient: i64,
    subject: StatQuerySubject,
) -> ValueExpr {
    ValueExpr::Multiply {
        lhs: Box::new(ValueExpr::Convert {
            value: Box::new(ValueExpr::QueryEffectStacks { subject, effect }),
            target: RuleValueKind::Scalar,
            rounding: Rounding::NearestTiesEven,
        }),
        rhs: Box::new(scalar(coefficient)),
        rounding: Rounding::NearestTiesEven,
    }
}

fn programs() -> Vec<ProgramDefinition> {
    let mut output = Vec::new();
    let starts = [
        (0, ENEMIES, EMBER_EFFECT),
        (1, ENEMIES, BLIGHTED_EFFECT),
        (2, ENEMIES, UNSTOPPABLE_EFFECT),
        (3, ENEMIES, SHATTER_EFFECT),
        (4, FIRST_PLAYER, LINEBREAKER_EFFECT),
        (7, ENEMIES, KNOWLEDGE_EFFECT),
        (8, ENEMIES, OPPOSE_EFFECT),
        (9, ENEMIES, MOMENT_EFFECT),
    ];
    for (offset, selector, effect) in starts {
        output.push(program(
            PROGRAM_BASE + offset * 4,
            vec![apply(selector, effect, 1)],
        ));
    }
    output.push(program_with_children(
        PROGRAM_BASE + 1,
        vec![PROGRAM_BASE + 2],
        vec![
            ProgramStep::Operation(RuleOperationTemplate::Cleanse {
                selector: PLAYERS,
                maximum: u16::MAX,
                order: EffectRemovalOrder::OldestFirst,
            }),
            ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                selector: PLAYERS,
                resource: RuleResourceKind::SkillPoints,
                update: ResourceUpdateKind::Gain,
                amount: scalar(i64::from(u16::MAX) * 1_000_000),
                scales_with_regeneration: false,
                rounding: Rounding::Floor,
            }),
            ProgramStep::ForEach {
                selector: PLAYERS,
                body: program_id(PROGRAM_BASE + 2),
                maximum: 16,
            },
        ],
    ));
    output.push(program(
        PROGRAM_BASE + 2,
        vec![ProgramStep::Operation(
            RuleOperationTemplate::ModifyResource {
                selector: CURRENT_PLAYER,
                resource: RuleResourceKind::Energy,
                update: ResourceUpdateKind::Set,
                amount: ValueExpr::QueryMaximumEnergy(StatQuerySubject::CurrentTarget),
                scales_with_regeneration: false,
                rounding: Rounding::Floor,
            },
        )],
    ));
    output.push(program(
        PROGRAM_BASE + 5,
        vec![ProgramStep::Operation(
            RuleOperationTemplate::ModifyResource {
                selector: ACTOR,
                resource: RuleResourceKind::Energy,
                update: ResourceUpdateKind::Gain,
                amount: scalar(1_000_000),
                scales_with_regeneration: false,
                rounding: Rounding::Floor,
            },
        )],
    ));
    output.push(program(
        PROGRAM_BASE + 13,
        vec![apply(ENEMIES, SHATTER_EFFECT, 1)],
    ));
    output.push(program(
        PROGRAM_BASE + 20,
        vec![apply(ACTOR, APOTHEOSIS_EFFECT, 1)],
    ));
    output.push(program(
        PROGRAM_BASE + 24,
        vec![apply(ACTOR, WHIRLWIND_EFFECT, 1)],
    ));
    output.push(program(
        PROGRAM_BASE + 33,
        vec![ProgramStep::Operation(
            RuleOperationTemplate::ModifyResource {
                selector: PLAYERS,
                resource: RuleResourceKind::Team(APOCALYPTIC_PUNCHLINE_KEY.into()),
                update: ResourceUpdateKind::Gain,
                amount: scalar(3_000_000),
                scales_with_regeneration: false,
                rounding: Rounding::Floor,
            },
        )],
    ));
    output
}

fn rules(source: &RuleSource) -> Vec<RuleDefinition> {
    (0..10)
        .map(|index| {
            let mut triggers = vec![start_trigger(
                TRIGGER_BASE + index * 4,
                PROGRAM_BASE + index * 4,
            )];
            match index {
                0 => triggers.push(boss_break_trigger(TRIGGER_BASE + 1, PROGRAM_BASE + 1)),
                1 => triggers.push(dot_trigger(TRIGGER_BASE + 5, PROGRAM_BASE + 5)),
                3 => triggers.push(break_trigger(TRIGGER_BASE + 13, PROGRAM_BASE + 13)),
                4 => {}
                5 => {
                    triggers.clear();
                    triggers.push(skill_point_spent_trigger(
                        TRIGGER_BASE + 20,
                        PROGRAM_BASE + 20,
                    ));
                }
                6 => {
                    triggers.clear();
                    triggers.push(dot_target_action_trigger(
                        TRIGGER_BASE + 24,
                        PROGRAM_BASE + 24,
                    ));
                }
                8 => triggers.push(boss_break_trigger(TRIGGER_BASE + 33, PROGRAM_BASE + 33)),
                _ => {}
            }
            let mut programs = triggers
                .iter()
                .map(|trigger| trigger.program)
                .collect::<Vec<_>>();
            if index == 0 {
                programs.push(program_id(PROGRAM_BASE + 2));
            }
            programs.sort_unstable();
            RuleDefinition::new(
                rule_id(RULE_BASE + index),
                programs,
                vec![
                    PLAYERS,
                    FIRST_PLAYER,
                    ENEMIES,
                    ACTOR,
                    EVENT_TARGET,
                    CURRENT_PLAYER,
                ],
            )
            .with_runtime(BattleRuleDefinition::new(
                source.clone(),
                Vec::new(),
                triggers,
                None,
            ))
        })
        .collect()
}

fn start_trigger(raw: u32, program: u32) -> TriggerDef {
    trigger(
        raw,
        RuleEventPoint::BattleStarted,
        EventFilter::default(),
        ConditionExpr::Literal(true),
        OnceScope::Battle,
        program,
    )
}

fn boss_break_trigger(raw: u32, program: u32) -> TriggerDef {
    trigger(
        raw,
        RuleEventPoint::WeaknessBroken,
        EventFilter {
            target_selector: Some(EVENT_TARGET),
            ..EventFilter::default()
        },
        ConditionExpr::EnemyRank(EVENT_TARGET, EnemyRank::Boss),
        OnceScope::Event,
        program,
    )
}

fn break_trigger(raw: u32, program: u32) -> TriggerDef {
    trigger(
        raw,
        RuleEventPoint::WeaknessBroken,
        EventFilter {
            target_selector: Some(EVENT_TARGET),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        OnceScope::Event,
        program,
    )
}

fn dot_trigger(raw: u32, program: u32) -> TriggerDef {
    trigger(
        raw,
        RuleEventPoint::DamageApplied,
        EventFilter {
            actor_selector: Some(ACTOR),
            damage_class: Some(RuleDamageClass::Dot),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        OnceScope::Event,
        program,
    )
}

fn skill_point_spent_trigger(raw: u32, program: u32) -> TriggerDef {
    trigger(
        raw,
        RuleEventPoint::ResourceChanged,
        EventFilter {
            actor_selector: Some(ACTOR),
            resource: Some(RuleResourceKind::SkillPoints),
            ..EventFilter::default()
        },
        ConditionExpr::Compare {
            lhs: Box::new(ValueExpr::ReadEventProperty(
                EventValueProperty::ResourceDelta,
            )),
            operator: Comparison::Less,
            rhs: Box::new(scalar(0)),
        },
        OnceScope::Event,
        program,
    )
}

fn dot_target_action_trigger(raw: u32, program: u32) -> TriggerDef {
    trigger(
        raw,
        RuleEventPoint::ActionResolved,
        EventFilter {
            actor_selector: Some(ACTOR),
            target_selector: Some(EVENT_TARGET),
            ..EventFilter::default()
        },
        ConditionExpr::Compare {
            lhs: Box::new(ValueExpr::SelectorSum {
                selector: EVENT_TARGET,
                value: Box::new(ValueExpr::QueryEffectCategoryStacks {
                    subject: StatQuerySubject::CurrentTarget,
                    category: EffectCategory::Dot,
                }),
            }),
            operator: Comparison::GreaterOrEqual,
            rhs: Box::new(integer(1)),
        },
        OnceScope::Action,
        program,
    )
}

fn trigger(
    raw: u32,
    point: RuleEventPoint,
    filter: EventFilter,
    condition: ConditionExpr,
    once_scope: OnceScope,
    program: u32,
) -> TriggerDef {
    TriggerDef {
        id: trigger_id(raw),
        event: point.kind(),
        event_point: point,
        phase: TriggerPhase::AfterEvent,
        filter,
        condition,
        once_scope,
        priority: ReactionPriority::new(0),
        program: program_id(program),
    }
}

fn program(raw: u32, steps: Vec<ProgramStep>) -> ProgramDefinition {
    program_with_children(raw, Vec::new(), steps)
}

fn program_with_children(
    raw: u32,
    children: Vec<u32>,
    steps: Vec<ProgramStep>,
) -> ProgramDefinition {
    ProgramDefinition::new(
        program_id(raw),
        children.into_iter().map(program_id).collect(),
        vec![
            PLAYERS,
            FIRST_PLAYER,
            ENEMIES,
            ACTOR,
            EVENT_TARGET,
            CURRENT_PLAYER,
        ],
        vec![
            EMBER_EFFECT,
            BLIGHTED_EFFECT,
            UNSTOPPABLE_EFFECT,
            SHATTER_EFFECT,
            LINEBREAKER_EFFECT,
            APOTHEOSIS_EFFECT,
            WHIRLWIND_EFFECT,
            KNOWLEDGE_EFFECT,
            OPPOSE_EFFECT,
            MOMENT_EFFECT,
        ],
        Vec::new(),
    )
    .with_steps(steps)
}

fn apply(selector: SelectorId, effect: EffectDefinitionId, stacks: i64) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector,
        effect,
        stacks: integer(stacks),
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })
}

fn units(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    ordering: RuleSelectorOrdering,
    maximum: u16,
    predicates: Vec<RuleSelectorPredicate>,
) -> RuleUnitSelector {
    RuleUnitSelector::new(
        origin,
        side,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        ordering,
        0,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::All,
        None,
        false,
    )
    .expect("Apocalyptic selector is valid")
    .with_predicates(predicates)
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

fn scalar(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(value)))
}
fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
}
const fn bundle(value: u32) -> RuleBundleId {
    RuleBundleId::new(value).expect("bundle id is non-zero")
}
const fn source(value: u32) -> SourceDefinitionId {
    SourceDefinitionId::new(value).expect("source id is non-zero")
}
const fn selector_id(value: u32) -> SelectorId {
    SelectorId::new(value).expect("selector id is non-zero")
}
const fn effect_id(value: u32) -> EffectDefinitionId {
    EffectDefinitionId::new(value).expect("effect id is non-zero")
}
const fn group_id(value: u32) -> ModifierStackingGroupId {
    ModifierStackingGroupId::new(value).expect("group id is non-zero")
}
const fn modifier_id(value: u32) -> ModifierDefinitionId {
    ModifierDefinitionId::new(value).expect("modifier id is non-zero")
}
const fn program_id(value: u32) -> ProgramId {
    ProgramId::new(value).expect("program id is non-zero")
}
const fn rule_id(value: u32) -> RuleId {
    RuleId::new(value).expect("rule id is non-zero")
}
const fn trigger_id(value: u32) -> TriggerId {
    TriggerId::new(value).expect("trigger id is non-zero")
}

#[cfg(test)]
mod tests {
    use super::{ApocalypticMechanicsDefinitions, released_axioms};

    #[test]
    fn active_definitions_cover_every_released_bundle() {
        let definitions = ApocalypticMechanicsDefinitions::active();
        assert_eq!(definitions.bundles.len(), 10);
        assert!(
            released_axioms()
                .into_iter()
                .all(|id| definitions.bundles.iter().any(|bundle| bundle.id() == id))
        );
    }
}
