//! Immutable reference-only mechanic source and semantic-family definitions.

use std::collections::BTreeMap;

macro_rules! stable_id {
    ($name:ident,$prefix:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(Box<str>);
        impl $name {
            pub fn new(value: impl Into<Box<str>>) -> Result<Self, DivergentUniverseMechanicError> {
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
    DivergentUniverseMechanicRuleId,
    "divergent-universe.mechanic-rule."
);
stable_id!(
    DivergentUniverseMechanicSourceId,
    "divergent-universe.mechanic-source."
);
stable_id!(
    DivergentUniverseSemanticFamilyId,
    "divergent-universe.semantic-family."
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOperationShape {
    pub operation_type: Box<str>,
    pub source_occurrences: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseOrderedOperationShape {
    pub operation_type: Box<str>,
    pub ordinal: u16,
    pub source_occurrences: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMechanicRuleDefinition {
    pub id: DivergentUniverseMechanicRuleId,
    pub source: DivergentUniverseMechanicSourceId,
    pub fixture_ids: Box<[Box<str>]>,
    pub ordered_operations: Box<[DivergentUniverseOrderedOperationShape]>,
    pub scope: Box<str>,
    pub state_lifecycle: Box<str>,
    pub trigger: Box<str>,
    pub runtime_lowered: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMechanicSourceDefinition {
    pub id: DivergentUniverseMechanicSourceId,
    pub consumer_rules: Box<[DivergentUniverseMechanicRuleId]>,
    pub disposition: Box<str>,
    pub mechanic_family: Box<str>,
    pub operation_occurrence_count: u32,
    pub operation_types: Box<[DivergentUniverseOperationShape]>,
    pub scope: Box<str>,
    pub source_path: Box<str>,
    pub source_sha256: Box<str>,
    pub runtime_lowered: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseSemanticFamilyDefinition {
    pub id: DivergentUniverseSemanticFamilyId,
    pub minimum_cases: u16,
    pub must_cover: Box<[Box<str>]>,
    pub selected_source_record_ids: Box<[Box<str>]>,
    pub runtime_executable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMechanicCatalogParts {
    pub rules: Vec<DivergentUniverseMechanicRuleDefinition>,
    pub sources: Vec<DivergentUniverseMechanicSourceDefinition>,
    pub semantic_families: Vec<DivergentUniverseSemanticFamilyDefinition>,
    pub source_obligations: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMechanicCatalog {
    parts: DivergentUniverseMechanicCatalogParts,
}

impl DivergentUniverseMechanicCatalog {
    pub fn new(
        mut parts: DivergentUniverseMechanicCatalogParts,
    ) -> Result<Self, DivergentUniverseMechanicError> {
        if parts.rules.len() != 669
            || parts.sources.len() != 669
            || parts.semantic_families.len() != 25
        {
            return Err(error("mechanic identity denominator drift"));
        }
        if parts.source_obligations != 694 {
            return Err(error("mechanic source obligation closure drift"));
        }
        parts
            .rules
            .sort_unstable_by(|left, right| left.id.cmp(&right.id));
        parts
            .sources
            .sort_unstable_by(|left, right| left.id.cmp(&right.id));
        parts
            .semantic_families
            .sort_unstable_by(|left, right| left.id.cmp(&right.id));
        validate(&parts)?;
        Ok(Self { parts })
    }

    #[must_use]
    pub fn rules(&self) -> &[DivergentUniverseMechanicRuleDefinition] {
        &self.parts.rules
    }
    #[must_use]
    pub fn sources(&self) -> &[DivergentUniverseMechanicSourceDefinition] {
        &self.parts.sources
    }
    #[must_use]
    pub fn semantic_families(&self) -> &[DivergentUniverseSemanticFamilyDefinition] {
        &self.parts.semantic_families
    }
    #[must_use]
    pub const fn source_obligations(&self) -> usize {
        self.parts.source_obligations
    }
    #[cfg(test)]
    pub(crate) fn into_parts(self) -> DivergentUniverseMechanicCatalogParts {
        self.parts
    }
}

fn validate(
    parts: &DivergentUniverseMechanicCatalogParts,
) -> Result<(), DivergentUniverseMechanicError> {
    let rules = parts
        .rules
        .iter()
        .map(|value| (&value.id, value))
        .collect::<BTreeMap<_, _>>();
    let sources = parts
        .sources
        .iter()
        .map(|value| (&value.id, value))
        .collect::<BTreeMap<_, _>>();
    let families = parts
        .semantic_families
        .iter()
        .map(|value| (&value.id, value))
        .collect::<BTreeMap<_, _>>();
    if rules.len() != 669 || sources.len() != 669 || families.len() != 25 {
        return Err(error("duplicate mechanic or semantic-family identity"));
    }
    for rule in &parts.rules {
        let source = sources
            .get(&rule.source)
            .ok_or_else(|| error("unknown mechanic source"))?;
        if rule.runtime_lowered
            || rule.fixture_ids.len() != 1
            || rule.ordered_operations.len() != source.operation_types.len()
            || rule
                .ordered_operations
                .iter()
                .enumerate()
                .any(|(index, operation)| {
                    operation.ordinal != u16::try_from(index + 1).unwrap_or_default()
                        || source
                            .operation_types
                            .get(index)
                            .is_none_or(|source_operation| {
                                source_operation.operation_type != operation.operation_type
                                    || source_operation.source_occurrences
                                        != operation.source_occurrences
                            })
                })
        {
            return Err(error("reference mechanic operation-shape closure drift"));
        }
    }
    for source in &parts.sources {
        if source.runtime_lowered
            || source.disposition.as_ref() != "ReferenceOnlyNotLowered"
            || source.consumer_rules.len() != 1
            || rules
                .get(&source.consumer_rules[0])
                .is_none_or(|rule| rule.source != source.id)
            || source.operation_occurrence_count
                != source
                    .operation_types
                    .iter()
                    .map(|value| value.source_occurrences)
                    .sum::<u32>()
            || source.source_path.is_empty()
            || !sha256(&source.source_sha256)
        {
            return Err(error("mechanic source exact-once closure drift"));
        }
    }
    if parts.semantic_families.iter().any(|family| {
        family.runtime_executable
            || family.minimum_cases != 1
            || family.must_cover.is_empty()
            || family.selected_source_record_ids.is_empty()
    }) {
        return Err(error("semantic family reference-only boundary drift"));
    }
    Ok(())
}

fn sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMechanicError {
    message: Box<str>,
}
impl std::fmt::Display for DivergentUniverseMechanicError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl std::error::Error for DivergentUniverseMechanicError {}
fn error(message: &str) -> DivergentUniverseMechanicError {
    DivergentUniverseMechanicError {
        message: message.into(),
    }
}
