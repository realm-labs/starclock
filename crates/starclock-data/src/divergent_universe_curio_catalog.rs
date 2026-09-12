//! Immutable Curio, weighted-pool and Grand Miracle definitions.

use std::collections::{BTreeMap, BTreeSet};

macro_rules! stable_id {
    ($name:ident,$prefix:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(Box<str>);
        impl $name {
            pub fn new(value: impl Into<Box<str>>) -> Result<Self, DivergentUniverseCurioError> {
                let value = value.into();
                if !value.starts_with($prefix) || value.len() == $prefix.len() {
                    Err(error(concat!(stringify!($name), " namespace mismatch")))
                } else {
                    Ok(Self(value))
                }
            }
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}
stable_id!(DivergentUniverseCurioId, "divergent-universe.curio.");
stable_id!(
    DivergentUniverseCurioStateId,
    "divergent-universe.curio-state."
);
stable_id!(
    DivergentUniverseCurioGroupId,
    "divergent-universe.curio-group."
);
stable_id!(
    DivergentUniverseCurioLifecycleId,
    "divergent-universe.curio-lifecycle."
);
stable_id!(
    DivergentUniverseCurioMembershipId,
    "divergent-universe.curio-catalog-membership."
);
stable_id!(
    DivergentUniverseCurioPoolId,
    "divergent-universe.curio-catalog."
);
stable_id!(
    DivergentUniverseGrandMiracleId,
    "divergent-universe.grand-miracle."
);
stable_id!(
    DivergentUniverseGrandMiracleEligibilityId,
    "divergent-universe.grand-miracle-eligibility."
);
stable_id!(
    DivergentUniverseGrandMiracleStateId,
    "divergent-universe.grand-miracle-state."
);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseCurioCategory {
    Common,
    Rare,
    Legendary,
    Negative,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioDefinition {
    pub id: DivergentUniverseCurioId,
    pub category: DivergentUniverseCurioCategory,
    pub states: Box<[DivergentUniverseCurioStateId]>,
    pub lifecycle: DivergentUniverseCurioLifecycleId,
    pub eligibility_rule_ids: Box<[Box<str>]>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioStateDefinition {
    pub id: DivergentUniverseCurioStateId,
    pub curio: Option<DivergentUniverseCurioId>,
    pub category: DivergentUniverseCurioCategory,
    pub effect_ids: Box<[Box<str>]>,
    pub effect_parameters: Box<[Box<str>]>,
    pub trigger_kinds: Box<[Box<str>]>,
    pub mechanic_visibility: Box<str>,
    pub charges: Box<str>,
    pub activation: Box<str>,
    pub destruction: Box<str>,
    pub repair: Box<str>,
    pub replacement: Box<str>,
    pub counter_parameter_index: u16,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioGroupDefinition {
    pub id: DivergentUniverseCurioGroupId,
    pub candidates: Box<[DivergentUniverseCurioStateId]>,
    pub consumers: Box<[Box<str>]>,
    pub weights: Box<[Box<str>]>,
    pub draw_count: Box<str>,
    pub eligibility: Box<str>,
    pub membership_resolution: Box<str>,
    pub fallback: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioLifecycleDefinition {
    pub id: DivergentUniverseCurioLifecycleId,
    pub curio: DivergentUniverseCurioId,
    pub activation: Box<str>,
    pub charges: Box<str>,
    pub destruction: Box<str>,
    pub repair: Box<str>,
    pub replacement: Box<str>,
    pub simultaneous_trigger_order: Box<str>,
    pub fallback: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioPoolMembershipDefinition {
    pub id: DivergentUniverseCurioMembershipId,
    pub state: DivergentUniverseCurioStateId,
    pub pool: DivergentUniverseCurioPoolId,
    pub source_groups: Box<[DivergentUniverseCurioGroupId]>,
    pub eligibility: Box<str>,
    pub membership_basis: Box<str>,
    pub weight: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseGrandMiracleDefinition {
    pub id: DivergentUniverseGrandMiracleId,
    pub states: Box<[DivergentUniverseGrandMiracleStateId]>,
    pub eligibility_rules: Box<[DivergentUniverseGrandMiracleEligibilityId]>,
    pub maze_buff_id: Box<str>,
    pub maze_buff_resolution: Box<str>,
    pub effect_ids: Box<[Box<str>]>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseGrandMiracleEligibilityDefinition {
    pub id: DivergentUniverseGrandMiracleEligibilityId,
    pub miracle: Option<DivergentUniverseGrandMiracleId>,
    pub selector_scope: Box<str>,
    pub character_paths: Box<[Box<str>]>,
    pub elements: Box<[Box<str>]>,
    pub eligibility: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseGrandMiracleStateDefinition {
    pub id: DivergentUniverseGrandMiracleStateId,
    pub miracle: DivergentUniverseGrandMiracleId,
    pub state: Box<str>,
    pub activation: Box<str>,
    pub duration: Box<str>,
    pub teardown: Box<str>,
    pub simultaneous_trigger_order: Box<str>,
    pub fallback: Box<str>,
    pub runtime_lowered: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioCatalogParts {
    pub curios: Vec<DivergentUniverseCurioDefinition>,
    pub states: Vec<DivergentUniverseCurioStateDefinition>,
    pub groups: Vec<DivergentUniverseCurioGroupDefinition>,
    pub lifecycle: Vec<DivergentUniverseCurioLifecycleDefinition>,
    pub pool_membership: Vec<DivergentUniverseCurioPoolMembershipDefinition>,
    pub miracles: Vec<DivergentUniverseGrandMiracleDefinition>,
    pub miracle_eligibility: Vec<DivergentUniverseGrandMiracleEligibilityDefinition>,
    pub miracle_states: Vec<DivergentUniverseGrandMiracleStateDefinition>,
    pub source_obligations: usize,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioCatalog {
    parts: DivergentUniverseCurioCatalogParts,
}

impl DivergentUniverseCurioCatalog {
    pub fn new(
        mut p: DivergentUniverseCurioCatalogParts,
    ) -> Result<Self, DivergentUniverseCurioError> {
        let counts = [
            (p.curios.len(), 179),
            (p.states.len(), 235),
            (p.groups.len(), 286),
            (p.lifecycle.len(), 179),
            (p.pool_membership.len(), 235),
            (p.miracles.len(), 17),
            (p.miracle_eligibility.len(), 74),
            (p.miracle_states.len(), 34),
        ];
        if counts.iter().any(|(actual, expected)| actual != expected) {
            return Err(error("Curio/Grand Miracle table denominator drift"));
        }
        if p.source_obligations != 774 {
            return Err(error("Curio source obligation closure drift"));
        }
        p.curios.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.states.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.groups.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.lifecycle.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.pool_membership.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.miracles.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.miracle_eligibility
            .sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.miracle_states.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        validate(&p)?;
        Ok(Self { parts: p })
    }
    #[must_use]
    pub fn curios(&self) -> &[DivergentUniverseCurioDefinition] {
        &self.parts.curios
    }
    #[must_use]
    pub fn states(&self) -> &[DivergentUniverseCurioStateDefinition] {
        &self.parts.states
    }
    #[must_use]
    pub fn groups(&self) -> &[DivergentUniverseCurioGroupDefinition] {
        &self.parts.groups
    }
    #[must_use]
    pub fn lifecycle(&self) -> &[DivergentUniverseCurioLifecycleDefinition] {
        &self.parts.lifecycle
    }
    #[must_use]
    pub fn pool_membership(&self) -> &[DivergentUniverseCurioPoolMembershipDefinition] {
        &self.parts.pool_membership
    }
    #[must_use]
    pub fn miracles(&self) -> &[DivergentUniverseGrandMiracleDefinition] {
        &self.parts.miracles
    }
    #[must_use]
    pub fn miracle_eligibility(&self) -> &[DivergentUniverseGrandMiracleEligibilityDefinition] {
        &self.parts.miracle_eligibility
    }
    #[must_use]
    pub fn miracle_states(&self) -> &[DivergentUniverseGrandMiracleStateDefinition] {
        &self.parts.miracle_states
    }
    #[must_use]
    pub const fn source_obligations(&self) -> usize {
        self.parts.source_obligations
    }
    #[cfg(test)]
    pub(crate) fn into_parts(self) -> DivergentUniverseCurioCatalogParts {
        self.parts
    }
}

fn validate(p: &DivergentUniverseCurioCatalogParts) -> Result<(), DivergentUniverseCurioError> {
    let curios = p
        .curios
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let states = p
        .states
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let lifecycle = p
        .lifecycle
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let miracles = p
        .miracles
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let eligibility = p
        .miracle_eligibility
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let miracle_states = p
        .miracle_states
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    if curios.len() != 179
        || states.len() != 235
        || lifecycle.len() != 179
        || miracles.len() != 17
        || eligibility.len() != 74
        || miracle_states.len() != 34
    {
        return Err(error("duplicate Curio identity"));
    }
    if p.curios.iter().any(|x| {
        x.runtime_lowered
            || x.states.is_empty()
            || !x.eligibility_rule_ids.is_empty()
            || lifecycle.get(&x.lifecycle).is_none_or(|l| l.curio != x.id)
            || x.states.iter().any(|id| {
                states
                    .get(id)
                    .is_none_or(|s| s.curio.as_ref() != Some(&x.id) || s.category != x.category)
            })
    }) {
        return Err(error("Curio state/lifecycle closure drift"));
    }
    if p.states.iter().any(|x| {
        x.runtime_lowered
            || x.effect_ids.len() != 1
            || x.trigger_kinds.is_empty()
            || x.activation.as_ref() != "DefinedByReleasedEffectText"
    }) {
        return Err(error("Curio state effect boundary drift"));
    }
    if p.groups.iter().any(|x| {
        x.runtime_lowered
            || !x.candidates.is_empty()
            || !x.weights.is_empty()
            || x.fallback.as_ref() != "RejectWithoutMutation"
    }) {
        return Err(error("Curio groups must remain fail closed"));
    }
    if p.lifecycle.iter().any(|x| {
        x.runtime_lowered
            || x.activation.as_ref() != "Unspecified"
            || x.charges.as_ref() != "Unspecified"
            || x.destruction.as_ref() != "Unspecified"
            || x.repair.as_ref() != "Unspecified"
            || x.replacement.as_ref() != "Unspecified"
            || x.fallback.as_ref() != "RejectWithoutMutation"
            || !curios.contains_key(&x.curio)
    }) {
        return Err(error("Curio lifecycle policy boundary drift"));
    }
    let memberships = p
        .pool_membership
        .iter()
        .map(|x| &x.state)
        .collect::<BTreeSet<_>>();
    if memberships.len() != 235
        || p.pool_membership.iter().any(|x| {
            x.runtime_lowered
                || !states.contains_key(&x.state)
                || !x.source_groups.is_empty()
                || x.weight.as_ref() != "Unspecified"
                || x.membership_basis.as_ref() != "ExplicitTourn3ModeAndCategory"
        })
    {
        return Err(error("weighted Curio pool boundary drift"));
    }
    if p.miracles.iter().any(|x| {
        x.runtime_lowered
            || x.states.len() != 2
            || x.eligibility_rules.len() != 1
            || x.maze_buff_resolution.as_ref() != "MissingReleasedRogueMazeBuffRow"
            || x.states
                .iter()
                .any(|id| miracle_states.get(id).is_none_or(|s| s.miracle != x.id))
            || x.eligibility_rules.iter().any(|id| {
                eligibility
                    .get(id)
                    .is_none_or(|e| e.miracle.as_ref() != Some(&x.id))
            })
    }) {
        return Err(error("Grand Miracle closure drift"));
    }
    if p.miracle_states.iter().any(|x| {
        x.runtime_lowered
            || !miracles.contains_key(&x.miracle)
            || x.activation.as_ref() != "Unspecified"
            || x.duration.as_ref() != "Unspecified"
            || x.teardown.as_ref() != "Unspecified"
            || x.fallback.as_ref() != "RejectWithoutMutation"
    }) {
        return Err(error("Grand Miracle lifecycle boundary drift"));
    }
    let current = p
        .miracle_eligibility
        .iter()
        .filter(|x| x.selector_scope.as_ref() == "Tourn3")
        .count();
    let excluded = p
        .miracle_eligibility
        .iter()
        .filter(|x| x.miracle.is_none())
        .count();
    if current != 17 || excluded != 57 || p.miracle_eligibility.iter().any(|x| x.runtime_lowered) {
        return Err(error("Grand Miracle eligibility boundary drift"));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurioError {
    message: Box<str>,
}
impl std::fmt::Display for DivergentUniverseCurioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for DivergentUniverseCurioError {}
fn error(message: &str) -> DivergentUniverseCurioError {
    DivergentUniverseCurioError {
        message: message.into(),
    }
}
