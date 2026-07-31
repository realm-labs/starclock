use super::*;

#[test]
fn ability_program_can_queue_actions_without_rule_attribution() {
    let program = ProgramDefinition::new(id(1), vec![], vec![id(2), id(4)], vec![], vec![])
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::QueueAction {
                actor_selector: id(4),
                target_selector: id(2),
                ability: id(3),
                priority: ReactionPriority::new(0),
                forced_use: true,
                boundary: starclock_combat::catalog::action::ReactionBoundary::AfterAction,
                owner: RuleActionOwner::Actor,
                payment: Some(RuleActionPaymentPolicy::Suppressed),
            },
        )]);
    let mut battle = battle(
        catalog(program, false, true, false, true),
        false,
        true,
        true,
    );

    let resolution = start_and_use(&mut battle).unwrap();

    assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    assert!(resolution.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Action(starclock_combat::ActionEventData::Queued {
                ability,
                ..
            }) if ability.get() == 3
        )
    }));
}
