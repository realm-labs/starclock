use std::sync::Arc;

use starclock_combat::{
    ActionBoundaryEventData, ActionEventData, ActionFrameInput, ActionOrigin, AssemblyDigest,
    Battle, BattleEvent, BattleEventKind, BattleSeed, BattleSpec, CombatantSpecDigest, Command,
    CommandErrorKind, ConcedePolicy, DecisionKind, Energy, FormationIndex, HitEventData, Hp,
    ParticipantSource, ParticipantSpec, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionResourcePolicy, ActionSegmentDefinition,
            AutomaticSegmentTarget, InitialActionSegment, SegmentedActionDefinition,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::{CatalogBuildErrorKind, CombatCatalogBuilder},
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

fn action(kind: AbilityKind, energy_cost: Energy) -> AbilityActionDefinition {
    AbilityActionDefinition::new(
        kind,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(0, 0, energy_cost, Energy::ZERO),
    )
    .unwrap()
}

fn catalog_with_segment_gain(
    segment_energy_gain: Energy,
) -> Result<Arc<CombatCatalog>, CatalogBuildErrorKind> {
    let mut builder = CombatCatalogBuilder::new([0xa1; 32]);
    for (raw, relation, pattern) in [
        (1, TargetRelation::Opposing, TargetPattern::Single),
        (2, TargetRelation::Opposing, TargetPattern::Single),
        (3, TargetRelation::Opposing, TargetPattern::All),
        (4, TargetRelation::SelfUnit, TargetPattern::Single),
        (5, TargetRelation::Opposing, TargetPattern::Single),
        (6, TargetRelation::Opposing, TargetPattern::Single),
        (7, TargetRelation::Opposing, TargetPattern::Single),
        (8, TargetRelation::Opposing, TargetPattern::Single),
        (9, TargetRelation::Opposing, TargetPattern::Single),
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

    let full_energy = Energy::from_scaled(100_000_000).unwrap();
    let acheron_flow = SegmentedActionDefinition::new(
        InitialActionSegment::ExecuteParent,
        vec![
            ActionSegmentDefinition::SelectTarget {
                ability: definition(2),
            },
            ActionSegmentDefinition::SelectTarget {
                ability: definition(2),
            },
            ActionSegmentDefinition::Automatic {
                ability: definition(3),
                target: AutomaticSegmentTarget::AbilitySelector,
            },
        ],
    )
    .unwrap();
    let mut feixiao_steps = (0..6)
        .map(|_| {
            ActionSegmentDefinition::select_option(vec![definition(7), definition(8)]).unwrap()
        })
        .collect::<Vec<_>>();
    feixiao_steps.push(ActionSegmentDefinition::Automatic {
        ability: definition(9),
        target: AutomaticSegmentTarget::Retained,
    });
    let feixiao_flow =
        SegmentedActionDefinition::new(InitialActionSegment::RetainTarget, feixiao_steps).unwrap();

    builder.add_ability(
        AbilityDefinition::new(definition(1), definition(1), definition(1), vec![]).with_action(
            action(AbilityKind::Ultimate, full_energy)
                .with_segmented_flow(acheron_flow)
                .unwrap(),
        ),
    );
    for raw in [2, 3, 7, 8, 9] {
        builder.add_ability(
            AbilityDefinition::new(definition(raw), definition(raw), definition(raw), vec![])
                .with_action(
                    AbilityActionDefinition::new(
                        AbilityKind::ExtraAction,
                        1,
                        TargetInvalidationPolicy::CancelRemainingForTarget,
                        ActionResourcePolicy::new(0, 0, Energy::ZERO, segment_energy_gain),
                    )
                    .unwrap(),
                ),
        );
    }
    builder.add_ability(
        AbilityDefinition::new(definition(4), definition(4), definition(4), vec![])
            .with_action(action(AbilityKind::Basic, Energy::ZERO)),
    );
    builder.add_ability(
        AbilityDefinition::new(definition(5), definition(5), definition(5), vec![])
            .with_action(action(AbilityKind::Basic, Energy::ZERO)),
    );
    builder.add_ability(
        AbilityDefinition::new(definition(6), definition(6), definition(6), vec![]).with_action(
            action(AbilityKind::Ultimate, full_energy)
                .with_segmented_flow(feixiao_flow)
                .unwrap(),
        ),
    );
    builder.add_unit(UnitDefinition::new(
        definition(1),
        vec![definition(1), definition(4), definition(6)],
        vec![],
    ));
    builder.add_unit(UnitDefinition::new(
        definition(2),
        vec![definition(5)],
        vec![],
    ));
    builder.add_enemy(EnemyDefinition::new(
        definition(1),
        definition(2),
        vec![definition(5)],
    ));
    builder.add_encounter(EncounterDefinition::new(
        definition(1),
        vec![definition(1)],
        vec![],
    ));
    builder.build().map_err(|error| error.kind())
}

fn catalog() -> Arc<CombatCatalog> {
    catalog_with_segment_gain(Energy::ZERO).unwrap()
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
    let player = combatant(1, vec![1, 4, 6], 150_000_000, 1)
        .with_energy(
            Energy::from_scaled(100_000_000).unwrap(),
            Energy::from_scaled(100_000_000).unwrap(),
        )
        .unwrap();
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xa2; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                player,
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(1).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(2, vec![5], 100_000_000, 2),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(2).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(2, vec![5], 100_000_000, 3),
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog(), spec, BattleSeed::new([0xa3; 32])).unwrap()
}

fn request_ultimate(battle: &mut Battle, ability: u32) -> Vec<BattleEvent> {
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    let option = battle
        .available_ultimates()
        .into_iter()
        .find(|option| option.ability().get() == ability)
        .unwrap();
    let request = battle.request_ultimate_command(option).unwrap();
    battle.apply(request).unwrap();
    let commit = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::CommitPreparedAction { .. }))
        .unwrap()
        .clone();
    battle.apply(commit).unwrap().events().to_vec()
}

#[test]
fn target_selected_segments_share_one_paid_action_and_finish_automatically() {
    let mut battle = battle();
    let mut events = request_ultimate(&mut battle, 1);
    let frame = battle.view().action_frame().unwrap();
    let action = frame.action();
    let frame_id = frame.id();
    let mut committed_targets = frame
        .inputs()
        .iter()
        .filter_map(|input| match input {
            ActionFrameInput::Target(target) => Some(*target),
            ActionFrameInput::Option(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(frame.cursor(), 0);
    assert!(frame.paid());
    assert_eq!(battle.decision().unwrap().kind(), DecisionKind::ActionFrame);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .find(|unit| unit.id() == frame.actor())
            .unwrap()
            .current_energy(),
        Energy::ZERO
    );

    for expected_cursor in [1, 2] {
        let command = battle
            .decision()
            .unwrap()
            .legal_commands()
            .iter()
            .find(|command| {
                matches!(
                    command,
                    Command::CommitActionFrame {
                        input: ActionFrameInput::Target(_),
                        ..
                    }
                )
            })
            .unwrap()
            .clone();
        if let Command::CommitActionFrame {
            input: ActionFrameInput::Target(target),
            ..
        } = &command
        {
            committed_targets.push(*target);
        }
        let resolution = battle.apply(command).unwrap();
        events.extend_from_slice(resolution.events());
        if expected_cursor == 1 {
            assert_eq!(battle.view().action_frame().unwrap().cursor(), 1);
        }
    }
    assert!(battle.view().action_frame().is_none());
    assert_eq!(committed_targets.len(), 3);
    let hit_actions = events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Hit(HitEventData::Started { action, .. }) => Some(*action),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(hit_actions.len(), 4);
    assert!(hit_actions.iter().all(|candidate| *candidate == action));
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                event.kind(),
                BattleEventKind::Action(ActionEventData::Resolved {
                    action: resolved,
                    origin: ActionOrigin::UltimateInterrupt,
                    ..
                }) if *resolved == action
            ))
            .count(),
        1
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                event.kind(),
                BattleEventKind::ActionBoundary(ActionBoundaryEventData::ActionFrameOpened {
                    frame,
                    action: opened,
                    ..
                }) if *frame == frame_id && *opened == action
            ))
            .count(),
        1
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                event.kind(),
                BattleEventKind::ActionBoundary(
                    ActionBoundaryEventData::ActionFrameInputCommitted { frame, .. }
                ) if *frame == frame_id
            ))
            .count(),
        2
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                event.kind(),
                BattleEventKind::ActionBoundary(ActionBoundaryEventData::ActionFrameCompleted {
                    frame,
                    action: completed,
                }) if *frame == frame_id && *completed == action
            ))
            .count(),
        1
    );
}

#[test]
fn retained_target_option_segments_offer_six_choices_then_one_finisher() {
    let mut battle = battle();
    let mut events = request_ultimate(&mut battle, 6);
    let action = battle.view().action_frame().unwrap().action();
    for cursor in 0..6 {
        let command = battle
            .decision()
            .unwrap()
            .legal_commands()
            .iter()
            .find(|command| {
                matches!(
                    command,
                    Command::CommitActionFrame {
                        input: ActionFrameInput::Option(ability),
                        ..
                    } if ability.get() == if cursor % 2 == 0 { 7 } else { 8 }
                )
            })
            .unwrap()
            .clone();
        let resolution = battle.apply(command).unwrap();
        events.extend_from_slice(resolution.events());
        if cursor < 5 {
            assert_eq!(battle.view().action_frame().unwrap().cursor(), cursor + 1);
        }
    }
    assert!(battle.view().action_frame().is_none());
    let hit_actions = events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Hit(HitEventData::Started { action, .. }) => Some(*action),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(hit_actions.len(), 7);
    assert!(hit_actions.iter().all(|candidate| *candidate == action));
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                event.kind(),
                BattleEventKind::ActionBoundary(
                    ActionBoundaryEventData::ActionFrameInputCommitted { .. }
                )
            ))
            .count(),
        6
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                event.kind(),
                BattleEventKind::Action(ActionEventData::Declared { action: declared, .. })
                    if *declared == action
            ))
            .count(),
        1
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                event.kind(),
                BattleEventKind::Action(ActionEventData::Resolved { action: resolved, .. })
                    if *resolved == action
            ))
            .count(),
        1
    );
}

#[test]
fn segmented_definitions_are_bounded_and_ultimate_only() {
    let step = ActionSegmentDefinition::Automatic {
        ability: definition(2),
        target: AutomaticSegmentTarget::Retained,
    };
    assert!(SegmentedActionDefinition::new(InitialActionSegment::RetainTarget, vec![]).is_none());
    assert!(
        SegmentedActionDefinition::new(InitialActionSegment::RetainTarget, vec![step.clone(); 33],)
            .is_none()
    );
    let flow =
        SegmentedActionDefinition::new(InitialActionSegment::RetainTarget, vec![step]).unwrap();
    assert!(
        action(AbilityKind::Basic, Energy::ZERO)
            .with_segmented_flow(flow)
            .is_none()
    );
    assert_eq!(
        catalog_with_segment_gain(Energy::from_scaled(1).unwrap()).unwrap_err(),
        CatalogBuildErrorKind::InvalidDefinition
    );
}

#[test]
fn unoffered_segment_input_is_byte_inert() {
    let mut battle = battle();
    request_ultimate(&mut battle, 6);
    let before_hash = battle.state_hash();
    let before_decision = battle.decision().cloned();
    let frame = battle.view().action_frame().unwrap();
    let before = (frame.id(), frame.cursor(), frame.inputs().to_vec());
    let error = battle
        .apply(Command::CommitActionFrame {
            decision: battle.decision().unwrap().id(),
            input: ActionFrameInput::Option(definition(9)),
        })
        .unwrap_err();
    assert_eq!(error.kind(), CommandErrorKind::NotOffered);
    assert_eq!(battle.state_hash(), before_hash);
    assert_eq!(battle.decision(), before_decision.as_ref());
    let frame = battle.view().action_frame().unwrap();
    assert_eq!(
        (frame.id(), frame.cursor(), frame.inputs().to_vec()),
        before
    );
}
