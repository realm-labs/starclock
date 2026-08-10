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
            0x77, 0x60, 0xc5, 0xa1, 0x3a, 0x24, 0xba, 0xf1, 0x55, 0xa9, 0xb5, 0x12, 0xa7, 0xab,
            0xe9, 0xc3, 0xc3, 0xb0, 0x5d, 0x95, 0xde, 0xb4, 0xbf, 0x6f, 0x65, 0x0b, 0x28, 0xe4,
            0x30, 0x61, 0x5f, 0x11,
        ])
    );
    assert_eq!(
        String::from_utf8(first_run.stdout).unwrap().trim(),
        "{\"kind\":\"battle-run\",\"scenario\":\"synthetic-standard\",\"seed\":7,\"controller\":\"baseline\",\"commands\":2,\"phase\":\"won\",\"state_hash\":\"65b941b53e8307c57848f50a31762065f8ecb182cb84979ba0edccf8679d0db8\",\"replay_bytes\":375}"
    );

    let verified = output(&["replay", "verify", first.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{:?}", verified);
    assert_eq!(
        String::from_utf8(verified.stdout).unwrap().trim(),
        "{\"kind\":\"replay-verify\",\"entry\":\"battle\",\"commands\":2,\"phase\":\"won\",\"state_hash\":\"65b941b53e8307c57848f50a31762065f8ecb182cb84979ba0edccf8679d0db8\"}"
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
        "{\"kind\":\"battle-run\",\"scenario\":\"scenario.standard.basic-single-wave\",\"seed\":104729,\"controller\":\"baseline\",\"commands\":20,\"phase\":\"won\",\"state_hash\":\"bc675b17a0b641155e3ad6fc102a1d2b5921923049cee22e3a830cc2a8d14c06\",\"replay_bytes\":1763}"
    );
    let verified = output(&["replay", "verify", replay.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{:?}", verified);
    assert_eq!(
        String::from_utf8(verified.stdout).unwrap().trim(),
        "{\"kind\":\"replay-verify\",\"entry\":\"battle\",\"commands\":20,\"phase\":\"won\",\"state_hash\":\"bc675b17a0b641155e3ad6fc102a1d2b5921923049cee22e3a830cc2a8d14c06\"}"
    );
    fs::remove_file(replay).unwrap();
}
