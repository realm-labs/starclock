use std::collections::BTreeMap;

use serde::Deserialize;
use sha2::{Digest, Sha256};
use starclock_mode_currency_wars::{
    CurrencyWarsCatalog, CurrencyWarsCatalogParts, CurrencyWarsContentReference,
    CurrencyWarsInvestment, CurrencyWarsInvestmentId, CurrencyWarsInvestmentKind,
    CurrencyWarsPolicy, CurrencyWarsPositionKind, CurrencyWarsProgressionCatalog,
    CurrencyWarsReferenceKind, CurrencyWarsRole, CurrencyWarsRoleId,
    CurrencyWarsRoleOverrideCatalog,
};

use crate::currency_wars_blessing_formula::lower_currency_wars_blessing_formula;
use crate::currency_wars_bond::lower_currency_wars_bonds;
use crate::currency_wars_build::lower_currency_wars_build;
use crate::currency_wars_content::lower_currency_wars_content;
use crate::currency_wars_cross_investment::lower_currency_wars_cross_investments;
use crate::currency_wars_economy::lower_currency_wars_economy;
use crate::currency_wars_empowerment::lower_currency_wars_empowerment;
use crate::currency_wars_encounter::lower_currency_wars_encounters;
use crate::currency_wars_flow::lower_currency_wars_flow;
use crate::currency_wars_generated::{
    SCHEMA_FINGERPRINT, SoraConfig,
    currency_wars_augment_definitions::CurrencyWarsAugmentDefinitions,
    currency_wars_enhancements::CurrencyWarsEnhancements,
    currency_wars_orbs::CurrencyWarsOrbs,
    currency_wars_portal_buffs::CurrencyWarsPortalBuffs,
    currency_wars_projections::CurrencyWarsProjections,
    currency_wars_talents::CurrencyWarsTalents,
    runtime::{SoraBundle, SoraTableSource},
};
use crate::currency_wars_investment::lower_currency_wars_augments;
use crate::currency_wars_occurrence::lower_currency_wars_occurrences;
use crate::currency_wars_service::lower_currency_wars_services;

const PRODUCTION_BUNDLE: &[u8] =
    include_bytes!("../../../config/currency-wars-generated/config.sora");
const PRODUCTION_SCHEMA_LOCK: &[u8] =
    include_bytes!("../../../config/currency-wars-generated/schema.lock");
const EXPECTED_TABLES: u32 = 111;
const EXPECTED_ROWS: u32 = 78_607;

macro_rules! digest_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name([u8; 32]);

        impl $name {
            #[must_use]
            pub const fn bytes(self) -> [u8; 32] {
                self.0
            }
        }
    };
}

digest_type!(CurrencyWarsSchemaDigest);
digest_type!(CurrencyWarsConfigurationDigest);
digest_type!(CurrencyWarsContentDigest);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsCatalogIdentity {
    schema_fingerprint: Box<str>,
    schema_digest: CurrencyWarsSchemaDigest,
    configuration_digest: CurrencyWarsConfigurationDigest,
    content_digest: CurrencyWarsContentDigest,
    table_count: u32,
    row_count: u32,
}

impl CurrencyWarsCatalogIdentity {
    #[must_use]
    pub fn schema_fingerprint(&self) -> &str {
        &self.schema_fingerprint
    }

    #[must_use]
    pub const fn schema_digest(&self) -> CurrencyWarsSchemaDigest {
        self.schema_digest
    }

    #[must_use]
    pub const fn configuration_digest(&self) -> CurrencyWarsConfigurationDigest {
        self.configuration_digest
    }

    #[must_use]
    pub const fn content_digest(&self) -> CurrencyWarsContentDigest {
        self.content_digest
    }

    #[must_use]
    pub const fn table_count(&self) -> u32 {
        self.table_count
    }

    #[must_use]
    pub const fn row_count(&self) -> u32 {
        self.row_count
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsCatalogCandidate {
    catalog: CurrencyWarsCatalog,
    identity: CurrencyWarsCatalogIdentity,
}

impl CurrencyWarsCatalogCandidate {
    #[must_use]
    pub const fn catalog(&self) -> &CurrencyWarsCatalog {
        &self.catalog
    }

    #[must_use]
    pub const fn identity(&self) -> &CurrencyWarsCatalogIdentity {
        &self.identity
    }

    #[must_use]
    pub fn into_catalog(self) -> CurrencyWarsCatalog {
        self.catalog
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsCatalogSummary {
    pub routes: usize,
    pub nodes: usize,
    pub difficulties: usize,
    pub roles: usize,
    pub bonds: usize,
    pub investments: usize,
    pub policies: usize,
}

pub fn load_currency_wars_catalog() -> Result<CurrencyWarsCatalog, CurrencyWarsDataError> {
    load_currency_wars_catalog_candidate().map(CurrencyWarsCatalogCandidate::into_catalog)
}

pub fn load_currency_wars_catalog_candidate()
-> Result<CurrencyWarsCatalogCandidate, CurrencyWarsDataError> {
    load_currency_wars_catalog_candidate_from_bundle(PRODUCTION_BUNDLE)
}

pub fn load_currency_wars_catalog_from_bundle(
    bytes: &[u8],
) -> Result<CurrencyWarsCatalog, CurrencyWarsDataError> {
    load_currency_wars_catalog_candidate_from_bundle(bytes)
        .map(CurrencyWarsCatalogCandidate::into_catalog)
}

pub fn load_currency_wars_catalog_candidate_from_bundle(
    bytes: &[u8],
) -> Result<CurrencyWarsCatalogCandidate, CurrencyWarsDataError> {
    let bundle = SoraBundle::parse(bytes).map_err(debug_error)?;
    let config = SoraConfig::from_source(&bundle).map_err(debug_error)?;
    let identity = catalog_identity(bytes, &bundle, &config)?;
    let economy =
        lower_currency_wars_economy(&config).map_err(|error| error.context("economy catalog"))?;
    let front_cap = economy.rules().team_size.front_maximum;
    let back_cap = economy.rules().team_size.back_maximum;
    let encounter = lower_currency_wars_encounters(&config)
        .map_err(|error| error.context("encounter catalog"))?;
    let progression =
        CurrencyWarsProgressionCatalog::from_mechanic_programs(encounter.mechanic_programs())
            .map_err(debug_error)?;
    let role_overrides =
        CurrencyWarsRoleOverrideCatalog::from_mechanic_programs(encounter.mechanic_programs())
            .map_err(debug_error)?;
    let parts = CurrencyWarsCatalogParts {
        flow: lower_currency_wars_flow(&config).map_err(|error| error.context("flow catalog"))?,
        economy,
        build: lower_currency_wars_build(&config)
            .map_err(|error| error.context("Build catalog"))?,
        empowerment: lower_currency_wars_empowerment(&config)
            .map_err(|error| error.context("Empowerment catalog"))?,
        content: lower_currency_wars_content(&config)
            .map_err(|error| error.context("content catalog"))?,
        encounter,
        roles: lower_roles(&config).map_err(|error| error.context("roles"))?,
        bonds: lower_currency_wars_bonds(&config).map_err(|error| error.context("Bond catalog"))?,
        blessing_formula: lower_currency_wars_blessing_formula(&config)
            .map_err(|error| error.context("Blessing/formula catalog"))?,
        occurrences: lower_currency_wars_occurrences(&config)
            .map_err(|error| error.context("Occurrence catalog"))?,
        services: lower_currency_wars_services(&config)
            .map_err(|error| error.context("service catalog"))?,
        augments: lower_currency_wars_augments(&config)
            .map_err(|error| error.context("Augment catalog"))?,
        cross_investments: lower_currency_wars_cross_investments(&config)
            .map_err(|error| error.context("cross-investment catalog"))?,
        progression,
        role_overrides,
        investments: lower_investments(&config).map_err(|error| error.context("investments"))?,
        policies: lower_policies(&config).map_err(|error| error.context("policies"))?,
        front_cap,
        back_cap,
    };
    let catalog = CurrencyWarsCatalog::new(parts).map_err(debug_error)?;
    Ok(CurrencyWarsCatalogCandidate { catalog, identity })
}

#[must_use]
pub fn summarize_currency_wars_catalog(
    catalog: &CurrencyWarsCatalog,
) -> CurrencyWarsCatalogSummary {
    CurrencyWarsCatalogSummary {
        routes: catalog.routes().len(),
        nodes: catalog.routes().iter().map(|route| route.nodes.len()).sum(),
        difficulties: catalog.difficulties().len(),
        roles: catalog.roles().len(),
        bonds: catalog.bonds().len(),
        investments: catalog.investments().len(),
        policies: catalog.policies().len(),
    }
}

fn catalog_identity(
    bytes: &[u8],
    bundle: &SoraBundle<'_>,
    config: &SoraConfig,
) -> Result<CurrencyWarsCatalogIdentity, CurrencyWarsDataError> {
    let fingerprint = bundle.schema_fingerprint().map_err(debug_error)?;
    if fingerprint != SCHEMA_FINGERPRINT {
        return Err(error("Currency Wars schema fingerprint mismatch"));
    }
    let mut tables = config
        .tables()
        .map(|table| {
            Ok((
                table.info().name,
                u32::try_from(table.len()).map_err(debug_error)?,
            ))
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    tables.sort_unstable_by_key(|(name, _)| *name);
    if tables.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(error(
            "Currency Wars generated table identity is duplicated",
        ));
    }
    let table_count = u32::try_from(tables.len()).map_err(debug_error)?;
    let row_count = tables.iter().try_fold(0_u32, |total, (_, rows)| {
        total
            .checked_add(*rows)
            .ok_or_else(|| error("Currency Wars generated row count overflow"))
    })?;
    if table_count != EXPECTED_TABLES || row_count != EXPECTED_ROWS {
        return Err(error(&format!(
            "Currency Wars generated inventory mismatch: expected {EXPECTED_TABLES} tables/{EXPECTED_ROWS} rows, got {table_count} tables/{row_count} rows"
        )));
    }

    let schema_digest = CurrencyWarsSchemaDigest(sha256(PRODUCTION_SCHEMA_LOCK));
    let configuration_digest = CurrencyWarsConfigurationDigest(sha256(bytes));
    let mut content = Sha256::new();
    content.update(b"starclock.currency-wars.mode-content\0");
    content.update(schema_digest.bytes());
    content.update(configuration_digest.bytes());
    hash_text(&mut content, fingerprint)?;
    content.update(table_count.to_be_bytes());
    content.update(row_count.to_be_bytes());
    for (name, rows) in tables {
        hash_text(&mut content, name)?;
        content.update(rows.to_be_bytes());
    }
    Ok(CurrencyWarsCatalogIdentity {
        schema_fingerprint: fingerprint.into(),
        schema_digest,
        configuration_digest,
        content_digest: CurrencyWarsContentDigest(content.finalize().into()),
        table_count,
        row_count,
    })
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn hash_text(hasher: &mut Sha256, value: &str) -> Result<(), CurrencyWarsDataError> {
    let length =
        u32::try_from(value.len()).map_err(|_| error("Currency Wars identity text exceeds u32"))?;
    hasher.update(length.to_be_bytes());
    hasher.update(value.as_bytes());
    Ok(())
}

fn lower_roles(config: &SoraConfig) -> Result<Vec<CurrencyWarsRole>, CurrencyWarsDataError> {
    let maximum_stars = config
        .currency_wars_star_states()
        .ordered_rows()
        .map(|row| {
            Ok((
                parse_required(row.avatar_id.as_ref(), "star-state avatar ID")?,
                parse_required(row.star_level.as_ref(), "star-state level")?,
            ))
        })
        .collect::<Result<Vec<(u32, u8)>, CurrencyWarsDataError>>()?
        .into_iter()
        .fold(BTreeMap::<u32, u8>::new(), |mut values, (role, star)| {
            values
                .entry(role)
                .and_modify(|current| *current = (*current).max(star))
                .or_insert(star);
            values
        });
    config
        .currency_wars_roster_avatars()
        .ordered_rows()
        .map(|row| {
            let role_raw = parse_required(row.role_id.as_ref(), "roster role ID")?;
            let id = CurrencyWarsRoleId::new(role_raw)
                .ok_or_else(|| error("Currency Wars role ID is zero"))?;
            Ok(CurrencyWarsRole {
                id,
                stable_key: row.stable_key.clone().into_boxed_str(),
                avatar_id: parse_required(row.avatar_id.as_ref(), "roster avatar ID")?,
                rarity: parse_required(row.rarity.as_ref(), "roster rarity")?,
                build_mapping_id: required(&row.build_mapping_id, "build mapping ID")?.into(),
                maximum_star: maximum_stars
                    .get(&role_raw)
                    .copied()
                    .ok_or_else(|| error("Currency Wars role has no star state"))?,
                positions: position_kinds(required(&row.position_kind, "role position")?)?,
                trait_ids: parse_string_array(row.trait_ids.as_ref())?
                    .iter()
                    .map(|value| value.parse().map_err(debug_error))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
                backend_rank_ids: parse_string_array(row.backend_rank_ids.as_ref())?
                    .iter()
                    .map(|value| value.parse().map_err(debug_error))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
            })
        })
        .collect()
}

fn lower_investments(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsInvestment>, CurrencyWarsDataError> {
    let mut investments = Vec::new();
    macro_rules! extend_family {
        ($rows:expr, $kind:expr, $prefix:expr, $refs:expr, $attrs:expr) => {
            for row in $rows.ordered_rows() {
                let raw = $prefix + u64::try_from(row.id).map_err(debug_error)?;
                investments.push(CurrencyWarsInvestment {
                    id: CurrencyWarsInvestmentId::new(raw)
                        .ok_or_else(|| error("Currency Wars investment ID is zero"))?,
                    stable_key: row.stable_key.clone().into_boxed_str(),
                    kind: $kind,
                    effect_ids: parse_string_array(row.effect_ids.as_ref())?,
                    source_id: row
                        .stable_key
                        .rsplit('.')
                        .next()
                        .unwrap_or(&row.stable_key)
                        .into(),
                    references: $refs(&row)?.into_boxed_slice(),
                    attributes_json: serde_json::to_string(&$attrs(&row))
                        .map_err(debug_error)?
                        .into_boxed_str(),
                    runtime_binding_exact: matches!(
                        $kind,
                        CurrencyWarsInvestmentKind::Augment
                            | CurrencyWarsInvestmentKind::Enhancement
                            | CurrencyWarsInvestmentKind::Orb
                            | CurrencyWarsInvestmentKind::Portal
                            | CurrencyWarsInvestmentKind::Projection
                            | CurrencyWarsInvestmentKind::Talent
                    ),
                });
            }
        };
    }
    extend_family!(
        config.currency_wars_augment_definitions(),
        CurrencyWarsInvestmentKind::Augment,
        1_000_000,
        |_: &CurrencyWarsAugmentDefinitions| Ok(Vec::new()),
        |row: &CurrencyWarsAugmentDefinitions| (
            row.category_id.clone(),
            row.quality.clone(),
            row.chapter_limits.clone(),
            row.config_path.clone(),
            row.lifecycle.clone()
        )
    );
    extend_family!(
        config.currency_wars_enhancements(),
        CurrencyWarsInvestmentKind::Enhancement,
        2_000_000,
        |_: &CurrencyWarsEnhancements| Ok(Vec::new()),
        |row: &CurrencyWarsEnhancements| (
            row.group_id.clone(),
            row.cost.clone(),
            row.parameters.clone()
        )
    );
    extend_family!(
        config.currency_wars_orbs(),
        CurrencyWarsInvestmentKind::Orb,
        3_000_000,
        |_: &CurrencyWarsOrbs| Ok(Vec::new()),
        |row: &CurrencyWarsOrbs| (row.bonus_id.clone(), row.orb_type.clone())
    );
    extend_family!(
        config.currency_wars_portal_buffs(),
        CurrencyWarsInvestmentKind::Portal,
        4_000_000,
        |_: &CurrencyWarsPortalBuffs| Ok(Vec::new()),
        |row: &CurrencyWarsPortalBuffs| (
            row.config_path.clone(),
            row.bonus_ids.clone(),
            row.lifecycle.clone()
        )
    );
    extend_family!(
        config.currency_wars_projections(),
        CurrencyWarsInvestmentKind::Projection,
        5_000_000,
        |row: &CurrencyWarsProjections| Ok(single_content_reference(
            CurrencyWarsReferenceKind::Role,
            row.role_id.as_ref()
        )),
        |row: &CurrencyWarsProjections| (row.unlock_type.clone(), row.trait_ids.clone())
    );
    extend_family!(
        config.currency_wars_talents(),
        CurrencyWarsInvestmentKind::Talent,
        6_000_000,
        |row: &CurrencyWarsTalents| {
            let mut values = content_references(
                CurrencyWarsReferenceKind::Prerequisite,
                row.prerequisite_ids.as_ref(),
            )?;
            values.extend(content_references(
                CurrencyWarsReferenceKind::Successor,
                row.successor_ids.as_ref(),
            )?);
            Ok(values)
        },
        |row: &CurrencyWarsTalents| (row.cost.clone(), row.config_path.clone())
    );
    Ok(investments)
}

fn content_references(
    kind: CurrencyWarsReferenceKind,
    value: Option<&String>,
) -> Result<Vec<CurrencyWarsContentReference>, CurrencyWarsDataError> {
    let values = value
        .filter(|value| !value.is_empty())
        .map_or_else(|| Ok(Vec::new()), |value| parse_string_vec(value))?;
    Ok(values
        .into_iter()
        .map(|target| CurrencyWarsContentReference {
            kind,
            target: target.into(),
        })
        .collect())
}

fn single_content_reference(
    kind: CurrencyWarsReferenceKind,
    value: Option<&String>,
) -> Vec<CurrencyWarsContentReference> {
    value
        .filter(|value| !value.is_empty())
        .map(|target| {
            vec![CurrencyWarsContentReference {
                kind,
                target: target.clone().into(),
            }]
        })
        .unwrap_or_default()
}

fn lower_policies(config: &SoraConfig) -> Result<Vec<CurrencyWarsPolicy>, CurrencyWarsDataError> {
    config
        .currency_wars_research_gaps()
        .ordered_rows()
        .map(|row| {
            Ok(CurrencyWarsPolicy {
                id: row.stable_key.clone().into_boxed_str(),
                field: required(&row.field, "research-gap field")?.into(),
                known_facts: parse_string_array(row.known_facts.as_ref())?,
                selected_behavior: required(&row.selected_policy, "selected policy")?.into(),
                alternatives: parse_string_array(row.alternatives.as_ref())?,
                replacement_condition: required(
                    &row.replacement_condition,
                    "policy replacement condition",
                )?
                .into(),
            })
        })
        .collect()
}

fn required<'a>(value: &'a Option<String>, name: &str) -> Result<&'a str, CurrencyWarsDataError> {
    value
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| error(&format!("{name} is missing")))
}

fn parse_required<T: std::str::FromStr>(
    value: Option<&String>,
    name: &str,
) -> Result<T, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    value
        .filter(|value| !value.is_empty())
        .ok_or_else(|| error(&format!("{name} is missing")))?
        .parse()
        .map_err(debug_error)
}

fn position_kinds(value: &str) -> Result<Box<[CurrencyWarsPositionKind]>, CurrencyWarsDataError> {
    match value {
        "Front" => Ok(Box::new([CurrencyWarsPositionKind::Front])),
        "Back" => Ok(Box::new([CurrencyWarsPositionKind::Back])),
        "Unspecified" | "Front-Back candidate" => Ok(Box::new([
            CurrencyWarsPositionKind::Front,
            CurrencyWarsPositionKind::Back,
        ])),
        _ => Err(error("Currency Wars role position is unknown")),
    }
}

fn parse_string_array(value: Option<&String>) -> Result<Box<[Box<str>]>, CurrencyWarsDataError> {
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return Ok(Vec::new().into_boxed_slice());
    };
    parse_string_vec(value).map(|values| {
        values
            .into_iter()
            .map(String::into_boxed_str)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    })
}

fn parse_string_vec(value: &str) -> Result<Vec<String>, CurrencyWarsDataError> {
    parse_json(value)
}

fn parse_json<T: for<'de> Deserialize<'de>>(value: &str) -> Result<T, CurrencyWarsDataError> {
    serde_json::from_str(value).map_err(debug_error)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsDataError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsDataError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsDataError {}

impl CurrencyWarsDataError {
    pub(super) fn context(self, context: &str) -> Self {
        Self {
            message: format!("{context}: {}", self.message).into_boxed_str(),
        }
    }
}

pub(super) fn error(message: &str) -> CurrencyWarsDataError {
    CurrencyWarsDataError {
        message: message.into(),
    }
}

pub(super) fn debug_error(value: impl std::fmt::Debug) -> CurrencyWarsDataError {
    CurrencyWarsDataError {
        message: format!("{value:?}").into_boxed_str(),
    }
}

#[cfg(test)]
#[path = "currency_wars_tests.rs"]
mod tests;
