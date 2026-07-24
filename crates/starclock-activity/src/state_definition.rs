use crate::{
    ActivityInstanceId, ActivityInventoryId, ActivityModifierId, ActivityScope,
    ActivitySlotDefinition, ActivitySlotId, AttemptId, LogicalScopeDefinitions, NodeId, SectionId,
};

pub const MAX_ACTIVITY_STATE_SLOTS: usize = 4_096;
pub const MAX_ACTIVITY_INVENTORIES: usize = 256;
pub const MAX_ACTIVITY_MODIFIERS: usize = 4_096;
pub const MAX_INVENTORY_ENTRIES: u32 = 4_096;
pub const MAX_INVENTORY_STACK: u32 = 1_000_000;

/// A generic scope path may stop at any currently entered Activity lifetime.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActivityScopePath {
    activity: ActivityInstanceId,
    section: Option<SectionId>,
    node: Option<NodeId>,
    attempt: Option<AttemptId>,
}

impl ActivityScopePath {
    #[must_use]
    pub const fn new(activity: ActivityInstanceId) -> Self {
        Self {
            activity,
            section: None,
            node: None,
            attempt: None,
        }
    }

    pub fn enter_section(self, section: SectionId) -> Result<Self, ActivityScopePathError> {
        if self.section.is_some() {
            return Err(ActivityScopePathError::SectionAlreadyEntered);
        }
        Ok(Self {
            section: Some(section),
            ..self
        })
    }

    pub fn enter_node(self, node: NodeId) -> Result<Self, ActivityScopePathError> {
        if self.section.is_none() {
            return Err(ActivityScopePathError::MissingSection);
        }
        if self.node.is_some() {
            return Err(ActivityScopePathError::NodeAlreadyEntered);
        }
        Ok(Self {
            node: Some(node),
            ..self
        })
    }

    pub fn enter_attempt(self, attempt: AttemptId) -> Result<Self, ActivityScopePathError> {
        if self.node.is_none() {
            return Err(ActivityScopePathError::MissingNode);
        }
        if self.attempt.is_some() {
            return Err(ActivityScopePathError::AttemptAlreadyEntered);
        }
        Ok(Self {
            attempt: Some(attempt),
            ..self
        })
    }

    pub fn leave_attempt(self) -> Result<Self, ActivityScopePathError> {
        if self.attempt.is_none() {
            return Err(ActivityScopePathError::MissingAttempt);
        }
        Ok(Self {
            attempt: None,
            ..self
        })
    }

    pub fn leave_node(self) -> Result<Self, ActivityScopePathError> {
        if self.attempt.is_some() {
            return Err(ActivityScopePathError::AttemptStillEntered);
        }
        if self.node.is_none() {
            return Err(ActivityScopePathError::MissingNode);
        }
        Ok(Self { node: None, ..self })
    }

    pub fn leave_section(self) -> Result<Self, ActivityScopePathError> {
        if self.node.is_some() {
            return Err(ActivityScopePathError::NodeStillEntered);
        }
        if self.section.is_none() {
            return Err(ActivityScopePathError::MissingSection);
        }
        Ok(Self {
            section: None,
            ..self
        })
    }

    #[must_use]
    pub const fn activity(self) -> ActivityInstanceId {
        self.activity
    }
    #[must_use]
    pub const fn section(self) -> Option<SectionId> {
        self.section
    }
    #[must_use]
    pub const fn node(self) -> Option<NodeId> {
        self.node
    }
    #[must_use]
    pub const fn attempt(self) -> Option<AttemptId> {
        self.attempt
    }

    #[must_use]
    pub const fn active_scope(self) -> ActivityScope {
        if self.attempt.is_some() {
            ActivityScope::Attempt
        } else if self.node.is_some() {
            ActivityScope::Node
        } else if self.section.is_some() {
            ActivityScope::Section
        } else {
            ActivityScope::Activity
        }
    }

    pub fn identity(
        self,
        scope: ActivityScope,
    ) -> Result<ActivityScopeIdentity, ActivityScopePathError> {
        match scope {
            ActivityScope::Activity => Ok(ActivityScopeIdentity::Activity(self.activity)),
            ActivityScope::Section => Ok(ActivityScopeIdentity::Section {
                activity: self.activity,
                section: self.section.ok_or(ActivityScopePathError::MissingSection)?,
            }),
            ActivityScope::Node => Ok(ActivityScopeIdentity::Node {
                activity: self.activity,
                section: self.section.ok_or(ActivityScopePathError::MissingSection)?,
                node: self.node.ok_or(ActivityScopePathError::MissingNode)?,
            }),
            ActivityScope::Attempt => Ok(ActivityScopeIdentity::Attempt {
                activity: self.activity,
                section: self.section.ok_or(ActivityScopePathError::MissingSection)?,
                node: self.node.ok_or(ActivityScopePathError::MissingNode)?,
                attempt: self.attempt.ok_or(ActivityScopePathError::MissingAttempt)?,
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ActivityScopeIdentity {
    Activity(ActivityInstanceId),
    Section {
        activity: ActivityInstanceId,
        section: SectionId,
    },
    Node {
        activity: ActivityInstanceId,
        section: SectionId,
        node: NodeId,
    },
    Attempt {
        activity: ActivityInstanceId,
        section: SectionId,
        node: NodeId,
        attempt: AttemptId,
    },
}

impl ActivityScopeIdentity {
    #[must_use]
    pub const fn scope(self) -> ActivityScope {
        match self {
            Self::Activity(_) => ActivityScope::Activity,
            Self::Section { .. } => ActivityScope::Section,
            Self::Node { .. } => ActivityScope::Node,
            Self::Attempt { .. } => ActivityScope::Attempt,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityScopePathError {
    MissingSection,
    MissingNode,
    MissingAttempt,
    SectionAlreadyEntered,
    NodeAlreadyEntered,
    AttemptAlreadyEntered,
    AttemptStillEntered,
    NodeStillEntered,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ActivitySnapshotBoundary {
    SectionExit = 0,
    NodeExit = 1,
    AttemptExit = 2,
    BattleEnd = 3,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ActivityAccumulationPolicy {
    Sum = 0,
    Minimum = 1,
    Maximum = 2,
    OrderedAppend = 3,
}

/// Cross-scope policy is explicit even when its expression is implemented in P2-B3.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SlotCarryPolicy {
    Reset,
    CarryExact,
    CarryClamped,
    Project,
    Accumulate(ActivityAccumulationPolicy),
    Replace,
    Snapshot(ActivitySnapshotBoundary),
    Discard,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ActivityStateVisibility {
    Player = 0,
    DebugOnly = 1,
    Private = 2,
}

/// Stable, non-text provenance key for an authored state definition.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActivityStateSource(u64);

impl ActivityStateSource {
    #[must_use]
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityInventoryDefinition {
    id: ActivityInventoryId,
    owner: ActivityScope,
    maximum_entries: u32,
    maximum_stack: u32,
    carry: SlotCarryPolicy,
    visibility: ActivityStateVisibility,
    source: ActivityStateSource,
}

impl ActivityInventoryDefinition {
    pub fn new(
        id: ActivityInventoryId,
        owner: ActivityScope,
        maximum_entries: u32,
        maximum_stack: u32,
        carry: SlotCarryPolicy,
        visibility: ActivityStateVisibility,
        source: ActivityStateSource,
    ) -> Result<Self, ActivityStateDefinitionError> {
        if maximum_entries == 0 || maximum_entries > MAX_INVENTORY_ENTRIES {
            return Err(ActivityStateDefinitionError::InvalidInventoryEntryLimit(id));
        }
        if maximum_stack == 0 || maximum_stack > MAX_INVENTORY_STACK {
            return Err(ActivityStateDefinitionError::InvalidInventoryStackLimit(id));
        }
        validate_policy(owner, carry)?;
        Ok(Self {
            id,
            owner,
            maximum_entries,
            maximum_stack,
            carry,
            visibility,
            source,
        })
    }

    #[must_use]
    pub const fn id(self) -> ActivityInventoryId {
        self.id
    }
    #[must_use]
    pub const fn owner(self) -> ActivityScope {
        self.owner
    }
    #[must_use]
    pub const fn maximum_entries(self) -> u32 {
        self.maximum_entries
    }
    #[must_use]
    pub const fn maximum_stack(self) -> u32 {
        self.maximum_stack
    }
    #[must_use]
    pub const fn carry(self) -> SlotCarryPolicy {
        self.carry
    }
    #[must_use]
    pub const fn visibility(self) -> ActivityStateVisibility {
        self.visibility
    }
    #[must_use]
    pub const fn source(self) -> ActivityStateSource {
        self.source
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityModifierOwner {
    Scope(ActivityScope),
    Inventory(ActivityInventoryId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityModifierDefinition {
    id: ActivityModifierId,
    owner: ActivityModifierOwner,
    stacking_group: u64,
    maximum_stacks: u32,
    carry: SlotCarryPolicy,
    source: ActivityStateSource,
}

impl ActivityModifierDefinition {
    pub fn new(
        id: ActivityModifierId,
        owner: ActivityModifierOwner,
        stacking_group: u64,
        maximum_stacks: u32,
        carry: SlotCarryPolicy,
        source: ActivityStateSource,
    ) -> Result<Self, ActivityStateDefinitionError> {
        if stacking_group == 0 {
            return Err(ActivityStateDefinitionError::InvalidModifierStackingGroup(
                id,
            ));
        }
        if maximum_stacks == 0 || maximum_stacks > MAX_INVENTORY_STACK {
            return Err(ActivityStateDefinitionError::InvalidModifierStackLimit(id));
        }
        if let ActivityModifierOwner::Scope(scope) = owner {
            validate_policy(scope, carry)?;
        }
        Ok(Self {
            id,
            owner,
            stacking_group,
            maximum_stacks,
            carry,
            source,
        })
    }

    #[must_use]
    pub const fn id(self) -> ActivityModifierId {
        self.id
    }
    #[must_use]
    pub const fn owner(self) -> ActivityModifierOwner {
        self.owner
    }
    #[must_use]
    pub const fn stacking_group(self) -> u64 {
        self.stacking_group
    }
    #[must_use]
    pub const fn maximum_stacks(self) -> u32 {
        self.maximum_stacks
    }
    #[must_use]
    pub const fn carry(self) -> SlotCarryPolicy {
        self.carry
    }
    #[must_use]
    pub const fn source(self) -> ActivityStateSource {
        self.source
    }
}

/// Canonical collection of state contracts. It contains no mutable values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityStateDefinition {
    slots: Box<[ActivitySlotDefinition]>,
    inventories: Box<[ActivityInventoryDefinition]>,
    modifiers: Box<[ActivityModifierDefinition]>,
    logical_scopes: LogicalScopeDefinitions,
}

impl ActivityStateDefinition {
    pub fn new(
        mut slots: Vec<ActivitySlotDefinition>,
        mut inventories: Vec<ActivityInventoryDefinition>,
        mut modifiers: Vec<ActivityModifierDefinition>,
    ) -> Result<Self, ActivityStateDefinitionError> {
        if slots.len() > MAX_ACTIVITY_STATE_SLOTS {
            return Err(ActivityStateDefinitionError::TooManySlots);
        }
        if inventories.len() > MAX_ACTIVITY_INVENTORIES {
            return Err(ActivityStateDefinitionError::TooManyInventories);
        }
        if modifiers.len() > MAX_ACTIVITY_MODIFIERS {
            return Err(ActivityStateDefinitionError::TooManyModifiers);
        }
        slots.sort_by_key(ActivitySlotDefinition::id);
        inventories.sort_by_key(|item| item.id());
        modifiers.sort_by_key(|item| item.id());
        if let Some(pair) = slots.windows(2).find(|pair| pair[0].id() == pair[1].id()) {
            return Err(ActivityStateDefinitionError::DuplicateSlot(pair[0].id()));
        }
        if let Some(pair) = inventories
            .windows(2)
            .find(|pair| pair[0].id() == pair[1].id())
        {
            return Err(ActivityStateDefinitionError::DuplicateInventory(
                pair[0].id(),
            ));
        }
        if let Some(pair) = modifiers
            .windows(2)
            .find(|pair| pair[0].id() == pair[1].id())
        {
            return Err(ActivityStateDefinitionError::DuplicateModifier(
                pair[0].id(),
            ));
        }
        for modifier in &modifiers {
            if let ActivityModifierOwner::Inventory(owner) = modifier.owner {
                let inventory = inventories
                    .binary_search_by_key(&owner, |item| item.id())
                    .ok()
                    .map(|index| inventories[index])
                    .ok_or(ActivityStateDefinitionError::MissingModifierInventory(
                        modifier.id,
                    ))?;
                validate_policy(inventory.owner, modifier.carry)?;
            }
        }
        Ok(Self {
            slots: slots.into_boxed_slice(),
            inventories: inventories.into_boxed_slice(),
            modifiers: modifiers.into_boxed_slice(),
            logical_scopes: LogicalScopeDefinitions::default(),
        })
    }

    #[must_use]
    pub fn with_logical_scopes(mut self, logical_scopes: LogicalScopeDefinitions) -> Self {
        self.logical_scopes = logical_scopes;
        self
    }

    #[must_use]
    pub fn slots(&self) -> &[ActivitySlotDefinition] {
        &self.slots
    }
    #[must_use]
    pub fn inventories(&self) -> &[ActivityInventoryDefinition] {
        &self.inventories
    }
    #[must_use]
    pub fn modifiers(&self) -> &[ActivityModifierDefinition] {
        &self.modifiers
    }
    #[must_use]
    pub const fn logical_scopes(&self) -> &LogicalScopeDefinitions {
        &self.logical_scopes
    }
}

pub(crate) fn validate_policy(
    owner: ActivityScope,
    carry: SlotCarryPolicy,
) -> Result<(), ActivityStateDefinitionError> {
    if let SlotCarryPolicy::Snapshot(boundary) = carry {
        let boundary_scope = match boundary {
            ActivitySnapshotBoundary::SectionExit => ActivityScope::Section,
            ActivitySnapshotBoundary::NodeExit => ActivityScope::Node,
            ActivitySnapshotBoundary::AttemptExit | ActivitySnapshotBoundary::BattleEnd => {
                ActivityScope::Attempt
            }
        };
        if boundary_scope < owner {
            return Err(ActivityStateDefinitionError::SnapshotBeforeOwnerExit);
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityStateDefinitionError {
    TooManySlots,
    TooManyInventories,
    TooManyModifiers,
    DuplicateSlot(ActivitySlotId),
    DuplicateInventory(ActivityInventoryId),
    DuplicateModifier(ActivityModifierId),
    InvalidInventoryEntryLimit(ActivityInventoryId),
    InvalidInventoryStackLimit(ActivityInventoryId),
    InvalidModifierStackingGroup(ActivityModifierId),
    InvalidModifierStackLimit(ActivityModifierId),
    MissingModifierInventory(ActivityModifierId),
    SnapshotBeforeOwnerExit,
}
