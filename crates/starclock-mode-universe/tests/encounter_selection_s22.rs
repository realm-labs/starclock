use std::sync::{Arc, OnceLock};

use sha2::{Digest, Sha256};
use starclock_mode_universe::{
    catalog::UniverseCatalog, definition::DomainKind, encounter::EncounterSelectionPolicy,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");
const EXPECTED_KEYS: [&str; 28] = [
    "universe.encounter-pool.room.300232",
    "universe.encounter-pool.room.300233",
    "universe.encounter-pool.room.300241",
    "universe.encounter-pool.room.300242",
    "universe.encounter-pool.room.300243",
    "universe.encounter-pool.room.300611",
    "universe.encounter-pool.room.300612",
    "universe.encounter-pool.room.300713",
    "universe.encounter-pool.room.301",
    "universe.encounter-pool.room.302",
    "universe.encounter-pool.room.304",
    "universe.encounter-pool.room.307",
    "universe.encounter-pool.room.400111",
    "universe.encounter-pool.room.400112",
    "universe.encounter-pool.room.400121",
    "universe.encounter-pool.room.400122",
    "universe.encounter-pool.room.400131",
    "universe.encounter-pool.room.400132",
    "universe.encounter-pool.room.400142",
    "universe.encounter-pool.room.400211",
    "universe.encounter-pool.room.400212",
    "universe.encounter-pool.room.400221",
    "universe.encounter-pool.room.400222",
    "universe.encounter-pool.room.400231",
    "universe.encounter-pool.room.400232",
    "universe.encounter-pool.room.400611",
    "universe.encounter-pool.room.400612",
    "universe.encounter-pool.room.501",
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

fn text(hasher: &mut Sha256, value: &str) {
    hasher.update(u64::try_from(value.len()).unwrap().to_le_bytes());
    hasher.update(value.as_bytes());
}

#[test]
fn goal07_p5_m15_s22_materializes_all_frozen_encounter_pools_exactly() {
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
    assert!(selected.iter().map(|pool| pool.id().get()).eq(65_u32..=92));
    assert_eq!(
        selected
            .iter()
            .filter(|pool| {
                pool.selection_policy()
                    == EncounterSelectionPolicy::ExactConditionThenWeightedStableOrder
            })
            .count(),
        15
    );
    assert_eq!(
        selected
            .iter()
            .filter(|pool| {
                pool.selection_policy()
                    == EncounterSelectionPolicy::ConditionThenGroupOrDifficultyBinding
            })
            .count(),
        13
    );
    assert_eq!(
        selected
            .iter()
            .map(|pool| pool.weighted().len())
            .sum::<usize>(),
        53
    );
    assert_eq!(
        selected
            .iter()
            .map(|pool| pool.fixed().len())
            .sum::<usize>(),
        14
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
        6
    );

    let mut hasher = Sha256::new();
    text(
        &mut hasher,
        "starclock-goal07-p5-m15-s22-encounter-pools-v1",
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
            78, 0, 98, 28, 52, 233, 58, 253, 80, 141, 49, 209, 160, 108, 223, 80, 96, 247, 98, 114,
            247, 20, 144, 18, 54, 128, 50, 186, 185, 210, 103, 168,
        ]
    );
}

#[test]
fn goal07_p5_m15_s22_closes_all_ninety_two_encounter_pools() {
    let pools = catalog().encounter_pools();
    assert_eq!(pools.len(), 92);
    assert!(pools.iter().map(|pool| pool.id().get()).eq(1_u32..=92));
    assert_eq!(
        pools
            .iter()
            .filter(|pool| {
                pool.selection_policy()
                    == EncounterSelectionPolicy::ExactConditionThenWeightedStableOrder
            })
            .count(),
        59
    );
    assert_eq!(
        pools
            .iter()
            .filter(|pool| {
                pool.selection_policy() == EncounterSelectionPolicy::WorldDifficultyBossEliteBinding
            })
            .count(),
        4
    );
    assert_eq!(
        pools
            .iter()
            .filter(|pool| {
                pool.selection_policy()
                    == EncounterSelectionPolicy::ConditionThenGroupOrDifficultyBinding
            })
            .count(),
        29
    );
    assert_eq!(
        pools
            .iter()
            .map(|pool| pool.weighted().len())
            .sum::<usize>(),
        174
    );
    assert_eq!(
        pools.iter().map(|pool| pool.fixed().len()).sum::<usize>(),
        36
    );
}
