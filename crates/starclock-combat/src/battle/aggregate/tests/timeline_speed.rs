//! Controlled resolver inputs supplement public mode assembly regression tests.
use super::{fixture_battle, fixture_catalog_builder, supported_command};
use crate::{
    Battle, Command, EffectInstanceId, ModifierDefinitionId, ModifierInstanceId,
    ModifierStackingGroupId, Scalar, SourceDefinitionId, Speed, TimelineActorId, UnitId,
    actor::{link::LinkedEntityKind, store::TimelineActorState},
    modifier::model::{
        ActiveModifier, FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{RuleValue, SourceClass, ValueExpr},
};

fn controlled_battle(bonus: i64) -> Battle {
    let mut builder = fixture_catalog_builder();
    builder.add_modifier_group(ModifierStackingGroup {
        id: ModifierStackingGroupId::new(1).unwrap(),
        aggregation: ModifierAggregation::Sum,
        comparator: None,
    });
    builder.add_modifier(ModifierDefinition {
        id: ModifierDefinitionId::new(1).unwrap(),
        stat: StatKind::Spd,
        stage: FormulaStage::PercentOfBase,
        purpose: FormulaPurpose::ActionOrder,
        value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(bonus))),
        stacking_group: ModifierStackingGroupId::new(1).unwrap(),
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::PercentOfBase,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::default(),
    });
    let mut battle = fixture_battle();
    battle._catalog = builder.build().unwrap();
    // Deliberate counterfactual holding: test the generic resolver, not a producer.
    battle.state.modifiers.insert(ActiveModifier {
        instance: ModifierInstanceId::new(1).unwrap(),
        definition: ModifierDefinitionId::new(1).unwrap(),
        owner: UnitId::new(1).unwrap(),
        subject: UnitId::new(1).unwrap(),
        source: SourceDefinitionId::new(1).unwrap(),
        source_class: SourceClass::Mode,
        insertion_sequence: 1,
        application_action: None,
        source_effect: Some(EffectInstanceId::new(1).unwrap()),
        slots: Box::default(),
        captured_value: None,
        captured_stats: Box::default(),
    });
    battle
}

#[test]
fn action_order_speed_changes_time_and_gauges_without_compounding() {
    let mut battle = controlled_battle(1_000_000);
    let started = battle.apply(supported_command(&battle)).unwrap();
    assert!(started.fault().is_none());
    assert_eq!(started.timeline_elapsed_scaled(), 50_000_000);
    let actors = battle.view().timeline_actors().collect::<Vec<_>>();
    assert_eq!(actors[0].action_gauge().scaled(), 0);
    assert_eq!(actors[1].action_gauge().scaled(), 5_000_000_000);
    let mut elapsed = 0;
    for _ in 0..8 {
        let resolution = battle.apply(supported_command(&battle)).unwrap();
        assert!(resolution.fault().is_none());
        elapsed += resolution.timeline_elapsed_scaled();
        assert!(
            battle
                .view()
                .timeline_actors()
                .all(|actor| actor.speed().scaled() == 100_000_000)
        );
        if elapsed > 0 {
            break;
        }
    }
    assert_eq!(
        elapsed, 50_000_000,
        "next selection uses the same 200 SPD, not 400"
    );
}

#[test]
fn speed_slowdown_and_removed_modifier_are_resolved_from_current_inputs() {
    for (remove, expected) in [(false, 80_000_000), (true, 100_000_000)] {
        let mut battle = controlled_battle(1_000_000);
        // A pre-existing break slowdown is an independent 0.625 clock ratio.
        battle
            .state
            .actors
            .get_mut(TimelineActorId::new(1).unwrap())
            .unwrap()
            .speed = Speed::from_scaled(62_500_000).unwrap();
        if remove {
            battle
                .state
                .modifiers
                .remove_by_effect(EffectInstanceId::new(1).unwrap());
        }
        let resolution = battle.apply(supported_command(&battle)).unwrap();
        assert!(resolution.fault().is_none());
        assert_eq!(resolution.timeline_elapsed_scaled(), expected);
    }
}

#[test]
fn owner_speed_does_not_scale_a_timeline_only_actor() {
    let mut battle = controlled_battle(1_000_000);
    let mut actor: TimelineActorState = battle.state.actors.iter_by_id().next().unwrap().clone();
    actor.id = TimelineActorId::new(3).unwrap();
    actor.unit = None;
    actor.kind = Some(LinkedEntityKind::Countdown);
    actor.speed = Speed::from_scaled(150_000_000).unwrap();
    battle.state.actors.insert(actor);
    let resolution = battle.apply(supported_command(&battle)).unwrap();
    assert!(resolution.fault().is_none());
    assert_eq!(resolution.timeline_elapsed_scaled(), 50_000_000);
    assert_eq!(
        battle
            .state
            .actors
            .get(TimelineActorId::new(3).unwrap())
            .unwrap()
            .gauge
            .scaled(),
        2_500_000_000
    );
}

#[test]
fn nonpositive_resolved_speed_faults_and_rolls_back() {
    for bonus in [-1_000_000, -2_000_000] {
        let mut battle = controlled_battle(bonus);
        let actors = battle.state.actors.clone();
        let units = battle.state.units.clone();
        let resolution = battle
            .apply(Command::StartBattle {
                decision: battle.decision().unwrap().id(),
            })
            .unwrap();
        assert!(resolution.fault().is_some());
        assert_eq!(battle.state.actors, actors);
        assert_eq!(battle.state.units, units);
        assert_eq!(resolution.timeline_elapsed_scaled(), 0);
    }
}
