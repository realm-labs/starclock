use std::{fs, path::PathBuf, process::Command};

fn output(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_starclock"))
        .args(args)
        .output()
        .expect("starclock CLI launches")
}

fn text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).expect("CLI emits UTF-8")
}

fn temporary(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("starclock-g01-p6-b5-{}-{name}", std::process::id()))
}

#[test]
fn config_validation_uses_only_a_validated_sora_bundle() {
    let default = output(&["config", "validate", "--json"]);
    assert!(default.status.success(), "{:?}", default);
    assert_eq!(
        text(default.stdout).trim(),
        "{\"schema_revision\":\"starclock-cli-v1\",\"kind\":\"config-validation\",\"valid\":true,\"game_version\":\"4.4\",\"data_revision\":\"core-combat-v1-phase7-c03\",\"bundle_sha256\":\"006c5eb393cc22abb767ea42d69159fa4238e346a445909938c7952d4db65eb8\",\"identities\":1838,\"enabled\":1615}"
    );

    let bundle =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/generated/config.sora");
    let explicit = output(&["config", "validate", "--bundle", bundle.to_str().unwrap()]);
    assert!(explicit.status.success(), "{:?}", explicit);
    let human = text(explicit.stdout);
    assert!(human.contains("config valid game_version=4.4"));
    assert!(human.contains("identities=1838 enabled=1615"));

    let invalid = temporary("invalid.sora");
    fs::write(&invalid, br#"{\"debug\":\"json\"}"#).unwrap();
    let rejected = output(&["config", "validate", "--bundle", invalid.to_str().unwrap()]);
    assert_eq!(rejected.status.code(), Some(3));
    assert!(text(rejected.stderr).contains("configuration error"));
    fs::remove_file(invalid).unwrap();
}

#[test]
fn coverage_is_goal_aware_filterable_and_not_readiness_inflated() {
    let all = output(&["catalog", "coverage", "--goal", "core-combat-v1", "--json"]);
    assert!(all.status.success(), "{:?}", all);
    let all = text(all.stdout);
    assert!(all.contains("\"goal_id\":\"core-combat-v1\""));
    assert!(
        all.contains("\"required\":283,\"enabled\":60,\"data_ready\":60,\"golden_verified\":60")
    );
    for expected in [
        "released-character-combat-forms\",\"required\":88",
        "released-light-cones\",\"required\":165",
        "standard-v1-enemy-variants\",\"required\":17",
        "standard-v1-encounters\",\"required\":6",
        "standard-v1-scenarios\",\"required\":6",
        "standard-v1-profile\",\"required\":1",
    ] {
        assert!(all.contains(expected), "missing {expected}");
    }

    let filtered = output(&[
        "catalog",
        "coverage",
        "--category",
        "released-light-cones",
        "--json",
    ]);
    assert!(filtered.status.success(), "{:?}", filtered);
    let filtered = text(filtered.stdout);
    assert!(filtered.contains("\"required\":165"));
    assert!(!filtered.contains("released-character-combat-forms"));

    let unknown = output(&["catalog", "coverage", "--category", "characters"]);
    assert_eq!(unknown.status.code(), Some(2));

    let unknown_goal = output(&["catalog", "coverage", "--goal", "future-goal"]);
    assert_eq!(unknown_goal.status.code(), Some(2));
}

#[test]
fn battle_controller_and_exit_classes_are_explicit() {
    let replay_controller = output(&[
        "battle",
        "run",
        "--scenario",
        "synthetic-standard-v1",
        "--seed",
        "7",
        "--controller",
        "replay",
    ]);
    assert_eq!(replay_controller.status.code(), Some(2));
    assert!(text(replay_controller.stderr).contains("use replay verify FILE"));

    let unknown = output(&[
        "battle",
        "run",
        "--scenario",
        "not-a-scenario",
        "--seed",
        "7",
    ]);
    assert_eq!(unknown.status.code(), Some(5));

    let bad_seed = output(&[
        "battle",
        "run",
        "--scenario",
        "synthetic-standard-v1",
        "--seed",
        "-1",
    ]);
    assert_eq!(bad_seed.status.code(), Some(2));
}
