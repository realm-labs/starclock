use super::*;

#[test]
fn blaze_out_of_space_uses_exact_curve_and_authored_combustion_cycle() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .unwrap();
    let variant_key = "enemy.blaze-out-of-space.elite.variant.01";
    let enemy = materialized
        .enemies()
        .iter()
        .find(|enemy| enemy.stable_key() == variant_key)
        .expect("S04 enemy materialization");
    assert_eq!(enemy.definition_match(), EnemyDefinitionMatch::Exact);

    let expected = [
        (8, 1_520, 25_000_000, 280_000_000, 120_000_000, 0, 200_000),
        (14, 2_117, 37_000_000, 340_000_000, 120_000_000, 0, 200_000),
        (
            44,
            13_551,
            187_000_000,
            640_000_000,
            120_000_000,
            0,
            200_000,
        ),
        (
            50,
            20_590,
            234_000_000,
            700_000_000,
            120_000_000,
            0,
            200_000,
        ),
        (
            66,
            72_616,
            397_000_000,
            860_000_000,
            132_000_000,
            128_000,
            264_000,
        ),
        (
            75,
            112_994,
            494_000_000,
            950_000_000,
            132_000_000,
            200_000,
            300_000,
        ),
        (
            84,
            167_044,
            597_000_000,
            1_040_000_000,
            144_000_000,
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
            .unwrap_or_else(|| panic!("level-{level} binding"));
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
                CombatElement::Physical,
                CombatElement::Ice,
                CombatElement::Quantum,
            ]
        );
        assert_eq!(combatant.toughness_layers().len(), 1);
        assert_eq!(combatant.toughness_layers()[0].maximum().get(), 300);
    }

    let combat_catalog = materialized.combat_catalog();
    let blaze = combat_catalog
        .enemy(enemy.combat_enemy())
        .expect("authored enemy");
    let encounter_member = materialized
        .overlay()
        .binding(
            starclock_mode_universe::id::EncounterMemberId::new(103).expect("encounter member 103"),
        )
        .expect("encounter group 11901 member");
    let level_17 = encounter_member
        .preparation()
        .variants()
        .iter()
        .find(|variant| variant.techniques().is_empty())
        .expect("normal engagement")
        .battle_spec()
        .participants()
        .iter()
        .find(|participant| {
            participant.side() == TeamSide::Enemy && participant.combatant().form() == blaze.unit()
        })
        .expect("level-17 Blaze participant")
        .combatant();
    assert_eq!(level_17.maximum_hp().get(), 2_466);
    assert_eq!(level_17.base_attack().scaled(), 45_000_000);
    assert_eq!(level_17.base_defense().scaled(), 370_000_000);
    assert_eq!(level_17.speed().scaled(), 120_000_000);
    assert_eq!(level_17.base_effect_hit_rate().scaled(), 0);
    assert_eq!(level_17.base_effect_resistance().scaled(), 200_000);

    assert_eq!(blaze.phases().len(), 1);
    assert_eq!(
        blaze.abilities(),
        &[
            starclock_combat::AbilityId::new(1_010_101).unwrap(),
            starclock_combat::AbilityId::new(1_010_102).unwrap(),
            starclock_combat::AbilityId::new(1_010_103).unwrap(),
            starclock_combat::AbilityId::new(1_010_104).unwrap(),
        ]
    );
    let graph = combat_catalog
        .ai_graph(blaze.phases()[0].ai_graph())
        .expect("Blaze AI");
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
            vec![1_010_101],
            vec![1_010_102],
            vec![1_010_101, 1_010_103],
            vec![1_010_101, 1_010_104],
            vec![1_010_101, 1_010_103],
            vec![1_010_102],
        ]
    );

    let rain = combat_catalog
        .ability(starclock_combat::AbilityId::new(1_010_103).unwrap())
        .expect("Rain of Purifying Flames");
    let rain_program = rain
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("Rain program");
    assert_eq!(rain_program.steps().len(), 5);
    assert!(rain_program.steps().iter().all(|step| matches!(
        step,
        starclock_combat::rule::model::ProgramStep::ForEach { maximum: 1, .. }
    )));
    let hit_program = rain_program
        .called_programs()
        .first()
        .and_then(|program| combat_catalog.program(*program))
        .expect("Rain hit program");
    assert!(matches!(
        hit_program.steps(),
        [
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::Damage { .. }
            ),
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::ApplyEffect {
                    effect,
                    ..
                }
            )
        ] if effect.get() == 1_010_501
    ));

    let enkindle = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_010_501).unwrap())
        .expect("Enkindle")
        .runtime_template()
        .expect("Enkindle runtime");
    assert_eq!(enkindle.category(), starclock_combat::EffectCategory::Dot);
    assert_eq!(enkindle.stack_limit(), 5);
    assert_eq!(
        enkindle.tick_phase(),
        starclock_combat::EffectTickPhase::TurnStart
    );
    let spontaneous = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_010_502).unwrap())
        .expect("Spontaneous Combustion");
    assert_eq!(
        spontaneous.rules(),
        &[starclock_combat::RuleId::new(1_010_541).unwrap()]
    );
    let reset = combat_catalog
        .rule(starclock_combat::RuleId::new(1_010_541).unwrap())
        .and_then(|rule| rule.runtime())
        .expect("WeaknessBroken reset rule");
    assert_eq!(reset.triggers().len(), 1);
    assert_eq!(
        reset.triggers()[0].event_point,
        starclock_combat::rule::model::RuleEventPoint::WeaknessBroken
    );
    let molten = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_010_503).unwrap())
        .expect("Molten Fusion")
        .runtime_template()
        .expect("Molten Fusion runtime");
    assert_eq!(molten.stack_limit(), 3);
}
