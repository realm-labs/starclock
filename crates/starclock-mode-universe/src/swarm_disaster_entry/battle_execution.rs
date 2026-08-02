//! Verified Swarm Activity handoff, real combat execution and settlement.

use std::sync::Arc;

use starclock_activity::{
    ActivityBattleHandoff, ActivityBattleInPlaceSettlement, ActivityBattlePreparationRequest,
    ActivityBattleResultContract, ActivityBattleResultSubmission, ActivityBattleStartRequest,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityOptionId,
    ActivityParticipantCarryDefinition, ActivityPreparationBoundary, ActivityProgramDefinition,
    ActivityProgramId, ActivityRngStreams, ActivityRosterLock, ActivityScopePath,
    ActivityStateHash, ActivityTransactionState, ActivityValue, AttemptId, BattleBinding,
    BattleOutcome, BattleResult, BattleSequence, EncounterInitiativePolicy,
    EncounterPreparationDefinition, EnergyCarryPolicy, HpCarryPolicy, LifeCarryPolicy,
    PreparedBattleVariant, PresenceCarryPolicy, ProjectionField, ProjectionId,
    TechniqueContributionDigest,
};
use starclock_combat::{Ratio, catalog::CombatCatalog};

use crate::{
    baseline_runner::{NestedBattleExecutionError, NestedBattleExecutor},
    battle_materialization::UniverseBattleRoster,
    error::UniverseCatalogLoadError,
    nested_battle_executor::{NestedBattleExecutionReport, UniverseNestedBattleExecutor},
    service_effect_runtime::{ServiceAction, ServiceEffectRuntimeCatalog},
};

use super::{
    SwarmDisasterRuntimeInstance,
    battle_materialization::SwarmBattleMaterialization,
    content_runtime::{CurioState, counter_key, state_key},
    encounter_runtime::{EncounterRole, EncounterSelection},
    state::{CONTENT, CURIO_INVENTORY},
    validate::{error as invalid, reference},
};

pub const SWARM_DISASTER_BATTLE_EXECUTION_REVISION: &str =
    "swarm-disaster-nested-battle-execution-v1";

const NORMAL_ENGAGEMENT_OPTION: u64 = 0x7f95_0001;
const RESULT_PROJECTION_ID: u32 = 0x7f95_0001;
const POST_BATTLE_PROGRAM_BASE: u32 = 0x7f96_0000;
const REVIVAL_PROGRAM_BASE: u32 = 0x7f97_0000;

pub(super) struct SwarmBattleStart {
    handoff: ActivityBattleHandoff,
    combat_catalog: Arc<CombatCatalog>,
    role: EncounterRole,
    plane: u8,
}

impl SwarmBattleStart {
    pub(super) const fn handoff(&self) -> &ActivityBattleHandoff {
        &self.handoff
    }

    #[cfg(test)]
    pub(super) const fn combat_catalog(&self) -> &Arc<CombatCatalog> {
        &self.combat_catalog
    }
}

impl SwarmDisasterRuntimeInstance {
    /// Starts, executes and atomically settles the current real nested battle.
    ///
    /// `previous_first_plane_completed` is caller-owned account progression
    /// used only by the released conditional Communing Trail contribution.
    /// No new Swarm command processor or battle state machine is introduced.
    #[allow(clippy::too_many_arguments)]
    pub fn execute_current_battle(
        &self,
        state: &mut ActivityTransactionState,
        rng: &mut ActivityRngStreams,
        expected_state_hash: ActivityStateHash,
        identity: ActivityDefinitionIdentity,
        activity_instance: ActivityInstanceId,
        attempt: AttemptId,
        sequence: BattleSequence,
        roster: &UniverseBattleRoster,
        previous_first_plane_completed: bool,
    ) -> Result<
        (
            BattleResult,
            NestedBattleExecutionReport,
            ActivityBattleInPlaceSettlement,
        ),
        UniverseCatalogLoadError,
    > {
        let start = self.start_current_battle(
            state,
            rng,
            expected_state_hash,
            identity,
            activity_instance,
            attempt,
            sequence,
            roster,
        )?;
        self.execute_started_battle(
            state,
            rng,
            identity,
            activity_instance,
            &start,
            previous_first_plane_completed,
        )
    }

    /// Compiles the released shared Reviver effect together with the exact
    /// Swarm Service price/use accounting in one Activity program.
    pub fn compile_service_revival(
        &self,
        service: &str,
        participant: starclock_activity::ParticipantId,
        expected_uses: u8,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let (service_id, shared_key, authored_cost) =
            self.service_adventure.reviver_binding(service)?;
        let purchase = self.compile_service_purchase(service, authored_cost, expected_uses)?;
        let shared = self
            .content_runtime
            .standard
            .services()
            .iter()
            .find(|candidate| candidate.stable_key() == shared_key)
            .ok_or_else(|| reference("missing shared Swarm Reviver definition"))?;
        let runtime = ServiceEffectRuntimeCatalog::compile(&self.content_runtime.run)
            .map_err(|_| invalid("invalid shared Swarm Service runtime"))?;
        let applied = runtime
            .execute(shared.id())
            .map_err(|_| invalid("invalid shared Swarm Reviver execution"))?;
        let ServiceAction::ReviveCharacter {
            cost,
            restored_hp_percent,
        } = applied.action()
        else {
            return Err(reference("Swarm Service is not a released Reviver"));
        };
        if *cost != authored_cost {
            return Err(reference("Swarm Reviver cost binding drift"));
        }
        let hp_ratio = u32::from(*restored_hp_percent)
            .checked_mul(10_000)
            .map(i64::from)
            .map(Ratio::from_scaled)
            .ok_or_else(|| invalid("invalid Swarm Reviver HP ratio"))?;
        let mut operations = purchase.operations().to_vec();
        operations.push(starclock_activity::ActivityOperation::Require(
            starclock_activity::ActivityCondition::ParticipantDefeated(participant),
        ));
        operations.push(starclock_activity::ActivityOperation::RestoreParticipant {
            participant,
            hp_ratio,
        });
        let id = REVIVAL_PROGRAM_BASE
            .checked_add(service_id)
            .and_then(|value| value.checked_add(participant.get()))
            .and_then(ActivityProgramId::new)
            .ok_or_else(|| invalid("Swarm Reviver program ID overflow"))?;
        ActivityProgramDefinition::new(id, operations)
            .map_err(|_| invalid("invalid Swarm Reviver program"))
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn start_current_battle(
        &self,
        state: &mut ActivityTransactionState,
        rng: &mut ActivityRngStreams,
        expected_state_hash: ActivityStateHash,
        identity: ActivityDefinitionIdentity,
        activity_instance: ActivityInstanceId,
        attempt: AttemptId,
        sequence: BattleSequence,
        roster: &UniverseBattleRoster,
    ) -> Result<SwarmBattleStart, UniverseCatalogLoadError> {
        if state.state_hash(identity, &self.graph, activity_instance, rng) != expected_state_hash {
            return Err(reference("stale Swarm battle start state"));
        }
        rng.transact(|working_rng| {
            let materialization = self.resolve_current_battle(state, working_rng, roster)?;
            validate_boss_choice(self, state, &materialization.selection)?;
            start_materialization(
                self,
                state,
                working_rng,
                identity,
                activity_instance,
                attempt,
                sequence,
                roster,
                materialization,
            )
        })
    }

    pub(super) fn execute_started_battle(
        &self,
        state: &mut ActivityTransactionState,
        rng: &ActivityRngStreams,
        identity: ActivityDefinitionIdentity,
        activity_instance: ActivityInstanceId,
        start: &SwarmBattleStart,
        previous_first_plane_completed: bool,
    ) -> Result<
        (
            BattleResult,
            NestedBattleExecutionReport,
            ActivityBattleInPlaceSettlement,
        ),
        UniverseCatalogLoadError,
    > {
        let mut executor = UniverseNestedBattleExecutor::new(Arc::clone(&start.combat_catalog));
        let result = executor.execute(&start.handoff).map_err(execution_error)?;
        let report = executor
            .last_report()
            .ok_or_else(|| invalid("missing Swarm nested battle report"))?
            .clone();
        let post_battle = if report.outcome() == BattleOutcome::Won {
            self.compile_post_battle_program(
                state,
                start.role,
                start.plane,
                previous_first_plane_completed,
            )?
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
            .map_err(|_| reference("Swarm nested battle settlement rejected"))?;
        Ok((result, report, settlement))
    }

    fn compile_post_battle_program(
        &self,
        state: &ActivityTransactionState,
        role: EncounterRole,
        plane: u8,
        previous_first_plane_completed: bool,
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        let mut operations = Vec::new();
        if let Some(accounting) = self.compile_trail_battle_entry_accounting(
            state,
            plane,
            !matches!(role, EncounterRole::Combat | EncounterRole::Elite),
            previous_first_plane_completed,
        )? {
            operations.extend_from_slice(accounting.operations());
        }
        let owned = state
            .inventory_entries(
                starclock_activity::ActivityInventoryId::new(CURIO_INVENTORY)
                    .expect("static Swarm Curio inventory is non-zero"),
            )
            .ok_or_else(|| reference("missing Swarm Curio inventory"))?;
        for (raw, count) in owned {
            if count == 0 {
                continue;
            }
            let id = u32::try_from(raw)
                .map_err(|_| reference("invalid Swarm Curio inventory identity"))?;
            if lifecycle_counter(state, state_key(id)) == i64::from(CurioState::Repairing as u8) {
                let progress = u8::try_from(lifecycle_counter(state, counter_key(id)))
                    .map_err(|_| reference("invalid Swarm Curio repair progress"))?;
                operations.extend_from_slice(
                    self.compile_curio_repair_progress(id, progress)?
                        .operations(),
                );
            }
        }
        if !matches!(role, EncounterRole::Combat | EncounterRole::Elite)
            && self
                .plane_ends()
                .nth(usize::from(plane - 1))
                .is_some_and(|end| end == state.current_node())
        {
            operations.extend_from_slice(self.compile_plane_completion(state, plane)?.operations());
        }
        if operations.is_empty() {
            return Ok(None);
        }
        let id = POST_BATTLE_PROGRAM_BASE
            .checked_add(state.current_node().get())
            .and_then(ActivityProgramId::new)
            .ok_or_else(|| invalid("Swarm post-battle program ID overflow"))?;
        ActivityProgramDefinition::new(id, operations)
            .map(Some)
            .map_err(|_| invalid("invalid Swarm post-battle program"))
    }
}

#[allow(clippy::too_many_arguments)]
fn start_materialization(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    rng: &ActivityRngStreams,
    identity: ActivityDefinitionIdentity,
    activity_instance: ActivityInstanceId,
    attempt: AttemptId,
    sequence: BattleSequence,
    roster: &UniverseBattleRoster,
    materialization: SwarmBattleMaterialization,
) -> Result<SwarmBattleStart, UniverseCatalogLoadError> {
    let node = state.current_node();
    let section = instance
        .graph
        .node(node)
        .ok_or_else(|| reference("missing Swarm battle graph node"))?
        .section();
    let path = ActivityScopePath::new(activity_instance)
        .enter_section(section)
        .and_then(|path| path.enter_node(node))
        .and_then(|path| path.enter_attempt(attempt))
        .map_err(|_| invalid("invalid Swarm battle scope path"))?;
    let roster_lock = ActivityRosterLock::new(
        ActivityScopePath::new(activity_instance),
        instance.participants().clone(),
    )
    .map_err(|_| reference("invalid Swarm Activity roster lock"))?;
    let binding = BattleBinding::new(
        materialization.battle_spec.clone(),
        "swarm-disaster-battle",
        SWARM_DISASTER_BATTLE_EXECUTION_REVISION,
        roster.participant_lock(),
    )
    .map_err(|_| invalid("invalid Swarm battle binding"))?;
    let normal = ActivityOptionId::new(NORMAL_ENGAGEMENT_OPTION)
        .expect("reserved Swarm engagement option is non-zero");
    let preparation = EncounterPreparationDefinition::new(
        normal,
        EncounterInitiativePolicy::PlayerControlled,
        roster.participant_lock(),
        0,
        Vec::new(),
        vec![PreparedBattleVariant::new(
            Vec::new(),
            TechniqueContributionDigest::new(materialization.snapshot_digest)
                .expect("Swarm snapshot digest is SHA-256"),
            binding,
        )],
    )
    .map(Arc::new)
    .map_err(|_| invalid("invalid Swarm encounter preparation"))?;
    let boundary = state
        .begin_battle_preparation(
            activity_instance,
            &instance.graph,
            ActivityBattlePreparationRequest::new(path, roster_lock, sequence, 0, preparation),
        )
        .map_err(|_| reference("Swarm battle preparation rejected"))?;
    if boundary != ActivityPreparationBoundary::Decision
        || state
            .choose_preparation_option(normal)
            .map_err(|_| reference("Swarm battle preparation option rejected"))?
            != ActivityPreparationBoundary::BattleReady
    {
        return Err(invalid("invalid Swarm battle preparation boundary"));
    }
    let contract = settlement_contract(roster)?;
    let handoff = state
        .start_pending_battle(
            &instance.graph,
            rng,
            ActivityBattleStartRequest::new(
                state.state_hash(identity, &instance.graph, activity_instance, rng),
                identity,
                activity_instance,
                contract,
            ),
        )
        .map_err(|_| reference("Swarm pending battle start rejected"))?;
    Ok(SwarmBattleStart {
        handoff,
        combat_catalog: materialization.combat_catalog,
        role: materialization.selection.role,
        plane: u8::try_from(section.get()).map_err(|_| invalid("Swarm plane section overflow"))?,
    })
}

fn settlement_contract(
    roster: &UniverseBattleRoster,
) -> Result<Arc<ActivityBattleResultContract>, UniverseCatalogLoadError> {
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
    .map_err(|_| invalid("invalid Swarm battle result projection"))?;
    let carry = roster
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
        .map_err(|_| invalid("invalid Swarm battle result contract"))
}

fn validate_boss_choice(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    selection: &EncounterSelection,
) -> Result<(), UniverseCatalogLoadError> {
    let plane = match selection.role {
        EncounterRole::Combat | EncounterRole::Elite => return Ok(()),
        EncounterRole::FirstPlaneBoss => 1,
        EncounterRole::SecondPlaneBoss => 2,
        EncounterRole::FinalBoss => 3,
    };
    let selected = instance
        .selected_boss(state, plane)
        .ok_or_else(|| reference("missing explicit Swarm boss choice"))?;
    let authored = selection
        .waves
        .iter()
        .flat_map(|wave| &wave.slots)
        .flat_map(|slot| &slot.boss_choices)
        .collect::<Vec<_>>();
    if !authored.is_empty() && authored.iter().all(|choice| choice.as_ref() != selected) {
        return Err(reference(
            "selected Swarm boss is absent from authored encounter",
        ));
    }
    Ok(())
}

fn lifecycle_counter(state: &ActivityTransactionState, key: u64) -> i64 {
    match state.slot(
        starclock_activity::ActivitySlotId::new(CONTENT)
            .expect("static Swarm content slot is non-zero"),
    ) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1),
        _ => 0,
    }
}

fn execution_error(_: NestedBattleExecutionError) -> UniverseCatalogLoadError {
    reference("Swarm nested battle execution failed")
}
