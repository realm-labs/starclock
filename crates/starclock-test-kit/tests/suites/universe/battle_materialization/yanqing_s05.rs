use super::*;

#[test]
fn yanqing_complete_materializes_exact_world_eight_stats_and_sword_formation() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .unwrap();
    let variant_key = "enemy.cloud-knight-lieutenant-yanqing-complete.littleboss.variant.01";
    let enemy = materialized
        .enemies()
        .iter()
        .find(|enemy| enemy.stable_key() == variant_key)
        .expect("S05 enemy materialization");
    assert_eq!(enemy.definition_match(), EnemyDefinitionMatch::Exact);

    let expected = [
        (
            56,
            75_316,
            447_000_000,
            760_000_000,
            144_000_000,
            48_000,
            324_000,
        ),
        (
            72,
            118_422,
            561_000_000,
            920_000_000,
            158_400_000,
            176_000,
            388_000,
        ),
        (
            81,
            356_874,
            881_000_000,
            1_010_000_000,
            172_800_000,
            248_000,
            400_000,
        ),
        (
            90,
            685_410,
            876_000_000,
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
            .unwrap_or_else(|| panic!("World 8 level-{level} binding"));
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
                CombatElement::Lightning,
                CombatElement::Wind,
                CombatElement::Imaginary,
            ]
        );
        assert_eq!(combatant.toughness_layers().len(), 1);
        assert_eq!(combatant.toughness_layers()[0].maximum().get(), 120);
    }

    let combat_catalog = materialized.combat_catalog();
    let yanqing = combat_catalog
        .enemy(enemy.combat_enemy())
        .expect("authored Yanqing enemy");
    assert_eq!(yanqing.phases().len(), 3);
    let phase_abilities = yanqing
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
            vec![1_020_101, 1_020_102, 1_020_103, 1_020_104],
            vec![
                1_020_102, 1_020_101, 1_020_104, 1_020_105, 1_020_106, 1_020_103,
            ],
            vec![
                1_020_109, 1_020_101, 1_020_107, 1_020_104, 1_020_108, 1_020_103,
            ],
        ]
    );

    let swallow = combat_catalog
        .ability(starclock_combat::AbilityId::new(1_020_102).unwrap())
        .expect("Swallow Return");
    let swallow_program = swallow
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("Swallow Return program");
    let summons = swallow_program
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
    assert_eq!(summons, vec![1_020_201, 1_020_202, 1_020_204, 1_020_205]);
    assert!(swallow_program.steps().iter().any(|step| matches!(
        step,
        starclock_combat::rule::model::ProgramStep::ForEach { maximum: 4, .. }
    )));
    for linked_id in summons {
        let sword = combat_catalog
            .linked_unit(starclock_combat::UnitDefinitionId::new(linked_id).unwrap())
            .expect("linked Flying Sword");
        assert_eq!(
            sword.abilities(),
            &[starclock_combat::AbilityId::new(1_020_121).unwrap()]
        );
    }

    let core_program = swallow_program
        .called_programs()
        .iter()
        .find_map(|program| {
            let definition = combat_catalog.program(*program)?;
            definition.steps().iter().any(|step| {
                matches!(
                    step,
                    starclock_combat::rule::model::ProgramStep::Operation(
                        starclock_combat::rule::model::RuleOperationTemplate::ApplyRandomEffect {
                            effects,
                            ..
                        }
                    ) if effects.len() == 3
                )
            })
            .then_some(definition)
        })
        .expect("random Formation Core program");
    assert_eq!(core_program.called_programs().len(), 3);

    let ordeal_swallow = combat_catalog
        .ability(starclock_combat::AbilityId::new(1_020_109).unwrap())
        .expect("phase-three Swallow Return");
    let ordeal_program = ordeal_swallow
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("Ordeal Swallow Return program");
    let ordeal_timeline_selectors = ordeal_program
        .steps()
        .iter()
        .filter_map(|step| match step {
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::AdvanceAction {
                    selector,
                    ..
                },
            ) => Some(selector.get()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(ordeal_timeline_selectors, vec![1_020_419, 1_020_420]);
    let second_source_pair = combat_catalog
        .selector(starclock_combat::SelectorId::new(1_020_413).unwrap())
        .and_then(|selector| selector.rule_units())
        .expect("second source-pair selector");
    assert!(
        second_source_pair
            .predicates()
            .iter()
            .any(|predicate| matches!(
                predicate,
                starclock_combat::catalog::selector::RuleSelectorPredicate::Excludes(selector)
                    if selector.get() == 1_020_418
            ))
    );
    for selector_id in [1_020_419, 1_020_420] {
        let selector = combat_catalog
            .selector(starclock_combat::SelectorId::new(selector_id).unwrap())
            .and_then(|selector| selector.rule_units())
            .expect("Ordeal source-pair selector");
        assert!(selector.predicates().iter().any(|predicate| matches!(
            predicate,
            starclock_combat::catalog::selector::RuleSelectorPredicate::HasEffect(effect)
                if effect.get() == 1_020_506
        )));
    }

    let formation = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_020_502).unwrap())
        .expect("Sword Formation")
        .runtime_template()
        .expect("Sword Formation runtime");
    assert!(formation.prevents_toughness_reduction());
    let freeze = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_020_507).unwrap())
        .expect("Yanqing Freeze")
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
    let freeze_runtime = freeze
        .resolve(
            Some(1),
            starclock_combat::Scalar::from_scaled(1_200_000),
            None,
        )
        .expect("damaging control resolves");
    assert_eq!(
        freeze_runtime
            .dot()
            .expect("Freeze delayed Ice damage")
            .element(),
        CombatElement::Ice
    );
    let ordeal = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_020_506).unwrap())
        .expect("Ordeal");
    assert_eq!(
        ordeal.rules(),
        &[starclock_combat::RuleId::new(1_020_545).unwrap()]
    );
}
