//! Immutable current-Activity contribution snapshot for one Gold and Gears battle.

use crate::{
    blessing_runtime::BlessingContributionSet, curio_runtime::CurioContributionSet, digest::Encoder,
};
use starclock_activity::{
    ActivityInventoryId, ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use crate::{
    ability_runtime::{AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope},
    battle_contribution::UniverseBattleContributionSet,
    curio::CurioStateKind,
    id::{BlessingId, CurioStateId, ResonanceId},
};

use super::GoldAndGearsEncounterRole;
use super::{
    GoldAndGearsCurioContributionSet, GoldAndGearsCurioId, GoldAndGearsCurioState,
    GoldAndGearsEntryError, GoldAndGearsExtrapolationSelection, GoldAndGearsPathBoostCombatSet,
    GoldAndGearsResonanceCombatSet, GoldAndGearsRuntimeInstance,
    GoldAndGearsStatsConundrumModifierSet,
    state_layout::{
        BLESSING_INVENTORY, CONTENT_CURIO_CHARGE_BASE, CONTENT_CURIO_STATE_BASE,
        CONTENT_LIFECYCLE_SLOT, CURIO_INVENTORY,
    },
};

/// Versioned current-state-to-battle contribution projection.
pub const GOLD_AND_GEARS_BATTLE_SNAPSHOT_REVISION: &str = "gold-and-gears-battle-snapshot-v1";

/// Caller-owned choices that are not inventories: unlocked Formation slots and
/// the separately selected Third Plane boss Extrapolation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsBattleAssemblyContext {
    selected_formations: Box<[Box<str>]>,
    extrapolation: Option<GoldAndGearsExtrapolationSelection>,
    previous_battle_completed: bool,
}

impl GoldAndGearsBattleAssemblyContext {
    #[must_use]
    pub fn new(selected_formations: Vec<String>, previous_battle_completed: bool) -> Self {
        Self {
            selected_formations: selected_formations.into_iter().map(Into::into).collect(),
            extrapolation: None,
            previous_battle_completed,
        }
    }

    #[must_use]
    pub fn with_extrapolation(mut self, selection: GoldAndGearsExtrapolationSelection) -> Self {
        self.extrapolation = Some(selection);
        self
    }

    pub fn selected_formations(&self) -> impl ExactSizeIterator<Item = &str> {
        self.selected_formations.iter().map(Box::as_ref)
    }

    #[must_use]
    pub const fn extrapolation(&self) -> Option<&GoldAndGearsExtrapolationSelection> {
        self.extrapolation.as_ref()
    }

    #[must_use]
    pub const fn previous_battle_completed(&self) -> bool {
        self.previous_battle_completed
    }
}

/// Auditable contribution counts and component digests consumed by one battle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsBattleContributionSnapshot {
    blessing_count: u16,
    curio_count: u16,
    gold_curio_count: u16,
    neural_count: u16,
    conundrum_modifier_count: u16,
    selected_formation_count: u8,
    shared_digest: [u8; 32],
    path_boost_digest: [u8; 32],
    resonance_digest: [u8; 32],
    extrapolation_digest: Option<[u8; 32]>,
    curio_digest: [u8; 32],
    neural_digest: [u8; 32],
    conundrum_digest: [u8; 32],
    digest: [u8; 32],
}

impl GoldAndGearsBattleContributionSnapshot {
    pub const fn blessing_count(&self) -> u16 {
        self.blessing_count
    }
    pub const fn curio_count(&self) -> u16 {
        self.curio_count
    }
    pub const fn gold_curio_count(&self) -> u16 {
        self.gold_curio_count
    }
    pub const fn neural_count(&self) -> u16 {
        self.neural_count
    }
    pub const fn conundrum_modifier_count(&self) -> u16 {
        self.conundrum_modifier_count
    }
    pub const fn selected_formation_count(&self) -> u8 {
        self.selected_formation_count
    }
    pub const fn shared_digest(&self) -> [u8; 32] {
        self.shared_digest
    }
    pub const fn path_boost_digest(&self) -> [u8; 32] {
        self.path_boost_digest
    }
    pub const fn resonance_digest(&self) -> [u8; 32] {
        self.resonance_digest
    }
    pub const fn extrapolation_digest(&self) -> Option<[u8; 32]> {
        self.extrapolation_digest
    }
    pub const fn curio_digest(&self) -> [u8; 32] {
        self.curio_digest
    }
    pub const fn neural_digest(&self) -> [u8; 32] {
        self.neural_digest
    }
    pub const fn conundrum_digest(&self) -> [u8; 32] {
        self.conundrum_digest
    }
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

pub(super) struct CompiledGoldBattleSnapshot {
    pub(super) summary: GoldAndGearsBattleContributionSnapshot,
    pub(super) shared: UniverseBattleContributionSet,
    pub(super) path_boost: GoldAndGearsPathBoostCombatSet,
    pub(super) conundrum: GoldAndGearsStatsConundrumModifierSet,
}

impl GoldAndGearsRuntimeInstance {
    pub(super) fn compile_battle_snapshot(
        &self,
        state: &ActivityTransactionState,
        context: &GoldAndGearsBattleAssemblyContext,
    ) -> Result<CompiledGoldBattleSnapshot, GoldAndGearsEntryError> {
        let blessings = inventory(state, BLESSING_INVENTORY)?
            .into_iter()
            .map(|(raw, count)| {
                u32::try_from(raw)
                    .ok()
                    .and_then(BlessingId::new)
                    .map(|id| (id, count))
                    .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let blessing_set = self.blessing_contributions(&blessings)?;
        let blessing_counts = self.blessing_path_counts(&blessing_set)?;
        let formations = context
            .selected_formations()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let resonance_set = self.resonance_additions(&blessing_counts, &formations)?;
        let resonance = self.compile_resonance_combat_set(&resonance_set)?;

        let standard = &self.content_runtime.standard;
        let selected_path = standard
            .paths()
            .iter()
            .find(|path| path.stable_key() == self.path())
            .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
        let formation_ids = formations
            .iter()
            .map(|key| {
                standard
                    .resonances()
                    .iter()
                    .find(|candidate| candidate.stable_key() == key)
                    .map(|candidate| (candidate.id(), 1))
                    .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)
            })
            .collect::<Result<Vec<(ResonanceId, u32)>, _>>()?;
        let path = self
            .content_runtime
            .paths
            .contributions(selected_path.id(), &blessing_set, &formation_ids)
            .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?;

        let (curios, shared_curios) = self.current_curios(state)?;
        let abilities = self
            .content_runtime
            .run
            .ability_contributions(&[])
            .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?;
        let boundary = if matches!(
            self.encounter_role_for_node(state, state.current_node()),
            Some(
                GoldAndGearsEncounterRole::Elite
                    | GoldAndGearsEncounterRole::FirstPlaneBoss
                    | GoldAndGearsEncounterRole::SecondPlaneBoss
                    | GoldAndGearsEncounterRole::FinalBoss
            )
        ) {
            AbilityBoundary::EnterEliteOrBossDomain
        } else {
            AbilityBoundary::BattleStart
        };
        let ability_context = AbilityExecutionContext::new(
            AbilityProjectionScope::Battle,
            boundary,
            path.selected_path_blessings(),
            context.previous_battle_completed,
        );
        let projection = self
            .content_runtime
            .abilities
            .project(&[], ability_context)
            .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?;
        let shared = self
            .content_runtime
            .battle_contributions
            .compile_snapshot(
                &path,
                &blessing_set,
                &shared_curios,
                &abilities,
                &projection,
            )
            .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?;
        let path_boost = self.compile_path_boost_combat_set(state)?;
        let conundrum = self.compile_stats_conundrum_modifiers()?;
        let extrapolation = context
            .extrapolation()
            .map(|selection| self.compile_extrapolation_combat_set(selection))
            .transpose()?;
        let digest = snapshot_digest(
            shared.digest(),
            path_boost.digest(),
            resonance.digest(),
            extrapolation
                .as_ref()
                .map(GoldAndGearsResonanceCombatSet::digest),
            curios.digest(),
            self.neural_contribution_digest(),
            conundrum.digest(),
        );
        let gold_curio_count = curios
            .entries()
            .iter()
            .filter(|entry| entry.shared_curio().is_none())
            .count();
        let summary = GoldAndGearsBattleContributionSnapshot {
            blessing_count: checked_count(blessing_set.entries().len())?,
            curio_count: checked_count(curios.entries().len())?,
            gold_curio_count: checked_count(gold_curio_count)?,
            neural_count: checked_count(self.neural_battle_stat_contributions().len())?,
            conundrum_modifier_count: checked_count(conundrum.bindings().len())?,
            selected_formation_count: u8::try_from(formations.len())
                .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?,
            shared_digest: shared.digest(),
            path_boost_digest: path_boost.digest(),
            resonance_digest: resonance.digest(),
            extrapolation_digest: extrapolation
                .as_ref()
                .map(GoldAndGearsResonanceCombatSet::digest),
            curio_digest: curios.digest(),
            neural_digest: self.neural_contribution_digest(),
            conundrum_digest: conundrum.digest(),
            digest,
        };
        Ok(CompiledGoldBattleSnapshot {
            summary,
            shared,
            path_boost,
            conundrum,
        })
    }

    fn blessing_path_counts(
        &self,
        blessings: &BlessingContributionSet,
    ) -> Result<Vec<(String, u8)>, GoldAndGearsEntryError> {
        self.content_runtime
            .standard
            .paths()
            .iter()
            .map(|path| {
                let count = blessings
                    .entries()
                    .iter()
                    .filter(|entry| entry.path() == path.id())
                    .count();
                Ok((
                    path.stable_key().to_owned(),
                    u8::try_from(count)
                        .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?,
                ))
            })
            .collect()
    }

    fn current_curios(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<(GoldAndGearsCurioContributionSet, CurioContributionSet), GoldAndGearsEntryError>
    {
        let owned = inventory(state, CURIO_INVENTORY)?
            .into_iter()
            .map(|(raw, count)| {
                u32::try_from(raw)
                    .ok()
                    .and_then(GoldAndGearsCurioId::new)
                    .map(|id| (id, count))
                    .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let counters = counter_map(state, CONTENT_LIFECYCLE_SLOT)?;
        let mut states = Vec::with_capacity(owned.len());
        let mut remaining = Vec::new();
        for (id, _) in &owned {
            let raw = value_for(&counters, CONTENT_CURIO_STATE_BASE + u64::from(id.get()));
            let state = match raw {
                1 => GoldAndGearsCurioState::Active,
                2 => GoldAndGearsCurioState::Repairing,
                3 => GoldAndGearsCurioState::Fixed,
                4 => GoldAndGearsCurioState::Destroyed,
                5 => GoldAndGearsCurioState::Replaced,
                _ => return Err(GoldAndGearsEntryError::InvalidBattleMaterialization),
            };
            states.push((*id, state));
            let raw = value_for(&counters, CONTENT_CURIO_CHARGE_BASE + u64::from(id.get()));
            if raw != 0 {
                remaining.push((
                    *id,
                    u8::try_from(raw)
                        .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?,
                ));
            }
        }
        let gold = self.curio_contributions(&owned, &states, &remaining)?;
        let mut shared_owned = Vec::new();
        let mut shared_states = Vec::new();
        let mut shared_charges = Vec::new();
        for entry in gold.entries().iter().filter(|entry| {
            entry.shared_curio().is_some()
                && !matches!(
                    entry.state(),
                    GoldAndGearsCurioState::Destroyed | GoldAndGearsCurioState::Replaced
                )
        }) {
            let id = entry.shared_curio().expect("filtered shared Curio");
            let definition = self
                .content_runtime
                .shared_curios
                .definition(id)
                .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
            let kind = match entry.state() {
                GoldAndGearsCurioState::Active => CurioStateKind::Active,
                GoldAndGearsCurioState::Repairing => CurioStateKind::Repairing,
                GoldAndGearsCurioState::Fixed => CurioStateKind::Fixed,
                GoldAndGearsCurioState::Destroyed | GoldAndGearsCurioState::Replaced => {
                    unreachable!()
                }
            };
            let state_id = definition
                .states()
                .iter()
                .find(|state| state.kind() == kind)
                .map(|state| state.id())
                .or_else(|| (kind == CurioStateKind::Active).then_some(definition.initial_state()))
                .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
            shared_owned.push((id, 1));
            shared_states.push((
                id,
                CurioStateId::new(state_id.get()).expect("state ID is non-zero"),
            ));
            if entry.remaining_or_progress() != 0 {
                shared_charges.push((id, entry.remaining_or_progress()));
            }
        }
        let shared = self
            .content_runtime
            .shared_curios
            .contributions_from_owned(&shared_owned, &shared_states, &shared_charges)
            .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?;
        Ok((gold, shared))
    }
}

fn inventory(
    state: &ActivityTransactionState,
    raw: u32,
) -> Result<Vec<(u64, u32)>, GoldAndGearsEntryError> {
    state
        .inventory_entries(ActivityInventoryId::new(raw).expect("static inventory is non-zero"))
        .map(Iterator::collect)
        .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)
}

fn counter_map(
    state: &ActivityTransactionState,
    raw: u32,
) -> Result<Box<[(u64, i64)]>, GoldAndGearsEntryError> {
    match state.slot(ActivitySlotId::new(raw).expect("static slot is non-zero")) {
        Some(ActivityValue::BoundedCounterMap(values)) => Ok(values.clone()),
        _ => Err(GoldAndGearsEntryError::InvalidBattleMaterialization),
    }
}

fn value_for(values: &[(u64, i64)], key: u64) -> i64 {
    values
        .binary_search_by_key(&key, |entry| entry.0)
        .ok()
        .map_or(0, |index| values[index].1)
}

fn checked_count(value: usize) -> Result<u16, GoldAndGearsEntryError> {
    u16::try_from(value).map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)
}

fn snapshot_digest(
    shared: [u8; 32],
    path_boost: [u8; 32],
    resonance: [u8; 32],
    extrapolation: Option<[u8; 32]>,
    curios: [u8; 32],
    neural: [u8; 32],
    conundrum: [u8; 32],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-and-gears.battle-contribution-snapshot.v1");
    encoder.digest(shared);
    encoder.digest(path_boost);
    encoder.digest(resonance);
    encoder.digest(extrapolation.unwrap_or([0; 32]));
    encoder.digest(curios);
    encoder.digest(neural);
    encoder.digest(conundrum);
    encoder.finish()
}
