use std::sync::Arc;

use starclock_activity::{
    ActivityBattleHandoff, ActivityBattlePreparationRequest, ActivityBattleResultContract,
    ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityEdgeId, ActivityExpression, ActivityGraphDefinition, ActivityInstanceId,
    ActivityMasterSeed, ActivityMetricProjectionBinding, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionDefinition, ActivityOptionId,
    ActivityParticipantCarryDefinition, ActivityProgramDefinition, ActivityProgramId,
    ActivityRandomPolicies, ActivityRosterLock, ActivityScope, ActivityScopePath,
    ActivitySlotDefinition, ActivitySlotId, ActivityStateDefinition, ActivityStateHash,
    ActivityStateSource, ActivityStateVisibility, ActivityTerminalOutcome, ActivityValue,
    AttemptId, BattleBinding, BattleOutcome, BattleResult, BattleResultProjection, BattleSequence,
    EncounterInitiativePolicy, EncounterPreparationDefinition, EnergyCarryPolicy, GraphActivity,
    GraphActivityBattleResolution, GraphActivityDefinition, GraphActivityNodeProgram,
    GraphActivityPreparationResolution, HpCarryPolicy, LifeCarryPolicy, LoadoutLockScope,
    MetricSettlementPolicy, MetricValueKind, NodeId, ParticipantId, ParticipantLock,
    PresenceCarryPolicy, ProjectionField, ProjectionId, SectionId, SlotCarryPolicy,
    TechniqueContributionDigest,
};
use starclock_combat::{BattleSpec, RuleBundleId, TeamSide};

use crate::{
    ObjectiveEvaluation, PureFictionProfile, PureFictionStage,
    pure_fiction_projection::{
        NODE_SCORE_KEY, WAVE_ONE_SCORE_KEY, WAVE_THREE_SCORE_KEY, WAVE_TWO_SCORE_KEY,
    },
};

const SECTION: u32 = 2;
const COMPLETED: u32 = 100;
const FAILED: u32 = 101;
const FAULTED: u32 = 102;
const SCORE_SLOT_BASE: u32 = 10;

/// Fully linked ordinary or three-node Starward Pure Fiction attempt definition.
#[derive(Debug)]
pub struct PureFictionAttemptDefinition {
    profile: Arc<PureFictionProfile>,
    stage_index: usize,
    battles: Box<[BattleSpec]>,
    cacophonies: Box<[RuleBundleId]>,
    activity: Arc<GraphActivityDefinition>,
    participants: Arc<ParticipantLock>,
    contracts: Box<[Arc<ActivityBattleResultContract>]>,
}

impl PureFictionAttemptDefinition {
    pub fn new(
        identity: starclock_activity::ActivityDefinitionIdentity,
        profile: Arc<PureFictionProfile>,
        stage_index: usize,
        participants: ParticipantLock,
        battles: Vec<BattleSpec>,
        cacophonies: Vec<RuleBundleId>,
    ) -> Result<Self, PureFictionAttemptError> {
        let stage = profile
            .stages
            .get(stage_index)
            .ok_or_else(|| error("PureFiction stage index is out of bounds"))?;
        let node_count = stage.nodes.len();
        if !(2..=3).contains(&node_count)
            || battles.len() != node_count
            || cacophonies.len() != node_count
            || usize::from(participants.policy().team_count()) != node_count
            || participants.policy().loadout_lock_scope() != LoadoutLockScope::Section
        {
            return Err(error(
                "PureFiction attempts require one Section-locked team and battle per node",
            ));
        }
        for (index, battle) in battles.iter().enumerate() {
            if battle.encounter() != stage.nodes[index].encounter {
                return Err(error(
                    "battle encounter does not match the selected PureFiction node",
                ));
            }
            if !stage.nodes[index]
                .cacophony_bundles
                .contains(&cacophonies[index])
                || !battle.participants().iter().any(|participant| {
                    participant.side() == TeamSide::Player
                        && participant
                            .combatant()
                            .rule_bundles()
                            .contains(&cacophonies[index])
                })
            {
                return Err(error(
                    "selected Pure Fiction Cacophony is not compiled into its node battle",
                ));
            }
        }
        let team_participants = (0..node_count)
            .map(|index| {
                participant_ids(
                    &participants,
                    u8::try_from(index).expect("at most three Pure Fiction nodes"),
                )
            })
            .collect::<Vec<_>>();
        if team_participants.iter().any(Vec::is_empty) {
            return Err(error(
                "every PureFiction team must contain at least one participant",
            ));
        }
        let contracts = team_participants
            .iter()
            .enumerate()
            .map(|(index, participants)| {
                score_contract(
                    participants,
                    u32::try_from(index + 1).expect("at most three projections"),
                    index,
                )
                .map(Arc::new)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let participants = Arc::new(participants);
        let activity = Arc::new(
            GraphActivityDefinition::new(
                identity,
                activity_graph(node_count)?,
                activity_state(node_count)?,
                Arc::clone(&participants),
                (0..node_count)
                    .map(|index| battle_program(index + 1, index))
                    .collect::<Result<Vec<_>, _>>()?,
                None,
                ActivityRandomPolicies::default(),
            )
            .map_err(debug_error)?,
        );
        Ok(Self {
            profile,
            stage_index,
            battles: battles.into_boxed_slice(),
            cacophonies: cacophonies.into_boxed_slice(),
            activity,
            participants,
            contracts: contracts.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn stage(&self) -> &PureFictionStage {
        &self.profile.stages[self.stage_index]
    }

    #[must_use]
    pub fn cacophonies(&self) -> &[RuleBundleId] {
        &self.cacophonies
    }
}

/// Running wrapper; all authoritative mutation remains in `GraphActivity`.
#[derive(Debug)]
pub struct PureFictionAttempt {
    definition: Arc<PureFictionAttemptDefinition>,
    activity: GraphActivity,
}

impl PureFictionAttempt {
    pub fn start(
        definition: Arc<PureFictionAttemptDefinition>,
        instance: ActivityInstanceId,
        master_seed: ActivityMasterSeed,
    ) -> Result<Self, PureFictionAttemptError> {
        let activity =
            GraphActivity::start(Arc::clone(&definition.activity), instance, master_seed)
                .map_err(debug_error)?
                .into_activity();
        Ok(Self {
            definition,
            activity,
        })
    }

    #[must_use]
    pub fn state_hash(&self) -> ActivityStateHash {
        self.activity.state_hash()
    }

    #[must_use]
    pub fn player_view(&self) -> starclock_activity::ActivityPlayerView {
        self.activity.player_view()
    }

    #[must_use]
    pub fn debug_view(&self) -> starclock_activity::ActivityDebugView {
        self.activity.debug_view()
    }

    #[must_use]
    pub fn node_score(&self, node_index: usize) -> i64 {
        self.integer_slot(score_slot(node_index))
    }

    #[must_use]
    pub fn wave_one_score(&self, node_index: usize) -> i64 {
        self.integer_slot(wave_one_slot(node_index))
    }

    #[must_use]
    pub fn wave_two_score(&self, node_index: usize) -> i64 {
        self.integer_slot(wave_two_slot(node_index))
    }

    #[must_use]
    pub fn wave_three_score(&self, node_index: usize) -> i64 {
        self.integer_slot(wave_three_slot(node_index))
    }

    #[must_use]
    pub fn total_score(&self) -> i64 {
        (0..self.definition.stage().nodes.len())
            .map(|index| self.node_score(index))
            .sum()
    }

    #[must_use]
    pub fn cleared(&self) -> bool {
        self.total_score() >= self.definition.stage().clear_score
    }

    #[must_use]
    pub fn objectives(&self) -> ObjectiveEvaluation {
        self.definition.stage().evaluate(self.total_score())
    }

    pub fn engage_current_node(
        &mut self,
        attempt: AttemptId,
    ) -> Result<GraphActivityPreparationResolution, PureFictionAttemptError> {
        let node_index = self.node_index()?;
        let view = self.activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| decision.kind() == ActivityDecisionKind::Encounter)
            .ok_or_else(|| error("current PureFiction node is not offering an encounter"))?;
        if !decision
            .options()
            .iter()
            .any(|option| option.id() == encounter_option(node_index))
        {
            return Err(error("current PureFiction encounter option is not offered"));
        }
        let battle = self
            .definition
            .profile
            .compile_battle(
                self.definition.stage_index,
                self.definition.battles[node_index].clone(),
            )
            .ok_or_else(|| error("Pure Fiction cycle clock cannot be compiled"))?;
        let lock = self.definition.participants.digest();
        let preparation = Arc::new(
            EncounterPreparationDefinition::new(
                normal_option(node_index),
                EncounterInitiativePolicy::PlayerControlled,
                lock,
                u8::try_from(node_index).expect("PureFiction has at most three nodes"),
                vec![],
                vec![starclock_activity::PreparedBattleVariant::new(
                    vec![],
                    contribution(node_index),
                    BattleBinding::new(battle, seed_label(node_index), lock)
                        .map_err(debug_error)?,
                )],
            )
            .map_err(debug_error)?,
        );
        let path = ActivityScopePath::new(self.activity.instance())
            .enter_section(section())
            .and_then(|path| path.enter_node(node(node_index + 1)))
            .and_then(|path| path.enter_attempt(attempt))
            .map_err(debug_error)?;
        let roster = ActivityRosterLock::new(
            ActivityScopePath::new(self.activity.instance())
                .enter_section(section())
                .map_err(debug_error)?,
            (*self.definition.participants).clone(),
        )
        .map_err(debug_error)?;
        self.activity
            .engage_encounter(
                view.state_hash(),
                decision.id(),
                encounter_option(node_index),
                ActivityBattlePreparationRequest::new(
                    path,
                    roster,
                    BattleSequence::new(
                        u32::try_from(node_index + 1).expect("PureFiction has at most three nodes"),
                    )
                    .expect("battle sequence is non-zero"),
                    0,
                    preparation,
                ),
            )
            .map_err(debug_error)
    }

    pub fn choose_normal_engagement(&mut self) -> Result<(), PureFictionAttemptError> {
        let node_index = self.node_index()?;
        self.activity
            .choose_preparation_option(self.state_hash(), normal_option(node_index))
            .map_err(debug_error)?;
        Ok(())
    }

    pub fn start_pending_battle(
        &mut self,
    ) -> Result<ActivityBattleHandoff, PureFictionAttemptError> {
        let node_index = self.node_index()?;
        self.activity
            .start_pending_battle(
                self.state_hash(),
                Arc::clone(&self.definition.contracts[node_index]),
            )
            .map_err(debug_error)
    }

    pub fn submit_battle_result(
        &mut self,
        result: BattleResult,
    ) -> Result<GraphActivityBattleResolution, PureFictionAttemptError> {
        self.activity
            .submit_pending_battle_result(self.state_hash(), result)
            .map_err(debug_error)
    }

    fn node_index(&self) -> Result<usize, PureFictionAttemptError> {
        let raw = usize::try_from(self.activity.current_node().get())
            .map_err(|_| error("PureFiction node identity exceeds usize"))?;
        if (1..=self.definition.stage().nodes.len()).contains(&raw) {
            Ok(raw - 1)
        } else {
            Err(error("PureFiction attempt is not at a battle node"))
        }
    }

    fn integer_slot(&self, slot: ActivitySlotId) -> i64 {
        self.activity
            .player_view()
            .slots()
            .iter()
            .find(|entry| entry.id() == slot)
            .and_then(|entry| match entry.value() {
                ActivityValue::BoundedInteger(value) => Some(*value),
                _ => None,
            })
            .unwrap_or_default()
    }
}

fn activity_graph(node_count: usize) -> Result<ActivityGraphDefinition, PureFictionAttemptError> {
    let mut nodes = (1..=node_count)
        .map(|raw| {
            activity_node(
                u32::try_from(raw).expect("at most three nodes"),
                ActivityNodeKind::Battle,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    nodes.extend([
        activity_node(
            COMPLETED,
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
        )?,
        activity_node(
            FAILED,
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Failed),
        )?,
        activity_node(
            FAULTED,
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Faulted),
        )?,
    ]);
    let mut edges = Vec::with_capacity(node_count * 3);
    for index in 0..node_count {
        let from = u32::try_from(index + 1).expect("at most three nodes");
        let next = if index + 1 == node_count {
            COMPLETED
        } else {
            from + 1
        };
        let edge = u32::try_from(index * 3 + 1).expect("at most nine edges");
        edges.extend([
            outcome_edge(edge, from, next, BattleOutcome::Won)?,
            outcome_edge(edge + 1, from, FAILED, BattleOutcome::Lost)?,
            outcome_edge(edge + 2, from, FAULTED, BattleOutcome::Faulted)?,
        ]);
    }
    ActivityGraphDefinition::new(
        node(1),
        nodes,
        edges,
        u32::try_from(node_count + 2).expect("small graph depth"),
    )
    .map_err(debug_error)
}

fn activity_state(node_count: usize) -> Result<ActivityStateDefinition, PureFictionAttemptError> {
    let mut slots = Vec::new();
    for index in 0..node_count {
        for (slot, maximum) in [
            (score_slot(index), 40_000),
            (wave_one_slot(index), 8_000),
            (wave_two_slot(index), 16_000),
            (wave_three_slot(index), 16_000),
        ] {
            slots.push(
                ActivitySlotDefinition::new_with_policy(
                    slot,
                    ActivityScope::Section,
                    ActivityValue::BoundedInteger(0),
                    Some((0, maximum)),
                    None,
                    vec![],
                    SlotCarryPolicy::CarryExact,
                    ActivityStateVisibility::Player,
                    ActivityStateSource::new(u64::from(slot.get())).expect("slot id is non-zero"),
                )
                .map_err(debug_error)?,
            );
        }
    }
    ActivityStateDefinition::new(slots, vec![], vec![]).map_err(debug_error)
}

fn score_contract(
    participants: &[ParticipantId],
    projection_id: u32,
    node_index: usize,
) -> Result<ActivityBattleResultContract, PureFictionAttemptError> {
    let mut fields = vec![
        ProjectionField::Outcome,
        ProjectionField::FinalStateHash,
        ProjectionField::EventDigest,
        ProjectionField::TerminalFault,
    ];
    fields.extend(
        participants
            .iter()
            .copied()
            .map(ProjectionField::ParticipantState),
    );
    fields.extend(
        [
            NODE_SCORE_KEY,
            WAVE_ONE_SCORE_KEY,
            WAVE_TWO_SCORE_KEY,
            WAVE_THREE_SCORE_KEY,
        ]
        .map(|key| ProjectionField::Metric {
            key: key.into(),
            kind: MetricValueKind::BoundedInteger,
        }),
    );
    let projection = Arc::new(
        BattleResultProjection::new(
            ProjectionId::new(projection_id).expect("projection id is non-zero"),
            fields,
        )
        .map_err(debug_error)?,
    );
    ActivityBattleResultContract::new(
        projection,
        participants
            .iter()
            .copied()
            .map(|participant| {
                ActivityParticipantCarryDefinition::new(
                    participant,
                    HpCarryPolicy::CarryExact,
                    EnergyCarryPolicy::CarryExact,
                    LifeCarryPolicy::CarryExact,
                    PresenceCarryPolicy::CarryExact,
                )
            })
            .collect(),
        vec![
            metric_binding(NODE_SCORE_KEY, score_slot(node_index)),
            metric_binding(WAVE_ONE_SCORE_KEY, wave_one_slot(node_index)),
            metric_binding(WAVE_TWO_SCORE_KEY, wave_two_slot(node_index)),
            metric_binding(WAVE_THREE_SCORE_KEY, wave_three_slot(node_index)),
        ],
    )
    .map_err(debug_error)
}

fn metric_binding(key: &str, slot: ActivitySlotId) -> ActivityMetricProjectionBinding {
    ActivityMetricProjectionBinding::new(
        key,
        MetricValueKind::BoundedInteger,
        slot,
        MetricSettlementPolicy::Replace,
    )
    .expect("score metric key is valid")
}

fn participant_ids(participants: &ParticipantLock, team_index: u8) -> Vec<ParticipantId> {
    participants
        .entries()
        .iter()
        .filter(|entry| entry.team_index() == team_index)
        .map(|entry| entry.participant())
        .collect()
}

fn battle_program(
    raw_node: usize,
    index: usize,
) -> Result<GraphActivityNodeProgram, PureFictionAttemptError> {
    let option = ActivityOptionDefinition::new(
        encounter_option(index),
        0,
        ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true))),
        vec![],
    );
    ActivityProgramDefinition::new(
        ActivityProgramId::new(u32::try_from(raw_node).expect("at most three nodes"))
            .expect("program id is non-zero"),
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Encounter,
            options: vec![option].into_boxed_slice(),
        }],
    )
    .map(|program| GraphActivityNodeProgram::new(node(index + 1), program))
    .map_err(debug_error)
}

fn activity_node(
    raw: u32,
    kind: ActivityNodeKind,
) -> Result<ActivityNodeDefinition, PureFictionAttemptError> {
    ActivityNodeDefinition::new(node(raw as usize), section(), kind, 1).map_err(debug_error)
}

fn outcome_edge(
    id: u32,
    from: u32,
    to: u32,
    outcome: BattleOutcome,
) -> Result<ActivityEdgeDefinition, PureFictionAttemptError> {
    ActivityEdgeDefinition::new(
        ActivityEdgeId::new(id).expect("edge id is non-zero"),
        node(from as usize),
        node(to as usize),
        ActivityEdgeCondition::BattleOutcome(outcome.into()),
        i32::try_from(id).expect("small edge id"),
        1,
    )
    .map_err(debug_error)
}

fn encounter_option(index: usize) -> ActivityOptionId {
    ActivityOptionId::new(3_001 + u64::try_from(index).expect("small node index"))
        .expect("encounter option id is non-zero")
}

fn normal_option(index: usize) -> ActivityOptionId {
    ActivityOptionId::new(4_001 + u64::try_from(index).expect("small node index"))
        .expect("normal option id is non-zero")
}

fn contribution(index: usize) -> TechniqueContributionDigest {
    TechniqueContributionDigest::new([u8::try_from(index + 3).expect("small node index"); 32])
        .expect("non-zero contribution digest")
}

fn seed_label(index: usize) -> &'static str {
    match index {
        0 => "pure_fiction-shadow/node-1",
        1 => "pure_fiction-shadow/node-2",
        2 => "pure_fiction-shadow/node-3",
        _ => unreachable!("PureFiction has at most three nodes"),
    }
}

fn score_slot(index: usize) -> ActivitySlotId {
    slot_id(SCORE_SLOT_BASE + u32::try_from(index).expect("small node index") * 4)
}

fn wave_one_slot(index: usize) -> ActivitySlotId {
    slot_id(SCORE_SLOT_BASE + u32::try_from(index).expect("small node index") * 4 + 1)
}

fn wave_two_slot(index: usize) -> ActivitySlotId {
    slot_id(SCORE_SLOT_BASE + u32::try_from(index).expect("small node index") * 4 + 2)
}

fn wave_three_slot(index: usize) -> ActivitySlotId {
    slot_id(SCORE_SLOT_BASE + u32::try_from(index).expect("small node index") * 4 + 3)
}

fn section() -> SectionId {
    SectionId::new(SECTION).expect("section id is non-zero")
}

fn node(raw: usize) -> NodeId {
    NodeId::new(u32::try_from(raw).expect("small node id")).expect("node id is non-zero")
}

fn slot_id(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("slot id is non-zero")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PureFictionAttemptError {
    message: Box<str>,
}

impl std::fmt::Display for PureFictionAttemptError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for PureFictionAttemptError {}

fn debug_error(error: impl std::fmt::Debug) -> PureFictionAttemptError {
    PureFictionAttemptError {
        message: format!("{error:?}").into_boxed_str(),
    }
}

fn error(message: &str) -> PureFictionAttemptError {
    PureFictionAttemptError {
        message: message.into(),
    }
}
