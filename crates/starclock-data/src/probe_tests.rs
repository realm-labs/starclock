use std::collections::BTreeMap;

use starclock_combat::{
    AbilityId, ActionId, EventId, ModifierDefinitionId, ModifierInstanceId, ProgramId, RuleId,
    RuleInstanceId, Scalar, SourceDefinitionId, StateSlotDefinitionId, UnitId, WaveInstanceId,
    formula::hp,
    modifier::model::{
        ActionTargetLedger, ActiveModifier, FormulaPurpose, ModifierQueryContext, StatKind,
        StatQuery,
    },
    modifier::resolve::StatResolver,
    rule::{
        evaluate::{EvaluationBudget, RuleEvaluationError, StatQueryReader, evaluate_program},
        model::{
            ResourceUpdateKind, RuleCause, RuleEffectChancePolicy, RuleEmission,
            RuleEvaluationInput, RuleEventKind, RuleOccurrence, RuleValue, SourceClass,
        },
    },
};

use crate::catalog::{LoadMode, load_with_mode};

const ASTA_PROBE: &[u8] = include_bytes!("../../../config/probes/v1a/asta-modifier/config.sora");
const FIREFLY_PROBE: &[u8] =
    include_bytes!("../../../config/probes/v1a/firefly-damage/config.sora");
const FIREFLY_TRANSFORM_PROBE: &[u8] =
    include_bytes!("../../../config/probes/v1a/firefly-transform/config.sora");
const KAFKA_PROBE: &[u8] = include_bytes!("../../../config/probes/v1a/kafka-dot/config.sora");
const CLARA_PROBE: &[u8] = include_bytes!("../../../config/probes/v1a/clara-counter/config.sora");
const AGLAEA_PROBE: &[u8] =
    include_bytes!("../../../config/probes/v1a/aglaea-memosprite/config.sora");

#[test]
fn asta_ultimate_effect_keeps_an_independent_two_turn_clock() {
    let catalog = load_with_mode(ASTA_PROBE, LoadMode::Fixture).expect("Asta probe must lower");
    let effect = catalog
        .effect(starclock_combat::EffectDefinitionId::new(6).unwrap())
        .unwrap();
    assert_eq!(
        effect.duration_clock(),
        starclock_combat::DurationClock::TargetTurnEnd
    );
    assert_eq!(
        effect.stack_policy(),
        starclock_combat::EffectStackPolicy::Refresh
    );
    assert!(matches!(
        effect.duration(),
        Some(starclock_combat::rule::model::ValueExpr::Literal(
            RuleValue::Integer(2)
        ))
    ));
    let emissions = evaluate_program(
        &*catalog,
        ProgramId::new(7).unwrap(),
        firefly_input(&FireflyStats {
            maximum_hp: 1,
            attack: 1,
        }),
        EvaluationBudget::STANDARD,
    )
    .unwrap();
    assert!(
        matches!(emissions.as_slice(), [RuleEmission::ApplyEffect { effect, chance: RuleEffectChancePolicy::Guaranteed, base_chance: None, rng_purpose: None, .. }] if effect.get() == 6)
    );
}

#[test]
fn kafka_skill_ultimate_and_follow_up_lower_in_authored_order() {
    let catalog = load_with_mode(KAFKA_PROBE, LoadMode::Fixture).expect("Kafka probe must lower");
    let stats = FireflyStats {
        maximum_hp: 1,
        attack: 1_000_000_000,
    };
    let skill = evaluate_program(
        &*catalog,
        ProgramId::new(5).unwrap(),
        firefly_input(&stats),
        EvaluationBudget::STANDARD,
    )
    .unwrap();
    assert!(
        matches!(&skill[0], RuleEmission::Damage { amount: RuleValue::Scalar(value), .. } if value.scaled() == 1_600_000_000)
    );
    assert!(
        matches!(&skill[1], RuleEmission::DetonateDot { fraction: RuleValue::Scalar(value), required_tag: None, .. } if value.scaled() == 750_000)
    );
    let ultimate = evaluate_program(
        &*catalog,
        ProgramId::new(6).unwrap(),
        firefly_input(&stats),
        EvaluationBudget::STANDARD,
    )
    .unwrap();
    assert!(
        matches!(&ultimate[0], RuleEmission::Damage { amount: RuleValue::Scalar(value), .. } if value.scaled() == 800_000_000)
    );
    assert!(
        matches!(&ultimate[1], RuleEmission::ApplyEffect { effect, chance: RuleEffectChancePolicy::Resistible, base_chance: Some(RuleValue::Scalar(value)), rng_purpose: Some(_), .. } if effect.get() == 4 && value.scaled() == 1_000_000)
    );
    assert!(
        matches!(&ultimate[2], RuleEmission::DetonateDot { fraction: RuleValue::Scalar(value), .. } if value.scaled() == 1_000_000)
    );
    let follow_up = evaluate_program(
        &*catalog,
        ProgramId::new(7).unwrap(),
        firefly_input(&stats),
        EvaluationBudget::STANDARD,
    )
    .unwrap();
    assert!(matches!(
        follow_up.as_slice(),
        [
            RuleEmission::Damage { .. },
            RuleEmission::ApplyEffect { .. }
        ]
    ));
    let effect = catalog
        .effect(starclock_combat::EffectDefinitionId::new(4).unwrap())
        .unwrap();
    assert_eq!(
        (
            effect.category(),
            effect.duration_clock(),
            effect.tick_phase()
        ),
        (
            starclock_combat::EffectCategory::Dot,
            starclock_combat::DurationClock::TargetTurnStart,
            starclock_combat::EffectTickPhase::TurnStart,
        )
    );
    assert_eq!(
        effect.snapshot_policy(),
        starclock_combat::EffectSnapshotPolicy::Dynamic
    );
    let runtime = catalog
        .battle_rule(RuleId::new(8).unwrap())
        .expect("Kafka follow-up guard rule must lower");
    assert_eq!(runtime.source().definition().get(), 3);
    assert_eq!(runtime.state_slots().len(), 1);
    let slot = &runtime.state_slots()[0];
    assert_eq!(slot.id().get(), 9);
    assert_eq!(
        slot.scope(),
        starclock_combat::rule::model::BattleRuleScope::Turn
    );
    assert_eq!(slot.initial(), &RuleValue::Integer(0));
    assert_eq!(slot.minimum(), Some(&RuleValue::Integer(0)));
    assert_eq!(slot.maximum(), Some(&RuleValue::Integer(1)));
    assert_eq!(
        slot.reset_points(),
        &[starclock_combat::rule::model::SlotResetPoint::TurnStart]
    );
}

#[test]
fn clara_counter_trigger_and_shared_charge_program_lower_in_authored_order() {
    let catalog = load_with_mode(CLARA_PROBE, LoadMode::Fixture).expect("Clara probe must lower");
    let rule = catalog.battle_rule(RuleId::new(5).unwrap()).unwrap();
    let [trigger] = rule.triggers() else {
        panic!("Clara probe must expose exactly one trigger");
    };
    assert_eq!(trigger.event, RuleEventKind::Hit);
    assert_eq!(
        trigger.phase,
        starclock_combat::rule::model::TriggerPhase::AfterEvent
    );
    assert_eq!(trigger.priority.get(), -100);
    assert_eq!(
        trigger.once_scope,
        starclock_combat::rule::model::OnceScope::Event
    );
    assert!(matches!(
        trigger.condition,
        starclock_combat::rule::model::ConditionExpr::Literal(true)
    ));
    assert_eq!(rule.state_slots()[0].initial(), &RuleValue::Integer(2));

    let stats = FireflyStats {
        maximum_hp: 1,
        attack: 1,
    };
    let emissions = evaluate_program(
        &*catalog,
        ProgramId::new(4).unwrap(),
        firefly_input(&stats),
        EvaluationBudget::STANDARD,
    )
    .unwrap();
    assert!(matches!(
        emissions.as_slice(),
        [
            RuleEmission::QueueAction {
                actor_selector,
                target_selector,
                ability,
                priority,
                ..
            },
            RuleEmission::ModifyStateSlot {
                slot,
                update: starclock_combat::rule::model::StateSlotUpdateKind::Subtract,
                value: RuleValue::Integer(1),
                ..
            }
        ] if actor_selector.get() == 1
            && target_selector.get() == 2
            && ability.get() == 1
            && priority.get() == -100
            && slot.get() == 6
    ));
}

#[test]
fn production_loader_rejects_the_nonproduction_clara_probe() {
    let error = crate::catalog::load(CLARA_PROBE).expect_err("probe cannot enter production");
    assert_eq!(error.kind(), crate::catalog::CatalogLoadErrorKind::Metadata);
}

#[test]
fn production_loader_rejects_the_nonproduction_kafka_probe() {
    let error = crate::catalog::load(KAFKA_PROBE).expect_err("probe cannot enter production");
    assert_eq!(error.kind(), crate::catalog::CatalogLoadErrorKind::Metadata);
}

#[test]
fn asta_dynamic_team_aura_tracks_one_charging_instance() {
    let catalog = load_with_mode(ASTA_PROBE, LoadMode::Fixture).expect("Asta probe must lower");
    let asta = unit(1);
    let ally = unit(2);
    let slot = StateSlotDefinitionId::new(3).unwrap();
    let mut instances = vec![ActiveModifier {
        instance: ModifierInstanceId::new(1).unwrap(),
        definition: ModifierDefinitionId::new(4).unwrap(),
        owner: asta,
        subject: ally,
        source: SourceDefinitionId::new(1).unwrap(),
        source_class: SourceClass::Ability,
        insertion_sequence: 1,
        application_action: None,
        slots: vec![(slot, RuleValue::Integer(0))].into_boxed_slice(),
        captured_value: None,
        captured_stats: Box::new([]),
    }];
    let bases = BTreeMap::from([((ally, StatKind::Atk), Scalar::from_scaled(1_000_000_000))]);
    let query = StatQuery {
        subject: ally,
        stat: StatKind::Atk,
        purpose: FormulaPurpose::Stat,
    };
    for (stacks, expected) in [
        (0, 1_000_000_000),
        (1, 1_140_000_000),
        (5, 1_700_000_000),
        (3, 1_420_000_000),
    ] {
        assert!(instances[0].set_slot(slot, RuleValue::Integer(stacks)));
        let actual = StatResolver::new(catalog.modifiers(), &bases, &instances)
            .query(query, &ModifierQueryContext::default())
            .unwrap();
        assert_eq!(actual.scaled(), expected);
        assert_eq!(
            instances.len(),
            1,
            "stack changes must not replace the aura instance"
        );
    }
}

#[test]
fn production_loader_rejects_the_nonproduction_asta_probe() {
    let error = crate::catalog::load(ASTA_PROBE).expect_err("probe cannot enter production");
    assert_eq!(error.kind(), crate::catalog::CatalogLoadErrorKind::Metadata);
}

#[test]
fn firefly_hp_energy_and_damage_program_is_ordered_and_checked_before_mutation() {
    let catalog = load_with_mode(FIREFLY_PROBE, LoadMode::Fixture).expect("Firefly probe lowers");
    let emissions = evaluate_program(
        &*catalog,
        ProgramId::new(3).unwrap(),
        firefly_input(&FireflyStats {
            maximum_hp: 1_000_000_000,
            attack: 1_000_000_000,
        }),
        EvaluationBudget::STANDARD,
    )
    .unwrap();
    assert!(matches!(
        &emissions[0],
        RuleEmission::ConsumeHp {
            amount: RuleValue::Scalar(value),
            floor: RuleValue::Scalar(floor),
            ..
        } if value.scaled() == 400_000_000 && floor.scaled() == 1_000_000
    ));
    assert!(matches!(
        &emissions[1],
        RuleEmission::ModifyResource {
            resource: starclock_combat::rule::model::RuleResourceKind::Energy,
            update: ResourceUpdateKind::Gain,
            amount: RuleValue::Scalar(value),
            ..
        } if value.scaled() == 600_000
    ));
    assert!(matches!(
        &emissions[2],
        RuleEmission::Damage {
            amount: RuleValue::Scalar(value),
            can_crit: true,
            ..
        } if value.scaled() == 2_000_000_000
    ));

    let ordinary = hp::consume(
        starclock_combat::Hp::new(1_000).unwrap(),
        starclock_combat::Hp::new(400).unwrap(),
        starclock_combat::Hp::new(1).unwrap(),
    )
    .unwrap();
    let floored = hp::consume(
        starclock_combat::Hp::new(1).unwrap(),
        starclock_combat::Hp::new(400).unwrap(),
        starclock_combat::Hp::new(1).unwrap(),
    )
    .unwrap();
    assert_eq!((ordinary.after.get(), floored.after.get()), (600, 1));

    let before = (1_000_i64, 0_i64);
    let invalid = materialize_firefly_program(&emissions, before, 0);
    assert!(invalid.is_err());
    assert_eq!(
        before,
        (1_000, 0),
        "failed preparation cannot partially mutate"
    );
    assert_eq!(
        materialize_firefly_program(&emissions, before, 240).unwrap(),
        (600, 144)
    );
}

#[test]
fn firefly_ultimate_visibility_order_precedes_action_advance() {
    let catalog = load_with_mode(FIREFLY_PROBE, LoadMode::Fixture).expect("Firefly probe lowers");
    let emissions = evaluate_program(
        &*catalog,
        ProgramId::new(4).unwrap(),
        firefly_input(&FireflyStats {
            maximum_hp: 1_000_000_000,
            attack: 1_000_000_000,
        }),
        EvaluationBudget::STANDARD,
    )
    .unwrap();
    assert!(matches!(
        emissions[0],
        RuleEmission::CreateCountdown { code: 4, .. }
    ));
    assert!(matches!(emissions[1], RuleEmission::ApplyEffect { effect, .. } if effect.get() == 5));
    assert!(matches!(
        emissions[2],
        RuleEmission::AdvanceAction {
            amount: RuleValue::Scalar(value),
            ..
        } if value.scaled() == 1_000_000
    ));
    assert!(matches!(
        emissions[3],
        RuleEmission::ModifyResource {
            resource: starclock_combat::rule::model::RuleResourceKind::Energy,
            update: ResourceUpdateKind::Set,
            amount: RuleValue::Scalar(value),
            ..
        } if value.scaled() == 1_000_000
    ));
}

#[test]
fn firefly_transformation_program_keeps_form_ability_and_presence_order() {
    let catalog = load_with_mode(FIREFLY_TRANSFORM_PROBE, LoadMode::Fixture)
        .expect("Firefly transform probe lowers");
    let emissions = evaluate_program(
        &*catalog,
        ProgramId::new(5).unwrap(),
        firefly_input(&FireflyStats {
            maximum_hp: 1,
            attack: 1,
        }),
        EvaluationBudget::STANDARD,
    )
    .unwrap();
    assert!(matches!(
        emissions.as_slice(),
        [
            RuleEmission::Transform {
                replacement_definition,
                ..
            },
            RuleEmission::ReplaceAbility {
                old_ability,
                new_ability,
                ..
            },
            RuleEmission::ChangePresence {
                presence: starclock_combat::PresenceState::Transformed,
                ..
            }
        ] if replacement_definition.get() == 2
            && old_ability.get() == 3
            && new_ability.get() == 4
    ));
}

#[test]
fn aglaea_memosprite_program_keeps_owner_link_and_departure_proposals() {
    let catalog = load_with_mode(AGLAEA_PROBE, LoadMode::Fixture).expect("Aglaea probe lowers");
    let emissions = evaluate_program(
        &*catalog,
        ProgramId::new(6).unwrap(),
        firefly_input(&FireflyStats {
            maximum_hp: 1,
            attack: 1,
        }),
        EvaluationBudget::STANDARD,
    )
    .unwrap();
    assert!(matches!(
        emissions.as_slice(),
        [
            RuleEmission::Summon {
                owner_selector,
                unit_definition,
                ..
            },
            RuleEmission::ChangePresence {
                presence: starclock_combat::PresenceState::Linked,
                ..
            },
            RuleEmission::Despawn { .. }
        ] if owner_selector.get() == 1 && unit_definition.get() == 5
    ));
}

#[test]
fn production_loader_rejects_the_nonproduction_aglaea_probe() {
    let error = crate::catalog::load(AGLAEA_PROBE).expect_err("probe cannot enter production");
    assert_eq!(error.kind(), crate::catalog::CatalogLoadErrorKind::Metadata);
}

#[test]
fn production_loader_rejects_the_nonproduction_firefly_transform_probe() {
    let error = crate::catalog::load(FIREFLY_TRANSFORM_PROBE)
        .expect_err("transform probe cannot enter production");
    assert_eq!(error.kind(), crate::catalog::CatalogLoadErrorKind::Metadata);
}

#[test]
fn firefly_enhanced_skill_adds_weakness_before_toughness_and_super_break() {
    let catalog = load_with_mode(FIREFLY_PROBE, LoadMode::Fixture).expect("Firefly probe lowers");
    let emissions = evaluate_program(
        &*catalog,
        ProgramId::new(6).unwrap(),
        firefly_input(&FireflyStats {
            maximum_hp: 1_000_000_000,
            attack: 1_000_000_000,
        }),
        EvaluationBudget::STANDARD,
    )
    .unwrap();
    assert!(matches!(
        emissions[0],
        RuleEmission::AddWeakness {
            element: starclock_combat::formula::model::CombatElement::Fire,
            ..
        }
    ));
    assert!(matches!(&emissions[1], RuleEmission::ReduceToughness {
        amount: RuleValue::Scalar(value),
        element: starclock_combat::formula::model::CombatElement::Fire,
        ..
    } if value.scaled() == 90_000_000));
    assert!(matches!(&emissions[2], RuleEmission::SuperBreak {
        multiplier: RuleValue::Scalar(value), ..
    } if value.scaled() == 500_000));
}

#[test]
fn production_loader_rejects_the_nonproduction_firefly_probe() {
    let error = crate::catalog::load(FIREFLY_PROBE).expect_err("probe cannot enter production");
    assert_eq!(error.kind(), crate::catalog::CatalogLoadErrorKind::Metadata);
}

#[test]
fn asta_credit_is_distinct_per_action_and_uses_hit_time_weakness() {
    let _catalog = load_with_mode(ASTA_PROBE, LoadMode::Fixture).expect("Asta probe must lower");
    let action = ActionId::new(41).unwrap();
    let [first, second, third] = [unit(11), unit(12), unit(13)];
    let mut ledger = ActionTargetLedger::default();
    let hits = [
        (first, false),
        (second, true),
        (first, true),
        (third, false),
    ];
    let credits = hits.map(|(target, fire_weak_at_hit)| {
        ledger.credit(action, target, 1, u16::from(fire_weak_at_hit), 5)
    });
    assert_eq!(credits, [1, 2, 0, 1]);
    assert_eq!(ledger.len(), 3);
    ledger.clear_action(action);
    assert_eq!(ledger.credit(action, first, 1, 1, 5), 2);
}

fn unit(value: u64) -> UnitId {
    UnitId::new(value).unwrap()
}

struct FireflyStats {
    maximum_hp: i64,
    attack: i64,
}

impl StatQueryReader for FireflyStats {
    fn query_stat(
        &self,
        _origin: starclock_combat::modifier::model::StatQuerySubject,
        _subject: UnitId,
        stat: StatKind,
        _purpose: FormulaPurpose,
    ) -> Result<Scalar, RuleEvaluationError> {
        Ok(Scalar::from_scaled(match stat {
            StatKind::Hp => self.maximum_hp,
            StatKind::Atk => self.attack,
            _ => unreachable!("Firefly probe queries only HP and ATK"),
        }))
    }
}

fn firefly_input(stats: &FireflyStats) -> RuleEvaluationInput<'_> {
    RuleEvaluationInput {
        event_kind: RuleEventKind::Action,
        cause: RuleCause {
            owner: Some(unit(1)),
            actor: Some(unit(1)),
            applier: Some(unit(1)),
            target: Some(unit(2)),
            source: Some(SourceDefinitionId::new(1).unwrap()),
        },
        occurrence: RuleOccurrence {
            rule_instance: RuleInstanceId::new(1).unwrap(),
            event: EventId::new(1).unwrap(),
            hit: None,
            target: Some(unit(2)),
            ability: Some(AbilityId::new(1).unwrap()),
            action: Some(ActionId::new(1).unwrap()),
            turn_event: None,
            wave: WaveInstanceId::new(1).unwrap(),
        },
        source_tags: &[],
        slots: &[],
        selectors: &[],
        stat_reader: Some(stats),
    }
}

fn materialize_firefly_program(
    emissions: &[RuleEmission],
    before: (i64, i64),
    maximum_energy: i64,
) -> Result<(i64, i64), ()> {
    let [
        RuleEmission::ConsumeHp {
            amount: RuleValue::Scalar(amount),
            floor: RuleValue::Scalar(floor),
            ..
        },
        RuleEmission::ModifyResource {
            resource: starclock_combat::rule::model::RuleResourceKind::Energy,
            update: ResourceUpdateKind::Gain,
            amount: RuleValue::Scalar(energy_ratio),
            ..
        },
        RuleEmission::Damage { .. },
    ] = emissions
    else {
        return Err(());
    };
    if maximum_energy <= 0 {
        return Err(());
    }
    let requested = starclock_combat::Hp::from_scalar(*amount, starclock_combat::Rounding::Floor)
        .map_err(|_| ())?;
    let floor = starclock_combat::Hp::from_scalar(*floor, starclock_combat::Rounding::Floor)
        .map_err(|_| ())?;
    let consumed = hp::consume(
        starclock_combat::Hp::new(before.0).map_err(|_| ())?,
        requested,
        floor,
    )
    .map_err(|_| ())?;
    let gain = energy_ratio
        .checked_mul(
            Scalar::checked_from_integer(maximum_energy).map_err(|_| ())?,
            starclock_combat::Rounding::NearestTiesEven,
        )
        .and_then(|value| value.rounded_integer(starclock_combat::Rounding::Floor))
        .map_err(|_| ())?;
    Ok((consumed.after.get(), (before.1 + gain).min(maximum_energy)))
}
