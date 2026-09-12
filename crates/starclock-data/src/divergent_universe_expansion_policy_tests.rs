use super::{EditedRows, compile};
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_decisions::{DecisionCatalog, EquationExpansionRewardPolicy},
    divergent_universe_decisions_generated::SoraConfig,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(super) fn production(catalog: &DecisionCatalog) {
    let [rule] = catalog.equation_expansion_rewards() else {
        panic!("one reviewed rule")
    };
    assert_eq!(rule.state.as_str(), "divergent-universe.curio-state.9074");
    assert_eq!(rule.effect_id.as_ref(), "2074");
    assert_eq!((rule.count, rule.trigger_limit), (1, 3));
    assert_eq!(rule.policy, EquationExpansionRewardPolicy::VersionedProjectPolicyActivePreStateUniformUnownedBoundedCascade);
    assert_eq!(rule.sources.len(), 3);
}

pub(super) fn table(config: &SoraConfig) -> (&'static str, Value) {
    (
        "DuEquationExpansionRewards",
        serde_json::to_value(
            config
                .du_equation_expansion_rewards()
                .ordered_rows()
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    )
}

pub(super) fn validate(
    baseline: &BTreeMap<&'static str, Value>,
    reference: &DivergentUniverseBundleCandidate,
) {
    for (field, value) in [
        ("count", json!(2)),
        ("trigger_limit", json!(4)),
        ("count_parameter", json!(2)),
        ("limit_parameter", json!(1)),
        ("minimum_rarity", json!(2)),
        ("maximum_rarity", json!(2)),
        ("state_key", json!("divergent-universe.curio-state.9075")),
        ("effect_id", json!("2075")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
        ("source_ids", json!([])),
        ("source_ids", json!([1])),
        ("summary_en", json!("")),
        ("summary_zh_cn", json!("")),
    ] {
        let mut tables = baseline.clone();
        tables.get_mut("DuEquationExpansionRewards").unwrap()[0][field] = value;
        let config = SoraConfig::from_source(&EditedRows(tables)).unwrap();
        assert!(compile(&config, reference).is_err(), "accepted {field}");
    }
    for (state, effect) in [("9072", "2072"), ("9075", "2075")] {
        let mut tables = baseline.clone();
        let row = &mut tables.get_mut("DuEquationExpansionRewards").unwrap()[0];
        row["state_key"] = json!(format!("divergent-universe.curio-state.{state}"));
        row["effect_id"] = json!(effect);
        let config = SoraConfig::from_source(&EditedRows(tables)).unwrap();
        assert!(
            compile(&config, reference).is_err(),
            "same-count non-expansion effect admitted"
        );
    }
    for values in [
        json!([]),
        json!([
            baseline["DuEquationExpansionRewards"][0].clone(),
            baseline["DuEquationExpansionRewards"][0].clone()
        ]),
    ] {
        let mut tables = baseline.clone();
        tables.insert("DuEquationExpansionRewards", values);
        let result = SoraConfig::from_source(&EditedRows(tables));
        assert!(result.is_err() || compile(&result.unwrap(), reference).is_err());
    }
}
