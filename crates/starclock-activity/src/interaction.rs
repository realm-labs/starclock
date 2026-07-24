//! Validated bindings from offered external outcomes to composed handlers.

use std::sync::Arc;

use crate::{
    ActivityDecisionKind, ActivityExternalOutcomeId, ActivityGraphDefinition, ActivityHandlerId,
    ActivityHandlerRegistry, ActivityOptionId, GraphActivityNodeProgram,
    MAX_ACTIVITY_HANDLER_PAYLOAD_BYTES, NodeId,
};

pub const MAX_ACTIVITY_INTERACTION_BINDINGS: usize = 16_384;
pub const MAX_ACTIVITY_COMPONENT_ID_BYTES: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityInteractionBinding {
    node: NodeId,
    offered_outcome: ActivityExternalOutcomeId,
    handler: ActivityHandlerId,
    payload: Box<[u8]>,
    component_id: Box<str>,
}

impl ActivityInteractionBinding {
    pub fn new(
        node: NodeId,
        offered_outcome: ActivityExternalOutcomeId,
        handler: ActivityHandlerId,
        payload: Vec<u8>,
        component_id: impl Into<Box<str>>,
    ) -> Result<Self, ActivityInteractionBindingError> {
        let component_id = component_id.into();
        if payload.len() > MAX_ACTIVITY_HANDLER_PAYLOAD_BYTES {
            return Err(ActivityInteractionBindingError::PayloadTooLarge);
        }
        validate_component_id(&component_id)?;
        Ok(Self {
            node,
            offered_outcome,
            handler,
            payload: payload.into_boxed_slice(),
            component_id,
        })
    }

    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }

    #[must_use]
    pub const fn offered_outcome(&self) -> ActivityExternalOutcomeId {
        self.offered_outcome
    }

    #[must_use]
    pub const fn handler(&self) -> ActivityHandlerId {
        self.handler
    }

    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    #[must_use]
    pub fn component_id(&self) -> &str {
        &self.component_id
    }
}

#[derive(Clone, Debug)]
pub struct ActivityInteractionBindings {
    registry: Arc<ActivityHandlerRegistry>,
    bindings: Arc<[ActivityInteractionBinding]>,
}

impl ActivityInteractionBindings {
    pub fn new(
        registry: ActivityHandlerRegistry,
        mut bindings: Vec<ActivityInteractionBinding>,
        graph: &ActivityGraphDefinition,
        programs: &[GraphActivityNodeProgram],
    ) -> Result<Self, ActivityInteractionBindingError> {
        if bindings.len() > MAX_ACTIVITY_INTERACTION_BINDINGS {
            return Err(ActivityInteractionBindingError::TooManyBindings);
        }
        bindings.sort_unstable_by_key(binding_key);
        if bindings
            .windows(2)
            .any(|pair| binding_key(&pair[0]) == binding_key(&pair[1]))
        {
            return Err(ActivityInteractionBindingError::DuplicateBinding);
        }
        for binding in &bindings {
            validate_binding(binding, &registry, graph, programs)?;
        }
        Ok(Self {
            registry: Arc::new(registry),
            bindings: bindings.into(),
        })
    }

    #[must_use]
    pub const fn registry(&self) -> &Arc<ActivityHandlerRegistry> {
        &self.registry
    }

    #[must_use]
    pub fn bindings(&self) -> &[ActivityInteractionBinding] {
        &self.bindings
    }

    #[must_use]
    pub fn binding(
        &self,
        node: NodeId,
        outcome: ActivityExternalOutcomeId,
    ) -> Option<&ActivityInteractionBinding> {
        self.bindings
            .binary_search_by_key(&(node, outcome), binding_key)
            .ok()
            .map(|index| &self.bindings[index])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityInteractionBindingError {
    InvalidComponentId,
    PayloadTooLarge,
    TooManyBindings,
    DuplicateBinding,
    MissingNode,
    WrongNodeKind,
    OutcomeNotOffered,
    MissingHandler,
}

impl core::fmt::Display for ActivityInteractionBindingError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "Activity interaction binding error: {self:?}")
    }
}

impl std::error::Error for ActivityInteractionBindingError {}

fn binding_key(binding: &ActivityInteractionBinding) -> (NodeId, ActivityExternalOutcomeId) {
    (binding.node, binding.offered_outcome)
}

fn validate_binding(
    binding: &ActivityInteractionBinding,
    registry: &ActivityHandlerRegistry,
    graph: &ActivityGraphDefinition,
    programs: &[GraphActivityNodeProgram],
) -> Result<(), ActivityInteractionBindingError> {
    let node = graph
        .node(binding.node)
        .ok_or(ActivityInteractionBindingError::MissingNode)?;
    if node.kind() != crate::ActivityNodeKind::ExternalOutcome {
        return Err(ActivityInteractionBindingError::WrongNodeKind);
    }
    let option = ActivityOptionId::new(binding.offered_outcome.get())
        .expect("external outcomes and options share non-zero u64 identity");
    let offered = programs
        .binary_search_by_key(&binding.node, GraphActivityNodeProgram::node)
        .ok()
        .map(|index| programs[index].program())
        .is_some_and(|program| {
            program.operations().iter().any(|operation| {
                matches!(
                    operation,
                    crate::ActivityOperation::Offer { kind, options }
                        if *kind == ActivityDecisionKind::ExternalOutcome
                            && options.iter().any(|candidate| candidate.id() == option)
                )
            })
        });
    if !offered {
        return Err(ActivityInteractionBindingError::OutcomeNotOffered);
    }
    if registry.handler(binding.handler).is_none() {
        return Err(ActivityInteractionBindingError::MissingHandler);
    }
    Ok(())
}

fn validate_component_id(value: &str) -> Result<(), ActivityInteractionBindingError> {
    if value.is_empty()
        || value.len() > MAX_ACTIVITY_COMPONENT_ID_BYTES
        || !value.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(ActivityInteractionBindingError::InvalidComponentId);
    }
    Ok(())
}
