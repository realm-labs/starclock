//! Current level-one Adventure settlement; challenges are explicit external inputs.

#[path = "adventure_room_binding.rs"]
mod binding;

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseCurrencyKind, DivergentUniverseRuntimeFactory,
    domain_route::{DomainRoomComposition, DomainRoomContext, DomainRoomProgram, DomainRouteError},
    economy::compile as compile_economy,
    state::{ROOM_DOORS_OPEN_SLOT, ROOM_FINISHED_SLOT, SERVICE_RECEIPTS_SLOT},
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityExpression, ActivityExternalOutcomeId, ActivityInteractionBinding,
    ActivityNodeDefinition, ActivityNodeKind, ActivityOperation, ActivityOptionDefinition,
    ActivityOptionId, ActivityProgramDefinition, ActivityProgramId, ActivityScope,
    ActivitySlotDefinition, ActivitySlotId, ActivityStateSource, ActivityStateVisibility,
    ActivityValue, GraphActivityDefinition, GraphActivityNodeProgram, NodeId, SlotCarryPolicy,
    SlotResetPoint,
};
use starclock_data::{
    divergent_universe_decisions::adventure_rewards::{
        AdventureRewardDefinition, AdventureRewardPolicy,
    },
    divergent_universe_domain_decks::DomainCardKind,
};
use std::sync::Arc;

pub const LEAVE_ADVENTURE: u64 = u64::MAX;
const RECEIPT: u64 = 0x2265_0001;

/// Typed host result, not a score, timer or inferred mini-game outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdventureEarnedChests {
    None,
    One,
    Two,
    Three,
}
impl AdventureEarnedChests {
    #[must_use]
    pub const fn count(self) -> u16 {
        match self {
            Self::None => 0,
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
        }
    }
    #[must_use]
    pub fn outcome(self) -> ActivityExternalOutcomeId {
        ActivityExternalOutcomeId::new(u64::from(self.count()) + 1)
            .expect("bounded external result has nonzero identity")
    }
}

#[derive(Debug)]
pub enum AdventureRoomError {
    InvalidSlot,
    InvalidContext,
    MissingReward,
    DefinitionMismatch,
    Route(DomainRouteError),
}
impl std::fmt::Display for AdventureRoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Divergent Universe Adventure: {self:?}")
    }
}
impl std::error::Error for AdventureRoomError {}

#[derive(Clone, Debug)]
pub struct AdventureRoomCompiler {
    factory: DivergentUniverseRuntimeFactory,
    earned: ActivitySlotDefinition,
}
#[derive(Clone, Debug)]
pub struct CompiledAdventureRoom {
    compiler: AdventureRoomCompiler,
    context: DomainRoomContext,
    reward: AdventureRewardDefinition,
    fragment: DomainRoomProgram,
    entry_program: GraphActivityNodeProgram,
    menu: NodeId,
    bindings: Box<[ActivityInteractionBinding]>,
}
#[derive(Clone, Debug)]
pub struct BoundAdventureRoom {
    room: CompiledAdventureRoom,
    definition: Arc<GraphActivityDefinition>,
}

impl DivergentUniverseRuntimeFactory {
    /// Host slot >=70 is a physical-node reset count (-1 means unresolved).
    /// Compilation/entry never grants rewards or simulates challenges.
    pub fn adventure_room_compiler(
        &self,
        earned: ActivitySlotId,
    ) -> Result<AdventureRoomCompiler, AdventureRoomError> {
        if earned.get() < 70 {
            return Err(AdventureRoomError::InvalidSlot);
        }
        let earned = ActivitySlotDefinition::new_with_policy(
            earned,
            ActivityScope::Node,
            ActivityValue::BoundedInteger(-1),
            Some((-1, 3)),
            None,
            vec![SlotResetPoint::NodeStart],
            SlotCarryPolicy::Reset,
            ActivityStateVisibility::Player,
            ActivityStateSource::new(u64::from(earned.get()))
                .ok_or(AdventureRoomError::InvalidSlot)?,
        )
        .map_err(|_| AdventureRoomError::InvalidSlot)?;
        Ok(AdventureRoomCompiler {
            factory: self.clone(),
            earned,
        })
    }
}
impl AdventureRoomCompiler {
    /// Only reviewed current Adventure cards with an authored level/preset policy.
    /// Old service candidates and guessed challenge membership are not admitted.
    pub fn compile(
        &self,
        context: &DomainRoomContext,
    ) -> Result<CompiledAdventureRoom, AdventureRoomError> {
        if !self.factory.room_context_matches(context)
            || context.composition != DomainRoomComposition::Card(DomainCardKind::Adventure)
        {
            return Err(AdventureRoomError::InvalidContext);
        }
        let reward = self
            .factory
            .decision_catalog()
            .adventure_rewards()
            .iter()
            .find(|row| row.preset_source == context.preset_source && row.level == context.level)
            .ok_or(AdventureRoomError::MissingReward)?
            .clone();
        match reward.policy {
            AdventureRewardPolicy::VersionedProjectPolicyExternalEarnedChestCountAggregateFragmentsOnly => {}
        }
        let currency = compile_economy(
            self.factory.bundle.service_catalog(),
            self.factory.bundle.progression_catalog(),
            self.factory.bundle.curio_catalog(),
            self.factory.decision_catalog(),
        )
        .map_err(|_| AdventureRoomError::MissingReward)?;
        let currency = currency.currency(DivergentUniverseCurrencyKind::CosmicFragment);
        let entry = context.entry_node();
        let menu = context.node(1).map_err(AdventureRoomError::Route)?;
        let enter = context.edge(0).map_err(AdventureRoomError::Route)?;
        let mut options = Vec::new();
        let mut bindings = Vec::new();
        for result in [
            AdventureEarnedChests::None,
            AdventureEarnedChests::One,
            AdventureEarnedChests::Two,
            AdventureEarnedChests::Three,
        ] {
            let count = result.count();
            let mut operations = vec![ActivityOperation::Require(unresolved(self.earned.id()))];
            if count != 0 {
                let amount = reward
                    .fragments_per_chest
                    .checked_mul(u64::from(count))
                    .ok_or(AdventureRoomError::MissingReward)?;
                operations.extend(
                    currency
                        .credit_operations(amount)
                        .map_err(|_| AdventureRoomError::MissingReward)?,
                );
            }
            operations.extend([
                ActivityOperation::SetSlot {
                    slot: self.earned.id(),
                    value: integer(i64::from(count)),
                },
                ActivityOperation::AddCounter {
                    slot: SERVICE_RECEIPTS_SLOT,
                    key: RECEIPT,
                    delta: integer(1),
                },
                boolean(ROOM_FINISHED_SLOT, true),
                boolean(ROOM_DOORS_OPEN_SLOT, true),
                ActivityOperation::Offer {
                    kind: ActivityDecisionKind::Route,
                    options: vec![ActivityOptionDefinition::new(
                        ActivityOptionId::new(LEAVE_ADVENTURE).expect("nonzero Leave"),
                        0,
                        finished(),
                        vec![
                            ActivityOperation::Require(finished()),
                            ActivityOperation::Traverse(context.exit_edge()),
                        ],
                    )]
                    .into_boxed_slice(),
                },
            ]);
            options.push(ActivityOptionDefinition::new(
                ActivityOptionId::new(result.outcome().get()).expect("nonzero result"),
                0,
                unresolved(self.earned.id()),
                operations,
            ));
            bindings.push(
                ActivityInteractionBinding::authored_option(
                    menu,
                    result.outcome(),
                    reward.key.clone(),
                )
                .map_err(|_| AdventureRoomError::DefinitionMismatch)?,
            );
        }
        let fragment = DomainRoomProgram {
            exit_node: menu,
            nodes: vec![
                ActivityNodeDefinition::new(entry, context.section, ActivityNodeKind::Choice, 1),
                ActivityNodeDefinition::new(
                    menu,
                    context.section,
                    ActivityNodeKind::ExternalOutcome,
                    1,
                ),
            ]
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(DomainRouteError::Graph)
            .map_err(AdventureRoomError::Route)?,
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
                .map_err(AdventureRoomError::Route)?,
            ],
            programs: vec![
                program(entry, vec![ActivityOperation::Traverse(enter)])?,
                program(
                    menu,
                    vec![ActivityOperation::Offer {
                        kind: ActivityDecisionKind::ExternalOutcome,
                        options: options.into_boxed_slice(),
                    }],
                )?,
            ],
            random_offers: Vec::new(),
        };
        let entry_program = program(
            entry,
            self.factory
                .curio_runtime()
                .map_err(|_| AdventureRoomError::DefinitionMismatch)?
                .compiled_domain_entry_operations(
                    fragment.programs[0].program().operations().to_vec(),
                )
                .map_err(|_| AdventureRoomError::DefinitionMismatch)?,
        )?;
        Ok(CompiledAdventureRoom {
            compiler: self.clone(),
            context: context.clone(),
            reward,
            fragment,
            entry_program,
            menu,
            bindings: bindings.into_boxed_slice(),
        })
    }
}
impl CompiledAdventureRoom {
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
        &self.compiler.earned
    }
    #[must_use]
    pub const fn menu_node(&self) -> NodeId {
        self.menu
    }
    #[must_use]
    pub fn interaction_bindings(&self) -> &[ActivityInteractionBinding] {
        &self.bindings
    }
    #[must_use]
    pub const fn reward(&self) -> &AdventureRewardDefinition {
        &self.reward
    }
    /// Owning profile must additionally bind every other immutable graph input.
    #[must_use]
    pub fn configuration_digest(&self) -> [u8; 32] {
        let mut digest = CanonicalDigestBuilder::new();
        digest.update(b"du.adventure.external-earned-chest-count.aggregate-credit.once.finish-then-doors.independent-leave.no-challenge-simulation");
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
            digest.update(u64::try_from(key.len()).expect("bounded key").to_le_bytes());
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
        digest.update(self.reward.maximum_chests.to_le_bytes());
        digest.update(self.reward.fragments_per_chest.to_le_bytes());
        digest.update(self.compiler.earned.id().get().to_le_bytes());
        digest.update(RECEIPT.to_le_bytes());
        digest.finalize()
    }
}
fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
fn boolean(slot: ActivitySlotId, value: bool) -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot,
        value: ActivityExpression::Literal(ActivityValue::Boolean(value)),
    }
}
fn unresolved(slot: ActivitySlotId) -> ActivityCondition {
    ActivityCondition::Equal(ActivityExpression::Slot(slot), integer(-1))
}
fn finished() -> ActivityCondition {
    ActivityCondition::All(
        vec![ROOM_FINISHED_SLOT, ROOM_DOORS_OPEN_SLOT]
            .into_iter()
            .map(|slot| ActivityCondition::Boolean(ActivityExpression::Slot(slot)))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    )
}
fn program(
    node: NodeId,
    operations: Vec<ActivityOperation>,
) -> Result<GraphActivityNodeProgram, AdventureRoomError> {
    Ok(GraphActivityNodeProgram::new(
        node,
        ActivityProgramDefinition::new(
            ActivityProgramId::new(node.get()).ok_or(AdventureRoomError::DefinitionMismatch)?,
            operations,
        )
        .map_err(|_| AdventureRoomError::DefinitionMismatch)?,
    ))
}
