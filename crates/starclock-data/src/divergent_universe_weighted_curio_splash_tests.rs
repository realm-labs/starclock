use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe_decisions::{
    BUNDLE, DecisionCatalog, validation, weighted_curio_splashes::WeightedCurioSplashPolicy,
};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn weighted_curio_splash_production_resolves_generic_maze_buff_not_rogue_maze_buff() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    assert_eq!(catalog.weighted_curio_splashes().len(), 1);
    let row = &catalog.weighted_curio_splashes()[0];
    assert_eq!(
        row.weighted_curio.as_str(),
        "divergent-universe.weighted-curio.1001"
    );
    assert_eq!(row.maze_buff_id.as_ref(), "633401");
    assert_eq!(row.fraction_millionths, 300_000);
    assert_eq!(
        row.policy,
        WeightedCurioSplashPolicy::VersionedProjectPolicyHitCalculatedCopyAdjacentTrueDamage
    );
    assert_eq!(row.sources.len(), 4);
    assert!(row.policy_note.contains("overkill"));
    assert!(row.replacement_condition.contains("not observed parity"));
}

#[test]
fn weighted_curio_splash_rejects_foreign_join_noncanonical_fraction_policy_and_provenance() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_weighted_curio_splashes()
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
        ("weighted_curio_key", json!("divergent-universe.curio.1001")),
        ("maze_buff_id", json!("633402")),
        ("character_path", json!("Warrior")),
        ("fraction_parameter", json!(2)),
        ("damage_fraction", json!("0")),
        ("damage_fraction", json!("1.3")),
        ("damage_fraction", json!("0.30")),
        ("damage_fraction", json!("0.0000001")),
        ("damage_fraction", json!("-0.3")),
        ("damage_fraction", json!("0.000000")),
        ("damage_fraction", json!("0.3e0")),
        ("policy", json!("Exact")),
        ("summary_en", json!(" ")),
        ("summary_zh_cn", json!("")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
        ("source_ids", json!([])),
        ("source_ids", json!([73, 74, 75, 75])),
        ("source_ids", json!([73, 74, 75])),
        ("source_ids", json!([999])),
        ("source_ids", json!([1, 2, 3])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
            "DuWeightedCurioSplashes",
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
    rows[1]["stable_key"] = json!("du.weighted-curio-splash.duplicate");
    let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
        "DuWeightedCurioSplashes",
        rows,
    )])))
    .unwrap();
    assert!(validation::compile(&parsed, &reference).is_err());
}
