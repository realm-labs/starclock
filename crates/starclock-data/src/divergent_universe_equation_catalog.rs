//! Immutable Equation definitions for Divergent Universe.

use std::collections::BTreeMap;

macro_rules! stable_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(Box<str>);
        impl $name {
            pub fn new(value: impl Into<Box<str>>) -> Result<Self, DivergentUniverseEquationError> {
                let value = value.into();
                if !value.starts_with($prefix) || value.len() == $prefix.len() {
                    return Err(error(concat!(stringify!($name), " namespace mismatch")));
                }
                Ok(Self(value))
            }
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

stable_id!(DivergentUniverseEquationId, "divergent-universe.equation.");
stable_id!(
    DivergentUniverseEquationRecipeId,
    "divergent-universe.equation-recipe."
);
stable_id!(
    DivergentUniverseEquationCategoryId,
    "divergent-universe.equation-category."
);
stable_id!(
    DivergentUniverseEquationOfferId,
    "divergent-universe.equation-offer."
);
stable_id!(
    DivergentUniverseEquationProgressId,
    "divergent-universe.equation-progress."
);
stable_id!(
    DivergentUniverseEquationStateId,
    "divergent-universe.equation-state."
);
stable_id!(
    DivergentUniverseEquationEffectId,
    "divergent-universe.equation-effect."
);
stable_id!(
    DivergentUniverseEquationTransitionId,
    "divergent-universe.equation-transition."
);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DivergentUniversePathType(Box<str>);

impl DivergentUniversePathType {
    pub fn new(value: impl Into<Box<str>>) -> Result<Self, DivergentUniverseEquationError> {
        let value = value.into();
        if !matches!(
            value.as_ref(),
            "121" | "122" | "124" | "125" | "126" | "127" | "128" | "129"
        ) {
            return Err(error("unknown active Divergent Universe Path type"));
        }
        Ok(Self(value))
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseEquationCategory {
    Rare,
    Epic,
    Legendary,
    PathEcho,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseEquationState {
    Expanded,
    Unexpanded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationDefinition {
    pub id: DivergentUniverseEquationId,
    pub category: DivergentUniverseEquationCategory,
    pub display_id: Box<str>,
    pub display_extra_effect_ids: Box<[Box<str>]>,
    pub effect_ids: Box<[DivergentUniverseEquationEffectId]>,
    pub handbook_visible: bool,
    pub main_path: DivergentUniversePathType,
    pub sub_path: Option<DivergentUniversePathType>,
    pub maze_buff_id: Box<str>,
    pub recipe: DivergentUniverseEquationRecipeId,
    pub story_payload_included: bool,
    pub runtime_lowered: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationCategoryDefinition {
    pub id: DivergentUniverseEquationCategoryId,
    pub category: DivergentUniverseEquationCategory,
    pub equations: Box<[DivergentUniverseEquationId]>,
    pub expansion_boundary: Box<str>,
    pub offer_rule_ids: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationRecipeDefinition {
    pub id: DivergentUniverseEquationRecipeId,
    pub equation: DivergentUniverseEquationId,
    pub main_path: DivergentUniversePathType,
    pub main_count: u16,
    pub sub_path: Option<DivergentUniversePathType>,
    pub sub_count: u16,
    pub contribution_unit: Box<str>,
    pub enhanced_level_contribution: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationOfferDefinition {
    pub id: DivergentUniverseEquationOfferId,
    pub candidate_ids: Box<[DivergentUniverseEquationId]>,
    pub consumer_ids: Box<[Box<str>]>,
    pub selection_count: Box<str>,
    pub weight_program: Box<str>,
    pub replacement_allowed: Box<str>,
    pub no_legal_candidate: Box<str>,
    pub runtime_lowered: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationProgressDefinition {
    pub id: DivergentUniverseEquationProgressId,
    pub equation: DivergentUniverseEquationId,
    pub recipe: DivergentUniverseEquationRecipeId,
    pub main_required: u16,
    pub sub_required: u16,
    pub storage: Box<str>,
    pub refresh_trigger: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationStateDefinition {
    pub id: DivergentUniverseEquationStateId,
    pub equation: DivergentUniverseEquationId,
    pub state: DivergentUniverseEquationState,
    pub entry_condition: Box<str>,
    pub exit_condition: Box<str>,
    pub effect_active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationEffectDefinition {
    pub id: DivergentUniverseEquationEffectId,
    pub current_path: bool,
    pub path_type_id: Box<str>,
    pub keyword_id: Box<str>,
    pub maze_buff_id: Box<str>,
    pub maze_buff_ids: Box<[Box<str>]>,
    pub extra_effect_id: Box<str>,
    pub keyword_extra_effect_id: Box<str>,
    pub formula_source_ids: Box<[Box<str>]>,
    pub parameters: Box<[Box<str>]>,
    pub rule_contribution_ids: Box<[Box<str>]>,
    pub runtime_lowered: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationTransitionDefinition {
    pub id: DivergentUniverseEquationTransitionId,
    pub operation: Box<str>,
    pub candidate_policy: Box<str>,
    pub ordered_operations: Box<[Box<str>]>,
    pub no_legal_candidate: Box<str>,
    pub preserved_state: Box<str>,
    pub runtime_lowered: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationCatalogParts {
    pub equations: Vec<DivergentUniverseEquationDefinition>,
    pub categories: Vec<DivergentUniverseEquationCategoryDefinition>,
    pub recipes: Vec<DivergentUniverseEquationRecipeDefinition>,
    pub offers: Vec<DivergentUniverseEquationOfferDefinition>,
    pub progress: Vec<DivergentUniverseEquationProgressDefinition>,
    pub states: Vec<DivergentUniverseEquationStateDefinition>,
    pub effects: Vec<DivergentUniverseEquationEffectDefinition>,
    pub transitions: Vec<DivergentUniverseEquationTransitionDefinition>,
    pub source_obligations: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationCatalog {
    equations: Box<[DivergentUniverseEquationDefinition]>,
    categories: Box<[DivergentUniverseEquationCategoryDefinition]>,
    recipes: Box<[DivergentUniverseEquationRecipeDefinition]>,
    offers: Box<[DivergentUniverseEquationOfferDefinition]>,
    progress: Box<[DivergentUniverseEquationProgressDefinition]>,
    states: Box<[DivergentUniverseEquationStateDefinition]>,
    effects: Box<[DivergentUniverseEquationEffectDefinition]>,
    transitions: Box<[DivergentUniverseEquationTransitionDefinition]>,
    source_obligations: usize,
}

impl DivergentUniverseEquationCatalog {
    pub fn new(
        mut parts: DivergentUniverseEquationCatalogParts,
    ) -> Result<Self, DivergentUniverseEquationError> {
        for (actual, expected, label) in [
            (parts.equations.len(), 80, "equations"),
            (parts.categories.len(), 4, "equation categories"),
            (parts.recipes.len(), 80, "equation recipes"),
            (parts.offers.len(), 136, "equation offers"),
            (parts.progress.len(), 80, "equation progress"),
            (parts.states.len(), 160, "equation states"),
            (parts.effects.len(), 25, "equation effects"),
            (parts.transitions.len(), 4, "equation transitions"),
        ] {
            if actual != expected {
                return Err(error(&format!("expected {expected} {label}, got {actual}")));
            }
        }
        if parts.source_obligations != 330 {
            return Err(error("Equation source obligation closure drift"));
        }
        parts.equations.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        parts.categories.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        parts.recipes.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        parts.offers.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        parts.progress.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        parts.states.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        parts.effects.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        parts.transitions.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        validate(&parts)?;
        Ok(Self {
            equations: parts.equations.into_boxed_slice(),
            categories: parts.categories.into_boxed_slice(),
            recipes: parts.recipes.into_boxed_slice(),
            offers: parts.offers.into_boxed_slice(),
            progress: parts.progress.into_boxed_slice(),
            states: parts.states.into_boxed_slice(),
            effects: parts.effects.into_boxed_slice(),
            transitions: parts.transitions.into_boxed_slice(),
            source_obligations: parts.source_obligations,
        })
    }
    #[must_use]
    pub const fn equations(&self) -> &[DivergentUniverseEquationDefinition] {
        &self.equations
    }
    #[must_use]
    pub const fn categories(&self) -> &[DivergentUniverseEquationCategoryDefinition] {
        &self.categories
    }
    #[must_use]
    pub const fn recipes(&self) -> &[DivergentUniverseEquationRecipeDefinition] {
        &self.recipes
    }
    #[must_use]
    pub const fn offers(&self) -> &[DivergentUniverseEquationOfferDefinition] {
        &self.offers
    }
    #[must_use]
    pub const fn progress(&self) -> &[DivergentUniverseEquationProgressDefinition] {
        &self.progress
    }
    #[must_use]
    pub const fn states(&self) -> &[DivergentUniverseEquationStateDefinition] {
        &self.states
    }
    #[must_use]
    pub const fn effects(&self) -> &[DivergentUniverseEquationEffectDefinition] {
        &self.effects
    }
    #[must_use]
    pub const fn transitions(&self) -> &[DivergentUniverseEquationTransitionDefinition] {
        &self.transitions
    }
    #[must_use]
    pub const fn source_obligations(&self) -> usize {
        self.source_obligations
    }
    #[must_use]
    pub fn contains(&self, id: &DivergentUniverseEquationId) -> bool {
        self.equations
            .binary_search_by(|value| value.id.cmp(id))
            .is_ok()
    }
    #[cfg(test)]
    pub(crate) fn into_parts(self) -> DivergentUniverseEquationCatalogParts {
        DivergentUniverseEquationCatalogParts {
            equations: self.equations.into_vec(),
            categories: self.categories.into_vec(),
            recipes: self.recipes.into_vec(),
            offers: self.offers.into_vec(),
            progress: self.progress.into_vec(),
            states: self.states.into_vec(),
            effects: self.effects.into_vec(),
            transitions: self.transitions.into_vec(),
            source_obligations: self.source_obligations,
        }
    }
}

fn validate(
    parts: &DivergentUniverseEquationCatalogParts,
) -> Result<(), DivergentUniverseEquationError> {
    unique(&parts.equations, |v| &v.id, "equation")?;
    unique(&parts.categories, |v| &v.id, "category")?;
    unique(&parts.recipes, |v| &v.id, "recipe")?;
    unique(&parts.offers, |v| &v.id, "offer")?;
    unique(&parts.progress, |v| &v.id, "progress")?;
    unique(&parts.states, |v| &v.id, "state")?;
    unique(&parts.effects, |v| &v.id, "effect")?;
    unique(&parts.transitions, |v| &v.id, "transition")?;
    let recipes = parts
        .recipes
        .iter()
        .map(|v| (&v.id, v))
        .collect::<BTreeMap<_, _>>();
    let progress = parts
        .progress
        .iter()
        .map(|v| (&v.equation, v))
        .collect::<BTreeMap<_, _>>();
    let states = parts
        .states
        .iter()
        .fold(BTreeMap::<_, Vec<_>>::new(), |mut m, v| {
            m.entry(&v.equation).or_default().push(v);
            m
        });
    let equations = parts
        .equations
        .iter()
        .map(|v| (&v.id, v))
        .collect::<BTreeMap<_, _>>();
    if equations.len() != 80
        || parts.equations.iter().any(|v| {
            v.runtime_lowered
                || v.story_payload_included
                || v.effect_ids.len() != 1
                || recipes.get(&v.recipe).is_none_or(|r| {
                    r.equation != v.id || r.main_path != v.main_path || r.sub_path != v.sub_path
                })
                || progress.get(&v.id).is_none_or(|p| p.recipe != v.recipe)
                || states.get(&v.id).is_none_or(|s| {
                    s.len() != 2 || s.iter().filter(|x| x.effect_active).count() != 1
                })
        })
    {
        return Err(error(
            "Equation definition/recipe/progress/state closure drift",
        ));
    }
    let distribution = parts.equations.iter().fold(BTreeMap::new(), |mut m, v| {
        *m.entry(v.category).or_insert(0usize) += 1;
        m
    });
    if distribution
        != BTreeMap::from([
            (DivergentUniverseEquationCategory::Epic, 24),
            (DivergentUniverseEquationCategory::Legendary, 16),
            (DivergentUniverseEquationCategory::PathEcho, 8),
            (DivergentUniverseEquationCategory::Rare, 32),
        ])
    {
        return Err(error("Equation category distribution drift"));
    }
    let categorized = parts
        .categories
        .iter()
        .flat_map(|c| c.equations.iter().map(move |id| (id, c.category)))
        .collect::<BTreeMap<_, _>>();
    if categorized.len() != 80
        || parts
            .equations
            .iter()
            .any(|v| categorized.get(&v.id) != Some(&v.category))
        || parts.categories.iter().any(|v| {
            v.expansion_boundary.as_ref() != "RecipeCountsSatisfied" || !v.offer_rule_ids.is_empty()
        })
    {
        return Err(error("Equation category closure drift"));
    }
    if parts.offers.iter().any(|v| {
        !v.candidate_ids.is_empty()
            || !v.consumer_ids.is_empty()
            || v.selection_count.as_ref() != "Unspecified"
            || v.weight_program.as_ref() != "Unspecified"
            || v.replacement_allowed.as_ref() != "Unspecified"
            || v.no_legal_candidate.as_ref() != "Unspecified"
            || v.runtime_lowered
    }) || parts.effects.iter().filter(|v| v.current_path).count() != 23
        || parts
            .effects
            .iter()
            .filter(|v| !v.parameters.is_empty())
            .count()
            != 9
        || parts
            .effects
            .iter()
            .any(|v| v.runtime_lowered || v.parameters.iter().any(|p| !canonical_decimal(p)))
    {
        return Err(error("Equation offer/effect boundary drift"));
    }
    if parts.transitions.iter().any(|v| {
        v.candidate_policy.as_ref() != "ExplicitStableIDSelection"
            || v.no_legal_candidate.as_ref() != "RejectWithoutMutation"
            || v.preserved_state.as_ref() != "AllAuthoritativeStateOnRejection"
            || v.ordered_operations.is_empty()
            || v.runtime_lowered
    }) {
        return Err(error("Equation transition boundary drift"));
    }
    Ok(())
}

fn canonical_decimal(value: &str) -> bool {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let mut p = unsigned.split('.');
    let i = p.next().unwrap_or_default();
    let f = p.next();
    p.next().is_none()
        && !i.is_empty()
        && i.bytes().all(|b| b.is_ascii_digit())
        && (i.len() == 1 || !i.starts_with('0'))
        && f.is_none_or(|x| {
            !x.is_empty() && x.bytes().all(|b| b.is_ascii_digit()) && !x.ends_with('0')
        })
}

fn unique<T, K: Ord>(
    values: &[T],
    key: impl Fn(&T) -> &K,
    label: &str,
) -> Result<(), DivergentUniverseEquationError> {
    if values.windows(2).any(|p| key(&p[0]) == key(&p[1])) {
        Err(error(&format!("duplicate {label} identity")))
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEquationError {
    message: Box<str>,
}
impl std::fmt::Display for DivergentUniverseEquationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for DivergentUniverseEquationError {}
fn error(message: &str) -> DivergentUniverseEquationError {
    DivergentUniverseEquationError {
        message: message.into(),
    }
}
