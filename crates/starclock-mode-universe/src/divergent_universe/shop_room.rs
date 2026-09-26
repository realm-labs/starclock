//! Explicit fixed stock on reviewed Shop-card contexts, no original NPC claim.

#[path = "shop_room_binding.rs"]
mod binding;
#[path = "shop_room_graph.rs"]
mod graph;

use std::sync::Arc;

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseRuntimeFactory,
    decision_rewards::{DecisionRewardError, DecisionRewardRuntime},
    domain_route::{DomainRoomComposition, DomainRoomContext, DomainRoomProgram, DomainRouteError},
    shop_purchase::{ShopPurchaseError, ShopPurchaseRuntime, ShopStockItem},
};
use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityScope, ActivitySlotDefinition, ActivitySlotId, ActivityStateSource,
    ActivityStateVisibility, ActivityValue, GraphActivityCommandError, GraphActivityDefinition,
    GraphActivityNodeProgram, GraphActivityRuntimeError, NodeId, SlotCarryPolicy, SlotResetPoint,
};
use starclock_data::divergent_universe_domain_decks::DomainCardKind;

pub const LEAVE_SHOP: u64 = u64::MAX;

/// Host addresses >=70, disjoint. Sold-out marks are logical-room scoped;
/// `accepted` resets on every physical menu to reject raw shared choices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShopRoomSlots {
    pub purchased: ActivitySlotId,
    pub accepted: ActivitySlotId,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShopRoomAccuracy {
    VersionedProjectPolicyExplicitStockAtReviewedShopCardsOnePurchasePerItemNoRefresh,
}
#[derive(Debug)]
pub enum ShopRoomError {
    InvalidSlots,
    InvalidContext,
    DefinitionMismatch,
    Purchase(ShopPurchaseError),
    Reward(DecisionRewardError),
    Route(DomainRouteError),
}
impl std::fmt::Display for ShopRoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Divergent Universe shop room: {self:?}")
    }
}
impl std::error::Error for ShopRoomError {}

#[derive(Clone, Debug)]
pub struct ShopRoomCompiler {
    factory: DivergentUniverseRuntimeFactory,
    runtime: Arc<ShopPurchaseRuntime>,
    rewards: Arc<DecisionRewardRuntime>,
    slots: ShopRoomSlots,
    declarations: Vec<ActivitySlotDefinition>,
}
#[derive(Clone, Debug)]
pub struct CompiledShopRoom {
    compiler: ShopRoomCompiler,
    context: DomainRoomContext,
    fragment: DomainRoomProgram,
    entry_program: GraphActivityNodeProgram,
    menu: NodeId,
}
/// Immutable capability for one exact complete Activity definition and room.
#[derive(Clone, Debug)]
pub struct BoundShopRoom {
    room: CompiledShopRoom,
    definition: Arc<GraphActivityDefinition>,
}

impl DivergentUniverseRuntimeFactory {
    /// Explicit typed stock/price policy, not a recovered merchant. Only reviewed
    /// Shop-card contexts compile; other room kinds and fabricated contexts reject.
    /// The owning profile binds each compiled digest and uses the Curio-entry route.
    pub fn shop_room_compiler(
        &self,
        items: Vec<ShopStockItem>,
        slots: ShopRoomSlots,
    ) -> Result<ShopRoomCompiler, ShopRoomError> {
        if slots.accepted.get() < 70 || slots.accepted == slots.purchased {
            return Err(ShopRoomError::InvalidSlots);
        }
        let runtime = Arc::new(
            self.shop_purchase_runtime(items, slots.purchased)
                .map_err(ShopRoomError::Purchase)?,
        );
        let accepted = ActivitySlotDefinition::new_with_policy(
            slots.accepted,
            ActivityScope::Node,
            ActivityValue::Boolean(false),
            None,
            None,
            vec![SlotResetPoint::NodeStart],
            SlotCarryPolicy::Reset,
            ActivityStateVisibility::Player,
            ActivityStateSource::new(u64::from(slots.accepted.get()))
                .ok_or(ShopRoomError::InvalidSlots)?,
        )
        .map_err(|_| ShopRoomError::InvalidSlots)?;
        let declarations = vec![runtime.slot_definition().clone(), accepted];
        Ok(ShopRoomCompiler {
            factory: self.clone(),
            runtime,
            rewards: Arc::new(
                self.decision_reward_runtime()
                    .map_err(ShopRoomError::Reward)?,
            ),
            slots,
            declarations,
        })
    }
}
impl ShopRoomCompiler {
    /// Two physical nodes: one single entry, one finite regenerated Shop menu.
    /// No RNG is consumed by compilation or menu creation. Purchase reward draws
    /// occur only in the authenticated generated prefix after offered-ID checks.
    pub fn compile(&self, context: &DomainRoomContext) -> Result<CompiledShopRoom, ShopRoomError> {
        if !self.factory.room_context_matches(context)
            || context.composition != DomainRoomComposition::Card(DomainCardKind::Shop)
        {
            return Err(ShopRoomError::InvalidContext);
        }
        let menu = context.node(1).map_err(ShopRoomError::Route)?;
        let fragment = self.compile_graph(context, menu)?;
        let original = &fragment.programs[0];
        let entry_program = program(
            original.node(),
            self.runtime
                .curios
                .compiled_domain_entry_operations(original.program().operations().to_vec())
                .map_err(ShopPurchaseError::Curio)
                .map_err(ShopRoomError::Purchase)?,
        )?;
        Ok(CompiledShopRoom {
            compiler: self.clone(),
            context: context.clone(),
            fragment,
            entry_program,
            menu,
        })
    }
}
impl CompiledShopRoom {
    #[must_use]
    pub const fn context(&self) -> &DomainRoomContext {
        &self.context
    }
    #[must_use]
    pub const fn fragment(&self) -> &DomainRoomProgram {
        &self.fragment
    }
    #[must_use]
    pub fn slot_definitions(&self) -> &[ActivitySlotDefinition] {
        &self.compiler.declarations
    }
    #[must_use]
    pub const fn menu_node(&self) -> NodeId {
        self.menu
    }
    #[must_use]
    pub const fn accuracy(&self) -> ShopRoomAccuracy {
        ShopRoomAccuracy::VersionedProjectPolicyExplicitStockAtReviewedShopCardsOnePurchasePerItemNoRefresh
    }
    /// Current inputs, stock, policy, full source placement and physical layout.
    /// Owning profiles additionally bind graph, deck and bootstrap/other inputs.
    #[must_use]
    pub fn configuration_digest(&self) -> [u8; 32] {
        let mut digest = CanonicalDigestBuilder::new();
        digest.update(b"starclock.du.shop-room.reviewed-shop-card-context.explicit-stock.no-refresh.generated-acceptance-gate.dynamic-admission.one-entry.finite-self-menu.always-gated-leave");
        digest.update(self.compiler.runtime.configuration_digest());
        for value in [
            self.context.area.as_str(),
            self.context.layer.as_str(),
            self.context.position_key.as_ref(),
            self.context.preset_source.as_ref(),
        ] {
            digest.update(
                u64::try_from(value.len())
                    .expect("bounded source identity")
                    .to_le_bytes(),
            );
            digest.update(value.as_bytes());
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
        digest.update(self.context.position_ordinal.to_le_bytes());
        digest.update(self.context.plane_ordinal.to_le_bytes());
        digest.update(self.context.level.to_le_bytes());
        digest.update(self.compiler.slots.accepted.get().to_le_bytes());
        digest.update(LEAVE_SHOP.to_le_bytes());
        digest.finalize()
    }
}
fn program(
    node: NodeId,
    operations: Vec<ActivityOperation>,
) -> Result<GraphActivityNodeProgram, ShopRoomError> {
    let id = ActivityProgramId::new(node.get()).ok_or(ShopRoomError::DefinitionMismatch)?;
    Ok(GraphActivityNodeProgram::new(
        node,
        ActivityProgramDefinition::new(id, operations)
            .map_err(DomainRouteError::Program)
            .map_err(ShopRoomError::Route)?,
    ))
}
fn set(slot: ActivitySlotId, value: ActivityValue) -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot,
        value: ActivityExpression::Literal(value),
    }
}
fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
