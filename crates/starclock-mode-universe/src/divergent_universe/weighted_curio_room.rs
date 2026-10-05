//! Explicit equipment service on proven current Reforge card positions.
//!
//! Capacity is caller-authored, not inferred from the source preset's level.
//! This fragment implements no divination, domain enhancement or automatic
//! Forge admission. All execution remains in the shared Activity graph.
#[path = "weighted_curio_room_binding.rs"]
mod binding;

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseLogicalScopeKind, DivergentUniverseRuntimeFactory,
    domain_route::{DomainRoomComposition, DomainRoomContext, DomainRoomProgram, DomainRouteError},
    state::{WEIGHTED_CURIO_REFERENCES_SLOT, weighted_curio_slot},
    weighted_curio::{WeightedCurioError, WeightedCurioRuntime, WeightedCurioSlotLimit},
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityExpression, ActivityNodeDefinition, ActivityNodeKind, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityScope, ActivitySlotDefinition, ActivitySlotId, ActivityStateSource,
    ActivityStateVisibility, ActivityValue, GraphActivityDefinition, GraphActivityNodeProgram,
    NodeId, SlotCarryPolicy, SlotResetPoint,
};
use starclock_data::divergent_universe_domain_decks::DomainCardKind;
use std::sync::Arc;

pub const CLEAR_EQUIPMENT: u64 = u64::MAX - 1;
pub const LEAVE_EQUIPMENT: u64 = u64::MAX;
const CHANGE_LIMIT: i64 = 64;

/// Explicit menu/capacity policy, not observed released Forge execution parity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightedCurioRoomAccuracy {
    VersionedProjectPolicyExplicitReforgeCapacityCanonicalToggle64Changes,
}

/// Host-owned isolated slot addresses. Changes persist through internal menu
/// loops in the logical room; authentication resets at each physical node entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WeightedCurioRoomSlots {
    pub changes: ActivitySlotId,
    pub accepted: ActivitySlotId,
}

#[derive(Debug)]
pub enum WeightedCurioRoomError {
    InvalidContext,
    InvalidSlots,
    DefinitionMismatch,
    Equipment(WeightedCurioError),
    Route(DomainRouteError),
}
impl std::fmt::Display for WeightedCurioRoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Divergent Universe equipment room: {self:?}")
    }
}
impl std::error::Error for WeightedCurioRoomError {}

/// Immutable compiler; slot capacity is independent of the current source level.
#[derive(Clone, Debug)]
pub struct WeightedCurioRoomCompiler {
    factory: DivergentUniverseRuntimeFactory,
    runtime: WeightedCurioRuntime,
    limit: WeightedCurioSlotLimit,
    slots: WeightedCurioRoomSlots,
    declarations: Vec<ActivitySlotDefinition>,
}
#[derive(Clone, Debug)]
pub struct CompiledWeightedCurioRoom {
    compiler: WeightedCurioRoomCompiler,
    context: DomainRoomContext,
    fragment: DomainRoomProgram,
    entry_program: GraphActivityNodeProgram,
    menu: NodeId,
}
/// Capability authenticating one exact immutable whole graph, not live state.
#[derive(Clone, Debug)]
pub struct BoundWeightedCurioRoom {
    room: CompiledWeightedCurioRoom,
    definition: Arc<GraphActivityDefinition>,
}

impl DivergentUniverseRuntimeFactory {
    /// Construct an explicit equipment service, not an original level-to-slot
    /// selector. The host supplies distinct nonconflicting slots, includes their
    /// declarations and binds each compiled room digest into its configuration.
    pub fn weighted_curio_room_compiler(
        &self,
        limit: WeightedCurioSlotLimit,
        slots: WeightedCurioRoomSlots,
    ) -> Result<WeightedCurioRoomCompiler, WeightedCurioRoomError> {
        if slots.changes == slots.accepted
            || [slots.changes, slots.accepted].contains(&WEIGHTED_CURIO_REFERENCES_SLOT)
        {
            return Err(WeightedCurioRoomError::InvalidSlots);
        }
        let mut declarations =
            vec![weighted_curio_slot().map_err(|_| WeightedCurioRoomError::InvalidSlots)?];
        for (id, initial, bounds, logical) in [
            (
                slots.changes,
                ActivityValue::BoundedInteger(0),
                Some((0, CHANGE_LIMIT)),
                true,
            ),
            (slots.accepted, ActivityValue::Boolean(false), None, false),
        ] {
            let declaration = ActivitySlotDefinition::new_with_policy(
                id,
                ActivityScope::Node,
                initial,
                bounds,
                None,
                vec![SlotResetPoint::NodeStart],
                SlotCarryPolicy::Reset,
                ActivityStateVisibility::Player,
                ActivityStateSource::new(u64::from(id.get()))
                    .ok_or(WeightedCurioRoomError::InvalidSlots)?,
            )
            .map_err(|_| WeightedCurioRoomError::InvalidSlots)?;
            declarations.push(if logical {
                declaration.with_logical_scope(DivergentUniverseLogicalScopeKind::Node.class_id())
            } else {
                declaration
            });
        }
        Ok(WeightedCurioRoomCompiler {
            factory: self.clone(),
            runtime: self
                .weighted_curio_runtime()
                .map_err(WeightedCurioRoomError::Equipment)?,
            limit,
            slots,
            declarations,
        })
    }
}
impl WeightedCurioRoomCompiler {
    #[must_use]
    pub const fn accuracy(&self) -> WeightedCurioRoomAccuracy {
        WeightedCurioRoomAccuracy::VersionedProjectPolicyExplicitReforgeCapacityCanonicalToggle64Changes
    }

    /// Admit only a validated current Reforge card context. No inferred NPC,
    /// historical preset, randomly selected equipment or source-level upgrade.
    /// The owning route must use `compile_curio_domain_route` for entry effects.
    pub fn compile(
        &self,
        context: &DomainRoomContext,
    ) -> Result<CompiledWeightedCurioRoom, WeightedCurioRoomError> {
        if !self.factory.room_context_matches(context)
            || context.composition != DomainRoomComposition::Card(DomainCardKind::Reforge)
        {
            return Err(WeightedCurioRoomError::InvalidContext);
        }
        let entry = context.entry_node();
        let menu = context.node(1).map_err(WeightedCurioRoomError::Route)?;
        let enter = context.edge(0).map_err(WeightedCurioRoomError::Route)?;
        let repeat = context.edge(1).map_err(WeightedCurioRoomError::Route)?;
        let mut options = Vec::new();
        for (index, _) in self.runtime.candidates().iter().enumerate() {
            let key = u64::try_from(index)
                .ok()
                .and_then(|value| value.checked_add(1))
                .ok_or(WeightedCurioRoomError::DefinitionMismatch)?;
            let removable = ActivityCondition::Equal(
                ActivityExpression::CounterValue {
                    slot: WEIGHTED_CURIO_REFERENCES_SLOT,
                    key,
                },
                integer(1),
            );
            let capacity = ActivityCondition::LessThan(
                ActivityExpression::CounterEntryCount(WEIGHTED_CURIO_REFERENCES_SLOT),
                integer(i64::from(self.limit.get())),
            );
            options.push(option(
                key,
                ActivityCondition::All(
                    vec![
                        self.can_change(),
                        ActivityCondition::Any(vec![removable, capacity].into()),
                    ]
                    .into(),
                ),
                vec![
                    require_accepted(self.slots),
                    ActivityOperation::Traverse(repeat),
                ],
            )?);
        }
        options.push(option(
            CLEAR_EQUIPMENT,
            ActivityCondition::All(
                vec![
                    self.can_change(),
                    ActivityCondition::LessThan(
                        integer(0),
                        ActivityExpression::CounterEntryCount(WEIGHTED_CURIO_REFERENCES_SLOT),
                    ),
                ]
                .into(),
            ),
            vec![
                require_accepted(self.slots),
                ActivityOperation::Traverse(repeat),
            ],
        )?);
        options.push(option(
            LEAVE_EQUIPMENT,
            yes(),
            vec![
                require_accepted(self.slots),
                ActivityOperation::Traverse(context.exit_edge()),
            ],
        )?);
        let fragment = DomainRoomProgram {
            exit_node: menu,
            nodes: [(entry, 1), (menu, 65)]
                .into_iter()
                .map(|(id, visits)| {
                    ActivityNodeDefinition::new(
                        id,
                        context.section,
                        ActivityNodeKind::Choice,
                        visits,
                    )
                    .map_err(|_| WeightedCurioRoomError::DefinitionMismatch)
                })
                .collect::<Result<_, _>>()?,
            edges: [(enter, entry, menu, 1), (repeat, menu, menu, 64)]
                .into_iter()
                .map(|(id, from, to, traversals)| {
                    ActivityEdgeDefinition::new(
                        id,
                        from,
                        to,
                        ActivityEdgeCondition::Always,
                        0,
                        traversals,
                    )
                    .map_err(|_| WeightedCurioRoomError::DefinitionMismatch)
                })
                .collect::<Result<_, _>>()?,
            programs: vec![
                program(
                    entry,
                    vec![
                        ActivityOperation::Require(ActivityCondition::LessThan(
                            ActivityExpression::CounterEntryCount(WEIGHTED_CURIO_REFERENCES_SLOT),
                            integer(i64::from(self.limit.get()) + 1),
                        )),
                        ActivityOperation::SetSlot {
                            slot: self.slots.changes,
                            value: integer(0),
                        },
                        ActivityOperation::Traverse(enter),
                    ],
                )?,
                program(
                    menu,
                    vec![ActivityOperation::Offer {
                        kind: ActivityDecisionKind::Service,
                        options: options.into(),
                    }],
                )?,
            ],
            random_offers: Vec::new(),
        };
        let entry_program = program(
            entry,
            self.factory
                .curio_runtime()
                .map_err(|_| WeightedCurioRoomError::DefinitionMismatch)?
                .compiled_domain_entry_operations(
                    fragment.programs[0].program().operations().to_vec(),
                )
                .map_err(|_| WeightedCurioRoomError::DefinitionMismatch)?,
        )?;
        Ok(CompiledWeightedCurioRoom {
            compiler: self.clone(),
            context: context.clone(),
            fragment,
            entry_program,
            menu,
        })
    }
    fn can_change(&self) -> ActivityCondition {
        ActivityCondition::LessThan(
            ActivityExpression::Slot(self.slots.changes),
            integer(CHANGE_LIMIT),
        )
    }
}
impl CompiledWeightedCurioRoom {
    pub(in crate::divergent_universe) fn matches_factory(
        &self,
        factory: &DivergentUniverseRuntimeFactory,
    ) -> bool {
        self.compiler.factory.bundle_identity() == factory.bundle_identity()
            && self.compiler.factory.decision_catalog().digest()
                == factory.decision_catalog().digest()
            && factory.room_context_matches(&self.context)
    }

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
    /// Includes the production equipped declaration plus both host service slots.
    /// Replace identical existing declarations; do not introduce duplicate IDs.
    #[must_use]
    pub fn slot_definitions(&self) -> &[ActivitySlotDefinition] {
        &self.compiler.declarations
    }
    /// Exact current source/decision inputs, explicit capacity, slots and placement.
    /// This named policy is not observed original Forge menu/timing parity.
    #[must_use]
    pub fn configuration_digest(&self) -> [u8; 32] {
        let mut hash = CanonicalDigestBuilder::new();
        hash.update(b"du.equipment-room.explicit-capacity.toggle-clear.free.canonical-all-seventeen.64-changes.curio-entry.no-automatic-forge");
        hash.update(
            self.compiler
                .factory
                .bundle_identity()
                .component_digest()
                .bytes(),
        );
        hash.update(self.compiler.factory.decision_catalog().digest());
        for value in [
            self.context.area.as_str(),
            self.context.layer.as_str(),
            &self.context.position_key,
            &self.context.preset_source,
        ] {
            hash.update(
                u64::try_from(value.len())
                    .expect("bounded validated source string")
                    .to_le_bytes(),
            );
            hash.update(value.as_bytes());
        }
        for value in [
            self.context.entry_node().get(),
            self.context.exit_edge().get(),
            self.context.successor().get(),
            self.context.plane_ordinal,
            self.compiler.slots.changes.get(),
            self.compiler.slots.accepted.get(),
        ] {
            hash.update(value.to_le_bytes());
        }
        hash.update(self.context.level.to_le_bytes());
        hash.update(self.context.position_ordinal.to_le_bytes());
        hash.update(self.compiler.limit.get().to_le_bytes());
        hash.finalize()
    }
}
fn program(
    node: NodeId,
    operations: Vec<ActivityOperation>,
) -> Result<GraphActivityNodeProgram, WeightedCurioRoomError> {
    Ok(GraphActivityNodeProgram::new(
        node,
        ActivityProgramDefinition::new(
            ActivityProgramId::new(node.get()).ok_or(WeightedCurioRoomError::DefinitionMismatch)?,
            operations,
        )
        .map_err(|_| WeightedCurioRoomError::DefinitionMismatch)?,
    ))
}
fn option(
    raw: u64,
    enabled: ActivityCondition,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityOptionDefinition, WeightedCurioRoomError> {
    Ok(ActivityOptionDefinition::new(
        ActivityOptionId::new(raw).ok_or(WeightedCurioRoomError::DefinitionMismatch)?,
        0,
        enabled,
        operations,
    ))
}
fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
fn yes() -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true)))
}
fn require_accepted(slots: WeightedCurioRoomSlots) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Boolean(ActivityExpression::Slot(
        slots.accepted,
    )))
}
