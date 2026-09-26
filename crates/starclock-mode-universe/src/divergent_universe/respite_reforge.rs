//! Optional offered overwrite over an explicitly selected current closed group.

#[path = "respite_reforge_graph.rs"]
mod graph;

use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityOptionId, ActivityPlayerView, ActivityRngLabel,
    ActivityRngStreams, ActivityScope, ActivitySlotDefinition, ActivitySlotId, ActivityStateSource,
    ActivityStateVisibility, ActivityValue, GraphActivityCommandError, GraphActivityRuntimeError,
    MAX_ACTIVITY_OPTIONS, NodeId, SlotCarryPolicy, SlotResetPoint,
};
use starclock_data::divergent_universe_blessing_catalog::{
    DivergentUniverseBlessingGroupId, DivergentUniverseBlessingId,
};
use std::{iter::once, slice::from_ref};

use super::{CompiledRespiteRoom, RespiteRoomError, integer, literal};
use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseAcceptedBlessingRewrite, DivergentUniverseBlessingRuntime,
    DivergentUniverseLogicalScopeKind, DivergentUniverseWorkbenchBlessingReforgePolicy,
    state::{CURRENCIES_SLOT, SERVICE_RECEIPTS_SLOT, WORKBENCH_SLOT},
};

pub const OPEN_REFORGE: u64 = u64::MAX - 100;
const COMPLETION_LIMIT: i64 = 64;
const DRAW_PURPOSE: u16 = 22_557;

/// Caller-owned disjoint addresses, above the current core and deck allocation.
/// Selected input, candidates and completion count are logical-room scoped;
/// the transaction gate is physical-node scoped and cannot authorize raw choices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RespiteReforgeSlots {
    pub selected: ActivitySlotId,
    pub offers: ActivitySlotId,
    pub completed: ActivitySlotId,
    pub accepted: ActivitySlotId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RespiteReforgeAccuracy {
    VersionedProjectPolicyExplicitClosedGroupUniformThreeMandatoryConfirmation,
}

#[derive(Clone, Debug)]
pub(super) struct ReforgeRoom {
    runtime: DivergentUniverseBlessingRuntime,
    pool: Box<[(u64, DivergentUniverseBlessingId)]>,
    group: DivergentUniverseBlessingGroupId,
    price: DivergentUniverseWorkbenchBlessingReforgePolicy,
    addresses: RespiteReforgeSlots,
    pub(super) slots: Vec<ActivitySlotDefinition>,
    inputs: Box<[NodeId]>,
    output: NodeId,
    fragments: u64,
    receipt: u64,
    workbench: u64,
}

impl CompiledRespiteRoom {
    /// Adds offered overwrite to this exact fixed room. The selected group's
    /// current base identities form an explicit policy pool, NOT original
    /// Workbench selector proof. Three unowned identities are sampled uniformly
    /// without replacement (or all remaining when fewer). Input selection draws
    /// candidates; confirmation atomically pays and replaces. There is no free
    /// cancel/reroll after sampling. Other room services/placement remain pending.
    pub fn with_blessing_reforge(
        mut self,
        group: &DivergentUniverseBlessingGroupId,
        price: DivergentUniverseWorkbenchBlessingReforgePolicy,
        addresses: RespiteReforgeSlots,
    ) -> Result<Self, RespiteRoomError> {
        if self.reforge.is_some() {
            return Err(RespiteRoomError::InvalidPolicy);
        }
        let runtime = self
            .compiler
            .factory
            .blessing_runtime()
            .map_err(|_| RespiteRoomError::InvalidPolicy)?;
        let selected = runtime
            .groups()
            .iter()
            .find(|candidate| candidate.id() == group)
            .ok_or(RespiteRoomError::InvalidPolicy)?;
        let mut pool = selected
            .candidates()
            .iter()
            .filter(|candidate| candidate.level() == 1)
            .map(|candidate| (candidate.state_key(), candidate.blessing().clone()))
            .collect::<Vec<_>>();
        pool.sort_unstable_by_key(|value| value.0);
        if pool.is_empty()
            || pool.len() > MAX_ACTIVITY_OPTIONS
            || pool.windows(2).any(|pair| pair[0].0 == pair[1].0)
        {
            return Err(RespiteRoomError::InvalidPolicy);
        }
        let (fragments, receipt) = self
            .compiler
            .factory
            .workbench_curse_runtime()
            .map_err(RespiteRoomError::Workbench)?
            .reforge_service_keys(&self.compiler.workbench)
            .map_err(RespiteRoomError::Workbench)?;
        let first = u16::try_from(self.fragment.nodes.len())
            .map_err(|_| RespiteRoomError::InvalidPolicy)?;
        let inputs = (0..self.menus.len())
            .map(|index| {
                let index = u16::try_from(index)
                    .ok()
                    .and_then(|index| first.checked_add(index))
                    .ok_or(RespiteRoomError::InvalidPolicy)?;
                self.context.node(index).map_err(RespiteRoomError::Route)
            })
            .collect::<Result<Box<[_]>, _>>()?;
        let output_index = first
            .checked_add(u16::try_from(inputs.len()).map_err(|_| RespiteRoomError::InvalidPolicy)?)
            .ok_or(RespiteRoomError::InvalidPolicy)?;
        let service = ReforgeRoom {
            runtime,
            pool: pool.into(),
            group: group.clone(),
            price,
            addresses,
            slots: addresses.definitions()?,
            inputs,
            output: self
                .context
                .node(output_index)
                .map_err(RespiteRoomError::Route)?,
            fragments,
            receipt,
            workbench: self.compiler.workbench_key,
        };
        service.attach(&mut self)?;
        self.menus = self
            .menus
            .iter()
            .copied()
            .chain(service.inputs.iter().copied())
            .chain(once(service.output))
            .collect();
        self.reforge = Some(service);
        Ok(self)
    }

    /// Exact required room-state declarations; include these before whole-profile binding.
    #[must_use]
    pub fn slot_definitions(&self) -> &[ActivitySlotDefinition] {
        self.reforge.as_ref().map_or(&[], |reforge| &reforge.slots)
    }
    #[must_use]
    pub fn reforge_input_nodes(&self) -> &[NodeId] {
        self.reforge.as_ref().map_or(&[], |reforge| &reforge.inputs)
    }
    #[must_use]
    pub fn reforge_output_node(&self) -> Option<NodeId> {
        self.reforge.as_ref().map(|reforge| reforge.output)
    }
    #[must_use]
    pub fn reforge_accuracy(&self) -> Option<RespiteReforgeAccuracy> {
        self.reforge.as_ref().map(|_| RespiteReforgeAccuracy::VersionedProjectPolicyExplicitClosedGroupUniformThreeMandatoryConfirmation)
    }
}

impl RespiteReforgeSlots {
    fn definitions(self) -> Result<Vec<ActivitySlotDefinition>, RespiteRoomError> {
        let ids = [self.selected, self.offers, self.completed, self.accepted];
        if ids.iter().any(|id| id.get() < 70)
            || ids
                .iter()
                .enumerate()
                .any(|(index, id)| ids[..index].contains(id))
        {
            return Err(RespiteRoomError::InvalidPolicy);
        }
        let definitions = [
            (
                self.selected,
                ActivityValue::OptionalId(None),
                None,
                None,
                true,
            ),
            (
                self.offers,
                ActivityValue::BoundedCounterMap(Box::new([])),
                Some((1, 1)),
                Some(3),
                true,
            ),
            (
                self.completed,
                ActivityValue::BoundedInteger(0),
                Some((0, COMPLETION_LIMIT)),
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
        definitions
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
                        .ok_or(RespiteRoomError::InvalidPolicy)?,
                )
                .map_err(|_| RespiteRoomError::InvalidPolicy)?;
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

impl ReforgeRoom {
    pub(super) fn configuration_digest(&self) -> [u8; 32] {
        let mut hash = CanonicalDigestBuilder::new();
        hash.update(
            b"starclock.du.respite-reforge.explicit-group.uniform-three.mandatory-confirmation.v1",
        );
        hash.update(
            u64::try_from(self.group.as_str().len())
                .expect("bounded current group identity")
                .to_le_bytes(),
        );
        hash.update(self.group.as_str().as_bytes());
        hash.update(self.price.configuration_digest());
        for slot in [
            self.addresses.selected,
            self.addresses.offers,
            self.addresses.completed,
            self.addresses.accepted,
        ] {
            hash.update(slot.get().to_le_bytes());
        }
        hash.update(COMPLETION_LIMIT.to_le_bytes());
        hash.update(
            u64::try_from(self.pool.len())
                .expect("bounded current group pool")
                .to_le_bytes(),
        );
        for (key, _) in &self.pool {
            hash.update(key.to_le_bytes());
        }
        hash.finalize()
    }

    pub(super) fn generate(
        &self,
        view: &ActivityPlayerView,
        selected: ActivityOptionId,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        if self.inputs.contains(&view.current_node()) && selected.get() <= 414 {
            let owned = self.runtime.reward_owned(view).map_err(|_| invalid())?;
            let input = self
                .runtime
                .blessings()
                .iter()
                .find(|blessing| blessing.state_key() == selected.get())
                .ok_or_else(invalid)?;
            if !owned
                .iter()
                .any(|blessing| blessing.blessing() == input.id())
                || optional(view, self.addresses.selected)?.is_some()
                || !counter(view, self.addresses.offers)?.is_empty()
                || int(view, self.addresses.completed)? >= COMPLETION_LIMIT
            {
                return Err(invalid());
            }
            self.check_price(view)?;
            let candidates = self
                .pool
                .iter()
                .filter(|(_, id)| !owned.iter().any(|blessing| blessing.blessing() == id))
                .collect::<Vec<_>>();
            if candidates.is_empty() {
                return Err(invalid());
            }
            let sampled = rng
                .choose_weighted_without_replacement(
                    ActivityRngLabel::Reward,
                    DRAW_PURPOSE,
                    &vec![1; candidates.len()],
                    3,
                )
                .map_err(|_| invalid())?;
            let mut offers = sampled
                .iter()
                .map(|index| {
                    let index = usize::try_from(*index).map_err(|_| invalid())?;
                    Ok((candidates.get(index).ok_or_else(invalid)?.0, 1))
                })
                .collect::<Result<Vec<_>, GraphActivityCommandError>>()?;
            offers.sort_unstable_by_key(|value| value.0);
            return Ok(vec![
                ActivityOperation::SetSlot {
                    slot: self.addresses.selected,
                    value: literal(ActivityValue::OptionalId(Some(selected.get()))),
                },
                ActivityOperation::SetCounterMap {
                    slot: self.addresses.offers,
                    values: offers.into(),
                },
                self.accepted(true),
            ]);
        }
        if view.current_node() != self.output {
            return Ok(Vec::new());
        }
        let input = optional(view, self.addresses.selected)?.ok_or_else(invalid)?;
        let removed = self
            .runtime
            .blessings()
            .iter()
            .find(|blessing| blessing.state_key() == input)
            .ok_or_else(invalid)?;
        let owned = self.runtime.reward_owned(view).map_err(|_| invalid())?;
        let offers = counter(view, self.addresses.offers)?;
        if offers.is_empty()
            || offers.len() > 3
            || !offers
                .iter()
                .any(|(key, level)| *key == selected.get() && *level == 1)
            || offers.iter().any(|(key, level)| {
                *level != 1
                    || !self.pool.iter().any(|(candidate, id)| {
                        candidate == key && !owned.iter().any(|blessing| blessing.blessing() == id)
                    })
            })
            || int(view, self.addresses.completed)? >= COMPLETION_LIMIT
        {
            return Err(invalid());
        }
        let acquired = self
            .pool
            .iter()
            .find(|(key, _)| *key == selected.get())
            .ok_or_else(invalid)?;
        let rewrite =
            DivergentUniverseAcceptedBlessingRewrite::new(removed.id().clone(), acquired.1.clone())
                .map_err(|_| invalid())?;
        let price = self.check_price(view)?;
        let mut operations = self
            .runtime
            .rewrite_operations(
                view,
                from_ref(&rewrite),
                vec![
                    ActivityOperation::AddCounter {
                        slot: CURRENCIES_SLOT,
                        key: self.fragments,
                        delta: integer(-price),
                    },
                    ActivityOperation::AddCounter {
                        slot: SERVICE_RECEIPTS_SLOT,
                        key: self.receipt,
                        delta: integer(1),
                    },
                    ActivityOperation::SetSlot {
                        slot: self.addresses.completed,
                        value: ActivityExpression::Add(
                            Box::new(ActivityExpression::Slot(self.addresses.completed)),
                            Box::new(integer(1)),
                        ),
                    },
                ],
                rng,
            )
            .map_err(|_| invalid())?;
        operations.extend([
            ActivityOperation::SetSlot {
                slot: self.addresses.selected,
                value: literal(ActivityValue::OptionalId(None)),
            },
            ActivityOperation::SetCounterMap {
                slot: self.addresses.offers,
                values: Box::new([]),
            },
            self.accepted(true),
        ]);
        Ok(operations)
    }

    fn check_price(&self, view: &ActivityPlayerView) -> Result<i64, GraphActivityCommandError> {
        if optional(view, WORKBENCH_SLOT)? != Some(self.workbench) {
            return Err(invalid());
        }
        let receipts = counter(view, SERVICE_RECEIPTS_SLOT)?;
        let count = receipts
            .iter()
            .find(|(key, _)| *key == self.receipt)
            .map_or(0, |(_, value)| *value);
        let count = u64::try_from(count).map_err(|_| invalid())?;
        let price = i64::try_from(self.price.price_for_count(count).map_err(|_| invalid())?)
            .map_err(|_| invalid())?;
        let wallet = counter(view, CURRENCIES_SLOT)?;
        if wallet
            .iter()
            .find(|(key, _)| *key == self.fragments)
            .map_or(0, |(_, value)| *value)
            < price
        {
            return Err(invalid());
        }
        Ok(price)
    }
    fn accepted(&self, value: bool) -> ActivityOperation {
        ActivityOperation::SetSlot {
            slot: self.addresses.accepted,
            value: literal(ActivityValue::Boolean(value)),
        }
    }
}

fn value(
    view: &ActivityPlayerView,
    slot: ActivitySlotId,
) -> Result<&ActivityValue, GraphActivityCommandError> {
    view.slots()
        .iter()
        .find(|value| value.id() == slot)
        .map(|value| value.value())
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
fn counter(
    view: &ActivityPlayerView,
    slot: ActivitySlotId,
) -> Result<&[(u64, i64)], GraphActivityCommandError> {
    match value(view, slot)? {
        ActivityValue::BoundedCounterMap(value) => Ok(value),
        _ => Err(invalid()),
    }
}
fn int(view: &ActivityPlayerView, slot: ActivitySlotId) -> Result<i64, GraphActivityCommandError> {
    match value(view, slot)? {
        ActivityValue::BoundedInteger(value) => Ok(*value),
        _ => Err(invalid()),
    }
}
fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
