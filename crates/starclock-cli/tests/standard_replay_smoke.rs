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
            0x2e, 0x05, 0x6e, 0x47, 0x5b, 0xb9, 0x03, 0x2b, 0xa9, 0xfe, 0x3d, 0x71, 0x13, 0xbe,
            0x3b, 0x32, 0x30, 0x56, 0x86, 0x1c, 0xcd, 0xd4, 0x0a, 0x63, 0x5f, 0xf2, 0xde, 0xf3,
            0x9f, 0xee, 0x9e, 0xfc,
        ])
    );
    assert_eq!(
        String::from_utf8(first_run.stdout).unwrap().trim(),
        "{\"schema\":1,\"kind\":\"battle-run\",\"scenario\":\"synthetic-standard-v1\",\"seed\":7,\"commands\":3,\"phase\":\"won\",\"state_hash\":\"d1c7fcb16df5dabca17690888f44842b915a53ec0e5f80df4536d61e592cff67\",\"replay_bytes\":530}"
    );

    let verified = output(&["replay", "verify", first.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{:?}", verified);
    assert_eq!(
        String::from_utf8(verified.stdout).unwrap().trim(),
        "{\"schema\":1,\"kind\":\"replay-verify\",\"commands\":3,\"phase\":\"won\",\"state_hash\":\"d1c7fcb16df5dabca17690888f44842b915a53ec0e5f80df4536d61e592cff67\"}"
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
