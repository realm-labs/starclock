use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityOptionDefinition,
    ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityRandomBoundaryResolution, ActivityRngLabel, ActivityStateHash, ActivityValue,
    GraphActivityCommandError,
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
const CURIO_BLESSING_ENHANCE_PROGRAM: u32 = 9_700_004;
const CURIO_BLESSING_COUNT_PURPOSE: u16 = 0x7c01;
const CURIO_BLESSING_CHOICE_PURPOSE: u16 = 0x7c02;
const CURIO_ENHANCE_COUNT_PURPOSE: u16 = 0x7c03;
const CURIO_ENHANCE_CHOICE_PURPOSE: u16 = 0x7c04;

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
        let view = self.graph.debug_view();
        let event_values = view
            .all_slots()
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

    pub fn pending_curio_acquisition_enhancements(
        &self,
    ) -> Result<Box<[CurioId]>, StandardUniverseCurioCommandError> {
        self.pending_acquisition_curios(|effect| {
            matches!(effect, CurioEffect::EnhanceRandomBlessings { .. })
        })
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

    /// Resolves an acquisition-time random enhancement from the authoritative
    /// Reward RNG stream. Only owned, unenhanced Blessings are eligible.
    pub fn resolve_curio_acquisition_enhancements(
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
        let count = self
            .curio_effect_runtime
            .execute(curio, CurioEvent::Acquired, CurioEffectFacts::default())
            .map_err(StandardUniverseCurioCommandError::Effect)?
            .iter()
            .find_map(|applied| match applied.effect() {
                CurioEffect::EnhanceRandomBlessings { count } => Some(*count),
                _ => None,
            })
            .ok_or(StandardUniverseCurioCommandError::NoRandomBlessingEnhancement)?;
        let view = self.view();
        let inventory = view
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.blessing_inventory)
            .ok_or(StandardUniverseCurioCommandError::InvalidEventState)?;
        let eligible = self
            .blessing_runtime
            .definitions()
            .iter()
            .filter(|definition| {
                inventory
                    .entries()
                    .binary_search_by_key(&u64::from(definition.blessing().get()), |entry| entry.0)
                    .ok()
                    .is_some_and(|index| inventory.entries()[index].1 == 1)
            })
            .count();
        let selected_count = u16::from(count).min(
            u16::try_from(eligible)
                .map_err(|_| StandardUniverseCurioCommandError::InvalidEventState)?,
        );
        let candidates = self
            .blessing_runtime
            .definitions()
            .iter()
            .enumerate()
            .map(|(priority, definition)| {
                let content = u64::from(definition.blessing().get());
                (
                    ActivityOptionDefinition::new(
                        ActivityOptionId::new(content).expect("Blessing ID is non-zero"),
                        i32::try_from(priority).unwrap_or(i32::MAX),
                        ActivityCondition::Equal(
                            ActivityExpression::InventoryCount {
                                inventory: self.blessing_inventory,
                                content,
                            },
                            integer(1),
                        ),
                        vec![ActivityOperation::AddInventory {
                            inventory: self.blessing_inventory,
                            content,
                            count: integer(1),
                        }],
                    ),
                    1,
                )
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
                ActivityProgramId::new(CURIO_BLESSING_ENHANCE_PROGRAM)
                    .expect("static Curio program ID is non-zero"),
                ActivityRngLabel::Reward,
                CURIO_ENHANCE_COUNT_PURPOSE,
                CURIO_ENHANCE_CHOICE_PURPOSE,
                selected_count,
                selected_count,
                &prefix,
                &candidates,
            )
            .map_err(StandardUniverseCurioCommandError::Activity)
    }

    fn pending_acquisition_curios(
        &self,
        predicate: impl Fn(&CurioEffect) -> bool,
    ) -> Result<Box<[CurioId]>, StandardUniverseCurioCommandError> {
        let view = self.graph.debug_view();
        let event_values = view
            .all_slots()
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
            if effects.iter().any(|effect| predicate(effect.effect())) {
                pending.push(contribution.curio());
            }
        }
        pending.sort_unstable();
        Ok(pending.into_boxed_slice())
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
    NoRandomBlessingEnhancement,
    InvalidEventState,
    InvalidProgram,
    Effect(CurioEffectRuntimeError),
    Curio(CurioRuntimeError),
    Activity(GraphActivityCommandError),
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::Arc;

    use starclock_activity::{
        BattleOutcome, BattleResult, EventDigest, ParticipantBattleState, ProjectedValue,
        ProjectionField,
    };
    use starclock_combat::{BattleStateHash, Hp, LifeState, PresenceState};

    use crate::{
        baseline_runner::{
            StandardUniverseBaselinePolicy, StandardUniverseBaselineRunner,
            StandardUniverseBaselineStep,
        },
        catalog::UniverseCatalog,
        curio_activity::{CurioActivityBindings, acquisition_operations, compile_records},
        production_runtime::{StandardUniverseControllerIdentity, StandardUniverseRuntimeFactory},
    };

    use super::*;

    const CORE: &[u8] = include_bytes!("../../../../config/generated/config.sora");
    const UNIVERSE: &[u8] = include_bytes!("../../../../config/universe-generated/config.sora");

    #[test]
    fn entropic_die_enhances_two_owned_blessings_on_authoritative_reward_rng() {
        let (mut activity, catalog) = activity();
        let blessing_ids = catalog
            .blessings()
            .iter()
            .take(2)
            .map(|blessing| blessing.id())
            .collect::<Vec<_>>();
        acquire_with_blessings(&mut activity, CurioId::new(25).unwrap(), &blessing_ids);
        assert_eq!(
            activity
                .pending_curio_acquisition_enhancements()
                .unwrap()
                .as_ref(),
            &[CurioId::new(25).unwrap()]
        );
        let resolution = activity
            .resolve_curio_acquisition_enhancements(
                activity.view().state_hash(),
                CurioId::new(25).unwrap(),
            )
            .unwrap();
        assert_eq!(resolution.selected_options().len(), 2);
        let inventory = activity
            .view()
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == activity.blessing_inventory)
            .unwrap()
            .entries()
            .to_vec();
        assert!(blessing_ids.iter().all(|blessing| {
            inventory
                .binary_search_by_key(&u64::from(blessing.get()), |entry| entry.0)
                .ok()
                .is_some_and(|index| inventory[index].1 == 2)
        }));
    }

    #[test]
    fn erudition_sealing_wax_grants_one_erudition_blessing() {
        let (mut activity, catalog) = activity();
        let curio = CurioId::new(27).unwrap();
        acquire_with_blessings(&mut activity, curio, &[]);
        assert_eq!(
            activity
                .pending_curio_acquisition_blessings()
                .unwrap()
                .as_ref(),
            &[curio]
        );
        let resolution = activity
            .resolve_curio_acquisition_blessings(activity.view().state_hash(), curio)
            .unwrap();
        assert_eq!(resolution.selected_options().len(), 1);
        let selected = u32::try_from(resolution.selected_options()[0].get())
            .ok()
            .and_then(BlessingId::new)
            .unwrap();
        let erudition = catalog
            .paths()
            .iter()
            .find(|path| path.stable_key() == "universe.path.erudition")
            .unwrap()
            .id();
        assert_eq!(catalog.blessing(selected).unwrap().path(), erudition);
    }

    #[test]
    fn remaining_sealing_waxes_grant_their_authored_path_blessing() {
        for (curio_raw, path_key) in [
            (29, "universe.path.elation"),
            (30, "universe.path.hunt"),
            (31, "universe.path.destruction"),
            (32, "universe.path.remembrance"),
            (33, "universe.path.nihility"),
            (34, "universe.path.abundance"),
        ] {
            let (mut activity, catalog) = activity();
            let curio = CurioId::new(curio_raw).unwrap();
            acquire_with_blessings(&mut activity, curio, &[]);
            let resolution = activity
                .resolve_curio_acquisition_blessings(activity.view().state_hash(), curio)
                .unwrap();
            let selected = u32::try_from(resolution.selected_options()[0].get())
                .ok()
                .and_then(BlessingId::new)
                .unwrap();
            let expected = catalog
                .paths()
                .iter()
                .find(|path| path.stable_key() == path_key)
                .unwrap()
                .id();
            assert_eq!(catalog.blessing(selected).unwrap().path(), expected);
        }
    }

    #[test]
    fn alien_tree_revives_defeated_carry_and_destroys_itself_after_victory() {
        let (mut activity, _) = activity();
        let fruit = CurioId::new(36).unwrap();
        acquire_with_blessings(&mut activity, fruit, &[]);
        let invoked = Cell::new(false);
        let mut executor = |handoff: &starclock_activity::ActivityBattleHandoff| {
            invoked.set(true);
            Ok(defeated_victory(handoff))
        };
        let runner = StandardUniverseBaselineRunner::default();
        for _ in 0..64 {
            let step = runner
                .advance(
                    &mut activity,
                    &StandardUniverseBaselinePolicy::default(),
                    &mut executor,
                )
                .unwrap();
            if matches!(step, StandardUniverseBaselineStep::Battle { .. }) {
                break;
            }
        }
        assert!(invoked.get());
        let view = activity.view();
        let restored = view
            .participant_carry()
            .iter()
            .find(|state| state.participant().get() == 1)
            .unwrap();
        assert_eq!(restored.life(), LifeState::Alive);
        assert_eq!(restored.current_hp(), restored.maximum_hp());
        assert!(
            activity
                .curio_contributions()
                .unwrap()
                .entries()
                .iter()
                .all(|entry| entry.curio() != fruit)
        );
    }

    fn acquire_with_blessings(
        activity: &mut StandardUniverseActivity,
        curio: CurioId,
        blessings: &[BlessingId],
    ) {
        let record = compile_records(&activity.curio_runtime)
            .unwrap()
            .iter()
            .copied()
            .find(|record| record.id() == curio)
            .unwrap();
        let mut operations = blessings
            .iter()
            .map(|blessing| ActivityOperation::AddInventory {
                inventory: activity.blessing_inventory,
                content: u64::from(blessing.get()),
                count: integer(1),
            })
            .collect::<Vec<_>>();
        operations.extend(acquisition_operations(
            record,
            CurioActivityBindings {
                inventory: activity.curio_inventory,
                state_slot: activity.curio_state_slot,
                charge_slot: activity.curio_charge_slot,
                event_slot: activity.curio_event_slot,
                fragments_slot: activity.cosmic_fragments_slot,
            },
        ));
        let program =
            ActivityProgramDefinition::new(ActivityProgramId::new(9_700_090).unwrap(), operations)
                .unwrap();
        activity
            .graph
            .apply_boundary_program(activity.view().state_hash(), &program)
            .unwrap();
    }

    fn defeated_victory(handoff: &starclock_activity::ActivityBattleHandoff) -> BattleResult {
        let values = handoff
            .projection()
            .fields()
            .iter()
            .map(|field| match field {
                ProjectionField::Outcome => ProjectedValue::Outcome(BattleOutcome::Won),
                ProjectionField::FinalStateHash => {
                    ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x71; 32]))
                }
                ProjectionField::EventDigest => {
                    ProjectedValue::EventDigest(EventDigest::new([0x72; 32]).unwrap())
                }
                ProjectionField::TerminalFault => ProjectedValue::TerminalFault(None),
                ProjectionField::ParticipantState(participant) => {
                    let defeated = participant.get() == 1;
                    let carry = handoff
                        .participant_carry()
                        .iter()
                        .find(|carry| carry.participant() == *participant)
                        .unwrap();
                    ProjectedValue::ParticipantState(
                        ParticipantBattleState::new(
                            *participant,
                            if defeated {
                                Hp::new(0).unwrap()
                            } else {
                                carry.maximum_hp()
                            },
                            carry.maximum_hp(),
                            carry.current_energy(),
                            carry.maximum_energy(),
                            if defeated {
                                LifeState::Defeated
                            } else {
                                LifeState::Alive
                            },
                            PresenceState::Present,
                        )
                        .unwrap(),
                    )
                }
                ProjectionField::Metric { .. } => panic!("fixture has no metric projection"),
            })
            .collect();
        BattleResult::seal(handoff.identity(), values)
    }

    fn activity() -> (StandardUniverseActivity, Arc<UniverseCatalog>) {
        let factory = StandardUniverseRuntimeFactory::load(CORE, UNIVERSE).unwrap();
        let catalog = Arc::clone(factory.catalog());
        let world = catalog.worlds()[0].id().get();
        let instance = factory
            .start(
                world,
                0,
                9_700_090,
                StandardUniverseControllerIdentity {
                    id: "goal07-test",
                    revision: "v1",
                    digest: [0x70; 32],
                },
            )
            .unwrap();
        let (_, activity, _, _, _) = instance.into_dynamic_parts();
        (activity, catalog)
    }
}
