/// Life axis for a unit; zero HP does not collapse presence into this enum.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LifeState {
    /// Eligible living unit.
    Alive,
    /// Zero-HP candidate while replacements/revival settle.
    Downed,
    /// Settled defeat record retained for attribution/revival policy.
    Defeated,
}

/// Independent battlefield-presence axis.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PresenceState {
    /// Occupies its ordinary battlefield role and may be targetable.
    Present,
    /// Retained outside the active formation.
    Reserved,
    /// Removed from the current battlefield while state remains recorded.
    Departed,
    /// Present for lifecycle purposes but not ordinarily targetable.
    Untargetable,
    /// Linked entity without an ordinary occupied formation role.
    Linked,
    /// Temporarily represented through an authored transformed state.
    Transformed,
}
