use std::sync::{Arc, OnceLock};

use sha2::{Digest, Sha256};
use starclock_mode_universe::{
    catalog::UniverseCatalog,
    definition::DomainKind,
    encounter::{EncounterSelectionPolicy, EnemyRole},
    encounter_content_runtime::{
        EncounterContentRuntimeCatalog, EncounterContentRuntimeError, EncounterSelection,
    },
    id::EncounterPoolId,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] =
    include_bytes!("../../../../../config/universe-generated/config.sora");
const EXPECTED_KEYS: [&str; 32] = [
    "universe.encounter-pool.room.200213",
    "universe.encounter-pool.room.200221",
    "universe.encounter-pool.room.200222",
    "universe.encounter-pool.room.200223",
    "universe.encounter-pool.room.200231",
    "universe.encounter-pool.room.200232",
    "universe.encounter-pool.room.200233",
    "universe.encounter-pool.room.200241",
    "universe.encounter-pool.room.200242",
    "universe.encounter-pool.room.200243",
    "universe.encounter-pool.room.200611",
    "universe.encounter-pool.room.200612",
    "universe.encounter-pool.room.200713",
    "universe.encounter-pool.room.201",
    "universe.encounter-pool.room.202",
    "universe.encounter-pool.room.203",
    "universe.encounter-pool.room.300111",
    "universe.encounter-pool.room.300112",
    "universe.encounter-pool.room.300121",
    "universe.encounter-pool.room.300122",
    "universe.encounter-pool.room.300131",
    "universe.encounter-pool.room.300132",
    "universe.encounter-pool.room.300141",
    "universe.encounter-pool.room.300142",
    "universe.encounter-pool.room.300152",
    "universe.encounter-pool.room.300211",
    "universe.encounter-pool.room.300212",
    "universe.encounter-pool.room.300213",
    "universe.encounter-pool.room.300221",
    "universe.encounter-pool.room.300222",
    "universe.encounter-pool.room.300223",
    "universe.encounter-pool.room.300231",
];

fn catalog() -> &'static UniverseCatalog {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    CATALOG
        .get_or_init(|| {
            let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
            UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog")
        })
        .as_ref()
}

fn pool(key: &str) -> EncounterPoolId {
    catalog()
        .encounter_pools()
        .iter()
        .find(|pool| pool.stable_key() == key)
        .expect("frozen encounter pool")
        .id()
}

fn text(hasher: &mut Sha256, value: &str) {
    hasher.update(u64::try_from(value.len()).unwrap().to_le_bytes());
    hasher.update(value.as_bytes());
}

#[test]
fn goal07_p5_m15_s21_materializes_all_frozen_encounter_pools_exactly() {
    let selected = catalog()
        .encounter_pools()
        .iter()
        .filter(|pool| EXPECTED_KEYS.contains(&pool.stable_key()))
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), EXPECTED_KEYS.len());
    assert!(
        selected
            .iter()
            .map(|pool| pool.stable_key())
            .eq(EXPECTED_KEYS)
    );
    assert_eq!(
        selected
            .iter()
            .filter(|pool| {
                pool.selection_policy()
                    == EncounterSelectionPolicy::ExactConditionThenWeightedStableOrder
            })
            .count(),
        30
    );
    assert_eq!(
        selected
            .iter()
            .filter(|pool| {
                pool.selection_policy() == EncounterSelectionPolicy::WorldDifficultyBossEliteBinding
            })
            .count(),
        1
    );
    assert_eq!(
        selected
            .iter()
            .filter(|pool| {
                pool.selection_policy()
                    == EncounterSelectionPolicy::ConditionThenGroupOrDifficultyBinding
            })
            .count(),
        1
    );
    assert_eq!(
        selected
            .iter()
            .map(|pool| pool.weighted().len())
            .sum::<usize>(),
        60
    );
    assert_eq!(
        selected
            .iter()
            .map(|pool| pool.fixed().len())
            .sum::<usize>(),
        2
    );
    assert_eq!(
        selected
            .iter()
            .filter(|pool| pool.domain_kind() == DomainKind::Boss)
            .count(),
        2
    );
    assert_eq!(
        selected
            .iter()
            .filter(|pool| pool.domain_kind() == DomainKind::Elite)
            .count(),
        2
    );

    let mut hasher = Sha256::new();
    text(
        &mut hasher,
        "starclock-goal07-p5-m15-s21-encounter-pools-v1",
    );
    for pool in selected {
        hasher.update(pool.id().get().to_le_bytes());
        text(&mut hasher, pool.stable_key());
        hasher.update(pool.room().get().to_le_bytes());
        hasher.update([pool.domain_kind() as u8]);
        text(&mut hasher, pool.map_entrance());
        hasher.update([pool.selection_policy() as u8]);
        text(&mut hasher, pool.source_primary_condition_key());
        text(&mut hasher, pool.text().name_en());
        text(&mut hasher, pool.text().name_zh_cn());
        text(&mut hasher, pool.text().summary_en());
        text(&mut hasher, pool.text().summary_zh_cn());
        hasher.update(u64::try_from(pool.fixed().len()).unwrap().to_le_bytes());
        for binding in pool.fixed() {
            text(&mut hasher, binding.condition_key());
            text(&mut hasher, binding.source_content_id());
        }
        hasher.update(u64::try_from(pool.weighted().len()).unwrap().to_le_bytes());
        for binding in pool.weighted() {
            text(&mut hasher, binding.condition_key());
            hasher.update(binding.group().get().to_le_bytes());
            hasher.update(binding.weight().coefficient().to_le_bytes());
            hasher.update([binding.weight().scale()]);
        }
    }
    let digest: [u8; 32] = hasher.finalize().into();
    assert_eq!(
        digest,
        [
            164, 75, 95, 49, 43, 138, 140, 10, 165, 86, 94, 242, 33, 93, 105, 222, 92, 247, 252,
            251, 192, 89, 138, 166, 12, 146, 84, 233, 65, 72, 40, 84,
        ]
    );
}

#[test]
fn goal07_p5_m15_s21_selects_exact_condition_key_in_stable_weighted_order() {
    let runtime =
        EncounterContentRuntimeCatalog::compile(catalog()).expect("encounter content runtime");
    let difficulty = catalog().worlds()[0].difficulties()[0];
    let pool = pool("universe.encounter-pool.room.201");
    let selected = runtime
        .resolve(catalog(), pool, "19", difficulty)
        .expect("weighted condition");
    let EncounterSelection::WeightedGroups(groups) = selected else {
        panic!("weighted encounter groups");
    };
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].weight().coefficient(), 1);
    assert_eq!(groups[0].weight().scale(), 0);
    assert_eq!(
        catalog()
            .encounter_group(groups[0].group())
            .expect("encounter group")
            .source_group_id(),
        "2003"
    );
    assert_eq!(
        runtime.resolve(catalog(), pool, "not-offered", difficulty),
        Err(EncounterContentRuntimeError::ConditionNotOffered)
    );
}

#[test]
fn goal07_p5_m15_s21_resolves_world_difficulty_boss_binding() {
    let runtime =
        EncounterContentRuntimeCatalog::compile(catalog()).expect("encounter content runtime");
    let difficulty = catalog().worlds()[0].difficulties()[0];
    let selected = runtime
        .resolve(
            catalog(),
            pool("universe.encounter-pool.room.203"),
            "29",
            difficulty,
        )
        .expect("difficulty binding");
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
        .find(|binding| binding.difficulty() == difficulty && binding.role() == role)
        .expect("world difficulty boss binding");
    assert_eq!(enemy_variant_key.as_ref(), binding.enemy_variant_key());
    assert_eq!(level, binding.level());
}
