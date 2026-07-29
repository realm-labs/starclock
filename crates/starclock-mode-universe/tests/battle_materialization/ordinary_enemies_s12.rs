use super::*;

#[test]
fn ordinary_enemy_batch_s12_materializes_all_frozen_variants_and_level_rows() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .expect("S12 materialization");
    let expected = [
        (
            "enemy.abundance-sprite-golden-hound.minionlv2.variant.01",
            95,
            2,
            5,
        ),
        (
            "enemy.abundance-sprite-malefic-ape-bug.elite.variant.01",
            1_090_002,
            3,
            5,
        ),
        (
            "enemy.abundance-sprite-malefic-ape.elite.variant.01",
            1_090_003,
            3,
            4,
        ),
        (
            "enemy.abundance-sprite-wooden-lupus.minionlv2.variant.01",
            1_090_004,
            2,
            5,
        ),
        ("enemy.antibaryon.minion.variant.01", 1_090_005, 1, 7),
        (
            "enemy.aurumaton-gatekeeper-bug.elite.variant.01",
            1_090_006,
            3,
            4,
        ),
        ("enemy.aurumaton-gatekeeper.elite.variant.01", 96, 4, 5),
        (
            "enemy.aurumaton-spectral-envoy.elite.variant.01",
            1_090_008,
            4,
            4,
        ),
        (
            "enemy.automaton-beetle.minionlv2.variant.01",
            1_090_009,
            1,
            4,
        ),
        ("enemy.automaton-direwolf.elite.variant.01", 1_090_010, 4, 6),
        ("enemy.automaton-grizzly.elite.variant.01", 1_090_011, 5, 5),
        (
            "enemy.automaton-hound.minionlv2.variant.01",
            1_090_012,
            2,
            5,
        ),
    ];
    let core = catalog.simulation_catalog();
    for (stable_key, raw_id, ability_count, stat_count) in expected {
        let enemy = materialized
            .enemies()
            .iter()
            .find(|enemy| enemy.stable_key() == stable_key)
            .unwrap_or_else(|| panic!("{stable_key} materialization"));
        assert_eq!(enemy.definition_match(), EnemyDefinitionMatch::Exact);
        assert_eq!(enemy.source_enemy().map(|id| id.get()), Some(raw_id));
        let definition = core
            .enemy(starclock_combat::EnemyDefinitionId::new(raw_id).unwrap())
            .expect("S12 enemy definition");
        assert_eq!(definition.abilities().len(), ability_count);
        assert_eq!(definition.phases().len(), 1);
        assert_eq!(
            definition.ai_graph().map(|id| id.get()),
            Some(1_090_041 + expected_index(stable_key))
        );
        assert_eq!(
            (1_u8..=90)
                .filter(|level| {
                    core.enemy_runtime_stat(
                        starclock_combat::EnemyDefinitionId::new(raw_id).unwrap(),
                        UnitLevel::new(*level).unwrap(),
                        "standard-universe-v1",
                    )
                    .is_some()
                })
                .count(),
            stat_count
        );
    }

    let golden_hound = core
        .enemy_runtime_stat(
            starclock_combat::EnemyDefinitionId::new(95).unwrap(),
            UnitLevel::new(47).unwrap(),
            "standard-universe-v1",
        )
        .expect("Golden Hound level 47");
    assert_eq!(golden_hound.hp().scaled(), 2_731_000_000);
    assert_eq!(golden_hound.attack().scaled(), 210_000_000);
    assert_eq!(golden_hound.defense().scaled(), 670_000_000);
    assert_eq!(golden_hound.speed().scaled(), 118_000_000);
    assert_eq!(golden_hound.effect_hit_rate().scaled(), 0);
    assert_eq!(golden_hound.effect_resistance().scaled(), 100_000);

    let gatekeeper = core
        .enemy_runtime_stat(
            starclock_combat::EnemyDefinitionId::new(96).unwrap(),
            UnitLevel::new(84).unwrap(),
            "standard-universe-v1",
        )
        .expect("Aurumaton Gatekeeper level 84");
    assert_eq!(gatekeeper.hp().scaled(), 167_044_000_000);
    assert_eq!(gatekeeper.attack().scaled(), 597_000_000);
    assert_eq!(gatekeeper.defense().scaled(), 1_040_000_000);
    assert_eq!(gatekeeper.speed().scaled(), 120_000_000);
    assert_eq!(gatekeeper.effect_hit_rate().scaled(), 272_000);
    assert_eq!(gatekeeper.effect_resistance().scaled(), 300_000);
}

#[test]
fn ordinary_enemy_batch_s12_retains_rebound_sanction_and_source_cycles() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .expect("S12 materialization");
    let combat = materialized.combat_catalog();

    let golden_hound = combat
        .enemy(starclock_combat::EnemyDefinitionId::new(95).unwrap())
        .expect("Golden Hound");
    assert_eq!(
        golden_hound.abilities(),
        &[
            starclock_combat::AbilityId::new(1_090_102).unwrap(),
            starclock_combat::AbilityId::new(1_090_103).unwrap(),
        ]
    );
    let rebound = combat
        .effect(starclock_combat::EffectDefinitionId::new(1_095_001).unwrap())
        .expect("Rebound Roar effect");
    assert_eq!(rebound.rules().len(), 1);
    let entry = combat
        .program(
            golden_hound.phases()[0]
                .entry_program()
                .expect("rebound entry"),
        )
        .expect("Golden Hound phase entry");
    assert!(matches!(
        entry.steps(),
        [starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::ApplyEffect {
                effect,
                ..
            }
        )] if effect.get() == 1_095_001
    ));

    let gatekeeper = combat
        .enemy(starclock_combat::EnemyDefinitionId::new(96).unwrap())
        .expect("Aurumaton Gatekeeper");
    let graph = combat
        .ai_graph(gatekeeper.ai_graph().expect("Gatekeeper AI"))
        .expect("Gatekeeper AI graph");
    assert_eq!(
        graph
            .states()
            .iter()
            .map(|state| state.candidates()[0].ability().get())
            .collect::<Vec<_>>(),
        vec![
            1_090_222, 1_090_222, 1_090_222, 1_090_223, 1_090_224, 1_090_225,
        ]
    );
    assert_eq!(
        summoned_units(ability_program(combat, 1_090_223)),
        vec![1_090_351, 1_090_352]
    );
    for linked in 1_090_351..=1_090_352 {
        assert_eq!(
            combat
                .linked_unit(starclock_combat::UnitDefinitionId::new(linked).unwrap())
                .expect("Illumination Dragonfish")
                .abilities(),
            &[starclock_combat::AbilityId::new(1_090_381).unwrap()]
        );
    }
    assert!(matches!(
        ability_program(combat, 1_090_225).steps().last(),
        Some(starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::DelayAction {
                amount,
                ..
            }
        )) if scalar_literal(amount) == Some(500_000)
    ));
}

fn expected_index(stable_key: &str) -> u32 {
    [
        "enemy.abundance-sprite-golden-hound.minionlv2.variant.01",
        "enemy.abundance-sprite-malefic-ape-bug.elite.variant.01",
        "enemy.abundance-sprite-malefic-ape.elite.variant.01",
        "enemy.abundance-sprite-wooden-lupus.minionlv2.variant.01",
        "enemy.antibaryon.minion.variant.01",
        "enemy.aurumaton-gatekeeper-bug.elite.variant.01",
        "enemy.aurumaton-gatekeeper.elite.variant.01",
        "enemy.aurumaton-spectral-envoy.elite.variant.01",
        "enemy.automaton-beetle.minionlv2.variant.01",
        "enemy.automaton-direwolf.elite.variant.01",
        "enemy.automaton-grizzly.elite.variant.01",
        "enemy.automaton-hound.minionlv2.variant.01",
    ]
    .iter()
    .position(|candidate| *candidate == stable_key)
    .expect("known S12 variant") as u32
}

fn ability_program(
    catalog: &starclock_combat::catalog::CombatCatalog,
    ability: u32,
) -> &starclock_combat::catalog::definition::ProgramDefinition {
    catalog
        .ability(starclock_combat::AbilityId::new(ability).unwrap())
        .expect("authored S12 ability")
        .programs()
        .first()
        .and_then(|binding| catalog.program(binding.program()))
        .expect("authored S12 program")
}

fn summoned_units(program: &starclock_combat::catalog::definition::ProgramDefinition) -> Vec<u32> {
    program
        .steps()
        .iter()
        .filter_map(|step| match step {
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::Summon {
                    unit_definition,
                    ..
                },
            ) => Some(unit_definition.get()),
            _ => None,
        })
        .collect()
}

fn scalar_literal(expression: &starclock_combat::rule::model::ValueExpr) -> Option<i64> {
    match expression {
        starclock_combat::rule::model::ValueExpr::Literal(
            starclock_combat::rule::model::RuleValue::Scalar(value),
        ) => Some(value.scaled()),
        _ => None,
    }
}
