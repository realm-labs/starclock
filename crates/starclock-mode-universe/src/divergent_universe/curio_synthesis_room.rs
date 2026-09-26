//! Explicitly placed, bounded synthesis menus on the shared source-room graph.

#[path = "curio_synthesis_room_binding.rs"]
mod binding;
#[path = "curio_synthesis_room_graph.rs"]
mod graph;

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseCurioRuntime, DivergentUniverseLogicalScopeKind,
    DivergentUniverseRuntimeFactory,
    curio_synthesis::{
        CurioSynthesisError,
        offers::{CurioSynthesisOfferError, CurioSynthesisOffers},
    },
    domain_route::{DomainRoomContext, DomainRoomProgram, DomainRouteError},
};
use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityPlayerView, ActivityScope,
    ActivitySlotDefinition, ActivitySlotId, ActivityStateSource, ActivityStateVisibility,
    ActivityValue, GraphActivityCommandError, GraphActivityDefinition, GraphActivityNodeProgram,
    GraphActivityRuntimeError, NodeId, SlotCarryPolicy, SlotResetPoint,
};
use starclock_data::divergent_universe_service_catalog::DivergentUniverseWorkbenchId;
use std::sync::Arc;

pub const OPEN_SYNTHESIS: u64 = u64::MAX - 200;
pub const CANCEL_SYNTHESIS: u64 = u64::MAX - 201;
pub const LEAVE_SYNTHESIS: u64 = u64::MAX;
const OPEN_LIMIT: u16 = 64;

/// Disjoint host addresses (all >=70). Five logical-room values survive menu
/// movement; `accepted` resets on every physical node to prohibit raw choices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurioSynthesisSlots {
    pub first: ActivitySlotId,
    pub second: ActivitySlotId,
    pub choices: ActivitySlotId,
    pub completed: ActivitySlotId,
    pub accepted: ActivitySlotId,
    pub opens: ActivitySlotId,
}

/// Explicit confirmed-synthesis limit 1..=64 per logical room. A separate
/// 64-opening budget bounds pre-draw cancellations. Neither is original parity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurioSynthesisRoomPolicy {
    limit: u16,
}
impl CurioSynthesisRoomPolicy {
    pub fn new(limit: u16) -> Result<Self, CurioSynthesisRoomError> {
        if !(1..=OPEN_LIMIT).contains(&limit) {
            return Err(CurioSynthesisRoomError::InvalidPolicy);
        }
        Ok(Self { limit })
    }
    #[must_use]
    pub const fn accuracy(self) -> CurioSynthesisRoomAccuracy {
        CurioSynthesisRoomAccuracy::VersionedProjectPolicyExplicitPlacementBoundedOpeningsCachedPairMandatoryConfirmation
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioSynthesisRoomAccuracy {
    VersionedProjectPolicyExplicitPlacementBoundedOpeningsCachedPairMandatoryConfirmation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioSynthesisRoomPhase {
    Menu,
    FirstInput,
    SecondInput,
    Confirmation,
}
#[derive(Debug)]
pub enum CurioSynthesisRoomError {
    InvalidPolicy,
    InvalidContext,
    DefinitionMismatch,
    Route(DomainRouteError),
    Synthesis(CurioSynthesisError),
    Offers(CurioSynthesisOfferError),
}
impl std::fmt::Display for CurioSynthesisRoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Divergent Universe synthesis room: {self:?}")
    }
}
impl std::error::Error for CurioSynthesisRoomError {}

#[derive(Clone, Debug)]
struct SynthesisService {
    curios: DivergentUniverseCurioRuntime,
    offers: CurioSynthesisOffers,
    policy: CurioSynthesisRoomPolicy,
    slots: CurioSynthesisSlots,
    workbench: u64,
    receipt: u64,
}
/// Reused immutable source-backed compiler. Placement is caller policy, NOT
/// automatic Reforge/Respite/NPC admission or a complete default profile.
#[derive(Clone, Debug)]
pub struct CurioSynthesisRoomCompiler {
    factory: DivergentUniverseRuntimeFactory,
    service: Arc<SynthesisService>,
}
#[derive(Clone, Debug)]
pub struct CompiledCurioSynthesisRoom {
    factory: DivergentUniverseRuntimeFactory,
    context: DomainRoomContext,
    service: Arc<SynthesisService>,
    fragment: DomainRoomProgram,
    entry_program: GraphActivityNodeProgram,
    slots: Vec<ActivitySlotDefinition>,
    menu: NodeId,
    first: NodeId,
    second: NodeId,
    output: NodeId,
}
/// Authenticated capability for one exact whole graph, not mutable mode state.
#[derive(Clone, Debug)]
pub struct BoundCurioSynthesisRoom {
    room: CompiledCurioSynthesisRoom,
    definition: Arc<GraphActivityDefinition>,
}

impl DivergentUniverseRuntimeFactory {
    /// Requires exact function-4 Workbench membership. Caller owns placement and
    /// binds the resulting digest into profile identity; no NPC selector is inferred.
    pub fn curio_synthesis_room_compiler(
        &self,
        workbench: &DivergentUniverseWorkbenchId,
        policy: CurioSynthesisRoomPolicy,
        slots: CurioSynthesisSlots,
    ) -> Result<CurioSynthesisRoomCompiler, CurioSynthesisRoomError> {
        slots.definitions()?;
        let (workbench, receipt) = self
            .workbench_curse_runtime()
            .map_err(CurioSynthesisError::Workbench)
            .map_err(CurioSynthesisRoomError::Synthesis)?
            .synthesis_service_keys(workbench)
            .map_err(CurioSynthesisError::Workbench)
            .map_err(CurioSynthesisRoomError::Synthesis)?;
        Ok(CurioSynthesisRoomCompiler {
            factory: self.clone(),
            service: Arc::new(SynthesisService {
                curios: self
                    .curio_runtime()
                    .map_err(CurioSynthesisError::Curio)
                    .map_err(CurioSynthesisRoomError::Synthesis)?,
                offers: self
                    .curio_synthesis_offers()
                    .map_err(CurioSynthesisRoomError::Offers)?,
                policy,
                slots,
                workbench,
                receipt,
            }),
        })
    }
}
impl CurioSynthesisRoomCompiler {
    pub fn compile(
        &self,
        context: &DomainRoomContext,
    ) -> Result<CompiledCurioSynthesisRoom, CurioSynthesisRoomError> {
        if !self.factory.room_context_matches(context) {
            return Err(CurioSynthesisRoomError::InvalidContext);
        }
        let [menu, first, second, output] = [1, 2, 3, 4]
            .map(|index| context.node(index))
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(CurioSynthesisRoomError::Route)?
            .try_into()
            .map_err(|_| CurioSynthesisRoomError::InvalidPolicy)?;
        let fragment = self
            .service
            .compile_graph(context, [menu, first, second, output])?;
        let original = &fragment.programs[0];
        let entry_program = graph::program(
            original.node(),
            self.service
                .curios
                .compiled_domain_entry_operations(original.program().operations().to_vec())
                .map_err(CurioSynthesisError::Curio)
                .map_err(CurioSynthesisRoomError::Synthesis)?,
        )?;
        Ok(CompiledCurioSynthesisRoom {
            factory: self.factory.clone(),
            context: context.clone(),
            service: Arc::clone(&self.service),
            fragment,
            entry_program,
            slots: self.service.slots.definitions()?,
            menu,
            first,
            second,
            output,
        })
    }
}
impl CompiledCurioSynthesisRoom {
    pub(in crate::divergent_universe) fn matches_factory(
        &self,
        factory: &DivergentUniverseRuntimeFactory,
    ) -> bool {
        self.factory.bundle_identity().component_digest()
            == factory.bundle_identity().component_digest()
            && self.factory.decision_catalog().digest() == factory.decision_catalog().digest()
            && factory.room_context_matches(&self.context)
    }

    #[must_use]
    pub fn context(&self) -> &DomainRoomContext {
        &self.context
    }
    #[must_use]
    pub fn fragment(&self) -> &DomainRoomProgram {
        &self.fragment
    }
    #[must_use]
    pub fn slot_definitions(&self) -> &[ActivitySlotDefinition] {
        &self.slots
    }
    #[must_use]
    pub const fn menu_node(&self) -> NodeId {
        self.menu
    }
    #[must_use]
    pub const fn first_input_node(&self) -> NodeId {
        self.first
    }
    #[must_use]
    pub const fn second_input_node(&self) -> NodeId {
        self.second
    }
    #[must_use]
    pub const fn output_node(&self) -> NodeId {
        self.output
    }
    /// Exact inputs, selection policy, placement, layout and both explicit caps.
    #[must_use]
    pub fn configuration_digest(&self) -> [u8; 32] {
        let mut hash = CanonicalDigestBuilder::new();
        hash.update(b"starclock.du.curio-synthesis-room.explicit-placement.pre-draw-cancel.cached-inputs-and-sample.mandatory-confirmation.physical-command-gate.logical-room-counts");
        hash.update(self.service.offers.configuration_digest());
        for value in [
            self.context.area.as_str(),
            self.context.layer.as_str(),
            &self.context.position_key,
            &self.context.preset_source,
        ] {
            hash.update(
                u64::try_from(value.len())
                    .expect("validated bounded context length")
                    .to_le_bytes(),
            );
            hash.update(value.as_bytes());
        }
        for value in [
            self.context.entry_node(),
            self.context.successor(),
            self.menu,
            self.first,
            self.second,
            self.output,
        ] {
            hash.update(value.get().to_le_bytes());
        }
        hash.update(self.context.exit_edge().get().to_le_bytes());
        hash.update(self.context.position_ordinal.to_le_bytes());
        hash.update(self.context.plane_ordinal.to_le_bytes());
        hash.update(self.context.level.to_le_bytes());
        hash.update(self.context.section.get().to_le_bytes());
        hash.update(self.service.policy.limit.to_le_bytes());
        hash.update(OPEN_LIMIT.to_le_bytes());
        hash.update(self.service.workbench.to_le_bytes());
        hash.update(self.service.receipt.to_le_bytes());
        for slot in self.service.slots.ids() {
            hash.update(slot.get().to_le_bytes());
        }
        hash.finalize()
    }
}
impl CurioSynthesisSlots {
    fn ids(self) -> [ActivitySlotId; 6] {
        [
            self.first,
            self.second,
            self.choices,
            self.completed,
            self.accepted,
            self.opens,
        ]
    }
    fn definitions(self) -> Result<Vec<ActivitySlotDefinition>, CurioSynthesisRoomError> {
        let ids = self.ids();
        if ids.iter().any(|id| id.get() < 70)
            || ids.iter().enumerate().any(|(i, id)| ids[..i].contains(id))
        {
            return Err(CurioSynthesisRoomError::InvalidPolicy);
        }
        let values = [
            (
                self.first,
                ActivityValue::OptionalId(None),
                None,
                None,
                true,
            ),
            (
                self.second,
                ActivityValue::OptionalId(None),
                None,
                None,
                true,
            ),
            (
                self.choices,
                ActivityValue::BoundedCounterMap(Box::new([])),
                Some((1, 1)),
                Some(235),
                true,
            ),
            (
                self.completed,
                ActivityValue::BoundedInteger(0),
                Some((0, 64)),
                None,
                true,
            ),
            (
                self.opens,
                ActivityValue::BoundedInteger(0),
                Some((0, 64)),
                None,
                true,
            ),
            (
                self.accepted,
                ActivityValue::Boolean(false),
                None,
                None,
                false,
            ),
        ];
        values
            .into_iter()
            .map(|(id, initial, bounds, entries, logical)| {
                let definition = ActivitySlotDefinition::new_with_policy(
                    id,
                    ActivityScope::Node,
                    initial,
                    bounds,
                    entries,
                    vec![SlotResetPoint::NodeStart],
                    SlotCarryPolicy::Reset,
                    ActivityStateVisibility::Player,
                    ActivityStateSource::new(u64::from(id.get()))
                        .ok_or(CurioSynthesisRoomError::InvalidPolicy)?,
                )
                .map_err(|_| CurioSynthesisRoomError::InvalidPolicy)?;
                Ok(if logical {
                    definition
                        .with_logical_scope(DivergentUniverseLogicalScopeKind::Node.class_id())
                } else {
                    definition
                })
            })
            .collect()
    }
}
fn literal(value: ActivityValue) -> ActivityExpression {
    ActivityExpression::Literal(value)
}
fn integer(value: i64) -> ActivityExpression {
    literal(ActivityValue::BoundedInteger(value))
}
fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
fn value(
    view: &ActivityPlayerView,
    slot: ActivitySlotId,
) -> Result<&ActivityValue, GraphActivityCommandError> {
    view.slots()
        .iter()
        .find(|entry| entry.id() == slot)
        .map(|entry| entry.value())
        .ok_or_else(invalid)
}
fn optional(
    view: &ActivityPlayerView,
    slot: ActivitySlotId,
) -> Result<Option<u64>, GraphActivityCommandError> {
    match value(view, slot)? {
        ActivityValue::OptionalId(value) => Ok(*value),
        _ => Err(invalid()),
    }
}
fn count(
    view: &ActivityPlayerView,
    slot: ActivitySlotId,
) -> Result<i64, GraphActivityCommandError> {
    match value(view, slot)? {
        ActivityValue::BoundedInteger(value) => Ok(*value),
        _ => Err(invalid()),
    }
}
fn choices(
    view: &ActivityPlayerView,
    slot: ActivitySlotId,
) -> Result<&[(u64, i64)], GraphActivityCommandError> {
    match value(view, slot)? {
        ActivityValue::BoundedCounterMap(value) => Ok(value),
        _ => Err(invalid()),
    }
}
fn set(slot: ActivitySlotId, value: ActivityValue) -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot,
        value: literal(value),
    }
}
