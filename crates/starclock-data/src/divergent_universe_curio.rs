use serde::Deserialize;

use crate::divergent_universe::{DivergentUniverseDataError, debug_error};
use crate::divergent_universe_curio_catalog::*;
use crate::divergent_universe_generated::SoraConfig;

const COVERAGE: [&str; 5] = [
    "curios",
    "curio_states",
    "curio_groups",
    "grand_miracles",
    "grand_miracle_eligibility",
];

pub(super) fn lower_divergent_universe_curios(
    config: &SoraConfig,
) -> Result<DivergentUniverseCurioCatalog, DivergentUniverseDataError> {
    let parts = DivergentUniverseCurioCatalogParts {
        curios: config
            .divergent_universe_curios()
            .ordered_rows()
            .map(|r| {
                let v: CurioPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseCurioDefinition {
                    id: id(&r.stable_key, DivergentUniverseCurioId::new)?,
                    category: category(&v.category)?,
                    states: ids(v.state_ids, DivergentUniverseCurioStateId::new)?,
                    lifecycle: id(&v.lifecycle_rule_id, DivergentUniverseCurioLifecycleId::new)?,
                    eligibility_rule_ids: texts(v.eligibility_rule_ids),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        states: config
            .divergent_universe_curio_states()
            .ordered_rows()
            .map(|r| {
                let v: StatePayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseCurioStateDefinition {
                    id: id(&r.stable_key, DivergentUniverseCurioStateId::new)?,
                    curio: optional_id(v.curio_id, DivergentUniverseCurioId::new)?,
                    category: category(&v.category)?,
                    effect_ids: texts(v.effect_ids),
                    effect_parameters: texts(v.effect_parameters),
                    trigger_kinds: texts(v.trigger_kinds),
                    mechanic_visibility: v.mechanic_visibility.into(),
                    charges: v.charges.into(),
                    activation: v.activation.into(),
                    destruction: v.destruction.into(),
                    repair: v.repair.into(),
                    replacement: v.replacement.into(),
                    counter_parameter_index: v.counter_parameter_index,
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        groups: config
            .divergent_universe_curio_groups()
            .ordered_rows()
            .map(|r| {
                let v: GroupPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseCurioGroupDefinition {
                    id: id(&r.stable_key, DivergentUniverseCurioGroupId::new)?,
                    candidates: ids(v.candidate_state_ids, DivergentUniverseCurioStateId::new)?,
                    consumers: texts(v.consumers),
                    weights: texts(v.weights),
                    draw_count: v.draw_count.into(),
                    eligibility: v.eligibility.into(),
                    membership_resolution: v.membership_resolution.into(),
                    fallback: v.fallback.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        lifecycle: config
            .divergent_universe_curio_lifecycle_rules()
            .ordered_rows()
            .map(|r| {
                let v: LifecyclePayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseCurioLifecycleDefinition {
                    id: id(&r.stable_key, DivergentUniverseCurioLifecycleId::new)?,
                    curio: id(&v.curio_id, DivergentUniverseCurioId::new)?,
                    activation: v.activation.into(),
                    charges: v.charges.into(),
                    destruction: v.destruction.into(),
                    repair: v.repair.into(),
                    replacement: v.replacement.into(),
                    simultaneous_trigger_order: v.simultaneous_trigger_order.into(),
                    fallback: v.fallback.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        pool_membership: config
            .divergent_universe_curio_pool_membership()
            .ordered_rows()
            .map(|r| {
                let v: MembershipPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseCurioPoolMembershipDefinition {
                    id: id(&r.stable_key, DivergentUniverseCurioMembershipId::new)?,
                    state: id(&v.curio_state_id, DivergentUniverseCurioStateId::new)?,
                    pool: id(&v.pool_id, DivergentUniverseCurioPoolId::new)?,
                    source_groups: ids(v.source_group_ids, DivergentUniverseCurioGroupId::new)?,
                    eligibility: v.eligibility.into(),
                    membership_basis: v.membership_basis.into(),
                    weight: v.weight.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        miracles: config
            .divergent_universe_grand_miracles()
            .ordered_rows()
            .map(|r| {
                let v: MiraclePayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseGrandMiracleDefinition {
                    id: id(&r.stable_key, DivergentUniverseGrandMiracleId::new)?,
                    states: ids(v.state_ids, DivergentUniverseGrandMiracleStateId::new)?,
                    eligibility_rules: ids(
                        v.eligibility_rule_ids,
                        DivergentUniverseGrandMiracleEligibilityId::new,
                    )?,
                    maze_buff_id: v.maze_buff_id.into(),
                    maze_buff_resolution: v.maze_buff_resolution.into(),
                    effect_ids: texts(v.effect_ids),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        miracle_eligibility: config
            .divergent_universe_grand_miracle_eligibility()
            .ordered_rows()
            .map(|r| {
                let v: EligibilityPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseGrandMiracleEligibilityDefinition {
                    id: id(
                        &r.stable_key,
                        DivergentUniverseGrandMiracleEligibilityId::new,
                    )?,
                    miracle: optional_id(v.grand_miracle_id, DivergentUniverseGrandMiracleId::new)?,
                    selector_scope: v.selector_scope.into(),
                    character_paths: texts(v.character_path),
                    elements: texts(v.element),
                    eligibility: v.eligibility.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        miracle_states: config
            .divergent_universe_grand_miracle_states()
            .ordered_rows()
            .map(|r| {
                let v: MiracleStatePayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseGrandMiracleStateDefinition {
                    id: id(&r.stable_key, DivergentUniverseGrandMiracleStateId::new)?,
                    miracle: id(&v.grand_miracle_id, DivergentUniverseGrandMiracleId::new)?,
                    state: v.state.into(),
                    activation: v.activation.into(),
                    duration: v.duration.into(),
                    teardown: v.teardown.into(),
                    simultaneous_trigger_order: v.simultaneous_trigger_order.into(),
                    fallback: v.fallback.into(),
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
    DivergentUniverseCurioCatalog::new(parts).map_err(debug_error)
}

fn category(v: &str) -> Result<DivergentUniverseCurioCategory, DivergentUniverseDataError> {
    match v {
        "Common" => Ok(DivergentUniverseCurioCategory::Common),
        "Rare" => Ok(DivergentUniverseCurioCategory::Rare),
        "Legendary" => Ok(DivergentUniverseCurioCategory::Legendary),
        "Negative" => Ok(DivergentUniverseCurioCategory::Negative),
        _ => Err(debug_error("unknown Curio category")),
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
fn id<T, E: std::fmt::Debug>(
    v: &str,
    f: impl FnOnce(Box<str>) -> Result<T, E>,
) -> Result<T, DivergentUniverseDataError> {
    f(v.into()).map_err(debug_error)
}
fn ids<T, E: std::fmt::Debug>(
    v: Vec<String>,
    f: fn(Box<str>) -> Result<T, E>,
) -> Result<Box<[T]>, DivergentUniverseDataError> {
    v.into_iter()
        .map(|x| f(x.into()).map_err(debug_error))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}
fn optional_id<T, E: std::fmt::Debug>(
    v: String,
    f: fn(Box<str>) -> Result<T, E>,
) -> Result<Option<T>, DivergentUniverseDataError> {
    if v.is_empty() {
        Ok(None)
    } else {
        f(v.into()).map(Some).map_err(debug_error)
    }
}

#[derive(Deserialize)]
struct CurioPayload {
    category: String,
    eligibility_rule_ids: Vec<String>,
    lifecycle_rule_id: String,
    runtime_lowered: bool,
    state_ids: Vec<String>,
}
#[derive(Deserialize)]
struct StatePayload {
    activation: String,
    category: String,
    charges: String,
    counter_parameter_index: u16,
    curio_id: String,
    destruction: String,
    effect_ids: Vec<String>,
    effect_parameters: Vec<String>,
    mechanic_visibility: String,
    repair: String,
    replacement: String,
    runtime_lowered: bool,
    trigger_kinds: Vec<String>,
}
#[derive(Deserialize)]
struct GroupPayload {
    candidate_state_ids: Vec<String>,
    consumers: Vec<String>,
    draw_count: String,
    eligibility: String,
    fallback: String,
    membership_resolution: String,
    runtime_lowered: bool,
    weights: Vec<String>,
}
#[derive(Deserialize)]
struct LifecyclePayload {
    activation: String,
    charges: String,
    curio_id: String,
    destruction: String,
    fallback: String,
    repair: String,
    replacement: String,
    runtime_lowered: bool,
    simultaneous_trigger_order: String,
}
#[derive(Deserialize)]
struct MembershipPayload {
    curio_state_id: String,
    eligibility: String,
    membership_basis: String,
    pool_id: String,
    runtime_lowered: bool,
    source_group_ids: Vec<String>,
    weight: String,
}
#[derive(Deserialize)]
struct MiraclePayload {
    effect_ids: Vec<String>,
    eligibility_rule_ids: Vec<String>,
    maze_buff_id: String,
    maze_buff_resolution: String,
    runtime_lowered: bool,
    state_ids: Vec<String>,
}
#[derive(Deserialize)]
struct EligibilityPayload {
    character_path: Vec<String>,
    element: Vec<String>,
    eligibility: String,
    grand_miracle_id: String,
    runtime_lowered: bool,
    selector_scope: String,
}
#[derive(Deserialize)]
struct MiracleStatePayload {
    activation: String,
    duration: String,
    fallback: String,
    grand_miracle_id: String,
    runtime_lowered: bool,
    simultaneous_trigger_order: String,
    state: String,
    teardown: String,
}
