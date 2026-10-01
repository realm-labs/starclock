use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe_decisions::{BUNDLE, DecisionCatalog, validation};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn production_weighted_curio_prayer_preserves_three_exact_resource_operands() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    let row = &catalog.weighted_curio_prayers()[0];
    assert_eq!(catalog.weighted_curio_prayers().len(), 1);
    assert_eq!(
        row.weighted_curio.as_str(),
        "divergent-universe.weighted-curio.1004"
    );
    assert_eq!(row.maze_buff_id.as_ref(), "633404");
    assert_eq!(
        (
            row.hp_millionths,
            row.consume_millionths,
            row.shield_millionths
        ),
        (600_000, 150_000, 250_000)
    );
    assert_eq!(row.sources.len(), 4);
    assert!(row.policy_note.contains("live HP resource capacity"));
    assert!(row.replacement_condition.contains("not observed parity"));
}

#[test]
fn weighted_curio_prayer_rejects_seasonal_joins_operands_and_provenance() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_weighted_curio_prayers()
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
        ("character_paths", json!(["Rogue", "Mage"])),
        ("character_paths", json!(["Warlock", "Mage"])),
        ("hp_parameter", json!(2)),
        ("consume_parameter", json!(1)),
        ("shield_parameter", json!(2)),
        ("hp_fraction", json!("0.60")),
        ("hp_fraction", json!("0.5")),
        ("consume_fraction", json!("0.150")),
        ("shield_fraction", json!("0.35")),
        ("policy", json!("Exact")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
        ("summary_en", json!("")),
        ("summary_zh_cn", json!("")),
        ("source_ids", json!([89, 90, 91])),
        ("source_ids", json!([85, 86, 87, 88])),
        ("source_ids", json!([89, 90, 91, 91])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
            "DuWeightedCurioPrayers",
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
    rows[1]["stable_key"] = json!("du.weighted-curio-prayer.duplicate");
    let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
        "DuWeightedCurioPrayers",
        rows,
    )])))
    .unwrap();
    assert!(validation::compile(&parsed, &reference).is_err());
}

#[test]
fn weighted_curio_prayer_rejects_forged_released_source_fields() {
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
            .filter(|row| row["id"].as_i64().unwrap() >= 89)
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
