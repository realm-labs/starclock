use starclock_activity::{
    ActivityCause, ActivityExpression, ActivityInstanceId, ActivityMasterSeed, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivityRngContext, ActivityRngLabel,
    ActivityRngStreams, ActivitySlotId, ActivityTerminalOutcome, ActivityTransactionEventKind,
    ActivityTransactionOutcome, ActivityTransactionState, ActivityValue,
};
use starclock_replay::{codec::CanonicalSink, digest::Sha256Sink};

use super::{
    SwarmDisasterRuntimeFactory,
    incremental_run::SwarmDisasterIncrementalRun,
    state::COUNTDOWN,
};
use super::{tests, seeded_run_tests, battle_materialization_tests, SwarmDisasterRuntimeInstance};

const BUNDLE: &[u8] = tests::BUNDLE;
const DOMAINS: [(&str, ActivityRngLabel, u16); 8] = [
    ("graph", ActivityRngLabel::Graph, 0x5001),
    ("dice", ActivityRngLabel::Spawn, 0x5002),
    ("map", ActivityRngLabel::Graph, 0x5003),
    ("reward", ActivityRngLabel::Reward, 0x5004),
    ("shop", ActivityRngLabel::Shop, 0x5005),
    ("occurrence", ActivityRngLabel::Occurrence, 0x5006),
    ("encounter", ActivityRngLabel::Encounter, 0x5007),
    ("adventure", ActivityRngLabel::ExternalOutcomeTest, 0x5008),
];

#[test]
fn swarm_rng_domains_are_golden_and_do_not_shift_battle_or_unrelated_streams() {
    let (instance, _) = seeded_run_tests::representative_runtime();
    let mut digest = Sha256Sink::new();
    for (domain, perturbed_label, purpose) in DOMAINS {
        let mut baseline = rng(&instance, 20_101);
        let mut perturbed = rng(&instance, 20_101);
        for ordinal in 1..=257_u16 {
            let draw = perturbed
                .choose_index(perturbed_label, purpose + ordinal, 97)
                .unwrap()
                .unwrap();
            digest.write(&(domain.len() as u32).to_le_bytes());
            digest.write(domain.as_bytes());
            digest.write(&[draw.label() as u8]);
            digest.write(&draw.purpose().to_le_bytes());
            digest.write(&draw.index().to_le_bytes());
            digest.write(&draw.raw().to_le_bytes());
            digest.write(&draw.upper().to_le_bytes());
            digest.write(&draw.value().to_le_bytes());
            digest.write(&draw.rejected_draws().to_le_bytes());
        }
        for label in ActivityRngLabel::ALL {
            if label == perturbed_label {
                continue;
            }
            let expected = baseline.choose_index(label, 0x50ff, 113).unwrap().unwrap();
            let actual = perturbed.choose_index(label, 0x50ff, 113).unwrap().unwrap();
            assert_eq!(actual, expected, "{domain} shifted {label:?}");
        }
        assert_eq!(
            next_draw(&mut baseline, ActivityRngLabel::Battle),
            next_draw(&mut perturbed, ActivityRngLabel::Battle),
            "{domain} shifted the next Battle draw"
        );
    }
    assert_eq!(
        hex(digest.finalize().bytes()),
        "75504a94c8176df92db96d669420667fba96c906545e6c21253e05845864e08e"
    );
}

#[test]
fn initial_offers_and_state_are_property_stable_across_swarm_seed_corpus() {
    let (instance, roster) = seeded_run_tests::representative_runtime();
    for seed in 20_120..20_184_u64 {
        let mut first = SwarmDisasterIncrementalRun::start(
            &instance,
            seed,
            battle_materialization_tests::activity_identity(),
            ActivityInstanceId::new(1).unwrap(),
        );
        let mut second = SwarmDisasterIncrementalRun::start(
            &instance,
            seed,
            battle_materialization_tests::activity_identity(),
            ActivityInstanceId::new(1).unwrap(),
        );
        first.settle_automatic(&instance, &roster).unwrap();
        second.settle_automatic(&instance, &roster).unwrap();
        assert_eq!(first.player_view(&instance), second.player_view(&instance));
        assert_eq!(
            first.offered_commands(&instance).unwrap(),
            second.offered_commands(&instance).unwrap(),
            "seed {seed} changed canonical offers"
        );
    }
}

#[test]
fn corrupted_swarm_candidate_failures_are_repeatable_and_bounded() {
    for index in [0, BUNDLE.len() / 3, BUNDLE.len() - 1] {
        let mut corrupted = BUNDLE.to_vec();
        corrupted[index] ^= 0x80;
        let first = SwarmDisasterRuntimeFactory::load_candidate(&corrupted)
            .expect_err("corrupted bundle must fail");
        let second = SwarmDisasterRuntimeFactory::load_candidate(&corrupted)
            .expect_err("corrupted bundle must fail repeatably");
        assert_eq!(format!("{first:?}"), format!("{second:?}"));
    }
}

#[test]
fn swarm_state_fault_is_deterministic_and_discards_partial_mutation() {
    let (instance, _) = seeded_run_tests::representative_runtime();
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(0x50f0_0001).unwrap(),
        vec![ActivityOperation::SetSlot {
            slot: ActivitySlotId::new(COUNTDOWN).unwrap(),
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::MAX)),
        }],
    )
    .unwrap();
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
    let (first_outcome, first_bytes) = fault_once(&instance, &program);
    let (second_outcome, second_bytes) = fault_once(&instance, &program);
    assert_eq!(first_outcome, second_outcome);
    assert_eq!(first_bytes, second_bytes);
    assert!(matches!(
        &first_outcome,
        ActivityTransactionOutcome::Faulted(events, starclock_activity::ActivityFault::SlotBounds(slot))
            if events.len() == 1
                && slot.get() == COUNTDOWN
                && matches!(events[0].kind(), ActivityTransactionEventKind::Faulted(_))
    ));
}

fn fault_once(
    instance: &SwarmDisasterRuntimeInstance,
    program: &ActivityProgramDefinition,
) -> (ActivityTransactionOutcome, Box<[u8]>) {
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    let rng = rng(instance, 20_199);
    let cause = ActivityCause::new(1, program.id(), state.current_node()).unwrap();
    let outcome = state.apply_program(program, cause, instance.graph_definition());
    assert_eq!(state.terminal(), Some(ActivityTerminalOutcome::Faulted));
    assert_eq!(
        state.slot(ActivitySlotId::new(COUNTDOWN).unwrap()),
        Some(&ActivityValue::BoundedInteger(20))
    );
    let bytes = state.canonical_state_bytes(
        battle_materialization_tests::activity_identity(),
        instance.graph_definition(),
        ActivityInstanceId::new(1).unwrap(),
        &rng,
    );
    (outcome, bytes)
}

fn rng(instance: &SwarmDisasterRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = battle_materialization_tests::activity_identity();
    ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(seed),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        instance.graph_definition().digest(),
        ActivityInstanceId::new(1).unwrap(),
        None,
        Some(instance.graph_definition().entry()),
        None,
        0,
    ))
}

fn next_draw(rng: &mut ActivityRngStreams, label: ActivityRngLabel) -> u64 {
    rng.choose_index(label, 0x50fe, 127)
        .unwrap()
        .unwrap()
        .raw()
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
