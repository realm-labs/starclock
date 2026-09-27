use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe_decisions::{
    BUNDLE, DecisionCatalog, validation, weighted_curio_shields::WeightedCurioShieldPolicy,
};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn weighted_curio_shield_production_preserves_harmony_ally_actions_hp_fraction_and_duration() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    assert_eq!(catalog.weighted_curio_shields().len(), 1);
    let row = &catalog.weighted_curio_shields()[0];
    assert_eq!(
        row.weighted_curio.as_str(),
        "divergent-universe.weighted-curio.1002"
    );
    assert_eq!(row.maze_buff_id.as_ref(), "633402");
    assert_eq!((row.fraction_millionths, row.duration_turns), (350_000, 2));
    assert_eq!(
        row.policy,
        WeightedCurioShieldPolicy::VersionedProjectPolicyAllyActionResolvedReplaceTargetTurnShield
    );
    assert_eq!(row.sources.len(), 4);
    assert!(row.policy_note.contains("committed allied targets"));
    assert!(row.replacement_condition.contains("not observed parity"));
}

#[test]
fn weighted_curio_shield_rejects_invalid_joins_fractions_durations_and_provenance() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_weighted_curio_shields()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for (field, value) in [
        ("id", json!(0)),
        ("stable_key", json!("UPPER")),
        (
            "weighted_curio_key",
            json!("divergent-universe.weighted-curio.1001"),
        ),
        ("weighted_curio_key", json!("unknown")),
        ("maze_buff_id", json!("633401")),
        ("character_path", json!("Rogue")),
        ("fraction_parameter", json!(2)),
        ("duration_parameter", json!(1)),
        ("duration_turns", json!(0)),
        ("duration_turns", json!(-1)),
        ("duration_turns", json!(65536)),
        ("shield_fraction", json!("0")),
        ("shield_fraction", json!("1.35")),
        ("shield_fraction", json!("0.350")),
        ("shield_fraction", json!("0.0000001")),
        ("shield_fraction", json!("-0.35")),
        ("shield_fraction", json!("0.35e0")),
        ("policy", json!("Exact")),
        ("summary_en", json!(" ")),
        ("summary_zh_cn", json!("")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
        ("source_ids", json!([])),
        ("source_ids", json!([77, 78, 79, 79])),
        ("source_ids", json!([77, 78, 79])),
        ("source_ids", json!([999])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
            "DuWeightedCurioShields",
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
    rows[1]["stable_key"] = json!("du.weighted-curio-shield.duplicate");
    let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
        "DuWeightedCurioShields",
        rows,
    )])))
    .unwrap();
    assert!(validation::compile(&parsed, &reference).is_err());
}
