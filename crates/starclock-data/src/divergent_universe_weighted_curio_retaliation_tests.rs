use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe_decisions::{BUNDLE, DecisionCatalog, validation};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn production_weighted_curio_retaliation_preserves_released_values_and_explicit_policy() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    let row = &catalog.weighted_curio_retaliations()[0];
    assert_eq!(catalog.weighted_curio_retaliations().len(), 1);
    assert_eq!(
        row.weighted_curio.as_str(),
        "divergent-universe.weighted-curio.1013"
    );
    assert_eq!(row.maze_buff_id.as_ref(), "633413");
    assert_eq!(
        (row.additional_millionths, row.aggro_millionths),
        (4_000_000, 300_000)
    );
    assert_eq!(
        (
            row.base_aggro.destruction,
            row.base_aggro.hunt,
            row.base_aggro.erudition,
            row.base_aggro.harmony,
            row.base_aggro.nihility,
            row.base_aggro.preservation,
            row.base_aggro.abundance,
            row.base_aggro.remembrance,
            row.base_aggro.elation
        ),
        (125, 75, 75, 100, 100, 150, 100, 100, 100)
    );
    assert_eq!(row.sources.len(), 6);
    assert!(row.policy_note.contains("DamageFromOwner"));
    assert!(row.policy_note.contains("+30% percent-of-base"));
    assert!(row.replacement_condition.contains("not observed parity"));
}

#[test]
fn weighted_curio_retaliation_rejects_wrong_version_joins_operands_weights_and_sources() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_weighted_curio_retaliations()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for (field, value) in [
        ("id", json!(0)),
        ("stable_key", json!("UPPER")),
        (
            "weighted_curio_key",
            json!("divergent-universe.weighted-curio.1017"),
        ),
        ("maze_buff_id", json!("633401")),
        ("character_elements", json!(["Fire"])),
        ("additional_parameter", json!(2)),
        ("aggro_parameter", json!(1)),
        ("additional_multiplier", json!("2.5")),
        ("additional_multiplier", json!("4.0")),
        ("aggro_fraction", json!("0.30")),
        ("aggro_fraction", json!("5")),
        ("base_aggro_destruction", json!("150")),
        ("base_aggro_hunt", json!("75.0")),
        ("base_aggro_preservation", json!("0")),
        ("base_aggro_remembrance", json!("100.0")),
        ("policy", json!("Exact")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
        ("summary_en", json!("")),
        ("summary_zh_cn", json!("")),
        ("source_ids", json!([93, 94, 95, 96])),
        ("source_ids", json!([89, 90, 91, 92, 97, 98])),
        ("source_ids", json!([93, 94, 95, 96, 97, 97])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
            "DuWeightedCurioRetaliations",
            rows,
        )])));
        assert!(
            parsed.is_err() || validation::compile(&parsed.unwrap(), &reference).is_err(),
            "{field}"
        );
    }
    for rows in [json!([]), json!([baseline[0].clone(), baseline[0].clone()])] {
        let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
            "DuWeightedCurioRetaliations",
            rows,
        )])));
        assert!(parsed.is_err() || validation::compile(&parsed.unwrap(), &reference).is_err());
    }
}

#[test]
fn weighted_curio_retaliation_rejects_forged_fact_source_fields() {
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
            .filter(|row| row["id"].as_i64().unwrap() >= 93)
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
