use super::*;

use starclock_combat::{BattleEventKind, formula::model::DamageClass};
use starclock_mode_universe::curio_effect_runtime::{
    CurioBlessingGrantPool, CurioDestructibleReward, CurioEffect, CurioEffectFacts,
    CurioEffectRuntimeCatalog, CurioEvent,
};

const ASSIGNED: [&str; 8] = [
    "universe.curio.5",
    "universe.curio.58",
    "universe.curio.6",
    "universe.curio.61",
    "universe.curio.62",
    "universe.curio.63",
    "universe.curio.64",
    "universe.curio.68",
];

#[test]
fn goal07_p3_m11_s05_keeps_all_assigned_curios_out_of_native_handlers() {
    let catalog = catalog();
    let runtime = CurioRuntimeCatalog::compile(&catalog).unwrap();
    let build_roster = compiled_build_roster(&catalog);
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
        let materialization = if stable_key == "universe.curio.62" {
            materialize_with_roster(&catalog, &build_roster, &snapshot)
        } else {
            materialize(&catalog, &snapshot)
        };
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
fn space_time_prism_recompiles_each_exact_build_at_one_higher_eidolon() {
    let catalog = catalog();
    let roster = compiled_build_roster(&catalog);
    let plain = contributions(&catalog, "universe.path.erudition", None, None, false);
    let prism = contributions(
        &catalog,
        "universe.path.erudition",
        None,
        Some("universe.curio.62"),
        false,
    );
    assert_eq!(prism.eidolon_resonance_levels(), 1);
    let plain = materialize_with_roster(&catalog, &roster, &plain);
    let prism = materialize_with_roster(&catalog, &roster, &prism);
    let plain_players = first_variant(&plain)
        .participants()
        .iter()
        .filter(|participant| participant.side() == TeamSide::Player)
        .map(|participant| participant.combatant())
        .collect::<Vec<_>>();
    let prism_players = first_variant(&prism)
        .participants()
        .iter()
        .filter(|participant| participant.side() == TeamSide::Player)
        .map(|participant| participant.combatant())
        .collect::<Vec<_>>();
    assert_eq!(plain_players.len(), prism_players.len());
    assert!(
        plain_players
            .iter()
            .zip(&prism_players)
            .any(|(plain, prism)| {
                plain.abilities() != prism.abilities()
                    || plain.rule_bundles() != prism.rule_bundles()
                    || plain.modifiers() != prism.modifiers()
            })
    );
}

#[test]
fn s05_runtime_preserves_exact_released_typed_effects() {
    let catalog = catalog();
    let curios = CurioRuntimeCatalog::compile(&catalog).unwrap();
    let effects = CurioEffectRuntimeCatalog::compile(&catalog, &curios).unwrap();

    let casket = curio_id(&curios, "universe.curio.5");
    assert!(
        effects
            .execute(casket, CurioEvent::Acquired, CurioEffectFacts::default())
            .unwrap()
            .iter()
            .any(|effect| matches!(
                effect.effect(),
                CurioEffect::GrantRandomBlessings {
                    pool: CurioBlessingGrantPool::AllEligible,
                    minimum: 1,
                    maximum: 2,
                    ..
                }
            ))
    );
    let lotto = curio_id(&curios, "universe.curio.63");
    assert!(
        effects
            .execute(
                lotto,
                CurioEvent::DestructibleDestroyed,
                CurioEffectFacts::default(),
            )
            .unwrap()
            .iter()
            .any(|effect| matches!(
                effect.effect(),
                CurioEffect::ConfigureDestructibleLottery {
                    reward: CurioDestructibleReward::Curio,
                    failure_current_hp_loss_ratio,
                    ..
                } if failure_current_hp_loss_ratio.raw_six_decimal() == 990_000
            ))
    );
    let crown = curio_id(&curios, "universe.curio.61");
    assert!(
        effects
            .execute(
                crown,
                CurioEvent::RunDefeated,
                CurioEffectFacts {
                    final_domain: false,
                    ..CurioEffectFacts::default()
                },
            )
            .unwrap()
            .iter()
            .any(|effect| matches!(
                effect.effect(),
                CurioEffect::TreatNonFinalDefeatAsVictoryAndRestoreFullHp
            ))
    );
}

#[test]
fn wick_trimmer_applies_three_percent_damage_per_destroyed_object() {
    let catalog = catalog();
    let plain = contributions(&catalog, "universe.path.erudition", None, None, false);
    let wick = contributions_many_with_curio_runtime(
        &catalog,
        "universe.path.erudition",
        &[],
        &[],
        Some("universe.curio.58"),
        false,
        0,
        4,
        &[],
    );
    let plain = materialize(&catalog, &plain);
    let wick = materialize(&catalog, &wick);
    let (mut plain_battle, _) = start(&plain, durable_spec(&plain, 0xd1, false), 0xd2);
    let (mut wick_battle, _) = start(&wick, durable_spec(&wick, 0xd1, false), 0xd2);
    let plain_damage = direct_damage(&first_normal_action(&mut plain_battle));
    let wick_damage = direct_damage(&first_normal_action(&mut wick_battle));
    assert!(plain_damage > 0);
    assert_eq!(wick_damage, plain_damage * 112 / 100);
}

#[test]
fn punklorde_implants_one_shared_allied_element_for_three_target_turns() {
    let catalog = catalog();
    let contribution = contributions(
        &catalog,
        "universe.path.erudition",
        None,
        Some("universe.curio.68"),
        false,
    );
    let materialization = materialize(&catalog, &contribution);
    let (_, resolution) = start(
        &materialization,
        durable_spec_with_two_enemies(&materialization, 0xd3),
        0xd4,
    );
    let additions = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Toughness(starclock_combat::ToughnessEventData::WeaknessAdded {
                target,
                element,
                duration_turns,
                ..
            }) => Some((*target, *element, *duration_turns)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(additions.len(), 2);
    assert!(additions.iter().all(|entry| entry.1 == additions[0].1));
    assert!(additions.iter().all(|entry| entry.2 == Some(3)));
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

fn direct_damage(resolution: &starclock_combat::Resolution) -> i64 {
    resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) if data.class == DamageClass::Direct => {
                Some(data.raw.scaled())
            }
            _ => None,
        })
        .sum()
}

fn compiled_build_roster(catalog: &Arc<UniverseCatalog>) -> UniverseBattleRoster {
    use starclock_build::{
        ability::AbilityInvestment,
        compiler::LoadoutCompiler,
        spec::{CombatantBuildSpec, EidolonLevel, PromotionStage},
    };
    let policy = ParticipantPolicy::new(
        1,
        4,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .unwrap();
    let core = catalog.simulation_catalog();
    let mut locks = Vec::new();
    let mut builds = Vec::new();
    for (index, raw) in [8_u32, 1, 2, 3].into_iter().enumerate() {
        let form = UnitDefinitionId::new(raw).unwrap();
        let character = core.build_catalog().character(form).unwrap();
        let investments = character
            .ability_levels()
            .iter()
            .map(|table| AbilityInvestment::new(table.family(), table.invested_cap()))
            .collect::<Vec<_>>();
        let spec = CombatantBuildSpec::new(
            form,
            UnitLevel::new(80).unwrap(),
            PromotionStage::new(6).unwrap(),
        )
        .with_ability_levels(investments)
        .unwrap()
        .with_eidolon(EidolonLevel::new(2).unwrap());
        let compiled = LoadoutCompiler
            .compile(core.build_catalog(), core.combat_catalog(), &spec)
            .unwrap();
        let participant = ParticipantId::new(u32::try_from(index + 1).unwrap()).unwrap();
        locks.push(
            ParticipantLockEntry::new(
                participant,
                0,
                u8::try_from(index).unwrap(),
                form,
                OpaqueParticipantBuild::new(
                    compiled.combatant().digest(),
                    BuildDigest::new(compiled.build_digest().bytes()).unwrap(),
                    core.build_catalog().revision().as_str(),
                    ParticipantSourceKind::CompiledBuild,
                )
                .unwrap(),
            )
            .unwrap(),
        );
        builds.push((participant, spec, compiled.combatant().clone()));
    }
    let lock = ParticipantLock::seal(policy, locks).unwrap();
    UniverseBattleRoster::new_with_build_specs(&lock, builds).unwrap()
}

fn first_variant(materialization: &UniverseBattleMaterialization) -> &starclock_combat::BattleSpec {
    materialization.overlay().bindings()[0]
        .preparation()
        .variants()[0]
        .battle_spec()
}
