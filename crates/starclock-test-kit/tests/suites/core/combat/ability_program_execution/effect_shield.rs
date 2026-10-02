//! Shared native capability probes, not admission or parity evidence for any Curio.

use super::{battle, catalog, empty_action, id, start_and_use};
use starclock_combat::{
    Battle, BattleEvent, BattleEventKind, Command, DispelCategory, DurationClock, EffectCategory,
    EffectRuntimeDefinition, EffectStackPolicy, EffectTickPhase, Rounding, Scalar, ShieldEventData,
    TeamSide,
    catalog::{
        action::{AbilityProgramBinding, AbilityProgramTiming},
        builder::CombatCatalogBuilder,
        definition::{EffectDefinition, ProgramDefinition, RuleBundle, RuleDefinition},
    },
    formula::shield::{ShieldAbsorptionPolicy, ShieldAdjustmentKind},
    modifier::model::StatQuerySubject,
    rule::model::{
        BattleRuleDefinition, Comparison, ConditionExpr, EventFilter, EventValueProperty,
        OnceScope, ProgramStep, ReactionPriority, RuleEffectChancePolicy, RuleEventKind,
        RuleEventPoint, RuleOperationTemplate, RuleShieldEventKind, RuleSource, RuleValue,
        ShieldObservation, SourceClass, TriggerDef, TriggerPhase, ValueExpr,
    },
};
use starclock_replay::battle_event::encode_battle_event_payload;

fn scalar(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(
        Scalar::checked_from_integer(value).unwrap(),
    ))
}
fn op(operation: RuleOperationTemplate) -> ProgramStep {
    ProgramStep::Operation(operation)
}
fn apply(effect: u32, selector: u32) -> ProgramStep {
    op(RuleOperationTemplate::ApplyEffect {
        selector: id(selector),
        effect: id(effect),
        stacks: ValueExpr::Literal(RuleValue::Integer(1)),
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })
}
fn adjustment(
    effect: u32,
    selector: u32,
    kind: ShieldAdjustmentKind,
    amount: ValueExpr,
) -> ProgramStep {
    op(RuleOperationTemplate::AdjustEffectShield {
        selector: id(selector),
        effect: id(effect),
        kind,
        policy: ShieldAbsorptionPolicy::ConcurrentLargest,
        amount,
    })
}
fn increase(amount: i64) -> ProgramStep {
    adjustment(21, 4, ShieldAdjustmentKind::Increase, scalar(amount))
}
fn decrease(amount: i64) -> ProgramStep {
    adjustment(21, 4, ShieldAdjustmentKind::Decrease, scalar(amount))
}
fn damage(amount: i64) -> ProgramStep {
    op(RuleOperationTemplate::TrueDamage {
        selector: id(4),
        amount: scalar(amount),
    })
}
fn fixture(steps: Vec<ProgramStep>, healing: bool) -> Battle {
    let base = catalog(
        ProgramDefinition::new(id(1), vec![], vec![], vec![], vec![]),
        false,
        false,
        false,
        false,
    );
    let mut builder = CombatCatalogBuilder::from_catalog(&base, [0x91; 32]);
    for raw in [21, 22] {
        builder.add_effect(
            EffectDefinition::new(id(raw), vec![], vec![]).with_runtime(
                EffectRuntimeDefinition::new(
                    EffectCategory::Buff,
                    DispelCategory::DispellableBuff,
                    1,
                    None,
                    DurationClock::Permanent,
                    EffectTickPhase::None,
                    EffectStackPolicy::Refresh,
                )
                .unwrap(),
            ),
        );
    }
    builder.add_program(
        ProgramDefinition::new(
            id(11),
            vec![],
            vec![id(2), id(4)],
            vec![id(21), id(22)],
            vec![],
        )
        .with_steps(steps),
    );
    assert!(
        builder.replace_ability(
            base.ability(id(1))
                .unwrap()
                .clone()
                .with_action(empty_action())
                .with_programs(vec![
                    AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, id(11)).unwrap(),
                ])
        )
    );
    let mut triggers = Vec::new();
    for (raw, axis, effect) in [
        (12, None, None),
        (13, Some(RuleShieldEventKind::Adjusted), Some(id(21))),
        (14, Some(RuleShieldEventKind::Applied), None),
        (15, Some(RuleShieldEventKind::Absorbed), None),
        (16, Some(RuleShieldEventKind::Removed), None),
    ] {
        let steps = if raw == 13 {
            vec![
                op(RuleOperationTemplate::EmitRuleEvent {
                    code: 130,
                    value: Some(ValueExpr::ReadEventProperty(
                        EventValueProperty::ShieldChangeAmount,
                    )),
                }),
                op(RuleOperationTemplate::EmitRuleEvent {
                    code: 131,
                    value: Some(ValueExpr::QueryEffectShield {
                        subject: StatQuerySubject::EventTarget,
                        effect: id(21),
                    }),
                }),
                op(RuleOperationTemplate::EmitRuleEvent {
                    code: 132,
                    value: Some(ValueExpr::QueryShield {
                        subject: StatQuerySubject::EventTarget,
                        observation: ShieldObservation::Current,
                    }),
                }),
            ]
        } else {
            vec![op(RuleOperationTemplate::EmitRuleEvent {
                code: raw * 10,
                value: Some(ValueExpr::ReadEventProperty(
                    EventValueProperty::ShieldChangeAmount,
                )),
            })]
        };
        builder.add_program(
            ProgramDefinition::new(id(raw), vec![], vec![], vec![id(21)], vec![]).with_steps(steps),
        );
        triggers.push(TriggerDef {
            id: id(raw),
            event: RuleEventKind::Heal,
            event_point: RuleEventPoint::ShieldChanged,
            phase: TriggerPhase::AfterMutation,
            filter: EventFilter {
                shield_event: axis,
                shield_effect: effect,
                ..EventFilter::default()
            },
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Event,
            priority: ReactionPriority::new(0),
            program: id(raw),
        });
    }
    if healing {
        builder.add_program(
            ProgramDefinition::new(id(17), vec![], vec![id(4)], vec![], vec![]).with_steps(vec![
                op(RuleOperationTemplate::Heal {
                    selector: id(4),
                    apply_formula_modifiers: false,
                    amount: ValueExpr::Multiply {
                        lhs: Box::new(ValueExpr::Negate(Box::new(ValueExpr::ReadEventProperty(
                            EventValueProperty::ShieldChangeAmount,
                        )))),
                        rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                            100_000,
                        )))),
                        rounding: Rounding::NearestTiesEven,
                    },
                }),
            ]),
        );
        triggers.push(TriggerDef {
            id: id(17),
            event: RuleEventKind::Heal,
            event_point: RuleEventPoint::ShieldChanged,
            phase: TriggerPhase::AfterMutation,
            filter: EventFilter {
                shield_event: Some(RuleShieldEventKind::Adjusted),
                shield_effect: Some(id(21)),
                ..EventFilter::default()
            },
            condition: ConditionExpr::Compare {
                operator: Comparison::Less,
                lhs: Box::new(ValueExpr::ReadEventProperty(
                    EventValueProperty::ShieldChangeAmount,
                )),
                rhs: Box::new(scalar(0)),
            },
            once_scope: OnceScope::Event,
            priority: ReactionPriority::new(1),
            program: id(17),
        });
    }
    builder.add_rule(
        RuleDefinition::new(
            id(1),
            triggers.iter().map(|trigger| trigger.program).collect(),
            vec![],
        )
        .with_runtime(BattleRuleDefinition::new(
            RuleSource::new(id(91), SourceClass::Synthetic, vec![], [0x92; 32]),
            vec![],
            triggers,
            None,
        )),
    );
    builder.add_rule_bundle(RuleBundle::new(id(1), vec![id(1)]));
    battle(builder.build().unwrap(), false, true, false)
}
fn changes(events: &[BattleEvent]) -> Vec<ShieldEventData> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Shield(data @ ShieldEventData::Adjusted { .. }) => Some(*data),
            _ => None,
        })
        .collect()
}
fn signals(events: &[BattleEvent], code: u32) -> Vec<i64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::RuleSignal(data) if data.code == code => match data.value {
                Some(RuleValue::Scalar(value)) => Some(value.scaled()),
                _ => panic!("scalar signal"),
            },
            _ => None,
        })
        .collect()
}

#[test]
fn adjustments_floor_preserve_identity_and_recreate_only_after_exhaustion() {
    let mut battle = fixture(
        vec![
            apply(21, 4),
            increase(100),
            increase(50),
            decrease(20),
            decrease(999),
            increase(7),
            increase(0),
        ],
        false,
    );
    let result = start_and_use(&mut battle).unwrap();
    assert!(result.fault().is_none());
    let adjusted = changes(result.events());
    assert_eq!(adjusted.len(), 5);
    let entries = adjusted
        .iter()
        .map(|data| match data {
            ShieldEventData::Adjusted {
                shield,
                before,
                after,
                ..
            } => (shield.get(), before.get(), after.get()),
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        entries,
        vec![
            (1, 0, 100),
            (1, 100, 150),
            (1, 150, 130),
            (1, 130, 0),
            (2, 0, 7)
        ]
    );
    assert_eq!(battle.view().effects_by_id().count(), 1);
    assert_eq!(battle.view().shields_by_id().count(), 1);

    let mut fractional = fixture(
        vec![
            apply(21, 4),
            adjustment(
                21,
                4,
                ShieldAdjustmentKind::Increase,
                ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(7_999_999))),
            ),
        ],
        false,
    );
    let result = start_and_use(&mut fractional).unwrap();
    assert!(
        matches!(changes(result.events())[0], ShieldEventData::Adjusted { after, .. } if after.get() == 7)
    );
}

#[test]
fn absent_owner_effect_and_subintegral_or_empty_decrease_do_not_create_shields() {
    let mut battle = fixture(
        vec![
            increase(100),
            apply(21, 4),
            decrease(10),
            adjustment(
                21,
                4,
                ShieldAdjustmentKind::Increase,
                ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(999_999))),
            ),
        ],
        false,
    );
    let result = start_and_use(&mut battle).unwrap();
    assert!(result.fault().is_none());
    assert!(changes(result.events()).is_empty());
    assert_eq!(battle.view().shields_by_id().count(), 0);
}

#[test]
fn total_and_effect_capacity_are_separate_and_damage_absorbs_both_instances() {
    let mut battle = fixture(
        vec![
            apply(21, 4),
            apply(22, 4),
            op(RuleOperationTemplate::Shield {
                selector: id(4),
                effect: id(22),
                amount: scalar(200),
            }),
            increase(100),
            damage(40),
        ],
        false,
    );
    let result = start_and_use(&mut battle).unwrap();
    assert!(result.fault().is_none());
    // Reactions use a fresh immutable snapshot after the enclosing program's
    // ordered mutations; the increase event still retains its exact +100 delta.
    assert_eq!(signals(result.events(), 130), vec![100_000_000]);
    assert_eq!(signals(result.events(), 131), vec![60_000_000]);
    assert_eq!(signals(result.events(), 132), vec![160_000_000]);
    assert_eq!(signals(result.events(), 140), vec![200_000_000]);
    assert_eq!(
        signals(result.events(), 150),
        vec![-40_000_000, -40_000_000]
    );
    assert_eq!(
        battle
            .view()
            .shields_by_id()
            .map(|shield| shield.remaining().get())
            .collect::<Vec<_>>(),
        vec![160, 60]
    );
    let player = battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Player)
        .unwrap();
    assert_eq!(player.current_hp().get(), 1000);
}

#[test]
fn explicit_shield_teardown_and_effect_removal_prevent_regrowth() {
    let mut battle = fixture(
        vec![
            apply(21, 4),
            increase(100),
            op(RuleOperationTemplate::RemoveShield {
                selector: id(4),
                effect: id(21),
            }),
            op(RuleOperationTemplate::RemoveEffect {
                selector: id(4),
                effect: id(21),
            }),
            increase(50),
        ],
        false,
    );
    let result = start_and_use(&mut battle).unwrap();
    assert!(result.fault().is_none());
    assert_eq!(changes(result.events()).len(), 1);
    assert_eq!(signals(result.events(), 160), vec![-100_000_000]);
    assert_eq!(battle.view().shields_by_id().count(), 0);
    assert_eq!(battle.view().effects_by_id().count(), 0);
}

#[test]
fn committed_decrease_drives_exact_heal_without_matching_absorption_or_other_effects() {
    let mut battle = fixture(
        vec![
            damage(200),
            apply(21, 4),
            apply(22, 4),
            increase(600),
            decrease(270),
            adjustment(22, 4, ShieldAdjustmentKind::Increase, scalar(100)),
            adjustment(22, 4, ShieldAdjustmentKind::Decrease, scalar(50)),
            damage(10),
        ],
        true,
    );
    let result = start_and_use(&mut battle).unwrap();
    assert!(result.fault().is_none());
    assert_eq!(
        signals(result.events(), 130),
        vec![600_000_000, -270_000_000]
    );
    let heals = result
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Heal(data) => Some(data.effective.get()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(heals, vec![27]);
    let player = battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Player)
        .unwrap();
    assert_eq!(player.current_hp().get(), 827);
}

#[test]
fn duplicate_effect_instances_and_conflicting_policies_fault_with_rollback() {
    for conflicting in [false, true] {
        let shield = op(RuleOperationTemplate::Shield {
            selector: id(4),
            effect: id(21),
            amount: scalar(20),
        });
        let tail = if conflicting {
            op(RuleOperationTemplate::AdjustEffectShield {
                selector: id(4),
                effect: id(21),
                kind: ShieldAdjustmentKind::Increase,
                policy: ShieldAbsorptionPolicy::AdditiveByInstance,
                amount: scalar(1),
            })
        } else {
            increase(1)
        };
        let mut steps = vec![apply(21, 4), shield.clone()];
        if !conflicting {
            steps.push(shield);
        }
        steps.push(tail);
        let mut battle = fixture(steps, false);
        let result = start_and_use(&mut battle).unwrap();
        assert!(result.fault().is_some());
        assert_eq!(battle.view().shields_by_id().count(), 0);
        assert_eq!(battle.view().effects_by_id().count(), 0);
        assert_eq!(battle.view().rng_draw_count(), 0);
    }
}

#[test]
fn fresh_commands_have_identical_canonical_events_state_and_rng() {
    let run = || {
        let mut battle = fixture(
            vec![apply(21, 4), increase(100), decrease(30), damage(10)],
            true,
        );
        let result = start_and_use(&mut battle).unwrap();
        assert!(result.fault().is_none());
        (
            result
                .events()
                .iter()
                .map(|event| encode_battle_event_payload(event).unwrap())
                .collect::<Vec<_>>(),
            battle.state_hash(),
            battle.view().rng_draw_count(),
        )
    };
    assert_eq!(run(), run());
    let mut battle = fixture(vec![apply(21, 4), increase(100)], false);
    let stale = Command::StartBattle {
        decision: battle.decision().unwrap().id(),
    };
    start_and_use(&mut battle).unwrap();
    let before = battle.state_hash();
    assert!(battle.apply(stale).is_err());
    assert_eq!(battle.state_hash(), before);
    assert_eq!(battle.view().rng_draw_count(), 0);
}

#[test]
fn effect_query_is_scoped_by_recipient_and_program_input_is_immutable() {
    let mut battle = fixture(
        vec![
            apply(21, 4),
            apply(21, 2),
            increase(100),
            adjustment(21, 2, ShieldAdjustmentKind::Increase, scalar(50)),
            op(RuleOperationTemplate::EmitRuleEvent {
                code: 199,
                value: Some(ValueExpr::QueryEffectShield {
                    subject: StatQuerySubject::Owner,
                    effect: id(21),
                }),
            }),
        ],
        false,
    );
    let result = start_and_use(&mut battle).unwrap();
    assert!(result.fault().is_none());
    assert_eq!(signals(result.events(), 199), vec![0]);
    assert_eq!(signals(result.events(), 131), vec![100_000_000, 50_000_000]);
}

#[test]
fn absorption_exhausts_capacity_without_consuming_the_effect_or_reusing_identity() {
    let mut battle = fixture(
        vec![apply(21, 4), increase(100), damage(150), increase(5)],
        false,
    );
    let result = start_and_use(&mut battle).unwrap();
    assert!(result.fault().is_none());
    let changes = changes(result.events());
    assert!(matches!(changes[0], ShieldEventData::Adjusted { shield, .. } if shield.get() == 1));
    assert!(matches!(changes[1], ShieldEventData::Adjusted { shield, .. } if shield.get() == 2));
    assert_eq!(battle.view().effects_by_id().count(), 1);
    assert_eq!(battle.view().shields_by_id().count(), 1);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .find(|unit| unit.side() == TeamSide::Player)
            .unwrap()
            .current_hp()
            .get(),
        950
    );
}

#[test]
fn unavailable_scalar_capacity_faults_instead_of_returning_a_guessed_zero() {
    let mut battle = fixture(
        vec![
            apply(21, 4),
            adjustment(
                21,
                4,
                ShieldAdjustmentKind::Increase,
                ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(i64::MAX))),
            ),
            adjustment(
                21,
                4,
                ShieldAdjustmentKind::Increase,
                ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(i64::MAX))),
            ),
        ],
        false,
    );
    let result = start_and_use(&mut battle).unwrap();
    assert!(result.fault().is_some());
    assert_eq!(battle.view().shields_by_id().count(), 0);
    assert_eq!(battle.view().effects_by_id().count(), 0);
    assert_eq!(battle.view().rng_draw_count(), 0);
}
