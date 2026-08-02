use std::collections::BTreeSet;

use starclock_activity::{
    ActivityConfigDigest, ActivityInstanceId, ActivityTerminalOutcome,
};

use super::{
    SwarmDisasterEntry, SwarmDisasterRuntimeFactory,
    seeded_run::{
        SWARM_DISASTER_SEEDED_RUN_REVISION, SwarmSeededBoundary, SwarmSeededRunRequest,
    },
};

struct MatrixRow {
    id: &'static str,
    seed: u64,
    area: &'static str,
    path: &'static str,
    die: &'static str,
    faces: &'static [&'static str],
    boundary: SwarmSeededBoundary,
    probes: &'static [&'static str],
}

macro_rules! row {
    ($id:literal, $seed:literal, $area:literal, $path:literal, $die:literal,
     [$($face:literal),*], $boundary:expr, [$($probe:literal),*]) => {
        MatrixRow {
            id: $id,
            seed: $seed,
            area: $area,
            path: $path,
            die: $die,
            faces: &[$($face),*],
            boundary: $boundary,
            probes: &[$($probe),*],
        }
    };
}

static MATRIX: &[MatrixRow] = &[
    row!("G20-MATRIX-01", 20001, "swarm-disaster.area.201", "universe.path.preservation", "swarm-disaster.audience-die.1", ["swarm-disaster.dice-face.102", "swarm-disaster.dice-face.104", "swarm-disaster.dice-face.101", "swarm-disaster.dice-face.103", "swarm-disaster.dice-face.105"], SwarmSeededBoundary::Baseline, ["swarm-disaster.research-gap.source-goal09-project-policy-abstract-adventure-outcome", "swarm-disaster.research-gap.source-goal09-project-policy-mechanical-chapter-locators"]),
    row!("G20-MATRIX-02", 20002, "swarm-disaster.area.202", "universe.path.remembrance", "swarm-disaster.audience-die.2", ["swarm-disaster.dice-face.204", "swarm-disaster.dice-face.205", "swarm-disaster.dice-face.203", "swarm-disaster.dice-face.202", "swarm-disaster.dice-face.201"], SwarmSeededBoundary::Baseline, ["swarm-disaster.research-gap.source-goal09-project-policy-audience-dice", "swarm-disaster.research-gap.source-goal09-project-policy-occurrence-pool-selection"]),
    row!("G20-MATRIX-03", 20003, "swarm-disaster.area.203", "universe.path.nihility", "swarm-disaster.audience-die.3", ["swarm-disaster.dice-face.305", "swarm-disaster.dice-face.301", "swarm-disaster.dice-face.302", "swarm-disaster.dice-face.303", "swarm-disaster.dice-face.304", "swarm-disaster.dice-face.306"], SwarmSeededBoundary::Baseline, ["swarm-disaster.research-gap.source-goal09-project-policy-audience-paths", "swarm-disaster.research-gap.source-goal09-project-policy-occurrence-random-outcome"]),
    row!("G20-MATRIX-04", 20004, "swarm-disaster.area.204", "universe.path.abundance", "swarm-disaster.audience-die.4", ["swarm-disaster.dice-face.405", "swarm-disaster.dice-face.403", "swarm-disaster.dice-face.406", "swarm-disaster.dice-face.402", "swarm-disaster.dice-face.401", "swarm-disaster.dice-face.404"], SwarmSeededBoundary::Baseline, ["swarm-disaster.research-gap.source-goal09-project-policy-beacons", "swarm-disaster.research-gap.source-goal09-project-policy-path-resonance-boundaries"]),
    row!("G20-MATRIX-05", 20005, "swarm-disaster.area.205", "universe.path.hunt", "swarm-disaster.audience-die.5", ["swarm-disaster.dice-face.502", "swarm-disaster.dice-face.504", "swarm-disaster.dice-face.503", "swarm-disaster.dice-face.501", "swarm-disaster.dice-face.505"], SwarmSeededBoundary::Baseline, ["swarm-disaster.research-gap.source-goal09-project-policy-boss-choices", "swarm-disaster.research-gap.source-goal09-project-policy-pathstrider-cabinets"]),
    row!("G20-MATRIX-06", 20006, "swarm-disaster.area.201", "universe.path.destruction", "swarm-disaster.audience-die.6", ["swarm-disaster.dice-face.601", "swarm-disaster.dice-face.602", "swarm-disaster.dice-face.604", "swarm-disaster.dice-face.605", "swarm-disaster.dice-face.603"], SwarmSeededBoundary::Baseline, ["swarm-disaster.research-gap.source-goal09-project-policy-boss-decay-levels", "swarm-disaster.research-gap.source-goal09-project-policy-pathstrider-objectives"]),
    row!("G20-MATRIX-07", 20007, "swarm-disaster.area.202", "universe.path.elation", "swarm-disaster.audience-die.7", ["swarm-disaster.dice-face.706", "swarm-disaster.dice-face.705", "swarm-disaster.dice-face.701", "swarm-disaster.dice-face.702", "swarm-disaster.dice-face.703"], SwarmSeededBoundary::Baseline, ["swarm-disaster.research-gap.source-goal09-project-policy-communing-choices", "swarm-disaster.research-gap.source-goal09-project-policy-pathstrider-unlocks"]),
    row!("G20-MATRIX-08", 20008, "swarm-disaster.area.203", "universe.path.propagation", "swarm-disaster.audience-die.8", ["swarm-disaster.dice-face.805", "swarm-disaster.dice-face.801", "swarm-disaster.dice-face.802", "swarm-disaster.dice-face.803", "swarm-disaster.dice-face.804"], SwarmSeededBoundary::Baseline, ["swarm-disaster.research-gap.source-goal09-project-policy-communing-dimensions", "swarm-disaster.research-gap.source-goal09-project-policy-profile"]),
    row!("G20-MATRIX-09", 20009, "swarm-disaster.area.203", "universe.path.preservation", "swarm-disaster.audience-die.1", ["swarm-disaster.dice-face.102", "swarm-disaster.dice-face.104", "swarm-disaster.dice-face.101", "swarm-disaster.dice-face.103", "swarm-disaster.dice-face.105"], SwarmSeededBoundary::InitialCountdown, ["swarm-disaster.research-gap.source-goal09-project-policy-communing-trail-prerequisites", "swarm-disaster.research-gap.source-goal09-project-policy-rooms"]),
    row!("G20-MATRIX-10", 20010, "swarm-disaster.area.204", "universe.path.remembrance", "swarm-disaster.audience-die.2", ["swarm-disaster.dice-face.204", "swarm-disaster.dice-face.205", "swarm-disaster.dice-face.203", "swarm-disaster.dice-face.202", "swarm-disaster.dice-face.201"], SwarmSeededBoundary::MoveOneToZero, ["swarm-disaster.research-gap.source-goal09-project-policy-countdown-and-disarray", "swarm-disaster.research-gap.source-goal09-project-policy-service-transaction-boundary"]),
    row!("G20-MATRIX-11", 20011, "swarm-disaster.area.205", "universe.path.nihility", "swarm-disaster.audience-die.3", ["swarm-disaster.dice-face.305", "swarm-disaster.dice-face.301", "swarm-disaster.dice-face.302", "swarm-disaster.dice-face.303", "swarm-disaster.dice-face.304", "swarm-disaster.dice-face.306"], SwarmSeededBoundary::EnterDisarrayOne, ["swarm-disaster.research-gap.source-goal09-project-policy-curio-selection-and-lifecycle", "swarm-disaster.research-gap.source-goal09-project-policy-shared-content-pool-weight"]),
    row!("G20-MATRIX-12", 20012, "swarm-disaster.area.201", "universe.path.abundance", "swarm-disaster.audience-die.4", ["swarm-disaster.dice-face.405", "swarm-disaster.dice-face.403", "swarm-disaster.dice-face.406", "swarm-disaster.dice-face.402", "swarm-disaster.dice-face.401", "swarm-disaster.dice-face.404"], SwarmSeededBoundary::ReachDisarray(5), ["swarm-disaster.research-gap.source-goal09-project-policy-dice-roll-controls", "swarm-disaster.research-gap.source-goal09-project-policy-topology-consequences"]),
    row!("G20-MATRIX-13", 20013, "swarm-disaster.area.202", "universe.path.hunt", "swarm-disaster.audience-die.5", ["swarm-disaster.dice-face.502", "swarm-disaster.dice-face.504", "swarm-disaster.dice-face.503", "swarm-disaster.dice-face.501", "swarm-disaster.dice-face.505"], SwarmSeededBoundary::ReachDisarray(10), ["swarm-disaster.research-gap.source-goal09-project-policy-dice-target-rules", "swarm-disaster.research-gap.source-goal09-project-policy-topology-policy"]),
    row!("G20-MATRIX-14", 20014, "swarm-disaster.area.203", "universe.path.destruction", "swarm-disaster.audience-die.6", ["swarm-disaster.dice-face.601", "swarm-disaster.dice-face.602", "swarm-disaster.dice-face.604", "swarm-disaster.dice-face.605", "swarm-disaster.dice-face.603"], SwarmSeededBoundary::ReachDisarray(20), ["swarm-disaster.research-gap.source-goal09-project-policy-domains", "swarm-disaster.research-gap.source-goal09-project-policy-trailblaze-bonus-boundary"]),
    row!("G20-MATRIX-15", 20015, "swarm-disaster.area.204", "universe.path.elation", "swarm-disaster.audience-die.7", ["swarm-disaster.dice-face.706", "swarm-disaster.dice-face.705", "swarm-disaster.dice-face.701", "swarm-disaster.dice-face.702", "swarm-disaster.dice-face.703"], SwarmSeededBoundary::CrossPlaneCountdownCarry, ["swarm-disaster.research-gap.source-goal09-project-policy-encounter-difficulty-binding", "swarm-disaster.research-gap.source-goal09-public-hoyolab-swarm-progression-countdown"]),
    row!("G20-MATRIX-16", 20016, "swarm-disaster.area.205", "universe.path.propagation", "swarm-disaster.audience-die.8", ["swarm-disaster.dice-face.805", "swarm-disaster.dice-face.801", "swarm-disaster.dice-face.802", "swarm-disaster.dice-face.803", "swarm-disaster.dice-face.804"], SwarmSeededBoundary::FinalBossDecay, ["swarm-disaster.research-gap.source-goal09-project-policy-encounter-selection"]),
];

static GOLDENS: &[(&str, u32, u32, i64, &str, &str)] = &[
    ("G20-MATRIX-01", 12, 48, 2, "059710ea6ac74f7ae919a5f066b17fed91e13b249621eaba30e876126a207c11", "6cffe30e7476f330d63569264aaa22a6fe035e73a65658d8b683ded26aa3e703"),
    ("G20-MATRIX-02", 13, 49, 2, "718749bb417fcaacc65a9097c4b8f9112b3f7d0da7d30f478afb10334ee49a29", "6abec441c96b70e31cf27fc00d6b943710dde9721d43b3f9d33877302286eeff"),
    ("G20-MATRIX-03", 13, 49, 2, "c5c21ce295fad6ef5bfcbda7b245477b532d042405a9282bcc41b7e95b78af6e", "0a600fcd4071467c71df417c738e7e959b5e4119b847ff78d646236634344b60"),
    ("G20-MATRIX-04", 13, 49, 2, "a29f0364ab0cc68ecb552bca608e37998859ab43fbf93065ee8d275629d8170f", "9ccbcccf880e9a8dfcc51680445a48019d29370f6a8bf03ff20ef10e61f72143"),
    ("G20-MATRIX-05", 13, 49, 2, "59ea4192cf26f23e157da3fa8f16bc3d2bd30ec90db1524a64d3fd5b41fef025", "1d30162f0d7902a0118d60fe6019e327f368326690bafa9f51c905ca42d6a473"),
    ("G20-MATRIX-06", 13, 49, 2, "1c95dbc4693e7eb69072aebfc9c7ce13e3f5e1ed18946eb66e0607475fe0d960", "c93be1d917e705023e3c5f22e2ca5cefe69ff296c254c31d4085ae8e8dbaef09"),
    ("G20-MATRIX-07", 13, 49, 2, "42e47aa193aa7d560cb5aaf177fb6e3140553ba911506e817b5ae7724802a83b", "c9c1ac5852ca8a27755e66a5829ab89f604d855d1319870b1cfa1acb378cfa0e"),
    ("G20-MATRIX-08", 12, 48, 2, "c5a6414cc7b783e90401e54c4fe67ea9f1181d062e5f3fd03f5f356e34903d90", "9ee6382e9551b76372f0a79127fc1a602b725d7613e779eea98b0e9bb21e4e1a"),
    ("G20-MATRIX-09", 12, 48, 2, "d5cd2fbb8efe71d32b4475dcc3956ca6857ee63a3f3f06a545124841d9f6439a", "1d1574dbe4b47d7e7089f30831b2ad9eed1cfb63372ed9c852bb2dcf9baf0c25"),
    ("G20-MATRIX-10", 13, 50, 21, "40c9675cfaae6e950e6ed11bd7e0fbfaf7682925958846d0332a36c030d8ce50", "1d9c4a850e8315a055be15e9763f3d11d079fc81e3a15fe9c87b1da83fdbb5ee"),
    ("G20-MATRIX-11", 12, 49, 22, "7f2c9ff15f6ef523213b3a79611490d56bf593a2acd09b1e83cdb592ceeb4939", "201314ed6517abd11ffb13fb1299979bcea9ac1cbde1b25d54fd8a769fed24c0"),
    ("G20-MATRIX-12", 13, 50, 22, "d833fa2dab4cb330cd9e9a3d699f57968355ab77e425ff2b1c4be973bba2813f", "f29e2575dbecb4cefb81b994bad2aa32908533472d311054af9fc39930b44635"),
    ("G20-MATRIX-13", 13, 50, 22, "420bfe99c4b79d461f17f2738e4c8581cdde76c18f095dcca742f72cd150985e", "ca306a97db72e4ead8b598d80aa22a3e8c58218a8525e9357d24f31faaf46c38"),
    ("G20-MATRIX-14", 12, 49, 22, "49e8c91333941cee4ad78130180a24f670adb4ea1c0dbc4276043ef75ab40021", "3a2760a5d491de65791bb568183e82c7b12ebbb22e172af13e40556492b55902"),
    ("G20-MATRIX-15", 13, 50, 12, "a39bf7bca0acd600989ed9112883c2a2a80adf0d1e3b4a2f3b5ead1879b68e09", "269e8e666b1fb59939da3141a8642052fc68b4db9380913c2067593a5d10d494"),
    ("G20-MATRIX-16", 12, 48, 2, "2514f3c1346b6699af909c94888f192a7c50dda4e4cc63d7f6f26ef39e240dd1", "89cc40a3e57309c416e9db055ed2f257daa41e23dd706984e73b249efeb6968e"),
];

#[test]
fn frozen_matrix_completes_real_battles_and_verifies_from_fresh_factories() {
    assert_eq!(
        SWARM_DISASTER_SEEDED_RUN_REVISION,
        "swarm-disaster-seeded-run-v1"
    );
    assert_frozen_axes();
    let mut total_battles = 0_u32;
    for row in MATRIX {
        let (instance, roster) = runtime(row);
        assert_eq!(
            instance.audience_die_faces().collect::<Vec<_>>(),
            row.faces
        );
        let request = request(row);
        let report = instance.execute_seeded_run(request, &roster).unwrap();
        assert_eq!(report.terminal, ActivityTerminalOutcome::Completed);
        let expected = GOLDENS.iter().find(|golden| golden.0 == row.id).unwrap();
        assert_eq!(report.battle_count, expected.1, "{} battles", row.id);
        assert_eq!(report.step_count, expected.2, "{} steps", row.id);
        assert_eq!(report.maximum_disarray_level, expected.3, "{} disarray", row.id);
        assert_eq!(hex(report.final_state_hash.bytes()), expected.4, "{} state", row.id);
        assert_eq!(hex(report.transcript_digest), expected.5, "{} transcript", row.id);
        total_battles += report.battle_count;
        let (fresh, fresh_roster) = runtime(row);
        fresh
            .verify_seeded_run(request, &fresh_roster, &report)
            .unwrap();
    }
    assert_eq!(total_battles, 202);
}

fn runtime(
    row: &MatrixRow,
) -> (
    super::SwarmDisasterRuntimeInstance,
    crate::battle_materialization::UniverseBattleRoster,
) {
    let factory = SwarmDisasterRuntimeFactory::load_candidate(super::tests::BUNDLE).unwrap();
    let mut progression = factory
        .unique
        .trail_runtime_input()
        .nodes
        .iter()
        .map(|node| node.key.to_string())
        .collect::<Vec<_>>();
    progression.extend(
        factory
            .unique
            .communing_runtime_input()
            .cabinets
            .iter()
            .map(|cabinet| cabinet.key.to_string()),
    );
    progression.extend(
        factory
            .unique
            .path_runtime_input()
            .interplays
            .iter()
            .map(|interplay| interplay.key.to_string()),
    );
    let points = (1..=7)
        .map(|id| (format!("swarm-disaster.communing-dimension.{id}"), 20))
        .collect();
    let entry = super::tests::released_entry(
        row.area,
        row.path,
        row.die,
        super::battle_materialization_tests::battle_participants(),
    )
    .with_dice_control_unlocks(vec!["1000022".into()])
    .with_progression(points, progression, None);
    compiled(factory, entry)
}

pub(super) fn representative_runtime() -> (
    super::SwarmDisasterRuntimeInstance,
    crate::battle_materialization::UniverseBattleRoster,
) {
    runtime(&MATRIX[0])
}

fn compiled(
    factory: SwarmDisasterRuntimeFactory,
    entry: SwarmDisasterEntry,
) -> (
    super::SwarmDisasterRuntimeInstance,
    crate::battle_materialization::UniverseBattleRoster,
) {
    let instance = factory.compile_entry(entry).unwrap();
    let roster = super::battle_materialization_tests::seeded_matrix_roster(&instance);
    (instance, roster)
}

fn request(row: &MatrixRow) -> SwarmSeededRunRequest {
    SwarmSeededRunRequest {
        seed: row.seed,
        identity: super::battle_materialization_tests::activity_identity(),
        activity_instance: ActivityInstanceId::new(1).unwrap(),
        config_digest: ActivityConfigDigest::new([0x6d; 32]).unwrap(),
        boundary: row.boundary,
    }
}

fn assert_frozen_axes() {
    assert_eq!(MATRIX.len(), 16);
    assert_eq!(
        MATRIX.iter().map(|row| row.area).collect::<BTreeSet<_>>().len(),
        5
    );
    assert_eq!(
        MATRIX.iter().map(|row| row.path).collect::<BTreeSet<_>>().len(),
        8
    );
    assert_eq!(
        MATRIX.iter().map(|row| row.die).collect::<BTreeSet<_>>().len(),
        8
    );
    assert_eq!(
        MATRIX
            .iter()
            .flat_map(|row| row.faces.iter().copied())
            .collect::<BTreeSet<_>>()
            .len(),
        42
    );
    assert_eq!(
        MATRIX
            .iter()
            .flat_map(|row| row.probes.iter().copied())
            .collect::<BTreeSet<_>>()
            .len(),
        31
    );
    assert_eq!(MATRIX.iter().map(|row| row.probes.len()).sum::<usize>(), 31);
}

fn hex<const N: usize>(bytes: [u8; N]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
