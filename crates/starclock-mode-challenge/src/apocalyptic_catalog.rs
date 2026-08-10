use starclock_combat::{EncounterId, FormationIndex, UnitLevel};

use crate::{ApocalypticEnemyBindingId, MemoryEnemyStats};

/// One exact released boss identity and its temporary executable behavior source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApocalypticEnemyBinding {
    id: ApocalypticEnemyBindingId,
    upstream_monster: u32,
    stable_key: Box<str>,
    behavior_source_key: Box<str>,
    behavior_exact: bool,
}

impl ApocalypticEnemyBinding {
    #[must_use]
    pub fn new(
        id: ApocalypticEnemyBindingId,
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
    pub const fn id(&self) -> ApocalypticEnemyBindingId {
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

/// One boss or explicitly selected auxiliary scoring placement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApocalypticEnemySlot {
    binding: ApocalypticEnemyBindingId,
    formation: FormationIndex,
    score_included: bool,
    stats: MemoryEnemyStats,
}

impl ApocalypticEnemySlot {
    #[must_use]
    pub const fn new(
        binding: ApocalypticEnemyBindingId,
        formation: FormationIndex,
        score_included: bool,
        stats: MemoryEnemyStats,
    ) -> Self {
        Self {
            binding,
            formation,
            score_included,
            stats,
        }
    }
    #[must_use]
    pub const fn binding(&self) -> ApocalypticEnemyBindingId {
        self.binding
    }
    #[must_use]
    pub const fn formation(&self) -> FormationIndex {
        self.formation
    }
    #[must_use]
    pub const fn score_included(&self) -> bool {
        self.score_included
    }
    #[must_use]
    pub const fn stats(&self) -> &MemoryEnemyStats {
        &self.stats
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApocalypticEncounter {
    id: EncounterId,
    level: UnitLevel,
    slots: Box<[ApocalypticEnemySlot]>,
}

impl ApocalypticEncounter {
    #[must_use]
    pub fn new(
        id: EncounterId,
        level: UnitLevel,
        mut slots: Vec<ApocalypticEnemySlot>,
    ) -> Option<Self> {
        slots.sort_by_key(ApocalypticEnemySlot::formation);
        if slots.is_empty()
            || slots
                .windows(2)
                .any(|pair| pair[0].formation == pair[1].formation)
            || slots.iter().any(|slot| !slot.score_included)
        {
            return None;
        }
        Some(Self {
            id,
            level,
            slots: slots.into_boxed_slice(),
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
    pub fn slots(&self) -> &[ApocalypticEnemySlot] {
        &self.slots
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApocalypticCombatDefinitions {
    enemies: Box<[ApocalypticEnemyBinding]>,
    encounters: Box<[ApocalypticEncounter]>,
}

impl ApocalypticCombatDefinitions {
    #[must_use]
    pub fn new(
        mut enemies: Vec<ApocalypticEnemyBinding>,
        mut encounters: Vec<ApocalypticEncounter>,
    ) -> Option<Self> {
        enemies.sort_by_key(ApocalypticEnemyBinding::id);
        encounters.sort_by_key(ApocalypticEncounter::id);
        if enemies.is_empty()
            || encounters.is_empty()
            || enemies.windows(2).any(|pair| pair[0].id == pair[1].id)
            || encounters.windows(2).any(|pair| pair[0].id == pair[1].id)
            || encounters
                .iter()
                .flat_map(ApocalypticEncounter::slots)
                .any(|slot| {
                    enemies
                        .binary_search_by_key(&slot.binding(), ApocalypticEnemyBinding::id)
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
    pub fn enemies(&self) -> &[ApocalypticEnemyBinding] {
        &self.enemies
    }
    #[must_use]
    pub fn encounters(&self) -> &[ApocalypticEncounter] {
        &self.encounters
    }
}
