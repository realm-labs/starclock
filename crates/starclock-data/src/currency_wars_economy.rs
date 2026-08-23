use std::collections::BTreeMap;

use serde::Deserialize;
use starclock_mode_currency_wars::{
    CurrencyWarsActionValueDecrement, CurrencyWarsActionValueInitial, CurrencyWarsActionValueLimit,
    CurrencyWarsActionValueLimitKind, CurrencyWarsActionValueProjection,
    CurrencyWarsAuthoredProperty, CurrencyWarsBattleOutcomeProjection,
    CurrencyWarsBattleResultProjection, CurrencyWarsContributionParameter,
    CurrencyWarsContributionParameterKind, CurrencyWarsCurrency, CurrencyWarsCurrencyGain,
    CurrencyWarsCurrencyReset, CurrencyWarsCurrencySpend, CurrencyWarsDecimal,
    CurrencyWarsEconomyCatalog, CurrencyWarsEconomyCatalogParts, CurrencyWarsEconomyRules,
    CurrencyWarsExperienceRules, CurrencyWarsInfluenceProperty, CurrencyWarsInfluenceSubject,
    CurrencyWarsInterestRules, CurrencyWarsOfferCostRule, CurrencyWarsOfferFallback,
    CurrencyWarsOfferLevel, CurrencyWarsPositionDefinition, CurrencyWarsPositionDefinitionKind,
    CurrencyWarsPositionEligibility, CurrencyWarsPriceRule, CurrencyWarsRankAttachment,
    CurrencyWarsRefreshRules, CurrencyWarsRoleId, CurrencyWarsRunDisposition,
    CurrencyWarsSquadHpLossRule, CurrencyWarsSquadHpMaximum, CurrencyWarsSquadHpProjection,
    CurrencyWarsSquadHpRecoveryRule, CurrencyWarsSquadHpRules, CurrencyWarsStarLifecycleOperation,
    CurrencyWarsStarLifecycleRule, CurrencyWarsStarOverflowRule, CurrencyWarsStarRule,
    CurrencyWarsStarState, CurrencyWarsStarStateOwner, CurrencyWarsTeamLevel,
    CurrencyWarsTeamLevelTransition, CurrencyWarsTeamSizeRules, CurrencyWarsTimeoutBoundary,
    CurrencyWarsTransactionChange,
};

use crate::{
    currency_wars::{CurrencyWarsDataError, debug_error, error},
    currency_wars_build::canonical_json,
    currency_wars_flow::{parse_boxed_strings, parse_json, required},
    currency_wars_generated::SoraConfig,
};

#[derive(Deserialize)]
struct AuthoredPropertyRow {
    property_type: String,
    value: Option<String>,
}

#[derive(Deserialize)]
struct StarPropertyRow {
    #[serde(rename = "PropertyType")]
    property_type: String,
    #[serde(rename = "Value")]
    value: Option<String>,
}

#[derive(Deserialize)]
struct RankAttachmentRow {
    rank: u8,
    property_modifiers: Vec<StarPropertyRow>,
}

#[derive(Deserialize)]
struct InfluencePropertyRow {
    property_type: String,
    value: String,
}

pub(super) fn lower_currency_wars_economy(
    config: &SoraConfig,
) -> Result<CurrencyWarsEconomyCatalog, CurrencyWarsDataError> {
    CurrencyWarsEconomyCatalog::new(CurrencyWarsEconomyCatalogParts {
        currencies: lower_currencies(config)?,
        rules: lower_rules(config)?,
        offers: lower_offers(config)?,
        prices: lower_prices(config)?,
        team_levels: lower_team_levels(config)?,
        positions: lower_positions(config)?,
        star_states: lower_star_states(config)?,
        influence_properties: lower_influence_properties(config)?,
        contribution_parameters: lower_contribution_parameters(config)?,
        star_rules: lower_star_rules(config)?,
        star_lifecycle: lower_star_lifecycle(config)?,
        squad_hp: lower_squad_hp(config)?,
        action_value_limits: lower_action_value_limits(config)?,
        battle_result_projections: lower_battle_result_projections(config)?,
    })
    .map_err(debug_error)
}

fn lower_currencies(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsCurrency>, CurrencyWarsDataError> {
    config
        .currency_wars_currencies()
        .ordered_rows()
        .map(|row| {
            if required(&row.scope, "currency scope")? != "CurrencyWarsRun"
                || required(&row.reset_rule, "currency reset")? != "Discard at run teardown"
            {
                return Err(error("Currency Wars currency scope/reset is unknown"));
            }
            let gains = parse_strings(row.gain_rules.as_ref())?;
            let spends = parse_strings(row.spend_rules.as_ref())?;
            if gains.as_slice() != ["Authored battle, event, interest and service outcomes"]
                || spends.as_slice()
                    != ["Recruitment, Store refresh and explicitly priced service operations"]
            {
                return Err(error("Currency Wars currency rule is unknown"));
            }
            Ok(CurrencyWarsCurrency {
                stable_key: row.stable_key.clone().into(),
                gains: Box::new([
                    CurrencyWarsCurrencyGain::AuthoredBattleEventInterestAndServiceOutcomes,
                ]),
                spends: Box::new([CurrencyWarsCurrencySpend::RecruitmentRefreshAndPricedServices]),
                reset: CurrencyWarsCurrencyReset::DiscardAtRunTeardown,
            })
        })
        .collect()
}

fn lower_rules(config: &SoraConfig) -> Result<CurrencyWarsEconomyRules, CurrencyWarsDataError> {
    let mut rows = config.currency_wars_economy_rules().ordered_rows();
    let row = rows
        .next()
        .ok_or_else(|| error("Currency Wars economy rule is missing"))?;
    if rows.next().is_some() {
        return Err(error("Currency Wars economy rule is duplicated"));
    }
    let experience: Experience = parse_json(required(&row.experience_rules, "experience rules")?)?;
    let refresh: Refresh = parse_json(required(&row.refresh_rules, "refresh rules")?)?;
    let interest: Interest = parse_json(required(&row.interest_rules, "interest rules")?)?;
    let team: TeamSize = parse_json(required(&row.team_size_rules, "team-size rules")?)?;
    Ok(CurrencyWarsEconomyRules {
        stable_key: row.stable_key.clone().into(),
        currency_ids: parse_boxed_strings(row.currency_ids.as_ref())?,
        experience: CurrencyWarsExperienceRules {
            resource_id: number(&experience.resource_id)?,
            standard_wave_gain: number(&experience.standard_wave_gain)?,
            standard_boss_wave_gain: number(&experience.standard_boss_wave_gain)?,
            overclock_wave_gain: array3(experience.overclock_wave_gain)?,
            overclock_boss_wave_gain: array3(experience.overclock_boss_wave_gain)?,
            direct_level_up_experience: number(&experience.direct_level_up_exp)?,
            direct_level_up_gold: number(&experience.direct_level_up_gold)?,
        },
        interest: CurrencyWarsInterestRules {
            deposit_per_interest: number(&interest.deposit_per_interest)?,
            standard_maximum: number(&interest.standard_max_interest)?,
            overclock_maximum: number(&interest.overclock_max_interest)?,
        },
        refresh: CurrencyWarsRefreshRules {
            cards_per_refresh: number(&refresh.cards_per_refresh)?,
            gold_cost: number(&refresh.refresh_gold)?,
            copies_per_role_by_rarity: array5(refresh.copies_per_role_by_rarity)?,
            role_initial_weight: number(&refresh.role_initial_weight)?,
            maximum_stolen_same_card_by_rarity: array5(refresh.maximum_stolen_same_card_by_rarity)?,
            stolen_pool_refund_initial_purchase: number(
                &refresh.stolen_pool_refund_initial_purchase,
            )?,
            stolen_pool_refund_sell: number(&refresh.stolen_pool_refund_sell)?,
            stolen_pool_refund_hold: number(&refresh.stolen_pool_refund_hold)?,
        },
        team_size: CurrencyWarsTeamSizeRules {
            front_minimum: number(&team.front_min)?,
            front_maximum: number(&team.front_max)?,
            back_initial: number(&team.back_initial)?,
            back_maximum: number(&team.back_max)?,
            bench_authored: number(&team.bench_authored)?,
            bench_overflow: number(&team.bench_overflow)?,
        },
    })
}

fn lower_offers(config: &SoraConfig) -> Result<Vec<CurrencyWarsOfferLevel>, CurrencyWarsDataError> {
    config
        .currency_wars_roster_offers()
        .ordered_rows()
        .map(|row| {
            if required(&row.cost_rule, "offer cost rule")? != "GridFightShopPrice.BuyGoldStar1"
                || required(&row.fallback, "offer fallback")? != "RejectIfNoPositiveRarityWeight"
            {
                return Err(error("Currency Wars offer policy is unknown"));
            }
            let weights: BTreeMap<u8, String> =
                parse_json(required(&row.weights, "offer weights")?)?;
            Ok(CurrencyWarsOfferLevel {
                level: stable_tail(&row.stable_key)?
                    .try_into()
                    .map_err(debug_error)?,
                candidates: parse_strings(row.candidate_avatar_ids.as_ref())?
                    .into_iter()
                    .map(|stable| role_id(stable_tail(&stable)?))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
                rarity_weights: [
                    weight(&weights, 1)?,
                    weight(&weights, 2)?,
                    weight(&weights, 3)?,
                    weight(&weights, 4)?,
                    weight(&weights, 5)?,
                ],
                cost_rule: CurrencyWarsOfferCostRule::BuyGoldAtStarOne,
                fallback: CurrencyWarsOfferFallback::RejectIfNoPositiveRarityWeight,
            })
        })
        .collect()
}

fn lower_prices(config: &SoraConfig) -> Result<Vec<CurrencyWarsPriceRule>, CurrencyWarsDataError> {
    const CHANGES: [&str; 3] = [
        "Validate roster and Gold preconditions.",
        "Apply the authored Gold price.",
        "Apply the roster mutation.",
    ];
    config
        .currency_wars_roster_transactions()
        .ordered_rows()
        .map(|row| {
            if required(&row.operation, "roster transaction operation")? != "BuyOrSellRosterRole"
                || parse_strings(row.ordered_state_changes.as_ref())? != CHANGES
            {
                return Err(error("Currency Wars roster transaction order is unknown"));
            }
            let eligibility: Eligibility =
                parse_json(required(&row.eligibility, "transaction eligibility")?)?;
            let prices: Prices = parse_json(required(&row.price_rule, "transaction prices")?)?;
            Ok(CurrencyWarsPriceRule {
                stable_key: row.stable_key.clone().into(),
                rarity: number(&eligibility.rarity)?,
                star_levels: eligibility
                    .star_levels
                    .into_iter()
                    .map(|v| number(&v))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
                buy_by_star: prices
                    .buy_by_star
                    .into_iter()
                    .map(|v| number(&v))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
                sell_by_star: prices
                    .sell_by_star
                    .into_iter()
                    .map(|v| number(&v))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
                ordered_changes: Box::new([
                    CurrencyWarsTransactionChange::ValidateRosterAndGold,
                    CurrencyWarsTransactionChange::ApplyAuthoredGoldPrice,
                    CurrencyWarsTransactionChange::ApplyRosterMutation,
                ]),
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
            let level = parse_required(row.level.as_ref(), "team level")?;
            let experience_to_next = parse_optional(row.next_level_experience.as_ref())?;
            let expected = if let Some(cost) = experience_to_next {
                format!("Spend {cost} Experience to reach level {}.", level + 1)
            } else {
                "Maximum authored roster level.".to_owned()
            };
            if parse_strings(row.transition_rules.as_ref())?.as_slice() != [expected] {
                return Err(error("Currency Wars team-level transition is unknown"));
            }
            Ok(CurrencyWarsTeamLevel {
                level,
                field_cap: parse_required(row.field_cap.as_ref(), "team field cap")?,
                bench_cap: parse_required(row.bench_cap.as_ref(), "team bench cap")?,
                experience_to_next,
                transition: if experience_to_next.is_some() {
                    CurrencyWarsTeamLevelTransition::SpendExperienceToNext
                } else {
                    CurrencyWarsTeamLevelTransition::MaximumAuthoredLevel
                },
                properties: lower_authored_properties(required(
                    &row.general_properties,
                    "team-level properties",
                )?)?,
            })
        })
        .collect()
}

fn lower_positions(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsPositionDefinition>, CurrencyWarsDataError> {
    config
        .currency_wars_positions()
        .ordered_rows()
        .map(|row| {
            let value = required(&row.position_kind, "position kind")?;
            let (kind, field_index, validation, eligibility) = match value {
                "Front" => (
                    CurrencyWarsPositionDefinitionKind::Front,
                    "front",
                    "RoleBasicInfo.FrontBackType is Front.",
                    CurrencyWarsPositionEligibility::DirectFront,
                ),
                "Back" => (
                    CurrencyWarsPositionDefinitionKind::Back,
                    "back",
                    "RoleBasicInfo.FrontBackType is Back.",
                    CurrencyWarsPositionEligibility::DirectBack,
                ),
                "Front-Back candidate" => (
                    CurrencyWarsPositionDefinitionKind::FrontBackCandidate,
                    "front-or-back",
                    "RoleBasicInfo omits FrontBackType and exact Front/Back display rows both exist.",
                    CurrencyWarsPositionEligibility::MissingSourceTypeWithBothDisplays,
                ),
                _ => return Err(error("Currency Wars position kind is unknown")),
            };
            if required(&row.field_index, "position field index")? != field_index
                || parse_strings(row.validation_rules.as_ref())?.as_slice() != [validation]
                || parse_strings(row.battle_contributions.as_ref())?.as_slice()
                != ["Activate the role's matching Character Empowerment at battle entry."]
            {
                return Err(error("Currency Wars position definition is unknown"));
            }
            Ok(CurrencyWarsPositionDefinition {
                stable_key: row.stable_key.clone().into(),
                kind,
                field_index: field_index.into(),
                eligibility,
            })
        })
        .collect()
}

fn lower_star_states(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsStarState>, CurrencyWarsDataError> {
    config
        .currency_wars_star_states()
        .ordered_rows()
        .map(|row| {
            let avatar_id = parse_required(row.avatar_id.as_ref(), "star-state avatar ID")?;
            let star = parse_required(row.star_level.as_ref(), "star-state level")?;
            Ok(CurrencyWarsStarState {
                stable_key: row.stable_key.clone().into(),
                owner: star_state_owner(&row.stable_key, avatar_id, star)?,
                star,
                copy_count: parse_required(row.copy_count.as_ref(), "star-state copies")?,
                scaling_refs: parse_boxed_strings(row.scaling_refs.as_ref())?,
                rank_attachments: parse_json::<Vec<RankAttachmentRow>>(required(
                    &row.rank_attachments,
                    "star-state rank attachments",
                )?)?
                .into_iter()
                .map(|attachment| {
                    Ok(CurrencyWarsRankAttachment {
                        rank: attachment.rank,
                        properties: lower_star_properties(attachment.property_modifiers)?,
                    })
                })
                .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?
                .into_boxed_slice(),
                battle_event_id: parse_optional(row.battle_event_id.as_ref())?,
                skill_override_source_ids: parse_number_array(
                    row.skill_override_source_ids.as_ref(),
                )?,
                front_execution_skill_ids: parse_number_array(
                    row.skill_override_destination_ids.as_ref(),
                )?,
                front_display_skill_ids: parse_number_array(row.front_skill_ids.as_ref())?,
                back_execution_skill_ids: parse_number_array(
                    row.back_execution_skill_ids.as_ref(),
                )?,
                back_display_skill_ids: parse_number_array(row.back_skill_ids.as_ref())?,
                back_ability_name: optional_boxed(row.back_ability_name.clone()),
                config_path: optional_boxed(row.config_path.clone()),
                ai_path: optional_boxed(row.ai_path.clone()),
                property_modifiers: lower_star_properties(parse_json(required(
                    &row.property_modifiers,
                    "star-state property modifiers",
                )?)?)?,
                front_power_base: optional_decimal(row.front_power_base.as_deref())?,
                back_power_base: optional_decimal(row.back_power_base.as_deref())?,
                luck_chance: optional_decimal(row.luck_chance.as_deref())?,
                luck_damage: optional_decimal(row.luck_damage.as_deref())?,
                extra_heal_base: optional_decimal(row.extra_heal_base.as_deref())?,
                extra_shield_base: optional_decimal(row.extra_shield_base.as_deref())?,
                hp_base: optional_boxed(row.hp_base.clone()),
                hp_inherit: optional_boxed(row.hp_inherit.clone()),
                hp_skill_id: parse_optional(row.hp_skill_id.as_ref())?,
                speed_base: optional_boxed(row.speed_base.clone()),
                speed_inherit: optional_boxed(row.speed_inherit.clone()),
                speed_skill_id: parse_optional(row.speed_skill_id.as_ref())?,
            })
        })
        .collect()
}

fn lower_influence_properties(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsInfluenceProperty>, CurrencyWarsDataError> {
    config
        .currency_wars_influence_properties()
        .ordered_rows()
        .map(|row| {
            let subject = match required(&row.subject_kind, "influence subject")? {
                "Star" => CurrencyWarsInfluenceSubject::Star,
                "Rarity" => CurrencyWarsInfluenceSubject::Rarity,
                _ => return Err(error("Currency Wars influence subject is unknown")),
            };
            let properties = parse_json::<Vec<InfluencePropertyRow>>(required(
                &row.properties,
                "influence properties",
            )?)?
            .into_iter()
            .map(|property| {
                Ok(CurrencyWarsAuthoredProperty {
                    property: property.property_type.into_boxed_str(),
                    value: Some(decimal(&property.value)?),
                })
            })
            .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
            Ok(CurrencyWarsInfluenceProperty {
                stable_key: row.stable_key.clone().into(),
                subject,
                level: parse_required(row.subject_level.as_ref(), "influence level")?,
                properties: properties.into_boxed_slice(),
            })
        })
        .collect()
}

fn lower_contribution_parameters(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsContributionParameter>, CurrencyWarsDataError> {
    config
        .currency_wars_contribution_parameters()
        .ordered_rows()
        .map(|row| {
            let kind = match required(&row.source_kind, "contribution parameter kind")? {
                "CombinationBonus" => CurrencyWarsContributionParameterKind::CombinationBonus,
                "RuntimeConstant" => CurrencyWarsContributionParameterKind::RuntimeConstant,
                _ => {
                    return Err(error(
                        "Currency Wars contribution parameter kind is unknown",
                    ));
                }
            };
            let combination_ids = parse_number_array(row.combination_ids.as_ref())?;
            let bonus_numbers = parse_number_array(row.bonus_numbers.as_ref())?;
            if kind == CurrencyWarsContributionParameterKind::CombinationBonus
                && (combination_ids.is_empty() || combination_ids.len() != bonus_numbers.len())
            {
                return Err(error("Currency Wars combination parameter is invalid"));
            }
            Ok(CurrencyWarsContributionParameter {
                stable_key: row.stable_key.clone().into(),
                kind,
                source_id: required(&row.source_id, "contribution parameter source ID")?.into(),
                combination_ids,
                bonus_numbers,
                value_json: row.value.as_deref().map(canonical_json).transpose()?,
                consumer_policy: required(
                    &row.consumer_policy,
                    "contribution parameter consumer policy",
                )?
                .into(),
            })
        })
        .collect()
}

fn parse_number_array(value: Option<&String>) -> Result<Box<[u32]>, CurrencyWarsDataError> {
    parse_json::<Vec<String>>(value.map_or("[]", String::as_str))?
        .into_iter()
        .map(|value| value.parse().map_err(debug_error))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn lower_authored_properties(
    value: &str,
) -> Result<Box<[CurrencyWarsAuthoredProperty]>, CurrencyWarsDataError> {
    parse_json::<Vec<AuthoredPropertyRow>>(value)?
        .into_iter()
        .map(|property| {
            Ok(CurrencyWarsAuthoredProperty {
                property: property.property_type.into_boxed_str(),
                value: property.value.as_deref().map(decimal).transpose()?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn lower_star_properties(
    values: Vec<StarPropertyRow>,
) -> Result<Box<[CurrencyWarsAuthoredProperty]>, CurrencyWarsDataError> {
    values
        .into_iter()
        .map(|property| {
            Ok(CurrencyWarsAuthoredProperty {
                property: property.property_type.into_boxed_str(),
                value: property.value.as_deref().map(decimal).transpose()?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn optional_decimal(
    value: Option<&str>,
) -> Result<Option<CurrencyWarsDecimal>, CurrencyWarsDataError> {
    value
        .filter(|value| !value.is_empty())
        .map(decimal)
        .transpose()
}

fn decimal(value: &str) -> Result<CurrencyWarsDecimal, CurrencyWarsDataError> {
    let (negative, unsigned) = value
        .strip_prefix('-')
        .map_or((false, value), |unsigned| (true, unsigned));
    let (whole, fractional) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if whole.is_empty()
        || fractional.len() > 18
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fractional.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(error("Currency Wars authored decimal is invalid"));
    }
    let significand = format!("{whole}{fractional}")
        .parse::<i64>()
        .map_err(debug_error)?;
    CurrencyWarsDecimal::new(
        if negative { -significand } else { significand },
        u8::try_from(fractional.len()).expect("at most 18 decimal places"),
    )
    .ok_or_else(|| error("Currency Wars authored decimal scale is invalid"))
}

fn optional_boxed(value: Option<String>) -> Option<Box<str>> {
    value
        .filter(|value| !value.is_empty())
        .map(String::into_boxed_str)
}

fn lower_star_rules(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsStarRule>, CurrencyWarsDataError> {
    config
        .currency_wars_star_combination_rules()
        .ordered_rows()
        .map(|row| {
            if required(&row.overflow_rule, "star overflow rule")?
                != "Repeat while at least three equal-star copies remain."
            {
                return Err(error("Currency Wars star overflow rule is unknown"));
            }
            let (role, input_star) = role_star(required(&row.input_state, "star input")?)?;
            let (output_role, output_star) =
                role_star(required(&row.output_state, "star output")?)?;
            if role != output_role {
                return Err(error("Currency Wars star rule changes role"));
            }
            Ok(CurrencyWarsStarRule {
                stable_key: row.stable_key.clone().into(),
                role: role_id(role)?,
                input_star,
                required_copies: parse_required(row.required_copies.as_ref(), "star copies")?,
                output_star,
                overflow: CurrencyWarsStarOverflowRule::RepeatEqualStarTriples,
            })
        })
        .collect()
}

fn lower_star_lifecycle(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsStarLifecycleRule>, CurrencyWarsDataError> {
    config
        .currency_wars_star_lifecycle_rules()
        .ordered_rows()
        .map(|row| {
            let (operation, replacement, sale, teardown) =
                match required(&row.operation, "star lifecycle operation")? {
                    "AcquireCopy" => (
                        CurrencyWarsStarLifecycleOperation::AcquireCopy,
                        "Add the copy, then repeatedly combine legal equal-star triples.",
                        "No sale in this operation.",
                        "Preserve the resulting star state in run scope.",
                    ),
                    "SellRole" => (
                        CurrencyWarsStarLifecycleOperation::SellRole,
                        "Remove the selected role state.",
                        "Use the exact rarity/star sell price from roster-transactions.json.",
                        "Remove role, star, position and contribution state together.",
                    ),
                    "AcquireAtMaximumStar" => (
                        CurrencyWarsStarLifecycleOperation::AcquireAtMaximumStar,
                        "Do not synthesize an unauthored higher star state.",
                        "Retain or sell using an explicit user decision.",
                        "No implicit overflow conversion is claimed.",
                    ),
                    _ => return Err(error("Currency Wars star lifecycle operation is unknown")),
                };
            if required(&row.replacement_rule, "star replacement rule")? != replacement
                || required(&row.sale_rule, "star sale rule")? != sale
                || required(&row.teardown, "star teardown rule")? != teardown
            {
                return Err(error("Currency Wars star lifecycle rule is unknown"));
            }
            Ok(CurrencyWarsStarLifecycleRule {
                stable_key: row.stable_key.clone().into(),
                operation,
            })
        })
        .collect()
}

fn lower_squad_hp(config: &SoraConfig) -> Result<CurrencyWarsSquadHpRules, CurrencyWarsDataError> {
    let mut rows = config.currency_wars_squad_hp_rules().ordered_rows();
    let row = rows
        .next()
        .ok_or_else(|| error("Currency Wars Squad HP is missing"))?;
    if rows.next().is_some() {
        return Err(error("Currency Wars Squad HP is duplicated"));
    }
    let maximum: MaximumHp = parse_json(required(&row.maximum_hp, "Squad HP maximum")?)?;
    if maximum.initial_value != "100"
        || maximum.mutation != "ContentDefinedIncreaseOrRecovery"
        || maximum.resolution != "ProjectPolicy"
    {
        return Err(error("Currency Wars Squad HP maximum is unknown"));
    }
    let loss: Vec<LossRule> = parse_json(required(&row.loss_rules, "Squad HP loss rules")?)?;
    let recovery: Vec<RecoveryRule> =
        parse_json(required(&row.recovery_rules, "Squad HP recovery rules")?)?;
    if loss.len() != 1
        || loss[0].trigger != "NodeNonVictory"
        || loss[0].amount != "ConfiguredByNodeOrDifficulty"
        || loss[0].operation != "SubtractThenClampToMinimum"
        || loss[0].resolution != "ExactTriggerPolicyBoundAmount"
        || recovery.len() != 2
        || recovery[0].trigger != "NodeVictory"
        || recovery[0].amount != "0"
        || recovery[0].operation != "PreserveSquadHp"
        || recovery[0].resolution != "ExactPublicText"
        || recovery[1].trigger != "ContentContribution"
        || recovery[1].amount != "ConfiguredByContent"
        || recovery[1].operation != "RestoreOrIncreaseMaximumAsAuthored"
        || recovery[1].resolution != "DeferredToOwningContentBatch"
    {
        return Err(error("Currency Wars Squad HP rule is unknown"));
    }
    Ok(CurrencyWarsSquadHpRules {
        stable_key: row.stable_key.clone().into(),
        initial: parse_required(row.initial_hp.as_ref(), "initial Squad HP")?,
        minimum: parse_required(row.minimum_hp.as_ref(), "minimum Squad HP")?,
        maximum: CurrencyWarsSquadHpMaximum::InitialWithContentDefinedIncreaseOrRecovery,
        loss_rules: Box::new([CurrencyWarsSquadHpLossRule::ConfiguredNodeOrDifficultyOnNonVictory]),
        recovery_rules: Box::new([
            CurrencyWarsSquadHpRecoveryRule::PreserveOnVictory,
            CurrencyWarsSquadHpRecoveryRule::AuthoredContentRestoreOrMaximumIncrease,
        ]),
    })
}

fn lower_action_value_limits(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsActionValueLimit>, CurrencyWarsDataError> {
    config
        .currency_wars_action_value_limits()
        .ordered_rows()
        .map(|row| {
            let source_initial = required(&row.initial_value, "action-value initial")?;
            let source_decrements: Vec<ActionValueDecrement> =
                parse_json(required(&row.decrement_rules, "action-value decrements")?)?;
            let source_timeout: ActionValueTimeout =
                parse_json(required(&row.timeout_boundary, "action-value timeout")?)?;
            let (kind, initial, decrements, timeout) =
                match required(&row.limit_kind, "action-value limit kind")? {
                    "FiniteNodeConfigured"
                        if source_initial == "ConfiguredByNodeOrDifficulty"
                            && source_decrements.as_slice()
                                == [
                                    ActionValueDecrement {
                                        trigger: "CombatTimelineProgress".into(),
                                        amount: "ElapsedAuthoritativeActionValue".into(),
                                        resolution: "ProjectPolicy".into(),
                                    },
                                    ActionValueDecrement {
                                        trigger: "CharacterLethalRescue".into(),
                                        amount: "ConfiguredByBattleContribution".into(),
                                        resolution: "DeferredToP1B4".into(),
                                    },
                                ]
                            && source_timeout
                                == ActionValueTimeout {
                                    condition: "LimitExhaustedBeforeAllEnemiesDefeated".into(),
                                    battle_outcome: "NonVictory".into(),
                                    squad_hp_projection: "ApplyConfiguredNodeOrDifficultyLoss"
                                        .into(),
                                } =>
                    {
                        (
                            CurrencyWarsActionValueLimitKind::FiniteNodeConfigured,
                            CurrencyWarsActionValueInitial::ConfiguredByNodeOrDifficulty,
                            Box::new([
                                CurrencyWarsActionValueDecrement::ElapsedAuthoritativeActionValue,
                                CurrencyWarsActionValueDecrement::ConfiguredCharacterLethalRescue,
                            ]) as Box<[_]>,
                            CurrencyWarsTimeoutBoundary::NonVictoryAndConfiguredSquadHpLoss,
                        )
                    }
                    "Unlimited"
                        if source_initial == "Infinite"
                            && source_decrements.is_empty()
                            && source_timeout
                                == ActionValueTimeout {
                                    condition: "UnreachableForActionValueLimit".into(),
                                    battle_outcome: "NotApplicable".into(),
                                    squad_hp_projection: "None".into(),
                                } =>
                    {
                        (
                            CurrencyWarsActionValueLimitKind::Unlimited,
                            CurrencyWarsActionValueInitial::Infinite,
                            Box::new([]) as Box<[_]>,
                            CurrencyWarsTimeoutBoundary::Unreachable,
                        )
                    }
                    _ => return Err(error("Currency Wars action-value limit is unknown")),
                };
            Ok(CurrencyWarsActionValueLimit {
                stable_key: row.stable_key.clone().into(),
                kind,
                initial,
                decrements,
                timeout,
            })
        })
        .collect()
}

fn lower_battle_result_projections(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsBattleResultProjection>, CurrencyWarsDataError> {
    config
        .currency_wars_battle_result_projections()
        .ordered_rows()
        .map(|row| {
            let (outcome, squad_hp, action_value, run, expected) =
                match required(&row.battle_outcome, "battle outcome")? {
                    "Victory" => (
                        CurrencyWarsBattleOutcomeProjection::Victory,
                        CurrencyWarsSquadHpProjection::PreserveBeforeContentContributions,
                        CurrencyWarsActionValueProjection::CaptureForFinalizationThenDiscard,
                        CurrencyWarsRunDisposition::ContinueUnlessFinalBoss,
                        (
                            "PreserveBeforeContentContributions",
                            "CaptureForFinalizationThenDiscard",
                            "ContinueUnlessFinalBossCompletesRun",
                        ),
                    ),
                    "NonVictory" => (
                        CurrencyWarsBattleOutcomeProjection::NonVictory,
                        CurrencyWarsSquadHpProjection::SubtractConfiguredLossClampToZero,
                        CurrencyWarsActionValueProjection::CaptureExhaustedThenDiscard,
                        CurrencyWarsRunDisposition::FailAtZeroOtherwiseContinue,
                        (
                            "SubtractConfiguredLossClampToZeroThenEvaluateRun",
                            "CaptureExhaustedLimitThenDiscard",
                            "FailAtZeroOtherwiseContinue",
                        ),
                    ),
                    _ => return Err(error("Currency Wars battle-result outcome is unknown")),
                };
            if required(&row.squad_hp_projection, "Squad HP projection")? != expected.0
                || required(&row.action_value_projection, "action-value projection")? != expected.1
                || required(&row.run_disposition, "run disposition")? != expected.2
            {
                return Err(error("Currency Wars battle-result projection is unknown"));
            }
            Ok(CurrencyWarsBattleResultProjection {
                stable_key: row.stable_key.clone().into(),
                outcome,
                squad_hp,
                action_value,
                run,
            })
        })
        .collect()
}

#[derive(Deserialize)]
struct Experience {
    resource_id: String,
    standard_wave_gain: String,
    standard_boss_wave_gain: String,
    overclock_wave_gain: Vec<String>,
    overclock_boss_wave_gain: Vec<String>,
    direct_level_up_exp: String,
    direct_level_up_gold: String,
}
#[derive(Deserialize)]
struct Refresh {
    cards_per_refresh: String,
    refresh_gold: String,
    copies_per_role_by_rarity: Vec<String>,
    role_initial_weight: String,
    maximum_stolen_same_card_by_rarity: Vec<String>,
    stolen_pool_refund_initial_purchase: String,
    stolen_pool_refund_sell: String,
    stolen_pool_refund_hold: String,
}
#[derive(Deserialize)]
struct Interest {
    deposit_per_interest: String,
    standard_max_interest: String,
    overclock_max_interest: String,
}
#[derive(Deserialize)]
struct TeamSize {
    front_min: String,
    front_max: String,
    back_initial: String,
    back_max: String,
    bench_authored: String,
    bench_overflow: String,
}
#[derive(Deserialize)]
struct Eligibility {
    rarity: String,
    star_levels: Vec<String>,
}
#[derive(Deserialize)]
struct Prices {
    buy_by_star: Vec<String>,
    sell_by_star: Vec<String>,
}
#[derive(Deserialize)]
struct MaximumHp {
    initial_value: String,
    mutation: String,
    resolution: String,
}
#[derive(Deserialize)]
struct LossRule {
    trigger: String,
    amount: String,
    operation: String,
    resolution: String,
}
#[derive(Deserialize)]
struct RecoveryRule {
    trigger: String,
    amount: String,
    operation: String,
    resolution: String,
}
#[derive(Deserialize, PartialEq)]
struct ActionValueDecrement {
    trigger: String,
    amount: String,
    resolution: String,
}
#[derive(Deserialize, PartialEq)]
struct ActionValueTimeout {
    condition: String,
    battle_outcome: String,
    squad_hp_projection: String,
}

fn parse_strings(value: Option<&String>) -> Result<Vec<String>, CurrencyWarsDataError> {
    parse_json(value.map_or("[]", String::as_str))
}
fn parse_required<T: std::str::FromStr>(
    value: Option<&String>,
    name: &str,
) -> Result<T, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    value
        .filter(|v| !v.is_empty())
        .ok_or_else(|| error(&format!("{name} is missing")))?
        .parse()
        .map_err(debug_error)
}
fn parse_optional<T: std::str::FromStr>(
    value: Option<&String>,
) -> Result<Option<T>, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    value
        .filter(|v| !v.is_empty())
        .map(|v| v.parse().map_err(debug_error))
        .transpose()
}
fn number<T: std::str::FromStr>(value: &str) -> Result<T, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    value.parse().map_err(debug_error)
}
fn array3(values: Vec<String>) -> Result<[u32; 3], CurrencyWarsDataError> {
    values
        .into_iter()
        .map(|v| number(&v))
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(debug_error)
}
fn array5(values: Vec<String>) -> Result<[u32; 5], CurrencyWarsDataError> {
    values
        .into_iter()
        .map(|value| number(&value))
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(debug_error)
}
fn stable_tail(stable: &str) -> Result<u32, CurrencyWarsDataError> {
    stable
        .rsplit('.')
        .next()
        .ok_or_else(|| error("stable ID has no tail"))?
        .parse()
        .map_err(debug_error)
}
fn role_id(raw: u32) -> Result<CurrencyWarsRoleId, CurrencyWarsDataError> {
    CurrencyWarsRoleId::new(raw).ok_or_else(|| error("Currency Wars role ID is zero"))
}
fn role_star(stable: &str) -> Result<(u32, u8), CurrencyWarsDataError> {
    let mut values = stable.rsplit('.');
    let star = values
        .next()
        .ok_or_else(|| error("star state has no level"))?
        .parse()
        .map_err(debug_error)?;
    let role = values
        .next()
        .ok_or_else(|| error("star state has no role"))?
        .parse()
        .map_err(debug_error)?;
    Ok((role, star))
}
fn star_state_owner(
    stable: &str,
    avatar_id: u32,
    star: u8,
) -> Result<CurrencyWarsStarStateOwner, CurrencyWarsDataError> {
    let values = stable.split('.').collect::<Vec<_>>();
    match values.as_slice() {
        [
            "currency-wars",
            "star-state",
            "role",
            stable_role,
            stable_star,
        ] if stable_role.parse::<u32>().ok() == Some(avatar_id)
            && stable_star.parse::<u8>().ok() == Some(star) =>
        {
            Ok(CurrencyWarsStarStateOwner::Role(role_id(avatar_id)?))
        }
        [
            "currency-wars",
            "star-state",
            "servant",
            stable_role,
            servant,
            stable_star,
        ] if stable_role.parse::<u32>().ok() == Some(avatar_id)
            && stable_star.parse::<u8>().ok() == Some(star) =>
        {
            Ok(CurrencyWarsStarStateOwner::Servant {
                avatar_id,
                servant_id: servant.parse().map_err(debug_error)?,
            })
        }
        _ => Err(error("Currency Wars star-state identity is invalid")),
    }
}
fn weight(weights: &BTreeMap<u8, String>, rarity: u8) -> Result<u32, CurrencyWarsDataError> {
    number(
        weights
            .get(&rarity)
            .ok_or_else(|| error("rarity weight is missing"))?,
    )
}
