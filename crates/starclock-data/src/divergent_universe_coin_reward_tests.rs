use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe_decisions::{
    BUNDLE, DecisionCatalog, coin_rewards::CoinRewardPolicy, validation,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn source() -> SoraConfig {
    SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap()
}
fn rows(config: &SoraConfig) -> Value {
    serde_json::to_value(config.du_coin_rewards().ordered_rows().collect::<Vec<_>>()).unwrap()
}

#[test]
fn coin_rewards_production_sora_preserves_reviewed_presets_and_explicit_chest_policy() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    assert_eq!(
        catalog
            .coin_rewards()
            .iter()
            .map(|row| (row.preset_source.as_ref(), row.level, row.amount))
            .collect::<Vec<_>>(),
        [("1008", 1, 100), ("1024", 2, 200), ("9008", 1, 100)]
    );
    for row in catalog.coin_rewards() {
        assert_eq!(
            row.policy,
            CoinRewardPolicy::VersionedProjectPolicyFixedSingleChestCreditNoAmusementFacilities
        );
        assert!(row.policy_note.contains("are unavailable"));
        assert!(row.summary_en.contains("unimplemented"));
        assert_eq!(row.sources.len(), 3);
        assert!(
            row.replacement_condition
                .contains("released chest/level programs")
        );
    }
}

#[test]
fn coin_rewards_reject_invalid_amount_context_metadata_identity_and_provenance() {
    let config = source();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = rows(&config);
    for (field, value) in [
        ("id", json!(0)),
        ("stable_key", json!("UPPER")),
        ("preset_source", json!("1009")),
        ("level", json!(2)),
        ("amount", json!("0")),
        ("amount", json!("01")),
        ("amount", json!("1.0")),
        ("amount", json!("9223372036854775808")),
        ("policy", json!("Exact")),
        ("summary_en", json!(" ")),
        ("summary_zh_cn", json!("")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
        ("source_ids", json!([])),
        ("source_ids", json!([68, 68, 70])),
        ("source_ids", json!([999])),
        ("source_ids", json!([68, 69])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        let parsed =
            SoraConfig::from_source(&EditedRows(BTreeMap::from([("DuCoinRewards", rows)])));
        assert!(
            parsed.is_err() || validation::compile(&parsed.unwrap(), &reference).is_err(),
            "{field}"
        );
    }
    let mut duplicate = baseline.clone();
    duplicate[1]["preset_source"] = duplicate[0]["preset_source"].clone();
    let parsed =
        SoraConfig::from_source(&EditedRows(BTreeMap::from([("DuCoinRewards", duplicate)])))
            .unwrap();
    assert!(validation::compile(&parsed, &reference).is_err());
    for amount in ["1", "9223372036854775807"] {
        let mut rows = baseline.clone();
        rows[0]["amount"] = json!(amount);
        let parsed =
            SoraConfig::from_source(&EditedRows(BTreeMap::from([("DuCoinRewards", rows)])))
                .unwrap();
        assert_eq!(
            validation::compile(&parsed, &reference)
                .unwrap()
                .coin_rewards()[0]
                .amount
                .to_string(),
            amount
        );
    }
}
