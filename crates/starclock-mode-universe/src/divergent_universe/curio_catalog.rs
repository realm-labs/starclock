//! Exact Curio identities and state bindings with explicit policy boundaries.

use std::sync::Arc;

use starclock_data::divergent_universe_curio_catalog::{
    DivergentUniverseCurioCatalog, DivergentUniverseCurioCategory, DivergentUniverseCurioGroupId,
    DivergentUniverseCurioId, DivergentUniverseCurioLifecycleId, DivergentUniverseCurioPoolId,
    DivergentUniverseCurioStateId,
};

use crate::path::ExactParameter;
use crate::path_lowering::parse_decimal;
use starclock_data::divergent_universe_decisions::CurioEvolutionDefinition;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseCurioAccuracy {
    ExactReleasedIdentitiesStatesEffectsTriggersAndCatalogMembership,
    VersionedProjectPolicyUnavailableOfferMembershipAndLifecycle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioStateRuntime {
    id: DivergentUniverseCurioStateId,
    curio: Option<DivergentUniverseCurioId>,
    evolution_owner: Option<DivergentUniverseCurioId>,
    state_key: u64,
    category: DivergentUniverseCurioCategory,
    effect_ids: Box<[Box<str>]>,
    effect_parameters: Box<[ExactParameter]>,
    trigger_kinds: Box<[Box<str>]>,
    mechanic_visibility: Box<str>,
    charge_source: Box<str>,
    declared_charges: Option<u16>,
    counter_parameter_index: u16,
}

impl DivergentUniverseCurioStateRuntime {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseCurioStateId {
        &self.id
    }
    #[must_use]
    pub const fn curio(&self) -> Option<&DivergentUniverseCurioId> {
        self.curio.as_ref()
    }
    /// Runtime-only policy alias; this does not alter the source handbook join
    /// returned by `curio`, or admit this state to an ordinary reward pool.
    #[must_use]
    pub const fn evolution_owner(&self) -> Option<&DivergentUniverseCurioId> {
        self.evolution_owner.as_ref()
    }
    #[must_use]
    pub const fn category(&self) -> DivergentUniverseCurioCategory {
        self.category
    }
    #[must_use]
    pub fn effect_ids(&self) -> &[Box<str>] {
        &self.effect_ids
    }
    #[must_use]
    pub fn effect_parameters(&self) -> &[ExactParameter] {
        &self.effect_parameters
    }
    #[must_use]
    pub fn trigger_kinds(&self) -> &[Box<str>] {
        &self.trigger_kinds
    }
    #[must_use]
    pub fn mechanic_visibility(&self) -> &str {
        &self.mechanic_visibility
    }
    #[must_use]
    pub fn charge_source(&self) -> &str {
        &self.charge_source
    }
    #[must_use]
    pub const fn declared_charges(&self) -> Option<u16> {
        self.declared_charges
    }
    #[must_use]
    pub const fn counter_parameter_index(&self) -> u16 {
        self.counter_parameter_index
    }
    pub(super) const fn state_key(&self) -> u64 {
        self.state_key
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioRuntimeDefinition {
    id: DivergentUniverseCurioId,
    category: DivergentUniverseCurioCategory,
    states: Box<[DivergentUniverseCurioStateId]>,
    lifecycle: DivergentUniverseCurioLifecycleId,
}

impl DivergentUniverseCurioRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseCurioId {
        &self.id
    }
    #[must_use]
    pub const fn category(&self) -> DivergentUniverseCurioCategory {
        self.category
    }
    #[must_use]
    pub fn states(&self) -> &[DivergentUniverseCurioStateId] {
        &self.states
    }
    #[must_use]
    pub const fn lifecycle(&self) -> &DivergentUniverseCurioLifecycleId {
        &self.lifecycle
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioGroupPolicy {
    id: DivergentUniverseCurioGroupId,
    consumers: Box<[Box<str>]>,
    eligibility: Box<str>,
    membership_resolution: Box<str>,
    fallback: Box<str>,
}

impl DivergentUniverseCurioGroupPolicy {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseCurioGroupId {
        &self.id
    }
    #[must_use]
    pub fn consumers(&self) -> &[Box<str>] {
        &self.consumers
    }
    #[must_use]
    pub fn eligibility(&self) -> &str {
        &self.eligibility
    }
    #[must_use]
    pub fn membership_resolution(&self) -> &str {
        &self.membership_resolution
    }
    #[must_use]
    pub fn fallback(&self) -> &str {
        &self.fallback
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioCatalogMembershipRuntime {
    state: DivergentUniverseCurioStateId,
    pool: DivergentUniverseCurioPoolId,
}

impl DivergentUniverseCurioCatalogMembershipRuntime {
    #[must_use]
    pub const fn state(&self) -> &DivergentUniverseCurioStateId {
        &self.state
    }
    #[must_use]
    pub const fn pool(&self) -> &DivergentUniverseCurioPoolId {
        &self.pool
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CompiledCurioCatalog {
    pub(super) curios: Arc<[DivergentUniverseCurioRuntimeDefinition]>,
    pub(super) states: Arc<[DivergentUniverseCurioStateRuntime]>,
    pub(super) groups: Arc<[DivergentUniverseCurioGroupPolicy]>,
    pub(super) memberships: Arc<[DivergentUniverseCurioCatalogMembershipRuntime]>,
}

impl CompiledCurioCatalog {
    pub(super) fn with_evolutions(
        mut self,
        evolutions: &[CurioEvolutionDefinition],
    ) -> Result<Self, CurioCatalogCompileError> {
        for edge in evolutions {
            let target = Arc::make_mut(&mut self.states)
                .iter_mut()
                .find(|state| state.id == edge.to)
                .ok_or(CurioCatalogCompileError("evolution target"))?;
            if target.curio.is_some() || target.evolution_owner.is_some() {
                return Err(CurioCatalogCompileError("evolution ownership"));
            }
            target.evolution_owner = Some(edge.owner.clone());
        }
        Ok(self)
    }
    pub(super) fn compile(
        catalog: &DivergentUniverseCurioCatalog,
    ) -> Result<Self, CurioCatalogCompileError> {
        let states = catalog
            .states()
            .iter()
            .enumerate()
            .map(|(index, state)| {
                if state.effect_ids.is_empty()
                    || state.effect_ids.iter().any(|value| value.is_empty())
                    || state.trigger_kinds.is_empty()
                    || state.trigger_kinds.iter().any(|value| value.is_empty())
                    || !matches!(
                        state.mechanic_visibility.as_ref(),
                        "CrossBattle"
                            | "BattleAndCrossBattle"
                            | "BattleVisible"
                            | "InventoryPassive"
                    )
                    || state.activation.as_ref() != "DefinedByReleasedEffectText"
                    || state.runtime_lowered
                {
                    return Err(CurioCatalogCompileError("state definition"));
                }
                let effect_parameters = state
                    .effect_parameters
                    .iter()
                    .map(|value| {
                        parse_decimal(value)
                            .map_err(|_| CurioCatalogCompileError("state parameter"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(DivergentUniverseCurioStateRuntime {
                    id: state.id.clone(),
                    curio: state.curio.clone(),
                    evolution_owner: None,
                    state_key: u64::try_from(index + 1)
                        .map_err(|_| CurioCatalogCompileError("state key"))?,
                    category: state.category,
                    effect_ids: state.effect_ids.clone(),
                    effect_parameters: effect_parameters.into_boxed_slice(),
                    trigger_kinds: state.trigger_kinds.clone(),
                    mechanic_visibility: state.mechanic_visibility.clone(),
                    charge_source: state.charges.clone(),
                    declared_charges: parse_declared_charges(&state.charges),
                    counter_parameter_index: state.counter_parameter_index,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let curios = catalog
            .curios()
            .iter()
            .map(|curio| {
                if curio.states.is_empty()
                    || curio.runtime_lowered
                    || curio.states.iter().any(|state| {
                        states
                            .binary_search_by(|candidate| candidate.id.cmp(state))
                            .ok()
                            .and_then(|index| states.get(index))
                            .is_none_or(|candidate| candidate.curio.as_ref() != Some(&curio.id))
                    })
                {
                    return Err(CurioCatalogCompileError("Curio definition"));
                }
                Ok(DivergentUniverseCurioRuntimeDefinition {
                    id: curio.id.clone(),
                    category: curio.category,
                    states: curio.states.clone(),
                    lifecycle: curio.lifecycle.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let groups = catalog
            .groups()
            .iter()
            .map(|group| {
                let unresolved_group = group.consumers.is_empty()
                    && group.eligibility.as_ref() == "Unspecified"
                    && group.membership_resolution.as_ref() == "UnavailableInReleasedGroupRow";
                let consumer_resolved_group = group.consumers.len() == 1
                    && group.consumers.iter().all(|consumer| !consumer.is_empty())
                    && matches!(
                        group.eligibility.as_ref(),
                        "MiracleCategory:Common"
                            | "MiracleCategory:Rare"
                            | "MiracleCategory:Legendary"
                            | "MiracleCategory:Negative"
                    )
                    && group.membership_resolution.as_ref()
                        == "ExactConsumerCategoryMembershipUnavailable";
                if !group.candidates.is_empty()
                    || !group.weights.is_empty()
                    || group.draw_count.as_ref() != "Unspecified"
                    || !(unresolved_group || consumer_resolved_group)
                    || group.fallback.as_ref() != "RejectWithoutMutation"
                    || group.runtime_lowered
                {
                    return Err(CurioCatalogCompileError("group definition"));
                }
                Ok(DivergentUniverseCurioGroupPolicy {
                    id: group.id.clone(),
                    consumers: group.consumers.clone(),
                    eligibility: group.eligibility.clone(),
                    membership_resolution: group.membership_resolution.clone(),
                    fallback: group.fallback.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let memberships = catalog
            .pool_membership()
            .iter()
            .map(|membership| {
                if membership.weight.as_ref() != "Unspecified"
                    || membership.eligibility.as_ref()
                        != "Tourn3CatalogOnly;OfferSpecificEligibilityUnspecified"
                    || membership.membership_basis.as_ref() != "ExplicitTourn3ModeAndCategory"
                    || !membership.source_groups.is_empty()
                    || membership.runtime_lowered
                    || states
                        .binary_search_by(|state| state.id.cmp(&membership.state))
                        .is_err()
                {
                    return Err(CurioCatalogCompileError("catalog membership"));
                }
                Ok(DivergentUniverseCurioCatalogMembershipRuntime {
                    state: membership.state.clone(),
                    pool: membership.pool.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if curios.len() != 179
            || states.len() != 235
            || states.iter().filter(|state| state.curio.is_some()).count() != 223
            || groups.len() != 286
            || memberships.len() != 235
            || catalog.lifecycle().len() != 179
            || catalog.lifecycle().iter().any(|lifecycle| {
                [
                    lifecycle.activation.as_ref(),
                    lifecycle.charges.as_ref(),
                    lifecycle.destruction.as_ref(),
                    lifecycle.repair.as_ref(),
                    lifecycle.replacement.as_ref(),
                    lifecycle.simultaneous_trigger_order.as_ref(),
                ]
                .iter()
                .any(|value| *value != "Unspecified")
                    || lifecycle.fallback.as_ref() != "RejectWithoutMutation"
                    || lifecycle.runtime_lowered
            })
        {
            return Err(CurioCatalogCompileError("catalog denominator or lifecycle"));
        }
        Ok(Self {
            curios: curios.into(),
            states: states.into(),
            groups: groups.into(),
            memberships: memberships.into(),
        })
    }
}

fn parse_declared_charges(value: &str) -> Option<u16> {
    value
        .parse::<u16>()
        .ok()
        .filter(|parsed| *parsed > 0 && parsed.to_string() == value)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CurioCatalogCompileError(pub(super) &'static str);
