use std::{
    collections::BTreeSet,
    sync::{Arc, OnceLock},
};

use starclock_mode_universe::{
    ability_runtime::{
        ABILITY_RUNTIME_REVISION, AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope,
        AbilityRuntimeCatalog, AbilityTarget,
    },
    catalog::UniverseCatalog,
    progression::AbilityOperation,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

fn catalog() -> &'static UniverseCatalog {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    CATALOG
        .get_or_init(|| {
            let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
            UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog")
        })
        .as_ref()
}

#[test]
fn all_ten_operations_and_fifty_effects_compile_to_a_closed_runtime() {
    let catalog = catalog();
    let runtime = AbilityRuntimeCatalog::compile(catalog).expect("Ability runtime");
    let selected = catalog
        .ability_tree_nodes()
        .iter()
        .map(|node| node.id())
        .collect::<Vec<_>>();
    let operations = runtime
        .project(
            &selected,
            AbilityExecutionContext::new(
                AbilityProjectionScope::Run,
                AbilityBoundary::AfterBattle,
                14,
                true,
            ),
        )
        .expect("run projection")
        .applied_effects()
        .iter()
        .map(|effect| effect.operation())
        .chain(
            runtime
                .project(
                    &selected,
                    AbilityExecutionContext::new(
                        AbilityProjectionScope::Battle,
                        AbilityBoundary::BattleStart,
                        14,
                        true,
                    ),
                )
                .expect("battle projection")
                .applied_effects()
                .iter()
                .map(|effect| effect.operation()),
        )
        .collect::<BTreeSet<_>>();

    assert_eq!(
        ABILITY_RUNTIME_REVISION,
        "standard-universe-ability-runtime-v1"
    );
    assert_eq!(runtime.effect_count(), 50);
    assert_eq!(
        operations,
        BTreeSet::from([
            AbilityOperation::Unlock,
            AbilityOperation::AddStat,
            AbilityOperation::UnlockFormationSlot,
            AbilityOperation::Set,
            AbilityOperation::AddLimit,
            AbilityOperation::Enable,
            AbilityOperation::AddCurrency,
            AbilityOperation::AddChoice,
            AbilityOperation::AddResource,
            AbilityOperation::SetRatio,
        ])
    );
    assert_eq!(
        runtime.digest(),
        [
            23, 136, 165, 10, 193, 242, 98, 194, 13, 100, 24, 94, 11, 197, 237, 225, 123, 226, 54,
            120, 151, 86, 63, 80, 22, 178, 129, 141, 128, 47, 58, 110,
        ]
    );
}

#[test]
fn run_operations_execute_only_at_their_authored_boundaries() {
    let catalog = catalog();
    let runtime = AbilityRuntimeCatalog::compile(catalog).expect("Ability runtime");
    let selected = catalog
        .ability_tree_nodes()
        .iter()
        .map(|node| node.id())
        .collect::<Vec<_>>();
    let start = runtime
        .project(&selected, AbilityExecutionContext::run_start())
        .expect("run start");
    assert_integral(&start, AbilityTarget::InitialCosmicFragments, 50);
    assert_integral(&start, AbilityTarget::BlessingChoiceResetCount, 1);
    assert_eq!(start.value(AbilityTarget::RunPathResonance), None);
    assert_eq!(start.value(AbilityTarget::FirstBattleBlessingCount), None);

    let threshold = runtime
        .project(
            &selected,
            AbilityExecutionContext::new(
                AbilityProjectionScope::Run,
                AbilityBoundary::AfterBattle,
                14,
                true,
            ),
        )
        .expect("post-battle run projection");
    assert_integral(&threshold, AbilityTarget::RunPathResonance, 3);
    assert_integral(&threshold, AbilityTarget::FirstBattleBlessingCount, 1);
}

#[test]
fn battle_projection_aggregates_exact_stats_and_boundary_resources() {
    let catalog = catalog();
    let runtime = AbilityRuntimeCatalog::compile(catalog).expect("Ability runtime");
    let selected = catalog
        .ability_tree_nodes()
        .iter()
        .map(|node| node.id())
        .collect::<Vec<_>>();
    let start = runtime
        .project(
            &selected,
            AbilityExecutionContext::new(
                AbilityProjectionScope::Battle,
                AbilityBoundary::BattleStart,
                14,
                false,
            ),
        )
        .expect("battle start projection");
    assert_integral(&start, AbilityTarget::PartyAttackFlat, 420);
    assert_integral(&start, AbilityTarget::PartyDefenseFlat, 335);
    assert_integral(&start, AbilityTarget::PartyMaximumHpFlat, 640);
    assert_integral(&start, AbilityTarget::PathResonanceInitialEnergy, 20);
    assert_raw(&start, AbilityTarget::PathResonanceDamageRatio, 800_000);
    assert_raw(&start, AbilityTarget::PartyInitialEnergy, 1_000_000);
    assert_eq!(start.value(AbilityTarget::PartyEnergy), None);

    let elite = runtime
        .project(
            &selected,
            AbilityExecutionContext::new(
                AbilityProjectionScope::Battle,
                AbilityBoundary::EnterEliteOrBossDomain,
                14,
                false,
            ),
        )
        .expect("elite projection");
    assert_raw(&elite, AbilityTarget::PartyEnergy, 1_000_000);
    assert_eq!(elite.value(AbilityTarget::PartyInitialEnergy), None);
}

#[test]
fn all_ten_frozen_operation_fixtures_execute_the_expected_operation() {
    let runtime = AbilityRuntimeCatalog::compile(catalog()).expect("Ability runtime");
    let cases = [
        (21, AbilityOperation::AddChoice, run_after_battle()),
        (
            17,
            AbilityOperation::AddCurrency,
            AbilityExecutionContext::run_start(),
        ),
        (
            11,
            AbilityOperation::AddLimit,
            AbilityExecutionContext::run_start(),
        ),
        (27, AbilityOperation::AddResource, battle_start()),
        (2, AbilityOperation::AddStat, battle_start()),
        (
            12,
            AbilityOperation::Enable,
            AbilityExecutionContext::run_start(),
        ),
        (
            7,
            AbilityOperation::Set,
            AbilityExecutionContext::run_start(),
        ),
        (32, AbilityOperation::SetRatio, battle_start()),
        (
            1,
            AbilityOperation::Unlock,
            AbilityExecutionContext::run_start(),
        ),
        (6, AbilityOperation::UnlockFormationSlot, run_after_battle()),
    ];
    for (source_number, expected, context) in cases {
        let stable_key = format!("universe.ability-tree.{source_number}");
        let selected = [catalog()
            .ability_tree_nodes()
            .iter()
            .find(|node| node.stable_key() == stable_key)
            .expect("fixture node")
            .id()];
        let projection = runtime
            .project(&selected, context)
            .expect("fixture projection");
        assert!(
            projection
                .applied_effects()
                .iter()
                .any(|effect| effect.operation() == expected),
            "node {source_number} did not execute {expected:?}"
        );
    }
}

const fn battle_start() -> AbilityExecutionContext {
    AbilityExecutionContext::new(
        AbilityProjectionScope::Battle,
        AbilityBoundary::BattleStart,
        14,
        false,
    )
}

const fn run_after_battle() -> AbilityExecutionContext {
    AbilityExecutionContext::new(
        AbilityProjectionScope::Run,
        AbilityBoundary::AfterBattle,
        14,
        true,
    )
}

fn assert_integral(
    projection: &starclock_mode_universe::ability_runtime::AbilityRuntimeProjection,
    target: AbilityTarget,
    expected: i64,
) {
    assert_eq!(
        projection.value(target).and_then(|value| value.integral()),
        Some(expected)
    );
}

fn assert_raw(
    projection: &starclock_mode_universe::ability_runtime::AbilityRuntimeProjection,
    target: AbilityTarget,
    expected: i64,
) {
    assert_eq!(
        projection
            .value(target)
            .map(|value| value.raw_six_decimal()),
        Some(expected)
    );
}
