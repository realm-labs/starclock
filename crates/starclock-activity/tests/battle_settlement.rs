use std::sync::Arc;

use starclock_activity::{
    ActivityBattlePreparationRequest, ActivityBattleResultContract, ActivityBattleResultSubmission,
    ActivityBattleSettlementError, ActivityBattleStartRequest, ActivityConfigDigest,
    ActivityDefinitionDigest, ActivityDefinitionId, ActivityDefinitionIdentity,
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId, ActivityGraphDefinition,
    ActivityInstanceId, ActivityMasterSeed, ActivityMetricProjectionBinding,
    ActivityNodeDefinition, ActivityNodeKind, ActivityOptionId, ActivityParticipantCarryDefinition,
    ActivityRngContext, ActivityRngStreams, ActivityRosterLock, ActivityScope, ActivityScopePath,
    ActivitySlotDefinition, ActivitySlotId, ActivityStateDefinition, ActivityStateSource,
    ActivityStateVisibility, ActivityTransactionState, ActivityValue, BattleBinding, BattleOutcome,
    BattleResult, BattleResultProjection, BattleSequence, BuildDigest, EncounterInitiativePolicy,
    EncounterPreparationDefinition, EnergyCarryPolicy, EventDigest, HpCarryPolicy, LifeCarryPolicy,
    LoadoutLockScope, MetricSettlementPolicy, MetricValue, MetricValueKind, NodeId,
    OpaqueParticipantBuild, ParticipantBattleState, ParticipantId, ParticipantLock,
    ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
    PreparedBattleVariant, PresenceCarryPolicy, ProjectedValue, ProjectionField, ProjectionId,
    SectionId, SlotCarryPolicy, TechniqueContributionDigest,
};
use starclock_combat::{
    AbilityId, BattleSpec, BattleSpecDigest, BattleStateHash, CombatantSpecDigest, ConcedePolicy,
    EncounterId, EnemyDefinitionId, Energy, FormationIndex, Hp, LifeState, ParticipantSource,
    ParticipantSpec, PresenceState, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    TeamResourceSpec, TeamSide, UnitDefinitionId, UnitLevel,
};

#[test]
fn verified_result_projects_metrics_and_exact_participant_carry() {
    let setup = Setup::new(false);
    let mut state = setup.state();
    setup.prepare(&mut state, node(20), 1, 1);
    let before = state.state_hash(setup.identity, &setup.graph, setup.instance, &setup.rng);
    let rng_before = setup.rng.snapshots();
    let handoff = state
        .start_pending_battle(
            &setup.graph,
            &setup.rng,
            ActivityBattleStartRequest::new(
                before,
                setup.identity,
                setup.instance,
                Arc::clone(&setup.contract),
            ),
        )
        .unwrap();
    assert_eq!(setup.rng.snapshots(), rng_before);
    assert_eq!(handoff.battle_spec().digest().bytes(), [0x33; 32]);
    assert_eq!(handoff.participant_carry().len(), 1);
    assert_eq!(handoff.participant_carry()[0].participant(), participant(1));
    assert_eq!(handoff.participant_carry()[0].current_hp(), hp(1_000));
    assert_eq!(
        handoff.participant_carry()[0].current_energy(),
        Energy::ZERO
    );
    assert_eq!(
        handoff.contract_digest().bytes(),
        [
            111, 71, 57, 73, 47, 111, 9, 120, 200, 156, 141, 179, 251, 132, 115, 90, 170, 68, 43,
            164, 88, 77, 176, 55, 142, 162, 138, 245, 36, 158, 112, 244,
        ]
    );
    assert_eq!(
        handoff.identity().seed().bytes(),
        [
            121, 121, 24, 248, 126, 92, 84, 44, 250, 191, 126, 114, 108, 83, 15, 226, 84, 165, 125,
            186, 47, 146, 157, 177, 19, 30, 169, 80, 248, 150, 80, 34,
        ]
    );
    let awaiting = state.state_hash(setup.identity, &setup.graph, setup.instance, &setup.rng);
    let result = result(
        handoff.identity(),
        BattleOutcome::Won,
        participant_state(700, 60, LifeState::Alive, PresenceState::Present),
        12,
    );
    let settlement = state
        .submit_pending_battle_result(
            setup.identity,
            &setup.graph,
            setup.instance,
            &setup.rng,
            ActivityBattleResultSubmission::new(awaiting, result),
        )
        .unwrap();
    assert_eq!(settlement.outcome(), BattleOutcome::Won);
    assert_eq!(settlement.target(), node(21));
    assert_eq!(
        state.terminal(),
        Some(starclock_activity::ActivityTerminalOutcome::Completed)
    );
    assert_eq!(
        state.slot(slot(1)),
        Some(&ActivityValue::BoundedInteger(17))
    );
    let carry = state
        .player_view(setup.identity, &setup.graph, setup.instance, &setup.rng)
        .participant_carry()[0];
    assert_eq!(carry.participant(), participant(1));
    assert_eq!(carry.current_hp(), hp(700));
    assert_eq!(carry.current_energy(), energy(60));
    assert_eq!(carry.life(), LifeState::Alive);
    assert_eq!(carry.presence(), PresenceState::Present);
    assert_eq!(setup.rng.snapshots(), rng_before);
    assert_eq!(
        settlement.state_hash().bytes(),
        [
            118, 199, 159, 64, 18, 81, 163, 83, 222, 126, 254, 161, 157, 14, 70, 255, 26, 55, 221,
            72, 194, 112, 113, 107, 163, 189, 29, 135, 226, 6, 198, 121,
        ]
    );
}

#[test]
fn loss_preserves_defeat_and_departure_and_selects_failed_transition() {
    let setup = Setup::new(false);
    let mut state = setup.state();
    setup.prepare(&mut state, node(20), 1, 1);
    let handoff = setup.start(&mut state);
    let awaiting = state.state_hash(setup.identity, &setup.graph, setup.instance, &setup.rng);
    let result = result(
        handoff.identity(),
        BattleOutcome::Lost,
        participant_state(0, 25, LifeState::Defeated, PresenceState::Departed),
        0,
    );
    let settlement = state
        .submit_pending_battle_result(
            setup.identity,
            &setup.graph,
            setup.instance,
            &setup.rng,
            ActivityBattleResultSubmission::new(awaiting, result),
        )
        .unwrap();
    assert_eq!(settlement.target(), node(22));
    assert_eq!(
        settlement.terminal(),
        Some(starclock_activity::ActivityTerminalOutcome::Failed)
    );
    let carry = state
        .player_view(setup.identity, &setup.graph, setup.instance, &setup.rng)
        .participant_carry()[0];
    assert_eq!(carry.current_hp(), hp(0));
    assert_eq!(carry.life(), LifeState::Defeated);
    assert_eq!(carry.presence(), PresenceState::Departed);
}

#[test]
fn stale_forged_and_incompatible_results_preserve_bytes_and_rng() {
    let setup = Setup::new(false);
    let mut state = setup.state();
    setup.prepare(&mut state, node(20), 1, 1);
    let pending = state.state_hash(setup.identity, &setup.graph, setup.instance, &setup.rng);
    let stale = starclock_activity::ActivityStateHash::new([0xfe; 32]).unwrap();
    assert_eq!(
        state.start_pending_battle(
            &setup.graph,
            &setup.rng,
            ActivityBattleStartRequest::new(
                stale,
                setup.identity,
                setup.instance,
                Arc::clone(&setup.contract),
            ),
        ),
        Err(ActivityBattleSettlementError::StaleState)
    );
    assert_eq!(
        state.state_hash(setup.identity, &setup.graph, setup.instance, &setup.rng),
        pending
    );

    let handoff = setup.start(&mut state);
    let awaiting = state.state_hash(setup.identity, &setup.graph, setup.instance, &setup.rng);
    let bytes =
        state.canonical_state_bytes(setup.identity, &setup.graph, setup.instance, &setup.rng);
    let rng = setup.rng.snapshots();
    let valid_values = values(
        BattleOutcome::Won,
        participant_state(800, 30, LifeState::Alive, PresenceState::Present),
        1,
    );
    let forged = BattleResult::new(
        handoff.identity(),
        valid_values,
        starclock_activity::BattleResultDigest::new([0xee; 32]).unwrap(),
    );
    assert_eq!(
        state.submit_pending_battle_result(
            setup.identity,
            &setup.graph,
            setup.instance,
            &setup.rng,
            ActivityBattleResultSubmission::new(awaiting, forged),
        ),
        Err(ActivityBattleSettlementError::ResultDigestMismatch)
    );
    assert_eq!(
        state.canonical_state_bytes(setup.identity, &setup.graph, setup.instance, &setup.rng),
        bytes
    );
    assert_eq!(setup.rng.snapshots(), rng);

    let wrong_maximum = ParticipantBattleState::new(
        participant(1),
        hp(700),
        hp(999),
        energy(20),
        energy(100),
        LifeState::Alive,
        PresenceState::Present,
    )
    .unwrap();
    let incompatible = result(handoff.identity(), BattleOutcome::Won, wrong_maximum, 1);
    assert_eq!(
        state.submit_pending_battle_result(
            setup.identity,
            &setup.graph,
            setup.instance,
            &setup.rng,
            ActivityBattleResultSubmission::new(awaiting, incompatible),
        ),
        Err(ActivityBattleSettlementError::ParticipantMaximumMismatch)
    );
    assert_eq!(
        state.canonical_state_bytes(setup.identity, &setup.graph, setup.instance, &setup.rng),
        bytes
    );
    assert_eq!(setup.rng.snapshots(), rng);
}

#[test]
fn settled_attempt_can_enter_the_next_battle_with_the_carry_ledger() {
    let setup = Setup::new(true);
    let mut state = setup.state();
    setup.prepare(&mut state, node(20), 1, 1);
    let first = setup.start(&mut state);
    let awaiting = state.state_hash(setup.identity, &setup.graph, setup.instance, &setup.rng);
    state
        .submit_pending_battle_result(
            setup.identity,
            &setup.graph,
            setup.instance,
            &setup.rng,
            ActivityBattleResultSubmission::new(
                awaiting,
                result(
                    first.identity(),
                    BattleOutcome::Won,
                    participant_state(640, 73, LifeState::Alive, PresenceState::Present),
                    2,
                ),
            ),
        )
        .unwrap();
    assert_eq!(state.current_node(), node(30));
    setup.prepare(&mut state, node(30), 2, 2);
    let second = setup.start(&mut state);
    assert_ne!(first.identity().seed(), second.identity().seed());
    assert_eq!(second.participant_carry().len(), 1);
    assert_eq!(second.participant_carry()[0].current_hp(), hp(640));
    assert_eq!(second.participant_carry()[0].current_energy(), energy(73));
}

#[test]
fn contract_rejects_undeclared_participants_and_wrong_metric_slots() {
    let setup = Setup::new(false);
    let projection = Arc::new(projection());
    assert_eq!(
        ActivityBattleResultContract::new(Arc::clone(&projection), vec![], vec![metric_binding()]),
        Err(starclock_activity::ActivityBattleResultContractError::ParticipantProjectionMismatch)
    );
    assert_eq!(
        ActivityBattleResultContract::new(
            Arc::clone(&projection),
            vec![ActivityParticipantCarryDefinition::new(
                participant(1),
                HpCarryPolicy::RestoreFull,
                EnergyCarryPolicy::CarryExact,
                LifeCarryPolicy::CarryExact,
                PresenceCarryPolicy::CarryExact,
            )],
            vec![metric_binding()],
        ),
        Err(starclock_activity::ActivityBattleResultContractError::InvalidCarryPolicy)
    );
    let bad_slot_contract = Arc::new(
        ActivityBattleResultContract::new(
            projection,
            vec![carry_definition()],
            vec![
                ActivityMetricProjectionBinding::new(
                    "score",
                    MetricValueKind::BoundedInteger,
                    slot(2),
                    MetricSettlementPolicy::Replace,
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    );
    let mut state = setup.state();
    setup.prepare(&mut state, node(20), 1, 1);
    let hash = state.state_hash(setup.identity, &setup.graph, setup.instance, &setup.rng);
    assert_eq!(
        state.start_pending_battle(
            &setup.graph,
            &setup.rng,
            ActivityBattleStartRequest::new(
                hash,
                setup.identity,
                setup.instance,
                bad_slot_contract
            ),
        ),
        Err(ActivityBattleSettlementError::MetricSlotMismatch)
    );
    assert_eq!(
        state.state_hash(setup.identity, &setup.graph, setup.instance, &setup.rng),
        hash
    );
}

struct Setup {
    graph: ActivityGraphDefinition,
    identity: ActivityDefinitionIdentity,
    instance: ActivityInstanceId,
    roster: ParticipantLock,
    preparation: Arc<EncounterPreparationDefinition>,
    contract: Arc<ActivityBattleResultContract>,
    rng: ActivityRngStreams,
}

impl Setup {
    fn new(two_battles: bool) -> Self {
        let graph = graph(two_battles);
        let identity = identity();
        let instance = ActivityInstanceId::new(7).unwrap();
        let roster = participant_lock();
        let lock = roster.digest();
        let preparation = Arc::new(
            EncounterPreparationDefinition::new(
                ActivityOptionId::new(10).unwrap(),
                EncounterInitiativePolicy::PlayerControlled,
                lock,
                0,
                vec![],
                vec![PreparedBattleVariant::new(
                    vec![],
                    TechniqueContributionDigest::new([0x34; 32]).unwrap(),
                    BattleBinding::new(battle_spec(), "battle", "battle-spec-v1", lock).unwrap(),
                )],
            )
            .unwrap(),
        );
        let contract = Arc::new(
            ActivityBattleResultContract::new(
                Arc::new(projection()),
                vec![carry_definition()],
                vec![metric_binding()],
            )
            .unwrap(),
        );
        let rng = ActivityRngStreams::new(ActivityRngContext::new(
            ActivityMasterSeed::from_u64(5),
            identity.id(),
            identity.definition_digest(),
            identity.config_digest(),
            graph.digest(),
            instance,
            Some(section()),
            Some(node(20)),
            Some(starclock_activity::AttemptId::new(1).unwrap()),
            1,
        ));
        Self {
            graph,
            identity,
            instance,
            roster,
            preparation,
            contract,
            rng,
        }
    }

    fn state(&self) -> ActivityTransactionState {
        ActivityTransactionState::new(state_definition(), node(20))
    }

    fn prepare(
        &self,
        state: &mut ActivityTransactionState,
        at: NodeId,
        attempt: u32,
        sequence: u32,
    ) {
        state
            .begin_battle_preparation(
                self.instance,
                &self.graph,
                ActivityBattlePreparationRequest::new(
                    ActivityScopePath::new(self.instance)
                        .enter_section(section())
                        .unwrap()
                        .enter_node(at)
                        .unwrap()
                        .enter_attempt(starclock_activity::AttemptId::new(attempt).unwrap())
                        .unwrap(),
                    ActivityRosterLock::new(
                        ActivityScopePath::new(self.instance),
                        self.roster.clone(),
                    )
                    .unwrap(),
                    BattleSequence::new(sequence).unwrap(),
                    0,
                    Arc::clone(&self.preparation),
                ),
            )
            .unwrap();
        state
            .choose_preparation_option(ActivityOptionId::new(10).unwrap())
            .unwrap();
    }

    fn start(
        &self,
        state: &mut ActivityTransactionState,
    ) -> starclock_activity::ActivityBattleHandoff {
        let hash = state.state_hash(self.identity, &self.graph, self.instance, &self.rng);
        state
            .start_pending_battle(
                &self.graph,
                &self.rng,
                ActivityBattleStartRequest::new(
                    hash,
                    self.identity,
                    self.instance,
                    Arc::clone(&self.contract),
                ),
            )
            .unwrap()
    }
}

fn state_definition() -> ActivityStateDefinition {
    ActivityStateDefinition::new(
        vec![
            ActivitySlotDefinition::new_with_policy(
                slot(1),
                ActivityScope::Activity,
                ActivityValue::BoundedInteger(5),
                Some((0, 100)),
                None,
                vec![],
                SlotCarryPolicy::CarryExact,
                ActivityStateVisibility::Player,
                ActivityStateSource::new(1).unwrap(),
            )
            .unwrap(),
            ActivitySlotDefinition::new_with_policy(
                slot(2),
                ActivityScope::Activity,
                ActivityValue::Boolean(false),
                None,
                None,
                vec![],
                SlotCarryPolicy::CarryExact,
                ActivityStateVisibility::Private,
                ActivityStateSource::new(2).unwrap(),
            )
            .unwrap(),
        ],
        vec![],
        vec![],
    )
    .unwrap()
}

fn projection() -> BattleResultProjection {
    BattleResultProjection::new(
        ProjectionId::new(1).unwrap(),
        vec![
            ProjectionField::Outcome,
            ProjectionField::FinalStateHash,
            ProjectionField::EventDigest,
            ProjectionField::TerminalFault,
            ProjectionField::ParticipantState(participant(1)),
            ProjectionField::Metric {
                key: "score".into(),
                kind: MetricValueKind::BoundedInteger,
            },
        ],
    )
    .unwrap()
}

fn carry_definition() -> ActivityParticipantCarryDefinition {
    ActivityParticipantCarryDefinition::new(
        participant(1),
        HpCarryPolicy::CarryExact,
        EnergyCarryPolicy::CarryExact,
        LifeCarryPolicy::CarryExact,
        PresenceCarryPolicy::CarryExact,
    )
}

fn metric_binding() -> ActivityMetricProjectionBinding {
    ActivityMetricProjectionBinding::new(
        "score",
        MetricValueKind::BoundedInteger,
        slot(1),
        MetricSettlementPolicy::Sum,
    )
    .unwrap()
}

fn result(
    identity: starclock_activity::BattleResultIdentity,
    outcome: BattleOutcome,
    participant: ParticipantBattleState,
    score: i64,
) -> BattleResult {
    BattleResult::seal(identity, values(outcome, participant, score))
}

fn values(
    outcome: BattleOutcome,
    participant: ParticipantBattleState,
    score: i64,
) -> Vec<ProjectedValue> {
    vec![
        ProjectedValue::Outcome(outcome),
        ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x71; 32])),
        ProjectedValue::EventDigest(EventDigest::new([0x72; 32]).unwrap()),
        ProjectedValue::TerminalFault(None),
        ProjectedValue::ParticipantState(participant),
        ProjectedValue::Metric {
            key: "score".into(),
            value: MetricValue::BoundedInteger(score),
        },
    ]
}

fn participant_state(
    current_hp: i64,
    current_energy: i64,
    life: LifeState,
    presence: PresenceState,
) -> ParticipantBattleState {
    ParticipantBattleState::new(
        participant(1),
        hp(current_hp),
        hp(1_000),
        energy(current_energy),
        energy(100),
        life,
        presence,
    )
    .unwrap()
}

fn participant_lock() -> ParticipantLock {
    ParticipantLock::seal(
        ParticipantPolicy::new(
            1,
            1,
            1,
            ParticipantUniquenessScope::Activity,
            LoadoutLockScope::Activity,
        )
        .unwrap(),
        vec![
            ParticipantLockEntry::new(
                participant(1),
                0,
                0,
                UnitDefinitionId::new(101).unwrap(),
                OpaqueParticipantBuild::new(
                    CombatantSpecDigest::new([0x81; 32]).unwrap(),
                    BuildDigest::new([0x82; 32]).unwrap(),
                    "build-v1",
                    ParticipantSourceKind::CompiledBuild,
                )
                .unwrap(),
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

fn battle_spec() -> BattleSpec {
    BattleSpec::new(
        "rules-v1",
        BattleSpecDigest::new([0x33; 32]).unwrap(),
        EncounterId::new(1).unwrap(),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(101, 0x81, true),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(EnemyDefinitionId::new(201).unwrap()),
                combatant(201, 0x91, false),
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap()
}

fn combatant(form: u32, digest: u8, player: bool) -> ResolvedCombatantSpec {
    let value = ResolvedCombatantSpec::new(
        UnitDefinitionId::new(form).unwrap(),
        UnitLevel::new(80).unwrap(),
        hp(1_000),
        Speed::from_scaled(100_000_000).unwrap(),
        ResolvedDefinitionBindings::new(vec![AbilityId::new(form).unwrap()], vec![], vec![])
            .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap();
    if player {
        value.with_energy(Energy::ZERO, energy(100)).unwrap()
    } else {
        value
    }
}

fn graph(two_battles: bool) -> ActivityGraphDefinition {
    let mut nodes = vec![
        graph_node(20, ActivityNodeKind::Battle),
        graph_node(
            21,
            ActivityNodeKind::Terminal(starclock_activity::ActivityTerminalOutcome::Completed),
        ),
        graph_node(
            22,
            ActivityNodeKind::Terminal(starclock_activity::ActivityTerminalOutcome::Failed),
        ),
        graph_node(
            23,
            ActivityNodeKind::Terminal(starclock_activity::ActivityTerminalOutcome::Faulted),
        ),
    ];
    let first_win = if two_battles { node(30) } else { node(21) };
    if two_battles {
        nodes.push(graph_node(30, ActivityNodeKind::Battle));
    }
    let mut edges = vec![
        edge(1, node(20), first_win, BattleOutcome::Won),
        edge(2, node(20), node(22), BattleOutcome::Lost),
        edge(3, node(20), node(23), BattleOutcome::Faulted),
    ];
    if two_battles {
        edges.extend([
            edge(4, node(30), node(21), BattleOutcome::Won),
            edge(5, node(30), node(22), BattleOutcome::Lost),
            edge(6, node(30), node(23), BattleOutcome::Faulted),
        ]);
    }
    ActivityGraphDefinition::new(node(20), nodes, edges, 3).unwrap()
}

fn graph_node(value: u32, kind: ActivityNodeKind) -> ActivityNodeDefinition {
    ActivityNodeDefinition::new(node(value), section(), kind, 1).unwrap()
}

fn edge(id: u32, from: NodeId, to: NodeId, outcome: BattleOutcome) -> ActivityEdgeDefinition {
    ActivityEdgeDefinition::new(
        ActivityEdgeId::new(id).unwrap(),
        from,
        to,
        ActivityEdgeCondition::BattleOutcome(outcome.into()),
        id as i32,
        1,
    )
    .unwrap()
}

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(1).unwrap(),
        ActivityDefinitionDigest::new([0xa1; 32]).unwrap(),
        ActivityConfigDigest::new([0xa2; 32]).unwrap(),
    )
}

fn participant(value: u32) -> ParticipantId {
    ParticipantId::new(value).unwrap()
}
fn slot(value: u32) -> ActivitySlotId {
    ActivitySlotId::new(value).unwrap()
}
fn node(value: u32) -> NodeId {
    NodeId::new(value).unwrap()
}
fn section() -> SectionId {
    SectionId::new(10).unwrap()
}
fn hp(value: i64) -> Hp {
    Hp::new(value).unwrap()
}
fn energy(value: i64) -> Energy {
    Energy::from_scaled(value * 1_000_000).unwrap()
}
