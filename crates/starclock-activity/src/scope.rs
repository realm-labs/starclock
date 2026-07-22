use crate::{ActivityInstanceId, AttemptId, NodeId, SectionId};

/// Generic activity-owned lifetime. Battle and shorter lifetimes remain combat-owned.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ActivityScope {
    Activity = 0,
    Section = 1,
    Node = 2,
    Attempt = 3,
}

/// Exact generic scope path for the one battle supported by Goal 01.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ScopeIdentity {
    activity: ActivityInstanceId,
    section: SectionId,
    node: NodeId,
    attempt: AttemptId,
}

impl ScopeIdentity {
    #[must_use]
    pub const fn new(
        activity: ActivityInstanceId,
        section: SectionId,
        node: NodeId,
        attempt: AttemptId,
    ) -> Self {
        Self {
            activity,
            section,
            node,
            attempt,
        }
    }

    #[must_use]
    pub const fn activity(self) -> ActivityInstanceId {
        self.activity
    }
    #[must_use]
    pub const fn section(self) -> SectionId {
        self.section
    }
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.node
    }
    #[must_use]
    pub const fn attempt(self) -> AttemptId {
        self.attempt
    }
}

/// Generic terminal outcome for a minimum Activity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum TerminalOutcome {
    Complete = 0,
    Failed = 1,
    Faulted = 2,
}

/// Frozen one-Battle graph: one Section, one Battle node and three terminal nodes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OneBattleFlow {
    section: SectionId,
    battle: NodeId,
    complete: NodeId,
    failed: NodeId,
    faulted: NodeId,
}

impl OneBattleFlow {
    pub fn new(
        section: SectionId,
        battle: NodeId,
        complete: NodeId,
        failed: NodeId,
        faulted: NodeId,
    ) -> Result<Self, OneBattleFlowError> {
        let nodes = [battle, complete, failed, faulted];
        if nodes
            .iter()
            .enumerate()
            .any(|(index, node)| nodes[..index].contains(node))
        {
            return Err(OneBattleFlowError::DuplicateNode);
        }
        Ok(Self {
            section,
            battle,
            complete,
            failed,
            faulted,
        })
    }

    #[must_use]
    pub const fn section(self) -> SectionId {
        self.section
    }
    #[must_use]
    pub const fn battle_node(self) -> NodeId {
        self.battle
    }
    #[must_use]
    pub const fn terminal_node(self, outcome: TerminalOutcome) -> NodeId {
        match outcome {
            TerminalOutcome::Complete => self.complete,
            TerminalOutcome::Failed => self.failed,
            TerminalOutcome::Faulted => self.faulted,
        }
    }

    /// Compiles the legacy one-battle profile into the generic immutable graph.
    #[must_use]
    pub fn into_graph(self) -> crate::ActivityGraphDefinition {
        crate::ActivityGraphDefinition::one_battle(self)
    }

    pub(crate) fn encode(self, writer: &mut crate::codec::CanonicalWriter) {
        writer.u32(self.section.get());
        writer.u32(self.battle.get());
        writer.u32(self.complete.get());
        writer.u32(self.failed.get());
        writer.u32(self.faulted.get());
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OneBattleFlowError {
    DuplicateNode,
}
