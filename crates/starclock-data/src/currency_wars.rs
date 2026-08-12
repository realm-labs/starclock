use std::collections::BTreeMap;

use serde::Deserialize;
use starclock_combat::EncounterId;
use starclock_mode_currency_wars::{
    CurrencyWarsBond, CurrencyWarsBondId, CurrencyWarsBondLevel, CurrencyWarsCatalog,
    CurrencyWarsCatalogParts, CurrencyWarsDifficulty, CurrencyWarsInvestment,
    CurrencyWarsInvestmentId, CurrencyWarsInvestmentKind, CurrencyWarsNode, CurrencyWarsNodeId,
    CurrencyWarsNodeKind, CurrencyWarsOfferLevel, CurrencyWarsPolicy, CurrencyWarsPositionKind,
    CurrencyWarsPriceRule, CurrencyWarsRole, CurrencyWarsRoleId, CurrencyWarsRoute,
    CurrencyWarsRouteId, CurrencyWarsStarRule, CurrencyWarsTeamLevel,
};

use crate::currency_wars_generated::{SoraConfig, runtime::SoraBundle};

const PRODUCTION_BUNDLE: &[u8] =
    include_bytes!("../../../config/currency-wars-generated/config.sora");

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
    load_currency_wars_catalog_from_bundle(PRODUCTION_BUNDLE)
}

pub fn load_currency_wars_catalog_from_bundle(
    bytes: &[u8],
) -> Result<CurrencyWarsCatalog, CurrencyWarsDataError> {
    let bundle = SoraBundle::parse(bytes).map_err(debug_error)?;
    let config = SoraConfig::from_source(&bundle).map_err(debug_error)?;
    let nodes = lower_nodes(&config).map_err(|error| error.context("nodes"))?;
    let parts = CurrencyWarsCatalogParts {
        routes: lower_routes(nodes).map_err(|error| error.context("routes"))?,
        difficulties: lower_difficulties(&config).map_err(|error| error.context("difficulties"))?,
        roles: lower_roles(&config).map_err(|error| error.context("roles"))?,
        offers: lower_offers(&config).map_err(|error| error.context("offers"))?,
        prices: lower_prices(&config).map_err(|error| error.context("prices"))?,
        team_levels: lower_team_levels(&config).map_err(|error| error.context("team levels"))?,
        star_rules: lower_star_rules(&config).map_err(|error| error.context("star rules"))?,
        bonds: lower_bonds(&config).map_err(|error| error.context("bonds"))?,
        investments: lower_investments(&config).map_err(|error| error.context("investments"))?,
        policies: lower_policies(&config).map_err(|error| error.context("policies"))?,
        initial_squad_hp: parse_required(
            config
                .currency_wars_squad_hp_rules()
                .ordered_rows()
                .next()
                .ok_or_else(|| error("Currency Wars Squad HP row is missing"))?
                .initial_hp
                .as_ref(),
            "Currency Wars initial Squad HP",
        )?,
        refresh_cost: economy(&config)?.refresh.refresh_gold,
        cards_per_refresh: economy(&config)?.refresh.cards_per_refresh,
        direct_experience_cost: economy(&config)?.experience.direct_level_up_gold,
        direct_experience_gain: economy(&config)?.experience.direct_level_up_exp,
        standard_wave_experience: economy(&config)?.experience.standard_wave_gain,
        standard_boss_experience: economy(&config)?.experience.standard_boss_wave_gain,
        overclock_wave_experience: economy(&config)?.experience.overclock_wave_gain,
        overclock_boss_experience: economy(&config)?.experience.overclock_boss_wave_gain,
        front_cap: economy(&config)?.team_size.front_max,
        back_cap: economy(&config)?.team_size.back_max,
    };
    CurrencyWarsCatalog::new(parts).map_err(debug_error)
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

fn lower_nodes(config: &SoraConfig) -> Result<Vec<(u32, CurrencyWarsNode)>, CurrencyWarsDataError> {
    let identities = config
        .currency_wars_nodes()
        .ordered_rows()
        .map(|row| {
            Ok((
                row.stable_key.as_str(),
                CurrencyWarsNodeId::new(u32::try_from(row.id).map_err(debug_error)?)
                    .ok_or_else(|| error("Currency Wars node ID is zero"))?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, CurrencyWarsDataError>>()?;
    config
        .currency_wars_nodes()
        .ordered_rows()
        .map(|row| {
            let route = stable_component(&row.stable_key, "route")?;
            let id = *identities
                .get(row.stable_key.as_str())
                .ok_or_else(|| error("Currency Wars node identity is missing"))?;
            let next = row
                .next_node_id
                .as_deref()
                .filter(|value| !value.is_empty())
                .map(|stable| {
                    identities
                        .get(stable)
                        .copied()
                        .ok_or_else(|| error("Currency Wars next node is missing"))
                })
                .transpose()?;
            Ok((
                route,
                CurrencyWarsNode {
                    id,
                    stable_key: row.stable_key.clone().into_boxed_str(),
                    plane: stable_component(required(&row.plane_id, "node plane")?, "plane")?
                        .try_into()
                        .map_err(debug_error)?,
                    ordinal: parse_required(row.ordinal.as_ref(), "node ordinal")?,
                    kind: node_kind(required(&row.node_type, "node type")?)?,
                    node_template_id: parse_required(
                        row.node_template_id.as_ref(),
                        "node template ID",
                    )?,
                    encounter: EncounterId::new(parse_required(
                        row.stage_id.as_ref(),
                        "node Stage ID",
                    )?)
                    .ok_or_else(|| error("Currency Wars encounter ID is zero"))?,
                    penalty_bonus_rule_id: parse_optional_number(
                        row.penalty_bonus_rule_id.as_ref(),
                    )?,
                    basic_gold_reward: parse_optional_number(row.basic_gold_reward.as_ref())?,
                    next,
                },
            ))
        })
        .collect()
}

fn lower_routes(
    nodes: Vec<(u32, CurrencyWarsNode)>,
) -> Result<Vec<CurrencyWarsRoute>, CurrencyWarsDataError> {
    let mut routes = BTreeMap::<u32, Vec<CurrencyWarsNode>>::new();
    for (route, node) in nodes {
        routes.entry(route).or_default().push(node);
    }
    routes
        .into_iter()
        .map(|(raw, mut nodes)| {
            nodes.sort_by_key(|node| (node.plane, node.ordinal));
            Ok(CurrencyWarsRoute {
                id: CurrencyWarsRouteId::new(raw)
                    .ok_or_else(|| error("Currency Wars route ID is zero"))?,
                stable_key: format!("currency-wars.route.{raw}").into_boxed_str(),
                nodes: nodes.into_boxed_slice(),
            })
        })
        .collect()
}

fn lower_difficulties(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsDifficulty>, CurrencyWarsDataError> {
    config
        .currency_wars_difficulties()
        .ordered_rows()
        .map(|row| {
            let rank = parse_json::<RankBounds>(required(&row.rank_bounds, "difficulty rank")?)?;
            let gambit =
                parse_json::<GambitRules>(required(&row.gambit_rules, "difficulty gambit rules")?)?;
            Ok(CurrencyWarsDifficulty {
                source_id: stable_tail(&row.stable_key)?,
                stable_key: row.stable_key.clone().into_boxed_str(),
                season_id: rank.season_id,
                progress: rank.progress,
                standard_score_rule: gambit.standard_score_rule,
                overclock_score_rule: gambit.overclock_score_rule,
                enemy_scaling_refs: parse_string_array(row.enemy_scaling_refs.as_ref())?,
            })
        })
        .collect()
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
            })
        })
        .collect()
}

fn lower_offers(config: &SoraConfig) -> Result<Vec<CurrencyWarsOfferLevel>, CurrencyWarsDataError> {
    config
        .currency_wars_roster_offers()
        .ordered_rows()
        .map(|row| {
            let weights =
                parse_json::<BTreeMap<u8, String>>(required(&row.weights, "offer weights")?)?;
            Ok(CurrencyWarsOfferLevel {
                level: stable_tail(&row.stable_key)?
                    .try_into()
                    .map_err(debug_error)?,
                candidates: parse_string_vec(required(
                    &row.candidate_avatar_ids,
                    "offer candidates",
                )?)?
                .into_iter()
                .map(|stable| {
                    CurrencyWarsRoleId::new(stable_tail(&stable)?)
                        .ok_or_else(|| error("Currency Wars offered role ID is zero"))
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(),
                rarity_weights: [
                    map_weight(&weights, 1)?,
                    map_weight(&weights, 2)?,
                    map_weight(&weights, 3)?,
                    map_weight(&weights, 4)?,
                    map_weight(&weights, 5)?,
                ],
            })
        })
        .collect()
}

fn lower_prices(config: &SoraConfig) -> Result<Vec<CurrencyWarsPriceRule>, CurrencyWarsDataError> {
    config
        .currency_wars_roster_transactions()
        .ordered_rows()
        .map(|row| {
            let eligibility =
                parse_json::<Eligibility>(required(&row.eligibility, "transaction eligibility")?)?;
            let prices = parse_json::<Prices>(required(&row.price_rule, "transaction price")?)?;
            Ok(CurrencyWarsPriceRule {
                rarity: eligibility.rarity,
                buy_by_star: prices.buy_by_star.into_boxed_slice(),
                sell_by_star: prices.sell_by_star.into_boxed_slice(),
            })
        })
        .collect()
}

fn lower_team_levels(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsTeamLevel>, CurrencyWarsDataError> {
    config
        .currency_wars_team_size_states()
        .ordered_rows()
        .map(|row| {
            Ok(CurrencyWarsTeamLevel {
                level: parse_required(row.level.as_ref(), "team level")?,
                field_cap: parse_required(row.field_cap.as_ref(), "team field cap")?,
                bench_cap: parse_required(row.bench_cap.as_ref(), "team bench cap")?,
                experience_to_next: row
                    .next_level_experience
                    .as_deref()
                    .filter(|value| !value.is_empty())
                    .map(|value| value.parse().map_err(debug_error))
                    .transpose()?,
            })
        })
        .collect()
}

fn lower_star_rules(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsStarRule>, CurrencyWarsDataError> {
    config
        .currency_wars_star_combination_rules()
        .ordered_rows()
        .map(|row| {
            let (role, input_star) = role_star(required(&row.input_state, "star input")?)?;
            let (output_role, output_star) =
                role_star(required(&row.output_state, "star output")?)?;
            if role != output_role {
                return Err(error("Currency Wars star rule changes role identity"));
            }
            Ok(CurrencyWarsStarRule {
                role: CurrencyWarsRoleId::new(role)
                    .ok_or_else(|| error("Currency Wars star-rule role ID is zero"))?,
                input_star,
                required_copies: parse_required(
                    row.required_copies.as_ref(),
                    "star required copies",
                )?,
                output_star,
            })
        })
        .collect()
}

fn lower_bonds(config: &SoraConfig) -> Result<Vec<CurrencyWarsBond>, CurrencyWarsDataError> {
    let mut levels = BTreeMap::<String, Vec<CurrencyWarsBondLevel>>::new();
    for row in config.currency_wars_bond_levels().ordered_rows() {
        levels
            .entry(required(&row.bond_id, "bond-level parent")?.to_owned())
            .or_default()
            .push(CurrencyWarsBondLevel {
                level: parse_required(row.level.as_ref(), "bond level")?,
                threshold: parse_required(row.threshold.as_ref(), "bond threshold")?,
                effect_ids: parse_string_array(row.effect_ids.as_ref())?,
            });
    }
    config
        .currency_wars_bonds()
        .ordered_rows()
        .map(|row| {
            let raw = stable_tail(&row.stable_key)?;
            let mut bond_levels = levels
                .remove(&row.stable_key)
                .ok_or_else(|| error("Currency Wars bond levels are missing"))?;
            bond_levels.sort_by_key(|level| level.threshold);
            Ok(CurrencyWarsBond {
                id: CurrencyWarsBondId::new(raw)
                    .ok_or_else(|| error("Currency Wars bond ID is zero"))?,
                stable_key: row.stable_key.clone().into_boxed_str(),
                members: parse_string_vec(required(&row.member_ids, "bond members")?)?
                    .into_iter()
                    .map(|stable| {
                        CurrencyWarsRoleId::new(stable_tail(&stable)?)
                            .ok_or_else(|| error("Currency Wars bond role ID is zero"))
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
                levels: bond_levels.into_boxed_slice(),
            })
        })
        .collect()
}

fn lower_investments(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsInvestment>, CurrencyWarsDataError> {
    let mut investments = Vec::new();
    macro_rules! extend_family {
        ($rows:expr, $kind:expr, $prefix:expr) => {
            for row in $rows.ordered_rows() {
                let raw = $prefix + u64::try_from(row.id).map_err(debug_error)?;
                investments.push(CurrencyWarsInvestment {
                    id: CurrencyWarsInvestmentId::new(raw)
                        .ok_or_else(|| error("Currency Wars investment ID is zero"))?,
                    stable_key: row.stable_key.clone().into_boxed_str(),
                    kind: $kind,
                    effect_ids: parse_string_array(row.effect_ids.as_ref())?,
                    runtime_binding_exact: false,
                });
            }
        };
    }
    extend_family!(
        config.currency_wars_augment_definitions(),
        CurrencyWarsInvestmentKind::Augment,
        1_000_000
    );
    extend_family!(
        config.currency_wars_enhancements(),
        CurrencyWarsInvestmentKind::Enhancement,
        2_000_000
    );
    extend_family!(
        config.currency_wars_orbs(),
        CurrencyWarsInvestmentKind::Orb,
        3_000_000
    );
    extend_family!(
        config.currency_wars_portal_buffs(),
        CurrencyWarsInvestmentKind::Portal,
        4_000_000
    );
    extend_family!(
        config.currency_wars_projections(),
        CurrencyWarsInvestmentKind::Projection,
        5_000_000
    );
    extend_family!(
        config.currency_wars_talents(),
        CurrencyWarsInvestmentKind::Talent,
        6_000_000
    );
    Ok(investments)
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

#[derive(Clone, Debug, Deserialize)]
struct Economy {
    experience: ExperienceRules,
    refresh: RefreshRules,
    team_size: TeamSizeRules,
}

#[derive(Clone, Debug, Deserialize)]
struct ExperienceRules {
    #[serde(deserialize_with = "string_number")]
    direct_level_up_exp: u32,
    #[serde(deserialize_with = "string_number")]
    direct_level_up_gold: u32,
    #[serde(deserialize_with = "string_array_3")]
    overclock_boss_wave_gain: [u32; 3],
    #[serde(deserialize_with = "string_array_3")]
    overclock_wave_gain: [u32; 3],
    #[serde(deserialize_with = "string_number")]
    standard_boss_wave_gain: u32,
    #[serde(deserialize_with = "string_number")]
    standard_wave_gain: u32,
}

#[derive(Clone, Debug, Deserialize)]
struct RefreshRules {
    #[serde(deserialize_with = "string_number")]
    cards_per_refresh: u8,
    #[serde(deserialize_with = "string_number")]
    refresh_gold: u32,
}

#[derive(Clone, Debug, Deserialize)]
struct TeamSizeRules {
    #[serde(deserialize_with = "string_number")]
    front_max: u8,
    #[serde(deserialize_with = "string_number")]
    back_max: u8,
}

fn economy(config: &SoraConfig) -> Result<Economy, CurrencyWarsDataError> {
    let row = config
        .currency_wars_economy_rules()
        .ordered_rows()
        .next()
        .ok_or_else(|| error("Currency Wars economy row is missing"))?;
    Ok(Economy {
        experience: parse_json(required(&row.experience_rules, "experience rules")?)?,
        refresh: parse_json(required(&row.refresh_rules, "refresh rules")?)?,
        team_size: parse_json(required(&row.team_size_rules, "team-size rules")?)?,
    })
}

#[derive(Deserialize)]
struct RankBounds {
    #[serde(deserialize_with = "string_number")]
    season_id: u16,
    #[serde(deserialize_with = "string_number")]
    progress: u16,
}

#[derive(Deserialize)]
struct GambitRules {
    #[serde(deserialize_with = "string_number")]
    standard_score_rule: u32,
    #[serde(deserialize_with = "string_number")]
    overclock_score_rule: u32,
}

#[derive(Deserialize)]
struct Eligibility {
    #[serde(deserialize_with = "string_number")]
    rarity: u8,
}

#[derive(Deserialize)]
struct Prices {
    #[serde(deserialize_with = "string_vec")]
    buy_by_star: Vec<u32>,
    #[serde(deserialize_with = "string_vec")]
    sell_by_star: Vec<u32>,
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

fn parse_optional_number<T: std::str::FromStr>(
    value: Option<&String>,
) -> Result<Option<T>, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    value
        .map(String::as_str)
        .filter(|value| !value.is_empty() && *value != "undefined")
        .map(|value| value.parse().map_err(debug_error))
        .transpose()
}

fn stable_tail(stable: &str) -> Result<u32, CurrencyWarsDataError> {
    stable
        .rsplit('.')
        .next()
        .ok_or_else(|| error("stable ID has no tail"))?
        .parse()
        .map_err(debug_error)
}

fn stable_component(stable: &str, label: &str) -> Result<u32, CurrencyWarsDataError> {
    let values = stable.split('.').collect::<Vec<_>>();
    values
        .windows(2)
        .find(|pair| pair[0] == label)
        .map(|pair| pair[1])
        .ok_or_else(|| error(&format!("stable ID has no {label} component")))?
        .parse()
        .map_err(debug_error)
}

fn role_star(stable: &str) -> Result<(u32, u8), CurrencyWarsDataError> {
    let values = stable.rsplit('.').take(2).collect::<Vec<_>>();
    if values.len() != 2 {
        return Err(error("Currency Wars role-star stable ID is invalid"));
    }
    Ok((
        values[1].parse().map_err(debug_error)?,
        values[0].parse().map_err(debug_error)?,
    ))
}

fn node_kind(value: &str) -> Result<CurrencyWarsNodeKind, CurrencyWarsDataError> {
    match value {
        "Monster" => Ok(CurrencyWarsNodeKind::Monster),
        "CampMonster" => Ok(CurrencyWarsNodeKind::CampMonster),
        "EliteBranch" => Ok(CurrencyWarsNodeKind::EliteBranch),
        "Boss" => Ok(CurrencyWarsNodeKind::Boss),
        "Supply" => Ok(CurrencyWarsNodeKind::Supply),
        _ => Err(error("Currency Wars node type is unknown")),
    }
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

fn map_weight(weights: &BTreeMap<u8, String>, rarity: u8) -> Result<u32, CurrencyWarsDataError> {
    weights
        .get(&rarity)
        .ok_or_else(|| error("Currency Wars rarity weight is missing"))?
        .parse()
        .map_err(debug_error)
}

fn string_number<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    String::deserialize(deserializer)?
        .parse()
        .map_err(serde::de::Error::custom)
}

fn string_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    Vec::<String>::deserialize(deserializer)?
        .into_iter()
        .map(|value| value.parse().map_err(serde::de::Error::custom))
        .collect()
}

fn string_array_3<'de, D>(deserializer: D) -> Result<[u32; 3], D::Error>
where
    D: serde::Deserializer<'de>,
{
    let values = string_vec(deserializer)?;
    values
        .try_into()
        .map_err(|_| serde::de::Error::custom("expected exactly three values"))
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
    fn context(self, context: &str) -> Self {
        Self {
            message: format!("{context}: {}", self.message).into_boxed_str(),
        }
    }
}

fn error(message: &str) -> CurrencyWarsDataError {
    CurrencyWarsDataError {
        message: message.into(),
    }
}

fn debug_error(value: impl std::fmt::Debug) -> CurrencyWarsDataError {
    CurrencyWarsDataError {
        message: format!("{value:?}").into_boxed_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::{load_currency_wars_catalog, summarize_currency_wars_catalog};

    #[test]
    fn production_bundle_lowers_to_complete_runtime_denominators() {
        let catalog = load_currency_wars_catalog().unwrap();
        let summary = summarize_currency_wars_catalog(&catalog);

        assert_eq!(summary.routes, 26);
        assert_eq!(summary.nodes, 493);
        assert_eq!(summary.roles, 77);
        assert_eq!(summary.bonds, 49);
        assert_eq!(summary.policies, 12);
    }
}
