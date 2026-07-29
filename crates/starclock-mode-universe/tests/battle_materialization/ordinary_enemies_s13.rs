use super::*;

#[test]
fn ordinary_enemy_batch_s13_materializes_all_frozen_variants_and_level_rows() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .expect("S13 materialization");
    let expected = [
        ("enemy.automaton-spider.minionlv2.variant.01", 97, 4, 6),
        ("enemy.baryon.minion.variant.01", 1_100_002, 1, 6),
        (
            "enemy.cloud-knights-patroller.minionlv2.variant.01",
            1_100_003,
            2,
            3,
        ),
        ("enemy.decaying-shadow.elite.variant.01", 1_100_004, 4, 7),
        (
            "enemy.disciples-of-sanctus-medicus-ballistarius.minionlv2.variant.01",
            99,
            1,
            3,
        ),
        (
            "enemy.disciples-of-sanctus-medicus-internal-alchemist.minionlv2.variant.01",
            1_100_006,
            3,
            2,
        ),
        (
            "enemy.disciples-of-sanctus-medicus-shape-shifter-bug.elite.variant.01",
            1_100_007,
            4,
            4,
        ),
        (
            "enemy.disciples-of-sanctus-medicus-shape-shifter.elite.variant.01",
            100,
            4,
            4,
        ),
        (
            "enemy.dreamjolt-troupes-beyond-overcooked-bug.elite.variant.01",
            1_100_009,
            5,
            1,
        ),
        (
            "enemy.dreamjolt-troupes-beyond-overcooked.elite.variant.01",
            1_100_010,
            5,
            5,
        ),
        (
            "enemy.dreamjolt-troupes-birdskull.minion.variant.01",
            1_100_011,
            1,
            3,
        ),
        (
            "enemy.dreamjolt-troupes-bubble-hound.minionlv2.variant.01",
            1_100_012,
            3,
            4,
        ),
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
            .expect("S13 enemy definition");
        assert_eq!(definition.abilities().len(), ability_count);
        assert_eq!(definition.phases().len(), 1);
        assert_eq!(
            definition.ai_graph().map(|id| id.get()),
            Some(1_100_041 + index as u32)
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

    let spider = core
        .enemy_runtime_stat(
            starclock_combat::EnemyDefinitionId::new(97).unwrap(),
            UnitLevel::new(31).unwrap(),
            "standard-universe-v1",
        )
        .expect("Automaton Spider level 31");
    assert_eq!(spider.hp().scaled(), 396_000_000);
    assert_eq!(spider.attack().scaled(), 104_000_000);
    assert_eq!(spider.defense().scaled(), 510_000_000);
    assert_eq!(spider.speed().scaled(), 83_000_000);
    assert_eq!(spider.effect_hit_rate().scaled(), 0);
    assert_eq!(spider.effect_resistance().scaled(), 100_000);
}

#[test]
fn ordinary_enemy_batch_s13_retains_chains_bounces_summons_and_vigor_drain() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .expect("S13 materialization");
    let combat = materialized.combat_catalog();

    let spider = combat
        .enemy(starclock_combat::EnemyDefinitionId::new(97).unwrap())
        .expect("Automaton Spider");
    assert_eq!(
        spider.abilities(),
        &[
            starclock_combat::AbilityId::new(1_100_102).unwrap(),
            starclock_combat::AbilityId::new(1_100_103).unwrap(),
            starclock_combat::AbilityId::new(1_100_104).unwrap(),
            starclock_combat::AbilityId::new(1_100_105).unwrap(),
        ]
    );
    assert_eq!(
        combat
            .effect(starclock_combat::EffectDefinitionId::new(1_105_001).unwrap())
            .expect("Chains of Destruction")
            .rules()
            .len(),
        1
    );
    let entry = combat
        .program(spider.phases()[0].entry_program().expect("chains entry"))
        .expect("Automaton Spider phase entry");
    assert!(matches!(
        entry.steps(),
        [starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::ApplyEffect {
                effect,
                ..
            }
        )] if effect.get() == 1_105_001
    ));

    let crossbow = ability_program(combat, 1_100_182);
    assert_eq!(
        crossbow
            .steps()
            .iter()
            .filter(|step| matches!(
                step,
                starclock_combat::rule::model::ProgramStep::Operation(
                    starclock_combat::rule::model::RuleOperationTemplate::Damage { .. }
                )
            ))
            .count(),
        6
    );

    assert_eq!(
        summoned_units(ability_program(combat, 1_100_244)),
        vec![1_100_351, 1_100_352]
    );
    for linked in 1_100_351..=1_100_352 {
        assert_eq!(
            combat
                .linked_unit(starclock_combat::UnitDefinitionId::new(linked).unwrap())
                .expect("Mara-Struck Soldier")
                .abilities(),
            &[starclock_combat::AbilityId::new(1_100_381).unwrap()]
        );
    }
    assert!(
        ability_program(combat, 1_100_242)
            .steps()
            .iter()
            .any(|step| matches!(
                step,
                starclock_combat::rule::model::ProgramStep::Operation(
                    starclock_combat::rule::model::RuleOperationTemplate::Heal { .. }
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
        .expect("authored S13 ability")
        .programs()
        .first()
        .and_then(|binding| catalog.program(binding.program()))
        .expect("authored S13 program")
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
