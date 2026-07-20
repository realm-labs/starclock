use std::sync::Arc;

use starclock_combat::{
    Battle, BattleEventKind, BattleSeed, BattleSpec, BattleSpecDigest, CombatantSpecDigest,
    CombatantSpecError, Command, CommandErrorKind, ConcedePolicy, Energy, FormationIndex,
    HitEventData, Hp, ParticipantSource, ParticipantSpec, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, ResourceEventData, Speed, TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionResourcePolicy, TargetInvalidationPolicy,
            TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
    },
};

fn definition<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
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
    let mut builder = CombatCatalogBuilder::new("action-resource-v1", [0x71; 32]);
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
        "action-resource-rules-v1",
        BattleSpecDigest::new([0x61; 32]).unwrap(),
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
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseInterrupt { .. }))
        .unwrap()
        .clone();
    let resolution = battle.apply(interrupt).unwrap();
    assert_eq!(
        resolution.state_hash().bytes(),
        [
            0xe0, 0xa3, 0x6c, 0x1a, 0x61, 0xbe, 0x7e, 0x45, 0x47, 0x45, 0x59, 0x88, 0x00, 0xa1,
            0xc2, 0x71, 0x12, 0xe8, 0x24, 0x85, 0xfb, 0x2a, 0xe5, 0x34, 0x25, 0x29, 0xf3, 0x86,
            0x4a, 0xbd, 0x82, 0x7a,
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
    assert_eq!(battle.decision().unwrap().legal_commands().len(), 1);

    battle
        .apply(Command::PassInterruptWindow {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
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
            0x41, 0xcb, 0x15, 0xfe, 0xaf, 0x50, 0x01, 0x02, 0x7e, 0x2c, 0xa8, 0x6d, 0xb0, 0x04,
            0xf1, 0xb2, 0x36, 0xa8, 0xa2, 0xa4, 0xde, 0x41, 0xf9, 0xa4, 0xeb, 0x37, 0x58, 0x34,
            0x8e, 0x62, 0xb8, 0x4b,
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
    battle
        .apply(Command::PassInterruptWindow {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
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
    no_skill_points
        .apply(Command::PassInterruptWindow {
            decision: no_skill_points.decision().unwrap().id(),
        })
        .unwrap();
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
