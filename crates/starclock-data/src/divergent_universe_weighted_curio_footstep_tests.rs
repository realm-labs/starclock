use crate::divergent_universe_decisions::tests::transport::EditedRows;
use crate::{
    divergent_universe::load_divergent_universe_bundle,
    divergent_universe_decisions::{
        BUNDLE, DecisionCatalog, DecisionDataError, weighted_curio_footsteps,
        weighted_curio_footsteps::WeightedCurioFootstepPolicy,
    },
    divergent_universe_decisions_generated::{SoraConfig, runtime::SoraBundle},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn config() -> SoraConfig {
    SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap()
}
fn rows() -> Vec<Value> {
    config()
        .du_weighted_curio_footsteps()
        .ordered_rows()
        .map(|row| serde_json::to_value(row).unwrap())
        .collect()
}
fn check(rows: Vec<Value>) -> Result<(), DecisionDataError> {
    let source = EditedRows(BTreeMap::from([("DuWeightedCurioFootsteps", json!(rows))]));
    let config = SoraConfig::from_source(&source).map_err(|_| DecisionDataError::Transport)?;
    weighted_curio_footsteps::compile(&config, &load_divergent_universe_bundle().unwrap())
        .map(|_| ())
}

#[test]
fn weighted_curio_footstep_production_loads_exact_operands_and_distinct_policy_sources() {
    let reference = load_divergent_universe_bundle().unwrap();
    let a = DecisionCatalog::production(&reference).unwrap();
    let b = DecisionCatalog::production(&reference).unwrap();
    assert_eq!(a.digest(), b.digest());
    assert_eq!(a.weighted_curio_footsteps(), b.weighted_curio_footsteps());
    assert_eq!(a.weighted_curio_footsteps().len(), 1);
    let row = &a.weighted_curio_footsteps()[0];
    assert_eq!(
        row.weighted_curio.as_str(),
        "divergent-universe.weighted-curio.1011"
    );
    assert_eq!(
        (
            row.loss_fraction_millionths,
            row.damage_per_stack_millionths,
            row.maximum_stacks
        ),
        (500_000, 80_000, 10)
    );
    assert_eq!(
        row.policy,
        WeightedCurioFootstepPolicy::VersionedProjectPolicyEffectiveHpLossAfterSkillOriginalDamage
    );
    assert_eq!(row.sources.len(), 7);
    assert!(row.hp_policy_note.contains("low-confidence"));
    assert!(row.skill_policy_note.contains("low-confidence"));
    assert!(!row.hp_replacement_condition.is_empty());
    assert!(!row.skill_replacement_condition.is_empty());
}

#[test]
fn weighted_curio_footstep_rejects_missing_duplicate_foreign_and_eligibility_drift() {
    assert!(check(vec![]).is_err());
    let current = rows();
    assert!(check(vec![current[0].clone(), current[0].clone()]).is_err());
    for (field, value) in [
        ("stable_key", json!("foreign")),
        (
            "weighted_curio_key",
            json!("divergent-universe.weighted-curio.1016"),
        ),
        ("maze_buff_id", json!("633416")),
        ("character_paths", json!(["Warrior"])),
        ("character_paths", json!(["Memory", "Warrior"])),
        ("loss_parameter", json!(2)),
        ("damage_parameter", json!(3)),
        ("cap_parameter", json!(1)),
    ] {
        let mut changed = current.clone();
        changed[0][field] = value;
        assert!(check(changed).is_err(), "{field}");
    }
}

#[test]
fn weighted_curio_footstep_rejects_operand_precision_numeric_transport_and_false_parity() {
    for (field, value) in [
        ("loss_fraction", json!("0.50")),
        ("loss_fraction", json!("0.500001")),
        ("loss_fraction", json!(1)),
        ("damage_per_stack", json!("0.080000")),
        ("damage_per_stack", json!("8")),
        ("maximum_stacks", json!(9)),
        ("maximum_stacks", json!(0)),
        ("policy", json!("ExactRuleIr")),
        ("hp_policy_note", json!("Observed")),
        ("skill_policy_note", json!("Observed")),
        ("hp_replacement_condition", json!("")),
        ("skill_replacement_condition", json!("")),
        ("source_ids", json!([134, 135, 136, 137, 138, 139])),
    ] {
        let mut changed = rows();
        changed[0][field] = value;
        assert!(check(changed).is_err(), "{field}");
    }
}

#[test]
fn weighted_curio_footstep_each_required_source_rejects_forged_evidence() {
    let reference = load_divergent_universe_bundle().unwrap();
    for id in 134..=140 {
        for (field, value) in [
            ("sha256", json!("0".repeat(64))),
            ("quality", json!("ObservedCommunity")),
            ("revision", json!("foreign")),
            ("game_version", json!("4.3")),
            ("locator", json!("foreign")),
        ] {
            let mut sources = config()
                .du_decision_sources()
                .ordered_rows()
                .map(|row| serde_json::to_value(row).unwrap())
                .collect::<Vec<_>>();
            sources.iter_mut().find(|row| row["id"] == id).unwrap()[field] = value;
            let source = EditedRows(BTreeMap::from([("DuDecisionSources", json!(sources))]));
            let config = SoraConfig::from_source(&source).unwrap();
            assert!(
                weighted_curio_footsteps::compile(&config, &reference).is_err(),
                "{id}/{field}"
            );
        }
    }
    for field in ["hp_policy_note", "skill_policy_note"] {
        let mut changed = rows();
        changed[0][field] = json!("VersionedProjectPolicy: altered semantics");
        assert_eq!(check(changed), Err(DecisionDataError::InvalidProvenance));
    }
}
