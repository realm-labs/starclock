//! Bounded logical lifetimes layered over physical Activity graph nodes.

use std::collections::BTreeMap;

use crate::{ActivityGraphDefinition, LogicalScopeClassId, NodeId, codec::ActivityStateEncoder};

pub const ACTIVITY_LOGICAL_SCOPE_REVISION: &str = "activity-logical-scope-v1";
pub const MAX_LOGICAL_SCOPE_CLASSES: usize = 64;
pub const MAX_LOGICAL_SCOPE_BINDINGS: usize = 16_384;
pub const MAX_LOGICAL_SCOPE_DEPTH: usize = 16;
pub const MAX_LOGICAL_SCOPE_INSTANCES: u32 = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LogicalScopeAddress {
    class: LogicalScopeClassId,
    key: u64,
}

impl LogicalScopeAddress {
    #[must_use]
    pub const fn new(class: LogicalScopeClassId, key: u64) -> Option<Self> {
        if key == 0 {
            None
        } else {
            Some(Self { class, key })
        }
    }

    #[must_use]
    pub const fn class(self) -> LogicalScopeClassId {
        self.class
    }

    #[must_use]
    pub const fn key(self) -> u64 {
        self.key
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LogicalScopeClassDefinition {
    id: LogicalScopeClassId,
    parent: Option<LogicalScopeClassId>,
    maximum_instances: u32,
}

impl LogicalScopeClassDefinition {
    #[must_use]
    pub const fn new(
        id: LogicalScopeClassId,
        parent: Option<LogicalScopeClassId>,
        maximum_instances: u32,
    ) -> Option<Self> {
        if maximum_instances == 0
            || maximum_instances > MAX_LOGICAL_SCOPE_INSTANCES
            || matches!(parent, Some(parent) if parent.get() == id.get())
        {
            None
        } else {
            Some(Self {
                id,
                parent,
                maximum_instances,
            })
        }
    }

    #[must_use]
    pub const fn id(self) -> LogicalScopeClassId {
        self.id
    }

    #[must_use]
    pub const fn parent(self) -> Option<LogicalScopeClassId> {
        self.parent
    }

    #[must_use]
    pub const fn maximum_instances(self) -> u32 {
        self.maximum_instances
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogicalScopeNodeBinding {
    node: NodeId,
    path: Box<[LogicalScopeAddress]>,
}

impl LogicalScopeNodeBinding {
    pub fn new(
        node: NodeId,
        path: Vec<LogicalScopeAddress>,
    ) -> Result<Self, LogicalScopeDefinitionError> {
        if path.is_empty() || path.len() > MAX_LOGICAL_SCOPE_DEPTH {
            return Err(LogicalScopeDefinitionError::InvalidPath);
        }
        Ok(Self {
            node,
            path: path.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }

    #[must_use]
    pub fn path(&self) -> &[LogicalScopeAddress] {
        &self.path
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LogicalScopeDefinitions {
    classes: Box<[LogicalScopeClassDefinition]>,
    bindings: Box<[LogicalScopeNodeBinding]>,
}

impl LogicalScopeDefinitions {
    pub fn new(
        mut classes: Vec<LogicalScopeClassDefinition>,
        mut bindings: Vec<LogicalScopeNodeBinding>,
    ) -> Result<Self, LogicalScopeDefinitionError> {
        if classes.len() > MAX_LOGICAL_SCOPE_CLASSES {
            return Err(LogicalScopeDefinitionError::TooManyClasses);
        }
        if bindings.len() > MAX_LOGICAL_SCOPE_BINDINGS {
            return Err(LogicalScopeDefinitionError::TooManyBindings);
        }
        classes.sort_unstable_by_key(|definition| definition.id);
        bindings.sort_unstable_by_key(|binding| binding.node);
        if classes.windows(2).any(|pair| pair[0].id == pair[1].id) {
            return Err(LogicalScopeDefinitionError::DuplicateClass);
        }
        if bindings.windows(2).any(|pair| pair[0].node == pair[1].node) {
            return Err(LogicalScopeDefinitionError::DuplicateNodeBinding);
        }
        for class in &classes {
            if let Some(parent) = class.parent {
                let parent_index = classes
                    .binary_search_by_key(&parent, |candidate| candidate.id)
                    .map_err(|_| LogicalScopeDefinitionError::MissingParent)?;
                let class_index = classes
                    .binary_search_by_key(&class.id, |candidate| candidate.id)
                    .expect("current class exists");
                if parent_index >= class_index {
                    return Err(LogicalScopeDefinitionError::InvalidParentOrder);
                }
            }
        }
        for binding in &bindings {
            validate_path(&classes, binding.path())?;
        }
        Ok(Self {
            classes: classes.into_boxed_slice(),
            bindings: bindings.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }

    #[must_use]
    pub fn classes(&self) -> &[LogicalScopeClassDefinition] {
        &self.classes
    }

    #[must_use]
    pub fn bindings(&self) -> &[LogicalScopeNodeBinding] {
        &self.bindings
    }

    pub(crate) fn validate_graph(
        &self,
        graph: &ActivityGraphDefinition,
    ) -> Result<(), LogicalScopeDefinitionError> {
        if self
            .bindings
            .iter()
            .any(|binding| graph.node(binding.node).is_none())
        {
            return Err(LogicalScopeDefinitionError::MissingNode);
        }
        Ok(())
    }

    pub(crate) fn path(&self, node: NodeId) -> &[LogicalScopeAddress] {
        self.bindings
            .binary_search_by_key(&node, |binding| binding.node)
            .ok()
            .map(|index| self.bindings[index].path())
            .unwrap_or(&[])
    }

    fn class(&self, id: LogicalScopeClassId) -> LogicalScopeClassDefinition {
        let index = self
            .classes
            .binary_search_by_key(&id, |definition| definition.id)
            .expect("validated logical path references an existing class");
        self.classes[index]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LogicalScopeInstance {
    address: LogicalScopeAddress,
    visit_sequence: u32,
}

impl LogicalScopeInstance {
    #[must_use]
    pub const fn address(self) -> LogicalScopeAddress {
        self.address
    }

    #[must_use]
    pub const fn visit_sequence(self) -> u32 {
        self.visit_sequence
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct LogicalScopeRuntimeState {
    active: Vec<LogicalScopeInstance>,
    class_entries: BTreeMap<LogicalScopeClassId, u32>,
    address_visits: BTreeMap<LogicalScopeAddress, u32>,
}

impl LogicalScopeRuntimeState {
    pub(crate) fn new(
        definitions: &LogicalScopeDefinitions,
        entry: NodeId,
    ) -> Result<Self, LogicalScopeRuntimeError> {
        let mut state = Self::default();
        state.transition(definitions, entry)?;
        Ok(state)
    }

    pub(crate) fn transition(
        &mut self,
        definitions: &LogicalScopeDefinitions,
        target: NodeId,
    ) -> Result<(), LogicalScopeRuntimeError> {
        let target = definitions.path(target);
        let common = self
            .active
            .iter()
            .map(|instance| instance.address)
            .zip(target.iter().copied())
            .take_while(|(left, right)| left == right)
            .count();
        self.active.truncate(common);
        for address in &target[common..] {
            let class = definitions.class(address.class);
            let entries = self
                .class_entries
                .get(&address.class)
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or(LogicalScopeRuntimeError::InstanceLimit)?;
            if entries > class.maximum_instances {
                return Err(LogicalScopeRuntimeError::InstanceLimit);
            }
            let visit = self
                .address_visits
                .get(address)
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or(LogicalScopeRuntimeError::InstanceLimit)?;
            self.class_entries.insert(address.class, entries);
            self.address_visits.insert(*address, visit);
            self.active.push(LogicalScopeInstance {
                address: *address,
                visit_sequence: visit,
            });
        }
        Ok(())
    }

    #[must_use]
    pub(crate) fn active(&self) -> &[LogicalScopeInstance] {
        &self.active
    }

    pub(crate) fn encode(&self, writer: &mut ActivityStateEncoder) {
        writer.text(ACTIVITY_LOGICAL_SCOPE_REVISION);
        writer.u32(self.active.len() as u32);
        for instance in &self.active {
            writer.u32(instance.address.class.get());
            writer.u64(instance.address.key);
            writer.u32(instance.visit_sequence);
        }
        writer.u32(self.class_entries.len() as u32);
        for (class, entries) in &self.class_entries {
            writer.u32(class.get());
            writer.u32(*entries);
        }
        writer.u32(self.address_visits.len() as u32);
        for (address, visits) in &self.address_visits {
            writer.u32(address.class.get());
            writer.u64(address.key);
            writer.u32(*visits);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogicalScopeDefinitionError {
    TooManyClasses,
    TooManyBindings,
    DuplicateClass,
    DuplicateNodeBinding,
    MissingParent,
    InvalidParentOrder,
    MissingClass,
    InvalidPath,
    MissingNode,
}

impl core::fmt::Display for LogicalScopeDefinitionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "logical scope definition error: {self:?}")
    }
}

impl std::error::Error for LogicalScopeDefinitionError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LogicalScopeRuntimeError {
    InstanceLimit,
}

fn validate_path(
    classes: &[LogicalScopeClassDefinition],
    path: &[LogicalScopeAddress],
) -> Result<(), LogicalScopeDefinitionError> {
    for (index, address) in path.iter().enumerate() {
        let class = classes
            .binary_search_by_key(&address.class, |definition| definition.id)
            .ok()
            .map(|class_index| classes[class_index])
            .ok_or(LogicalScopeDefinitionError::MissingClass)?;
        let expected_parent = index
            .checked_sub(1)
            .map(|parent_index| path[parent_index].class);
        if class.parent != expected_parent {
            return Err(LogicalScopeDefinitionError::InvalidPath);
        }
    }
    Ok(())
}
