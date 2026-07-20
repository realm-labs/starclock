//! Exact Light Cone stat rows, applicability and S1-S5 passive selections.

use starclock_combat::{Hp, StatValue, rule::model::RuleSource};

use crate::{id::LightConeId, patch::BuildPatch, spec::PromotionStage};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CombatPath {
    Destruction,
    Hunt,
    Erudition,
    Harmony,
    Nihility,
    Preservation,
    Abundance,
    Remembrance,
    Elation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LightConeApplicability {
    MatchingPath,
    Always,
    BaseStatsOnly,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LightConeLevel(u8);

impl LightConeLevel {
    pub const MAX: u8 = 80;
    #[must_use]
    pub const fn new(raw: u8) -> Option<Self> {
        if raw > 0 && raw <= Self::MAX {
            Some(Self(raw))
        } else {
            None
        }
    }
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Superimposition(u8);

impl Superimposition {
    pub const MAX: u8 = 5;
    #[must_use]
    pub const fn new(raw: u8) -> Option<Self> {
        if raw > 0 && raw <= Self::MAX {
            Some(Self(raw))
        } else {
            None
        }
    }
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LightConeStatRow {
    level: LightConeLevel,
    promotion: PromotionStage,
    maximum_hp: Hp,
    attack: StatValue,
    defense: StatValue,
}

impl LightConeStatRow {
    #[must_use]
    pub const fn new(
        level: LightConeLevel,
        promotion: PromotionStage,
        maximum_hp: Hp,
        attack: StatValue,
        defense: StatValue,
    ) -> Self {
        Self {
            level,
            promotion,
            maximum_hp,
            attack,
            defense,
        }
    }
    #[must_use]
    pub const fn level(self) -> LightConeLevel {
        self.level
    }
    #[must_use]
    pub const fn promotion(self) -> PromotionStage {
        self.promotion
    }
    #[must_use]
    pub const fn maximum_hp(self) -> Hp {
        self.maximum_hp
    }
    #[must_use]
    pub const fn attack(self) -> StatValue {
        self.attack
    }
    #[must_use]
    pub const fn defense(self) -> StatValue {
        self.defense
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LightConePassiveRank {
    rank: Superimposition,
    patches: Box<[BuildPatch]>,
}

impl LightConePassiveRank {
    #[must_use]
    pub fn new(rank: Superimposition, patches: Vec<BuildPatch>) -> Self {
        Self {
            rank,
            patches: patches.into_boxed_slice(),
        }
    }
    #[must_use]
    pub const fn rank(&self) -> Superimposition {
        self.rank
    }
    #[must_use]
    pub fn patches(&self) -> &[BuildPatch] {
        &self.patches
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LightConeDefinition {
    id: LightConeId,
    source: RuleSource,
    path: CombatPath,
    applicability: LightConeApplicability,
    stats: Box<[LightConeStatRow]>,
    passive_ranks: Box<[LightConePassiveRank]>,
}

impl LightConeDefinition {
    #[must_use]
    pub fn new(
        id: LightConeId,
        source: RuleSource,
        path: CombatPath,
        applicability: LightConeApplicability,
        stats: Vec<LightConeStatRow>,
        passive_ranks: Vec<LightConePassiveRank>,
    ) -> Self {
        Self {
            id,
            source,
            path,
            applicability,
            stats: stats.into_boxed_slice(),
            passive_ranks: passive_ranks.into_boxed_slice(),
        }
    }
    pub(crate) fn canonicalize(&mut self) -> Result<(), LightConeDefinitionError> {
        self.stats
            .sort_unstable_by_key(|row| (row.level(), row.promotion()));
        if self.stats.is_empty()
            || self.stats.windows(2).any(|rows| {
                (rows[0].level(), rows[0].promotion()) == (rows[1].level(), rows[1].promotion())
            })
            || self.stats.iter().any(|row| {
                row.maximum_hp().get() <= 0
                    || row.attack().scaled() <= 0
                    || row.defense().scaled() <= 0
            })
        {
            return Err(LightConeDefinitionError::InvalidStatCurve);
        }
        self.passive_ranks
            .sort_unstable_by_key(LightConePassiveRank::rank);
        if self.passive_ranks.len() != usize::from(Superimposition::MAX)
            || self
                .passive_ranks
                .iter()
                .enumerate()
                .any(|(index, rank)| rank.rank().get() != u8::try_from(index + 1).unwrap())
        {
            return Err(LightConeDefinitionError::IncompletePassiveRanks);
        }
        Ok(())
    }
    #[must_use]
    pub const fn id(&self) -> LightConeId {
        self.id
    }
    #[must_use]
    pub const fn source(&self) -> &RuleSource {
        &self.source
    }
    #[must_use]
    pub const fn path(&self) -> CombatPath {
        self.path
    }
    #[must_use]
    pub const fn applicability(&self) -> LightConeApplicability {
        self.applicability
    }
    #[must_use]
    pub fn stat_row(
        &self,
        level: LightConeLevel,
        promotion: PromotionStage,
    ) -> Option<&LightConeStatRow> {
        self.stats
            .binary_search_by_key(&(level, promotion), |row| (row.level(), row.promotion()))
            .ok()
            .map(|index| &self.stats[index])
    }
    #[must_use]
    pub fn stats(&self) -> &[LightConeStatRow] {
        &self.stats
    }
    #[must_use]
    pub fn passive_rank(&self, rank: Superimposition) -> &LightConePassiveRank {
        &self.passive_ranks[usize::from(rank.get() - 1)]
    }
    #[must_use]
    pub fn passive_ranks(&self) -> &[LightConePassiveRank] {
        &self.passive_ranks
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LightConeDefinitionError {
    InvalidStatCurve,
    IncompletePassiveRanks,
}
