use super::*;

#[test]
fn something_unto_death_materializes_world_9_capture_and_tomb_lifecycle() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .unwrap();
    let variant_key = "enemy.memory-zone-meme-something-unto-death-complete.littleboss.variant.01";
    let enemy = materialized
        .enemies()
        .iter()
        .find(|enemy| enemy.stable_key() == variant_key)
        .expect("S09 enemy materialization");
    assert_eq!(enemy.definition_match(), EnemyDefinitionMatch::Exact);

    let expected = [
        (
            56,
            41_422,
            297_000_000,
            760_000_000,
            144_000_000,
            48_000,
            324_000,
        ),
        (
            72,
            108_021,
            459_000_000,
            920_000_000,
            158_400_000,
            176_000,
            388_000,
        ),
        (
            81,
            159_064,
            563_000_000,
            1_010_000_000,
            172_800_000,
            248_000,
            400_000,
        ),
        (
            90,
            241_975,
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
            .unwrap_or_else(|| panic!("level-{level} World 9 binding"));
        let combatant = spec
            .battle_spec()
            .participants()
            .iter()
            .find(|participant| participant.side() == TeamSide::Enemy)
            .expect("boss participant")
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
                CombatElement::Wind,
                CombatElement::Imaginary,
            ]
        );
        assert_eq!(combatant.toughness_layers().len(), 1);
        assert_eq!(combatant.toughness_layers()[0].maximum().get(), 720);
    }

    let combat_catalog = materialized.combat_catalog();
    let boss = combat_catalog
        .enemy(enemy.combat_enemy())
        .expect("Something Unto Death enemy");
    assert_eq!(boss.phases().len(), 3);
    assert_eq!(
        boss.abilities(),
        &[
            starclock_combat::AbilityId::new(1_060_101).unwrap(),
            starclock_combat::AbilityId::new(1_060_102).unwrap(),
            starclock_combat::AbilityId::new(1_060_103).unwrap(),
            starclock_combat::AbilityId::new(1_060_104).unwrap(),
            starclock_combat::AbilityId::new(1_060_105).unwrap(),
            starclock_combat::AbilityId::new(1_060_106).unwrap(),
            starclock_combat::AbilityId::new(1_060_107).unwrap(),
            starclock_combat::AbilityId::new(1_060_108).unwrap(),
            starclock_combat::AbilityId::new(1_060_109).unwrap(),
        ]
    );

    let funereal = combat_catalog
        .ability(starclock_combat::AbilityId::new(1_060_101).unwrap())
        .expect("Funereal Kiss");
    let funereal_program = funereal
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("Funereal Kiss program");
    assert!(matches!(
        funereal_program.steps(),
        [starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::Damage {
                element: CombatElement::Physical,
                ..
            }
        )]
    ));

    let losing = combat_catalog
        .ability(starclock_combat::AbilityId::new(1_060_105).unwrap())
        .expect("Losing Eventide Light");
    let losing_program = losing
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("formation-linked capture program");
    assert_eq!(losing_program.called_programs().len(), 4);
    assert_eq!(
        losing_program
            .steps()
            .iter()
            .filter(|step| matches!(step, starclock_combat::rule::model::ProgramStep::If { .. }))
            .count(),
        4
    );

    let nightfall = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_060_504).unwrap())
        .expect("Nightfall");
    assert_eq!(
        nightfall.rules(),
        &[
            starclock_combat::RuleId::new(1_060_543).unwrap(),
            starclock_combat::RuleId::new(1_060_544).unwrap(),
        ]
    );
    assert!(
        nightfall
            .runtime_template()
            .expect("Nightfall runtime")
            .prevents_toughness_reduction()
    );

    let dream = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_060_505).unwrap())
        .and_then(|effect| effect.runtime_template())
        .expect("Morbid Dream runtime");
    assert_eq!(
        dream.controlled_actions(),
        &[starclock_combat::ControlledAction::NormalAction]
    );

    let tomb_bars = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_060_506).unwrap())
        .expect("Sombrous Sepulcher bars");
    assert_eq!(
        tomb_bars.rules(),
        &[starclock_combat::RuleId::new(1_060_545).unwrap()]
    );
    for index in 0_u32..4 {
        let linked_id = 1_060_201 + index;
        let tomb = combat_catalog
            .linked_unit(starclock_combat::UnitDefinitionId::new(linked_id).unwrap())
            .expect("formation-linked Sombrous Sepulcher");
        assert_eq!(
            tomb.abilities(),
            &[starclock_combat::AbilityId::new(1_060_108).unwrap()]
        );
        let marker = combat_catalog
            .effect(starclock_combat::EffectDefinitionId::new(1_060_510 + index).unwrap())
            .expect("formation release marker");
        assert_eq!(
            marker.rules(),
            &[starclock_combat::RuleId::new(1_060_546 + index).unwrap()]
        );
    }
}
