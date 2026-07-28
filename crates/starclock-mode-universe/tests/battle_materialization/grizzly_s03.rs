use super::*;

#[test]
fn automaton_grizzly_complete_uses_world_two_stats_and_authored_mechanics() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .unwrap();
    let variant_key = "enemy.automaton-grizzly-complete.elite.variant.01";
    let enemy = materialized
        .enemies()
        .iter()
        .find(|enemy| enemy.stable_key() == variant_key)
        .expect("S03 enemy materialization");
    assert_eq!(enemy.definition_match(), EnemyDefinitionMatch::Exact);

    let spec = materialized
        .difficulty_specs()
        .iter()
        .find(|spec| spec.enemy_variant_key() == variant_key && spec.level().get() == 27)
        .expect("World 2 level-27 binding");
    let combatant = spec
        .battle_spec()
        .participants()
        .iter()
        .find(|participant| participant.side() == TeamSide::Enemy)
        .expect("enemy participant")
        .combatant();
    assert_eq!(combatant.maximum_hp().get(), 4_683);
    assert_eq!(combatant.base_attack().scaled(), 84_000_000);
    assert_eq!(combatant.base_defense().scaled(), 470_000_000);
    assert_eq!(combatant.speed().scaled(), 144_000_000);
    assert_eq!(combatant.base_effect_hit_rate().scaled(), 0);
    assert_eq!(combatant.base_effect_resistance().scaled(), 200_000);
    assert_eq!(
        combatant.weaknesses(),
        &[
            CombatElement::Fire,
            CombatElement::Ice,
            CombatElement::Lightning,
        ]
    );
    assert_eq!(combatant.toughness_layers().len(), 1);
    assert_eq!(combatant.toughness_layers()[0].maximum().get(), 480);

    let combat_catalog = materialized.combat_catalog();
    let grizzly = combat_catalog
        .enemy(enemy.combat_enemy())
        .expect("authored enemy");
    assert_eq!(grizzly.phases().len(), 2);
    let phase_ability_ids = grizzly
        .phases()
        .iter()
        .map(|phase| {
            combat_catalog
                .ai_graph(phase.ai_graph())
                .expect("phase AI")
                .states()
                .iter()
                .map(|state| {
                    state
                        .candidates()
                        .iter()
                        .map(|candidate| candidate.ability().get())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        phase_ability_ids,
        vec![
            vec![
                vec![1_000_102],
                vec![1_000_103],
                vec![1_000_104],
                vec![1_000_105],
                vec![1_000_101],
            ],
            vec![
                vec![1_000_101, 1_000_102],
                vec![1_000_103],
                vec![1_000_109],
                vec![1_000_105],
                vec![1_000_101],
            ],
        ]
    );

    let purge = combat_catalog
        .ability(starclock_combat::AbilityId::new(1_000_101).unwrap())
        .expect("Purge Order");
    let purge_program = purge
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("Purge Order program");
    assert_eq!(purge_program.steps().len(), 3);
    assert!(purge_program.steps().iter().any(|step| matches!(
        step,
        starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::Damage { .. }
        )
    )));

    let detonation = combat_catalog
        .ability(starclock_combat::AbilityId::new(1_000_103).unwrap())
        .expect("Detonation Order");
    let detonation_program = detonation
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("Detonation Order program");
    assert!(detonation_program.steps().iter().any(|step| matches!(
        step,
        starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::Summon {
                unit_definition,
                ..
            }
        ) if unit_definition.get() == 1_000_201
    )));
    let spider = combat_catalog
        .linked_unit(starclock_combat::UnitDefinitionId::new(1_000_201).unwrap())
        .expect("linked Automaton Spider");
    assert_eq!(
        spider.abilities(),
        &[starclock_combat::AbilityId::new(1_000_110).unwrap()]
    );

    let taunt = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_000_501).unwrap())
        .expect("Enrage taunt")
        .runtime_template()
        .expect("taunt runtime");
    assert_eq!(taunt.stack_limit(), 1);
    assert_eq!(
        taunt.forced_normal_action(),
        Some(starclock_combat::ForcedNormalAction::BasicAttackApplier)
    );
    let obliteration = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_000_503).unwrap())
        .expect("Obliteration effect")
        .runtime_template()
        .expect("Obliteration runtime");
    assert_eq!(obliteration.stack_limit(), 100);
}
