use super::*;

#[test]
fn ordinary_enemy_batch_s17_materializes_all_frozen_variants_and_level_rows() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .expect("S17 materialization");
    let expected = [
        ("enemy.thunderspawn.minion.variant.01", 1_140_001, 1, 4),
        (
            "enemy.trotter-of-abundance.minionlv2.02.variant.01",
            1_140_002,
            4,
            5,
        ),
        (
            "enemy.trotter-of-abundance.minionlv2.variant.01",
            1_140_003,
            4,
            1,
        ),
        (
            "enemy.trotter-of-destruction.minionlv2.02.variant.01",
            1_140_004,
            4,
            5,
        ),
        (
            "enemy.trotter-of-destruction.minionlv2.variant.01",
            1_140_005,
            4,
            1,
        ),
        (
            "enemy.trotter-of-preservation.minionlv2.03.variant.01",
            1_140_006,
            4,
            5,
        ),
        (
            "enemy.trotter-of-preservation.minionlv2.variant.01",
            1_140_007,
            4,
            1,
        ),
        ("enemy.vagrant.minionlv2.variant.01", 1_140_008, 2, 5),
        (
            "enemy.voidranger-distorter.minionlv2.variant.01",
            1_140_009,
            2,
            7,
        ),
        (
            "enemy.voidranger-eliminator.minionlv2.variant.01",
            1_140_010,
            2,
            5,
        ),
        ("enemy.voidranger-reaver.minionlv2.variant.01", 110, 2, 10),
        (
            "enemy.voidranger-trampler-bug.elite.variant.01",
            1_140_012,
            5,
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
            .expect("S17 enemy definition");
        assert_eq!(definition.abilities().len(), ability_count);
        assert_eq!(definition.phases().len(), 1);
        assert_eq!(
            definition.ai_graph().map(|id| id.get()),
            Some(1_140_041 + index as u32)
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
fn ordinary_enemy_batch_s17_retains_trotter_and_voidranger_boundaries() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .expect("S17 materialization");
    let combat = materialized.combat_catalog();

    assert!(has_effect(ability_program(combat, 1_140_102), 1_145_001));
    for (enemy, effect) in [
        (1_140_002, 1_145_004),
        (1_140_004, 1_145_005),
        (1_140_006, 1_145_006),
    ] {
        let entry = combat
            .enemy(starclock_combat::EnemyDefinitionId::new(enemy).unwrap())
            .and_then(|definition| definition.phases()[0].entry_program())
            .and_then(|program| combat.program(program))
            .expect("Trotter passive entry");
        assert!(has_effect(entry, effect));
    }

    assert!(has_effect(ability_program(combat, 1_140_243), 1_145_007));
    assert!(has_effect(ability_program(combat, 1_140_262), 1_145_008));
    assert!(has_effect(ability_program(combat, 1_140_282), 1_145_009));
    assert!(has_effect(ability_program(combat, 1_140_282), 1_145_010));
    assert!(has_effect(ability_program(combat, 1_140_325), 1_145_011));
    assert!(has_effect(ability_program(combat, 1_140_326), 1_145_012));

    let detonated = combat
        .effect(starclock_combat::EffectDefinitionId::new(1_145_009).unwrap())
        .expect("Detonated");
    assert_eq!(detonated.rules().len(), 1);
    let runtime = combat
        .rule(detonated.rules()[0])
        .and_then(|rule| rule.runtime())
        .expect("Detonated runtime rule");
    let program = combat
        .program(runtime.triggers()[0].program)
        .expect("Detonated additional damage");
    assert!(program.steps().iter().any(|step| matches!(
        step,
        starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::Damage {
                class: starclock_combat::formula::model::DamageClass::Additional,
                element: starclock_combat::formula::model::CombatElement::Imaginary,
                ..
            }
        )
    )));
}

fn ability_program(
    catalog: &starclock_combat::catalog::CombatCatalog,
    ability: u32,
) -> &starclock_combat::catalog::definition::ProgramDefinition {
    catalog
        .ability(starclock_combat::AbilityId::new(ability).unwrap())
        .expect("authored S17 ability")
        .programs()
        .first()
        .and_then(|binding| catalog.program(binding.program()))
        .expect("authored S17 program")
}

fn has_effect(
    program: &starclock_combat::catalog::definition::ProgramDefinition,
    effect_id: u32,
) -> bool {
    program.steps().iter().any(|step| {
        matches!(
            step,
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::ApplyEffect {
                    effect,
                    ..
                }
            ) if effect.get() == effect_id
        )
    })
}
