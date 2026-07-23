use std::sync::{Arc, OnceLock};

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    id::OccurrenceChoiceId,
    occurrence::{AuthoredScalarUnit, OccurrenceOperation, OccurrenceTarget, RandomOutcomePolicy},
    occurrence_effect_runtime::{
        OCCURRENCE_EFFECT_RUNTIME_REVISION, OccurrenceEffectRuntimeCatalog,
    },
    run_runtime::RunRuntimeCatalog,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

fn catalog() -> &'static UniverseCatalog {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    CATALOG
        .get_or_init(|| {
            let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core");
            UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe")
        })
        .as_ref()
}

fn runtime() -> OccurrenceEffectRuntimeCatalog {
    let run = RunRuntimeCatalog::compile(catalog()).expect("run");
    OccurrenceEffectRuntimeCatalog::compile(catalog(), &run).expect("Occurrence effects")
}

fn choice(key: &str) -> OccurrenceChoiceId {
    catalog()
        .occurrence_choices()
        .iter()
        .find(|value| value.stable_key() == key)
        .unwrap()
        .id()
}

#[test]
fn complete_occurrence_partition_compiles() {
    let runtime = runtime();
    assert_eq!(
        OCCURRENCE_EFFECT_RUNTIME_REVISION,
        "standard-universe-occurrence-effect-runtime-v1"
    );
    assert_eq!((runtime.content_count(), runtime.rule_count()), (447, 0));
    assert_eq!(
        (runtime.choice_count(), runtime.random_policy_count()),
        (321, 52)
    );
    assert_eq!(
        runtime.digest(),
        [
            14, 130, 109, 192, 13, 60, 34, 124, 241, 48, 123, 225, 160, 166, 143, 32, 131, 250,
            135, 58, 76, 84, 108, 86, 233, 105, 143, 175, 252, 157, 192, 88,
        ]
    );
}

#[test]
fn every_choice_executes_to_one_source_attributed_plan() {
    let runtime = runtime();
    for id in runtime.choice_ids() {
        let effect = runtime.execute(id).unwrap();
        assert_eq!(effect.choice(), id);
        assert!(!effect.source_key().is_empty());
        assert_eq!(
            effect.condition_keys(),
            catalog().occurrence_choice(id).unwrap().condition_keys()
        );
        assert!(!effect.outcome().operations().is_empty());
    }
}

#[test]
fn cost_result_branch_and_random_policy_remain_distinct() {
    let runtime = runtime();
    let effect = runtime
        .execute(choice("universe.occurrence.39.variant.12201.choice.05"))
        .unwrap();
    assert_eq!(effect.costs().len(), 1);
    assert_eq!(effect.costs()[0].operation(), OccurrenceOperation::Discard);
    assert_eq!(effect.costs()[0].targets(), &[OccurrenceTarget::Blessing]);
    assert_eq!(
        effect.outcome().operations(),
        &[OccurrenceOperation::Obtain, OccurrenceOperation::Discard]
    );
    assert_eq!(
        effect.outcome().random_policy(),
        Some(RandomOutcomePolicy::StableUniformOrderedCandidates)
    );
}

#[test]
fn authored_percent_chance_and_battle_handoff_are_typed() {
    let runtime = runtime();
    let chance = runtime
        .execute(choice("universe.occurrence.41.variant.12401.choice.08"))
        .unwrap();
    assert_eq!(chance.outcome().operations(), &[OccurrenceOperation::Lose]);
    assert_eq!(
        chance.outcome().targets(),
        &[OccurrenceTarget::CosmicFragments]
    );
    assert_eq!(
        chance.outcome().numeric_literals()[0].unit(),
        AuthoredScalarUnit::Percent
    );
    assert_eq!(chance.outcome().chance_percentages()[0].coefficient(), 50);

    let battle = runtime
        .execute(choice("universe.occurrence.62.variant.19501.choice.04"))
        .unwrap();
    assert_eq!(
        battle.outcome().operations(),
        &[OccurrenceOperation::Battle]
    );
    assert!(battle.outcome().targets().is_empty());
}

#[test]
fn special_transition_is_explicit_and_not_an_implicit_resource_mutation() {
    let effect = runtime()
        .execute(choice("universe.occurrence.40.variant.12301.choice.01"))
        .unwrap();
    assert_eq!(
        effect.outcome().operations(),
        &[OccurrenceOperation::Special]
    );
    assert!(effect.outcome().targets().is_empty());
    assert!(effect.outcome().numeric_literals().is_empty());
}

#[test]
fn nine_frozen_occurrence_operation_fixtures_are_runtime_backed() {
    let runtime = runtime();
    let fixtures = [
        (
            "universe.occurrence.62.variant.19501.choice.04",
            OccurrenceOperation::Battle,
        ),
        (
            "universe.occurrence.43.variant.12601.choice.01",
            OccurrenceOperation::Consume,
        ),
        (
            "universe.occurrence.39.variant.12201.choice.05",
            OccurrenceOperation::Discard,
        ),
        (
            "universe.occurrence.39.variant.12201.choice.01",
            OccurrenceOperation::Enhance,
        ),
        (
            "universe.occurrence.41.variant.12401.choice.08",
            OccurrenceOperation::Lose,
        ),
        (
            "universe.occurrence.39.variant.12201.choice.02",
            OccurrenceOperation::Obtain,
        ),
        (
            "universe.occurrence.43.variant.12601.choice.01",
            OccurrenceOperation::Repair,
        ),
        (
            "universe.occurrence.8.variant.10601.choice.03",
            OccurrenceOperation::Restore,
        ),
        (
            "universe.occurrence.40.variant.12301.choice.01",
            OccurrenceOperation::Special,
        ),
    ];
    for (key, operation) in fixtures {
        let effect = runtime.execute(choice(key)).unwrap();
        assert!(effect.outcome().operations().contains(&operation));
    }
}
