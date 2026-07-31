use super::*;

#[test]
fn ordinary_enemy_batch_s16_materializes_all_frozen_variants_and_level_rows() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .expect("S16 materialization");
    let expected = [
        (
            "enemy.memory-zone-meme-shell-of-faded-rage.elite.variant.01",
            1_130_001,
            5,
            5,
        ),
        (
            "enemy.memory-zone-meme-something-in-the-mirror.minionlv2.variant.02",
            1_130_002,
            2,
            1,
        ),
        ("enemy.searing-prowler.elite.variant.01", 1_130_003, 5, 7),
        (
            "enemy.senior-staff-team-leader-bug.elite.variant.01",
            1_130_004,
            8,
            5,
        ),
        (
            "enemy.senior-staff-team-leader.elite.variant.01",
            1_130_005,
            7,
            4,
        ),
        (
            "enemy.silvermane-cannoneer.minionlv2.variant.01",
            1_130_006,
            3,
            3,
        ),
        (
            "enemy.silvermane-gunner.minionlv2.variant.01",
            1_130_007,
            1,
            3,
        ),
        (
            "enemy.silvermane-lieutenant-bug.elite.variant.01",
            1_130_008,
            5,
            6,
        ),
        (
            "enemy.silvermane-soldier.minionlv2.variant.01",
            1_130_009,
            1,
            1,
        ),
        ("enemy.stormbringer-bug.elite.variant.01", 1_130_010, 9, 6),
        ("enemy.stormbringer.elite.variant.01", 1_130_011, 6, 4),
        ("enemy.the-ascended.elite.variant.01", 1_130_012, 5, 4),
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
            .expect("S16 enemy definition");
        assert_eq!(definition.abilities().len(), ability_count);
        assert_eq!(definition.phases().len(), 1);
        assert_eq!(
            definition.ai_graph().map(|id| id.get()),
            Some(1_130_041 + index as u32)
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
fn ordinary_enemy_batch_s16_retains_burn_reinforcement_reflect_and_prana() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .expect("S16 materialization");
    let combat = materialized.combat_catalog();

    assert!(has_effect(ability_program(combat, 1_130_142), 1_135_004));
    assert!(has_effect(ability_program(combat, 1_130_145), 1_135_006));

    for ability in [1_130_167, 1_130_187, 1_130_245] {
        assert_eq!(
            summoned_units(ability_program(combat, ability)),
            vec![1_130_351, 1_130_352]
        );
    }
    for linked in 1_130_351..=1_130_352 {
        assert_eq!(
            combat
                .linked_unit(starclock_combat::UnitDefinitionId::new(linked).unwrap())
                .expect("Silvermane Guard")
                .abilities(),
            &[starclock_combat::AbilityId::new(1_130_381).unwrap()]
        );
    }

    assert!(has_effect(ability_program(combat, 1_130_203), 1_135_009));
    assert!(has_effect(ability_program(combat, 1_130_244), 1_135_011));
    let lieutenant = combat
        .enemy(starclock_combat::EnemyDefinitionId::new(1_130_008).unwrap())
        .expect("Silvermane Lieutenant (Bug)");
    let entry = combat
        .program(
            lieutenant.phases()[0]
                .entry_program()
                .expect("shield-reflect entry"),
        )
        .expect("Silvermane Lieutenant phase entry");
    assert!(has_effect(entry, 1_135_011));
    let reflect = combat
        .effect(starclock_combat::EffectDefinitionId::new(1_135_011).unwrap())
        .expect("Shield Reflect");
    assert_eq!(reflect.rules().len(), 1);
    let reflect_rule = combat
        .rule(reflect.rules()[0])
        .and_then(|rule| rule.runtime())
        .expect("Shield Reflect rule");
    let counter = combat
        .program(reflect_rule.triggers()[0].program)
        .expect("Shield Reflect counter program");
    assert!(counter.steps().iter().any(|step| matches!(
        step,
        starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::Damage {
                class: starclock_combat::formula::model::DamageClass::Additional,
                element: starclock_combat::formula::model::CombatElement::Physical,
                ..
            }
        )
    )));

    assert!(has_effect(ability_program(combat, 1_130_284), 1_135_015));
    assert!(has_effect(ability_program(combat, 1_130_285), 1_135_014));
    assert!(has_effect(ability_program(combat, 1_130_325), 1_135_016));
    assert!(has_effect(ability_program(combat, 1_130_326), 1_135_017));
}

fn ability_program(
    catalog: &starclock_combat::catalog::CombatCatalog,
    ability: u32,
) -> &starclock_combat::catalog::definition::ProgramDefinition {
    catalog
        .ability(starclock_combat::AbilityId::new(ability).unwrap())
        .expect("authored S16 ability")
        .programs()
        .first()
        .and_then(|binding| catalog.program(binding.program()))
        .expect("authored S16 program")
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
