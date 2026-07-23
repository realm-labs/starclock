use std::sync::{Arc, OnceLock};

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    encounter::{EncounterSelectionPolicy, EnemyRole, WavePolicy},
    encounter_content_runtime::{
        ENCOUNTER_CONTENT_RUNTIME_REVISION, EncounterContentRuntimeCatalog, EncounterSelection,
    },
    id::EncounterPoolId,
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

fn runtime() -> EncounterContentRuntimeCatalog {
    EncounterContentRuntimeCatalog::compile(catalog()).expect("encounter content")
}

fn pool(key: &str) -> EncounterPoolId {
    catalog()
        .encounter_pools()
        .iter()
        .find(|pool| pool.stable_key() == key)
        .unwrap()
        .id()
}

#[test]
fn complete_encounter_world_and_difficulty_partition_compiles() {
    let runtime = runtime();
    assert_eq!(
        ENCOUNTER_CONTENT_RUNTIME_REVISION,
        "standard-universe-encounter-content-runtime-v1"
    );
    assert_eq!(
        (
            runtime.content_count(),
            runtime.rule_count(),
            runtime.semantic_fixture_count()
        ),
        (959, 0, 4)
    );
    assert_eq!(runtime.enemy_variant_keys().len(), 86);
    assert_eq!(
        (
            runtime.bundled_enemy_definition_count(),
            runtime.extension_enemy_definition_count()
        ),
        (13, 73)
    );
    assert_eq!(
        runtime.digest(),
        [
            32, 196, 139, 154, 115, 201, 166, 104, 172, 173, 95, 94, 235, 144, 92, 75, 20, 141,
            197, 208, 135, 117, 28, 209, 249, 32, 155, 176, 250, 91, 65, 3,
        ]
    );
}

#[test]
fn exact_condition_policy_returns_authored_weighted_group() {
    let runtime = runtime();
    let difficulty = catalog().worlds()[0].difficulties()[0];
    let selected = runtime
        .resolve(
            catalog(),
            pool("universe.encounter-pool.room.201"),
            "19",
            difficulty,
        )
        .unwrap();
    let EncounterSelection::WeightedGroups(groups) = selected else {
        panic!("weighted group");
    };
    assert_eq!(groups.len(), 1);
    assert_eq!(
        catalog()
            .encounter_group(groups[0].group())
            .unwrap()
            .source_group_id(),
        "2003"
    );
    assert_eq!(groups[0].weight().coefficient(), 1);
    assert_eq!(
        catalog()
            .encounter_pool(pool("universe.encounter-pool.room.201"))
            .unwrap()
            .selection_policy(),
        EncounterSelectionPolicy::ExactConditionThenWeightedStableOrder
    );
}

#[test]
fn condition_then_group_policy_preserves_fixed_source_content() {
    let runtime = runtime();
    let difficulty = catalog().worlds()[0].difficulties()[0];
    let selected = runtime
        .resolve(
            catalog(),
            pool("universe.encounter-pool.room.100"),
            "10",
            difficulty,
        )
        .unwrap();
    let EncounterSelection::FixedContent { source_content_id } = selected else {
        panic!("fixed source content");
    };
    assert_eq!(source_content_id.as_ref(), "1006");
}

#[test]
fn world_difficulty_policy_resolves_exact_boss_binding() {
    let runtime = runtime();
    let difficulty = catalog().worlds()[0].difficulties()[0];
    let selected = runtime
        .resolve(
            catalog(),
            pool("universe.encounter-pool.room.203"),
            "29",
            difficulty,
        )
        .unwrap();
    let EncounterSelection::DifficultyEnemy {
        role,
        enemy_variant_key,
        level,
    } = selected
    else {
        panic!("difficulty enemy");
    };
    assert_eq!(role, EnemyRole::Boss);
    let binding = catalog()
        .difficulty_enemy_bindings()
        .iter()
        .find(|value| value.difficulty() == difficulty && value.role() == EnemyRole::Boss)
        .unwrap();
    assert_eq!(enemy_variant_key.as_ref(), binding.enemy_variant_key());
    assert_eq!(level, binding.level());
}

#[test]
fn single_wave_fixture_and_all_compositions_are_runtime_backed() {
    let group = catalog()
        .encounter_groups()
        .iter()
        .find(|group| group.source_group_id() == "1001")
        .unwrap();
    assert_eq!(group.wave_policy(), WavePolicy::SingleWave);
    assert_eq!(group.members().len(), 1);
    assert_eq!(group.members()[0].waves().len(), 1);
    assert!(!group.members()[0].waves()[0].enemies().is_empty());
    assert!(
        catalog()
            .encounter_groups()
            .iter()
            .flat_map(|group| group.members())
            .flat_map(|member| member.waves())
            .all(|wave| !wave.enemies().is_empty())
    );
}
