use starclock_activity::{ActivityStateHash, ActivityTerminalOutcome, BattleOutcome, EventDigest};
use starclock_combat::{ActionValue, BattleEvent, BattleStateHash, Command, Ratio};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CurrencyWarsBaselineTraceController {
    System = 0,
    Player = 1,
    Enemy = 2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBaselineTraceEntry {
    pub(super) controller: CurrencyWarsBaselineTraceController,
    pub(super) command: Command,
    pub(super) state_hash: BattleStateHash,
    pub(super) events: Box<[BattleEvent]>,
}

impl CurrencyWarsBaselineTraceEntry {
    #[must_use]
    pub const fn controller(&self) -> CurrencyWarsBaselineTraceController {
        self.controller
    }

    #[must_use]
    pub const fn command(&self) -> &Command {
        &self.command
    }

    #[must_use]
    pub const fn state_hash(&self) -> BattleStateHash {
        self.state_hash
    }

    #[must_use]
    pub fn events(&self) -> &[BattleEvent] {
        &self.events
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CurrencyWarsBaselineActivityAction {
    EngageEncounter = 1,
    PrepareBattle = 2,
    ContinueSupply = 3,
    ContinuePlane = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBaselineActivityTraceEntry {
    pub(super) action: CurrencyWarsBaselineActivityAction,
    pub(super) state_hash: ActivityStateHash,
    pub(super) battle_index: Option<u32>,
}

impl CurrencyWarsBaselineActivityTraceEntry {
    #[must_use]
    pub const fn new(
        action: CurrencyWarsBaselineActivityAction,
        state_hash: ActivityStateHash,
        battle_index: Option<u32>,
    ) -> Self {
        Self {
            action,
            state_hash,
            battle_index,
        }
    }

    #[must_use]
    pub const fn action(self) -> CurrencyWarsBaselineActivityAction {
        self.action
    }

    #[must_use]
    pub const fn state_hash(self) -> ActivityStateHash {
        self.state_hash
    }

    #[must_use]
    pub const fn battle_index(self) -> Option<u32> {
        self.battle_index
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBaselineBattleReport {
    pub(super) catalog_digest: [u8; 32],
    pub(super) combat_input_digest: [u8; 32],
    pub(super) assembly_digest: [u8; 32],
    pub(super) outcome: BattleOutcome,
    pub(super) final_state_hash: BattleStateHash,
    pub(super) event_digest: EventDigest,
    pub(super) progress: Ratio,
    pub(super) remaining_action_value: ActionValue,
    pub(super) trace: Box<[CurrencyWarsBaselineTraceEntry]>,
}

impl CurrencyWarsBaselineBattleReport {
    #[must_use]
    pub const fn catalog_digest(&self) -> [u8; 32] {
        self.catalog_digest
    }

    #[must_use]
    pub const fn combat_input_digest(&self) -> [u8; 32] {
        self.combat_input_digest
    }

    #[must_use]
    pub const fn assembly_digest(&self) -> [u8; 32] {
        self.assembly_digest
    }
    #[must_use]
    pub const fn outcome(&self) -> BattleOutcome {
        self.outcome
    }

    #[must_use]
    pub const fn final_state_hash(&self) -> BattleStateHash {
        self.final_state_hash
    }

    #[must_use]
    pub const fn event_digest(&self) -> EventDigest {
        self.event_digest
    }

    #[must_use]
    pub const fn progress(&self) -> Ratio {
        self.progress
    }

    #[must_use]
    pub const fn remaining_action_value(&self) -> ActionValue {
        self.remaining_action_value
    }

    #[must_use]
    pub fn trace(&self) -> &[CurrencyWarsBaselineTraceEntry] {
        &self.trace
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBaselineRunReport {
    pub(super) terminal: ActivityTerminalOutcome,
    pub(super) final_state_hash: ActivityStateHash,
    pub(super) activity_steps: u32,
    pub(super) supply_decisions: u32,
    pub(super) route_decisions: u32,
    pub(super) activity_trace: Box<[CurrencyWarsBaselineActivityTraceEntry]>,
    pub(super) battles: Box<[CurrencyWarsBaselineBattleReport]>,
}

impl CurrencyWarsBaselineRunReport {
    pub fn new(
        terminal: ActivityTerminalOutcome,
        final_state_hash: ActivityStateHash,
        supply_decisions: u32,
        route_decisions: u32,
        activity_trace: Vec<CurrencyWarsBaselineActivityTraceEntry>,
        battles: Vec<CurrencyWarsBaselineBattleReport>,
    ) -> Option<Self> {
        let activity_steps = u32::try_from(activity_trace.len()).ok()?;
        if activity_trace
            .iter()
            .filter_map(|entry| entry.battle_index)
            .any(|index| {
                index == 0 || usize::try_from(index).map_or(true, |value| value > battles.len())
            })
        {
            return None;
        }
        Some(Self {
            terminal,
            final_state_hash,
            activity_steps,
            supply_decisions,
            route_decisions,
            activity_trace: activity_trace.into_boxed_slice(),
            battles: battles.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn terminal(&self) -> ActivityTerminalOutcome {
        self.terminal
    }

    #[must_use]
    pub const fn final_state_hash(&self) -> ActivityStateHash {
        self.final_state_hash
    }

    #[must_use]
    pub const fn activity_steps(&self) -> u32 {
        self.activity_steps
    }

    #[must_use]
    pub const fn supply_decisions(&self) -> u32 {
        self.supply_decisions
    }

    #[must_use]
    pub const fn route_decisions(&self) -> u32 {
        self.route_decisions
    }

    #[must_use]
    pub fn activity_trace(&self) -> &[CurrencyWarsBaselineActivityTraceEntry] {
        &self.activity_trace
    }

    #[must_use]
    pub fn battles(&self) -> &[CurrencyWarsBaselineBattleReport] {
        &self.battles
    }
}
