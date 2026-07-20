use crate::id::{
    ActionId, CommandId, EventId, HitId, PhaseId, SourceDefinitionId, TimelineActorId, UnitId,
};

/// Runtime actor credited with performing one action or emitted fact.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CauseActor {
    /// A target-capable combat unit.
    Unit(UnitId),
    /// A timeline-only or unit-linked action actor.
    TimelineActor(TimelineActorId),
}

/// Complete immutable attribution carried by every battle event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cause {
    parent_event: Option<EventId>,
    root_command: CommandId,
    action: Option<ActionId>,
    phase: Option<PhaseId>,
    hit: Option<HitId>,
    owner: Option<UnitId>,
    actor: Option<CauseActor>,
    applier: Option<UnitId>,
    source_definition: Option<SourceDefinitionId>,
    primary_target: Option<UnitId>,
    activity_source: Option<SourceDefinitionId>,
}

impl Cause {
    pub(crate) const fn root(root_command: CommandId) -> Self {
        Self {
            parent_event: None,
            root_command,
            action: None,
            phase: None,
            hit: None,
            owner: None,
            actor: None,
            applier: None,
            source_definition: None,
            primary_target: None,
            activity_source: None,
        }
    }

    pub(crate) const fn with_parent(self, parent_event: EventId) -> Self {
        Self {
            parent_event: Some(parent_event),
            ..self
        }
    }

    pub(crate) const fn for_action(
        root_command: CommandId,
        action: ActionId,
        owner: UnitId,
        actor: CauseActor,
        source_definition: SourceDefinitionId,
    ) -> Self {
        Self {
            action: Some(action),
            owner: Some(owner),
            actor: Some(actor),
            source_definition: Some(source_definition),
            ..Self::root(root_command)
        }
    }

    pub(crate) const fn for_turn(
        root_command: CommandId,
        owner: UnitId,
        actor: TimelineActorId,
    ) -> Self {
        Self {
            owner: Some(owner),
            actor: Some(CauseActor::TimelineActor(actor)),
            ..Self::root(root_command)
        }
    }

    pub(crate) const fn with_phase(self, phase: PhaseId) -> Self {
        Self {
            phase: Some(phase),
            ..self
        }
    }

    pub(crate) const fn with_hit(self, hit: HitId) -> Self {
        Self {
            hit: Some(hit),
            ..self
        }
    }

    pub(crate) const fn with_primary_target(self, target: Option<UnitId>) -> Self {
        Self {
            primary_target: target,
            ..self
        }
    }

    pub(crate) const fn with_applier(self, applier: UnitId) -> Self {
        Self {
            applier: Some(applier),
            ..self
        }
    }

    pub(crate) const fn with_owner(self, owner: UnitId) -> Self {
        Self {
            owner: Some(owner),
            ..self
        }
    }

    pub(crate) const fn with_source_definition(
        self,
        source_definition: SourceDefinitionId,
    ) -> Self {
        Self {
            source_definition: Some(source_definition),
            ..self
        }
    }

    /// Returns the immediate event that caused this fact.
    #[must_use]
    pub const fn parent_event(self) -> Option<EventId> {
        self.parent_event
    }
    /// Returns the accepted command at the root of the complete chain.
    #[must_use]
    pub const fn root_command(self) -> CommandId {
        self.root_command
    }
    /// Returns the action envelope identity when one exists.
    #[must_use]
    pub const fn action(self) -> Option<ActionId> {
        self.action
    }
    /// Returns the authored action-phase identity when one exists.
    #[must_use]
    pub const fn phase(self) -> Option<PhaseId> {
        self.phase
    }
    /// Returns the authored hit identity when one exists.
    #[must_use]
    pub const fn hit(self) -> Option<HitId> {
        self.hit
    }
    /// Returns the unit that owns the responsible rule/source.
    #[must_use]
    pub const fn owner(self) -> Option<UnitId> {
        self.owner
    }
    /// Returns the unit or timeline actor performing the action.
    #[must_use]
    pub const fn actor(self) -> Option<CauseActor> {
        self.actor
    }
    /// Returns the unit receiving application credit.
    #[must_use]
    pub const fn applier(self) -> Option<UnitId> {
        self.applier
    }
    /// Returns the ability/effect/equipment/enemy/mode source identity.
    #[must_use]
    pub const fn source_definition(self) -> Option<SourceDefinitionId> {
        self.source_definition
    }
    /// Returns the primary target attached to this cause.
    #[must_use]
    pub const fn primary_target(self) -> Option<UnitId> {
        self.primary_target
    }
    /// Returns an optional opaque activity-supplied source identity.
    #[must_use]
    pub const fn activity_source(self) -> Option<SourceDefinitionId> {
        self.activity_source
    }
}
