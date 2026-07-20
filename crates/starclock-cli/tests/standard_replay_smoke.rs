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
            0x58, 0x83, 0x6d, 0xc8, 0x53, 0xff, 0xe6, 0x26, 0x4e, 0x0b, 0xeb, 0x03, 0xf3, 0x32,
            0xab, 0x51, 0xe2, 0x2b, 0x6a, 0x4a, 0xef, 0x9f, 0x18, 0x17, 0xef, 0xe6, 0x99, 0x5e,
            0x20, 0xf6, 0x20, 0xa8,
        ])
    );
    assert_eq!(
        String::from_utf8(first_run.stdout).unwrap().trim(),
        "{\"schema_revision\":\"starclock-cli-v1\",\"kind\":\"battle-run\",\"scenario\":\"synthetic-standard-v1\",\"seed\":7,\"controller\":\"baseline\",\"commands\":3,\"phase\":\"won\",\"state_hash\":\"301b429a53c8772d026876a58603fdb4f53f662b21bee1a4029053c4fca1b4d2\",\"replay_bytes\":533}"
    );

    let verified = output(&["replay", "verify", first.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{:?}", verified);
    assert_eq!(
        String::from_utf8(verified.stdout).unwrap().trim(),
        "{\"schema_revision\":\"starclock-cli-v1\",\"kind\":\"replay-verify\",\"entry\":\"battle\",\"commands\":3,\"phase\":\"won\",\"state_hash\":\"301b429a53c8772d026876a58603fdb4f53f662b21bee1a4029053c4fca1b4d2\"}"
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
        "{\"schema_revision\":\"starclock-cli-v1\",\"kind\":\"battle-run\",\"scenario\":\"scenario.standard-v1.basic-single-wave\",\"seed\":104729,\"controller\":\"baseline\",\"commands\":9,\"phase\":\"won\",\"state_hash\":\"0293f04b6fb04dc020dd78db08dfa1284d430076ca6995683792820a9ac83e06\",\"replay_bytes\":991}"
    );
    let verified = output(&["replay", "verify", replay.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{:?}", verified);
    assert_eq!(
        String::from_utf8(verified.stdout).unwrap().trim(),
        "{\"schema_revision\":\"starclock-cli-v1\",\"kind\":\"replay-verify\",\"entry\":\"battle\",\"commands\":9,\"phase\":\"won\",\"state_hash\":\"0293f04b6fb04dc020dd78db08dfa1284d430076ca6995683792820a9ac83e06\"}"
    );
    fs::remove_file(replay).unwrap();
}
