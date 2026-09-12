use serde::Deserialize;

use crate::divergent_universe::{DivergentUniverseDataError, debug_error};
use crate::divergent_universe_generated::SoraConfig;
use crate::divergent_universe_mechanic_catalog::*;

const COVERAGE: [&str; 2] = ["mechanic_source_files", "semantic_fixture_families"];

pub(super) fn lower_divergent_universe_mechanics(
    config: &SoraConfig,
) -> Result<DivergentUniverseMechanicCatalog, DivergentUniverseDataError> {
    let parts = DivergentUniverseMechanicCatalogParts {
        rules: config
            .divergent_universe_mechanic_rules()
            .ordered_rows()
            .map(|row| {
                let value: RulePayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseMechanicRuleDefinition {
                    id: rule_id(&row.stable_key)?,
                    source: source_id(&value.source_file_id)?,
                    fixture_ids: texts(value.fixture_ids),
                    ordered_operations: value
                        .ordered_operations
                        .into_iter()
                        .map(|operation| DivergentUniverseOrderedOperationShape {
                            operation_type: operation.operation_type.into(),
                            ordinal: operation.ordinal,
                            source_occurrences: operation.source_occurrences,
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    scope: value.scope.into(),
                    state_lifecycle: value.state_lifecycle.into(),
                    trigger: value.trigger.into(),
                    runtime_lowered: value.runtime_lowered,
                })
            })
            .collect::<Result<_, DivergentUniverseDataError>>()?,
        sources: config
            .divergent_universe_mechanic_source_files()
            .ordered_rows()
            .map(|row| {
                let value: SourcePayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseMechanicSourceDefinition {
                    id: source_id(&row.stable_key)?,
                    consumer_rules: ids(value.consumer_rule_ids, rule_id)?,
                    disposition: value.disposition.into(),
                    mechanic_family: value.mechanic_family.into(),
                    operation_occurrence_count: value.operation_occurrence_count,
                    operation_types: value
                        .operation_types
                        .into_iter()
                        .map(|operation| DivergentUniverseOperationShape {
                            operation_type: operation.operation_type.into(),
                            source_occurrences: operation.source_occurrences,
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    scope: value.scope.into(),
                    source_path: value.source_path.into(),
                    source_sha256: value.source_sha256.into(),
                    runtime_lowered: value.runtime_lowered,
                })
            })
            .collect::<Result<_, DivergentUniverseDataError>>()?,
        semantic_families: config
            .divergent_universe_semantic_fixture_families()
            .ordered_rows()
            .map(|row| {
                let value: FamilyPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseSemanticFamilyDefinition {
                    id: family_id(&row.stable_key)?,
                    minimum_cases: value.minimum_cases,
                    must_cover: texts(value.must_cover),
                    selected_source_record_ids: texts(value.selected_source_record_ids),
                    runtime_executable: value.runtime_executable,
                })
            })
            .collect::<Result<_, DivergentUniverseDataError>>()?,
        source_obligations: config
            .divergent_universe_coverage()
            .ordered_rows()
            .filter(|row| {
                row.manifest_category
                    .as_deref()
                    .is_some_and(|value| COVERAGE.contains(&value))
            })
            .count(),
    };
    DivergentUniverseMechanicCatalog::new(parts).map_err(debug_error)
}

fn payload<T: for<'de> Deserialize<'de>>(value: &str) -> Result<T, DivergentUniverseDataError> {
    serde_json::from_str(value).map_err(debug_error)
}
fn texts(values: Vec<String>) -> Box<[Box<str>]> {
    values
        .into_iter()
        .map(String::into_boxed_str)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}
fn ids<T>(
    values: Vec<String>,
    parse: fn(&str) -> Result<T, DivergentUniverseDataError>,
) -> Result<Box<[T]>, DivergentUniverseDataError> {
    values
        .into_iter()
        .map(|value| parse(&value))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}
macro_rules! parser {
    ($function:ident,$kind:ty) => {
        fn $function(value: &str) -> Result<$kind, DivergentUniverseDataError> {
            <$kind>::new(value).map_err(debug_error)
        }
    };
}
parser!(rule_id, DivergentUniverseMechanicRuleId);
parser!(source_id, DivergentUniverseMechanicSourceId);
parser!(family_id, DivergentUniverseSemanticFamilyId);

#[derive(Deserialize)]
struct OrderedOperation {
    operation_type: String,
    ordinal: u16,
    source_occurrences: u32,
}
#[derive(Deserialize)]
struct Operation {
    operation_type: String,
    source_occurrences: u32,
}
#[derive(Deserialize)]
struct RulePayload {
    fixture_ids: Vec<String>,
    ordered_operations: Vec<OrderedOperation>,
    runtime_lowered: bool,
    scope: String,
    source_file_id: String,
    state_lifecycle: String,
    trigger: String,
}
#[derive(Deserialize)]
struct SourcePayload {
    consumer_rule_ids: Vec<String>,
    disposition: String,
    mechanic_family: String,
    operation_occurrence_count: u32,
    operation_types: Vec<Operation>,
    runtime_lowered: bool,
    scope: String,
    source_path: String,
    source_sha256: String,
}
#[derive(Deserialize)]
struct FamilyPayload {
    minimum_cases: u16,
    must_cover: Vec<String>,
    runtime_executable: bool,
    selected_source_record_ids: Vec<String>,
}
