use serde::Deserialize;

use crate::divergent_universe::{DivergentUniverseDataError, debug_error, error};
use crate::divergent_universe_blessing_catalog::*;
use crate::divergent_universe_equation_catalog::{
    DivergentUniverseEquationCatalog, DivergentUniverseEquationError, DivergentUniverseEquationId,
    DivergentUniversePathType,
};
use crate::divergent_universe_generated::SoraConfig;

const COVERAGE_CATEGORIES: [&str; 4] = [
    "blessing_paths",
    "blessings",
    "blessing_levels",
    "blessing_groups",
];

pub(super) fn lower_divergent_universe_blessings(
    config: &SoraConfig,
    equations: &DivergentUniverseEquationCatalog,
) -> Result<DivergentUniverseBlessingCatalog, DivergentUniverseDataError> {
    let parts = DivergentUniverseBlessingCatalogParts {
        paths: config
            .divergent_universe_blessing_paths()
            .ordered_rows()
            .map(|row| {
                let v: PathPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseBlessingPathDefinition {
                    id: path_id(&row.stable_key)?,
                    path: path(v.path_type_id)?,
                    equation_roles: texts(v.equation_roles),
                    rewrite_rules: ids(v.rewrite_rules, rewrite_id)?,
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        blessings: config
            .divergent_universe_blessings()
            .ordered_rows()
            .map(|row| {
                let v: BlessingPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseBlessingDefinition {
                    id: blessing_id(&row.stable_key)?,
                    category: category(&v.category)?,
                    effect_ids: texts(v.effect_ids),
                    handbook_visible: v.handbook_visible,
                    levels: ids(v.level_ids, level_id)?,
                    path_id: path_id(&v.path_id)?,
                    path: path(v.path_type_id)?,
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        levels: config
            .divergent_universe_blessing_levels()
            .ordered_rows()
            .map(|row| {
                let v: LevelPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseBlessingLevelDefinition {
                    id: level_id(&row.stable_key)?,
                    blessing: blessing_id(&v.blessing_id)?,
                    category: category(&v.category)?,
                    level: v.level,
                    state: state(&v.state)?,
                    binding_key: required(v.binding_key, "Blessing binding key")?,
                    binding_type: v.binding_type.into(),
                    modifier_name: required(v.modifier_name, "Blessing modifier name")?,
                    parameters: texts(v.parameters),
                    path: path(v.path_type_id)?,
                    rogue_buff_tag: required(v.rogue_buff_tag, "Blessing buff tag")?,
                    extra_effect_ids: texts(v.extra_effect_ids),
                    equation_contribution_identity: required(
                        v.equation_contribution_identity,
                        "Equation contribution identity",
                    )?,
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        rewrites: config
            .divergent_universe_blessing_rewrite_rules()
            .ordered_rows()
            .map(|row| {
                let v: RewritePayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseBlessingRewriteDefinition {
                    id: rewrite_id(&row.stable_key)?,
                    candidate_policy: v.candidate_policy.into(),
                    input: optional_blessing_id(v.input_blessing_id)?,
                    input_state: v.input_state.into(),
                    output: optional_blessing_id(v.output_blessing_id)?,
                    output_state: v.output_state.into(),
                    timing: v.timing.into(),
                    equation_identity_preserved: v.equation_identity_preserved,
                    no_legal_candidate: v.no_legal_candidate.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        groups: config
            .divergent_universe_blessing_groups()
            .ordered_rows()
            .map(|row| {
                let v: GroupPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseBlessingGroupDefinition {
                    id: group_id(&row.stable_key)?,
                    membership_resolution: v.membership_resolution.into(),
                    source_candidate_ids: texts(v.source_candidate_ids),
                    resolved_levels: ids(v.resolved_mode_level_ids, level_id)?,
                    resolved_subgroups: ids(v.resolved_subgroup_ids, group_id)?,
                    unresolved_source_ids: texts(v.unresolved_source_ids),
                    selection_policy: v.selection_policy.into(),
                    weight_program: v.weight_program.into(),
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        contributions: config
            .divergent_universe_blessing_equation_contributions()
            .ordered_rows()
            .map(|row| {
                let v: ContributionPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseBlessingContributionDefinition {
                    id: contribution_id(&row.stable_key)?,
                    blessing: blessing_id(&v.blessing_id)?,
                    path: path(v.path_type_id)?,
                    contribution: v.contribution,
                    contribution_unit: v.contribution_unit.into(),
                    equations: equation_ids(v.equation_ids)?,
                    base_and_enhanced_count_equally: v.base_and_enhanced_count_equally,
                    refresh_timing: v.refresh_timing.into(),
                    replacement_behavior: v.replacement_behavior.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        source_obligations: config
            .divergent_universe_coverage()
            .ordered_rows()
            .filter(|row| {
                row.manifest_category
                    .as_deref()
                    .is_some_and(|v| COVERAGE_CATEGORIES.contains(&v))
            })
            .count(),
    };
    DivergentUniverseBlessingCatalog::new(parts, equations).map_err(blessing_error)
}

fn category(v: &str) -> Result<DivergentUniverseBlessingCategory, DivergentUniverseDataError> {
    match v {
        "Common" => Ok(DivergentUniverseBlessingCategory::Common),
        "Rare" => Ok(DivergentUniverseBlessingCategory::Rare),
        "Legendary" => Ok(DivergentUniverseBlessingCategory::Legendary),
        _ => Err(error("unknown Blessing category")),
    }
}
fn state(v: &str) -> Result<DivergentUniverseBlessingState, DivergentUniverseDataError> {
    match v {
        "Base" => Ok(DivergentUniverseBlessingState::Base),
        "Enhanced" => Ok(DivergentUniverseBlessingState::Enhanced),
        _ => Err(error("unknown Blessing state")),
    }
}
fn path(v: String) -> Result<DivergentUniversePathType, DivergentUniverseDataError> {
    DivergentUniversePathType::new(v).map_err(equation_error)
}
fn required(v: String, label: &str) -> Result<Box<str>, DivergentUniverseDataError> {
    if v.is_empty() {
        Err(error(&format!("missing {label}")))
    } else {
        Ok(v.into())
    }
}
fn texts(v: Vec<String>) -> Box<[Box<str>]> {
    v.into_iter()
        .map(String::into_boxed_str)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}
fn ids<T>(
    v: Vec<String>,
    parse: fn(&str) -> Result<T, DivergentUniverseDataError>,
) -> Result<Box<[T]>, DivergentUniverseDataError> {
    v.into_iter()
        .map(|x| parse(&x))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}
fn equation_ids(
    v: Vec<String>,
) -> Result<Box<[DivergentUniverseEquationId]>, DivergentUniverseDataError> {
    v.into_iter()
        .map(|x| DivergentUniverseEquationId::new(x).map_err(equation_error))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}
fn optional_blessing_id(
    value: String,
) -> Result<Option<DivergentUniverseBlessingId>, DivergentUniverseDataError> {
    if value.is_empty() {
        Ok(None)
    } else {
        blessing_id(&value).map(Some)
    }
}
fn payload<T: for<'de> Deserialize<'de>>(v: &str) -> Result<T, DivergentUniverseDataError> {
    serde_json::from_str(v).map_err(debug_error)
}
fn blessing_error(v: DivergentUniverseBlessingError) -> DivergentUniverseDataError {
    debug_error(v)
}
fn equation_error(v: DivergentUniverseEquationError) -> DivergentUniverseDataError {
    debug_error(v)
}
macro_rules! id_parser {
    ($f:ident,$t:ty) => {
        fn $f(v: &str) -> Result<$t, DivergentUniverseDataError> {
            <$t>::new(v).map_err(blessing_error)
        }
    };
}
id_parser!(path_id, DivergentUniverseBlessingPathId);
id_parser!(blessing_id, DivergentUniverseBlessingId);
id_parser!(level_id, DivergentUniverseBlessingLevelId);
id_parser!(rewrite_id, DivergentUniverseBlessingRewriteId);
id_parser!(group_id, DivergentUniverseBlessingGroupId);
id_parser!(contribution_id, DivergentUniverseBlessingContributionId);

#[derive(Deserialize)]
struct PathPayload {
    equation_roles: Vec<String>,
    path_type_id: String,
    rewrite_rules: Vec<String>,
}
#[derive(Deserialize)]
struct BlessingPayload {
    category: String,
    effect_ids: Vec<String>,
    handbook_visible: bool,
    level_ids: Vec<String>,
    path_id: String,
    path_type_id: String,
    runtime_lowered: bool,
}
#[derive(Deserialize)]
struct LevelPayload {
    binding_key: String,
    binding_type: String,
    blessing_id: String,
    category: String,
    equation_contribution_identity: String,
    extra_effect_ids: Vec<String>,
    level: u16,
    modifier_name: String,
    parameters: Vec<String>,
    path_type_id: String,
    rogue_buff_tag: String,
    runtime_lowered: bool,
    state: String,
}
#[derive(Deserialize)]
struct RewritePayload {
    candidate_policy: String,
    equation_identity_preserved: bool,
    input_blessing_id: String,
    input_state: String,
    no_legal_candidate: String,
    output_blessing_id: String,
    output_state: String,
    runtime_lowered: bool,
    timing: String,
}
#[derive(Deserialize)]
struct GroupPayload {
    membership_resolution: String,
    resolved_mode_level_ids: Vec<String>,
    resolved_subgroup_ids: Vec<String>,
    selection_policy: String,
    source_candidate_ids: Vec<String>,
    unresolved_source_ids: Vec<String>,
    weight_program: String,
}
#[derive(Deserialize)]
struct ContributionPayload {
    base_and_enhanced_count_equally: bool,
    blessing_id: String,
    contribution: u16,
    contribution_unit: String,
    equation_ids: Vec<String>,
    path_type_id: String,
    refresh_timing: String,
    replacement_behavior: String,
    runtime_lowered: bool,
}
