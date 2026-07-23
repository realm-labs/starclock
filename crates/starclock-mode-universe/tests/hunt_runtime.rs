use std::sync::{Arc, OnceLock};

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    hunt_runtime::{HUNT_RUNTIME_REVISION, HuntRuntimeCatalog},
    id::{BlessingId, ResonanceId},
    path_effect_runtime::{
        PathBattleEvent, PathEffect, PathEffectFacts, PathEffectStat, PathEffectTarget,
        PathEffectValue,
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
fn runtime() -> HuntRuntimeCatalog {
    HuntRuntimeCatalog::compile(catalog()).expect("Hunt")
}
fn blessing(key: &str) -> BlessingId {
    catalog()
        .blessings()
        .iter()
        .find(|v| v.stable_key() == key)
        .unwrap()
        .id()
}
fn resonance(key: &str) -> ResonanceId {
    catalog()
        .resonances()
        .iter()
        .find(|v| v.stable_key() == key)
        .unwrap()
        .id()
}
fn value(v: i64) -> PathEffectValue {
    PathEffectValue::from_integral(v).unwrap()
}
fn facts() -> PathEffectFacts {
    PathEffectFacts {
        actor_maximum_hp: value(1_000),
        highest_ally_attack: value(1_000),
        last_acting_ally_attack: value(500),
        enemy_current_hp_ratio: PathEffectValue::from_raw_six_decimal(400_000),
        path_blessing_count: 9,
        critical_boost_stacks: 4,
        weakness_broken_enemy_is_elite: true,
        ..PathEffectFacts::default()
    }
}

#[test]
fn complete_partition_compiles_through_one_closed_registry() {
    let runtime = runtime();
    assert_eq!(HUNT_RUNTIME_REVISION, "standard-universe-hunt-runtime-v1");
    assert_eq!((runtime.content_count(), runtime.rule_count()), (59, 58));
    assert_eq!(runtime.blessing_ids().len(), 18);
    assert_eq!(runtime.resonance_ids().len(), 4);
    assert_eq!(
        runtime.digest(),
        [
            43, 53, 103, 169, 116, 55, 82, 241, 46, 75, 244, 99, 129, 251, 145, 112, 9, 68, 191,
            98, 254, 141, 6, 122, 154, 219, 190, 109, 108, 130, 161, 61,
        ]
    );
}

#[test]
fn every_level_and_resonance_has_an_executable_event() {
    let runtime = runtime();
    let events = [
        PathBattleEvent::BattleStarted,
        PathBattleEvent::TurnStarted,
        PathBattleEvent::TurnEnded,
        PathBattleEvent::WeaknessBroken,
        PathBattleEvent::EnemyDefeated,
        PathBattleEvent::UltimateUsed,
        PathBattleEvent::FollowUpAttackUsed,
        PathBattleEvent::ConsecutiveActionStarted,
        PathBattleEvent::StatQueried,
        PathBattleEvent::PathResonanceActivated,
    ];
    for id in runtime.blessing_ids() {
        for level in [1, 2] {
            assert!(events.iter().any(|event| {
                runtime
                    .execute_blessing(id, level, *event, facts())
                    .is_ok_and(|v| !v.is_empty())
            }));
        }
    }
    for id in runtime.resonance_ids() {
        assert!(events.iter().any(|event| {
            runtime
                .execute_resonance(id, *event, facts())
                .is_ok_and(|v| !v.is_empty())
        }));
    }
}

#[test]
fn critical_boost_advance_and_static_stats_are_exact() {
    let runtime = runtime();
    let boost = runtime
        .execute_blessing(
            blessing("universe.blessing.612430"),
            2,
            PathBattleEvent::TurnStarted,
            facts(),
        )
        .unwrap();
    assert!(
        matches!(boost[0].effect(), PathEffect::ApplyCriticalBoost { stacks: 1, maximum_stacks: 12, critical_rate_ratio_per_stack, critical_damage_ratio_per_stack, .. }
        if critical_rate_ratio_per_stack.raw_six_decimal() == 60_000 && critical_damage_ratio_per_stack.raw_six_decimal() == 120_000)
    );
    let advance = runtime
        .execute_blessing(
            blessing("universe.blessing.612432"),
            2,
            PathBattleEvent::WeaknessBroken,
            facts(),
        )
        .unwrap();
    assert_eq!(advance.len(), 3);
    assert!(
        matches!(advance[2].effect(), PathEffect::ActionAdvance { target: PathEffectTarget::AllAllies, ratio, .. } if ratio.raw_six_decimal() == 1_000_000)
    );
    let speed = runtime
        .execute_blessing(
            blessing("universe.blessing.612450"),
            2,
            PathBattleEvent::StatQueried,
            facts(),
        )
        .unwrap();
    assert!(
        matches!(speed[0].effect(), PathEffect::AddStat { stat: PathEffectStat::SpeedRatio, value, .. } if value.raw_six_decimal() == 360_000)
    );
}

#[test]
fn healing_energy_and_resonance_formations_are_typed() {
    let runtime = runtime();
    let healing = runtime
        .execute_blessing(
            blessing("universe.blessing.612441"),
            2,
            PathBattleEvent::UltimateUsed,
            facts(),
        )
        .unwrap();
    assert!(
        matches!(healing[0].effect(), PathEffect::HealMaximumHpRatio { ratio, .. } if ratio.raw_six_decimal() == 200_000)
    );
    let base = runtime
        .execute_resonance(
            resonance("universe.resonance.612420"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .unwrap();
    assert!(
        matches!(base[0].effect(), PathEffect::Damage { amount, .. } if amount.raw_six_decimal() == 5_500_000_000)
    );
    let arrow = runtime
        .execute_resonance(
            resonance("universe.resonance.612421"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .unwrap();
    assert!(
        matches!(arrow[1].effect(), PathEffect::ApplyLightHuntingCelestialArrow { critical_damage_from_critical_rate_ratio, .. } if critical_damage_from_critical_rate_ratio.raw_six_decimal() == 500_000)
    );
    let energy = runtime
        .execute_resonance(
            resonance("universe.resonance.612423"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .unwrap();
    assert!(
        matches!(energy[0].effect(), PathEffect::ConfigureResonanceEnergy { maximum, gain_on_ally_turn_ratio } if maximum.raw_six_decimal() == 200_000_000 && gain_on_ally_turn_ratio.raw_six_decimal() == 30_000)
    );
}

#[test]
fn two_frozen_semantic_fixtures_are_runtime_backed() {
    let id = blessing("universe.blessing.612432");
    assert!(runtime().blessing_ids().any(|candidate| candidate == id));
    assert_eq!(runtime().blessing_ids().len(), 18);
}
