use starclock_combat::{
    ActionValue, ActionValueClockSpec, BattleClockExpiry, BattleClockSpec, CycleClockSpec,
};

/// Authored cycle-window policy compiled into each battle-local clock.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CycleClockRule {
    initial_cycles: u16,
    first_window: ActionValue,
    later_window: ActionValue,
    reset_window_on_wave: bool,
    expiry: BattleClockExpiry,
}

impl CycleClockRule {
    #[must_use]
    pub fn new(
        initial_cycles: u16,
        first_window: ActionValue,
        later_window: ActionValue,
        reset_window_on_wave: bool,
        expiry: BattleClockExpiry,
    ) -> Option<Self> {
        CycleClockSpec::new(
            initial_cycles,
            first_window,
            later_window,
            reset_window_on_wave,
            expiry,
        )?;
        Some(Self {
            initial_cycles,
            first_window,
            later_window,
            reset_window_on_wave,
            expiry,
        })
    }

    #[must_use]
    pub const fn initial_cycles(self) -> u16 {
        self.initial_cycles
    }

    #[must_use]
    pub fn compile(self, remaining_cycles: u16) -> Option<BattleClockSpec> {
        CycleClockSpec::new(
            remaining_cycles,
            self.first_window,
            self.later_window,
            self.reset_window_on_wave,
            self.expiry,
        )
        .map(BattleClockSpec::Cycles)
    }
}

/// Authored exact Action Value budget compiled into one boss node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionValueClockRule {
    initial: ActionValue,
    expiry: BattleClockExpiry,
}

impl ActionValueClockRule {
    #[must_use]
    pub fn new(initial: ActionValue, expiry: BattleClockExpiry) -> Option<Self> {
        ActionValueClockSpec::new(initial, expiry)?;
        Some(Self { initial, expiry })
    }

    #[must_use]
    pub const fn initial(self) -> ActionValue {
        self.initial
    }

    #[must_use]
    pub fn compile(self, remaining: ActionValue) -> Option<BattleClockSpec> {
        ActionValueClockSpec::new(remaining, self.expiry).map(BattleClockSpec::ActionValue)
    }
}
