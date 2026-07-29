use super::*;

#[test]
fn gepard_complete_materializes_world_three_encounter_and_public_phase_rotations() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .unwrap();
    let variant_key = "enemy.gepard-complete.littleboss.variant.01";
    let enemy = materialized
        .enemies()
        .iter()
        .find(|enemy| enemy.stable_key() == variant_key)
        .expect("S07 enemy materialization");
    assert_eq!(enemy.definition_match(), EnemyDefinitionMatch::Exact);

    let expected = [
        (35, 4_463, 106_000_000, 550_000_000, 144_000_000, 0, 200_000),
        (
            67,
            49_755,
            339_000_000,
            870_000_000,
            158_400_000,
            136_000,
            268_000,
        ),
        (
            72,
            63_830,
            383_000_000,
            920_000_000,
            158_400_000,
            176_000,
            288_000,
        ),
        (
            81,
            93_992,
            469_000_000,
            1_010_000_000,
            172_800_000,
            248_000,
            300_000,
        ),
        (
            90,
            142_985,
            552_000_000,
            1_100_000_000,
            190_080_000,
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
            .unwrap_or_else(|| panic!("World 3 level-{level} binding"));
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
                CombatElement::Lightning,
                CombatElement::Imaginary,
            ]
        );
        assert_eq!(combatant.toughness_layers()[0].maximum().get(), 100);
    }

    let combat_catalog = materialized.combat_catalog();
    let gepard = combat_catalog
        .enemy(enemy.combat_enemy())
        .expect("authored Gepard enemy");
    let encounter_member = materialized
        .overlay()
        .binding(
            starclock_mode_universe::id::EncounterMemberId::new(107).expect("encounter member 107"),
        )
        .expect("encounter group 13901 member");
    let level_50 = encounter_member
        .preparation()
        .variants()
        .iter()
        .find(|variant| variant.techniques().is_empty())
        .expect("normal engagement")
        .battle_spec()
        .participants()
        .iter()
        .find(|participant| {
            participant.side() == TeamSide::Enemy && participant.combatant().form() == gepard.unit()
        })
        .expect("level-50 Gepard participant")
        .combatant();
    assert_eq!(level_50.maximum_hp().get(), 13_384);
    assert_eq!(level_50.base_attack().scaled(), 195_000_000);
    assert_eq!(level_50.base_defense().scaled(), 700_000_000);
    assert_eq!(level_50.speed().scaled(), 144_000_000);
    assert_eq!(level_50.base_effect_hit_rate().scaled(), 0);
    assert_eq!(level_50.base_effect_resistance().scaled(), 200_000);

    assert_eq!(gepard.phases().len(), 3);
    let phase_abilities = gepard
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
            vec![1_040_105, 1_040_104, 1_040_108, 1_040_101],
            vec![
                1_040_107, 1_040_104, 1_040_103, 1_040_104, 1_040_112, 1_040_104, 1_040_106,
                1_040_108, 1_040_112, 1_040_113,
            ],
            vec![
                1_040_107, 1_040_104, 1_040_103, 1_040_104, 1_040_112, 1_040_104, 1_040_106,
                1_040_108, 1_040_112, 1_040_113,
            ],
        ]
    );

    let phase_summons = gepard
        .phases()
        .iter()
        .map(|phase| {
            combat_catalog
                .program(phase.entry_program().expect("phase support program"))
                .expect("phase support program definition")
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
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        phase_summons,
        vec![
            vec![1_040_201, 1_040_202],
            vec![1_040_203, 1_040_204],
            vec![1_040_205, 1_040_206],
        ]
    );

    let garrison = combat_catalog
        .ability(starclock_combat::AbilityId::new(1_040_103).unwrap())
        .expect("Garrison Aura Field");
    let garrison_program = garrison
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("Garrison program");
    assert!(matches!(
        garrison_program.steps(),
        [
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::Shield {
                    effect,
                    ..
                }
            ),
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::GrantExtraTurn { .. }
            )
        ] if effect.get() == 1_040_502
    ));
    let collective_shield = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_040_502).unwrap())
        .expect("Collective Shield")
        .runtime_template()
        .expect("Collective Shield runtime");
    assert_eq!(
        collective_shield.category(),
        starclock_combat::EffectCategory::Shield
    );

    let besiege = combat_catalog
        .ability(starclock_combat::AbilityId::new(1_040_105).unwrap())
        .expect("Besiege");
    let besiege_program = besiege
        .programs()
        .first()
        .and_then(|binding| combat_catalog.program(binding.program()))
        .expect("Besiege program");
    assert!(besiege_program.steps().iter().any(|step| matches!(
        step,
        starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::QueueAction {
                ability,
                ..
            }
        ) if ability.get() == 1_040_124
    )));

    let escalation = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(1_040_501).unwrap())
        .expect("Frigid Waterfall escalation");
    assert_eq!(escalation.modifiers().len(), 1);
    assert_eq!(
        escalation
            .runtime_template()
            .expect("escalation runtime")
            .stack_limit(),
        100
    );

    let counter = combat_catalog
        .rule(starclock_combat::RuleId::new(1_040_541).unwrap())
        .and_then(|rule| rule.runtime())
        .expect("Tit for Tat counter rule");
    assert_eq!(counter.triggers().len(), 1);
    assert_eq!(
        counter.triggers()[0].event_point,
        starclock_combat::rule::model::RuleEventPoint::DamageApplied
    );
    let counter_program = combat_catalog
        .program(counter.triggers()[0].program)
        .expect("Tit for Tat counter program");
    assert!(counter_program.steps().iter().any(|step| matches!(
        step,
        starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::QueueAction {
                ability,
                boundary,
                ..
            }
        ) if ability.get() == 1_040_102
            && *boundary == starclock_combat::catalog::action::ReactionBoundary::AfterHit
    )));
}
