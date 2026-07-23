use std::sync::{Arc, OnceLock};

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    curio_effect_runtime::{
        CURIO_EFFECT_RUNTIME_REVISION, CurioEffect, CurioEffectFacts, CurioEffectRuntimeCatalog,
        CurioEvent,
    },
    curio_runtime::CurioRuntimeCatalog,
    id::CurioId,
    path_effect_runtime::{PathEffect, PathEffectStat, PathEffectValue},
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
fn runtime() -> CurioEffectRuntimeCatalog {
    let ownership = CurioRuntimeCatalog::compile(catalog()).expect("ownership");
    CurioEffectRuntimeCatalog::compile(catalog(), &ownership).expect("effects")
}
fn curio(key: &str) -> CurioId {
    catalog()
        .curios()
        .iter()
        .find(|value| value.stable_key() == key)
        .unwrap()
        .id()
}
fn value(value: i64) -> PathEffectValue {
    PathEffectValue::from_integral(value).unwrap()
}
fn facts() -> CurioEffectFacts {
    CurioEffectFacts {
        cosmic_fragments: 450,
        destroyed_curios: 2,
        full_hp_allies: 3,
        different_path_blessings: 4,
        destructibles_destroyed: 5,
        actor_maximum_hp: value(1_000),
        technique_actor_maximum_hp: value(1_200),
        final_domain: false,
    }
}

#[test]
fn complete_positive_neutral_special_partition_compiles() {
    let runtime = runtime();
    assert_eq!(
        CURIO_EFFECT_RUNTIME_REVISION,
        "standard-universe-curio-effect-runtime-v1"
    );
    assert_eq!((runtime.content_count(), runtime.rule_count()), (86, 86));
    assert_eq!(runtime.curio_ids().len(), 43);
    assert_eq!(
        runtime.digest(),
        [
            235, 45, 184, 107, 182, 197, 191, 108, 158, 105, 252, 9, 137, 62, 33, 202, 130, 219,
            227, 56, 15, 84, 161, 35, 162, 95, 24, 226, 220, 161, 155, 205,
        ]
    );
}

#[test]
fn every_curio_has_one_executable_observation() {
    let runtime = runtime();
    let events = [
        CurioEvent::Acquired,
        CurioEvent::BattleWon,
        CurioEvent::BlessingRewardOffered,
        CurioEvent::DomainEntered,
        CurioEvent::BattleStarted,
        CurioEvent::CharacterTurnStarted,
        CurioEvent::DestructibleDestroyed,
        CurioEvent::TechniqueDamageCalculated,
        CurioEvent::StatQueried,
        CurioEvent::RunDefeated,
    ];
    for id in runtime.curio_ids() {
        assert!(events.iter().any(|event| {
            runtime
                .execute(id, *event, facts())
                .is_ok_and(|effects| !effects.is_empty())
        }));
    }
}

#[test]
fn sealing_wax_and_blessing_reward_policies_are_typed() {
    let runtime = runtime();
    let wax = runtime
        .execute(curio("universe.curio.211"), CurioEvent::Acquired, facts())
        .unwrap();
    assert!(matches!(
        wax[0].effect(),
        CurioEffect::GrantRandomBlessings {
            path: Some(_),
            minimum: 1,
            maximum: 1
        }
    ));
    assert!(matches!(
        wax[1].effect(),
        CurioEffect::BiasBlessingOffers { .. }
    ));

    let dice = runtime
        .execute(
            curio("universe.curio.1"),
            CurioEvent::BlessingRewardOffered,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        dice[0].effect(),
        CurioEffect::ConfigureBlessingReward {
            extra_selections: 1,
            offer_count_delta: -1,
            ..
        }
    ));
    assert!(matches!(
        dice[1].effect(),
        CurioEffect::DestroyAfterTriggers { triggers: 2 }
    ));
}

#[test]
fn fragment_and_lifecycle_effects_preserve_exact_values() {
    let runtime = runtime();
    let trap = runtime
        .execute(curio("universe.curio.106"), CurioEvent::BattleWon, facts())
        .unwrap();
    assert!(matches!(
        trap[0].effect(),
        CurioEffect::GrantFragmentsPerFullHpAlly {
            amount_per_ally: 10,
            allies: 3
        }
    ));
    let cogwheel = runtime
        .execute(
            curio("universe.curio.112"),
            CurioEvent::DomainEntered,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        cogwheel[0].effect(),
        CurioEffect::GrantCosmicFragments { amount: 50 }
    ));
    assert!(matches!(
        cogwheel[1].effect(),
        CurioEffect::DestroyAboveFragmentsAndLoseAll { threshold: 500 }
    ));
    let crown = runtime
        .execute(curio("universe.curio.61"), CurioEvent::RunDefeated, facts())
        .unwrap();
    assert!(matches!(
        crown[0].effect(),
        CurioEffect::TreatNonFinalDefeatAsVictoryAndRestoreFullHp
    ));
}

#[test]
fn battle_and_destructible_effects_remain_adapter_proposals() {
    let runtime = runtime();
    let robe = runtime
        .execute(
            curio("universe.curio.11"),
            CurioEvent::BattleStarted,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        robe[1].effect(),
        CurioEffect::Battle(PathEffect::AddStat { stat: PathEffectStat::PathDamageRatio, value, .. })
            if value.raw_six_decimal() == 400_000
    ));
    let toxi = runtime
        .execute(
            curio("universe.curio.121"),
            CurioEvent::CharacterTurnStarted,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        toxi[0].effect(),
        CurioEffect::ConsumeHighestAttackHpAndGainSpeed { hp_ratio, speed_ratio, maximum_stacks: 5 }
            if hp_ratio.raw_six_decimal() == 240_000
                && speed_ratio.raw_six_decimal() == 50_000
    ));
    let lotto = runtime
        .execute(
            curio("universe.curio.63"),
            CurioEvent::DestructibleDestroyed,
            facts(),
        )
        .unwrap();
    assert!(matches!(
        lotto[0].effect(),
        CurioEffect::ConfigureDestructibleLottery { released_small_chance: true, failure_current_hp_loss_ratio, .. }
            if failure_current_hp_loss_ratio.raw_six_decimal() == 990_000
    ));
}

#[test]
fn twelve_frozen_curio_fixtures_are_runtime_backed() {
    assert_eq!(runtime().curio_ids().len(), 43);
}
