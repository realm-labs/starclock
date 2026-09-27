//! Reviewed fixed Blank domain: no local payload, normal entry lifecycle and exit.

#[path = "blank_room_binding.rs"]
mod binding;

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseRuntimeFactory,
    domain_route::{DomainRoomComposition, DomainRoomContext, DomainRoomProgram, DomainRouteError},
    state::{ROOM_DOORS_OPEN_SLOT, ROOM_FINISHED_SLOT},
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityExpression, ActivityNodeDefinition, ActivityNodeKind, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivitySlotId, ActivityValue, GraphActivityDefinition, GraphActivityNodeProgram, NodeId,
};
use starclock_data::divergent_universe_domain_layout::FixedDomainKind;
use std::sync::Arc;

pub const LEAVE_BLANK: u64 = u64::MAX;

/// Empty content is a released fact; immediate completion/door timing is not.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlankRoomAccuracy {
    VersionedProjectPolicyEmptyPayloadImmediateCompletionIndependentLeave,
}

#[derive(Debug)]
pub enum BlankRoomError {
    InvalidContext,
    DefinitionMismatch,
    Route(DomainRouteError),
}
impl std::fmt::Display for BlankRoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Divergent Universe Blank room: {self:?}")
    }
}
impl std::error::Error for BlankRoomError {}

/// Compiler for the current fixed Blank preset only. Blank-card creation,
/// mask/beacon mutations and other presets require separate admission proof.
#[derive(Clone, Debug)]
pub struct BlankRoomCompiler {
    factory: DivergentUniverseRuntimeFactory,
}
#[derive(Clone, Debug)]
pub struct CompiledBlankRoom {
    compiler: BlankRoomCompiler,
    context: DomainRoomContext,
    fragment: DomainRoomProgram,
    entry_program: GraphActivityNodeProgram,
    menu: NodeId,
}
/// Immutable whole-definition capability; it owns no live Activity state.
#[derive(Clone, Debug)]
pub struct BoundBlankRoom {
    room: CompiledBlankRoom,
    definition: Arc<GraphActivityDefinition>,
}

impl DivergentUniverseRuntimeFactory {
    /// No rewards, services, native callbacks or additional host slots.
    #[must_use]
    pub fn blank_room_compiler(&self) -> BlankRoomCompiler {
        BlankRoomCompiler {
            factory: self.clone(),
        }
    }
}
impl BlankRoomCompiler {
    #[must_use]
    pub const fn accuracy(&self) -> BlankRoomAccuracy {
        BlankRoomAccuracy::VersionedProjectPolicyEmptyPayloadImmediateCompletionIndependentLeave
    }

    /// Validates the exact current source-position join. Completion is automatic
    /// because there is no room-local content; Leave remains a separate command.
    /// The owning route must lower the normal Curio entry lifecycle once using
    /// `compile_curio_domain_route`, and bind this room digest into its identity.
    pub fn compile(
        &self,
        context: &DomainRoomContext,
    ) -> Result<CompiledBlankRoom, BlankRoomError> {
        if !self.factory.room_context_matches(context)
            || context.composition != DomainRoomComposition::Fixed(FixedDomainKind::Blank)
            || context.preset_source.as_ref() != "9007"
            || context.level != 1
        {
            return Err(BlankRoomError::InvalidContext);
        }
        let entry = context.entry_node();
        let menu = context.node(1).map_err(BlankRoomError::Route)?;
        let enter = context.edge(0).map_err(BlankRoomError::Route)?;
        let leave = ActivityOptionDefinition::new(
            ActivityOptionId::new(LEAVE_BLANK).ok_or(BlankRoomError::DefinitionMismatch)?,
            0,
            finished(),
            vec![
                ActivityOperation::Require(finished()),
                ActivityOperation::Traverse(context.exit_edge()),
            ],
        );
        let fragment = DomainRoomProgram {
            exit_node: menu,
            nodes: [entry, menu]
                .into_iter()
                .map(|node| {
                    ActivityNodeDefinition::new(node, context.section, ActivityNodeKind::Choice, 1)
                        .map_err(|_| BlankRoomError::DefinitionMismatch)
                })
                .collect::<Result<Vec<_>, _>>()?,
            edges: vec![
                ActivityEdgeDefinition::new(
                    enter,
                    entry,
                    menu,
                    ActivityEdgeCondition::Always,
                    0,
                    1,
                )
                .map_err(|_| BlankRoomError::DefinitionMismatch)?,
            ],
            programs: vec![
                program(entry, vec![ActivityOperation::Traverse(enter)])?,
                program(
                    menu,
                    vec![
                        // These physical-node flags reset between entry and menu.
                        // Publish finish before doors at the actual waiting boundary.
                        set_true(ROOM_FINISHED_SLOT),
                        set_true(ROOM_DOORS_OPEN_SLOT),
                        ActivityOperation::Offer {
                            kind: ActivityDecisionKind::Route,
                            options: vec![leave].into_boxed_slice(),
                        },
                    ],
                )?,
            ],
            random_offers: Vec::new(),
        };
        let entry_program = program(
            entry,
            self.factory
                .curio_runtime()
                .map_err(|_| BlankRoomError::DefinitionMismatch)?
                .compiled_domain_entry_operations(
                    fragment.programs[0].program().operations().to_vec(),
                )
                .map_err(|_| BlankRoomError::DefinitionMismatch)?,
        )?;
        Ok(CompiledBlankRoom {
            compiler: self.clone(),
            context: context.clone(),
            fragment,
            entry_program,
            menu,
        })
    }
}
impl CompiledBlankRoom {
    #[must_use]
    pub const fn context(&self) -> &DomainRoomContext {
        &self.context
    }
    #[must_use]
    pub const fn fragment(&self) -> &DomainRoomProgram {
        &self.fragment
    }
    #[must_use]
    pub const fn menu_node(&self) -> NodeId {
        self.menu
    }
    /// Binds the exact source/decision inputs and source-position addresses.
    /// The owning profile must also bind all other immutable graph inputs.
    #[must_use]
    pub fn configuration_digest(&self) -> [u8; 32] {
        let mut digest = CanonicalDigestBuilder::new();
        digest.update(b"du.blank.fixed.no-local-payload.normal-curio-entry.finish-then-doors.independent-leave");
        digest.update(
            self.compiler
                .factory
                .bundle_identity()
                .component_digest()
                .bytes(),
        );
        digest.update(self.compiler.factory.decision_catalog().digest());
        for key in [
            self.context.area.as_str(),
            self.context.layer.as_str(),
            self.context.position_key.as_ref(),
            self.context.preset_source.as_ref(),
        ] {
            digest.update(
                u64::try_from(key.len())
                    .expect("bounded source key")
                    .to_le_bytes(),
            );
            digest.update(key.as_bytes());
        }
        for node in [
            self.context.entry_node(),
            self.menu,
            self.context.successor(),
        ] {
            digest.update(node.get().to_le_bytes());
        }
        digest.update(self.context.exit_edge().get().to_le_bytes());
        digest.update(self.context.section.get().to_le_bytes());
        digest.update(self.context.plane_ordinal.to_le_bytes());
        digest.update(self.context.position_ordinal.to_le_bytes());
        digest.update(self.context.level.to_le_bytes());
        digest.finalize()
    }
}
fn finished() -> ActivityCondition {
    ActivityCondition::All(
        [ROOM_FINISHED_SLOT, ROOM_DOORS_OPEN_SLOT]
            .into_iter()
            .map(|slot| ActivityCondition::Boolean(ActivityExpression::Slot(slot)))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    )
}
fn set_true(slot: ActivitySlotId) -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot,
        value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
    }
}
fn program(
    node: NodeId,
    operations: Vec<ActivityOperation>,
) -> Result<GraphActivityNodeProgram, BlankRoomError> {
    Ok(GraphActivityNodeProgram::new(
        node,
        ActivityProgramDefinition::new(
            ActivityProgramId::new(node.get()).ok_or(BlankRoomError::DefinitionMismatch)?,
            operations,
        )
        .map_err(|_| BlankRoomError::DefinitionMismatch)?,
    ))
}
