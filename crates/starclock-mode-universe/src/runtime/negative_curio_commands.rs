//! Replayable Activity commands for negative Curio lifecycle effects.

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityOptionDefinition,
    ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityRandomBoundaryResolution, ActivityRngLabel, ActivityStateHash,
    ActivityTransactionEvent, ActivityValue, GraphActivityCommandError,
};

use crate::{
    curio_activity::{
        CurioActivityBindings, acquisition_operations, compile_records, event_key,
        negative::{
            FISSION_EXTRA_COPY_KEY, destroyed_available, destroyed_curios,
            restore_destroyed_operations,
        },
        teardown_operations,
    },
    curio_effect_runtime::{CurioEffect, CurioEvent},
    curio_runtime::{CurioContribution, CurioRuntimeError},
    id::{BlessingId, CurioId},
    negative_curio_runtime::{NegativeCurioEvent, NegativeCurioRuntimeError},
};

use super::StandardUniverseActivity;

const REPAIR_PROGRAM: u32 = 9_700_005;
const REPLACE_CURIO_PROGRAM: u32 = 9_700_006;
const REPLACE_BLESSING_PROGRAM: u32 = 9_700_007;
const FISSION_PROGRAM: u32 = 9_700_008;
const REPAIR_COUNT_PURPOSE: u16 = 0x7c05;
const REPAIR_CHOICE_PURPOSE: u16 = 0x7c06;
const REPLACE_COUNT_PURPOSE: u16 = 0x7c07;
const REPLACE_CHOICE_PURPOSE: u16 = 0x7c08;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioFissionOutcome {
    NoSplit,
    Split,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlessingReplacement {
    removed: BlessingId,
    acquired: BlessingId,
}

impl BlessingReplacement {
    #[must_use]
    pub const fn new(removed: BlessingId, acquired: BlessingId) -> Self {
        Self { removed, acquired }
    }
    #[must_use]
    pub const fn removed(self) -> BlessingId {
        self.removed
    }
    #[must_use]
    pub const fn acquired(self) -> BlessingId {
        self.acquired
    }
}

impl StandardUniverseActivity {
    pub(crate) const fn curio_activity_bindings(&self) -> CurioActivityBindings {
        CurioActivityBindings {
            inventory: self.curio_inventory,
            state_slot: self.curio_state_slot,
            charge_slot: self.curio_charge_slot,
            event_slot: self.curio_event_slot,
            fragments_slot: self.cosmic_fragments_slot,
        }
    }

    /// Lists owned Curios whose acquisition-time negative effect is unresolved.
    pub fn pending_negative_curio_acquisitions(
        &self,
    ) -> Result<Box<[CurioId]>, StandardUniverseNegativeCurioCommandError> {
        let events = self.curio_event_values()?;
        let contributions = self
            .curio_contributions()
            .map_err(StandardUniverseNegativeCurioCommandError::Curio)?;
        let mut pending = Vec::new();
        for contribution in contributions.entries() {
            if event_count(
                &events,
                event_key(contribution.curio(), CurioEvent::Acquired),
            ) <= 0
            {
                continue;
            }
            let effects = self
                .negative_curio_runtime
                .execute(contribution, NegativeCurioEvent::Acquired)
                .map_err(StandardUniverseNegativeCurioCommandError::Effect)?;
            if !effects.is_empty() {
                pending.push(contribution.curio());
            }
        }
        pending.sort_unstable();
        Ok(pending.into_boxed_slice())
    }

    /// Repairs up to the released maximum of destroyed Curios on Reward RNG.
    pub fn resolve_void_wick_trimmer(
        &mut self,
        expected_state_hash: ActivityStateHash,
        curio: CurioId,
    ) -> Result<ActivityRandomBoundaryResolution, StandardUniverseNegativeCurioCommandError> {
        let contribution = self.owned_curio(curio)?;
        let maximum = self
            .negative_curio_runtime
            .execute(&contribution, NegativeCurioEvent::Acquired)
            .map_err(StandardUniverseNegativeCurioCommandError::Effect)?
            .iter()
            .find_map(|effect| match effect.effect() {
                CurioEffect::RepairRandomDestroyedCurios { maximum, .. } => Some(*maximum),
                _ => None,
            })
            .ok_or(StandardUniverseNegativeCurioCommandError::EffectMismatch)?;
        let events = self.curio_event_values()?;
        let destroyed = destroyed_curios(&ActivityValue::BoundedCounterMap(events))
            .ok_or(StandardUniverseNegativeCurioCommandError::InvalidEventState)?;
        let records = compile_records(&self.curio_runtime)
            .map_err(StandardUniverseNegativeCurioCommandError::Curio)?;
        let bindings = self.curio_activity_bindings();
        let candidates = destroyed
            .iter()
            .filter_map(|(id, _)| {
                let record = records.iter().copied().find(|record| record.id() == *id)?;
                Some((
                    ActivityOptionDefinition::new(
                        ActivityOptionId::new(u64::from(id.get()))?,
                        i32::try_from(id.get()).ok()?,
                        destroyed_available(*id, bindings),
                        restore_destroyed_operations(record, bindings),
                    ),
                    1,
                ))
            })
            .collect::<Vec<_>>();
        let count = u16::try_from(candidates.len())
            .unwrap_or(u16::MAX)
            .min(u16::from(maximum));
        if candidates.is_empty() {
            return self.resolve_empty_acquisition_event(expected_state_hash, curio);
        }
        self.graph
            .apply_random_option_boundary(
                expected_state_hash,
                program(REPAIR_PROGRAM),
                ActivityRngLabel::Reward,
                REPAIR_COUNT_PURPOSE,
                REPAIR_CHOICE_PURPOSE,
                count,
                count,
                &consume_acquired_event(curio, self.curio_event_slot),
                &candidates,
            )
            .map_err(StandardUniverseNegativeCurioCommandError::Activity)
    }

    /// Replaces every currently owned Curio through the authoritative Reward RNG.
    pub fn resolve_shining_trapezohedron_die(
        &mut self,
        expected_state_hash: ActivityStateHash,
        curio: CurioId,
    ) -> Result<ActivityRandomBoundaryResolution, StandardUniverseNegativeCurioCommandError> {
        let contribution = self.owned_curio(curio)?;
        let valid = self
            .negative_curio_runtime
            .execute(&contribution, NegativeCurioEvent::Acquired)
            .map_err(StandardUniverseNegativeCurioCommandError::Effect)?
            .iter()
            .any(|effect| {
                matches!(
                    effect.effect(),
                    CurioEffect::ReplaceAllOwnedCuriosRandomly {
                        include_source: true
                    }
                )
            });
        if !valid {
            return Err(StandardUniverseNegativeCurioCommandError::EffectMismatch);
        }
        let owned = self
            .curio_contributions()
            .map_err(StandardUniverseNegativeCurioCommandError::Curio)?;
        let records = compile_records(&self.curio_runtime)
            .map_err(StandardUniverseNegativeCurioCommandError::Curio)?;
        let bindings = self.curio_activity_bindings();
        let candidates = records
            .iter()
            .filter(|record| {
                owned
                    .entries()
                    .iter()
                    .all(|entry| entry.curio() != record.id())
            })
            .map(|record| {
                (
                    ActivityOptionDefinition::new(
                        ActivityOptionId::new(u64::from(record.id().get()))
                            .expect("Curio ID is non-zero"),
                        i32::try_from(record.id().get()).unwrap_or(i32::MAX),
                        available(record.id(), bindings),
                        acquisition_operations(*record, bindings),
                    ),
                    1,
                )
            })
            .collect::<Vec<_>>();
        let count = u16::try_from(owned.entries().len())
            .map_err(|_| StandardUniverseNegativeCurioCommandError::InvalidEffectValue)?;
        let mut prefix = consume_acquired_event(curio, self.curio_event_slot).to_vec();
        for entry in owned.entries() {
            prefix.extend(teardown_operations(entry.curio(), bindings));
        }
        self.graph
            .apply_random_option_boundary(
                expected_state_hash,
                program(REPLACE_CURIO_PROGRAM),
                ActivityRngLabel::Reward,
                REPLACE_COUNT_PURPOSE,
                REPLACE_CHOICE_PURPOSE,
                count,
                count,
                &prefix,
                &candidates,
            )
            .map_err(StandardUniverseNegativeCurioCommandError::Activity)
    }

    /// Applies the replay-recorded Fool's Mask mapping atomically.
    ///
    /// Released data does not publish the higher-rarity probability. The host
    /// records the complete same-or-higher-rarity mapping instead of inventing
    /// a distribution; enhancement levels are retained exactly.
    pub fn resolve_fools_mask(
        &mut self,
        expected_state_hash: ActivityStateHash,
        curio: CurioId,
        replacements: &[BlessingReplacement],
    ) -> Result<Box<[ActivityTransactionEvent]>, StandardUniverseNegativeCurioCommandError> {
        let contribution = self.owned_curio(curio)?;
        let valid = self
            .negative_curio_runtime
            .execute(&contribution, NegativeCurioEvent::Acquired)
            .map_err(StandardUniverseNegativeCurioCommandError::Effect)?
            .iter()
            .any(|effect| {
                matches!(
                    effect.effect(),
                    CurioEffect::ReplaceAllBlessingsRandomly {
                        retain_enhancement: true,
                        released_higher_rarity_chance: true
                    }
                )
            });
        if !valid {
            return Err(StandardUniverseNegativeCurioCommandError::EffectMismatch);
        }
        if replacements
            .windows(2)
            .any(|pair| pair[0].removed >= pair[1].removed)
            || replacements
                .iter()
                .map(|entry| entry.acquired)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != replacements.len()
        {
            return Err(StandardUniverseNegativeCurioCommandError::NonCanonicalOutcome);
        }
        let inventory = self
            .view()
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.blessing_inventory)
            .ok_or(StandardUniverseNegativeCurioCommandError::MissingInventory)?
            .entries()
            .to_vec();
        if inventory.len() != replacements.len()
            || !inventory
                .iter()
                .zip(replacements)
                .all(|((id, _), replacement)| *id == u64::from(replacement.removed.get()))
        {
            return Err(StandardUniverseNegativeCurioCommandError::OutcomeMismatch);
        }
        let mut validated = Vec::with_capacity(replacements.len());
        for ((_, level), replacement) in inventory.iter().zip(replacements) {
            let removed = self.blessing_definition(replacement.removed)?;
            let acquired = self.blessing_definition(replacement.acquired)?;
            if acquired.rarity() < removed.rarity() || !(1..=2).contains(level) {
                return Err(StandardUniverseNegativeCurioCommandError::OutcomeMismatch);
            }
            validated.push((
                replacement.removed,
                removed,
                replacement.acquired,
                acquired,
                *level,
            ));
        }
        let mut operations = consume_acquired_event(curio, self.curio_event_slot).to_vec();
        for (removed_id, removed, _, _, level) in &validated {
            operations.extend([
                ActivityOperation::RemoveInventory {
                    inventory: self.blessing_inventory,
                    content: u64::from(removed_id.get()),
                    count: integer(i64::from(*level)),
                },
                ActivityOperation::AddCounter {
                    slot: self.path_blessing_count_slot,
                    key: u64::from(removed.path().get()),
                    delta: integer(-1),
                },
            ]);
        }
        for (_, _, acquired_id, acquired, level) in &validated {
            operations.extend([
                ActivityOperation::AddInventory {
                    inventory: self.blessing_inventory,
                    content: u64::from(acquired_id.get()),
                    count: integer(i64::from(*level)),
                },
                ActivityOperation::AddCounter {
                    slot: self.path_blessing_count_slot,
                    key: u64::from(acquired.path().get()),
                    delta: integer(1),
                },
            ]);
        }
        let definition =
            ActivityProgramDefinition::new(program(REPLACE_BLESSING_PROGRAM), operations)
                .map_err(|_| StandardUniverseNegativeCurioCommandError::InvalidProgram)?;
        self.graph
            .apply_boundary_program(expected_state_hash, &definition)
            .map_err(StandardUniverseNegativeCurioCommandError::Activity)
    }

    /// Resolves one unpublished Fission Cuckoo Clock split check.
    pub fn resolve_fission_cuckoo_clock(
        &mut self,
        expected_state_hash: ActivityStateHash,
        curio: CurioId,
        outcome: CurioFissionOutcome,
    ) -> Result<Box<[ActivityTransactionEvent]>, StandardUniverseNegativeCurioCommandError> {
        let contribution = self.owned_curio(curio)?;
        let maximum = self
            .negative_curio_runtime
            .execute(&contribution, NegativeCurioEvent::BattleWon)
            .map_err(StandardUniverseNegativeCurioCommandError::Effect)?
            .iter()
            .find_map(|effect| match effect.effect() {
                CurioEffect::ConfigureCurioFission {
                    maximum_concurrent_copies,
                    ..
                } => Some(*maximum_concurrent_copies),
                _ => None,
            })
            .ok_or(StandardUniverseNegativeCurioCommandError::EffectMismatch)?;
        let pending_key = event_key(curio, CurioEvent::BattleWon);
        let extra = crate::curio_activity::negative::fission_extra_copies(
            &ActivityValue::BoundedCounterMap(self.curio_event_values()?),
        )
        .ok_or(StandardUniverseNegativeCurioCommandError::InvalidEventState)?;
        let mut operations = vec![
            ActivityOperation::Require(ActivityCondition::LessThan(
                integer(0),
                ActivityExpression::CounterValue {
                    slot: self.curio_event_slot,
                    key: pending_key,
                },
            )),
            ActivityOperation::AddCounter {
                slot: self.curio_event_slot,
                key: pending_key,
                delta: integer(-1),
            },
        ];
        if outcome == CurioFissionOutcome::Split {
            if extra.saturating_add(1) >= maximum {
                return Err(StandardUniverseNegativeCurioCommandError::CopyCapReached);
            }
            operations.push(ActivityOperation::AddCounter {
                slot: self.curio_event_slot,
                key: FISSION_EXTRA_COPY_KEY,
                delta: integer(1),
            });
        }
        let definition = ActivityProgramDefinition::new(program(FISSION_PROGRAM), operations)
            .map_err(|_| StandardUniverseNegativeCurioCommandError::InvalidProgram)?;
        self.graph
            .apply_boundary_program(expected_state_hash, &definition)
            .map_err(StandardUniverseNegativeCurioCommandError::Activity)
    }

    fn owned_curio(
        &self,
        curio: CurioId,
    ) -> Result<CurioContribution, StandardUniverseNegativeCurioCommandError> {
        self.curio_contributions()
            .map_err(StandardUniverseNegativeCurioCommandError::Curio)?
            .entries()
            .iter()
            .find(|entry| entry.curio() == curio)
            .cloned()
            .ok_or(StandardUniverseNegativeCurioCommandError::NotOwned)
    }

    fn curio_event_values(
        &self,
    ) -> Result<Box<[(u64, i64)]>, StandardUniverseNegativeCurioCommandError> {
        self.graph
            .debug_view()
            .all_slots()
            .iter()
            .find(|slot| slot.id() == self.curio_event_slot)
            .and_then(|slot| match slot.value() {
                ActivityValue::BoundedCounterMap(values) => Some(values.clone()),
                _ => None,
            })
            .ok_or(StandardUniverseNegativeCurioCommandError::InvalidEventState)
    }

    fn blessing_definition(
        &self,
        id: BlessingId,
    ) -> Result<
        &crate::blessing_runtime::BlessingRuntimeDefinition,
        StandardUniverseNegativeCurioCommandError,
    > {
        self.blessing_runtime
            .definitions()
            .iter()
            .find(|definition| definition.blessing() == id)
            .ok_or(StandardUniverseNegativeCurioCommandError::UnknownBlessing)
    }

    fn resolve_empty_acquisition_event(
        &mut self,
        expected_state_hash: ActivityStateHash,
        curio: CurioId,
    ) -> Result<ActivityRandomBoundaryResolution, StandardUniverseNegativeCurioCommandError> {
        let candidates = [(
            ActivityOptionDefinition::new(
                ActivityOptionId::new(1).expect("dummy option ID is non-zero"),
                0,
                ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(
                    false,
                ))),
                Vec::new(),
            ),
            1,
        )];
        self.graph
            .apply_random_option_boundary(
                expected_state_hash,
                program(REPAIR_PROGRAM),
                ActivityRngLabel::Reward,
                REPAIR_COUNT_PURPOSE,
                REPAIR_CHOICE_PURPOSE,
                0,
                0,
                &consume_acquired_event(curio, self.curio_event_slot),
                &candidates,
            )
            .map_err(StandardUniverseNegativeCurioCommandError::Activity)
    }
}

fn consume_acquired_event(
    curio: CurioId,
    slot: starclock_activity::ActivitySlotId,
) -> [ActivityOperation; 2] {
    let key = event_key(curio, CurioEvent::Acquired);
    [
        ActivityOperation::Require(ActivityCondition::LessThan(
            integer(0),
            ActivityExpression::CounterValue { slot, key },
        )),
        ActivityOperation::AddCounter {
            slot,
            key,
            delta: integer(-1),
        },
    ]
}

fn available(id: CurioId, bindings: CurioActivityBindings) -> ActivityCondition {
    ActivityCondition::LessThan(
        ActivityExpression::InventoryCount {
            inventory: bindings.inventory,
            content: u64::from(id.get()),
        },
        integer(1),
    )
}

fn event_count(entries: &[(u64, i64)], key: u64) -> i64 {
    entries
        .iter()
        .find(|entry| entry.0 == key)
        .map_or(0, |entry| entry.1)
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn program(raw: u32) -> ActivityProgramId {
    ActivityProgramId::new(raw).expect("static negative Curio program ID is non-zero")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseNegativeCurioCommandError {
    NotOwned,
    MissingInventory,
    UnknownBlessing,
    EffectMismatch,
    OutcomeMismatch,
    NonCanonicalOutcome,
    InvalidEffectValue,
    InvalidEventState,
    InvalidProgram,
    CopyCapReached,
    Curio(CurioRuntimeError),
    Effect(NegativeCurioRuntimeError),
    Activity(GraphActivityCommandError),
}

#[cfg(test)]
mod tests {
    use starclock_activity::{
        ActivityExternalOutcomeId, BattleOutcome, BattleResult, EventDigest,
        ParticipantBattleState, ProjectedValue, ProjectionField,
    };
    use starclock_combat::{BattleStateHash, LifeState, PresenceState};

    use crate::{
        baseline_runner::{
            StandardUniverseBaselineError, StandardUniverseBaselinePolicy,
            StandardUniverseBaselineRunner, StandardUniverseBaselineStep,
        },
        curio::CurioStateKind,
        production_runtime::{StandardUniverseControllerIdentity, StandardUniverseRuntimeFactory},
    };

    use super::*;

    const CORE: &[u8] = include_bytes!("../../../../config/generated/config.sora");
    const UNIVERSE: &[u8] = include_bytes!("../../../../config/universe-generated/config.sora");
    const TEST_PROGRAM: u32 = 9_700_090;

    #[test]
    fn void_wick_repairs_two_tracked_destroyed_curios_on_reward_rng() {
        let mut activity = activity();
        let first = curio(&activity, "universe.curio.6");
        let second = curio(&activity, "universe.curio.8");
        let trimmer = curio(&activity, "universe.curio.17");
        acquire_curios(&mut activity, &[first, second, trimmer]);
        destroy_curios(&mut activity, &[first, second]);
        assert_eq!(
            activity.curio_contributions().unwrap().destroyed_curios(),
            2
        );

        let resolution = activity
            .resolve_void_wick_trimmer(activity.view().state_hash(), trimmer)
            .unwrap();
        assert_eq!(resolution.selected_options().len(), 2);
        let contributions = activity.curio_contributions().unwrap();
        assert_eq!(contributions.destroyed_curios(), 0);
        assert!(
            contributions
                .entries()
                .iter()
                .any(|entry| entry.curio() == first)
        );
        assert!(
            contributions
                .entries()
                .iter()
                .any(|entry| entry.curio() == second)
        );
    }

    #[test]
    fn shining_die_replaces_every_owned_curio_in_one_atomic_random_boundary() {
        let mut activity = activity();
        let old = [
            curio(&activity, "universe.curio.6"),
            curio(&activity, "universe.curio.8"),
            curio(&activity, "universe.curio.21"),
        ];
        acquire_curios(&mut activity, &old);
        let resolution = activity
            .resolve_shining_trapezohedron_die(activity.view().state_hash(), old[2])
            .unwrap();
        assert_eq!(resolution.selected_options().len(), old.len());
        let current = activity.curio_contributions().unwrap();
        assert_eq!(current.entries().len(), old.len());
        assert!(
            old.iter()
                .all(|old| current.entries().iter().all(|entry| entry.curio() != *old))
        );
        assert_eq!(current.destroyed_curios(), 0);
    }

    #[test]
    fn fools_mask_preserves_enhancement_levels_and_validates_complete_mapping() {
        let mut activity = activity();
        let mut pair = activity
            .blessing_runtime
            .definitions()
            .iter()
            .filter(|definition| definition.rarity() == 1)
            .take(2)
            .map(|definition| (definition.blessing(), definition.path()))
            .collect::<Vec<_>>();
        pair.sort_unstable_by_key(|entry| entry.0);
        acquire_blessings(&mut activity, &[(pair[0], 1), (pair[1], 2)]);
        let mask = curio(&activity, "universe.curio.115");
        acquire_curios(&mut activity, &[mask]);
        let replacements = [
            BlessingReplacement::new(pair[0].0, pair[1].0),
            BlessingReplacement::new(pair[1].0, pair[0].0),
        ];
        activity
            .resolve_fools_mask(activity.view().state_hash(), mask, &replacements)
            .unwrap();
        let entries = activity
            .view()
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == activity.blessing_inventory)
            .unwrap()
            .entries()
            .to_vec();
        assert_eq!(
            entries,
            vec![
                (u64::from(pair[0].0.get()), 2),
                (u64::from(pair[1].0.get()), 1),
            ]
        );
    }

    #[test]
    fn fission_outcome_is_replayable_and_enforces_three_copy_cap() {
        let mut activity = activity();
        let fission = curio(&activity, "universe.curio.108");
        acquire_curios(&mut activity, &[fission]);
        record_battle_wins(&mut activity, fission, 3);
        for _ in 0..2 {
            activity
                .resolve_fission_cuckoo_clock(
                    activity.view().state_hash(),
                    fission,
                    CurioFissionOutcome::Split,
                )
                .unwrap();
        }
        assert_eq!(
            activity
                .curio_contributions()
                .unwrap()
                .runtime_value(FISSION_EXTRA_COPY_KEY),
            Some(2)
        );
        assert_eq!(
            activity.resolve_fission_cuckoo_clock(
                activity.view().state_hash(),
                fission,
                CurioFissionOutcome::Split,
            ),
            Err(StandardUniverseNegativeCurioCommandError::CopyCapReached)
        );
        activity
            .resolve_fission_cuckoo_clock(
                activity.view().state_hash(),
                fission,
                CurioFissionOutcome::NoSplit,
            )
            .unwrap();
    }

    #[test]
    fn repairing_codes_transition_only_after_three_won_battles() {
        let mut activity = activity();
        let energy = curio(&activity, "universe.curio.45");
        let hp = curio(&activity, "universe.curio.47");
        acquire_curios(&mut activity, &[energy, hp]);
        for remaining in [2, 1] {
            assert!(run_one_battle(&mut activity));
            let contributions = activity.curio_contributions().unwrap();
            assert!(contributions.entries().iter().all(|entry| {
                entry.state().kind() == CurioStateKind::Repairing
                    && entry.state().charge().unwrap().remaining() == remaining
            }));
        }
        assert!(run_one_battle(&mut activity));
        assert!(
            activity
                .curio_contributions()
                .unwrap()
                .entries()
                .iter()
                .all(|entry| entry.state().kind() == CurioStateKind::Fixed)
        );
    }

    #[test]
    fn iou_dispenser_suppresses_five_battle_rewards_then_doubles_and_destroys() {
        let mut activity = activity();
        let debt = curio(&activity, "universe.curio.60");
        let full_hp_reward = curio(&activity, "universe.curio.106");
        acquire_curios(&mut activity, &[debt, full_hp_reward]);
        let initial = activity.cosmic_fragments().unwrap().get();
        for completed in 0..4 {
            if !run_one_battle(&mut activity) {
                activity = fresh_activity();
                acquire_curios(&mut activity, &[debt, full_hp_reward]);
                record_battle_wins(&mut activity, debt, completed);
                assert!(run_one_battle(&mut activity));
            }
            assert_eq!(activity.cosmic_fragments().unwrap().get(), initial);
            assert!(
                activity
                    .curio_contributions()
                    .unwrap()
                    .entries()
                    .iter()
                    .any(|entry| entry.curio() == debt)
            );
        }
        if !run_one_battle(&mut activity) {
            activity = fresh_activity();
            acquire_curios(&mut activity, &[debt, full_hp_reward]);
            record_battle_wins(&mut activity, debt, 4);
            assert!(run_one_battle(&mut activity));
        }
        assert_eq!(activity.cosmic_fragments().unwrap().get(), initial * 2);
        assert!(
            activity
                .curio_contributions()
                .unwrap()
                .entries()
                .iter()
                .all(|entry| entry.curio() != debt)
        );
    }

    fn acquire_curios(activity: &mut StandardUniverseActivity, curios: &[CurioId]) {
        let records = compile_records(&activity.curio_runtime).unwrap();
        let bindings = activity.curio_activity_bindings();
        let operations = curios
            .iter()
            .flat_map(|id| {
                let record = records
                    .iter()
                    .copied()
                    .find(|record| record.id() == *id)
                    .unwrap();
                acquisition_operations(record, bindings)
            })
            .collect::<Vec<_>>();
        apply(activity, operations);
    }

    fn fresh_activity() -> StandardUniverseActivity {
        activity()
    }

    fn destroy_curios(activity: &mut StandardUniverseActivity, curios: &[CurioId]) {
        let bindings = activity.curio_activity_bindings();
        let operations = curios
            .iter()
            .flat_map(|id| crate::curio_activity::destroy_and_count_operations(*id, bindings))
            .collect::<Vec<_>>();
        apply(activity, operations);
    }

    fn acquire_blessings(
        activity: &mut StandardUniverseActivity,
        blessings: &[((BlessingId, crate::id::PathId), u32)],
    ) {
        let operations = blessings
            .iter()
            .flat_map(|((blessing, path), level)| {
                [
                    ActivityOperation::AddInventory {
                        inventory: activity.blessing_inventory,
                        content: u64::from(blessing.get()),
                        count: integer(i64::from(*level)),
                    },
                    ActivityOperation::AddCounter {
                        slot: activity.path_blessing_count_slot,
                        key: u64::from(path.get()),
                        delta: integer(1),
                    },
                ]
            })
            .collect();
        apply(activity, operations);
    }

    fn record_battle_wins(activity: &mut StandardUniverseActivity, curio: CurioId, count: i64) {
        apply(
            activity,
            vec![ActivityOperation::AddCounter {
                slot: activity.curio_event_slot,
                key: event_key(curio, CurioEvent::BattleWon),
                delta: integer(count),
            }],
        );
    }

    fn apply(activity: &mut StandardUniverseActivity, operations: Vec<ActivityOperation>) {
        let definition = ActivityProgramDefinition::new(program(TEST_PROGRAM), operations).unwrap();
        activity
            .graph
            .apply_boundary_program(activity.view().state_hash(), &definition)
            .unwrap();
    }

    fn curio(activity: &StandardUniverseActivity, stable_key: &str) -> CurioId {
        activity
            .curio_runtime
            .definitions()
            .iter()
            .find(|definition| definition.stable_key() == stable_key)
            .unwrap()
            .curio()
    }

    fn run_one_battle(activity: &mut StandardUniverseActivity) -> bool {
        let runner = StandardUniverseBaselineRunner::default();
        let mut executor =
            |handoff: &starclock_activity::ActivityBattleHandoff| Ok(victory(handoff));
        for _ in 0..128 {
            let step = match runner.advance(
                activity,
                &StandardUniverseBaselinePolicy::default(),
                &mut executor,
            ) {
                Ok(step) => step,
                Err(StandardUniverseBaselineError::AlreadyTerminal) => return false,
                Err(error) => panic!("baseline runner failed: {error:?}"),
            };
            if matches!(step, StandardUniverseBaselineStep::Battle { .. }) {
                return true;
            }
        }
        panic!("baseline runner did not reach a battle");
    }

    fn victory(handoff: &starclock_activity::ActivityBattleHandoff) -> BattleResult {
        let values = handoff
            .projection()
            .fields()
            .iter()
            .map(|field| match field {
                ProjectionField::Outcome => ProjectedValue::Outcome(BattleOutcome::Won),
                ProjectionField::FinalStateHash => {
                    ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x91; 32]))
                }
                ProjectionField::EventDigest => {
                    ProjectedValue::EventDigest(EventDigest::new([0x92; 32]).unwrap())
                }
                ProjectionField::TerminalFault => ProjectedValue::TerminalFault(None),
                ProjectionField::ParticipantState(participant) => {
                    let carry = handoff
                        .participant_carry()
                        .iter()
                        .find(|carry| carry.participant() == *participant)
                        .unwrap();
                    ProjectedValue::ParticipantState(
                        ParticipantBattleState::new(
                            *participant,
                            carry.current_hp(),
                            carry.maximum_hp(),
                            carry.current_energy(),
                            carry.maximum_energy(),
                            LifeState::Alive,
                            PresenceState::Present,
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

    fn activity() -> StandardUniverseActivity {
        let factory = StandardUniverseRuntimeFactory::load(CORE, UNIVERSE).unwrap();
        let world = factory.catalog().worlds()[0].id().get();
        let instance = factory
            .start(
                world,
                0,
                u64::from(TEST_PROGRAM),
                StandardUniverseControllerIdentity {
                    id: "negative-curio-test",
                    revision: "v1",
                    digest: [0x90; 32],
                },
            )
            .unwrap();
        let (_, mut activity, _, _, _) = instance.into_dynamic_parts();
        let path = activity.view();
        activity
            .choose_option(
                path.state_hash(),
                path.decision().unwrap().id(),
                path.decision().unwrap().options()[0].id(),
            )
            .unwrap();
        let bonus = activity.view();
        activity
            .submit_external_outcome(
                bonus.state_hash(),
                bonus.decision().unwrap().id(),
                ActivityExternalOutcomeId::new(bonus.decision().unwrap().options()[0].id().get())
                    .unwrap(),
            )
            .unwrap();
        activity
    }
}
