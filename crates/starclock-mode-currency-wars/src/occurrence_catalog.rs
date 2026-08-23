use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsOccurrenceKind {
    Pray,
    Present,
    TutorialTask,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOccurrence {
    pub stable_key: Box<str>,
    pub source_id: u32,
    pub kind: CurrencyWarsOccurrenceKind,
    pub unlock_rules_json: Box<str>,
    pub variant_keys: Box<[Box<str>]>,
    pub choice_keys: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsOccurrenceCondition {
    PrayFinish {
        finish_type: Box<str>,
        parameter_type: Box<str>,
        integer_1: Option<u32>,
        string_1: Option<Box<str>>,
        integer_list: Box<[u32]>,
        item_list_json: Box<str>,
        required_progress: u32,
        backtracks: bool,
    },
    TutorialTask {
        task_id: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOccurrenceVariant {
    pub stable_key: Box<str>,
    pub source_id: u32,
    pub occurrence_key: Box<str>,
    pub graph_path: Option<Box<str>>,
    pub condition: CurrencyWarsOccurrenceCondition,
    pub choice_keys: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsOccurrenceOutcomeKind {
    ApplyAcceptBonus,
    ApplyBonus,
    ApplyFinishBonus,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsOccurrenceOutcome {
    pub kind: CurrencyWarsOccurrenceOutcomeKind,
    pub bonus_id: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsOccurrenceCost {
    pub bonus_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOccurrenceChoice {
    pub stable_key: Box<str>,
    pub source_id: u32,
    pub variant_key: Box<str>,
    pub ordinal: u16,
    pub conditions_json: Box<str>,
    pub costs: Box<[CurrencyWarsOccurrenceCost]>,
    pub ordered_outcomes: Box<[CurrencyWarsOccurrenceOutcome]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOccurrenceProgress {
    pub current: u32,
    pub required: u32,
    pub completed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOccurrenceCatalog {
    occurrences: Box<[CurrencyWarsOccurrence]>,
    variants: Box<[CurrencyWarsOccurrenceVariant]>,
    choices: Box<[CurrencyWarsOccurrenceChoice]>,
}

impl CurrencyWarsOccurrenceCatalog {
    pub fn new(
        mut occurrences: Vec<CurrencyWarsOccurrence>,
        mut variants: Vec<CurrencyWarsOccurrenceVariant>,
        mut choices: Vec<CurrencyWarsOccurrenceChoice>,
    ) -> Result<Self, CurrencyWarsOccurrenceCatalogError> {
        occurrences.sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        variants.sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        choices.sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        validate(&occurrences, &variants, &choices)?;
        Ok(Self {
            occurrences: occurrences.into_boxed_slice(),
            variants: variants.into_boxed_slice(),
            choices: choices.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn occurrences(&self) -> &[CurrencyWarsOccurrence] {
        &self.occurrences
    }

    #[must_use]
    pub fn variants(&self) -> &[CurrencyWarsOccurrenceVariant] {
        &self.variants
    }

    #[must_use]
    pub fn choices(&self) -> &[CurrencyWarsOccurrenceChoice] {
        &self.choices
    }

    #[must_use]
    pub fn occurrence(&self, stable_key: &str) -> Option<&CurrencyWarsOccurrence> {
        by_key(&self.occurrences, stable_key, |row| &row.stable_key)
    }

    #[must_use]
    pub fn variant(&self, stable_key: &str) -> Option<&CurrencyWarsOccurrenceVariant> {
        by_key(&self.variants, stable_key, |row| &row.stable_key)
    }

    #[must_use]
    pub fn choice(&self, stable_key: &str) -> Option<&CurrencyWarsOccurrenceChoice> {
        by_key(&self.choices, stable_key, |row| &row.stable_key)
    }

    pub fn resolve_external_progress(
        &self,
        variant_key: &str,
        reported_progress: u32,
    ) -> Result<CurrencyWarsOccurrenceProgress, CurrencyWarsOccurrenceCatalogError> {
        let variant = self
            .variant(variant_key)
            .ok_or_else(|| error("Currency Wars occurrence variant is missing"))?;
        let required = match &variant.condition {
            CurrencyWarsOccurrenceCondition::PrayFinish {
                required_progress, ..
            } => *required_progress,
            CurrencyWarsOccurrenceCondition::TutorialTask { .. } => {
                return Err(error(
                    "Currency Wars tutorial graph result requires its assigned Activity program",
                ));
            }
        };
        let current = reported_progress.min(required);
        Ok(CurrencyWarsOccurrenceProgress {
            current,
            required,
            completed: current == required,
        })
    }

    pub fn ordered_outcomes(
        &self,
        occurrence_key: &str,
        choice_key: &str,
        completed: bool,
    ) -> Result<Box<[CurrencyWarsOccurrenceOutcome]>, CurrencyWarsOccurrenceCatalogError> {
        let occurrence = self
            .occurrence(occurrence_key)
            .ok_or_else(|| error("Currency Wars occurrence is missing"))?;
        if !occurrence
            .choice_keys
            .iter()
            .any(|key| key.as_ref() == choice_key)
        {
            return Err(error(
                "Currency Wars choice does not belong to the occurrence",
            ));
        }
        let choice = self
            .choice(choice_key)
            .ok_or_else(|| error("Currency Wars occurrence choice is missing"))?;
        if !choice.costs.is_empty() && !completed {
            return Err(error(
                "Currency Wars occurrence cost cannot be accepted without its outcome",
            ));
        }
        let outcomes = choice
            .ordered_outcomes
            .iter()
            .copied()
            .filter(|outcome| {
                completed || outcome.kind == CurrencyWarsOccurrenceOutcomeKind::ApplyAcceptBonus
            })
            .collect::<Box<[_]>>();
        if outcomes.is_empty() {
            return Err(error("Currency Wars occurrence has no resolved outcome"));
        }
        Ok(outcomes)
    }
}

#[cfg(test)]
impl CurrencyWarsOccurrenceCatalog {
    pub(crate) fn test_fixture() -> Self {
        let occurrence_key: Box<str> = "occurrence.fixture".into();
        let variant_key: Box<str> = "occurrence-variant.fixture".into();
        let choice_key: Box<str> = "occurrence-choice.fixture".into();
        Self::new(
            vec![CurrencyWarsOccurrence {
                stable_key: occurrence_key.clone(),
                source_id: 1,
                kind: CurrencyWarsOccurrenceKind::Pray,
                unlock_rules_json: "[]".into(),
                variant_keys: Box::new([variant_key.clone()]),
                choice_keys: Box::new([choice_key.clone()]),
            }],
            vec![CurrencyWarsOccurrenceVariant {
                stable_key: variant_key.clone(),
                source_id: 1,
                occurrence_key: occurrence_key.clone(),
                graph_path: None,
                condition: CurrencyWarsOccurrenceCondition::PrayFinish {
                    finish_type: "Fixture".into(),
                    parameter_type: "NoPara".into(),
                    integer_1: None,
                    string_1: None,
                    integer_list: Box::new([]),
                    item_list_json: "[]".into(),
                    required_progress: 1,
                    backtracks: false,
                },
                choice_keys: Box::new([choice_key.clone()]),
            }],
            vec![CurrencyWarsOccurrenceChoice {
                stable_key: choice_key,
                source_id: 1,
                variant_key,
                ordinal: 0,
                conditions_json: "{}".into(),
                costs: Box::new([]),
                ordered_outcomes: Box::new([CurrencyWarsOccurrenceOutcome {
                    kind: CurrencyWarsOccurrenceOutcomeKind::ApplyFinishBonus,
                    bonus_id: 1,
                }]),
            }],
        )
        .expect("test occurrence catalog is valid")
    }
}

fn validate(
    occurrences: &[CurrencyWarsOccurrence],
    variants: &[CurrencyWarsOccurrenceVariant],
    choices: &[CurrencyWarsOccurrenceChoice],
) -> Result<(), CurrencyWarsOccurrenceCatalogError> {
    if occurrences.is_empty() || variants.is_empty() || choices.is_empty() {
        return Err(error("Currency Wars occurrence catalog is empty"));
    }
    let occurrence_keys = unique(occurrences.iter().map(|row| row.stable_key.as_ref()))?;
    let variant_keys = unique(variants.iter().map(|row| row.stable_key.as_ref()))?;
    let choice_keys = unique(choices.iter().map(|row| row.stable_key.as_ref()))?;
    let variants_by_occurrence = variants.iter().fold(
        BTreeMap::<&str, BTreeSet<&str>>::new(),
        |mut values, row| {
            values
                .entry(&row.occurrence_key)
                .or_default()
                .insert(&row.stable_key);
            values
        },
    );
    for occurrence in occurrences {
        if occurrence.variant_keys.iter().any(|key| {
            !variant_keys.contains(key.as_ref())
                || !variants_by_occurrence
                    .get(occurrence.stable_key.as_ref())
                    .is_some_and(|values| values.contains(key.as_ref()))
        }) || occurrence
            .choice_keys
            .iter()
            .any(|key| !choice_keys.contains(key.as_ref()))
        {
            return Err(error("Currency Wars occurrence relationship is unresolved"));
        }
    }
    for variant in variants {
        if !occurrence_keys.contains(variant.occurrence_key.as_ref())
            || variant
                .choice_keys
                .iter()
                .any(|key| !choice_keys.contains(key.as_ref()))
            || matches!(
                variant.condition,
                CurrencyWarsOccurrenceCondition::TutorialTask { .. }
            ) != variant.graph_path.is_some()
        {
            return Err(error("Currency Wars occurrence variant is invalid"));
        }
    }
    for choice in choices {
        if (!variant_keys.contains(choice.variant_key.as_ref())
            && !occurrence_keys.contains(choice.variant_key.as_ref()))
            || choice.ordered_outcomes.is_empty()
        {
            return Err(error("Currency Wars occurrence choice is invalid"));
        }
    }
    Ok(())
}

fn unique<'a>(
    values: impl Iterator<Item = &'a str>,
) -> Result<BTreeSet<&'a str>, CurrencyWarsOccurrenceCatalogError> {
    let mut result = BTreeSet::new();
    if values
        .into_iter()
        .any(|value| value.is_empty() || !result.insert(value))
    {
        return Err(error("Currency Wars occurrence identity is invalid"));
    }
    Ok(result)
}

fn by_key<'a, T>(rows: &'a [T], stable_key: &str, key: impl Fn(&T) -> &Box<str>) -> Option<&'a T> {
    rows.binary_search_by(|row| key(row).as_ref().cmp(stable_key))
        .ok()
        .map(|index| &rows[index])
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOccurrenceCatalogError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsOccurrenceCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsOccurrenceCatalogError {}

fn error(message: &'static str) -> CurrencyWarsOccurrenceCatalogError {
    CurrencyWarsOccurrenceCatalogError {
        message: message.into(),
    }
}
