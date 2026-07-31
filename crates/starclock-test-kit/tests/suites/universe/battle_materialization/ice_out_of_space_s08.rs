use super::*;

#[test]
fn ice_out_of_space_materializes_exact_levels_encounter_and_freezing_point_cycle() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .unwrap();
    let variant_key = "enemy.ice-out-of-space.elite.variant.01";
    let enemy = materialized
        .enemies()
        .iter()
        .find(|enemy| enemy.stable_key() == variant_key)
        .expect("S08 enemy materialization");
    assert_eq!(enemy.definition_match(), EnemyDefinitionMatch::Exact);

    let expected = [
        (14, 2_117, 37_000_000, 340_000_000, 100_000_000, 0, 200_000),
        (
            49,
            19_417,
            226_000_000,
            690_000_000,
            100_000_000,
            0,
            200_000,
        ),
        (
            50,
            20_590,
            234_000_000,
            700_000_000,
            100_000_000,
            0,
            200_000,
        ),
        (
            66,
            72_616,
            397_000_000,
            860_000_000,
            110_000_000,
            128_000,
            264_000,
        ),
        (
            75,
            112_994,
            494_000_000,
            950_000_000,
            110_000_000,
            200_000,
            300_000,
        ),
        (
            84,
            167_044,
            597_000_000,
            1_040_000_000,
            120_000_000,
            272_000,
            300_000,
        ),
    ];
    for (level, hp, atk, def, spd, effect_hit_rate, effect_resistance) in expected {
        let spec = materialized
            .difficulty_specs()
            .iter()
            .find(|spec| {
                spec.enemy_variant_key() == variant_key && usize::from(spec.level().get()) == level
            })
            .unwrap_or_else(|| panic!("level-{level} universe binding"));
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
                CombatElement::Wind,
                CombatElement::Quantum,
            ]
        );
        assert_eq!(combatant.toughness_layers().len(), 1);
        assert_eq!(combatant.toughness_layers()[0].maximum().get(), 300);
    }

    let combat_catalog = materialized.combat_catalog();
    let ice = combat_catalog
        .enemy(enemy.combat_enemy())
        .expect("authored Ice Out of Space enemy");
    let encounter_member = materialized
        .overlay()
        .binding(
            starclock_mode_universe::id::EncounterMemberId::new(108).expect("encounter member 108"),
        )
        .expect("encounter group 19001 member");
    let encounter_spec = encounter_member
        .preparation()
        .variants()
        .iter()
        .find(|variant| variant.techniques().is_empty())
        .expect("normal engagement")
        .battle_spec();
    assert_eq!(
        encounter_spec
            .participants()
            .iter()
            .filter(|participant| participant.side() == TeamSide::Enemy)
            .count(),
        3
    );
    let level_44 = encounter_spec
        .participants()
        .iter()
        .find(|participant| {
            participant.side() == TeamSide::Enemy && participant.combatant().form() == ice.unit()
        })
        .expect("level-44 Ice Out of Space participant")
        .combatant();
    assert_eq!(level_44.maximum_hp().get(), 13_551);
    assert_eq!(level_44.base_attack().scaled(), 187_000_000);
    assert_eq!(level_44.base_defense().scaled(), 640_000_000);
    assert_eq!(level_44.speed().scaled(), 100_000_000);
    assert_eq!(level_44.base_effect_hit_rate().scaled(), 0);
    assert_eq!(level_44.base_effect_resistance().scaled(), 200_000);

    assert_eq!(ice.phases().len(), 1);
    assert_eq!(
        ice.abilities(),
        &[
            starclock_combat::AbilityId::new(1_050_101).unwrap(),
            starclock_combat::AbilityId::new(1_050_102).unwrap(),
            starclock_combat::AbilityId::new(1_050_103).unwrap(),
            starclock_combat::AbilityId::new(1_050_104).unwrap(),
        ]
    );
    let graph = combat_catalog
        .ai_graph(ice.phases()[0].ai_graph())
        .expect("Ice Out of Space AI");
    let candidate_abilities = graph
        .states()
        .iter()
        .map(|state| {
            state
                .candidates()
                .iter()
                .map(|candidate| candidate.ability().get())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        candidate_abilities,
        vec![
            vec![1_050_101],
            vec![1_050_102],
            vec![1_050_101, 1_050_103],
            vec![1_050_101, 1_050_104],
            vec![1_050_102],
        ]
    );

    let rain = combat_catalog
        .ability(starclock_combat::AbilityId::new(1_050_103).unwrap())
        .expect("Everwinter Rain");
    let rain_program = rain
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("Everwinter Rain program");
    assert!(matches!(
        rain_program.steps(),
        [
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::Damage {
                    element: CombatElement::Ice,
                    ..
                }
            ),
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::ApplyEffect {
                    effect,
                    ..
                }
            )
        ] if effect.get() == 1_050_501
    ));

    let enhanced = combat_catalog
        .ability(starclock_combat::AbilityId::new(1_050_104).unwrap())
        .expect("enhanced Chilling Lament");
    let enhanced_program = enhanced
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("enhanced Chilling Lament program");
    assert_eq!(
        enhanced_program
            .steps()
            .iter()
            .filter(|step| matches!(
                step,
                starclock_combat::rule::model::ProgramStep::Operation(
                    starclock_combat::rule::model::RuleOperationTemplate::Damage {
                        element: CombatElement::Ice,
                        ..
                    }
                )
            ))
            .count(),
        2
    );

    let freeze_definition = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_050_501).unwrap())
        .expect("Ice Out of Space Freeze");
    let freeze = freeze_definition
        .runtime_template()
        .expect("Freeze runtime");
    assert_eq!(
        freeze.controlled_actions(),
        &[starclock_combat::ControlledAction::NormalAction]
    );
    assert_eq!(
        freeze.tick_phase(),
        starclock_combat::EffectTickPhase::TurnStart
    );
    assert_eq!(
        freeze
            .resolve(Some(1), starclock_combat::Scalar::ONE)
            .expect("damaging Freeze")
            .dot()
            .expect("Freeze delayed damage")
            .element(),
        CombatElement::Ice
    );

    let freezing_point = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_050_502).unwrap())
        .expect("Freezing Point");
    assert_eq!(
        freezing_point.rules(),
        &[starclock_combat::RuleId::new(1_050_541).unwrap()]
    );
    let reset = combat_catalog
        .rule(starclock_combat::RuleId::new(1_050_541).unwrap())
        .and_then(|rule| rule.runtime())
        .expect("Weakness Break reset rule");
    assert_eq!(reset.triggers().len(), 1);
    assert_eq!(
        reset.triggers()[0].event_point,
        starclock_combat::rule::model::RuleEventPoint::WeaknessBroken
    );
}
