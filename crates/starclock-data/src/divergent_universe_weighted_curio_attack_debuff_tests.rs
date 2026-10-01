use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe_decisions::{BUNDLE, DecisionCatalog, validation};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn production_weighted_curio_attack_debuff_preserves_both_operands_and_duration() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    assert_eq!(catalog.weighted_curio_attack_debuffs().len(), 1);
    let row = &catalog.weighted_curio_attack_debuffs()[0];
    assert_eq!(
        row.weighted_curio.as_str(),
        "divergent-universe.weighted-curio.1014"
    );
    assert_eq!(row.maze_buff_id.as_ref(), "633414");
    assert_eq!(
        (
            row.advance_millionths,
            row.reduction_millionths,
            row.duration_turns
        ),
        (200_000, 300_000, 1)
    );
    assert_eq!(row.sources.len(), 4);
    assert!(row.policy_note.contains("not hit"));
    assert!(row.replacement_condition.contains("not observed parity"));
}

#[test]
fn weighted_curio_attack_debuff_rejects_invalid_joins_operands_and_provenance() {
    let config = SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap();
    let reference = load_divergent_universe_bundle().unwrap();
    let baseline = serde_json::to_value(
        config
            .du_weighted_curio_attack_debuffs()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for (field, value) in [
        ("id", json!(0)),
        ("stable_key", json!("UPPER")),
        ("weighted_curio_key", json!("unknown")),
        (
            "weighted_curio_key",
            json!("divergent-universe.weighted-curio.1002"),
        ),
        ("maze_buff_id", json!("633402")),
        ("character_paths", json!(["Warrior"])),
        ("character_paths", json!(["Warlock", "Warrior"])),
        ("character_paths", json!(["Shaman"])),
        ("advance_parameter", json!(2)),
        ("reduction_parameter", json!(1)),
        ("duration_parameter", json!(2)),
        ("duration_turns", json!(0)),
        ("duration_turns", json!(65536)),
        ("policy", json!("Exact")),
        ("summary_en", json!(" ")),
        ("summary_zh_cn", json!("")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
        ("source_ids", json!([])),
        ("source_ids", json!([81, 82, 83])),
        ("source_ids", json!([81, 82, 83, 83])),
        ("source_ids", json!([999])),
        ("source_ids", json!([77, 78, 79, 80])),
    ]
    .into_iter()
    .chain(
        ["advance_fraction", "reduction_fraction"]
            .into_iter()
            .flat_map(|field| {
                ["0", "1.2", "-0.2", "0.20", "0.0000001", "0.2e0"]
                    .into_iter()
                    .map(move |value| (field, json!(value)))
            }),
    ) {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
            "DuWeightedCurioAttackDebuffs",
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
    rows[1]["stable_key"] = json!("du.weighted-curio-attack-debuff.duplicate");
    let parsed = SoraConfig::from_source(&EditedRows(BTreeMap::from([(
        "DuWeightedCurioAttackDebuffs",
        rows,
    )])))
    .unwrap();
    assert!(validation::compile(&parsed, &reference).is_err());
}
