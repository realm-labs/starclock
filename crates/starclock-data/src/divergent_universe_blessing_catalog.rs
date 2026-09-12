//! Immutable Blessing and Equation-contribution definitions.

use std::collections::{BTreeMap, BTreeSet};

use crate::divergent_universe_equation_catalog::{
    DivergentUniverseEquationCatalog, DivergentUniverseEquationId, DivergentUniversePathType,
};

macro_rules! stable_id {
    ($name:ident,$prefix:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(Box<str>);
        impl $name {
            pub fn new(value: impl Into<Box<str>>) -> Result<Self, DivergentUniverseBlessingError> {
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
stable_id!(
    DivergentUniverseBlessingPathId,
    "divergent-universe.blessing-path."
);
stable_id!(DivergentUniverseBlessingId, "divergent-universe.blessing.");
stable_id!(
    DivergentUniverseBlessingLevelId,
    "divergent-universe.blessing-level."
);
stable_id!(
    DivergentUniverseBlessingRewriteId,
    "divergent-universe.blessing-rewrite."
);
stable_id!(
    DivergentUniverseBlessingGroupId,
    "divergent-universe.blessing-group."
);
stable_id!(
    DivergentUniverseBlessingContributionId,
    "divergent-universe.blessing-contribution."
);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseBlessingCategory {
    Common,
    Rare,
    Legendary,
}
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseBlessingState {
    Base,
    Enhanced,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingPathDefinition {
    pub id: DivergentUniverseBlessingPathId,
    pub path: DivergentUniversePathType,
    pub equation_roles: Box<[Box<str>]>,
    pub rewrite_rules: Box<[DivergentUniverseBlessingRewriteId]>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingDefinition {
    pub id: DivergentUniverseBlessingId,
    pub category: DivergentUniverseBlessingCategory,
    pub effect_ids: Box<[Box<str>]>,
    pub handbook_visible: bool,
    pub levels: Box<[DivergentUniverseBlessingLevelId]>,
    pub path_id: DivergentUniverseBlessingPathId,
    pub path: DivergentUniversePathType,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingLevelDefinition {
    pub id: DivergentUniverseBlessingLevelId,
    pub blessing: DivergentUniverseBlessingId,
    pub category: DivergentUniverseBlessingCategory,
    pub level: u16,
    pub state: DivergentUniverseBlessingState,
    pub binding_key: Box<str>,
    pub binding_type: Box<str>,
    pub modifier_name: Box<str>,
    pub parameters: Box<[Box<str>]>,
    pub path: DivergentUniversePathType,
    pub rogue_buff_tag: Box<str>,
    pub extra_effect_ids: Box<[Box<str>]>,
    pub equation_contribution_identity: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingRewriteDefinition {
    pub id: DivergentUniverseBlessingRewriteId,
    pub candidate_policy: Box<str>,
    pub input: Option<DivergentUniverseBlessingId>,
    pub input_state: Box<str>,
    pub output: Option<DivergentUniverseBlessingId>,
    pub output_state: Box<str>,
    pub timing: Box<str>,
    pub equation_identity_preserved: bool,
    pub no_legal_candidate: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingGroupDefinition {
    pub id: DivergentUniverseBlessingGroupId,
    pub membership_resolution: Box<str>,
    pub source_candidate_ids: Box<[Box<str>]>,
    pub resolved_levels: Box<[DivergentUniverseBlessingLevelId]>,
    pub resolved_subgroups: Box<[DivergentUniverseBlessingGroupId]>,
    pub unresolved_source_ids: Box<[Box<str>]>,
    pub selection_policy: Box<str>,
    pub weight_program: Box<str>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingContributionDefinition {
    pub id: DivergentUniverseBlessingContributionId,
    pub blessing: DivergentUniverseBlessingId,
    pub path: DivergentUniversePathType,
    pub contribution: u16,
    pub contribution_unit: Box<str>,
    pub equations: Box<[DivergentUniverseEquationId]>,
    pub base_and_enhanced_count_equally: bool,
    pub refresh_timing: Box<str>,
    pub replacement_behavior: Box<str>,
    pub runtime_lowered: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingCatalogParts {
    pub paths: Vec<DivergentUniverseBlessingPathDefinition>,
    pub blessings: Vec<DivergentUniverseBlessingDefinition>,
    pub levels: Vec<DivergentUniverseBlessingLevelDefinition>,
    pub rewrites: Vec<DivergentUniverseBlessingRewriteDefinition>,
    pub groups: Vec<DivergentUniverseBlessingGroupDefinition>,
    pub contributions: Vec<DivergentUniverseBlessingContributionDefinition>,
    pub source_obligations: usize,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingCatalog {
    paths: Box<[DivergentUniverseBlessingPathDefinition]>,
    blessings: Box<[DivergentUniverseBlessingDefinition]>,
    levels: Box<[DivergentUniverseBlessingLevelDefinition]>,
    rewrites: Box<[DivergentUniverseBlessingRewriteDefinition]>,
    groups: Box<[DivergentUniverseBlessingGroupDefinition]>,
    contributions: Box<[DivergentUniverseBlessingContributionDefinition]>,
    source_obligations: usize,
}

impl DivergentUniverseBlessingCatalog {
    pub fn new(
        mut p: DivergentUniverseBlessingCatalogParts,
        equations: &DivergentUniverseEquationCatalog,
    ) -> Result<Self, DivergentUniverseBlessingError> {
        for (actual, expected, label) in [
            (p.paths.len(), 8, "Blessing Paths"),
            (p.blessings.len(), 414, "Blessings"),
            (p.levels.len(), 828, "Blessing levels"),
            (p.rewrites.len(), 416, "Blessing rewrites"),
            (p.groups.len(), 118, "Blessing groups"),
            (p.contributions.len(), 414, "Blessing contributions"),
        ] {
            if actual != expected {
                return Err(error(&format!("expected {expected} {label}, got {actual}")));
            }
        }
        if p.source_obligations != 1368 {
            return Err(error("Blessing source obligation closure drift"));
        }
        p.paths.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.blessings.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.levels.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.rewrites.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.groups.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.contributions.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        validate(&p, equations)?;
        Ok(Self {
            paths: p.paths.into_boxed_slice(),
            blessings: p.blessings.into_boxed_slice(),
            levels: p.levels.into_boxed_slice(),
            rewrites: p.rewrites.into_boxed_slice(),
            groups: p.groups.into_boxed_slice(),
            contributions: p.contributions.into_boxed_slice(),
            source_obligations: p.source_obligations,
        })
    }
    #[must_use]
    pub const fn paths(&self) -> &[DivergentUniverseBlessingPathDefinition] {
        &self.paths
    }
    #[must_use]
    pub const fn blessings(&self) -> &[DivergentUniverseBlessingDefinition] {
        &self.blessings
    }
    #[must_use]
    pub const fn levels(&self) -> &[DivergentUniverseBlessingLevelDefinition] {
        &self.levels
    }
    #[must_use]
    pub const fn rewrites(&self) -> &[DivergentUniverseBlessingRewriteDefinition] {
        &self.rewrites
    }
    #[must_use]
    pub const fn groups(&self) -> &[DivergentUniverseBlessingGroupDefinition] {
        &self.groups
    }
    #[must_use]
    pub const fn contributions(&self) -> &[DivergentUniverseBlessingContributionDefinition] {
        &self.contributions
    }
    #[must_use]
    pub const fn source_obligations(&self) -> usize {
        self.source_obligations
    }
    #[cfg(test)]
    pub(crate) fn into_parts(self) -> DivergentUniverseBlessingCatalogParts {
        DivergentUniverseBlessingCatalogParts {
            paths: self.paths.into_vec(),
            blessings: self.blessings.into_vec(),
            levels: self.levels.into_vec(),
            rewrites: self.rewrites.into_vec(),
            groups: self.groups.into_vec(),
            contributions: self.contributions.into_vec(),
            source_obligations: self.source_obligations,
        }
    }
}

fn validate(
    p: &DivergentUniverseBlessingCatalogParts,
    equations: &DivergentUniverseEquationCatalog,
) -> Result<(), DivergentUniverseBlessingError> {
    unique(&p.paths, |v| &v.id, "Blessing Path")?;
    unique(&p.blessings, |v| &v.id, "Blessing")?;
    unique(&p.levels, |v| &v.id, "Blessing level")?;
    unique(&p.rewrites, |v| &v.id, "Blessing rewrite")?;
    unique(&p.groups, |v| &v.id, "Blessing group")?;
    unique(&p.contributions, |v| &v.id, "Blessing contribution")?;
    let paths = p
        .paths
        .iter()
        .map(|v| (&v.id, &v.path))
        .collect::<BTreeMap<_, _>>();
    let blessings = p
        .blessings
        .iter()
        .map(|v| (&v.id, v))
        .collect::<BTreeMap<_, _>>();
    let levels = p
        .levels
        .iter()
        .map(|v| (&v.id, v))
        .collect::<BTreeMap<_, _>>();
    if paths.len() != 8
        || p.paths.iter().any(|v| {
            v.equation_roles.len() != 2
                || v.equation_roles[0].as_ref() != "MainPath"
                || v.equation_roles[1].as_ref() != "SubPath"
                || !v.rewrite_rules.is_empty()
        })
    {
        return Err(error("Blessing Path closure drift"));
    }
    if p.blessings.iter().any(|v| {
        v.runtime_lowered
            || !v.handbook_visible
            || v.levels.len() != 2
            || paths.get(&v.path_id) != Some(&&v.path)
            || v.levels.iter().any(|id| {
                levels.get(id).is_none_or(|l| {
                    l.blessing != v.id || l.category != v.category || l.path != v.path
                })
            })
    }) {
        return Err(error("Blessing/level closure drift"));
    }
    let categories = p.blessings.iter().fold(BTreeMap::new(), |mut m, v| {
        *m.entry(v.category).or_insert(0usize) += 1;
        m
    });
    if categories
        != BTreeMap::from([
            (DivergentUniverseBlessingCategory::Common, 184),
            (DivergentUniverseBlessingCategory::Legendary, 69),
            (DivergentUniverseBlessingCategory::Rare, 161),
        ])
    {
        return Err(error("Blessing category distribution drift"));
    }
    let path_counts = p.blessings.iter().fold(BTreeMap::new(), |mut m, v| {
        *m.entry(v.path.as_str()).or_insert(0usize) += 1;
        m
    });
    if path_counts
        != BTreeMap::from([
            ("121", 54),
            ("122", 54),
            ("124", 54),
            ("125", 54),
            ("126", 54),
            ("127", 54),
            ("128", 54),
            ("129", 36),
        ])
    {
        return Err(error("Blessing Path distribution drift"));
    }
    if p.levels.iter().any(|v| {
        v.binding_key.is_empty()
            || v.binding_type.as_ref() != "StageAbilityBeforeCharacterBorn"
            || v.modifier_name.is_empty()
            || v.parameters.is_empty()
            || v.parameters.iter().any(|x| !canonical_decimal(x))
            || v.equation_contribution_identity.is_empty()
            || v.runtime_lowered
    }) {
        return Err(error("Blessing level binding drift"));
    }
    let group_sizes = p.groups.iter().fold(BTreeMap::new(), |mut m, v| {
        *m.entry(v.source_candidate_ids.len()).or_insert(0usize) += 1;
        m
    });
    if group_sizes != BTreeMap::from([(2, 37), (3, 34), (7, 25), (8, 22)])
        || p.groups.iter().any(|v| {
            v.source_candidate_ids.len()
                != v.resolved_levels.len()
                    + v.resolved_subgroups.len()
                    + v.unresolved_source_ids.len()
                || !v.unresolved_source_ids.is_empty()
                || v.selection_policy.as_ref() != "OrderedSourceCandidates"
                || v.weight_program.as_ref() != "Unspecified"
                || v.resolved_levels.iter().any(|id| !levels.contains_key(id))
        })
        || p.groups
            .iter()
            .map(|v| v.resolved_levels.len())
            .sum::<usize>()
            != 351
        || p.groups
            .iter()
            .map(|v| v.resolved_subgroups.len())
            .sum::<usize>()
            != 176
    {
        return Err(error("Blessing group closure drift"));
    }
    let groups = p.groups.iter().map(|v| &v.id).collect::<BTreeSet<_>>();
    if p.groups
        .iter()
        .flat_map(|v| v.resolved_subgroups.iter())
        .any(|id| !groups.contains(id))
    {
        return Err(error("unknown Blessing subgroup"));
    }
    if p.rewrites.iter().any(|v| {
        v.no_legal_candidate.as_ref() != "RejectWithoutMutation"
            || v.runtime_lowered
            || v.input
                .as_ref()
                .is_some_and(|id| !blessings.contains_key(id))
            || v.output
                .as_ref()
                .is_some_and(|id| !blessings.contains_key(id))
    }) || p
        .rewrites
        .iter()
        .filter(|v| v.timing.as_ref() == "AcceptedEnhanceOperation")
        .count()
        != 414
        || p.rewrites
            .iter()
            .filter(|v| v.timing.as_ref() == "AcceptedServiceOperation")
            .count()
            != 2
        || p.rewrites.iter().any(|v| {
            if v.timing.as_ref() == "AcceptedEnhanceOperation" {
                v.input.is_none() || v.output.is_none()
            } else {
                v.input.is_some() || v.output.is_some()
            }
        })
    {
        return Err(error("Blessing rewrite closure drift"));
    }
    let equation_by_id = equations
        .equations()
        .iter()
        .map(|v| (&v.id, v))
        .collect::<BTreeMap<_, _>>();
    if p.contributions.iter().any(|v| {
        v.contribution != 1
            || !v.base_and_enhanced_count_equally
            || v.runtime_lowered
            || v.contribution_unit.as_ref() != "OwnedBlessingIdentity"
            || v.equations.is_empty()
            || blessings.get(&v.blessing).is_none_or(|b| b.path != v.path)
            || v.equations.iter().any(|id| {
                equation_by_id
                    .get(id)
                    .is_none_or(|e| e.main_path != v.path && e.sub_path.as_ref() != Some(&v.path))
            })
    }) {
        return Err(error("Blessing-to-Equation contribution closure drift"));
    }
    Ok(())
}
fn canonical_decimal(value: &str) -> bool {
    let u = value.strip_prefix('-').unwrap_or(value);
    let mut p = u.split('.');
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
    v: &[T],
    key: impl Fn(&T) -> &K,
    label: &str,
) -> Result<(), DivergentUniverseBlessingError> {
    if v.windows(2).any(|p| key(&p[0]) == key(&p[1])) {
        Err(error(&format!("duplicate {label} identity")))
    } else {
        Ok(())
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBlessingError {
    message: Box<str>,
}
impl std::fmt::Display for DivergentUniverseBlessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for DivergentUniverseBlessingError {}
fn error(message: &str) -> DivergentUniverseBlessingError {
    DivergentUniverseBlessingError {
        message: message.into(),
    }
}
