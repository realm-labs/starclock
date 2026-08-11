use std::sync::Arc;

use starclock_activity::{
    ActivityBattleHandoff, ActivityBattlePreparationRequest, ActivityBattleResultContract,
    ActivityCondition, ActivityDecisionKind, ActivityDefinitionIdentity, ActivityExpression,
    ActivityGraphDefinition, ActivityInstanceId, ActivityMasterSeed,
    ActivityMetricProjectionBinding, ActivityOperation, ActivityOptionDefinition, ActivityOptionId,
    ActivityParticipantCarryDefinition, ActivityProgramDefinition, ActivityProgramId,
    ActivityRandomOffer, ActivityRandomPolicies, ActivityRngLabel, ActivityRosterLock,
    ActivityScope, ActivityScopePath, ActivitySlotDefinition, ActivitySlotId,
    ActivityStateDefinition, ActivityStateHash, ActivityStateSource, ActivityStateVisibility,
    ActivityValue, AttemptId, BattleBinding, BattleResult, BattleResultProjection, BattleSequence,
    EncounterInitiativePolicy, EncounterPreparationDefinition, EnergyCarryPolicy, GraphActivity,
    GraphActivityBattleResolution, GraphActivityDefinition, GraphActivityNodeProgram,
    GraphActivityPreparationResolution, HpCarryPolicy, LifeCarryPolicy, MetricSettlementPolicy,
    MetricValueKind, NodeId, ParticipantId, ParticipantLock, PresenceCarryPolicy, ProjectionField,
    ProjectionId, SectionId, SlotCarryPolicy, TechniqueContributionDigest,
};
use starclock_combat::BattleSpec;

use crate::{
    BaseballerCatalog, BaseballerInventoryBindings, BaseballerPeriodRank, BaseballerProfile,
    BaseballerProgressionSnapshot, BaseballerScoreRule, BaseballerSettlement, BaseballerStage,
    BaseballerStageFlow, BaseballerStageId, BaseballerStagePeriod,
};

pub const BASEBALLER_SCORE_KEY: &str = "baseballer_score";

const SECTION: u32 = 1;
const SELECTED_PERIOD: u32 = 1;
const EQUIPMENT_LEVELS: u32 = 2;
const USED_WEAPONS: u32 = 3;
const UNLOCKED_WEAPONS: u32 = 4;
const USED_ACCESSORIES: u32 = 5;
const UNLOCKED_ACCESSORIES: u32 = 6;
const SCORE: u32 = 7;

#[derive(Debug)]
pub struct BaseballerRunDefinition {
    catalog: Arc<BaseballerCatalog>,
    stage: BaseballerStageId,
    score_rule: BaseballerScoreRule,
    activity: Arc<GraphActivityDefinition>,
    participants: Arc<ParticipantLock>,
    contracts: Box<[Arc<ActivityBattleResultContract>]>,
}

impl BaseballerRunDefinition {
    pub fn new(
        identity: ActivityDefinitionIdentity,
        catalog: Arc<BaseballerCatalog>,
        stage: BaseballerStageId,
        score_rule: BaseballerScoreRule,
        participants: ParticipantLock,
    ) -> Result<Self, BaseballerRuntimeError> {
        Self::new_with_progression(identity, catalog, stage, score_rule, participants, None)
    }

    pub fn new_with_progression(
        identity: ActivityDefinitionIdentity,
        catalog: Arc<BaseballerCatalog>,
        stage: BaseballerStageId,
        score_rule: BaseballerScoreRule,
        participants: ParticipantLock,
        progression: Option<&BaseballerProgressionSnapshot>,
    ) -> Result<Self, BaseballerRuntimeError> {
        let stage_definition = catalog
            .stages()
            .iter()
            .find(|candidate| candidate.id == stage)
            .ok_or_else(|| error("Baseballer stage is not in the catalog"))?;
        let profile = catalog
            .profiles()
            .iter()
            .find(|profile| profile.id == stage_definition.profile)
            .ok_or_else(|| error("Baseballer stage profile is missing"))?;
        if progression.is_some_and(|snapshot| !snapshot.validate_for_catalog(&catalog)) {
            return Err(error(
                "Baseballer progression snapshot does not match the catalog",
            ));
        }
        let periods = catalog
            .periods_for_stage(stage)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        validate_participants(&participants)?;
        if stage_definition.rating_thresholds.as_ref() != score_rule.rating_thresholds.as_ref() {
            return Err(error(
                "Baseballer score thresholds do not match the selected stage",
            ));
        }
        let graph = BaseballerStageFlow::compile(section(), &periods).map_err(debug_error)?;
        let inventory = inventory_bindings();
        let state = activity_state(&catalog, profile, stage_definition, inventory, progression)?;
        let (programs, random_policies) =
            node_programs(&catalog, profile, &periods, &graph, inventory)?;
        let participant_ids = participant_ids(&participants);
        let rank_count = ordered_ranks(&periods).len();
        let contracts = (0..rank_count)
            .map(|index| stage_contract(&participant_ids, index).map(Arc::new))
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
                random_policies,
            )
            .map_err(debug_error)?,
        );
        Ok(Self {
            catalog,
            stage,
            score_rule,
            activity,
            participants,
            contracts: contracts.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn stage(&self) -> &BaseballerStage {
        self.catalog
            .stages()
            .iter()
            .find(|stage| stage.id == self.stage)
            .expect("definition stage was validated")
    }
}

#[derive(Debug)]
pub struct BaseballerRun {
    definition: Arc<BaseballerRunDefinition>,
    activity: GraphActivity,
}

impl BaseballerRun {
    pub fn start(
        definition: Arc<BaseballerRunDefinition>,
        instance: ActivityInstanceId,
        master_seed: ActivityMasterSeed,
    ) -> Result<Self, BaseballerRuntimeError> {
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
    pub fn equipment_level(&self, equipment: crate::BaseballerEquipmentId) -> u8 {
        self.activity
            .player_view()
            .slots()
            .iter()
            .find(|entry| entry.id() == slot(EQUIPMENT_LEVELS))
            .and_then(|entry| match entry.value() {
                ActivityValue::BoundedCounterMap(levels) => levels
                    .binary_search_by_key(&u64::from(equipment.get()), |item| item.0)
                    .ok()
                    .and_then(|index| u8::try_from(levels[index].1).ok()),
                _ => None,
            })
            .unwrap_or_default()
    }

    #[must_use]
    pub fn raw_score(&self) -> i64 {
        integer_slot(&self.activity, slot(SCORE))
    }

    #[must_use]
    pub fn settlement(&self, final_stage: bool) -> BaseballerSettlement {
        self.definition
            .score_rule
            .settle(self.raw_score(), final_stage)
    }

    pub fn engage_offered_period(
        &mut self,
        attempt: AttemptId,
        battle: BattleSpec,
    ) -> Result<GraphActivityPreparationResolution, BaseballerRuntimeError> {
        let rank_index = current_rank_index(self.activity.current_node())?;
        let view = self.activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| decision.kind() == ActivityDecisionKind::Encounter)
            .ok_or_else(|| error("Baseballer period encounter is not currently offered"))?;
        if decision.options().len() != 1 {
            return Err(error(
                "Baseballer period offer was not reduced to one candidate",
            ));
        }
        let option = decision.options()[0].id();
        let period = self
            .period_for_option(option)
            .ok_or_else(|| error("offered Baseballer period is unknown"))?;
        if battle.encounter() != period.encounter {
            return Err(error(
                "battle encounter does not match the offered Baseballer period",
            ));
        }
        let lock = self.definition.participants.digest();
        let preparation = Arc::new(
            EncounterPreparationDefinition::new(
                preparation_option(rank_index),
                EncounterInitiativePolicy::PlayerControlled,
                lock,
                0,
                vec![],
                vec![starclock_activity::PreparedBattleVariant::new(
                    vec![],
                    contribution(period),
                    BattleBinding::new(battle, seed_label(rank_index), lock)
                        .map_err(debug_error)?,
                )],
            )
            .map_err(debug_error)?,
        );
        let path = ActivityScopePath::new(self.activity.instance())
            .enter_section(section())
            .and_then(|path| path.enter_node(self.activity.current_node()))
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
                option,
                ActivityBattlePreparationRequest::new(
                    path,
                    roster,
                    BattleSequence::new(u32::try_from(rank_index + 1).map_err(debug_error)?)
                        .ok_or_else(|| error("Baseballer battle sequence is zero"))?,
                    0,
                    preparation,
                ),
            )
            .map_err(debug_error)
    }

    pub fn choose_prepared_battle(&mut self) -> Result<(), BaseballerRuntimeError> {
        let rank = current_rank_index(self.activity.current_node())?;
        self.activity
            .choose_preparation_option(self.state_hash(), preparation_option(rank))
            .map_err(debug_error)?;
        Ok(())
    }

    pub fn start_pending_battle(
        &mut self,
    ) -> Result<ActivityBattleHandoff, BaseballerRuntimeError> {
        let rank = current_rank_index(self.activity.current_node())?;
        self.activity
            .start_pending_battle(
                self.state_hash(),
                Arc::clone(&self.definition.contracts[rank]),
            )
            .map_err(debug_error)
    }

    pub fn submit_battle_result(
        &mut self,
        result: BattleResult,
    ) -> Result<GraphActivityBattleResolution, BaseballerRuntimeError> {
        let clear = ActivityProgramDefinition::new(
            ActivityProgramId::new(900).expect("boundary program id is non-zero"),
            vec![ActivityOperation::SetSlot {
                slot: slot(SELECTED_PERIOD),
                value: ActivityExpression::Literal(ActivityValue::OptionalId(None)),
            }],
        )
        .map_err(debug_error)?;
        self.activity
            .submit_pending_battle_result_with_boundary_program(
                self.state_hash(),
                result,
                Some(&clear),
            )
            .map_err(debug_error)
    }

    pub fn choose_equipment(
        &mut self,
        option: ActivityOptionId,
    ) -> Result<(), BaseballerRuntimeError> {
        let view = self.activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| decision.kind() == ActivityDecisionKind::Reward)
            .ok_or_else(|| error("Baseballer equipment reward is not currently offered"))?;
        self.activity
            .choose_option(view.state_hash(), decision.id(), option)
            .map_err(debug_error)?;
        Ok(())
    }

    fn period_for_option(&self, option: ActivityOptionId) -> Option<&BaseballerStagePeriod> {
        self.definition
            .catalog
            .periods_for_stage(self.definition.stage)
            .into_iter()
            .find(|period| period_option(period).ok() == Some(option))
    }
}

fn activity_state(
    catalog: &BaseballerCatalog,
    profile: &BaseballerProfile,
    stage: &BaseballerStage,
    inventory: BaseballerInventoryBindings,
    progression: Option<&BaseballerProgressionSnapshot>,
) -> Result<ActivityStateDefinition, BaseballerRuntimeError> {
    let mut slots = vec![
        optional_slot(SELECTED_PERIOD)?,
        integer(SCORE, 0, i64::MAX)?,
    ];
    slots.extend(
        inventory
            .definitions_with_progression(catalog, profile, stage, progression)
            .map_err(debug_error)?,
    );
    ActivityStateDefinition::new(slots, vec![], vec![]).map_err(debug_error)
}

fn node_programs(
    catalog: &BaseballerCatalog,
    profile: &BaseballerProfile,
    periods: &[BaseballerStagePeriod],
    graph: &ActivityGraphDefinition,
    inventory: BaseballerInventoryBindings,
) -> Result<(Vec<GraphActivityNodeProgram>, ActivityRandomPolicies), BaseballerRuntimeError> {
    let ranks = ordered_ranks(periods);
    let mut programs = Vec::new();
    let mut offers = Vec::new();
    for (index, rank) in ranks.iter().enumerate() {
        let node = battle_node(index)?;
        let candidates = periods
            .iter()
            .filter(|period| period.rank == *rank)
            .collect::<Vec<_>>();
        let options = candidates
            .iter()
            .map(|period| {
                Ok(ActivityOptionDefinition::new(
                    period_option(period)?,
                    i32::try_from(period.id.get()).map_err(debug_error)?,
                    always(),
                    vec![ActivityOperation::SetSlot {
                        slot: slot(SELECTED_PERIOD),
                        value: ActivityExpression::Literal(ActivityValue::OptionalId(Some(
                            u64::from(period.id.get()),
                        ))),
                    }],
                ))
            })
            .collect::<Result<Vec<_>, BaseballerRuntimeError>>()?;
        programs.push(program(node, ActivityDecisionKind::Encounter, options)?);
        offers.push(
            ActivityRandomOffer::new(
                node,
                ActivityRngLabel::Encounter,
                u16::try_from(index + 1).map_err(debug_error)?,
                1,
                candidates
                    .iter()
                    .map(|period| Ok((period_option(period)?, u64::from(period.selection_weight))))
                    .collect::<Result<Vec<_>, BaseballerRuntimeError>>()?,
                None,
            )
            .map_err(debug_error)?,
        );
        if index + 1 < ranks.len() {
            let reward = reward_node(index)?;
            let route = graph
                .edges()
                .iter()
                .copied()
                .find(|edge| edge.from() == reward)
                .map(|edge| edge.id())
                .ok_or_else(|| error("Baseballer reward route is missing"))?;
            let inventory_options = inventory
                .equipment_options(catalog, profile, route)
                .map_err(debug_error)?;
            programs.push(program(
                reward,
                ActivityDecisionKind::Reward,
                inventory_options.options,
            )?);
            offers.push(
                ActivityRandomOffer::new(
                    reward,
                    ActivityRngLabel::Reward,
                    u16::try_from(index + 1).map_err(debug_error)?,
                    3,
                    inventory_options.weights,
                    None,
                )
                .map_err(debug_error)?,
            );
        }
    }
    Ok((programs, ActivityRandomPolicies::new(vec![], offers)))
}

fn program(
    node: NodeId,
    kind: ActivityDecisionKind,
    options: Vec<ActivityOptionDefinition>,
) -> Result<GraphActivityNodeProgram, BaseballerRuntimeError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(node.get()).expect("node program id is non-zero"),
        vec![ActivityOperation::Offer {
            kind,
            options: options.into_boxed_slice(),
        }],
    )
    .map(|program| GraphActivityNodeProgram::new(node, program))
    .map_err(debug_error)
}

fn stage_contract(
    participants: &[ParticipantId],
    rank: usize,
) -> Result<ActivityBattleResultContract, BaseballerRuntimeError> {
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
    fields.push(ProjectionField::Metric {
        key: BASEBALLER_SCORE_KEY.into(),
        kind: MetricValueKind::BoundedInteger,
    });
    let projection = Arc::new(
        BattleResultProjection::new(
            ProjectionId::new(u32::try_from(rank + 1).map_err(debug_error)?)
                .ok_or_else(|| error("Baseballer projection id is zero"))?,
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
                BASEBALLER_SCORE_KEY,
                MetricValueKind::BoundedInteger,
                slot(SCORE),
                MetricSettlementPolicy::Sum,
            )
            .ok_or_else(|| error("Baseballer score metric binding is invalid"))?,
        ],
    )
    .map_err(debug_error)
}

fn validate_participants(participants: &ParticipantLock) -> Result<(), BaseballerRuntimeError> {
    if participants.policy().team_count() != 1 || participant_ids(participants).is_empty() {
        return Err(error("Baseballer runtime requires one non-empty team"));
    }
    Ok(())
}

fn participant_ids(participants: &ParticipantLock) -> Vec<ParticipantId> {
    participants
        .entries()
        .iter()
        .filter(|entry| entry.team_index() == 0)
        .map(|entry| entry.participant())
        .collect()
}

fn ordered_ranks(periods: &[BaseballerStagePeriod]) -> Vec<BaseballerPeriodRank> {
    let mut ranks = periods.iter().map(|period| period.rank).collect::<Vec<_>>();
    ranks.sort_unstable();
    ranks.dedup();
    ranks
}

fn current_rank_index(node: NodeId) -> Result<usize, BaseballerRuntimeError> {
    let raw = node.get();
    if raw.is_multiple_of(2) {
        return Err(error("Baseballer runtime is not at a battle period"));
    }
    usize::try_from((raw - 1) / 2).map_err(debug_error)
}

fn battle_node(index: usize) -> Result<NodeId, BaseballerRuntimeError> {
    u32::try_from(index)
        .ok()
        .and_then(|value| value.checked_mul(2))
        .and_then(|value| value.checked_add(1))
        .and_then(NodeId::new)
        .ok_or_else(|| error("Baseballer battle node overflow"))
}

fn reward_node(index: usize) -> Result<NodeId, BaseballerRuntimeError> {
    u32::try_from(index)
        .ok()
        .and_then(|value| value.checked_mul(2))
        .and_then(|value| value.checked_add(2))
        .and_then(NodeId::new)
        .ok_or_else(|| error("Baseballer reward node overflow"))
}

fn inventory_bindings() -> BaseballerInventoryBindings {
    BaseballerInventoryBindings {
        levels: slot(EQUIPMENT_LEVELS),
        used_weapon_slots: slot(USED_WEAPONS),
        unlocked_weapon_slots: slot(UNLOCKED_WEAPONS),
        used_accessory_slots: slot(USED_ACCESSORIES),
        unlocked_accessory_slots: slot(UNLOCKED_ACCESSORIES),
    }
}

fn optional_slot(raw: u32) -> Result<ActivitySlotDefinition, BaseballerRuntimeError> {
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

fn integer(
    raw: u32,
    initial: i64,
    maximum: i64,
) -> Result<ActivitySlotDefinition, BaseballerRuntimeError> {
    ActivitySlotDefinition::new_with_policy(
        slot(raw),
        ActivityScope::Activity,
        ActivityValue::BoundedInteger(initial),
        Some((0, maximum)),
        None,
        vec![],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        ActivityStateSource::new(u64::from(raw)).expect("slot source is non-zero"),
    )
    .map_err(debug_error)
}

fn integer_slot(activity: &GraphActivity, id: ActivitySlotId) -> i64 {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|entry| entry.id() == id)
        .and_then(|entry| match entry.value() {
            ActivityValue::BoundedInteger(value) => Some(*value),
            _ => None,
        })
        .unwrap_or_default()
}

fn period_option(
    period: &BaseballerStagePeriod,
) -> Result<ActivityOptionId, BaseballerRuntimeError> {
    1_000_000_000_u64
        .checked_add(u64::from(period.id.get()))
        .and_then(ActivityOptionId::new)
        .ok_or_else(|| error("Baseballer period option overflow"))
}

fn preparation_option(rank: usize) -> ActivityOptionId {
    ActivityOptionId::new(2_000_000_000 + u64::try_from(rank).expect("rank fits u64"))
        .expect("preparation option id is non-zero")
}

fn contribution(period: &BaseballerStagePeriod) -> TechniqueContributionDigest {
    let mut bytes = [0_u8; 32];
    bytes[..4].copy_from_slice(&period.id.get().to_le_bytes());
    TechniqueContributionDigest::new(bytes).expect("period contribution is non-zero")
}

fn seed_label(rank: usize) -> &'static str {
    match rank {
        0 => "galactic-baseballer/period-1",
        1 => "galactic-baseballer/period-2",
        2 => "galactic-baseballer/period-3",
        3 => "galactic-baseballer/period-extra",
        _ => unreachable!("Baseballer has at most four period ranks"),
    }
}

fn always() -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true)))
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("slot id is non-zero")
}

fn section() -> SectionId {
    SectionId::new(SECTION).expect("section id is non-zero")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerRuntimeError {
    message: Box<str>,
}

impl std::fmt::Display for BaseballerRuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for BaseballerRuntimeError {}

fn error(message: &str) -> BaseballerRuntimeError {
    BaseballerRuntimeError {
        message: message.into(),
    }
}

fn debug_error(error: impl std::fmt::Debug) -> BaseballerRuntimeError {
    BaseballerRuntimeError {
        message: format!("{error:?}").into_boxed_str(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use starclock_activity::{
        ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest, ActivityDefinitionId,
        ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityRngLabel,
        BuildDigest, LoadoutLockScope, OpaqueParticipantBuild, ParticipantId, ParticipantLock,
        ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
    };
    use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};

    use super::{BaseballerRun, BaseballerRunDefinition};
    use crate::{BaseballerScoreRule, catalog::tests_support};

    #[test]
    fn run_starts_with_one_deterministically_selected_period() {
        let run = BaseballerRun::start(
            definition(),
            ActivityInstanceId::new(1).unwrap(),
            ActivityMasterSeed::from_u64(7),
        )
        .unwrap();
        let view = run.player_view();
        let decision = view.decision().unwrap();

        assert_eq!(decision.kind(), ActivityDecisionKind::Encounter);
        assert_eq!(decision.options().len(), 1);
    }

    #[test]
    fn reward_offer_uses_bounded_uniform_stable_options() {
        let definition = definition();
        let reward = definition
            .activity
            .random_offers()
            .iter()
            .find(|offer| offer.label() == ActivityRngLabel::Reward)
            .unwrap();

        assert_eq!(reward.maximum_options(), 3);
        assert!(reward.weights().iter().all(|(_, weight)| *weight == 1));
        assert!(
            reward
                .weights()
                .windows(2)
                .all(|pair| pair[0].0 < pair[1].0)
        );
    }

    fn definition() -> Arc<BaseballerRunDefinition> {
        Arc::new(
            BaseballerRunDefinition::new(
                identity(),
                Arc::new(tests_support::catalog()),
                tests_support::stage_id(),
                BaseballerScoreRule::new(
                    1,
                    vec![1, 1, 1, 1],
                    vec![1, 1, 1, 1, 1],
                    100,
                    0,
                    vec![0, 1, 2, 3, 4],
                )
                .unwrap(),
                participants(),
            )
            .unwrap(),
        )
    }

    fn identity() -> ActivityDefinitionIdentity {
        ActivityDefinitionIdentity::new(
            ActivityDefinitionId::new(1).unwrap(),
            ActivityDefinitionDigest::new([1; 32]).unwrap(),
            ActivityConfigDigest::new([2; 32]).unwrap(),
        )
    }

    fn participants() -> ParticipantLock {
        let policy = ParticipantPolicy::new(
            1,
            1,
            4,
            ParticipantUniquenessScope::Team,
            LoadoutLockScope::Activity,
        )
        .unwrap();
        let entry = ParticipantLockEntry::new(
            ParticipantId::new(1).unwrap(),
            0,
            0,
            UnitDefinitionId::new(1).unwrap(),
            OpaqueParticipantBuild::new(
                CombatantSpecDigest::new([3; 32]).unwrap(),
                BuildDigest::new([4; 32]).unwrap(),
                ParticipantSourceKind::FixedResolved,
            )
            .unwrap(),
        )
        .unwrap();
        ParticipantLock::seal(policy, vec![entry]).unwrap()
    }
}
