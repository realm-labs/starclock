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
    assert_eq!(row.sources.len(), 7);
    assert_eq!(row.base.fixed_damage_millionths, 100_000_000);
    assert_eq!(row.base.hard_level_group, 1);
    assert_eq!(row.base.hp_ratios_millionths.len(), 95);
    for (level, expected) in [
        (1, 800_000),
        (40, 9_524_581),
        (80, 148_011_020),
        (95, 294_421_720),
    ] {
        assert_eq!(row.base.hp_ratios_millionths[level - 1], expected);
    }
    assert!(row.base.policy_note.starts_with("VersionedProjectPolicy:"));
    assert!(!row.base.replacement_condition.is_empty());
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
        ("base_fixed_damage", json!("100.0")),
        ("base_fixed_damage", json!("0")),
        ("base_hard_level_group", json!(2)),
        ("base_policy", json!("ExactObservedPostfix")),
        ("base_policy_note", json!("Exact observed parity")),
        ("base_policy_note", json!("VersionedProjectPolicy: forged")),
        ("base_replacement_condition", json!("")),
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
    for id in 126..=132 {
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

#[test]
fn weighted_curio_overflow_level_curve_rejects_incomplete_noncanonical_and_foreign_rows() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let baseline = serde_json::to_value(
        config
            .du_weighted_curio_overflow_levels()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for (field, value) in [
        ("id", json!(0)),
        ("unit_level", json!(2)),
        ("stable_key", json!("du.weighted-curio-overflow.level.002")),
        (
            "weighted_curio_key",
            json!("divergent-universe.weighted-curio.1009"),
        ),
        ("hard_level_group", json!(2)),
        ("source_ids", json!([130])),
        ("source_ids", json!([131, 131])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        rejects("DuWeightedCurioOverflowLevels", rows);
    }
    for value in [
        json!("0"),
        json!("-1"),
        json!("0.80"),
        json!("0.8000001"),
        json!("01"),
        json!(".8"),
        json!("9999999999999999999"),
        json!(0.8),
    ] {
        let mut rows = baseline.clone();
        rows[0]["hp_ratio"] = value;
        rejects("DuWeightedCurioOverflowLevels", rows);
    }
    let mut missing = baseline.clone();
    missing.as_array_mut().unwrap().remove(40);
    rejects("DuWeightedCurioOverflowLevels", missing);
    let mut extra = baseline.clone();
    extra.as_array_mut().unwrap().push(baseline[0].clone());
    extra[95]["id"] = json!(96);
    rejects("DuWeightedCurioOverflowLevels", extra);
    rejects("DuWeightedCurioOverflowLevels", json!([]));
}
