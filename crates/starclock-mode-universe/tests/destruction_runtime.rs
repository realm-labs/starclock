use std::sync::{Arc, OnceLock};

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    destruction_runtime::{DESTRUCTION_RUNTIME_REVISION, DestructionRuntimeCatalog},
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
fn runtime() -> DestructionRuntimeCatalog {
    DestructionRuntimeCatalog::compile(catalog()).expect("Destruction")
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
fn ratio(raw: i64) -> PathEffectValue {
    PathEffectValue::from_raw_six_decimal(raw)
}
fn facts() -> PathEffectFacts {
    PathEffectFacts {
        actor_maximum_hp: value(1_000),
        actor_current_hp: value(300),
        actor_current_hp_ratio: ratio(300_000),
        actor_hp_lost_ratio: ratio(700_000),
        hp_lost: value(700),
        party_hp_lost: value(1_200),
        actor_base_attack: value(1_000),
        enemy_current_hp_ratio: ratio(400_000),
        path_blessing_count: 9,
        grit_stacks: 4,
        ..PathEffectFacts::default()
    }
}

#[test]
fn complete_partition_compiles_through_one_closed_registry() {
    let runtime = runtime();
    assert_eq!(
        DESTRUCTION_RUNTIME_REVISION,
        "standard-universe-destruction-runtime-v1"
    );
    assert_eq!((runtime.content_count(), runtime.rule_count()), (59, 58));
    assert_eq!(runtime.blessing_ids().len(), 18);
    assert_eq!(runtime.resonance_ids().len(), 4);
    assert_eq!(
        runtime.digest(),
        [
            192, 210, 254, 1, 176, 92, 239, 4, 157, 221, 66, 143, 33, 61, 88, 120, 85, 233, 214,
            196, 175, 199, 184, 163, 229, 64, 77, 27, 166, 205, 125, 51,
        ]
    );
}

#[test]
fn every_level_and_resonance_has_an_executable_event() {
    let runtime = runtime();
    let events = [
        PathBattleEvent::BattleStarted,
        PathBattleEvent::TurnEnded,
        PathBattleEvent::AttackStarted,
        PathBattleEvent::CharacterAttacked,
        PathBattleEvent::HpLost,
        PathBattleEvent::DamageCalculated,
        PathBattleEvent::UltimateUsed,
        PathBattleEvent::LethalDamageReceived,
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
fn grit_gain_retaliation_and_hp_consumption_are_exact() {
    let runtime = runtime();
    let grit = runtime
        .execute_blessing(
            blessing("universe.blessing.612531"),
            2,
            PathBattleEvent::HpLost,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        grit[0].effect(),
        PathEffect::ModifyGrit {
            stacks: 4,
            adjacent_stacks: 1,
            maximum_stacks: 35,
            once_per_action: true,
            ..
        }
    ));
    let retaliation = runtime
        .execute_blessing(
            blessing("universe.blessing.612540"),
            2,
            PathBattleEvent::CharacterAttacked,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        retaliation[0].effect(),
        PathEffect::RetaliateFromGrit { target: PathEffectTarget::Attacker, amount, can_defeat: false }
            if amount.raw_six_decimal() == 216_000_000
    ));
    let consumption = runtime
        .execute_blessing(
            blessing("universe.blessing.612541"),
            2,
            PathBattleEvent::AttackStarted,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        consumption[0].effect(),
        PathEffect::ConsumeCurrentHpAndDamage { hp_cost_ratio, damage_amount, .. }
            if hp_cost_ratio.raw_six_decimal() == 100_000
                && damage_amount.raw_six_decimal() == 19_200_000
    ));
}

#[test]
fn lost_hp_modifiers_healing_and_shields_are_typed() {
    let runtime = runtime();
    let stats = runtime
        .execute_blessing(
            blessing("universe.blessing.612543"),
            2,
            PathBattleEvent::StatQueried,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        stats[0].effect(),
        PathEffect::AddStat { stat: PathEffectStat::AttackRatio, value, .. }
            if value.raw_six_decimal() == 560_000
    ));
    assert!(matches!(
        stats[1].effect(),
        PathEffect::AddStat { stat: PathEffectStat::DefenseRatio, value, .. }
            if value.raw_six_decimal() == 350_000
    ));
    let healing = runtime
        .execute_blessing(
            blessing("universe.blessing.612545"),
            2,
            PathBattleEvent::HpLost,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        healing[0].effect(),
        PathEffect::HealMaximumHpRatioCappedPerAction { ratio, cap_ratio, .. }
            if ratio.raw_six_decimal() == 200_000 && cap_ratio.raw_six_decimal() == 500_000
    ));
    let shield = runtime
        .execute_blessing(
            blessing("universe.blessing.612546"),
            2,
            PathBattleEvent::UltimateUsed,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        shield[0].effect(),
        PathEffect::Shield { amount, duration_turns: 2, .. }
            if amount.raw_six_decimal() == 245_000_000
    ));
}

#[test]
fn resonance_damage_and_three_formations_preserve_released_values() {
    let runtime = runtime();
    let damage = runtime
        .execute_resonance(
            resonance("universe.resonance.612520"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        damage[0].effect(),
        PathEffect::Damage { amount, element: starclock_mode_universe::path_effect_runtime::PathEffectElement::Fire, .. }
            if amount.raw_six_decimal() == 3_000_000_000
    ));
    let shield = runtime
        .execute_resonance(
            resonance("universe.resonance.612521"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        shield[0].effect(),
        PathEffect::ConsumePartyHpForResonance { remaining_hp_ratio, resonance_damage_bonus_ratio, shield_duration_turns: 2 }
            if remaining_hp_ratio.raw_six_decimal() == 400_000
                && resonance_damage_bonus_ratio.raw_six_decimal() == 200_000
    ));
    let entropic = runtime
        .execute_resonance(
            resonance("universe.resonance.612522"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        entropic[0].effect(),
        PathEffect::ApplyEntropicRetribution { base_chance, duration_turns: 2, defense_reduction_ratio, party_hp_lost_damage_ratio, .. }
            if base_chance.raw_six_decimal() == 1_500_000
                && defense_reduction_ratio.raw_six_decimal() == 200_000
                && party_hp_lost_damage_ratio.raw_six_decimal() == 1_250_000
    ));
    let automatic = runtime
        .execute_resonance(
            resonance("universe.resonance.612523"),
            PathBattleEvent::CharacterAttacked,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        automatic[0].effect(),
        PathEffect::AutoActivateResonance {
            maximum_triggers_per_battle: 2,
            consume_energy: false,
            ..
        }
    ));
}

#[test]
fn frozen_destruction_path_fixture_is_runtime_backed() {
    assert_eq!(runtime().blessing_ids().len(), 18);
    assert_eq!(runtime().resonance_ids().len(), 4);
}
