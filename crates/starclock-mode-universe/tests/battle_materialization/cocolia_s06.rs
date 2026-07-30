use super::*;

#[test]
fn cocolia_complete_materializes_world_six_and_three_phase_control_cycle() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .unwrap();
    let variant_key = "enemy.cocolia-complete.littleboss.variant.01";
    let enemy = materialized
        .enemies()
        .iter()
        .find(|enemy| enemy.stable_key() == variant_key)
        .expect("S06 enemy materialization");
    assert_eq!(enemy.definition_match(), EnemyDefinitionMatch::Exact);

    let expected = [
        (
            55,
            27_850,
            286_000_000,
            750_000_000,
            144_000_000,
            40_000,
            320_000,
        ),
        (
            72,
            78_560,
            459_000_000,
            920_000_000,
            158_400_000,
            176_000,
            388_000,
        ),
        (
            81,
            115_683,
            563_000_000,
            1_010_000_000,
            172_800_000,
            248_000,
            400_000,
        ),
        (
            90,
            175_982,
            663_000_000,
            1_100_000_000,
            190_080_000,
            320_000,
            400_000,
        ),
    ];
    for (level, hp, atk, def, spd, effect_hit_rate, effect_resistance) in expected {
        let spec = materialized
            .difficulty_specs()
            .iter()
            .find(|spec| {
                spec.enemy_variant_key() == variant_key && usize::from(spec.level().get()) == level
            })
            .unwrap_or_else(|| panic!("World 6 level-{level} binding"));
        let combatant = spec
            .battle_spec()
            .participants()
            .iter()
            .find(|participant| participant.side() == TeamSide::Enemy)
            .expect("enemy participant")
            .combatant();
        assert_eq!(combatant.maximum_hp().get(), hp);
        assert_eq!(combatant.base_attack().scaled(), atk);
        assert_eq!(combatant.base_defense().scaled(), def);
        assert_eq!(combatant.speed().scaled(), spd);
        assert_eq!(combatant.base_effect_hit_rate().scaled(), effect_hit_rate);
        assert_eq!(
            combatant.base_effect_resistance().scaled(),
            effect_resistance
        );
        assert_eq!(
            combatant.weaknesses(),
            &[
                CombatElement::Fire,
                CombatElement::Lightning,
                CombatElement::Quantum,
            ]
        );
        assert_eq!(combatant.toughness_layers()[0].maximum().get(), 120);
    }

    let combat_catalog = materialized.combat_catalog();
    let cocolia = combat_catalog
        .enemy(enemy.combat_enemy())
        .expect("authored Cocolia enemy");
    assert_eq!(cocolia.phases().len(), 3);
    let phase_abilities = cocolia
        .phases()
        .iter()
        .map(|phase| {
            combat_catalog
                .ai_graph(phase.ai_graph())
                .expect("phase AI")
                .states()
                .iter()
                .map(|state| state.candidates()[0].ability().get())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        phase_abilities,
        vec![
            vec![1_030_101, 1_030_102, 1_030_104],
            vec![
                1_030_108, 1_030_101, 1_030_104, 1_030_106, 1_030_105, 1_030_103,
            ],
            vec![
                1_030_109, 1_030_102, 1_030_104, 1_030_106, 1_030_105, 1_030_103,
            ],
        ]
    );

    let reinforced_omen = combat_catalog
        .ability(starclock_combat::AbilityId::new(1_030_108).unwrap())
        .expect("reinforced Omen");
    let reinforced_program = reinforced_omen
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("reinforced Omen program");
    let summons = reinforced_program
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
        .collect::<Vec<_>>();
    assert_eq!(summons, vec![1_030_201, 1_030_202, 1_030_203]);
    assert_eq!(
        combat_catalog
            .linked_unit(starclock_combat::UnitDefinitionId::new(1_030_203).unwrap())
            .expect("Bronya summon")
            .abilities(),
        &[starclock_combat::AbilityId::new(1_030_122).unwrap()]
    );

    let freeze_definition = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_030_501).unwrap())
        .expect("Cocolia Freeze");
    assert_eq!(freeze_definition.rules().len(), 2);
    let freeze = freeze_definition
        .runtime_template()
        .expect("Freeze runtime");
    assert_eq!(
        freeze.controlled_actions(),
        &[starclock_combat::ControlledAction::NormalAction]
    );
    assert_eq!(
        freeze
            .resolve(Some(1), starclock_combat::Scalar::from_scaled(1_125_000))
            .expect("damaging Freeze")
            .dot()
            .expect("Freeze delayed damage")
            .element(),
        CombatElement::Ice
    );

    let reverberating = combat_catalog
        .ability(starclock_combat::AbilityId::new(1_030_106).unwrap())
        .expect("Reverberating Ice");
    let reverberating_program = reverberating
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("Reverberating Ice program");
    assert!(
        reverberating_program
            .steps()
            .iter()
            .any(|step| matches!(step, starclock_combat::rule::model::ProgramStep::If { .. }))
    );

    let wrath = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_030_502).unwrap())
        .expect("Wrath charging effect");
    assert_eq!(wrath.modifiers().len(), 1);
}
