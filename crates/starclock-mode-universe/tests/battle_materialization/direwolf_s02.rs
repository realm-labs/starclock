use super::*;

#[test]
fn automaton_direwolf_complete_uses_world_two_stats_and_authored_mechanics() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .unwrap();
    let variant_key = "enemy.automaton-direwolf-complete.elite.variant.01";
    let enemy = materialized
        .enemies()
        .iter()
        .find(|enemy| enemy.stable_key() == variant_key)
        .expect("S02 enemy materialization");
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
    assert_eq!(combatant.maximum_hp().get(), 2_554);
    assert_eq!(combatant.base_attack().scaled(), 84_000_000);
    assert_eq!(combatant.base_defense().scaled(), 470_000_000);
    assert_eq!(combatant.speed().scaled(), 172_000_000);
    assert_eq!(combatant.base_effect_hit_rate().scaled(), 0);
    assert_eq!(combatant.base_effect_resistance().scaled(), 200_000);
    assert_eq!(
        combatant.weaknesses(),
        &[
            CombatElement::Ice,
            CombatElement::Lightning,
            CombatElement::Imaginary,
        ]
    );
    assert_eq!(combatant.toughness_layers().len(), 1);
    assert_eq!(combatant.toughness_layers()[0].maximum().get(), 300);

    let combat_catalog = materialized.combat_catalog();
    let direwolf = combat_catalog
        .enemy(enemy.combat_enemy())
        .expect("authored enemy");
    assert_eq!(direwolf.phases().len(), 3);
    let phase_ability_ids = direwolf
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
        phase_ability_ids,
        vec![
            vec![990_102, 990_101, 990_105],
            vec![990_104, 990_101, 990_105],
            vec![990_103, 990_101, 990_105],
        ]
    );

    let felling = combat_catalog
        .ability(starclock_combat::AbilityId::new(990_101).unwrap())
        .expect("Felling Order");
    let felling_program = felling
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("Felling Order program");
    assert_eq!(
        felling_program
            .steps()
            .iter()
            .filter(|step| matches!(
                step,
                starclock_combat::rule::model::ProgramStep::Operation(
                    starclock_combat::rule::model::RuleOperationTemplate::Damage { .. }
                )
            ))
            .count(),
        10
    );
    assert_eq!(
        felling_program
            .steps()
            .iter()
            .filter(|step| matches!(
                step,
                starclock_combat::rule::model::ProgramStep::Operation(
                    starclock_combat::rule::model::RuleOperationTemplate::ApplyEffect { .. }
                )
            ))
            .count(),
        10
    );
    assert!(felling_program.steps().iter().any(|step| matches!(
        step,
        starclock_combat::rule::model::ProgramStep::ForEach { maximum: 2, .. }
    )));

    let phase_three_program = direwolf.phases()[2]
        .entry_program()
        .and_then(|program| combat_catalog.program(program))
        .expect("phase-three speed program");
    assert!(phase_three_program.steps().iter().any(|step| matches!(
        step,
        starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::ApplyEffect {
                effect,
                ..
            }
        ) if effect.get() == 990_504
    )));
    assert_eq!(
        combat_catalog
            .effect(starclock_combat::EffectDefinitionId::new(990_505).unwrap())
            .expect("Targeting Order speed effect")
            .runtime_template()
            .expect("speed runtime")
            .stack_limit(),
        2
    );
}
