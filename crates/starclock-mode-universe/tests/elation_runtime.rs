use std::sync::{Arc, OnceLock};

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    elation_runtime::{ELATION_RUNTIME_REVISION, ElationRuntimeCatalog},
    id::{BlessingId, ResonanceId},
    path_effect_runtime::{
        PathBattleEvent, PathEffect, PathEffectFacts, PathEffectStat, PathEffectValue,
    },
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
fn runtime() -> ElationRuntimeCatalog {
    ElationRuntimeCatalog::compile(catalog()).expect("Elation")
}
fn blessing(key: &str) -> BlessingId {
    catalog()
        .blessings()
        .iter()
        .find(|value| value.stable_key() == key)
        .unwrap()
        .id()
}
fn resonance(key: &str) -> ResonanceId {
    catalog()
        .resonances()
        .iter()
        .find(|value| value.stable_key() == key)
        .unwrap()
        .id()
}
fn value(value: i64) -> PathEffectValue {
    PathEffectValue::from_integral(value).unwrap()
}
fn facts() -> PathEffectFacts {
    PathEffectFacts {
        actor_base_attack: value(1_000),
        path_base_damage: value(600),
        aftertaste_damage: value(500),
        path_blessing_count: 9,
        aftertaste_element_count: 3,
        follow_up_targets_hit: 2,
        enemy_is_weakness_broken: true,
        ..PathEffectFacts::default()
    }
}

#[test]
fn complete_partition_compiles_through_one_closed_registry() {
    let runtime = runtime();
    assert_eq!(
        ELATION_RUNTIME_REVISION,
        "standard-universe-elation-runtime-v1"
    );
    assert_eq!((runtime.content_count(), runtime.rule_count()), (59, 58));
    assert_eq!(runtime.blessing_ids().len(), 18);
    assert_eq!(runtime.resonance_ids().len(), 4);
    assert_eq!(
        runtime.digest(),
        [
            166, 155, 183, 2, 108, 42, 58, 107, 17, 76, 91, 125, 225, 44, 250, 174, 15, 75, 92,
            251, 43, 60, 247, 168, 55, 142, 49, 151, 67, 44, 143, 71,
        ]
    );
}

#[test]
fn every_level_and_resonance_has_an_executable_event() {
    let runtime = runtime();
    let events = [
        PathBattleEvent::BattleStarted,
        PathBattleEvent::UltimateUsed,
        PathBattleEvent::FollowUpAttackUsed,
        PathBattleEvent::FollowUpDamageDealt,
        PathBattleEvent::AftertasteDamageDealt,
        PathBattleEvent::StatQueried,
        PathBattleEvent::PathResonanceActivated,
    ];
    for id in runtime.blessing_ids() {
        for level in [1, 2] {
            assert!(events.iter().any(|event| {
                runtime
                    .execute_blessing(id, level, *event, facts())
                    .is_ok_and(|effects| !effects.is_empty())
            }));
        }
    }
    for id in runtime.resonance_ids() {
        assert!(events.iter().any(|event| {
            runtime
                .execute_resonance(id, *event, facts())
                .is_ok_and(|effects| !effects.is_empty())
        }));
    }
}

#[test]
fn aftertaste_generation_and_broken_enemy_bonus_are_exact() {
    let runtime = runtime();
    let random = runtime
        .execute_blessing(
            blessing("universe.blessing.612630"),
            2,
            PathBattleEvent::FollowUpAttackUsed,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        random[0].effect(),
        PathEffect::DealAftertaste { minimum_hits: 1, maximum_hits: 3, amount_per_hit, damage_bonus_ratio, random_element_each_hit: true, .. }
            if amount_per_hit.raw_six_decimal() == 600_000_000
                && damage_bonus_ratio.raw_six_decimal() == 350_000
    ));
    let broken = runtime
        .execute_blessing(
            blessing("universe.blessing.612631"),
            2,
            PathBattleEvent::FollowUpAttackUsed,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        broken[0].effect(),
        PathEffect::DealAftertaste { minimum_hits: 1, maximum_hits: 3, amount_per_hit, .. }
            if amount_per_hit.raw_six_decimal() == 800_000_000
    ));
}

#[test]
fn aftertaste_types_and_follow_up_target_count_are_preserved() {
    let runtime = runtime();
    let vulnerability = runtime
        .execute_blessing(
            blessing("universe.blessing.612641"),
            2,
            PathBattleEvent::AftertasteDamageDealt,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        vulnerability[0].effect(),
        PathEffect::ApplyAftertasteTypeModifier { stat: PathEffectStat::DamageTakenRatio, value_per_element, element_count: 3, until_end_of_next_action: true, .. }
            if value_per_element.raw_six_decimal() == 120_000
    ));
    let target_damage = runtime
        .execute_blessing(
            blessing("universe.blessing.612643"),
            2,
            PathBattleEvent::FollowUpAttackUsed,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        target_damage[0].effect(),
        PathEffect::Damage { amount, .. } if amount.raw_six_decimal() == 720_000_000
    ));
    let stack = runtime
        .execute_blessing(
            blessing("universe.blessing.612650"),
            2,
            PathBattleEvent::StatQueried,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        stack[0].effect(),
        PathEffect::AddFollowUpModifier { stat: PathEffectStat::DamageRatio, value }
            if value.raw_six_decimal() == 1_080_000
    ));
}

#[test]
fn resonance_and_three_formations_are_typed() {
    let runtime = runtime();
    let damage = runtime
        .execute_resonance(
            resonance("universe.resonance.612620"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        damage[0].effect(),
        PathEffect::RandomElementFollowUpDamage { amount_per_hit, minimum_hits: 3, maximum_hits: 5, .. }
            if amount_per_hit.raw_six_decimal() == 600_000_000
    ));
    let sensory = runtime
        .execute_resonance(
            resonance("universe.resonance.612621"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        sensory[0].effect(),
        PathEffect::ApplySensoryPursuit { base_chance, duration_turns: 1, follow_up_damage_taken_ratio, .. }
            if base_chance.raw_six_decimal() == 1_500_000
                && follow_up_damage_taken_ratio.raw_six_decimal() == 250_000
    ));
    let variable = runtime
        .execute_resonance(
            resonance("universe.resonance.612622"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        variable[0].effect(),
        PathEffect::ConfigureVariableResonanceEnergy { maximum, consume_all_energy: true, excess_energy_ratio_per_extra_hit }
            if maximum.raw_six_decimal() == 200_000_000
                && excess_energy_ratio_per_extra_hit.raw_six_decimal() == 200_000
    ));
    let energy = runtime
        .execute_resonance(
            resonance("universe.resonance.612623"),
            PathBattleEvent::BattleStarted,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        energy[0].effect(),
        PathEffect::ConfigureResonanceEnergyGain { battle_start_maximum_ratio, follow_up_attack_maximum_ratio }
            if battle_start_maximum_ratio.raw_six_decimal() == 400_000
                && follow_up_attack_maximum_ratio.raw_six_decimal() == 50_000
    ));
}

#[test]
fn two_frozen_elation_fixtures_are_runtime_backed() {
    assert_eq!(runtime().blessing_ids().len(), 18);
    assert_eq!(runtime().resonance_ids().len(), 4);
}
