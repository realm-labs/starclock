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
    ApocalypticProfile, ApocalypticStage, ObjectiveEvaluation,
    apocalyptic_projection::{BOSS_PROGRESS_SCORE_KEY, NODE_SCORE_KEY, REMAINING_AV_SCORE_KEY},
};

const SECTION: u32 = 2;
const COMPLETED: u32 = 100;
const FAILED: u32 = 101;
const FAULTED: u32 = 102;
const SCORE_SLOT_BASE: u32 = 10;

/// Fully linked ordinary or three-node Starward Apocalyptic Shadow attempt definition.
#[derive(Debug)]
pub struct ApocalypticAttemptDefinition {
    profile: Arc<ApocalypticProfile>,
    stage_index: usize,
    battles: Box<[BattleSpec]>,
    axioms: Box<[RuleBundleId]>,
    activity: Arc<GraphActivityDefinition>,
    participants: Arc<ParticipantLock>,
    contracts: Box<[Arc<ActivityBattleResultContract>]>,
}

impl ApocalypticAttemptDefinition {
    pub fn new(
        identity: starclock_activity::ActivityDefinitionIdentity,
        profile: Arc<ApocalypticProfile>,
        stage_index: usize,
        participants: ParticipantLock,
        battles: Vec<BattleSpec>,
        axioms: Vec<RuleBundleId>,
    ) -> Result<Self, ApocalypticAttemptError> {
        let stage = profile
            .stages
            .get(stage_index)
            .ok_or_else(|| error("Apocalyptic stage index is out of bounds"))?;
        let node_count = stage.nodes.len();
        if !(2..=3).contains(&node_count)
            || battles.len() != node_count
            || axioms.len() != node_count
            || usize::from(participants.policy().team_count()) != node_count
            || participants.policy().loadout_lock_scope() != LoadoutLockScope::Section
        {
            return Err(error(
                "Apocalyptic attempts require one Section-locked team and battle per node",
            ));
        }
        for (index, battle) in battles.iter().enumerate() {
            if battle.encounter() != stage.nodes[index].encounter {
                return Err(error(
                    "battle encounter does not match the selected Apocalyptic node",
                ));
            }
            if !stage.nodes[index].axiom_bundles.contains(&axioms[index])
                || !battle.participants().iter().any(|participant| {
                    participant.side() == TeamSide::Player
                        && participant
                            .combatant()
                            .rule_bundles()
                            .contains(&axioms[index])
                })
            {
                return Err(error(
                    "selected Apocalyptic Axiom is not compiled into its node battle",
                ));
            }
        }
        let team_participants = (0..node_count)
            .map(|index| {
                participant_ids(
                    &participants,
                    u8::try_from(index).expect("at most three Apocalyptic nodes"),
                )
            })
            .collect::<Vec<_>>();
        if team_participants.iter().any(Vec::is_empty) {
            return Err(error(
                "every Apocalyptic team must contain at least one participant",
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
            axioms: axioms.into_boxed_slice(),
            activity,
            participants,
            contracts: contracts.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn stage(&self) -> &ApocalypticStage {
        &self.profile.stages[self.stage_index]
    }

    #[must_use]
    pub fn axioms(&self) -> &[RuleBundleId] {
        &self.axioms
    }
}

/// Running wrapper; all authoritative mutation remains in `GraphActivity`.
#[derive(Debug)]
pub struct ApocalypticAttempt {
    definition: Arc<ApocalypticAttemptDefinition>,
    activity: GraphActivity,
}

impl ApocalypticAttempt {
    pub fn start(
        definition: Arc<ApocalypticAttemptDefinition>,
        instance: ActivityInstanceId,
        master_seed: ActivityMasterSeed,
    ) -> Result<Self, ApocalypticAttemptError> {
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
    pub fn boss_progress_score(&self, node_index: usize) -> i64 {
        self.integer_slot(progress_slot(node_index))
    }

    #[must_use]
    pub fn remaining_action_value_score(&self, node_index: usize) -> i64 {
        self.integer_slot(action_value_slot(node_index))
    }

    #[must_use]
    pub fn total_score(&self) -> i64 {
        (0..self.definition.stage().nodes.len())
            .map(|index| self.node_score(index))
            .sum()
    }

    #[must_use]
    pub fn objectives(&self) -> ObjectiveEvaluation {
        self.definition.stage().evaluate(self.total_score())
    }

    pub fn engage_current_node(
        &mut self,
        attempt: AttemptId,
    ) -> Result<GraphActivityPreparationResolution, ApocalypticAttemptError> {
        let node_index = self.node_index()?;
        let view = self.activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| decision.kind() == ActivityDecisionKind::Encounter)
            .ok_or_else(|| error("current Apocalyptic node is not offering an encounter"))?;
        if !decision
            .options()
            .iter()
            .any(|option| option.id() == encounter_option(node_index))
        {
            return Err(error("current Apocalyptic encounter option is not offered"));
        }
        let battle = self
            .definition
            .profile
            .compile_battle(
                self.definition.battles[node_index].clone(),
                self.definition.profile.clock.initial(),
            )
            .ok_or_else(|| error("Apocalyptic Action Value clock cannot be compiled"))?;
        let lock = self.definition.participants.digest();
        let preparation = Arc::new(
            EncounterPreparationDefinition::new(
                normal_option(node_index),
                EncounterInitiativePolicy::PlayerControlled,
                lock,
                u8::try_from(node_index).expect("Apocalyptic has at most three nodes"),
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
                        u32::try_from(node_index + 1).expect("Apocalyptic has at most three nodes"),
                    )
                    .expect("battle sequence is non-zero"),
                    0,
                    preparation,
                ),
            )
            .map_err(debug_error)
    }

    pub fn choose_normal_engagement(&mut self) -> Result<(), ApocalypticAttemptError> {
        let node_index = self.node_index()?;
        self.activity
            .choose_preparation_option(self.state_hash(), normal_option(node_index))
            .map_err(debug_error)?;
        Ok(())
    }

    pub fn start_pending_battle(
        &mut self,
    ) -> Result<ActivityBattleHandoff, ApocalypticAttemptError> {
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
    ) -> Result<GraphActivityBattleResolution, ApocalypticAttemptError> {
        self.activity
            .submit_pending_battle_result(self.state_hash(), result)
            .map_err(debug_error)
    }

    fn node_index(&self) -> Result<usize, ApocalypticAttemptError> {
        let raw = usize::try_from(self.activity.current_node().get())
            .map_err(|_| error("Apocalyptic node identity exceeds usize"))?;
        if (1..=self.definition.stage().nodes.len()).contains(&raw) {
            Ok(raw - 1)
        } else {
            Err(error("Apocalyptic attempt is not at a battle node"))
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

fn activity_graph(node_count: usize) -> Result<ActivityGraphDefinition, ApocalypticAttemptError> {
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

fn activity_state(node_count: usize) -> Result<ActivityStateDefinition, ApocalypticAttemptError> {
    let mut slots = Vec::new();
    for index in 0..node_count {
        for (slot, maximum) in [
            (score_slot(index), 4_000),
            (progress_slot(index), 2_000),
            (action_value_slot(index), 2_000),
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
) -> Result<ActivityBattleResultContract, ApocalypticAttemptError> {
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
            BOSS_PROGRESS_SCORE_KEY,
            REMAINING_AV_SCORE_KEY,
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
            metric_binding(BOSS_PROGRESS_SCORE_KEY, progress_slot(node_index)),
            metric_binding(REMAINING_AV_SCORE_KEY, action_value_slot(node_index)),
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
) -> Result<GraphActivityNodeProgram, ApocalypticAttemptError> {
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
) -> Result<ActivityNodeDefinition, ApocalypticAttemptError> {
    ActivityNodeDefinition::new(node(raw as usize), section(), kind, 1).map_err(debug_error)
}

fn outcome_edge(
    id: u32,
    from: u32,
    to: u32,
    outcome: BattleOutcome,
) -> Result<ActivityEdgeDefinition, ApocalypticAttemptError> {
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
        0 => "apocalyptic-shadow/node-1",
        1 => "apocalyptic-shadow/node-2",
        2 => "apocalyptic-shadow/node-3",
        _ => unreachable!("Apocalyptic has at most three nodes"),
    }
}

fn score_slot(index: usize) -> ActivitySlotId {
    slot_id(SCORE_SLOT_BASE + u32::try_from(index).expect("small node index") * 3)
}

fn progress_slot(index: usize) -> ActivitySlotId {
    slot_id(SCORE_SLOT_BASE + u32::try_from(index).expect("small node index") * 3 + 1)
}

fn action_value_slot(index: usize) -> ActivitySlotId {
    slot_id(SCORE_SLOT_BASE + u32::try_from(index).expect("small node index") * 3 + 2)
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
pub struct ApocalypticAttemptError {
    message: Box<str>,
}

impl std::fmt::Display for ApocalypticAttemptError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ApocalypticAttemptError {}

fn debug_error(error: impl std::fmt::Debug) -> ApocalypticAttemptError {
    ApocalypticAttemptError {
        message: format!("{error:?}").into_boxed_str(),
    }
}

fn error(message: &str) -> ApocalypticAttemptError {
    ApocalypticAttemptError {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use starclock_activity::{
        ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
        ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed,
        ActivityTerminalOutcome, BattleOutcome, BattleResult, BuildDigest, EventDigest,
        LoadoutLockScope, MetricValue, OpaqueParticipantBuild, ParticipantBattleState,
        ParticipantId, ParticipantLock, ParticipantLockEntry, ParticipantPolicy,
        ParticipantSourceKind, ParticipantUniquenessScope, ProjectedValue,
    };
    use starclock_combat::{
        AbilityId, ActionValue, AssemblyDigest, BattleClockExpiry, BattleStateHash,
        CombatantSpecDigest, ConcedePolicy, EncounterId, FormationIndex, Hp, LifeState,
        ParticipantSource, ParticipantSpec, PresenceState, ResolvedCombatantSpec,
        ResolvedDefinitionBindings, RuleBundleId, Speed, TeamResourceSpec, TeamSide,
        UnitDefinitionId, UnitLevel,
    };

    use super::{ApocalypticAttempt, ApocalypticAttemptDefinition};
    use crate::{
        ActionValueClockRule, ApocalypticNode, ApocalypticProfile, ApocalypticStage,
        ChallengeNodeId, ChallengeProfileId, ChallengeStageId, Objective, ObjectiveId,
        ObjectiveKind,
    };

    #[test]
    fn independent_node_scores_aggregate_and_timeout_advances() {
        let definition = Arc::new(
            ApocalypticAttemptDefinition::new(
                identity(),
                Arc::new(profile()),
                0,
                participants(),
                vec![battle(10, 101, 201, 0x11), battle(20, 102, 202, 0x12)],
                vec![axiom(), axiom()],
            )
            .unwrap(),
        );
        let mut attempt = ApocalypticAttempt::start(
            definition,
            ActivityInstanceId::new(7).unwrap(),
            ActivityMasterSeed::from_u64(9),
        )
        .unwrap();
        let first = start_node(&mut attempt, 1);
        assert_eq!(
            first.battle_spec().clock().and_then(|clock| match clock {
                starclock_combat::BattleClockSpec::ActionValue(clock) => {
                    Some(clock.remaining().scaled())
                }
                starclock_combat::BattleClockSpec::Cycles(_) => None,
            }),
            Some(2_000_000_000)
        );
        attempt
            .submit_battle_result(result(&first, BattleOutcome::Finalized, 1_000, 1_000, 0))
            .unwrap();
        let second = start_node(&mut attempt, 2);
        attempt
            .submit_battle_result(result(&second, BattleOutcome::Won, 2_500, 2_000, 500))
            .unwrap();
        assert_eq!(attempt.total_score(), 3_500);
        assert_eq!(attempt.node_score(0), 1_000);
        assert_eq!(attempt.remaining_action_value_score(1), 500);
        assert_eq!(attempt.objectives().stars(), 1);
        assert_eq!(
            attempt.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
    }

    #[test]
    fn starward_runs_three_independent_nodes() {
        let definition = Arc::new(
            ApocalypticAttemptDefinition::new(
                identity(),
                Arc::new(profile_with_nodes(3)),
                0,
                participants_with_teams(3),
                vec![
                    battle(10, 101, 201, 0x11),
                    battle(20, 102, 202, 0x12),
                    battle(30, 103, 203, 0x13),
                ],
                vec![axiom(), axiom(), axiom()],
            )
            .unwrap(),
        );
        let mut attempt = ApocalypticAttempt::start(
            definition,
            ActivityInstanceId::new(8).unwrap(),
            ActivityMasterSeed::from_u64(10),
        )
        .unwrap();
        for index in 0..3 {
            let handoff = start_node(&mut attempt, u32::try_from(index + 1).unwrap());
            attempt
                .submit_battle_result(result(&handoff, BattleOutcome::Won, 3_300, 2_000, 1_300))
                .unwrap();
        }
        assert_eq!(attempt.total_score(), 9_900);
        assert_eq!(attempt.objectives().stars(), 1);
        assert_eq!(
            attempt.debug_view().player().state_hash(),
            attempt.state_hash()
        );
        assert_eq!(
            attempt.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
    }

    fn start_node(
        attempt: &mut ApocalypticAttempt,
        raw_attempt: u32,
    ) -> starclock_activity::ActivityBattleHandoff {
        attempt
            .engage_current_node(starclock_activity::AttemptId::new(raw_attempt).unwrap())
            .unwrap();
        attempt.choose_normal_engagement().unwrap();
        attempt.start_pending_battle().unwrap()
    }

    fn profile() -> ApocalypticProfile {
        profile_with_nodes(2)
    }

    fn profile_with_nodes(count: usize) -> ApocalypticProfile {
        ApocalypticProfile {
            id: ChallengeProfileId::new(1).unwrap(),
            clock: ActionValueClockRule::new(
                ActionValue::from_scaled(2_000_000_000).unwrap(),
                BattleClockExpiry::Finalize,
            )
            .unwrap(),
            stages: vec![ApocalypticStage {
                id: ChallengeStageId::new(30_191).unwrap(),
                nodes: (0..count)
                    .map(|index| {
                        node(
                            u32::try_from(index + 1).unwrap(),
                            u32::try_from((index + 1) * 10).unwrap(),
                            u8::try_from(index).unwrap(),
                        )
                    })
                    .collect(),
                objectives: vec![Objective::new(
                    ObjectiveId::new(if count == 3 { 5_003 } else { 3_001 }).unwrap(),
                    ObjectiveKind::ScoreAtLeast(if count == 3 { 9_900 } else { 3_000 }),
                )]
                .into_boxed_slice(),
            }]
            .into_boxed_slice(),
            policies: Vec::new().into_boxed_slice(),
        }
    }

    fn node(id: u32, encounter: u32, team: u8) -> ApocalypticNode {
        ApocalypticNode {
            id: ChallengeNodeId::new(id).unwrap(),
            encounter: EncounterId::new(encounter).unwrap(),
            team_index: team,
            axiom_bundles: vec![axiom()].into_boxed_slice(),
        }
    }

    fn identity() -> ActivityDefinitionIdentity {
        ActivityDefinitionIdentity::new(
            ActivityDefinitionId::new(60).unwrap(),
            ActivityDefinitionDigest::new([0x51; 32]).unwrap(),
            ActivityConfigDigest::new([0x52; 32]).unwrap(),
        )
    }

    fn participants() -> ParticipantLock {
        participants_with_teams(2)
    }

    fn participants_with_teams(count: u8) -> ParticipantLock {
        ParticipantLock::seal(
            ParticipantPolicy::new(
                count,
                1,
                4,
                ParticipantUniquenessScope::Section,
                LoadoutLockScope::Section,
            )
            .unwrap(),
            (0..count)
                .map(|index| {
                    participant(
                        u32::from(index) + 1,
                        index,
                        u32::from(index) + 101,
                        0x11 + index,
                    )
                })
                .collect(),
        )
        .unwrap()
    }

    fn participant(id: u32, team: u8, character: u32, digest: u8) -> ParticipantLockEntry {
        ParticipantLockEntry::new(
            ParticipantId::new(id).unwrap(),
            team,
            0,
            UnitDefinitionId::new(character).unwrap(),
            OpaqueParticipantBuild::new(
                CombatantSpecDigest::new([digest; 32]).unwrap(),
                BuildDigest::new([digest.wrapping_add(1); 32]).unwrap(),
                ParticipantSourceKind::CompiledBuild,
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn battle(encounter: u32, player: u32, enemy: u32, digest: u8) -> starclock_combat::BattleSpec {
        starclock_combat::BattleSpec::new(
            AssemblyDigest::new([u8::try_from(encounter).unwrap(); 32]).unwrap(),
            EncounterId::new(encounter).unwrap(),
            vec![
                ParticipantSpec::new(
                    TeamSide::Player,
                    FormationIndex::new(0).unwrap(),
                    ParticipantSource::Player,
                    combatant(player, digest, true),
                ),
                ParticipantSpec::new(
                    TeamSide::Enemy,
                    FormationIndex::new(0).unwrap(),
                    ParticipantSource::Player,
                    combatant(enemy, digest.wrapping_add(0x20), false),
                ),
            ],
            TeamResourceSpec::new(3, 5).unwrap(),
            TeamResourceSpec::new(0, 0).unwrap(),
            ConcedePolicy::Allowed,
        )
        .unwrap()
    }

    fn combatant(form: u32, digest: u8, with_axiom: bool) -> ResolvedCombatantSpec {
        ResolvedCombatantSpec::new(
            UnitDefinitionId::new(form).unwrap(),
            UnitLevel::new(80).unwrap(),
            Hp::new(1_000).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
            ResolvedDefinitionBindings::new(
                vec![AbilityId::new(form).unwrap()],
                if with_axiom { vec![axiom()] } else { vec![] },
                vec![],
            )
            .unwrap(),
            CombatantSpecDigest::new([digest; 32]).unwrap(),
        )
        .unwrap()
    }

    fn axiom() -> RuleBundleId {
        RuleBundleId::new(777).unwrap()
    }

    fn result(
        handoff: &starclock_activity::ActivityBattleHandoff,
        outcome: BattleOutcome,
        total: i64,
        progress: i64,
        action_value: i64,
    ) -> BattleResult {
        let mut values = vec![
            ProjectedValue::Outcome(outcome),
            ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x61; 32])),
            ProjectedValue::EventDigest(EventDigest::new([0x62; 32]).unwrap()),
            ProjectedValue::TerminalFault(None),
        ];
        values.extend(handoff.participant_carry().iter().map(|carry| {
            ProjectedValue::ParticipantState(
                ParticipantBattleState::new(
                    carry.participant(),
                    carry.current_hp(),
                    carry.maximum_hp(),
                    carry.current_energy(),
                    carry.maximum_energy(),
                    LifeState::Alive,
                    PresenceState::Present,
                )
                .unwrap(),
            )
        }));
        for (key, value) in [
            ("node_score", total),
            ("boss_progress_score", progress),
            ("remaining_action_value_score", action_value),
        ] {
            values.push(ProjectedValue::Metric {
                key: key.into(),
                value: MetricValue::BoundedInteger(value),
            });
        }
        BattleResult::seal(handoff.identity(), values)
    }
}
