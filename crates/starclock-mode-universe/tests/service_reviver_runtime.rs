use std::sync::{Arc, OnceLock};

use starclock_activity::{
    ActivityBattlePreparationRequest, ActivityBattleResultContract, ActivityBattleResultSubmission,
    ActivityBattleStartRequest, ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest,
    ActivityDefinitionId, ActivityDefinitionIdentity, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityGraphDefinition, ActivityHandlerId,
    ActivityHandlerInput, ActivityInstanceId, ActivityMasterSeed, ActivityNodeDefinition,
    ActivityNodeKind, ActivityParticipantCarryDefinition, ActivityProgramDefinition,
    ActivityProgramId, ActivityRngContext, ActivityRngStreams, ActivityRosterLock, ActivityScope,
    ActivityScopePath, ActivitySlotDefinition, ActivitySlotId, ActivityStateDefinition,
    ActivityStateSource, ActivityStateVisibility, ActivityTerminalOutcome,
    ActivityTransactionOutcome, ActivityTransactionState, ActivityValue, BattleBinding,
    BattleOutcome, BattleResult, BattleResultProjection, BattleSequence, BuildDigest,
    EncounterInitiativePolicy, EncounterPreparationDefinition, EnergyCarryPolicy, EventDigest,
    HpCarryPolicy, LifeCarryPolicy, LoadoutLockScope, NodeId, OpaqueParticipantBuild,
    ParticipantBattleState, ParticipantId, ParticipantLock, ParticipantLockEntry,
    ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope, PreparedBattleVariant,
    PresenceCarryPolicy, ProjectedValue, ProjectionField, ProjectionId, SectionId, SlotCarryPolicy,
    TechniqueContributionDigest,
};
use starclock_combat::{
    AbilityId, AssemblyDigest, BattleSpec, BattleStateHash, CombatantSpecDigest, ConcedePolicy,
    EncounterId, EnemyDefinitionId, Energy, FormationIndex, Hp, LifeState, ParticipantSource,
    ParticipantSpec, PresenceState, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    TeamResourceSpec, TeamSide, UnitDefinitionId, UnitLevel,
};
use starclock_mode_universe::{
    ability_runtime::AbilityTarget,
    catalog::UniverseCatalog,
    entry::{CompiledActivity, StandardUniverseEntry, StandardUniverseProfile},
    service_interaction::{SERVICE_INTERACTION_HANDLER_ID, ServiceInteractionSelection},
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

fn catalog() -> Arc<UniverseCatalog> {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    Arc::clone(CATALOG.get_or_init(|| {
        let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core");
        UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe")
    }))
}

fn compiled() -> CompiledActivity {
    let catalog = catalog();
    let world = &catalog.worlds()[0];
    StandardUniverseProfile::new(Arc::clone(&catalog))
        .compile(StandardUniverseEntry::new(
            world.id(),
            world.difficulties()[0],
            participants(),
            vec![],
        ))
        .expect("compiled Standard Universe")
}

#[test]
fn goal07_p4_m14_s01_reviver_restores_defeated_battle_carry_atomically() {
    let compiled = compiled();
    let participant = participant();
    let service = catalog()
        .services()
        .iter()
        .find(|value| value.stable_key() == "universe.service.reviver")
        .expect("reviver service")
        .id();
    let interaction = compiled
        .service_interaction_runtime()
        .compile_selection(
            service,
            &ServiceInteractionSelection::ReviveCharacter(participant),
        )
        .expect("reviver selection");
    assert_eq!(interaction.required_fragments(), Some(80));

    let fixture = ReviverFixture::new();
    let mut state = fixture.state(&compiled);
    fixture.prepare(&mut state);
    let before = fixture.hash(&state);
    let handoff = state
        .start_pending_battle(
            &fixture.graph,
            &fixture.rng,
            ActivityBattleStartRequest::new(
                before,
                fixture.identity,
                fixture.instance,
                Arc::clone(&fixture.contract),
            ),
        )
        .expect("battle start");
    let awaiting = fixture.hash(&state);
    state
        .submit_pending_battle_result(
            fixture.identity,
            &fixture.graph,
            fixture.instance,
            &fixture.rng,
            ActivityBattleResultSubmission::new(awaiting, defeated_result(handoff.identity())),
        )
        .expect("defeated participant carry");

    let view = fixture.view(&state);
    assert_eq!(view.participant_carry()[0].life(), LifeState::Defeated);
    let registration = compiled
        .runtime_definition()
        .interactions()
        .unwrap()
        .registry()
        .handler(ActivityHandlerId::new(SERVICE_INTERACTION_HANDLER_ID).unwrap())
        .expect("production service handler");
    let output = registration
        .execute(ActivityHandlerInput::new(&view, interaction.payload()).unwrap())
        .expect("reviver handler output");
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(80_001).unwrap(),
        output.operations().to_vec(),
    )
    .unwrap();
    let cause = ActivityCause::new(1, program.id(), fixture.service_node).unwrap();
    assert!(matches!(
        state.apply_program(&program, cause, &fixture.graph),
        ActivityTransactionOutcome::Committed(_)
    ));

    let after = fixture.view(&state);
    assert_eq!(
        after
            .slots()
            .iter()
            .find(|slot| slot.id() == compiled.cosmic_fragments_slot())
            .map(|slot| slot.value()),
        Some(&ActivityValue::BoundedInteger(920))
    );
    let restored = after.participant_carry()[0];
    assert_eq!(restored.life(), LifeState::Alive);
    assert_eq!(restored.presence(), PresenceState::Present);
    assert_eq!(restored.current_hp(), restored.maximum_hp());
}

struct ReviverFixture {
    graph: ActivityGraphDefinition,
    identity: ActivityDefinitionIdentity,
    instance: ActivityInstanceId,
    service_node: NodeId,
    preparation: Arc<EncounterPreparationDefinition>,
    contract: Arc<ActivityBattleResultContract>,
    rng: ActivityRngStreams,
}

impl ReviverFixture {
    fn new() -> Self {
        let battle_node = node(70);
        let service_node = node(71);
        let graph = graph(battle_node, service_node);
        let identity = ActivityDefinitionIdentity::new(
            ActivityDefinitionId::new(9_071).unwrap(),
            ActivityDefinitionDigest::new([0x71; 32]).unwrap(),
            ActivityConfigDigest::new([0x72; 32]).unwrap(),
        );
        let instance = ActivityInstanceId::new(9_071).unwrap();
        let lock = participants().digest();
        let preparation = Arc::new(
            EncounterPreparationDefinition::new(
                starclock_activity::ActivityOptionId::new(70).unwrap(),
                EncounterInitiativePolicy::PlayerControlled,
                lock,
                0,
                vec![],
                vec![PreparedBattleVariant::new(
                    vec![],
                    TechniqueContributionDigest::new([0x73; 32]).unwrap(),
                    BattleBinding::new(
                        battle_spec(),
                        "goal07-reviver-battle",
                        "battle-spec-v1",
                        lock,
                    )
                    .unwrap(),
                )],
            )
            .unwrap(),
        );
        let projection = Arc::new(
            BattleResultProjection::new(
                ProjectionId::new(70).unwrap(),
                vec![
                    ProjectionField::Outcome,
                    ProjectionField::FinalStateHash,
                    ProjectionField::EventDigest,
                    ProjectionField::TerminalFault,
                    ProjectionField::ParticipantState(participant()),
                ],
            )
            .unwrap(),
        );
        let contract = Arc::new(
            ActivityBattleResultContract::new(
                projection,
                vec![ActivityParticipantCarryDefinition::new(
                    participant(),
                    HpCarryPolicy::CarryExact,
                    EnergyCarryPolicy::CarryExact,
                    LifeCarryPolicy::CarryExact,
                    PresenceCarryPolicy::CarryExact,
                )],
                vec![],
            )
            .unwrap(),
        );
        let rng = ActivityRngStreams::new(ActivityRngContext::new(
            ActivityMasterSeed::from_u64(9_071),
            identity.id(),
            identity.definition_digest(),
            identity.config_digest(),
            graph.digest(),
            instance,
            Some(section()),
            Some(battle_node),
            Some(starclock_activity::AttemptId::new(1).unwrap()),
            1,
        ));
        Self {
            graph,
            identity,
            instance,
            service_node,
            preparation,
            contract,
            rng,
        }
    }

    fn state(&self, compiled: &CompiledActivity) -> ActivityTransactionState {
        let state = ActivityStateDefinition::new(
            vec![
                integer_slot(compiled.cosmic_fragments_slot()),
                counter_slot(
                    compiled.service_use_slot(),
                    ActivityStateVisibility::Player,
                    0x7102,
                    Box::new([]),
                ),
                counter_slot(
                    compiled.service_effect_slot(),
                    ActivityStateVisibility::Private,
                    0x7103,
                    Box::new([]),
                ),
                counter_slot(
                    compiled.ability_projection_slot(),
                    ActivityStateVisibility::Player,
                    0x7104,
                    vec![
                        (AbilityTarget::ServiceReviver.activity_key(), 1_000_000),
                        (
                            AbilityTarget::ServiceReviverRestoredHpRatio.activity_key(),
                            1_000_000,
                        ),
                    ]
                    .into_boxed_slice(),
                ),
                cloned_slot(compiled, compiled.curio_state_slot()),
                cloned_slot(compiled, compiled.curio_charge_slot()),
                cloned_slot(compiled, compiled.curio_event_slot()),
            ],
            vec![
                *compiled
                    .state_definition()
                    .inventories()
                    .iter()
                    .find(|value| value.id() == compiled.blessing_inventory())
                    .unwrap(),
                *compiled
                    .state_definition()
                    .inventories()
                    .iter()
                    .find(|value| value.id() == compiled.curio_inventory())
                    .unwrap(),
            ],
            vec![],
        )
        .unwrap();
        ActivityTransactionState::new(state, node(70))
    }

    fn prepare(&self, state: &mut ActivityTransactionState) {
        let path = ActivityScopePath::new(self.instance)
            .enter_section(section())
            .unwrap()
            .enter_node(node(70))
            .unwrap()
            .enter_attempt(starclock_activity::AttemptId::new(1).unwrap())
            .unwrap();
        state
            .begin_battle_preparation(
                self.instance,
                &self.graph,
                ActivityBattlePreparationRequest::new(
                    path,
                    ActivityRosterLock::new(ActivityScopePath::new(self.instance), participants())
                        .unwrap(),
                    BattleSequence::new(1).unwrap(),
                    0,
                    Arc::clone(&self.preparation),
                ),
            )
            .unwrap();
        state
            .choose_preparation_option(starclock_activity::ActivityOptionId::new(70).unwrap())
            .unwrap();
    }

    fn hash(&self, state: &ActivityTransactionState) -> starclock_activity::ActivityStateHash {
        state.state_hash(self.identity, &self.graph, self.instance, &self.rng)
    }

    fn view(&self, state: &ActivityTransactionState) -> starclock_activity::ActivityPlayerView {
        state.player_view(self.identity, &self.graph, self.instance, &self.rng)
    }
}

fn graph(battle: NodeId, service: NodeId) -> ActivityGraphDefinition {
    ActivityGraphDefinition::new(
        battle,
        vec![
            graph_node(battle, ActivityNodeKind::Battle),
            graph_node(service, ActivityNodeKind::ExternalOutcome),
            graph_node(
                node(72),
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Failed),
            ),
            graph_node(
                node(73),
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Faulted),
            ),
            graph_node(
                node(74),
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            ),
        ],
        vec![
            battle_edge(70, battle, service, BattleOutcome::Won),
            battle_edge(71, battle, node(72), BattleOutcome::Lost),
            battle_edge(72, battle, node(73), BattleOutcome::Faulted),
            ActivityEdgeDefinition::new(
                ActivityEdgeId::new(73).unwrap(),
                service,
                node(74),
                ActivityEdgeCondition::OptionSelected,
                0,
                1,
            )
            .unwrap(),
        ],
        3,
    )
    .unwrap()
}

fn graph_node(id: NodeId, kind: ActivityNodeKind) -> ActivityNodeDefinition {
    ActivityNodeDefinition::new(id, section(), kind, 1).unwrap()
}

fn battle_edge(
    id: u32,
    from: NodeId,
    to: NodeId,
    outcome: BattleOutcome,
) -> ActivityEdgeDefinition {
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

fn integer_slot(id: ActivitySlotId) -> ActivitySlotDefinition {
    ActivitySlotDefinition::new_with_policy(
        id,
        ActivityScope::Activity,
        ActivityValue::BoundedInteger(1_000),
        Some((
            0,
            starclock_mode_universe::run_runtime::MAX_COSMIC_FRAGMENTS,
        )),
        None,
        vec![],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        ActivityStateSource::new(0x7101).unwrap(),
    )
    .unwrap()
}

fn counter_slot(
    id: ActivitySlotId,
    visibility: ActivityStateVisibility,
    source: u64,
    values: Box<[(u64, i64)]>,
) -> ActivitySlotDefinition {
    ActivitySlotDefinition::new_with_policy(
        id,
        ActivityScope::Activity,
        ActivityValue::BoundedCounterMap(values),
        Some((0, i64::from(u32::MAX))),
        Some(94),
        vec![],
        SlotCarryPolicy::CarryExact,
        visibility,
        ActivityStateSource::new(source).unwrap(),
    )
    .unwrap()
}

fn cloned_slot(compiled: &CompiledActivity, id: ActivitySlotId) -> ActivitySlotDefinition {
    compiled
        .state_definition()
        .slots()
        .iter()
        .find(|value| value.id() == id)
        .unwrap()
        .clone()
}

fn defeated_result(identity: starclock_activity::BattleResultIdentity) -> BattleResult {
    BattleResult::seal(
        identity,
        vec![
            ProjectedValue::Outcome(BattleOutcome::Won),
            ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x81; 32])),
            ProjectedValue::EventDigest(EventDigest::new([0x82; 32]).unwrap()),
            ProjectedValue::TerminalFault(None),
            ProjectedValue::ParticipantState(
                ParticipantBattleState::new(
                    participant(),
                    hp(0),
                    hp(1_000),
                    Energy::ZERO,
                    energy(100),
                    LifeState::Defeated,
                    PresenceState::Departed,
                )
                .unwrap(),
            ),
        ],
    )
}

fn battle_spec() -> BattleSpec {
    BattleSpec::new(
        "rules-v1",
        AssemblyDigest::new([0x83; 32]).unwrap(),
        EncounterId::new(1).unwrap(),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(20_001, 0x01, true),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(EnemyDefinitionId::new(1).unwrap()),
                combatant(30_001, 0x91, false),
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

fn participants() -> ParticipantLock {
    ParticipantLock::seal(
        ParticipantPolicy::new(
            1,
            1,
            4,
            ParticipantUniquenessScope::Activity,
            LoadoutLockScope::Activity,
        )
        .unwrap(),
        vec![
            ParticipantLockEntry::new(
                participant(),
                0,
                0,
                UnitDefinitionId::new(20_001).unwrap(),
                OpaqueParticipantBuild::new(
                    CombatantSpecDigest::new([1; 32]).unwrap(),
                    BuildDigest::new([2; 32]).unwrap(),
                    "service-reviver-test-v1",
                    ParticipantSourceKind::CompiledBuild,
                )
                .unwrap(),
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

fn participant() -> ParticipantId {
    ParticipantId::new(1).unwrap()
}

fn node(value: u32) -> NodeId {
    NodeId::new(value).unwrap()
}

fn section() -> SectionId {
    SectionId::new(7).unwrap()
}

fn hp(value: i64) -> Hp {
    Hp::new(value).unwrap()
}

fn energy(value: i64) -> Energy {
    Energy::from_scaled(value * 1_000_000).unwrap()
}
