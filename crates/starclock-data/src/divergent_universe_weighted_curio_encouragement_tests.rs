use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_decisions::{
        BUNDLE, DecisionCatalog, validation,
        weighted_curio_encouragements::WeightedCurioEncouragementPolicy,
    },
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[test]
fn production_weighted_curio_encouragement_preserves_operand_without_execution_credit() {
    let reference = load_divergent_universe_bundle().unwrap();
    let first = DecisionCatalog::production(&reference).unwrap();
    let fresh = DecisionCatalog::production(&load_divergent_universe_bundle().unwrap()).unwrap();
    assert_eq!(
        first.weighted_curio_encouragements(),
        fresh.weighted_curio_encouragements()
    );
    assert_eq!(first.digest(), fresh.digest());
    assert_eq!(first.weighted_curio_encouragements().len(), 1);
    let row = &first.weighted_curio_encouragements()[0];
    assert_eq!(
        row.key.as_ref(),
        "du.weighted-curio-encouragement.encouragement-for-you"
    );
    assert_eq!(
        row.weighted_curio.as_str(),
        "divergent-universe.weighted-curio.1008"
    );
    assert_eq!(row.maze_buff_id.as_ref(), "633408");
    assert_eq!(row.follow_up_crit_damage_millionths, 1_500_000);
    assert_eq!(
        row.policy,
        WeightedCurioEncouragementPolicy::VersionedProjectPolicyOriginalElationFollowUpDamage
    );
    assert_eq!(row.sources.len(), 4);
    assert!(row.policy_note.starts_with("VersionedProjectPolicy:"));
    assert!(
        row.policy_note
            .contains("without changing DamageClass::Elation")
    );
    assert!(
        row.policy_note
            .contains("does not implement or admit the effect")
    );
    assert!(row.replacement_condition.contains("Low confidence"));
    assert!(
        row.replacement_condition
            .contains("do not prove battle execution")
    );
}

#[test]
fn weighted_curio_encouragement_rejects_wrong_joins_noncanonical_operands_and_false_parity() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_weighted_curio_encouragements()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for (field, value) in [
        ("id", json!(0)),
        ("stable_key", json!("UPPER")),
        (
            "weighted_curio_key",
            json!("divergent-universe.weighted-curio.1015"),
        ),
        (
            "weighted_curio_key",
            json!("divergent-universe.weighted-curio.missing"),
        ),
        ("maze_buff_id", json!("633415")),
        ("character_paths", json!(["Memory"])),
        ("character_paths", json!([])),
        ("character_paths", json!(["Elation", "Elation"])),
        ("crit_parameter", json!(2)),
        ("follow_up_crit_damage", json!("1.50")),
        ("follow_up_crit_damage", json!("150")),
        ("follow_up_crit_damage", json!("-1.5")),
        ("follow_up_crit_damage", json!(1.5)),
        ("policy", json!("Exact")),
        ("summary_en", json!("")),
        ("summary_zh_cn", json!("")),
        ("policy_note", json!("")),
        ("policy_note", json!("Observed parity")),
        ("replacement_condition", json!("")),
        ("source_ids", json!([118, 119, 120])),
        ("source_ids", json!([118, 119, 120, 120])),
        ("source_ids", json!([114, 115, 116, 117])),
        ("source_ids", json!([118, 119, 120, 121, 117])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        assert_rejected("DuWeightedCurioEncouragements", rows, &reference, field);
    }
    for rows in [json!([]), json!([baseline[0].clone(), baseline[0].clone()])] {
        assert_rejected(
            "DuWeightedCurioEncouragements",
            rows,
            &reference,
            "cardinality",
        );
    }
}

#[test]
fn weighted_curio_encouragement_rejects_forged_provenance_at_every_required_source() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_decision_sources()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for id in 118..=121 {
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
fn weighted_curio_encouragement_key_cannot_alias_an_existing_effect_family() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let mut rows = serde_json::to_value(
        config
            .du_weighted_curio_encouragements()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    rows[0]["stable_key"] = json!(
        config
            .du_weighted_curio_elations()
            .ordered_rows()
            .next()
            .unwrap()
            .stable_key
    );
    assert_rejected(
        "DuWeightedCurioEncouragements",
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
