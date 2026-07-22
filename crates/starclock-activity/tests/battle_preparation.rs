use std::sync::Arc;

use starclock_activity::{
    ActivityBattlePreparationRequest, ActivityConfigDigest, ActivityDefinitionDigest,
    ActivityDefinitionId, ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed,
    ActivityOptionId, ActivityPreparationBoundary, ActivityPreparationError, ActivityRngContext,
    ActivityRngStreams, ActivityRosterLock, ActivityScopePath, ActivityStateDefinition,
    ActivityTransactionState, BattleBinding, BattleSequence, BuildDigest,
    EncounterInitiativePolicy, EncounterPreparationDefinition, EncounterPreparationDefinitionError,
    LoadoutLockScope, OneBattleFlow, OpaqueParticipantBuild, ParticipantId, ParticipantLock,
    ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
    PreparedBattleVariant, TechniqueContributionDigest, TechniqueEngagement,
    TechniqueOptionDefinition,
};
use starclock_combat::{
    AbilityId, BattleSpec, BattleSpecDigest, CombatantSpecDigest, ConcedePolicy, EncounterId,
    EnemyDefinitionId, FormationIndex, Hp, ParticipantSource, ParticipantSpec,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed, TeamResourceSpec, TeamSide,
    UnitDefinitionId, UnitLevel,
};

#[test]
fn accumulated_and_attacking_techniques_select_one_exact_immutable_battle_variant() {
    let graph = graph();
    let instance = instance();
    let roster = roster_lock(instance, participants());
    let lock_digest = roster.digest();
    let definition = Arc::new(definition(
        lock_digest,
        EncounterInitiativePolicy::PlayerControlled,
        vec![
            technique(11, 1, TechniqueEngagement::Accumulate),
            technique(12, 2, TechniqueEngagement::Engage),
        ],
        vec![
            variant(lock_digest, &[], 0x31),
            variant(lock_digest, &[11], 0x32),
            variant(lock_digest, &[12], 0x33),
            variant(lock_digest, &[11, 12], 0x34),
        ],
    ));
    let mut state = state();
    assert_eq!(
        definition.digest().bytes(),
        [
            3, 136, 252, 52, 42, 133, 45, 105, 16, 218, 181, 70, 27, 48, 66, 92, 145, 136, 193,
            179, 151, 128, 134, 22, 235, 237, 165, 181, 141, 139, 133, 83,
        ]
    );
    assert_eq!(
        state
            .begin_battle_preparation(
                instance,
                &graph,
                request(instance, roster, Arc::clone(&definition), 2)
            )
            .unwrap(),
        ActivityPreparationBoundary::Decision
    );
    let offered = state.preparation_view().unwrap();
    assert_eq!(
        offered
            .options()
            .iter()
            .map(|option| option.id().get())
            .collect::<Vec<_>>(),
        [10, 11, 12]
    );

    assert_eq!(
        state.choose_preparation_option(option(11)).unwrap(),
        ActivityPreparationBoundary::Decision
    );
    let after_first = state.preparation_view().unwrap();
    assert_eq!(after_first.remaining_points(), 1);
    assert_eq!(after_first.selected(), [option(11)]);
    let before_rejection = state.state_hash(identity(), &graph, instance, &rng());
    assert_eq!(
        state.choose_preparation_option(option(11)),
        Err(ActivityPreparationError::DecisionNotOffered)
    );
    assert_eq!(
        state.state_hash(identity(), &graph, instance, &rng()),
        before_rejection
    );

    assert_eq!(
        state.choose_preparation_option(option(12)).unwrap(),
        ActivityPreparationBoundary::BattleReady
    );
    assert!(state.preparation_view().is_none());
    let pending = state.pending_battle().unwrap();
    assert_eq!(pending.techniques(), [option(11), option(12)]);
    assert_eq!(pending.remaining_technique_points(), 0);
    assert_eq!(pending.battle_spec_digest().bytes(), [0x34; 32]);
    assert_eq!(pending.participant_lock_digest(), lock_digest);
    let player = state.player_view(identity(), &graph, instance, &rng());
    let pending_state_hash = state.state_hash(identity(), &graph, instance, &rng());
    assert_eq!(
        pending_state_hash.bytes(),
        [
            159, 176, 244, 0, 6, 149, 230, 20, 115, 192, 123, 19, 213, 108, 61, 0, 192, 133, 254,
            203, 192, 94, 165, 113, 198, 104, 65, 181, 57, 18, 16, 20,
        ]
    );
    assert_eq!(
        player.pending_battle().unwrap().battle_spec_digest(),
        pending.battle_spec_digest()
    );
}

#[test]
fn normal_engagement_uses_the_variant_for_the_exact_accumulated_sequence() {
    let graph = graph();
    let instance = instance();
    let roster = roster_lock(instance, participants());
    let lock_digest = roster.digest();
    let definition = Arc::new(definition(
        lock_digest,
        EncounterInitiativePolicy::PlayerControlled,
        vec![technique(11, 1, TechniqueEngagement::Accumulate)],
        vec![
            variant(lock_digest, &[], 0x41),
            variant(lock_digest, &[11], 0x42),
        ],
    ));
    let mut state = state();
    state
        .begin_battle_preparation(instance, &graph, request(instance, roster, definition, 1))
        .unwrap();
    state.choose_preparation_option(option(11)).unwrap();
    state.choose_preparation_option(option(10)).unwrap();
    let pending = state.pending_battle().unwrap();
    assert_eq!(pending.techniques(), [option(11)]);
    assert_eq!(pending.battle_spec_digest().bytes(), [0x42; 32]);
}

#[test]
fn enemy_preemptive_policy_skips_player_preparation_and_is_data_driven() {
    let graph = graph();
    let instance = instance();
    let roster = roster_lock(instance, participants());
    let lock_digest = roster.digest();
    let definition = Arc::new(definition(
        lock_digest,
        EncounterInitiativePolicy::EnemyPreemptive,
        vec![],
        vec![variant(lock_digest, &[], 0x51)],
    ));
    let mut state = state();
    assert_eq!(
        state
            .begin_battle_preparation(instance, &graph, request(instance, roster, definition, 3))
            .unwrap(),
        ActivityPreparationBoundary::BattleReady
    );
    assert!(state.preparation_view().is_none());
    assert_eq!(
        state.pending_battle().unwrap().initiative(),
        EncounterInitiativePolicy::EnemyPreemptive
    );

    let error = EncounterPreparationDefinition::new(
        option(10),
        EncounterInitiativePolicy::EnemyPreemptive,
        lock_digest,
        0,
        vec![technique(11, 1, TechniqueEngagement::Accumulate)],
        vec![variant(lock_digest, &[], 0x52)],
    )
    .unwrap_err();
    assert_eq!(
        error,
        EncounterPreparationDefinitionError::PreemptiveOffersTechnique
    );
}

#[test]
fn roster_and_variant_validation_reject_mismatch_without_mutating_attempt_state() {
    let graph = graph();
    let instance = instance();
    let roster = roster_lock(instance, participants());
    let lock_digest = roster.digest();
    let bad_variant = variant_with_player_digest(lock_digest, &[], 0x61, 0xee);
    let definition = Arc::new(definition(
        lock_digest,
        EncounterInitiativePolicy::PlayerControlled,
        vec![],
        vec![bad_variant],
    ));
    let mut state = state();
    let before = state.state_hash(identity(), &graph, instance, &rng());
    assert_eq!(
        state.begin_battle_preparation(instance, &graph, request(instance, roster, definition, 0)),
        Err(ActivityPreparationError::BattleParticipantMismatch)
    );
    assert!(state.pending_battle().is_none());
    assert_eq!(
        state.state_hash(identity(), &graph, instance, &rng()),
        before
    );
}

#[test]
fn preparation_definition_requires_prefix_closed_reachable_sequences() {
    let lock = participants().digest();
    let missing_prefix = EncounterPreparationDefinition::new(
        option(10),
        EncounterInitiativePolicy::PlayerControlled,
        lock,
        0,
        vec![
            technique(11, 1, TechniqueEngagement::Accumulate),
            technique(12, 2, TechniqueEngagement::Engage),
        ],
        vec![variant(lock, &[], 0x71), variant(lock, &[11, 12], 0x72)],
    )
    .unwrap_err();
    assert_eq!(
        missing_prefix,
        EncounterPreparationDefinitionError::MissingPrefixVariant
    );

    let after_engage = EncounterPreparationDefinition::new(
        option(10),
        EncounterInitiativePolicy::PlayerControlled,
        lock,
        0,
        vec![
            technique(11, 1, TechniqueEngagement::Engage),
            technique(12, 2, TechniqueEngagement::Accumulate),
        ],
        vec![
            variant(lock, &[], 0x73),
            variant(lock, &[11], 0x74),
            variant(lock, &[11, 12], 0x75),
        ],
    )
    .unwrap_err();
    assert_eq!(
        after_engage,
        EncounterPreparationDefinitionError::SequenceAfterEngagement
    );
}

fn request(
    instance: ActivityInstanceId,
    roster: ActivityRosterLock,
    definition: Arc<EncounterPreparationDefinition>,
    points: u16,
) -> ActivityBattlePreparationRequest {
    ActivityBattlePreparationRequest::new(
        ActivityScopePath::new(instance)
            .enter_section(section())
            .unwrap()
            .enter_node(node())
            .unwrap()
            .enter_attempt(starclock_activity::AttemptId::new(1).unwrap())
            .unwrap(),
        roster,
        BattleSequence::new(1).unwrap(),
        points,
        definition,
    )
}

fn definition(
    lock: starclock_activity::ParticipantLockDigest,
    initiative: EncounterInitiativePolicy,
    techniques: Vec<TechniqueOptionDefinition>,
    variants: Vec<PreparedBattleVariant>,
) -> EncounterPreparationDefinition {
    EncounterPreparationDefinition::new(option(10), initiative, lock, 0, techniques, variants)
        .unwrap()
}

fn technique(
    option_id: u64,
    participant: u32,
    engagement: TechniqueEngagement,
) -> TechniqueOptionDefinition {
    TechniqueOptionDefinition::new(
        option(option_id),
        participant_id(participant),
        1,
        engagement,
    )
    .unwrap()
}

fn variant(
    lock: starclock_activity::ParticipantLockDigest,
    sequence: &[u64],
    digest: u8,
) -> PreparedBattleVariant {
    variant_with_player_digest(lock, sequence, digest, 0x81)
}

fn variant_with_player_digest(
    lock: starclock_activity::ParticipantLockDigest,
    sequence: &[u64],
    digest: u8,
    first_player_digest: u8,
) -> PreparedBattleVariant {
    PreparedBattleVariant::new(
        sequence.iter().copied().map(option).collect(),
        TechniqueContributionDigest::new([digest.wrapping_add(1); 32]).unwrap(),
        BattleBinding::new(
            battle_spec(digest, first_player_digest),
            "battle",
            "battle-spec-v1",
            lock,
        )
        .unwrap(),
    )
}

fn participants() -> ParticipantLock {
    ParticipantLock::seal(
        ParticipantPolicy::new(
            1,
            2,
            2,
            ParticipantUniquenessScope::Activity,
            LoadoutLockScope::Activity,
        )
        .unwrap(),
        vec![lock_entry(1, 0, 101, 0x81), lock_entry(2, 1, 102, 0x82)],
    )
    .unwrap()
}

fn roster_lock(instance: ActivityInstanceId, lock: ParticipantLock) -> ActivityRosterLock {
    ActivityRosterLock::new(ActivityScopePath::new(instance), lock).unwrap()
}

fn lock_entry(
    participant: u32,
    formation: u8,
    character: u32,
    resolved: u8,
) -> ParticipantLockEntry {
    ParticipantLockEntry::new(
        participant_id(participant),
        0,
        formation,
        UnitDefinitionId::new(character).unwrap(),
        OpaqueParticipantBuild::new(
            CombatantSpecDigest::new([resolved; 32]).unwrap(),
            BuildDigest::new([resolved.wrapping_add(1); 32]).unwrap(),
            "build-v1",
            ParticipantSourceKind::CompiledBuild,
        )
        .unwrap(),
    )
    .unwrap()
}

fn battle_spec(digest: u8, first_player_digest: u8) -> BattleSpec {
    BattleSpec::new(
        "rules-v1",
        BattleSpecDigest::new([digest; 32]).unwrap(),
        EncounterId::new(1).unwrap(),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(101, first_player_digest),
            ),
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(1).unwrap(),
                ParticipantSource::Player,
                combatant(102, 0x82),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(EnemyDefinitionId::new(201).unwrap()),
                combatant(201, 0x91),
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap()
}

fn combatant(form: u32, digest: u8) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        UnitDefinitionId::new(form).unwrap(),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(100_000_000).unwrap(),
        ResolvedDefinitionBindings::new(vec![AbilityId::new(form).unwrap()], vec![], vec![])
            .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
}

fn graph() -> starclock_activity::ActivityGraphDefinition {
    OneBattleFlow::new(
        section(),
        node(),
        starclock_activity::NodeId::new(21).unwrap(),
        starclock_activity::NodeId::new(22).unwrap(),
        starclock_activity::NodeId::new(23).unwrap(),
    )
    .unwrap()
    .into_graph()
}

fn state() -> ActivityTransactionState {
    ActivityTransactionState::new(
        ActivityStateDefinition::new(vec![], vec![], vec![]).unwrap(),
        node(),
    )
}

fn rng() -> ActivityRngStreams {
    let graph = graph();
    ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(5),
        identity().id(),
        identity().definition_digest(),
        identity().config_digest(),
        graph.digest(),
        instance(),
        Some(section()),
        Some(node()),
        Some(starclock_activity::AttemptId::new(1).unwrap()),
        1,
    ))
}

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(1).unwrap(),
        ActivityDefinitionDigest::new([0xa1; 32]).unwrap(),
        ActivityConfigDigest::new([0xa2; 32]).unwrap(),
    )
}
fn instance() -> ActivityInstanceId {
    ActivityInstanceId::new(7).unwrap()
}
fn option(value: u64) -> ActivityOptionId {
    ActivityOptionId::new(value).unwrap()
}
fn participant_id(value: u32) -> ParticipantId {
    ParticipantId::new(value).unwrap()
}
fn section() -> starclock_activity::SectionId {
    starclock_activity::SectionId::new(10).unwrap()
}
fn node() -> starclock_activity::NodeId {
    starclock_activity::NodeId::new(20).unwrap()
}
