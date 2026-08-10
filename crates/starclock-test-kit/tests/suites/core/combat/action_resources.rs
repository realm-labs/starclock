use crate::combat_decision::{advance_boundary_if_offered, settle_ready_boundaries};
use std::sync::Arc;

use starclock_combat::{
    ActionEventData, ActionOrigin, AssemblyDigest, Battle, BattleEventKind, BattleSeed, BattleSpec,
    CombatantSpecDigest, CombatantSpecError, Command, CommandErrorKind, ConcedePolicy,
    DecisionKind, DecisionOwner, Energy, FormationIndex, HitEventData, Hp, ParticipantSource,
    ParticipantSpec, ResolvedCombatantSpec, ResolvedDefinitionBindings, ResourceEventData, Speed,
    TeamResourceSpec, TeamSide, TurnEventData, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionResourcePolicy, CharacterResourceCost,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::{CatalogBuildErrorKind, CombatCatalogBuilder},
        definition::{
            AbilityDefinition, CharacterResourceDefinition, EncounterDefinition, EnemyDefinition,
            ProgramDefinition, SelectorDefinition, UnitDefinition,
        },
    },
};

fn definition<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

#[test]
fn player_ultimate_interrupts_an_enemy_after_its_action_and_before_turn_end() {
    let mut battle = battle();
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    advance_boundary_if_offered(&mut battle)
        .expect("full initial Energy opens the player interrupt window");

    let basic = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseAbility {
                    ability,
                    primary_target: None,
                    ..
                } if ability.get() == 1
            )
        })
        .unwrap()
        .clone();
    battle.apply(basic).unwrap();
    assert_eq!(
        battle.view().phase(),
        starclock_combat::BattlePhase::ReadyToAdvance
    );
    assert!(battle.view().action_boundary().is_some());
    advance_boundary_if_offered(&mut battle)
        .expect("the charged Ultimate remains available after the player action");

    let before_enemy = battle.view().action_boundary().unwrap();
    assert_eq!(before_enemy.turn().side(), TeamSide::Enemy);
    assert_eq!(
        battle.decision().unwrap().owner(),
        DecisionOwner::Team(TeamSide::Enemy)
    );
    advance_boundary_if_offered(&mut battle)
        .expect("the player declines the opportunity before the enemy action");

    let enemy_action = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { actor, .. } if actor.get() == 2))
        .unwrap()
        .clone();
    battle.apply(enemy_action).unwrap();
    let after_enemy = battle.view().action_boundary().unwrap();
    assert_eq!(after_enemy.turn().side(), TeamSide::Enemy);
    assert!(battle.decision().is_none());

    let ultimate = battle
        .available_ultimates()
        .into_iter()
        .find(|option| option.ability().get() == 3)
        .and_then(|option| battle.request_ultimate_command(option))
        .unwrap();
    let prepared = battle.apply(ultimate).unwrap();
    assert!(prepared.events().iter().all(|event| !matches!(
        event.kind(),
        BattleEventKind::Action(ActionEventData::Declared { .. })
    )));
    let commit = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::CommitPreparedAction { .. }))
        .unwrap()
        .clone();
    let inserted = battle.apply(commit).unwrap();
    let ultimate_declared = inserted
        .events()
        .iter()
        .find(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Action(ActionEventData::Declared {
                    origin: ActionOrigin::UltimateInterrupt,
                    ..
                })
            )
        })
        .unwrap()
        .id();
    let resumed = battle.advance().unwrap();
    let enemy_turn_ended = resumed
        .events()
        .iter()
        .find(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Turn(TurnEventData::Ended { owner, .. }) if owner.get() == 2
            )
        })
        .unwrap()
        .id();
    assert!(ultimate_declared < enemy_turn_ended);
}

fn runtime<I: TryFrom<u64>>(raw: u64) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn action(
    kind: AbilityKind,
    hits: u16,
    invalidation: TargetInvalidationPolicy,
    resources: ActionResourcePolicy,
) -> AbilityActionDefinition {
    AbilityActionDefinition::new(kind, hits, invalidation, resources).unwrap()
}

fn catalog() -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new([0x71; 32]);
    for (raw, relation, pattern) in [
        (1, TargetRelation::SelfUnit, TargetPattern::Single),
        (2, TargetRelation::Opposing, TargetPattern::Blast),
        (3, TargetRelation::Opposing, TargetPattern::All),
        (4, TargetRelation::SelfUnit, TargetPattern::Single),
    ] {
        builder.add_selector(
            SelectorDefinition::new(definition(raw))
                .with_unit_targets(UnitTargetSelector::new(relation, pattern).unwrap()),
        );
        builder.add_program(ProgramDefinition::new(
            definition(raw),
            vec![],
            vec![definition(raw)],
            vec![],
            vec![],
        ));
    }
    let zero = Energy::ZERO;
    let energy_20 = Energy::from_scaled(20_000_000).unwrap();
    let energy_30 = Energy::from_scaled(30_000_000).unwrap();
    let energy_100 = Energy::from_scaled(100_000_000).unwrap();
    builder.add_ability(
        AbilityDefinition::new(definition(1), definition(1), definition(1), vec![]).with_action(
            action(
                AbilityKind::Basic,
                1,
                TargetInvalidationPolicy::CancelRemainingForTarget,
                ActionResourcePolicy::new(0, 1, zero, energy_20),
            ),
        ),
    );
    builder.add_ability(
        AbilityDefinition::new(definition(2), definition(2), definition(2), vec![]).with_action(
            action(
                AbilityKind::Skill,
                3,
                TargetInvalidationPolicy::CancelRemainingForTarget,
                ActionResourcePolicy::new(1, 0, zero, energy_30),
            ),
        ),
    );
    builder.add_ability(
        AbilityDefinition::new(definition(3), definition(3), definition(3), vec![]).with_action(
            action(
                AbilityKind::Ultimate,
                2,
                TargetInvalidationPolicy::RetargetPrimaryThenRebuildPattern,
                ActionResourcePolicy::new(0, 0, energy_100, zero),
            ),
        ),
    );
    builder.add_ability(
        AbilityDefinition::new(definition(4), definition(4), definition(4), vec![]).with_action(
            action(
                AbilityKind::Basic,
                1,
                TargetInvalidationPolicy::CancelRemainingForTarget,
                ActionResourcePolicy::new(0, 0, zero, zero),
            ),
        ),
    );
    builder.add_unit(UnitDefinition::new(
        definition(1),
        vec![definition(1), definition(2), definition(3)],
        vec![],
    ));
    builder.add_unit(UnitDefinition::new(
        definition(2),
        vec![definition(4)],
        vec![],
    ));
    builder.add_enemy(EnemyDefinition::new(
        definition(1),
        definition(2),
        vec![definition(4)],
    ));
    builder.add_encounter(EncounterDefinition::new(
        definition(1),
        vec![definition(1)],
        vec![],
    ));
    builder.build().unwrap()
}

fn combatant(form: u32, abilities: Vec<u32>, speed: i64, digest: u8) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        definition(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(speed).unwrap(),
        ResolvedDefinitionBindings::new(
            abilities.into_iter().map(definition).collect(),
            vec![],
            vec![],
        )
        .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
}

fn battle() -> Battle {
    battle_with_skill_points(3)
}

fn battle_with_skill_points(skill_points: u16) -> Battle {
    let player = combatant(1, vec![1, 2, 3], 150_000_000, 0x41)
        .with_energy(
            Energy::from_scaled(100_000_000).unwrap(),
            Energy::from_scaled(100_000_000).unwrap(),
        )
        .unwrap();
    let mut participants = vec![ParticipantSpec::new(
        TeamSide::Player,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::Player,
        player,
    )];
    for (formation, digest) in [(3, 0x51), (4, 0x52), (5, 0x53)] {
        participants.push(ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(formation).unwrap(),
            ParticipantSource::EncounterEnemy(definition(1)),
            combatant(2, vec![4], 100_000_000, digest),
        ));
    }
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x61; 32]).unwrap(),
        definition(1),
        participants,
        TeamResourceSpec::new(skill_points, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog(), spec, BattleSeed::new([0x81; 32])).unwrap()
}

#[test]
fn ultimate_and_skill_resources_gate_offers_and_multi_hit_target_locks() {
    let mut battle = battle();
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    let interrupt = battle
        .available_ultimates()
        .into_iter()
        .next()
        .and_then(|option| battle.request_ultimate_command(option))
        .unwrap();
    let requested = battle.apply(interrupt).unwrap();
    assert!(
        requested
            .events()
            .iter()
            .all(|event| !matches!(event.kind(), BattleEventKind::Resource(_)))
    );
    let commit = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::CommitPreparedAction { .. }))
        .unwrap()
        .clone();
    let resolution = battle.apply(commit).unwrap();
    assert_eq!(
        resolution.state_hash().bytes(),
        [
            2, 28, 150, 150, 172, 112, 103, 94, 183, 22, 75, 179, 183, 204, 249, 64, 95, 163, 223,
            123, 151, 141, 120, 176, 102, 27, 90, 122, 28, 117, 161, 84,
        ]
    );
    assert!(matches!(
        resolution.events()[2].kind(),
        BattleEventKind::Resource(ResourceEventData::Energy { before, after, .. })
            if before.scaled() == 100_000_000 && *after == Energy::ZERO
    ));
    let ultimate_targets = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Hit(HitEventData::Started { targets, .. }) => Some(targets.as_ref()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        ultimate_targets,
        [
            [runtime(2), runtime(3), runtime(4)].as_slice(),
            [runtime(2), runtime(3), runtime(4)].as_slice(),
        ]
    );
    assert_eq!(
        battle.view().units_by_id().next().unwrap().current_energy(),
        Energy::ZERO
    );
    assert_eq!(
        battle.decision().unwrap().kind(),
        DecisionKind::NormalAction
    );
    assert!(battle.view().action_boundary().is_some());
    let skill = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseAbility {
                    ability,
                    primary_target: Some(target),
                    ..
                } if ability.get() == 2 && target.get() == 3
            )
        })
        .unwrap()
        .clone();
    let before = battle.state_hash();
    assert_eq!(
        battle
            .apply(Command::UseAbility {
                decision: battle.decision().unwrap().id(),
                actor: runtime(1),
                ability: definition(2),
                primary_target: Some(runtime(99)),
            })
            .unwrap_err()
            .kind(),
        CommandErrorKind::NotOffered
    );
    assert_eq!(battle.state_hash(), before);
    let resolution = battle.apply(skill).unwrap();
    assert_eq!(
        resolution.state_hash().bytes(),
        [
            127, 232, 127, 110, 200, 112, 136, 22, 34, 97, 142, 208, 13, 65, 239, 77, 108, 51, 237,
            199, 238, 48, 153, 247, 99, 223, 227, 250, 18, 240, 28, 53,
        ]
    );
    assert!(matches!(
        resolution.events()[2].kind(),
        BattleEventKind::Resource(ResourceEventData::SkillPoints {
            before: 3,
            after: 2,
            overflow: 0,
            ..
        })
    ));
    let skill_targets = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Hit(HitEventData::Started { targets, .. }) => Some(targets.as_ref()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(skill_targets.len(), 3);
    assert!(
        skill_targets
            .iter()
            .all(|targets| *targets == [runtime(2), runtime(3), runtime(4)])
    );
    assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 2);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .next()
            .unwrap()
            .current_energy()
            .scaled(),
        30_000_000
    );
    settle_ready_boundaries(&mut battle);
    assert_eq!(battle.view().active_turn().unwrap().owner().get(), 2);
}

#[test]
fn basic_gain_clamps_at_caps_and_reports_overflow() {
    assert_eq!(
        combatant(1, vec![1], 100_000_000, 0x31).with_energy(
            Energy::from_scaled(2_000_000).unwrap(),
            Energy::from_scaled(1_000_000).unwrap(),
        ),
        Err(CombatantSpecError::EnergyAboveMaximum)
    );
    let mut battle = battle();
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    battle.advance().unwrap();
    let basic = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseAbility {
                    ability,
                    primary_target: None,
                    ..
                } if ability.get() == 1
            )
        })
        .unwrap()
        .clone();
    let resolution = battle.apply(basic).unwrap();
    assert!(resolution.next_decision().is_none());
    assert_eq!(
        battle.view().phase(),
        starclock_combat::BattlePhase::ReadyToAdvance
    );
    assert!(battle.view().action_boundary().is_some());
    assert!(!resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Turn(starclock_combat::TurnEventData::Ended { .. })
    )));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Resource(ResourceEventData::SkillPoints {
            before: 3,
            after: 4,
            overflow: 0,
            ..
        })
    )));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Resource(ResourceEventData::Energy {
            before,
            after,
            overflow,
            ..
        }) if before.scaled() == 100_000_000
            && after.scaled() == 100_000_000
            && overflow.scaled() == 20_000_000
    )));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Hit(HitEventData::Started { targets, .. })
            if targets.as_ref() == [runtime(1)]
    )));
    assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 4);

    let mut no_skill_points = battle_with_skill_points(0);
    no_skill_points
        .apply(Command::StartBattle {
            decision: no_skill_points.decision().unwrap().id(),
        })
        .unwrap();
    no_skill_points.advance().unwrap();
    assert!(
        !no_skill_points
            .decision()
            .unwrap()
            .legal_commands()
            .iter()
            .any(|command| matches!(
                command,
                Command::UseAbility { ability, .. } if ability.get() == 2
            ))
    );
}

fn named_resource_catalog() -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new([0x91; 32]);
    for (raw, relation) in [
        (10, TargetRelation::SelfUnit),
        (11, TargetRelation::Opposing),
        (12, TargetRelation::SelfUnit),
    ] {
        builder.add_selector(
            SelectorDefinition::new(definition(raw)).with_unit_targets(
                UnitTargetSelector::new(relation, TargetPattern::Single).unwrap(),
            ),
        );
        builder.add_program(ProgramDefinition::new(
            definition(raw),
            vec![],
            vec![definition(raw)],
            vec![],
            vec![],
        ));
    }
    let free = ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO);
    builder.add_ability(
        AbilityDefinition::new(definition(10), definition(10), definition(10), vec![]).with_action(
            action(
                AbilityKind::Basic,
                1,
                TargetInvalidationPolicy::CancelRemainingForTarget,
                free.clone(),
            ),
        ),
    );
    let named_cost = ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO)
        .with_character_resource_costs(vec![
            CharacterResourceCost::new(
                "newbud",
                starclock_combat::Scalar::checked_from_integer(100).unwrap(),
            )
            .unwrap(),
        ])
        .unwrap();
    builder.add_ability(
        AbilityDefinition::new(definition(11), definition(11), definition(11), vec![]).with_action(
            action(
                AbilityKind::Ultimate,
                1,
                TargetInvalidationPolicy::CancelRemainingForTarget,
                named_cost,
            ),
        ),
    );
    builder.add_ability(
        AbilityDefinition::new(definition(12), definition(12), definition(12), vec![]).with_action(
            action(
                AbilityKind::Basic,
                1,
                TargetInvalidationPolicy::CancelRemainingForTarget,
                free,
            ),
        ),
    );
    builder.add_unit(
        UnitDefinition::new(definition(10), vec![definition(10), definition(11)], vec![])
            .with_resources(vec![
                CharacterResourceDefinition::new(
                    "newbud",
                    starclock_combat::Scalar::checked_from_integer(100).unwrap(),
                    starclock_combat::Scalar::checked_from_integer(100).unwrap(),
                )
                .unwrap(),
            ]),
    );
    builder.add_unit(UnitDefinition::new(
        definition(12),
        vec![definition(12)],
        vec![],
    ));
    builder.add_enemy(EnemyDefinition::new(
        definition(10),
        definition(12),
        vec![definition(12)],
    ));
    builder.add_encounter(EncounterDefinition::new(
        definition(10),
        vec![definition(10)],
        vec![],
    ));
    builder.build().unwrap()
}

fn named_resource_battle() -> Battle {
    let participants = vec![
        ParticipantSpec::new(
            TeamSide::Player,
            FormationIndex::new(0).unwrap(),
            ParticipantSource::Player,
            combatant(10, vec![10, 11], 150_000_000, 0xa1),
        ),
        ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(3).unwrap(),
            ParticipantSource::EncounterEnemy(definition(10)),
            combatant(12, vec![12], 100_000_000, 0xa2),
        ),
    ];
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xa3; 32]).unwrap(),
        definition(10),
        participants,
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(named_resource_catalog(), spec, BattleSeed::new([0xa4; 32])).unwrap()
}

#[test]
fn named_character_resource_costs_are_canonical_and_make_ultimates_payable() {
    let amount = starclock_combat::Scalar::checked_from_integer(1).unwrap();
    assert!(CharacterResourceCost::new("", amount).is_none());
    assert!(CharacterResourceCost::new("newbud", starclock_combat::Scalar::ZERO).is_none());
    let duplicate = CharacterResourceCost::new("newbud", amount).unwrap();
    assert!(
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO)
            .with_character_resource_costs(vec![duplicate.clone(), duplicate])
            .is_none()
    );

    let mut builder = CombatCatalogBuilder::new([0xb1; 32]);
    builder.add_selector(SelectorDefinition::new(definition(1)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_program(ProgramDefinition::new(
        definition(1),
        vec![],
        vec![definition(1)],
        vec![],
        vec![],
    ));
    builder.add_ability(
        AbilityDefinition::new(definition(1), definition(1), definition(1), vec![]).with_action(
            action(
                AbilityKind::Ultimate,
                1,
                TargetInvalidationPolicy::CancelRemainingForTarget,
                ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
            ),
        ),
    );
    assert_eq!(
        builder.build().unwrap_err().kind(),
        CatalogBuildErrorKind::InvalidDefinition
    );

    assert!(named_resource_catalog().ability(definition(11)).is_some());
}

#[test]
fn named_character_resource_cost_gates_offers_and_pays_at_action_start() {
    let mut battle = named_resource_battle();
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    let offered = battle
        .available_ultimates()
        .into_iter()
        .find(|option| option.ability().get() == 11)
        .and_then(|option| battle.request_ultimate_command(option))
        .unwrap();
    let prepared = battle.apply(offered).unwrap();
    assert!(prepared.events().iter().all(|event| !matches!(
        event.kind(),
        BattleEventKind::Resource(ResourceEventData::CharacterResource { .. })
    )));
    let before = battle.state_hash();
    assert_eq!(
        battle
            .apply(Command::CommitPreparedAction {
                decision: battle.decision().unwrap().id(),
                primary_target: Some(runtime(99)),
            })
            .unwrap_err()
            .kind(),
        CommandErrorKind::NotOffered
    );
    assert_eq!(battle.state_hash(), before);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .next()
            .unwrap()
            .character_resource("newbud")
            .unwrap()
            .0
            .scaled(),
        100_000_000
    );

    let commit = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::CommitPreparedAction { .. }))
        .unwrap()
        .clone();
    let resolution = battle.apply(commit).unwrap();
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Resource(ResourceEventData::CharacterResource {
            unit,
            resource,
            before,
            after,
            maximum,
        }) if unit.get() == 1
            && resource.as_ref() == "newbud"
            && before.scaled() == 100_000_000
            && after.scaled() == 0
            && maximum.scaled() == 100_000_000
    )));
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .next()
            .unwrap()
            .character_resource("newbud")
            .unwrap()
            .0,
        starclock_combat::Scalar::ZERO
    );
    assert!(
        !battle
            .available_ultimates()
            .iter()
            .any(|option| option.ability().get() == 11)
    );
}

#[test]
fn cancelling_a_prepared_ultimate_restores_the_suspended_boundary_without_payment() {
    let mut battle = named_resource_battle();
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    let request = battle
        .available_ultimates()
        .into_iter()
        .find(|option| option.ability().get() == 11)
        .and_then(|option| battle.request_ultimate_command(option))
        .unwrap();
    let requested = battle.apply(request).unwrap();
    assert!(battle.view().action_boundary().is_none());
    assert!(battle.view().prepared_action().is_some());
    assert_eq!(
        battle.decision().unwrap().kind(),
        DecisionKind::PreparedAction
    );
    assert!(requested.events().iter().all(|event| !matches!(
        event.kind(),
        BattleEventKind::Action(ActionEventData::Declared { .. }) | BattleEventKind::Resource(_)
    )));
    let resource_before = battle
        .view()
        .units_by_id()
        .next()
        .unwrap()
        .character_resource("newbud")
        .unwrap()
        .0;
    let cancel = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::CancelPreparedAction { .. }))
        .unwrap()
        .clone();
    let cancelled = battle.apply(cancel).unwrap();
    assert!(battle.view().prepared_action().is_none());
    assert!(battle.view().action_boundary().is_some());
    assert_eq!(
        battle.decision().unwrap().kind(),
        DecisionKind::NormalAction
    );
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .next()
            .unwrap()
            .character_resource("newbud")
            .unwrap()
            .0,
        resource_before
    );
    assert!(cancelled.events().iter().all(|event| !matches!(
        event.kind(),
        BattleEventKind::Action(ActionEventData::Declared { .. }) | BattleEventKind::Resource(_)
    )));
}
