use std::sync::{Arc, OnceLock};

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    id::{BlessingId, ResonanceId},
    path_effect_runtime::{
        PathBattleEvent, PathEffect, PathEffectDamageKind, PathEffectElement, PathEffectFacts,
        PathEffectStat, PathEffectTarget, PathEffectValue,
    },
    remembrance_runtime::{REMEMBRANCE_RUNTIME_REVISION, RemembranceRuntimeCatalog},
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

fn runtime() -> RemembranceRuntimeCatalog {
    RemembranceRuntimeCatalog::compile(catalog()).expect("Remembrance runtime")
}

fn blessing(key: &str) -> BlessingId {
    catalog()
        .blessings()
        .iter()
        .find(|entry| entry.stable_key() == key)
        .expect("Blessing")
        .id()
}

fn resonance(key: &str) -> ResonanceId {
    catalog()
        .resonances()
        .iter()
        .find(|entry| entry.stable_key() == key)
        .expect("Resonance")
        .id()
}

fn value(integral: i64) -> PathEffectValue {
    PathEffectValue::from_integral(integral).expect("bounded fixture value")
}

fn facts() -> PathEffectFacts {
    PathEffectFacts {
        actor_maximum_hp: value(2_000),
        path_base_damage: value(1_000),
        damage_dealt: value(500),
        enemy_current_hp_ratio: PathEffectValue::from_raw_six_decimal(400_000),
        path_blessing_count: 9,
        enemy_attack_count: 6,
        enemy_is_frozen: true,
        enemy_is_dissociated: true,
        enemy_has_dissociation_vulnerability: true,
        enemy_crossed_hp_threshold_first_time: true,
        action_is_skill_or_ultimate: true,
        ..PathEffectFacts::default()
    }
}

#[test]
fn complete_partition_compiles_through_one_closed_registry() {
    let runtime = runtime();
    assert_eq!(
        REMEMBRANCE_RUNTIME_REVISION,
        "standard-universe-remembrance-runtime-v1"
    );
    assert_eq!(runtime.content_count(), 59);
    assert_eq!(runtime.rule_count(), 58);
    assert_eq!(runtime.blessing_ids().len(), 18);
    assert_eq!(runtime.resonance_ids().len(), 4);
    assert_eq!(
        runtime.digest(),
        [
            216, 72, 8, 207, 24, 86, 136, 61, 220, 185, 104, 126, 105, 135, 94, 160, 170, 103, 139,
            100, 205, 183, 135, 79, 219, 184, 115, 168, 208, 209, 113, 164,
        ]
    );
}

#[test]
fn every_blessing_level_and_resonance_has_an_executable_event() {
    let runtime = runtime();
    let events = [
        PathBattleEvent::BattleStarted,
        PathBattleEvent::AttackHit,
        PathBattleEvent::WeaknessBroken,
        PathBattleEvent::PathResonanceActivated,
        PathBattleEvent::StatQueried,
        PathBattleEvent::DamageCalculated,
        PathBattleEvent::DamageDealt,
        PathBattleEvent::IceDamageDealt,
        PathBattleEvent::EnemyFrozen,
        PathBattleEvent::DissociationRemoved,
        PathBattleEvent::UltimateUsed,
    ];
    for blessing in runtime.blessing_ids() {
        for level in [1, 2] {
            assert!(events.iter().any(|event| {
                runtime
                    .execute_blessing(blessing, level, *event, facts())
                    .is_ok_and(|effects| !effects.is_empty())
            }));
        }
    }
    for resonance in runtime.resonance_ids() {
        assert!(events.iter().any(|event| {
            runtime
                .execute_resonance(resonance, *event, facts())
                .is_ok_and(|effects| !effects.is_empty())
        }));
    }
}

#[test]
fn dissociation_freeze_and_conditional_formulas_are_exact() {
    let runtime = runtime();
    let dissociation = runtime
        .execute_blessing(
            blessing("universe.blessing.612130"),
            2,
            PathBattleEvent::AttackHit,
            facts(),
        )
        .expect("Dissociation");
    assert_eq!(
        dissociation[0].source_key(),
        "universe.blessing.612130.level.2"
    );
    assert!(matches!(
        dissociation[0].effect(),
        PathEffect::ApplyDissociation {
            base_chance,
            duration_turns: 1,
            removal_damage_bonus_ratio,
            ..
        } if base_chance.raw_six_decimal() == 1_000_000
            && removal_damage_bonus_ratio.raw_six_decimal() == 200_000
    ));

    let repeated = runtime
        .execute_blessing(
            blessing("universe.blessing.612132"),
            2,
            PathBattleEvent::AttackHit,
            facts(),
        )
        .expect("Repeated attack Freeze");
    assert!(matches!(
        repeated[0].effect(),
        PathEffect::ApplyFreeze {
            base_chance,
            duration_turns: 1,
            ..
        } if base_chance.raw_six_decimal() == 1_500_000
    ));

    let resistance = runtime
        .execute_blessing(
            blessing("universe.blessing.612150"),
            2,
            PathBattleEvent::BattleStarted,
            facts(),
        )
        .expect("Freeze resistance");
    assert!(matches!(
        resistance[0].effect(),
        PathEffect::AddStat {
            target: PathEffectTarget::AllEnemies,
            stat: PathEffectStat::FreezeResistanceReductionRatio,
            value,
            ..
        } if value.raw_six_decimal() == 720_000
    ));
}

#[test]
fn splash_weakness_energy_and_shield_are_typed() {
    let runtime = runtime();
    let splash = runtime
        .execute_blessing(
            blessing("universe.blessing.612143"),
            2,
            PathBattleEvent::IceDamageDealt,
            facts(),
        )
        .expect("Ice splash");
    assert!(matches!(
        splash[0].effect(),
        PathEffect::Damage {
            target: PathEffectTarget::OtherEnemies,
            amount,
            kind: PathEffectDamageKind::PathAdditional,
            element: PathEffectElement::Ice,
            ..
        } if amount.raw_six_decimal() == 120_000_000
    ));

    let weakness = runtime
        .execute_blessing(
            blessing("universe.blessing.612145"),
            2,
            PathBattleEvent::UltimateUsed,
            facts(),
        )
        .expect("Ice Weakness");
    assert!(matches!(
        weakness[0].effect(),
        PathEffect::ApplyIceWeakness {
            target: PathEffectTarget::RandomEnemyWithoutIceWeakness,
            duration_turns: 2,
            ..
        }
    ));

    let energy = runtime
        .execute_blessing(
            blessing("universe.blessing.612156"),
            2,
            PathBattleEvent::EnemyFrozen,
            facts(),
        )
        .expect("Energy");
    assert!(matches!(
        energy[0].effect(),
        PathEffect::GainEnergy {
            amount,
            once_per_action: true,
            ..
        } if amount.raw_six_decimal() == 12_000_000
    ));

    let shield = runtime
        .execute_blessing(
            blessing("universe.blessing.612157"),
            2,
            PathBattleEvent::EnemyFrozen,
            facts(),
        )
        .expect("Shield");
    assert!(matches!(
        shield[0].effect(),
        PathEffect::Shield {
            amount,
            duration_turns: 3,
            ..
        } if amount.raw_six_decimal() == 480_000_000
    ));
}

#[test]
fn resonance_and_all_three_formations_emit_typed_effects() {
    let runtime = runtime();
    let base = runtime
        .execute_resonance(
            resonance("universe.resonance.612120"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .expect("Resonance");
    assert_eq!(base.len(), 2);
    assert!(matches!(
        base[0].effect(),
        PathEffect::Damage {
            amount,
            kind: PathEffectDamageKind::PathResonance,
            element: PathEffectElement::Ice,
            ..
        } if amount.raw_six_decimal() == 600_000_000
    ));
    assert!(matches!(
        base[1].effect(),
        PathEffect::ApplyFreeze { base_chance, .. }
            if base_chance.raw_six_decimal() == 1_200_000
    ));

    let resistance = runtime
        .execute_resonance(
            resonance("universe.resonance.612121"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .expect("Resistance formation");
    assert!(matches!(
        resistance[0].effect(),
        PathEffect::ApplyFreezeResistanceReduction {
            base_chance,
            value,
            duration_turns: 1,
            ..
        } if base_chance.raw_six_decimal() == 1_500_000
            && value.raw_six_decimal() == 1_000_000
    ));

    let river = runtime
        .execute_resonance(
            resonance("universe.resonance.612122"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .expect("Eonian River formation");
    assert!(matches!(
        river[0].effect(),
        PathEffect::ApplyEonianRiver {
            base_chance,
            duration_turns: 1,
            ..
        } if base_chance.raw_six_decimal() == 1_500_000
    ));

    let energy = runtime
        .execute_resonance(
            resonance("universe.resonance.612123"),
            PathBattleEvent::BattleStarted,
            facts(),
        )
        .expect("Energy formation");
    assert!(matches!(
        energy[0].effect(),
        PathEffect::GainResonanceEnergy { maximum_ratio }
            if maximum_ratio.raw_six_decimal() == 400_000
    ));
}

#[test]
fn five_frozen_semantic_fixtures_are_runtime_backed() {
    let runtime = runtime();
    for (key, tag) in [
        ("universe.blessing.612150", "effect-res"),
        ("universe.blessing.612130", "freeze"),
        ("universe.blessing.612152", "hp-loss"),
        ("universe.blessing.612145", "ultimate"),
    ] {
        let id = blessing(key);
        assert!(runtime.blessing_ids().any(|candidate| candidate == id));
        assert!(
            catalog()
                .blessing(id)
                .expect("Blessing")
                .mechanic_tags()
                .iter()
                .any(|candidate| candidate.as_ref() == tag)
        );
    }
    assert_eq!(runtime.blessing_ids().len(), 18);
}
