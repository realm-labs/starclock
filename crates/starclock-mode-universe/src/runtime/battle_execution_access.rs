//! Nested battle handoff and settlement accessors for the runtime facade.

use std::sync::Arc;

use starclock_activity::{
    ActivityBattleHandoff, ActivityBattleResultContract, ActivityProgramDefinition,
    ActivityProgramId, ActivityStateHash, BattleBinding, BattleOutcome, BattleResult,
    GraphActivityBattleError, GraphActivityBattleResolution, GraphActivityRuntimeError,
    ParticipantBattleState, ProjectedValue, TechniqueContributionDigest,
};
use starclock_combat::{LifeState, PresenceState, Ratio};

use crate::ability_runtime::{AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope};
use crate::{
    curio::CurioStateKind,
    curio_activity::event_key,
    curio_effect_runtime::{
        AppliedCurioEffect, CurioEffect, CurioEffectFacts, CurioEffectRuntimeError, CurioEvent,
    },
    curio_runtime::CurioRuntimeBindings,
    definition::DomainKind,
    path_effect_runtime::{PathEffect, PathEffectTarget},
};

use super::{StandardUniverseActivity, StandardUniverseBattleStartError};
const AFTER_BATTLE_ABILITY_PROGRAM: u32 = 9_700_001;

impl StandardUniverseActivity {
    pub(crate) fn start_assembled_pending_battle(
        &mut self,
        expected_state_hash: ActivityStateHash,
        binding: BattleBinding,
        contribution: TechniqueContributionDigest,
        contract: Arc<ActivityBattleResultContract>,
    ) -> Result<ActivityBattleHandoff, StandardUniverseBattleStartError> {
        self.graph
            .start_assembled_pending_battle(expected_state_hash, binding, contribution, contract)
            .map_err(StandardUniverseBattleStartError::Activity)
    }

    pub fn start_pending_battle(
        &mut self,
        expected_state_hash: ActivityStateHash,
    ) -> Result<ActivityBattleHandoff, StandardUniverseBattleStartError> {
        let digest = self
            .graph
            .pending_battle()
            .ok_or(StandardUniverseBattleStartError::MissingPendingBattle)?
            .battle_spec_digest();
        let binding = self
            .overlay
            .binding_for_spec(digest.bytes())
            .ok_or(StandardUniverseBattleStartError::MissingBattleOverlay)?;
        self.graph
            .start_pending_battle(expected_state_hash, Arc::clone(binding.contract()))
            .map_err(StandardUniverseBattleStartError::Activity)
    }

    pub(crate) fn rollback_pending_battle_start(&mut self) -> bool {
        self.graph.rollback_pending_battle_start()
    }

    pub fn submit_pending_battle_result(
        &mut self,
        expected_state_hash: ActivityStateHash,
        mut result: BattleResult,
    ) -> Result<GraphActivityBattleResolution, GraphActivityBattleError> {
        let laurel = self.non_final_defeat_laurel(&result)?;
        if laurel.is_some() {
            result = full_restore_victory(&result)?;
        }
        let won = result
            .values()
            .iter()
            .any(|value| matches!(value, ProjectedValue::Outcome(BattleOutcome::Won)));
        let boundary_program = if won {
            let first_battle_won = self.view().completed_battle_count() == 0;
            let chosen_path_blessings = self
                .path_contributions()
                .map_err(|_| invalid_boundary())?
                .selected_path_blessings();
            let projection = self
                .ability_activity_delta_projection(AbilityExecutionContext::new(
                    AbilityProjectionScope::Run,
                    AbilityBoundary::AfterBattle,
                    chosen_path_blessings,
                    first_battle_won,
                ))
                .map_err(|_| invalid_boundary())?;
            let mut operations = projection.operations().to_vec();
            let contributions = self.curio_contributions().map_err(|_| invalid_boundary())?;
            if let Some(curio) = laurel {
                operations.push(starclock_activity::ActivityOperation::AddCounter {
                    slot: self.curio_event_slot,
                    key: event_key(curio, CurioEvent::RunDefeated),
                    delta: integer(1),
                });
                operations.extend(crate::curio_activity::destroy_and_count_operations(
                    curio,
                    self.curio_activity_bindings(),
                ));
            }
            for contribution in contributions.entries() {
                let effects = optional_curio_effects(
                    &self.curio_effect_runtime,
                    contribution.curio(),
                    CurioEvent::BattleWon,
                    CurioEffectFacts::default(),
                )
                .map_err(|_| invalid_boundary())?;
                let Some(ratio) = effects.iter().find_map(|applied| match applied.effect() {
                    CurioEffect::Battle(PathEffect::HealMaximumHpRatio {
                        target: PathEffectTarget::AllAllies,
                        ratio,
                    }) => Some(ratio.raw_six_decimal()),
                    _ => None,
                }) else {
                    continue;
                };
                if !(1..=1_000_000).contains(&ratio) {
                    return Err(invalid_boundary());
                }
                operations.extend(result.values().iter().filter_map(|value| match value {
                    ProjectedValue::ParticipantState(state) if state.life() == LifeState::Alive => {
                        Some(
                            starclock_activity::ActivityOperation::HealParticipantMaximumHpRatio {
                                participant: state.participant(),
                                hp_ratio: Ratio::from_scaled(ratio),
                            },
                        )
                    }
                    _ => None,
                }));
                operations.push(starclock_activity::ActivityOperation::AddCounter {
                    slot: self.curio_event_slot,
                    key: event_key(contribution.curio(), CurioEvent::BattleWon),
                    delta: integer(1),
                });
            }
            for contribution in contributions
                .entries()
                .iter()
                .filter(|entry| entry.state().kind() == CurioStateKind::Repairing)
            {
                let remaining = contribution
                    .state()
                    .charge()
                    .ok_or_else(invalid_boundary)?
                    .remaining();
                operations.extend(
                    self.curio_runtime
                        .consume_charge_operations(
                            contribution.curio(),
                            remaining,
                            CurioRuntimeBindings {
                                inventory: self.curio_inventory,
                                state_slot: self.curio_state_slot,
                                charge_slot: self.curio_charge_slot,
                            },
                        )
                        .map_err(|_| invalid_boundary())?,
                );
            }
            if let Some(fission) = contributions
                .entries()
                .iter()
                .find(|entry| entry.state().source_effect_id() == "78")
            {
                operations.push(starclock_activity::ActivityOperation::AddCounter {
                    slot: self.curio_event_slot,
                    key: event_key(fission.curio(), CurioEvent::BattleWon),
                    delta: integer(1),
                });
            }
            if let Some(curio) = contributions
                .entries()
                .iter()
                .find(|entry| entry.state().source_effect_id() == "76")
            {
                let full_hp_allies = result
                    .values()
                    .iter()
                    .filter_map(|value| match value {
                        ProjectedValue::ParticipantState(state) => Some(*state),
                        _ => None,
                    })
                    .filter(|state| state.current_hp() == state.maximum_hp())
                    .count();
                let event = CurioEvent::BattleWon;
                let projection = self
                    .curio_activity_projection(
                        curio.curio(),
                        event,
                        CurioEffectFacts {
                            full_hp_allies: u32::try_from(full_hp_allies)
                                .map_err(|_| invalid_boundary())?,
                            destroyed_curios: contributions.destroyed_curios(),
                            ..CurioEffectFacts::default()
                        },
                    )
                    .map_err(|_| invalid_boundary())?;
                operations.push(starclock_activity::ActivityOperation::AddCounter {
                    slot: self.curio_event_slot,
                    key: event_key(curio.curio(), event),
                    delta: starclock_activity::ActivityExpression::Literal(
                        starclock_activity::ActivityValue::BoundedInteger(1),
                    ),
                });
                operations.extend_from_slice(projection.operations());
            }
            let defeated = result
                .values()
                .iter()
                .filter_map(|value| match value {
                    ProjectedValue::ParticipantState(state)
                        if state.life() != LifeState::Alive && state.current_hp().get() == 0 =>
                    {
                        Some(state.participant())
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            if !defeated.is_empty() {
                for contribution in contributions.entries() {
                    let effects = optional_curio_effects(
                        &self.curio_effect_runtime,
                        contribution.curio(),
                        CurioEvent::BattleWon,
                        CurioEffectFacts::default(),
                    )
                    .map_err(|_| invalid_boundary())?;
                    if !effects.iter().any(|effect| {
                        matches!(effect.effect(), CurioEffect::RevivePartyAndRestoreFullHp)
                    }) {
                        continue;
                    }
                    operations.extend(defeated.iter().map(|participant| {
                        starclock_activity::ActivityOperation::RestoreParticipant {
                            participant: *participant,
                            hp_ratio: Ratio::ONE,
                        }
                    }));
                    operations.push(starclock_activity::ActivityOperation::AddCounter {
                        slot: self.curio_event_slot,
                        key: event_key(contribution.curio(), CurioEvent::BattleWon),
                        delta: starclock_activity::ActivityExpression::Literal(
                            starclock_activity::ActivityValue::BoundedInteger(1),
                        ),
                    });
                    let destroys_once = effects.iter().any(|effect| {
                        matches!(
                            effect.effect(),
                            CurioEffect::DestroyAfterTriggers { triggers: 1 }
                        )
                    });
                    if !destroys_once {
                        return Err(invalid_boundary());
                    }
                    operations.extend(crate::curio_activity::destroy_and_count_operations(
                        contribution.curio(),
                        self.curio_activity_bindings(),
                    ));
                }
            }
            (!operations.is_empty())
                .then(|| {
                    ActivityProgramDefinition::new(
                        ActivityProgramId::new(AFTER_BATTLE_ABILITY_PROGRAM)
                            .expect("static boundary program ID is non-zero"),
                        operations,
                    )
                })
                .transpose()
                .map_err(|_| invalid_boundary())?
        } else {
            None
        };
        self.graph
            .submit_pending_battle_result_with_boundary_program(
                expected_state_hash,
                result,
                boundary_program.as_ref(),
            )
    }

    fn non_final_defeat_laurel(
        &self,
        result: &BattleResult,
    ) -> Result<Option<crate::id::CurioId>, GraphActivityBattleError> {
        if result.actual_digest() != result.claimed_digest()
            || !result
                .values()
                .iter()
                .any(|value| matches!(value, ProjectedValue::Outcome(BattleOutcome::Lost)))
            || self.pending_domain_kind()? == DomainKind::Boss
        {
            return Ok(None);
        }
        let contributions = self.curio_contributions().map_err(|_| invalid_boundary())?;
        for contribution in contributions.entries() {
            let effects = optional_curio_effects(
                &self.curio_effect_runtime,
                contribution.curio(),
                CurioEvent::RunDefeated,
                CurioEffectFacts {
                    final_domain: false,
                    ..CurioEffectFacts::default()
                },
            )
            .map_err(|_| invalid_boundary())?;
            if effects.iter().any(|effect| {
                matches!(
                    effect.effect(),
                    CurioEffect::TreatNonFinalDefeatAsVictoryAndRestoreFullHp
                )
            }) {
                return Ok(Some(contribution.curio()));
            }
        }
        Ok(None)
    }

    fn pending_domain_kind(&self) -> Result<DomainKind, GraphActivityBattleError> {
        let pending = self.graph.pending_battle().ok_or_else(invalid_boundary)?;
        let member = self
            .overlay
            .binding_for_spec(pending.battle_spec_digest().bytes())
            .ok_or_else(invalid_boundary)?
            .member();
        let room = self
            .graph
            .debug_view()
            .all_slots()
            .iter()
            .find(|slot| slot.id() == self.selected_room_slot)
            .and_then(|slot| match slot.value() {
                starclock_activity::ActivityValue::OptionalId(Some(value)) => Some(*value),
                _ => None,
            })
            .and_then(|value| u32::try_from(value).ok())
            .and_then(crate::id::RoomId::new)
            .ok_or_else(invalid_boundary)?;
        self.encounter_options
            .iter()
            .find(|binding| binding.member() == member && binding.room() == room)
            .map(|binding| binding.domain_kind())
            .ok_or_else(invalid_boundary)
    }
}

const fn invalid_boundary() -> GraphActivityBattleError {
    GraphActivityBattleError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}

fn optional_curio_effects(
    runtime: &crate::curio_effect_runtime::CurioEffectRuntimeCatalog,
    curio: crate::id::CurioId,
    event: CurioEvent,
    facts: CurioEffectFacts,
) -> Result<Box<[AppliedCurioEffect]>, CurioEffectRuntimeError> {
    match runtime.execute(curio, event, facts) {
        Err(CurioEffectRuntimeError::UnknownCurio) => Ok(Box::new([])),
        outcome => outcome,
    }
}

fn full_restore_victory(result: &BattleResult) -> Result<BattleResult, GraphActivityBattleError> {
    if result.actual_digest() != result.claimed_digest() {
        return Err(invalid_boundary());
    }
    let values = result
        .values()
        .iter()
        .map(|value| match value {
            ProjectedValue::Outcome(_) => Ok(ProjectedValue::Outcome(BattleOutcome::Won)),
            ProjectedValue::ParticipantState(state) => ParticipantBattleState::new(
                state.participant(),
                state.maximum_hp(),
                state.maximum_hp(),
                state.current_energy(),
                state.maximum_energy(),
                LifeState::Alive,
                PresenceState::Present,
            )
            .map(ProjectedValue::ParticipantState)
            .ok_or_else(invalid_boundary),
            value => Ok(value.clone()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BattleResult::seal(result.identity(), values))
}

fn integer(value: i64) -> starclock_activity::ActivityExpression {
    starclock_activity::ActivityExpression::Literal(
        starclock_activity::ActivityValue::BoundedInteger(value),
    )
}
