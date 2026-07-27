//! Closed typed execution plans for Standard Universe services and interactables.

use crate::{
    digest::Encoder,
    id::ServiceId,
    progression::{ServiceKind, ServiceProfileOwner},
    run_runtime::{RunRuntimeCatalog, ServiceRuntimeDefinition},
};

pub const SERVICE_EFFECT_RUNTIME_REVISION: &str = "standard-universe-service-effect-runtime-v2";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServicePriceStep {
    use_index: u8,
    amount: u32,
}

impl ServicePriceStep {
    #[must_use]
    pub const fn use_index(self) -> u8 {
        self.use_index
    }
    #[must_use]
    pub const fn amount(self) -> u32 {
        self.amount
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RespiteOfferKind {
    OneStarBlessing,
    Curio,
    EnhanceRandomBlessings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RespiteOffer {
    kind: RespiteOfferKind,
    amount: u8,
    cost: u32,
}

impl RespiteOffer {
    #[must_use]
    pub const fn kind(self) -> RespiteOfferKind {
        self.kind
    }
    #[must_use]
    pub const fn amount(self) -> u8 {
        self.amount
    }
    #[must_use]
    pub const fn cost(self) -> u32 {
        self.cost
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrailblazeBonusEffect {
    AddFragments {
        amount: u32,
    },
    RandomBlessing {
        quantity: u8,
        minimum_rarity: u8,
        maximum_rarity: u8,
    },
    RandomCurio {
        quantity: u8,
        cost: Option<u32>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrailblazeBonusTier {
    Ordinary,
    Enhanced,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServiceAction {
    InitializeCurrency {
        amount: u32,
    },
    ResetBlessingOffer {
        cost_schedule: Box<[ServicePriceStep]>,
        offer_pool_key: Box<str>,
    },
    ReviveCharacter {
        cost: u32,
        restored_hp_percent: u8,
    },
    AddReserveCharacter {
        amount: u8,
    },
    OfferRespiteChoices {
        offers: Box<[RespiteOffer]>,
    },
    EnhanceBlessing {
        maximum_enhancements: u8,
        rarity_costs: [u32; 3],
    },
    OpenBlessingShop {
        price_formula_key: Box<str>,
        offer_pool_key: Box<str>,
    },
    OpenCurioShop {
        price_formula_key: Box<str>,
        offer_pool_key: Box<str>,
    },
    GrantTrailblazeBonus {
        offer_pool_key: Box<str>,
        source_event_id: u32,
        tier: TrailblazeBonusTier,
        position: u8,
        effect: TrailblazeBonusEffect,
    },
    ProfileExcluded {
        owner: ServiceProfileOwner,
        source_event_id: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledServiceEffect {
    service: ServiceId,
    source_key: Box<str>,
    rule_key: Box<str>,
    currency_key: Option<Box<str>>,
    price_formula_key: Option<Box<str>>,
    action: ServiceAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedServiceEffect {
    service: ServiceId,
    source_key: Box<str>,
    rule_key: Box<str>,
    currency_key: Option<Box<str>>,
    price_formula_key: Option<Box<str>>,
    action: ServiceAction,
}

impl AppliedServiceEffect {
    #[must_use]
    pub const fn service(&self) -> ServiceId {
        self.service
    }
    #[must_use]
    pub fn source_key(&self) -> &str {
        &self.source_key
    }
    #[must_use]
    pub fn rule_key(&self) -> &str {
        &self.rule_key
    }
    #[must_use]
    pub fn currency_key(&self) -> Option<&str> {
        self.currency_key.as_deref()
    }
    #[must_use]
    pub fn price_formula_key(&self) -> Option<&str> {
        self.price_formula_key.as_deref()
    }
    #[must_use]
    pub const fn action(&self) -> &ServiceAction {
        &self.action
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceEffectRuntimeCatalog {
    programs: Box<[CompiledServiceEffect]>,
    digest: [u8; 32],
}

impl ServiceEffectRuntimeCatalog {
    pub fn compile(runtime: &RunRuntimeCatalog) -> Result<Self, ServiceEffectRuntimeError> {
        let mut programs = runtime
            .services()
            .iter()
            .map(compile_service)
            .collect::<Result<Vec<_>, _>>()?;
        programs.sort_by_key(|program| program.service);
        let rule_count = programs
            .iter()
            .filter(|program| !program.rule_key.is_empty())
            .count();
        let parameter_count = runtime
            .services()
            .iter()
            .map(|service| service.parameters().len())
            .sum::<usize>();
        if programs.len() != 94
            || rule_count != 94
            || parameter_count != 41
            || programs
                .windows(2)
                .any(|pair| pair[0].service == pair[1].service)
        {
            return Err(ServiceEffectRuntimeError::InvalidDenominator);
        }
        let digest = catalog_digest(&programs);
        Ok(Self {
            programs: programs.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub const fn content_count(&self) -> usize {
        94
    }
    #[must_use]
    pub const fn rule_count(&self) -> usize {
        94
    }
    #[must_use]
    pub const fn semantic_fixture_count(&self) -> usize {
        9
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    #[must_use]
    pub fn service_ids(&self) -> impl ExactSizeIterator<Item = ServiceId> + '_ {
        self.programs.iter().map(|program| program.service)
    }
    pub fn trailblaze_bonuses(
        &self,
    ) -> impl Iterator<Item = (ServiceId, TrailblazeBonusTier, u8)> + '_ {
        self.programs
            .iter()
            .filter_map(|program| match program.action {
                ServiceAction::GrantTrailblazeBonus { tier, position, .. } => {
                    Some((program.service, tier, position))
                }
                _ => None,
            })
    }

    pub fn execute(
        &self,
        service: ServiceId,
    ) -> Result<AppliedServiceEffect, ServiceEffectRuntimeError> {
        let program = self
            .programs
            .binary_search_by_key(&service, |program| program.service)
            .ok()
            .map(|index| &self.programs[index])
            .ok_or(ServiceEffectRuntimeError::UnknownService)?;
        Ok(AppliedServiceEffect {
            service: program.service,
            source_key: program.source_key.clone(),
            rule_key: program.rule_key.clone(),
            currency_key: program.currency_key.clone(),
            price_formula_key: program.price_formula_key.clone(),
            action: program.action.clone(),
        })
    }
    pub fn initial_currency_amount(&self) -> Option<u32> {
        let mut values = self
            .programs
            .iter()
            .filter_map(|program| match program.action {
                ServiceAction::InitializeCurrency { amount } => Some(amount),
                _ => None,
            });
        let value = values.next()?;
        values.next().is_none().then_some(value)
    }
}

fn compile_service(
    service: &ServiceRuntimeDefinition,
) -> Result<CompiledServiceEffect, ServiceEffectRuntimeError> {
    if service.rule_key().is_empty() {
        return Err(ServiceEffectRuntimeError::InvalidService);
    }
    let action = if service.profile_owner() != ServiceProfileOwner::Standard {
        if service.kind() != ServiceKind::TrailblazeBonus
            || service.offer_pool_key().is_some()
            || !service.parameters().is_empty()
        {
            return Err(ServiceEffectRuntimeError::InvalidService);
        }
        ServiceAction::ProfileExcluded {
            owner: service.profile_owner(),
            source_event_id: service
                .source_event_id()
                .ok_or(ServiceEffectRuntimeError::InvalidService)?,
        }
    } else {
        match service.kind() {
            ServiceKind::Currency => ServiceAction::InitializeCurrency {
                amount: parameter_u32(service, "initial_amount")?,
            },
            ServiceKind::ResetBlessing => ServiceAction::ResetBlessingOffer {
                cost_schedule: parse_cost_schedule(parameter(service, "source_cost_schedule")?)?,
                offer_pool_key: required(service.offer_pool_key())?.into(),
            },
            ServiceKind::Reviver => ServiceAction::ReviveCharacter {
                cost: parameter_u32(service, "cost")?,
                restored_hp_percent: percentage(service, "restored_hp_percent")?,
            },
            ServiceKind::Downloader => ServiceAction::AddReserveCharacter {
                amount: parameter_u8(service, "characters_per_device")?,
            },
            ServiceKind::RespiteOffers => ServiceAction::OfferRespiteChoices {
                offers: [
                    RespiteOffer {
                        kind: RespiteOfferKind::OneStarBlessing,
                        amount: 1,
                        cost: parameter_u32(service, "one_star_blessing_cost")?,
                    },
                    RespiteOffer {
                        kind: RespiteOfferKind::Curio,
                        amount: 1,
                        cost: parameter_u32(service, "curio_cost")?,
                    },
                    RespiteOffer {
                        kind: RespiteOfferKind::EnhanceRandomBlessings,
                        amount: 2,
                        cost: parameter_u32(service, "two_random_enhancements_cost")?,
                    },
                ]
                .into(),
            },
            ServiceKind::EnhanceBlessing => ServiceAction::EnhanceBlessing {
                maximum_enhancements: parameter_u8(service, "max_enhancements")?,
                rarity_costs: [
                    parameter_u32(service, "rarity_1_cost")?,
                    parameter_u32(service, "rarity_2_cost")?,
                    parameter_u32(service, "rarity_3_cost")?,
                ],
            },
            ServiceKind::BlessingShop => ServiceAction::OpenBlessingShop {
                price_formula_key: required(service.price_formula_key())?.into(),
                offer_pool_key: required(service.offer_pool_key())?.into(),
            },
            ServiceKind::CurioShop => ServiceAction::OpenCurioShop {
                price_formula_key: required(service.price_formula_key())?.into(),
                offer_pool_key: required(service.offer_pool_key())?.into(),
            },
            ServiceKind::TrailblazeBonus => ServiceAction::GrantTrailblazeBonus {
                offer_pool_key: required(service.offer_pool_key())?.into(),
                source_event_id: service
                    .source_event_id()
                    .ok_or(ServiceEffectRuntimeError::InvalidService)?,
                tier: match parameter(service, "offer_tier")? {
                    "Ordinary" => TrailblazeBonusTier::Ordinary,
                    "Enhanced" => TrailblazeBonusTier::Enhanced,
                    _ => return Err(ServiceEffectRuntimeError::InvalidParameter),
                },
                position: {
                    let value = positive_u8(service, "offer_position")?;
                    if value > 3 {
                        return Err(ServiceEffectRuntimeError::InvalidParameter);
                    }
                    value
                },
                effect: trailblaze_bonus_effect(service)?,
            },
        }
    };
    Ok(CompiledServiceEffect {
        service: service.id(),
        source_key: service.stable_key().into(),
        rule_key: service.rule_key().into(),
        currency_key: service.currency_key().map(Into::into),
        price_formula_key: service.price_formula_key().map(Into::into),
        action,
    })
}

fn trailblaze_bonus_effect(
    service: &ServiceRuntimeDefinition,
) -> Result<TrailblazeBonusEffect, ServiceEffectRuntimeError> {
    match parameter(service, "effect_kind")? {
        "AddFragments" => Ok(TrailblazeBonusEffect::AddFragments {
            amount: positive_u32(service, "amount")?,
        }),
        "RandomBlessing" => {
            let minimum_rarity = parameter_u8(service, "minimum_rarity")?;
            let maximum_rarity = parameter_u8(service, "maximum_rarity")?;
            if minimum_rarity == 0 || minimum_rarity > maximum_rarity || maximum_rarity > 3 {
                return Err(ServiceEffectRuntimeError::InvalidParameter);
            }
            Ok(TrailblazeBonusEffect::RandomBlessing {
                quantity: positive_u8(service, "quantity")?,
                minimum_rarity,
                maximum_rarity,
            })
        }
        "RandomCurio" => Ok(TrailblazeBonusEffect::RandomCurio {
            quantity: positive_u8(service, "quantity")?,
            cost: None,
        }),
        "RandomCurioWithCost" => Ok(TrailblazeBonusEffect::RandomCurio {
            quantity: positive_u8(service, "quantity")?,
            cost: Some(positive_u32(service, "cost")?),
        }),
        _ => Err(ServiceEffectRuntimeError::InvalidParameter),
    }
}

fn required(value: Option<&str>) -> Result<&str, ServiceEffectRuntimeError> {
    value
        .filter(|value| !value.is_empty())
        .ok_or(ServiceEffectRuntimeError::InvalidService)
}

fn parameter<'a>(
    service: &'a ServiceRuntimeDefinition,
    key: &str,
) -> Result<&'a str, ServiceEffectRuntimeError> {
    service
        .parameters()
        .iter()
        .find(|parameter| parameter.key() == key)
        .map(|parameter| parameter.value())
        .ok_or(ServiceEffectRuntimeError::InvalidParameter)
}

fn parameter_u32(
    service: &ServiceRuntimeDefinition,
    key: &str,
) -> Result<u32, ServiceEffectRuntimeError> {
    parameter(service, key)?
        .parse::<u32>()
        .map_err(|_| ServiceEffectRuntimeError::InvalidParameter)
}

fn parameter_u8(
    service: &ServiceRuntimeDefinition,
    key: &str,
) -> Result<u8, ServiceEffectRuntimeError> {
    parameter(service, key)?
        .parse::<u8>()
        .map_err(|_| ServiceEffectRuntimeError::InvalidParameter)
}

fn positive_u32(
    service: &ServiceRuntimeDefinition,
    key: &str,
) -> Result<u32, ServiceEffectRuntimeError> {
    parameter_u32(service, key)?
        .checked_sub(1)
        .map(|value| value + 1)
        .ok_or(ServiceEffectRuntimeError::InvalidParameter)
}

fn positive_u8(
    service: &ServiceRuntimeDefinition,
    key: &str,
) -> Result<u8, ServiceEffectRuntimeError> {
    parameter_u8(service, key)?
        .checked_sub(1)
        .map(|value| value + 1)
        .ok_or(ServiceEffectRuntimeError::InvalidParameter)
}

fn percentage(
    service: &ServiceRuntimeDefinition,
    key: &str,
) -> Result<u8, ServiceEffectRuntimeError> {
    let value = parameter_u8(service, key)?;
    if value > 100 {
        return Err(ServiceEffectRuntimeError::InvalidParameter);
    }
    Ok(value)
}

fn parse_cost_schedule(value: &str) -> Result<Box<[ServicePriceStep]>, ServiceEffectRuntimeError> {
    let mut steps = Vec::new();
    for (index, entry) in value.split(',').enumerate() {
        let body = entry
            .strip_prefix("[31:")
            .and_then(|entry| entry.strip_suffix(']'))
            .ok_or(ServiceEffectRuntimeError::InvalidParameter)?;
        steps.push(ServicePriceStep {
            use_index: u8::try_from(index + 1)
                .map_err(|_| ServiceEffectRuntimeError::InvalidParameter)?,
            amount: body
                .parse::<u32>()
                .map_err(|_| ServiceEffectRuntimeError::InvalidParameter)?,
        });
    }
    if steps.is_empty() || steps.iter().any(|step| step.amount == 0) {
        return Err(ServiceEffectRuntimeError::InvalidParameter);
    }
    Ok(steps.into_boxed_slice())
}

fn catalog_digest(programs: &[CompiledServiceEffect]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-service-effect-runtime-catalog-v1");
    encoder.text(SERVICE_EFFECT_RUNTIME_REVISION);
    encoder.u32(programs.len() as u32);
    for program in programs {
        encoder.u32(program.service.get());
        encoder.text(&program.source_key);
        encoder.text(&program.rule_key);
        encoder.text(program.currency_key.as_deref().unwrap_or(""));
        encoder.text(program.price_formula_key.as_deref().unwrap_or(""));
        encode_action(&mut encoder, &program.action);
    }
    encoder.finish()
}

fn encode_action(encoder: &mut Encoder, action: &ServiceAction) {
    match action {
        ServiceAction::InitializeCurrency { amount } => {
            encoder.u8(0);
            encoder.u32(*amount);
        }
        ServiceAction::ResetBlessingOffer {
            cost_schedule,
            offer_pool_key,
        } => {
            encoder.u8(1);
            encoder.u32(cost_schedule.len() as u32);
            for step in cost_schedule {
                encoder.u8(step.use_index);
                encoder.u32(step.amount);
            }
            encoder.text(offer_pool_key);
        }
        ServiceAction::ReviveCharacter {
            cost,
            restored_hp_percent,
        } => {
            encoder.u8(2);
            encoder.u32(*cost);
            encoder.u8(*restored_hp_percent);
        }
        ServiceAction::AddReserveCharacter { amount } => {
            encoder.u8(3);
            encoder.u8(*amount);
        }
        ServiceAction::OfferRespiteChoices { offers } => {
            encoder.u8(4);
            encoder.u32(offers.len() as u32);
            for offer in offers {
                encoder.u8(offer.kind as u8);
                encoder.u8(offer.amount);
                encoder.u32(offer.cost);
            }
        }
        ServiceAction::EnhanceBlessing {
            maximum_enhancements,
            rarity_costs,
        } => {
            encoder.u8(5);
            encoder.u8(*maximum_enhancements);
            for cost in rarity_costs {
                encoder.u32(*cost);
            }
        }
        ServiceAction::OpenBlessingShop {
            price_formula_key,
            offer_pool_key,
        } => {
            encoder.u8(6);
            encoder.text(price_formula_key);
            encoder.text(offer_pool_key);
        }
        ServiceAction::OpenCurioShop {
            price_formula_key,
            offer_pool_key,
        } => {
            encoder.u8(7);
            encoder.text(price_formula_key);
            encoder.text(offer_pool_key);
        }
        ServiceAction::GrantTrailblazeBonus {
            offer_pool_key,
            source_event_id,
            tier,
            position,
            effect,
        } => {
            encoder.u8(8);
            encoder.text(offer_pool_key);
            encoder.u32(*source_event_id);
            encoder.u8(*tier as u8);
            encoder.u8(*position);
            match effect {
                TrailblazeBonusEffect::AddFragments { amount } => {
                    encoder.u8(0);
                    encoder.u32(*amount);
                }
                TrailblazeBonusEffect::RandomBlessing {
                    quantity,
                    minimum_rarity,
                    maximum_rarity,
                } => {
                    encoder.u8(1);
                    encoder.u8(*quantity);
                    encoder.u8(*minimum_rarity);
                    encoder.u8(*maximum_rarity);
                }
                TrailblazeBonusEffect::RandomCurio { quantity, cost } => {
                    encoder.u8(2);
                    encoder.u8(*quantity);
                    encoder.u32(cost.unwrap_or_default());
                }
            }
        }
        ServiceAction::ProfileExcluded {
            owner,
            source_event_id,
        } => {
            encoder.u8(9);
            encoder.u8(*owner as u8);
            encoder.u32(*source_event_id);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceEffectRuntimeError {
    InvalidDenominator,
    InvalidService,
    InvalidParameter,
    UnknownService,
}
