use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use starclock_activity::ActivityTerminalOutcome;
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_mode_universe::divergent_universe::{
    DivergentUniverseBaselineFixture, encode_divergent_universe_replay,
    record_divergent_universe_selected_run, verify_divergent_universe_selected_replay,
};

const MATRIX: &str = include_str!(
    "../../../content-manifests/divergent-universe-runtime-v1/verification-contract.json"
);

// This checks authored target assignment only. IDs from the contract must
// never count as evidence that a run executed the corresponding gameplay.
#[test]
fn generated_matrix_declares_each_target_exactly_once() {
    let contract: Value = serde_json::from_str(MATRIX).expect("generated matrix parses");
    let rows = contract["matrix_cases"]
        .as_array()
        .expect("matrix cases exist");
    let mut declared = BTreeMap::<String, BTreeSet<String>>::new();
    for row in rows {
        collect_targets(row, &mut declared);
    }
    for (axis, expected) in contract["axis_denominators"]
        .as_object()
        .expect("axis denominators exist")
    {
        assert_eq!(
            declared.get(axis).map_or(0, BTreeSet::len),
            usize::try_from(expected.as_u64().expect("denominator is u64"))
                .expect("denominator fits usize"),
            "{axis} declaration drift"
        );
    }
}

// Area/difficulty baseline smoke test, not a complete-content release matrix.
#[test]
#[ignore = "explicit Goal 22 generated legal matrix"]
fn generated_legal_matrix_completes_real_battles_and_fresh_replay() {
    let contract: Value = serde_json::from_str(MATRIX).expect("generated matrix parses");
    let rows = contract["matrix_cases"]
        .as_array()
        .expect("matrix cases exist");
    assert_eq!(rows.len(), 104);
    let fixture = DivergentUniverseBaselineFixture::production()
        .expect("production Divergent Universe fixture loads");
    let matrix_filter = std::env::var("STARCLOCK_DIVERGENT_UNIVERSE_MATRIX_ID").ok();
    let mut families = BTreeSet::new();
    let mut areas = BTreeSet::new();
    let mut difficulties = BTreeSet::new();
    let mut layers = BTreeSet::new();
    let mut executed = 0_usize;

    for row in rows {
        let case_id = text(row, "case_id");
        if matrix_filter
            .as_deref()
            .is_some_and(|filter| filter != case_id)
        {
            continue;
        }
        executed += 1;
        let selected = &row["selected_run"];
        let family = match text(selected, "run_family") {
            "Ordinary" => DivergentUniverseRunFamily::Ordinary,
            "Cyclical" => DivergentUniverseRunFamily::Cyclical,
            other => panic!("{case_id}: unsupported run family {other}"),
        };
        let area = text(selected, "area_id");
        let difficulty = text(selected, "difficulty_id");
        let layer = text(selected, "layer_id");
        let finish = text(selected, "finish_condition_id");
        let seed = seed(text(row, "seed_hex"));

        let flow = fixture
            .flow_for_selection(family, area, difficulty)
            .unwrap_or_else(|error| panic!("{case_id}: selected flow rejected: {error:?}"));
        assert_eq!(flow.run_family(), family, "{case_id}");
        assert_eq!(flow.area().as_str(), area, "{case_id}");
        assert_eq!(flow.difficulty().as_str(), difficulty, "{case_id}");
        assert!(
            flow.layers().iter().any(|value| value.as_str() == layer),
            "{case_id}: selected layer is not in the compiled area"
        );
        assert!(
            flow.finish_conditions()
                .iter()
                .any(|value| value.as_str() == finish),
            "{case_id}: selected finish condition is not in the compiled profile"
        );

        let recorded =
            record_divergent_universe_selected_run(&fixture, family, area, difficulty, seed)
                .unwrap_or_else(|error| panic!("{case_id}: complete run failed: {error:?}"));
        assert_eq!(
            recorded.report().terminal(),
            ActivityTerminalOutcome::Completed
        );
        assert_eq!(recorded.report().completed_battles(), 1, "{case_id}");
        assert!(recorded.battle_command_count() > 0, "{case_id}");
        let replay = encode_divergent_universe_replay(&recorded)
            .unwrap_or_else(|error| panic!("{case_id}: replay encode failed: {error:?}"));
        let verification =
            verify_divergent_universe_selected_replay(&replay, &fixture, family, area, difficulty)
                .unwrap_or_else(|error| panic!("{case_id}: fresh replay failed: {error:?}"));
        assert_eq!(verification.run_family(), family, "{case_id}");
        assert_eq!(verification.terminal(), ActivityTerminalOutcome::Completed);
        assert_eq!(verification.battle_count(), 1, "{case_id}");
        assert!(verification.battle_command_count() > 0, "{case_id}");

        families.insert(text(selected, "run_family").to_owned());
        areas.insert(area.to_owned());
        difficulties.insert(difficulty.to_owned());
        layers.insert(layer.to_owned());
        eprintln!("{case_id}: {area} {difficulty} completed and replayed");
    }

    assert_eq!(executed, matrix_filter.as_ref().map_or(rows.len(), |_| 1));
    if matrix_filter.is_none() {
        assert_eq!(families.len(), denominator(&contract, "run_families"));
        assert_eq!(areas.len(), denominator(&contract, "areas"));
        assert_eq!(difficulties.len(), denominator(&contract, "difficulties"));
        assert_eq!(layers.len(), denominator(&contract, "layers"));
    }
}

fn collect_targets(row: &Value, covered: &mut BTreeMap<String, BTreeSet<String>>) {
    for (axis, targets) in row["targets"].as_object().expect("case targets exist") {
        let values = covered.entry(axis.clone()).or_default();
        for target in targets.as_array().expect("axis target list") {
            assert!(
                values.insert(target.as_str().expect("target is text").to_owned()),
                "duplicate matrix target on {axis}"
            );
        }
    }
}

fn denominator(contract: &Value, axis: &str) -> usize {
    contract["axis_denominators"][axis]
        .as_u64()
        .unwrap_or_else(|| panic!("{axis} denominator is u64")) as usize
}

fn text<'a>(value: &'a Value, field: &str) -> &'a str {
    value[field]
        .as_str()
        .unwrap_or_else(|| panic!("{field} is text"))
}

fn seed(value: &str) -> u64 {
    assert_eq!(value.len(), 64, "matrix seed is SHA-256 hex");
    u64::from_str_radix(&value[..16], 16).expect("matrix seed prefix is hexadecimal")
}
