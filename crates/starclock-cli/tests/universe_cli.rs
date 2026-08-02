use std::{fs, path::PathBuf, process::Command};

use starclock_replay::{
    codec::CanonicalSink,
    digest::{Sha256Digest, Sha256Sink},
};

fn output(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_starclock"))
        .args(args)
        .output()
        .expect("starclock CLI launches")
}

fn text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).expect("CLI emits UTF-8")
}

fn fixture_path(suffix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "starclock-g07-p6-b2-{}-{suffix}.scrp",
        std::process::id()
    ))
}

#[test]
fn universe_configuration_and_coverage_are_machine_readable() {
    let validation = output(&["universe", "config", "validate", "--json"]);
    assert!(validation.status.success(), "{validation:?}");
    assert_eq!(
        text(validation.stdout).trim(),
        "{\"kind\":\"universe-config-validation\",\"valid\":true,\"bundle_sha256\":\"5e5234ee3977f794ae9b1b833372f51c38408c205105c464f11827e9e9ae6a75\",\"worlds\":9,\"difficulties\":33,\"paths\":9,\"blessings\":162,\"curios\":61}"
    );

    let coverage = output(&["universe", "coverage", "--json"]);
    assert!(coverage.status.success(), "{coverage:?}");
    assert_eq!(
        text(coverage.stdout).trim(),
        "{\"kind\":\"universe-coverage\",\"content_records\":2201,\"rule_bindings\":786,\"fixtures\":78,\"worlds\":9,\"difficulties\":33,\"paths\":9,\"encounter_groups\":74}"
    );
}

#[test]
fn gold_and_gears_configuration_and_coverage_are_machine_readable() {
    let validation = output(&[
        "universe",
        "config",
        "validate",
        "--mode",
        "gold-and-gears",
        "--json",
    ]);
    assert!(validation.status.success(), "{validation:?}");
    assert_eq!(
        text(validation.stdout).trim(),
        "{\"kind\":\"universe-config-validation\",\"mode\":\"gold-and-gears\",\"valid\":true,\"bundle_sha256\":\"97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b\",\"tables\":52,\"rows\":29140,\"source_obligations\":7913,\"mechanic_rules\":1224,\"fixtures\":18,\"policy_boundaries\":16}"
    );

    let coverage = output(&["universe", "coverage", "--mode", "gold-and-gears", "--json"]);
    assert!(coverage.status.success(), "{coverage:?}");
    assert_eq!(
        text(coverage.stdout).trim(),
        "{\"kind\":\"universe-coverage\",\"mode\":\"gold-and-gears\",\"source_categories\":42,\"runtime_slices\":44,\"source_obligations\":7913,\"integrated\":7181,\"shared_integrated\":706,\"external_outcomes\":8,\"metadata\":18,\"mechanic_rules\":1224,\"fixtures\":18,\"native_handlers\":0,\"coverage_digest\":\"f2d927d197cb77c548522bf39383a68e927f3881412f44dee8a0b4302c38ca9d\"}"
    );
}

#[test]
fn swarm_disaster_configuration_and_coverage_are_machine_readable() {
    let validation = output(&[
        "universe",
        "config",
        "validate",
        "--mode",
        "swarm-disaster",
        "--json",
    ]);
    assert!(validation.status.success(), "{validation:?}");
    assert_eq!(
        text(validation.stdout).trim(),
        "{\"kind\":\"universe-config-validation\",\"mode\":\"swarm-disaster\",\"valid\":true,\"bundle_sha256\":\"385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362\",\"tables\":65,\"rows\":33380,\"source_obligations\":6963,\"mechanic_rules\":23,\"fixtures\":23,\"policy_boundaries\":31}"
    );

    let coverage = output(&["universe", "coverage", "--mode", "swarm-disaster", "--json"]);
    assert!(coverage.status.success(), "{coverage:?}");
    assert_eq!(
        text(coverage.stdout).trim(),
        "{\"kind\":\"universe-coverage\",\"mode\":\"swarm-disaster\",\"source_categories\":42,\"runtime_slices\":42,\"source_obligations\":6963,\"integrated\":6282,\"shared_integrated\":652,\"external_outcomes\":6,\"metadata\":23,\"mechanic_rules\":23,\"fixtures\":23,\"native_handlers\":0,\"coverage_digest\":\"8aeb60d2c1b322f9dcf8f84bc45dc1901276633398cdb60a984ccc4846f0bff4\"}"
    );
}

#[test]
#[ignore = "complete Gold and Gears run and replay boundary"]
fn gold_and_gears_human_diagnostics_match_the_json_run() {
    let validation = output(&["universe", "config", "validate", "--mode", "gold-and-gears"]);
    assert!(validation.status.success(), "{validation:?}");
    assert_eq!(
        text(validation.stdout).trim(),
        "universe config valid mode=gold-and-gears bundle_sha256=97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b tables=52 rows=29140 source_obligations=7913 rules=1224 fixtures=18 policies=16"
    );

    let coverage = output(&["universe", "coverage", "--mode", "gold-and-gears"]);
    assert!(coverage.status.success(), "{coverage:?}");
    assert_eq!(
        text(coverage.stdout).trim(),
        "universe coverage mode=gold-and-gears categories=42 slices=44 source_obligations=7913 integrated=7181 shared_integrated=706 external_outcomes=8 metadata=18 rules=1224 fixtures=18 native_handlers=0 digest=f2d927d197cb77c548522bf39383a68e927f3881412f44dee8a0b4302c38ca9d"
    );

    let run = output(&[
        "universe",
        "run",
        "--mode",
        "gold-and-gears",
        "--seed",
        "14001",
        "--controller",
        "baseline",
    ]);
    assert!(run.status.success(), "{run:?}");
    assert_eq!(
        text(run.stdout).trim(),
        "universe completed mode=gold-and-gears seed=14001 profile=gold-and-gears-real-battle-replay controller=baseline fixture_accuracy=SyntheticBalanceIndependentNotObservedNumericParity component_root=5e2b458569e50ab3e085bf4fa4c3fb6810ed766e1f2e974c4171c0b30962b395 actions=61 nested_battles=16 battle_commands=94 hash=0019d89fb81f2654a324b07561de8a301e3f07c735b1e3407592c186641b326b replay_bytes=104210 replay_sha256=96c949529b5648ae4bb1c7eb6ec4b7754e0f7b0a2408c817672e2d69093ebbe8"
    );
}

#[test]
#[ignore = "complete Swarm Disaster run and replay boundary"]
fn swarm_disaster_human_diagnostics_match_the_json_run() {
    let validation = output(&["universe", "config", "validate", "--mode", "swarm-disaster"]);
    assert!(validation.status.success(), "{validation:?}");
    assert_eq!(
        text(validation.stdout).trim(),
        "universe config valid mode=swarm-disaster bundle_sha256=385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362 tables=65 rows=33380 source_obligations=6963 rules=23 fixtures=23 policies=31"
    );

    let coverage = output(&["universe", "coverage", "--mode", "swarm-disaster"]);
    assert!(coverage.status.success(), "{coverage:?}");
    assert_eq!(
        text(coverage.stdout).trim(),
        "universe coverage mode=swarm-disaster categories=42 slices=42 source_obligations=6963 integrated=6282 shared_integrated=652 external_outcomes=6 metadata=23 rules=23 fixtures=23 native_handlers=0 digest=8aeb60d2c1b322f9dcf8f84bc45dc1901276633398cdb60a984ccc4846f0bff4"
    );

    let run = output(&[
        "universe",
        "run",
        "--mode",
        "swarm-disaster",
        "--seed",
        "20001",
        "--controller",
        "baseline",
    ]);
    assert!(run.status.success(), "{run:?}");
    assert_eq!(
        text(run.stdout).trim(),
        "universe completed mode=swarm-disaster seed=20001 profile=swarm-disaster-real-battle-replay controller=baseline fixture_accuracy=SyntheticBalanceIndependentNotObservedNumericParity component_root=a34f73e4a8f373ca5876a9205f1dc0d4e6cce03f0538f676c9ec173643da179c actions=48 nested_battles=12 battle_commands=72 hash=ddb36a9a2dead46083382defdbb26f7d6ace63217d568aae7ac6aab76979d7dd replay_bytes=85764 replay_sha256=184ef3262266fbf0d217dbd8a420437618126431ce511a1d9f592699a1e1a89c"
    );
}

#[test]
#[ignore = "complete Standard Universe run and replay corruption boundary"]
fn universe_run_round_trips_a_canonical_replay_and_detects_corruption() {
    let replay = fixture_path("run");
    let corrupt = fixture_path("corrupt");
    for path in [&replay, &corrupt] {
        let _ = fs::remove_file(path);
    }

    let run = output(&[
        "universe",
        "run",
        "--world",
        "1",
        "--difficulty-index",
        "0",
        "--seed",
        "1",
        "--controller",
        "baseline",
        "--replay-out",
        replay.to_str().unwrap(),
        "--json",
    ]);
    assert!(run.status.success(), "{run:?}");
    assert_eq!(
        text(run.stdout).trim(),
        "{\"kind\":\"universe-run\",\"world\":1,\"difficulty_index\":0,\"seed\":1,\"controller\":\"baseline\",\"actions\":53,\"nested_battles\":3,\"battle_commands\":19,\"terminal\":\"completed\",\"state_hash\":\"6a23119f6f5e29836e15923c855af6e26f9f5a81afa26181b297e30b96679bd6\",\"replay_bytes\":34959}"
    );

    let replay_bytes = fs::read(&replay).unwrap();
    assert_eq!(replay_bytes.len(), 34_959);
    let decoded = starclock_replay::format::decode_replay(&replay_bytes).unwrap();
    assert_eq!(decoded.header().components().components().len(), 9);
    assert!(decoded.records().iter().any(|record| {
        record.kind() == starclock_replay::record::RecordKind::AcceptedBattleCommand
    }));
    let mut replay_hash = Sha256Sink::new();
    replay_hash.write(&replay_bytes);
    assert_eq!(
        replay_hash.finalize(),
        Sha256Digest::new([
            23, 82, 56, 107, 253, 242, 237, 85, 134, 149, 3, 215, 5, 60, 139, 7, 124, 224, 226,
            180, 81, 37, 148, 208, 141, 254, 75, 40, 231, 109, 82, 226,
        ])
    );

    let verified = output(&["replay", "verify", replay.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{verified:?}");
    assert_eq!(
        text(verified.stdout).trim(),
        "{\"kind\":\"replay-verify\",\"entry\":\"standard-universe\",\"actions\":53,\"nested_battles\":3,\"battle_commands\":19,\"terminal\":\"completed\",\"state_hash\":\"6a23119f6f5e29836e15923c855af6e26f9f5a81afa26181b297e30b96679bd6\"}"
    );

    let mut changed = replay_bytes;
    let last = changed.len() - 1;
    changed[last] ^= 1;
    fs::write(&corrupt, changed).unwrap();
    let rejected = output(&["replay", "verify", corrupt.to_str().unwrap()]);
    assert_eq!(rejected.status.code(), Some(4));
    assert!(text(rejected.stderr).contains("universe replay error"));

    fs::remove_file(replay).unwrap();
    fs::remove_file(corrupt).unwrap();
}

#[test]
#[ignore = "complete Gold and Gears run and replay corruption boundary"]
fn gold_and_gears_run_round_trips_component_replay_and_detects_corruption() {
    let replay = fixture_path("gold-run");
    let corrupt = fixture_path("gold-corrupt");
    for path in [&replay, &corrupt] {
        let _ = fs::remove_file(path);
    }

    let run = output(&[
        "universe",
        "run",
        "--mode",
        "gold-and-gears",
        "--seed",
        "14001",
        "--controller",
        "baseline",
        "--replay-out",
        replay.to_str().unwrap(),
        "--json",
    ]);
    assert!(run.status.success(), "{run:?}");
    assert_eq!(
        text(run.stdout).trim(),
        "{\"kind\":\"universe-run\",\"mode\":\"gold-and-gears\",\"seed\":14001,\"profile\":\"gold-and-gears-real-battle-replay\",\"area\":\"gold-gears.area.401\",\"path\":\"universe.path.abundance\",\"custom_dice\":\"gold-gears.custom-dice.101\",\"controller\":\"baseline\",\"fixture_accuracy\":\"SyntheticBalanceIndependentNotObservedNumericParity\",\"component_root\":\"5e2b458569e50ab3e085bf4fa4c3fb6810ed766e1f2e974c4171c0b30962b395\",\"actions\":61,\"nested_battles\":16,\"battle_commands\":94,\"terminal\":\"completed\",\"state_hash\":\"0019d89fb81f2654a324b07561de8a301e3f07c735b1e3407592c186641b326b\",\"replay_bytes\":104210,\"replay_sha256\":\"96c949529b5648ae4bb1c7eb6ec4b7754e0f7b0a2408c817672e2d69093ebbe8\"}"
    );

    let replay_bytes = fs::read(&replay).unwrap();
    assert_eq!(replay_bytes.len(), 104_210);
    let decoded = starclock_replay::format::decode_replay(&replay_bytes).unwrap();
    assert_eq!(decoded.header().components().components().len(), 10);
    assert!(decoded.records().iter().any(|record| {
        record.kind() == starclock_replay::record::RecordKind::AcceptedBattleCommand
    }));

    let verified = output(&["replay", "verify", replay.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{verified:?}");
    assert_eq!(
        text(verified.stdout).trim(),
        "{\"kind\":\"replay-verify\",\"entry\":\"gold-and-gears\",\"actions\":61,\"nested_battles\":16,\"battle_commands\":94,\"terminal\":\"completed\",\"state_hash\":\"0019d89fb81f2654a324b07561de8a301e3f07c735b1e3407592c186641b326b\"}"
    );

    let mut changed = replay_bytes;
    let last = changed.len() - 1;
    changed[last] ^= 1;
    fs::write(&corrupt, changed).unwrap();
    let rejected = output(&["replay", "verify", corrupt.to_str().unwrap()]);
    assert_eq!(rejected.status.code(), Some(4));
    assert!(text(rejected.stderr).contains("gold-and-gears replay error"));

    fs::remove_file(replay).unwrap();
    fs::remove_file(corrupt).unwrap();
}

#[test]
#[ignore = "complete Swarm Disaster run and replay corruption boundary"]
fn swarm_disaster_run_round_trips_component_replay_and_detects_corruption() {
    let replay = fixture_path("swarm-run");
    let corrupt = fixture_path("swarm-corrupt");
    for path in [&replay, &corrupt] {
        let _ = fs::remove_file(path);
    }

    let run = output(&[
        "universe",
        "run",
        "--mode",
        "swarm-disaster",
        "--seed",
        "20001",
        "--controller",
        "baseline",
        "--replay-out",
        replay.to_str().unwrap(),
        "--json",
    ]);
    assert!(run.status.success(), "{run:?}");
    assert_eq!(
        text(run.stdout).trim(),
        "{\"kind\":\"universe-run\",\"mode\":\"swarm-disaster\",\"seed\":20001,\"profile\":\"swarm-disaster-real-battle-replay\",\"area\":\"swarm-disaster.area.201\",\"path\":\"universe.path.preservation\",\"audience_die\":\"swarm-disaster.audience-die.1\",\"controller\":\"baseline\",\"fixture_accuracy\":\"SyntheticBalanceIndependentNotObservedNumericParity\",\"component_root\":\"a34f73e4a8f373ca5876a9205f1dc0d4e6cce03f0538f676c9ec173643da179c\",\"actions\":48,\"nested_battles\":12,\"battle_commands\":72,\"terminal\":\"completed\",\"state_hash\":\"ddb36a9a2dead46083382defdbb26f7d6ace63217d568aae7ac6aab76979d7dd\",\"replay_bytes\":85764,\"replay_sha256\":\"184ef3262266fbf0d217dbd8a420437618126431ce511a1d9f592699a1e1a89c\"}"
    );

    let replay_bytes = fs::read(&replay).unwrap();
    assert_eq!(replay_bytes.len(), 85_764);
    let decoded = starclock_replay::format::decode_replay(&replay_bytes).unwrap();
    assert_eq!(decoded.header().components().components().len(), 10);
    assert!(decoded.records().iter().any(|record| {
        record.kind() == starclock_replay::record::RecordKind::AcceptedBattleCommand
    }));

    let verified = output(&["replay", "verify", replay.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{verified:?}");
    assert_eq!(
        text(verified.stdout).trim(),
        "{\"kind\":\"replay-verify\",\"entry\":\"swarm-disaster\",\"actions\":48,\"nested_battles\":12,\"battle_commands\":72,\"terminal\":\"completed\",\"state_hash\":\"ddb36a9a2dead46083382defdbb26f7d6ace63217d568aae7ac6aab76979d7dd\"}"
    );

    let mut changed = replay_bytes;
    let last = changed.len() - 1;
    changed[last] ^= 1;
    fs::write(&corrupt, changed).unwrap();
    let rejected = output(&["replay", "verify", corrupt.to_str().unwrap()]);
    assert_eq!(rejected.status.code(), Some(4));
    assert!(text(rejected.stderr).contains("swarm-disaster replay error"));

    fs::remove_file(replay).unwrap();
    fs::remove_file(corrupt).unwrap();
}

#[test]
fn universe_cli_keeps_usage_and_unknown_content_exit_classes_distinct() {
    let invalid_seed = output(&[
        "universe",
        "run",
        "--world",
        "1",
        "--difficulty-index",
        "0",
        "--seed",
        "not-a-seed",
    ]);
    assert_eq!(invalid_seed.status.code(), Some(2));

    let unknown_world = output(&[
        "universe",
        "run",
        "--world",
        "100",
        "--difficulty-index",
        "0",
        "--seed",
        "1",
    ]);
    assert_eq!(unknown_world.status.code(), Some(5));
    assert!(text(unknown_world.stderr).contains("unknown universe world or difficulty"));
}
