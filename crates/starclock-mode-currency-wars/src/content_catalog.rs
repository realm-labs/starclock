use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsContentKind {
    AugmentMazeBuff,
    AugmentMonsterRule,
    AugmentRemark,
    ModuleBanRule,
    OrbDisplay,
    PortalMazeBuff,
    PortalRemark,
    ProjectionMazeBuff,
    SeasonAugmentMembership,
    SeasonPortalMembership,
    SeasonTalent,
    SelectedEnhancement,
    TalentMazeBuff,
    BlessingPath,
    Blessing,
    BlessingLevel,
    BlessingGroup,
    Formula,
    FormulaDisplay,
    FormulaRandomizer,
    Occurrence,
    OccurrenceVariant,
    OccurrenceChoice,
    Workbench,
    WorkbenchFunction,
    GambleGroup,
    GambleUnit,
    CurseChest,
    AdventureOutcome,
    ShopService,
    ServiceOfferRule,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsReferenceKind {
    Augment,
    Portal,
    Role,
    Trait,
    Prerequisite,
    Successor,
    Blessing,
    Formula,
    OccurrenceVariant,
    OccurrenceChoice,
    Occurrence,
    WorkbenchFunction,
    Currency,
    Service,
    Candidate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsContentReference {
    pub kind: CurrencyWarsReferenceKind,
    pub target: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsContentRecord {
    pub stable_key: Box<str>,
    pub source_id: Option<Box<str>>,
    pub kind: CurrencyWarsContentKind,
    pub references: Box<[CurrencyWarsContentReference]>,
    pub effect_ids: Box<[Box<str>]>,
    /// Canonical JSON tuple whose field order is fixed by `kind`.
    pub attributes_json: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsContentCatalog {
    records: Box<[CurrencyWarsContentRecord]>,
}

impl CurrencyWarsContentCatalog {
    pub fn new(
        mut records: Vec<CurrencyWarsContentRecord>,
    ) -> Result<Self, CurrencyWarsContentCatalogError> {
        records.sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        if records.is_empty()
            || records
                .windows(2)
                .any(|pair| pair[0].stable_key == pair[1].stable_key)
            || records
                .iter()
                .any(|record| record.attributes_json.is_empty())
        {
            return Err(error("Currency Wars content catalog inventory is invalid"));
        }
        let occurrence_variants = keys(&records, CurrencyWarsContentKind::OccurrenceVariant);
        let occurrence_choices = keys(&records, CurrencyWarsContentKind::OccurrenceChoice);
        let occurrences = keys(&records, CurrencyWarsContentKind::Occurrence);
        for record in &records {
            for reference in &record.references {
                let valid = match reference.kind {
                    CurrencyWarsReferenceKind::OccurrenceVariant => {
                        occurrence_variants.contains(reference.target.as_ref())
                    }
                    CurrencyWarsReferenceKind::OccurrenceChoice => {
                        occurrence_choices.contains(reference.target.as_ref())
                    }
                    CurrencyWarsReferenceKind::Occurrence => {
                        occurrences.contains(reference.target.as_ref())
                    }
                    _ => true,
                };
                if !valid {
                    return Err(error(format!(
                        "Currency Wars content reference is unresolved: {} -> {}",
                        record.stable_key, reference.target
                    )));
                }
            }
        }
        Ok(Self {
            records: records.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn records(&self) -> &[CurrencyWarsContentRecord] {
        &self.records
    }

    pub fn records_of_kind(
        &self,
        kind: CurrencyWarsContentKind,
    ) -> impl Iterator<Item = &CurrencyWarsContentRecord> {
        self.records
            .iter()
            .filter(move |record| record.kind == kind)
    }
}

fn keys(records: &[CurrencyWarsContentRecord], kind: CurrencyWarsContentKind) -> BTreeSet<&str> {
    records
        .iter()
        .filter(|record| record.kind == kind)
        .map(|record| record.stable_key.as_ref())
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsContentCatalogError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsContentCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsContentCatalogError {}

fn error(message: impl Into<Box<str>>) -> CurrencyWarsContentCatalogError {
    CurrencyWarsContentCatalogError {
        message: message.into(),
    }
}
