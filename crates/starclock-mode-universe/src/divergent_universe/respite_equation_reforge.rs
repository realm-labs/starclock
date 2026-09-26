//! Policy-backed public Equation overwrite, sharing the existing Respite executor.

#[path = "respite_equation_reforge_graph.rs"]
mod graph;

use super::reforge::{RespiteReforgeSlots, counter, int, invalid, optional};
use super::{CompiledRespiteRoom, RespiteRoomError, integer, literal};
use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseEquationOfferRuntime, DivergentUniverseWorkbenchEquationReforgePolicy,
    state::{CURRENCIES_SLOT, SERVICE_RECEIPTS_SLOT, WORKBENCH_SLOT},
};
use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityOptionId, ActivityPlayerView, ActivityRngLabel,
    ActivityRngStreams, ActivityValue, GraphActivityCommandError, NodeId,
};
use starclock_data::divergent_universe_equation_catalog::{
    DivergentUniverseEquationCategory, DivergentUniverseEquationId,
};

pub const OPEN_EQUATION_REFORGE: u64 = u64::MAX - 101;
const DRAW_PURPOSE: u16 = 22_558;

/// Explicit caller-selected positive per-logical-room attempt limit (1..=64)
/// and checked increasing Fragment price. The numeric cap is NOT original
/// Workbench-limit parity. Candidate admission and mandatory confirmation are
/// separate explicit policies, not original selectors/payment timing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RespiteEquationReforgePolicy {
    price: DivergentUniverseWorkbenchEquationReforgePolicy,
    limit: u16,
}
impl RespiteEquationReforgePolicy {
    pub fn new(
        price: DivergentUniverseWorkbenchEquationReforgePolicy,
        limit: u16,
    ) -> Result<Self, RespiteRoomError> {
        if !(1..=64).contains(&limit) {
            return Err(RespiteRoomError::InvalidPolicy);
        }
        Ok(Self { price, limit })
    }
    #[must_use]
    pub const fn accuracy(&self) -> RespiteEquationReforgeAccuracy {
        RespiteEquationReforgeAccuracy::VersionedProjectPolicyCurrentSameQualityUniformThreeMandatoryConfirmationExplicitRoomLimit
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RespiteEquationReforgeAccuracy {
    VersionedProjectPolicyCurrentSameQualityUniformThreeMandatoryConfirmationExplicitRoomLimit,
}

#[derive(Clone, Debug)]
struct EquationCandidate {
    key: u64,
    id: DivergentUniverseEquationId,
    category: DivergentUniverseEquationCategory,
}
#[derive(Clone, Debug)]
pub(super) struct EquationReforgeRoom {
    runtime: DivergentUniverseEquationOfferRuntime,
    entries: Box<[EquationCandidate]>,
    policy: RespiteEquationReforgePolicy,
    slots: RespiteReforgeSlots,
    input: NodeId,
    output: NodeId,
    fragments: u64,
    receipt: u64,
    workbench: u64,
}
impl CompiledRespiteRoom {
    /// Adds an offered same-quality overwrite alongside existing services.
    /// All current unowned identities of the input's category are uniform
    /// policy candidates. Requires exact function-3 membership and disjoint
    /// host slots. Every command and automatic next menu uses the existing
    /// generated-choice transaction; no original NPC/pool/price parity is claimed.
    pub fn with_equation_reforge(
        mut self,
        policy: RespiteEquationReforgePolicy,
        slots: RespiteReforgeSlots,
    ) -> Result<Self, RespiteRoomError> {
        if self.equation_reforge.is_some() {
            return Err(RespiteRoomError::InvalidPolicy);
        }
        let factory = &self.compiler.factory;
        let (fragments, receipt) = factory
            .workbench_curse_runtime()
            .map_err(RespiteRoomError::Workbench)?
            .equation_reforge_service_keys(&self.compiler.workbench)
            .map_err(RespiteRoomError::Workbench)?;
        let entries = factory
            .bundle
            .equation_catalog()
            .equations()
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                Ok(EquationCandidate {
                    key: u64::try_from(index + 1).map_err(|_| RespiteRoomError::InvalidPolicy)?,
                    id: entry.id.clone(),
                    category: entry.category,
                })
            })
            .collect::<Result<Box<[_]>, RespiteRoomError>>()?;
        let first = u16::try_from(self.fragment.nodes.len())
            .map_err(|_| RespiteRoomError::InvalidPolicy)?;
        let service = EquationReforgeRoom {
            runtime: factory
                .equation_offer_runtime()
                .map_err(|_| RespiteRoomError::InvalidPolicy)?,
            entries,
            policy,
            slots,
            input: self.context.node(first).map_err(RespiteRoomError::Route)?,
            output: self
                .context
                .node(
                    first
                        .checked_add(1)
                        .ok_or(RespiteRoomError::InvalidPolicy)?,
                )
                .map_err(RespiteRoomError::Route)?,
            fragments,
            receipt,
            workbench: self.compiler.workbench_key,
        };
        self.add_service_slots(&slots.definitions()?)?;
        service.attach(&mut self)?;
        self.menus = self
            .menus
            .iter()
            .copied()
            .chain([service.input, service.output])
            .collect();
        self.equation_reforge = Some(service);
        Ok(self)
    }
    #[must_use]
    pub fn equation_reforge_input_node(&self) -> Option<NodeId> {
        self.equation_reforge.as_ref().map(|service| service.input)
    }
    #[must_use]
    pub fn equation_reforge_output_node(&self) -> Option<NodeId> {
        self.equation_reforge.as_ref().map(|service| service.output)
    }
}
impl EquationReforgeRoom {
    pub(super) fn configuration_digest(&self) -> [u8; 32] {
        let mut hash = CanonicalDigestBuilder::new();
        hash.update(b"starclock.du.respite-equation-reforge.current-same-quality.uniform-three.mandatory-confirmation.explicit-room-limit");
        hash.update(self.policy.price.configuration_digest());
        hash.update(self.policy.limit.to_le_bytes());
        hash.update(self.input.get().to_le_bytes());
        hash.update(self.output.get().to_le_bytes());
        for slot in [
            self.slots.selected,
            self.slots.offers,
            self.slots.completed,
            self.slots.accepted,
        ] {
            hash.update(slot.get().to_le_bytes());
        }
        // The exact catalog/category joins are bound by the parent source digest.
        hash.finalize()
    }
    pub(super) fn generate(
        &self,
        view: &ActivityPlayerView,
        option: ActivityOptionId,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        if view.current_node() != self.input && view.current_node() != self.output {
            return Ok(Vec::new());
        }
        if view.current_node() == self.input
            && !self.entries.iter().any(|entry| entry.key == option.get())
        {
            return Ok(Vec::new());
        }
        let owned = self
            .runtime
            .rewrite_owned_from_view(view)
            .map_err(|_| invalid())?;
        let price = self.check_price(view)?;
        if int(view, self.slots.completed)? >= i64::from(self.policy.limit) {
            return Err(invalid());
        }
        if view.current_node() == self.input {
            let input = self.entry(option.get())?;
            if !owned.contains(&input.key)
                || optional(view, self.slots.selected)?.is_some()
                || !counter(view, self.slots.offers)?.is_empty()
            {
                return Err(invalid());
            }
            let candidates = self
                .entries
                .iter()
                .filter(|entry| entry.category == input.category && !owned.contains(&entry.key))
                .collect::<Vec<_>>();
            if candidates.is_empty() {
                return Err(invalid());
            }
            let sample = rng
                .choose_weighted_without_replacement(
                    ActivityRngLabel::Reward,
                    DRAW_PURPOSE,
                    &vec![1; candidates.len()],
                    3,
                )
                .map_err(GraphActivityCommandError::Rng)?;
            let mut offers = sample
                .iter()
                .map(|index| {
                    let index = usize::try_from(*index).map_err(|_| invalid())?;
                    Ok((candidates.get(index).ok_or_else(invalid)?.key, 1))
                })
                .collect::<Result<Vec<_>, GraphActivityCommandError>>()?;
            offers.sort_unstable_by_key(|entry| entry.0);
            return Ok(vec![
                ActivityOperation::SetSlot {
                    slot: self.slots.selected,
                    value: literal(ActivityValue::OptionalId(Some(input.key))),
                },
                ActivityOperation::SetCounterMap {
                    slot: self.slots.offers,
                    values: offers.into(),
                },
                self.accepted(true),
            ]);
        }
        let input = self.entry(optional(view, self.slots.selected)?.ok_or_else(invalid)?)?;
        let output = self.entry(option.get())?;
        let offers = counter(view, self.slots.offers)?;
        if offers.is_empty()
            || offers.len() > 3
            || !offers.contains(&(output.key, 1))
            || offers.iter().any(|(key, level)| {
                *level != 1
                    || match self.entry(*key) {
                        Ok(entry) => entry.category != input.category || owned.contains(&entry.key),
                        Err(_) => true,
                    }
            })
        {
            return Err(invalid());
        }
        let mut operations = self
            .runtime
            .accepted_rewrite_operations(
                view,
                &input.id,
                &output.id,
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
                        slot: self.slots.completed,
                        value: ActivityExpression::Add(
                            Box::new(ActivityExpression::Slot(self.slots.completed)),
                            Box::new(integer(1)),
                        ),
                    },
                ],
                rng,
            )
            .map_err(|_| invalid())?;
        operations.extend([
            ActivityOperation::SetSlot {
                slot: self.slots.selected,
                value: literal(ActivityValue::OptionalId(None)),
            },
            ActivityOperation::SetCounterMap {
                slot: self.slots.offers,
                values: Box::new([]),
            },
            self.accepted(true),
        ]);
        Ok(operations)
    }
    fn entry(&self, key: u64) -> Result<&EquationCandidate, GraphActivityCommandError> {
        self.entries
            .iter()
            .find(|entry| entry.key == key)
            .ok_or_else(invalid)
    }
    fn check_price(&self, view: &ActivityPlayerView) -> Result<i64, GraphActivityCommandError> {
        if optional(view, WORKBENCH_SLOT)? != Some(self.workbench) {
            return Err(invalid());
        }
        let count = counter(view, SERVICE_RECEIPTS_SLOT)?
            .iter()
            .find(|entry| entry.0 == self.receipt)
            .map_or(0, |entry| entry.1);
        let price = self
            .policy
            .price
            .price_for_count(u64::try_from(count).map_err(|_| invalid())?)
            .map_err(|_| invalid())?;
        let price = i64::try_from(price).map_err(|_| invalid())?;
        if counter(view, CURRENCIES_SLOT)?
            .iter()
            .find(|entry| entry.0 == self.fragments)
            .map_or(0, |entry| entry.1)
            < price
        {
            return Err(invalid());
        }
        Ok(price)
    }
    fn accepted(&self, value: bool) -> ActivityOperation {
        ActivityOperation::SetSlot {
            slot: self.slots.accepted,
            value: literal(ActivityValue::Boolean(value)),
        }
    }
}
