use super::*;

#[test]
fn ordinary_enemy_batch_s14_materializes_all_frozen_variants_and_level_rows() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .expect("S14 materialization");
    let expected = [
        (
            "enemy.dreamjolt-troupes-mr-domescreen.minionlv2.variant.01",
            1_110_001,
            4,
            6,
        ),
        (
            "enemy.dreamjolt-troupes-spring-loader.minion.variant.01",
            1_110_002,
            1,
            3,
        ),
        (
            "enemy.dreamjolt-troupes-sweet-gorilla-bug.elite.variant.01",
            1_110_003,
            4,
            1,
        ),
        (
            "enemy.dreamjolt-troupes-sweet-gorilla.elite.variant.01",
            1_110_004,
            4,
            5,
        ),
        (
            "enemy.dreamjolt-troupes-winder-goon.minionlv2.variant.01",
            1_110_005,
            3,
            2,
        ),
        (
            "enemy.entranced-ingenium-golden-cloud-toad.minion.variant.01",
            1_110_006,
            1,
            3,
        ),
        (
            "enemy.entranced-ingenium-illumination-dragonfish.minionlv2.variant.01",
            102,
            2,
            2,
        ),
        (
            "enemy.entranced-ingenium-obedient-dracolion.minion.variant.01",
            1_110_008,
            1,
            3,
        ),
        (
            "enemy.everwinter-shadewalker.minionlv2.variant.01",
            1_110_009,
            1,
            2,
        ),
        ("enemy.flamespawn.minion.variant.01", 103, 1, 7),
        ("enemy.frigid-prowler.elite.variant.01", 1_110_011, 5, 6),
        ("enemy.frostspawn.minion.variant.01", 104, 1, 5),
    ];
    let core = catalog.simulation_catalog();
    for (index, (stable_key, raw_id, ability_count, stat_count)) in expected.into_iter().enumerate()
    {
        let enemy = materialized
            .enemies()
            .iter()
            .find(|enemy| enemy.stable_key() == stable_key)
            .unwrap_or_else(|| panic!("{stable_key} materialization"));
        assert_eq!(enemy.definition_match(), EnemyDefinitionMatch::Exact);
        assert_eq!(enemy.source_enemy().map(|id| id.get()), Some(raw_id));
        let definition = core
            .enemy(starclock_combat::EnemyDefinitionId::new(raw_id).unwrap())
            .expect("S14 enemy definition");
        assert_eq!(definition.abilities().len(), ability_count);
        assert_eq!(definition.phases().len(), 1);
        assert_eq!(
            definition.ai_graph().map(|id| id.get()),
            Some(1_110_041 + index as u32)
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
}

#[test]
fn ordinary_enemy_batch_s14_retains_candle_flame_summons_and_control() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .expect("S14 materialization");
    let combat = materialized.combat_catalog();

    let dragonfish = combat
        .enemy(starclock_combat::EnemyDefinitionId::new(102).unwrap())
        .expect("Illumination Dragonfish");
    assert_eq!(
        dragonfish.abilities(),
        &[
            starclock_combat::AbilityId::new(1_110_222).unwrap(),
            starclock_combat::AbilityId::new(1_110_223).unwrap(),
        ]
    );
    assert_eq!(
        combat
            .effect(starclock_combat::EffectDefinitionId::new(1_115_009).unwrap())
            .expect("Candle Flame")
            .rules()
            .len(),
        1
    );
    let entry = combat
        .program(
            dragonfish.phases()[0]
                .entry_program()
                .expect("candle entry"),
        )
        .expect("Illumination Dragonfish phase entry");
    assert!(matches!(
        entry.steps(),
        [starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::ApplyEffect {
                effect,
                ..
            }
        )] if effect.get() == 1_115_009
    ));

    assert_eq!(
        summoned_units(ability_program(combat, 1_110_303)),
        vec![1_110_351, 1_110_352]
    );
    for linked in 1_110_351..=1_110_352 {
        assert_eq!(
            combat
                .linked_unit(starclock_combat::UnitDefinitionId::new(linked).unwrap())
                .expect("Everwinter Shadewalker")
                .abilities(),
            &[starclock_combat::AbilityId::new(1_110_381).unwrap()]
        );
    }
    assert!(
        ability_program(combat, 1_110_305)
            .steps()
            .iter()
            .any(|step| matches!(
                step,
                starclock_combat::rule::model::ProgramStep::Operation(
                    starclock_combat::rule::model::RuleOperationTemplate::ApplyEffect {
                        effect,
                        ..
                    }
                ) if effect.get() == 1_115_010
            ))
    );
    assert!(
        ability_program(combat, 1_110_262)
            .steps()
            .iter()
            .any(|step| matches!(
                step,
                starclock_combat::rule::model::ProgramStep::Operation(
                    starclock_combat::rule::model::RuleOperationTemplate::DelayAction { .. }
                )
            ))
    );
}

fn ability_program(
    catalog: &starclock_combat::catalog::CombatCatalog,
    ability: u32,
) -> &starclock_combat::catalog::definition::ProgramDefinition {
    catalog
        .ability(starclock_combat::AbilityId::new(ability).unwrap())
        .expect("authored S14 ability")
        .programs()
        .first()
        .and_then(|binding| catalog.program(binding.program()))
        .expect("authored S14 program")
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
