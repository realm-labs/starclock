//! Read-only projections of pending timeline work and allocator cursors.

use crate::UnitId;
use crate::timeline::state::PendingExtraTurn;

/// One queued extra-turn request in deterministic insertion order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingExtraTurnView {
    insertion: u64,
    unit: UnitId,
}

impl PendingExtraTurnView {
    #[must_use]
    pub const fn insertion(self) -> u64 {
        self.insertion
    }
    #[must_use]
    pub const fn unit(self) -> UnitId {
        self.unit
    }
}

impl From<PendingExtraTurn> for PendingExtraTurnView {
    fn from(value: PendingExtraTurn) -> Self {
        Self {
            insertion: value.insertion,
            unit: value.unit,
        }
    }
}

/// Named deterministic allocator cursors included in canonical battle state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SequenceCursorsView {
    values: [u64; 16],
}

impl SequenceCursorsView {
    pub(super) const fn new(values: [u64; 16]) -> Self {
        Self { values }
    }
    #[must_use]
    pub const fn next_unit(self) -> u64 {
        self.values[0]
    }
    #[must_use]
    pub const fn next_actor(self) -> u64 {
        self.values[1]
    }
    #[must_use]
    pub const fn next_spawn(self) -> u64 {
        self.values[2]
    }
    #[must_use]
    pub const fn next_wave(self) -> u64 {
        self.values[3]
    }
    #[must_use]
    pub const fn next_decision(self) -> u64 {
        self.values[4]
    }
    #[must_use]
    pub const fn next_command(self) -> u64 {
        self.values[5]
    }
    #[must_use]
    pub const fn next_event(self) -> u64 {
        self.values[6]
    }
    #[must_use]
    pub const fn next_action(self) -> u64 {
        self.values[7]
    }
    #[must_use]
    pub const fn next_phase(self) -> u64 {
        self.values[8]
    }
    #[must_use]
    pub const fn next_hit(self) -> u64 {
        self.values[9]
    }
    #[must_use]
    pub const fn next_operation(self) -> u64 {
        self.values[10]
    }
    #[must_use]
    pub const fn next_shield(self) -> u64 {
        self.values[11]
    }
    #[must_use]
    pub const fn next_effect(self) -> u64 {
        self.values[12]
    }
    #[must_use]
    pub const fn next_rule(self) -> u64 {
        self.values[13]
    }
    #[must_use]
    pub const fn next_modifier(self) -> u64 {
        self.values[14]
    }
    #[must_use]
    pub const fn next_extra_turn(self) -> u64 {
        self.values[15]
    }
}
