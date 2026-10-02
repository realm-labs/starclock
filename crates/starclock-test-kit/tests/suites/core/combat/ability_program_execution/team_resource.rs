//! Hit-plan and Rule IR keyed resources share checked event arithmetic.
use super::{battle_spec, catalog, id, start_and_use};
use starclock_combat::{
    Battle, BattleEventKind, BattleSeed, BattleSpec, ConcedePolicy, KeyedTeamResourceSpec,
    ResourceEventData, Rounding, Scalar, SourceDefinitionId, TeamResourceSpec,
    TeamResourceWavePolicy, TeamSide,
    catalog::{
        action::{AbilityProgramBinding, AbilityProgramTiming},
        builder::CombatCatalogBuilder,
        definition::ProgramDefinition,
    },
    rule::model::{
        ProgramStep, ResourceUpdateKind, RuleOperationTemplate, RuleResourceKind, RuleValue,
        ValueExpr,
    },
};

fn resource_battle(program: ProgramDefinition) -> Battle {
    let base = battle_spec(false, false, false);
    let resources = TeamResourceSpec::new(0, 5)
        .unwrap()
        .with_keyed(vec![
            KeyedTeamResourceSpec::new(
                SourceDefinitionId::new(90).unwrap(),
                1,
                5,
                TeamResourceWavePolicy::Persist,
            )
            .unwrap()
            .with_stable_key("shared.punchline")
            .unwrap(),
        ])
        .unwrap();
    let spec = BattleSpec::new(
        base.assembly_digest(),
        base.encounter(),
        base.participants().to_vec(),
        resources,
        base.resources(TeamSide::Enemy).clone(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let catalog = catalog(program, false, false, false, false);
    let mut builder = CombatCatalogBuilder::from_catalog(&catalog, [0x43; 32]);
    // Execute once at entry, not once per hit of the parent two-hit fixture.
    assert!(
        builder.replace_ability(catalog.ability(id(1)).unwrap().clone().with_programs(vec![
            AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, id(1)).unwrap()
        ],))
    );
    Battle::create(builder.build().unwrap(), spec, BattleSeed::new([0x47; 32])).unwrap()
}

#[test]
fn ability_program_team_resource_preserves_gain_overflow_and_set_attempted_values() {
    for (update, amount, expected) in [
        (ResourceUpdateKind::Gain, 0, (0, 0, 1, 1, 0)),
        (ResourceUpdateKind::Gain, 2, (2, 2, 1, 3, 0)),
        (ResourceUpdateKind::Gain, 4, (4, 4, 1, 5, 0)),
        (ResourceUpdateKind::Gain, 6, (6, 4, 1, 5, 2)),
        (ResourceUpdateKind::Gain, 65535, (65535, 4, 1, 5, 65531)),
        (ResourceUpdateKind::Spend, 1, (1, 1, 1, 0, 0)),
        (ResourceUpdateKind::Reserve, 1, (1, 1, 1, 0, 0)),
        (ResourceUpdateKind::Set, 1, (1, 0, 1, 1, 0)),
        (ResourceUpdateKind::Set, 5, (5, 4, 1, 5, 0)),
    ] {
        let run = || {
            let program = ProgramDefinition::new(id(1), vec![], vec![id(4)], vec![], vec![])
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::ModifyResource {
                        selector: id(4),
                        resource: RuleResourceKind::Team("shared.punchline".into()),
                        update,
                        amount: ValueExpr::Literal(RuleValue::Scalar(
                            Scalar::checked_from_integer(amount).unwrap(),
                        )),
                        scales_with_regeneration: false,
                        rounding: Rounding::Floor,
                    },
                )]);
            let mut battle = resource_battle(program);
            let result = start_and_use(&mut battle).unwrap();
            assert!(result.fault().is_none(), "{:?}", result.fault());
            let observed = result
                .events()
                .iter()
                .filter_map(|event| match event.kind() {
                    BattleEventKind::Resource(ResourceEventData::TeamResource {
                        resource,
                        attempted,
                        effective,
                        before,
                        after,
                        overflow,
                        ..
                    }) if resource.get() == 90 => {
                        Some((*attempted, *effective, *before, *after, *overflow))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(observed, [expected]);
            assert_eq!(
                battle
                    .view()
                    .team(TeamSide::Player)
                    .keyed_resource(SourceDefinitionId::new(90).unwrap())
                    .unwrap()
                    .0,
                expected.3
            );
            (
                result.events().to_vec(),
                battle.state_hash(),
                battle.view().rng_draw_count(),
            )
        };
        assert_eq!(run(), run());
    }
}

#[test]
fn ability_program_team_resource_invalid_amount_or_update_rolls_back() {
    for (update, amount) in [
        (ResourceUpdateKind::Gain, 65536),
        (ResourceUpdateKind::Spend, 2),
        (ResourceUpdateKind::Reserve, 2),
        (ResourceUpdateKind::Set, 6),
    ] {
        let program = ProgramDefinition::new(id(1), vec![], vec![id(4)], vec![], vec![])
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ModifyResource {
                    selector: id(4),
                    resource: RuleResourceKind::Team("shared.punchline".into()),
                    update,
                    amount: ValueExpr::Literal(RuleValue::Scalar(
                        Scalar::checked_from_integer(amount).unwrap(),
                    )),
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                },
            )]);
        let mut battle = resource_battle(program);
        let result = start_and_use(&mut battle).unwrap();
        assert!(result.fault().is_some());
        assert_eq!(
            battle
                .view()
                .team(TeamSide::Player)
                .keyed_resource(SourceDefinitionId::new(90).unwrap())
                .unwrap()
                .0,
            1
        );
        assert!(!result.events().iter().any(|event| matches!(
            event.kind(),
            BattleEventKind::Resource(ResourceEventData::TeamResource { .. })
        )));
        assert_eq!(battle.view().rng_draw_count(), 0);
    }
}
