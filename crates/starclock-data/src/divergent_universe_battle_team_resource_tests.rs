use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe::DivergentUniverseBundleCandidate;
use crate::divergent_universe_decisions::battle_team_resources::BattleTeamResourcePolicy;
use crate::divergent_universe_decisions::{BUNDLE, DecisionCatalog, validation};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[test]
fn production_battle_team_resource_has_independent_shared_meter_policy_and_sources() {
    let reference = load_divergent_universe_bundle().unwrap();
    let first = DecisionCatalog::production(&reference).unwrap();
    let fresh = DecisionCatalog::production(&load_divergent_universe_bundle().unwrap()).unwrap();
    assert_eq!(first.battle_team_resources(), fresh.battle_team_resources());
    assert_eq!(first.digest(), fresh.digest());
    let [row] = first.battle_team_resources() else {
        panic!("one meter policy")
    };
    assert_eq!(row.key.as_ref(), "du.battle-team-resource.punchline");
    assert_eq!(row.resource_key.as_ref(), "shared.punchline");
    assert_eq!((row.initial_value, row.maximum_value), (0, 9999));
    assert_eq!(
        row.policy,
        BattleTeamResourcePolicy::VersionedProjectPolicyOriginalElationZeroClampPersist
    );
    assert_eq!(row.sources.len(), 3);
    for clause in [
        "VersionedProjectPolicy",
        "not observed",
        "No Aha actor",
        "Persist across waves",
    ] {
        assert!(row.policy_note.contains(clause), "{clause}");
    }
    assert!(row.replacement_condition.contains("Low confidence"));
    assert!(
        row.replacement_condition
            .contains("ElationBattle_MaxPower=99")
    );
}

#[test]
fn battle_team_resource_rejects_unknown_keys_policies_bounds_joins_and_cardinality() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_battle_team_resources()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for (field, value) in [
        ("id", json!(0)),
        ("stable_key", json!("UPPER")),
        ("resource_key", json!("_ElationEnergy")),
        ("character_paths", json!([])),
        ("character_paths", json!(["Elation", "Elation"])),
        ("character_paths", json!(["Priest"])),
        ("initial_value", json!(-1)),
        ("initial_value", json!(1)),
        ("maximum_value", json!(99)),
        ("maximum_value", json!(65536)),
        ("initial_value", json!(10000)),
        ("policy", json!("Exact")),
        ("summary_en", json!("")),
        ("summary_zh_cn", json!("")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
        ("source_ids", json!([111, 112])),
        ("source_ids", json!([111, 112, 112])),
        ("source_ids", json!([107, 109, 110])),
        ("source_ids", json!([111, 112, 113, 110])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        assert_rejected("DuBattleTeamResources", rows, &reference, field);
    }
    for rows in [json!([]), json!([baseline[0].clone(), baseline[0].clone()])] {
        assert_rejected("DuBattleTeamResources", rows, &reference, "cardinality");
    }
    let mut rows = baseline;
    rows[0]["stable_key"] = json!(
        config
            .du_weighted_curio_elations()
            .ordered_rows()
            .next()
            .unwrap()
            .stable_key
    );
    assert_rejected(
        "DuBattleTeamResources",
        rows,
        &reference,
        "cross-family duplicate",
    );
}

#[test]
fn battle_team_resource_rejects_every_forged_provenance_field() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_decision_sources()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for id in 111..=113 {
        for field in [
            "url",
            "revision",
            "game_version",
            "access_date",
            "sha256",
            "locator",
            "quality",
        ] {
            let mut rows = baseline.clone();
            rows.as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["id"] == id)
                .unwrap()[field] = json!("forged");
            assert_rejected(
                "DuDecisionSources",
                rows,
                &reference,
                &format!("{id}:{field}"),
            );
        }
    }
}

fn assert_rejected(
    table: &'static str,
    rows: Value,
    reference: &DivergentUniverseBundleCandidate,
    label: &str,
) {
    let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(table, rows)])));
    assert!(
        parsed.is_err() || validation::compile(&parsed.unwrap(), reference).is_err(),
        "{label}"
    );
}
