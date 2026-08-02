//! Canonical keys and bounded scratch cache for Standard Universe battle assembly.
//!
//! Cache contents and counters are deliberately excluded from Activity state,
//! replay payloads and assembly identity. Clearing this cache may cost time but
//! cannot change a produced battle.

use std::{
    collections::{BTreeMap, VecDeque},
    num::NonZeroUsize,
    sync::Arc,
};

use starclock_activity::ParticipantLockDigest;

use crate::{battle_materialization::UniverseBattleMaterialization, digest::Encoder};

pub const DEFAULT_BATTLE_ASSEMBLY_CACHE_CAPACITY: usize = 8;

/// Exact immutable inputs consumed by one assembly operation.
///
/// The encounter and carry fields are digests rather than mode IDs so this key
/// remains usable for a full encounter overlay today and a single prepared
/// encounter in the next assembly revision.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BattleAssemblyKey {
    catalog_composition: [u8; 32],
    participant_lock: ParticipantLockDigest,
    encounter: [u8; 32],
    contributions: [u8; 32],
    carry: [u8; 32],
    technique: Option<[u8; 32]>,
    digest: [u8; 32],
}

impl BattleAssemblyKey {
    #[must_use]
    pub fn new(
        catalog_composition: [u8; 32],
        participant_lock: ParticipantLockDigest,
        encounter: [u8; 32],
        contributions: [u8; 32],
        carry: [u8; 32],
        technique: Option<[u8; 32]>,
    ) -> Self {
        let mut encoder = Encoder::new(b"starclock.standard-universe.battle-assembly-key");
        encoder.digest(catalog_composition);
        encoder.digest(participant_lock.bytes());
        encoder.digest(encounter);
        encoder.digest(contributions);
        encoder.digest(carry);
        match technique {
            Some(value) => {
                encoder.u8(1);
                encoder.digest(value);
            }
            None => encoder.u8(0),
        }
        Self {
            catalog_composition,
            participant_lock,
            encounter,
            contributions,
            carry,
            technique,
            digest: encoder.finish(),
        }
    }

    #[must_use]
    pub const fn catalog_composition(self) -> [u8; 32] {
        self.catalog_composition
    }

    #[must_use]
    pub const fn participant_lock(self) -> ParticipantLockDigest {
        self.participant_lock
    }

    #[must_use]
    pub const fn encounter(self) -> [u8; 32] {
        self.encounter
    }

    #[must_use]
    pub const fn contributions(self) -> [u8; 32] {
        self.contributions
    }

    #[must_use]
    pub const fn carry(self) -> [u8; 32] {
        self.carry
    }

    #[must_use]
    pub const fn technique(self) -> Option<[u8; 32]> {
        self.technique
    }

    #[must_use]
    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BattleAssemblyCacheMetrics {
    hits: u64,
    misses: u64,
    insertions: u64,
    evictions: u64,
}

impl BattleAssemblyCacheMetrics {
    #[must_use]
    pub const fn hits(self) -> u64 {
        self.hits
    }

    #[must_use]
    pub const fn misses(self) -> u64 {
        self.misses
    }

    #[must_use]
    pub const fn insertions(self) -> u64 {
        self.insertions
    }

    #[must_use]
    pub const fn evictions(self) -> u64 {
        self.evictions
    }
}

/// Deterministic FIFO cache for immutable completed assemblies.
///
/// Hits do not alter eviction order, so identical insertion sequences have
/// identical scratch behavior. The cache is never consulted as authority:
/// every returned entry must carry the exact requested key.
#[derive(Debug)]
pub struct BattleAssemblyCache {
    capacity: NonZeroUsize,
    entries: BTreeMap<BattleAssemblyKey, Arc<UniverseBattleMaterialization>>,
    insertion_order: VecDeque<BattleAssemblyKey>,
    metrics: BattleAssemblyCacheMetrics,
}

impl Default for BattleAssemblyCache {
    fn default() -> Self {
        Self::new(
            NonZeroUsize::new(DEFAULT_BATTLE_ASSEMBLY_CACHE_CAPACITY)
                .expect("default cache capacity is non-zero"),
        )
    }
}

impl BattleAssemblyCache {
    #[must_use]
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            capacity,
            entries: BTreeMap::new(),
            insertion_order: VecDeque::new(),
            metrics: BattleAssemblyCacheMetrics::default(),
        }
    }

    #[must_use]
    pub const fn capacity(&self) -> NonZeroUsize {
        self.capacity
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub const fn metrics(&self) -> BattleAssemblyCacheMetrics {
        self.metrics
    }

    pub fn get(&mut self, key: BattleAssemblyKey) -> Option<Arc<UniverseBattleMaterialization>> {
        let value = self
            .entries
            .get(&key)
            .filter(|entry| entry.assembly_key() == key)
            .cloned();
        if value.is_some() {
            self.metrics.hits = self.metrics.hits.saturating_add(1);
        } else {
            self.metrics.misses = self.metrics.misses.saturating_add(1);
        }
        value
    }

    pub fn insert(
        &mut self,
        key: BattleAssemblyKey,
        value: Arc<UniverseBattleMaterialization>,
    ) -> Result<(), BattleAssemblyCacheError> {
        if value.assembly_key() != key {
            return Err(BattleAssemblyCacheError::KeyMismatch);
        }
        if let std::collections::btree_map::Entry::Occupied(mut entry) = self.entries.entry(key) {
            entry.insert(value);
            return Ok(());
        }
        if self.entries.len() == self.capacity.get() {
            let evicted = self
                .insertion_order
                .pop_front()
                .ok_or(BattleAssemblyCacheError::CorruptOrder)?;
            if self.entries.remove(&evicted).is_none() {
                return Err(BattleAssemblyCacheError::CorruptOrder);
            }
            self.metrics.evictions = self.metrics.evictions.saturating_add(1);
        }
        self.entries.insert(key, value);
        self.insertion_order.push_back(key);
        self.metrics.insertions = self.metrics.insertions.saturating_add(1);
        Ok(())
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.insertion_order.clear();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleAssemblyCacheError {
    KeyMismatch,
    CorruptOrder,
}

impl core::fmt::Display for BattleAssemblyCacheError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "battle assembly cache error: {self:?}")
    }
}

impl std::error::Error for BattleAssemblyCacheError {}
