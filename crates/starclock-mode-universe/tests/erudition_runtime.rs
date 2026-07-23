use std::sync::{Arc, OnceLock};

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    erudition_runtime::{ERUDITION_RUNTIME_REVISION, EruditionRuntimeCatalog},
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
fn runtime() -> EruditionRuntimeCatalog {
    EruditionRuntimeCatalog::compile(catalog()).expect("Erudition")
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
        actor_maximum_hp: value(1_000),
        actor_maximum_energy: value(120),
        excess_energy: value(10),
        path_blessing_count: 9,
        ultimate_targets_hit: 3,
        maximum_ultimate_targets_hit: 4,
        attacked_enemy_count: 1,
        defeated_enemy_count: 3,
        enemy_is_weakness_broken: true,
        ..PathEffectFacts::default()
    }
}

#[test]
fn complete_partition_compiles_through_one_closed_registry() {
    let runtime = runtime();
    assert_eq!(
        ERUDITION_RUNTIME_REVISION,
        "standard-universe-erudition-runtime-v1"
    );
    assert_eq!((runtime.content_count(), runtime.rule_count()), (59, 58));
    assert_eq!(runtime.blessing_ids().len(), 18);
    assert_eq!(runtime.resonance_ids().len(), 4);
    assert_eq!(
        runtime.digest(),
        [
            190, 199, 55, 169, 80, 220, 63, 248, 160, 216, 119, 183, 169, 244, 212, 165, 247, 88,
            209, 114, 114, 146, 174, 251, 117, 134, 71, 236, 51, 19, 2, 219,
        ]
    );
}

#[test]
fn every_level_and_resonance_has_an_executable_event() {
    let runtime = runtime();
    let events = [
        PathBattleEvent::BattleStarted,
        PathBattleEvent::WeaknessBroken,
        PathBattleEvent::AttackHit,
        PathBattleEvent::EnemyDefeated,
        PathBattleEvent::StatQueried,
        PathBattleEvent::UltimateViaBrainInVatUsed,
        PathBattleEvent::EnergyOverflowed,
        PathBattleEvent::AttackCompleted,
        PathBattleEvent::AoeAttackUsed,
        PathBattleEvent::UltimateUsed,
        PathBattleEvent::LethalDamageReceived,
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
fn brain_charge_overflow_and_vat_ultimate_effects_are_exact() {
    let runtime = runtime();
    let entry = runtime
        .execute_blessing(
            blessing("universe.blessing.612830"),
            2,
            PathBattleEvent::BattleStarted,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        entry[0].effect(),
        PathEffect::ChargeBrainInVat { ratio, once_per_enemy_per_attack: false }
            if ratio.raw_six_decimal() == 1_000_000
    ));
    let overflow = runtime
        .execute_blessing(
            blessing("universe.blessing.612841"),
            2,
            PathBattleEvent::EnergyOverflowed,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        overflow[0].effect(),
        PathEffect::ChargeBrainInVat { ratio, .. } if ratio.raw_six_decimal() == 120_000
    ));
    let shield = runtime
        .execute_blessing(
            blessing("universe.blessing.612842"),
            2,
            PathBattleEvent::UltimateViaBrainInVatUsed,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        shield[0].effect(),
        PathEffect::Shield { amount, duration_turns: 3, .. }
            if amount.raw_six_decimal() == 450_000_000
    ));
}

#[test]
fn ultimate_aoe_and_target_count_modifiers_are_typed() {
    let runtime = runtime();
    let resistance = runtime
        .execute_blessing(
            blessing("universe.blessing.612832"),
            2,
            PathBattleEvent::StatQueried,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        resistance[0].effect(),
        PathEffect::AddUltimateModifier { stat: PathEffectStat::AllTypeResistancePenetrationRatio, value, until_next_ultimate: false }
            if value.raw_six_decimal() == 370_000
    ));
    let mut multi_target_facts = facts();
    multi_target_facts.attacked_enemy_count = 2;
    let additional = runtime
        .execute_blessing(
            blessing("universe.blessing.612843"),
            2,
            PathBattleEvent::AttackCompleted,
            multi_target_facts,
        )
        .unwrap();
    assert!(matches!(
        additional[0].effect(),
        PathEffect::AdditionalDamagePerAttackedEnemy {
            enemy_count: 5,
            include_defeated_enemies_up_to: 5,
            ..
        }
    ));
    let ultimate = runtime
        .execute_blessing(
            blessing("universe.blessing.612850"),
            2,
            PathBattleEvent::StatQueried,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        ultimate[0].effect(),
        PathEffect::AddUltimateModifier { stat: PathEffectStat::DamageRatio, value, .. }
            if value.raw_six_decimal() == 900_000
    ));
}

#[test]
fn resonance_and_three_formations_are_typed() {
    let runtime = runtime();
    let synapse = runtime
        .execute_resonance(
            resonance("universe.resonance.612820"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        synapse[0].effect(),
        PathEffect::ApplySynapseResonance { damage_ratio_to_linked_targets, maximum_triggers: 15, .. }
            if damage_ratio_to_linked_targets.raw_six_decimal() == 300_000
    ));
    let melt = runtime
        .execute_resonance(
            resonance("universe.resonance.612821"),
            PathBattleEvent::BattleStarted,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        melt[0].effect(),
        PathEffect::ConfigureSynapseResonance { ultimate_attack_ratio, .. }
            if ultimate_attack_ratio.raw_six_decimal() == 500_000
    ));
    let memetic = runtime
        .execute_resonance(
            resonance("universe.resonance.612823"),
            PathBattleEvent::BattleStarted,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        memetic[0].effect(),
        PathEffect::ConfigureSynapseResonance { enemy_appearance_energy_maximum_ratio, .. }
            if enemy_appearance_energy_maximum_ratio.raw_six_decimal() == 50_000
    ));
}

#[test]
fn two_frozen_erudition_fixtures_are_runtime_backed() {
    assert_eq!(runtime().blessing_ids().len(), 18);
    assert_eq!(runtime().resonance_ids().len(), 4);
}
