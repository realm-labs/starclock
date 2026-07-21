use super::*;
use starclock_build::{
    ability::AbilityInvestment,
    compiler::LoadoutCompiler,
    id::LightConeId,
    light_cone::{LightConeLevel, Superimposition},
    patch::BuildPatch,
    spec::{CombatantBuildSpec, LightConeLoadout, PromotionStage},
};
use starclock_combat::UnitLevel;

const PRODUCTION_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");

#[test]
fn production_has_all_complete_curves_ranks_and_compilable_passives() {
    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let builds = catalog.build_catalog();
    let combat = catalog.combat_catalog();
    assert_eq!(builds.light_cone_ids().count(), 165);

    for raw in 112..=276 {
        let id = LightConeId::new(raw).unwrap();
        let cone = builds.light_cone(id).expect("released cone must lower");
        assert_eq!(cone.stats().len(), 86);
        assert!(
            cone.stat_row(
                LightConeLevel::new(1).unwrap(),
                PromotionStage::new(0).unwrap()
            )
            .is_some()
        );
        assert!(
            cone.stat_row(
                LightConeLevel::new(80).unwrap(),
                PromotionStage::new(6).unwrap()
            )
            .is_some()
        );
        assert_eq!(cone.passive_ranks().len(), 5);
        for rank in 1..=5 {
            let passive = cone.passive_rank(Superimposition::new(rank).unwrap());
            assert!(matches!(passive.patches()[0], BuildPatch::AddRuleBundle(_)));
            for patch in passive.patches() {
                match *patch {
                    BuildPatch::AddRuleBundle(rule) => assert!(combat.rule_bundle(rule).is_some()),
                    BuildPatch::AddModifier(modifier) => {
                        assert!(combat.modifier(modifier).is_some())
                    }
                    _ => panic!("Light Cone rank contains a non-passive build patch"),
                }
            }
        }

        let wearer = builds
            .character_ids()
            .find(|wearer| builds.character(*wearer).unwrap().path() == cone.path())
            .expect("every released Light Cone path has a released wearer");
        let character = builds.character(wearer).unwrap();
        let investments = character
            .ability_levels()
            .iter()
            .map(|table| AbilityInvestment::new(table.family(), table.invested_cap()))
            .collect::<Vec<_>>();
        for rank in [1, 5] {
            let spec = CombatantBuildSpec::new(
                wearer,
                UnitLevel::new(80).unwrap(),
                PromotionStage::new(6).unwrap(),
            )
            .with_ability_levels(investments.clone())
            .unwrap()
            .with_light_cone(LightConeLoadout::new(
                id,
                LightConeLevel::new(80).unwrap(),
                PromotionStage::new(6).unwrap(),
                Superimposition::new(rank).unwrap(),
            ));
            LoadoutCompiler
                .compile(builds, combat, &spec)
                .expect("matching-path S1/S5 fixture must compile");
        }
    }
}

#[test]
fn seeded_build_matrix_fuzzes_matching_and_mismatched_content_compilation() {
    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let builds = catalog.build_catalog();
    let combat = catalog.combat_catalog();
    let characters = builds.character_ids().collect::<Vec<_>>();
    let cones = builds.light_cone_ids().collect::<Vec<_>>();
    let mut state = 0x636f_6e74_656e_7431_u64;

    for _ in 0..1_024 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let character_id = characters[(state as usize) % characters.len()];
        state = state.rotate_left(29) ^ 0x9e37_79b9_7f4a_7c15;
        let cone_id = cones[(state as usize) % cones.len()];
        let rank = ((state >> 32) % 5 + 1) as u8;
        let character = builds.character(character_id).unwrap();
        let cone = builds.light_cone(cone_id).unwrap();
        let investments = character
            .ability_levels()
            .iter()
            .map(|table| AbilityInvestment::new(table.family(), table.invested_cap()))
            .collect::<Vec<_>>();
        let spec = CombatantBuildSpec::new(
            character_id,
            UnitLevel::new(80).unwrap(),
            PromotionStage::new(6).unwrap(),
        )
        .with_ability_levels(investments)
        .unwrap()
        .with_light_cone(LightConeLoadout::new(
            cone_id,
            LightConeLevel::new(80).unwrap(),
            PromotionStage::new(6).unwrap(),
            Superimposition::new(rank).unwrap(),
        ));
        let compiled = LoadoutCompiler
            .compile(builds, combat, &spec)
            .expect("valid selected content must compile even when its passive is inactive");
        let passive = cone.passive_rank(Superimposition::new(rank).unwrap());
        for patch in passive.patches() {
            let active = match *patch {
                BuildPatch::AddRuleBundle(rule) => {
                    compiled.combatant().rule_bundles().contains(&rule)
                }
                BuildPatch::AddModifier(modifier) => {
                    compiled.combatant().modifiers().contains(&modifier)
                }
                _ => panic!("Light Cone passive contains an unsupported patch"),
            };
            assert_eq!(
                active,
                character.path() == cone.path(),
                "path applicability changed for character {} and Light Cone {}",
                character_id.get(),
                cone_id.get(),
            );
        }
    }
}

#[test]
fn dream_scented_in_wheat_keeps_exact_boundary_stats_and_rank_modifiers() {
    let catalog = load(PRODUCTION_BUNDLE).unwrap();
    let cone = catalog
        .build_catalog()
        .light_cone(LightConeId::new(112).unwrap())
        .unwrap();
    let level_one = cone
        .stat_row(
            LightConeLevel::new(1).unwrap(),
            PromotionStage::new(0).unwrap(),
        )
        .unwrap();
    assert_eq!(level_one.maximum_hp().get(), 43);
    assert_eq!(level_one.attack().scaled(), 24_000_000);
    assert_eq!(level_one.defense().scaled(), 18_000_000);
    let level_eighty = cone
        .stat_row(
            LightConeLevel::new(80).unwrap(),
            PromotionStage::new(6).unwrap(),
        )
        .unwrap();
    assert_eq!(level_eighty.maximum_hp().get(), 953);
    assert_eq!(level_eighty.attack().scaled(), 529_200_000);
    assert_eq!(level_eighty.defense().scaled(), 396_900_000);
    let s1 = cone.passive_rank(Superimposition::new(1).unwrap());
    let s5 = cone.passive_rank(Superimposition::new(5).unwrap());
    assert_eq!(s1.patches().len(), 2);
    assert_eq!(s5.patches().len(), 2);
    assert_ne!(s1.patches()[1], s5.patches()[1]);
}
