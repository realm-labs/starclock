use std::sync::{Arc, OnceLock};

use sha2::{Digest, Sha256};
use starclock_mode_universe::{
    battle_materialization::catalog_composition::UniverseBattleCatalogComposition,
    catalog::UniverseCatalog, definition::DifficultyKind, encounter::EnemyRole,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");
const WORLD_DIFFICULTY_COUNTS: [usize; 9] = [2, 1, 5, 5, 4, 4, 4, 4, 4];

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
fn goal07_p5_m15_s36_materializes_exact_world_difficulty_and_enemy_bindings() {
    assert_eq!(catalog().worlds().len(), 9);
    assert_eq!(catalog().difficulties().len(), 33);
    assert_eq!(catalog().difficulty_enemy_bindings().len(), 182);

    let mut hasher = Sha256::new();
    text(
        &mut hasher,
        "starclock-goal07-p5-m15-s36-world-difficulty-v1",
    );
    for world in catalog().worlds() {
        hasher.update(world.id().get().to_le_bytes());
        hasher.update(world.profile().get().to_le_bytes());
        text(&mut hasher, world.stable_key());
        hasher.update([world.number()]);
        text(&mut hasher, world.text().name_en());
        text(&mut hasher, world.text().name_zh_cn());
        text(&mut hasher, world.text().summary_en());
        text(&mut hasher, world.text().summary_zh_cn());
        text(&mut hasher, world.entry_rule_key());
        text(&mut hasher, world.terminal_rule_key());
        hasher.update(
            u64::try_from(world.difficulties().len())
                .unwrap()
                .to_le_bytes(),
        );
        for difficulty in world.difficulties() {
            hasher.update(difficulty.get().to_le_bytes());
        }
    }
    for difficulty in catalog().difficulties() {
        hasher.update(difficulty.id().get().to_le_bytes());
        text(&mut hasher, difficulty.stable_key());
        hasher.update(difficulty.world().get().to_le_bytes());
        text(&mut hasher, difficulty.source_area_id());
        hasher.update([difficulty.ordinal(), difficulty.kind() as u8]);
        hasher.update([difficulty.recommended_level()]);
        hasher.update(
            u64::try_from(difficulty.recommended_elements().len())
                .unwrap()
                .to_le_bytes(),
        );
        for element in difficulty.recommended_elements() {
            hasher.update([*element as u8]);
        }
        hasher.update(
            u64::try_from(difficulty.score_curve().len())
                .unwrap()
                .to_le_bytes(),
        );
        for threshold in difficulty.score_curve() {
            hasher.update([threshold.tier()]);
            hasher.update(threshold.score().to_le_bytes());
        }
        match difficulty.unlock_source_id() {
            Some(value) => {
                hasher.update([1]);
                text(&mut hasher, value);
            }
            None => hasher.update([0]),
        }
    }
    for binding in catalog().difficulty_enemy_bindings() {
        hasher.update(binding.difficulty().get().to_le_bytes());
        hasher.update([binding.role() as u8]);
        text(&mut hasher, binding.source_monster_id());
        text(&mut hasher, binding.enemy_variant_key());
        hasher.update(binding.level().to_le_bytes());
    }

    let digest: [u8; 32] = hasher.finalize().into();
    assert_eq!(
        digest,
        [
            212, 19, 1, 10, 48, 30, 139, 186, 97, 15, 72, 249, 119, 124, 153, 253, 197, 184, 237,
            45, 123, 211, 170, 102, 215, 19, 57, 69, 254, 240, 123, 70,
        ]
    );
}

#[test]
fn goal07_p5_m15_s36_closes_world_difficulty_runtime_contracts() {
    assert!(
        catalog()
            .worlds()
            .iter()
            .map(|world| world.number())
            .eq(1_u8..=9)
    );
    assert_eq!(
        catalog()
            .difficulties()
            .iter()
            .filter(|difficulty| difficulty.kind() == DifficultyKind::Tutorial)
            .count(),
        1
    );
    assert_eq!(
        catalog()
            .difficulties()
            .iter()
            .filter(|difficulty| difficulty.kind() == DifficultyKind::Standard)
            .count(),
        32
    );
    assert_eq!(
        catalog()
            .difficulty_enemy_bindings()
            .iter()
            .filter(|binding| binding.role() == EnemyRole::Boss)
            .count(),
        35
    );
    assert_eq!(
        catalog()
            .difficulty_enemy_bindings()
            .iter()
            .filter(|binding| binding.role() == EnemyRole::Elite)
            .count(),
        147
    );

    for (world, expected_count) in catalog().worlds().iter().zip(WORLD_DIFFICULTY_COUNTS) {
        assert_eq!(world.difficulties().len(), expected_count);
        assert!(world.difficulties().iter().all(|id| {
            catalog()
                .difficulty(*id)
                .is_some_and(|difficulty| difficulty.world() == world.id())
        }));
    }
    for difficulty in catalog().difficulties() {
        assert!(
            catalog()
                .world(difficulty.world())
                .is_some_and(|world| world.difficulties().contains(&difficulty.id()))
        );
        assert!(!difficulty.recommended_elements().is_empty());
        assert!(!difficulty.score_curve().is_empty());
        assert!(
            difficulty
                .score_curve()
                .windows(2)
                .all(|pair| pair[0].tier() < pair[1].tier())
        );
        assert!(
            difficulty
                .score_curve()
                .iter()
                .all(|threshold| threshold.score() > 0)
        );
        let bindings = catalog()
            .difficulty_enemy_bindings()
            .iter()
            .filter(|binding| binding.difficulty() == difficulty.id())
            .collect::<Vec<_>>();
        assert!(!bindings.is_empty());
        assert!(
            bindings
                .iter()
                .any(|binding| binding.role() == EnemyRole::Boss)
        );
        assert!(bindings.iter().all(|binding| {
            !binding.source_monster_id().is_empty()
                && !binding.enemy_variant_key().is_empty()
                && binding.level() > 0
        }));
    }

    UniverseBattleCatalogComposition::compile(catalog())
        .expect("all 182 difficulty enemy bindings resolve into the battle catalog");
}
