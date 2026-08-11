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
use starclock_combat::{BattleSpec, TeamSide};

use crate::{AnomalyProfile, AnomalyQuadrantId, AnomalyStage, AnomalyStageKind, ChallengeStageId};

pub const ANOMALY_WON_KEY: &str = "anomaly_won";
pub const ANOMALY_STARS_KEY: &str = "anomaly_stars";

const SECTION: u32 = 4;
const HUB: u32 = 1;
const BATTLE_BASE: u32 = 10;
const COMPLETED: u32 = 100;
const FAULTED: u32 = 102;
const SELECTED_STAGE: u32 = 1;
const SELECTED_QUADRANT: u32 = 2;
const CLEAR_BASE: u32 = 10;
const STAR_BASE: u32 = 20;

#[derive(Debug)]
pub struct AnomalyArbitrationDefinition {
    profile: Arc<AnomalyProfile>,
    activity: Arc<GraphActivityDefinition>,
    participants: Arc<ParticipantLock>,
    contracts: Box<[Arc<ActivityBattleResultContract>]>,
}

impl AnomalyArbitrationDefinition {
    pub fn new(
        identity: starclock_activity::ActivityDefinitionIdentity,
        profile: Arc<AnomalyProfile>,
        participants: ParticipantLock,
    ) -> Result<Self, AnomalyArbitrationError> {
        validate_participants(&participants)?;
        let contracts = profile
            .stages
            .iter()
            .enumerate()
            .map(|(index, stage)| {
                stage_contract(
                    &participant_ids(&participants, stage.team_index),
                    u32::try_from(index + 1).expect("Anomaly has five stages"),
                    index,
                )
                .map(Arc::new)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let participants = Arc::new(participants);
        let activity = Arc::new(
            GraphActivityDefinition::new(
                identity,
                activity_graph(&profile)?,
                activity_state()?,
                Arc::clone(&participants),
                node_programs(&profile)?,
                None,
                ActivityRandomPolicies::default(),
            )
            .map_err(debug_error)?,
        );
        Ok(Self {
            profile,
            activity,
            participants,
            contracts: contracts.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn profile(&self) -> &AnomalyProfile {
        &self.profile
    }
}

#[derive(Debug)]
pub struct AnomalyArbitration {
    definition: Arc<AnomalyArbitrationDefinition>,
    activity: GraphActivity,
}

impl AnomalyArbitration {
    pub fn start(
        definition: Arc<AnomalyArbitrationDefinition>,
        instance: ActivityInstanceId,
        master_seed: ActivityMasterSeed,
    ) -> Result<Self, AnomalyArbitrationError> {
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
    pub fn stage_record(&self, stage: ChallengeStageId) -> Option<AnomalyStageRecord> {
        let index = self.stage_index(stage)?;
        Some(AnomalyStageRecord {
            cleared: self.integer_slot(clear_slot(index)) > 0,
            best_stars: u8::try_from(self.integer_slot(star_slot(index))).unwrap_or_default(),
        })
    }

    #[must_use]
    pub fn cleared_knight_count(&self) -> u8 {
        self.definition
            .profile
            .stages
            .iter()
            .enumerate()
            .filter(|(index, stage)| {
                matches!(stage.kind, AnomalyStageKind::Knight { .. })
                    && self.integer_slot(clear_slot(*index)) > 0
            })
            .count()
            .try_into()
            .expect("there are exactly three Knights")
    }

    #[must_use]
    pub fn normal_king_available(&self) -> bool {
        self.cleared_knight_count() == 3
    }

    #[must_use]
    pub fn selected_stage(&self) -> Option<&AnomalyStage> {
        let raw = self.optional_id(SELECTED_STAGE)?;
        self.definition
            .profile
            .stage(ChallengeStageId::new(u32::try_from(raw).ok()?)?)
    }

    #[must_use]
    pub fn selected_quadrant(&self) -> Option<AnomalyQuadrantId> {
        AnomalyQuadrantId::new(u32::try_from(self.optional_id(SELECTED_QUADRANT)?).ok()?)
    }

    #[must_use]
    pub fn king_protection_contributions(&self) -> u8 {
        self.selected_stage()
            .map(|stage| AnomalyStage::protection_contributions(stage.kind))
            .unwrap_or_default()
    }

    pub fn choose_stage(
        &mut self,
        stage: ChallengeStageId,
        quadrant: Option<AnomalyQuadrantId>,
    ) -> Result<(), AnomalyArbitrationError> {
        let index = self
            .stage_index(stage)
            .ok_or_else(|| error("Anomaly stage is not in the active profile"))?;
        let stage = &self.definition.profile.stages[index];
        let quadrant_index = match (stage.kind, quadrant) {
            (AnomalyStageKind::Knight { .. }, None) => None,
            (AnomalyStageKind::Knight { .. }, Some(_)) => {
                return Err(error("Knight stages do not accept a Quadrant"));
            }
            (AnomalyStageKind::KingNormal | AnomalyStageKind::KingPlight, Some(id)) => Some(
                self.definition
                    .profile
                    .quadrants
                    .iter()
                    .position(|candidate| candidate.id == id)
                    .ok_or_else(|| error("Quadrant is not in the active profile"))?,
            ),
            (AnomalyStageKind::KingNormal | AnomalyStageKind::KingPlight, None) => {
                return Err(error("King stages require exactly one Quadrant"));
            }
        };
        let view = self.activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| decision.kind() == ActivityDecisionKind::Route)
            .ok_or_else(|| error("Anomaly route selection is not currently offered"))?;
        self.activity
            .choose_option(
                view.state_hash(),
                decision.id(),
                route_option(index, quadrant_index),
            )
            .map_err(debug_error)?;
        Ok(())
    }

    pub fn engage_selected_stage(
        &mut self,
        attempt: AttemptId,
        battle: BattleSpec,
    ) -> Result<GraphActivityPreparationResolution, AnomalyArbitrationError> {
        let index = self.current_stage_index()?;
        let stage = &self.definition.profile.stages[index];
        if battle.encounter() != stage.encounter {
            return Err(error(
                "battle encounter does not match selected Anomaly stage",
            ));
        }
        if let Some(quadrant) = self.selected_quadrant() {
            let bundle = self
                .definition
                .profile
                .quadrant(quadrant)
                .expect("selected Quadrant came from the profile")
                .rule_bundle;
            if !battle.participants().iter().any(|participant| {
                participant.side() == TeamSide::Player
                    && participant.combatant().rule_bundles().contains(&bundle)
            }) {
                return Err(error(
                    "selected Quadrant is not compiled into the King battle",
                ));
            }
        }
        let battle = stage
            .compile_battle(battle)
            .ok_or_else(|| error("Anomaly stage clock cannot be compiled"))?;
        let view = self.activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| decision.kind() == ActivityDecisionKind::Encounter)
            .ok_or_else(|| error("selected Anomaly stage is not offering an encounter"))?;
        let lock = self.definition.participants.digest();
        let preparation = Arc::new(
            EncounterPreparationDefinition::new(
                preparation_option(index),
                EncounterInitiativePolicy::PlayerControlled,
                lock,
                stage.team_index,
                vec![],
                vec![starclock_activity::PreparedBattleVariant::new(
                    vec![],
                    contribution(index, self.selected_quadrant()),
                    BattleBinding::new(battle, seed_label(index), lock).map_err(debug_error)?,
                )],
            )
            .map_err(debug_error)?,
        );
        let path = ActivityScopePath::new(self.activity.instance())
            .enter_section(section())
            .and_then(|path| path.enter_node(battle_node(index)))
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
                encounter_option(index),
                ActivityBattlePreparationRequest::new(
                    path,
                    roster,
                    BattleSequence::new(u32::try_from(index + 1).expect("five stages"))
                        .expect("battle sequence is non-zero"),
                    0,
                    preparation,
                ),
            )
            .map_err(debug_error)
    }

    pub fn choose_prepared_battle(&mut self) -> Result<(), AnomalyArbitrationError> {
        let index = self.current_stage_index()?;
        self.activity
            .choose_preparation_option(self.state_hash(), preparation_option(index))
            .map_err(debug_error)?;
        Ok(())
    }

    pub fn start_pending_battle(
        &mut self,
    ) -> Result<ActivityBattleHandoff, AnomalyArbitrationError> {
        let index = self.current_stage_index()?;
        self.activity
            .start_pending_battle(
                self.state_hash(),
                Arc::clone(&self.definition.contracts[index]),
            )
            .map_err(debug_error)
    }

    pub fn submit_battle_result(
        &mut self,
        result: BattleResult,
    ) -> Result<GraphActivityBattleResolution, AnomalyArbitrationError> {
        let program = clear_selection_program()?;
        self.activity
            .submit_pending_battle_result_with_boundary_program(
                self.state_hash(),
                result,
                Some(&program),
            )
            .map_err(debug_error)
    }

    fn stage_index(&self, stage: ChallengeStageId) -> Option<usize> {
        self.definition
            .profile
            .stages
            .iter()
            .position(|candidate| candidate.id == stage)
    }

    fn current_stage_index(&self) -> Result<usize, AnomalyArbitrationError> {
        let raw = self.activity.current_node().get();
        let index = raw
            .checked_sub(BATTLE_BASE)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| error("Anomaly runtime is not at a battle stage"))?;
        if index >= self.definition.profile.stages.len() {
            return Err(error("Anomaly battle node is outside the active profile"));
        }
        Ok(index)
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

    fn optional_id(&self, raw: u32) -> Option<u64> {
        self.activity
            .player_view()
            .slots()
            .iter()
            .find(|entry| entry.id() == slot(raw))
            .and_then(|entry| match entry.value() {
                ActivityValue::OptionalId(value) => *value,
                _ => None,
            })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnomalyStageRecord {
    pub cleared: bool,
    pub best_stars: u8,
}

fn validate_participants(participants: &ParticipantLock) -> Result<(), AnomalyArbitrationError> {
    if participants.policy().team_count() != 4
        || participants.policy().loadout_lock_scope() != LoadoutLockScope::Activity
        || (0..4).any(|team| participant_ids(participants, team).is_empty())
    {
        return Err(error(
            "Anomaly runtime requires three Knight teams and one King team locked to the Activity",
        ));
    }
    let knight_entries = participants
        .entries()
        .iter()
        .filter(|entry| entry.team_index() < 3)
        .collect::<Vec<_>>();
    if knight_entries.iter().enumerate().any(|(index, entry)| {
        knight_entries[..index]
            .iter()
            .any(|prior| prior.character() == entry.character())
    }) {
        return Err(error(
            "Anomaly Knight teams must use disjoint character identities",
        ));
    }
    Ok(())
}

fn activity_graph(
    profile: &AnomalyProfile,
) -> Result<ActivityGraphDefinition, AnomalyArbitrationError> {
    let mut nodes = vec![activity_node(HUB, ActivityNodeKind::Choice, 128)?];
    nodes.extend(
        (0..profile.stages.len())
            .map(|index| activity_node(BATTLE_BASE + raw(index), ActivityNodeKind::Battle, 64))
            .collect::<Result<Vec<_>, _>>()?,
    );
    nodes.extend([
        activity_node(
            COMPLETED,
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            1,
        )?,
        activity_node(
            FAULTED,
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Faulted),
            1,
        )?,
    ]);
    let mut edges = (0..profile.stages.len())
        .map(|index| {
            ActivityEdgeDefinition::new(
                ActivityEdgeId::new(100 + raw(index)).expect("route edge id is non-zero"),
                node(HUB),
                battle_node(index),
                ActivityEdgeCondition::OptionSelected,
                i32::try_from(index).expect("five stages"),
                64,
            )
            .map_err(debug_error)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for (index, stage) in profile.stages.iter().enumerate() {
        let from = BATTLE_BASE + raw(index);
        let edge_base = raw(index) * 3 + 1;
        let won_target = if matches!(
            stage.kind,
            AnomalyStageKind::KingNormal | AnomalyStageKind::KingPlight
        ) {
            COMPLETED
        } else {
            HUB
        };
        edges.extend([
            outcome_edge(edge_base, from, won_target, BattleOutcome::Won, 64)?,
            outcome_edge(edge_base + 1, from, HUB, BattleOutcome::Lost, 64)?,
            outcome_edge(edge_base + 2, from, FAULTED, BattleOutcome::Faulted, 1)?,
        ]);
    }
    ActivityGraphDefinition::new(node(HUB), nodes, edges, 512).map_err(debug_error)
}

fn activity_state() -> Result<ActivityStateDefinition, AnomalyArbitrationError> {
    let mut slots = vec![
        optional_slot(SELECTED_STAGE)?,
        optional_slot(SELECTED_QUADRANT)?,
    ];
    for index in 0..5 {
        slots.push(integer_slot(clear_slot(index), 1)?);
        slots.push(integer_slot(star_slot(index), 3)?);
    }
    ActivityStateDefinition::new(slots, vec![], vec![]).map_err(debug_error)
}

fn node_programs(
    profile: &AnomalyProfile,
) -> Result<Vec<GraphActivityNodeProgram>, AnomalyArbitrationError> {
    let mut programs = vec![hub_program(profile)?];
    programs.extend(
        (0..profile.stages.len())
            .map(battle_program)
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(programs)
}

fn hub_program(
    profile: &AnomalyProfile,
) -> Result<GraphActivityNodeProgram, AnomalyArbitrationError> {
    let mut options = Vec::new();
    for (index, stage) in profile.stages.iter().enumerate() {
        match stage.kind {
            AnomalyStageKind::Knight { .. } => {
                options.push(route_definition(index, stage, None, always()));
            }
            AnomalyStageKind::KingNormal => {
                for (quadrant_index, quadrant) in profile.quadrants.iter().enumerate() {
                    options.push(route_definition(
                        index,
                        stage,
                        Some((quadrant_index, quadrant.id)),
                        all_knights_cleared(profile),
                    ));
                }
            }
            AnomalyStageKind::KingPlight => {
                for (quadrant_index, quadrant) in profile.quadrants.iter().enumerate() {
                    options.push(route_definition(
                        index,
                        stage,
                        Some((quadrant_index, quadrant.id)),
                        always(),
                    ));
                }
            }
        }
    }
    ActivityProgramDefinition::new(
        ActivityProgramId::new(HUB).expect("hub program id is non-zero"),
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Route,
            options: options.into_boxed_slice(),
        }],
    )
    .map(|program| GraphActivityNodeProgram::new(node(HUB), program))
    .map_err(debug_error)
}

fn route_definition(
    index: usize,
    stage: &AnomalyStage,
    quadrant: Option<(usize, AnomalyQuadrantId)>,
    enabled: ActivityCondition,
) -> ActivityOptionDefinition {
    ActivityOptionDefinition::new(
        route_option(index, quadrant.map(|item| item.0)),
        i32::try_from(index).expect("five stages"),
        enabled,
        vec![
            ActivityOperation::SetSlot {
                slot: slot(SELECTED_STAGE),
                value: ActivityExpression::Literal(ActivityValue::OptionalId(Some(u64::from(
                    stage.id.get(),
                )))),
            },
            ActivityOperation::SetSlot {
                slot: slot(SELECTED_QUADRANT),
                value: ActivityExpression::Literal(ActivityValue::OptionalId(
                    quadrant.map(|item| u64::from(item.1.get())),
                )),
            },
            ActivityOperation::Relocate(battle_node(index)),
        ],
    )
}

fn battle_program(index: usize) -> Result<GraphActivityNodeProgram, AnomalyArbitrationError> {
    let option = ActivityOptionDefinition::new(encounter_option(index), 0, always(), vec![]);
    ActivityProgramDefinition::new(
        ActivityProgramId::new(BATTLE_BASE + raw(index)).expect("battle program id is non-zero"),
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Encounter,
            options: vec![option].into_boxed_slice(),
        }],
    )
    .map(|program| GraphActivityNodeProgram::new(battle_node(index), program))
    .map_err(debug_error)
}

fn clear_selection_program() -> Result<ActivityProgramDefinition, AnomalyArbitrationError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(900).expect("boundary program id is non-zero"),
        vec![
            ActivityOperation::SetSlot {
                slot: slot(SELECTED_STAGE),
                value: ActivityExpression::Literal(ActivityValue::OptionalId(None)),
            },
            ActivityOperation::SetSlot {
                slot: slot(SELECTED_QUADRANT),
                value: ActivityExpression::Literal(ActivityValue::OptionalId(None)),
            },
        ],
    )
    .map_err(debug_error)
}

fn stage_contract(
    participants: &[ParticipantId],
    projection_id: u32,
    index: usize,
) -> Result<ActivityBattleResultContract, AnomalyArbitrationError> {
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
            key: ANOMALY_WON_KEY.into(),
            kind: MetricValueKind::BoundedInteger,
        },
        ProjectionField::Metric {
            key: ANOMALY_STARS_KEY.into(),
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
                ANOMALY_WON_KEY,
                MetricValueKind::BoundedInteger,
                clear_slot(index),
                MetricSettlementPolicy::Maximum,
            )
            .expect("Anomaly won metric is valid"),
            ActivityMetricProjectionBinding::new(
                ANOMALY_STARS_KEY,
                MetricValueKind::BoundedInteger,
                star_slot(index),
                MetricSettlementPolicy::Maximum,
            )
            .expect("Anomaly stars metric is valid"),
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

fn all_knights_cleared(profile: &AnomalyProfile) -> ActivityCondition {
    ActivityCondition::All(
        profile
            .stages
            .iter()
            .enumerate()
            .filter(|(_, stage)| matches!(stage.kind, AnomalyStageKind::Knight { .. }))
            .map(|(index, _)| {
                ActivityCondition::Equal(
                    ActivityExpression::Slot(clear_slot(index)),
                    ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
                )
            })
            .collect(),
    )
}

fn always() -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true)))
}

fn activity_node(
    raw: u32,
    kind: ActivityNodeKind,
    maximum_visits: u32,
) -> Result<ActivityNodeDefinition, AnomalyArbitrationError> {
    ActivityNodeDefinition::new(node(raw), section(), kind, maximum_visits).map_err(debug_error)
}

fn outcome_edge(
    id: u32,
    from: u32,
    to: u32,
    outcome: BattleOutcome,
    maximum: u32,
) -> Result<ActivityEdgeDefinition, AnomalyArbitrationError> {
    ActivityEdgeDefinition::new(
        ActivityEdgeId::new(id).expect("edge id is non-zero"),
        node(from),
        node(to),
        ActivityEdgeCondition::BattleOutcome(outcome.into()),
        i32::try_from(id).expect("small edge id"),
        maximum,
    )
    .map_err(debug_error)
}

fn optional_slot(raw: u32) -> Result<ActivitySlotDefinition, AnomalyArbitrationError> {
    ActivitySlotDefinition::new_with_policy(
        slot(raw),
        ActivityScope::Activity,
        ActivityValue::OptionalId(None),
        None,
        None,
        vec![],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        ActivityStateSource::new(u64::from(raw)).expect("slot source is non-zero"),
    )
    .map_err(debug_error)
}

fn integer_slot(
    id: ActivitySlotId,
    maximum: i64,
) -> Result<ActivitySlotDefinition, AnomalyArbitrationError> {
    ActivitySlotDefinition::new_with_policy(
        id,
        ActivityScope::Activity,
        ActivityValue::BoundedInteger(0),
        Some((0, maximum)),
        None,
        vec![],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        ActivityStateSource::new(u64::from(id.get())).expect("slot source is non-zero"),
    )
    .map_err(debug_error)
}

fn route_option(index: usize, quadrant_index: Option<usize>) -> ActivityOptionId {
    let suffix = quadrant_index.map_or(0, |value| raw(value) + 1);
    ActivityOptionId::new(1_000 + u64::from(raw(index)) * 10 + u64::from(suffix))
        .expect("route option id is non-zero")
}

fn encounter_option(index: usize) -> ActivityOptionId {
    ActivityOptionId::new(2_000 + u64::from(raw(index))).expect("encounter option id is non-zero")
}

fn preparation_option(index: usize) -> ActivityOptionId {
    ActivityOptionId::new(3_000 + u64::from(raw(index))).expect("preparation option id is non-zero")
}

fn contribution(index: usize, quadrant: Option<AnomalyQuadrantId>) -> TechniqueContributionDigest {
    let mut bytes = [0_u8; 32];
    bytes[0] = u8::try_from(index + 1).expect("five stages");
    if let Some(quadrant) = quadrant {
        bytes[1..5].copy_from_slice(&quadrant.get().to_le_bytes());
    }
    TechniqueContributionDigest::new(bytes).expect("stage contribution is non-zero")
}

fn seed_label(index: usize) -> &'static str {
    match index {
        0 => "anomaly-arbitration/knight-1",
        1 => "anomaly-arbitration/knight-2",
        2 => "anomaly-arbitration/knight-3",
        3 => "anomaly-arbitration/king-normal",
        4 => "anomaly-arbitration/king-plight",
        _ => unreachable!("Anomaly has five stages"),
    }
}

fn section() -> SectionId {
    SectionId::new(SECTION).expect("section id is non-zero")
}

fn node(raw: u32) -> NodeId {
    NodeId::new(raw).expect("node id is non-zero")
}

fn battle_node(index: usize) -> NodeId {
    node(BATTLE_BASE + raw(index))
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("slot id is non-zero")
}

fn clear_slot(index: usize) -> ActivitySlotId {
    slot(CLEAR_BASE + raw(index))
}

fn star_slot(index: usize) -> ActivitySlotId {
    slot(STAR_BASE + raw(index))
}

fn raw(index: usize) -> u32 {
    u32::try_from(index).expect("Anomaly collections are bounded")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnomalyArbitrationError {
    message: Box<str>,
}

impl std::fmt::Display for AnomalyArbitrationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AnomalyArbitrationError {}

fn debug_error(error: impl std::fmt::Debug) -> AnomalyArbitrationError {
    AnomalyArbitrationError {
        message: format!("{error:?}").into_boxed_str(),
    }
}

fn error(message: &str) -> AnomalyArbitrationError {
    AnomalyArbitrationError {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use starclock_activity::{
        ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
        ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, BuildDigest,
        LoadoutLockScope, MetricSettlementPolicy, OpaqueParticipantBuild, ParticipantId,
        ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
        ParticipantUniquenessScope,
    };
    use starclock_combat::{
        ActionValue, CombatantSpecDigest, EncounterId, RuleBundleId, UnitDefinitionId,
    };

    use super::{AnomalyArbitration, AnomalyArbitrationDefinition, stage_contract};
    use crate::{
        AnomalyProfile, AnomalyQuadrant, AnomalyQuadrantId, AnomalyStage, AnomalyStageKind,
        ChallengeProfileId, ChallengeStageId, anomaly_clock,
    };

    #[test]
    fn normal_king_requires_three_knight_clears() {
        let profile = profile();
        let normal = profile.stages[3].id;
        let quadrant = profile.quadrants[0].id;
        let mut runtime = runtime(profile);

        assert!(!runtime.normal_king_available());
        assert!(runtime.choose_stage(normal, Some(quadrant)).is_err());
    }

    #[test]
    fn plight_is_directly_available() {
        let profile = profile();
        let plight = profile.stages[4].id;
        let quadrant = profile.quadrants[0].id;
        let mut runtime = runtime(profile);

        runtime.choose_stage(plight, Some(quadrant)).unwrap();
        assert_eq!(runtime.selected_stage().map(|stage| stage.id), Some(plight));
        assert_eq!(runtime.selected_quadrant(), Some(quadrant));
        assert_eq!(runtime.king_protection_contributions(), 3);
    }

    #[test]
    fn protection_count_tracks_stage_kind() {
        assert_eq!(
            AnomalyStage::protection_contributions(AnomalyStageKind::Knight { slot: 0 }),
            0
        );
        assert_eq!(
            AnomalyStage::protection_contributions(AnomalyStageKind::KingNormal),
            0
        );
        assert_eq!(
            AnomalyStage::protection_contributions(AnomalyStageKind::KingPlight),
            3
        );
    }

    #[test]
    fn definition_rejects_duplicate_knight_characters() {
        let error = AnomalyArbitrationDefinition::new(identity(), profile(), participants(true))
            .unwrap_err();
        assert!(error.to_string().contains("disjoint character identities"));
    }

    #[test]
    fn failed_retry_preserves_best_record() {
        let participants = [ParticipantId::new(1).unwrap()];
        let contract = stage_contract(&participants, 1, 0).unwrap();

        assert!(
            contract
                .metrics()
                .iter()
                .all(|binding| binding.policy() == MetricSettlementPolicy::Maximum)
        );
    }

    fn runtime(profile: Arc<AnomalyProfile>) -> AnomalyArbitration {
        let definition = Arc::new(
            AnomalyArbitrationDefinition::new(identity(), profile, participants(false)).unwrap(),
        );
        AnomalyArbitration::start(
            definition,
            ActivityInstanceId::new(1).unwrap(),
            ActivityMasterSeed::from_u64(7),
        )
        .unwrap()
    }

    fn profile() -> Arc<AnomalyProfile> {
        let clock = anomaly_clock(
            6,
            ActionValue::from_scaled(150_000_000).unwrap(),
            ActionValue::from_scaled(100_000_000).unwrap(),
        )
        .unwrap();
        let kinds = [
            AnomalyStageKind::Knight { slot: 0 },
            AnomalyStageKind::Knight { slot: 1 },
            AnomalyStageKind::Knight { slot: 2 },
            AnomalyStageKind::KingNormal,
            AnomalyStageKind::KingPlight,
        ];
        let stages = kinds
            .into_iter()
            .enumerate()
            .map(|(index, kind)| AnomalyStage {
                id: ChallengeStageId::new(u32::try_from(index + 1).unwrap()).unwrap(),
                kind,
                encounter: EncounterId::new(u32::try_from(index + 1).unwrap()).unwrap(),
                team_index: match kind {
                    AnomalyStageKind::Knight { slot } => slot,
                    AnomalyStageKind::KingNormal | AnomalyStageKind::KingPlight => 3,
                },
                clock,
                targets: Box::new([]),
            })
            .collect();
        let quadrants = (1..=3)
            .map(|raw| AnomalyQuadrant {
                id: AnomalyQuadrantId::new(raw).unwrap(),
                upstream_buff_id: raw,
                rule_bundle: RuleBundleId::new(raw).unwrap(),
                behavior_exact: false,
            })
            .collect();
        Arc::new(
            AnomalyProfile::new(
                ChallengeProfileId::new(1).unwrap(),
                stages,
                quadrants,
                vec![],
            )
            .unwrap(),
        )
    }

    fn participants(duplicate_knight: bool) -> ParticipantLock {
        let policy = ParticipantPolicy::new(
            4,
            1,
            4,
            ParticipantUniquenessScope::Team,
            LoadoutLockScope::Activity,
        )
        .unwrap();
        let entries = (0..4)
            .map(|team| {
                let character = if duplicate_knight && team == 1 {
                    1
                } else {
                    team + 1
                };
                ParticipantLockEntry::new(
                    ParticipantId::new(team + 1).unwrap(),
                    u8::try_from(team).unwrap(),
                    0,
                    UnitDefinitionId::new(character).unwrap(),
                    OpaqueParticipantBuild::new(
                        CombatantSpecDigest::new([u8::try_from(team + 1).unwrap(); 32]).unwrap(),
                        BuildDigest::new([u8::try_from(team + 5).unwrap(); 32]).unwrap(),
                        ParticipantSourceKind::FixedResolved,
                    )
                    .unwrap(),
                )
                .unwrap()
            })
            .collect();
        ParticipantLock::seal(policy, entries).unwrap()
    }

    fn identity() -> ActivityDefinitionIdentity {
        ActivityDefinitionIdentity::new(
            ActivityDefinitionId::new(1).unwrap(),
            ActivityDefinitionDigest::new([1; 32]).unwrap(),
            ActivityConfigDigest::new([2; 32]).unwrap(),
        )
    }
}
