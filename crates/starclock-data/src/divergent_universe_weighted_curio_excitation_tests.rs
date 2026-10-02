use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe::DivergentUniverseBundleCandidate;
use crate::divergent_universe_decisions::weighted_curio_excitations::WeightedCurioExcitationPolicy;
use crate::divergent_universe_decisions::{BUNDLE, DecisionCatalog, validation};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[test]
fn production_weighted_curio_excitation_preserves_five_operands_and_separate_cap_policy() {
    let reference = load_divergent_universe_bundle().unwrap();
    let first = DecisionCatalog::production(&reference).unwrap();
    let fresh = DecisionCatalog::production(&load_divergent_universe_bundle().unwrap()).unwrap();
    assert_eq!(
        first.weighted_curio_excitations(),
        fresh.weighted_curio_excitations()
    );
    assert_eq!(first.digest(), fresh.digest());
    assert_eq!(first.weighted_curio_excitations().len(), 1);
    let row = &first.weighted_curio_excitations()[0];
    assert_eq!(
        row.key.as_ref(),
        "du.weighted-curio-excitation.genius-confusion"
    );
    assert_eq!(
        row.weighted_curio.as_str(),
        "divergent-universe.weighted-curio.1006"
    );
    assert_eq!(row.maze_buff_id.as_ref(), "633406");
    assert_eq!(
        (
            row.stack_gain,
            row.stack_consumption,
            row.attack_multiplier_millionths,
            row.base_chance_millionths,
            row.duration_turns
        ),
        (2, 1, 2_500_000, 500_000, 1)
    );
    assert_eq!(row.policy_maximum_stacks, 65535);
    assert_eq!(row.policy, WeightedCurioExcitationPolicy::VersionedProjectPolicyEffectiveGainActionResolvedTeamConsumption);
    assert_eq!(row.sources.len(), 4);
    assert!(row.policy_note.starts_with("VersionedProjectPolicy:"));
    for text in [
        "policy maximum 65535",
        "Entanglement",
        "This authoring batch does not implement or admit the effect",
    ] {
        assert!(row.policy_note.contains(text), "{text}");
    }
    assert!(row.replacement_condition.contains("Low confidence"));
    assert!(
        row.replacement_condition
            .contains("do not prove battle execution")
    );
}

#[test]
fn weighted_curio_excitation_rejects_wrong_joins_each_operand_and_policy() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_weighted_curio_excitations()
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
        ("maze_buff_id", json!("633415")),
        ("character_elements", json!(["Physical"])),
        ("character_elements", json!(["Quantum", "Quantum"])),
        ("character_elements", json!(["Quantum", "Wind"])),
        ("gain_parameter", json!(2)),
        ("consume_parameter", json!(1)),
        ("damage_parameter", json!(4)),
        ("chance_parameter", json!(3)),
        ("duration_parameter", json!(4)),
        ("stack_gain", json!(1)),
        ("stack_gain", json!(65536)),
        ("stack_consumption", json!(2)),
        ("attack_multiplier", json!("2.50")),
        ("attack_multiplier", json!("250")),
        ("attack_multiplier", json!(2.5)),
        ("base_chance", json!("0.50")),
        ("base_chance", json!("50")),
        ("base_chance", json!(0.5)),
        ("duration_turns", json!(2)),
        ("policy_maximum_stacks", json!(0)),
        ("policy_maximum_stacks", json!(65534)),
        ("policy_maximum_stacks", json!(65536)),
        ("policy", json!("Exact")),
        ("summary_en", json!("")),
        ("summary_zh_cn", json!("")),
        ("policy_note", json!("")),
        ("policy_note", json!("Exact: source parity")),
        ("replacement_condition", json!("")),
        ("source_ids", json!([114, 115, 116])),
        ("source_ids", json!([114, 115, 116, 116])),
        ("source_ids", json!([107, 108, 109, 110])),
        ("source_ids", json!([114, 115, 116, 117, 113])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        assert_rejected("DuWeightedCurioExcitations", rows, &reference, field);
    }
    for rows in [json!([]), json!([baseline[0].clone(), baseline[0].clone()])] {
        assert_rejected(
            "DuWeightedCurioExcitations",
            rows,
            &reference,
            "cardinality",
        );
    }
}

#[test]
fn weighted_curio_excitation_rejects_forged_provenance_at_every_required_source() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_decision_sources()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for id in 114..=117 {
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
fn weighted_curio_excitation_key_cannot_alias_an_existing_effect_family() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let mut rows = serde_json::to_value(
        config
            .du_weighted_curio_excitations()
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
        "DuWeightedCurioExcitations",
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
