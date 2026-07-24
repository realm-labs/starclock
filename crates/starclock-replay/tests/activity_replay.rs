use starclock_activity::{
    Activity, ActivityCommand, ActivityConfigDigest, ActivityDefinitionDigest,
    ActivityDefinitionId, ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed,
    ActivityPhase, ActivitySlotDefinition, ActivitySlotId, ActivitySpec, ActivityValue,
    BattleBinding, BattleOutcome, BattleResult, BattleResultProjection, BuildDigest, EventDigest,
    LoadoutLockScope, OneBattleFlow, OpaqueParticipantBuild, ParticipantId, ParticipantLock,
    ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
    ProjectedValue, ProjectionField, ProjectionId, SlotResetPoint,
};
use starclock_combat::{
    AbilityId, BattleSpec, BattleSpecDigest, BattleStateHash, CombatantSpecDigest, ConcedePolicy,
    EncounterId, EnemyDefinitionId, FormationIndex, Hp, ParticipantSource, ParticipantSpec,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed, TeamResourceSpec, TeamSide,
    UnitDefinitionId, UnitLevel,
};
use starclock_replay::{
    activity::{
        ActivityIdentityField, ActivityReplayError, ActivityTraceEntry, ControllerDecisionKind,
        ControllerDiagnostic, ControllerOptionScore, NestedBattleBoundary, activity_record_count,
        encode_activity_trace, verify_activity_replay,
    },
    digest::{ConfigBundleDigest, ControllerDigest, DefinitionDigest, EntrySpecDigest},
    format::{
        ControllerIdentity, ReplayEntry, ReplayHeader, ReplayIdentity, STATE_HASH_REVISION,
        decode_replay,
    },
    record::RecordKind,
};

const DEFINITION_DIGEST: [u8; 32] = [0x11; 32];
const CONFIG_DIGEST: [u8; 32] = [0x12; 32];
const SPEC_DIGEST: [u8; 32] = [0x33; 32];

#[test]
fn activity_trace_round_trips_with_nested_boundaries_and_diagnostics() {
    let fixture = replay_fixture();
    let report = verify_activity_replay(&fixture.bytes, activity(), "standard-v1")
        .expect("the canonical activity replay verifies");

    assert_eq!(report.command_count(), 2);
    assert_eq!(report.diagnostic_count(), 2);
    assert_eq!(
        report.phase(),
        ActivityPhase::Terminal(starclock_activity::TerminalOutcome::Complete)
    );
    assert_eq!(report.final_hash().bytes(), fixture.final_hash);

    let digest = starclock_replay::codec::hash_canonical(&ReplayBytes(&fixture.bytes))
        .expect("replay bytes hash");
    assert_eq!(
        digest.bytes(),
        [
            217, 100, 2, 166, 164, 5, 171, 15, 250, 191, 71, 75, 46, 127, 140, 80, 227, 87, 176, 0,
            140, 97, 220, 241, 161, 66, 180, 87, 229, 190, 181, 254,
        ]
    );
}

#[test]
fn verification_reports_the_first_state_and_nested_boundary_divergence() {
    let fixture = replay_fixture();
    let mut state_corrupt = fixture.bytes.clone();
    let state_offset = payload_offset(&state_corrupt, RecordKind::ExpectedActivityState, 0);
    state_corrupt[state_offset] ^= 0x80;
    assert!(matches!(
        verify_activity_replay(&state_corrupt, activity(), "standard-v1"),
        Err(ActivityReplayError::StateDivergence {
            command_index: 0,
            ..
        })
    ));

    let mut start_corrupt = fixture.bytes.clone();
    let start_offset = payload_offset(&start_corrupt, RecordKind::NestedBattleStart, 0);
    start_corrupt[start_offset + 2] ^= 1;
    assert!(matches!(
        verify_activity_replay(&start_corrupt, activity(), "standard-v1"),
        Err(ActivityReplayError::NestedStartDivergence {
            command_index: 0,
            ..
        })
    ));

    let mut end_corrupt = fixture.bytes.clone();
    let end_offset = payload_offset(&end_corrupt, RecordKind::NestedBattleEnd, 0);
    end_corrupt[end_offset + 2] ^= 1;
    assert!(matches!(
        verify_activity_replay(&end_corrupt, activity(), "standard-v1"),
        Err(ActivityReplayError::NestedEndDivergence {
            command_index: 1,
            ..
        })
    ));
}

#[test]
fn verification_rejects_wrong_profile_before_executing_commands() {
    let fixture = replay_fixture();
    assert_eq!(
        verify_activity_replay(&fixture.bytes, activity(), "other-profile"),
        Err(ActivityReplayError::IdentityMismatch(
            ActivityIdentityField::Profile
        ))
    );
}

struct Fixture {
    bytes: Vec<u8>,
    final_hash: [u8; 32],
}

fn replay_fixture() -> Fixture {
    let mut recorder = activity();
    let initial_hash = recorder.state_hash();
    let start_command = ActivityCommand::StartBattle {
        expected_state_hash: initial_hash,
    };
    let started = recorder
        .apply(start_command.clone())
        .expect("start accepted");
    let identity = started
        .battle_handoff()
        .expect("handoff returned")
        .identity();
    let start_hash = started.state_hash();
    let result = BattleResult::seal(identity, successful_values());
    let result_digest = result.claimed_digest();
    let submit_command = ActivityCommand::SubmitBattleResult {
        expected_state_hash: start_hash,
        result: Box::new(result),
    };
    let terminal = recorder
        .apply(submit_command.clone())
        .expect("result accepted");
    let final_hash = terminal.state_hash().bytes();

    let trace = vec![
        ActivityTraceEntry::new(
            start_command,
            start_hash,
            NestedBattleBoundary::Start(identity),
            Some(diagnostic(ControllerDecisionKind::Activity, 0, &[(0, 10)])),
        )
        .expect("start trace entry"),
        ActivityTraceEntry::new(
            submit_command,
            terminal.state_hash(),
            NestedBattleBoundary::End(result_digest),
            Some(diagnostic(
                ControllerDecisionKind::BattlePlayer,
                1,
                &[(0, -5), (1, 20)],
            )),
        )
        .expect("end trace entry"),
    ];
    let header = header(activity_record_count(&trace).expect("bounded record count"));
    let bytes = encode_activity_trace(&header, &trace).expect("trace encodes");
    Fixture { bytes, final_hash }
}

fn diagnostic(
    kind: ControllerDecisionKind,
    sequence: u64,
    scores: &[(u32, i64)],
) -> ControllerDiagnostic {
    let selected = scores
        .iter()
        .max_by_key(|(ordinal, score)| (*score, core::cmp::Reverse(*ordinal)))
        .expect("at least one score")
        .0;
    ControllerDiagnostic::new(
        kind,
        sequence,
        selected,
        Some(sequence + 2),
        scores
            .iter()
            .map(|(ordinal, score)| ControllerOptionScore::new(*ordinal, *score))
            .collect(),
    )
    .expect("valid diagnostic")
}

fn header(record_count: u32) -> ReplayHeader {
    let identity = ReplayIdentity::new(
        "4.4",
        "standard-rules-v1",
        "catalog-v4.4",
        ConfigBundleDigest::new(CONFIG_DIGEST),
        "fixed-i64-6dp-v1",
        "chacha8-rand-0.10.2-intmap-v1",
        STATE_HASH_REVISION,
    )
    .expect("identity valid");
    let controller =
        ControllerIdentity::new("baseline-controller-v1", ControllerDigest::new([0x22; 32]))
            .expect("controller valid");
    ReplayHeader::new(
        identity,
        controller,
        0x5eed,
        ReplayEntry::Activity {
            profile_id: "standard-v1".into(),
            definition_id: 1,
            definition_digest: DefinitionDigest::new(DEFINITION_DIGEST),
            spec_digest: EntrySpecDigest::new(SPEC_DIGEST),
            builds: None,
        },
        record_count,
    )
    .expect("header valid")
}

fn payload_offset(bytes: &[u8], kind: RecordKind, ordinal: usize) -> usize {
    let decoded = decode_replay(bytes).expect("fixture decodes");
    let payload = decoded
        .records()
        .iter()
        .filter(|record| record.kind() == kind)
        .nth(ordinal)
        .expect("record exists")
        .payload();
    payload.as_ptr() as usize - bytes.as_ptr() as usize
}

struct ReplayBytes<'a>(&'a [u8]);

impl starclock_replay::codec::CanonicalEncode for ReplayBytes<'_> {
    fn encode<S: starclock_replay::codec::CanonicalSink>(
        &self,
        encoder: &mut starclock_replay::codec::Encoder<S>,
    ) -> Result<(), starclock_replay::codec::CodecError> {
        encoder.raw(self.0);
        Ok(())
    }
}

fn activity() -> Activity {
    Activity::new(
        activity_spec(),
        ActivityInstanceId::new(7).unwrap(),
        ActivityMasterSeed::from_u64(0x5eed),
    )
}

fn activity_spec() -> ActivitySpec {
    let participants =
        ParticipantLock::seal(participant_policy(), vec![participant_entry()]).unwrap();
    let projection = BattleResultProjection::new(
        id::<ProjectionId>(1),
        vec![
            ProjectionField::Outcome,
            ProjectionField::FinalStateHash,
            ProjectionField::EventDigest,
            ProjectionField::TerminalFault,
        ],
    )
    .unwrap();
    let binding = BattleBinding::new(
        battle_spec(),
        "battle",
        "battle-spec-policy-v1",
        participants.digest(),
    )
    .unwrap();
    ActivitySpec::new(
        ActivityDefinitionIdentity::new(
            id::<ActivityDefinitionId>(1),
            ActivityDefinitionDigest::new(DEFINITION_DIGEST).unwrap(),
            ActivityConfigDigest::new(CONFIG_DIGEST).unwrap(),
        ),
        OneBattleFlow::new(
            starclock_activity::SectionId::new(10).unwrap(),
            starclock_activity::NodeId::new(20).unwrap(),
            starclock_activity::NodeId::new(21).unwrap(),
            starclock_activity::NodeId::new(22).unwrap(),
            starclock_activity::NodeId::new(23).unwrap(),
        )
        .unwrap(),
        vec![
            ActivitySlotDefinition::new(
                id::<ActivitySlotId>(1),
                starclock_activity::ActivityScope::Activity,
                ActivityValue::BoundedInteger(3),
                Some((0, 9)),
                vec![SlotResetPoint::ActivityStart, SlotResetPoint::BattleEnd],
            )
            .unwrap(),
        ],
        participants,
        projection,
        binding,
    )
    .unwrap()
}

fn participant_policy() -> ParticipantPolicy {
    ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .unwrap()
}

fn participant_entry() -> ParticipantLockEntry {
    ParticipantLockEntry::new(
        id::<ParticipantId>(1),
        0,
        0,
        UnitDefinitionId::new(1).unwrap(),
        OpaqueParticipantBuild::new(
            CombatantSpecDigest::new([0x55; 32]).unwrap(),
            BuildDigest::new([0x44; 32]).unwrap(),
            "build-catalog-v1",
            ParticipantSourceKind::CompiledBuild,
        )
        .unwrap(),
    )
    .unwrap()
}

fn successful_values() -> Vec<ProjectedValue> {
    vec![
        ProjectedValue::Outcome(BattleOutcome::Won),
        ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x71; 32])),
        ProjectedValue::EventDigest(EventDigest::new([0x72; 32]).unwrap()),
        ProjectedValue::TerminalFault(None),
    ]
}

fn battle_spec() -> BattleSpec {
    BattleSpec::new(
        "combat-rules-v1",
        BattleSpecDigest::new(SPEC_DIGEST).unwrap(),
        EncounterId::new(1).unwrap(),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 1, 1_000, 200_000_000, 0x55),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(EnemyDefinitionId::new(1).unwrap()),
                combatant(2, 2, 600, 50_000_000, 0x56),
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

trait TestId: Sized {
    fn new_test(value: u32) -> Option<Self>;
}

macro_rules! test_id {
    ($($kind:ty),+ $(,)?) => { $(impl TestId for $kind { fn new_test(value: u32) -> Option<Self> { Self::new(value) } })+ };
}

test_id!(
    ActivityDefinitionId,
    ActivitySlotId,
    ParticipantId,
    ProjectionId
);

fn id<T: TestId>(value: u32) -> T {
    T::new_test(value).unwrap()
}
