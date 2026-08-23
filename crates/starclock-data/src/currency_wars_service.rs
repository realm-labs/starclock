use std::collections::BTreeSet;

use serde::Deserialize;
use starclock_mode_currency_wars::{
    CurrencyWarsConsumableDefinition, CurrencyWarsConsumableKind, CurrencyWarsDecimal,
    CurrencyWarsEquipmentId, CurrencyWarsEquipmentRecipe, CurrencyWarsEquipmentUpgrade,
    CurrencyWarsForgeService, CurrencyWarsForgeTarget, CurrencyWarsItemDefinition,
    CurrencyWarsItemId, CurrencyWarsManagedFunction, CurrencyWarsRewardDefinition,
    CurrencyWarsRewardKind, CurrencyWarsRewardPool, CurrencyWarsRewardPoolCandidate,
    CurrencyWarsServiceCatalog, CurrencyWarsServiceCatalogParts, CurrencyWarsServiceConstant,
    CurrencyWarsServiceConstantValue, CurrencyWarsSpecialGood, CurrencyWarsSpecialGoodAcquisition,
};

use crate::{
    currency_wars::{CurrencyWarsDataError, debug_error, error},
    currency_wars_build::equipment_category,
    currency_wars_flow::{parse_json, required},
    currency_wars_generated::SoraConfig,
};

pub(super) fn lower_currency_wars_services(
    config: &SoraConfig,
) -> Result<CurrencyWarsServiceCatalog, CurrencyWarsDataError> {
    let (items, special_goods) = lower_shop_services(config)?;
    CurrencyWarsServiceCatalog::new(CurrencyWarsServiceCatalogParts {
        items,
        special_goods,
        season_items: lower_season_items(config)?,
        consumables: lower_consumables(config)?,
        managed_functions: lower_managed_functions(config)?,
        rewards: lower_rewards(config)?,
        reward_pools: lower_reward_pools(config)?,
        recipes: lower_recipes(config)?,
        upgrades: lower_upgrades(config)?,
        forge_services: lower_forge_services(config)?,
        constants: lower_constants(config)?,
        gamble_group_count: config.currency_wars_gamble_groups().len(),
        gamble_unit_count: config.currency_wars_gamble_units().len(),
        curse_chest_count: config.currency_wars_curse_chests().len(),
        hex_state_count: config.currency_wars_hex_states().len(),
        hex_eligibility_count: config.currency_wars_hex_eligibility().len(),
        curio_count: config.currency_wars_curios().len(),
        curio_group_count: config.currency_wars_curio_groups().len(),
        curio_state_count: config.currency_wars_curio_states().len(),
        curio_lifecycle_count: config.currency_wars_curio_lifecycle_rules().len(),
    })
    .map_err(debug_error)
}

#[derive(Deserialize)]
struct ItemInventoryRule {
    priority: String,
}

#[derive(Deserialize)]
struct SpecialGoodPrice {
    acquisition_kind: String,
    amount: String,
}

#[derive(Deserialize)]
struct SpecialGoodInventoryRule {
    group_id: String,
    quality: String,
    config_path: String,
    effect_parameters: Vec<String>,
}

fn lower_shop_services(
    config: &SoraConfig,
) -> Result<
    (
        Vec<CurrencyWarsItemDefinition>,
        Vec<CurrencyWarsSpecialGood>,
    ),
    CurrencyWarsDataError,
> {
    let mut items = Vec::new();
    let mut special_goods = Vec::new();
    for row in config.currency_wars_shop_services().ordered_rows() {
        let source_id = stable_tail(&row.stable_key)?;
        match required(&row.service_kind, "shop service kind")? {
            "ItemCatalogIdentity" => {
                let inventory: ItemInventoryRule =
                    parse_json(required(&row.inventory_rule, "item inventory rule")?)?;
                items.push(CurrencyWarsItemDefinition {
                    id: item_id(source_id)?,
                    stable_key: row.stable_key.clone().into(),
                    priority: inventory.priority.parse().map_err(debug_error)?,
                });
            }
            "SpecialGood" => {
                let price: SpecialGoodPrice =
                    parse_json(required(&row.price_rule, "special-good price rule")?)?;
                let inventory: SpecialGoodInventoryRule = parse_json(required(
                    &row.inventory_rule,
                    "special-good inventory rule",
                )?)?;
                special_goods.push(CurrencyWarsSpecialGood {
                    id: source_id,
                    stable_key: row.stable_key.clone().into(),
                    group_id: inventory.group_id.parse().map_err(debug_error)?,
                    quality: inventory.quality.parse().map_err(debug_error)?,
                    acquisition: match price.acquisition_kind.as_str() {
                        "ShopPurchase" => CurrencyWarsSpecialGoodAcquisition::ShopPurchase {
                            price: price.amount.parse().map_err(debug_error)?,
                        },
                        "CyreneThreeStar" if price.amount.is_empty() => {
                            CurrencyWarsSpecialGoodAcquisition::CyreneThreeStar
                        }
                        _ => {
                            return Err(error("Currency Wars special-good acquisition is invalid"));
                        }
                    },
                    config_path: inventory.config_path.into(),
                    parameters: inventory
                        .effect_parameters
                        .iter()
                        .map(|value| decimal(value))
                        .collect::<Result<Vec<_>, _>>()?
                        .into_boxed_slice(),
                });
            }
            _ => return Err(error("Currency Wars shop service kind is unknown")),
        }
    }
    Ok((items, special_goods))
}

fn lower_season_items(
    config: &SoraConfig,
) -> Result<BTreeSet<CurrencyWarsItemId>, CurrencyWarsDataError> {
    config
        .currency_wars_service_offer_rules()
        .ordered_rows()
        .map(|row| {
            if required(&row.service_id, "service offer owner")? != "season:1" {
                return Err(error("Currency Wars service offer has an unknown season"));
            }
            let candidates: Vec<String> =
                parse_json(required(&row.candidate_ids, "service offer candidates")?)?;
            if candidates.len() != 1 {
                return Err(error("Currency Wars season item offer is not singular"));
            }
            item_id(candidates[0].parse().map_err(debug_error)?)
        })
        .collect()
}

#[derive(Deserialize)]
struct ConsumableInput {
    consume: bool,
    stack: bool,
}

#[derive(Deserialize)]
struct ConsumableOutput {
    parameters: Vec<String>,
}

fn lower_consumables(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsConsumableDefinition>, CurrencyWarsDataError> {
    config
        .currency_wars_workbench_functions()
        .ordered_rows()
        .map(|row| {
            let input: ConsumableInput =
                parse_json(required(&row.input_policy, "consumable input policy")?)?;
            let output: ConsumableOutput =
                parse_json(required(&row.output_policy, "consumable output policy")?)?;
            Ok(CurrencyWarsConsumableDefinition {
                item: item_id(stable_tail(&row.stable_key)?)?,
                stable_key: row.stable_key.clone().into(),
                kind: consumable_kind(required(&row.function_type, "consumable function type")?)?,
                consume: input.consume,
                stack: input.stack,
                parameters: parse_u32s(output.parameters)?.into_boxed_slice(),
            })
        })
        .collect()
}

#[derive(Deserialize)]
struct ManagedAvailability {
    unlock_id: String,
    locked_show_type: String,
}

fn lower_managed_functions(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsManagedFunction>, CurrencyWarsDataError> {
    config
        .currency_wars_workbenches()
        .ordered_rows()
        .map(|row| {
            let function_ids: Vec<String> =
                parse_json(required(&row.function_ids, "managed function IDs")?)?;
            let [function_id] = function_ids.as_slice() else {
                return Err(error("Currency Wars managed function is not singular"));
            };
            let availability: ManagedAvailability = parse_json(required(
                &row.availability,
                "managed function availability",
            )?)?;
            Ok(CurrencyWarsManagedFunction {
                stable_key: row.stable_key.clone().into(),
                function_id: function_id.as_str().into(),
                unlock_id: availability.unlock_id.parse().map_err(debug_error)?,
                hidden_while_locked: match availability.locked_show_type.as_str() {
                    "Hide" => true,
                    "Show" => false,
                    _ => {
                        return Err(error(
                            "Currency Wars managed function visibility is unknown",
                        ));
                    }
                },
            })
        })
        .collect()
}

fn lower_rewards(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsRewardDefinition>, CurrencyWarsDataError> {
    config
        .currency_wars_reward_definitions()
        .ordered_rows()
        .map(|row| {
            let id = required(&row.source_id, "reward source ID")?
                .parse()
                .map_err(debug_error)?;
            let parameters: Vec<String> =
                parse_json(required(&row.parameters, "reward parameters")?)?;
            let scalar_parameter = row
                .scalar_parameter
                .as_deref()
                .filter(|value| !value.is_empty())
                .map(|value| value.parse().map_err(debug_error))
                .transpose()?;
            Ok(CurrencyWarsRewardDefinition {
                id,
                stable_key: row.stable_key.clone().into(),
                budget_cost: reward_budget_cost(id, row.budget_cost.as_deref())?,
                scalar_parameter,
                kind: reward_kind(
                    required(&row.operation_kind, "reward operation kind")?,
                    parameters,
                )?,
            })
        })
        .collect()
}

fn lower_reward_pools(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsRewardPool>, CurrencyWarsDataError> {
    config
        .currency_wars_reward_pools()
        .ordered_rows()
        .map(|row| {
            let ids: Vec<String> = parse_json(required(
                &row.candidate_bonus_ids,
                "reward pool candidates",
            )?)?;
            let maximums: Vec<String> =
                parse_json(required(&row.candidate_maximums, "reward pool maximums")?)?;
            let weights: Vec<String> =
                parse_json(required(&row.candidate_weights, "reward pool weights")?)?;
            if ids.len() != maximums.len() || ids.len() != weights.len() {
                return Err(error("Currency Wars reward pool vectors differ in length"));
            }
            let candidates = ids
                .iter()
                .zip(maximums.iter())
                .zip(weights.iter())
                .map(|((id, maximum), weight)| {
                    Ok(CurrencyWarsRewardPoolCandidate {
                        reward_id: id.parse().map_err(debug_error)?,
                        maximum: maximum.parse().map_err(debug_error)?,
                        weight: weight.parse().map_err(debug_error)?,
                    })
                })
                .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
            Ok(CurrencyWarsRewardPool {
                id: required(&row.source_id, "reward pool source ID")?
                    .parse()
                    .map_err(debug_error)?,
                stable_key: row.stable_key.clone().into(),
                total_value: required(&row.total_value, "reward pool total value")?
                    .parse()
                    .map_err(debug_error)?,
                candidates: candidates.into_boxed_slice(),
            })
        })
        .collect()
}

fn lower_recipes(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsEquipmentRecipe>, CurrencyWarsDataError> {
    config
        .currency_wars_equipment_recipes()
        .ordered_rows()
        .map(|row| {
            let inputs: Vec<String> = parse_json(required(
                &row.input_equipment_ids,
                "equipment recipe inputs",
            )?)?;
            Ok(CurrencyWarsEquipmentRecipe {
                id: required(&row.source_id, "equipment recipe source ID")?
                    .parse()
                    .map_err(debug_error)?,
                stable_key: row.stable_key.clone().into(),
                season_id: required(&row.season_id, "equipment recipe season")?
                    .parse()
                    .map_err(debug_error)?,
                output: equipment_id(required(
                    &row.output_equipment_id,
                    "equipment recipe output",
                )?)?,
                inputs: inputs
                    .iter()
                    .map(|value| equipment_id(value))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
            })
        })
        .collect()
}

fn lower_upgrades(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsEquipmentUpgrade>, CurrencyWarsDataError> {
    config
        .currency_wars_equipment_upgrades()
        .ordered_rows()
        .map(|row| {
            Ok(CurrencyWarsEquipmentUpgrade {
                source: equipment_id(required(
                    &row.source_equipment_id,
                    "equipment upgrade source",
                )?)?,
                output: equipment_id(required(
                    &row.output_equipment_id,
                    "equipment upgrade output",
                )?)?,
            })
        })
        .collect()
}

fn lower_forge_services(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsForgeService>, CurrencyWarsDataError> {
    config
        .currency_wars_forge_services()
        .ordered_rows()
        .map(|row| {
            let parameters: Vec<String> =
                parse_json(required(&row.parameters, "forge parameters")?)?;
            let parameters = parse_u32s(parameters)?;
            Ok(CurrencyWarsForgeService {
                item: item_id(
                    required(&row.source_id, "forge source ID")?
                        .parse()
                        .map_err(debug_error)?,
                )?,
                stable_key: row.stable_key.clone().into(),
                category: equipment_category(required(
                    &row.equipment_category,
                    "forge equipment category",
                )?)?,
                offer_count: required(&row.offer_count, "forge offer count")?
                    .parse()
                    .map_err(debug_error)?,
                target: forge_target(
                    required(&row.target_kind, "forge target kind")?,
                    &parameters,
                )?,
            })
        })
        .collect()
}

fn lower_constants(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsServiceConstant>, CurrencyWarsDataError> {
    config
        .currency_wars_service_constants()
        .ordered_rows()
        .map(|row| {
            Ok(CurrencyWarsServiceConstant {
                name: required(&row.source_id, "service constant name")?.into(),
                value: service_constant_value(required(&row.value, "service constant value")?)?,
            })
        })
        .collect()
}

#[derive(Deserialize)]
struct RawServiceConstantValue {
    #[serde(rename = "IntValue")]
    integer: Option<String>,
    #[serde(rename = "ArrayValue", default)]
    integers: Vec<RawServiceConstantInteger>,
}

#[derive(Deserialize)]
struct RawServiceConstantInteger {
    #[serde(rename = "IntValue")]
    integer: String,
}

fn service_constant_value(
    value: &str,
) -> Result<CurrencyWarsServiceConstantValue, CurrencyWarsDataError> {
    let raw: RawServiceConstantValue = parse_json(value)?;
    match (raw.integer, raw.integers.as_slice()) {
        (Some(integer), []) => Ok(CurrencyWarsServiceConstantValue::Integer(
            integer.parse().map_err(debug_error)?,
        )),
        (None, values) if !values.is_empty() => Ok(CurrencyWarsServiceConstantValue::IntegerArray(
            values
                .iter()
                .map(|value| value.integer.parse().map_err(debug_error))
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(),
        )),
        _ => Err(error("Currency Wars service constant shape is invalid")),
    }
}

fn reward_kind(
    kind: &str,
    parameters: Vec<String>,
) -> Result<CurrencyWarsRewardKind, CurrencyWarsDataError> {
    let values = parse_u32s(parameters)?;
    match (kind, values.as_slice()) {
        ("DefaultCurrency", []) | ("DefaultCurrency", [_]) => {
            Ok(CurrencyWarsRewardKind::DefaultCurrency)
        }
        ("Refresh", []) | ("Refresh", [_]) => Ok(CurrencyWarsRewardKind::Refresh),
        ("Exp", []) => Ok(CurrencyWarsRewardKind::Experience),
        ("Item", [item]) => Ok(CurrencyWarsRewardKind::Item {
            item: item_id(*item)?,
            count: 1,
        }),
        ("Item", [item, count]) if *count != 0 => Ok(CurrencyWarsRewardKind::Item {
            item: item_id(*item)?,
            count: *count,
        }),
        ("Orb", [orb]) => Ok(CurrencyWarsRewardKind::Orb(*orb)),
        ("RandomAvatar", [rarity, star]) => Ok(CurrencyWarsRewardKind::RandomRole {
            rarity: u8::try_from(*rarity).map_err(debug_error)?,
            star: u8::try_from(*star).map_err(debug_error)?,
        }),
        ("SpecificAvatar", [avatar_id, star]) => Ok(CurrencyWarsRewardKind::SpecificAvatar {
            avatar_id: *avatar_id,
            star: u8::try_from(*star).map_err(debug_error)?,
        }),
        ("RandomEquipByCategory", [selector]) => {
            Ok(CurrencyWarsRewardKind::RandomEquipmentByCategory(*selector))
        }
        ("RandomEquipByFunc", [selector]) => {
            Ok(CurrencyWarsRewardKind::RandomEquipmentByFunction(*selector))
        }
        ("SpecificAvatarWithEquip", [avatar_id, star, equipment @ ..]) if !equipment.is_empty() => {
            Ok(CurrencyWarsRewardKind::SpecificAvatarWithEquipment {
                avatar_id: *avatar_id,
                star: u8::try_from(*star).map_err(debug_error)?,
                equipment: equipment
                    .iter()
                    .copied()
                    .map(equipment_id_raw)
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
            })
        }
        ("SpecificAvatarWithRandomEquip", [avatar_id, star, selector, count]) => {
            Ok(CurrencyWarsRewardKind::SpecificAvatarWithRandomEquipment {
                avatar_id: *avatar_id,
                star: u8::try_from(*star).map_err(debug_error)?,
                category_selector: *selector,
                count: u8::try_from(*count).map_err(debug_error)?,
            })
        }
        _ => Err(error(
            "Currency Wars reward operation is unknown or malformed",
        )),
    }
}

fn consumable_kind(value: &str) -> Result<CurrencyWarsConsumableKind, CurrencyWarsDataError> {
    match value {
        "DirectConsumable" => Ok(CurrencyWarsConsumableKind::RemoveEquipment),
        "Roll" => Ok(CurrencyWarsConsumableKind::RerollEquipment),
        "Upgrade" => Ok(CurrencyWarsConsumableKind::UpgradeEquipment),
        "Copy" => Ok(CurrencyWarsConsumableKind::CopyRole),
        "GainRecommendEquip" => Ok(CurrencyWarsConsumableKind::GainRecommendedEquipment),
        _ => Err(error("Currency Wars consumable function is unknown")),
    }
}

fn forge_target(
    kind: &str,
    parameters: &[u32],
) -> Result<CurrencyWarsForgeTarget, CurrencyWarsDataError> {
    match (kind, parameters) {
        ("Equip", [_]) => Ok(CurrencyWarsForgeTarget::Equipment),
        ("Role", [rarity, star]) => Ok(CurrencyWarsForgeTarget::Role {
            rarity: u8::try_from(*rarity).map_err(debug_error)?,
            star: u8::try_from(*star).map_err(debug_error)?,
        }),
        ("Expert", [minimum, maximum]) => Ok(CurrencyWarsForgeTarget::Expert {
            minimum: u8::try_from(*minimum).map_err(debug_error)?,
            maximum: u8::try_from(*maximum).map_err(debug_error)?,
        }),
        _ => Err(error("Currency Wars forge target is unknown or malformed")),
    }
}

fn reward_budget_cost(id: u32, value: Option<&str>) -> Result<Option<u32>, CurrencyWarsDataError> {
    match value.filter(|value| !value.is_empty()) {
        Some(value) => value.parse().map(Some).map_err(debug_error),
        // Released 4.4 data omits only reward 350101. Its sole pool has value 3
        // beside a value-2 candidate, so the versioned project policy assigns 1.
        None if id == 350_101 => Ok(Some(1)),
        None => Ok(None),
    }
}

fn parse_u32s(values: Vec<String>) -> Result<Vec<u32>, CurrencyWarsDataError> {
    values
        .into_iter()
        .map(|value| value.parse().map_err(debug_error))
        .collect()
}

fn decimal(value: &str) -> Result<CurrencyWarsDecimal, CurrencyWarsDataError> {
    let (negative, unsigned) = value
        .strip_prefix('-')
        .map_or((false, value), |rest| (true, rest));
    let mut parts = unsigned.split('.');
    let integer = parts.next().unwrap_or_default();
    let fraction = parts.next().unwrap_or_default();
    if unsigned.is_empty()
        || integer.is_empty()
        || parts.next().is_some()
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        || (integer.len() > 1 && integer.starts_with('0'))
        || (!fraction.is_empty() && fraction.ends_with('0'))
        || fraction.len() > 18
    {
        return Err(error("Currency Wars service decimal is not canonical"));
    }
    let digits = format!("{integer}{fraction}");
    let magnitude: i64 = digits.parse().map_err(debug_error)?;
    let significand = if negative {
        magnitude
            .checked_neg()
            .ok_or_else(|| error("Currency Wars service decimal overflow"))?
    } else {
        magnitude
    };
    CurrencyWarsDecimal::new(
        significand,
        u8::try_from(fraction.len()).map_err(debug_error)?,
    )
    .ok_or_else(|| error("Currency Wars service decimal precision is invalid"))
}

fn stable_tail(value: &str) -> Result<u32, CurrencyWarsDataError> {
    value
        .rsplit('.')
        .next()
        .ok_or_else(|| error("Currency Wars service stable key has no tail"))?
        .parse()
        .map_err(debug_error)
}

fn item_id(raw: u32) -> Result<CurrencyWarsItemId, CurrencyWarsDataError> {
    CurrencyWarsItemId::new(raw).ok_or_else(|| error("Currency Wars item ID is zero"))
}

fn equipment_id(value: &str) -> Result<CurrencyWarsEquipmentId, CurrencyWarsDataError> {
    equipment_id_raw(value.parse().map_err(debug_error)?)
}

fn equipment_id_raw(raw: u32) -> Result<CurrencyWarsEquipmentId, CurrencyWarsDataError> {
    CurrencyWarsEquipmentId::new(raw).ok_or_else(|| error("Currency Wars equipment ID is zero"))
}
