use crate::swarm_disaster_entry::{
    dice_control::{CHEAT_CHARGE_KEY, REROLL_CHARGE_KEY},
    state::RESOURCES,
    tests::{BUNDLE, participants, policy, released_entry},
};
use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityExpression, ActivityInstanceId, ActivityMasterSeed,
    ActivityOperation, ActivityProgramDefinition, ActivityProgramId, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams, ActivitySlotId, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue,
};

use crate::{
    error::UniverseCatalogLoadErrorKind,
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::AudienceRuleRuntimeCatalog;
use crate::swarm_disaster_entry::{SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance};

const FAMILIES: [&str; 3] = [
    "audience-die-passive",
    "dice-face-targeting",
    "dice-roll-reroll-cheat",
];

#[test]
fn exact_sora_partition_binds_and_contract_drift_fails_closed() {
    let factory = factory();
    let _instance = instance(&factory);
    for family in FAMILIES {
        let mut inputs = inputs(&factory);
        inputs
            .iter_mut()
            .find(|input| input.family.as_ref() == family)
            .unwrap()
            .domain = "CrossBattle".into();
        assert_eq!(
            AudienceRuleRuntimeCatalog::compile(inputs)
                .unwrap_err()
                .kind(),
            UniverseCatalogLoadErrorKind::InvalidReference
        );
    }
}

#[test]
fn selected_path_passive_initializes_once_through_activity_state() {
    let instance = instance(&factory());
    let mut state = state(&instance);
    assert!(!instance.audience_initialization_applied(&state).unwrap());
    let program = instance.compile_audience_initialization(&state).unwrap();
    let stale = program.clone();
    commit(&instance, &mut state, program);
    assert!(instance.audience_initialization_applied(&state).unwrap());
    let sequence = state.command_sequence();
    assert!(matches!(
        state.apply_program(
            &stale,
            cause(&state, stale.id()),
            instance.graph_definition(),
        ),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state.command_sequence(), sequence);
}

#[test]
fn roll_reroll_cheat_and_targeting_reuse_labeled_runtime_paths() {
    let instance = instance(&factory());
    let mut state = state(&instance);
    grant_charges(&instance, &mut state, 1, 1);
    let mut rng = rng(&instance, 0x2050_0401);
    let before = spawn_draws(&rng);
    let roll = instance.compile_dice_roll(&state, &mut rng).unwrap();
    assert_eq!(spawn_draws(&rng), before + 1);
    commit(&instance, &mut state, roll);
    assert_eq!(instance.dice_resolution_kind(&state).unwrap(), Some(1));

    let before = spawn_draws(&rng);
    let reroll = instance.compile_dice_reroll(&state, &mut rng).unwrap();
    assert_eq!(spawn_draws(&rng), before + 1);
    commit(&instance, &mut state, reroll);
    assert_eq!(instance.dice_resolution_kind(&state).unwrap(), Some(2));

    let selected = instance.audience_die_faces().next().unwrap().to_owned();
    let snapshots = rng.snapshots();
    let cheat = instance.compile_dice_cheat(&state, &selected).unwrap();
    assert_eq!(rng.snapshots(), snapshots);
    commit(&instance, &mut state, cheat);
    assert_eq!(
        instance.dice_resolution_face(&state),
        Some(selected.as_str())
    );
    assert_eq!(instance.dice_resolution_kind(&state).unwrap(), Some(3));

    let activation = instance
        .compile_dice_face_activation(&state, None, &mut rng)
        .unwrap();
    commit(&instance, &mut state, activation);
    assert!(!instance.dice_reroll_available(&state).unwrap());
    assert!(!instance.dice_cheat_available(&state).unwrap());
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap()
}

fn instance(factory: &SwarmDisasterRuntimeFactory) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(released_entry(
            "swarm-disaster.area.201",
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
            participants(policy()),
        ))
        .unwrap()
}

fn inputs(factory: &SwarmDisasterRuntimeFactory) -> [MechanicRuleRuntimeInput; 3] {
    FAMILIES.map(|family| factory.content.mechanic_rule_runtime_input(family).unwrap())
}

fn state(instance: &SwarmDisasterRuntimeInstance) -> ActivityTransactionState {
    ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    )
}

fn grant_charges(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    rerolls: i64,
    cheats: i64,
) {
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(0x5320_ff10).unwrap(),
        vec![
            ActivityOperation::AddCounter {
                slot: ActivitySlotId::new(RESOURCES).unwrap(),
                key: REROLL_CHARGE_KEY,
                delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(rerolls)),
            },
            ActivityOperation::AddCounter {
                slot: ActivitySlotId::new(RESOURCES).unwrap(),
                key: CHEAT_CHARGE_KEY,
                delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(cheats)),
            },
        ],
    )
    .unwrap();
    commit(instance, state, program);
}

fn commit(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: ActivityProgramDefinition,
) {
    assert!(matches!(
        state.apply_program(
            &program,
            cause(state, program.id()),
            instance.graph_definition(),
        ),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn cause(state: &ActivityTransactionState, program: ActivityProgramId) -> ActivityCause {
    ActivityCause::new(state.command_sequence() + 1, program, state.current_node()).unwrap()
}

fn rng(instance: &SwarmDisasterRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(20).unwrap(),
        ActivityDefinitionDigest::new([0x20; 32]).unwrap(),
        ActivityConfigDigest::new([0x53; 32]).unwrap(),
    );
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

fn spawn_draws(rng: &ActivityRngStreams) -> u64 {
    rng.snapshots()
        .iter()
        .find(|snapshot| snapshot.label() == ActivityRngLabel::Spawn)
        .unwrap()
        .draw_count()
}
