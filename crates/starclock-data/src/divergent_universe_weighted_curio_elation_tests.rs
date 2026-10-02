use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe::DivergentUniverseBundleCandidate;
use crate::divergent_universe_decisions::weighted_curio_elations::WeightedCurioElationPolicy;
use crate::divergent_universe_decisions::{BUNDLE, DecisionCatalog, validation};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[test]
fn production_weighted_curio_elation_preserves_all_three_operands_and_policy_boundary() {
    let reference = load_divergent_universe_bundle().unwrap();
    let first = DecisionCatalog::production(&reference).unwrap();
    let fresh = DecisionCatalog::production(&load_divergent_universe_bundle().unwrap()).unwrap();
    assert_eq!(
        first.weighted_curio_elations(),
        fresh.weighted_curio_elations()
    );
    assert_eq!(first.digest(), fresh.digest());
    assert_eq!(first.weighted_curio_elations().len(), 1);
    let row = &first.weighted_curio_elations()[0];
    assert_eq!(row.key.as_ref(), "du.weighted-curio-elation.sapient-pen");
    assert_eq!(
        row.weighted_curio.as_str(),
        "divergent-universe.weighted-curio.1015"
    );
    assert_eq!(row.maze_buff_id.as_ref(), "633415");
    assert_eq!(
        (
            row.punchline_gain,
            row.elation_bonus_millionths,
            row.duration_turns
        ),
        (2, 500_000, 2)
    );
    assert_eq!(
        row.policy,
        WeightedCurioElationPolicy::VersionedProjectPolicyActionResolvedOriginalPartyRefresh
    );
    assert_eq!(row.sources.len(), 4);
    assert!(row.policy_note.contains("authored but not yet executed"));
    assert!(
        row.policy_note
            .contains("separate unimplemented assembly policy fields")
    );
    assert!(row.replacement_condition.contains("Low confidence"));
    assert!(
        row.replacement_condition
            .contains("do not prove battle execution")
    );
}

#[test]
fn weighted_curio_elation_rejects_wrong_joins_noncanonical_operands_and_missing_policy() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_weighted_curio_elations()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for (field, value) in [
        ("id", json!(0)),
        ("stable_key", json!("UPPER")),
        (
            "weighted_curio_key",
            json!("divergent-universe.weighted-curio.1003"),
        ),
        ("maze_buff_id", json!("633403")),
        ("character_paths", json!(["Priest"])),
        ("character_paths", json!(["Elation", "Elation"])),
        ("gain_parameter", json!(2)),
        ("bonus_parameter", json!(1)),
        ("duration_parameter", json!(2)),
        ("punchline_gain", json!(1)),
        ("punchline_gain", json!(65536)),
        ("elation_bonus", json!("0.50")),
        ("elation_bonus", json!("50")),
        ("elation_bonus", json!(0.5)),
        ("duration_turns", json!(1)),
        ("policy", json!("Exact")),
        ("summary_en", json!("")),
        ("summary_zh_cn", json!("")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
        ("source_ids", json!([107, 108, 109])),
        ("source_ids", json!([107, 108, 109, 109])),
        ("source_ids", json!([103, 104, 105, 106])),
        ("source_ids", json!([107, 108, 109, 110, 106])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        assert_rejected("DuWeightedCurioElations", rows, &reference, field);
    }
    for rows in [json!([]), json!([baseline[0].clone(), baseline[0].clone()])] {
        assert_rejected("DuWeightedCurioElations", rows, &reference, "cardinality");
    }
}

#[test]
fn weighted_curio_elation_rejects_forged_provenance_at_every_required_source() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_decision_sources()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for id in 107..=110 {
        for field in [
            "url",
            "revision",
            "game_version",
            "access_date",
            "sha256",
            "locator",
            "quality",
        ] {
            let mut rows = baseline.clone();
            rows.as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["id"] == id)
                .unwrap()[field] = json!("forged");
            assert_rejected(
                "DuDecisionSources",
                rows,
                &reference,
                &format!("{id}:{field}"),
            );
        }
    }
}

#[test]
fn weighted_curio_elation_key_cannot_alias_an_existing_effect_family() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let mut rows = serde_json::to_value(
        config
            .du_weighted_curio_elations()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    rows[0]["stable_key"] = json!(
        config
            .du_weighted_curio_necroses()
            .ordered_rows()
            .next()
            .unwrap()
            .stable_key
    );
    assert_rejected(
        "DuWeightedCurioElations",
        rows,
        &reference,
        "cross-family duplicate",
    );
}

fn assert_rejected(
    table: &'static str,
    rows: Value,
    reference: &DivergentUniverseBundleCandidate,
    label: &str,
) {
    let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(table, rows)])));
    assert!(
        parsed.is_err() || validation::compile(&parsed.unwrap(), reference).is_err(),
        "{label}"
    );
}
