use super::*;

#[test]
fn stellaron_hunter_kafka_materializes_world_5_control_and_shock_cycle() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .unwrap();
    let variant_key = "enemy.stellaron-hunter-kafka-complete.littleboss.variant.01";
    let enemy = materialized
        .enemies()
        .iter()
        .find(|enemy| enemy.stable_key() == variant_key)
        .expect("S10 enemy materialization");
    assert_eq!(enemy.definition_match(), EnemyDefinitionMatch::Exact);

    let expected = [
        (
            50,
            18_531,
            234_000_000,
            700_000_000,
            119_000_000,
            0,
            200_000,
        ),
        (
            72,
            88_380,
            459_000_000,
            920_000_000,
            130_900_000,
            176_000,
            288_000,
        ),
        (
            81,
            130_143,
            563_000_000,
            1_010_000_000,
            142_800_000,
            248_000,
            300_000,
        ),
        (
            90,
            197_980,
            663_000_000,
            1_100_000_000,
            157_080_000,
            320_000,
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
            .unwrap_or_else(|| panic!("level-{level} World 5 binding"));
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
                CombatElement::Physical,
                CombatElement::Wind,
                CombatElement::Imaginary,
            ]
        );
        assert_eq!(combatant.toughness_layers().len(), 1);
        assert_eq!(combatant.toughness_layers()[0].maximum().get(), 450);
    }

    let combat_catalog = materialized.combat_catalog();
    let kafka = combat_catalog
        .enemy(enemy.combat_enemy())
        .expect("Stellaron Hunter Kafka enemy");
    assert_eq!(kafka.phases().len(), 3);
    assert_eq!(
        kafka.abilities(),
        &[
            starclock_combat::AbilityId::new(1_070_101).unwrap(),
            starclock_combat::AbilityId::new(1_070_102).unwrap(),
            starclock_combat::AbilityId::new(1_070_103).unwrap(),
            starclock_combat::AbilityId::new(1_070_104).unwrap(),
            starclock_combat::AbilityId::new(1_070_105).unwrap(),
            starclock_combat::AbilityId::new(1_070_106).unwrap(),
            starclock_combat::AbilityId::new(1_070_107).unwrap(),
            starclock_combat::AbilityId::new(1_070_108).unwrap(),
            starclock_combat::AbilityId::new(1_070_109).unwrap(),
            starclock_combat::AbilityId::new(1_070_110).unwrap(),
        ]
    );

    let midnight = combat_catalog
        .ability(starclock_combat::AbilityId::new(1_070_101).unwrap())
        .expect("Midnight Tumult");
    let midnight_program = midnight
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("Midnight Tumult program");
    assert_eq!(midnight_program.steps().len(), 4);
    assert!(matches!(
        midnight_program.steps(),
        [
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::Damage {
                    element: CombatElement::Lightning,
                    ..
                }
            ),
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::DetonateDot {
                    required_tag: None,
                    ..
                }
            ),
            ..
        ]
    ));

    let psychological = combat_catalog
        .ability(starclock_combat::AbilityId::new(1_070_105).unwrap())
        .expect("Psychological Suggestion");
    let psychological_program = psychological
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("Psychological Suggestion program");
    assert!(matches!(
        psychological_program.steps(),
        [
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::ApplyEffect {
                    effect,
                    ..
                }
            ),
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::QueueAction {
                    ability,
                    forced_use: true,
                    ..
                }
            )
        ] if effect.get() == 1_070_503 && ability.get() == 1_070_110
    ));

    let shock = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_070_501).unwrap())
        .and_then(|effect| effect.runtime_template())
        .expect("Kafka Shock runtime");
    let resolved_shock = shock
        .resolve(Some(2), starclock_combat::Scalar::ONE, None)
        .expect("two-turn Kafka Shock");
    assert_eq!(resolved_shock.duration(), Some(2));
    assert_eq!(
        resolved_shock.dot().expect("Shock DoT").element(),
        CombatElement::Lightning
    );

    let dominated = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_070_502).unwrap())
        .and_then(|effect| effect.runtime_template())
        .expect("Dominated runtime");
    assert_eq!(
        dominated.forced_normal_action(),
        Some(starclock_combat::ForcedNormalAction::BasicAttackRandomAlly)
    );

    let cruelty = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_070_504).unwrap())
        .expect("Cruelty listener");
    assert_eq!(
        cruelty.rules(),
        &[starclock_combat::RuleId::new(1_070_541).unwrap()]
    );
    let cruelty_rule = combat_catalog
        .rule(starclock_combat::RuleId::new(1_070_541).unwrap())
        .and_then(|rule| rule.runtime())
        .expect("Cruelty runtime rule");
    assert_eq!(cruelty_rule.triggers().len(), 1);
    assert_eq!(
        cruelty_rule.triggers()[0].event_point,
        starclock_combat::rule::model::RuleEventPoint::DamageApplied
    );
}
