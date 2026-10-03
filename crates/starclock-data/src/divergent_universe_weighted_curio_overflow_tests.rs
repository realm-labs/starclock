use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe_decisions::{
    BUNDLE, DecisionCatalog, validation, weighted_curio_overflows::WeightedCurioOverflowStatus,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn rejects(table: &'static str, rows: Value) {
    let reference = load_divergent_universe_bundle().unwrap();
    let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(table, rows)])));
    assert!(parsed.is_err() || validation::compile(&parsed.unwrap(), &reference).is_err());
}

#[test]
fn production_weighted_curio_overflow_preserves_exact_ratios_and_pending_admission() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    let fresh = DecisionCatalog::production(&reference).unwrap();
    assert_eq!(
        catalog.weighted_curio_overflows(),
        fresh.weighted_curio_overflows()
    );
    assert_eq!(catalog.digest(), fresh.digest());
    assert_eq!(catalog.weighted_curio_overflows().len(), 1);
    let row = &catalog.weighted_curio_overflows()[0];
    assert_eq!(
        row.weighted_curio.as_str(),
        "divergent-universe.weighted-curio.1016"
    );
    assert_eq!(row.maze_buff_id.as_ref(), "633416");
    assert_eq!(
        (
            row.base_multiplier_millionths,
            row.overflow_multiplier_millionths,
            row.attack_increase_millionths
        ),
        (10_000_000, 1_000_000, 800_000),
    );
    assert_eq!(
        row.status,
        WeightedCurioOverflowStatus::PendingNativeDeathCallbackAndBaseDamage
    );
    assert_eq!(row.sources.len(), 5);
    assert!(row.source_semantics.contains("random ties"));
    assert!(row.source_semantics.contains("death/deathrattle"));
    assert!(row.unresolved_runtime.starts_with("Unimplemented:"));
    assert!(row.unresolved_runtime.contains("not established"));
}

#[test]
fn weighted_curio_overflow_rejects_changed_exact_fields_or_false_admission() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let baseline = serde_json::to_value(
        config
            .du_weighted_curio_overflows()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for (field, value) in [
        ("id", json!(0)),
        (
            "stable_key",
            json!("du.weighted-curio-transfer.dignity-and-passion"),
        ),
        (
            "weighted_curio_key",
            json!("divergent-universe.weighted-curio.1009"),
        ),
        ("maze_buff_id", json!("633409")),
        ("character_paths", json!(["Mage"])),
        ("character_paths", json!(["Rogue", "Mage"])),
        ("character_paths", json!(["Mage", "Mage"])),
        ("base_parameter", json!(2)),
        ("overflow_parameter", json!(1)),
        ("attack_parameter", json!(2)),
        ("base_multiplier", json!("10.0")),
        ("overflow_multiplier", json!("2")),
        ("attack_increase", json!("0.80")),
        ("attack_increase", json!("1")),
        ("attack_increase", json!(0.8)),
        ("status", json!("ExactIntegrated")),
        ("summary_en", json!("")),
        ("summary_zh_cn", json!("")),
        ("source_semantics", json!("")),
        ("unresolved_runtime", json!("Exact observed parity")),
        ("source_ids", json!([126, 127, 128, 129])),
        ("source_ids", json!([126, 127, 128, 129, 129])),
        ("source_ids", json!([122, 123, 124, 125, 126])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        rejects("DuWeightedCurioOverflows", rows);
    }
    rejects("DuWeightedCurioOverflows", json!([]));
    let mut duplicate = baseline.clone();
    duplicate.as_array_mut().unwrap().push(baseline[0].clone());
    duplicate[1]["id"] = json!(2);
    duplicate[1]["stable_key"] = json!("du.weighted-curio-overflow.duplicate");
    rejects("DuWeightedCurioOverflows", duplicate);
}

#[test]
fn weighted_curio_overflow_rejects_forged_evidence_at_each_required_source() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let baseline = serde_json::to_value(
        config
            .du_decision_sources()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for id in 126..=130 {
        for field in [
            "stable_key",
            "url",
            "revision",
            "game_version",
            "access_date",
            "sha256",
            "locator",
            "quality",
        ] {
            let mut rows = baseline.clone();
            let row = rows
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["id"] == id)
                .unwrap();
            row[field] = json!("forged");
            rejects("DuDecisionSources", rows);
        }
    }
}
