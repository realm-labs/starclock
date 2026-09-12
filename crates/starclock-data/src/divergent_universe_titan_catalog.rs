//! Immutable Titan, Golden Blood Boon and permanent Titan talent definitions.

use std::collections::{BTreeMap, BTreeSet};

macro_rules! stable_id {
    ($name:ident,$prefix:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(Box<str>);
        impl $name {
            pub fn new(v: impl Into<Box<str>>) -> Result<Self, DivergentUniverseTitanError> {
                let v = v.into();
                if !v.starts_with($prefix) || v.len() == $prefix.len() {
                    Err(error(concat!(stringify!($name), " namespace mismatch")))
                } else {
                    Ok(Self(v))
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
    DivergentUniverseTitanTypeId,
    "divergent-universe.titan-type."
);
stable_id!(
    DivergentUniverseTitanBoonId,
    "divergent-universe.titan-boon."
);
stable_id!(
    DivergentUniverseTitanTalentId,
    "divergent-universe.titan-talent."
);
stable_id!(
    DivergentUniverseTitanChoiceId,
    "divergent-universe.titan-choice."
);
stable_id!(
    DivergentUniverseTitanContributionId,
    "divergent-universe.titan-contribution."
);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseTitanCategory {
    Day,
    Night,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseItemCost {
    pub item_id: Box<str>,
    pub amount: Box<str>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanEffectProgram {
    pub condition: Box<str>,
    pub description_hash: Box<str>,
    pub metric: Box<str>,
    pub operation: Box<str>,
    pub parameters: Box<[Box<str>]>,
    pub scope: Box<str>,
    pub value: Option<Box<str>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanOrderedEffect {
    pub binding_key: Option<Box<str>>,
    pub condition: Option<Box<str>>,
    pub description_hash: Option<Box<str>>,
    pub extra_effect_ids: Box<[Box<str>]>,
    pub maze_buff_id: Option<Box<str>>,
    pub metric: Option<Box<str>>,
    pub operation: Box<str>,
    pub parameters: Box<[Box<str>]>,
    pub scope: Option<Box<str>>,
    pub value: Option<Box<str>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanTypeDefinition {
    pub id: DivergentUniverseTitanTypeId,
    pub category: DivergentUniverseTitanCategory,
    pub boons: Box<[DivergentUniverseTitanBoonId]>,
    pub talents: Box<[DivergentUniverseTitanTalentId]>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanBoonDefinition {
    pub id: DivergentUniverseTitanBoonId,
    pub titan_type: DivergentUniverseTitanTypeId,
    pub level: u16,
    pub contribution: DivergentUniverseTitanContributionId,
    pub binding_key: Box<str>,
    pub binding_type: Box<str>,
    pub maze_buff_id: Box<str>,
    pub maze_buff_level: u16,
    pub modifier_name: Box<str>,
    pub parameters: Box<[Box<str>]>,
    pub effect_ids: Box<[Box<str>]>,
    pub authored_ratio: Option<Box<str>>,
    pub battle_display_categories: Box<[Box<str>]>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanTalentDefinition {
    pub id: DivergentUniverseTitanTalentId,
    pub titan_type: DivergentUniverseTitanTypeId,
    pub level: u16,
    pub predecessor: Option<DivergentUniverseTitanTalentId>,
    pub contribution: DivergentUniverseTitanContributionId,
    pub cost: Box<[DivergentUniverseItemCost]>,
    pub effect_program: DivergentUniverseTitanEffectProgram,
    pub presentation_graph_excluded: bool,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanChoiceDefinition {
    pub id: DivergentUniverseTitanChoiceId,
    pub titan_type: DivergentUniverseTitanTypeId,
    pub level: u16,
    pub candidates: Box<[DivergentUniverseTitanBoonId]>,
    pub eligibility: Box<str>,
    pub ordering: Box<str>,
    pub selection_count: u16,
    pub reroll: Box<str>,
    pub fallback: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanContributionDefinition {
    pub id: DivergentUniverseTitanContributionId,
    pub activation: Box<str>,
    pub ordered_effects: Box<[DivergentUniverseTitanOrderedEffect]>,
    pub scope: Box<str>,
    pub teardown: Box<str>,
    pub runtime_lowered: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanCatalogParts {
    pub types: Vec<DivergentUniverseTitanTypeDefinition>,
    pub boons: Vec<DivergentUniverseTitanBoonDefinition>,
    pub talents: Vec<DivergentUniverseTitanTalentDefinition>,
    pub choices: Vec<DivergentUniverseTitanChoiceDefinition>,
    pub contributions: Vec<DivergentUniverseTitanContributionDefinition>,
    pub source_obligations: usize,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanCatalog {
    parts: DivergentUniverseTitanCatalogParts,
}
impl DivergentUniverseTitanCatalog {
    pub fn new(
        mut p: DivergentUniverseTitanCatalogParts,
    ) -> Result<Self, DivergentUniverseTitanError> {
        if p.types.len() != 12
            || p.boons.len() != 84
            || p.talents.len() != 36
            || p.choices.len() != 36
            || p.contributions.len() != 120
        {
            return Err(error("Titan table denominator drift"));
        }
        if p.source_obligations != 132 {
            return Err(error("Titan source obligation closure drift"));
        }
        p.types.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.boons.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.talents.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.choices.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.contributions.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        validate(&p)?;
        Ok(Self { parts: p })
    }
    #[must_use]
    pub fn types(&self) -> &[DivergentUniverseTitanTypeDefinition] {
        &self.parts.types
    }
    #[must_use]
    pub fn boons(&self) -> &[DivergentUniverseTitanBoonDefinition] {
        &self.parts.boons
    }
    #[must_use]
    pub fn talents(&self) -> &[DivergentUniverseTitanTalentDefinition] {
        &self.parts.talents
    }
    #[must_use]
    pub fn choices(&self) -> &[DivergentUniverseTitanChoiceDefinition] {
        &self.parts.choices
    }
    #[must_use]
    pub fn contributions(&self) -> &[DivergentUniverseTitanContributionDefinition] {
        &self.parts.contributions
    }
    #[must_use]
    pub const fn source_obligations(&self) -> usize {
        self.parts.source_obligations
    }
    #[cfg(test)]
    pub(crate) fn into_parts(self) -> DivergentUniverseTitanCatalogParts {
        self.parts
    }
}
fn validate(p: &DivergentUniverseTitanCatalogParts) -> Result<(), DivergentUniverseTitanError> {
    let types = p
        .types
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let boons = p
        .boons
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let talents = p
        .talents
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let contributions = p
        .contributions
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    if types.len() != 12 || boons.len() != 84 || talents.len() != 36 || contributions.len() != 120 {
        return Err(error("duplicate Titan identity"));
    }
    if p.types.iter().any(|x| {
        x.runtime_lowered
            || x.boons.len() != 7
            || x.talents.len() != 3
            || x.boons
                .iter()
                .any(|id| boons.get(id).is_none_or(|b| b.titan_type != x.id))
            || x.talents
                .iter()
                .any(|id| talents.get(id).is_none_or(|t| t.titan_type != x.id))
    }) {
        return Err(error("Titan type child closure drift"));
    }
    if p.boons.iter().any(|x| {
        x.runtime_lowered
            || !types.contains_key(&x.titan_type)
            || x.binding_type.as_ref() != "StageAbilityBeforeCharacterBorn"
            || x.binding_key.is_empty()
            || x.maze_buff_level != 1
            || !contributions.contains_key(&x.contribution)
    }) {
        return Err(error("Titan Boon binding closure drift"));
    }
    if p.talents.iter().any(|x| {
        x.runtime_lowered
            || !types.contains_key(&x.titan_type)
            || !x.presentation_graph_excluded
            || x.cost.len() != 1
            || x.cost[0].item_id.as_ref() != "281020"
            || x.effect_program.description_hash.is_empty()
            || !contributions.contains_key(&x.contribution)
            || x.predecessor
                .as_ref()
                .is_some_and(|id| !talents.contains_key(id))
    }) {
        return Err(error("Titan talent closure drift"));
    }
    if p.choices.iter().any(|x| {
        x.runtime_lowered
            || !types.contains_key(&x.titan_type)
            || x.selection_count != 1
            || x.ordering.as_ref() != "StableCandidateId"
            || x.reroll.as_ref() != "Unspecified"
            || x.fallback.as_ref() != "RejectWithoutMutation"
            || x.candidates.iter().any(|id| {
                boons
                    .get(id)
                    .is_none_or(|b| b.titan_type != x.titan_type || b.level != x.level)
            })
    }) {
        return Err(error("Titan choice closure drift"));
    }
    let referenced = p
        .boons
        .iter()
        .map(|x| &x.contribution)
        .chain(p.talents.iter().map(|x| &x.contribution))
        .collect::<BTreeSet<_>>();
    if referenced.len() != 120
        || p.contributions.iter().any(|x| {
            x.runtime_lowered || x.ordered_effects.len() != 1 || !referenced.contains(&x.id)
        })
    {
        return Err(error("Titan contribution exact-once closure drift"));
    }
    Ok(())
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanError {
    message: Box<str>,
}
impl std::fmt::Display for DivergentUniverseTitanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for DivergentUniverseTitanError {}
fn error(message: &str) -> DivergentUniverseTitanError {
    DivergentUniverseTitanError {
        message: message.into(),
    }
}
