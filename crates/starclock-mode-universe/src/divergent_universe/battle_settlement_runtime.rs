//! Atomic nested-battle start, execution and Activity settlement.

use std::sync::Arc;

use starclock_activity::{
    ActivityBattleHandoff, ActivityBattlePreparationRequest, ActivityBattleResultContract,
    ActivityDecisionKind, ActivityOptionId, ActivityParticipantCarryDefinition,
    ActivityPreparationBoundary, ActivityProgramId, ActivityRosterLock, ActivityScopePath,
    ActivityStateHash, AttemptId, BattleBinding, BattleOutcome, BattleResult, BattleSequence,
    EncounterInitiativePolicy, EncounterPreparationDefinition, EnergyCarryPolicy, GraphActivity,
    GraphActivityBattleResolution, GraphActivityCommandError, HpCarryPolicy, LifeCarryPolicy,
    PreparedBattleVariant, PresenceCarryPolicy, ProjectionField, ProjectionId,
};

use crate::{
    baseline_runner::{NestedBattleExecutionError, NestedBattleExecutor},
    nested_battle_executor::{NestedBattleExecutionReport, UniverseNestedBattleExecutor},
};

use super::{
    DivergentUniverseAssembledBattle, DivergentUniverseFlowInstance,
    DivergentUniverseRuntimeFactory,
};

const PREPARATION_OPTION: u64 = 0x7e25_0001;
const RESULT_PROJECTION: u32 = 0x7e25_0002;
/// Binds generator order and fresh-view semantics into the mode configuration.
pub(super) const SETTLEMENT_POLICY_IDENTITY: &[u8] =
    b"verified-carry;base-domain-fragments/24060;fresh-view;curio-victory-grants/23700;fresh-view;domain-curio-blessings/24050;fresh-view;curio-battle-lifetimes/24110;fresh-view;normal-blessing-offer/23702;atomic-pump";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseBattleRewardDisposition {
    BlessingOfferGeneratedOtherRewardsPending,
    NoBlessingOfferOtherRewardsPending,
    NoVictoryRewards,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseBattleRetryDisposition {
    DefeatTerminatesCurrentActivityFreshActivityRequired,
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseBattleStart {
    handoff: ActivityBattleHandoff,
    combat_catalog: Arc<starclock_combat::catalog::CombatCatalog>,
}

impl DivergentUniverseBattleStart {
    #[must_use]
    pub const fn handoff(&self) -> &ActivityBattleHandoff {
        &self.handoff
    }

    pub fn execute(
        &self,
    ) -> Result<(BattleResult, NestedBattleExecutionReport), DivergentUniverseBattleSettlementError>
    {
        let mut executor = UniverseNestedBattleExecutor::new(Arc::clone(&self.combat_catalog));
        let result = executor
            .execute(&self.handoff)
            .map_err(DivergentUniverseBattleSettlementError::Execution)?;
        let report = executor
            .last_report()
            .cloned()
            .ok_or(DivergentUniverseBattleSettlementError::MissingExecutionReport)?;
        Ok((result, report))
    }
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseBattleSettlementResolution {
    result: BattleResult,
    report: Option<NestedBattleExecutionReport>,
    settlement: starclock_activity::ActivityBattleSettlement,
    events: Box<[starclock_activity::ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
    reward: DivergentUniverseBattleRewardDisposition,
    retry: DivergentUniverseBattleRetryDisposition,
}

impl DivergentUniverseBattleSettlementResolution {
    #[must_use]
    pub const fn result(&self) -> &BattleResult {
        &self.result
    }
    #[must_use]
    pub const fn report(&self) -> Option<&NestedBattleExecutionReport> {
        self.report.as_ref()
    }
    #[must_use]
    pub const fn settlement(&self) -> starclock_activity::ActivityBattleSettlement {
        self.settlement
    }
    #[must_use]
    pub fn events(&self) -> &[starclock_activity::ActivityTransactionEvent] {
        &self.events
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
    #[must_use]
    pub const fn reward_disposition(&self) -> DivergentUniverseBattleRewardDisposition {
        self.reward
    }
    #[must_use]
    pub const fn retry_disposition(&self) -> DivergentUniverseBattleRetryDisposition {
        self.retry
    }
}

#[derive(Clone, Debug, Default)]
pub struct DivergentUniverseBattleSettlementRuntime;

impl DivergentUniverseRuntimeFactory {
    #[must_use]
    pub const fn battle_settlement_runtime(&self) -> DivergentUniverseBattleSettlementRuntime {
        DivergentUniverseBattleSettlementRuntime
    }
}

impl DivergentUniverseBattleSettlementRuntime {
    pub fn start_current_battle(
        &self,
        flow: &DivergentUniverseFlowInstance,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        attempt: AttemptId,
        sequence: BattleSequence,
        materialization: &DivergentUniverseAssembledBattle,
    ) -> Result<DivergentUniverseBattleStart, DivergentUniverseBattleSettlementError> {
        validate_start(flow, activity, expected, materialization)?;
        let view = activity.player_view();
        let decision = view
            .decision()
            .filter(|decision| {
                decision.kind() == starclock_activity::ActivityDecisionKind::Encounter
                    && decision.options().len() == 1
            })
            .ok_or(DivergentUniverseBattleSettlementError::EncounterNotOffered)?;
        let decision_id = decision.id();
        let engagement = decision.options()[0].id();
        let (battle_node, section) = flow
            .encounter_destination(activity.current_node())
            .ok_or(DivergentUniverseBattleSettlementError::InvalidScope)?;
        let path = ActivityScopePath::new(activity.instance())
            .enter_section(section)
            .and_then(|path| path.enter_node(battle_node))
            .and_then(|path| path.enter_attempt(attempt))
            .map_err(|_| DivergentUniverseBattleSettlementError::InvalidScope)?;
        let participants = flow.definition().participants().as_ref().clone();
        let roster = ActivityRosterLock::new(
            ActivityScopePath::new(activity.instance()),
            participants.clone(),
        )
        .map_err(|_| DivergentUniverseBattleSettlementError::InvalidScope)?;
        let binding = BattleBinding::new(
            materialization.battle_spec().clone(),
            "divergent-universe-current-battle",
            participants.digest(),
        )
        .map_err(|_| DivergentUniverseBattleSettlementError::InvalidBattleBinding)?;
        let normal = ActivityOptionId::new(PREPARATION_OPTION)
            .expect("reserved preparation option is non-zero");
        let preparation = EncounterPreparationDefinition::new(
            normal,
            EncounterInitiativePolicy::PlayerControlled,
            participants.digest(),
            0,
            Vec::new(),
            vec![PreparedBattleVariant::new(
                Vec::new(),
                materialization.contribution_digest(),
                binding,
            )],
        )
        .map(Arc::new)
        .map_err(|_| DivergentUniverseBattleSettlementError::InvalidBattleBinding)?;
        let request = ActivityBattlePreparationRequest::new(path, roster, sequence, 0, preparation);
        let resolution = activity
            .engage_encounter(expected, decision_id, engagement, request)
            .map_err(|_| DivergentUniverseBattleSettlementError::PreparationRejected)?;
        if resolution.boundary() != ActivityPreparationBoundary::Decision
            || activity
                .choose_preparation_option(resolution.state_hash(), normal)
                .map_err(|_| DivergentUniverseBattleSettlementError::PreparationRejected)?
                != ActivityPreparationBoundary::BattleReady
        {
            return Err(DivergentUniverseBattleSettlementError::PreparationRejected);
        }
        let handoff = activity
            .start_pending_battle(activity.state_hash(), settlement_contract(&participants)?)
            .map_err(|_| DivergentUniverseBattleSettlementError::BattleStartRejected)?;
        Ok(DivergentUniverseBattleStart {
            handoff,
            combat_catalog: Arc::clone(materialization.combat_catalog()),
        })
    }

    pub fn execute_current_battle(
        &self,
        flow: &DivergentUniverseFlowInstance,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        attempt: AttemptId,
        sequence: BattleSequence,
        materialization: &DivergentUniverseAssembledBattle,
    ) -> Result<DivergentUniverseBattleSettlementResolution, DivergentUniverseBattleSettlementError>
    {
        let start = self.start_current_battle(
            flow,
            activity,
            expected,
            attempt,
            sequence,
            materialization,
        )?;
        let (result, report) = match start.execute() {
            Ok(value) => value,
            Err(error) => {
                let restored = activity.rollback_pending_battle_start();
                debug_assert!(restored, "this runtime started the exact pending battle");
                return Err(error);
            }
        };
        self.settle_started_result(flow, activity, activity.state_hash(), result, Some(report))
    }

    /// Settles the exact pending result against its bound flow. Verification,
    /// carry, base fragments, reviewed Curio victory grants, Blessing sampling and the next offer share one
    /// rollback boundary. Loss/fault does not sample victory rewards; other
    /// battle reward programs remain explicitly pending. A mismatched flow or
    /// rejected result publishes no partial state, events or RNG changes.
    pub fn settle_started_result(
        &self,
        flow: &DivergentUniverseFlowInstance,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        result: BattleResult,
        report: Option<NestedBattleExecutionReport>,
    ) -> Result<DivergentUniverseBattleSettlementResolution, DivergentUniverseBattleSettlementError>
    {
        if activity.definition().identity() != flow.definition().identity()
            || activity.definition().graph().digest() != flow.definition().graph().digest()
            || flow
                .position_battles
                .as_ref()
                .is_some_and(|rooms| !rooms.matches(activity))
        {
            return Err(DivergentUniverseBattleSettlementError::DefinitionMismatch);
        }
        let rewards = flow
            .battle_blessings
            .as_ref()
            .ok_or(DivergentUniverseBattleSettlementError::DefinitionMismatch)?;
        let retained_result = result.clone();
        let fragments = flow
            .battle_fragments
            .as_ref()
            .ok_or(DivergentUniverseBattleSettlementError::DefinitionMismatch)?;
        let base_fragments = ActivityProgramId::new(24_060).expect("reserved base drop program");
        let grants = flow
            .curio_battle_grants
            .as_ref()
            .ok_or(DivergentUniverseBattleSettlementError::DefinitionMismatch)?;
        let curio_grants = ActivityProgramId::new(23_700).expect("reserved Curio reward program");
        let victory_blessings = flow
            .curio_victory_blessings
            .as_ref()
            .ok_or(DivergentUniverseBattleSettlementError::DefinitionMismatch)?;
        let curio_blessings =
            ActivityProgramId::new(24_050).expect("reserved victory Blessing program");
        let blessing_offer =
            ActivityProgramId::new(23_702).expect("reserved Blessing offer program");
        let lifetimes =
            ActivityProgramId::new(24_110).expect("reserved Curio battle-lifetime program");
        let battle_stats = flow
            .curio_battle_stats
            .as_ref()
            .ok_or(DivergentUniverseBattleSettlementError::DefinitionMismatch)?;
        let resolution: GraphActivityBattleResolution = activity
            .submit_pending_battle_result_with_generated_boundary(
                expected,
                result,
                &[
                    base_fragments,
                    curio_grants,
                    curio_blessings,
                    lifetimes,
                    blessing_offer,
                ],
                |program, view, settlement, rng| {
                    if program == lifetimes
                        && matches!(
                            settlement.outcome(),
                            BattleOutcome::Won | BattleOutcome::Lost
                        )
                    {
                        battle_stats.settle_lifetimes(view)
                    } else if settlement.outcome() != BattleOutcome::Won {
                        Ok(Vec::new())
                    } else if program == base_fragments {
                        fragments.generate(view, flow)
                    } else if program == curio_grants {
                        grants.generate(view, flow)
                    } else if program == curio_blessings {
                        let domain = flow
                            .battle_reward_domain(view)
                            .ok_or(GraphActivityCommandError::DecisionNotOffered)?;
                        victory_blessings.generate(view, domain, rng)
                    } else {
                        rewards.generate(view, rng)
                    }
                },
            )
            .map_err(|_| DivergentUniverseBattleSettlementError::SettlementRejected)?;
        let reward = if resolution.settlement().outcome() != BattleOutcome::Won {
            DivergentUniverseBattleRewardDisposition::NoVictoryRewards
        } else if activity
            .player_view()
            .decision()
            .is_some_and(|decision| decision.kind() == ActivityDecisionKind::Reward)
        {
            DivergentUniverseBattleRewardDisposition::BlessingOfferGeneratedOtherRewardsPending
        } else {
            DivergentUniverseBattleRewardDisposition::NoBlessingOfferOtherRewardsPending
        };
        Ok(DivergentUniverseBattleSettlementResolution {
            result: retained_result,
            report,
            settlement: resolution.settlement(),
            events: resolution.events().to_vec().into_boxed_slice(),
            state_hash: resolution.state_hash(),
            reward,
            retry: DivergentUniverseBattleRetryDisposition::DefeatTerminatesCurrentActivityFreshActivityRequired,
        })
    }
}

fn validate_start(
    flow: &DivergentUniverseFlowInstance,
    activity: &GraphActivity,
    expected: ActivityStateHash,
    materialization: &DivergentUniverseAssembledBattle,
) -> Result<(), DivergentUniverseBattleSettlementError> {
    if !flow.has_runtime_battle_route()
        || activity.definition().identity() != flow.definition().identity()
        || activity.definition().graph().digest() != flow.definition().graph().digest()
        || materialization.difficulty() != flow.difficulty()
    {
        return Err(DivergentUniverseBattleSettlementError::DefinitionMismatch);
    }
    if activity.player_view().terminal().is_some() {
        return Err(DivergentUniverseBattleSettlementError::ActivityCompleted);
    }
    if expected != activity.state_hash() || materialization.source_state_hash() != expected {
        return Err(DivergentUniverseBattleSettlementError::StaleStateHash);
    }
    if let Some(binding) = flow
        .offered_encounter(activity)
        .map_err(|_| DivergentUniverseBattleSettlementError::EncounterNotOffered)?
        && binding != materialization.encounter_binding()
    {
        return Err(DivergentUniverseBattleSettlementError::InvalidBattleBinding);
    }
    Ok(())
}

fn settlement_contract(
    participants: &starclock_activity::ParticipantLock,
) -> Result<Arc<ActivityBattleResultContract>, DivergentUniverseBattleSettlementError> {
    let mut fields = vec![
        ProjectionField::Outcome,
        ProjectionField::FinalStateHash,
        ProjectionField::EventDigest,
        ProjectionField::TerminalFault,
    ];
    fields.extend(
        participants
            .entries()
            .iter()
            .map(|entry| ProjectionField::ParticipantState(entry.participant())),
    );
    let projection = starclock_activity::BattleResultProjection::new(
        ProjectionId::new(RESULT_PROJECTION).expect("reserved projection ID is non-zero"),
        fields,
    )
    .map_err(|_| DivergentUniverseBattleSettlementError::InvalidSettlementContract)?;
    let carry = participants
        .entries()
        .iter()
        .map(|entry| {
            ActivityParticipantCarryDefinition::new(
                entry.participant(),
                HpCarryPolicy::CarryClamped,
                EnergyCarryPolicy::CarryClamped,
                LifeCarryPolicy::DefeatOnZero,
                PresenceCarryPolicy::DepartIfDefeated,
            )
        })
        .collect();
    ActivityBattleResultContract::new(Arc::new(projection), carry, Vec::new())
        .map(Arc::new)
        .map_err(|_| DivergentUniverseBattleSettlementError::InvalidSettlementContract)
}

#[derive(Debug)]
pub enum DivergentUniverseBattleSettlementError {
    DefinitionMismatch,
    ActivityCompleted,
    StaleStateHash,
    EncounterNotOffered,
    InvalidScope,
    InvalidBattleBinding,
    InvalidSettlementContract,
    PreparationRejected,
    BattleStartRejected,
    Execution(NestedBattleExecutionError),
    MissingExecutionReport,
    SettlementRejected,
}

impl core::fmt::Display for DivergentUniverseBattleSettlementError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe battle settlement error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseBattleSettlementError {}
