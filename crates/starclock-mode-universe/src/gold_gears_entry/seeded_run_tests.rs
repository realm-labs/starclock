use std::collections::BTreeSet;

use starclock_activity::{ActivityInstanceId, ActivityTerminalOutcome};

use super::{
    GOLD_AND_GEARS_SEEDED_RUN_REVISION, GoldAndGearsRuntimeFactory,
    GoldAndGearsSeededRunRequest,
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
    row!("G14-MATRIX-01", 14001, "gold-gears.area.401", "universe.path.abundance", "gold-gears.custom-dice.101", 0, 0, ["G14-R01"], 15, "42e138d9362d55844fe18020434ed7d8609cea5e9f13e8522540be74b0088168", "b27668a62d803800de9f38563f1bd9cbdc825538486126b2eead2e1ed807b854"),
    row!("G14-MATRIX-02", 14002, "gold-gears.area.402", "universe.path.destruction", "gold-gears.custom-dice.102", 0, 0, ["G14-R02"], 16, "6cd253e88a8bc61786af85ab5700013d63f0e32f798ae292f1e2058b14260a8d", "0ba04fee49bae8eb30de467f100eb55725de29fd1e281d61c520bf3d688eeb61"),
    row!("G14-MATRIX-03", 14003, "gold-gears.area.403", "universe.path.elation", "gold-gears.custom-dice.103", 0, 0, ["G14-R03"], 18, "1fb1ed5e02b9daed8199444330e53cbb3d7afa00a27e6891c7f5cc3dfe35b7b0", "ef8d682788fd5abdf61e2785ea0b6b158bd2500932604fcea3a9f15834dac972"),
    row!("G14-MATRIX-04", 14004, "gold-gears.area.404", "universe.path.erudition", "gold-gears.custom-dice.201", 0, 0, ["G14-R04"], 16, "51963a6d547a619eebe469128ecc74f1a3926f6d9c8e3fa156005011a6a5ae2f", "8317d808052971482c7b42d8db0175d493526fc1cdcf85a30354d70c99949d83"),
    row!("G14-MATRIX-05", 14005, "gold-gears.area.405", "universe.path.hunt", "gold-gears.custom-dice.202", 0, 0, ["G14-R05"], 16, "277bd43845e8d86bef5865ed52700b00fb065876e98b34d3915b0835f1c220eb", "0155f4b2a3ca59432a6dfe6957d4380dbc81dcd491d01d4ceb746d0f0d4ac3bd"),
    row!("G14-MATRIX-06", 14006, "gold-gears.area.401", "universe.path.nihility", "gold-gears.custom-dice.203", 0, 0, ["G14-R06"], 16, "d102157bee3673db91cfc37451f6e8ed3d471c0430d542c1f35d21ca8f3387aa", "dcb6abc0aececc3ff4b16824639a06bc208af313ad448f4af53fd8bec14726aa"),
    row!("G14-MATRIX-07", 14007, "gold-gears.area.402", "universe.path.preservation", "gold-gears.custom-dice.301", 0, 0, ["G14-R07"], 15, "5d751782d0bb366fb8e09784bb619b37945cc856cfeb1cd558a8f5a0cb01f2bf", "00e4de1f3de5df72712a1a94f9530d361874d906163cede9229a19d3c55a7c9e"),
    row!("G14-MATRIX-08", 14008, "gold-gears.area.403", "universe.path.propagation", "gold-gears.custom-dice.302", 0, 0, ["G14-R08"], 15, "132a9a0aa20428ec5fcf134deb7282d9c70f5786d118d7f27e731fe66c67513b", "621b92fdd4760c9e2c4b60a26c90446e865608f88ab110c540f71606fa7c0c85"),
    row!("G14-MATRIX-09", 14009, "gold-gears.area.404", "universe.path.remembrance", "gold-gears.custom-dice.303", 0, 0, ["G14-R09"], 15, "bd1e9cea5124e465597b092eb6d019d3ff08a7d283c6b06f08de1101df4f4832", "849090cdaf05fe06eef1ef9498fb719b47bc157d22f79fe5a387757c0bbbfa1c"),
    row!("G14-MATRIX-10", 14010, "gold-gears.area.405", "universe.path.abundance", "gold-gears.custom-dice.401", 0, 0, ["G14-R10"], 18, "9fb99f13f9336679b1b532004d40beebd7536057808a45957bcdf9a2fb9e59d3", "f8d4a0fde3eecf45a37e603f5f318607b8988a57dcdd4ab06b6c085218bbf398"),
    row!("G14-MATRIX-11", 14011, "gold-gears.area.401", "universe.path.destruction", "gold-gears.custom-dice.402", 0, 0, ["G14-R11"], 16, "f4a980d5b1c2f3b2eb5de0dbdf34afd10547b2bda1bf6ed72c0caeaefe3a920d", "98008d101f8e3f6e814b038b806df1bcaf9ded88961f2e367821a6cd1c1ada4c"),
    row!("G14-MATRIX-12", 14012, "gold-gears.area.402", "universe.path.elation", "gold-gears.custom-dice.403", 0, 0, ["G14-R12"], 16, "345b575cd091e0d114c5d0074abe09f752a04678fef49b70d06f5c9261dbb3c9", "cdb181300a5929f318e6d5b97dc89bf385823204f67c200f61c8004168500722"),
    row!("G14-MATRIX-13", 14101, "gold-gears.area.405", "universe.path.erudition", "gold-gears.custom-dice.203", 0, 1, ["G14-R13"], 15, "64fbdb35dfb9553a1275b7600f1aef2a687d98808a3d04ab1521beae91f7c056", "c89aa727c72a645f36a4a916d2cdd3136d3ede54d3e75e042f4abe010af4dd04"),
    row!("G14-MATRIX-14", 14102, "gold-gears.area.405", "universe.path.hunt", "gold-gears.custom-dice.301", 0, 2, ["G14-R14"], 17, "488d57c1479c7f782dde92f50e1a5da05cc805ef66dab710e1ae847e27bebed1", "d305560af1a423975d4fa5ab561f86788c51613eb61a1f3617ff290c64d6b5ab"),
    row!("G14-MATRIX-15", 14103, "gold-gears.area.405", "universe.path.nihility", "gold-gears.custom-dice.302", 0, 3, ["G14-R15"], 17, "11db3bd58d3214ffc019ebaadc87baa1a079566226f07fff49d13a2c42fe32fa", "ce85ce47baa42a087cec131acec5f7f54e9ed3ec4853dfce694be2e59b65eaab"),
    row!("G14-MATRIX-16", 14104, "gold-gears.area.405", "universe.path.preservation", "gold-gears.custom-dice.303", 0, 4, ["G14-R16"], 17, "7d9651fbd8d7b9c69df95be82162ccf2a60be13f757006960dcbe63e7161d33d", "6bc1b540731fe95c84556b6ff98e6d68aaf7f65303fd316e563d2370cf2e2579"),
    row!("G14-MATRIX-17", 14105, "gold-gears.area.405", "universe.path.propagation", "gold-gears.custom-dice.401", 0, 5, [], 16, "d5c409a14ea23e0d16a8575989e2487c7c8560ddf0cd98de21f38586c2c10a30", "2e92b89dda800b341fb1b96f2cb46e8ca71f9d2546fc1663b1f575935068744b"),
    row!("G14-MATRIX-18", 14106, "gold-gears.area.405", "universe.path.remembrance", "gold-gears.custom-dice.402", 0, 6, [], 16, "1f07fff6df84781def7f3b270d02aebd70f41adf3c9ce3b3f3b1017e01c48d9c", "6885ab9a2d020d66511cd4a803f50c85a0071dc1f3172268e78c8968e822f95c"),
    row!("G14-MATRIX-19", 14107, "gold-gears.area.405", "universe.path.abundance", "gold-gears.custom-dice.403", 1, 0, [], 16, "0aa0bc090c68c3ccc83d6f7c1fb135202eb7915bed12e6f63e0b82787283f2eb", "86ee1301f4c289382c7123ec98e849a7a288719141ddce665e63807f4a6d89b3"),
    row!("G14-MATRIX-20", 14108, "gold-gears.area.405", "universe.path.destruction", "gold-gears.custom-dice.101", 2, 0, [], 16, "be3fdb8979a0c387c653d6411c459a483a96149af66cf4e1f504dad4e4bfa441", "1ad856b8f2a053c7af0e30404e293839c98a1244fbe778a678a010d026a3afa6"),
    row!("G14-MATRIX-21", 14109, "gold-gears.area.405", "universe.path.elation", "gold-gears.custom-dice.102", 3, 0, [], 17, "e4a1cd95d9c15a1ca7fc35c70afd96aefa59d0fd6cfc540396f53b9918fa620b", "b790558527a2a5e4644f44865778382ed41d0f0b295a61aa852f5de6152d3d05"),
    row!("G14-MATRIX-22", 14110, "gold-gears.area.405", "universe.path.erudition", "gold-gears.custom-dice.103", 4, 0, [], 17, "b7779bea81f138853dd4c0158fd64a26268b3b223782ac54f2c8b06861b63294", "96a5be4ad945677e0b5687f5cf888465cdda9d892d4f6d91b9a60e473d35f218"),
    row!("G14-MATRIX-23", 14111, "gold-gears.area.405", "universe.path.hunt", "gold-gears.custom-dice.201", 5, 0, [], 16, "e9acffce88d1d264aecc12026b9097e83270c7095c36635aa8b1976b70dc8c99", "f7c8cc5b74dd1d0d1f6c1a107470adaf4b35bddf0a0d74156e88b262235e1dd6"),
    row!("G14-MATRIX-24", 14112, "gold-gears.area.405", "universe.path.nihility", "gold-gears.custom-dice.202", 6, 0, [], 17, "8cbc83a6712bc8f1ecfdc935fb17c0412888cddc4105eab7294bb9e52216f5d8", "ce47b9533f00a5dca040785056a0edad601c3d75041d4e03134c8ed8848be948"),
    row!("G14-MATRIX-25", 14113, "gold-gears.area.405", "universe.path.remembrance", "gold-gears.custom-dice.403", 6, 6, [], 15, "bc29093354482939a3f6b1ceecd5401236885e2c4b1175571148aa2a4e15f3ed", "43d9c39f3cfb2b2469f56f0a5f05fe7bacf5b124d8ab1d68330b6fef1b44b68c"),
];

#[test]
fn frozen_matrix_completes_real_battles_and_verifies_from_a_fresh_factory() {
    assert_eq!(MATRIX.len(), 25);
    assert_eq!(
        GOLD_AND_GEARS_SEEDED_RUN_REVISION,
        "gold-and-gears-seeded-run-v1"
    );
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
