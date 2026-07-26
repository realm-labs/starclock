use super::*;
use starclock_mode_universe::curio_effect_runtime::{
    CurioEffect, CurioEffectFacts, CurioEffectRuntimeCatalog, CurioEvent,
};

const ASSIGNED: [&str; 8] = [
    "universe.curio.23",
    "universe.curio.24",
    "universe.curio.25",
    "universe.curio.26",
    "universe.curio.27",
    "universe.curio.28",
    "universe.curio.3",
    "universe.curio.4",
];

#[test]
fn goal07_p3_m11_s04_executes_every_assigned_curio_without_native_handlers() {
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
                    .is_none_or(|runtime| runtime.native_handler().is_none())),
            "{stable_key} remains outside native combat handlers"
        );
    }
}

#[test]
fn s04_activity_effects_compile_to_typed_reward_and_revival_primitives() {
    let catalog = catalog();
    let curios = CurioRuntimeCatalog::compile(&catalog).unwrap();
    let effects = CurioEffectRuntimeCatalog::compile(&catalog, &curios).unwrap();

    for stable_key in &ASSIGNED[..6] {
        let curio = curios
            .definitions()
            .iter()
            .find(|definition| definition.stable_key() == *stable_key)
            .unwrap()
            .curio();
        let emitted = effects
            .execute(curio, CurioEvent::Acquired, CurioEffectFacts::default())
            .unwrap();
        assert!(emitted.iter().any(|effect| matches!(
            effect.effect(),
            CurioEffect::GrantRandomBlessings {
                minimum: 1,
                maximum: 1,
                ..
            }
        )));
        assert!(
            emitted
                .iter()
                .any(|effect| matches!(effect.effect(), CurioEffect::BiasBlessingOffers { .. }))
        );
    }

    let eye = curio_id(&curios, "universe.curio.3");
    let eye_effects = effects
        .execute(
            eye,
            CurioEvent::BlessingRewardOffered,
            CurioEffectFacts::default(),
        )
        .unwrap();
    assert!(eye_effects.iter().any(|effect| matches!(
        effect.effect(),
        CurioEffect::ConfigureBlessingReward {
            enhance_all_one_star: true,
            ..
        }
    )));

    let fruit = curio_id(&curios, "universe.curio.4");
    let fruit_effects = effects
        .execute(fruit, CurioEvent::BattleWon, CurioEffectFacts::default())
        .unwrap();
    assert!(
        fruit_effects
            .iter()
            .any(|effect| matches!(effect.effect(), CurioEffect::RevivePartyAndRestoreFullHp))
    );
    assert!(fruit_effects.iter().any(|effect| matches!(
        effect.effect(),
        CurioEffect::DestroyAfterTriggers { triggers: 1 }
    )));
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
