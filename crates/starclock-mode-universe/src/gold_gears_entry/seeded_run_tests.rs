use std::collections::BTreeSet;

use starclock_activity::{ActivityInstanceId, ActivityTerminalOutcome};

use super::{
    GoldAndGearsRuntimeFactory, GoldAndGearsSeededRunRequest,
    battle_materialization_tests::{activity_identity, seeded_matrix_roster},
};

const BUNDLE: &[u8] = include_bytes!("../../../../config/gold-and-gears-generated/config.sora");

struct MatrixRow {
    id: &'static str,
    seed: u64,
    area_id: &'static str,
    path_id: &'static str,
    custom_dice_id: &'static str,
    stats_conundrum: u8,
    auxiliary_conundrum: u8,
    policy_probes: &'static [&'static str],
    battle_count: u32,
    final_state_hash: &'static str,
    transcript_digest: &'static str,
}

macro_rules! row {
    ($id:literal, $seed:literal, $area:literal, $path:literal, $dice:literal,
     $stats:literal, $auxiliary:literal, [$($probe:literal),*], $battles:literal,
     $state:literal, $transcript:literal) => {
        MatrixRow {
            id: $id,
            seed: $seed,
            area_id: $area,
            path_id: $path,
            custom_dice_id: $dice,
            stats_conundrum: $stats,
            auxiliary_conundrum: $auxiliary,
            policy_probes: &[$($probe),*],
            battle_count: $battles,
            final_state_hash: $state,
            transcript_digest: $transcript,
        }
    };
}

static MATRIX: &[MatrixRow] = &[
    row!("G14-MATRIX-01", 14001, "gold-gears.area.401", "universe.path.abundance", "gold-gears.custom-dice.101", 0, 0, ["G14-R01"], 15, "e729a37d832dd1fea02976f12e59e3734c197d27240bdcc2c4047cb0d9cb0aeb", "bc82fe8b12a4e861f2195d46596f1352c8bc4dc368bf813202541e3518e74656"),
    row!("G14-MATRIX-02", 14002, "gold-gears.area.402", "universe.path.destruction", "gold-gears.custom-dice.102", 0, 0, ["G14-R02"], 16, "0f8a4fd5f42776f5ae11c10c366d1e93b2389f07f93d7eb12599e19b93db1b22", "f1c6375afbdf7c7483f1632b938c9be19c07b1ccbf6051a1674605b20d61f530"),
    row!("G14-MATRIX-03", 14003, "gold-gears.area.403", "universe.path.elation", "gold-gears.custom-dice.103", 0, 0, ["G14-R03"], 18, "2a60126df9f2592e04e6497701e0a64dee02e0f8965f103cf29345908d04edfb", "fbb6c10dba814d1820af5962fdc010ef8b59fdd465b3be58d23d599654eefea9"),
    row!("G14-MATRIX-04", 14004, "gold-gears.area.404", "universe.path.erudition", "gold-gears.custom-dice.201", 0, 0, ["G14-R04"], 16, "77178e073e2462d96c9cf3476101caa88e47a3dbb4fea9f87e08ba9bc62b67b1", "d357234c4228cfbdcc7f8779645d8df7c78c70e10d96a6de81d7057718cc8494"),
    row!("G14-MATRIX-05", 14005, "gold-gears.area.405", "universe.path.hunt", "gold-gears.custom-dice.202", 0, 0, ["G14-R05"], 16, "f5be816320a561f17698f9acaa01fab3af3f95ea2c27c966b1cfcb8f67f4537e", "0861f4d56f09eb663cd110943450f8608079ed34049eea42d5d69837100e4656"),
    row!("G14-MATRIX-06", 14006, "gold-gears.area.401", "universe.path.nihility", "gold-gears.custom-dice.203", 0, 0, ["G14-R06"], 16, "e6b5273dd78922681c9734c186aa5ed51f42e06c693ce305173ef0d6908148f3", "44a2c4401ce4fd9af17c6ae500fa1882b3c55cc6dc44e8dff599567b41d74134"),
    row!("G14-MATRIX-07", 14007, "gold-gears.area.402", "universe.path.preservation", "gold-gears.custom-dice.301", 0, 0, ["G14-R07"], 15, "5107fa778b96743bc3e50fdbb2f8109a64ed00abc10fc095283706a3a05a8bc9", "f73151504c244ce0f0e60b9e0587922138178df38d8a378166dbce15ef88b493"),
    row!("G14-MATRIX-08", 14008, "gold-gears.area.403", "universe.path.propagation", "gold-gears.custom-dice.302", 0, 0, ["G14-R08"], 15, "d9327657abf7bc2d5bb0ffc66cade9960d9e940614912954f25714086f6af076", "66456abfa6c1e25502abf0d8b666e9072f52482189e52a52195d10d6c2c1ca66"),
    row!("G14-MATRIX-09", 14009, "gold-gears.area.404", "universe.path.remembrance", "gold-gears.custom-dice.303", 0, 0, ["G14-R09"], 15, "2898a5d158f61cdf78051bf9fd5059f6c11cb2b66e4405c7344c97278a1f986b", "31743209637e42bbf0b180a379001606382bf56d9b67348212ea85ec73845d22"),
    row!("G14-MATRIX-10", 14010, "gold-gears.area.405", "universe.path.abundance", "gold-gears.custom-dice.401", 0, 0, ["G14-R10"], 18, "59be031d9594b93bdb374d10c56b195ed6bc31caad0c7615fe37c21f242fd39f", "8a15559b14a845a313c4d2edc6c2a1452b60c06ae02116d78eff1f654fe28a0c"),
    row!("G14-MATRIX-11", 14011, "gold-gears.area.401", "universe.path.destruction", "gold-gears.custom-dice.402", 0, 0, ["G14-R11"], 16, "a7154a0bcf7a38e688ec2119b2acf2bec44cf4a3eedea22a94295da0e9968a20", "2c86b9205d95dcc0719a59716e72e7089aa1f681f5db6f75adf6504544e39ea6"),
    row!("G14-MATRIX-12", 14012, "gold-gears.area.402", "universe.path.elation", "gold-gears.custom-dice.403", 0, 0, ["G14-R12"], 16, "4d21fe2d9bff1f1f27df48b95d7eddbc237a2f87c6cb6475dfda86ad1b54a401", "6c9a6b951b2532994e1f1c8034e736fba330f4bd36dde636ba01aa2c32ba24fe"),
    row!("G14-MATRIX-13", 14101, "gold-gears.area.405", "universe.path.erudition", "gold-gears.custom-dice.203", 0, 1, ["G14-R13"], 15, "f33eb91f46c2c7140b29a425d3cef0f8ed509773a229fedd4dad01cb03e43a63", "9bf8d130fcd31c071ff169b9f9e2c5dff050a3d3d91e28cf7302ae98dd49265a"),
    row!("G14-MATRIX-14", 14102, "gold-gears.area.405", "universe.path.hunt", "gold-gears.custom-dice.301", 0, 2, ["G14-R14"], 17, "5758343e8de7cb873b361926491383ecb9b456cfd3f00dfb8d905be0c86ee280", "d7c7b1a7e69cfcf417129ea8c0851981674acc64e19dbcbbcbf48d5cbae190ee"),
    row!("G14-MATRIX-15", 14103, "gold-gears.area.405", "universe.path.nihility", "gold-gears.custom-dice.302", 0, 3, ["G14-R15"], 17, "311309a09d086bbf2f8ae178e2bba390a40a9b49444ee88085efb6fc56e9a267", "34b4bea1782672d86aa17860b7264aca3fadbad6db28aa61fd6023471aec4aae"),
    row!("G14-MATRIX-16", 14104, "gold-gears.area.405", "universe.path.preservation", "gold-gears.custom-dice.303", 0, 4, ["G14-R16"], 17, "0cdbbbb32cd870c3cbaaf9bf209c3ba42158cd11504ea152785533d95eaf0cc9", "25907ecf297947504cb91fceafd587eb0f404f2c1869483cdf90ee8e024aee0b"),
    row!("G14-MATRIX-17", 14105, "gold-gears.area.405", "universe.path.propagation", "gold-gears.custom-dice.401", 0, 5, [], 16, "90ae21038d8b3d726fc5cc3825627b181654de868215e644a3d65d1463b620d7", "391b8eb9f000a40faff2fd8a891367ca3682b918853cc789db6ac286bac7cc2f"),
    row!("G14-MATRIX-18", 14106, "gold-gears.area.405", "universe.path.remembrance", "gold-gears.custom-dice.402", 0, 6, [], 16, "d15187d9e6341cfd4e4c077d483ab302c78e00ea8e3962bf6ed61b5b07d4cadd", "8240c0da875609e5a8b7f7db18c83473437283f8e0ee8a6e5914595b581f175b"),
    row!("G14-MATRIX-19", 14107, "gold-gears.area.405", "universe.path.abundance", "gold-gears.custom-dice.403", 1, 0, [], 16, "ca2b0b142fb9870ec635fdcf742555f77bb7ff613e4dce090d489a6368e1d97f", "3eae2fdc879f9b42f3efa43fbe86e424ad67a47d34d890a9a86cd6f726928fb4"),
    row!("G14-MATRIX-20", 14108, "gold-gears.area.405", "universe.path.destruction", "gold-gears.custom-dice.101", 2, 0, [], 16, "f3ad25cbbca399575227636cb14fb3e5d317117488ebd4a70d6d9be056bd3295", "2b800957808e8cacd32023bd8bfee4ce8062a1e8adea64ecb6398f32bef4573f"),
    row!("G14-MATRIX-21", 14109, "gold-gears.area.405", "universe.path.elation", "gold-gears.custom-dice.102", 3, 0, [], 17, "a1d90909dcfb66dbf097f75d2301ede754d95944ec38c8ef3b1a222a5c62c548", "3c4d096403e1a44c308fee3e831975e1ae2f4d57fd9d48596e8fec21f6c1313d"),
    row!("G14-MATRIX-22", 14110, "gold-gears.area.405", "universe.path.erudition", "gold-gears.custom-dice.103", 4, 0, [], 17, "0ae83ac4c335273a200144f14bdce5f1d2b359aad3a0f6bb005233a96583839d", "cecc8dc706907e8c5cb8fd7c6ac39118daba6a883d4fadb7ff6dc87488fd9993"),
    row!("G14-MATRIX-23", 14111, "gold-gears.area.405", "universe.path.hunt", "gold-gears.custom-dice.201", 5, 0, [], 16, "d83d9eda513de25024b97d6cbffbb6ea6e43d8f5fd7a69cb15362d94e75f4d54", "cd81bb3345b5bdc7b00dfebbee70a39d3d91be23d4f3c51e6e418f3624ddef4f"),
    row!("G14-MATRIX-24", 14112, "gold-gears.area.405", "universe.path.nihility", "gold-gears.custom-dice.202", 6, 0, [], 17, "7fe014e3da3ba084f98dc06a6aec1ed810757f92662fbd94be7234d05ee7d221", "fd24802a509961b96f443eef47fba58eece5da71de74cb8f0987ef55244a2066"),
    row!("G14-MATRIX-25", 14113, "gold-gears.area.405", "universe.path.remembrance", "gold-gears.custom-dice.403", 6, 6, [], 15, "415bf3fab217bbf3dd8dd111459ff3056b21627231bbc612db9b20e46a0f190a", "280a313fcab3171cf639090a4df306d6ea3c1ce4d4620130a3a474196e654443"),
];

#[test]
#[ignore = "exhaustive current-state seeded matrix"]
fn frozen_matrix_completes_real_battles_and_verifies_from_a_fresh_factory() {
    assert_eq!(MATRIX.len(), 25);
    let primary = super::tests::shared_factory();
    let fresh = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let mut policies = BTreeSet::new();

    for row in MATRIX {
        let primary_instance = compile_row(primary, row);
        let primary_roster = seeded_matrix_roster(&primary_instance);
        let request = GoldAndGearsSeededRunRequest::new(
            row.seed,
            activity_identity(),
            ActivityInstanceId::new(1).unwrap(),
        );
        let report = primary_instance
            .execute_seeded_run(request, &primary_roster)
            .unwrap_or_else(|error| panic!("{} failed: {error:?}", row.id));
        assert_eq!(report.terminal(), ActivityTerminalOutcome::Completed);

        let fresh_instance = compile_row(&fresh, row);
        let fresh_roster = seeded_matrix_roster(&fresh_instance);
        let verified = fresh_instance
            .verify_seeded_run(request, &fresh_roster, &report)
            .unwrap_or_else(|error| panic!("{} replay failed: {error:?}", row.id));
        assert_eq!(verified, report);
        assert_eq!(report.battle_count(), row.battle_count);
        assert_eq!(hex(report.final_state_hash().bytes()), row.final_state_hash);
        assert_eq!(hex(report.transcript_digest()), row.transcript_digest);
        policies.extend(row.policy_probes.iter().map(|probe| (*probe).to_owned()));
    }

    assert_eq!(
        policies,
        (1..=16)
            .map(|ordinal| format!("G14-R{ordinal:02}"))
            .collect::<BTreeSet<_>>()
    );
}

fn compile_row(
    factory: &GoldAndGearsRuntimeFactory,
    row: &MatrixRow,
) -> super::GoldAndGearsRuntimeInstance {
    let dice = factory
        .unique
        .dice
        .iter()
        .find(|dice| dice.identity.stable_key.as_ref() == row.custom_dice_id)
        .unwrap();
    let mut entry = super::tests::battle_entry(factory, row.area_id, row.path_id, dice);
    if row.stats_conundrum > 0 || row.auxiliary_conundrum > 0 {
        entry = entry.with_conundrum(
            row.stats_conundrum,
            row.auxiliary_conundrum,
            vec![super::CONUNDRUM_AREA_KEY.to_owned()],
        );
    }
    factory.compile_entry(entry).unwrap()
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
