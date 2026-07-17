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
            0x9f, 0x47, 0x44, 0xd5, 0x58, 0xb3, 0x73, 0xb8, 0x8c, 0xe4, 0xd7, 0x61, 0x05, 0xe2,
            0x04, 0xd1, 0x00, 0xab, 0xf2, 0x00, 0x87, 0xd2, 0x39, 0xdf, 0xd8, 0x6e, 0x0e, 0x70,
            0x84, 0xa3, 0x4a, 0xa4,
        ])
    );
    assert_eq!(
        String::from_utf8(first_run.stdout).unwrap().trim(),
        "{\"schema\":1,\"kind\":\"battle-run\",\"scenario\":\"synthetic-standard-v1\",\"seed\":7,\"commands\":3,\"phase\":\"won\",\"state_hash\":\"c9abd8e1c9aacc6634ade8958938affe4187d44a245307854e2b2c3ac8dbd869\",\"replay_bytes\":530}"
    );

    let verified = output(&["replay", "verify", first.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{:?}", verified);
    assert_eq!(
        String::from_utf8(verified.stdout).unwrap().trim(),
        "{\"schema\":1,\"kind\":\"replay-verify\",\"commands\":3,\"phase\":\"won\",\"state_hash\":\"c9abd8e1c9aacc6634ade8958938affe4187d44a245307854e2b2c3ac8dbd869\"}"
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
