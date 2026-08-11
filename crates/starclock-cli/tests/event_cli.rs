use std::process::Command;

use serde_json::Value;

#[test]
fn event_configuration_loads_both_production_profiles() {
    let output = Command::new(env!("CARGO_BIN_EXE_starclock"))
        .args(["event", "config", "validate", "--json"])
        .output()
        .expect("starclock CLI launches");
    assert!(output.status.success(), "{output:?}");
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["kind"], "event-config-validation");
    assert_eq!(report["valid"], true);
    assert_eq!(report["bundle_sha256"].as_str().unwrap().len(), 64);
    assert_eq!(report["modes"].as_array().unwrap().len(), 2);
    assert_eq!(report["modes"][0]["stage_periods"], 102);
    assert_eq!(report["modes"][0]["shop_upgrades"], 114);
    assert_eq!(report["modes"][0]["strategies"], 56);
    assert_eq!(report["modes"][0]["team_bonuses"], 7);
    assert_eq!(report["modes"][0]["policies"], 6);
    assert_eq!(report["modes"][1]["boards"], 6);
    assert_eq!(report["modes"][1]["owners"], 6);
    assert_eq!(report["modes"][1]["decks"], 4);
    assert_eq!(report["modes"][1]["deck_recommendations"], 7);
    assert_eq!(report["modes"][1]["cards"], 107);
    assert_eq!(report["modes"][1]["story_fights"], 6);
    assert_eq!(report["modes"][1]["challenge_fights"], 4);
    assert_eq!(report["modes"][1]["map_fights"], 15);
    assert_eq!(report["modes"][1]["policies"], 16);
}

#[test]
fn event_configuration_rejects_unknown_options_as_usage() {
    let output = Command::new(env!("CARGO_BIN_EXE_starclock"))
        .args(["event", "config", "validate", "--unknown"])
        .output()
        .expect("starclock CLI launches");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage error"));
}
