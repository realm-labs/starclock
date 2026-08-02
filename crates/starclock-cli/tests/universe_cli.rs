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
        "{\"schema_revision\":\"starclock-cli-universe-v3\",\"kind\":\"universe-config-validation\",\"valid\":true,\"bundle_sha256\":\"5e5234ee3977f794ae9b1b833372f51c38408c205105c464f11827e9e9ae6a75\",\"worlds\":9,\"difficulties\":33,\"paths\":9,\"blessings\":162,\"curios\":61}"
    );

    let coverage = output(&["universe", "coverage", "--json"]);
    assert!(coverage.status.success(), "{coverage:?}");
    assert_eq!(
        text(coverage.stdout).trim(),
        "{\"schema_revision\":\"starclock-cli-universe-v3\",\"kind\":\"universe-coverage\",\"goal_id\":\"standard-universe-runtime-v1\",\"content_records\":2201,\"rule_bindings\":786,\"fixtures\":78,\"worlds\":9,\"difficulties\":33,\"paths\":9,\"encounter_groups\":74}"
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
        "{\"schema_revision\":\"starclock-cli-gold-and-gears-v1\",\"kind\":\"universe-config-validation\",\"mode\":\"gold-and-gears\",\"valid\":true,\"bundle_sha256\":\"97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b\",\"tables\":52,\"rows\":29140,\"source_obligations\":7913,\"mechanic_rules\":1224,\"fixtures\":18,\"policy_boundaries\":16}"
    );

    let coverage = output(&["universe", "coverage", "--mode", "gold-and-gears", "--json"]);
    assert!(coverage.status.success(), "{coverage:?}");
    assert_eq!(
        text(coverage.stdout).trim(),
        "{\"schema_revision\":\"starclock-cli-gold-and-gears-v1\",\"kind\":\"universe-coverage\",\"mode\":\"gold-and-gears\",\"goal_id\":\"gold-and-gears-runtime-v1\",\"source_categories\":42,\"runtime_slices\":44,\"source_obligations\":7913,\"integrated\":7181,\"shared_integrated\":706,\"external_outcomes\":8,\"metadata\":18,\"mechanic_rules\":1224,\"fixtures\":18,\"native_handlers\":0,\"coverage_digest\":\"f2d927d197cb77c548522bf39383a68e927f3881412f44dee8a0b4302c38ca9d\"}"
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
        "{\"schema_revision\":\"starclock-cli-swarm-disaster-v1\",\"kind\":\"universe-config-validation\",\"mode\":\"swarm-disaster\",\"valid\":true,\"bundle_sha256\":\"385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362\",\"tables\":65,\"rows\":33380,\"source_obligations\":6963,\"mechanic_rules\":23,\"fixtures\":23,\"policy_boundaries\":31}"
    );

    let coverage = output(&["universe", "coverage", "--mode", "swarm-disaster", "--json"]);
    assert!(coverage.status.success(), "{coverage:?}");
    assert_eq!(
        text(coverage.stdout).trim(),
        "{\"schema_revision\":\"starclock-cli-swarm-disaster-v1\",\"kind\":\"universe-coverage\",\"mode\":\"swarm-disaster\",\"goal_id\":\"swarm-disaster-runtime-v1\",\"source_categories\":42,\"runtime_slices\":42,\"source_obligations\":6963,\"integrated\":6282,\"shared_integrated\":652,\"external_outcomes\":6,\"metadata\":23,\"mechanic_rules\":23,\"fixtures\":23,\"native_handlers\":0,\"coverage_digest\":\"8aeb60d2c1b322f9dcf8f84bc45dc1901276633398cdb60a984ccc4846f0bff4\"}"
    );
}

#[test]
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
        "universe coverage mode=gold-and-gears goal=gold-and-gears-runtime-v1 categories=42 slices=44 source_obligations=7913 integrated=7181 shared_integrated=706 external_outcomes=8 metadata=18 rules=1224 fixtures=18 native_handlers=0 digest=f2d927d197cb77c548522bf39383a68e927f3881412f44dee8a0b4302c38ca9d"
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
        "universe completed mode=gold-and-gears seed=14001 profile=gold-gears.profile.v1 controller=baseline battle_executor=gold-and-gears-nested-battle-execution-v1 fixture_accuracy=SyntheticBalanceIndependentNotObservedNumericParity component_root=e52ba8dc22197daa70cbdc6e40f9327bc757e12bd17ae11a8fe65c410c780dc3 actions=62 nested_battles=17 battle_commands=97 hash=aa084c9c37e8c3b251fa3e97c6145668997a8160b9db2d7264a5e53c767f8455 replay_bytes=107359 replay_sha256=71ad733fb0c1a222d70cfd76f755bab65e23f1ca13ea81c3b612e74d0dc277ac"
    );
}

#[test]
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
        "universe coverage mode=swarm-disaster goal=swarm-disaster-runtime-v1 categories=42 slices=42 source_obligations=6963 integrated=6282 shared_integrated=652 external_outcomes=6 metadata=23 rules=23 fixtures=23 native_handlers=0 digest=8aeb60d2c1b322f9dcf8f84bc45dc1901276633398cdb60a984ccc4846f0bff4"
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
        "universe completed mode=swarm-disaster seed=20001 profile=swarm-disaster.profile.v1 controller=baseline battle_executor=swarm-disaster-nested-battle-execution-v1 fixture_accuracy=SyntheticBalanceIndependentNotObservedNumericParity component_root=a87894170e22188cb00078c339e806a6e3387f5e49baf7fd7782f6f0732c823c actions=48 nested_battles=12 battle_commands=68 hash=eb870454531b7d109bd43cef38f5d320df85dbbb76ce9732c4eca022a4881075 replay_bytes=81107 replay_sha256=d052a392d91dd93e9e8baf44b80940fb9a57111384b052332f6c21ad869a73a4"
    );
}

#[test]
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
        "{\"schema_revision\":\"starclock-cli-universe-v3\",\"kind\":\"universe-run\",\"world\":1,\"difficulty_index\":0,\"seed\":1,\"controller\":\"baseline\",\"battle_executor\":\"standard-universe-nested-battle-executor-v1\",\"actions\":35,\"nested_battles\":3,\"battle_commands\":17,\"terminal\":\"completed\",\"state_hash\":\"64078b94531239bc81096249bb7cc79b8f8a8dbddf8a8cc95b497f3de947c73b\",\"replay_bytes\":25678}"
    );

    let replay_bytes = fs::read(&replay).unwrap();
    assert_eq!(replay_bytes.len(), 25_678);
    let decoded = starclock_replay::format_v3::decode_replay_v3(&replay_bytes).unwrap();
    assert_eq!(decoded.header().components().components().len(), 9);
    assert!(decoded.records().iter().any(|record| {
        record.kind() == starclock_replay::record::RecordKind::AcceptedBattleCommand
    }));
    assert!(starclock_replay::format::decode_replay(&replay_bytes).is_err());
    let mut replay_hash = Sha256Sink::new();
    replay_hash.write(&replay_bytes);
    assert_eq!(
        replay_hash.finalize(),
        Sha256Digest::new([
            7, 246, 31, 139, 107, 155, 77, 87, 245, 102, 39, 127, 81, 207, 12, 253, 199, 186, 167,
            147, 90, 157, 247, 114, 122, 207, 189, 170, 142, 83, 191, 137,
        ])
    );

    let verified = output(&["replay", "verify", replay.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{verified:?}");
    assert_eq!(
        text(verified.stdout).trim(),
        "{\"schema_revision\":\"starclock-cli-universe-v3\",\"kind\":\"replay-verify\",\"entry\":\"standard-universe\",\"actions\":35,\"nested_battles\":3,\"battle_commands\":17,\"terminal\":\"completed\",\"state_hash\":\"64078b94531239bc81096249bb7cc79b8f8a8dbddf8a8cc95b497f3de947c73b\"}"
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
        "{\"schema_revision\":\"starclock-cli-gold-and-gears-v1\",\"kind\":\"universe-run\",\"mode\":\"gold-and-gears\",\"seed\":14001,\"profile\":\"gold-gears.profile.v1\",\"area\":\"gold-gears.area.401\",\"path\":\"universe.path.abundance\",\"custom_dice\":\"gold-gears.custom-dice.101\",\"controller\":\"baseline\",\"battle_executor\":\"gold-and-gears-nested-battle-execution-v1\",\"fixture_accuracy\":\"SyntheticBalanceIndependentNotObservedNumericParity\",\"component_root\":\"e52ba8dc22197daa70cbdc6e40f9327bc757e12bd17ae11a8fe65c410c780dc3\",\"actions\":62,\"nested_battles\":17,\"battle_commands\":97,\"terminal\":\"completed\",\"state_hash\":\"aa084c9c37e8c3b251fa3e97c6145668997a8160b9db2d7264a5e53c767f8455\",\"replay_bytes\":107359,\"replay_sha256\":\"71ad733fb0c1a222d70cfd76f755bab65e23f1ca13ea81c3b612e74d0dc277ac\"}"
    );

    let replay_bytes = fs::read(&replay).unwrap();
    assert_eq!(replay_bytes.len(), 107_359);
    let decoded = starclock_replay::format_v2::decode_replay_v2(&replay_bytes).unwrap();
    assert_eq!(decoded.header().components().components().len(), 10);
    assert!(decoded.records().iter().any(|record| {
        record.kind() == starclock_replay::record::RecordKind::AcceptedBattleCommand
    }));

    let verified = output(&["replay", "verify", replay.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{verified:?}");
    assert_eq!(
        text(verified.stdout).trim(),
        "{\"schema_revision\":\"starclock-cli-gold-and-gears-v1\",\"kind\":\"replay-verify\",\"entry\":\"gold-and-gears\",\"actions\":62,\"nested_battles\":17,\"battle_commands\":97,\"terminal\":\"completed\",\"state_hash\":\"aa084c9c37e8c3b251fa3e97c6145668997a8160b9db2d7264a5e53c767f8455\"}"
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
        "{\"schema_revision\":\"starclock-cli-swarm-disaster-v1\",\"kind\":\"universe-run\",\"mode\":\"swarm-disaster\",\"seed\":20001,\"profile\":\"swarm-disaster.profile.v1\",\"area\":\"swarm-disaster.area.201\",\"path\":\"universe.path.preservation\",\"audience_die\":\"swarm-disaster.audience-die.1\",\"controller\":\"baseline\",\"battle_executor\":\"swarm-disaster-nested-battle-execution-v1\",\"fixture_accuracy\":\"SyntheticBalanceIndependentNotObservedNumericParity\",\"component_root\":\"a87894170e22188cb00078c339e806a6e3387f5e49baf7fd7782f6f0732c823c\",\"actions\":48,\"nested_battles\":12,\"battle_commands\":68,\"terminal\":\"completed\",\"state_hash\":\"eb870454531b7d109bd43cef38f5d320df85dbbb76ce9732c4eca022a4881075\",\"replay_bytes\":81107,\"replay_sha256\":\"d052a392d91dd93e9e8baf44b80940fb9a57111384b052332f6c21ad869a73a4\"}"
    );

    let replay_bytes = fs::read(&replay).unwrap();
    assert_eq!(replay_bytes.len(), 81_107);
    let decoded = starclock_replay::format_v2::decode_replay_v2(&replay_bytes).unwrap();
    assert_eq!(decoded.header().components().components().len(), 10);
    assert!(decoded.records().iter().any(|record| {
        record.kind() == starclock_replay::record::RecordKind::AcceptedBattleCommand
    }));

    let verified = output(&["replay", "verify", replay.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{verified:?}");
    assert_eq!(
        text(verified.stdout).trim(),
        "{\"schema_revision\":\"starclock-cli-swarm-disaster-v1\",\"kind\":\"replay-verify\",\"entry\":\"swarm-disaster\",\"actions\":48,\"nested_battles\":12,\"battle_commands\":68,\"terminal\":\"completed\",\"state_hash\":\"eb870454531b7d109bd43cef38f5d320df85dbbb76ce9732c4eca022a4881075\"}"
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
