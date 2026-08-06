use crate::combat_decision::{advance_boundary_if_offered, settle_ready_boundaries};

use super::*;

#[test]
fn representative_rule_emissions_use_authoritative_runtime_services() {
    let program = ProgramDefinition::new(id(1), vec![], vec![], vec![], vec![]);
    let mut battle = battle(
        catalog(program, false, true, false, true),
        false,
        true,
        true,
    );
    let resolution = start_and_use(&mut battle).unwrap();

    assert!(
        resolution.fault().is_none(),
        "unexpected mechanics fault: {:?}",
        resolution.fault()
    );
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Action(starclock_combat::ActionEventData::Queued {
            ability,
            origin: starclock_combat::ActionOrigin::Forced,
            ..
        }) if ability.get() == 3
    )));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Toughness(starclock_combat::ToughnessEventData::LayerDepleted {
            changed_global_broken: true,
            ..
        })
    )));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::BreakDamage(starclock_combat::BreakDamageEventData {
            kind: starclock_combat::BreakDamageKind::SuperBreak,
            ..
        })
    )));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Turn(starclock_combat::TurnEventData::ExtraTurnGranted { owner, .. })
            if owner == &battle.view().units_by_id().next().unwrap().id()
    )));
    let gauge_changes = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Turn(starclock_combat::TurnEventData::ActionGaugeChanged {
                kind,
                before,
                after,
                ..
            }) => Some((*kind, before.scaled(), after.scaled())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        gauge_changes,
        [
            (
                starclock_combat::ActionGaugeChangeKind::Delay,
                0,
                10_000_000_000
            ),
            (
                starclock_combat::ActionGaugeChangeKind::Advance,
                10_000_000_000,
                0
            )
        ]
    );
    settle_ready_boundaries(&mut battle);
    assert_eq!(
        battle.view().active_turn().unwrap().origin(),
        starclock_combat::ActionOrigin::ExtraTurn
    );
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::RuleSignal(starclock_combat::RuleSignalEventData {
            code: 77,
            value: Some(RuleValue::Integer(3)),
            ..
        })
    )));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Unit(starclock_combat::UnitEventData::Summoned {
            kind: LinkedEntityKind::Memosprite,
            ..
        })
    )));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Unit(starclock_combat::UnitEventData::CountdownCreated {
            ability,
            ..
        }) if ability.get() == 4
    )));
    let effect_events = resolution
        .events()
        .iter()
        .filter(|event| matches!(event.kind(), BattleEventKind::Effect(_)))
        .count();
    assert_eq!(effect_events, 2);
    assert_eq!(battle.view().effects_by_id().count(), 0);
    assert_eq!(battle.view().units_by_id().count(), 3);
    assert_eq!(battle.view().links().count(), 2);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .next()
            .unwrap()
            .character_resource("enhanced-counter-charges"),
        Some((
            Scalar::checked_from_integer(1).unwrap(),
            Scalar::checked_from_integer(2).unwrap()
        ))
    );
    assert_eq!(
        battle.view().team(TeamSide::Player).keyed_resource(id(90)),
        Some((3, 5))
    );
    assert_eq!(
        battle
            .view()
            .rule_instances_by_id()
            .next()
            .unwrap()
            .slots()
            .next()
            .unwrap()
            .1,
        &RuleValue::Integer(0)
    );
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .find(|unit| unit.form().get() == 2)
            .unwrap()
            .current_hp()
            .get(),
        972
    );

    advance_boundary_if_offered(&mut battle);
    let extra_action = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(
            |command| matches!(command, Command::UseAbility { ability, .. } if ability.get() == 1),
        )
        .unwrap()
        .clone();
    let extra = battle.apply(extra_action).unwrap();
    let mut events = extra.events().to_vec();
    events.extend(settle_ready_boundaries(&mut battle));
    assert!(events.iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Action(starclock_combat::ActionEventData::Resolved {
            origin: starclock_combat::ActionOrigin::ExtraTurn,
            ..
        })
    )));
    assert!(events.iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Turn(starclock_combat::TurnEventData::Ended {
            origin: starclock_combat::ActionOrigin::ExtraTurn,
            ..
        })
    )));
    assert!(!events.iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Turn(starclock_combat::TurnEventData::ExtraTurnGranted { .. })
    )));
}
