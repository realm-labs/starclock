//! Authenticated chest-only Wealth pickup at reviewed source contexts.
//! Exact chest selectors and amusement facilities remain unimplemented.

#[path = "coin_room_binding.rs"]
mod binding;

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseCurioRuntime, DivergentUniverseCurrencyKind, DivergentUniverseCurrencyRuntime,
    DivergentUniverseRuntimeFactory,
    domain_route::{DomainRoomComposition, DomainRoomContext, DomainRoomProgram, DomainRouteError},
    economy::compile as compile_economy,
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityExpression, ActivityNodeDefinition, ActivityNodeKind, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityScope, ActivitySlotDefinition, ActivitySlotId, ActivityStateSource,
    ActivityStateVisibility, ActivityValue, GraphActivityCommandError, GraphActivityDefinition,
    GraphActivityNodeProgram, GraphActivityRuntimeError, NodeId, SlotCarryPolicy, SlotResetPoint,
};
use starclock_data::{
    divergent_universe_decisions::coin_rewards::{CoinRewardDefinition, CoinRewardPolicy},
    divergent_universe_domain_decks::DomainCardKind,
    divergent_universe_domain_layout::FixedDomainKind,
};
use std::sync::Arc;

pub const COLLECT_CHEST: u64 = 1;
pub const LEAVE_WEALTH: u64 = u64::MAX;
const RECEIPT: u64 = 0x2264_0001;

#[derive(Debug)]
pub enum CoinRoomError {
    InvalidSlot,
    InvalidContext,
    DefinitionMismatch,
    MissingReward,
    Route(DomainRouteError),
}
impl std::fmt::Display for CoinRoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Divergent Universe Wealth chest: {self:?}")
    }
}
impl std::error::Error for CoinRoomError {}

#[derive(Clone, Debug)]
pub struct CoinRoomCompiler {
    factory: DivergentUniverseRuntimeFactory,
    accepted: ActivitySlotDefinition,
    currency: DivergentUniverseCurrencyRuntime,
    curios: DivergentUniverseCurioRuntime,
}

#[derive(Clone, Debug)]
pub struct CompiledCoinRoom {
    compiler: CoinRoomCompiler,
    context: DomainRoomContext,
    reward: CoinRewardDefinition,
    fragment: DomainRoomProgram,
    entry_program: GraphActivityNodeProgram,
    menu: NodeId,
}

/// Capability for one exact immutable whole definition, not a caller-supplied room ID.
#[derive(Clone, Debug)]
pub struct BoundCoinRoom {
    room: CompiledCoinRoom,
    definition: Arc<GraphActivityDefinition>,
}

impl DivergentUniverseRuntimeFactory {
    /// Current Sora-authored chest policy and shared credit pipeline. Host slot
    /// at or above 70 resets at each physical menu. No reward is paid by compilation/entry.
    /// This compiles chest-only gameplay, not a complete Wealth-domain service.
    pub fn coin_room_compiler(
        &self,
        accepted: ActivitySlotId,
    ) -> Result<CoinRoomCompiler, CoinRoomError> {
        if accepted.get() < 70 {
            return Err(CoinRoomError::InvalidSlot);
        }
        let accepted = ActivitySlotDefinition::new_with_policy(
            accepted,
            ActivityScope::Node,
            ActivityValue::Boolean(false),
            None,
            None,
            vec![SlotResetPoint::NodeStart],
            SlotCarryPolicy::Reset,
            ActivityStateVisibility::Player,
            ActivityStateSource::new(u64::from(accepted.get()))
                .ok_or(CoinRoomError::InvalidSlot)?,
        )
        .map_err(|_| CoinRoomError::InvalidSlot)?;
        let currency = compile_economy(
            self.bundle.service_catalog(),
            self.bundle.progression_catalog(),
            self.bundle.curio_catalog(),
            self.decision_catalog(),
        )
        .map_err(|_| CoinRoomError::MissingReward)?
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .clone();
        let curios = self
            .curio_runtime()
            .map_err(|_| CoinRoomError::MissingReward)?;
        Ok(CoinRoomCompiler {
            factory: self.clone(),
            accepted,
            currency,
            curios,
        })
    }
}

impl CoinRoomCompiler {
    /// Reviewed Coin fixed/card context with an explicitly authored preset/level
    /// reward only. Unknown/foreign/non-Coin contexts reject before producing work.
    pub fn compile(&self, context: &DomainRoomContext) -> Result<CompiledCoinRoom, CoinRoomError> {
        if !self.factory.room_context_matches(context)
            || !matches!(
                context.composition,
                DomainRoomComposition::Card(DomainCardKind::Coin)
                    | DomainRoomComposition::Fixed(FixedDomainKind::Coin)
            )
        {
            return Err(CoinRoomError::InvalidContext);
        }
        let reward = self
            .factory
            .decision_catalog()
            .coin_rewards()
            .iter()
            .find(|row| row.preset_source == context.preset_source && row.level == context.level)
            .ok_or(CoinRoomError::MissingReward)?
            .clone();
        match reward.policy {
            CoinRewardPolicy::VersionedProjectPolicyFixedSingleChestCreditNoAmusementFacilities => {
            }
        }
        let entry = context.entry_node();
        let menu = context.node(1).map_err(CoinRoomError::Route)?;
        let enter = context.edge(0).map_err(CoinRoomError::Route)?;
        let options = [COLLECT_CHEST, LEAVE_WEALTH]
            .into_iter()
            .map(|id| {
                ActivityOptionDefinition::new(
                    ActivityOptionId::new(id).expect("nonzero chest option"),
                    0,
                    ActivityCondition::Boolean(ActivityExpression::Literal(
                        ActivityValue::Boolean(true),
                    )),
                    vec![
                        ActivityOperation::Require(ActivityCondition::Boolean(
                            ActivityExpression::Slot(self.accepted.id()),
                        )),
                        set(self.accepted.id(), false),
                        ActivityOperation::Traverse(context.exit_edge()),
                    ],
                )
            })
            .collect::<Vec<_>>();
        let fragment = DomainRoomProgram {
            exit_node: menu,
            nodes: [entry, menu]
                .into_iter()
                .map(|node| {
                    ActivityNodeDefinition::new(node, context.section, ActivityNodeKind::Choice, 1)
                        .map_err(DomainRouteError::Graph)
                        .map_err(CoinRoomError::Route)
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
                .map_err(DomainRouteError::Graph)
                .map_err(CoinRoomError::Route)?,
            ],
            programs: vec![
                program(entry, vec![ActivityOperation::Traverse(enter)])?,
                program(
                    menu,
                    vec![
                        set(self.accepted.id(), false),
                        ActivityOperation::Offer {
                            kind: ActivityDecisionKind::Reward,
                            options: options.into_boxed_slice(),
                        },
                    ],
                )?,
            ],
            random_offers: Vec::new(),
        };
        let entry_program = program(
            entry,
            self.curios
                .compiled_domain_entry_operations(
                    fragment.programs[0].program().operations().to_vec(),
                )
                .map_err(|_| CoinRoomError::DefinitionMismatch)?,
        )?;
        Ok(CompiledCoinRoom {
            compiler: self.clone(),
            context: context.clone(),
            reward,
            fragment,
            entry_program,
            menu,
        })
    }
}

impl CompiledCoinRoom {
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
    pub const fn slot_definition(&self) -> &ActivitySlotDefinition {
        &self.compiler.accepted
    }
    #[must_use]
    pub const fn menu_node(&self) -> NodeId {
        self.menu
    }
    #[must_use]
    pub const fn reward(&self) -> &CoinRewardDefinition {
        &self.reward
    }
    /// Exact current inputs, selected reward, physical/scoped placement and policy.
    /// Owning profile additionally binds all remaining graph/state/party inputs.
    #[must_use]
    pub fn configuration_digest(&self) -> [u8; 32] {
        let mut digest = CanonicalDigestBuilder::new();
        digest.update(b"starclock.du.wealth.single-explicit-chest-credit.no-facilities.gated-collect-or-leave.credit-receipt-and-next-entry-atomic");
        digest.update(
            self.compiler
                .factory
                .bundle_identity()
                .component_digest()
                .bytes(),
        );
        digest.update(self.compiler.factory.decision_catalog().digest());
        for key in [
            self.reward.key.as_ref(),
            self.context.area.as_str(),
            self.context.layer.as_str(),
            self.context.position_key.as_ref(),
            self.context.preset_source.as_ref(),
        ] {
            digest.update(
                u64::try_from(key.len())
                    .expect("bounded current key")
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
        digest.update(self.reward.amount.to_le_bytes());
        digest.update(self.compiler.accepted.id().get().to_le_bytes());
        digest.update(RECEIPT.to_le_bytes());
        digest.finalize()
    }
}

fn set(slot: ActivitySlotId, value: bool) -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot,
        value: ActivityExpression::Literal(ActivityValue::Boolean(value)),
    }
}
fn program(
    node: NodeId,
    operations: Vec<ActivityOperation>,
) -> Result<GraphActivityNodeProgram, CoinRoomError> {
    Ok(GraphActivityNodeProgram::new(
        node,
        ActivityProgramDefinition::new(
            ActivityProgramId::new(node.get()).ok_or(CoinRoomError::DefinitionMismatch)?,
            operations,
        )
        .map_err(|_| CoinRoomError::DefinitionMismatch)?,
    ))
}
fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
