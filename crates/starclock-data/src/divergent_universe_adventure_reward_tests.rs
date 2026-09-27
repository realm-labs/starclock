use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe_decisions::{BUNDLE, DecisionCatalog, validation};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn adventure_rewards_production_sora_binds_current_card_and_explicit_external_policy() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    let rows = catalog.adventure_rewards();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(
        (
            row.preset_source.as_ref(),
            row.level,
            row.maximum_chests,
            row.fragments_per_chest
        ),
        ("1011", 1, 3, 100)
    );
    assert!(row.policy_note.contains("are unavailable"));
    assert!(row.summary_en.contains("unimplemented"));
    assert!(
        row.replacement_condition
            .contains("released chest programs")
    );
    assert_eq!(row.sources.len(), 3);
}

#[test]
fn adventure_rewards_reject_foreign_context_bad_amount_bound_metadata_and_provenance() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_adventure_rewards()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for (field, value) in [
        ("id", json!(0)),
        ("stable_key", json!("UPPER")),
        ("preset_source", json!("1008")),
        ("level", json!(2)),
        ("maximum_chests", json!(2)),
        ("fragments_per_chest", json!("0")),
        ("fragments_per_chest", json!("01")),
        ("fragments_per_chest", json!("1.0")),
        ("fragments_per_chest", json!("3074457345618258603")),
        ("policy", json!("Exact")),
        ("summary_en", json!(" ")),
        ("summary_zh_cn", json!("")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
        ("source_ids", json!([])),
        ("source_ids", json!([68, 68, 72])),
        ("source_ids", json!([68, 69])),
        ("source_ids", json!([68, 69, 70])),
        ("source_ids", json!([999])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        let parsed =
            SoraConfig::from_source(&EditedRows(BTreeMap::from([("DuAdventureRewards", rows)])));
        assert!(
            parsed.is_err() || validation::compile(&parsed.unwrap(), &reference).is_err(),
            "{field}"
        );
    }
    let mut rows = baseline.clone();
    rows.as_array_mut().unwrap().push(baseline[0].clone());
    rows[1]["id"] = json!(2);
    rows[1]["stable_key"] = json!("du.adventure-reward.duplicate");
    let parsed =
        SoraConfig::from_source(&EditedRows(BTreeMap::from([("DuAdventureRewards", rows)])))
            .unwrap();
    assert!(validation::compile(&parsed, &reference).is_err());
}
