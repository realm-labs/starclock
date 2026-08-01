use starclock_activity::{
    ActivityCause, ActivityOperation, ActivityScope, ActivityTransactionOutcome,
    ActivityTransactionState, SlotCarryPolicy,
};

use super::{
    SWARM_DISASTER_AUDIENCE_RUNTIME_REVISION, SwarmDisasterEntry, SwarmDisasterRuntimeFactory,
    SwarmDisasterRuntimeInstance, audience::AudienceRuntimeCatalog,
};

#[test]
fn eight_audience_definitions_retain_exact_order_and_denominators() {
    let factory = factory();
    assert_eq!(
        SWARM_DISASTER_AUDIENCE_RUNTIME_REVISION,
        "swarm-disaster-audience-runtime-v1"
    );
    assert_eq!(factory.audience.denominators(), (8, 3, 42, 7, 1, 16, 26));
    assert_eq!(
        factory.audience.ordered_paths().collect::<Vec<_>>(),
        [
            (
                "swarm-disaster.audience-path.1",
                "swarm-disaster.audience-die.1"
            ),
            (
                "swarm-disaster.audience-path.2",
                "swarm-disaster.audience-die.2"
            ),
            (
                "swarm-disaster.audience-path.7",
                "swarm-disaster.audience-die.7"
            ),
            (
                "swarm-disaster.audience-path.5",
                "swarm-disaster.audience-die.5"
            ),
            (
                "swarm-disaster.audience-path.6",
                "swarm-disaster.audience-die.6"
            ),
            (
                "swarm-disaster.audience-path.3",
                "swarm-disaster.audience-die.3"
            ),
            (
                "swarm-disaster.audience-path.4",
                "swarm-disaster.audience-die.4"
            ),
            (
                "swarm-disaster.audience-path.8",
                "swarm-disaster.audience-die.8"
            ),
        ]
    );
}

#[test]
fn every_path_exposes_unlock_faces_and_typed_persistent_rule() {
    let factory = factory();
    let cases = [
        (
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
            1,
            Some("1000018"),
            5,
            "641200",
            "ProtectCellNoCollapse",
            &["0", "2"][..],
            &["200620", "3", "10003"][..],
        ),
        (
            "universe.path.remembrance",
            "swarm-disaster.audience-die.2",
            2,
            Some("1000016"),
            5,
            "641210",
            "ExtraMoneyAndRandomSwap",
            &["0", "2"][..],
            &[][..],
        ),
        (
            "universe.path.elation",
            "swarm-disaster.audience-die.7",
            3,
            Some("1000015"),
            5,
            "641260",
            "GetHelpOnEnterCell",
            &["200620", "3", "10004"][..],
            &["20002", "3"][..],
        ),
        (
            "universe.path.hunt",
            "swarm-disaster.audience-die.5",
            4,
            Some("1000017"),
            5,
            "641240",
            "ExtraMarkAndRandomSwap",
            &["1", "1"][..],
            &["2", "5"][..],
        ),
        (
            "universe.path.destruction",
            "swarm-disaster.audience-die.6",
            5,
            None,
            5,
            "641250",
            "DestroyAeonGain",
            &["2", "5"][..],
            &["200720", "3", "10007"][..],
        ),
        (
            "universe.path.nihility",
            "swarm-disaster.audience-die.3",
            6,
            Some("1000013"),
            6,
            "641220",
            "ReRandomEmptyCell",
            &["2"][..],
            &[][..],
        ),
        (
            "universe.path.abundance",
            "swarm-disaster.audience-die.4",
            7,
            Some("1000014"),
            6,
            "641230",
            "FertileAeonGain",
            &["1", "2"][..],
            &["2"][..],
        ),
        (
            "universe.path.propagation",
            "swarm-disaster.audience-die.8",
            8,
            Some("1000008"),
            5,
            "641270",
            "RandomGenSwarm",
            &["610031"][..],
            &[][..],
        ),
    ];
    let mut total_faces = 0;
    for (path, die, sort, unlock, faces, maze_buff, passive, primary, secondary) in cases {
        let instance = instance(&factory, path, die);
        assert_eq!(instance.audience_path_sort(), sort);
        assert_eq!(instance.audience_path_unlock_id(), unlock);
        assert_eq!(instance.audience_path_requires_unlock(), unlock.is_some());
        assert_eq!(instance.audience_initial_rule(), "AddMazeBuff");
        assert_eq!(
            instance.audience_initial_parameters().collect::<Vec<_>>(),
            [maze_buff]
        );
        assert_eq!(
            instance
                .audience_initial_secondary_parameters()
                .collect::<Vec<_>>(),
            ["0"]
        );
        assert_eq!(instance.audience_passive_rule(), passive);
        assert_eq!(
            instance.audience_passive_parameters().collect::<Vec<_>>(),
            primary
        );
        assert_eq!(
            instance
                .audience_passive_secondary_parameters()
                .collect::<Vec<_>>(),
            secondary
        );
        assert_eq!(instance.audience_die_faces().len(), faces);
        total_faces += faces;
    }
    assert_eq!(total_faces, 42);
    assert_eq!(
        instance(
            &factory,
            "universe.path.preservation",
            "swarm-disaster.audience-die.1"
        )
        .audience_die_faces()
        .collect::<Vec<_>>(),
        [
            "swarm-disaster.dice-face.102",
            "swarm-disaster.dice-face.104",
            "swarm-disaster.dice-face.101",
            "swarm-disaster.dice-face.103",
            "swarm-disaster.dice-face.105",
        ]
    );
}

#[test]
fn initialization_commits_once_and_uses_activity_owned_carry_state() {
    let factory = factory();
    let instance = instance(
        &factory,
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
    );
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    assert!(!instance.audience_initialization_applied(&state).unwrap());
    let program = instance.compile_audience_initialization(&state).unwrap();
    assert_eq!(program.operations().len(), 4);
    assert!(matches!(
        program.operations()[2],
        ActivityOperation::AddCounter { slot, .. }
            if slot.get() == super::state::CONTENT
    ));
    apply(&instance, &mut state, &program);
    assert!(instance.audience_initialization_applied(&state).unwrap());

    let sequence = state.command_sequence();
    let cause = cause(&state, program.id());
    assert!(matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state.command_sequence(), sequence);

    for raw in [super::state::AUDIENCE_DIE, super::state::CONTENT] {
        let slot = instance
            .state_definition()
            .slots()
            .iter()
            .find(|definition| definition.id().get() == raw)
            .unwrap();
        assert_eq!(slot.owner(), ActivityScope::Activity);
        assert_eq!(slot.carry(), SlotCarryPolicy::CarryExact);
    }
}

#[test]
fn malformed_unlock_effect_and_face_membership_fail_closed() {
    let factory = factory();
    let input = factory.unique.audience_runtime_input();

    let mut missing_unlock = input.clone();
    missing_unlock.paths[0].unlock_id = None;
    assert!(AudienceRuntimeCatalog::compile(missing_unlock).is_err());

    let mut bad_effect = input.clone();
    bad_effect.paths[0].initial_program = "[]".into();
    assert!(AudienceRuntimeCatalog::compile(bad_effect).is_err());

    let mut empty_faces = input;
    empty_faces.dice[0].face_keys = Box::new([]);
    assert!(AudienceRuntimeCatalog::compile(empty_faces).is_err());
}

#[test]
fn entry_requires_known_exact_once_authored_unlocks() {
    let factory = factory();
    let locked = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        super::tests::participants(super::tests::policy()),
    );
    assert!(factory.compile_entry(locked).is_err());

    let unknown = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        super::tests::participants(super::tests::policy()),
    )
    .with_audience_unlocks(vec!["1000018".into(), "unknown".into()]);
    assert!(factory.compile_entry(unknown).is_err());

    let duplicate = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        super::tests::participants(super::tests::policy()),
    )
    .with_audience_unlocks(vec!["1000018".into(), "1000018".into()]);
    assert!(factory.compile_entry(duplicate).is_err());

    let available = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.destruction",
        "swarm-disaster.audience-die.6",
        super::tests::participants(super::tests::policy()),
    );
    assert!(factory.compile_entry(available).is_ok());
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::tests::BUNDLE).unwrap()
}

fn instance(
    factory: &SwarmDisasterRuntimeFactory,
    path: &str,
    die: &str,
) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(super::tests::released_entry(
            "swarm-disaster.area.201",
            path,
            die,
            super::tests::participants(super::tests::policy()),
        ))
        .unwrap()
}

fn apply(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: &starclock_activity::ActivityProgramDefinition,
) {
    let cause = cause(state, program.id());
    assert!(matches!(
        state.apply_program(program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn cause(
    state: &ActivityTransactionState,
    program: starclock_activity::ActivityProgramId,
) -> ActivityCause {
    ActivityCause::new(state.command_sequence() + 1, program, state.current_node()).unwrap()
}
