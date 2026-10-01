use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe_decisions::{BUNDLE, DecisionCatalog, validation};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn production_weighted_curio_necrosis_lowers_all_four_current_operands_and_policy() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    assert_eq!(catalog.weighted_curio_necroses().len(), 1);
    let row = &catalog.weighted_curio_necroses()[0];
    assert_eq!(
        row.weighted_curio.as_str(),
        "divergent-universe.weighted-curio.1003"
    );
    assert_eq!(row.maze_buff_id.as_ref(), "633403");
    assert_eq!(
        (
            row.base_chance_millionths,
            row.attack_multiplier_millionths,
            row.duration_turns,
            row.detonation_fraction_millionths
        ),
        (1_500_000, 6_000_000, 3, 2_000_000)
    );
    assert_eq!(row.sources.len(), 4);
    assert!(
        row.policy_note
            .contains("No Burn detonation on application")
    );
    assert!(row.replacement_condition.contains("Low confidence"));
}

#[test]
fn weighted_curio_necrosis_rejects_wrong_joins_operands_and_missing_policy() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_weighted_curio_necroses()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for (field, value) in [
        ("id", json!(0)),
        ("stable_key", json!("UPPER")),
        (
            "weighted_curio_key",
            json!("divergent-universe.weighted-curio.1005"),
        ),
        ("maze_buff_id", json!("633405")),
        ("character_paths", json!(["Warlock"])),
        ("character_paths", json!(["Priest", "Priest"])),
        ("chance_parameter", json!(2)),
        ("damage_parameter", json!(1)),
        ("duration_parameter", json!(4)),
        ("detonation_parameter", json!(3)),
        ("base_chance", json!("1.50")),
        ("attack_multiplier", json!("4")),
        ("attack_multiplier", json!(6)),
        ("duration_turns", json!(2)),
        ("detonation_fraction", json!("2.0")),
        ("policy", json!("Exact")),
        ("summary_en", json!("")),
        ("summary_zh_cn", json!("")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
        ("source_ids", json!([103, 104, 105])),
        ("source_ids", json!([103, 104, 105, 105])),
        ("source_ids", json!([99, 100, 101, 102])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
            "DuWeightedCurioNecroses",
            rows,
        )])));
        assert!(
            parsed.is_err() || validation::compile(&parsed.unwrap(), &reference).is_err(),
            "{field}"
        );
    }
    for rows in [json!([]), json!([baseline[0].clone(), baseline[0].clone()])] {
        let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
            "DuWeightedCurioNecroses",
            rows,
        )])));
        assert!(parsed.is_err() || validation::compile(&parsed.unwrap(), &reference).is_err());
    }
}

#[test]
fn weighted_curio_necrosis_rejects_forged_provenance_at_all_four_sources() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_decision_sources()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for id in 103..=106 {
        for field in [
            "url",
            "revision",
            "game_version",
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
            let parsed =
                SoraConfig::from_source(&EditedRows(BTreeMap::from([("DuDecisionSources", rows)])));
            assert!(
                parsed.is_err() || validation::compile(&parsed.unwrap(), &reference).is_err(),
                "{id}: {field}"
            );
        }
    }
}
