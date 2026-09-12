use serde::Deserialize;

use crate::divergent_universe::{DivergentUniverseDataError, debug_error, error};
use crate::divergent_universe_equation_catalog::{
    DivergentUniverseEquationCatalog, DivergentUniverseEquationCatalogParts,
    DivergentUniverseEquationCategory, DivergentUniverseEquationCategoryDefinition,
    DivergentUniverseEquationCategoryId, DivergentUniverseEquationDefinition,
    DivergentUniverseEquationEffectDefinition, DivergentUniverseEquationEffectId,
    DivergentUniverseEquationError, DivergentUniverseEquationId,
    DivergentUniverseEquationOfferDefinition, DivergentUniverseEquationOfferId,
    DivergentUniverseEquationProgressDefinition, DivergentUniverseEquationProgressId,
    DivergentUniverseEquationRecipeDefinition, DivergentUniverseEquationRecipeId,
    DivergentUniverseEquationState, DivergentUniverseEquationStateDefinition,
    DivergentUniverseEquationStateId, DivergentUniverseEquationTransitionDefinition,
    DivergentUniverseEquationTransitionId, DivergentUniversePathType,
};
use crate::divergent_universe_generated::SoraConfig;

const COVERAGE_CATEGORIES: [&str; 5] = [
    "equations",
    "equation_displays",
    "equation_randomizers",
    "equation_keywords",
    "equation_keyword_params",
];

pub(super) fn lower_divergent_universe_equations(
    config: &SoraConfig,
) -> Result<DivergentUniverseEquationCatalog, DivergentUniverseDataError> {
    let parts = DivergentUniverseEquationCatalogParts {
        equations: config
            .divergent_universe_equations()
            .ordered_rows()
            .map(|row| {
                let v: EquationPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseEquationDefinition {
                    id: equation_id(&row.stable_key)?,
                    category: category(&v.category)?,
                    display_id: required(v.display_id, "Equation display ID")?,
                    display_extra_effect_ids: texts(v.display_extra_effect_ids),
                    effect_ids: ids(v.effect_ids, effect_id)?,
                    handbook_visible: v.handbook_visible,
                    main_path: path(v.main_path_type_id)?,
                    sub_path: optional_path(v.sub_path_type_id)?,
                    maze_buff_id: required(v.maze_buff_id, "Equation MazeBuff ID")?,
                    recipe: recipe_id(&v.recipe_id)?,
                    story_payload_included: v.story_payload_included,
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        categories: config
            .divergent_universe_equation_categories()
            .ordered_rows()
            .map(|row| {
                let v: CategoryPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseEquationCategoryDefinition {
                    id: category_id(&row.stable_key)?,
                    category: category(&v.category)?,
                    equations: ids(v.equation_ids, equation_id)?,
                    expansion_boundary: v.expansion_boundary.into(),
                    offer_rule_ids: texts(v.offer_rule_ids),
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        recipes: config
            .divergent_universe_equation_recipes()
            .ordered_rows()
            .map(|row| {
                let v: RecipePayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseEquationRecipeDefinition {
                    id: recipe_id(&row.stable_key)?,
                    equation: equation_id(&v.equation_id)?,
                    main_path: path(v.main_path_type_id)?,
                    main_count: v.main_path_count,
                    sub_path: optional_path(v.sub_path_type_id)?,
                    sub_count: v.sub_path_count,
                    contribution_unit: v.contribution_unit.into(),
                    enhanced_level_contribution: v.enhanced_level_contribution.into(),
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        offers: config
            .divergent_universe_equation_offers()
            .ordered_rows()
            .map(|row| {
                let v: OfferPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseEquationOfferDefinition {
                    id: offer_id(&row.stable_key)?,
                    candidate_ids: ids(v.candidate_ids, equation_id)?,
                    consumer_ids: texts(v.consumer_ids),
                    selection_count: v.selection_count.into(),
                    weight_program: v.weight_program.into(),
                    replacement_allowed: v.replacement_allowed.into(),
                    no_legal_candidate: v.no_legal_candidate.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        progress: config
            .divergent_universe_equation_progress()
            .ordered_rows()
            .map(|row| {
                let v: ProgressPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseEquationProgressDefinition {
                    id: progress_id(&row.stable_key)?,
                    equation: equation_id(&v.equation_id)?,
                    recipe: recipe_id(&v.recipe_id)?,
                    main_required: v.main_required,
                    sub_required: v.sub_required,
                    storage: v.progress_storage.into(),
                    refresh_trigger: v.refresh_trigger.into(),
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        states: config
            .divergent_universe_equation_expansion_states()
            .ordered_rows()
            .map(|row| {
                let v: StatePayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseEquationStateDefinition {
                    id: state_id(&row.stable_key)?,
                    equation: equation_id(&v.equation_id)?,
                    state: state(&v.state)?,
                    entry_condition: v.entry_condition.into(),
                    exit_condition: v.exit_condition.into(),
                    effect_active: v.effect_active,
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        effects: config
            .divergent_universe_equation_effects()
            .ordered_rows()
            .map(|row| {
                let v: EffectPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseEquationEffectDefinition {
                    id: effect_id(&row.stable_key)?,
                    current_path: v.current_path,
                    path_type_id: v.path_type_id.into(),
                    keyword_id: v.keyword_id.into(),
                    maze_buff_id: v.maze_buff_id.into(),
                    maze_buff_ids: texts(v.maze_buff_ids),
                    extra_effect_id: v.extra_effect_id.into(),
                    keyword_extra_effect_id: v.keyword_extra_effect_id.into(),
                    formula_source_ids: texts(v.formula_source_ids),
                    parameters: texts(v.parameters),
                    rule_contribution_ids: texts(v.rule_contribution_ids),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        transitions: config
            .divergent_universe_equation_replacement_rules()
            .ordered_rows()
            .map(|row| {
                let v: TransitionPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseEquationTransitionDefinition {
                    id: transition_id(&row.stable_key)?,
                    operation: v.operation.into(),
                    candidate_policy: v.candidate_policy.into(),
                    ordered_operations: texts(v.ordered_operations),
                    no_legal_candidate: v.no_legal_candidate.into(),
                    preserved_state: v.preserved_state.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        source_obligations: coverage_count(config),
    };
    DivergentUniverseEquationCatalog::new(parts).map_err(equation_error)
}

fn coverage_count(config: &SoraConfig) -> usize {
    config
        .divergent_universe_coverage()
        .ordered_rows()
        .filter(|row| {
            row.manifest_category
                .as_deref()
                .is_some_and(|v| COVERAGE_CATEGORIES.contains(&v))
        })
        .count()
}
fn category(v: &str) -> Result<DivergentUniverseEquationCategory, DivergentUniverseDataError> {
    match v {
        "Rare" => Ok(DivergentUniverseEquationCategory::Rare),
        "Epic" => Ok(DivergentUniverseEquationCategory::Epic),
        "Legendary" => Ok(DivergentUniverseEquationCategory::Legendary),
        "PathEcho" => Ok(DivergentUniverseEquationCategory::PathEcho),
        _ => Err(error("unknown Equation category")),
    }
}
fn state(v: &str) -> Result<DivergentUniverseEquationState, DivergentUniverseDataError> {
    match v {
        "Expanded" => Ok(DivergentUniverseEquationState::Expanded),
        "Unexpanded" => Ok(DivergentUniverseEquationState::Unexpanded),
        _ => Err(error("unknown Equation state")),
    }
}
fn path(v: String) -> Result<DivergentUniversePathType, DivergentUniverseDataError> {
    DivergentUniversePathType::new(v).map_err(equation_error)
}
fn optional_path(
    v: String,
) -> Result<Option<DivergentUniversePathType>, DivergentUniverseDataError> {
    if v.is_empty() {
        Ok(None)
    } else {
        path(v).map(Some)
    }
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
fn payload<T: for<'de> Deserialize<'de>>(v: &str) -> Result<T, DivergentUniverseDataError> {
    serde_json::from_str(v).map_err(debug_error)
}
fn equation_error(v: DivergentUniverseEquationError) -> DivergentUniverseDataError {
    debug_error(v)
}
macro_rules! id_parser {
    ($fn:ident,$ty:ty) => {
        fn $fn(v: &str) -> Result<$ty, DivergentUniverseDataError> {
            <$ty>::new(v).map_err(equation_error)
        }
    };
}
id_parser!(equation_id, DivergentUniverseEquationId);
id_parser!(recipe_id, DivergentUniverseEquationRecipeId);
id_parser!(category_id, DivergentUniverseEquationCategoryId);
id_parser!(offer_id, DivergentUniverseEquationOfferId);
id_parser!(progress_id, DivergentUniverseEquationProgressId);
id_parser!(state_id, DivergentUniverseEquationStateId);
id_parser!(effect_id, DivergentUniverseEquationEffectId);
id_parser!(transition_id, DivergentUniverseEquationTransitionId);

#[derive(Deserialize)]
struct EquationPayload {
    category: String,
    display_extra_effect_ids: Vec<String>,
    display_id: String,
    effect_ids: Vec<String>,
    handbook_visible: bool,
    main_path_type_id: String,
    maze_buff_id: String,
    recipe_id: String,
    runtime_lowered: bool,
    story_payload_included: bool,
    sub_path_type_id: String,
}
#[derive(Deserialize)]
struct CategoryPayload {
    category: String,
    equation_ids: Vec<String>,
    expansion_boundary: String,
    offer_rule_ids: Vec<String>,
}
#[derive(Deserialize)]
struct RecipePayload {
    contribution_unit: String,
    enhanced_level_contribution: String,
    equation_id: String,
    main_path_count: u16,
    main_path_type_id: String,
    sub_path_count: u16,
    sub_path_type_id: String,
}
#[derive(Deserialize)]
struct OfferPayload {
    candidate_ids: Vec<String>,
    consumer_ids: Vec<String>,
    no_legal_candidate: String,
    replacement_allowed: String,
    runtime_lowered: bool,
    selection_count: String,
    weight_program: String,
}
#[derive(Deserialize)]
struct ProgressPayload {
    equation_id: String,
    main_required: u16,
    progress_storage: String,
    recipe_id: String,
    refresh_trigger: String,
    sub_required: u16,
}
#[derive(Deserialize)]
struct StatePayload {
    effect_active: bool,
    entry_condition: String,
    equation_id: String,
    exit_condition: String,
    state: String,
}
#[derive(Deserialize)]
struct EffectPayload {
    current_path: bool,
    extra_effect_id: String,
    formula_source_ids: Vec<String>,
    keyword_extra_effect_id: String,
    keyword_id: String,
    maze_buff_id: String,
    maze_buff_ids: Vec<String>,
    parameters: Vec<String>,
    path_type_id: String,
    rule_contribution_ids: Vec<String>,
    runtime_lowered: bool,
}
#[derive(Deserialize)]
struct TransitionPayload {
    candidate_policy: String,
    no_legal_candidate: String,
    operation: String,
    ordered_operations: Vec<String>,
    preserved_state: String,
    runtime_lowered: bool,
}
