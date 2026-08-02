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
        "starclock-g07-p6-b2-{}-{suffix}.scrp",
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
            "synthetic-standard",
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
            0xfa, 0xd8, 0xcc, 0xbf, 0xc1, 0xf5, 0x03, 0xef, 0xae, 0x8e, 0x2a, 0x4e, 0x5f, 0xaf,
            0xcb, 0x51, 0x7d, 0x5e, 0x33, 0x16, 0x47, 0xea, 0x2c, 0x15, 0x39, 0x25, 0x0b, 0x75,
            0x7c, 0xe6, 0x1d, 0x83,
        ])
    );
    assert_eq!(
        String::from_utf8(first_run.stdout).unwrap().trim(),
        "{\"kind\":\"battle-run\",\"scenario\":\"synthetic-standard\",\"seed\":7,\"controller\":\"baseline\",\"commands\":3,\"phase\":\"won\",\"state_hash\":\"2f61f927be14e24df81813e49403e5367fdb51301c6e820a6e276676773e35e8\",\"replay_bytes\":456}"
    );

    let verified = output(&["replay", "verify", first.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{:?}", verified);
    assert_eq!(
        String::from_utf8(verified.stdout).unwrap().trim(),
        "{\"kind\":\"replay-verify\",\"entry\":\"battle\",\"commands\":3,\"phase\":\"won\",\"state_hash\":\"2f61f927be14e24df81813e49403e5367fdb51301c6e820a6e276676773e35e8\"}"
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
    let replay = fixture_path("public-standard");
    let _ = fs::remove_file(&replay);
    let run = output(&[
        "battle",
        "run",
        "--scenario",
        "scenario.standard.basic-single-wave",
        "--seed",
        "104729",
        "--replay-out",
        replay.to_str().unwrap(),
        "--json",
    ]);
    assert!(run.status.success(), "{:?}", run);
    assert_eq!(
        String::from_utf8(run.stdout).unwrap().trim(),
        "{\"kind\":\"battle-run\",\"scenario\":\"scenario.standard.basic-single-wave\",\"seed\":104729,\"controller\":\"baseline\",\"commands\":21,\"phase\":\"won\",\"state_hash\":\"eb95d3eba8dbb2cd53258e5e174bbb8f6e744c557d4693a65951c4876d7b6178\",\"replay_bytes\":1880}"
    );
    let verified = output(&["replay", "verify", replay.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{:?}", verified);
    assert_eq!(
        String::from_utf8(verified.stdout).unwrap().trim(),
        "{\"kind\":\"replay-verify\",\"entry\":\"battle\",\"commands\":21,\"phase\":\"won\",\"state_hash\":\"eb95d3eba8dbb2cd53258e5e174bbb8f6e744c557d4693a65951c4876d7b6178\"}"
    );
    fs::remove_file(replay).unwrap();
}
