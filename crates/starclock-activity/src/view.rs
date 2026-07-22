use crate::{
    ActivityDecisionId, ActivityDecisionKind, ActivityEdgeId, ActivityInventoryId,
    ActivityModifierId, ActivityOptionId, ActivityRngStreamSnapshot, ActivitySlotId,
    ActivityStateHash, ActivityTerminalOutcome, ActivityValue, NodeId,
    battle_preparation::{ActivityPendingBattleView, ActivityPreparationView},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivitySlotView {
    pub(crate) id: ActivitySlotId,
    pub(crate) value: ActivityValue,
}
impl ActivitySlotView {
    #[must_use]
    pub const fn id(&self) -> ActivitySlotId {
        self.id
    }
    #[must_use]
    pub const fn value(&self) -> &ActivityValue {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityInventoryView {
    pub(crate) id: ActivityInventoryId,
    pub(crate) entries: Box<[(u64, u32)]>,
}
impl ActivityInventoryView {
    #[must_use]
    pub const fn id(&self) -> ActivityInventoryId {
        self.id
    }
    #[must_use]
    pub fn entries(&self) -> &[(u64, u32)] {
        &self.entries
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityOptionView {
    pub(crate) id: ActivityOptionId,
    pub(crate) priority: i32,
}
impl ActivityOptionView {
    #[must_use]
    pub const fn id(self) -> ActivityOptionId {
        self.id
    }
    #[must_use]
    pub const fn priority(self) -> i32 {
        self.priority
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityDecisionView {
    pub(crate) id: ActivityDecisionId,
    pub(crate) kind: ActivityDecisionKind,
    pub(crate) options: Box<[ActivityOptionView]>,
}
impl ActivityDecisionView {
    #[must_use]
    pub const fn id(&self) -> ActivityDecisionId {
        self.id
    }
    #[must_use]
    pub const fn kind(&self) -> ActivityDecisionKind {
        self.kind
    }
    #[must_use]
    pub fn options(&self) -> &[ActivityOptionView] {
        &self.options
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityPlayerView {
    pub(crate) current_node: NodeId,
    pub(crate) command_sequence: u64,
    pub(crate) slots: Box<[ActivitySlotView]>,
    pub(crate) inventories: Box<[ActivityInventoryView]>,
    pub(crate) decision: Option<ActivityDecisionView>,
    pub(crate) preparation: Option<ActivityPreparationView>,
    pub(crate) pending_battle: Option<ActivityPendingBattleView>,
    pub(crate) terminal: Option<ActivityTerminalOutcome>,
    pub(crate) state_hash: ActivityStateHash,
}
impl ActivityPlayerView {
    #[must_use]
    pub const fn current_node(&self) -> NodeId {
        self.current_node
    }
    #[must_use]
    pub const fn command_sequence(&self) -> u64 {
        self.command_sequence
    }
    #[must_use]
    pub fn slots(&self) -> &[ActivitySlotView] {
        &self.slots
    }
    #[must_use]
    pub fn inventories(&self) -> &[ActivityInventoryView] {
        &self.inventories
    }
    #[must_use]
    pub const fn decision(&self) -> Option<&ActivityDecisionView> {
        self.decision.as_ref()
    }
    #[must_use]
    pub const fn preparation(&self) -> Option<&ActivityPreparationView> {
        self.preparation.as_ref()
    }
    #[must_use]
    pub const fn pending_battle(&self) -> Option<&ActivityPendingBattleView> {
        self.pending_battle.as_ref()
    }
    #[must_use]
    pub const fn terminal(&self) -> Option<ActivityTerminalOutcome> {
        self.terminal
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityDebugView {
    pub(crate) player: ActivityPlayerView,
    pub(crate) all_slots: Box<[ActivitySlotView]>,
    pub(crate) all_inventories: Box<[ActivityInventoryView]>,
    pub(crate) modifiers: Box<[(ActivityModifierId, u32)]>,
    pub(crate) node_visits: Box<[(NodeId, u32)]>,
    pub(crate) edge_traversals: Box<[(ActivityEdgeId, u32)]>,
    pub(crate) rng: Box<[ActivityRngStreamSnapshot]>,
}
impl ActivityDebugView {
    #[must_use]
    pub const fn player(&self) -> &ActivityPlayerView {
        &self.player
    }
    #[must_use]
    pub fn all_slots(&self) -> &[ActivitySlotView] {
        &self.all_slots
    }
    #[must_use]
    pub fn all_inventories(&self) -> &[ActivityInventoryView] {
        &self.all_inventories
    }
    #[must_use]
    pub fn modifiers(&self) -> &[(ActivityModifierId, u32)] {
        &self.modifiers
    }
    #[must_use]
    pub fn node_visits(&self) -> &[(NodeId, u32)] {
        &self.node_visits
    }
    #[must_use]
    pub fn edge_traversals(&self) -> &[(ActivityEdgeId, u32)] {
        &self.edge_traversals
    }
    #[must_use]
    pub fn rng(&self) -> &[ActivityRngStreamSnapshot] {
        &self.rng
    }
}
