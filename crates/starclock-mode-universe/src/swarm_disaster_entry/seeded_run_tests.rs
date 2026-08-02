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
    ("G20-MATRIX-01", 12, 48, 2, "19e64ca3bfa2b877b9a854bace2d21a3f9f6b0e3123d7a4bdc31a21622ab3749", "93e8cfb56430b076cecad5d56000aa58bc679536b997a9388a8498f239ef9ae3"),
    ("G20-MATRIX-02", 13, 49, 2, "13f79210faa01b6ca5c9f324d6a596bd7a49380b398d23b30bd0877f55586de0", "6e4363ab9f089df2a9ae1dd004d2d23f05e1848a7535bfaeb76a57528d6e3cb3"),
    ("G20-MATRIX-03", 13, 49, 2, "6e4812180849a5fa0577bae7d0374393743b015ea1764d3cf77da7301e46487e", "ec891a0307d9c9cd2d894a20dc58553c3531fc46755dfa8b284554bf8e9a59dd"),
    ("G20-MATRIX-04", 13, 49, 2, "03f2c04589224b7e8e71951a1b9051b9a4ecf28dc64fb3657363e63ab0be0d68", "be16618c8316ad19abba863249b3cffeada828c3696af2bd076e966cdbb46fe5"),
    ("G20-MATRIX-05", 13, 49, 2, "4f312c4bc5e783ecdd103f4018d8db0700c10e8d03dcab7d3419be1e034bb1ff", "88c2d0da23ecbe31fe4c835beaecac9ab7e655a751c452cdced740c4f36542a1"),
    ("G20-MATRIX-06", 13, 49, 2, "43c5aeb545e914a733abd0de0a47722f0687d8f7f63d3be1ed334d7efb034e0c", "e0816368c2eeffdbdc436acd591155f3a19ae37b67a04b898bd92a7ce1448089"),
    ("G20-MATRIX-07", 13, 49, 2, "bf30ff827c98c0d8438e79841ca3a4cd446e1d5c062ed7142967089a6e0d9db0", "8191ed529721112e2473e8a742064184c3d4dc0ca472112ecf75abc06319eb12"),
    ("G20-MATRIX-08", 12, 48, 2, "85afbf24c1448ee31843b15860195c2e38e5c7b0696ef6923ab7f668b241980b", "2a4e141276b7024a65590acd48e83e0409edf66f37f67d41eca2cf1a60df7253"),
    ("G20-MATRIX-09", 12, 48, 2, "ceb7d4d2afb5f16c75efd53fb102fbd5cf9f8c124d341595ec702f5993590346", "37703a929597f9446ad21e0ac90329db0ce154b46904888b9b5dde13355661c3"),
    ("G20-MATRIX-10", 13, 50, 21, "06cd42113b6e49ec140426319193ef34e1eac2b7cf214aa5b52dd35063b9b74a", "76726b9ff3c1d024104aec8ef02fe40326ee676828654c584a537ca372f95f64"),
    ("G20-MATRIX-11", 12, 49, 22, "e63c0a6b1b997e3e57056240c34325fba5426e7e52b2a57c91abd35246f54dd0", "f16df4c76a80bb05e00efeb26d2fa7a506b4d43b62d85d173d32a35bab96d2f5"),
    ("G20-MATRIX-12", 13, 50, 22, "626a45cbb153e02a54d2cad27188dd99a34b15c3546923146be86e7218f3edc7", "f95666ee4f99a6df6322619250f6806fd9c70e1a3f6929d3ff34b0a97490949c"),
    ("G20-MATRIX-13", 13, 50, 22, "00e9144d352d403876ac25ca9ecf756c5784dd882e152a10a713525921cb16a1", "a20193028bad65b7ec1369e8ade0f4357cde4269be44d5734491c544c9e48f2b"),
    ("G20-MATRIX-14", 12, 49, 22, "1391c272a6814c07584823bd37f4fe807eb06f9bd5846d02dfa24da30957b94c", "0d16e39f179905dad3369db1c31ac91536e9dae2375d73baafe4b3e88f872e69"),
    ("G20-MATRIX-15", 13, 50, 12, "4987a69addeeaf6cf8397cbe06a2331398a8487bd35e2167a68a49858771d0f4", "c73bbd379565be9af3ac2f9d69570f561f7e5c269bd3cbb4a9c57ef900a671ae"),
    ("G20-MATRIX-16", 12, 48, 2, "f5a9691d6d9db1cfa5e49ffaa4c4f3865146c4e741675dc7b58dc678307f7124", "f725311a06e766cd670c08c10e0bc559ca2ebaea38a1c0b1ff6cde5f4d252fe9"),
];

#[test]
#[ignore = "exhaustive current-state seeded matrix"]
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

pub(super) fn representative_request() -> SwarmSeededRunRequest {
    request(&MATRIX[0])
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
