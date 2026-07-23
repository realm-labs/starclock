use std::sync::{Arc, OnceLock};

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    curio::CurioStateKind,
    curio_effect_runtime::{CurioEffect, CurioEnergyChange, CurioHpChange},
    curio_runtime::{CurioContribution, CurioRuntimeCatalog},
    id::CurioId,
    negative_curio_runtime::{
        NEGATIVE_CURIO_RUNTIME_REVISION, NegativeCurioEvent, NegativeCurioRuntimeCatalog,
    },
    path_effect_runtime::{PathEffect, PathEffectStat, PathEffectTarget},
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

fn ownership() -> CurioRuntimeCatalog {
    CurioRuntimeCatalog::compile(catalog()).expect("ownership")
}

fn runtime() -> NegativeCurioRuntimeCatalog {
    NegativeCurioRuntimeCatalog::compile(&ownership()).expect("negative effects")
}

fn curio(key: &str) -> CurioId {
    catalog()
        .curios()
        .iter()
        .find(|value| value.stable_key() == key)
        .unwrap()
        .id()
}

fn contribution(key: &str, kind: CurioStateKind) -> CurioContribution {
    let ownership = ownership();
    let id = curio(key);
    let definition = ownership.definition(id).unwrap();
    let state = definition
        .states()
        .iter()
        .find(|state| state.kind() == kind)
        .unwrap();
    let charges = state
        .maximum_charges()
        .map_or_else(Vec::new, |maximum| vec![(id, maximum)]);
    ownership
        .contributions_from_owned(&[(id, 1)], &[(id, state.id())], &charges)
        .unwrap()
        .entries()[0]
        .clone()
}

#[test]
fn complete_negative_error_and_replacement_partition_compiles() {
    let runtime = runtime();
    assert_eq!(
        NEGATIVE_CURIO_RUNTIME_REVISION,
        "standard-universe-negative-curio-runtime-v1"
    );
    assert_eq!((runtime.content_count(), runtime.rule_count()), (42, 42));
    assert_eq!(
        (runtime.curio_count(), runtime.state_program_count()),
        (18, 24)
    );
    assert_eq!(
        runtime.digest(),
        [
            81, 185, 161, 165, 206, 97, 213, 180, 242, 14, 204, 157, 132, 172, 167, 170, 88, 200,
            60, 216, 18, 153, 215, 58, 167, 2, 221, 106, 14, 116, 122, 246,
        ]
    );
}

#[test]
fn every_negative_curio_state_has_an_executable_observation() {
    let ownership = ownership();
    let runtime = NegativeCurioRuntimeCatalog::compile(&ownership).unwrap();
    let events = [
        NegativeCurioEvent::Acquired,
        NegativeCurioEvent::BattleWon,
        NegativeCurioEvent::BlessingRewardOffered,
        NegativeCurioEvent::DomainEntered,
        NegativeCurioEvent::BattleStarted,
        NegativeCurioEvent::WeaknessBroken,
        NegativeCurioEvent::UltimateUsed,
        NegativeCurioEvent::DamageTakenCalculated,
        NegativeCurioEvent::SkillUsed,
        NegativeCurioEvent::EnemyDefeated,
        NegativeCurioEvent::BasicAttackUsed,
        NegativeCurioEvent::ActionEnded,
        NegativeCurioEvent::BlessingServicePriced,
        NegativeCurioEvent::StatQueried,
    ];
    let mut state_count = 0;
    for definition in ownership.definitions() {
        if !runtime.contains_curio(definition.curio()) {
            continue;
        }
        for state in definition.states() {
            let contribution = contribution(definition.stable_key(), state.kind());
            assert!(events.iter().any(|event| {
                runtime
                    .execute(&contribution, *event)
                    .is_ok_and(|effects| !effects.is_empty())
            }));
            state_count += 1;
        }
    }
    assert_eq!(state_count, 24);
}

#[test]
fn repair_and_replacement_effects_are_closed_and_typed() {
    let runtime = runtime();
    let trimmer = contribution("universe.curio.17", CurioStateKind::Active);
    let effects = runtime
        .execute(&trimmer, NegativeCurioEvent::Acquired)
        .unwrap();
    assert!(matches!(
        effects[0].effect(),
        CurioEffect::RepairRandomDestroyedCurios {
            maximum: 2,
            restore_default_charges: true
        }
    ));

    let die = contribution("universe.curio.21", CurioStateKind::Active);
    assert!(matches!(
        runtime.execute(&die, NegativeCurioEvent::Acquired).unwrap()[0].effect(),
        CurioEffect::ReplaceAllOwnedCuriosRandomly {
            include_source: true
        }
    ));
    let mask = contribution("universe.curio.115", CurioStateKind::Active);
    assert!(matches!(
        runtime
            .execute(&mask, NegativeCurioEvent::Acquired)
            .unwrap()[0]
            .effect(),
        CurioEffect::ReplaceAllBlessingsRandomly {
            retain_enhancement: true,
            released_higher_rarity_chance: true
        }
    ));
}

#[test]
fn error_codes_invert_behavior_only_after_the_authoritative_state_changes() {
    let runtime = runtime();
    let repairing = contribution("universe.curio.45", CurioStateKind::Repairing);
    let fixed = contribution("universe.curio.45", CurioStateKind::Fixed);
    assert!(matches!(
        runtime
            .execute(&repairing, NegativeCurioEvent::WeaknessBroken)
            .unwrap()[0]
            .effect(),
        CurioEffect::ChangeActorEnergy {
            change: CurioEnergyChange::Clear
        }
    ));
    assert!(matches!(
        runtime
            .execute(&fixed, NegativeCurioEvent::WeaknessBroken)
            .unwrap()[0]
            .effect(),
        CurioEffect::ChangeActorEnergy {
            change: CurioEnergyChange::RestoreMaximum
        }
    ));

    let hp = contribution("universe.curio.47", CurioStateKind::Repairing);
    assert!(matches!(
        runtime.execute(&hp, NegativeCurioEvent::UltimateUsed).unwrap()[0].effect(),
        CurioEffect::ChangeActorCurrentHpRatio {
            change: CurioHpChange::Consume,
            ratio,
            can_defeat: false
        } if ratio.raw_six_decimal() == 300_000
    ));

    let defense = contribution("universe.curio.49", CurioStateKind::Fixed);
    assert!(matches!(
        runtime
            .execute(&defense, NegativeCurioEvent::DamageTakenCalculated)
            .unwrap()[0]
            .effect(),
        CurioEffect::Battle(PathEffect::AddStat {
            target: PathEffectTarget::AllAllies,
            stat: PathEffectStat::DamageTakenReductionRatio,
            value,
            ..
        }) if value.raw_six_decimal() == 500_000
    ));
}

#[test]
fn negative_clocks_and_battle_effects_preserve_released_values() {
    let runtime = runtime();
    let parasitized = contribution("universe.curio.59", CurioStateKind::Active);
    assert!(matches!(
        runtime
            .execute(&parasitized, NegativeCurioEvent::BattleStarted)
            .unwrap()[0]
            .effect(),
        CurioEffect::ConfigureParasitized {
            attack_ratio,
            turn_current_hp_cost_ratio,
            transfer_to_random_ally_when_downed: true
        } if attack_ratio.raw_six_decimal() == 500_000
            && turn_current_hp_cost_ratio.raw_six_decimal() == 200_000
    ));

    let debt = contribution("universe.curio.60", CurioStateKind::Active);
    assert!(matches!(
        runtime
            .execute(&debt, NegativeCurioEvent::BattleWon)
            .unwrap()[0]
            .effect(),
        CurioEffect::SuppressBattleFragmentsThenDoubleCurrent { triggers: 5 }
    ));
    let ipc = contribution("universe.curio.70", CurioStateKind::Active);
    assert!(matches!(
        runtime
            .execute(&ipc, NegativeCurioEvent::BlessingServicePriced)
            .unwrap()[0]
            .effect(),
        CurioEffect::IncreaseBlessingServiceCost { ratio, .. }
            if ratio.raw_six_decimal() == 250_000
    ));
    let fission = contribution("universe.curio.108", CurioStateKind::Active);
    assert!(matches!(
        runtime
            .execute(&fission, NegativeCurioEvent::BattleWon)
            .unwrap()[0]
            .effect(),
        CurioEffect::ConfigureCurioFission {
            released_chance: true,
            maximum_concurrent_copies: 3
        }
    ));
}

#[test]
fn seven_frozen_negative_curio_fixtures_are_runtime_backed() {
    let runtime = runtime();
    assert_eq!((runtime.content_count(), runtime.rule_count()), (42, 42));
    assert_eq!(runtime.state_program_count(), 24);
}
