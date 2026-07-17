use std::sync::Arc;

use starclock_combat::{
    AbilityId, ActionEventData, ActionOrigin, Battle, BattleBuildErrorKind, BattleEventData,
    BattleEventKind, BattlePhase, BattleSeed, BattleSpec, BattleSpecDigest, BattleSpecError,
    CombatantSpecDigest, CombatantSpecError, Command, CommandErrorKind, ConcedePolicy,
    DecisionEventData, DecisionId, DecisionKind, DecisionOwner, EncounterId, EnemyDefinitionId,
    FormationIndex, HitEventData, Hp, InterruptWindowKind, LifeState, ParticipantSource,
    ParticipantSpec, PhaseEventData, PresenceState, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, Speed, TeamResourceSpec, TeamSide, TurnEventData, UnitDefinitionId,
    UnitId, UnitLevel,
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
    I::try_from(raw).expect("test definition ID is non-zero")
}

fn runtime<I: TryFrom<u64>>(raw: u64) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).expect("test runtime ID is non-zero")
}

fn catalog() -> Arc<CombatCatalog> {
    catalog_with_executable_actions(true)
}

fn catalog_with_executable_actions(executable: bool) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new("battle-boundary-catalog-v1", [0x41; 32]);
    builder.add_selector(SelectorDefinition::new(definition(1)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single).unwrap(),
    ));
    builder.add_selector(SelectorDefinition::new(definition(2)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single).unwrap(),
    ));
    builder.add_program(ProgramDefinition::new(
        definition(1),
        vec![],
        vec![definition(1)],
        vec![],
        vec![],
    ));
    builder.add_program(ProgramDefinition::new(
        definition(2),
        vec![],
        vec![definition(2)],
        vec![],
        vec![],
    ));
    let player_ability =
        AbilityDefinition::new(definition(1), definition(1), definition(1), vec![]);
    let enemy_ability = AbilityDefinition::new(definition(2), definition(2), definition(2), vec![]);
    builder.add_ability(if executable {
        player_ability.with_action(basic_action())
    } else {
        player_ability
    });
    builder.add_ability(if executable {
        enemy_ability.with_action(basic_action())
    } else {
        enemy_ability
    });
    builder.add_unit(UnitDefinition::new(
        definition(1),
        vec![definition(1)],
        vec![],
    ));
    builder.add_unit(UnitDefinition::new(
        definition(2),
        vec![definition(2)],
        vec![],
    ));
    builder.add_enemy(EnemyDefinition::new(
        definition(1),
        definition(2),
        vec![definition(2)],
    ));
    builder.add_enemy(EnemyDefinition::new(
        definition(2),
        definition(2),
        vec![definition(2)],
    ));
    builder.add_encounter(EncounterDefinition::new(
        definition(1),
        vec![definition(1)],
        vec![],
    ));
    builder.build().expect("battle fixture catalog is valid")
}

fn basic_action() -> AbilityActionDefinition {
    AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(
            0,
            0,
            starclock_combat::Energy::ZERO,
            starclock_combat::Energy::ZERO,
        ),
    )
    .unwrap()
}

fn combatant(form: u32, ability: u32, digest_byte: u8) -> ResolvedCombatantSpec {
    combatant_at_speed(form, ability, digest_byte, 100_000_000)
}

fn combatant_at_speed(
    form: u32,
    ability: u32,
    digest_byte: u8,
    speed_scaled: i64,
) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        definition(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(speed_scaled).unwrap(),
        ResolvedDefinitionBindings::new(vec![definition(ability)], vec![], vec![]).unwrap(),
        CombatantSpecDigest::new([digest_byte; 32]).unwrap(),
    )
    .unwrap()
}

fn spec_with(encounter: u32, player: ParticipantSpec, enemy: ParticipantSpec) -> BattleSpec {
    BattleSpec::new(
        "synthetic-rules-v1",
        BattleSpecDigest::new([0x51; 32]).unwrap(),
        definition(encounter),
        vec![enemy, player],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap()
}

fn valid_spec() -> BattleSpec {
    spec_with(
        1,
        ParticipantSpec::new(
            TeamSide::Player,
            FormationIndex::new(0).unwrap(),
            ParticipantSource::Player,
            combatant(1, 1, 0x61),
        ),
        ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(4).unwrap(),
            ParticipantSource::EncounterEnemy(definition(1)),
            combatant(2, 2, 0x62),
        ),
    )
}

#[derive(Debug, Eq, PartialEq)]
struct ObservableSnapshot {
    phase: BattlePhase,
    decision: Option<(u64, Vec<Command>)>,
    revision: u64,
    rng_draws: u64,
    state_hash: [u8; 32],
    units: Vec<(u64, u32, TeamSide, u8, i64)>,
}

fn snapshot(battle: &Battle) -> ObservableSnapshot {
    let view = battle.view();
    ObservableSnapshot {
        phase: view.phase(),
        decision: view
            .decision()
            .map(|decision| (decision.id().get(), decision.legal_commands().to_vec())),
        revision: view.committed_revision(),
        rng_draws: view.rng_draw_count(),
        state_hash: battle.state_hash().bytes(),
        units: view
            .units_by_id()
            .map(|unit| {
                (
                    unit.id().get(),
                    unit.form().get(),
                    unit.side(),
                    unit.formation().get(),
                    unit.current_hp().get(),
                )
            })
            .collect(),
    }
}

#[test]
fn battle_construction_allocates_canonical_private_stores_and_read_only_views() {
    let catalog = catalog();
    let battle = Battle::create(
        Arc::clone(&catalog),
        valid_spec(),
        BattleSeed::new([0x71; 32]),
    )
    .unwrap();
    assert_eq!(Arc::strong_count(&catalog), 2);

    let view = battle.view();
    assert_eq!(view.phase(), BattlePhase::Initializing);
    assert_eq!(view.fault(), None);
    assert_eq!(view.committed_revision(), 0);
    assert_eq!(view.rng_draw_count(), 0);
    assert_eq!(
        view.identity().catalog_revision(),
        "battle-boundary-catalog-v1"
    );
    assert_eq!(view.identity().catalog_digest().bytes(), [0x41; 32]);
    assert_eq!(view.identity().rules_revision(), "synthetic-rules-v1");
    assert_eq!(view.identity().spec_digest().bytes(), [0x51; 32]);
    assert_eq!(
        view.identity().numeric_policy_revision(),
        "fixed-i64-6dp-v1"
    );
    assert_eq!(
        view.identity().rng_algorithm_revision(),
        "chacha8-rand-0.10.2-intmap-v1"
    );
    assert_eq!(view.identity().state_hash_revision(), "sha256-v1");
    assert_eq!(view.identity().seed().bytes(), [0x71; 32]);
    assert_eq!(view.encounter().definition(), definition::<EncounterId>(1));
    assert_eq!(view.encounter().wave().get(), 1);

    let decision = view.decision().unwrap();
    assert_eq!(decision.id(), runtime::<DecisionId>(1));
    assert_eq!(decision.kind(), DecisionKind::BattleStart);
    assert_eq!(decision.owner(), DecisionOwner::System);
    assert_eq!(
        decision.legal_commands(),
        [Command::StartBattle {
            decision: runtime(1)
        }]
    );

    let units = view.units_by_id().collect::<Vec<_>>();
    assert_eq!(units.len(), 2);
    assert_eq!(units[0].id(), runtime::<UnitId>(1));
    assert_eq!(units[0].form(), definition::<UnitDefinitionId>(1));
    assert_eq!(units[0].source(), ParticipantSource::Player);
    assert_eq!(units[0].side(), TeamSide::Player);
    assert_eq!(units[0].formation(), FormationIndex::new(0).unwrap());
    assert_eq!(units[0].spawn_sequence().get(), 1);
    assert_eq!(units[0].life(), LifeState::Alive);
    assert_eq!(units[0].presence(), PresenceState::Present);
    assert_eq!(units[0].current_hp(), Hp::new(1_000).unwrap());
    assert_eq!(units[0].maximum_hp(), Hp::new(1_000).unwrap());
    assert_eq!(units[0].abilities(), [definition::<AbilityId>(1)]);
    assert!(units[0].rule_bundles().is_empty());
    assert!(units[0].modifiers().is_empty());
    assert_eq!(units[1].id(), runtime::<UnitId>(2));
    assert_eq!(units[1].side(), TeamSide::Enemy);

    let player_formation = view.formation(TeamSide::Player).collect::<Vec<_>>();
    assert_eq!(player_formation.len(), 1);
    assert_eq!(player_formation[0].unit(), runtime::<UnitId>(1));
    assert_eq!(player_formation[0].index(), FormationIndex::new(0).unwrap());
    let actors = view.timeline_actors().collect::<Vec<_>>();
    assert_eq!(actors.len(), 2);
    assert_eq!(actors[0].id().get(), 1);
    assert_eq!(actors[0].owner(), runtime::<UnitId>(1));
    assert_eq!(actors[0].action_gauge().scaled(), 10_000_000_000);
    assert_eq!(actors[0].speed().scaled(), 100_000_000);
    assert_eq!(view.team(TeamSide::Player).skill_points(), 3);
    assert_eq!(view.team(TeamSide::Player).maximum_skill_points(), 5);
}

#[test]
fn rejected_stale_forged_and_terminal_commands_preserve_observable_state() {
    let mut battle = Battle::create(catalog(), valid_spec(), BattleSeed::new([0x72; 32])).unwrap();
    assert_eq!(
        battle.state_hash().bytes(),
        [
            0x0e, 0x39, 0x84, 0xef, 0x08, 0x94, 0x9c, 0x71, 0x9f, 0x9e, 0xd8, 0x9b, 0xe2, 0xe5,
            0x9e, 0x9b, 0x82, 0x5d, 0x42, 0x48, 0xcf, 0x30, 0x8f, 0xa3, 0xcf, 0xf1, 0xa9, 0x9f,
            0x83, 0x41, 0x1e, 0x1b,
        ]
    );
    let before = snapshot(&battle);
    let stale = Command::StartBattle {
        decision: runtime(99),
    };
    assert_eq!(
        battle.apply(stale).unwrap_err().kind(),
        CommandErrorKind::StaleDecision
    );
    assert_eq!(snapshot(&battle), before);

    let forged = Command::Concede {
        decision: runtime(1),
    };
    assert_eq!(
        battle.apply(forged).unwrap_err().kind(),
        CommandErrorKind::NotOffered
    );
    assert_eq!(snapshot(&battle), before);

    let start = Command::StartBattle {
        decision: runtime(1),
    };
    let started = battle.apply(start).unwrap();
    assert_eq!(
        started.state_hash().bytes(),
        [
            0x75, 0x1c, 0x78, 0x86, 0xcb, 0x85, 0xa0, 0x4d, 0x0a, 0x68, 0x46, 0x26, 0x5a, 0x61,
            0x19, 0x55, 0x5e, 0x2c, 0x01, 0x15, 0x4b, 0x85, 0xd8, 0x16, 0xbf, 0xc3, 0x2c, 0xa3,
            0x14, 0x56, 0xc1, 0x69,
        ]
    );
    assert_eq!(started.phase(), BattlePhase::AwaitingCommand);
    assert_eq!(started.committed_revision(), 1);
    assert_eq!(started.rng_draw_count(), 0);
    assert_eq!(started.state_hash(), battle.state_hash());
    assert_eq!(started.root_command().get(), 1);
    assert_eq!(started.events().len(), 3);
    assert_eq!(started.events()[0].id().get(), 1);
    assert_eq!(started.events()[0].cause().root_command().get(), 1);
    assert_eq!(started.events()[0].cause().parent_event(), None);
    assert_eq!(
        started.events()[0].kind(),
        &BattleEventKind::Battle(BattleEventData::Started)
    );
    assert_eq!(
        started.events()[1].kind(),
        &BattleEventKind::Turn(TurnEventData::Started {
            actor: runtime(1),
            owner: runtime(1),
        })
    );
    assert_eq!(started.events()[2].id().get(), 3);
    assert_eq!(started.events()[2].cause().root_command().get(), 1);
    assert_eq!(started.events()[2].cause().parent_event().unwrap().get(), 2);
    assert_eq!(
        started.events()[2].kind(),
        &BattleEventKind::Decision(DecisionEventData::Offered {
            decision: runtime(2),
            kind: DecisionKind::InterruptWindow,
            owner: DecisionOwner::Team(TeamSide::Player),
        })
    );
    let next = started.next_decision().unwrap();
    assert_eq!(next.id(), runtime::<DecisionId>(2));
    assert_eq!(next.kind(), DecisionKind::InterruptWindow);
    assert_eq!(next.owner(), DecisionOwner::Team(TeamSide::Player));
    assert_eq!(
        next.legal_commands(),
        [Command::PassInterruptWindow {
            decision: runtime(2)
        }]
    );
    assert_eq!(battle.decision(), Some(next));
    assert_eq!(battle.view().active_turn().unwrap().owner(), runtime(1));
    assert_eq!(
        battle.view().interrupt_window().unwrap().kind(),
        InterruptWindowKind::PreAction
    );

    let interrupt = snapshot(&battle);
    let forged_action = Command::UseAbility {
        decision: runtime(2),
        actor: runtime(1),
        ability: definition(1),
        primary_target: None,
    };
    assert_eq!(
        battle.apply(forged_action).unwrap_err().kind(),
        CommandErrorKind::NotOffered
    );
    assert_eq!(snapshot(&battle), interrupt);

    let passed = battle
        .apply(Command::PassInterruptWindow {
            decision: runtime(2),
        })
        .unwrap();
    assert_eq!(
        passed.state_hash().bytes(),
        [
            0xdd, 0xe6, 0x43, 0xb2, 0xce, 0x3a, 0x03, 0x83, 0x36, 0x70, 0xb3, 0x20, 0x3b, 0xd4,
            0xb2, 0xba, 0x55, 0xc5, 0x44, 0x2f, 0x75, 0xc9, 0x59, 0xbd, 0xb3, 0xe8, 0xb5, 0xc1,
            0x2a, 0x44, 0xb1, 0xd9,
        ]
    );
    let next = passed.next_decision().unwrap();
    assert_eq!(next.id(), runtime::<DecisionId>(3));
    assert_eq!(next.kind(), DecisionKind::NormalAction);
    assert_eq!(
        next.legal_commands(),
        [
            Command::UseAbility {
                decision: runtime(3),
                actor: runtime(1),
                ability: definition(1),
                primary_target: None,
            },
            Command::Concede {
                decision: runtime(3),
            },
        ]
    );
    assert!(battle.view().interrupt_window().is_none());

    let awaiting = snapshot(&battle);
    let unoffered = Command::UseAbility {
        decision: runtime(3),
        actor: runtime(1),
        ability: definition(1),
        primary_target: Some(runtime(2)),
    };
    assert_eq!(
        battle.apply(unoffered).unwrap_err().kind(),
        CommandErrorKind::NotOffered
    );
    assert_eq!(snapshot(&battle), awaiting);

    let ended = battle
        .apply(Command::Concede {
            decision: runtime(3),
        })
        .unwrap();
    assert_eq!(
        ended.state_hash().bytes(),
        [
            0x44, 0xf7, 0x94, 0xdd, 0x5f, 0xea, 0x53, 0xbc, 0xbd, 0xd6, 0x18, 0xff, 0xda, 0xdb,
            0x09, 0xbd, 0x56, 0x4f, 0x52, 0x48, 0xb7, 0x41, 0xb6, 0xd9, 0x9c, 0xed, 0x67, 0x57,
            0x18, 0xa3, 0xa0, 0xdc,
        ]
    );
    assert_eq!(ended.phase(), BattlePhase::Lost);
    assert_eq!(ended.committed_revision(), 3);
    assert_eq!(ended.next_decision(), None);
    assert_eq!(ended.state_hash(), battle.state_hash());
    assert_eq!(ended.root_command().get(), 3);
    assert_eq!(ended.events().len(), 2);
    assert_eq!(ended.events()[0].id().get(), 6);
    assert_eq!(ended.events()[1].id().get(), 7);
    assert_eq!(ended.events()[1].cause().root_command().get(), 3);
    assert_eq!(
        ended.events()[1].kind(),
        &BattleEventKind::Battle(BattleEventData::Conceded {
            side: TeamSide::Player
        })
    );
    assert!(battle.decision().is_none());
    let terminal = snapshot(&battle);
    assert_eq!(
        battle
            .apply(Command::Concede {
                decision: runtime(3)
            })
            .unwrap_err()
            .kind(),
        CommandErrorKind::TerminalBattle
    );
    assert_eq!(snapshot(&battle), terminal);
}

#[test]
fn normal_action_lowers_one_phase_and_hit_then_selects_the_next_turn() {
    let mut battle = Battle::create(catalog(), valid_spec(), BattleSeed::new([0x73; 32])).unwrap();
    battle
        .apply(Command::StartBattle {
            decision: runtime(1),
        })
        .unwrap();
    battle
        .apply(Command::PassInterruptWindow {
            decision: runtime(2),
        })
        .unwrap();
    let resolution = battle
        .apply(Command::UseAbility {
            decision: runtime(3),
            actor: runtime(1),
            ability: definition(1),
            primary_target: None,
        })
        .unwrap();
    assert_eq!(
        resolution.state_hash().bytes(),
        [
            0xe2, 0x1f, 0x3f, 0xfd, 0xdf, 0x88, 0xaa, 0x5b, 0x70, 0xc5, 0xc9, 0xba, 0x1b, 0x2e,
            0x02, 0x27, 0x89, 0xef, 0xea, 0x93, 0x3f, 0xa1, 0xe8, 0xcb, 0xbd, 0xad, 0x3e, 0x61,
            0x2e, 0x44, 0x75, 0xa1,
        ]
    );

    assert_eq!(resolution.committed_revision(), 3);
    assert_eq!(resolution.events().len(), 11);
    assert!(matches!(
        resolution.events()[0].kind(),
        BattleEventKind::Decision(DecisionEventData::Closed { decision })
            if decision.get() == 3
    ));
    assert!(matches!(
        resolution.events()[1].kind(),
        BattleEventKind::Action(ActionEventData::Declared {
            action,
            actor,
            ability,
            origin: ActionOrigin::NormalTurn,
        }) if action.get() == 1 && actor.get() == 1 && ability.get() == 1
    ));
    assert!(matches!(
        resolution.events()[2].kind(),
        BattleEventKind::Action(ActionEventData::Started { action, .. }) if action.get() == 1
    ));
    assert!(matches!(
        resolution.events()[3].kind(),
        BattleEventKind::Phase(PhaseEventData::Started { action, phase })
            if action.get() == 1 && phase.get() == 1
    ));
    assert!(matches!(
        resolution.events()[4].kind(),
        BattleEventKind::Hit(HitEventData::Started { action, phase, hit, .. })
            if action.get() == 1 && phase.get() == 1 && hit.get() == 1
    ));
    assert!(matches!(
        resolution.events()[5].kind(),
        BattleEventKind::Hit(HitEventData::Ended { hit, .. }) if hit.get() == 1
    ));
    assert!(matches!(
        resolution.events()[6].kind(),
        BattleEventKind::Phase(PhaseEventData::Ended { phase, .. }) if phase.get() == 1
    ));
    assert!(matches!(
        resolution.events()[7].kind(),
        BattleEventKind::Action(ActionEventData::Resolved { action, .. }) if action.get() == 1
    ));
    assert!(matches!(
        resolution.events()[8].kind(),
        BattleEventKind::Turn(TurnEventData::Ended { actor, owner })
            if actor.get() == 1 && owner.get() == 1
    ));
    assert!(matches!(
        resolution.events()[9].kind(),
        BattleEventKind::Turn(TurnEventData::Started { actor, owner })
            if actor.get() == 2 && owner.get() == 2
    ));
    assert!(matches!(
        resolution.events()[10].kind(),
        BattleEventKind::Decision(DecisionEventData::Offered {
            decision,
            kind: DecisionKind::InterruptWindow,
            owner: DecisionOwner::Team(TeamSide::Enemy),
        }) if decision.get() == 4
    ));
    for event in &resolution.events()[1..8] {
        let cause = event.cause();
        assert_eq!(cause.root_command().get(), 3);
        assert_eq!(cause.action().unwrap().get(), 1);
        assert_eq!(cause.owner().unwrap().get(), 1);
    }
    for pair in resolution.events().windows(2) {
        assert_eq!(pair[1].cause().parent_event(), Some(pair[0].id()));
    }
    let view = battle.view();
    assert_eq!(view.active_turn().unwrap().owner().get(), 2);
    assert_eq!(view.interrupt_window().unwrap().pending_count(), 0);
    let gauges = view
        .timeline_actors()
        .map(|actor| actor.action_gauge().scaled())
        .collect::<Vec<_>>();
    assert_eq!(gauges, [10_000_000_000, 0]);
    assert_eq!(resolution.state_hash(), battle.state_hash());
}

#[test]
fn timeline_uses_exact_av_order_and_floored_gauge_elapsed_distance() {
    let spec = spec_with(
        1,
        ParticipantSpec::new(
            TeamSide::Player,
            FormationIndex::new(0).unwrap(),
            ParticipantSource::Player,
            combatant_at_speed(1, 1, 0x63, 150_000_000),
        ),
        ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(4).unwrap(),
            ParticipantSource::EncounterEnemy(definition(1)),
            combatant_at_speed(2, 2, 0x64, 100_000_000),
        ),
    );
    let mut battle = Battle::create(catalog(), spec, BattleSeed::new([0x74; 32])).unwrap();
    battle
        .apply(Command::StartBattle {
            decision: runtime(1),
        })
        .unwrap();
    assert_eq!(battle.view().active_turn().unwrap().owner().get(), 1);
    assert_eq!(
        battle
            .view()
            .timeline_actors()
            .map(|actor| actor.action_gauge().scaled())
            .collect::<Vec<_>>(),
        [0, 3_333_333_334]
    );
    battle
        .apply(Command::PassInterruptWindow {
            decision: runtime(2),
        })
        .unwrap();
    battle
        .apply(Command::UseAbility {
            decision: runtime(3),
            actor: runtime(1),
            ability: definition(1),
            primary_target: None,
        })
        .unwrap();
    assert_eq!(battle.view().active_turn().unwrap().owner().get(), 2);
    assert_eq!(
        battle
            .view()
            .timeline_actors()
            .map(|actor| actor.action_gauge().scaled())
            .collect::<Vec<_>>(),
        [4_999_999_999, 0]
    );
}

#[test]
fn catalog_and_participant_composition_fail_before_runtime_allocation() {
    let error = Battle::create(
        catalog_with_executable_actions(false),
        valid_spec(),
        BattleSeed::new([0; 32]),
    )
    .unwrap_err();
    assert_eq!(error.kind(), BattleBuildErrorKind::NoExecutableAbility);
    assert_eq!(error.participant_index(), Some(0));
    assert_eq!(error.definition_id(), None);

    let invalid_source = spec_with(
        1,
        ParticipantSpec::new(
            TeamSide::Player,
            FormationIndex::new(0).unwrap(),
            ParticipantSource::EncounterEnemy(definition(1)),
            combatant(1, 1, 0x63),
        ),
        ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(0).unwrap(),
            ParticipantSource::EncounterEnemy(definition(1)),
            combatant(2, 2, 0x64),
        ),
    );
    assert_eq!(
        Battle::create(catalog(), invalid_source, BattleSeed::new([1; 32]))
            .unwrap_err()
            .kind(),
        BattleBuildErrorKind::InvalidParticipantSource
    );

    let unlisted_enemy = spec_with(
        1,
        ParticipantSpec::new(
            TeamSide::Player,
            FormationIndex::new(0).unwrap(),
            ParticipantSource::Player,
            combatant(1, 1, 0x65),
        ),
        ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(0).unwrap(),
            ParticipantSource::EncounterEnemy(definition(2)),
            combatant(2, 2, 0x66),
        ),
    );
    let error = Battle::create(catalog(), unlisted_enemy, BattleSeed::new([2; 32])).unwrap_err();
    assert_eq!(error.kind(), BattleBuildErrorKind::EnemyNotInEncounter);
    assert_eq!(error.definition_id(), Some(2));

    let missing_ability = spec_with(
        1,
        ParticipantSpec::new(
            TeamSide::Player,
            FormationIndex::new(0).unwrap(),
            ParticipantSource::Player,
            combatant(1, 99, 0x67),
        ),
        ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(0).unwrap(),
            ParticipantSource::EncounterEnemy(definition(1)),
            combatant(2, 2, 0x68),
        ),
    );
    let error = Battle::create(catalog(), missing_ability, BattleSeed::new([3; 32])).unwrap_err();
    assert_eq!(error.kind(), BattleBuildErrorKind::MissingAbility);
    assert_eq!(error.participant_index(), Some(0));
    assert_eq!(error.definition_id(), Some(99));

    let missing_encounter = spec_with(
        99,
        ParticipantSpec::new(
            TeamSide::Player,
            FormationIndex::new(0).unwrap(),
            ParticipantSource::Player,
            combatant(1, 1, 0x69),
        ),
        ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(0).unwrap(),
            ParticipantSource::EncounterEnemy(definition(1)),
            combatant(2, 2, 0x6a),
        ),
    );
    assert_eq!(
        Battle::create(catalog(), missing_encounter, BattleSeed::new([4; 32]))
            .unwrap_err()
            .kind(),
        BattleBuildErrorKind::MissingEncounter
    );

    let _enemy_id: EnemyDefinitionId = definition(1);
}

#[test]
fn local_specs_reject_noncanonical_bindings_and_illegal_formations() {
    assert_eq!(
        ResolvedDefinitionBindings::new(
            vec![definition::<AbilityId>(2), definition(1)],
            vec![],
            vec![]
        ),
        Err(CombatantSpecError::NonCanonicalReferences)
    );
    assert_eq!(
        ResolvedCombatantSpec::new(
            definition(1),
            UnitLevel::new(80).unwrap(),
            Hp::new(0).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
            ResolvedDefinitionBindings::new(vec![definition(1)], vec![], vec![]).unwrap(),
            CombatantSpecDigest::new([0x77; 32]).unwrap(),
        ),
        Err(CombatantSpecError::ZeroMaximumHp)
    );

    let duplicate = BattleSpec::new(
        "synthetic-rules-v1",
        BattleSpecDigest::new([0x78; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 1, 0x79),
            ),
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 1, 0x7a),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(2, 2, 0x7b),
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    );
    assert_eq!(duplicate, Err(BattleSpecError::DuplicateFormation));
}
