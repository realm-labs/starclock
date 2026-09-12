use super::{load_divergent_universe_bundle, load_divergent_universe_bundle_from_bytes};
use crate::divergent_universe_blessing_catalog::DivergentUniverseBlessingCatalog;
use crate::divergent_universe_catalog::{
    DivergentUniverseAreaKind, DivergentUniverseDifficultyId, DivergentUniverseFlowCatalog,
    DivergentUniverseRunFamily,
};
use crate::divergent_universe_curio_catalog::{
    DivergentUniverseCurioCatalog, DivergentUniverseCurioStateId,
};
use crate::divergent_universe_encounter_catalog::{
    DivergentUniverseEncounterCatalog, DivergentUniverseEncounterGroupId,
};
use crate::divergent_universe_equation_catalog::{
    DivergentUniverseEquationCatalog, DivergentUniverseEquationId,
};
use crate::divergent_universe_mapping_catalog::{
    DivergentUniverseAvatarLocator, DivergentUniverseMappingCatalog,
    DivergentUniverseMappingCondition, DivergentUniverseMappingDecimal,
    DivergentUniverseMappingOperation, DivergentUniverseMappingRuleKind,
    DivergentUniverseMappingTiming, DivergentUniversePublicIdentityResolution,
};
use crate::divergent_universe_mechanic_catalog::{
    DivergentUniverseMechanicCatalog, DivergentUniverseMechanicSourceId,
};
use crate::divergent_universe_progression_catalog::{
    DivergentUniverseProgressionCatalog, DivergentUniverseProtocolId,
};
use crate::divergent_universe_service_catalog::{
    DivergentUniverseServiceCatalog, DivergentUniverseWorkbenchFunctionId,
};
use crate::divergent_universe_titan_catalog::{
    DivergentUniverseTitanBoonId, DivergentUniverseTitanCatalog,
};

const PRODUCTION_BUNDLE: &[u8] =
    include_bytes!("../../../config/divergent-universe-generated/config.sora");

#[test]
fn production_bundle_decodes_every_table_and_binds_current_identity() {
    let candidate = load_divergent_universe_bundle().expect("production bundle must validate");
    let identity = candidate.identity();
    assert_eq!(identity.schema_fingerprint(), "6dfaf7aba644d1c9");
    assert_eq!(identity.table_count(), 81);
    assert_eq!(identity.row_count(), 28_732);
    assert_eq!(identity.source_count(), 8_171);
    assert_eq!(identity.source_obligation_count(), 6_762);
    assert_eq!(identity.empty_table_count(), 2);
    assert_eq!(
        hex(identity.schema_digest().bytes()),
        "9bb73ea806d3fb13cbec1ad25c010cb1b6e66067978281ea885e58748cde5452",
    );
    assert_eq!(
        hex(identity.configuration_digest().bytes()),
        "2768fc939dee45c07233136f27c101e3f5a52c1c80ca11ec239dbac9c1a8a993",
    );
    assert_eq!(
        hex(identity.content_digest().bytes()),
        "3dd6b8460b8a24c82192c960537a13e2c24c058332709a08605f427458d4465a",
    );
    assert_eq!(
        hex(identity.source_digest().bytes()),
        "b92226b9743586baee2d4a93a65bcb59ca3853b835af82639be386d3d8c61656",
    );
    assert_eq!(
        hex(identity.component_digest().bytes()),
        "143a186a103995cb26d060cac71b8d15b94e086b507bff68ff97b2ac68527b19",
    );
}

#[test]
fn production_flow_catalog_closes_exact_topology_without_promoting_rooms() {
    let candidate = load_divergent_universe_bundle().expect("production bundle must validate");
    let catalog = candidate.catalog();
    assert_eq!(catalog.entries().len(), 2);
    assert_eq!(catalog.finish_conditions().len(), 13);
    assert_eq!(catalog.areas().len(), 28);
    assert_eq!(catalog.difficulties().len(), 22);
    assert_eq!(catalog.layers().len(), 11);
    assert_eq!(catalog.room_candidates().len(), 848);
    assert_eq!(catalog.flows().len(), 111);
    assert_eq!(catalog.cyclical_challenges().len(), 13);
    assert_eq!(
        catalog
            .areas()
            .iter()
            .filter(|area| area.kind == DivergentUniverseAreaKind::WeekChallenge)
            .count(),
        13,
    );
    assert_eq!(
        catalog
            .areas()
            .iter()
            .filter(|area| area.run_family() == DivergentUniverseRunFamily::Ordinary)
            .count(),
        15,
    );
    assert!(
        catalog
            .room_candidates()
            .iter()
            .all(|room| room.stage_refs.is_empty() && room.offered_pool_ids.is_empty())
    );
    assert_eq!(
        catalog
            .flows()
            .iter()
            .filter(|flow| flow.area.is_none())
            .count(),
        3
    );
}

#[test]
fn flow_catalog_rejects_an_invalid_area_difficulty_join() {
    let candidate = load_divergent_universe_bundle().expect("production bundle must validate");
    let mut parts = candidate.catalog().clone().into_parts();
    parts.areas[0].difficulties[0] =
        DivergentUniverseDifficultyId::new("divergent-universe.difficulty.invalid")
            .expect("well-formed fixture ID");

    let error = DivergentUniverseFlowCatalog::new(parts).expect_err("unknown join must fail");
    assert_eq!(error.to_string(), "area difficulty/layer closure drift");
}

#[test]
fn production_mapping_catalog_closes_sources_builds_and_lifecycle_boundaries() {
    let candidate = load_divergent_universe_bundle().expect("production bundle must validate");
    let catalog = candidate.mapping_catalog();
    assert_eq!(catalog.eligibility().len(), 84);
    assert_eq!(catalog.builds().len(), 95);
    assert_eq!(catalog.rules().len(), 7);
    assert_eq!(catalog.source_obligations(), 258);
    assert_eq!(
        catalog
            .builds()
            .iter()
            .filter(|build| build.public_identity
                == DivergentUniversePublicIdentityResolution::ResolvedAvatarConfig)
            .count(),
        91,
    );
    assert_eq!(
        catalog
            .builds()
            .iter()
            .filter(|build| build.special_avatar.is_some())
            .count(),
        79,
    );
    assert!(catalog.builds().iter().all(|build| !build.runtime_lowered));

    let light_cone = catalog
        .rules()
        .iter()
        .find(|rule| rule.kind == DivergentUniverseMappingRuleKind::LightCone)
        .expect("Light Cone boundary");
    assert_eq!(
        light_cone.timing,
        DivergentUniverseMappingTiming::Unspecified
    );
    assert_eq!(
        light_cone.condition,
        DivergentUniverseMappingCondition::Unspecified
    );
    assert_eq!(
        light_cone.operations.as_ref(),
        [DivergentUniverseMappingOperation::Unspecified]
    );

    let teardown = catalog
        .rules()
        .iter()
        .find(|rule| rule.kind == DivergentUniverseMappingRuleKind::Teardown)
        .expect("teardown boundary");
    assert!(!teardown.account_mutation);
    assert_eq!(
        teardown.timing,
        DivergentUniverseMappingTiming::RunFinalization
    );
}

#[test]
fn mapping_catalog_rejects_an_invalid_eligibility_build_join() {
    let candidate = load_divergent_universe_bundle().expect("production bundle must validate");
    let mut parts = candidate.mapping_catalog().clone().into_parts();
    parts.eligibility[0].avatar =
        DivergentUniverseAvatarLocator::new("999999").expect("well-formed fixture locator");

    let error = DivergentUniverseMappingCatalog::new(parts).expect_err("unknown join must fail");
    assert_eq!(error.to_string(), "eligibility has no mapping build");
}

#[test]
fn mapping_decimal_rejects_noncanonical_or_floating_inputs() {
    for invalid in ["", "+1", "01", "1.0", "1e-1", "NaN"] {
        assert!(
            DivergentUniverseMappingDecimal::new(invalid).is_err(),
            "accepted {invalid}",
        );
    }
    for valid in ["0", "1", "-1", "0.3", "-0.25"] {
        assert_eq!(
            DivergentUniverseMappingDecimal::new(valid)
                .expect("canonical decimal")
                .as_str(),
            valid,
        );
    }
}

#[test]
fn production_equation_and_blessing_catalogs_close_all_p1_b4_definitions() {
    let candidate = load_divergent_universe_bundle().expect("production bundle must validate");
    let equations = candidate.equation_catalog();
    assert_eq!(equations.equations().len(), 80);
    assert_eq!(equations.categories().len(), 4);
    assert_eq!(equations.recipes().len(), 80);
    assert_eq!(equations.offers().len(), 136);
    assert_eq!(equations.progress().len(), 80);
    assert_eq!(equations.states().len(), 160);
    assert_eq!(equations.effects().len(), 25);
    assert_eq!(equations.transitions().len(), 4);
    assert_eq!(equations.source_obligations(), 330);
    assert!(
        equations
            .offers()
            .iter()
            .all(|offer| offer.candidate_ids.is_empty())
    );

    let blessings = candidate.blessing_catalog();
    assert_eq!(blessings.paths().len(), 8);
    assert_eq!(blessings.blessings().len(), 414);
    assert_eq!(blessings.levels().len(), 828);
    assert_eq!(blessings.rewrites().len(), 416);
    assert_eq!(blessings.groups().len(), 118);
    assert_eq!(blessings.contributions().len(), 414);
    assert_eq!(blessings.source_obligations(), 1_368);
    assert_eq!(
        blessings
            .rewrites()
            .iter()
            .filter(|rewrite| rewrite.input.is_none() && rewrite.output.is_none())
            .count(),
        2,
    );
}

#[test]
fn equation_and_blessing_catalogs_reject_invalid_cross_joins() {
    let candidate = load_divergent_universe_bundle().expect("production bundle must validate");
    let mut equation_parts = candidate.equation_catalog().clone().into_parts();
    equation_parts.recipes[0].equation =
        DivergentUniverseEquationId::new("divergent-universe.equation.invalid")
            .expect("well-formed fixture ID");
    let error = DivergentUniverseEquationCatalog::new(equation_parts)
        .expect_err("unknown recipe join must fail");
    assert_eq!(
        error.to_string(),
        "Equation definition/recipe/progress/state closure drift"
    );

    let mut blessing_parts = candidate.blessing_catalog().clone().into_parts();
    blessing_parts.contributions[0].equations[0] =
        DivergentUniverseEquationId::new("divergent-universe.equation.invalid")
            .expect("well-formed fixture ID");
    let error = DivergentUniverseBlessingCatalog::new(blessing_parts, candidate.equation_catalog())
        .expect_err("unknown contribution join must fail");
    assert_eq!(
        error.to_string(),
        "Blessing-to-Equation contribution closure drift"
    );
}

#[test]
fn production_p1_b5_catalogs_close_all_definitions_without_execution_credit() {
    let candidate = load_divergent_universe_bundle().expect("production bundle must validate");

    let curios = candidate.curio_catalog();
    assert_eq!(curios.curios().len(), 179);
    assert_eq!(curios.states().len(), 235);
    assert_eq!(curios.groups().len(), 286);
    assert_eq!(curios.lifecycle().len(), 179);
    assert_eq!(curios.pool_membership().len(), 235);
    assert_eq!(curios.miracles().len(), 17);
    assert_eq!(curios.miracle_eligibility().len(), 74);
    assert_eq!(curios.miracle_states().len(), 34);
    assert_eq!(curios.source_obligations(), 774);
    assert!(
        curios
            .groups()
            .iter()
            .all(|group| group.candidates.is_empty())
    );

    let titans = candidate.titan_catalog();
    assert_eq!(titans.types().len(), 12);
    assert_eq!(titans.boons().len(), 84);
    assert_eq!(titans.talents().len(), 36);
    assert_eq!(titans.choices().len(), 36);
    assert_eq!(titans.contributions().len(), 120);
    assert_eq!(titans.source_obligations(), 132);

    let progression = candidate.progression_catalog();
    assert_eq!(progression.protocols().len(), 8);
    assert_eq!(progression.divisions().len(), 9);
    assert_eq!(progression.modes().len(), 2);
    assert_eq!(progression.cognoculi().len(), 9);
    assert_eq!(progression.talents().len(), 38);
    assert_eq!(progression.unlocks().len(), 97);
    assert_eq!(progression.constants().len(), 34);
    assert_eq!(progression.weekly_modifiers().len(), 103);
    assert_eq!(progression.room_marks().len(), 24);
    assert_eq!(progression.effects().len(), 38);
    assert_eq!(progression.source_obligations(), 313);
    assert!(
        progression
            .protocols()
            .iter()
            .all(|row| !row.runtime_lowered)
    );
}

#[test]
fn p1_b5_catalogs_reject_invalid_cross_joins() {
    let candidate = load_divergent_universe_bundle().expect("production bundle must validate");

    let mut curio_parts = candidate.curio_catalog().clone().into_parts();
    curio_parts.curios[0].states[0] =
        DivergentUniverseCurioStateId::new("divergent-universe.curio-state.invalid")
            .expect("well-formed fixture ID");
    let error = DivergentUniverseCurioCatalog::new(curio_parts)
        .expect_err("unknown Curio state join must fail");
    assert_eq!(error.to_string(), "Curio state/lifecycle closure drift");

    let mut titan_parts = candidate.titan_catalog().clone().into_parts();
    titan_parts.choices[0].candidates[0] =
        DivergentUniverseTitanBoonId::new("divergent-universe.titan-boon.invalid")
            .expect("well-formed fixture ID");
    let error = DivergentUniverseTitanCatalog::new(titan_parts)
        .expect_err("unknown Titan choice join must fail");
    assert_eq!(error.to_string(), "Titan choice closure drift");

    let mut progression_parts = candidate.progression_catalog().clone().into_parts();
    progression_parts.divisions[0].protocols[0] =
        DivergentUniverseProtocolId::new("divergent-universe.protocol.invalid")
            .expect("well-formed fixture ID");
    let error = DivergentUniverseProgressionCatalog::new(progression_parts, candidate.catalog())
        .expect_err("unknown Division Protocol join must fail");
    assert_eq!(error.to_string(), "Astronomical Division closure drift");
}

#[test]
fn production_mechanic_catalog_preserves_reference_only_identity() {
    let candidate = load_divergent_universe_bundle().expect("production bundle must validate");
    let catalog = candidate.mechanic_catalog();
    assert_eq!(catalog.rules().len(), 669);
    assert_eq!(catalog.sources().len(), 669);
    assert_eq!(catalog.semantic_families().len(), 25);
    assert_eq!(catalog.source_obligations(), 694);
    assert!(catalog.rules().iter().all(|rule| !rule.runtime_lowered));
    assert!(catalog.sources().iter().all(|source| {
        !source.runtime_lowered && source.disposition.as_ref() == "ReferenceOnlyNotLowered"
    }));
    assert!(
        catalog
            .semantic_families()
            .iter()
            .all(|family| !family.runtime_executable)
    );
}

#[test]
fn mechanic_catalog_rejects_an_unknown_source_join() {
    let candidate = load_divergent_universe_bundle().expect("production bundle must validate");
    let mut parts = candidate.mechanic_catalog().clone().into_parts();
    parts.rules[0].source =
        DivergentUniverseMechanicSourceId::new("divergent-universe.mechanic-source.invalid")
            .expect("well-formed fixture ID");
    let error = DivergentUniverseMechanicCatalog::new(parts)
        .expect_err("unknown mechanic source join must fail");
    assert_eq!(error.to_string(), "unknown mechanic source");
}

#[test]
fn production_service_catalog_closes_p1_b6_fail_closed_definitions() {
    let candidate = load_divergent_universe_bundle().expect("production bundle must validate");
    let catalog = candidate.service_catalog();
    assert_eq!(catalog.currencies().len(), 2);
    assert_eq!(catalog.workbenches().len(), 11);
    assert_eq!(catalog.functions().len(), 6);
    assert_eq!(catalog.gamble_groups().len(), 126);
    assert_eq!(catalog.gamble_units().len(), 89);
    assert_eq!(catalog.curse_chests().len(), 29);
    assert_eq!(catalog.occurrences().len(), 118);
    assert_eq!(catalog.variants().len(), 97);
    assert_eq!(catalog.mode_services().len(), 23);
    assert_eq!(catalog.service_rules().len(), 6);
    assert_eq!(catalog.offers().len(), 161);
    assert_eq!(catalog.adventures().len(), 32);
    assert_eq!(catalog.source_obligations(), 531);
    assert!(
        catalog
            .offers()
            .iter()
            .all(|offer| offer.candidates.is_empty())
    );
    assert!(catalog.variants().iter().all(|variant| {
        !variant.runtime_lowered && variant.graph_resolution.as_ref() == "MissingAtPinnedRevision"
    }));
}

#[test]
fn service_catalog_rejects_an_unknown_workbench_function() {
    let candidate = load_divergent_universe_bundle().expect("production bundle must validate");
    let mut parts = candidate.service_catalog().clone().into_parts();
    parts.workbenches[0].functions[0] =
        DivergentUniverseWorkbenchFunctionId::new("divergent-universe.workbench-function.invalid")
            .expect("well-formed fixture ID");
    let error = DivergentUniverseServiceCatalog::new(parts)
        .expect_err("unknown Workbench function must fail");
    assert_eq!(error.to_string(), "Workbench closure drift");
}

#[test]
fn production_encounter_catalog_preserves_zero_promoted_encounters() {
    let candidate = load_divergent_universe_bundle().expect("production bundle must validate");
    let catalog = candidate.encounter_catalog();
    assert_eq!(catalog.sources().len(), 877);
    assert_eq!(catalog.groups().len(), 43);
    assert_eq!(catalog.waves().len(), 176);
    assert_eq!(catalog.slots().len(), 385);
    assert_eq!(catalog.boss_pools().len(), 618);
    assert_eq!(catalog.source_obligations(), 877);
    assert!(
        catalog
            .sources()
            .iter()
            .all(|source| !source.runtime_lowered)
    );
    assert!(catalog.groups().iter().all(|group| {
        !group.runtime_lowered
            && group.reachability_disposition.as_ref() == "UnprovenWeeklyDisplayCandidate"
    }));
}

#[test]
fn encounter_catalog_rejects_an_unknown_display_group() {
    let candidate = load_divergent_universe_bundle().expect("production bundle must validate");
    let mut parts = candidate.encounter_catalog().clone().into_parts();
    parts.boss_pools[0].encounter_group =
        DivergentUniverseEncounterGroupId::new("divergent-universe.encounter-group.invalid")
            .expect("well-formed fixture ID");
    let error = DivergentUniverseEncounterCatalog::new(
        parts,
        candidate.catalog(),
        candidate.progression_catalog(),
    )
    .expect_err("unknown display group must fail");
    assert_eq!(error.to_string(), "display boss-pool boundary drift");
}

#[test]
fn repeated_load_has_identical_content_source_and_component_identity() {
    let first = load_divergent_universe_bundle().expect("first load");
    let second = load_divergent_universe_bundle_from_bytes(PRODUCTION_BUNDLE).expect("second load");
    assert_eq!(first.identity(), second.identity());
}

#[test]
fn non_sora_and_truncated_inputs_fail_before_a_candidate_exists() {
    let json = load_divergent_universe_bundle_from_bytes(br#"{"table":[]}"#)
        .expect_err("JSON must be rejected");
    assert!(
        json.to_string().contains("magic") || json.to_string().contains("header"),
        "unexpected error: {json}",
    );

    let truncated = load_divergent_universe_bundle_from_bytes(&PRODUCTION_BUNDLE[..64])
        .expect_err("truncated bundle must be rejected");
    assert!(
        truncated.to_string().contains("directory")
            || truncated.to_string().contains("payload")
            || truncated.to_string().contains("section"),
        "unexpected error: {truncated}",
    );
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
