use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe_decisions::{BUNDLE, DecisionCatalog, validation};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn production_weighted_curio_support_attack_preserves_all_three_operands() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    let rows = catalog.weighted_curio_support_attacks();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(
        row.weighted_curio.as_str(),
        "divergent-universe.weighted-curio.1017"
    );
    assert_eq!(row.maze_buff_id.as_ref(), "633417");
    assert_eq!(
        (
            row.crit_rate_millionths,
            row.crit_damage_millionths,
            row.additional_millionths
        ),
        (150_000, 300_000, 1_000_000)
    );
    assert_eq!(row.sources.len(), 4);
    assert!(row.policy_note.contains("once per action"));
    assert!(row.replacement_condition.contains("not observed parity"));
}

#[test]
fn weighted_curio_support_attack_rejects_variant_joins_operands_and_provenance() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    validation::compile(&config, &reference).unwrap();
    let baseline = serde_json::to_value(
        config
            .du_weighted_curio_support_attacks()
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
        ("character_paths", json!(["Shaman", "Priest"])),
        ("character_paths", json!(["Knight", "Priest", "Shaman"])),
        (
            "character_paths",
            json!(["Shaman", "Priest", "Knight", "Memory"]),
        ),
        ("crit_rate_parameter", json!(2)),
        ("crit_damage_parameter", json!(1)),
        ("additional_parameter", json!(2)),
        ("policy", json!("Exact")),
        ("crit_rate_fraction", json!("0.150")),
        ("crit_rate_fraction", json!("0.2")),
        ("crit_damage_fraction", json!("0.25")),
        ("crit_damage_fraction", json!("0.3e0")),
        ("additional_multiplier", json!("0.75")),
        ("additional_multiplier", json!("1.0")),
        ("summary_en", json!(" ")),
        ("summary_zh_cn", json!("")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
        ("source_ids", json!([])),
        ("source_ids", json!([85, 86, 87])),
        ("source_ids", json!([85, 86, 87, 87])),
        ("source_ids", json!([999])),
        ("source_ids", json!([81, 82, 83, 84])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
            "DuWeightedCurioSupportAttacks",
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
    rows[1]["stable_key"] = json!("du.weighted-curio-support-attack.duplicate");
    let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
        "DuWeightedCurioSupportAttacks",
        rows,
    )])))
    .unwrap();
    assert!(validation::compile(&parsed, &reference).is_err());
}

#[test]
fn weighted_curio_support_attack_rejects_forged_source_fields() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_decision_sources()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for (field, value) in [
        ("url", json!("https://example.com/unrelated")),
        (
            "revision",
            json!("0000000000000000000000000000000000000000"),
        ),
        ("game_version", json!("2.7")),
        (
            "sha256",
            json!("0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            "locator",
            json!("ExcelOutput/RogueTournHex.json; HexID=1002;"),
        ),
        ("quality", json!("ProjectPolicy")),
    ] {
        let mut rows = baseline.clone();
        let row = rows
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row["id"] == json!(85))
            .unwrap();
        row[field] = value;
        let parsed =
            SoraConfig::from_source(&EditedRows(BTreeMap::from([("DuDecisionSources", rows)])));
        assert!(
            parsed.is_err() || validation::compile(&parsed.unwrap(), &reference).is_err(),
            "{field}"
        );
    }
}
