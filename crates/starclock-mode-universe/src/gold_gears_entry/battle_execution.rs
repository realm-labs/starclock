//! Verified Activity handoff, real nested execution and post-battle settlement.

use std::sync::Arc;

use starclock_activity::{
    ActivityBattleHandoff, ActivityBattleInPlaceSettlement, ActivityBattleInPlaceSettlementError,
    ActivityBattlePreparationRequest, ActivityBattleResultContract, ActivityBattleResultSubmission,
    ActivityBattleSettlementError, ActivityBattleStartRequest, ActivityDefinitionIdentity,
    ActivityInstanceId, ActivityOptionId, ActivityPreparationBoundary, ActivityPreparationError,
    ActivityProgramDefinition, ActivityProgramId, ActivityRngStreams, ActivityRosterLock,
    ActivityScopePath, ActivityTransactionEvent, ActivityTransactionState, ActivityValue,
    AttemptId, BattleBinding, BattleOutcome, BattleResult, BattleSequence,
    EncounterInitiativePolicy, EncounterPreparationDefinition, EnergyCarryPolicy, HpCarryPolicy,
    LifeCarryPolicy, PreparedBattleVariant, PresenceCarryPolicy, ProjectionField, ProjectionId,
    TechniqueContributionDigest,
};
use starclock_combat::{Ratio, catalog::CombatCatalog};

use crate::{
    baseline_runner::{NestedBattleExecutionError, NestedBattleExecutor},
    battle_materialization::UniverseBattleRoster,
    nested_battle_executor::{NestedBattleExecutionReport, UniverseNestedBattleExecutor},
    service_effect_runtime::{ServiceAction, ServiceEffectRuntimeCatalog},
};

use super::{
    GoldAndGearsBattleAssemblyContext, GoldAndGearsEncounterRole, GoldAndGearsEncounterSelection,
    GoldAndGearsEntryError, GoldAndGearsRuntimeInstance,
    curio_types::{GoldAndGearsCurioId, GoldAndGearsCurioState},
    service_adventure_types::{GoldAndGearsServiceKind, GoldAndGearsServiceOfferSelector},
    state_layout::{
        CONTENT_CURIO_CHARGE_BASE, CONTENT_CURIO_STATE_BASE, CONTENT_LIFECYCLE_SLOT,
        CURIO_INVENTORY,
    },
};

pub const GOLD_AND_GEARS_BATTLE_EXECUTION_REVISION: &str =
    "gold-and-gears-nested-battle-execution-v1";

const NORMAL_ENGAGEMENT_OPTION: u64 = 0x7f71_0001;
const RESULT_PROJECTION_ID: u32 = 0x7f71_0001;
const POST_BATTLE_PROGRAM_BASE: u32 = 0x7f72_0000;
const REVIVAL_PROGRAM_BASE: u32 = 0x7f73_0000;

/// An exact started Activity handoff paired with the immutable catalog that
/// validated its materialized combat input.
pub struct GoldAndGearsBattleStart {
    handoff: ActivityBattleHandoff,
    combat_catalog: Arc<CombatCatalog>,
    role: GoldAndGearsEncounterRole,
    materialization_digest: [u8; 32],
    contribution_digest: [u8; 32],
}

impl GoldAndGearsBattleStart {
    #[must_use]
    pub const fn handoff(&self) -> &ActivityBattleHandoff {
        &self.handoff
    }
    #[must_use]
    pub const fn combat_catalog(&self) -> &Arc<CombatCatalog> {
        &self.combat_catalog
    }
    #[must_use]
    pub const fn role(&self) -> GoldAndGearsEncounterRole {
        self.role
    }
    #[must_use]
    pub const fn materialization_digest(&self) -> [u8; 32] {
        self.materialization_digest
    }
    #[must_use]
    pub const fn contribution_digest(&self) -> [u8; 32] {
        self.contribution_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsBattleExecution {
    result: BattleResult,
    report: NestedBattleExecutionReport,
    settlement: ActivityBattleInPlaceSettlement,
}

impl GoldAndGearsBattleExecution {
    #[must_use]
    pub const fn result(&self) -> &BattleResult {
        &self.result
    }
    #[must_use]
    pub const fn report(&self) -> &NestedBattleExecutionReport {
        &self.report
    }
    #[must_use]
    pub const fn settlement(&self) -> &ActivityBattleInPlaceSettlement {
        &self.settlement
    }
    #[must_use]
    pub fn post_battle_events(&self) -> &[ActivityTransactionEvent] {
        self.settlement.events()
    }
}

#[derive(Debug)]
pub enum GoldAndGearsBattleExecutionError {
    StaleState,
    InvalidInput(GoldAndGearsEntryError),
    Preparation(ActivityPreparationError),
    Start(ActivityBattleSettlementError),
    Execution(NestedBattleExecutionError),
    Settlement(ActivityBattleInPlaceSettlementError),
}

impl GoldAndGearsRuntimeInstance {
    /// Compiles the released shared Reviver effect together with Gold's
    /// exactly-once stock and currency accounting.
    pub fn compile_service_revival(
        &self,
        service: &str,
        participant: starclock_activity::ParticipantId,
        expected_uses: u8,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let definition = self
            .service_definitions()
            .iter()
            .find(|definition| definition.stable_key() == service)
            .filter(|definition| definition.kind() == GoldAndGearsServiceKind::Reviver)
            .ok_or_else(|| GoldAndGearsEntryError::UnknownService(service.into()))?;
        let purchase = self.compile_service_purchase(
            service,
            GoldAndGearsServiceOfferSelector::Reviver,
            expected_uses,
        )?;
        let released = self
            .content_runtime
            .standard
            .services()
            .iter()
            .find(|candidate| candidate.stable_key() == service)
            .ok_or(GoldAndGearsEntryError::InvalidServiceRuntime)?;
        let runtime = ServiceEffectRuntimeCatalog::compile(&self.content_runtime.run)
            .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)?;
        let applied = runtime
            .execute(released.id())
            .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)?;
        let ServiceAction::ReviveCharacter {
            cost,
            restored_hp_percent,
        } = applied.action()
        else {
            return Err(GoldAndGearsEntryError::InvalidServiceRuntime);
        };
        if definition
            .stock()
            .iter()
            .find(|stock| stock.selector() == GoldAndGearsServiceOfferSelector::Reviver)
            .is_none_or(|stock| stock.unit_cost() != *cost)
        {
            return Err(GoldAndGearsEntryError::InvalidServiceRuntime);
        }
        let hp_ratio = u32::from(*restored_hp_percent)
            .checked_mul(10_000)
            .map(i64::from)
            .map(Ratio::from_scaled)
            .ok_or(GoldAndGearsEntryError::InvalidServiceRuntime)?;
        let mut operations = purchase.operations().to_vec();
        operations.push(starclock_activity::ActivityOperation::Require(
            starclock_activity::ActivityCondition::ParticipantDefeated(participant),
        ));
        operations.push(starclock_activity::ActivityOperation::RestoreParticipant {
            participant,
            hp_ratio,
        });
        let id = REVIVAL_PROGRAM_BASE
            .checked_add(definition.id())
            .and_then(|value| value.checked_add(participant.get()))
            .and_then(ActivityProgramId::new)
            .ok_or(GoldAndGearsEntryError::InvalidServiceRuntime)?;
        ActivityProgramDefinition::new(id, operations)
            .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)
    }

    /// Materializes the current encounter and enters the generic Activity
    /// `StartBattle` boundary without consuming Activity RNG.
    #[allow(clippy::too_many_arguments)]
    pub fn start_current_battle(
        &self,
        state: &mut ActivityTransactionState,
        rng: &ActivityRngStreams,
        expected_state_hash: starclock_activity::ActivityStateHash,
        identity: ActivityDefinitionIdentity,
        activity_instance: ActivityInstanceId,
        attempt: AttemptId,
        sequence: BattleSequence,
        selection: &GoldAndGearsEncounterSelection,
        roster: &UniverseBattleRoster,
        context: &GoldAndGearsBattleAssemblyContext,
    ) -> Result<GoldAndGearsBattleStart, GoldAndGearsBattleExecutionError> {
        if state.state_hash(identity, &self.graph, activity_instance, rng) != expected_state_hash {
            return Err(GoldAndGearsBattleExecutionError::StaleState);
        }
        let materialization = self
            .materialize_current_battle(state, selection, roster, context)
            .map_err(GoldAndGearsBattleExecutionError::InvalidInput)?;
        let node = state.current_node();
        let section = self
            .graph
            .node(node)
            .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)
            .map_err(GoldAndGearsBattleExecutionError::InvalidInput)?
            .section();
        let path = ActivityScopePath::new(activity_instance)
            .enter_section(section)
            .and_then(|path| path.enter_node(node))
            .and_then(|path| path.enter_attempt(attempt))
            .map_err(|_| {
                GoldAndGearsBattleExecutionError::InvalidInput(
                    GoldAndGearsEntryError::InvalidBattleMaterialization,
                )
            })?;
        let roster_lock = ActivityRosterLock::new(
            ActivityScopePath::new(activity_instance),
            self.participants().as_ref().clone(),
        )
        .map_err(|_| {
            GoldAndGearsBattleExecutionError::InvalidInput(
                GoldAndGearsEntryError::InvalidBattleMaterialization,
            )
        })?;
        let contribution_digest = materialization.contributions().digest();
        let binding = BattleBinding::new(
            materialization.battle_spec().clone(),
            "gold-and-gears-battle",
            GOLD_AND_GEARS_BATTLE_EXECUTION_REVISION,
            materialization.participant_lock(),
        )
        .map_err(|_| {
            GoldAndGearsBattleExecutionError::InvalidInput(
                GoldAndGearsEntryError::InvalidBattleMaterialization,
            )
        })?;
        let normal = ActivityOptionId::new(NORMAL_ENGAGEMENT_OPTION)
            .expect("reserved engagement option is non-zero");
        let preparation = EncounterPreparationDefinition::new(
            normal,
            EncounterInitiativePolicy::PlayerControlled,
            materialization.participant_lock(),
            0,
            Vec::new(),
            vec![PreparedBattleVariant::new(
                Vec::new(),
                TechniqueContributionDigest::new(contribution_digest)
                    .expect("contribution digest is SHA-256"),
                binding,
            )],
        )
        .map(Arc::new)
        .map_err(|_| {
            GoldAndGearsBattleExecutionError::InvalidInput(
                GoldAndGearsEntryError::InvalidBattleMaterialization,
            )
        })?;
        let boundary = state
            .begin_battle_preparation(
                activity_instance,
                &self.graph,
                ActivityBattlePreparationRequest::new(path, roster_lock, sequence, 0, preparation),
            )
            .map_err(GoldAndGearsBattleExecutionError::Preparation)?;
        if boundary != ActivityPreparationBoundary::Decision
            || state
                .choose_preparation_option(normal)
                .map_err(GoldAndGearsBattleExecutionError::Preparation)?
                != ActivityPreparationBoundary::BattleReady
        {
            return Err(GoldAndGearsBattleExecutionError::InvalidInput(
                GoldAndGearsEntryError::InvalidBattleMaterialization,
            ));
        }
        let contract =
            settlement_contract(roster).map_err(GoldAndGearsBattleExecutionError::InvalidInput)?;
        let handoff = state
            .start_pending_battle(
                &self.graph,
                rng,
                ActivityBattleStartRequest::new(
                    state.state_hash(identity, &self.graph, activity_instance, rng),
                    identity,
                    activity_instance,
                    contract,
                ),
            )
            .map_err(GoldAndGearsBattleExecutionError::Start)?;
        Ok(GoldAndGearsBattleStart {
            handoff,
            combat_catalog: Arc::clone(materialization.combat_catalog()),
            role: selection.role(),
            materialization_digest: materialization.digest(),
            contribution_digest,
        })
    }

    /// Executes the exact started handoff in the real combat aggregate, then
    /// atomically verifies carry and commits Gold post-battle lifecycle work.
    pub fn execute_started_battle(
        &self,
        state: &mut ActivityTransactionState,
        rng: &ActivityRngStreams,
        identity: ActivityDefinitionIdentity,
        activity_instance: ActivityInstanceId,
        start: &GoldAndGearsBattleStart,
    ) -> Result<GoldAndGearsBattleExecution, GoldAndGearsBattleExecutionError> {
        let mut executor = UniverseNestedBattleExecutor::new(Arc::clone(&start.combat_catalog));
        let result = executor
            .execute(&start.handoff)
            .map_err(GoldAndGearsBattleExecutionError::Execution)?;
        let report = executor
            .last_report()
            .expect("successful execution records one report")
            .clone();
        let post_battle = if report.outcome() == BattleOutcome::Won {
            self.compile_post_battle_program(state, start.role)?
        } else {
            None
        };
        let settlement = state
            .submit_pending_battle_result_in_place(
                identity,
                &self.graph,
                activity_instance,
                rng,
                ActivityBattleResultSubmission::new(
                    state.state_hash(identity, &self.graph, activity_instance, rng),
                    result.clone(),
                ),
                post_battle.as_ref(),
            )
            .map_err(GoldAndGearsBattleExecutionError::Settlement)?;
        Ok(GoldAndGearsBattleExecution {
            result,
            report,
            settlement,
        })
    }

    fn compile_post_battle_program(
        &self,
        state: &ActivityTransactionState,
        role: GoldAndGearsEncounterRole,
    ) -> Result<Option<ActivityProgramDefinition>, GoldAndGearsBattleExecutionError> {
        let mut operations = Vec::new();
        let owned = state
            .inventory_entries(
                starclock_activity::ActivityInventoryId::new(CURIO_INVENTORY)
                    .expect("static inventory is non-zero"),
            )
            .ok_or(GoldAndGearsEntryError::InvalidCurioInventory)
            .map_err(GoldAndGearsBattleExecutionError::InvalidInput)?;
        for (raw, count) in owned {
            if count == 0 {
                continue;
            }
            let Some(id) = u32::try_from(raw).ok().and_then(GoldAndGearsCurioId::new) else {
                return Err(GoldAndGearsBattleExecutionError::InvalidInput(
                    GoldAndGearsEntryError::InvalidCurioInventory,
                ));
            };
            if lifecycle_counter(state, CONTENT_CURIO_STATE_BASE + raw)
                == i64::from(GoldAndGearsCurioState::Repairing as u8)
            {
                let progress =
                    u8::try_from(lifecycle_counter(state, CONTENT_CURIO_CHARGE_BASE + raw))
                        .map_err(|_| {
                            GoldAndGearsBattleExecutionError::InvalidInput(
                                GoldAndGearsEntryError::InvalidCurioInventory,
                            )
                        })?;
                operations.extend_from_slice(
                    self.compile_curio_repair_progress(id, progress)
                        .map_err(GoldAndGearsBattleExecutionError::InvalidInput)?
                        .operations(),
                );
            }
        }
        let plane = match role {
            GoldAndGearsEncounterRole::FirstPlaneBoss => Some(1),
            GoldAndGearsEncounterRole::SecondPlaneBoss => Some(2),
            GoldAndGearsEncounterRole::FinalBoss => Some(3),
            GoldAndGearsEncounterRole::Combat | GoldAndGearsEncounterRole::Elite => None,
        };
        if let Some(plane) = plane {
            operations.extend_from_slice(
                self.compile_plane_completion(plane)
                    .map_err(GoldAndGearsBattleExecutionError::InvalidInput)?
                    .operations(),
            );
        }
        if operations.is_empty() {
            return Ok(None);
        }
        let id = POST_BATTLE_PROGRAM_BASE
            .checked_add(state.current_node().get())
            .and_then(ActivityProgramId::new)
            .ok_or(GoldAndGearsBattleExecutionError::InvalidInput(
                GoldAndGearsEntryError::InvalidBattleMaterialization,
            ))?;
        ActivityProgramDefinition::new(id, operations)
            .map(Some)
            .map_err(|_| {
                GoldAndGearsBattleExecutionError::InvalidInput(
                    GoldAndGearsEntryError::InvalidBattleMaterialization,
                )
            })
    }
}

fn settlement_contract(
    roster: &UniverseBattleRoster,
) -> Result<Arc<ActivityBattleResultContract>, GoldAndGearsEntryError> {
    let mut fields = vec![
        ProjectionField::Outcome,
        ProjectionField::FinalStateHash,
        ProjectionField::EventDigest,
        ProjectionField::TerminalFault,
    ];
    fields.extend(
        roster
            .entries()
            .iter()
            .map(|entry| ProjectionField::ParticipantState(entry.participant())),
    );
    let projection = starclock_activity::BattleResultProjection::new(
        ProjectionId::new(RESULT_PROJECTION_ID).expect("reserved projection is non-zero"),
        fields,
    )
    .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?;
    let carry = roster
        .entries()
        .iter()
        .map(|entry| {
            starclock_activity::ActivityParticipantCarryDefinition::new(
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
        .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)
}

fn lifecycle_counter(state: &ActivityTransactionState, key: u64) -> i64 {
    match state.slot(
        starclock_activity::ActivitySlotId::new(CONTENT_LIFECYCLE_SLOT)
            .expect("static slot is non-zero"),
    ) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1),
        _ => 0,
    }
}
