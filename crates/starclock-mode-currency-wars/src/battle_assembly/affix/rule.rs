//! Dynamic enemy Affixes lowered to ordinary battle Rule IR.

mod equipment;
mod penalty;
mod permanent_trauma;
mod retaliation;
mod shutdown;
mod survival;
mod synchronization;
mod time_assassin;

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use starclock_combat::{
    ControlledAction, DispelCategory, DurationClock, EffectCategory, EffectDefinitionId,
    EffectRemovalOrder, EffectRuntimeTemplate, EffectStackPolicy, EffectTickPhase, ProgramId,
    ResolvedCombatantSpec, Rounding, RuleBundleId, RuleId, Scalar, SelectorId, SourceDefinitionId,
    StateSlotDefinitionId, TriggerId,
    catalog::{
        builder::CombatCatalogBuilder,
        definition::{
            EffectDefinition, ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
    rng::types::DrawPurpose,
    rule::model::{
        BattleRuleDefinition, BattleRuleScope, Comparison, ConditionExpr, EventFilter,
        EventValueProperty, OnceScope, ProgramStep, ReactionPriority, ResourceUpdateKind,
        RuleEffectChancePolicy, RuleEventPoint, RuleOperationTemplate, RuleResourceKind,
        RuleSource, RuleValue, RuleValueKind, SlotPersistence, SlotVisibility, SourceClass,
        StateSlotDef, TriggerDef, TriggerPhase, ValueExpr,
    },
};

use crate::{
    CurrencyWarsContributionSnapshot, CurrencyWarsEnemyAffixBehavior,
    CurrencyWarsEnemyAffixSemantic, CurrencyWarsRoleId,
    battle_assembly::{
        CurrencyWarsBattleAssemblyError, CurrencyWarsBattleResources,
        combatant_overlay::attach_rule_bundle, debug_error, error,
    },
};

const RULE_ID: u32 = 0x7d80_0001;
const BUNDLE_ID: u32 = 0x7d80_0002;
const SOURCE_ID: u32 = 0x7d80_0003;
const PLAYERS_ID: u32 = 0x7d80_0010;
const ENEMIES_ID: u32 = 0x7d80_0011;
const ACTOR_ID: u32 = 0x7d80_0012;
const EVENT_TARGET_ID: u32 = 0x7d80_0013;
const CURRENT_ID: u32 = 0x7d80_0014;
const BEHAVIOR_BASE: u32 = 0x7d81_0000;
const DEFINITIONS_PER_SOURCE: u32 = 32;

pub(crate) fn install_reactions(
    builder: &mut CombatCatalogBuilder,
    resources: &CurrencyWarsBattleResources,
    snapshot: &CurrencyWarsContributionSnapshot,
    combatants: &mut BTreeMap<CurrencyWarsRoleId, ResolvedCombatantSpec>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    equipment::tag_combatants(snapshot, combatants)?;
    shutdown::tag_combatants(resources, snapshot, combatants)?;
    let mut programs = Vec::new();
    let mut triggers = Vec::new();
    let mut slots = Vec::new();
    for behavior in &snapshot.enemy_affix_behaviors {
        compile_behavior(
            builder,
            resources,
            behavior,
            &mut programs,
            &mut triggers,
            &mut slots,
        )?;
    }
    if triggers.is_empty() {
        return Ok(());
    }
    let selectors = selectors(resources)?;
    for selector in selectors {
        builder.add_selector(selector);
    }
    let mut program_ids = programs
        .iter()
        .map(ProgramDefinition::id)
        .collect::<Vec<_>>();
    program_ids.sort_unstable();
    for program in programs {
        builder.add_program(program);
    }
    let digest = reaction_digest(snapshot.digest.bytes(), &snapshot.enemy_affix_behaviors);
    let source = RuleSource::new(source_id(SOURCE_ID)?, SourceClass::Mode, Vec::new(), digest);
    let rule_id = rule_id(RULE_ID)?;
    let bundle_id = bundle_id(BUNDLE_ID)?;
    builder.add_rule(
        RuleDefinition::new(rule_id, program_ids, selector_ids())
            .with_runtime(BattleRuleDefinition::new(source, slots, triggers, None)),
    );
    builder.add_rule_bundle(RuleBundle::new(bundle_id, vec![rule_id]));
    let host = combatants
        .keys()
        .next()
        .copied()
        .ok_or_else(|| error("Currency Wars enemy Affix reactions have no player host"))?;
    let combatant = combatants
        .get(&host)
        .ok_or_else(|| error("Currency Wars enemy Affix reaction host is missing"))?;
    let replacement = attach_rule_bundle(
        combatant,
        bundle_id,
        b"starclock.currency-wars.enemy-affix-reactions.v1",
        digest,
    )?;
    combatants.insert(host, replacement);
    Ok(())
}

fn compile_behavior(
    builder: &mut CombatCatalogBuilder,
    resources: &CurrencyWarsBattleResources,
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
    slots: &mut Vec<StateSlotDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    match behavior.semantic {
        CurrencyWarsEnemyAffixSemantic::HeavyFootfall => {
            let program = program_id_for(behavior, 1)?;
            programs.push(program_definition(
                program,
                Vec::new(),
                Vec::new(),
                vec![operation(RuleOperationTemplate::DelayAction {
                    selector: event_target(),
                    amount: scalar_parameter(behavior, 0)?,
                })],
            ));
            triggers.push(trigger(
                behavior,
                10,
                RuleEventPoint::DamageApplied,
                EventFilter {
                    actor_selector: Some(enemies()),
                    target_selector: Some(players()),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                OnceScope::TargetWithinAction,
                program,
            )?);
        }
        CurrencyWarsEnemyAffixSemantic::CarriedByInertia => {
            let program = program_id_for(behavior, 1)?;
            programs.push(program_definition(
                program,
                Vec::new(),
                Vec::new(),
                vec![operation(RuleOperationTemplate::AdvanceAction {
                    selector: enemies(),
                    amount: scalar_parameter(behavior, 0)?,
                })],
            ));
            triggers.push(trigger(
                behavior,
                10,
                RuleEventPoint::ActionResolved,
                EventFilter {
                    actor_selector: Some(enemies()),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                OnceScope::Battle,
                program,
            )?);
        }
        CurrencyWarsEnemyAffixSemantic::ExtraStrike => {
            equipment::compile_extra_strike(behavior, programs, triggers)?;
        }
        CurrencyWarsEnemyAffixSemantic::BlazingVengeance => {
            retaliation::compile_blazing_vengeance(builder, behavior, programs, triggers)?;
        }
        CurrencyWarsEnemyAffixSemantic::FrontendShutdown
        | CurrencyWarsEnemyAffixSemantic::BackendShutdown
        | CurrencyWarsEnemyAffixSemantic::CurbedWind
        | CurrencyWarsEnemyAffixSemantic::CurbedFire
        | CurrencyWarsEnemyAffixSemantic::CurbedIce
        | CurrencyWarsEnemyAffixSemantic::CurbedLightning
        | CurrencyWarsEnemyAffixSemantic::CurbedPhysical
        | CurrencyWarsEnemyAffixSemantic::CurbedQuantum
        | CurrencyWarsEnemyAffixSemantic::CurbedImaginary => {
            shutdown::compile(builder, behavior, programs, triggers)?;
        }
        CurrencyWarsEnemyAffixSemantic::BeyondEndurance => {
            compile_beyond_endurance(behavior, programs, triggers, slots)?;
        }
        CurrencyWarsEnemyAffixSemantic::SelfDefense => {
            retaliation::compile_self_defense(behavior, programs, triggers)?;
        }
        CurrencyWarsEnemyAffixSemantic::EnergyDisappearance => {
            let program = program_id_for(behavior, 1)?;
            programs.push(program_definition(
                program,
                Vec::new(),
                Vec::new(),
                vec![operation(RuleOperationTemplate::ModifyResource {
                    selector: actor(),
                    resource: RuleResourceKind::Energy,
                    update: ResourceUpdateKind::Spend,
                    amount: energy_disappearance_spend(behavior)?,
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                })],
            ));
            let current = ValueExpr::ReadResource {
                selector: actor(),
                resource: RuleResourceKind::Energy,
            };
            triggers.push(trigger(
                behavior,
                10,
                RuleEventPoint::DamageApplied,
                EventFilter {
                    actor_selector: Some(players()),
                    target_selector: Some(enemies()),
                    ..EventFilter::default()
                },
                ConditionExpr::Compare {
                    lhs: Box::new(current),
                    operator: Comparison::Less,
                    rhs: Box::new(ValueExpr::QueryMaximumEnergy(StatQuerySubject::Actor)),
                },
                OnceScope::TargetWithinAction,
                program,
            )?);
        }
        CurrencyWarsEnemyAffixSemantic::CriticalConundrum => {
            penalty::compile_critical_conundrum(builder, behavior, programs, triggers)?;
        }
        CurrencyWarsEnemyAffixSemantic::MagmaBombardment => {
            penalty::compile_magma_bombardment(builder, behavior, programs, triggers)?;
        }
        CurrencyWarsEnemyAffixSemantic::SynchronizedAction => {
            synchronization::compile(behavior, programs, triggers)?;
        }
        CurrencyWarsEnemyAffixSemantic::PermanentTrauma => {
            permanent_trauma::compile(behavior, programs, triggers)?;
        }
        CurrencyWarsEnemyAffixSemantic::CryogenicHibernation => {
            compile_cryogenic_hibernation(builder, behavior, programs, triggers)?;
        }
        CurrencyWarsEnemyAffixSemantic::GetOutOfJailFreeCard => {
            survival::compile(builder, behavior, programs, triggers)?;
        }
        CurrencyWarsEnemyAffixSemantic::RapidCooling => {
            compile_rapid_cooling(builder, behavior, programs, triggers, slots)?;
        }
        CurrencyWarsEnemyAffixSemantic::PurityOfFleshAndMind => {
            compile_purity_of_flesh_and_mind(builder, behavior, programs, triggers)?;
        }
        CurrencyWarsEnemyAffixSemantic::FightOrFlightResponse => {
            compile_fight_or_flight(builder, behavior, programs, triggers)?;
        }
        CurrencyWarsEnemyAffixSemantic::ItsATrap => {
            compile_its_a_trap(builder, behavior, programs, triggers)?;
        }
        CurrencyWarsEnemyAffixSemantic::EmergencyHemostasis => {
            compile_emergency_hemostasis(builder, behavior, programs, triggers)?;
        }
        CurrencyWarsEnemyAffixSemantic::TimeAssassin => {
            time_assassin::compile(resources, behavior, programs, triggers)?;
        }
        _ => {}
    }
    Ok(())
}

fn compile_beyond_endurance(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
    slots: &mut Vec<StateSlotDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    let required = integer_parameter(behavior, 0)?;
    if required == 0 {
        return Err(error("Currency Wars Beyond Endurance count is zero"));
    }
    let slot = slot_id_for(behavior, 20)?;
    slots.push(
        StateSlotDef::new(
            slot,
            RuleValueKind::Integer,
            BattleRuleScope::Battle,
            RuleValue::Integer(0),
        )
        .with_bounds(
            RuleValue::Integer(0),
            RuleValue::Integer(i64::from(required.saturating_sub(1))),
        )
        .with_policy(SlotVisibility::Public, SlotPersistence::OwnerLifetime),
    );
    let main = program_id_for(behavior, 1)?;
    let advance = program_id_for(behavior, 2)?;
    let increment = program_id_for(behavior, 3)?;
    programs.push(program_definition(
        main,
        vec![advance, increment],
        Vec::new(),
        vec![ProgramStep::If {
            condition: ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::Slot(slot)),
                operator: Comparison::GreaterOrEqual,
                rhs: Box::new(integer_value(required.saturating_sub(1))),
            },
            then_program: advance,
            else_program: Some(increment),
        }],
    ));
    programs.push(program_definition(
        advance,
        Vec::new(),
        Vec::new(),
        vec![
            operation(RuleOperationTemplate::AdvanceAction {
                selector: enemies(),
                amount: scalar_parameter(behavior, 1)?,
            }),
            operation(RuleOperationTemplate::SetSlot {
                slot,
                value: integer_value(0),
            }),
        ],
    ));
    programs.push(program_definition(
        increment,
        Vec::new(),
        Vec::new(),
        vec![operation(RuleOperationTemplate::AddSlot {
            slot,
            value: integer_value(1),
        })],
    ));
    triggers.push(trigger(
        behavior,
        10,
        RuleEventPoint::DamageApplied,
        EventFilter {
            target_selector: Some(enemies()),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        OnceScope::TargetWithinAction,
        main,
    )?);
    Ok(())
}

fn compile_cryogenic_hibernation(
    builder: &mut CombatCatalogBuilder,
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    let effect = effect_id_for(behavior, 20)?;
    builder.add_effect(control_effect(effect, integer_parameter(behavior, 0)?)?);
    let program = program_id_for(behavior, 1)?;
    programs.push(program_definition(
        program,
        Vec::new(),
        vec![effect],
        vec![apply_effect(
            event_target(),
            effect,
            RuleEffectChancePolicy::Guaranteed,
            None,
        )],
    ));
    triggers.push(trigger(
        behavior,
        10,
        RuleEventPoint::LethalRescued,
        EventFilter {
            target_selector: Some(players()),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        OnceScope::Event,
        program,
    )?);
    Ok(())
}

fn compile_rapid_cooling(
    builder: &mut CombatCatalogBuilder,
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
    slots: &mut Vec<StateSlotDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    let slot = slot_id_for(behavior, 20)?;
    slots.push(
        StateSlotDef::new(
            slot,
            RuleValueKind::Integer,
            BattleRuleScope::Turn,
            RuleValue::Integer(0),
        )
        .with_bounds(RuleValue::Integer(0), RuleValue::Integer(64))
        .with_policy(SlotVisibility::Public, SlotPersistence::ScopeLifetime),
    );
    let effect = effect_id_for(behavior, 21)?;
    builder.add_effect(control_effect(effect, integer_parameter(behavior, 1)?)?);
    let record = program_id_for(behavior, 1)?;
    let apply = program_id_for(behavior, 2)?;
    programs.push(program_definition(
        record,
        Vec::new(),
        Vec::new(),
        vec![operation(RuleOperationTemplate::AddSlot {
            slot,
            value: ValueExpr::Convert {
                value: Box::new(ValueExpr::Negate(Box::new(ValueExpr::ReadEventProperty(
                    EventValueProperty::ResourceDelta,
                )))),
                target: RuleValueKind::Integer,
                rounding: Rounding::NearestTiesAway,
            },
        })],
    ));
    programs.push(program_definition(
        apply,
        Vec::new(),
        vec![effect],
        vec![operation(RuleOperationTemplate::RandomGroupedEffect {
            selector: enemies(),
            effect,
            groups: ValueExpr::Slot(slot),
            applications_per_group: 1,
            stacks: integer_value(1),
            choice_rng_purpose: DrawPurpose::new(0x4701)
                .expect("Currency Wars effect target purpose is non-zero"),
            chance: RuleEffectChancePolicy::Fixed,
            base_chance: Some(scalar_parameter(behavior, 0)?),
            chance_rng_purpose: Some(
                DrawPurpose::new(0x4702).expect("Currency Wars effect chance purpose is non-zero"),
            ),
        })],
    ));
    triggers.push(trigger(
        behavior,
        10,
        RuleEventPoint::ResourceChanged,
        EventFilter {
            actor_selector: Some(players()),
            resource: Some(RuleResourceKind::SkillPoints),
            ..EventFilter::default()
        },
        ConditionExpr::Compare {
            lhs: Box::new(ValueExpr::ReadEventProperty(
                EventValueProperty::ResourceDelta,
            )),
            operator: Comparison::Less,
            rhs: Box::new(scalar_value(0)),
        },
        OnceScope::Event,
        record,
    )?);
    triggers.push(trigger(
        behavior,
        11,
        RuleEventPoint::TurnEnded,
        EventFilter {
            actor_selector: Some(players()),
            ..EventFilter::default()
        },
        ConditionExpr::Compare {
            lhs: Box::new(ValueExpr::Slot(slot)),
            operator: Comparison::Greater,
            rhs: Box::new(integer_value(0)),
        },
        OnceScope::Turn,
        apply,
    )?);
    Ok(())
}

fn compile_purity_of_flesh_and_mind(
    builder: &mut CombatCatalogBuilder,
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    let marker = effect_id_for(behavior, 20)?;
    builder.add_effect(marker_effect(marker, None)?);
    let program = program_id_for(behavior, 1)?;
    let maximum_hp = ValueExpr::QueryStat {
        subject: StatQuerySubject::EventTarget,
        stat: StatKind::Hp,
        purpose: FormulaPurpose::Stat,
    };
    let negative_stacks = [
        EffectCategory::Debuff,
        EffectCategory::Control,
        EffectCategory::Dot,
    ]
    .into_iter()
    .map(|category| ValueExpr::QueryEffectCategoryStacks {
        subject: StatQuerySubject::EventTarget,
        category,
    })
    .reduce(|left, right| ValueExpr::Add(Box::new(left), Box::new(right)))
    .expect("three negative effect categories are non-empty");
    let per_debuff = ValueExpr::Multiply {
        lhs: Box::new(maximum_hp.clone()),
        rhs: Box::new(ValueExpr::Multiply {
            lhs: Box::new(scalar_parameter(behavior, 2)?),
            rhs: Box::new(ValueExpr::Convert {
                value: Box::new(negative_stacks),
                target: RuleValueKind::Scalar,
                rounding: Rounding::NearestTiesAway,
            }),
            rounding: Rounding::NearestTiesAway,
        }),
        rounding: Rounding::NearestTiesAway,
    };
    let base_heal = ValueExpr::Multiply {
        lhs: Box::new(maximum_hp),
        rhs: Box::new(scalar_parameter(behavior, 1)?),
        rounding: Rounding::NearestTiesAway,
    };
    programs.push(program_definition(
        program,
        Vec::new(),
        vec![marker],
        vec![
            apply_effect(
                event_target(),
                marker,
                RuleEffectChancePolicy::Guaranteed,
                None,
            ),
            operation(RuleOperationTemplate::Cleanse {
                selector: event_target(),
                maximum: u16::MAX,
                order: EffectRemovalOrder::OldestFirst,
            }),
            operation(RuleOperationTemplate::Heal {
                selector: event_target(),
                amount: ValueExpr::Add(Box::new(base_heal), Box::new(per_debuff)),
                apply_formula_modifiers: false,
            }),
        ],
    ));
    let threshold = maximum_hp_ratio(StatQuerySubject::EventTarget, behavior, 0)?;
    let condition = ConditionExpr::All(
        vec![
            compare_event(
                EventValueProperty::HpBefore,
                Comparison::GreaterOrEqual,
                threshold.clone(),
            ),
            compare_event(EventValueProperty::HpAfter, Comparison::Less, threshold),
            ConditionExpr::Not(Box::new(ConditionExpr::EffectExists {
                selector: event_target(),
                effect: marker,
            })),
        ]
        .into_boxed_slice(),
    );
    for (offset, point) in [
        (10, RuleEventPoint::DamageApplied),
        (11, RuleEventPoint::HpChanged),
    ] {
        triggers.push(trigger(
            behavior,
            offset,
            point,
            EventFilter {
                target_selector: Some(enemies()),
                ..EventFilter::default()
            },
            condition.clone(),
            OnceScope::Event,
            program,
        )?);
    }
    Ok(())
}

fn compile_fight_or_flight(
    builder: &mut CombatCatalogBuilder,
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    let marker = effect_id_for(behavior, 20)?;
    builder.add_effect(marker_effect(marker, None)?);
    let program = program_id_for(behavior, 1)?;
    programs.push(program_definition(
        program,
        Vec::new(),
        vec![marker],
        vec![
            apply_effect(
                event_target(),
                marker,
                RuleEffectChancePolicy::Guaranteed,
                None,
            ),
            operation(RuleOperationTemplate::AdvanceAction {
                selector: event_target(),
                amount: scalar_parameter(behavior, 1)?,
            }),
        ],
    ));
    let threshold = maximum_hp_ratio(StatQuerySubject::EventTarget, behavior, 0)?;
    triggers.push(trigger(
        behavior,
        10,
        RuleEventPoint::DamageApplied,
        EventFilter {
            target_selector: Some(enemies()),
            ..EventFilter::default()
        },
        ConditionExpr::All(
            vec![
                compare_event(
                    EventValueProperty::HpBefore,
                    Comparison::GreaterOrEqual,
                    threshold.clone(),
                ),
                compare_event(EventValueProperty::HpAfter, Comparison::Less, threshold),
                ConditionExpr::Not(Box::new(ConditionExpr::EffectExists {
                    selector: event_target(),
                    effect: marker,
                })),
            ]
            .into_boxed_slice(),
        ),
        OnceScope::Event,
        program,
    )?);
    Ok(())
}

fn compile_its_a_trap(
    builder: &mut CombatCatalogBuilder,
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    let imprisonment = effect_id_for(behavior, 20)?;
    let entanglement = effect_id_for(behavior, 21)?;
    builder.add_effect(control_effect(
        imprisonment,
        integer_parameter(behavior, 2)?,
    )?);
    builder.add_effect(control_effect(
        entanglement,
        integer_parameter(behavior, 2)?,
    )?);
    let apply = program_id_for(behavior, 1)?;
    let delay = program_id_for(behavior, 2)?;
    programs.push(program_definition(
        apply,
        Vec::new(),
        vec![imprisonment, entanglement],
        vec![operation(RuleOperationTemplate::ApplyRandomEffect {
            selector: players(),
            effects: Box::new([imprisonment, entanglement]),
            stacks: integer_value(1),
            choice_rng_purpose: DrawPurpose::new(0x4616)
                .expect("Currency Wars trap choice purpose is non-zero"),
            chance: RuleEffectChancePolicy::Resistible,
            base_chance: Some(scalar_parameter(behavior, 0)?),
            chance_rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
        })],
    ));
    programs.push(program_definition(
        delay,
        Vec::new(),
        Vec::new(),
        vec![operation(RuleOperationTemplate::DelayAction {
            selector: event_target(),
            amount: scalar_parameter(behavior, 1)?,
        })],
    ));
    triggers.push(trigger(
        behavior,
        10,
        RuleEventPoint::BattleStarted,
        EventFilter::default(),
        ConditionExpr::Literal(true),
        OnceScope::Battle,
        apply,
    )?);
    for (offset, effect) in [(11, imprisonment), (12, entanglement)] {
        triggers.push(trigger(
            behavior,
            offset,
            RuleEventPoint::EffectApplied,
            EventFilter {
                effect_definition: Some(effect),
                target_selector: Some(players()),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            OnceScope::Event,
            delay,
        )?);
    }
    Ok(())
}

fn compile_emergency_hemostasis(
    builder: &mut CombatCatalogBuilder,
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    let marker = effect_id_for(behavior, 20)?;
    builder.add_effect(marker_effect(
        marker,
        Some(integer_parameter(behavior, 2)?),
    )?);
    let start = program_id_for(behavior, 1)?;
    let heal = program_id_for(behavior, 2)?;
    let consume = program_id_for(behavior, 3)?;
    programs.push(program_definition(
        start,
        vec![consume],
        vec![marker],
        vec![
            ProgramStep::ForEach {
                selector: enemies(),
                body: consume,
                maximum: 32,
            },
            apply_effect(enemies(), marker, RuleEffectChancePolicy::Guaranteed, None),
        ],
    ));
    programs.push(program_definition(
        consume,
        Vec::new(),
        Vec::new(),
        vec![operation(RuleOperationTemplate::ConsumeHp {
            selector: current(),
            amount: maximum_hp_ratio(StatQuerySubject::CurrentTarget, behavior, 0)?,
            floor: scalar_value(1),
        })],
    ));
    programs.push(program_definition(
        heal,
        Vec::new(),
        Vec::new(),
        vec![operation(RuleOperationTemplate::Heal {
            selector: actor(),
            amount: maximum_hp_ratio(StatQuerySubject::Actor, behavior, 1)?,
            apply_formula_modifiers: false,
        })],
    ));
    triggers.push(trigger(
        behavior,
        10,
        RuleEventPoint::BattleStarted,
        EventFilter::default(),
        ConditionExpr::Literal(true),
        OnceScope::Battle,
        start,
    )?);
    triggers.push(trigger(
        behavior,
        11,
        RuleEventPoint::ActionResolved,
        EventFilter {
            actor_selector: Some(enemies()),
            ..EventFilter::default()
        },
        ConditionExpr::EffectExists {
            selector: actor(),
            effect: marker,
        },
        OnceScope::Action,
        heal,
    )?);
    triggers.push(trigger(
        behavior,
        12,
        RuleEventPoint::WaveStarted,
        EventFilter::default(),
        ConditionExpr::Literal(true),
        OnceScope::Wave,
        start,
    )?);
    Ok(())
}

fn selectors(
    resources: &CurrencyWarsBattleResources,
) -> Result<Vec<SelectorDefinition>, CurrencyWarsBattleAssemblyError> {
    let mut selectors = vec![
        selector(
            players(),
            RuleSelectorOrigin::Owner,
            RuleSelectorSide::Same,
            32,
        )?,
        selector(
            enemies(),
            RuleSelectorOrigin::Owner,
            RuleSelectorSide::Opposing,
            32,
        )?,
        selector(actor(), RuleSelectorOrigin::Actor, RuleSelectorSide::Any, 1)?,
        selector(
            event_target(),
            RuleSelectorOrigin::PrimaryTarget,
            RuleSelectorSide::Any,
            1,
        )?,
        selector(
            current(),
            RuleSelectorOrigin::CurrentSubject,
            RuleSelectorSide::Any,
            1,
        )?,
    ];
    selectors.extend(equipment::selectors()?);
    selectors.extend(penalty::selectors()?);
    selectors.extend(retaliation::selectors()?);
    selectors.extend(shutdown::selectors()?);
    selectors.extend(synchronization::selectors()?);
    selectors.extend(time_assassin::selectors(resources)?);
    Ok(selectors)
}

fn selector(
    id: SelectorId,
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    maximum: u16,
) -> Result<SelectorDefinition, CurrencyWarsBattleAssemblyError> {
    let units = RuleUnitSelector::new(
        origin,
        side,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        0,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::All,
        None,
        false,
    )
    .ok_or_else(|| error("Currency Wars enemy Affix selector is invalid"))?;
    Ok(SelectorDefinition::new(id).with_rule_units(units))
}

fn program_definition(
    id: ProgramId,
    children: Vec<ProgramId>,
    effects: Vec<EffectDefinitionId>,
    steps: Vec<ProgramStep>,
) -> ProgramDefinition {
    ProgramDefinition::new(id, children, selector_ids(), effects, Vec::new()).with_steps(steps)
}

fn selector_ids() -> Vec<SelectorId> {
    let mut selectors = vec![players(), enemies(), actor(), event_target(), current()];
    selectors.extend(equipment::selector_ids());
    selectors.extend(penalty::selector_ids());
    selectors.extend(retaliation::selector_ids());
    selectors.extend(shutdown::selector_ids());
    selectors.extend(synchronization::selector_ids());
    selectors.extend(time_assassin::selector_ids());
    selectors.sort_unstable();
    selectors
}

#[allow(clippy::too_many_arguments)]
fn trigger(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    offset: u32,
    point: RuleEventPoint,
    filter: EventFilter,
    condition: ConditionExpr,
    once_scope: OnceScope,
    program: ProgramId,
) -> Result<TriggerDef, CurrencyWarsBattleAssemblyError> {
    Ok(TriggerDef {
        id: trigger_id_for(behavior, offset)?,
        event: point.kind(),
        event_point: point,
        phase: TriggerPhase::AfterEvent,
        filter,
        condition,
        once_scope,
        priority: ReactionPriority::new(0),
        program,
    })
}

fn operation(value: RuleOperationTemplate) -> ProgramStep {
    ProgramStep::Operation(value)
}

fn apply_effect(
    selector: SelectorId,
    effect: EffectDefinitionId,
    chance: RuleEffectChancePolicy,
    base_chance: Option<ValueExpr>,
) -> ProgramStep {
    operation(RuleOperationTemplate::ApplyEffect {
        selector,
        effect,
        stacks: integer_value(1),
        chance,
        base_chance,
        rng_purpose: (chance != RuleEffectChancePolicy::Guaranteed)
            .then_some(DrawPurpose::EFFECT_CHANCE),
    })
}

fn control_effect(
    id: EffectDefinitionId,
    duration: u32,
) -> Result<EffectDefinition, CurrencyWarsBattleAssemblyError> {
    let duration = u16::try_from(duration).map_err(debug_error)?;
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Control,
        DispelCategory::CleanseableControl,
        1,
        Some(integer_value(u32::from(duration))),
        DurationClock::TargetTurnStart,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .and_then(|runtime| runtime.with_control(controlled_actions()))
    .ok_or_else(|| error("Currency Wars enemy Affix control effect is invalid"))?;
    Ok(EffectDefinition::new(id, Vec::new(), Vec::new()).with_runtime_template(runtime))
}

fn marker_effect(
    id: EffectDefinitionId,
    duration_actions: Option<u32>,
) -> Result<EffectDefinition, CurrencyWarsBattleAssemblyError> {
    let (duration, clock) = match duration_actions {
        Some(duration) => (
            Some(integer_value(duration)),
            DurationClock::TargetActionEnd,
        ),
        None => (None, DurationClock::Permanent),
    };
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::NeutralState,
        DispelCategory::NonDispellable,
        1,
        duration,
        clock,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .ok_or_else(|| error("Currency Wars enemy Affix marker effect is invalid"))?;
    Ok(EffectDefinition::new(id, Vec::new(), Vec::new()).with_runtime_template(runtime))
}

fn controlled_actions() -> Vec<ControlledAction> {
    vec![
        ControlledAction::NormalAction,
        ControlledAction::Ultimate,
        ControlledAction::FollowUp,
        ControlledAction::Counter,
        ControlledAction::SummonAction,
    ]
}

fn maximum_hp_ratio(
    subject: StatQuerySubject,
    behavior: &CurrencyWarsEnemyAffixBehavior,
    parameter: usize,
) -> Result<ValueExpr, CurrencyWarsBattleAssemblyError> {
    Ok(ValueExpr::Multiply {
        lhs: Box::new(ValueExpr::QueryStat {
            subject,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        }),
        rhs: Box::new(scalar_parameter(behavior, parameter)?),
        rounding: Rounding::NearestTiesAway,
    })
}

fn compare_event(
    property: EventValueProperty,
    operator: Comparison,
    rhs: ValueExpr,
) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(property)),
        operator,
        rhs: Box::new(rhs),
    }
}

fn scalar_parameter(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    index: usize,
) -> Result<ValueExpr, CurrencyWarsBattleAssemblyError> {
    behavior
        .parameters
        .get(index)
        .copied()
        .map(|value| ValueExpr::Literal(RuleValue::Scalar(value)))
        .ok_or_else(|| error("Currency Wars enemy Affix rule parameter is missing"))
}

fn energy_disappearance_spend(
    behavior: &CurrencyWarsEnemyAffixBehavior,
) -> Result<ValueExpr, CurrencyWarsBattleAssemblyError> {
    Ok(ValueExpr::Minimum(
        Box::new(ValueExpr::ReadResource {
            selector: actor(),
            resource: RuleResourceKind::Energy,
        }),
        Box::new(scalar_parameter(behavior, 0)?),
    ))
}

fn integer_parameter(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    index: usize,
) -> Result<u32, CurrencyWarsBattleAssemblyError> {
    let scaled = behavior
        .parameters
        .get(index)
        .ok_or_else(|| error("Currency Wars enemy Affix integer parameter is missing"))?
        .scaled();
    if scaled < 0 || scaled % 1_000_000 != 0 {
        return Err(error(
            "Currency Wars enemy Affix integer parameter is invalid",
        ));
    }
    u32::try_from(scaled / 1_000_000).map_err(debug_error)
}

fn integer_value(value: u32) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(i64::from(value)))
}

fn scalar_value(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(
        Scalar::checked_from_integer(value).expect("small Currency Wars scalar is valid"),
    ))
}

fn definition_raw(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    offset: u32,
) -> Result<u32, CurrencyWarsBattleAssemblyError> {
    BEHAVIOR_BASE
        .checked_add(
            behavior
                .source_id
                .checked_mul(DEFINITIONS_PER_SOURCE)
                .ok_or_else(|| error("Currency Wars enemy Affix rule ID overflow"))?,
        )
        .and_then(|base| base.checked_add(offset))
        .ok_or_else(|| error("Currency Wars enemy Affix rule ID overflow"))
}

fn program_id_for(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    offset: u32,
) -> Result<ProgramId, CurrencyWarsBattleAssemblyError> {
    ProgramId::new(definition_raw(behavior, offset)?)
        .ok_or_else(|| error("Currency Wars enemy Affix program ID is invalid"))
}

fn trigger_id_for(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    offset: u32,
) -> Result<TriggerId, CurrencyWarsBattleAssemblyError> {
    TriggerId::new(definition_raw(behavior, offset)?)
        .ok_or_else(|| error("Currency Wars enemy Affix trigger ID is invalid"))
}

fn slot_id_for(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    offset: u32,
) -> Result<StateSlotDefinitionId, CurrencyWarsBattleAssemblyError> {
    StateSlotDefinitionId::new(definition_raw(behavior, offset)?)
        .ok_or_else(|| error("Currency Wars enemy Affix slot ID is invalid"))
}

fn effect_id_for(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    offset: u32,
) -> Result<EffectDefinitionId, CurrencyWarsBattleAssemblyError> {
    EffectDefinitionId::new(definition_raw(behavior, offset)?)
        .ok_or_else(|| error("Currency Wars enemy Affix effect ID is invalid"))
}

fn reaction_digest(root: [u8; 32], behaviors: &[CurrencyWarsEnemyAffixBehavior]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"starclock.currency-wars.enemy-affix-reaction-rule.v1");
    hash.update(root);
    for behavior in behaviors {
        hash.update(behavior.source_id.to_le_bytes());
    }
    hash.finalize().into()
}

fn players() -> SelectorId {
    SelectorId::new(PLAYERS_ID).expect("reserved selector ID is non-zero")
}

fn enemies() -> SelectorId {
    SelectorId::new(ENEMIES_ID).expect("reserved selector ID is non-zero")
}

fn actor() -> SelectorId {
    SelectorId::new(ACTOR_ID).expect("reserved selector ID is non-zero")
}

fn event_target() -> SelectorId {
    SelectorId::new(EVENT_TARGET_ID).expect("reserved selector ID is non-zero")
}

fn current() -> SelectorId {
    SelectorId::new(CURRENT_ID).expect("reserved selector ID is non-zero")
}

fn rule_id(raw: u32) -> Result<RuleId, CurrencyWarsBattleAssemblyError> {
    RuleId::new(raw).ok_or_else(|| error("Currency Wars enemy Affix rule ID is invalid"))
}

fn bundle_id(raw: u32) -> Result<RuleBundleId, CurrencyWarsBattleAssemblyError> {
    RuleBundleId::new(raw).ok_or_else(|| error("Currency Wars enemy Affix bundle ID is invalid"))
}

fn source_id(raw: u32) -> Result<SourceDefinitionId, CurrencyWarsBattleAssemblyError> {
    SourceDefinitionId::new(raw)
        .ok_or_else(|| error("Currency Wars enemy Affix source ID is invalid"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn energy_disappearance_spends_no_more_than_current_energy() {
        let amount = Scalar::checked_from_integer(4).unwrap();
        let behavior = CurrencyWarsEnemyAffixBehavior {
            source_id: 4003,
            semantic: CurrencyWarsEnemyAffixSemantic::EnergyDisappearance,
            maze_buff_ids: Box::new([35_304_006]),
            parameters: Box::new([amount]),
        };

        assert_eq!(
            energy_disappearance_spend(&behavior).unwrap(),
            ValueExpr::Minimum(
                Box::new(ValueExpr::ReadResource {
                    selector: actor(),
                    resource: RuleResourceKind::Energy,
                }),
                Box::new(ValueExpr::Literal(RuleValue::Scalar(amount))),
            ),
        );
    }
}
