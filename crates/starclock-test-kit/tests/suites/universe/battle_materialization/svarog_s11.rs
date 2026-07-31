use super::*;

#[test]
fn svarog_complete_materializes_world_four_support_and_arm_cycles() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .unwrap();
    let variant_key = "enemy.svarog-complete.littleboss.variant.01";
    let enemy = materialized
        .enemies()
        .iter()
        .find(|enemy| enemy.stable_key() == variant_key)
        .expect("S11 enemy materialization");
    assert_eq!(enemy.definition_match(), EnemyDefinitionMatch::Exact);

    let expected = [
        (42, 8_403, 171_000_000, 620_000_000, 144_000_000, 0, 300_000),
        (
            67,
            57_410,
            407_000_000,
            870_000_000,
            158_400_000,
            136_000,
            368_000,
        ),
        (
            72,
            73_650,
            459_000_000,
            920_000_000,
            158_400_000,
            176_000,
            388_000,
        ),
        (
            81,
            108_452,
            563_000_000,
            1_010_000_000,
            172_800_000,
            248_000,
            400_000,
        ),
        (
            90,
            164_983,
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
            .unwrap_or_else(|| panic!("World 4 level-{level} binding"));
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
                CombatElement::Lightning,
                CombatElement::Wind,
            ]
        );
        assert_eq!(combatant.toughness_layers().len(), 1);
        assert_eq!(combatant.toughness_layers()[0].maximum().get(), 360);
    }

    let combat_catalog = materialized.combat_catalog();
    let svarog = combat_catalog
        .enemy(enemy.combat_enemy())
        .expect("Svarog Complete enemy");
    assert_eq!(svarog.phases().len(), 3);
    assert_eq!(
        svarog.abilities(),
        &[
            starclock_combat::AbilityId::new(1_080_101).unwrap(),
            starclock_combat::AbilityId::new(1_080_102).unwrap(),
            starclock_combat::AbilityId::new(1_080_103).unwrap(),
            starclock_combat::AbilityId::new(1_080_104).unwrap(),
            starclock_combat::AbilityId::new(1_080_105).unwrap(),
            starclock_combat::AbilityId::new(1_080_106).unwrap(),
            starclock_combat::AbilityId::new(1_080_107).unwrap(),
        ]
    );
    let phase_abilities = svarog
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
            vec![1_080_105, 1_080_101, 1_080_102, 1_080_104],
            vec![1_080_106, 1_080_103, 1_080_101, 1_080_102, 1_080_104,],
            vec![1_080_107, 1_080_103, 1_080_101, 1_080_102],
        ]
    );

    let emergency = ability_program(combat_catalog, 1_080_105);
    assert_eq!(
        summoned_units(emergency),
        vec![1_080_201, 1_080_202, 1_080_203, 1_080_204]
    );
    for linked_id in 1_080_201..=1_080_204 {
        let support = combat_catalog
            .linked_unit(starclock_combat::UnitDefinitionId::new(linked_id).unwrap())
            .expect("random phase-one support");
        assert_eq!(
            support.abilities(),
            &[
                starclock_combat::AbilityId::new(1_080_120).unwrap(),
                starclock_combat::AbilityId::new(1_080_121).unwrap(),
                starclock_combat::AbilityId::new(1_080_122).unwrap(),
            ]
        );
    }
    let support_cycle = ability_program(combat_catalog, 1_080_120);
    let random_role = reachable_programs(combat_catalog, support_cycle)
        .into_iter()
        .flat_map(|program| program.steps())
        .find_map(|step| match step {
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::ApplyRandomEffect {
                    effects,
                    choice_rng_purpose,
                    ..
                },
            ) => Some((effects, choice_rng_purpose)),
            _ => None,
        })
        .expect("independent random support role");
    assert_eq!(
        random_role.0.as_ref(),
        &[
            starclock_combat::EffectDefinitionId::new(1_080_506).unwrap(),
            starclock_combat::EffectDefinitionId::new(1_080_507).unwrap(),
        ]
    );
    assert_eq!(
        *random_role.1,
        starclock_combat::rng::types::DrawPurpose::BEHAVIOR_CHOICE
    );

    let tactical = ability_program(combat_catalog, 1_080_106);
    assert_eq!(summoned_units(tactical), vec![1_080_205, 1_080_206]);
    for linked_id in 1_080_205..=1_080_206 {
        assert_eq!(
            combat_catalog
                .linked_unit(starclock_combat::UnitDefinitionId::new(linked_id).unwrap())
                .expect("phase-two Direwolf")
                .abilities(),
            &[starclock_combat::AbilityId::new(1_080_123).unwrap()]
        );
    }

    let boost = ability_program(combat_catalog, 1_080_107);
    assert_eq!(summoned_units(boost), vec![1_080_207]);
    assert_eq!(
        combat_catalog
            .linked_unit(starclock_combat::UnitDefinitionId::new(1_080_207).unwrap())
            .expect("phase-three Auxiliary Robot Arm")
            .abilities(),
        &[
            starclock_combat::AbilityId::new(1_080_124).unwrap(),
            starclock_combat::AbilityId::new(1_080_125).unwrap(),
            starclock_combat::AbilityId::new(1_080_126).unwrap(),
            starclock_combat::AbilityId::new(1_080_127).unwrap(),
            starclock_combat::AbilityId::new(1_080_128).unwrap(),
        ]
    );

    let bombardment = ability_program(combat_catalog, 1_080_103);
    assert_eq!(bombardment.steps().len(), 13);
    assert_eq!(
        bombardment
            .steps()
            .iter()
            .filter(|step| matches!(
                step,
                starclock_combat::rule::model::ProgramStep::Operation(
                    starclock_combat::rule::model::RuleOperationTemplate::Damage {
                        amount,
                        element: CombatElement::Physical,
                        ..
                    }
                ) if product_ratio(amount) == Some(150_000)
            ))
            .count(),
        12
    );
    assert!(matches!(
        bombardment.steps().last(),
        Some(starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::ApplyEffect {
                effect,
                base_chance: Some(chance),
                ..
            }
        )) if effect.get() == 1_080_501 && scalar_literal(chance) == Some(1_000_000)
    ));
    let def_down = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_080_501).unwrap())
        .expect("Bombardment DEF reduction");
    assert_eq!(
        def_down.modifiers(),
        &[starclock_combat::ModifierDefinitionId::new(1_080_521).unwrap()]
    );
    let def_down_runtime = def_down.runtime_template().expect("DEF-down runtime");
    assert_eq!(def_down_runtime.stack_limit(), 100);
    assert!(matches!(
        def_down_runtime.duration_expression(),
        Some(starclock_combat::rule::model::ValueExpr::Literal(
            starclock_combat::rule::model::RuleValue::Integer(3)
        ))
    ));

    let burning = ability_program(combat_catalog, 1_080_102);
    assert!(matches!(
        burning.steps(),
        [
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::Damage { amount, .. }
            ),
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::DelayAction {
                    amount: delay,
                    ..
                }
            )
        ] if product_ratio(amount) == Some(3_000_000)
            && scalar_literal(delay) == Some(500_000)
    ));
    let power = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_080_502).unwrap())
        .expect("Power Amplification");
    assert_eq!(
        power.modifiers(),
        &[starclock_combat::ModifierDefinitionId::new(1_080_522).unwrap()]
    );
    assert_eq!(
        scalar_literal(
            &combat_catalog
                .modifier(starclock_combat::ModifierDefinitionId::new(1_080_522).unwrap())
                .expect("Power Amplification modifier")
                .value
        ),
        Some(150_000)
    );

    let restrain = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_080_503).unwrap())
        .and_then(|effect| effect.runtime_template())
        .expect("Restrained runtime");
    assert_eq!(
        restrain.controlled_actions(),
        &[starclock_combat::ControlledAction::NormalAction]
    );
    let arm_cycle = ability_program(combat_catalog, 1_080_124);
    let arm_programs = reachable_programs(combat_catalog, arm_cycle);
    assert!(arm_programs.iter().any(|program| {
        program.steps().iter().any(|step| {
            matches!(
                step,
                starclock_combat::rule::model::ProgramStep::If {
                    condition: starclock_combat::rule::model::ConditionExpr::Any(branches),
                    ..
                } if branches.iter().any(|branch| matches!(
                    branch,
                    starclock_combat::rule::model::ConditionExpr::Compare {
                        lhs,
                        operator: starclock_combat::rule::model::Comparison::Less,
                        ..
                    } if matches!(
                        lhs.as_ref(),
                        starclock_combat::rule::model::ValueExpr::QueryHp { .. }
                    )
                ))
            )
        })
    }));
    assert!(
        arm_programs
            .iter()
            .any(|program| program.steps().iter().any(|step| matches!(
                step,
                starclock_combat::rule::model::ProgramStep::Operation(
                    starclock_combat::rule::model::RuleOperationTemplate::AdvanceAction {
                        amount,
                        ..
                    }
                ) if scalar_literal(amount) == Some(1_000_000)
            )))
    );
    assert!(arm_programs.iter().any(|program| {
        matches!(
            program.steps(),
            [
                starclock_combat::rule::model::ProgramStep::Operation(
                    starclock_combat::rule::model::RuleOperationTemplate::Damage {
                        amount,
                        ..
                    }
                ),
                starclock_combat::rule::model::ProgramStep::Operation(
                    starclock_combat::rule::model::RuleOperationTemplate::Despawn { .. }
                )
            ] if product_ratio(amount) == Some(15_000_000)
        )
    }));
    assert!(arm_programs.iter().all(|program| {
        program.steps().iter().all(|step| {
            !matches!(
                step,
                starclock_combat::rule::model::ProgramStep::Operation(
                    starclock_combat::rule::model::RuleOperationTemplate::InvokeNative { .. }
                )
            )
        })
    }));
}

fn ability_program(
    catalog: &starclock_combat::catalog::CombatCatalog,
    ability: u32,
) -> &starclock_combat::catalog::definition::ProgramDefinition {
    catalog
        .ability(starclock_combat::AbilityId::new(ability).unwrap())
        .expect("authored S11 ability")
        .programs()
        .first()
        .and_then(|binding| catalog.program(binding.program()))
        .expect("authored S11 program")
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

fn reachable_programs<'a>(
    catalog: &'a starclock_combat::catalog::CombatCatalog,
    root: &'a starclock_combat::catalog::definition::ProgramDefinition,
) -> Vec<&'a starclock_combat::catalog::definition::ProgramDefinition> {
    let mut programs = vec![root];
    let mut cursor = 0;
    while cursor < programs.len() {
        for called in programs[cursor].called_programs() {
            let definition = catalog.program(*called).expect("called S11 program");
            if !programs
                .iter()
                .any(|candidate| candidate.id() == definition.id())
            {
                programs.push(definition);
            }
        }
        cursor += 1;
    }
    programs
}

fn scalar_literal(expression: &starclock_combat::rule::model::ValueExpr) -> Option<i64> {
    match expression {
        starclock_combat::rule::model::ValueExpr::Literal(
            starclock_combat::rule::model::RuleValue::Scalar(value),
        ) => Some(value.scaled()),
        _ => None,
    }
}

fn product_ratio(expression: &starclock_combat::rule::model::ValueExpr) -> Option<i64> {
    match expression {
        starclock_combat::rule::model::ValueExpr::Multiply { rhs, .. } => scalar_literal(rhs),
        _ => None,
    }
}
