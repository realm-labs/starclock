use std::{fs, path::PathBuf, process::Command};

use serde_json::Value;
use starclock_replay::{format::decode_replay, record::RecordKind};

fn output(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_starclock"))
        .args(args)
        .output()
        .expect("starclock CLI launches")
}

fn json(output: std::process::Output) -> Value {
    assert!(output.status.success(), "{output:?}");
    serde_json::from_slice(&output.stdout).expect("CLI emits JSON")
}

fn text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).expect("CLI emits UTF-8")
}

fn replay_path(suffix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "starclock-g22-p7-b2-{}-{suffix}.scrp",
        std::process::id()
    ))
}

#[test]
fn divergent_universe_configuration_and_coverage_are_machine_readable() {
    let validation = json(output(&[
        "universe",
        "config",
        "validate",
        "--mode",
        "divergent-universe",
        "--json",
    ]));
    assert_eq!(validation["kind"], "universe-config-validation");
    assert_eq!(validation["mode"], "divergent-universe");
    assert_eq!(validation["valid"], true);
    assert_eq!(validation["tables"], 81);
    assert_eq!(validation["rows"], 28_732);
    assert_eq!(validation["source_rows"], 8_171);
    assert_eq!(validation["source_obligations"], 6_762);
    assert_eq!(validation["mechanic_programs"], 669);

    let coverage = json(output(&[
        "universe",
        "coverage",
        "--mode",
        "divergent-universe",
        "--json",
    ]));
    assert_eq!(coverage["kind"], "universe-coverage");
    assert_eq!(
        coverage["source_obligations"],
        validation["source_obligations"]
    );
    assert_eq!(coverage["runtime_release_ready"], false);
    assert_eq!(coverage["coverage_status"], "behavioral-audit-incomplete");
    assert!(coverage.get("terminal").is_some_and(Value::is_null));
    assert!(coverage.get("pending").is_some_and(Value::is_null));
    assert_eq!(coverage["mechanic_programs"], 669);
    assert!(
        coverage
            .get("mechanic_executable")
            .is_some_and(Value::is_null)
    );
    assert_eq!(coverage["mechanic_metadata"], 3);
    assert_eq!(coverage["mechanic_excluded"], 3);
    assert_eq!(coverage["fixtures"], 25);
    assert_eq!(coverage["research_gaps"], 25);
    assert_eq!(coverage["policy_sources"], 54);
}

#[test]
fn divergent_universe_both_families_complete_and_replay_verifies() {
    for (family, seed) in [("ordinary", "22001"), ("cyclical", "22002")] {
        let replay = replay_path(family);
        let _ = fs::remove_file(&replay);

        let run = json(output(&[
            "universe",
            "run",
            "--mode",
            "divergent-universe",
            "--family",
            family,
            "--seed",
            seed,
            "--controller",
            "baseline",
            "--replay-out",
            replay.to_str().expect("temporary replay path is UTF-8"),
            "--json",
        ]));
        assert_eq!(run["kind"], "universe-run");
        assert_eq!(run["family"], family);
        assert_eq!(run["configuration_components"], 9);
        assert_eq!(run["nested_battles"], 3);
        assert!(
            run["battle_commands"]
                .as_u64()
                .is_some_and(|count| count > 0)
        );
        assert_eq!(run["terminal"], "completed");

        let replay_bytes = fs::read(&replay).expect("run writes replay");
        let decoded = decode_replay(&replay_bytes).expect("run writes canonical replay");
        assert_eq!(decoded.header().components().components().len(), 9);
        assert_eq!(
            decoded
                .records()
                .iter()
                .filter(|record| record.kind() == RecordKind::AcceptedActivityCommand)
                .count(),
            11 // Accepted entry plus ten player actions.
        );
        assert!(
            decoded
                .records()
                .iter()
                .any(|record| record.kind() == RecordKind::AcceptedBattleCommand)
        );

        let verified = json(output(&[
            "replay",
            "verify",
            replay.to_str().expect("temporary replay path is UTF-8"),
            "--json",
        ]));
        assert_eq!(verified["entry"], "divergent-universe");
        assert_eq!(verified["family"], family);
        assert_eq!(verified["configuration_components"], 9);
        assert_eq!(verified["state_hash"], run["state_hash"]);

        fs::remove_file(replay).expect("temporary replay can be removed");
    }
}

#[test]
fn divergent_universe_rejects_corrupt_replay_and_invalid_options() {
    let replay = replay_path("valid");
    let corrupt = replay_path("corrupt");
    for path in [&replay, &corrupt] {
        let _ = fs::remove_file(path);
    }
    let run = output(&[
        "universe",
        "run",
        "--mode",
        "divergent-universe",
        "--seed",
        "22003",
        "--replay-out",
        replay.to_str().expect("temporary replay path is UTF-8"),
    ]);
    assert!(run.status.success(), "{run:?}");

    let mut bytes = fs::read(&replay).expect("run writes replay");
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    fs::write(&corrupt, bytes).expect("corrupt replay can be written");
    let rejected = output(&[
        "replay",
        "verify",
        corrupt.to_str().expect("temporary replay path is UTF-8"),
    ]);
    assert_eq!(rejected.status.code(), Some(4));
    assert!(text(rejected.stderr).contains("divergent-universe replay error"));

    let invalid_family = output(&[
        "universe",
        "run",
        "--mode",
        "divergent-universe",
        "--family",
        "unknown",
        "--seed",
        "22004",
    ]);
    assert_eq!(invalid_family.status.code(), Some(2));
    assert!(text(invalid_family.stderr).contains("divergent-universe usage error"));

    fs::remove_file(replay).expect("temporary replay can be removed");
    fs::remove_file(corrupt).expect("temporary replay can be removed");
}

#[test]
fn divergent_universe_cli_tawot_entries_replay_in_a_fresh_process() {
    for (family, level) in [("ordinary", "2"), ("cyclical", "5")] {
        let replay = replay_path(&format!("tawot-{family}"));
        let run = json(output(&[
            "universe",
            "run",
            "--mode",
            "divergent-universe",
            "--family",
            family,
            "--seed",
            "24121",
            "--tawot-forge-level",
            level,
            "--replay-out",
            replay.to_str().unwrap(),
            "--json",
        ]));
        assert!(run["actions"].as_u64().unwrap() > 10);
        assert_eq!(run["tawot_forge_level"], level.parse::<u16>().unwrap());
        let verified = json(output(&[
            "replay",
            "verify",
            replay.to_str().unwrap(),
            "--json",
        ]));
        assert_eq!(verified["family"], family);
        assert_eq!(verified["tawot_forge_level"], run["tawot_forge_level"]);
        assert_eq!(verified["state_hash"], run["state_hash"]);
        fs::remove_file(replay).unwrap();
    }
    for level in ["0", "1", "6", "65536", "-1", "abc"] {
        let rejected = output(&[
            "universe",
            "run",
            "--mode",
            "divergent-universe",
            "--seed",
            "24121",
            "--tawot-forge-level",
            level,
        ]);
        assert_eq!(rejected.status.code(), Some(2));
    }
    let duplicate = output(&[
        "universe",
        "run",
        "--mode",
        "divergent-universe",
        "--seed",
        "24121",
        "--tawot-forge-level",
        "2",
        "--tawot-forge-level",
        "3",
    ]);
    assert_eq!(duplicate.status.code(), Some(2));
}
