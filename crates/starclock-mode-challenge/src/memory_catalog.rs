use starclock_combat::{
    EncounterId, FormationIndex, Hp, RawToughness, Scalar, Speed, StatValue, UnitLevel,
    formula::{model::CombatElement, toughness::EnemyRank},
};

use crate::MemoryEnemyBindingId;

/// One exact released enemy identity and its currently executable behavior source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryEnemyBinding {
    id: MemoryEnemyBindingId,
    upstream_variant: u32,
    stable_key: Box<str>,
    behavior_source_key: Box<str>,
    behavior_exact: bool,
}

impl MemoryEnemyBinding {
    #[must_use]
    pub fn new(
        id: MemoryEnemyBindingId,
        upstream_variant: u32,
        stable_key: impl Into<Box<str>>,
        behavior_source_key: impl Into<Box<str>>,
        behavior_exact: bool,
    ) -> Option<Self> {
        let stable_key = stable_key.into();
        let behavior_source_key = behavior_source_key.into();
        if stable_key.trim().is_empty()
            || behavior_source_key.trim().is_empty()
            || (behavior_exact && stable_key != behavior_source_key)
        {
            return None;
        }
        Some(Self {
            id,
            upstream_variant,
            stable_key,
            behavior_source_key,
            behavior_exact,
        })
    }

    #[must_use]
    pub const fn id(&self) -> MemoryEnemyBindingId {
        self.id
    }
    #[must_use]
    pub const fn upstream_variant(&self) -> u32 {
        self.upstream_variant
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

/// One exact hostile placement in a released Memory wave.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryEnemyStats {
    maximum_hp: Hp,
    attack: StatValue,
    defense: StatValue,
    speed: Speed,
    effect_hit_rate: Scalar,
    effect_resistance: Scalar,
    rank: EnemyRank,
    weaknesses: Box<[CombatElement]>,
    toughness: RawToughness,
}

/// Fully resolved numeric input for one Memory enemy occurrence.
pub struct MemoryEnemyStatsInput {
    pub maximum_hp: Hp,
    pub attack: StatValue,
    pub defense: StatValue,
    pub speed: Speed,
    pub effect_hit_rate: Scalar,
    pub effect_resistance: Scalar,
    pub rank: EnemyRank,
    pub weaknesses: Vec<CombatElement>,
    pub toughness: RawToughness,
}

impl MemoryEnemyStats {
    #[must_use]
    pub fn new(input: MemoryEnemyStatsInput) -> Option<Self> {
        let MemoryEnemyStatsInput {
            maximum_hp,
            attack,
            defense,
            speed,
            effect_hit_rate,
            effect_resistance,
            rank,
            mut weaknesses,
            toughness,
        } = input;
        weaknesses.sort_unstable();
        if maximum_hp.get() == 0
            || weaknesses.windows(2).any(|pair| pair[0] == pair[1])
            || effect_hit_rate.scaled() < 0
            || effect_resistance.scaled() < 0
        {
            return None;
        }
        Some(Self {
            maximum_hp,
            attack,
            defense,
            speed,
            effect_hit_rate,
            effect_resistance,
            rank,
            weaknesses: weaknesses.into_boxed_slice(),
            toughness,
        })
    }
    #[must_use]
    pub const fn maximum_hp(&self) -> Hp {
        self.maximum_hp
    }
    #[must_use]
    pub const fn attack(&self) -> StatValue {
        self.attack
    }
    #[must_use]
    pub const fn defense(&self) -> StatValue {
        self.defense
    }
    #[must_use]
    pub const fn speed(&self) -> Speed {
        self.speed
    }
    #[must_use]
    pub const fn effect_hit_rate(&self) -> Scalar {
        self.effect_hit_rate
    }
    #[must_use]
    pub const fn effect_resistance(&self) -> Scalar {
        self.effect_resistance
    }
    #[must_use]
    pub const fn rank(&self) -> EnemyRank {
        self.rank
    }
    #[must_use]
    pub fn weaknesses(&self) -> &[CombatElement] {
        &self.weaknesses
    }
    #[must_use]
    pub const fn toughness(&self) -> RawToughness {
        self.toughness
    }
}

/// One exact hostile placement in a released Memory wave.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryEnemySlot {
    binding: MemoryEnemyBindingId,
    spawn_sequence: u16,
    formation: FormationIndex,
    stats: MemoryEnemyStats,
}

impl MemoryEnemySlot {
    #[must_use]
    pub fn new(
        binding: MemoryEnemyBindingId,
        spawn_sequence: u16,
        formation: FormationIndex,
        stats: MemoryEnemyStats,
    ) -> Option<Self> {
        if spawn_sequence == 0 {
            None
        } else {
            Some(Self {
                binding,
                spawn_sequence,
                formation,
                stats,
            })
        }
    }
    #[must_use]
    pub const fn binding(&self) -> MemoryEnemyBindingId {
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
}

/// One ordered released Memory encounter wave.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryWave {
    sequence: u16,
    slots: Box<[MemoryEnemySlot]>,
}

impl MemoryWave {
    #[must_use]
    pub fn new(sequence: u16, mut slots: Vec<MemoryEnemySlot>) -> Option<Self> {
        slots.sort_by_key(|slot| (slot.spawn_sequence, slot.formation));
        if sequence == 0
            || slots.is_empty()
            || slots.windows(2).any(|pair| {
                pair[0].spawn_sequence == pair[1].spawn_sequence
                    || pair[0].formation == pair[1].formation
            })
        {
            return None;
        }
        Some(Self {
            sequence,
            slots: slots.into_boxed_slice(),
        })
    }
    #[must_use]
    pub const fn sequence(&self) -> u16 {
        self.sequence
    }
    #[must_use]
    pub fn slots(&self) -> &[MemoryEnemySlot] {
        &self.slots
    }
}

/// Battle topology and exact authored difficulty selectors for one Memory node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryEncounter {
    id: EncounterId,
    level: UnitLevel,
    hard_level_group: u16,
    waves: Box<[MemoryWave]>,
}

impl MemoryEncounter {
    #[must_use]
    pub fn new(
        id: EncounterId,
        level: UnitLevel,
        hard_level_group: u16,
        mut waves: Vec<MemoryWave>,
    ) -> Option<Self> {
        waves.sort_by_key(MemoryWave::sequence);
        if waves.is_empty()
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
            hard_level_group,
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
    pub const fn hard_level_group(&self) -> u16 {
        self.hard_level_group
    }
    #[must_use]
    pub fn waves(&self) -> &[MemoryWave] {
        &self.waves
    }
}

/// Generated-row-free executable closure for all playable Version 4.4 Memory nodes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryCombatDefinitions {
    enemies: Box<[MemoryEnemyBinding]>,
    encounters: Box<[MemoryEncounter]>,
}

impl MemoryCombatDefinitions {
    #[must_use]
    pub fn new(
        mut enemies: Vec<MemoryEnemyBinding>,
        mut encounters: Vec<MemoryEncounter>,
    ) -> Option<Self> {
        enemies.sort_by_key(MemoryEnemyBinding::id);
        encounters.sort_by_key(MemoryEncounter::id);
        if enemies.is_empty()
            || encounters.is_empty()
            || enemies.windows(2).any(|pair| pair[0].id == pair[1].id)
            || encounters.windows(2).any(|pair| pair[0].id == pair[1].id)
            || encounters
                .iter()
                .flat_map(|encounter| encounter.waves())
                .flat_map(MemoryWave::slots)
                .any(|slot| {
                    enemies
                        .binary_search_by_key(&slot.binding(), MemoryEnemyBinding::id)
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
    pub fn enemies(&self) -> &[MemoryEnemyBinding] {
        &self.enemies
    }
    #[must_use]
    pub fn encounters(&self) -> &[MemoryEncounter] {
        &self.encounters
    }
    #[must_use]
    pub fn enemy(&self, id: MemoryEnemyBindingId) -> Option<&MemoryEnemyBinding> {
        self.enemies
            .binary_search_by_key(&id, MemoryEnemyBinding::id)
            .ok()
            .map(|index| &self.enemies[index])
    }
}
