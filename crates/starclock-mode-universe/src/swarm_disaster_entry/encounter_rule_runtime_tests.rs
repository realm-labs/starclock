use crate::error::UniverseCatalogLoadErrorKind;

use super::EncounterRuleRuntimeCatalog;
use crate::swarm_disaster_entry::{SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance};

#[test]
fn exact_sora_rule_binds_and_contract_drift_fails_closed() {
    let factory = factory();
    let _instance = instance(&factory, "swarm-disaster.area.201");

    let mut input = input(&factory);
    input.domain = "Activity".into();
    assert_eq!(
        EncounterRuleRuntimeCatalog::compile(input)
            .unwrap_err()
            .kind(),
        UniverseCatalogLoadErrorKind::InvalidReference
    );
}

#[test]
fn trigger_slot_and_ordered_program_drift_are_rejected_before_entry_use() {
    let factory = factory();
    let mut trigger = input(&factory);
    trigger.triggers[0] = "BattleStarted".into();
    assert!(EncounterRuleRuntimeCatalog::compile(trigger).is_err());

    let mut program = input(&factory);
    program.program = program
        .program
        .replace("ReviewBossAlternative", "ReviewUnknown")
        .into();
    assert!(EncounterRuleRuntimeCatalog::compile(program).is_err());
}

#[test]
fn all_formal_difficulties_share_one_immutable_encounter_contract() {
    let factory = factory();
    let expected = instance(&factory, "swarm-disaster.area.201").encounter_rule_runtime_digest();
    for area in [201, 202, 203, 204, 205] {
        assert_eq!(
            instance(&factory, &format!("swarm-disaster.area.{area}"))
                .encounter_rule_runtime_digest(),
            expected
        );
    }
}

fn input(
    factory: &SwarmDisasterRuntimeFactory,
) -> crate::swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput {
    factory
        .content
        .mechanic_rule_runtime_input("encounter-selection")
        .unwrap()
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::super::tests::BUNDLE).unwrap()
}

fn instance(factory: &SwarmDisasterRuntimeFactory, area: &str) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(super::super::tests::released_entry(
            area,
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
            super::super::tests::participants(super::super::tests::policy()),
        ))
        .unwrap()
}
