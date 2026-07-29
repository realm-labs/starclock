use super::*;

#[test]
fn ordinary_enemy_batch_s15_materializes_all_frozen_variants_and_level_rows() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .expect("S15 materialization");
    let expected = [
        (
            "enemy.grunt-field-personnel.minionlv2.variant.01",
            1_120_001,
            2,
            2,
        ),
        (
            "enemy.grunt-security-personnel.minionlv2.variant.01",
            1_120_002,
            2,
            2,
        ),
        ("enemy.guardian-shadow.elite.variant.01", 1_120_003, 7, 6),
        (
            "enemy.imaginary-weaver.minionlv2.variant.01",
            1_120_004,
            4,
            3,
        ),
        (
            "enemy.incineration-shadewalker.minionlv2.variant.01",
            106,
            1,
            3,
        ),
        ("enemy.juvenile-sting.minionlv2.variant.01", 1_120_006, 4, 1),
        ("enemy.lesser-sting.minionlv2.variant.01", 1_120_007, 4, 1),
        ("enemy.mara-struck-soldier.minionlv2.variant.01", 107, 2, 4),
        ("enemy.mara-struck-warden.minionlv2.variant.01", 108, 3, 3),
        (
            "enemy.mask-of-no-thought.minion.variant.01",
            1_120_010,
            3,
            10,
        ),
        (
            "enemy.memory-zone-meme-allseer.minion.variant.01",
            1_120_011,
            1,
            2,
        ),
        (
            "enemy.memory-zone-meme-heartbreaker.minionlv2.variant.01",
            1_120_012,
            2,
            2,
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
            .expect("S15 enemy definition");
        assert_eq!(definition.abilities().len(), ability_count);
        assert_eq!(definition.phases().len(), 1);
        assert_eq!(
            definition.ai_graph().map(|id| id.get()),
            Some(1_120_041 + index as u32)
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
fn ordinary_enemy_batch_s15_retains_burn_division_rebirth_and_delay() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .expect("S15 materialization");
    let combat = materialized.combat_catalog();

    assert!(has_effect(ability_program(combat, 1_120_182), 1_125_004));
    assert_eq!(
        summoned_units(ability_program(combat, 1_120_203)),
        vec![1_120_351, 1_120_352]
    );
    for linked in 1_120_351..=1_120_352 {
        assert_eq!(
            combat
                .linked_unit(starclock_combat::UnitDefinitionId::new(linked).unwrap())
                .expect("Juvenile Sting")
                .abilities(),
            &[starclock_combat::AbilityId::new(1_120_381).unwrap()]
        );
    }

    let soldier = combat
        .enemy(starclock_combat::EnemyDefinitionId::new(107).unwrap())
        .expect("Mara-Struck Soldier");
    let entry = combat
        .program(soldier.phases()[0].entry_program().expect("rebirth entry"))
        .expect("Mara-Struck Soldier phase entry");
    assert!(has_effect(entry, 1_125_008));
    let rebirth = combat
        .effect(starclock_combat::EffectDefinitionId::new(1_125_008).unwrap())
        .expect("Rebirth");
    assert_eq!(rebirth.rules().len(), 1);
    let rebirth_rule = combat
        .rule(rebirth.rules()[0])
        .and_then(|rule| rule.runtime())
        .expect("Rebirth rule");
    let trigger = combat
        .program(rebirth_rule.triggers()[0].program)
        .expect("Rebirth trigger");
    assert!(trigger.steps().iter().any(|step| matches!(
        step,
        starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::ChangePresence { .. }
        )
    )));
    assert!(trigger.steps().iter().any(|step| matches!(
        step,
        starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::Heal { .. }
        )
    )));

    assert!(
        ability_program(combat, 1_120_302)
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
        .expect("authored S15 ability")
        .programs()
        .first()
        .and_then(|binding| catalog.program(binding.program()))
        .expect("authored S15 program")
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
