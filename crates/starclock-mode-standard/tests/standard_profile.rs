use starclock_activity::{
    ActivityCommand, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivitySlotDefinition, ActivitySpec,
    ActivityValue, BattleBinding, BattleOutcome, BattleResult, BattleResultProjection, BuildDigest,
    EventDigest, LoadoutLockScope, MetricValueKind, OneBattleFlow, OpaqueParticipantBuild,
    ParticipantId, ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope, ProjectedValue, ProjectionField, ProjectionId, SlotResetPoint,
};
use starclock_combat::{
    AbilityId, BattleSpec, BattleSpecDigest, BattleStateHash, CombatantSpecDigest, ConcedePolicy,
    EncounterId, EnemyDefinitionId, FormationIndex, Hp, ParticipantSource, ParticipantSpec,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed, TeamResourceSpec, TeamSide,
    UnitDefinitionId, UnitLevel, catalog::encounter::WaveTransitionPolicy,
};
use starclock_mode_standard::{
    StandardActivityBinding, StandardBindingId, StandardExpectedOutcome, StandardProfile,
    StandardProfileError, StandardProfileId, StandardScenario, StandardScenarioError,
    StandardScenarioId, StandardTerminalError,
};

#[test]
fn profile_structurally_has_one_team_and_only_ordinary_defaults() {
    let profile = profile(4);
    assert_eq!(profile.player_team_count(), 1);
    assert_eq!(profile.maximum_party_size(), 4);
    assert_eq!(
        profile.default_wave_transition(),
        WaveTransitionPolicy::AfterAction
    );
    assert!(
        StandardProfile::new(
            profile_id(2),
            activity_id(1),
            0,
            WaveTransitionPolicy::AfterHit
        )
        .is_none()
    );
    assert!(
        StandardProfile::new(
            profile_id(2),
            activity_id(1),
            5,
            WaveTransitionPolicy::AfterHit
        )
        .is_none()
    );
}

#[test]
fn scenario_resolves_profile_activity_binding_and_exact_seed() {
    let scenario = standard_scenario(StandardExpectedOutcome::Won);
    assert_eq!(scenario.id().get(), 3);
    assert_eq!(scenario.profile().id().get(), 2);
    assert_eq!(scenario.binding().id().get(), 4);
    assert_eq!(scenario.master_seed(), 0x0123_4567_89ab_cdef);
    assert_eq!(scenario.expected_outcome(), StandardExpectedOutcome::Won);

    for invalid in ["123", "0123456789abcdeg", " 123456789abcdef"] {
        let error = StandardScenario::new(
            scenario_id(9),
            profile(4),
            StandardActivityBinding::new(
                binding_id(4),
                activity_spec(1, participant_policy(1, 4), standard_projection()),
            ),
            invalid,
            StandardExpectedOutcome::Won,
        )
        .unwrap_err();
        assert_eq!(error, StandardScenarioError::InvalidMasterSeed);
    }
}

#[test]
fn profile_rejects_cross_row_and_nonstandard_activity_shapes() {
    let mismatch = profile(4)
        .validate_activity(&activity_spec(
            2,
            participant_policy(1, 4),
            standard_projection(),
        ))
        .unwrap_err();
    assert_eq!(mismatch, StandardProfileError::ActivityIdentityMismatch);

    let oversized = profile(2)
        .validate_activity(&activity_spec(
            1,
            participant_policy(1, 4),
            standard_projection(),
        ))
        .unwrap_err();
    assert_eq!(oversized, StandardProfileError::PartySizeMismatch);

    let multiple = profile(4)
        .validate_activity(&activity_spec(
            1,
            participant_policy(2, 1),
            standard_projection(),
        ))
        .unwrap_err();
    assert_eq!(multiple, StandardProfileError::MultiplePlayerTeams);

    let projection = BattleResultProjection::new(
        projection_id(2),
        vec![
            ProjectionField::Outcome,
            ProjectionField::FinalStateHash,
            ProjectionField::EventDigest,
            ProjectionField::TerminalFault,
            ProjectionField::Metric {
                key: "score".into(),
                kind: MetricValueKind::BoundedInteger,
            },
        ],
    )
    .unwrap();
    let nonstandard = profile(4)
        .validate_activity(&activity_spec(1, participant_policy(1, 4), projection))
        .unwrap_err();
    assert_eq!(nonstandard, StandardProfileError::NonStandardProjection);
}

#[test]
fn instantiated_scenario_uses_generic_activity_and_verifies_terminal_outcome() {
    let scenario = standard_scenario(StandardExpectedOutcome::Won);
    let instance = ActivityInstanceId::new(101).unwrap();
    let mut run = scenario.instantiate(instance);
    let second = scenario.instantiate(instance);
    assert_eq!(run.profile_id(), scenario.profile().id());
    assert_eq!(run.scenario_id(), scenario.id());
    assert_eq!(run.binding_id(), scenario.binding().id());
    assert_eq!(run.activity().state_hash(), second.activity().state_hash());
    assert_eq!(
        run.verify_terminal(),
        Err(StandardTerminalError::NotTerminal)
    );

    let initial = run.activity().state_hash();
    let started = run
        .activity_mut()
        .apply(ActivityCommand::StartBattle {
            expected_state_hash: initial,
        })
        .unwrap();
    let identity = started.battle_handoff().unwrap().identity();
    let result = BattleResult::seal(
        identity,
        vec![
            ProjectedValue::Outcome(BattleOutcome::Won),
            ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x81; 32])),
            ProjectedValue::EventDigest(EventDigest::new([0x82; 32]).unwrap()),
            ProjectedValue::TerminalFault(None),
        ],
    );
    run.activity_mut()
        .apply(ActivityCommand::SubmitBattleResult {
            expected_state_hash: started.state_hash(),
            result: Box::new(result),
        })
        .unwrap();
    assert_eq!(run.verify_terminal(), Ok(()));

    let lost_scenario = standard_scenario(StandardExpectedOutcome::Lost);
    let mut mismatched = lost_scenario.instantiate(ActivityInstanceId::new(102).unwrap());
    complete_as_won(&mut mismatched);
    assert!(matches!(
        mismatched.verify_terminal(),
        Err(StandardTerminalError::OutcomeMismatch {
            expected: StandardExpectedOutcome::Lost,
            ..
        })
    ));
}

fn complete_as_won(run: &mut starclock_mode_standard::StandardActivity) {
    let initial = run.activity().state_hash();
    let started = run
        .activity_mut()
        .apply(ActivityCommand::StartBattle {
            expected_state_hash: initial,
        })
        .unwrap();
    let result = BattleResult::seal(
        started.battle_handoff().unwrap().identity(),
        vec![
            ProjectedValue::Outcome(BattleOutcome::Won),
            ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x91; 32])),
            ProjectedValue::EventDigest(EventDigest::new([0x92; 32]).unwrap()),
            ProjectedValue::TerminalFault(None),
        ],
    );
    run.activity_mut()
        .apply(ActivityCommand::SubmitBattleResult {
            expected_state_hash: started.state_hash(),
            result: Box::new(result),
        })
        .unwrap();
}

fn standard_scenario(outcome: StandardExpectedOutcome) -> StandardScenario {
    StandardScenario::new(
        scenario_id(3),
        profile(4),
        StandardActivityBinding::new(
            binding_id(4),
            activity_spec(1, participant_policy(1, 4), standard_projection()),
        ),
        "0123456789abcdef",
        outcome,
    )
    .unwrap()
}

fn profile(maximum_party_size: u8) -> StandardProfile {
    StandardProfile::new(
        profile_id(2),
        activity_id(1),
        maximum_party_size,
        WaveTransitionPolicy::AfterAction,
    )
    .unwrap()
}

fn participant_policy(team_count: u8, maximum_team_size: u8) -> ParticipantPolicy {
    ParticipantPolicy::new(
        team_count,
        1,
        maximum_team_size,
        ParticipantUniquenessScope::Team,
        LoadoutLockScope::Attempt,
    )
    .unwrap()
}

fn standard_projection() -> BattleResultProjection {
    BattleResultProjection::new(
        projection_id(1),
        vec![
            ProjectionField::Outcome,
            ProjectionField::FinalStateHash,
            ProjectionField::EventDigest,
            ProjectionField::TerminalFault,
        ],
    )
    .unwrap()
}

fn activity_spec(
    definition_id: u32,
    policy: ParticipantPolicy,
    projection: BattleResultProjection,
) -> ActivitySpec {
    let entries = (0..policy.team_count())
        .map(|team| participant_entry(u32::from(team) + 1, team))
        .collect();
    let participants = ParticipantLock::seal(policy, entries).unwrap();
    let binding = BattleBinding::new(
        battle_spec(),
        "standard-battle",
        "starclock.battle-spec.v1",
        participants.digest(),
    )
    .unwrap();
    ActivitySpec::new(
        ActivityDefinitionIdentity::new(
            activity_id(definition_id),
            ActivityDefinitionDigest::new([0x21; 32]).unwrap(),
            ActivityConfigDigest::new([0x22; 32]).unwrap(),
        ),
        OneBattleFlow::new(
            starclock_activity::SectionId::new(1).unwrap(),
            starclock_activity::NodeId::new(1).unwrap(),
            starclock_activity::NodeId::new(2).unwrap(),
            starclock_activity::NodeId::new(3).unwrap(),
            starclock_activity::NodeId::new(4).unwrap(),
        )
        .unwrap(),
        vec![
            ActivitySlotDefinition::new(
                starclock_activity::ActivitySlotId::new(1).unwrap(),
                starclock_activity::ActivityScope::Activity,
                ActivityValue::BoundedInteger(0),
                Some((0, 9)),
                vec![SlotResetPoint::ActivityStart],
            )
            .unwrap(),
        ],
        participants,
        projection,
        binding,
    )
    .unwrap()
}

fn participant_entry(raw: u32, team: u8) -> ParticipantLockEntry {
    ParticipantLockEntry::new(
        ParticipantId::new(raw).unwrap(),
        team,
        0,
        UnitDefinitionId::new(raw).unwrap(),
        OpaqueParticipantBuild::new(
            CombatantSpecDigest::new([0x50 + team; 32]).unwrap(),
            BuildDigest::new([0x60 + team; 32]).unwrap(),
            "build-catalog-v1",
            ParticipantSourceKind::CompiledBuild,
        )
        .unwrap(),
    )
    .unwrap()
}

fn battle_spec() -> BattleSpec {
    BattleSpec::new(
        "combat-rules-v1",
        BattleSpecDigest::new([0x31; 32]).unwrap(),
        EncounterId::new(1).unwrap(),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 1, 1_000, 200_000_000, 0x51),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(EnemyDefinitionId::new(1).unwrap()),
                combatant(9, 2, 600, 50_000_000, 0x59),
            ),
        ],
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap()
}

fn combatant(form: u32, ability: u32, hp: i64, speed: i64, digest: u8) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        UnitDefinitionId::new(form).unwrap(),
        UnitLevel::new(80).unwrap(),
        Hp::new(hp).unwrap(),
        Speed::from_scaled(speed).unwrap(),
        ResolvedDefinitionBindings::new(vec![AbilityId::new(ability).unwrap()], vec![], vec![])
            .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
}

fn activity_id(raw: u32) -> ActivityDefinitionId {
    ActivityDefinitionId::new(raw).unwrap()
}
fn profile_id(raw: u32) -> StandardProfileId {
    StandardProfileId::new(raw).unwrap()
}
fn scenario_id(raw: u32) -> StandardScenarioId {
    StandardScenarioId::new(raw).unwrap()
}
fn binding_id(raw: u32) -> StandardBindingId {
    StandardBindingId::new(raw).unwrap()
}
fn projection_id(raw: u32) -> ProjectionId {
    ProjectionId::new(raw).unwrap()
}
