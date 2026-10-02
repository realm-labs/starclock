use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe_decisions::{BUNDLE, DecisionCatalog, validation};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn production_weighted_curio_transfer_preserves_current_four_exact_operands() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    assert_eq!(catalog.weighted_curio_transfers().len(), 1);
    let row = &catalog.weighted_curio_transfers()[0];
    assert_eq!(
        row.weighted_curio.as_str(),
        "divergent-universe.weighted-curio.1009"
    );
    assert_eq!(row.maze_buff_id.as_ref(), "633409");
    assert_eq!(
        (
            row.transfer_millionths,
            row.threshold_millionths,
            row.decay_millionths,
            row.heal_millionths
        ),
        (750_000, 300_000, 900_000, 100_000)
    );
    assert_eq!(row.sources.len(), 4);
    assert!(
        row.policy_note
            .contains("actual post-formula applied capacity")
    );
    assert!(row.replacement_condition.contains("not observed parity"));
}

#[test]
fn weighted_curio_transfer_rejects_wrong_joins_parameters_policy_and_provenance() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_weighted_curio_transfers()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for (field, value) in [
        ("id", json!(0)),
        ("stable_key", json!("UPPER")),
        (
            "weighted_curio_key",
            json!("divergent-universe.weighted-curio.1002"),
        ),
        ("maze_buff_id", json!("633402")),
        ("character_path", json!("Shaman")),
        ("transfer_parameter", json!(2)),
        ("threshold_parameter", json!(1)),
        ("decay_parameter", json!(4)),
        ("heal_parameter", json!(3)),
        ("transfer_fraction", json!("1")),
        ("transfer_fraction", json!("0.750")),
        ("threshold_fraction", json!("0.30")),
        ("decay_fraction", json!("0.8")),
        ("heal_fraction", json!("0.10")),
        ("policy", json!("Exact")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
        ("summary_en", json!("")),
        ("summary_zh_cn", json!("")),
        ("source_ids", json!([122, 123, 124])),
        ("source_ids", json!([118, 119, 120, 121])),
        ("source_ids", json!([122, 123, 124, 124])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
            "DuWeightedCurioTransfers",
            rows,
        )])));
        assert!(
            parsed.is_err() || validation::compile(&parsed.unwrap(), &reference).is_err(),
            "{field}"
        );
    }
    let mut rows = baseline.clone();
    rows.as_array_mut().unwrap().push(baseline[0].clone());
    rows[1]["id"] = json!(2);
    rows[1]["stable_key"] = json!("du.weighted-curio-transfer.duplicate");
    let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
        "DuWeightedCurioTransfers",
        rows,
    )])))
    .unwrap();
    assert!(validation::compile(&parsed, &reference).is_err());
}

#[test]
fn weighted_curio_transfer_rejects_forged_source_fields() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_decision_sources()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for field in ["url", "revision", "game_version", "sha256", "locator"] {
        let mut rows = baseline.clone();
        for row in rows
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .filter(|row| row["id"].as_i64().unwrap() >= 122)
        {
            row[field] = json!("forged");
        }
        let parsed =
            SoraConfig::from_source(&EditedRows(BTreeMap::from([("DuDecisionSources", rows)])));
        assert!(
            parsed.is_err() || validation::compile(&parsed.unwrap(), &reference).is_err(),
            "{field}"
        );
    }
}
