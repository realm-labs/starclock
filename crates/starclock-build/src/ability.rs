//! Ability investment and exact effective-level selection.

use starclock_combat::AbilityId;

/// Checked ability level used for invested, effective and cap values.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AbilityLevel(u8);

impl AbilityLevel {
    pub const MAX: u8 = 15;
    #[must_use]
    pub const fn new(raw: u8) -> Option<Self> {
        if raw == 0 || raw > Self::MAX {
            None
        } else {
            Some(Self(raw))
        }
    }
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// One exact effective level and its selected combat ability definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityLevelRow {
    effective: AbilityLevel,
    resolved_ability: AbilityId,
}

impl AbilityLevelRow {
    #[must_use]
    pub const fn new(effective: AbilityLevel, resolved_ability: AbilityId) -> Self {
        Self {
            effective,
            resolved_ability,
        }
    }
    #[must_use]
    pub const fn effective(self) -> AbilityLevel {
        self.effective
    }
    #[must_use]
    pub const fn resolved_ability(self) -> AbilityId {
        self.resolved_ability
    }
}

/// Complete contiguous effective-level curve for one authored ability family.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityLevelTable {
    family: AbilityId,
    invested_cap: AbilityLevel,
    rows: Box<[AbilityLevelRow]>,
}

impl AbilityLevelTable {
    #[must_use]
    pub fn new(family: AbilityId, invested_cap: AbilityLevel, rows: Vec<AbilityLevelRow>) -> Self {
        Self {
            family,
            invested_cap,
            rows: rows.into_boxed_slice(),
        }
    }
    pub(crate) fn canonicalize(&mut self) {
        self.rows.sort_unstable_by_key(|row| row.effective);
    }
    pub(crate) fn is_complete(&self) -> bool {
        !self.rows.is_empty()
            && self.rows.len() <= usize::from(AbilityLevel::MAX)
            && self
                .rows
                .iter()
                .enumerate()
                .all(|(index, row)| usize::from(row.effective.get()) == index + 1)
            && usize::from(self.invested_cap.get()) <= self.rows.len()
    }
    #[must_use]
    pub const fn family(&self) -> AbilityId {
        self.family
    }
    #[must_use]
    pub const fn invested_cap(&self) -> AbilityLevel {
        self.invested_cap
    }
    #[must_use]
    pub fn rows(&self) -> &[AbilityLevelRow] {
        &self.rows
    }
    #[must_use]
    pub fn maximum_effective_level(&self) -> AbilityLevel {
        self.rows
            .last()
            .expect("validated ability table is non-empty")
            .effective
    }
    #[must_use]
    pub fn resolve(&self, effective: AbilityLevel) -> Option<AbilityId> {
        self.rows
            .get(usize::from(effective.get() - 1))
            .filter(|row| row.effective == effective)
            .map(|row| row.resolved_ability)
    }
}

/// Exact invested level selected for one ability family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityInvestment {
    family: AbilityId,
    invested: AbilityLevel,
}

impl AbilityInvestment {
    #[must_use]
    pub const fn new(family: AbilityId, invested: AbilityLevel) -> Self {
        Self { family, invested }
    }
    #[must_use]
    pub const fn family(self) -> AbilityId {
        self.family
    }
    #[must_use]
    pub const fn invested(self) -> AbilityLevel {
        self.invested
    }
}
