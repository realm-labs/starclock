use crate::divergent_universe::{DivergentUniverseDataError, debug_error};
use crate::divergent_universe_generated::SoraConfig;
use crate::divergent_universe_titan_catalog::*;
use serde::Deserialize;
const COVERAGE: [&str; 3] = ["titan_types", "titan_bless_levels", "titan_talent_levels"];

pub(super) fn lower_divergent_universe_titans(
    config: &SoraConfig,
) -> Result<DivergentUniverseTitanCatalog, DivergentUniverseDataError> {
    let p = DivergentUniverseTitanCatalogParts {
        types: config
            .divergent_universe_titan_types()
            .ordered_rows()
            .map(|r| {
                let v: TypePayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseTitanTypeDefinition {
                    id: type_id(&r.stable_key)?,
                    category: category(&v.category)?,
                    boons: ids(v.boon_ids, boon_id)?,
                    talents: ids(v.talent_ids, talent_id)?,
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        boons: config
            .divergent_universe_titan_boons()
            .ordered_rows()
            .map(|r| {
                let v: BoonPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseTitanBoonDefinition {
                    id: boon_id(&r.stable_key)?,
                    titan_type: type_id(&v.titan_type)?,
                    level: v.level,
                    contribution: contribution_id(&v.contribution_id)?,
                    binding_key: v.binding_key.into(),
                    binding_type: v.binding_type.into(),
                    maze_buff_id: v.maze_buff_id.into(),
                    maze_buff_level: v.maze_buff_level,
                    modifier_name: v.modifier_name.into(),
                    parameters: texts(v.parameters),
                    effect_ids: texts(v.effect_ids),
                    authored_ratio: optional(v.authored_ratio),
                    battle_display_categories: texts(v.battle_display_categories),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        talents: config
            .divergent_universe_titan_talents()
            .ordered_rows()
            .map(|r| {
                let v: TalentPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseTitanTalentDefinition {
                    id: talent_id(&r.stable_key)?,
                    titan_type: type_id(&v.titan_type)?,
                    level: v.level,
                    predecessor: optional_talent(v.predecessor_id)?,
                    contribution: contribution_id(&v.contribution_id)?,
                    cost: v
                        .cost
                        .into_iter()
                        .map(|x| DivergentUniverseItemCost {
                            item_id: x.item_id.into(),
                            amount: x.amount.into(),
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    effect_program: program(v.effect_program),
                    presentation_graph_excluded: v.presentation_graph_excluded,
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        choices: config
            .divergent_universe_titan_choices()
            .ordered_rows()
            .map(|r| {
                let v: ChoicePayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseTitanChoiceDefinition {
                    id: choice_id(&r.stable_key)?,
                    titan_type: type_id(&v.titan_type)?,
                    level: v.level,
                    candidates: ids(v.candidate_ids, boon_id)?,
                    eligibility: v.eligibility.into(),
                    ordering: v.ordering.into(),
                    selection_count: v.selection_count,
                    reroll: v.reroll.into(),
                    fallback: v.fallback.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        contributions: config
            .divergent_universe_titan_contributions()
            .ordered_rows()
            .map(|r| {
                let v: ContributionPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseTitanContributionDefinition {
                    id: contribution_id(&r.stable_key)?,
                    activation: v.activation.into(),
                    ordered_effects: v
                        .ordered_effects
                        .into_iter()
                        .map(effect)
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    scope: v.scope.into(),
                    teardown: v.teardown.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        source_obligations: config
            .divergent_universe_coverage()
            .ordered_rows()
            .filter(|r| {
                r.manifest_category
                    .as_deref()
                    .is_some_and(|x| COVERAGE.contains(&x))
            })
            .count(),
    };
    DivergentUniverseTitanCatalog::new(p).map_err(debug_error)
}
fn category(v: &str) -> Result<DivergentUniverseTitanCategory, DivergentUniverseDataError> {
    match v {
        "Day" => Ok(DivergentUniverseTitanCategory::Day),
        "Night" => Ok(DivergentUniverseTitanCategory::Night),
        _ => Err(debug_error("unknown Titan category")),
    }
}
fn program(v: EffectProgram) -> DivergentUniverseTitanEffectProgram {
    DivergentUniverseTitanEffectProgram {
        condition: v.condition.into(),
        description_hash: v.description_hash.into(),
        metric: v.metric.into(),
        operation: v.operation.into(),
        parameters: texts(v.parameters),
        scope: v.scope.into(),
        value: v.value.map(String::into_boxed_str),
    }
}
fn effect(v: OrderedEffect) -> DivergentUniverseTitanOrderedEffect {
    DivergentUniverseTitanOrderedEffect {
        binding_key: v.binding_key.map(String::into_boxed_str),
        condition: v.condition.map(String::into_boxed_str),
        description_hash: v.description_hash.map(String::into_boxed_str),
        extra_effect_ids: texts(v.extra_effect_ids),
        maze_buff_id: v.maze_buff_id.map(String::into_boxed_str),
        metric: v.metric.map(String::into_boxed_str),
        operation: v.operation.into(),
        parameters: texts(v.parameters),
        scope: v.scope.map(String::into_boxed_str),
        value: v.value.map(String::into_boxed_str),
    }
}
fn optional(v: String) -> Option<Box<str>> {
    if v.is_empty() { None } else { Some(v.into()) }
}
fn optional_talent(
    v: String,
) -> Result<Option<DivergentUniverseTitanTalentId>, DivergentUniverseDataError> {
    if v.is_empty() {
        Ok(None)
    } else {
        talent_id(&v).map(Some)
    }
}
fn payload<T: for<'de> Deserialize<'de>>(v: &str) -> Result<T, DivergentUniverseDataError> {
    serde_json::from_str(v).map_err(debug_error)
}
fn texts(v: Vec<String>) -> Box<[Box<str>]> {
    v.into_iter()
        .map(String::into_boxed_str)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}
fn ids<T>(
    v: Vec<String>,
    f: fn(&str) -> Result<T, DivergentUniverseDataError>,
) -> Result<Box<[T]>, DivergentUniverseDataError> {
    v.into_iter()
        .map(|x| f(&x))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}
macro_rules! parser {
    ($f:ident,$t:ty) => {
        fn $f(v: &str) -> Result<$t, DivergentUniverseDataError> {
            <$t>::new(v).map_err(debug_error)
        }
    };
}
parser!(type_id, DivergentUniverseTitanTypeId);
parser!(boon_id, DivergentUniverseTitanBoonId);
parser!(talent_id, DivergentUniverseTitanTalentId);
parser!(choice_id, DivergentUniverseTitanChoiceId);
parser!(contribution_id, DivergentUniverseTitanContributionId);
#[derive(Deserialize)]
struct TypePayload {
    boon_ids: Vec<String>,
    category: String,
    runtime_lowered: bool,
    talent_ids: Vec<String>,
}
#[derive(Deserialize)]
struct BoonPayload {
    authored_ratio: String,
    battle_display_categories: Vec<String>,
    binding_key: String,
    binding_type: String,
    contribution_id: String,
    effect_ids: Vec<String>,
    level: u16,
    maze_buff_id: String,
    maze_buff_level: u16,
    modifier_name: String,
    parameters: Vec<String>,
    runtime_lowered: bool,
    titan_type: String,
}
#[derive(Deserialize)]
struct Cost {
    amount: String,
    item_id: String,
}
#[derive(Deserialize)]
struct EffectProgram {
    condition: String,
    description_hash: String,
    metric: String,
    operation: String,
    parameters: Vec<String>,
    scope: String,
    value: Option<String>,
}
#[derive(Deserialize)]
struct TalentPayload {
    contribution_id: String,
    cost: Vec<Cost>,
    effect_program: EffectProgram,
    level: u16,
    predecessor_id: String,
    presentation_graph_excluded: bool,
    runtime_lowered: bool,
    titan_type: String,
}
#[derive(Deserialize)]
struct ChoicePayload {
    candidate_ids: Vec<String>,
    eligibility: String,
    fallback: String,
    level: u16,
    ordering: String,
    reroll: String,
    runtime_lowered: bool,
    selection_count: u16,
    titan_type: String,
}
#[derive(Deserialize)]
struct OrderedEffect {
    binding_key: Option<String>,
    condition: Option<String>,
    description_hash: Option<String>,
    #[serde(default)]
    extra_effect_ids: Vec<String>,
    maze_buff_id: Option<String>,
    metric: Option<String>,
    operation: String,
    #[serde(default)]
    parameters: Vec<String>,
    scope: Option<String>,
    value: Option<String>,
}
#[derive(Deserialize)]
struct ContributionPayload {
    activation: String,
    ordered_effects: Vec<OrderedEffect>,
    runtime_lowered: bool,
    scope: String,
    teardown: String,
}
