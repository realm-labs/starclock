//! Exact settlement overflow through accepted commands, not Curio admission.
use super::{battle, catalog, empty_action, id, start_and_use};
use starclock_combat::{
    Battle, BattleEvent, BattleEventKind, Command, DecisionId, DispelCategory, DurationClock,
    EffectCategory, EffectDamageGuard, EffectRuntimeDefinition, EffectStackPolicy, EffectTickPhase,
    Scalar,
    catalog::{
        action::{AbilityProgramBinding, AbilityProgramTiming},
        builder::CombatCatalogBuilder,
        definition::{EffectDefinition, ProgramDefinition, RuleBundle, RuleDefinition},
    },
    formula::model::CombatElement,
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, EventValueProperty, OnceScope,
        ProgramStep, ReactionPriority, RuleEffectChancePolicy, RuleEventPoint,
        RuleOperationTemplate, RuleSource, RuleValue, SourceClass, TriggerDef, TriggerPhase,
        ValueExpr,
    },
};
use starclock_replay::battle_event::encode_battle_event_payload;

fn scalar(amount: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(
        Scalar::checked_from_integer(amount).unwrap(),
    ))
}
fn op(operation: RuleOperationTemplate) -> ProgramStep {
    ProgramStep::Operation(operation)
}
fn damage(amount: i64) -> ProgramStep {
    op(RuleOperationTemplate::TrueDamage {
        selector: id(2),
        amount: scalar(amount),
    })
}
fn apply() -> ProgramStep {
    op(RuleOperationTemplate::ApplyEffect {
        selector: id(2),
        effect: id(21),
        stacks: ValueExpr::Literal(RuleValue::Integer(1)),
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })
}
fn shield(amount: i64) -> ProgramStep {
    op(RuleOperationTemplate::Shield {
        selector: id(2),
        effect: id(21),
        amount: scalar(amount),
    })
}

fn fixture(steps: Vec<ProgramStep>, guard: Option<EffectDamageGuard>, missing: bool) -> Battle {
    let base = catalog(
        ProgramDefinition::new(id(1), vec![], vec![], vec![], vec![]),
        false,
        false,
        false,
        false,
    );
    let mut builder = CombatCatalogBuilder::from_catalog(&base, [0xa1; 32]);
    let runtime = EffectRuntimeDefinition::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .unwrap();
    builder.add_effect(
        EffectDefinition::new(id(21), vec![], vec![])
            .with_runtime(guard.map_or(runtime.clone(), |guard| runtime.with_damage_guard(guard))),
    );
    builder.add_program(
        ProgramDefinition::new(id(11), vec![], vec![id(2)], vec![id(21)], vec![]).with_steps(steps),
    );
    assert!(
        builder.replace_ability(
            base.ability(id(1))
                .unwrap()
                .clone()
                .with_action(empty_action())
                .with_programs(vec![
                    AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, id(11)).unwrap()
                ])
        )
    );
    let read = op(RuleOperationTemplate::EmitRuleEvent {
        code: 210,
        value: Some(ValueExpr::ReadEventProperty(
            EventValueProperty::DamageOverflow,
        )),
    });
    // Earlier ordered program operations may heal or damage the same target.
    // The observer reads its committed event, not the later HP or shield state.
    builder.add_program(
        ProgramDefinition::new(id(12), vec![], vec![], vec![], vec![]).with_steps(vec![read]),
    );
    let point = if missing {
        RuleEventPoint::ActionResolved
    } else {
        RuleEventPoint::DamageApplied
    };
    builder.add_rule(
        RuleDefinition::new(id(1), vec![id(12)], vec![]).with_runtime(BattleRuleDefinition::new(
            RuleSource::new(id(61), SourceClass::Synthetic, vec![], [0xa2; 32]),
            vec![],
            vec![TriggerDef {
                id: id(12),
                event: point.kind(),
                event_point: point,
                phase: TriggerPhase::AfterEvent,
                filter: EventFilter::default(),
                condition: ConditionExpr::Literal(true),
                once_scope: OnceScope::Event,
                priority: ReactionPriority::new(0),
                program: id(12),
            }],
            None,
        )),
    );
    builder.add_rule_bundle(RuleBundle::new(id(1), vec![id(1)]));
    battle(builder.build().unwrap(), false, true, false)
}

fn signals(events: &[BattleEvent]) -> Vec<i64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::RuleSignal(data) if data.code == 210 => match data.value {
                Some(RuleValue::Scalar(amount)) => Some(amount.scaled()),
                _ => panic!("overflow is a Scalar"),
            },
            _ => None,
        })
        .collect()
}

#[test]
fn damage_overflow_projects_hp_bound_shield_and_guard_settlements() {
    for (amount, shield_amount, guard, expected, applied) in [
        (0, 0, None, 0, 0),
        (999, 0, None, 0, 999),
        (1000, 0, None, 0, 1000),
        (1200, 0, None, 200, 1000),
        (1200, 200, None, 0, 1000),
        (1200, 199, None, 1, 1000),
        (1200, 1300, None, 0, 0),
        (1200, 200, Some(EffectDamageGuard::ShieldOverflowOnce), 0, 0),
        (1200, 0, Some(EffectDamageGuard::TeamDefeatOnce), 0, 999),
    ] {
        let mut battle = fixture(
            vec![apply(), shield(shield_amount), damage(amount)],
            guard,
            false,
        );
        let result = start_and_use(&mut battle).unwrap();
        assert!(result.fault().is_none());
        assert_eq!(signals(result.events()), vec![expected * 1_000_000]);
        let settled = result
            .events()
            .iter()
            .find_map(|event| match event.kind() {
                BattleEventKind::Damage(data) => Some(data),
                _ => None,
            })
            .unwrap();
        assert_eq!(settled.applied.get(), applied);
        if guard.is_some() {
            assert!(settled.calculated.get() < amount);
        }
    }
}

#[test]
fn damage_overflow_distinguishes_nonlethal_floor_from_hp_excess() {
    for (amount, expected) in [(999, 0), (1000, 0), (1200, 200)] {
        let mut battle = fixture(
            vec![op(RuleOperationTemplate::NonlethalTrueDamage {
                selector: id(2),
                amount: scalar(amount),
            })],
            None,
            false,
        );
        let result = start_and_use(&mut battle).unwrap();
        assert!(result.fault().is_none());
        assert_eq!(signals(result.events()), vec![expected * 1_000_000]);
        assert_eq!(
            battle
                .view()
                .units_by_id()
                .find(|u| u.formation().get() == 4)
                .unwrap()
                .current_hp()
                .get(),
            1
        );
    }
}

#[test]
fn damage_overflow_retains_each_event_after_healing_and_later_damage() {
    let steps = vec![
        damage(400),
        op(RuleOperationTemplate::Heal {
            selector: id(2),
            apply_formula_modifiers: false,
            amount: scalar(200),
        }),
        damage(900),
    ];
    let mut left = fixture(steps.clone(), None, false);
    let mut right = fixture(steps, None, false);
    let result = start_and_use(&mut left).unwrap();
    let fresh = start_and_use(&mut right).unwrap();
    assert_eq!(signals(result.events()), vec![0, 100_000_000]);
    let payloads = |events: &[BattleEvent]| {
        events
            .iter()
            .map(|e| encode_battle_event_payload(e).unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(payloads(result.events()), payloads(fresh.events()));
    assert_eq!(left.state_hash(), right.state_hash());
    assert_eq!(left.view().rng_draw_count(), right.view().rng_draw_count());
}

#[test]
fn damage_overflow_projects_break_damage_and_keeps_other_events_missing() {
    let mut battle = fixture(
        vec![
            apply(),
            shield(50),
            op(RuleOperationTemplate::Break {
                selector: id(2),
                element: CombatElement::Fire,
            }),
        ],
        None,
        false,
    );
    let result = start_and_use(&mut battle).unwrap();
    assert!(result.fault().is_none());
    let settled = result
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::BreakDamage(data) => Some(data),
            _ => None,
        })
        .expect("real Break damage was committed");
    let expected =
        (settled.calculated.get() - settled.absorbed.get() - settled.hp_before.get()).max(0);
    assert_eq!(signals(result.events()), vec![expected * 1_000_000]);
    let mut missing = fixture(vec![], None, true);
    let failed = start_and_use(&mut missing).unwrap();
    assert!(failed.fault().is_some());
    assert!(signals(failed.events()).is_empty());
}

#[test]
fn damage_overflow_noninteger_damage_floors_once_and_rejected_commands_stay_inert() {
    for (amount, expected) in [(1_000_999_999, 0), (1_001_000_001, 1)] {
        let steps = vec![op(RuleOperationTemplate::TrueDamage {
            selector: id(2),
            amount: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(amount))),
        })];
        let mut battle = fixture(steps, None, false);
        let result = start_and_use(&mut battle).unwrap();
        assert!(result.fault().is_none());
        let settled = result
            .events()
            .iter()
            .find_map(|event| match event.kind() {
                BattleEventKind::Damage(data) => Some(data),
                _ => None,
            })
            .unwrap();
        assert_eq!(signals(result.events()), vec![expected * 1_000_000]);
        assert_eq!(settled.calculated.get(), 1000 + expected);
        let before = battle.state_hash();
        let rng = battle.view().rng_draw_count();
        assert!(
            battle
                .apply(Command::StartBattle {
                    decision: DecisionId::new(1).unwrap()
                })
                .is_err()
        );
        assert_eq!(battle.state_hash(), before);
        assert_eq!(battle.view().rng_draw_count(), rng);
    }
}
