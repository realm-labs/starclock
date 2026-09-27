use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe_decisions::{
    BUNDLE, DecisionCatalog, reward_occurrences::RewardOccurrencePolicy, validation,
};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn reward_occurrences_production_sora_preserves_single_reviewed_card_and_substitute_policy() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    let rows = catalog.reward_occurrences();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.id.as_str(), "du.reward-room.level-one-substitute");
    assert_eq!((row.preset_source.as_ref(), row.level), ("1010", 1));
    assert_eq!(
        row.variant.as_str(),
        "divergent-universe.occurrence-variant.722601"
    );
    assert_eq!(row.policy, RewardOccurrencePolicy::VersionedProjectPolicySingleExplicitRewardOnlySubstituteNoPoolSampling);
    assert!(row.policy_note.contains("without claiming"));
    assert!(row.replacement_condition.contains("candidate pool"));
    assert_eq!(row.sources.len(), 5);
}

#[test]
fn reward_occurrences_reject_nonreward_unadmitted_paid_empty_and_invalid_source_bindings() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_reward_occurrences()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for (field, value) in [
        ("id", json!(0)),
        ("stable_key", json!("du.shop-stock.foreign")),
        ("preset_source", json!("1008")),
        ("preset_source", json!("9006")),
        ("level", json!(2)),
        ("occurrence_id", json!(999)),
        ("policy", json!("Exact")),
        ("summary_en", json!(" ")),
        ("summary_zh_cn", json!("")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
        ("source_ids", json!([])),
        ("source_ids", json!([68, 69, 999])),
        ("source_ids", json!([68, 69])),
        ("source_ids", json!([68, 69, 71, 71])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        let parsed =
            SoraConfig::from_source(&EditedRows(BTreeMap::from([("DuRewardOccurrences", rows)])));
        assert!(
            parsed.is_err() || validation::compile(&parsed.unwrap(), &reference).is_err(),
            "{field}"
        );
    }
    let mut duplicate = baseline.clone();
    let mut second = duplicate[0].clone();
    second["id"] = json!(2);
    second["stable_key"] = json!("du.reward-room.duplicate");
    duplicate.as_array_mut().unwrap().push(second);
    let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
        "DuRewardOccurrences",
        duplicate,
    )])))
    .unwrap();
    assert!(validation::compile(&parsed, &reference).is_err());
    let mut paid = serde_json::to_value(
        config
            .du_decision_choices()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    paid[0]["fragment_cost"] = json!(1);
    let parsed =
        SoraConfig::from_source(&EditedRows(BTreeMap::from([("DuDecisionChoices", paid)])))
            .unwrap();
    assert!(validation::compile(&parsed, &reference).is_err());
    let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
        "DuDecisionChoices",
        json!([]),
    )])))
    .unwrap();
    assert!(validation::compile(&parsed, &reference).is_err());
}
