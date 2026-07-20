use std::{fs, path::Path};

use starclock_activity::{
    Activity, ActivityCommand, ActivityCommandErrorKind, ActivityConfigDigest,
    ActivityDefinitionDigest, ActivityDefinitionId, ActivityDefinitionIdentity, ActivityInstanceId,
    ActivityMasterSeed, ActivityPhase, ActivitySlotDefinition, ActivitySlotId, ActivitySpec,
    ActivityValue, BattleBinding, BattleOutcome, BattleResult, BattleResultConfiguration,
    BattleResultDigest, BattleResultIdentity, BattleResultProjection, BuildDigest, EventDigest,
    LoadoutLockScope, OneBattleFlow, OpaqueParticipantBuild, ParticipantId, ParticipantLock,
    ParticipantLockDigest, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope, ProjectedValue, ProjectionField, ProjectionId, ResultIdentityField,
    SlotDefinitionError, SlotResetPoint, TerminalOutcome,
};
use starclock_combat::{
    AbilityId, BattleSpec, BattleSpecDigest, BattleStateHash, CombatantSpecDigest, ConcedePolicy,
    EncounterId, EnemyDefinitionId, FormationIndex, Hp, ParticipantSource, ParticipantSpec,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed, TeamResourceSpec, TeamSide,
    UnitDefinitionId, UnitLevel,
};

const DEFINITION_DIGEST: [u8; 32] = [0x11; 32];
const CONFIG_DIGEST: [u8; 32] = [0x12; 32];
const SPEC_DIGEST: [u8; 32] = [0x33; 32];

#[test]
fn one_battle_handoff_accepts_only_the_declared_projection_and_reaches_terminal() {
    let mut activity = activity(7);
    assert_eq!(activity.phase(), ActivityPhase::ReadyToStartBattle);
    assert_eq!(activity.scope().activity().get(), 7);
    assert_eq!(activity.scope().section().get(), 10);
    assert_eq!(activity.scope().node().get(), 20);
    assert_eq!(activity.scope().attempt().get(), 1);
    assert_eq!(
        activity.slot_value(id::<ActivitySlotId>(1)),
        Some(&ActivityValue::BoundedInteger(3))
    );

    let initial_hash = activity.state_hash();
    assert_eq!(
        initial_hash.bytes(),
        [
            194, 66, 33, 96, 163, 97, 179, 52, 154, 253, 206, 31, 83, 165, 245, 61, 99, 251, 158,
            92, 16, 37, 154, 52, 233, 147, 124, 215, 202, 125, 239, 108,
        ]
    );
    let started = activity
        .apply(ActivityCommand::StartBattle {
            expected_state_hash: initial_hash,
        })
        .unwrap();
    let handoff = started.battle_handoff().expect("start returns one handoff");
    assert_eq!(handoff.battle_spec().digest().bytes(), SPEC_DIGEST);
    assert_eq!(handoff.seed(), handoff.identity().seed());
    assert_eq!(
        handoff.seed().bytes(),
        [
            29, 167, 11, 212, 239, 51, 161, 77, 120, 138, 102, 156, 242, 170, 138, 245, 228, 43,
            152, 181, 28, 11, 47, 202, 231, 110, 158, 235, 164, 157, 142, 70,
        ]
    );
    assert_ne!(started.state_hash(), initial_hash);
    assert_eq!(
        started.state_hash().bytes(),
        [
            237, 237, 61, 51, 73, 192, 225, 120, 19, 100, 97, 187, 155, 192, 52, 68, 41, 227, 86,
            217, 219, 245, 36, 255, 116, 103, 85, 236, 112, 80, 6, 56,
        ]
    );
    assert_eq!(activity.phase(), ActivityPhase::AwaitingBattleResult);

    let result = successful_result(handoff.identity());
    let terminal = activity
        .apply(ActivityCommand::SubmitBattleResult {
            expected_state_hash: started.state_hash(),
            result: Box::new(result),
        })
        .unwrap();
    assert_eq!(
        activity.phase(),
        ActivityPhase::Terminal(TerminalOutcome::Complete)
    );
    assert_eq!(activity.current_node().get(), 21);
    assert_eq!(terminal.battle_handoff(), None);
    assert_ne!(terminal.state_hash(), started.state_hash());
    assert_eq!(
        terminal.state_hash().bytes(),
        [
            89, 89, 16, 147, 45, 104, 234, 28, 95, 159, 115, 251, 223, 49, 190, 160, 16, 9, 9, 54,
            217, 13, 110, 236, 68, 2, 162, 212, 86, 87, 52, 122,
        ]
    );
    assert!(matches!(
        terminal.events().last(),
        Some(starclock_activity::ActivityEvent::Terminal(
            TerminalOutcome::Complete
        ))
    ));
}

#[test]
fn rejected_results_and_stale_commands_preserve_the_complete_activity_hash() {
    let mut activity = activity(9);
    let initial = activity.state_hash();
    let stale = ActivityConfigDigest::new([0xa0; 32]).unwrap();
    let error = activity
        .apply(ActivityCommand::StartBattle {
            expected_state_hash: starclock_activity::ActivityStateHash::new(stale.bytes()).unwrap(),
        })
        .unwrap_err();
    assert_eq!(error.kind(), ActivityCommandErrorKind::StaleStateHash);
    assert_eq!(activity.state_hash(), initial);

    let started = activity
        .apply(ActivityCommand::StartBattle {
            expected_state_hash: initial,
        })
        .unwrap();
    let identity = started.battle_handoff().unwrap().identity();
    let awaiting = activity.state_hash();

    let wrong_identity = BattleResultIdentity::new(
        identity.scope(),
        identity.battle_sequence(),
        BattleResultConfiguration::new(
            identity.definition_digest(),
            ActivityConfigDigest::new([0xfe; 32]).unwrap(),
            identity.participant_lock_digest(),
        ),
        identity.spec_digest(),
        identity.seed(),
    );
    let error = activity
        .apply(ActivityCommand::SubmitBattleResult {
            expected_state_hash: awaiting,
            result: Box::new(successful_result(wrong_identity)),
        })
        .unwrap_err();
    assert_eq!(
        error.kind(),
        ActivityCommandErrorKind::ResultIdentityMismatch(ResultIdentityField::ConfigDigest)
    );
    assert_eq!(activity.state_hash(), awaiting);

    let values = successful_values();
    let wrong_digest = BattleResult::new(
        identity,
        values,
        BattleResultDigest::new([0xee; 32]).unwrap(),
    );
    let error = activity
        .apply(ActivityCommand::SubmitBattleResult {
            expected_state_hash: awaiting,
            result: Box::new(wrong_digest),
        })
        .unwrap_err();
    assert_eq!(error.kind(), ActivityCommandErrorKind::ResultDigestMismatch);
    assert_eq!(activity.state_hash(), awaiting);

    let undeclared = BattleResult::seal(
        identity,
        vec![
            ProjectedValue::Outcome(BattleOutcome::Won),
            ProjectedValue::EventDigest(EventDigest::new([0x72; 32]).unwrap()),
            ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x71; 32])),
            ProjectedValue::TerminalFault(None),
        ],
    );
    let error = activity
        .apply(ActivityCommand::SubmitBattleResult {
            expected_state_hash: awaiting,
            result: Box::new(undeclared),
        })
        .unwrap_err();
    assert_eq!(error.kind(), ActivityCommandErrorKind::ProjectionMismatch);
    assert_eq!(activity.state_hash(), awaiting);

    let inconsistent = BattleResult::seal(
        identity,
        vec![
            ProjectedValue::Outcome(BattleOutcome::Faulted),
            ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x71; 32])),
            ProjectedValue::EventDigest(EventDigest::new([0x72; 32]).unwrap()),
            ProjectedValue::TerminalFault(None),
        ],
    );
    let error = activity
        .apply(ActivityCommand::SubmitBattleResult {
            expected_state_hash: awaiting,
            result: Box::new(inconsistent),
        })
        .unwrap_err();
    assert_eq!(error.kind(), ActivityCommandErrorKind::OutcomeFaultMismatch);
    assert_eq!(activity.state_hash(), awaiting);
}

#[test]
fn slot_owners_and_resets_stop_at_the_attempt_boundary() {
    let slot = ActivitySlotDefinition::new(
        id::<ActivitySlotId>(1),
        starclock_activity::ActivityScope::Activity,
        ActivityValue::BoundedInteger(5),
        Some((0, 10)),
        vec![SlotResetPoint::ActivityStart, SlotResetPoint::BattleEnd],
    )
    .unwrap();
    assert_eq!(slot.resets().len(), 2);

    let error = ActivitySlotDefinition::new(
        id::<ActivitySlotId>(2),
        starclock_activity::ActivityScope::Attempt,
        ActivityValue::Boolean(false),
        None,
        vec![SlotResetPoint::SectionStart],
    )
    .unwrap_err();
    assert_eq!(error, SlotDefinitionError::ResetBeforeOwnerLifetime);

    let error = ActivitySlotDefinition::new(
        id::<ActivitySlotId>(3),
        starclock_activity::ActivityScope::Node,
        ActivityValue::Boolean(false),
        None,
        vec![SlotResetPoint::BattleStart, SlotResetPoint::NodeStart],
    )
    .unwrap_err();
    assert_eq!(error, SlotDefinitionError::NonCanonicalResets);
}

#[test]
fn participant_lock_is_canonical_and_rejects_a_false_claim() {
    let policy = participant_policy();
    let entries = vec![participant_entry()];
    let left = ParticipantLock::seal(policy, entries.clone()).unwrap();
    let right = ParticipantLock::seal(policy, entries).unwrap();
    assert_eq!(left.digest(), right.digest());

    let error = ParticipantLock::new(
        policy,
        vec![participant_entry()],
        ParticipantLockDigest::new([0xfd; 32]).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error,
        starclock_activity::ParticipantLockError::DigestMismatch
    );
}

#[test]
fn combat_manifest_has_no_reverse_activity_dependency() {
    let activity_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let combat_manifest =
        fs::read_to_string(activity_dir.join("../starclock-combat/Cargo.toml")).unwrap();
    let activity_manifest = fs::read_to_string(activity_dir.join("Cargo.toml")).unwrap();
    assert!(!combat_manifest.contains("starclock-activity"));
    assert!(activity_manifest.contains("starclock-combat"));
}

fn activity(instance: u64) -> Activity {
    Activity::new(
        activity_spec(),
        ActivityInstanceId::new(instance).unwrap(),
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
    let slots = vec![
        ActivitySlotDefinition::new(
            id::<ActivitySlotId>(1),
            starclock_activity::ActivityScope::Activity,
            ActivityValue::BoundedInteger(3),
            Some((0, 9)),
            vec![SlotResetPoint::ActivityStart, SlotResetPoint::BattleEnd],
        )
        .unwrap(),
        ActivitySlotDefinition::new(
            id::<ActivitySlotId>(2),
            starclock_activity::ActivityScope::Attempt,
            ActivityValue::Boolean(false),
            None,
            vec![SlotResetPoint::AttemptStart, SlotResetPoint::BattleStart],
        )
        .unwrap(),
    ];
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
        slots,
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

fn successful_result(identity: BattleResultIdentity) -> BattleResult {
    BattleResult::seal(identity, successful_values())
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
    fn from_one(value: u32) -> Option<Self>;
}

macro_rules! test_id {
    ($($kind:ty),+ $(,)?) => { $(impl TestId for $kind { fn from_one(value: u32) -> Option<Self> { Self::new(value) } })+ };
}

test_id!(
    ActivityDefinitionId,
    ActivitySlotId,
    ParticipantId,
    ProjectionId
);

fn id<T: TestId>(value: u32) -> T {
    T::from_one(value).unwrap()
}
