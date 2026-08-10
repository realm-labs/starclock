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
use starclock_combat::BattleSpec;

use crate::{
    ObjectiveEvaluation, ObjectiveInput,
    memory_of_chaos::{MemoryProfile, MemoryStage},
};

const SECTION: u32 = 1;
const COMPLETED: u32 = 100;
const FAILED: u32 = 101;
const FAULTED: u32 = 102;
const REMAINING_CYCLES_SLOT: u32 = 1;
const DEFEATED_ALLIES_SLOT: u32 = 2;
const REMAINING_CYCLES_KEY: &str = "remaining_cycles";
const DEFEATED_ALLIES_KEY: &str = "defeated_allies";

/// Fully linked ordinary or three-node Starward Memory of Chaos attempt definition.
#[derive(Debug)]
pub struct MemoryAttemptDefinition {
    profile: Arc<MemoryProfile>,
    stage_index: usize,
    battles: Box<[BattleSpec]>,
    activity: Arc<GraphActivityDefinition>,
    participants: Arc<ParticipantLock>,
    contracts: Box<[Arc<ActivityBattleResultContract>]>,
}

impl MemoryAttemptDefinition {
    pub fn new(
        identity: starclock_activity::ActivityDefinitionIdentity,
        profile: Arc<MemoryProfile>,
        stage_index: usize,
        participants: ParticipantLock,
        battles: Vec<BattleSpec>,
    ) -> Result<Self, MemoryAttemptError> {
        let stage = profile
            .stages()
            .get(stage_index)
            .ok_or_else(|| error("Memory stage index is out of bounds"))?;
        let node_count = stage.nodes().len();
        if !(2..=3).contains(&node_count)
            || battles.len() != node_count
            || usize::from(participants.policy().team_count()) != node_count
            || participants.policy().loadout_lock_scope() != LoadoutLockScope::Section
        {
            return Err(error(
                "Memory attempts require one Section-locked team and battle per node",
            ));
        }
        for (index, battle) in battles.iter().enumerate() {
            if battle.encounter() != stage.nodes()[index].encounter() {
                return Err(error(
                    "battle encounter does not match the selected Memory node",
                ));
            }
        }
        let graph = memory_graph(node_count)?;
        let state = memory_state(
            profile
                .initial_cycles(stage_index)
                .ok_or_else(|| error("Memory stage clock is missing"))?,
        )?;
        let programs = (0..node_count)
            .map(|index| battle_program(index + 1, index))
            .collect::<Result<Vec<_>, _>>()?;
        let team_participants = (0..node_count)
            .map(|index| {
                participant_ids(
                    &participants,
                    u8::try_from(index).expect("at most three Memory nodes"),
                )
            })
            .collect::<Vec<_>>();
        if team_participants.iter().any(Vec::is_empty) {
            return Err(error(
                "every Memory team must contain at least one participant",
            ));
        }
        let contracts = team_participants
            .iter()
            .enumerate()
            .map(|(index, participants)| {
                memory_contract(
                    participants,
                    u32::try_from(index + 1).expect("at most three projections"),
                )
                .map(Arc::new)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let participants = Arc::new(participants);
        let activity = Arc::new(
            GraphActivityDefinition::new(
                identity,
                graph,
                state,
                Arc::clone(&participants),
                programs,
                None,
                ActivityRandomPolicies::default(),
            )
            .map_err(debug_error)?,
        );
        Ok(Self {
            profile,
            stage_index,
            battles: battles.into_boxed_slice(),
            activity,
            participants,
            contracts: contracts.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn stage(&self) -> &MemoryStage {
        &self.profile.stages()[self.stage_index]
    }
}

/// Running wrapper; authoritative mutation remains inside `GraphActivity`.
#[derive(Debug)]
pub struct MemoryAttempt {
    definition: Arc<MemoryAttemptDefinition>,
    activity: GraphActivity,
}

impl MemoryAttempt {
    pub fn start(
        definition: Arc<MemoryAttemptDefinition>,
        instance: ActivityInstanceId,
        master_seed: ActivityMasterSeed,
    ) -> Result<Self, MemoryAttemptError> {
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
    pub fn remaining_cycles(&self) -> u16 {
        u16::try_from(self.integer_slot(REMAINING_CYCLES_SLOT)).unwrap_or_default()
    }

    /// Accepts the current graph encounter and creates its player preparation boundary.
    pub fn engage_current_node(
        &mut self,
        attempt: AttemptId,
    ) -> Result<GraphActivityPreparationResolution, MemoryAttemptError> {
        let node_index = self.node_index()?;
        let view = self.activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| decision.kind() == ActivityDecisionKind::Encounter)
            .ok_or_else(|| error("current Memory node is not offering an encounter"))?;
        let encounter_option = encounter_option(node_index);
        if !decision
            .options()
            .iter()
            .any(|option| option.id() == encounter_option)
        {
            return Err(error("current Memory encounter option is not offered"));
        }
        let battle = self
            .definition
            .profile
            .compile_battle(
                self.definition.stage_index,
                self.definition.battles[node_index].clone(),
                self.remaining_cycles(),
            )
            .ok_or_else(|| error("remaining Memory cycles cannot compile a battle clock"))?;
        let lock = self.definition.participants.digest();
        let preparation = Arc::new(
            EncounterPreparationDefinition::new(
                normal_option(node_index),
                EncounterInitiativePolicy::PlayerControlled,
                lock,
                u8::try_from(node_index).expect("Memory has at most three nodes"),
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
                encounter_option,
                ActivityBattlePreparationRequest::new(
                    path,
                    roster,
                    BattleSequence::new(
                        u32::try_from(node_index + 1).expect("Memory has at most three nodes"),
                    )
                    .expect("battle sequence is non-zero"),
                    0,
                    preparation,
                ),
            )
            .map_err(debug_error)
    }

    pub fn choose_normal_engagement(&mut self) -> Result<(), MemoryAttemptError> {
        let node_index = self.node_index()?;
        self.activity
            .choose_preparation_option(self.state_hash(), normal_option(node_index))
            .map_err(debug_error)?;
        Ok(())
    }

    pub fn start_pending_battle(&mut self) -> Result<ActivityBattleHandoff, MemoryAttemptError> {
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
    ) -> Result<GraphActivityBattleResolution, MemoryAttemptError> {
        self.activity
            .submit_pending_battle_result(self.state_hash(), result)
            .map_err(debug_error)
    }

    #[must_use]
    pub fn objectives(&self) -> ObjectiveEvaluation {
        self.definition.stage().evaluate(ObjectiveInput {
            completed: self.activity.player_view().terminal()
                == Some(ActivityTerminalOutcome::Completed),
            any_participant_defeated: self.integer_slot(DEFEATED_ALLIES_SLOT) > 0,
            remaining_cycles: Some(self.remaining_cycles()),
            score: None,
        })
    }

    fn node_index(&self) -> Result<usize, MemoryAttemptError> {
        let raw = usize::try_from(self.activity.current_node().get())
            .map_err(|_| error("Memory node identity exceeds usize"))?;
        if (1..=self.definition.stage().nodes().len()).contains(&raw) {
            Ok(raw - 1)
        } else {
            Err(error("Memory attempt is not at a battle node"))
        }
    }

    fn integer_slot(&self, raw: u32) -> i64 {
        self.activity
            .player_view()
            .slots()
            .iter()
            .find(|slot| slot.id() == slot_id(raw))
            .and_then(|slot| match slot.value() {
                ActivityValue::BoundedInteger(value) => Some(*value),
                _ => None,
            })
            .unwrap_or_default()
    }
}

fn memory_graph(node_count: usize) -> Result<ActivityGraphDefinition, MemoryAttemptError> {
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

fn memory_state(initial_cycles: u16) -> Result<ActivityStateDefinition, MemoryAttemptError> {
    let slots = vec![
        ActivitySlotDefinition::new_with_policy(
            slot_id(REMAINING_CYCLES_SLOT),
            ActivityScope::Section,
            ActivityValue::BoundedInteger(i64::from(initial_cycles)),
            Some((0, i64::from(initial_cycles))),
            None,
            vec![],
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Player,
            ActivityStateSource::new(1).expect("source id is non-zero"),
        )
        .map_err(debug_error)?,
        ActivitySlotDefinition::new_with_policy(
            slot_id(DEFEATED_ALLIES_SLOT),
            ActivityScope::Section,
            ActivityValue::BoundedInteger(0),
            Some((0, 8)),
            None,
            vec![],
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Player,
            ActivityStateSource::new(2).expect("source id is non-zero"),
        )
        .map_err(debug_error)?,
    ];
    ActivityStateDefinition::new(slots, vec![], vec![]).map_err(debug_error)
}

fn memory_contract(
    participants: &[ParticipantId],
    projection_id: u32,
) -> Result<ActivityBattleResultContract, MemoryAttemptError> {
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
    fields.extend([
        ProjectionField::Metric {
            key: REMAINING_CYCLES_KEY.into(),
            kind: MetricValueKind::BoundedInteger,
        },
        ProjectionField::Metric {
            key: DEFEATED_ALLIES_KEY.into(),
            kind: MetricValueKind::BoundedInteger,
        },
    ]);
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
            ActivityMetricProjectionBinding::new(
                REMAINING_CYCLES_KEY,
                MetricValueKind::BoundedInteger,
                slot_id(REMAINING_CYCLES_SLOT),
                MetricSettlementPolicy::Replace,
            )
            .expect("metric key is valid"),
            ActivityMetricProjectionBinding::new(
                DEFEATED_ALLIES_KEY,
                MetricValueKind::BoundedInteger,
                slot_id(DEFEATED_ALLIES_SLOT),
                MetricSettlementPolicy::Sum,
            )
            .expect("metric key is valid"),
        ],
    )
    .map_err(debug_error)
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
) -> Result<GraphActivityNodeProgram, MemoryAttemptError> {
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
) -> Result<ActivityNodeDefinition, MemoryAttemptError> {
    ActivityNodeDefinition::new(node(raw as usize), section(), kind, 1).map_err(debug_error)
}

fn outcome_edge(
    id: u32,
    from: u32,
    to: u32,
    outcome: BattleOutcome,
) -> Result<ActivityEdgeDefinition, MemoryAttemptError> {
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
    ActivityOptionId::new(1_001 + u64::try_from(index).expect("small node index"))
        .expect("encounter option id is non-zero")
}

fn normal_option(index: usize) -> ActivityOptionId {
    ActivityOptionId::new(2_001 + u64::try_from(index).expect("small node index"))
        .expect("normal option id is non-zero")
}

fn contribution(index: usize) -> TechniqueContributionDigest {
    TechniqueContributionDigest::new([u8::try_from(index + 1).expect("small node index"); 32])
        .expect("non-zero contribution digest")
}

fn seed_label(index: usize) -> &'static str {
    match index {
        0 => "memory-of-chaos/node-1",
        1 => "memory-of-chaos/node-2",
        2 => "memory-of-chaos/node-3",
        _ => unreachable!("Memory has at most three nodes"),
    }
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
pub struct MemoryAttemptError {
    message: Box<str>,
}

impl std::fmt::Display for MemoryAttemptError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for MemoryAttemptError {}

fn debug_error(error: impl std::fmt::Debug) -> MemoryAttemptError {
    MemoryAttemptError {
        message: format!("{error:?}").into_boxed_str(),
    }
}

fn error(message: &str) -> MemoryAttemptError {
    MemoryAttemptError {
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
        CombatantSpecDigest, ConcedePolicy, EncounterId, EnemyDefinitionId, FormationIndex, Hp,
        LifeState, ParticipantSource, ParticipantSpec, PresenceState, ResolvedCombatantSpec,
        ResolvedDefinitionBindings, RuleBundleId, Speed, TeamResourceSpec, TeamSide,
        UnitDefinitionId, UnitLevel,
    };

    use super::{MemoryAttempt, MemoryAttemptDefinition};
    use crate::{
        ChallengeNodeId, ChallengeProfileId, ChallengeStageId, CycleClockRule, Objective,
        ObjectiveId, ObjectiveKind,
        memory_of_chaos::{MemoryNode, MemoryProfile, MemoryStage},
    };

    #[test]
    fn node_two_receives_node_one_remaining_cycles_and_objectives_use_stage_state() {
        let definition = Arc::new(
            MemoryAttemptDefinition::new(
                identity(),
                Arc::new(profile()),
                0,
                participants(),
                vec![battle(10, 101, 201, 0x11), battle(20, 102, 202, 0x12)],
            )
            .expect("Memory attempt definition is valid"),
        );
        let mut attempt = MemoryAttempt::start(
            definition,
            ActivityInstanceId::new(7).unwrap(),
            ActivityMasterSeed::from_u64(9),
        )
        .expect("Memory attempt starts");
        attempt
            .engage_current_node(starclock_activity::AttemptId::new(1).unwrap())
            .expect("node one encounter engages");
        attempt
            .choose_normal_engagement()
            .expect("normal engagement prepares node one");
        let first = attempt.start_pending_battle().expect("node one handoff");
        assert_eq!(
            first.battle_spec().clock().and_then(|clock| match clock {
                starclock_combat::BattleClockSpec::Cycles(clock) => {
                    Some(clock.remaining_cycles())
                }
                starclock_combat::BattleClockSpec::ActionValue(_) => None,
            }),
            Some(30)
        );
        attempt
            .submit_battle_result(result(&first, 24, 1))
            .expect("node one settles");
        assert_eq!(attempt.remaining_cycles(), 24);
        attempt
            .engage_current_node(starclock_activity::AttemptId::new(2).unwrap())
            .expect("node two encounter engages");
        attempt
            .choose_normal_engagement()
            .expect("normal engagement prepares node two");
        let second = attempt.start_pending_battle().expect("node two handoff");
        assert_eq!(
            second.battle_spec().clock().and_then(|clock| match clock {
                starclock_combat::BattleClockSpec::Cycles(clock) => {
                    Some(clock.remaining_cycles())
                }
                starclock_combat::BattleClockSpec::ActionValue(_) => None,
            }),
            Some(24)
        );
        attempt
            .submit_battle_result(result(&second, 20, 0))
            .expect("node two settles");
        assert_eq!(
            attempt.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
        assert_eq!(attempt.objectives().stars(), 2);
    }

    #[test]
    fn starward_carries_cycles_through_three_nodes() {
        let definition = Arc::new(
            MemoryAttemptDefinition::new(
                identity(),
                Arc::new(profile_with_nodes(3, 45)),
                0,
                participants_with_teams(3),
                vec![
                    battle(10, 101, 201, 0x11),
                    battle(20, 102, 202, 0x12),
                    battle(30, 103, 203, 0x13),
                ],
            )
            .unwrap(),
        );
        let mut attempt = MemoryAttempt::start(
            definition,
            ActivityInstanceId::new(8).unwrap(),
            ActivityMasterSeed::from_u64(10),
        )
        .unwrap();
        for (index, remaining) in [41, 37, 33].into_iter().enumerate() {
            attempt
                .engage_current_node(
                    starclock_activity::AttemptId::new(u32::try_from(index + 1).unwrap()).unwrap(),
                )
                .unwrap();
            attempt.choose_normal_engagement().unwrap();
            let handoff = attempt.start_pending_battle().unwrap();
            attempt
                .submit_battle_result(result(&handoff, remaining, 0))
                .unwrap();
        }
        assert_eq!(attempt.remaining_cycles(), 33);
        assert_eq!(attempt.objectives().stars(), 3);
        assert_eq!(
            attempt.debug_view().player().state_hash(),
            attempt.state_hash()
        );
        assert_eq!(
            attempt.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
    }

    fn profile() -> MemoryProfile {
        profile_with_nodes(2, 30)
    }

    fn profile_with_nodes(count: usize, cycles: u16) -> MemoryProfile {
        let nodes = (0..count)
            .map(|index| {
                MemoryNode::new(
                    ChallengeNodeId::new(u32::try_from(index + 1).unwrap()).unwrap(),
                    EncounterId::new(u32::try_from((index + 1) * 10).unwrap()).unwrap(),
                    u8::try_from(index).unwrap(),
                    vec![RuleBundleId::new(30_146).unwrap()],
                )
                .unwrap()
            })
            .collect();
        let clock = CycleClockRule::new(
            cycles,
            ActionValue::from_scaled(150_000_000).unwrap(),
            ActionValue::from_scaled(100_000_000).unwrap(),
            true,
            BattleClockExpiry::Lose,
        )
        .unwrap();
        let stage = MemoryStage::new(
            ChallengeStageId::new(5_201).unwrap(),
            clock,
            nodes,
            vec![
                Objective::new(
                    ObjectiveId::new(if count == 3 { 601 } else { 251 }).unwrap(),
                    ObjectiveKind::RemainingCyclesAtLeast(if count == 3 { 15 } else { 10 }),
                ),
                Objective::new(
                    ObjectiveId::new(if count == 3 { 602 } else { 252 }).unwrap(),
                    ObjectiveKind::RemainingCyclesAtLeast(if count == 3 { 30 } else { 20 }),
                ),
                Objective::new(
                    ObjectiveId::new(if count == 3 { 603 } else { 253 }).unwrap(),
                    ObjectiveKind::NoDefeatedParticipants,
                ),
            ],
        )
        .unwrap();
        MemoryProfile::new(
            ChallengeProfileId::new(1).unwrap(),
            clock,
            vec![stage],
            vec![],
        )
        .unwrap()
    }

    fn identity() -> ActivityDefinitionIdentity {
        ActivityDefinitionIdentity::new(
            ActivityDefinitionId::new(50).unwrap(),
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

    fn battle(
        encounter: u32,
        player: u32,
        enemy: u32,
        player_digest: u8,
    ) -> starclock_combat::BattleSpec {
        starclock_combat::BattleSpec::new(
            AssemblyDigest::new([u8::try_from(encounter).unwrap(); 32]).unwrap(),
            EncounterId::new(encounter).unwrap(),
            vec![
                ParticipantSpec::new(
                    TeamSide::Player,
                    FormationIndex::new(0).unwrap(),
                    ParticipantSource::Player,
                    combatant(player, player_digest),
                ),
                ParticipantSpec::new(
                    TeamSide::Enemy,
                    FormationIndex::new(0).unwrap(),
                    ParticipantSource::EncounterEnemy(EnemyDefinitionId::new(enemy).unwrap()),
                    combatant(enemy, player_digest.wrapping_add(0x20)),
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

    fn result(
        handoff: &starclock_activity::ActivityBattleHandoff,
        remaining_cycles: i64,
        defeated_allies: i64,
    ) -> BattleResult {
        let mut values = vec![
            ProjectedValue::Outcome(BattleOutcome::Won),
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
        values.extend([
            ProjectedValue::Metric {
                key: "remaining_cycles".into(),
                value: MetricValue::BoundedInteger(remaining_cycles),
            },
            ProjectedValue::Metric {
                key: "defeated_allies".into(),
                value: MetricValue::BoundedInteger(defeated_allies),
            },
        ]);
        BattleResult::seal(handoff.identity(), values)
    }
}
