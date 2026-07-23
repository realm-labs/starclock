use std::sync::{Arc, OnceLock};

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    id::{BlessingId, ResonanceId},
    path_effect_runtime::{
        PathBattleEvent, PathEffect, PathEffectFacts, PathEffectStat, PathEffectValue,
    },
    propagation_runtime::{PROPAGATION_RUNTIME_REVISION, PropagationRuntimeCatalog},
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
fn runtime() -> PropagationRuntimeCatalog {
    PropagationRuntimeCatalog::compile(catalog()).expect("Propagation")
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
fn facts() -> PathEffectFacts {
    PathEffectFacts {
        path_blessing_count: 9,
        skill_points_consumed: 2,
        skill_points_recovered: 1,
        all_enemy_spore_count: 8,
        spores_burst: 3,
        ..PathEffectFacts::default()
    }
}

#[test]
fn complete_partition_compiles_through_one_closed_registry() {
    let runtime = runtime();
    assert_eq!(
        PROPAGATION_RUNTIME_REVISION,
        "standard-universe-propagation-runtime-v1"
    );
    assert_eq!((runtime.content_count(), runtime.rule_count()), (59, 58));
    assert_eq!(runtime.blessing_ids().len(), 18);
    assert_eq!(runtime.resonance_ids().len(), 4);
    assert_eq!(
        runtime.digest(),
        [
            69, 203, 33, 65, 21, 255, 219, 202, 68, 82, 156, 241, 20, 15, 125, 52, 202, 157, 163,
            43, 72, 157, 215, 211, 234, 24, 206, 179, 181, 157, 109, 26,
        ]
    );
}

#[test]
fn every_level_and_resonance_has_an_executable_event() {
    let runtime = runtime();
    let events = [
        PathBattleEvent::BattleStarted,
        PathBattleEvent::SkillPointConsumed,
        PathBattleEvent::SkillPointRecovered,
        PathBattleEvent::UltimateUsed,
        PathBattleEvent::SporeBurst,
        PathBattleEvent::BasicAttackUsed,
        PathBattleEvent::BasicAttackDamageDealt,
        PathBattleEvent::NonAttackSkillUsed,
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
fn spores_preserve_stack_targets_caps_and_burst_scaling() {
    let runtime = runtime();
    let recovered = runtime
        .execute_blessing(
            blessing("universe.blessing.612731"),
            2,
            PathBattleEvent::SkillPointRecovered,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        recovered[0].effect(),
        PathEffect::ApplySpores {
            random_target_count: 2,
            maximum_stacks: Some(9),
            ..
        }
    ));
    let sustain = runtime
        .execute_blessing(
            blessing("universe.blessing.612742"),
            2,
            PathBattleEvent::SporeBurst,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        sustain[0].effect(),
        PathEffect::HealPerSporeBurst { maximum_hp_ratio_per_spore, spore_count: 3, .. }
            if maximum_hp_ratio_per_spore.raw_six_decimal() == 120_000
    ));
    assert!(matches!(
        sustain[1].effect(),
        PathEffect::AddPartyDamageReductionPerSpore { value_per_spore, spore_count: 8 }
            if value_per_spore.raw_six_decimal() == 8_000
    ));
}

#[test]
fn skill_point_and_basic_attack_mechanics_are_typed() {
    let runtime = runtime();
    let energy = runtime
        .execute_blessing(
            blessing("universe.blessing.612756"),
            2,
            PathBattleEvent::SkillPointConsumed,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        energy[0].effect(),
        PathEffect::GainEnergy { amount, once_per_action: false, .. }
            if amount.raw_six_decimal() == 8_000_000
    ));
    let basic = runtime
        .execute_blessing(
            blessing("universe.blessing.612750"),
            2,
            PathBattleEvent::StatQueried,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        basic[0].effect(),
        PathEffect::AddBasicAttackModifier { stat: PathEffectStat::DamageRatio, value }
            if value.raw_six_decimal() == 1_080_000
    ));
}

#[test]
fn resonance_and_three_formations_are_typed() {
    let runtime = runtime();
    let metamorphosis = runtime
        .execute_resonance(
            resonance("universe.resonance.612720"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        metamorphosis[0].effect(),
        PathEffect::ApplyMetamorphosis {
            action_advance_ratio: PathEffectValue::ONE,
            skill_points: 2,
            duration_turns: 1,
            ..
        }
    ));
    let phenol = runtime
        .execute_resonance(
            resonance("universe.resonance.612722"),
            PathBattleEvent::BattleStarted,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        phenol[0].effect(),
        PathEffect::ConfigureSkillPointResonanceEnergy { maximum, energy_ratio_per_consumed_or_recovered_point }
            if maximum.raw_six_decimal() == 200_000_000
                && energy_ratio_per_consumed_or_recovered_point.raw_six_decimal() == 10_000
    ));
    let crystal = runtime
        .execute_resonance(
            resonance("universe.resonance.612723"),
            PathBattleEvent::BattleStarted,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        crystal[0].effect(),
        PathEffect::ConfigureMetamorphosisSporeBurst { damage_ratio, basic_attack_ratio_per_spore, maximum_triggers_per_target: 3 }
            if damage_ratio.raw_six_decimal() == 400_000
                && basic_attack_ratio_per_spore.raw_six_decimal() == 800_000
    ));
}

#[test]
fn three_frozen_propagation_fixtures_are_runtime_backed() {
    assert_eq!(runtime().blessing_ids().len(), 18);
    assert_eq!(runtime().resonance_ids().len(), 4);
}
