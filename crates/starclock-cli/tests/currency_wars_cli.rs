use std::process::Command;

use serde_json::Value;

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
