//! Skill Point Rule IR updates preserve requested values and capped overflow.
use super::{balance, maximum, resource_battle, resource_battle_with_reaction, signals};
use crate::combat_ability_program_execution::start_and_use;
use starclock_combat::{
    BattleEvent, BattleEventKind, Command, ResourceEventData, Scalar, SkillPointPayer, TeamSide,
    rule::model::{
        ProgramStep, ResourceMaximumUpdateKind, ResourceUpdateKind, RuleOperationTemplate,
        RuleResourceKind, RuleValue, ValueExpr,
    },
};
use starclock_replay::battle_event::encode_battle_event_payload;

#[test]
fn skill_point_update_preserves_attempted_effective_and_overflow_for_every_update() {
    for (before, cap, update, amount, expected) in [
        (0, 0, ResourceUpdateKind::Gain, 0, (0, 0, 0, 0, 0)),
        (0, 0, ResourceUpdateKind::Gain, 2, (2, 0, 0, 0, 2)),
        (1, 5, ResourceUpdateKind::Gain, 2, (2, 2, 1, 3, 0)),
        (4, 5, ResourceUpdateKind::Gain, 2, (2, 1, 4, 5, 1)),
        (5, 5, ResourceUpdateKind::Gain, 2, (2, 0, 5, 5, 2)),
        (
            5,
            5,
            ResourceUpdateKind::Gain,
            65535,
            (65535, 0, 5, 5, 65535),
        ),
        (
            65535,
            65535,
            ResourceUpdateKind::Gain,
            65535,
            (65535, 0, 65535, 65535, 65535),
        ),
        (2, 5, ResourceUpdateKind::Spend, 2, (2, 2, 2, 0, 0)),
        (2, 5, ResourceUpdateKind::Reserve, 2, (2, 2, 2, 0, 0)),
        (2, 5, ResourceUpdateKind::Set, 2, (2, 0, 2, 2, 0)),
        (2, 5, ResourceUpdateKind::Set, 5, (5, 3, 2, 5, 0)),
        (2, 5, ResourceUpdateKind::Set, 0, (0, 2, 2, 0, 0)),
    ] {
        let run = || {
            let mut battle = resource_battle(
                vec![
                    maximum(ResourceMaximumUpdateKind::Set, cap),
                    balance(
                        RuleResourceKind::SkillPoints,
                        ResourceUpdateKind::Set,
                        before,
                    ),
                    balance(RuleResourceKind::SkillPoints, update, amount),
                ],
                0,
            );
            let result = start_and_use(&mut battle).unwrap();
            assert!(result.fault().is_none(), "{:?}", result.fault());
            let (event, observed) = result
                .events()
                .iter()
                .filter_map(|event| match event.kind() {
                    BattleEventKind::Resource(ResourceEventData::SkillPoints {
                        side,
                        attempted,
                        payer,
                        effective,
                        before,
                        after,
                        overflow,
                    }) => {
                        assert_eq!(*side, TeamSide::Player);
                        assert_eq!(*payer, SkillPointPayer::TeamSkillPoints);
                        Some((event, (*attempted, *effective, *before, *after, *overflow)))
                    }
                    _ => None,
                })
                .next_back()
                .unwrap();
            assert_eq!(observed, expected, "{update:?}, before {before}, cap {cap}");
            assert!(
                signals(result.events(), 40).contains(&(
                    event.id(),
                    Scalar::checked_from_integer(i64::from(expected.3) - i64::from(expected.2))
                        .unwrap()
                ))
            );
            assert!(signals(result.events(), 41).contains(&(
                event.id(),
                Scalar::checked_from_integer(i64::from(expected.4)).unwrap()
            )));
            assert_eq!(
                signals(result.events(), 60)
                    .iter()
                    .any(|(id, _)| *id == event.id()),
                expected.3 > expected.2
            );
            assert_eq!(
                battle.view().team(TeamSide::Player).skill_points(),
                expected.3
            );
            assert_eq!(battle.view().rng_draw_count(), 0);
            let payloads = result
                .events()
                .iter()
                .map(|event| encode_battle_event_payload(event).unwrap())
                .collect::<Vec<_>>();
            (result.events().to_vec(), payloads, battle.state_hash())
        };
        assert_eq!(run(), run());
    }
}

#[test]
fn skill_point_update_fractional_request_keeps_the_existing_integral_floor() {
    let mut step = balance(RuleResourceKind::SkillPoints, ResourceUpdateKind::Gain, 0);
    let ProgramStep::Operation(RuleOperationTemplate::ModifyResource { amount, .. }) = &mut step
    else {
        unreachable!()
    };
    *amount = ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(2_999_999)));
    let mut battle = resource_battle(
        vec![
            balance(RuleResourceKind::SkillPoints, ResourceUpdateKind::Set, 4),
            step,
        ],
        0,
    );
    let result = start_and_use(&mut battle).unwrap();
    assert!(result.fault().is_none());
    assert!(result.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Resource(ResourceEventData::SkillPoints {
            attempted: 2,
            effective: 1,
            before: 4,
            after: 5,
            overflow: 1,
            ..
        })
    )));
}

#[test]
fn skill_point_update_invalid_requests_roll_back_prior_operations_and_events() {
    for (update, amount) in [
        (ResourceUpdateKind::Gain, -1),
        (ResourceUpdateKind::Gain, 65536),
        (ResourceUpdateKind::Spend, 4),
        (ResourceUpdateKind::Reserve, 4),
        (ResourceUpdateKind::Set, 6),
        (ResourceUpdateKind::Set, 65536),
    ] {
        let run = || {
            let mut battle = resource_battle(
                vec![
                    maximum(ResourceMaximumUpdateKind::Add, 2),
                    balance(RuleResourceKind::SkillPoints, ResourceUpdateKind::Gain, 2),
                    // The first updates would succeed, but the final request is invalid.
                    balance(RuleResourceKind::SkillPoints, ResourceUpdateKind::Set, 3),
                    maximum(ResourceMaximumUpdateKind::Set, 5),
                    balance(RuleResourceKind::SkillPoints, update, amount),
                ],
                0,
            );
            let result = start_and_use(&mut battle).unwrap();
            assert!(result.fault().is_some(), "{update:?} {amount}");
            assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 3);
            assert_eq!(
                battle.view().team(TeamSide::Player).maximum_skill_points(),
                5
            );
            assert!(!result.events().iter().any(|event| matches!(
                event.kind(),
                BattleEventKind::Resource(_) | BattleEventKind::RuleSignal(_)
            )));
            assert_eq!(battle.view().rng_draw_count(), 0);
            (result.events().to_vec(), battle.state_hash())
        };
        assert_eq!(run(), run());
    }
}

#[test]
fn skill_point_update_rejected_command_keeps_the_corrected_result_inert() {
    let mut battle = resource_battle(
        vec![balance(
            RuleResourceKind::SkillPoints,
            ResourceUpdateKind::Gain,
            65535,
        )],
        0,
    );
    let stale = Command::StartBattle {
        decision: battle.decision().unwrap().id(),
    };
    let result = start_and_use(&mut battle).unwrap();
    assert!(result.fault().is_none());
    let before = battle.state_hash();
    assert!(battle.apply(stale).is_err());
    assert_eq!(battle.state_hash(), before);
    assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 5);
    assert_eq!(
        battle.view().team(TeamSide::Player).maximum_skill_points(),
        5
    );
    assert_eq!(battle.view().rng_draw_count(), 0);
}

#[test]
fn skill_point_update_reaction_observes_real_overflow_without_faking_effective_gain() {
    let run = || {
        let mut battle = resource_battle_with_reaction(
            vec![balance(
                RuleResourceKind::SkillPoints,
                ResourceUpdateKind::Set,
                4,
            )],
            0,
            Some(vec![balance(
                RuleResourceKind::SkillPoints,
                ResourceUpdateKind::Gain,
                6,
            )]),
        );
        let result = start_and_use(&mut battle).unwrap();
        assert!(result.fault().is_none(), "{:?}", result.fault());
        let reaction = result
            .events()
            .iter()
            .find(|event| {
                matches!(
                    event.kind(),
                    BattleEventKind::Resource(ResourceEventData::SkillPoints {
                        attempted: 6,
                        effective: 1,
                        before: 4,
                        after: 5,
                        overflow: 5,
                        ..
                    })
                )
            })
            .unwrap();
        assert_eq!(reaction.cause().source_definition().unwrap().get(), 80);
        assert!(signals(result.events(), 60).contains(&(reaction.id(), Scalar::ONE)));
        assert!(
            signals(result.events(), 41)
                .contains(&(reaction.id(), Scalar::checked_from_integer(5).unwrap()))
        );
        assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 5);
        assert_eq!(battle.view().rng_draw_count(), 0);
        let payloads = result
            .events()
            .iter()
            .map(|event| encode_battle_event_payload(event).unwrap())
            .collect::<Vec<_>>();
        (result.events().to_vec(), payloads, battle.state_hash())
    };
    assert_eq!(run(), run());
}

#[test]
fn skill_point_update_rule_and_action_gains_agree_on_the_complete_resource_payload() {
    for amount in [0, 1, 2, 6, u16::MAX] {
        let mut via_rule = resource_battle(
            vec![balance(
                RuleResourceKind::SkillPoints,
                ResourceUpdateKind::Gain,
                i64::from(amount),
            )],
            0,
        );
        let mut via_action = resource_battle(vec![], amount);
        let rule = start_and_use(&mut via_rule).unwrap();
        let action = start_and_use(&mut via_action).unwrap();
        assert!(rule.fault().is_none());
        assert!(action.fault().is_none());
        let resource = |event: &BattleEvent| match event.kind() {
            BattleEventKind::Resource(data @ ResourceEventData::SkillPoints { .. }) => {
                Some(data.clone())
            }
            _ => None,
        };
        // The action envelope deliberately omits a zero request; the native
        // program retains a zero balance event with zero effective delta.
        if amount == 0 {
            assert!(
                !action
                    .events()
                    .iter()
                    .any(|event| resource(event).is_some())
            );
        } else {
            assert_eq!(
                rule.events()
                    .iter()
                    .filter_map(resource)
                    .collect::<Vec<_>>(),
                action
                    .events()
                    .iter()
                    .filter_map(resource)
                    .collect::<Vec<_>>()
            );
        }
        assert_eq!(
            via_rule.view().team(TeamSide::Player).skill_points(),
            via_action.view().team(TeamSide::Player).skill_points()
        );
        assert_eq!(
            via_rule.view().rng_draw_count(),
            via_action.view().rng_draw_count()
        );
    }
}
