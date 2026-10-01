use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe_decisions::{BUNDLE, DecisionCatalog, validation};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn production_weighted_curio_break_effect_has_exact_raise_to_operand_and_independent_policy() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    assert_eq!(catalog.weighted_curio_break_effects().len(), 1);
    let row = &catalog.weighted_curio_break_effects()[0];
    assert_eq!(
        row.weighted_curio.as_str(),
        "divergent-universe.weighted-curio.1005"
    );
    assert_eq!(row.maze_buff_id.as_ref(), "633405");
    assert_eq!(row.multiplier_millionths, 1_200_000);
    assert_eq!(row.sources.len(), 4);
    assert!(row.policy_note.contains("One ApplyEffect"));
    assert!(row.replacement_condition.contains("not observed parity"));
}

#[test]
fn weighted_curio_break_effect_rejects_wrong_joins_noncanonical_values_and_missing_policy() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_weighted_curio_break_effects()
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
        ("maze_buff_id", json!("633401")),
        ("character_elements", json!(["Wind"])),
        ("character_elements", json!(["Thunder", "Wind"])),
        ("character_elements", json!(["Wind", "Fire"])),
        ("multiplier_parameter", json!(2)),
        ("multiplier", json!("1.20")),
        ("multiplier", json!("0.9")),
        ("multiplier", json!("2.2")),
        ("multiplier", json!(1.2)),
        ("policy", json!("Exact")),
        ("summary_en", json!("")),
        ("summary_zh_cn", json!("")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
        ("source_ids", json!([99, 100, 101])),
        ("source_ids", json!([99, 100, 101, 101])),
        ("source_ids", json!([93, 94, 95, 96])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
            "DuWeightedCurioBreakEffects",
            rows,
        )])));
        assert!(
            parsed.is_err() || validation::compile(&parsed.unwrap(), &reference).is_err(),
            "{field}"
        );
    }
    for rows in [json!([]), json!([baseline[0].clone(), baseline[0].clone()])] {
        let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
            "DuWeightedCurioBreakEffects",
            rows,
        )])));
        assert!(parsed.is_err() || validation::compile(&parsed.unwrap(), &reference).is_err());
    }
}

#[test]
fn weighted_curio_break_effect_rejects_forged_provenance_at_every_required_source() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_decision_sources()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for id in 99..=102 {
        for field in [
            "url",
            "revision",
            "game_version",
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
            let parsed =
                SoraConfig::from_source(&EditedRows(BTreeMap::from([("DuDecisionSources", rows)])));
            assert!(
                parsed.is_err() || validation::compile(&parsed.unwrap(), &reference).is_err(),
                "{id}: {field}"
            );
        }
    }
}
