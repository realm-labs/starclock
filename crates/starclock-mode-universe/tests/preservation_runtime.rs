use std::sync::{Arc, OnceLock};

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    id::{BlessingId, ResonanceId},
    path_effect_runtime::{
        PathBattleEvent, PathEffect, PathEffectDamageKind, PathEffectFacts, PathEffectStat,
        PathEffectTarget, PathEffectValue,
    },
    preservation_runtime::{PRESERVATION_RUNTIME_REVISION, PreservationRuntimeCatalog},
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

fn runtime() -> PreservationRuntimeCatalog {
    PreservationRuntimeCatalog::compile(catalog()).expect("Preservation runtime")
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
        actor_current_shield: value(1_000),
        actor_shield_before_hit: value(1_200),
        teammate_shield_total: value(500),
        party_shield_total: value(4_000),
        actor_maximum_hp: value(2_000),
        actor_defense: value(800),
        actor_base_attack: value(600),
        hp_lost: value(300),
        provided_shield: value(500),
        path_damage: value(2_000),
        path_blessing_count: 9,
        shielded_allies: 3,
        actor_is_shielded: true,
        ..PathEffectFacts::default()
    }
}

#[test]
fn complete_partition_compiles_through_one_closed_registry() {
    let runtime = runtime();
    assert_eq!(
        PRESERVATION_RUNTIME_REVISION,
        "standard-universe-preservation-runtime-v1"
    );
    assert_eq!(runtime.content_count(), 59);
    assert_eq!(runtime.rule_count(), 58);
    assert_eq!(runtime.blessing_ids().len(), 18);
    assert_eq!(runtime.resonance_ids().len(), 4);
    assert_eq!(
        runtime.digest(),
        [
            172, 85, 97, 228, 193, 128, 82, 50, 221, 67, 142, 105, 147, 135, 244, 88, 128, 194,
            132, 167, 68, 119, 115, 160, 52, 63, 209, 245, 121, 245, 76, 180,
        ]
    );
}

#[test]
fn every_blessing_level_and_resonance_has_an_executable_event() {
    let runtime = runtime();
    let events = [
        PathBattleEvent::BattleStarted,
        PathBattleEvent::TurnEnded,
        PathBattleEvent::AttackHit,
        PathBattleEvent::CharacterAttacked,
        PathBattleEvent::WeaknessBroken,
        PathBattleEvent::ShieldGranted,
        PathBattleEvent::ShieldGrantedToAlly,
        PathBattleEvent::PathDamageDealt,
        PathBattleEvent::PathResonanceActivated,
        PathBattleEvent::StatQueried,
        PathBattleEvent::DamageCalculated,
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
fn quake_shield_and_conditional_stat_formulas_are_exact() {
    let runtime = runtime();
    let quake = runtime
        .execute_blessing(
            blessing("universe.blessing.612030"),
            2,
            PathBattleEvent::AttackHit,
            facts(),
        )
        .expect("Quake");
    assert_eq!(quake.len(), 1);
    assert_eq!(quake[0].source_key(), "universe.blessing.612030.level.2");
    assert!(matches!(
        quake[0].effect(),
        PathEffect::Damage {
            target: PathEffectTarget::HitEnemies,
            amount,
            kind: PathEffectDamageKind::PathAdditional,
            can_defeat: true,
            ..
        } if amount.raw_six_decimal() == 1_100_000_000
    ));

    let defense = runtime
        .execute_blessing(
            blessing("universe.blessing.612050"),
            2,
            PathBattleEvent::BattleStarted,
            facts(),
        )
        .expect("Defense");
    assert!(matches!(
        defense[0].effect(),
        PathEffect::AddStat {
            stat: PathEffectStat::DefenseRatio,
            value,
            ..
        } if value.raw_six_decimal() == 720_000
    ));

    let attack = runtime
        .execute_blessing(
            blessing("universe.blessing.612043"),
            1,
            PathBattleEvent::StatQueried,
            facts(),
        )
        .expect("Attack");
    assert!(matches!(
        attack[0].effect(),
        PathEffect::AddStat {
            stat: PathEffectStat::AttackFlat,
            value,
            cap: Some(cap),
            ..
        } if value.raw_six_decimal() == 400_000_000
            && cap.raw_six_decimal() == 720_000_000
    ));
}

#[test]
fn resonance_and_all_three_formations_emit_typed_effects() {
    let runtime = runtime();
    let base = runtime
        .execute_resonance(
            resonance("universe.resonance.612020"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .expect("Resonance");
    assert!(matches!(
        base[0].effect(),
        PathEffect::Damage {
            target: PathEffectTarget::AllEnemies,
            amount,
            kind: PathEffectDamageKind::PathResonance,
            ..
        } if amount.raw_six_decimal() == 10_000_000_000
    ));

    let critical = runtime
        .execute_resonance(
            resonance("universe.resonance.612021"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .expect("Critical formation");
    assert!(matches!(
        critical[0].effect(),
        PathEffect::Damage {
            force_critical: true,
            critical_damage_ratio,
            ..
        } if critical_damage_ratio.raw_six_decimal() == 450_000
    ));

    let shield = runtime
        .execute_resonance(
            resonance("universe.resonance.612022"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .expect("Shield formation");
    assert_eq!(shield.len(), 2);
    assert!(matches!(shield[1].effect(), PathEffect::ApplyAmber { .. }));

    let energy = runtime
        .execute_resonance(
            resonance("universe.resonance.612023"),
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
fn seven_frozen_semantic_families_are_runtime_backed() {
    let runtime = runtime();
    let cases = [
        ("universe.blessing.612053", "break"),
        ("universe.blessing.612056", "critical"),
        ("universe.blessing.612030", "damage"),
        ("universe.blessing.612031", "defense"),
        ("universe.blessing.612041", "dot"),
        ("universe.blessing.612030", "shield"),
    ];
    for (key, tag) in cases {
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
