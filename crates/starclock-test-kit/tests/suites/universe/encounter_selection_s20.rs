use std::sync::{Arc, OnceLock};

use sha2::{Digest, Sha256};
use starclock_mode_universe::{
    catalog::UniverseCatalog,
    definition::DomainKind,
    encounter::EncounterSelectionPolicy,
    encounter_content_runtime::{
        EncounterContentRuntimeCatalog, EncounterContentRuntimeError, EncounterSelection,
    },
    id::EncounterPoolId,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] =
    include_bytes!("../../../../../config/universe-generated/config.sora");
const EXPECTED_KEYS: [&str; 32] = [
    "universe.encounter-pool.room.100",
    "universe.encounter-pool.room.1000001",
    "universe.encounter-pool.room.1000002",
    "universe.encounter-pool.room.1000003",
    "universe.encounter-pool.room.1000004",
    "universe.encounter-pool.room.1000005",
    "universe.encounter-pool.room.1000014",
    "universe.encounter-pool.room.1000015",
    "universe.encounter-pool.room.1000017",
    "universe.encounter-pool.room.1000018",
    "universe.encounter-pool.room.1000019",
    "universe.encounter-pool.room.1000020",
    "universe.encounter-pool.room.1000022",
    "universe.encounter-pool.room.1000024",
    "universe.encounter-pool.room.1000025",
    "universe.encounter-pool.room.1000026",
    "universe.encounter-pool.room.1000027",
    "universe.encounter-pool.room.1000029",
    "universe.encounter-pool.room.1000031",
    "universe.encounter-pool.room.1000032",
    "universe.encounter-pool.room.1000033",
    "universe.encounter-pool.room.200111",
    "universe.encounter-pool.room.200112",
    "universe.encounter-pool.room.200121",
    "universe.encounter-pool.room.200122",
    "universe.encounter-pool.room.200131",
    "universe.encounter-pool.room.200132",
    "universe.encounter-pool.room.200141",
    "universe.encounter-pool.room.200142",
    "universe.encounter-pool.room.200152",
    "universe.encounter-pool.room.200211",
    "universe.encounter-pool.room.200212",
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
fn goal07_p5_m15_s20_materializes_all_frozen_encounter_pools_exactly() {
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
        14
    );
    assert_eq!(
        selected
            .iter()
            .filter(|pool| {
                pool.selection_policy() == EncounterSelectionPolicy::WorldDifficultyBossEliteBinding
            })
            .count(),
        3
    );
    assert_eq!(
        selected
            .iter()
            .filter(|pool| {
                pool.selection_policy()
                    == EncounterSelectionPolicy::ConditionThenGroupOrDifficultyBinding
            })
            .count(),
        15
    );
    assert_eq!(
        selected
            .iter()
            .map(|pool| pool.weighted().len())
            .sum::<usize>(),
        61
    );
    assert_eq!(
        selected
            .iter()
            .map(|pool| pool.fixed().len())
            .sum::<usize>(),
        20
    );
    assert_eq!(
        selected
            .iter()
            .filter(|pool| pool.domain_kind() == DomainKind::Boss)
            .count(),
        5
    );
    assert_eq!(
        selected
            .iter()
            .filter(|pool| pool.domain_kind() == DomainKind::Elite)
            .count(),
        5
    );

    let mut hasher = Sha256::new();
    text(
        &mut hasher,
        "starclock-goal07-p5-m15-s20-encounter-pools-v1",
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
            223, 189, 65, 9, 217, 228, 139, 30, 187, 143, 12, 86, 89, 110, 67, 80, 204, 109, 88,
            207, 228, 227, 235, 81, 229, 63, 175, 146, 78, 192, 185, 225,
        ]
    );
}

#[test]
fn goal07_p5_m15_s20_selects_condition_before_group_or_difficulty_resolution() {
    let runtime =
        EncounterContentRuntimeCatalog::compile(catalog()).expect("encounter content runtime");
    let difficulty = catalog().worlds()[0].difficulties()[0];
    let pool = pool("universe.encounter-pool.room.100");

    let weighted = runtime
        .resolve(catalog(), pool, "3", difficulty)
        .expect("weighted condition");
    let EncounterSelection::WeightedGroups(groups) = weighted else {
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
        "1001"
    );

    let fixed = runtime
        .resolve(catalog(), pool, "10", difficulty)
        .expect("fixed condition");
    assert_eq!(
        fixed,
        EncounterSelection::FixedContent {
            source_content_id: "1006".into(),
        }
    );
    assert_eq!(
        runtime.resolve(catalog(), pool, "not-offered", difficulty),
        Err(EncounterContentRuntimeError::ConditionNotOffered)
    );
}
