use super::*;

use starclock_mode_universe::{
    curio_effect_runtime::{
        CurioBlessingRarity, CurioEffect, CurioEffectFacts, CurioEffectRuntimeCatalog, CurioEvent,
    },
    curio_runtime::CurioRuntimeCatalog,
};

const ASSIGNED: [&str; 3] = ["universe.curio.69", "universe.curio.7", "universe.curio.8"];

#[test]
fn goal07_p3_m11_s06_uses_shared_activity_and_rule_ir_without_native_handlers() {
    let catalog = catalog();
    let runtime = CurioRuntimeCatalog::compile(&catalog).unwrap();
    for stable_key in ASSIGNED {
        let definition = runtime
            .definitions()
            .iter()
            .find(|definition| definition.stable_key() == stable_key)
            .expect("assigned Curio");
        let snapshot = contributions(
            &catalog,
            "universe.path.erudition",
            None,
            Some(stable_key),
            false,
        );
        let binding = snapshot
            .rules()
            .iter()
            .find(|binding| {
                binding.source_binding_key()
                    == definition
                        .states()
                        .iter()
                        .find(|state| state.id() == definition.initial_state())
                        .map(|state| state.source_effect_id())
            })
            .expect("Curio state binding");
        let materialization = materialize(&catalog, &snapshot);
        assert!(
            materialization
                .combat_catalog()
                .rule(binding.rule())
                .is_none_or(|rule| rule
                    .runtime()
                    .is_none_or(|runtime| runtime.native_handler().is_none()))
        );
    }
}

#[test]
fn s06_runtime_preserves_exact_public_reward_and_entry_damage_parameters() {
    let catalog = catalog();
    let curios = CurioRuntimeCatalog::compile(&catalog).unwrap();
    let effects = CurioEffectRuntimeCatalog::compile(&catalog, &curios).unwrap();

    let fortune = curio_id(&curios, "universe.curio.7");
    let fortune = effects
        .execute(
            fortune,
            CurioEvent::BlessingRewardOffered,
            CurioEffectFacts::default(),
        )
        .unwrap();
    assert!(fortune.iter().any(|effect| matches!(
        effect.effect(),
        CurioEffect::ConfigureBlessingReward {
            guaranteed_rarity: Some(CurioBlessingRarity::ThreeStar),
            ..
        }
    )));
    assert!(fortune.iter().any(|effect| matches!(
        effect.effect(),
        CurioEffect::DestroyAfterTriggers { triggers: 1 }
    )));

    let beacon = curio_id(&curios, "universe.curio.69");
    assert!(
        effects
            .execute(
                beacon,
                CurioEvent::BlessingRewardOffered,
                CurioEffectFacts::default(),
            )
            .unwrap()
            .iter()
            .any(|effect| matches!(
                effect.effect(),
                CurioEffect::ConfigureBlessingReward {
                    enhance_random_count: 1,
                    ..
                }
            ))
    );

    let parchment = curio_id(&curios, "universe.curio.8");
    assert!(
        effects
            .execute(
                parchment,
                CurioEvent::BattleStarted,
                CurioEffectFacts::default(),
            )
            .unwrap()
            .iter()
            .any(|effect| matches!(
                effect.effect(),
                CurioEffect::DamageEnemiesMaximumHpRatio { ratio }
                    if ratio.raw_six_decimal() == 300_000
            ))
    );
}

#[test]
fn parchment_deals_exactly_thirty_percent_of_each_enemy_maximum_hp_on_entry() {
    let catalog = catalog();
    let snapshot = contributions(
        &catalog,
        "universe.path.erudition",
        None,
        Some("universe.curio.8"),
        false,
    );
    let materialization = materialize(&catalog, &snapshot);
    let (_, resolution) = start(
        &materialization,
        durable_spec_with_two_enemy_hp(
            &materialization,
            0xe1,
            [
                Hp::new(1_000_000_000).unwrap(),
                Hp::new(2_000_000_000).unwrap(),
            ],
        ),
        0xe2,
    );
    let damage = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some((data.target, data.applied.get())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(damage.len(), 2);
    assert_eq!(damage[0].1 + damage[1].1, 900_000_000);
    assert!(damage.iter().any(|entry| entry.1 == 300_000_000));
    assert!(damage.iter().any(|entry| entry.1 == 600_000_000));
}

fn curio_id(
    runtime: &CurioRuntimeCatalog,
    stable_key: &str,
) -> starclock_mode_universe::id::CurioId {
    runtime
        .definitions()
        .iter()
        .find(|definition| definition.stable_key() == stable_key)
        .unwrap()
        .curio()
}
