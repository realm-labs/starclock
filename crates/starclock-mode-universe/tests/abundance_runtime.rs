use std::sync::{Arc, OnceLock};

use starclock_mode_universe::{
    abundance_runtime::{ABUNDANCE_RUNTIME_REVISION, AbundanceRuntimeCatalog},
    catalog::UniverseCatalog,
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
            let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
            UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog")
        })
        .as_ref()
}

fn runtime() -> AbundanceRuntimeCatalog {
    AbundanceRuntimeCatalog::compile(catalog()).expect("Abundance runtime")
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

fn value(integer: i64) -> PathEffectValue {
    PathEffectValue::from_integral(integer).expect("value")
}

fn facts() -> PathEffectFacts {
    PathEffectFacts {
        actor_maximum_hp: value(1_000),
        actor_current_hp: value(800),
        actor_base_attack: value(500),
        healing_amount: value(400),
        dewdrop_charge: value(500),
        path_blessing_count: 9,
        actor_is_full_hp: true,
        healing_was_from_ally: true,
        ..PathEffectFacts::default()
    }
}

#[test]
fn complete_partition_compiles_through_one_closed_registry() {
    let runtime = runtime();
    assert_eq!(
        ABUNDANCE_RUNTIME_REVISION,
        "standard-universe-abundance-runtime-v1"
    );
    assert_eq!(runtime.content_count(), 59);
    assert_eq!(runtime.rule_count(), 58);
    assert_eq!(runtime.blessing_ids().len(), 18);
    assert_eq!(runtime.resonance_ids().len(), 4);
    assert_eq!(
        runtime.digest(),
        [
            103, 237, 199, 123, 21, 176, 32, 35, 134, 240, 239, 13, 29, 159, 183, 83, 171, 45, 216,
            236, 24, 47, 119, 154, 96, 200, 138, 156, 112, 10, 141, 209,
        ]
    );
}

#[test]
fn every_blessing_level_and_resonance_has_an_executable_event() {
    let runtime = runtime();
    let events = [
        PathBattleEvent::BattleStarted,
        PathBattleEvent::WeaknessBroken,
        PathBattleEvent::PathResonanceActivated,
        PathBattleEvent::StatQueried,
        PathBattleEvent::HealingReceived,
        PathBattleEvent::TurnStarted,
        PathBattleEvent::HealingProvided,
        PathBattleEvent::DewdropRuptured,
        PathBattleEvent::AttackCompleted,
        PathBattleEvent::LethalDamageReceived,
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
fn dewdrop_charge_rupture_and_full_hp_efficiency_are_exact() {
    let runtime = runtime();
    let charge = runtime
        .execute_blessing(
            blessing("universe.blessing.612330"),
            2,
            PathBattleEvent::HealingReceived,
            facts(),
        )
        .expect("charge");
    assert!(matches!(
        charge[0].effect(),
        PathEffect::ChargeDewdrop {
            amount,
            maximum_hp_cap_ratio,
            damage_bonus_ratio,
            ruptures_after_attack: true,
            ..
        } if amount.raw_six_decimal() == 400_000_000
            && maximum_hp_cap_ratio.raw_six_decimal() == 1_000_000
            && damage_bonus_ratio.raw_six_decimal() == 400_000
    ));

    let turn_charge = runtime
        .execute_blessing(
            blessing("universe.blessing.612331"),
            2,
            PathBattleEvent::TurnStarted,
            facts(),
        )
        .expect("turn charge");
    assert!(matches!(
        turn_charge[0].effect(),
        PathEffect::ChargeDewdrop { amount, .. }
            if amount.raw_six_decimal() == 700_000_000
    ));

    let rupture = runtime
        .execute_blessing(
            blessing("universe.blessing.612340"),
            2,
            PathBattleEvent::DewdropRuptured,
            facts(),
        )
        .expect("rupture heal");
    assert!(matches!(
        rupture[0].effect(),
        PathEffect::HealAmount { amount, .. }
            if amount.raw_six_decimal() == 120_000_000
    ));

    let efficiency = runtime
        .execute_blessing(
            blessing("universe.blessing.612341"),
            2,
            PathBattleEvent::StatQueried,
            facts(),
        )
        .expect("efficiency");
    assert!(matches!(
        efficiency[0].effect(),
        PathEffect::ModifyDewdropChargeEfficiency { value, .. }
            if value.raw_six_decimal() == 1_200_000
    ));
}

#[test]
fn healing_attack_damage_stats_and_resources_are_typed() {
    let runtime = runtime();
    let shared = runtime
        .execute_blessing(
            blessing("universe.blessing.612332"),
            2,
            PathBattleEvent::HealingProvided,
            facts(),
        )
        .expect("shared heal");
    assert!(matches!(
        shared[0].effect(),
        PathEffect::HealAmount {
            target: PathEffectTarget::OtherAllies,
            amount,
            ..
        } if amount.raw_six_decimal() == 120_000_000
    ));
    assert!(matches!(
        shared[1].effect(),
        PathEffect::ScaleAttackFromHealing {
            healing_ratio,
            base_attack_cap_ratio,
            until_next_turn_end: true,
            ..
        } if healing_ratio.raw_six_decimal() == 150_000
            && base_attack_cap_ratio.raw_six_decimal() == 800_000
    ));

    let damage = runtime
        .execute_blessing(
            blessing("universe.blessing.612344"),
            2,
            PathBattleEvent::AttackCompleted,
            facts(),
        )
        .expect("HP damage");
    assert!(matches!(
        damage[0].effect(),
        PathEffect::Damage { amount, .. }
            if amount.raw_six_decimal() == 420_000_000
    ));

    let maximum_hp = runtime
        .execute_blessing(
            blessing("universe.blessing.612350"),
            2,
            PathBattleEvent::StatQueried,
            facts(),
        )
        .expect("maximum HP");
    assert!(matches!(
        maximum_hp[0].effect(),
        PathEffect::AddStat {
            stat: PathEffectStat::MaximumHpRatio,
            value,
            ..
        } if value.raw_six_decimal() == 630_000
    ));

    let skill_point = runtime
        .execute_blessing(
            blessing("universe.blessing.612357"),
            2,
            PathBattleEvent::HealingProvided,
            facts(),
        )
        .expect("skill point");
    assert!(matches!(
        skill_point[0].effect(),
        PathEffect::GainSkillPoint {
            fixed_chance,
            amount: 1,
            once_per_action: true,
        } if fixed_chance.raw_six_decimal() == 450_000
    ));
}

#[test]
fn resonance_and_all_three_formations_emit_typed_effects() {
    let runtime = runtime();
    let base = runtime
        .execute_resonance(
            resonance("universe.resonance.612320"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .expect("resonance");
    assert!(matches!(
        base[0].effect(),
        PathEffect::HealMaximumHpRatio {
            target: PathEffectTarget::AllAllies,
            ratio,
        } if ratio.raw_six_decimal() == 500_000
    ));
    assert!(matches!(
        base[1].effect(),
        PathEffect::ApplyTimedStat {
            stat: PathEffectStat::MaximumHpRatio,
            value,
            duration_turns: 2,
            ..
        } if value.raw_six_decimal() == 150_000
    ));

    let prevention = runtime
        .execute_resonance(
            resonance("universe.resonance.612321"),
            PathBattleEvent::LethalDamageReceived,
            facts(),
        )
        .expect("prevention");
    assert!(matches!(
        prevention[0].effect(),
        PathEffect::PreventDefeatAndActivateResonance {
            maximum_triggers_per_battle: 1,
            consume_all_energy: true,
            ..
        }
    ));

    let cleanse = runtime
        .execute_resonance(
            resonance("universe.resonance.612322"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .expect("Subduing Evils");
    assert!(matches!(
        cleanse[1].effect(),
        PathEffect::ApplySubduingEvils {
            stacks: 1,
            maximum_stacks: 5,
            duration_turns: 1,
            heal_maximum_hp_ratio_on_block,
            ..
        } if heal_maximum_hp_ratio_on_block.raw_six_decimal() == 100_000
    ));

    let action = runtime
        .execute_resonance(
            resonance("universe.resonance.612323"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .expect("resonance action");
    assert!(matches!(
        action[0].effect(),
        PathEffect::InstallResonanceAction {
            healing_reduction_ratio,
            activate_after_first_manual_use: true,
        } if healing_reduction_ratio.raw_six_decimal() == 300_000
    ));
}

#[test]
fn three_frozen_semantic_fixtures_are_runtime_backed() {
    let runtime = runtime();
    for key in ["universe.blessing.612356", "universe.blessing.612357"] {
        let id = blessing(key);
        assert!(runtime.blessing_ids().any(|candidate| candidate == id));
    }
    assert_eq!(runtime.blessing_ids().len(), 18);
}
