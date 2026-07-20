use super::*;
use starclock_combat::{
    AbilityId, AiCandidateId, AiGraphId, DecisionId, SelectorId,
    catalog::encounter::{
        AiCandidateDefinition, AiCandidateSelection, AiGraphDefinition, AiNoTargetFallback,
        AiStateDefinition, AiTransitionDefinition, AiTransitionTiming,
    },
    rng::types::RngSeed,
    rule::model::ConditionExpr,
};

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn runtime<I: TryFrom<u64>>(raw: u64) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn candidate(raw: u32, ability: u32, no_target: AiNoTargetFallback) -> AiCandidateDefinition {
    AiCandidateDefinition::new(
        id::<AiCandidateId>(raw),
        id::<AbilityId>(ability),
        ConditionExpr::Literal(true),
        id::<SelectorId>(1),
        0,
        AiCandidateSelection::FirstLegal,
        no_target,
    )
}

fn offered(actor: UnitId, ability: u32) -> Command {
    Command::UseAbility {
        decision: runtime::<DecisionId>(7),
        actor,
        ability: id(ability),
        primary_target: None,
    }
}

#[test]
fn no_target_fallback_returns_only_an_exact_offered_command() {
    let state = AiStateDefinition::new(
        id(1),
        None,
        id(1),
        false,
        vec![candidate(
            1,
            2,
            AiNoTargetFallback::UseFallbackAbility(id(1)),
        )],
        vec![],
    );
    let graph = AiGraphDefinition::new(id::<AiGraphId>(1), id(1), 4, vec![state]).unwrap();
    let actor = runtime(3);
    let command = offered(actor, 1);
    let mut controller = EnemyController::new(RngSeed::new([0x19; 32]));
    let result = controller
        .decide_offered(
            &graph,
            id(1),
            actor,
            std::slice::from_ref(&command),
            &mut |_| true,
        )
        .unwrap();
    assert_eq!(result.command(), &command);
    assert_eq!(result.candidate(), Some(id(1)));
    assert_eq!(controller.draw_count(), 0);
}

#[test]
fn no_target_transition_is_bounded_and_selects_from_the_new_state() {
    let first = AiStateDefinition::new(
        id(1),
        None,
        id(1),
        false,
        vec![candidate(1, 2, AiNoTargetFallback::Transition(id(2)))],
        vec![],
    );
    let second = AiStateDefinition::new(
        id(2),
        None,
        id(1),
        true,
        vec![candidate(2, 1, AiNoTargetFallback::StayInState)],
        vec![],
    );
    let graph = AiGraphDefinition::new(id::<AiGraphId>(1), id(1), 4, vec![second, first]).unwrap();
    let actor = runtime(3);
    let command = offered(actor, 1);
    let mut controller = EnemyController::new(RngSeed::new([0x23; 32]));
    let result = controller
        .decide_offered(
            &graph,
            id(1),
            actor,
            std::slice::from_ref(&command),
            &mut |_| true,
        )
        .unwrap();
    assert_eq!(result.state(), id(2));
    assert_eq!(result.command(), &command);
    assert_eq!(controller.draw_count(), 0);
}

#[test]
fn post_action_transition_and_turn_counter_are_controller_owned() {
    let first = AiStateDefinition::new(
        id(1),
        None,
        id(1),
        false,
        vec![candidate(1, 1, AiNoTargetFallback::StayInState)],
        vec![AiTransitionDefinition::new(
            id(1),
            id(2),
            ConditionExpr::Literal(true),
            0,
            AiTransitionTiming::AfterAction,
        )],
    );
    let second = AiStateDefinition::new(
        id(2),
        None,
        id(1),
        true,
        vec![candidate(2, 1, AiNoTargetFallback::StayInState)],
        vec![],
    );
    let graph = AiGraphDefinition::new(id::<AiGraphId>(1), id(1), 4, vec![first, second]).unwrap();
    let actor = runtime(3);
    let mut controller = EnemyController::new(RngSeed::new([0x29; 32]));
    controller.cursors.insert(
        actor,
        EnemyCursor {
            graph: graph.id(),
            state: id(1),
            turns: 7,
        },
    );
    assert_eq!(
        controller
            .settle(&graph, actor, AiTransitionTiming::AfterAction, |_| true)
            .unwrap(),
        (id(2), 1)
    );
    assert_eq!(controller.cursor(actor), Some((graph.id(), id(2), 1)));
}

#[test]
fn weighted_behavior_draw_is_seeded_and_consumed_exactly_once() {
    let weighted = |raw, ability, weight| {
        AiCandidateDefinition::new(
            id(raw),
            id(ability),
            ConditionExpr::Literal(true),
            id(1),
            0,
            AiCandidateSelection::WeightedDraw {
                weight,
                purpose: starclock_combat::rng::types::DrawPurpose::BEHAVIOR_CHOICE,
            },
            AiNoTargetFallback::StayInState,
        )
    };
    let state = AiStateDefinition::new(
        id(1),
        None,
        id(1),
        false,
        vec![weighted(2, 2, 7), weighted(1, 1, 3)],
        vec![],
    );
    let graph = AiGraphDefinition::new(id(1), id(1), 4, vec![state]).unwrap();
    let actor = runtime(3);
    let commands = [offered(actor, 2), offered(actor, 1)];
    let decide = |controller: &mut EnemyController| {
        controller
            .decide_offered(&graph, id(1), actor, &commands, &mut |_| true)
            .unwrap()
    };
    let mut first = EnemyController::new(RngSeed::new([0x37; 32]));
    let mut second = EnemyController::new(RngSeed::new([0x37; 32]));
    assert_eq!(decide(&mut first), decide(&mut second));
    assert_eq!(first.draw_count(), 1);
    assert_eq!(second.draw_count(), 1);
}
