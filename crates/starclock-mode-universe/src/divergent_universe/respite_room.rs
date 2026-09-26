//! Optional Heat-funded enhancement at an explicitly admitted fixed Respite.
//! Prices, budget and Workbench admission are caller-owned project policy,
//! not source selectors. Optional overwrite has a separate host policy;
//! no automatic healing is inferred.

#[path = "respite_equation_reforge.rs"]
pub mod equation_reforge;
#[path = "respite_reforge.rs"]
pub mod reforge;

use equation_reforge::EquationReforgeRoom;
use reforge::ReforgeRoom;
use std::{iter::once, sync::Arc};

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseCurioRuntime, DivergentUniverseLogicalScopeKind,
    DivergentUniverseRuntimeFactory, DivergentUniverseWorkbenchCurseError,
    domain_route::{DomainRoomComposition, DomainRoomContext, DomainRoomProgram, DomainRouteError},
    state::{
        BLESSING_OFFER_SOURCE_SLOT, BLESSING_OFFERS_SLOT, BLESSINGS_SLOT, CURRENCIES_SLOT,
        EQUATION_PROGRESS_DIRTY_SLOT, SERVICE_RECEIPTS_SLOT, WORKBENCH_SLOT,
    },
};
use starclock_activity::{
    ActivityComparison, ActivityCondition, ActivityDecisionId, ActivityDecisionKind,
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityExpression, ActivityNodeDefinition,
    ActivityNodeKind, ActivityOperation, ActivityOptionDefinition, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivitySlotDefinition, ActivityStateHash,
    ActivityValue, GraphActivity, GraphActivityCommandError, GraphActivityDefinition,
    GraphActivityNodeProgram, GraphActivityRuntimeError, NodeId,
};
use starclock_data::divergent_universe_domain_layout::FixedDomainKind;
use starclock_data::divergent_universe_service_catalog::DivergentUniverseWorkbenchId;

const PAGE_WIDTH: usize = 128;
const LEAVE: u64 = u64::MAX;

/// Explicit host input to the existing accepted-price Workbench boundary.
/// A flat positive Heat price and a nonnegative per-room allowance; both are
/// configuration-bound. No rarity valuation or original price is inferred.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RespiteEnhancementPolicy {
    heat_allowance: i64,
    heat_price: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RespiteEnhancementAccuracy {
    VersionedProjectPolicyExplicitWorkbenchAdmissionHeatAllowanceAndFlatPrice,
}

impl RespiteEnhancementPolicy {
    #[must_use]
    pub const fn accuracy(&self) -> RespiteEnhancementAccuracy {
        RespiteEnhancementAccuracy::VersionedProjectPolicyExplicitWorkbenchAdmissionHeatAllowanceAndFlatPrice
    }

    pub fn new(heat_allowance: u64, heat_price: u64) -> Result<Self, RespiteRoomError> {
        let heat_allowance =
            i64::try_from(heat_allowance).map_err(|_| RespiteRoomError::InvalidPolicy)?;
        let heat_price = i64::try_from(heat_price).map_err(|_| RespiteRoomError::InvalidPolicy)?;
        if heat_price == 0 {
            return Err(RespiteRoomError::InvalidPolicy);
        }
        Ok(Self {
            heat_allowance,
            heat_price,
        })
    }
}

#[derive(Clone, Debug)]
pub struct RespiteRoomCompiler {
    factory: DivergentUniverseRuntimeFactory,
    curios: Arc<DivergentUniverseCurioRuntime>,
    workbench: DivergentUniverseWorkbenchId,
    workbench_key: u64,
    heat_key: u64,
    receipt_key: u64,
    policy: RespiteEnhancementPolicy,
}

/// Raw contribution for `compile_curio_domain_route`. The owning profile must
/// include `configuration_digest` in its payload and call `validate_definition`
/// on the completed graph. All actions are ordinary offered Activity commands.
#[derive(Clone, Debug)]
pub struct CompiledRespiteRoom {
    compiler: RespiteRoomCompiler,
    context: DomainRoomContext,
    fragment: DomainRoomProgram,
    entry_program: GraphActivityNodeProgram,
    menus: Box<[NodeId]>,
    enhancement_menus: Box<[NodeId]>,
    service_slots: Vec<ActivitySlotDefinition>,
    reforge: Option<ReforgeRoom>,
    equation_reforge: Option<EquationReforgeRoom>,
}

/// A trusted executor for one exact immutable whole definition. No external
/// work occurs inside the shared generated-option transaction.
#[derive(Clone, Debug)]
pub struct BoundRespiteRoom {
    room: CompiledRespiteRoom,
    definition: Arc<GraphActivityDefinition>,
}

#[derive(Debug)]
pub enum RespiteRoomError {
    InvalidPolicy,
    InvalidContext,
    DefinitionMismatch,
    Workbench(DivergentUniverseWorkbenchCurseError),
    Route(DomainRouteError),
}
impl std::fmt::Display for RespiteRoomError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Divergent Universe Respite room: {self:?}")
    }
}
impl std::error::Error for RespiteRoomError {}

impl DivergentUniverseRuntimeFactory {
    /// Admits a caller-selected current Workbench with the authored enhancement
    /// function. Unknown or transformation-only Workbenches reject. This is not
    /// original NPC placement, selector membership or complete Respite gameplay.
    pub fn respite_room_compiler(
        &self,
        workbench: &DivergentUniverseWorkbenchId,
        policy: RespiteEnhancementPolicy,
    ) -> Result<RespiteRoomCompiler, RespiteRoomError> {
        let runtime = self
            .workbench_curse_runtime()
            .map_err(RespiteRoomError::Workbench)?;
        let (workbench_key, heat_key, receipt_key) = runtime
            .enhancement_service_keys(workbench)
            .map_err(RespiteRoomError::Workbench)?;
        Ok(RespiteRoomCompiler {
            factory: self.clone(),
            curios: Arc::new(
                self.curio_runtime()
                    .map_err(DomainRouteError::Curio)
                    .map_err(RespiteRoomError::Route)?,
            ),
            workbench: workbench.clone(),
            workbench_key,
            heat_key,
            receipt_key,
            policy,
        })
    }
}

impl RespiteRoomCompiler {
    /// Only exact current fixed Respite contexts are admitted. Entry first runs
    /// the standard Curio lifetime prefix, then resets Heat to the explicit
    /// allowance. Internal menu returns neither reset Heat nor count a domain.
    /// Upgrade, debit, receipt and new offer commit together; Leave is always
    /// available, clears the Workbench and Heat, and traverses the sole exit.
    pub fn compile(
        &self,
        context: &DomainRoomContext,
    ) -> Result<CompiledRespiteRoom, RespiteRoomError> {
        if !self.factory.room_context_matches(context)
            || context.composition != DomainRoomComposition::Fixed(FixedDomainKind::Respite)
        {
            return Err(RespiteRoomError::InvalidContext);
        }
        let entry = context.entry_node();
        let blessings = self
            .factory
            .blessing_runtime()
            .map_err(|_| RespiteRoomError::DefinitionMismatch)?;
        let limit = u32::try_from(blessings.blessings().len())
            .map_err(|_| RespiteRoomError::DefinitionMismatch)?;
        let page_count = u16::try_from(blessings.blessings().len().div_ceil(PAGE_WIDTH))
            .map_err(|_| RespiteRoomError::DefinitionMismatch)?;
        let menus = (1..=page_count)
            .map(|index| context.node(index).map_err(RespiteRoomError::Route))
            .collect::<Result<Box<[_]>, _>>()?;
        let exit = context
            .node(
                page_count
                    .checked_add(1)
                    .ok_or(RespiteRoomError::DefinitionMismatch)?,
            )
            .map_err(RespiteRoomError::Route)?;
        let mut edges = Vec::new();
        let mut edge_index = 0_u16;
        let mut connect = |from, to, traversals| {
            let id = context.edge(edge_index).map_err(RespiteRoomError::Route)?;
            edge_index = edge_index
                .checked_add(1)
                .ok_or(RespiteRoomError::DefinitionMismatch)?;
            edges.push(
                ActivityEdgeDefinition::new(
                    id,
                    from,
                    to,
                    ActivityEdgeCondition::Always,
                    0,
                    traversals,
                )
                .map_err(DomainRouteError::Graph)
                .map_err(RespiteRoomError::Route)?,
            );
            Ok::<_, RespiteRoomError>(id)
        };
        let enter = connect(entry, menus[0], 1)?;
        let initialize = vec![
            ActivityOperation::SetCounter {
                slot: CURRENCIES_SLOT,
                key: self.heat_key,
                value: integer(self.policy.heat_allowance),
            },
            ActivityOperation::Traverse(enter),
        ];
        let mut candidates = Vec::new();
        for blessing in blessings.blessings() {
            let key = blessing.state_key();
            let available = ActivityCondition::All(
                vec![
                    ActivityCondition::Equal(
                        ActivityExpression::Slot(WORKBENCH_SLOT),
                        literal(ActivityValue::OptionalId(Some(self.workbench_key))),
                    ),
                    ActivityCondition::Equal(
                        ActivityExpression::CounterValue {
                            slot: BLESSINGS_SLOT,
                            key,
                        },
                        integer(1),
                    ),
                    ActivityCondition::Compare {
                        left: ActivityExpression::CounterValue {
                            slot: CURRENCIES_SLOT,
                            key: self.heat_key,
                        },
                        operator: ActivityComparison::GreaterOrEqual,
                        right: integer(self.policy.heat_price),
                    },
                    ActivityCondition::Equal(
                        ActivityExpression::Slot(BLESSING_OFFER_SOURCE_SLOT),
                        literal(ActivityValue::OptionalId(None)),
                    ),
                    ActivityCondition::Equal(
                        ActivityExpression::CounterEntryCount(BLESSING_OFFERS_SLOT),
                        integer(0),
                    ),
                    ActivityCondition::Equal(
                        ActivityExpression::Slot(EQUATION_PROGRESS_DIRTY_SLOT),
                        literal(ActivityValue::Boolean(false)),
                    ),
                ]
                .into(),
            );
            candidates.push((key, available));
        }
        let mut programs = Vec::new();
        for (page_index, page) in candidates.chunks(PAGE_WIDTH).enumerate() {
            let menu = menus[page_index];
            let again = connect(menu, menu, limit)?;
            let leave = connect(menu, exit, 1)?;
            let mut options = Vec::new();
            for (key, available) in page {
                options.push(ActivityOptionDefinition::new(
                    ActivityOptionId::new(*key).ok_or(RespiteRoomError::DefinitionMismatch)?,
                    0,
                    available.clone(),
                    vec![
                        ActivityOperation::Require(available.clone()),
                        ActivityOperation::SetCounter {
                            slot: BLESSINGS_SLOT,
                            key: *key,
                            value: integer(2),
                        },
                        ActivityOperation::AddCounter {
                            slot: CURRENCIES_SLOT,
                            key: self.heat_key,
                            delta: integer(-self.policy.heat_price),
                        },
                        ActivityOperation::AddCounter {
                            slot: SERVICE_RECEIPTS_SLOT,
                            key: self.receipt_key,
                            delta: integer(1),
                        },
                        ActivityOperation::Traverse(again),
                    ],
                ));
            }
            for (target, target_page) in candidates.chunks(PAGE_WIDTH).enumerate() {
                if target == page_index {
                    continue;
                }
                let available = ActivityCondition::Any(
                    target_page
                        .iter()
                        .map(|(_, available)| available.clone())
                        .collect(),
                );
                let route = connect(menu, menus[target], limit)?;
                let ordinal = u64::try_from(target)
                    .ok()
                    .and_then(|value| value.checked_add(1))
                    .ok_or(RespiteRoomError::DefinitionMismatch)?;
                options.push(ActivityOptionDefinition::new(
                    ActivityOptionId::new(
                        LEAVE
                            .checked_sub(ordinal)
                            .ok_or(RespiteRoomError::DefinitionMismatch)?,
                    )
                    .ok_or(RespiteRoomError::DefinitionMismatch)?,
                    0,
                    available.clone(),
                    vec![
                        ActivityOperation::Require(available),
                        ActivityOperation::Traverse(route),
                    ],
                ));
            }
            options.push(ActivityOptionDefinition::new(
                ActivityOptionId::new(LEAVE).expect("nonzero Leave key"),
                0,
                ActivityCondition::Boolean(literal(ActivityValue::Boolean(true))),
                vec![ActivityOperation::Traverse(leave)],
            ));
            options.sort_by_key(|option| (option.priority(), option.id()));
            programs.push(program(
                menu,
                // The existing Workbench identity is physical-node scoped.
                // Reassert this immutable service after that scope resets;
                // only the single room entry initializes the Heat allowance.
                vec![
                    ActivityOperation::SetSlot {
                        slot: WORKBENCH_SLOT,
                        value: literal(ActivityValue::OptionalId(Some(self.workbench_key))),
                    },
                    ActivityOperation::Offer {
                        kind: ActivityDecisionKind::Service,
                        options: options.into(),
                    },
                ],
            )?);
        }
        programs.push(program(
            exit,
            vec![
                ActivityOperation::SetSlot {
                    slot: WORKBENCH_SLOT,
                    value: literal(ActivityValue::OptionalId(None)),
                },
                ActivityOperation::SetCounter {
                    slot: CURRENCIES_SLOT,
                    key: self.heat_key,
                    value: integer(0),
                },
                ActivityOperation::Traverse(context.exit_edge()),
            ],
        )?);
        let raw_entry = program(entry, initialize.clone())?;
        let entry_program = program(
            entry,
            self.curios
                .compiled_domain_entry_operations(initialize)
                .map_err(DomainRouteError::Curio)
                .map_err(RespiteRoomError::Route)?,
        )?;
        programs.insert(0, raw_entry);
        let visits = limit
            .checked_add(1)
            .ok_or(RespiteRoomError::DefinitionMismatch)?;
        let nodes = once((entry, 1))
            .chain(menus.iter().map(|menu| (*menu, visits)))
            .chain(once((exit, 1)))
            .map(|(id, visits)| {
                ActivityNodeDefinition::new(id, context.section, ActivityNodeKind::Choice, visits)
                    .map_err(DomainRouteError::Graph)
                    .map_err(RespiteRoomError::Route)
            })
            .collect::<Result<_, _>>()?;
        Ok(CompiledRespiteRoom {
            compiler: self.clone(),
            context: context.clone(),
            entry_program,
            enhancement_menus: menus.clone(),
            menus,
            service_slots: Vec::new(),
            reforge: None,
            equation_reforge: None,
            fragment: DomainRoomProgram {
                exit_node: exit,
                nodes,
                edges,
                programs,
                random_offers: Vec::new(),
            },
        })
    }
}

impl CompiledRespiteRoom {
    pub(in crate::divergent_universe) fn matches_factory(
        &self,
        factory: &DivergentUniverseRuntimeFactory,
    ) -> bool {
        self.compiler.factory.bundle_identity().component_digest()
            == factory.bundle_identity().component_digest()
            && self.compiler.factory.decision_catalog().digest()
                == factory.decision_catalog().digest()
            && factory.room_context_matches(&self.context)
    }

    pub fn bind(
        &self,
        definition: Arc<GraphActivityDefinition>,
    ) -> Result<BoundRespiteRoom, RespiteRoomError> {
        self.validate_definition(&definition)?;
        Ok(BoundRespiteRoom {
            room: self.clone(),
            definition,
        })
    }
    /// Read-only physical menu addresses, all in the same logical room.
    #[must_use]
    pub fn menu_nodes(&self) -> &[NodeId] {
        &self.menus
    }
    #[must_use]
    pub fn fragment(&self) -> &DomainRoomProgram {
        &self.fragment
    }
    #[must_use]
    pub fn context(&self) -> &DomainRoomContext {
        &self.context
    }
    /// Binds all caller-selected policy inputs and exact source placement.
    #[must_use]
    pub fn configuration_digest(&self) -> [u8; 32] {
        let mut hash = CanonicalDigestBuilder::new();
        hash.update(b"starclock.du.explicit-respite-heat-enhancement.v1");
        hash.update(
            self.compiler
                .factory
                .bundle_identity()
                .component_digest()
                .bytes(),
        );
        hash.update(self.compiler.factory.decision_catalog().digest());
        for value in [
            self.compiler.workbench.as_str(),
            self.context.area.as_str(),
            self.context.layer.as_str(),
            self.context.position_key.as_ref(),
            self.context.preset_source.as_ref(),
        ] {
            hash.update(
                u64::try_from(value.len())
                    .expect("bounded authored string")
                    .to_le_bytes(),
            );
            hash.update(value.as_bytes());
        }
        hash.update(self.compiler.policy.heat_allowance.to_le_bytes());
        hash.update(self.compiler.policy.heat_price.to_le_bytes());
        hash.update(self.context.entry_node().get().to_le_bytes());
        hash.update(self.context.exit_edge().get().to_le_bytes());
        hash.update(self.context.successor().get().to_le_bytes());
        if let Some(reforge) = &self.reforge {
            hash.update(reforge.configuration_digest());
        }
        if let Some(reforge) = &self.equation_reforge {
            hash.update(reforge.configuration_digest());
        }
        hash.finalize()
    }
    /// Rejects changed programs, missing lifetime prefixes, bypass entry/exit,
    /// random policies or physical nodes assigned to a different logical room.
    /// Whole-profile binding separately authenticates its slots and identity.
    pub fn validate_definition(
        &self,
        definition: &GraphActivityDefinition,
    ) -> Result<(), RespiteRoomError> {
        let owns = |id| self.fragment.nodes.iter().any(|node| node.id() == id);
        let graph = definition.graph();
        if self
            .service_slots
            .iter()
            .any(|slot| !definition.state_definition().slots().contains(slot))
        {
            return Err(RespiteRoomError::DefinitionMismatch);
        }
        if definition.interactions().is_some()
            || self
                .fragment
                .nodes
                .iter()
                .any(|node| graph.node(node.id()) != Some(node))
            || self
                .fragment
                .edges
                .iter()
                .any(|edge| !graph.edges().contains(edge))
            || !graph.edges().iter().any(|edge| {
                edge.id() == self.context.exit_edge()
                    && edge.from() == self.fragment.exit_node
                    && edge.to() == self.context.successor()
                    && edge.condition() == ActivityEdgeCondition::Always
                    && edge.maximum_traversals() == 1
            })
            || graph.edges().iter().any(|edge| {
                (owns(edge.from())
                    && edge.id() != self.context.exit_edge()
                    && !self.fragment.edges.contains(edge))
                    || (!owns(edge.from())
                        && owns(edge.to())
                        && edge.to() != self.context.entry_node())
            })
            || self.fragment.programs.iter().any(|program| {
                !definition
                    .programs()
                    .contains(if program.node() == self.context.entry_node() {
                        &self.entry_program
                    } else {
                        program
                    })
            })
            || definition
                .random_offers()
                .iter()
                .any(|offer| owns(offer.node()))
            || definition
                .random_checkpoints()
                .iter()
                .any(|checkpoint| owns(checkpoint.node()))
            || self.fragment.nodes.iter().any(|node| {
                !definition
                    .state_definition()
                    .logical_scopes()
                    .bindings()
                    .iter()
                    .any(|binding| {
                        let path = binding.path();
                        binding.node() == node.id()
                            && path.len() == 3
                            && path[0].class() == DivergentUniverseLogicalScopeKind::Run.class_id()
                            && path[0].key() == 1
                            && path[1].class()
                                == DivergentUniverseLogicalScopeKind::Plane.class_id()
                            && path[1].key() == u64::from(self.context.plane_ordinal)
                            && path[2].class() == DivergentUniverseLogicalScopeKind::Node.class_id()
                            && path[2].key() == u64::from(self.context.position_ordinal)
                    })
            })
        {
            return Err(RespiteRoomError::DefinitionMismatch);
        }
        Ok(())
    }
}

impl BoundRespiteRoom {
    /// Only this exact definition's active menu pages are recognized.
    #[must_use]
    pub fn offered(&self, activity: &GraphActivity) -> Option<&DivergentUniverseWorkbenchId> {
        let actual = activity.definition();
        let matches = Arc::ptr_eq(actual, &self.definition)
            || (actual.identity() == self.definition.identity()
                && actual.graph().digest() == self.definition.graph().digest()
                && actual.state_definition() == self.definition.state_definition()
                && actual.participants().digest() == self.definition.participants().digest()
                && actual.programs() == self.definition.programs()
                && actual.bootstrap() == self.definition.bootstrap()
                && actual.random_offers() == self.definition.random_offers()
                && actual.random_checkpoints() == self.definition.random_checkpoints()
                && actual.interactions().is_none());
        (matches
            && self.room.menus.contains(&activity.current_node())
            && activity
                .player_view()
                .decision()
                .is_some_and(|offer| offer.kind() == ActivityDecisionKind::Service))
        .then_some(&self.room.compiler.workbench)
    }
    /// Uses the existing generated-option transaction even though all paid
    /// operations are statically authored. This extends rollback through the
    /// automatic destination pump; ordinary raw graph choices retain the shared
    /// low-level fault semantics and are not this mode-executor entry point.
    pub fn choose(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
    ) -> Result<(), GraphActivityCommandError> {
        if expected != activity.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        if self.offered(activity).is_none() {
            return Err(GraphActivityCommandError::Runtime(
                GraphActivityRuntimeError::InvalidBoundaryProgram,
            ));
        }
        activity.choose_option_with_generated_prefix(expected, decision, option, |view, rng| {
            let mut operations = self.room.reforge.as_ref().map_or_else(
                || Ok(Vec::new()),
                |reforge| reforge.generate(view, option, rng),
            )?;
            if let Some(reforge) = &self.room.equation_reforge {
                operations.extend(reforge.generate(view, option, rng)?);
            }
            Ok((operations, ()))
        })?;
        Ok(())
    }
}

fn program(
    node: NodeId,
    operations: Vec<ActivityOperation>,
) -> Result<GraphActivityNodeProgram, RespiteRoomError> {
    let id = ActivityProgramId::new(node.get()).ok_or(RespiteRoomError::DefinitionMismatch)?;
    Ok(GraphActivityNodeProgram::new(
        node,
        ActivityProgramDefinition::new(id, operations)
            .map_err(DomainRouteError::Program)
            .map_err(RespiteRoomError::Route)?,
    ))
}
fn literal(value: ActivityValue) -> ActivityExpression {
    ActivityExpression::Literal(value)
}
fn integer(value: i64) -> ActivityExpression {
    literal(ActivityValue::BoundedInteger(value))
}
