use super::*;

#[test]
fn ordinary_enemy_batch_s18_materializes_all_frozen_variants_and_level_rows() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .expect("S18 materialization");
    let expected = [
        ("enemy.voidranger-trampler.elite.variant.01", 111, 5, 8),
        ("enemy.windspawn.minion.variant.01", 1_150_002, 1, 4),
        ("enemy.wraith-warden.minionlv2.variant.01", 1_150_003, 1, 1),
    ];
    let core = catalog.simulation_catalog();
    for (index, (stable_key, raw_id, ability_count, stat_count)) in expected.into_iter().enumerate()
    {
        let enemy = materialized
            .enemies()
            .iter()
            .find(|enemy| enemy.stable_key() == stable_key)
            .unwrap_or_else(|| panic!("{stable_key} materialization"));
        assert_eq!(enemy.definition_match(), EnemyDefinitionMatch::Exact);
        assert_eq!(enemy.source_enemy().map(|id| id.get()), Some(raw_id));
        let definition = core
            .enemy(starclock_combat::EnemyDefinitionId::new(raw_id).unwrap())
            .expect("S18 enemy definition");
        assert_eq!(definition.abilities().len(), ability_count);
        assert_eq!(definition.phases().len(), 1);
        assert_eq!(
            definition.ai_graph().map(|id| id.get()),
            Some(1_150_041 + index as u32)
        );
        assert_eq!(
            (1_u8..=90)
                .filter(|level| {
                    core.enemy_runtime_stat(
                        starclock_combat::EnemyDefinitionId::new(raw_id).unwrap(),
                        UnitLevel::new(*level).unwrap(),
                        "standard-universe-v1",
                    )
                    .is_some()
                })
                .count(),
            stat_count
        );
    }
}

#[test]
fn ordinary_enemy_batch_s18_retains_trampler_windspawn_and_warden_boundaries() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .expect("S18 materialization");
    let combat = materialized.combat_catalog();

    assert!(has_effect(ability_program(combat, 1_150_105), 1_155_001));
    assert!(has_effect(ability_program(combat, 1_150_106), 1_155_002));
    assert!(has_effect(ability_program(combat, 1_150_122), 1_155_003));
    assert_eq!(
        damage_ratio(ability_program(combat, 1_150_106)),
        Some(6_000_000)
    );
    assert_eq!(
        damage_ratio(ability_program(combat, 1_150_122)),
        Some(2_500_000)
    );
    assert_eq!(
        damage_ratio(ability_program(combat, 1_150_142)),
        Some(2_500_000)
    );

    for effect_id in 1_155_001..=1_155_003 {
        let effect = combat
            .effect(starclock_combat::EffectDefinitionId::new(effect_id).unwrap())
            .expect("S18 effect");
        assert!(effect.rules().is_empty());
        let runtime = effect.runtime_template().expect("S18 effect runtime");
        let resolved = runtime
            .resolve(Some(2), starclock_combat::Scalar::ONE)
            .expect("S18 resolved effect");
        assert_eq!(resolved.duration(), Some(2));
        if effect_id == 1_155_003 {
            assert_eq!(
                resolved.dot().expect("Wind Shear DoT").element(),
                CombatElement::Wind
            );
        }
    }
}

fn ability_program(
    catalog: &starclock_combat::catalog::CombatCatalog,
    ability: u32,
) -> &starclock_combat::catalog::definition::ProgramDefinition {
    catalog
        .ability(starclock_combat::AbilityId::new(ability).unwrap())
        .expect("authored S18 ability")
        .programs()
        .first()
        .and_then(|binding| catalog.program(binding.program()))
        .expect("authored S18 program")
}

fn has_effect(
    program: &starclock_combat::catalog::definition::ProgramDefinition,
    effect_id: u32,
) -> bool {
    program.steps().iter().any(|step| {
        matches!(
            step,
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::ApplyEffect {
                    effect,
                    ..
                }
            ) if effect.get() == effect_id
        )
    })
}

fn damage_ratio(program: &starclock_combat::catalog::definition::ProgramDefinition) -> Option<i64> {
    program.steps().iter().find_map(|step| match step {
        starclock_combat::rule::model::ProgramStep::Operation(
            starclock_combat::rule::model::RuleOperationTemplate::Damage {
                amount: starclock_combat::rule::model::ValueExpr::Multiply { rhs, .. },
                ..
            },
        ) => match rhs.as_ref() {
            starclock_combat::rule::model::ValueExpr::Literal(
                starclock_combat::rule::model::RuleValue::Scalar(value),
            ) => Some(value.scaled()),
            _ => None,
        },
        _ => None,
    })
}
