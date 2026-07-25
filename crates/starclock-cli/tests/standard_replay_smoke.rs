use std::{fs, path::PathBuf, process::Command};

use starclock_replay::{
    codec::CanonicalSink,
    digest::{Sha256Digest, Sha256Sink},
    format::decode_replay,
};

fn output(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_starclock"))
        .args(args)
        .output()
        .expect("starclock CLI launches")
}

fn fixture_path(suffix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "starclock-g01-p3-b6-{}-{suffix}.scrp",
        std::process::id()
    ))
}

#[test]
fn cli_runs_writes_replays_reproduces_bytes_and_detects_divergence() {
    let first = fixture_path("first");
    let second = fixture_path("second");
    let corrupt = fixture_path("corrupt");
    for path in [&first, &second, &corrupt] {
        let _ = fs::remove_file(path);
    }

    let run = |path: &PathBuf| {
        output(&[
            "battle",
            "run",
            "--scenario",
            "synthetic-standard-v1",
            "--seed",
            "7",
            "--replay-out",
            path.to_str().unwrap(),
            "--json",
        ])
    };
    let first_run = run(&first);
    let second_run = run(&second);
    assert!(first_run.status.success(), "{:?}", first_run);
    assert!(second_run.status.success(), "{:?}", second_run);
    assert_eq!(first_run.stdout, second_run.stdout);
    let replay_bytes = fs::read(&first).unwrap();
    assert_eq!(replay_bytes, fs::read(&second).unwrap());
    let mut replay_hash = Sha256Sink::new();
    replay_hash.write(&replay_bytes);
    assert_eq!(
        replay_hash.finalize(),
        Sha256Digest::new([
            0x4a, 0x90, 0x39, 0x18, 0x16, 0x01, 0xd2, 0x0f, 0xd6, 0x80, 0x16, 0x08, 0xf6, 0x98,
            0x2e, 0xbc, 0xe2, 0x68, 0xbc, 0x18, 0x22, 0x35, 0xbf, 0x4f, 0x1e, 0x12, 0xb3, 0x7c,
            0xcd, 0x7e, 0x68, 0x82,
        ])
    );
    assert_eq!(
        String::from_utf8(first_run.stdout).unwrap().trim(),
        "{\"schema_revision\":\"starclock-cli-v1\",\"kind\":\"battle-run\",\"scenario\":\"synthetic-standard-v1\",\"seed\":7,\"controller\":\"baseline\",\"commands\":3,\"phase\":\"won\",\"state_hash\":\"3e53412dc2c0fe545c3f71d39dc97cf42e98c4fa2c93af2efc6b4c45f9a40af5\",\"replay_bytes\":533}"
    );

    let verified = output(&["replay", "verify", first.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{:?}", verified);
    assert_eq!(
        String::from_utf8(verified.stdout).unwrap().trim(),
        "{\"schema_revision\":\"starclock-cli-v1\",\"kind\":\"replay-verify\",\"entry\":\"battle\",\"commands\":3,\"phase\":\"won\",\"state_hash\":\"3e53412dc2c0fe545c3f71d39dc97cf42e98c4fa2c93af2efc6b4c45f9a40af5\"}"
    );

    let mut bytes = replay_bytes;
    let first_expected_hash = decode_replay(&bytes).unwrap().records()[1]
        .payload()
        .to_vec();
    let hash_offset = bytes
        .windows(first_expected_hash.len())
        .position(|window| window == first_expected_hash)
        .unwrap();
    bytes[hash_offset] ^= 1;
    fs::write(&corrupt, bytes).unwrap();
    let rejected = output(&["replay", "verify", corrupt.to_str().unwrap()]);
    assert_eq!(rejected.status.code(), Some(4));
    assert!(
        String::from_utf8(rejected.stderr)
            .unwrap()
            .contains("command_index: 0")
    );

    for path in [&first, &second, &corrupt] {
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn cli_runs_and_verifies_the_frozen_public_standard_scenario() {
    let replay = fixture_path("public-standard-v1");
    let _ = fs::remove_file(&replay);
    let run = output(&[
        "battle",
        "run",
        "--scenario",
        "scenario.standard-v1.basic-single-wave",
        "--seed",
        "104729",
        "--replay-out",
        replay.to_str().unwrap(),
        "--json",
    ]);
    assert!(run.status.success(), "{:?}", run);
    assert_eq!(
        String::from_utf8(run.stdout).unwrap().trim(),
        "{\"schema_revision\":\"starclock-cli-v1\",\"kind\":\"battle-run\",\"scenario\":\"scenario.standard-v1.basic-single-wave\",\"seed\":104729,\"controller\":\"baseline\",\"commands\":9,\"phase\":\"won\",\"state_hash\":\"2d2bcb50b70ad488cf17744c9fc8082dca3c2b66aa51eca00ee1327c2359fefe\",\"replay_bytes\":991}"
    );
    let verified = output(&["replay", "verify", replay.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{:?}", verified);
    assert_eq!(
        String::from_utf8(verified.stdout).unwrap().trim(),
        "{\"schema_revision\":\"starclock-cli-v1\",\"kind\":\"replay-verify\",\"entry\":\"battle\",\"commands\":9,\"phase\":\"won\",\"state_hash\":\"2d2bcb50b70ad488cf17744c9fc8082dca3c2b66aa51eca00ee1327c2359fefe\"}"
    );
    fs::remove_file(replay).unwrap();
}
