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
            0x05, 0x07, 0x03, 0x02, 0xc5, 0xc5, 0x2c, 0x7a, 0xb4, 0x02, 0x4e, 0xb3, 0x2d, 0x3c,
            0x14, 0x27, 0x79, 0x66, 0xf2, 0x83, 0xc1, 0x66, 0xf1, 0x1f, 0x48, 0xea, 0x5f, 0xff,
            0x3f, 0xbb, 0xbb, 0xcf,
        ])
    );
    assert_eq!(
        String::from_utf8(first_run.stdout).unwrap().trim(),
        "{\"schema_revision\":\"starclock-cli-v1\",\"kind\":\"battle-run\",\"scenario\":\"synthetic-standard-v1\",\"seed\":7,\"controller\":\"baseline\",\"commands\":3,\"phase\":\"won\",\"state_hash\":\"718e40a44ae42b9e8be1510bf9900d394eb129dfbd8dc6bcd8500f9122b8679c\",\"replay_bytes\":533}"
    );

    let verified = output(&["replay", "verify", first.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{:?}", verified);
    assert_eq!(
        String::from_utf8(verified.stdout).unwrap().trim(),
        "{\"schema_revision\":\"starclock-cli-v1\",\"kind\":\"replay-verify\",\"entry\":\"battle\",\"commands\":3,\"phase\":\"won\",\"state_hash\":\"718e40a44ae42b9e8be1510bf9900d394eb129dfbd8dc6bcd8500f9122b8679c\"}"
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
        "{\"schema_revision\":\"starclock-cli-v1\",\"kind\":\"battle-run\",\"scenario\":\"scenario.standard-v1.basic-single-wave\",\"seed\":104729,\"controller\":\"baseline\",\"commands\":9,\"phase\":\"won\",\"state_hash\":\"ab50a79228c9387e26abf88600a729baf438b2e94bfb281edb5fb7da1992a3d0\",\"replay_bytes\":991}"
    );
    let verified = output(&["replay", "verify", replay.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{:?}", verified);
    assert_eq!(
        String::from_utf8(verified.stdout).unwrap().trim(),
        "{\"schema_revision\":\"starclock-cli-v1\",\"kind\":\"replay-verify\",\"entry\":\"battle\",\"commands\":9,\"phase\":\"won\",\"state_hash\":\"ab50a79228c9387e26abf88600a729baf438b2e94bfb281edb5fb7da1992a3d0\"}"
    );
    fs::remove_file(replay).unwrap();
}
