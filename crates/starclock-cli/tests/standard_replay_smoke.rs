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
        "starclock-cli-replay-{}-{suffix}.scrp",
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
            0x11, 0x93, 0xde, 0xf5, 0x30, 0xbd, 0xb3, 0x21, 0x7f, 0x60, 0x10, 0x06, 0x7e, 0xfa,
            0xa9, 0xce, 0x53, 0x93, 0xef, 0x55, 0xce, 0x01, 0xd4, 0x2c, 0xba, 0x3a, 0xc1, 0x71,
            0x39, 0x36, 0xcb, 0xec,
        ])
    );
    assert_eq!(
        String::from_utf8(first_run.stdout).unwrap().trim(),
        "{\"kind\":\"battle-run\",\"scenario\":\"synthetic-standard\",\"seed\":7,\"controller\":\"baseline\",\"commands\":3,\"phase\":\"won\",\"state_hash\":\"7a4a4784658a86de69941cf265a9e10e5e3b9f086453568e39eb923f8f803970\",\"replay_bytes\":442}"
    );

    let verified = output(&["replay", "verify", first.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{:?}", verified);
    assert_eq!(
        String::from_utf8(verified.stdout).unwrap().trim(),
        "{\"kind\":\"replay-verify\",\"entry\":\"battle\",\"commands\":3,\"phase\":\"won\",\"state_hash\":\"7a4a4784658a86de69941cf265a9e10e5e3b9f086453568e39eb923f8f803970\"}"
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
fn cli_runs_and_verifies_the_public_standard_scenario() {
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
        "{\"kind\":\"battle-run\",\"scenario\":\"scenario.standard.basic-single-wave\",\"seed\":104729,\"controller\":\"baseline\",\"commands\":21,\"phase\":\"won\",\"state_hash\":\"cdb68a534f2cbb67393e55dacd2f052c42cc331d494db369ca4650188abcd474\",\"replay_bytes\":1830}"
    );
    let verified = output(&["replay", "verify", replay.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{:?}", verified);
    assert_eq!(
        String::from_utf8(verified.stdout).unwrap().trim(),
        "{\"kind\":\"replay-verify\",\"entry\":\"battle\",\"commands\":21,\"phase\":\"won\",\"state_hash\":\"cdb68a534f2cbb67393e55dacd2f052c42cc331d494db369ca4650188abcd474\"}"
    );
    fs::remove_file(replay).unwrap();
}
