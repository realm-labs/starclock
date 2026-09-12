use crate::divergent_universe::{DivergentUniverseDataError, debug_error};
use crate::divergent_universe_generated::SoraConfig;
use crate::divergent_universe_service_catalog::*;
use serde::Deserialize;
const COVERAGE: [&str; 9] = [
    "workbenches",
    "workbench_functions",
    "gamble_groups",
    "gamble_units",
    "curse_chests",
    "occurrences",
    "occurrence_variants",
    "mode_service_npcs",
    "adventure_outcomes",
];

pub(super) fn lower_divergent_universe_services(
    config: &SoraConfig,
) -> Result<DivergentUniverseServiceCatalog, DivergentUniverseDataError> {
    let p = DivergentUniverseServiceCatalogParts {
        currencies: config
            .divergent_universe_currencies()
            .ordered_rows()
            .map(|r| {
                let v: CurrencyPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseCurrencyDefinition {
                    id: currency_id(&r.stable_key)?,
                    scope: v.scope.into(),
                    gain_rules: texts(v.gain_rules),
                    spend_rules: texts(v.spend_rules),
                    reset_rule: v.reset_rule.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        workbenches: config
            .divergent_universe_workbenches()
            .ordered_rows()
            .map(|r| {
                let v: WorkbenchPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseWorkbenchDefinition {
                    id: workbench_id(&r.stable_key)?,
                    functions: ids(v.function_ids, function_id)?,
                    currencies: ids(v.currency_ids, currency_id)?,
                    availability: v.availability.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        functions: config
            .divergent_universe_workbench_functions()
            .ordered_rows()
            .map(|r| {
                let v: FunctionPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseWorkbenchFunctionDefinition {
                    id: function_id(&r.stable_key)?,
                    function_type: v.function_type.into(),
                    input_policy: v.input_policy.into(),
                    output_policy: v.output_policy.into(),
                    price_rule: DivergentUniversePriceRule {
                        currency: v.price_rule.currency.into(),
                        formula: v.price_rule.formula.into(),
                        reset: v.price_rule.reset.into(),
                    },
                    candidates: texts(v.candidate_ids),
                    weights: texts(v.weights),
                    fallback: v.fallback.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        gamble_groups: config
            .divergent_universe_gamble_groups()
            .ordered_rows()
            .map(|r| {
                let v: GambleGroupPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseGambleGroupDefinition {
                    id: group_id(&r.stable_key)?,
                    group_type: v.group_type.into(),
                    group_level: v.group_level.into(),
                    units: ids(v.unit_ids, unit_id)?,
                    weights: texts(v.weights),
                    draw_count: v.draw_count.into(),
                    offer_policy: v.offer_policy.into(),
                    fallback: v.fallback.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        gamble_units: config
            .divergent_universe_gamble_units()
            .ordered_rows()
            .map(|r| {
                let v: GambleUnitPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseGambleUnitDefinition {
                    id: unit_id(&r.stable_key)?,
                    unit_type: v.unit_type.into(),
                    outcome: DivergentUniverseGambleOutcome {
                        operation: v.outcome_program.operation.into(),
                        category: v.outcome_program.category.map(String::into_boxed_str),
                        source_group_id: v
                            .outcome_program
                            .source_group_id
                            .map(String::into_boxed_str),
                        currency_id: optional_currency(v.outcome_program.currency_id)?,
                        amount: v.outcome_program.amount.map(String::into_boxed_str),
                        resolution: v.outcome_program.resolution.map(String::into_boxed_str),
                    },
                    parameters: texts(v.parameters),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        curse_chests: config
            .divergent_universe_curse_chests()
            .ordered_rows()
            .map(|r| {
                let v: CurseChestPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseCurseChestDefinition {
                    id: chest_id(&r.stable_key)?,
                    chest_type: v.chest_type.into(),
                    choices: v
                        .choice_program
                        .into_iter()
                        .map(|x| DivergentUniverseCurseChoice {
                            operation: x.operation.into(),
                            count: x.count.map(String::into_boxed_str),
                            minimum: x.minimum.map(String::into_boxed_str),
                            maximum: x.maximum.map(String::into_boxed_str),
                            maximum_count: x.maximum_count.map(String::into_boxed_str),
                            pool: x.pool.map(String::into_boxed_str),
                            preferred_path: x.preferred_path.map(String::into_boxed_str),
                            shard: x.shard.map(String::into_boxed_str),
                            weight_policy: x.weights.map(String::into_boxed_str),
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    parameters: texts(v.parameters),
                    fallback: v.fallback.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        occurrences: config
            .divergent_universe_occurrences()
            .ordered_rows()
            .map(|r| {
                let v: OccurrencePayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseOccurrenceDefinition {
                    id: occurrence_id(&r.stable_key)?,
                    variants: ids(v.variant_ids, variant_id)?,
                    choice_ids: texts(v.choice_ids),
                    handbook_priority: v.handbook_priority,
                    handbook_used: v.handbook_used,
                    selection_policy: v.selection_policy.into(),
                    unlock_rules: v
                        .unlock_rules
                        .into_iter()
                        .map(|x| {
                            Ok(DivergentUniverseOccurrenceUnlock {
                                ordinal: x.ordinal,
                                source_field: x.source_field.into(),
                                variant: variant_id(&x.variant_id)?,
                            })
                        })
                        .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?
                        .into_boxed_slice(),
                    unresolved_offer_behavior: v.unresolved_offer_behavior.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        variants: config
            .divergent_universe_occurrence_variants()
            .ordered_rows()
            .map(|r| {
                let v: VariantPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseOccurrenceVariantDefinition {
                    id: variant_id(&r.stable_key)?,
                    occurrence: occurrence_id(&v.occurrence_id)?,
                    occurrences: ids(v.occurrence_ids, occurrence_id)?,
                    entry_conditions: v
                        .entry_conditions
                        .into_iter()
                        .map(|x| {
                            Ok(DivergentUniverseOccurrenceEntryCondition {
                                kind: x.kind.into(),
                                occurrence: occurrence_id(&x.occurrence_id)?,
                            })
                        })
                        .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?
                        .into_boxed_slice(),
                    choice_ids: texts(v.choice_ids),
                    graph_path: v.graph_path.into(),
                    graph_resolution: v.graph_resolution.into(),
                    fallback: v.fallback.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        mode_services: config
            .divergent_universe_mode_service_npcs()
            .ordered_rows()
            .map(|r| {
                let v: ModeServicePayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseModeServiceDefinition {
                    id: mode_service_id(&r.stable_key)?,
                    service_kind: v.service_kind.into(),
                    choice_ids: texts(v.choice_ids),
                    graph_path: v.graph_path.into(),
                    graph_resolution: v.graph_resolution.into(),
                    fallback: v.fallback.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        service_rules: config
            .divergent_universe_service_rules()
            .ordered_rows()
            .map(|r| {
                let v: ServiceRulePayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseServiceRuleDefinition {
                    id: service_rule_id(&r.stable_key)?,
                    service_kind: v.service_kind.into(),
                    currency: optional_currency(Some(v.currency_id))?,
                    price: v.price.into(),
                    ordered_operations: texts(v.ordered_operations),
                    fallback: v.fallback.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        offers: config
            .divergent_universe_service_offer_rules()
            .ordered_rows()
            .map(|r| {
                let v: OfferPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseServiceOfferDefinition {
                    id: offer_id(&r.stable_key)?,
                    service_id: v.service_id.into(),
                    candidates: texts(v.candidate_ids),
                    weights: texts(v.weights),
                    refresh_rule: v.refresh_rule.into(),
                    fallback: v.fallback.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        adventures: config
            .divergent_universe_adventure_outcomes()
            .ordered_rows()
            .map(|r| {
                let v: AdventurePayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseAdventureOutcomeDefinition {
                    id: adventure_id(&r.stable_key)?,
                    adventure_type: v.adventure_type.into(),
                    room_id: v.room_id.into(),
                    parameter_group_id: v.parameter_group_id.into(),
                    parameter_program: v
                        .parameter_program
                        .into_iter()
                        .map(parameter)
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    abstract_outcome: DivergentUniverseAbstractOutcome {
                        input: v.abstract_outcome.input.map(String::into_boxed_str),
                        kind: v.abstract_outcome.kind.into(),
                        ordered_operations: texts(v.abstract_outcome.ordered_operations),
                    },
                    action_gameplay: v.action_gameplay.into(),
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
    DivergentUniverseServiceCatalog::new(p).map_err(debug_error)
}

fn parameter(v: AdventureParameter) -> DivergentUniverseAdventureParameter {
    let fields = [
        ("GameTime", v.game_time),
        ("GameTimeperRound", v.game_time_per_round),
        ("MonsterNum", v.monster_num),
        ("ParamGroupID", v.param_group_id),
        ("PrepareTime", v.prepare_time),
        ("RewardLevel", v.reward_level),
        ("RoundRange", v.round_range),
        ("ScoreRange", v.score_range),
        ("ScoreperRound", v.score_per_round),
        ("ScoreperWave", v.score_per_wave),
        ("TotalRounds", v.total_rounds),
        ("TotalTime", v.total_time),
    ]
    .into_iter()
    .filter_map(|(key, value)| value.map(|x| (key.into(), service_value(x))))
    .collect::<Vec<_>>()
    .into_boxed_slice();
    DivergentUniverseAdventureParameter { fields }
}
fn service_value(value: StringOrArray) -> DivergentUniverseServiceValue {
    match value {
        StringOrArray::Scalar(value) => DivergentUniverseServiceValue::Scalar(value.into()),
        StringOrArray::Array(values) => DivergentUniverseServiceValue::Array(texts(values)),
    }
}
fn optional_currency(
    v: Option<String>,
) -> Result<Option<DivergentUniverseCurrencyId>, DivergentUniverseDataError> {
    match v {
        None => Ok(None),
        Some(x) if x.is_empty() => Ok(None),
        Some(x) => currency_id(&x).map(Some),
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
parser!(currency_id, DivergentUniverseCurrencyId);
parser!(workbench_id, DivergentUniverseWorkbenchId);
parser!(function_id, DivergentUniverseWorkbenchFunctionId);
parser!(group_id, DivergentUniverseGambleGroupId);
parser!(unit_id, DivergentUniverseGambleUnitId);
parser!(chest_id, DivergentUniverseCurseChestId);
parser!(occurrence_id, DivergentUniverseOccurrenceId);
parser!(variant_id, DivergentUniverseOccurrenceVariantId);
parser!(mode_service_id, DivergentUniverseModeServiceId);
parser!(service_rule_id, DivergentUniverseServiceRuleId);
parser!(offer_id, DivergentUniverseServiceOfferId);
parser!(adventure_id, DivergentUniverseAdventureOutcomeId);
#[derive(Deserialize)]
struct CurrencyPayload {
    gain_rules: Vec<String>,
    reset_rule: String,
    runtime_lowered: bool,
    scope: String,
    spend_rules: Vec<String>,
}
#[derive(Deserialize)]
struct WorkbenchPayload {
    availability: String,
    currency_ids: Vec<String>,
    function_ids: Vec<String>,
    runtime_lowered: bool,
}
#[derive(Deserialize)]
struct PriceRule {
    currency: String,
    formula: String,
    reset: String,
}
#[derive(Deserialize)]
struct FunctionPayload {
    candidate_ids: Vec<String>,
    fallback: String,
    function_type: String,
    input_policy: String,
    output_policy: String,
    price_rule: PriceRule,
    runtime_lowered: bool,
    weights: Vec<String>,
}
#[derive(Deserialize)]
struct GambleGroupPayload {
    draw_count: String,
    fallback: String,
    group_level: String,
    group_type: String,
    offer_policy: String,
    runtime_lowered: bool,
    unit_ids: Vec<String>,
    weights: Vec<String>,
}
#[derive(Deserialize)]
struct GambleOutcome {
    operation: String,
    category: Option<String>,
    source_group_id: Option<String>,
    currency_id: Option<String>,
    amount: Option<String>,
    resolution: Option<String>,
}
#[derive(Deserialize)]
struct GambleUnitPayload {
    outcome_program: GambleOutcome,
    parameters: Vec<String>,
    runtime_lowered: bool,
    unit_type: String,
}
#[derive(Deserialize)]
struct CurseChoice {
    operation: String,
    count: Option<String>,
    maximum: Option<String>,
    maximum_count: Option<String>,
    minimum: Option<String>,
    pool: Option<String>,
    preferred_path: Option<String>,
    shard: Option<String>,
    weights: Option<String>,
}
#[derive(Deserialize)]
struct CurseChestPayload {
    chest_type: String,
    choice_program: Vec<CurseChoice>,
    fallback: String,
    parameters: Vec<String>,
    runtime_lowered: bool,
}
#[derive(Deserialize)]
struct UnlockRule {
    ordinal: u16,
    source_field: String,
    variant_id: String,
}
#[derive(Deserialize)]
struct OccurrencePayload {
    choice_ids: Vec<String>,
    handbook_priority: u16,
    handbook_used: bool,
    runtime_lowered: bool,
    selection_policy: String,
    unlock_rules: Vec<UnlockRule>,
    unresolved_offer_behavior: String,
    variant_ids: Vec<String>,
}
#[derive(Deserialize)]
struct EntryCondition {
    kind: String,
    occurrence_id: String,
}
#[derive(Deserialize)]
struct VariantPayload {
    choice_ids: Vec<String>,
    entry_conditions: Vec<EntryCondition>,
    fallback: String,
    graph_path: String,
    graph_resolution: String,
    occurrence_id: String,
    occurrence_ids: Vec<String>,
    runtime_lowered: bool,
}
#[derive(Deserialize)]
struct ModeServicePayload {
    choice_ids: Vec<String>,
    fallback: String,
    graph_path: String,
    graph_resolution: String,
    runtime_lowered: bool,
    service_kind: String,
}
#[derive(Deserialize)]
struct ServiceRulePayload {
    currency_id: String,
    fallback: String,
    ordered_operations: Vec<String>,
    price: String,
    runtime_lowered: bool,
    service_kind: String,
}
#[derive(Deserialize)]
struct OfferPayload {
    candidate_ids: Vec<String>,
    fallback: String,
    refresh_rule: String,
    runtime_lowered: bool,
    service_id: String,
    weights: Vec<String>,
}
#[derive(Deserialize)]
struct AbstractOutcome {
    input: Option<String>,
    kind: String,
    ordered_operations: Vec<String>,
}
#[derive(Deserialize)]
struct AdventurePayload {
    abstract_outcome: AbstractOutcome,
    action_gameplay: String,
    adventure_type: String,
    fallback: String,
    parameter_group_id: String,
    parameter_program: Vec<AdventureParameter>,
    room_id: String,
    runtime_lowered: bool,
}
#[derive(Deserialize)]
struct AdventureParameter {
    #[serde(rename = "GameTime")]
    game_time: Option<StringOrArray>,
    #[serde(rename = "GameTimeperRound")]
    game_time_per_round: Option<StringOrArray>,
    #[serde(rename = "MonsterNum")]
    monster_num: Option<StringOrArray>,
    #[serde(rename = "ParamGroupID")]
    param_group_id: Option<StringOrArray>,
    #[serde(rename = "PrepareTime")]
    prepare_time: Option<StringOrArray>,
    #[serde(rename = "RewardLevel")]
    reward_level: Option<StringOrArray>,
    #[serde(rename = "RoundRange")]
    round_range: Option<StringOrArray>,
    #[serde(rename = "ScoreRange")]
    score_range: Option<StringOrArray>,
    #[serde(rename = "ScoreperRound")]
    score_per_round: Option<StringOrArray>,
    #[serde(rename = "ScoreperWave")]
    score_per_wave: Option<StringOrArray>,
    #[serde(rename = "TotalRounds")]
    total_rounds: Option<StringOrArray>,
    #[serde(rename = "TotalTime")]
    total_time: Option<StringOrArray>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrArray {
    Scalar(String),
    Array(Vec<String>),
}
