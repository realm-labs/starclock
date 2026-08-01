use starclock_activity::{
    ActivityCause, ActivityOperation, ActivitySlotId, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue, SlotCarryPolicy,
};

use super::{SwarmDisasterEntry, SwarmDisasterRuntimeFactory, state};

#[test]
fn catalog_denominators_and_activity_carry_are_exact() {
    let factory = factory();
    assert_eq!(factory.countdown.denominators(), (19, 15, 27, 20, 5, -1));
    let instance = instance(&factory);
    let countdown = instance
        .state_definition()
        .slots()
        .iter()
        .find(|slot| slot.id() == ActivitySlotId::new(state::COUNTDOWN).unwrap())
        .unwrap();
    assert_eq!(countdown.carry(), SlotCarryPolicy::CarryExact);
    assert_eq!(countdown.initial(), &ActivityValue::BoundedInteger(20));
    let state = transaction_state(&instance);
    assert_eq!(instance.countdown(&state).unwrap(), 20);
    assert_eq!(instance.disarray_level(&state).unwrap(), 0);
    assert_eq!(instance.disarray_modifiers(&state).unwrap(), (0, 0, 0));
    assert!(!instance.countdown_warning_active(&state).unwrap());
}

#[test]
fn accepted_moves_enter_disarray_and_cap_level_twenty_modifiers() {
    let factory = factory();
    let instance = instance(&factory);
    let mut state = transaction_state(&instance);
    for _ in 0..20 {
        apply_move(&mut state, &instance);
    }
    assert_eq!(instance.countdown(&state).unwrap(), 0);
    assert_eq!(instance.disarray_level(&state).unwrap(), 0);
    assert!(instance.countdown_warning_active(&state).unwrap());

    apply_move(&mut state, &instance);
    assert_eq!(instance.countdown(&state).unwrap(), -1);
    assert_eq!(instance.disarray_level(&state).unwrap(), 1);
    assert_eq!(instance.disarray_modifiers(&state).unwrap(), (5, 4, 0));
    assert!(!instance.countdown_warning_active(&state).unwrap());

    for _ in 1..5 {
        apply_move(&mut state, &instance);
    }
    assert_eq!(instance.disarray_level(&state).unwrap(), 5);
    assert_eq!(instance.disarray_modifiers(&state).unwrap(), (25, 20, 0));
    apply_move(&mut state, &instance);
    assert_eq!(instance.disarray_modifiers(&state).unwrap(), (35, 24, 5));

    for _ in 6..20 {
        apply_move(&mut state, &instance);
    }
    assert_eq!(instance.disarray_level(&state).unwrap(), 20);
    assert_eq!(instance.disarray_modifiers(&state).unwrap(), (275, 80, 125));
    apply_move(&mut state, &instance);
    assert_eq!(instance.disarray_level(&state).unwrap(), 21);
    assert_eq!(instance.disarray_modifiers(&state).unwrap(), (275, 80, 125));
}

#[test]
fn adjustments_are_stable_and_stale_programs_reject_without_mutation() {
    let factory = factory();
    let instance = instance(&factory);
    let mut state = transaction_state(&instance);
    let move_program = instance
        .compile_countdown_move(&state, &[(20, 2), (10, -3)])
        .unwrap();
    let deltas = move_program
        .operations()
        .iter()
        .filter_map(|operation| match operation {
            ActivityOperation::AddToSlot {
                delta:
                    starclock_activity::ActivityExpression::Literal(ActivityValue::BoundedInteger(
                        value,
                    )),
                ..
            } => Some(*value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(deltas, [-1, -3, 2]);
    apply(&mut state, &move_program, &instance);
    assert_eq!(instance.countdown(&state).unwrap(), 18);

    let stale = instance.compile_countdown_move(&state, &[]).unwrap();
    let adjustment = instance
        .compile_countdown_adjustments(&state, &[(2, 5), (1, -2)])
        .unwrap();
    apply(&mut state, &adjustment, &instance);
    assert_eq!(instance.countdown(&state).unwrap(), 21);
    let sequence = state.command_sequence();
    let stale_cause = cause(&state, stale.id(), &instance);
    assert!(matches!(
        state.apply_program(&stale, stale_cause, instance.graph_definition(),),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state.command_sequence(), sequence);
    assert_eq!(instance.countdown(&state).unwrap(), 21);
    assert!(
        instance
            .compile_countdown_adjustments(&state, &[(1, 1), (1, -1)])
            .is_err()
    );
    assert!(
        instance
            .compile_countdown_adjustments(&state, &[(1, 1_000_000)])
            .is_err()
    );
}

#[test]
fn boss_decay_contributions_are_bounded_sorted_and_fail_closed() {
    let factory = factory();
    let instance = instance(&factory);
    let mut state = transaction_state(&instance);
    let program = instance
        .compile_boss_decay_selection(
            &state,
            &[
                "swarm-disaster.boss-decay.25",
                "swarm-disaster.boss-decay.1",
            ],
        )
        .unwrap();
    apply(&mut state, &program, &instance);
    let contributions = instance.countdown.selected_boss_decay(&state).unwrap();
    assert_eq!(
        contributions
            .iter()
            .map(|row| row.key())
            .collect::<Vec<_>>(),
        [
            "swarm-disaster.boss-decay.1",
            "swarm-disaster.boss-decay.25"
        ]
    );
    assert_eq!(contributions[0].effect_program(), "[]");
    assert_eq!(contributions[1].effect_program(), "[\"0.3\"]");
    assert!(
        instance
            .compile_boss_decay_selection(&state, &["swarm-disaster.boss-decay.2"])
            .is_err()
    );
    assert!(
        instance
            .compile_boss_decay_selection(&state, &["swarm-disaster.boss-decay.101"])
            .is_err()
    );
    assert!(
        instance
            .compile_boss_decay_selection(&state, &["swarm-disaster.boss-decay.unknown"])
            .is_err()
    );
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::tests::BUNDLE).unwrap()
}

fn instance(factory: &SwarmDisasterRuntimeFactory) -> super::SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(SwarmDisasterEntry::new(
            "swarm-disaster.area.201",
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
            super::tests::participants(super::tests::policy()),
        ))
        .unwrap()
}

fn transaction_state(instance: &super::SwarmDisasterRuntimeInstance) -> ActivityTransactionState {
    ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    )
}

fn apply(
    state: &mut ActivityTransactionState,
    program: &starclock_activity::ActivityProgramDefinition,
    instance: &super::SwarmDisasterRuntimeInstance,
) {
    let cause = cause(state, program.id(), instance);
    assert!(matches!(
        state.apply_program(program, cause, instance.graph_definition(),),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn apply_move(
    state: &mut ActivityTransactionState,
    instance: &super::SwarmDisasterRuntimeInstance,
) {
    let program = instance.compile_countdown_move(state, &[]).unwrap();
    apply(state, &program, instance);
}

fn cause(
    state: &ActivityTransactionState,
    program: starclock_activity::ActivityProgramId,
    instance: &super::SwarmDisasterRuntimeInstance,
) -> ActivityCause {
    ActivityCause::new(
        state.command_sequence() + 1,
        program,
        instance.graph_definition().entry(),
    )
    .unwrap()
}
