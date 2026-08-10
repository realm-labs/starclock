use std::process::Command;

use serde_json::Value;

fn output(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_starclock"))
        .args(args)
        .output()
        .expect("starclock CLI launches")
}

#[test]
fn challenge_configuration_compiles_all_three_production_catalogs() {
    let validation = output(&["challenge", "config", "validate", "--json"]);
    assert!(validation.status.success(), "{validation:?}");
    let report: Value = serde_json::from_slice(&validation.stdout).unwrap();
    assert_eq!(report["kind"], "challenge-config-validation");
    assert_eq!(report["valid"], true);
    assert_eq!(report["bundle_sha256"].as_str().unwrap().len(), 64);
    assert_eq!(report["modes"].as_array().unwrap().len(), 3);

    let expected = [
        ("memory-of-chaos", 13, 27, 25, 8, 22),
        ("apocalyptic-shadow", 5, 11, 9, 5, 10),
        ("pure-fiction", 5, 11, 9, 5, 30),
    ];
    for (mode, stages, nodes, encounters, policies, approximate_enemies) in expected {
        let actual = report["modes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["mode"] == mode)
            .unwrap();
        assert_eq!(actual["stages"], stages);
        assert_eq!(actual["nodes"], nodes);
        assert_eq!(actual["encounters"], encounters);
        assert_eq!(actual["policies"], policies);
        assert_eq!(actual["approximate_enemies"], approximate_enemies);
    }
}

#[test]
fn challenge_configuration_rejects_unknown_options() {
    let rejected = output(&["challenge", "config", "validate", "--unknown"]);
    assert_eq!(rejected.status.code(), Some(2));
    assert!(rejected.stdout.is_empty());
    assert!(
        String::from_utf8(rejected.stderr)
            .unwrap()
            .contains("usage error")
    );
}
