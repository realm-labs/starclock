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
        "starclock-g05-p4-b2-{}-{suffix}.scrp",
        std::process::id()
    ))
}

#[test]
fn universe_configuration_and_coverage_are_machine_readable() {
    let validation = output(&["universe", "config", "validate", "--json"]);
    assert!(validation.status.success(), "{validation:?}");
    assert_eq!(
        text(validation.stdout).trim(),
        "{\"schema_revision\":\"starclock-cli-universe-v3\",\"kind\":\"universe-config-validation\",\"valid\":true,\"bundle_sha256\":\"ffffa6a539987e69bf0fe8f8b564044285cfe8b905103011561ecac8b0962bf5\",\"worlds\":9,\"difficulties\":33,\"paths\":9,\"blessings\":162,\"curios\":61}"
    );

    let coverage = output(&["universe", "coverage", "--json"]);
    assert!(coverage.status.success(), "{coverage:?}");
    assert_eq!(
        text(coverage.stdout).trim(),
        "{\"schema_revision\":\"starclock-cli-universe-v3\",\"kind\":\"universe-coverage\",\"goal_id\":\"standard-universe-runtime-v1\",\"content_records\":2201,\"rule_bindings\":786,\"fixtures\":78,\"worlds\":9,\"difficulties\":33,\"paths\":9,\"encounter_groups\":74}"
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
        "10",
        "--controller",
        "baseline",
        "--replay-out",
        replay.to_str().unwrap(),
        "--json",
    ]);
    assert!(run.status.success(), "{run:?}");
    assert_eq!(
        text(run.stdout).trim(),
        "{\"schema_revision\":\"starclock-cli-universe-v3\",\"kind\":\"universe-run\",\"world\":1,\"difficulty_index\":0,\"seed\":10,\"controller\":\"baseline\",\"battle_executor\":\"standard-universe-nested-battle-executor-v1\",\"actions\":61,\"nested_battles\":5,\"battle_commands\":29,\"terminal\":\"completed\",\"state_hash\":\"628f786ae991c21ddff45ab0dc499a34835ffede9d93e79675dc79591d060818\",\"replay_bytes\":49675}"
    );

    let replay_bytes = fs::read(&replay).unwrap();
    assert_eq!(replay_bytes.len(), 49_675);
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
            198, 56, 163, 32, 78, 34, 118, 13, 53, 139, 27, 146, 72, 254, 196, 89, 140, 178, 230,
            91, 222, 45, 175, 54, 193, 190, 206, 135, 113, 229, 133, 207,
        ])
    );

    let verified = output(&["replay", "verify", replay.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{verified:?}");
    assert_eq!(
        text(verified.stdout).trim(),
        "{\"schema_revision\":\"starclock-cli-universe-v3\",\"kind\":\"replay-verify\",\"entry\":\"standard-universe\",\"actions\":61,\"nested_battles\":5,\"battle_commands\":29,\"terminal\":\"completed\",\"state_hash\":\"628f786ae991c21ddff45ab0dc499a34835ffede9d93e79675dc79591d060818\"}"
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
