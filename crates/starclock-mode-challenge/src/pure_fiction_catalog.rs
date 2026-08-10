use starclock_combat::{EncounterId, FormationIndex, UnitLevel};

use crate::{MemoryEnemyStats, PureFictionEnemyBindingId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PureFictionEnemyBinding {
    id: PureFictionEnemyBindingId,
    upstream_monster: u32,
    stable_key: Box<str>,
    behavior_source_key: Box<str>,
    behavior_exact: bool,
}

impl PureFictionEnemyBinding {
    #[must_use]
    pub fn new(
        id: PureFictionEnemyBindingId,
        upstream_monster: u32,
        stable_key: impl Into<Box<str>>,
        behavior_source_key: impl Into<Box<str>>,
        behavior_exact: bool,
    ) -> Option<Self> {
        let stable_key = stable_key.into();
        let behavior_source_key = behavior_source_key.into();
        if upstream_monster == 0
            || stable_key.trim().is_empty()
            || behavior_source_key.trim().is_empty()
            || (behavior_exact && stable_key != behavior_source_key)
        {
            return None;
        }
        Some(Self {
            id,
            upstream_monster,
            stable_key,
            behavior_source_key,
            behavior_exact,
        })
    }

    #[must_use]
    pub const fn id(&self) -> PureFictionEnemyBindingId {
        self.id
    }
    #[must_use]
    pub const fn upstream_monster(&self) -> u32 {
        self.upstream_monster
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub fn behavior_source_key(&self) -> &str {
        &self.behavior_source_key
    }
    #[must_use]
    pub const fn behavior_exact(&self) -> bool {
        self.behavior_exact
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PureFictionEnemySlot {
    binding: PureFictionEnemyBindingId,
    spawn_sequence: u16,
    formation: FormationIndex,
    stats: MemoryEnemyStats,
}

impl PureFictionEnemySlot {
    #[must_use]
    pub const fn new(
        binding: PureFictionEnemyBindingId,
        spawn_sequence: u16,
        formation: FormationIndex,
        stats: MemoryEnemyStats,
    ) -> Self {
        Self {
            binding,
            spawn_sequence,
            formation,
            stats,
        }
    }
    #[must_use]
    pub const fn binding(&self) -> PureFictionEnemyBindingId {
        self.binding
    }
    #[must_use]
    pub const fn spawn_sequence(&self) -> u16 {
        self.spawn_sequence
    }
    #[must_use]
    pub const fn formation(&self) -> FormationIndex {
        self.formation
    }
    #[must_use]
    pub const fn stats(&self) -> &MemoryEnemyStats {
        &self.stats
    }

    #[must_use]
    pub fn relocated(&self, spawn_sequence: u16, formation: FormationIndex) -> Self {
        Self {
            binding: self.binding,
            spawn_sequence,
            formation,
            stats: self.stats.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PureFictionSpawnEnd {
    DefeatQuota(u16),
    RequiredSlotsDefeated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PureFictionWave {
    sequence: u16,
    slots: Box<[PureFictionEnemySlot]>,
    spawn_end: PureFictionSpawnEnd,
    refill_source_wave: Option<u16>,
    maximum_simultaneous: u8,
    score_cap: i64,
    normal_defeat_true_damage_scaled: i64,
}

impl PureFictionWave {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        sequence: u16,
        mut slots: Vec<PureFictionEnemySlot>,
        spawn_end: PureFictionSpawnEnd,
        refill_source_wave: Option<u16>,
        maximum_simultaneous: u8,
        score_cap: i64,
        normal_defeat_true_damage_scaled: i64,
    ) -> Option<Self> {
        slots.sort_by_key(PureFictionEnemySlot::formation);
        if sequence == 0
            || slots.is_empty()
            || slots
                .windows(2)
                .any(|pair| pair[0].formation == pair[1].formation)
            || maximum_simultaneous == 0
            || usize::from(maximum_simultaneous) < slots.len()
            || score_cap <= 0
            || !(0..=1_000_000).contains(&normal_defeat_true_damage_scaled)
            || refill_source_wave.is_some_and(|source| source >= sequence || source == 0)
        {
            return None;
        }
        Some(Self {
            sequence,
            slots: slots.into_boxed_slice(),
            spawn_end,
            refill_source_wave,
            maximum_simultaneous,
            score_cap,
            normal_defeat_true_damage_scaled,
        })
    }

    #[must_use]
    pub const fn sequence(&self) -> u16 {
        self.sequence
    }
    #[must_use]
    pub fn slots(&self) -> &[PureFictionEnemySlot] {
        &self.slots
    }
    #[must_use]
    pub const fn spawn_end(&self) -> PureFictionSpawnEnd {
        self.spawn_end
    }
    #[must_use]
    pub const fn refill_source_wave(&self) -> Option<u16> {
        self.refill_source_wave
    }
    #[must_use]
    pub const fn maximum_simultaneous(&self) -> u8 {
        self.maximum_simultaneous
    }
    #[must_use]
    pub const fn score_cap(&self) -> i64 {
        self.score_cap
    }
    #[must_use]
    pub const fn normal_defeat_true_damage_scaled(&self) -> i64 {
        self.normal_defeat_true_damage_scaled
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PureFictionEncounter {
    id: EncounterId,
    level: UnitLevel,
    waves: Box<[PureFictionWave]>,
}

impl PureFictionEncounter {
    #[must_use]
    pub fn new(id: EncounterId, level: UnitLevel, mut waves: Vec<PureFictionWave>) -> Option<Self> {
        waves.sort_by_key(PureFictionWave::sequence);
        if waves.len() != 3
            || waves
                .iter()
                .enumerate()
                .any(|(index, wave)| usize::from(wave.sequence) != index + 1)
        {
            return None;
        }
        Some(Self {
            id,
            level,
            waves: waves.into_boxed_slice(),
        })
    }
    #[must_use]
    pub const fn id(&self) -> EncounterId {
        self.id
    }
    #[must_use]
    pub const fn level(&self) -> UnitLevel {
        self.level
    }
    #[must_use]
    pub fn waves(&self) -> &[PureFictionWave] {
        &self.waves
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PureFictionCombatDefinitions {
    enemies: Box<[PureFictionEnemyBinding]>,
    encounters: Box<[PureFictionEncounter]>,
}

impl PureFictionCombatDefinitions {
    #[must_use]
    pub fn new(
        mut enemies: Vec<PureFictionEnemyBinding>,
        mut encounters: Vec<PureFictionEncounter>,
    ) -> Option<Self> {
        enemies.sort_by_key(PureFictionEnemyBinding::id);
        encounters.sort_by_key(PureFictionEncounter::id);
        if enemies.is_empty()
            || encounters.is_empty()
            || enemies.windows(2).any(|pair| pair[0].id == pair[1].id)
            || encounters.windows(2).any(|pair| pair[0].id == pair[1].id)
            || encounters
                .iter()
                .flat_map(PureFictionEncounter::waves)
                .flat_map(PureFictionWave::slots)
                .any(|slot| {
                    enemies
                        .binary_search_by_key(&slot.binding(), PureFictionEnemyBinding::id)
                        .is_err()
                })
        {
            return None;
        }
        Some(Self {
            enemies: enemies.into_boxed_slice(),
            encounters: encounters.into_boxed_slice(),
        })
    }
    #[must_use]
    pub fn enemies(&self) -> &[PureFictionEnemyBinding] {
        &self.enemies
    }
    #[must_use]
    pub fn encounters(&self) -> &[PureFictionEncounter] {
        &self.encounters
    }
}
