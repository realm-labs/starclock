use crate::divergent_universe_decisions::tests::transport::EditedRows;
use crate::{
    divergent_universe::{DivergentUniverseBundleCandidate, load_divergent_universe_bundle},
    divergent_universe_decisions::{
        BUNDLE, DecisionCatalog, DecisionDataError, weighted_curio_deflagrations,
        weighted_curio_deflagrations::{
            WeightedCurioDeflagrationBasePolicy, WeightedCurioDeflagrationPolicy,
            WeightedCurioDeflagrationStatus,
        },
    },
    divergent_universe_decisions_generated::{SoraConfig, runtime::SoraBundle},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::OnceLock;

fn reference() -> &'static DivergentUniverseBundleCandidate {
    static REFERENCE: OnceLock<DivergentUniverseBundleCandidate> = OnceLock::new();
    REFERENCE.get_or_init(|| load_divergent_universe_bundle().unwrap())
}

fn config() -> SoraConfig {
    SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap()
}
fn rows(table: &str) -> Vec<Value> {
    match table {
        "DuWeightedCurioDeflagrations" => config()
            .du_weighted_curio_deflagrations()
            .ordered_rows()
            .map(|row| serde_json::to_value(row).unwrap())
            .collect(),
        "DuWeightedCurioDeflagrationLevels" => config()
            .du_weighted_curio_deflagration_levels()
            .ordered_rows()
            .map(|row| serde_json::to_value(row).unwrap())
            .collect(),
        "DuDecisionSources" => config()
            .du_decision_sources()
            .ordered_rows()
            .map(|row| serde_json::to_value(row).unwrap())
            .collect(),
        _ => panic!("closed fixture table"),
    }
}
fn check(table: &'static str, rows: Vec<Value>) -> Result<(), DecisionDataError> {
    let source = EditedRows(BTreeMap::from([(table, json!(rows))]));
    let config = SoraConfig::from_source(&source).map_err(|_| DecisionDataError::Transport)?;
    weighted_curio_deflagrations::compile(&config, reference()).map(|_| ())
}

#[test]
fn weighted_curio_deflagration_production_loads_exact_operands_without_native_status() {
    let reference = load_divergent_universe_bundle().unwrap();
    let a = DecisionCatalog::production(&reference).unwrap();
    let b = DecisionCatalog::production(&reference).unwrap();
    assert_eq!(a.digest(), b.digest());
    assert_eq!(
        a.weighted_curio_deflagrations(),
        b.weighted_curio_deflagrations()
    );
    assert_eq!(a.weighted_curio_deflagrations().len(), 1);
    let row = &a.weighted_curio_deflagrations()[0];
    assert_eq!(
        row.weighted_curio.as_str(),
        "divergent-universe.weighted-curio.1012"
    );
    assert_eq!(
        row.burn_fractions_millionths,
        [500_000, 1_000_000, 1_500_000, 2_000_000]
    );
    assert_eq!(
        (
            row.damage_multiplier_millionths,
            row.duration_turns,
            row.fixed_base_damage_millionths,
            row.hard_level_group
        ),
        (2_000_000, 2, 100_000_000, 1)
    );
    assert_eq!(
        row.status,
        WeightedCurioDeflagrationStatus::AuthoredOperandsPendingNative
    );
    assert_eq!(row.policy, WeightedCurioDeflagrationPolicy::VersionedProjectPolicyOriginalFireAfterActionNaturalTickBurns);
    assert_eq!(row.base_policy, WeightedCurioDeflagrationBasePolicy::VersionedProjectPolicyTargetGroupOneHpRatioProtocolHpFloor);
    assert_eq!(row.sources.len(), 8);
    assert_eq!(row.hp_ratios_millionths.len(), 95);
    for (level, expected) in [
        (1, 800_000),
        (40, 9_524_581),
        (80, 148_011_020),
        (95, 294_421_720),
    ] {
        assert_eq!(row.hp_ratios_millionths[level - 1], expected);
    }
    assert!(row.runtime_policy_note.contains("DotDetonation never"));
    assert!(row.runtime_policy_note.contains("low-confidence"));
    assert!(row.base_policy_note.contains("low-confidence"));
}

#[test]
fn weighted_curio_deflagration_rejects_missing_duplicate_foreign_operands_and_false_parity() {
    const TABLE: &str = "DuWeightedCurioDeflagrations";
    assert!(check(TABLE, vec![]).is_err());
    let current = rows(TABLE);
    assert!(check(TABLE, vec![current[0].clone(), current[0].clone()]).is_err());
    for (field, value) in [
        ("id", json!(2)),
        ("stable_key", json!("foreign")),
        (
            "weighted_curio_key",
            json!("divergent-universe.weighted-curio.1003"),
        ),
        ("maze_buff_id", json!("633403")),
        ("elements", json!(["Ice"])),
        ("burn_fractions", json!(["0.50", "1", "1.5", "2"])),
        ("burn_fractions", json!(["2", "1.5", "1", "0.5"])),
        ("burn_fractions", json!(["0.5", "1", "1.5"])),
        ("damage_parameter", json!(4)),
        ("damage_multiplier", json!("2.0")),
        ("damage_multiplier", json!(2)),
        ("duration_parameter", json!(5)),
        ("duration_turns", json!(3)),
        ("base_fixed_damage", json!("100.0")),
        ("base_hard_level_group", json!(2)),
        ("status", json!("ExactRuleIr")),
        ("policy", json!("Observed")),
        ("base_policy", json!("Observed")),
        ("summary_en", json!("")),
        ("summary_zh_cn", json!("")),
        ("runtime_policy_note", json!("Observed")),
        ("base_policy_note", json!("Observed")),
        ("runtime_replacement_condition", json!("")),
        ("base_replacement_condition", json!("")),
        ("source_ids", json!([141, 142, 143, 144, 145, 146, 147])),
    ] {
        let mut changed = current.clone();
        changed[0][field] = value;
        assert!(check(TABLE, changed).is_err(), "{field}");
    }
}

#[test]
fn weighted_curio_deflagration_all_95_levels_reject_identity_precision_and_source_drift() {
    const TABLE: &str = "DuWeightedCurioDeflagrationLevels";
    let current = rows(TABLE);
    assert!(check(TABLE, vec![]).is_err());
    let mut missing = current.clone();
    missing.pop();
    assert!(check(TABLE, missing).is_err());
    let mut duplicate = current.clone();
    duplicate.push(current[0].clone());
    assert!(check(TABLE, duplicate).is_err());
    for index in 0..95 {
        for (field, value) in [
            ("id", json!(100)),
            ("stable_key", json!("foreign")),
            (
                "weighted_curio_key",
                json!("divergent-universe.weighted-curio.1016"),
            ),
            ("hard_level_group", json!(2)),
            ("unit_level", json!(0)),
            ("hp_ratio", json!("0")),
            ("hp_ratio", json!("-1")),
            ("hp_ratio", json!("1.0000001")),
            ("hp_ratio", json!("0.80")),
            ("hp_ratio", json!(1)),
            ("source_ids", json!([131])),
        ] {
            let mut changed = current.clone();
            changed[index][field] = value;
            assert!(
                check(TABLE, changed).is_err(),
                "level {} {field}",
                index + 1
            );
        }
    }
}

#[test]
fn weighted_curio_deflagration_each_fact_and_policy_rejects_forged_provenance() {
    for id in 141..=148 {
        for (field, value) in [
            ("sha256", json!("0".repeat(64))),
            ("quality", json!("ObservedCommunity")),
            ("revision", json!("foreign")),
            ("game_version", json!("4.3")),
            ("access_date", json!("2000-01-01")),
            ("url", json!("foreign")),
            ("locator", json!("foreign")),
            ("stable_key", json!("foreign")),
        ] {
            let mut sources = rows("DuDecisionSources");
            sources.iter_mut().find(|row| row["id"] == id).unwrap()[field] = value;
            assert!(
                check("DuDecisionSources", sources).is_err(),
                "source {id} {field}"
            );
        }
    }
    for field in ["runtime_policy_note", "base_policy_note"] {
        let mut changed = rows("DuWeightedCurioDeflagrations");
        changed[0][field] =
            json!("VersionedProjectPolicy: altered independently without rebinding provenance");
        assert!(check("DuWeightedCurioDeflagrations", changed).is_err());
    }
}
