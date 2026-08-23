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
    std::env::temp_dir().join(format!("starclock-g07-p6-b2-{}-{name}", std::process::id()))
}

#[test]
fn config_validation_uses_only_a_validated_sora_bundle() {
    let default = output(&["config", "validate", "--json"]);
    assert!(default.status.success(), "{:?}", default);
    assert_eq!(
        text(default.stdout).trim(),
        "{\"kind\":\"config-validation\",\"valid\":true,\"game_version\":\"4.4\",\"bundle_sha256\":\"ca9f235534183705e0b6b7f30a28f4972a1115087831a215ffab9e49d1f68724\",\"identities\":6807,\"enabled\":6807}"
    );

    let bundle =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/generated/config.sora");
    let explicit = output(&["config", "validate", "--bundle", bundle.to_str().unwrap()]);
    assert!(explicit.status.success(), "{:?}", explicit);
    let human = text(explicit.stdout);
    assert!(human.contains("config valid game_version=4.4"));
    assert!(human.contains("identities=6807 enabled=6807"));

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
    assert!(
        all.contains("\"required\":285,\"enabled\":285,\"data_ready\":285,\"golden_verified\":283")
    );
    for expected in [
        "released-character-combat-forms\",\"required\":90,\"enabled\":90,\"data_ready\":90,\"golden_verified\":88",
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
    assert!(filtered.contains("\"data_ready\":165"));
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
        "synthetic-standard",
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
        "synthetic-standard",
        "--seed",
        "-1",
    ]);
    assert_eq!(bad_seed.status.code(), Some(2));
}

#[test]
fn streamable_http_requires_an_explicit_exact_loopback_profile() {
    let implicit = output(&[
        "mcp",
        "serve",
        "--transport",
        "streamable-http",
        "--bind",
        "127.0.0.1:43123",
        "--allow-origin",
        "http://127.0.0.1:43123",
    ]);
    assert_eq!(implicit.status.code(), Some(2));

    for rejected in [
        [
            "mcp",
            "serve",
            "--transport",
            "streamable-http",
            "--development-loopback",
            "--bind",
            "0.0.0.0:43123",
            "--allow-origin",
            "http://127.0.0.1:43123",
        ]
        .as_slice(),
        [
            "mcp",
            "serve",
            "--transport",
            "streamable-http",
            "--development-loopback",
            "--bind",
            "127.0.0.1:43123",
            "--allow-origin",
            "*",
        ]
        .as_slice(),
    ] {
        let result = output(rejected);
        assert_eq!(result.status.code(), Some(8), "{result:?}");
        assert!(result.stdout.is_empty(), "{result:?}");
        let stderr = text(result.stderr);
        assert!(stderr.contains("MCP service error"), "{stderr}");
        assert!(!stderr.contains("0.0.0.0"), "{stderr}");
    }

    let no_origin = output(&[
        "mcp",
        "serve",
        "--transport",
        "streamable-http",
        "--development-loopback",
        "--bind",
        "127.0.0.1:43123",
    ]);
    assert_eq!(no_origin.status.code(), Some(8));
}
