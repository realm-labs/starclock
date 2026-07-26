use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityRandomBoundaryResolution,
    ActivityRngLabel, ActivityStateHash, ActivityValue, GraphActivityCommandError,
};
use starclock_combat::Energy;

use crate::{
    curio_effect_runtime::{CurioEffect, CurioEffectFacts, CurioEffectRuntimeError, CurioEvent},
    curio_runtime::{CurioRuntimeBindings, CurioRuntimeError},
    id::{BlessingId, CurioId},
    run_runtime::RunRuntimeError,
};

use super::StandardUniverseActivity;

const CURIO_DESTRUCTIBLE_PROGRAM: u32 = 9_700_002;
const CURIO_BLESSING_GRANT_PROGRAM: u32 = 9_700_003;
const CURIO_BLESSING_COUNT_PURPOSE: u16 = 0x7c01;
const CURIO_BLESSING_CHOICE_PURPOSE: u16 = 0x7c02;

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

    pub fn pending_curio_acquisition_blessings(
        &self,
    ) -> Result<Box<[CurioId]>, StandardUniverseCurioCommandError> {
        let view = self.view();
        let event_values = view
            .slots()
            .iter()
            .find(|slot| slot.id() == self.curio_event_slot)
            .and_then(|slot| match slot.value() {
                ActivityValue::BoundedCounterMap(values) => Some(values),
                _ => None,
            })
            .ok_or(StandardUniverseCurioCommandError::InvalidEventState)?;
        let mut pending = Vec::new();
        for contribution in self
            .curio_contributions()
            .map_err(StandardUniverseCurioCommandError::Curio)?
            .entries()
        {
            let key = crate::curio_activity::event_key(contribution.curio(), CurioEvent::Acquired);
            let count = event_values
                .iter()
                .find(|(candidate, _)| *candidate == key)
                .map_or(0, |(_, count)| *count);
            if count <= 0 {
                continue;
            }
            let effects = self
                .curio_effect_runtime
                .execute(
                    contribution.curio(),
                    CurioEvent::Acquired,
                    CurioEffectFacts::default(),
                )
                .map_err(StandardUniverseCurioCommandError::Effect)?;
            if effects
                .iter()
                .any(|effect| matches!(effect.effect(), CurioEffect::GrantRandomBlessings { .. }))
            {
                pending.push(contribution.curio());
            }
        }
        pending.sort_unstable();
        Ok(pending.into_boxed_slice())
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

    /// Resolves the hidden random Blessing grant recorded by an acquired Curio.
    ///
    /// The outcome is sampled by the authoritative Activity reward stream and
    /// committed with the ordinary Blessing acquisition operations. Callers do
    /// not supply random outcomes; replaying this command from the same accepted
    /// state reproduces the same selected IDs and post-command hash.
    pub fn resolve_curio_acquisition_blessings(
        &mut self,
        expected_state_hash: ActivityStateHash,
        curio: CurioId,
    ) -> Result<ActivityRandomBoundaryResolution, StandardUniverseCurioCommandError> {
        if !self
            .curio_contributions()
            .map_err(StandardUniverseCurioCommandError::Curio)?
            .entries()
            .iter()
            .any(|entry| entry.curio() == curio)
        {
            return Err(StandardUniverseCurioCommandError::NotOwned);
        }
        let effects = self
            .curio_effect_runtime
            .execute(curio, CurioEvent::Acquired, CurioEffectFacts::default())
            .map_err(StandardUniverseCurioCommandError::Effect)?;
        let (path, minimum, maximum) = effects
            .iter()
            .find_map(|applied| match applied.effect() {
                CurioEffect::GrantRandomBlessings {
                    path,
                    minimum,
                    maximum,
                } => Some((*path, *minimum, *maximum)),
                _ => None,
            })
            .ok_or(StandardUniverseCurioCommandError::NoRandomBlessingGrant)?;
        let path = path
            .or_else(|| self.selected_path())
            .ok_or(StandardUniverseCurioCommandError::MissingSelectedPath)?;
        let candidates = self
            .blessing_runtime
            .definitions()
            .iter()
            .filter(|definition| definition.path() == path)
            .enumerate()
            .map(|(priority, definition)| {
                let blessing = definition.blessing();
                let option = self
                    .blessing_runtime
                    .acquisition_option(
                        blessing,
                        ActivityOptionId::new(u64::from(blessing.get()))
                            .expect("Blessing ID is non-zero"),
                        i32::try_from(priority).unwrap_or(i32::MAX),
                        self.blessing_inventory,
                        vec![ActivityOperation::AddCounter {
                            slot: self.path_blessing_count_slot,
                            key: u64::from(path.get()),
                            delta: integer(1),
                        }],
                    )
                    .expect("compiled Blessing definition has an acquisition option");
                (option, 1)
            })
            .collect::<Vec<_>>();
        let acquired_key = crate::curio_activity::event_key(curio, CurioEvent::Acquired);
        let prefix = [
            ActivityOperation::Require(ActivityCondition::Not(Box::new(
                ActivityCondition::LessThan(
                    ActivityExpression::CounterValue {
                        slot: self.curio_event_slot,
                        key: acquired_key,
                    },
                    integer(1),
                ),
            ))),
            ActivityOperation::AddCounter {
                slot: self.curio_event_slot,
                key: acquired_key,
                delta: integer(-1),
            },
        ];
        self.graph
            .apply_random_option_boundary(
                expected_state_hash,
                ActivityProgramId::new(CURIO_BLESSING_GRANT_PROGRAM)
                    .expect("static Curio program ID is non-zero"),
                ActivityRngLabel::Reward,
                CURIO_BLESSING_COUNT_PURPOSE,
                CURIO_BLESSING_CHOICE_PURPOSE,
                u16::from(minimum),
                u16::from(maximum),
                &prefix,
                &candidates,
            )
            .map_err(StandardUniverseCurioCommandError::Activity)
    }

    fn selected_path(&self) -> Option<crate::id::PathId> {
        self.view()
            .slots()
            .iter()
            .find(|slot| slot.id() == self.selected_path_slot)
            .and_then(|slot| match slot.value() {
                ActivityValue::OptionalId(Some(value)) => u32::try_from(*value).ok(),
                _ => None,
            })
            .and_then(crate::id::PathId::new)
    }
}

#[derive(Debug)]
pub enum StandardUniverseCurioCommandError {
    NotOwned,
    UnknownBlessing,
    MissingSelectedPath,
    NoRandomBlessingGrant,
    InvalidEventState,
    InvalidProgram,
    Effect(CurioEffectRuntimeError),
    Curio(CurioRuntimeError),
    Activity(GraphActivityCommandError),
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
