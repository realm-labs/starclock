use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityOptionId, ActivityProgramDefinition,
    ActivityProgramId, ActivityStateHash, ActivityValue, GraphActivityCommandError,
};
use starclock_combat::Energy;

use crate::{
    curio_effect_runtime::CurioEvent,
    curio_runtime::{CurioRuntimeBindings, CurioRuntimeError},
    id::{BlessingId, CurioId},
    run_runtime::RunRuntimeError,
};

use super::StandardUniverseActivity;

const CURIO_DESTRUCTIBLE_PROGRAM: u32 = 9_700_002;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioDestructibleOutcome {
    NoEffect,
    Blessing(BlessingId),
    Failure,
}

impl StandardUniverseActivity {
    pub fn technique_points(&self) -> Result<u16, RunRuntimeError> {
        self.view()
            .slots()
            .iter()
            .find(|slot| slot.id() == self.technique_points_slot)
            .and_then(|slot| match slot.value() {
                ActivityValue::BoundedInteger(value) => u16::try_from(*value).ok(),
                _ => None,
            })
            .ok_or(RunRuntimeError::InvalidTechniquePoints)
    }

    pub fn resolve_destructible_lottery(
        &mut self,
        expected_state_hash: ActivityStateHash,
        outcome: CurioDestructibleOutcome,
    ) -> Result<
        Box<[starclock_activity::ActivityTransactionEvent]>,
        StandardUniverseCurioCommandError,
    > {
        let curio = CurioId::new(5).expect("Interastral Big Lotto ID is non-zero");
        if !self
            .curio_contributions()
            .map_err(StandardUniverseCurioCommandError::Curio)?
            .entries()
            .iter()
            .any(|entry| entry.curio() == curio)
        {
            return Err(StandardUniverseCurioCommandError::NotOwned);
        }
        let mut operations = vec![ActivityOperation::AddCounter {
            slot: self.curio_event_slot,
            key: crate::curio_activity::event_key(curio, CurioEvent::DestructibleDestroyed),
            delta: integer(1),
        }];
        match outcome {
            CurioDestructibleOutcome::NoEffect => {}
            CurioDestructibleOutcome::Blessing(blessing) => {
                let option = self
                    .blessing_runtime
                    .acquisition_option(
                        blessing,
                        ActivityOptionId::new(u64::from(blessing.get()))
                            .expect("Blessing ID is non-zero"),
                        0,
                        self.blessing_inventory,
                        Vec::new(),
                    )
                    .ok_or(StandardUniverseCurioCommandError::UnknownBlessing)?;
                operations.push(ActivityOperation::Require(option.enabled().clone()));
                operations.extend_from_slice(option.operations());
            }
            CurioDestructibleOutcome::Failure => {
                operations.extend(
                    self.curio_runtime
                        .teardown_operations(
                            curio,
                            CurioRuntimeBindings {
                                inventory: self.curio_inventory,
                                state_slot: self.curio_state_slot,
                                charge_slot: self.curio_charge_slot,
                            },
                        )
                        .map_err(StandardUniverseCurioCommandError::Curio)?,
                );
                operations.push(ActivityOperation::AddCounter {
                    slot: self.curio_event_slot,
                    key: crate::curio_activity::DESTROYED_CURIO_COUNT_KEY,
                    delta: integer(1),
                });
                operations.push(ActivityOperation::SetSlot {
                    slot: self.technique_points_slot,
                    value: integer(0),
                });
                operations.extend(self.participants.entries().iter().map(|participant| {
                    ActivityOperation::SetParticipantEnergy {
                        participant: participant.participant(),
                        energy: Energy::ZERO,
                    }
                }));
            }
        }
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(CURIO_DESTRUCTIBLE_PROGRAM)
                .expect("static Curio program ID is non-zero"),
            operations,
        )
        .map_err(|_| StandardUniverseCurioCommandError::InvalidProgram)?;
        self.graph
            .apply_boundary_program(expected_state_hash, &program)
            .map_err(StandardUniverseCurioCommandError::Activity)
    }
}

#[derive(Debug)]
pub enum StandardUniverseCurioCommandError {
    NotOwned,
    UnknownBlessing,
    InvalidProgram,
    Curio(CurioRuntimeError),
    Activity(GraphActivityCommandError),
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
