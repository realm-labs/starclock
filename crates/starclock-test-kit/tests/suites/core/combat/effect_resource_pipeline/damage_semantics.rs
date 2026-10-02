//! Real-command classification, formula, event-filter and selector regressions.
use super::{
    Contribution, Scenario, attack, battle, damages, definition, elation, fixture, hit, start,
};
use starclock_combat::{
    Battle, BattleEvent, BattleEventKind, DispelCategory, DurationClock, EffectCategory,
    EffectDamageGuard, EffectEventData, EffectRuntimeDefinition, EffectStackPolicy,
    EffectTickPhase, Ratio, Scalar, TeamSide,
    catalog::{
        CombatCatalog,
        action::{
            AbilityProgramBinding, AbilityProgramTiming, AbilityTag, HitCritPolicy,
            HitOperationDefinition, HitTargetGroup, OrdinaryDamageDefinition,
            OrdinaryDamageMultipliers,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EffectDefinition, ProgramDefinition, RuleDefinition,
            SelectorDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    damage::{DamageClassification, DamageSemantic, DamageSemantics},
    formula::model::{CombatElement, DamageClass},
    modifier::model::{FormulaStage, ModifierFilter, StatKind},
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleDamageClass, RuleEffectChancePolicy, RuleEventKind, RuleEventPoint,
        RuleOperationTemplate, RuleSource, RuleValue, SourceClass, TriggerDef, TriggerPhase,
        ValueExpr,
    },
};
use starclock_replay::battle_event::encode_battle_event_payload;
use std::sync::Arc;

fn ordinary(class: DamageClass) -> HitOperationDefinition {
    HitOperationDefinition::Damage(
        OrdinaryDamageDefinition::new(
            Scalar::from_scaled(100_000_000),
            OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
        )
        .unwrap()
        .with_class(class),
    )
}
fn scenario(policy: HitCritPolicy) -> Scenario {
    let mut value = Scenario::new(vec![hit(
        vec![
            ordinary(DamageClass::Direct),
            elation(100_000_000),
            ordinary(DamageClass::Additional),
        ],
        policy,
    )]);
    value.crit_rate = 1_000_000;
    let mut bonus = Contribution::stat(TeamSide::Player, StatKind::CritDamage, 1_500_000);
    bonus.filters = vec![ModifierFilter::DamageTag("follow_up".into())];
    value.contributions.push(bonus);
    value
}
fn runtime() -> EffectRuntimeDefinition {
    EffectRuntimeDefinition::new(
        EffectCategory::Buff,
        DispelCategory::DispellableBuff,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .unwrap()
}
fn labels() -> Vec<DamageClassification> {
    vec![DamageClassification {
        class: DamageClass::Elation,
        semantics: DamageSemantics::new(DamageSemantic::FollowUp),
    }]
}
fn selector(origin: RuleSelectorOrigin, side: RuleSelectorSide) -> RuleUnitSelector {
    RuleUnitSelector::new(
        origin,
        side,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StableId,
        0,
        2,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::All,
        None,
        false,
    )
    .unwrap()
}
fn apply(selector: u32, effect: u32) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector: definition(selector),
        effect: definition(effect),
        stacks: ValueExpr::Literal(RuleValue::Integer(1)),
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })
}
fn catalog(
    scenario: &Scenario,
    alias_holder: Option<u32>,
    duplicate: bool,
    follow_up: bool,
    tail: Vec<ProgramStep>,
) -> Arc<CombatCatalog> {
    catalog_with_guard(scenario, alias_holder, duplicate, follow_up, tail, false)
}
fn catalog_with_guard(
    scenario: &Scenario,
    alias_holder: Option<u32>,
    duplicate: bool,
    follow_up: bool,
    tail: Vec<ProgramStep>,
    guard: bool,
) -> Arc<CombatCatalog> {
    let base = fixture(scenario);
    let mut builder = CombatCatalogBuilder::from_catalog(&base, [0xb7; 32]);
    for (id, origin, side) in [
        (201, RuleSelectorOrigin::Owner, RuleSelectorSide::Same),
        (
            202,
            RuleSelectorOrigin::Encounter,
            RuleSelectorSide::Opposing,
        ),
    ] {
        builder.add_selector(
            SelectorDefinition::new(definition(id)).with_rule_units(selector(origin, side)),
        );
    }
    builder.add_selector(SelectorDefinition::new(definition(203)).with_rule_units(
        selector(RuleSelectorOrigin::EventTargets, RuleSelectorSide::Opposing).with_predicates(
            vec![RuleSelectorPredicate::EventDamageSemantic(
                DamageSemantic::FollowUp,
            )],
        ),
    ));
    for raw in [20, 21] {
        builder.add_effect(
            EffectDefinition::new(definition(raw), vec![], vec![])
                .with_runtime(if guard {
                    runtime().with_damage_guard(EffectDamageGuard::TeamDefeatOnce)
                } else {
                    runtime()
                })
                .with_damage_classifications(labels()),
        );
    }
    builder.add_effect(
        EffectDefinition::new(definition(22), vec![definition(200)], vec![])
            .with_runtime(runtime()),
    );
    let mut entry = vec![apply(201, 22)];
    if let Some(holder) = alias_holder {
        entry.push(apply(holder, 20));
        if duplicate {
            entry.push(apply(holder, 21));
            // Refresh the same definition, too: no classification multiplicity.
            entry.push(apply(holder, 20));
        }
    }
    for (id, steps) in [(301, entry), (302, tail)] {
        builder.add_program(
            ProgramDefinition::new(
                definition(id),
                vec![],
                vec![definition(201), definition(202), definition(203)],
                vec![definition(20), definition(21), definition(22)],
                vec![],
            )
            .with_steps(steps),
        );
    }
    let mut triggers = vec![];
    for (raw, filter) in [
        (
            401,
            EventFilter {
                damage_semantic: Some(DamageSemantic::FollowUp),
                damage_class: Some(RuleDamageClass::Elation),
                ability_tag: Some(AbilityTag::Basic),
                ..EventFilter::default()
            },
        ),
        (
            402,
            EventFilter {
                ability_tag: Some(AbilityTag::FollowUp),
                ..EventFilter::default()
            },
        ),
        (
            403,
            EventFilter {
                ..EventFilter::default()
            },
        ),
    ] {
        builder.add_program(
            ProgramDefinition::new(
                definition(raw),
                vec![],
                vec![definition(203)],
                vec![],
                vec![],
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::EmitRuleEvent {
                    code: raw,
                    value: Some(ValueExpr::SelectorCount(definition(203))),
                },
            )]),
        );
        triggers.push(TriggerDef {
            id: definition(raw),
            event: RuleEventKind::Damage,
            event_point: RuleEventPoint::DamageApplied,
            phase: TriggerPhase::AfterMutation,
            filter,
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Event,
            priority: ReactionPriority::new(0),
            program: definition(raw),
        });
    }
    builder.add_rule(
        RuleDefinition::new(
            definition(200),
            vec![definition(401), definition(402), definition(403)],
            vec![definition(203)],
        )
        .with_runtime(BattleRuleDefinition::new(
            RuleSource::new(definition(200), SourceClass::Synthetic, vec![], [0xb8; 32]),
            vec![],
            triggers,
            None,
        )),
    );
    let mut action = base
        .ability(definition(1))
        .unwrap()
        .action()
        .unwrap()
        .clone();
    if follow_up {
        action = action.with_tags(&[AbilityTag::Attack, AbilityTag::FollowUp]);
    }
    assert!(
        builder.replace_ability(
            AbilityDefinition::new(
                definition(1),
                definition(3),
                definition(3),
                vec![definition(10), definition(11), definition(12)],
            )
            .with_action(action)
            .with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, definition(301))
                    .unwrap(),
                AbilityProgramBinding::new(2, AbilityProgramTiming::AfterHits, definition(302))
                    .unwrap(),
            ])
        )
    );
    builder.build().unwrap()
}
fn execute(catalog: Arc<CombatCatalog>, scenario: &Scenario) -> (Battle, Vec<BattleEvent>) {
    let mut running = battle(Arc::clone(&catalog), scenario);
    let mut fresh = battle(catalog, scenario);
    start(&mut running, &mut fresh);
    let events = attack(&mut running, &mut fresh);
    (running, events)
}
fn signals(events: &[BattleEvent], code: u32) -> Vec<i64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::RuleSignal(value) if value.code == code => match &value.value {
                Some(RuleValue::Integer(value)) => Some(*value),
                other => panic!("unexpected signal: {other:?}"),
            },
            _ => None,
        })
        .collect()
}

#[test]
fn damage_semantics_aliases_qualify_only_the_holders_elation_not_the_action_or_other_damage() {
    let scenario = scenario(HitCritPolicy::Shared);
    let (battle, events) = execute(
        catalog(&scenario, Some(201), false, false, vec![]),
        &scenario,
    );
    let d = damages(&events);
    assert_eq!(
        d.iter().map(|d| d.calculated.get()).collect::<Vec<_>>(),
        [150, 300, 150]
    );
    assert_eq!(
        d.iter().map(|d| d.semantics.bits()).collect::<Vec<_>>(),
        [0, 1, 0]
    );
    assert_eq!(
        d.iter().map(|d| d.class).collect::<Vec<_>>(),
        [
            DamageClass::Direct,
            DamageClass::Elation,
            DamageClass::Additional
        ]
    );
    assert_eq!(signals(&events, 401), [1]);
    assert!(
        signals(&events, 402).is_empty(),
        "no FollowUp ability label was added"
    );
    assert_eq!(signals(&events, 403), [0, 1, 0]);
    assert_eq!(battle.view().rng_draw_count(), 0);
    for event in &events {
        if let BattleEventKind::Damage(value) = event.kind() {
            let encoded = encode_battle_event_payload(event).unwrap();
            // The current codec retains seven fixed-width values after the two
            // optional locators. The label byte is immediately before those.
            let tail = 7 * 8
                + 2
                + usize::from(value.element.is_some())
                + if value.source_effect.is_some() { 8 } else { 0 };
            assert_eq!(encoded[encoded.len() - tail - 1], value.semantics.bits());
        }
    }
}

#[test]
fn damage_semantics_duplicate_providers_union_and_native_follow_up_does_not_label_mixed_secondary_damage()
 {
    let scenario = scenario(HitCritPolicy::Shared);
    for (alias, duplicate, expected) in [
        (None, false, [300, 150, 150]),
        (Some(201), true, [300, 300, 150]),
    ] {
        let (_, events) = execute(
            catalog(&scenario, alias, duplicate, true, vec![]),
            &scenario,
        );
        assert_eq!(
            damages(&events)
                .iter()
                .map(|d| d.calculated.get())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            signals(&events, 402).len(),
            3,
            "action tags remain independent of per-damage labels"
        );
        assert_eq!(
            signals(&events, 403),
            if alias.is_some() {
                vec![1, 1, 0]
            } else {
                vec![1, 0, 0]
            }
        );
    }
}

#[test]
fn damage_semantics_victim_alias_cannot_qualify_an_unbuffed_producer_or_incoming_formula() {
    let mut scenario = scenario(HitCritPolicy::Shared);
    let mut incoming =
        Contribution::factor(TeamSide::Enemy, FormulaStage::Vulnerability, 1_000_000);
    incoming
        .filters
        .push(ModifierFilter::DamageTag("follow_up".into()));
    scenario.contributions.push(incoming);
    let (_, events) = execute(
        catalog(&scenario, Some(202), false, false, vec![]),
        &scenario,
    );
    assert_eq!(
        damages(&events)
            .iter()
            .map(|d| d.calculated.get())
            .collect::<Vec<_>>(),
        [150, 150, 150]
    );
    assert_eq!(signals(&events, 403), [0, 0, 0]);
    let (_, events) = execute(
        catalog(&scenario, Some(201), false, false, vec![]),
        &scenario,
    );
    assert_eq!(
        damages(&events)
            .iter()
            .map(|d| d.calculated.get())
            .collect::<Vec<_>>(),
        [150, 600, 150]
    );
}

#[test]
fn damage_semantics_never_crit_does_not_add_damage_and_shared_groups_requery_per_operation() {
    for policy in [
        HitCritPolicy::Never,
        HitCritPolicy::Shared,
        HitCritPolicy::PerTarget,
    ] {
        let mut scenario = scenario(policy);
        scenario.crit_rate = 500_000;
        let (battle, events) = execute(
            catalog(&scenario, Some(201), false, false, vec![]),
            &scenario,
        );
        let amounts = damages(&events)
            .iter()
            .map(|d| d.calculated.get())
            .collect::<Vec<_>>();
        assert!(matches!(amounts[0], 100 | 150));
        let expected = if amounts[0] == 150 {
            [150, 300, 150]
        } else {
            [100, 100, 100]
        };
        assert_eq!(amounts, expected);
        assert_eq!(
            battle.view().rng_draw_count(),
            u64::from(policy != HitCritPolicy::Never)
        );
    }
}

#[test]
fn damage_semantics_removal_requeries_labels_and_true_damage_does_not_inherit_follow_up() {
    let scenario = scenario(HitCritPolicy::Shared);
    let tail = vec![
        ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
            selector: definition(201),
            effect: definition(20),
        }),
        ProgramStep::Operation(RuleOperationTemplate::Damage {
            selector: definition(202),
            amount: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(100_000_000))),
            class: DamageClass::Elation,
            element: CombatElement::Fire,
            can_crit: false,
            can_defeat: true,
        }),
        ProgramStep::Operation(RuleOperationTemplate::TrueDamage {
            selector: definition(202),
            amount: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(100_000_000))),
        }),
    ];
    let (battle, events) = execute(catalog(&scenario, Some(201), false, true, tail), &scenario);
    assert_eq!(
        damages(&events)
            .iter()
            .map(|d| d.semantics.bits())
            .collect::<Vec<_>>(),
        [1, 1, 0, 0, 0]
    );
    assert_eq!(damages(&events)[3].calculated.get(), 100);
    assert_eq!(damages(&events).last().unwrap().calculated.get(), 100);
    assert!(
        battle
            .view()
            .effects_by_id()
            .all(|effect| effect.definition().get() != 20)
    );
    assert_eq!(signals(&events, 403), [1, 1, 0, 0, 0]);
}

#[test]
fn damage_semantics_event_retains_operation_entry_labels_when_a_damage_guard_consumes_the_provider()
{
    let mut scenario = scenario(HitCritPolicy::Shared);
    scenario.hits = vec![
        hit(vec![elation(300_000_000_000_000)], HitCritPolicy::Shared).with_profile(
            HitTargetGroup::SelfTarget,
            Ratio::ONE,
            Ratio::ONE,
            HitCritPolicy::Shared,
        ),
        hit(vec![elation(100_000_000)], HitCritPolicy::Shared),
    ];
    let (battle, events) = execute(
        catalog_with_guard(&scenario, Some(201), false, false, vec![], true),
        &scenario,
    );
    let d = damages(&events);
    assert_eq!(
        d[0].semantics.bits(),
        1,
        "do not requery after the guard removes its provider"
    );
    assert_eq!(d[0].raw.scaled(), 900_000_000_000_000);
    assert_eq!(d[0].hp_after.get(), 1);
    assert_eq!((d[1].semantics.bits(), d[1].calculated.get()), (0, 150));
    let removal = events.iter().position(|event| matches!(event.kind(), BattleEventKind::Effect(EffectEventData::Removed { definition, .. }) if definition.get() == 20)).unwrap();
    let damage = events
        .iter()
        .position(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
        .unwrap();
    assert!(removal < damage);
    assert!(
        battle
            .view()
            .effects_by_id()
            .all(|effect| effect.definition().get() != 20)
    );
    assert_eq!(
        signals(&events, 401),
        [0],
        "the event remains classified even after removal"
    );
}

#[test]
fn damage_semantics_catalog_rejects_identity_only_empty_duplicate_unsorted_and_dot_classifications()
{
    let scenario = scenario(HitCritPolicy::Never);
    let base = fixture(&scenario);
    for (runtime_required, entries) in [
        (false, labels()),
        (
            true,
            vec![DamageClassification {
                class: DamageClass::Elation,
                semantics: DamageSemantics::NONE,
            }],
        ),
        (true, vec![labels()[0], labels()[0]]),
        (
            true,
            vec![
                labels()[0],
                DamageClassification {
                    class: DamageClass::Direct,
                    semantics: labels()[0].semantics,
                },
            ],
        ),
        (
            true,
            vec![DamageClassification {
                class: DamageClass::Dot,
                semantics: labels()[0].semantics,
            }],
        ),
    ] {
        let mut builder = CombatCatalogBuilder::from_catalog(&base, [0xb9; 32]);
        let mut effect = EffectDefinition::new(definition(20), vec![], vec![])
            .with_damage_classifications(entries);
        if runtime_required {
            effect = effect.with_runtime(runtime());
        }
        builder.add_effect(effect);
        assert!(builder.build().is_err());
    }
    assert_eq!(
        DamageSemantics::from_bits(1),
        Some(DamageSemantics::new(DamageSemantic::FollowUp))
    );
    assert!(DamageSemantics::from_bits(2).is_none());
}

#[test]
fn damage_semantics_selector_on_a_non_damage_phase_is_empty_even_in_a_follow_up_action() {
    let scenario = scenario(HitCritPolicy::Shared);
    let tail = vec![ProgramStep::Operation(RuleOperationTemplate::Damage {
        selector: definition(203),
        amount: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(100_000_000))),
        class: DamageClass::Elation,
        element: CombatElement::Fire,
        can_crit: false,
        can_defeat: true,
    })];
    let (battle, events) = execute(catalog(&scenario, None, false, true, tail), &scenario);
    assert_eq!(
        damages(&events)
            .iter()
            .map(|damage| damage.calculated.get())
            .collect::<Vec<_>>(),
        [300, 150, 150]
    );
    assert_eq!(signals(&events, 403), [1, 0, 0]);
    assert_eq!(battle.view().rng_draw_count(), 0);
}
