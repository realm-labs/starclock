//! Bounded non-authoritative cache for repeated Gold battle assembly.

use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use starclock_activity::{ActivityStateHash, ActivityTransactionState};

use crate::{battle_materialization::UniverseBattleRoster, digest::Encoder};

use super::{
    GoldAndGearsBattleAssemblyContext, GoldAndGearsBattleMaterialization,
    GoldAndGearsEncounterRole, GoldAndGearsEncounterSelection, GoldAndGearsEntryError,
    GoldAndGearsRuntimeInstance,
};

const CACHE_CAPACITY: usize = 8;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GoldAndGearsBattleAssemblyCacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub entries: usize,
}

#[derive(Debug, Default)]
pub(super) struct GoldAndGearsBattleMaterializationCache {
    entries: Mutex<BTreeMap<[u8; 32], Arc<GoldAndGearsBattleMaterialization>>>,
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
}

impl GoldAndGearsRuntimeInstance {
    /// Resolves an immutable battle input through a bounded cache excluded from
    /// authoritative state, configuration identity and replay serialization.
    pub(super) fn resolve_current_battle(
        &self,
        expected_state_hash: ActivityStateHash,
        state: &ActivityTransactionState,
        selection: &GoldAndGearsEncounterSelection,
        roster: &UniverseBattleRoster,
        context: &GoldAndGearsBattleAssemblyContext,
    ) -> Result<Arc<GoldAndGearsBattleMaterialization>, GoldAndGearsEntryError> {
        let key = cache_key(expected_state_hash, selection, roster, context);
        let mut entries = self
            .battle_materialization_cache
            .entries
            .lock()
            .expect("Gold battle cache lock is not held across panicking code");
        if let Some(cached) = entries.get(&key) {
            self.battle_materialization_cache
                .hits
                .fetch_add(1, Ordering::Relaxed);
            return Ok(Arc::clone(cached));
        }
        self.battle_materialization_cache
            .misses
            .fetch_add(1, Ordering::Relaxed);
        let materialization =
            Arc::new(self.materialize_current_battle(state, selection, roster, context)?);
        if entries.len() == CACHE_CAPACITY
            && let Some(oldest) = entries.first_key_value().map(|(key, _)| *key)
        {
            entries.remove(&oldest);
            self.battle_materialization_cache
                .evictions
                .fetch_add(1, Ordering::Relaxed);
        }
        entries.insert(key, Arc::clone(&materialization));
        Ok(materialization)
    }

    #[must_use]
    pub fn battle_assembly_cache_metrics(&self) -> GoldAndGearsBattleAssemblyCacheMetrics {
        let entries = self
            .battle_materialization_cache
            .entries
            .lock()
            .expect("Gold battle cache lock is not held across panicking code")
            .len();
        GoldAndGearsBattleAssemblyCacheMetrics {
            hits: self
                .battle_materialization_cache
                .hits
                .load(Ordering::Relaxed),
            misses: self
                .battle_materialization_cache
                .misses
                .load(Ordering::Relaxed),
            evictions: self
                .battle_materialization_cache
                .evictions
                .load(Ordering::Relaxed),
            entries,
        }
    }
}

fn cache_key(
    state: ActivityStateHash,
    selection: &GoldAndGearsEncounterSelection,
    roster: &UniverseBattleRoster,
    context: &GoldAndGearsBattleAssemblyContext,
) -> [u8; 32] {
    let mut hash = Encoder::new(b"starclock.gold-and-gears.battle-cache-key");
    hash.digest(state.bytes());
    hash.digest(roster.participant_lock().bytes());
    hash.text(selection.group());
    hash.u32(selection.source_group_id());
    hash.text(selection.source_rogue_monster_id());
    hash.text(selection.source_primary_monster_id());
    hash.text(selection.source_stage_id());
    hash.u8(role_code(selection.role()));
    hash.text(selection.difficulty_segment());
    hash.u32(u32::from(selection.effective_level()));
    hash.u32(
        u32::try_from(context.selected_formations().len())
            .expect("the bounded formation set fits u32"),
    );
    for formation in context.selected_formations() {
        hash.text(formation);
    }
    hash.bool(context.previous_battle_completed());
    hash.optional_digest(
        context
            .extrapolation()
            .map(super::GoldAndGearsExtrapolationSelection::digest),
    );
    hash.finish()
}

const fn role_code(role: GoldAndGearsEncounterRole) -> u8 {
    match role {
        GoldAndGearsEncounterRole::Combat => 0,
        GoldAndGearsEncounterRole::Elite => 1,
        GoldAndGearsEncounterRole::FirstPlaneBoss => 2,
        GoldAndGearsEncounterRole::SecondPlaneBoss => 3,
        GoldAndGearsEncounterRole::FinalBoss => 4,
    }
}
