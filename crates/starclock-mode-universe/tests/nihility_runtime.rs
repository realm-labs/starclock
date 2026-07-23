use std::sync::{Arc, OnceLock};

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    id::{BlessingId, ResonanceId},
    nihility_runtime::{NIHILITY_RUNTIME_REVISION, NihilityRuntimeCatalog},
    path_effect_runtime::{
        PathBattleEvent, PathDotSelection, PathEffect, PathEffectFacts, PathEffectTarget,
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

fn runtime() -> NihilityRuntimeCatalog {
    NihilityRuntimeCatalog::compile(catalog()).expect("Nihility runtime")
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

fn facts() -> PathEffectFacts {
    PathEffectFacts {
        path_blessing_count: 9,
        suspicion_stacks: 75,
        dot_count: 4,
        enemy_is_weakness_broken: true,
        enemy_has_dot: true,
        dot_was_refreshed: true,
        ..PathEffectFacts::default()
    }
}

#[test]
fn complete_partition_compiles_through_one_closed_registry() {
    let runtime = runtime();
    assert_eq!(
        NIHILITY_RUNTIME_REVISION,
        "standard-universe-nihility-runtime-v1"
    );
    assert_eq!(runtime.content_count(), 59);
    assert_eq!(runtime.rule_count(), 58);
    assert_eq!(runtime.blessing_ids().len(), 18);
    assert_eq!(runtime.resonance_ids().len(), 4);
    assert_eq!(
        runtime.digest(),
        [
            209, 124, 68, 174, 98, 137, 142, 158, 49, 50, 237, 94, 28, 227, 129, 47, 234, 224, 226,
            254, 87, 131, 187, 111, 101, 55, 49, 53, 31, 76, 94, 179,
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
        PathBattleEvent::DotDamageTaken,
        PathBattleEvent::DotApplied,
        PathBattleEvent::DotRefreshed,
        PathBattleEvent::EnemyTurnStarted,
        PathBattleEvent::EnemyDefeated,
        PathBattleEvent::SuspicionApplying,
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
fn suspicion_decay_refresh_and_high_precision_parameters_are_exact() {
    let runtime = runtime();
    let suspicion = runtime
        .execute_blessing(
            blessing("universe.blessing.612230"),
            2,
            PathBattleEvent::DotDamageTaken,
            facts(),
        )
        .expect("Suspicion");
    assert_eq!(
        suspicion[0].source_key(),
        "universe.blessing.612230.level.2"
    );
    assert!(matches!(
        suspicion[0].effect(),
        PathEffect::ApplySuspicion {
            stacks: 1,
            maximum_stacks: 99,
            dot_vulnerability_per_stack,
            decay_per_turn: 2,
            prevent_decay: true,
            ..
        } if dot_vulnerability_per_stack.raw_six_decimal() == 10_000
    ));

    let refresh = runtime
        .execute_blessing(
            blessing("universe.blessing.612231"),
            2,
            PathBattleEvent::DotRefreshed,
            facts(),
        )
        .expect("Refresh Suspicion");
    assert!(matches!(
        refresh[0].effect(),
        PathEffect::ApplySuspicion { stacks: 1, .. }
    ));

    let reductions = runtime
        .execute_blessing(
            blessing("universe.blessing.612242"),
            2,
            PathBattleEvent::StatQueried,
            facts(),
        )
        .expect("Suspicion reductions");
    assert_eq!(reductions.len(), 2);
    for effect in reductions.iter() {
        assert!(matches!(
            effect.effect(),
            PathEffect::AddStat {
                value,
                cap: Some(cap),
                ..
            } if value.raw_six_decimal() == 300_000
                && cap.raw_six_decimal() == 300_000
        ));
    }
}

#[test]
fn dot_triggers_break_spread_healing_and_energy_are_typed() {
    let runtime = runtime();
    let all_dots = runtime
        .execute_blessing(
            blessing("universe.blessing.612232"),
            2,
            PathBattleEvent::EnemyTurnStarted,
            facts(),
        )
        .expect("All DoTs");
    assert!(matches!(
        all_dots[0].effect(),
        PathEffect::TriggerDots {
            selection: PathDotSelection::All,
            times: 1,
            damage_ratio,
            ..
        } if damage_ratio.raw_six_decimal() == 1_350_000
    ));

    let spread = runtime
        .execute_blessing(
            blessing("universe.blessing.612244"),
            2,
            PathBattleEvent::WeaknessBroken,
            facts(),
        )
        .expect("Break spread");
    assert!(matches!(
        spread[0].effect(),
        PathEffect::SpreadWeaknessBreak {
            target: PathEffectTarget::AllEnemies
        }
    ));

    let random_dot = runtime
        .execute_blessing(
            blessing("universe.blessing.612245"),
            2,
            PathBattleEvent::AttackHit,
            facts(),
        )
        .expect("Random DoT");
    assert!(matches!(
        random_dot[0].effect(),
        PathEffect::ApplyRandomBreakDot {
            duration_turns: 2,
            wind_shear_stacks: 1,
            dispel_attacker_debuff: true,
            ..
        }
    ));

    let heal = runtime
        .execute_blessing(
            blessing("universe.blessing.612256"),
            2,
            PathBattleEvent::DotDamageTaken,
            facts(),
        )
        .expect("Heal");
    assert!(matches!(
        heal[0].effect(),
        PathEffect::HealMaximumHpRatio {
            target: PathEffectTarget::AllAllies,
            ratio
        } if ratio.raw_six_decimal() == 15_000
    ));

    let energy = runtime
        .execute_blessing(
            blessing("universe.blessing.612257"),
            2,
            PathBattleEvent::DotDamageTaken,
            facts(),
        )
        .expect("Energy");
    assert!(matches!(
        energy[0].effect(),
        PathEffect::GainEnergy {
            target: PathEffectTarget::RandomAlly,
            amount,
            ..
        } if amount.raw_six_decimal() == 3_000_000
    ));
}

#[test]
fn resonance_and_all_three_formations_emit_typed_effects() {
    let runtime = runtime();
    let base = runtime
        .execute_resonance(
            resonance("universe.resonance.612220"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .expect("Resonance");
    assert!(matches!(
        base[0].effect(),
        PathEffect::ApplyResonanceDots {
            base_chance,
            duration_turns: 2,
            wind_shear_stacks: 2,
            burn_shock_attack_ratio,
            bleed_maximum_hp_ratio,
            ..
        } if base_chance.raw_six_decimal() == 800_000
            && burn_shock_attack_ratio.raw_six_decimal() == 100_000
            && bleed_maximum_hp_ratio.raw_six_decimal() == 50_000
    ));

    let application = runtime
        .execute_resonance(
            resonance("universe.resonance.612221"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .expect("Application formation");
    assert!(matches!(
        application[0].effect(),
        PathEffect::ModifyResonanceDotApplication {
            base_chance_bonus,
            duration_bonus_turns: 1,
            stackable_status_bonus: 1,
        } if base_chance_bonus.raw_six_decimal() == 1_000_000
    ));

    let statuses = runtime
        .execute_resonance(
            resonance("universe.resonance.612222"),
            PathBattleEvent::PathResonanceActivated,
            facts(),
        )
        .expect("Confusion/Devoid formation");
    assert!(matches!(
        statuses[0].effect(),
        PathEffect::ApplyConfusionAndDevoid {
            confusion_stacks: 2,
            confusion_dot_trigger_ratio,
            devoid_stacks: 2,
            toughness_recovery_reduction_per_stack,
            duration_turns: 2,
            ..
        } if confusion_dot_trigger_ratio.raw_six_decimal() == 300_000
            && toughness_recovery_reduction_per_stack.raw_six_decimal() == 100_000
    ));

    let energy = runtime
        .execute_resonance(
            resonance("universe.resonance.612223"),
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
fn two_frozen_semantic_fixtures_are_runtime_backed() {
    let runtime = runtime();
    let healing = blessing("universe.blessing.612256");
    assert!(runtime.blessing_ids().any(|candidate| candidate == healing));
    assert!(
        catalog()
            .blessing(healing)
            .expect("Blessing")
            .mechanic_tags()
            .iter()
            .any(|candidate| candidate.as_ref() == "healing")
    );
    assert_eq!(runtime.blessing_ids().len(), 18);
}
