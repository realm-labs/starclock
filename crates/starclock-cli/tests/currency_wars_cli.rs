use std::{fs, path::PathBuf, process::Command};

use serde_json::Value;
use starclock_replay::{format::decode_replay, record::RecordKind};

fn output(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_starclock"))
        .args(args)
        .output()
        .expect("starclock CLI launches")
}

fn fixture_path(suffix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "starclock-currency-wars-cli-{}-{suffix}.scrp",
        std::process::id()
    ))
}

#[test]
fn currency_wars_configuration_loads_production_catalog() {
    let output = Command::new(env!("CARGO_BIN_EXE_starclock"))
        .args(["currency-wars", "config", "validate", "--json"])
        .output()
        .expect("starclock CLI launches");
    assert!(output.status.success(), "{output:?}");
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["kind"], "currency-wars-config-validation");
    assert_eq!(report["valid"], true);
    assert_eq!(report["routes"], 26);
    assert_eq!(report["nodes"], 493);
    assert_eq!(report["difficulties"], 97);
    assert_eq!(report["roles"], 77);
    assert_eq!(report["bonds"], 49);
    assert_eq!(report["investments"], 834);
    assert_eq!(report["project_policies"], 12);
}

#[test]
fn currency_wars_route_inspection_exposes_direct_ids() {
    let output = Command::new(env!("CARGO_BIN_EXE_starclock"))
        .args(["currency-wars", "inspect", "--route", "100", "--json"])
        .output()
        .expect("starclock CLI launches");
    assert!(output.status.success(), "{output:?}");
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["kind"], "currency-wars-route");
    assert_eq!(report["route_id"], 100);
    let nodes = report["nodes"].as_array().unwrap();
    assert!(!nodes.is_empty());
    assert!(nodes[0]["id"].is_number());
    assert!(nodes[0]["node_template_id"].is_number());
    assert!(nodes[0]["encounter_id"].is_number());
}

#[test]
fn currency_wars_configuration_rejects_unknown_options_as_usage() {
    let output = Command::new(env!("CARGO_BIN_EXE_starclock"))
        .args(["currency-wars", "config", "validate", "--unknown"])
        .output()
        .expect("starclock CLI launches");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage error"));
}

#[test]
fn currency_wars_coverage_reports_current_terminal_and_pending_denominators() {
    let output = output(&["currency-wars", "coverage", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["kind"], "currency-wars-coverage");
    assert_eq!(report["source_obligations"], 19_250);
    assert_eq!(report["source_terminal"], 19_250);
    assert_eq!(report["source_pending"], 0);
    assert_eq!(report["mechanic_programs"], 2_367);
    assert_eq!(report["mechanic_terminal"], 2_367);
    assert_eq!(report["fixture_families"], 28);
    assert_eq!(report["project_policies"], 12);
    assert_eq!(report["native_handlers"], 0);
}

#[test]
fn currency_wars_standard_and_overclock_runs_export_fresh_verifiable_replays() {
    let standard = fixture_path("standard");
    let overclock = fixture_path("overclock");
    let corrupt = fixture_path("corrupt");
    for path in [&standard, &overclock, &corrupt] {
        let _ = fs::remove_file(path);
    }

    for (gambit, path) in [("standard", &standard), ("overclock", &overclock)] {
        let run = output(&[
            "currency-wars",
            "run",
            "--route",
            "801",
            "--difficulty",
            "1",
            "--gambit",
            gambit,
            "--seed",
            "31000501",
            "--controller",
            "baseline",
            "--replay-out",
            path.to_str().unwrap(),
            "--json",
        ]);
        assert!(run.status.success(), "{run:?}");
        let report: Value = serde_json::from_slice(&run.stdout).unwrap();
        assert_eq!(report["kind"], "currency-wars-run");
        assert_eq!(report["route"], 801);
        assert_eq!(report["difficulty"], 1);
        assert_eq!(report["gambit"], gambit);
        assert_eq!(report["nested_battles"], 7);
        assert_eq!(report["activity_actions"], 14);
        assert_eq!(report["terminal"], "completed");
        assert!(report["battle_commands"].as_u64().unwrap() > 0);

        let bytes = fs::read(path).unwrap();
        let replay = decode_replay(&bytes).unwrap();
        assert_eq!(replay.header().components().components().len(), 9);
        assert!(
            replay
                .records()
                .iter()
                .any(|record| record.kind() == RecordKind::AcceptedActivityCommand)
        );
        assert!(
            replay
                .records()
                .iter()
                .any(|record| record.kind() == RecordKind::AcceptedBattleCommand)
        );
    }

    let verified = output(&["replay", "verify", standard.to_str().unwrap(), "--json"]);
    assert!(verified.status.success(), "{verified:?}");
    let report: Value = serde_json::from_slice(&verified.stdout).unwrap();
    assert_eq!(report["kind"], "replay-verify");
    assert_eq!(report["entry"], "currency-wars");
    assert_eq!(report["nested_battles"], 7);
    assert_eq!(report["configuration_components"], 9);
    assert_eq!(report["activity_actions"], 14);

    let mut bytes = fs::read(&standard).unwrap();
    *bytes.last_mut().unwrap() ^= 1;
    fs::write(&corrupt, bytes).unwrap();
    let rejected = output(&["replay", "verify", corrupt.to_str().unwrap()]);
    assert_eq!(rejected.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("replay error"));

    for path in [&standard, &overclock, &corrupt] {
        fs::remove_file(path).unwrap();
    }
}
