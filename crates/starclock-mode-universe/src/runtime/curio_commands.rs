use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityOptionDefinition,
    ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityRandomBoundaryResolution, ActivityRngLabel, ActivityStateHash, ActivityValue,
    GraphActivityCommandError,
};
use starclock_combat::{Energy, Hp, LifeState, Ratio};

use crate::{
    curio_activity::{CurioActivityBindings, acquisition_operations, compile_records},
    curio_effect_runtime::{
        CurioBlessingGrantPool, CurioDestructibleReward, CurioEffect, CurioEffectFacts,
        CurioEffectRuntimeError, CurioEvent,
    },
    curio_runtime::CurioRuntimeError,
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
    Curio(CurioId),
    Failure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurioDestructiblePolicy {
    more_frequent: bool,
    reward_multiplier: u8,
}

impl CurioDestructiblePolicy {
    #[must_use]
    pub const fn more_frequent(self) -> bool {
        self.more_frequent
    }
    #[must_use]
    pub const fn reward_multiplier(self) -> u8 {
        self.reward_multiplier
    }
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
        self.resolve_destructible_event(expected_state_hash, &[(curio, outcome)])
    }

    /// Resolves one destructible-object lottery from an owned Curio.
    ///
    /// Released data does not publish the small-chance probabilities. The
    /// trusted host therefore supplies the closed outcome as a replay command;
    /// this boundary validates the configured reward family and applies the
    /// resulting Activity transaction atomically.
    pub fn resolve_curio_destructible_lottery(
        &mut self,
        expected_state_hash: ActivityStateHash,
        curio: CurioId,
        outcome: CurioDestructibleOutcome,
    ) -> Result<
        Box<[starclock_activity::ActivityTransactionEvent]>,
        StandardUniverseCurioCommandError,
    > {
        self.resolve_destructible_event(expected_state_hash, &[(curio, outcome)])
    }

    /// Commits one physical destructible event and all of its Curio lotteries.
    ///
    /// `lotteries` must contain each triggered Curio at most once in ascending
    /// ID order. This makes simultaneous Lotto Curios one atomic event and
    /// increments the shared destructible count exactly once.
    pub fn resolve_destructible_event(
        &mut self,
        expected_state_hash: ActivityStateHash,
        lotteries: &[(CurioId, CurioDestructibleOutcome)],
    ) -> Result<
        Box<[starclock_activity::ActivityTransactionEvent]>,
        StandardUniverseCurioCommandError,
    > {
        if lotteries.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
            return Err(StandardUniverseCurioCommandError::NonCanonicalOutcomes);
        }
        let mut operations = vec![ActivityOperation::AddCounter {
            slot: self.curio_event_slot,
            key: crate::curio_activity::DESTRUCTIBLE_DESTROYED_COUNT_KEY,
            delta: integer(1),
        }];
        for (curio, outcome) in lotteries {
            operations.extend(self.destructible_lottery_operations(*curio, *outcome)?);
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

    /// Returns the exact destructible-generation/reward policy visible to the
    /// spatial-free encounter host. “More frequent” remains qualitative because
    /// released data publishes no scalar; reward doubling is exact.
    pub fn destructible_policy(
        &self,
    ) -> Result<CurioDestructiblePolicy, StandardUniverseCurioCommandError> {
        let mut policy = CurioDestructiblePolicy {
            more_frequent: false,
            reward_multiplier: 1,
        };
        for contribution in self
            .curio_contributions()
            .map_err(StandardUniverseCurioCommandError::Curio)?
            .entries()
        {
            for applied in self
                .curio_effect_runtime
                .execute(
                    contribution.curio(),
                    CurioEvent::DestructibleDestroyed,
                    CurioEffectFacts::default(),
                )
                .map_err(StandardUniverseCurioCommandError::Effect)?
            {
                if let CurioEffect::ConfigureDestructibles {
                    more_frequent,
                    reward_multiplier,
                } = applied.effect()
                {
                    policy.more_frequent |= *more_frequent;
                    policy.reward_multiplier = policy
                        .reward_multiplier
                        .checked_mul(*reward_multiplier)
                        .ok_or(StandardUniverseCurioCommandError::InvalidEffectValue)?;
                }
            }
        }
        Ok(policy)
    }

    fn destructible_lottery_operations(
        &self,
        curio: CurioId,
        outcome: CurioDestructibleOutcome,
    ) -> Result<Vec<ActivityOperation>, StandardUniverseCurioCommandError> {
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
            .execute(
                curio,
                CurioEvent::DestructibleDestroyed,
                CurioEffectFacts::default(),
            )
            .map_err(StandardUniverseCurioCommandError::Effect)?;
        let (reward, hp_loss_ratio, loses_resources) = effects
            .iter()
            .find_map(|applied| match applied.effect() {
                CurioEffect::ConfigureDestructibleLottery {
                    reward,
                    failure_current_hp_loss_ratio,
                    failure_loses_energy_and_technique_points,
                    ..
                } => Some((
                    *reward,
                    failure_current_hp_loss_ratio.raw_six_decimal(),
                    *failure_loses_energy_and_technique_points,
                )),
                _ => None,
            })
            .ok_or(StandardUniverseCurioCommandError::NoDestructibleLottery)?;
        let mut operations = vec![ActivityOperation::AddCounter {
            slot: self.curio_event_slot,
            key: crate::curio_activity::event_key(curio, CurioEvent::DestructibleDestroyed),
            delta: integer(1),
        }];
        match outcome {
            CurioDestructibleOutcome::NoEffect => {}
            CurioDestructibleOutcome::Blessing(blessing) => {
                if reward != CurioDestructibleReward::Blessing {
                    return Err(StandardUniverseCurioCommandError::OutcomeMismatch);
                }
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
            CurioDestructibleOutcome::Curio(acquired) => {
                if reward != CurioDestructibleReward::Curio {
                    return Err(StandardUniverseCurioCommandError::OutcomeMismatch);
                }
                let record = compile_records(&self.curio_runtime)
                    .map_err(StandardUniverseCurioCommandError::Curio)?
                    .iter()
                    .copied()
                    .find(|record| record.id() == acquired)
                    .ok_or(StandardUniverseCurioCommandError::UnknownCurio)?;
                operations.extend(acquisition_operations(
                    record,
                    CurioActivityBindings {
                        inventory: self.curio_inventory,
                        state_slot: self.curio_state_slot,
                        charge_slot: self.curio_charge_slot,
                        event_slot: self.curio_event_slot,
                        fragments_slot: self.cosmic_fragments_slot,
                    },
                ));
            }
            CurioDestructibleOutcome::Failure => {
                operations.extend(crate::curio_activity::destroy_and_count_operations(
                    curio,
                    CurioActivityBindings {
                        inventory: self.curio_inventory,
                        state_slot: self.curio_state_slot,
                        charge_slot: self.curio_charge_slot,
                        event_slot: self.curio_event_slot,
                        fragments_slot: self.cosmic_fragments_slot,
                    },
                ));
                if hp_loss_ratio != 0 {
                    if !(1..=1_000_000).contains(&hp_loss_ratio) {
                        return Err(StandardUniverseCurioCommandError::InvalidEffectValue);
                    }
                    let ratio = Ratio::from_scaled(hp_loss_ratio);
                    operations.extend(
                        self.view()
                            .participant_carry()
                            .iter()
                            .filter(|state| state.life() == LifeState::Alive)
                            .map(|state| ActivityOperation::LoseParticipantCurrentHpRatio {
                                participant: state.participant(),
                                hp_ratio: ratio,
                                minimum_hp: Hp::new(1).expect("one HP is valid"),
                            }),
                    );
                }
                if loses_resources {
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
        }
        Ok(operations)
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
        let (pool, path, minimum, maximum) = effects
            .iter()
            .find_map(|applied| match applied.effect() {
                CurioEffect::GrantRandomBlessings {
                    pool,
                    path,
                    minimum,
                    maximum,
                } => Some((*pool, *path, *minimum, *maximum)),
                _ => None,
            })
            .ok_or(StandardUniverseCurioCommandError::NoRandomBlessingGrant)?;
        let selected_path = match pool {
            CurioBlessingGrantPool::AllEligible => None,
            CurioBlessingGrantPool::SelectedPath => Some(
                self.selected_path()
                    .ok_or(StandardUniverseCurioCommandError::MissingSelectedPath)?,
            ),
            CurioBlessingGrantPool::AuthoredPath => {
                Some(path.ok_or(StandardUniverseCurioCommandError::NoRandomBlessingGrant)?)
            }
        };
        let candidates = self
            .blessing_runtime
            .definitions()
            .iter()
            .filter(|definition| selected_path.is_none_or(|path| definition.path() == path))
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
                            key: u64::from(definition.path().get()),
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
    NoDestructibleLottery,
    OutcomeMismatch,
    UnknownCurio,
    InvalidEffectValue,
    NonCanonicalOutcomes,
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
            NestedBattleExecutionError, StandardUniverseBaselinePolicy,
            StandardUniverseBaselineRunner, StandardUniverseBaselineStep,
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
    fn casket_samples_one_or_two_blessings_from_the_complete_eligible_pool() {
        let (mut activity, catalog) = activity();
        let casket = CurioId::new(40).unwrap();
        acquire_with_blessings(&mut activity, casket, &[]);
        let resolution = activity
            .resolve_curio_acquisition_blessings(activity.view().state_hash(), casket)
            .unwrap();
        assert!((1..=2).contains(&resolution.selected_options().len()));
        assert!(resolution.selected_options().iter().all(|option| {
            let blessing = u32::try_from(option.get())
                .ok()
                .and_then(BlessingId::new)
                .unwrap();
            catalog.blessing(blessing).is_some()
        }));
    }

    #[test]
    fn destructible_event_counts_once_and_capsule_exposes_exact_spatial_free_policy() {
        let (mut activity, _) = activity();
        let wick = CurioId::new(45).unwrap();
        let capsule = CurioId::new(52).unwrap();
        acquire_with_blessings(&mut activity, wick, &[]);
        acquire_with_blessings(&mut activity, capsule, &[]);
        let policy = activity.destructible_policy().unwrap();
        assert!(policy.more_frequent());
        assert_eq!(policy.reward_multiplier(), 2);
        activity
            .resolve_destructible_event(activity.view().state_hash(), &[])
            .unwrap();
        assert_eq!(
            activity
                .curio_contributions()
                .unwrap()
                .destructibles_destroyed(),
            1
        );
    }

    #[test]
    fn cosmic_lotto_failure_destroys_itself_and_leaves_living_allies_at_one_percent() {
        let (mut activity, _) = activity();
        let lotto = CurioId::new(51).unwrap();
        acquire_with_blessings(&mut activity, lotto, &[]);
        activity
            .resolve_curio_destructible_lottery(
                activity.view().state_hash(),
                lotto,
                CurioDestructibleOutcome::Failure,
            )
            .unwrap();
        assert!(
            activity
                .curio_contributions()
                .unwrap()
                .entries()
                .iter()
                .all(|entry| entry.curio() != lotto)
        );
        assert!(activity.view().participant_carry().iter().all(|state| {
            state.life() != LifeState::Alive
                || state.current_hp().get() == state.maximum_hp().get() / 100
        }));
    }

    #[test]
    fn cosmic_lotto_curio_reward_uses_the_ordinary_acquisition_lifecycle() {
        let (mut activity, _) = activity();
        let lotto = CurioId::new(51).unwrap();
        let reward = CurioId::new(40).unwrap();
        acquire_with_blessings(&mut activity, lotto, &[]);
        activity
            .resolve_curio_destructible_lottery(
                activity.view().state_hash(),
                lotto,
                CurioDestructibleOutcome::Curio(reward),
            )
            .unwrap();
        assert!(
            activity
                .curio_contributions()
                .unwrap()
                .entries()
                .iter()
                .any(|entry| entry.curio() == reward)
        );
    }

    #[test]
    fn ambergris_cheese_heals_thirty_percent_of_maximum_hp_after_victory() {
        let (mut activity, _) = activity();
        acquire_with_blessings(&mut activity, CurioId::new(47).unwrap(), &[]);
        let mut executor = |handoff: &starclock_activity::ActivityBattleHandoff| {
            Ok(alive_result(handoff, BattleOutcome::Won, 1, 2))
        };
        run_until_battle(&mut activity, &mut executor);
        assert!(
            activity
                .view()
                .participant_carry()
                .iter()
                .all(|state| { state.current_hp().get() == state.maximum_hp().get() * 4 / 5 })
        );
    }

    #[test]
    fn laurel_crown_converts_non_boss_defeat_to_full_restore_and_destroys_itself() {
        let (mut activity, _) = activity();
        let crown = CurioId::new(49).unwrap();
        acquire_with_blessings(&mut activity, crown, &[]);
        let mut executor = |handoff: &starclock_activity::ActivityBattleHandoff| {
            Ok(alive_result(handoff, BattleOutcome::Lost, 0, 1))
        };
        run_until_battle(&mut activity, &mut executor);
        assert!(activity.view().participant_carry().iter().all(|state| {
            state.life() == LifeState::Alive && state.current_hp() == state.maximum_hp()
        }));
        assert!(
            activity
                .curio_contributions()
                .unwrap()
                .entries()
                .iter()
                .all(|entry| entry.curio() != crown)
        );
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

    fn alive_result(
        handoff: &starclock_activity::ActivityBattleHandoff,
        outcome: BattleOutcome,
        numerator: i64,
        denominator: i64,
    ) -> BattleResult {
        let values = handoff
            .projection()
            .fields()
            .iter()
            .map(|field| match field {
                ProjectionField::Outcome => ProjectedValue::Outcome(outcome),
                ProjectionField::FinalStateHash => {
                    ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x73; 32]))
                }
                ProjectionField::EventDigest => {
                    ProjectedValue::EventDigest(EventDigest::new([0x74; 32]).unwrap())
                }
                ProjectionField::TerminalFault => ProjectedValue::TerminalFault(None),
                ProjectionField::ParticipantState(participant) => {
                    let carry = handoff
                        .participant_carry()
                        .iter()
                        .find(|carry| carry.participant() == *participant)
                        .unwrap();
                    let hp = if numerator == 0 {
                        Hp::new(0).unwrap()
                    } else {
                        Hp::new(carry.maximum_hp().get() * numerator / denominator).unwrap()
                    };
                    ProjectedValue::ParticipantState(
                        ParticipantBattleState::new(
                            *participant,
                            hp,
                            carry.maximum_hp(),
                            carry.current_energy(),
                            carry.maximum_energy(),
                            if numerator == 0 {
                                LifeState::Defeated
                            } else {
                                LifeState::Alive
                            },
                            if numerator == 0 {
                                PresenceState::Departed
                            } else {
                                PresenceState::Present
                            },
                        )
                        .unwrap(),
                    )
                }
                ProjectionField::Metric { key, kind } => ProjectedValue::Metric {
                    key: key.clone(),
                    value: match kind {
                        starclock_activity::MetricValueKind::BoundedInteger => {
                            starclock_activity::MetricValue::BoundedInteger(0)
                        }
                        starclock_activity::MetricValueKind::FixedScalar => {
                            starclock_activity::MetricValue::FixedScalar(0)
                        }
                        starclock_activity::MetricValueKind::Ratio => {
                            starclock_activity::MetricValue::Ratio(0)
                        }
                        starclock_activity::MetricValueKind::Probability => {
                            starclock_activity::MetricValue::Probability(0)
                        }
                        starclock_activity::MetricValueKind::ActionValue => {
                            starclock_activity::MetricValue::ActionValue(0)
                        }
                    },
                },
            })
            .collect();
        BattleResult::seal(handoff.identity(), values)
    }

    fn run_until_battle(
        activity: &mut StandardUniverseActivity,
        executor: &mut impl FnMut(
            &starclock_activity::ActivityBattleHandoff,
        ) -> Result<BattleResult, NestedBattleExecutionError>,
    ) {
        let runner = StandardUniverseBaselineRunner::default();
        for _ in 0..64 {
            let step = runner
                .advance(
                    activity,
                    &StandardUniverseBaselinePolicy::default(),
                    executor,
                )
                .unwrap();
            if matches!(step, StandardUniverseBaselineStep::Battle { .. }) {
                return;
            }
        }
        panic!("baseline runner did not reach a battle");
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
