//! Gold and Gears service offers and external Adventure reward settlement.

use serde::Deserialize;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivityRngLabel, ActivityRngStreams, ActivitySlotId, ActivityValue,
};

use crate::{
    blessing_runtime::BlessingOfferEligibility,
    catalog::UniverseCatalog,
    digest::Encoder,
    gold_gears_content::{
        GoldAndGearsContentCatalog,
        types::{AdventureOutcome, Service},
    },
    id::BlessingId,
};

use super::{
    GoldAndGearsEntryError,
    api::{GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance},
    curio_types::{
        GoldAndGearsCurioCandidate, GoldAndGearsCurioCategory, GoldAndGearsCurioId,
        GoldAndGearsCurioOfferContext, GoldAndGearsCurioOfferSource,
    },
    service_adventure_types::{
        GoldAndGearsAdventureDefinition, GoldAndGearsAdventureExternalOutcome,
        GoldAndGearsAdventureMetric, GoldAndGearsAdventureRewardPlan,
        GoldAndGearsAdventureThreshold, GoldAndGearsAdventureType, GoldAndGearsServiceDefinition,
        GoldAndGearsServiceKind, GoldAndGearsServiceOfferSelector, GoldAndGearsServiceStock,
        GoldAndGearsTechniqueRule,
    },
    state_layout::{
        DEFERRED_ADVENTURE_SETTLED_BASE, DEFERRED_EFFECTS_SLOT, DEFERRED_SERVICE_USE_BASE,
        RESOURCE_COSMIC_FRAGMENTS_KEY, RUN_RESOURCES_SLOT,
    },
};

pub const GOLD_AND_GEARS_SERVICE_RUNTIME_REVISION: &str = "gold-and-gears-service-runtime-v1";
pub const GOLD_AND_GEARS_ADVENTURE_RUNTIME_REVISION: &str = "gold-and-gears-adventure-runtime-v1";
pub const GOLD_AND_GEARS_ADVENTURE_POLICY_REVISION: &str =
    "gold-and-gears-adventure-reward-policy-v1";
pub const GOLD_AND_GEARS_ADVENTURE_POLICY_ACCURACY: &str =
    "DeterministicProjectPolicyNotObservedParity";

const ADVENTURE_PURPOSE_BASE: u16 = 0x4900;
const ADVENTURE_BLESSING_PURPOSE_BASE: u16 = 0x4A00;
const ADVENTURE_CURIO_PURPOSE_BASE: u16 = 0x4B00;
const SERVICE_BLESSING_PURPOSE_BASE: u16 = 0x4C00;
const SERVICE_CURIO_PURPOSE_BASE: u16 = 0x4D00;

#[derive(Clone, Debug)]
pub(super) struct GoldAndGearsServiceAdventureRuntimeCatalog {
    services: Box<[GoldAndGearsServiceDefinition]>,
    adventures: Box<[GoldAndGearsAdventureDefinition]>,
    service_digest: [u8; 32],
    adventure_digest: [u8; 32],
}

impl GoldAndGearsServiceAdventureRuntimeCatalog {
    pub(super) fn compile(
        content: &GoldAndGearsContentCatalog,
        standard: &UniverseCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        if content.services.len() != 15 || content.adventure_outcomes.len() != 8 {
            return Err(GoldAndGearsEntryError::InvalidServiceRuntime);
        }
        let mut services = content
            .services
            .iter()
            .map(|service| compile_service(service, standard))
            .collect::<Result<Vec<_>, _>>()?;
        services.sort_by_key(|service| service.id);
        let mut adventures = content
            .adventure_outcomes
            .iter()
            .map(compile_adventure)
            .collect::<Result<Vec<_>, _>>()?;
        adventures.sort_by_key(|adventure| adventure.id);
        if services.windows(2).any(|pair| pair[0].id == pair[1].id)
            || adventures.windows(2).any(|pair| pair[0].id == pair[1].id)
        {
            return Err(GoldAndGearsEntryError::InvalidServiceRuntime);
        }
        let service_digest = service_digest(&services);
        let adventure_digest = adventure_digest(&adventures);
        Ok(Self {
            services: services.into_boxed_slice(),
            adventures: adventures.into_boxed_slice(),
            service_digest,
            adventure_digest,
        })
    }

    fn service(&self, key: &str) -> Option<&GoldAndGearsServiceDefinition> {
        self.services
            .iter()
            .find(|service| service.stable_key.as_ref() == key)
    }

    fn adventure(&self, id: u32) -> Option<(usize, &GoldAndGearsAdventureDefinition)> {
        self.adventures
            .binary_search_by_key(&id, |adventure| adventure.id)
            .ok()
            .map(|index| (index, &self.adventures[index]))
    }
}

impl GoldAndGearsRuntimeFactory {
    #[must_use]
    pub fn service_definitions(&self) -> &[GoldAndGearsServiceDefinition] {
        &self.content_runtime.service_adventure.services
    }

    #[must_use]
    pub fn adventure_definitions(&self) -> &[GoldAndGearsAdventureDefinition] {
        &self.content_runtime.service_adventure.adventures
    }

    #[must_use]
    pub fn service_runtime_digest(&self) -> [u8; 32] {
        self.content_runtime.service_adventure.service_digest
    }

    #[must_use]
    pub fn adventure_runtime_digest(&self) -> [u8; 32] {
        self.content_runtime.service_adventure.adventure_digest
    }
}

impl GoldAndGearsRuntimeInstance {
    #[must_use]
    pub fn service_definitions(&self) -> &[GoldAndGearsServiceDefinition] {
        &self.content_runtime.service_adventure.services
    }

    #[must_use]
    pub fn adventure_definitions(&self) -> &[GoldAndGearsAdventureDefinition] {
        &self.content_runtime.service_adventure.adventures
    }

    pub fn compile_service_purchase(
        &self,
        service: &str,
        selector: GoldAndGearsServiceOfferSelector,
        expected_uses: u8,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let definition = self
            .content_runtime
            .service_adventure
            .service(service)
            .ok_or_else(|| GoldAndGearsEntryError::UnknownService(service.into()))?;
        let stock = definition
            .stock
            .iter()
            .find(|stock| stock.selector == selector)
            .ok_or(GoldAndGearsEntryError::InvalidServiceOffer)?;
        if expected_uses >= stock.maximum_uses {
            return Err(GoldAndGearsEntryError::ServiceStockExhausted);
        }
        let use_key = service_use_key(definition.id, selector)?;
        let mut operations = vec![
            require_counter(DEFERRED_EFFECTS_SLOT, use_key, i64::from(expected_uses)),
            ActivityOperation::Require(ActivityCondition::LessThan(
                integer(i64::from(stock.unit_cost).saturating_sub(1)),
                counter(RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY),
            )),
        ];
        if stock.unit_cost > 0 {
            operations.push(add_counter(
                RUN_RESOURCES_SLOT,
                RESOURCE_COSMIC_FRAGMENTS_KEY,
                -i64::from(stock.unit_cost),
            ));
        }
        operations.push(add_counter(DEFERRED_EFFECTS_SLOT, use_key, 1));
        let id = 0x4C00_0000_u32
            .checked_add(
                definition
                    .id
                    .checked_mul(256)
                    .and_then(|value| value.checked_add(u32::from(selector_code(selector)) * 16))
                    .and_then(|value| value.checked_add(u32::from(expected_uses)))
                    .ok_or(GoldAndGearsEntryError::InvalidServiceRuntime)?,
            )
            .ok_or(GoldAndGearsEntryError::InvalidServiceRuntime)?;
        program(id, operations)
    }

    pub fn select_service_blessings(
        &self,
        service: &str,
        rarity: u8,
        owned: &[BlessingId],
        maximum: u16,
        rng: &mut ActivityRngStreams,
    ) -> Result<Box<[BlessingId]>, GoldAndGearsEntryError> {
        let definition = self.service_shop(service, GoldAndGearsServiceKind::BlessingShop)?;
        let stock = definition
            .stock
            .iter()
            .find(|stock| {
                stock.selector == GoldAndGearsServiceOfferSelector::BlessingRarity(rarity)
            })
            .ok_or(GoldAndGearsEntryError::InvalidServiceOffer)?;
        if maximum > u16::from(stock.maximum_uses) {
            return Err(GoldAndGearsEntryError::InvalidServiceOffer);
        }
        let candidates = blessing_candidates(self, rarity, owned)?;
        select_blessings(
            &candidates,
            maximum,
            ActivityRngLabel::Shop,
            service_purpose(SERVICE_BLESSING_PURPOSE_BASE, definition.id)?,
            rng,
        )
    }

    pub fn select_service_curios(
        &self,
        service: &str,
        owned: &[GoldAndGearsCurioId],
        maximum: u16,
        rng: &mut ActivityRngStreams,
    ) -> Result<Box<[GoldAndGearsCurioCandidate]>, GoldAndGearsEntryError> {
        let definition = self.service_shop(service, GoldAndGearsServiceKind::CurioShop)?;
        let maximum_stock = definition
            .stock
            .iter()
            .map(|stock| u16::from(stock.maximum_uses))
            .sum::<u16>();
        if maximum > maximum_stock {
            return Err(GoldAndGearsEntryError::InvalidServiceOffer);
        }
        let candidates = normal_curio_candidates(self, owned)?;
        select_curio_candidates(
            &candidates,
            maximum,
            ActivityRngLabel::Shop,
            service_purpose(SERVICE_CURIO_PURPOSE_BASE, definition.id)?,
            rng,
        )
    }

    pub fn resolve_adventure_outcome(
        &self,
        outcome: GoldAndGearsAdventureExternalOutcome,
        owned_blessings: &[BlessingId],
        owned_curios: &[GoldAndGearsCurioId],
        rng: &mut ActivityRngStreams,
    ) -> Result<GoldAndGearsAdventureRewardPlan, GoldAndGearsEntryError> {
        let (index, definition) = self
            .content_runtime
            .service_adventure
            .adventure(outcome.adventure())
            .ok_or(GoldAndGearsEntryError::UnknownAdventure(
                outcome.adventure(),
            ))?;
        if outcome.achieved_value() > definition.maximum_value {
            return Err(GoldAndGearsEntryError::InvalidAdventureOutcome);
        }
        let completed_objectives = definition
            .thresholds
            .iter()
            .filter(|threshold| outcome.achieved_value() >= threshold.minimum_value)
            .count() as u8;
        let blessing_candidates = if completed_objectives >= 1 {
            blessing_candidates(self, 2, owned_blessings)?
        } else {
            Vec::new()
        };
        let curio_candidates = if completed_objectives >= 2 {
            normal_curio_candidates(self, owned_curios)?.into_vec()
        } else {
            Vec::new()
        };
        if (completed_objectives >= 1 && blessing_candidates.is_empty())
            || (completed_objectives >= 2 && curio_candidates.is_empty())
        {
            return Err(GoldAndGearsEntryError::InvalidAdventureOutcome);
        }
        let ordinal = u16::try_from(index + 1)
            .map_err(|_| GoldAndGearsEntryError::InvalidAdventureRuntime)?;
        let (fragment_draw, blessing_draw, curio_draw) = rng.transact(|working| {
            let fragment = working
                .choose_index(
                    ActivityRngLabel::Reward,
                    adventure_purpose(ADVENTURE_PURPOSE_BASE, ordinal)?,
                    51,
                )
                .map_err(|_| GoldAndGearsEntryError::InvalidAdventureRuntime)?
                .ok_or(GoldAndGearsEntryError::InvalidAdventureRuntime)?;
            let blessing = optional_draw(
                working,
                adventure_purpose(ADVENTURE_BLESSING_PURPOSE_BASE, ordinal)?,
                blessing_candidates.len(),
            )?;
            let curio = optional_draw(
                working,
                adventure_purpose(ADVENTURE_CURIO_PURPOSE_BASE, ordinal)?,
                curio_candidates.len(),
            )?;
            Ok((fragment.value(), blessing, curio))
        })?;
        Ok(GoldAndGearsAdventureRewardPlan {
            adventure: definition.id,
            completed_objectives,
            cosmic_fragments: 100
                + u32::try_from(fragment_draw)
                    .map_err(|_| GoldAndGearsEntryError::InvalidAdventureRuntime)?,
            blessing_rarity: (completed_objectives >= 1).then_some(2),
            curio_choice: completed_objectives >= 2,
            blessing_offer: blessing_draw
                .map(|index| blessing_candidates[index])
                .filter(|_| completed_objectives >= 1),
            curio_offer: curio_draw
                .map(|index| curio_candidates[index].clone())
                .filter(|_| completed_objectives >= 2),
        })
    }

    pub fn compile_adventure_settlement(
        &self,
        plan: GoldAndGearsAdventureRewardPlan,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        self.content_runtime
            .service_adventure
            .adventure(plan.adventure)
            .ok_or(GoldAndGearsEntryError::UnknownAdventure(plan.adventure))?;
        let settled_key = DEFERRED_ADVENTURE_SETTLED_BASE + u64::from(plan.adventure);
        program(
            0x4D00_0000_u32
                .checked_add(plan.adventure)
                .ok_or(GoldAndGearsEntryError::InvalidAdventureRuntime)?,
            vec![
                require_counter(DEFERRED_EFFECTS_SLOT, settled_key, 0),
                add_counter(
                    RUN_RESOURCES_SLOT,
                    RESOURCE_COSMIC_FRAGMENTS_KEY,
                    i64::from(plan.cosmic_fragments),
                ),
                add_counter(DEFERRED_EFFECTS_SLOT, settled_key, 1),
            ],
        )
    }

    fn service_shop(
        &self,
        service: &str,
        kind: GoldAndGearsServiceKind,
    ) -> Result<&GoldAndGearsServiceDefinition, GoldAndGearsEntryError> {
        let definition = self
            .content_runtime
            .service_adventure
            .service(service)
            .ok_or_else(|| GoldAndGearsEntryError::UnknownService(service.into()))?;
        if definition.kind != kind {
            return Err(GoldAndGearsEntryError::InvalidServiceOffer);
        }
        Ok(definition)
    }
}

fn blessing_candidates(
    runtime: &GoldAndGearsRuntimeInstance,
    rarity: u8,
    owned: &[BlessingId],
) -> Result<Vec<BlessingId>, GoldAndGearsEntryError> {
    let mut owned = owned.to_vec();
    owned.sort_unstable();
    if owned.windows(2).any(|pair| pair[0] == pair[1])
        || owned
            .iter()
            .any(|id| runtime.content_runtime.blessings.definition(*id).is_none())
    {
        return Err(GoldAndGearsEntryError::InvalidBlessingInventory);
    }
    let policy = BlessingOfferEligibility::fully_unlocked(vec![rarity])
        .map_err(|_| GoldAndGearsEntryError::InvalidServiceOffer)?;
    Ok(runtime
        .content_runtime
        .blessings
        .eligible(&policy)
        .map(|definition| definition.blessing())
        .filter(|id| owned.binary_search(id).is_err())
        .collect())
}

fn normal_curio_candidates(
    runtime: &GoldAndGearsRuntimeInstance,
    owned: &[GoldAndGearsCurioId],
) -> Result<Box<[GoldAndGearsCurioCandidate]>, GoldAndGearsEntryError> {
    let keys = runtime
        .content_runtime
        .curios
        .definitions()
        .iter()
        .filter(|definition| definition.category() == GoldAndGearsCurioCategory::Normal)
        .map(|definition| definition.stable_key().into())
        .collect();
    let context = GoldAndGearsCurioOfferContext::explicit(
        GoldAndGearsCurioOfferSource::Service,
        GoldAndGearsCurioCategory::Normal,
        keys,
    )
    .ok_or(GoldAndGearsEntryError::InvalidServiceOffer)?;
    let mut candidates = runtime
        .content_runtime
        .curios
        .candidates(&context, owned)?
        .into_vec();
    candidates.sort_by_key(GoldAndGearsCurioCandidate::source_id);
    Ok(candidates.into_boxed_slice())
}

fn select_blessings(
    candidates: &[BlessingId],
    maximum: u16,
    label: ActivityRngLabel,
    purpose: u16,
    rng: &mut ActivityRngStreams,
) -> Result<Box<[BlessingId]>, GoldAndGearsEntryError> {
    if maximum == 0 || candidates.is_empty() {
        return Ok(Box::new([]));
    }
    let selected = rng.transact(|working| {
        working
            .choose_weighted_without_replacement(
                label,
                purpose,
                &vec![1; candidates.len()],
                maximum,
            )
            .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)
    })?;
    selected
        .iter()
        .map(|index| {
            candidates
                .get(
                    usize::try_from(*index)
                        .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)?,
                )
                .copied()
                .ok_or(GoldAndGearsEntryError::InvalidServiceRuntime)
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn select_curio_candidates(
    candidates: &[GoldAndGearsCurioCandidate],
    maximum: u16,
    label: ActivityRngLabel,
    purpose: u16,
    rng: &mut ActivityRngStreams,
) -> Result<Box<[GoldAndGearsCurioCandidate]>, GoldAndGearsEntryError> {
    if maximum == 0 || candidates.is_empty() {
        return Ok(Box::new([]));
    }
    let selected = rng.transact(|working| {
        working
            .choose_weighted_without_replacement(
                label,
                purpose,
                &vec![1; candidates.len()],
                maximum,
            )
            .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)
    })?;
    selected
        .iter()
        .map(|index| {
            candidates
                .get(
                    usize::try_from(*index)
                        .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)?,
                )
                .cloned()
                .ok_or(GoldAndGearsEntryError::InvalidServiceRuntime)
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn optional_draw(
    rng: &mut ActivityRngStreams,
    purpose: u16,
    candidate_count: usize,
) -> Result<Option<usize>, GoldAndGearsEntryError> {
    if candidate_count == 0 {
        return Ok(None);
    }
    rng.choose_index(
        ActivityRngLabel::Reward,
        purpose,
        u32::try_from(candidate_count)
            .map_err(|_| GoldAndGearsEntryError::InvalidAdventureRuntime)?,
    )
    .map_err(|_| GoldAndGearsEntryError::InvalidAdventureRuntime)?
    .map(|draw| {
        usize::try_from(draw.value()).map_err(|_| GoldAndGearsEntryError::InvalidAdventureRuntime)
    })
    .transpose()
}

fn adventure_purpose(base: u16, ordinal: u16) -> Result<u16, GoldAndGearsEntryError> {
    base.checked_add(ordinal)
        .ok_or(GoldAndGearsEntryError::InvalidAdventureRuntime)
}

fn service_purpose(base: u16, service: u32) -> Result<u16, GoldAndGearsEntryError> {
    base.checked_add(
        u16::try_from(service).map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)?,
    )
    .ok_or(GoldAndGearsEntryError::InvalidServiceRuntime)
}

fn compile_service(
    service: &Service,
    standard: &UniverseCatalog,
) -> Result<GoldAndGearsServiceDefinition, GoldAndGearsEntryError> {
    if !service.shared
        || standard
            .services()
            .iter()
            .all(|candidate| candidate.stable_key() != service.key.as_str())
    {
        return Err(GoldAndGearsEntryError::InvalidServiceRuntime);
    }
    let [parameters, selection, offer] = service.payloads.as_ref() else {
        return Err(GoldAndGearsEntryError::InvalidServiceRuntime);
    };
    let parameters: Vec<RawParameter> = serde_json::from_str(parameters.as_str())
        .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)?;
    let selection: RawSelectionPolicy = serde_json::from_str(selection.as_str())
        .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)?;
    validate_selection(&selection)?;
    let kind = service_kind(&service.kind)?;
    let (offer_pool, stock) = match kind {
        GoldAndGearsServiceKind::BlessingShop | GoldAndGearsServiceKind::CurioShop => {
            shop_stock(kind, offer.as_str())?
        }
        GoldAndGearsServiceKind::EnhanceBlessing => (
            None,
            [1_u8, 2, 3]
                .into_iter()
                .map(|rarity| {
                    Ok(GoldAndGearsServiceStock {
                        selector: GoldAndGearsServiceOfferSelector::BlessingRarity(rarity),
                        unit_cost: parameter_u32(&parameters, &format!("rarity_{rarity}_cost"))?,
                        maximum_uses: parameter_u8(&parameters, "max_enhancements")?,
                    })
                })
                .collect::<Result<Vec<_>, GoldAndGearsEntryError>>()?,
        ),
        GoldAndGearsServiceKind::ResetBlessing => (
            Some("gold-gears.blessing-pool.all".into()),
            parse_cost_schedule(parameter(&parameters, "source_cost_schedule")?)?,
        ),
        GoldAndGearsServiceKind::RespiteOffers => (
            None,
            [
                ("one_star_blessing_cost", 1_u8),
                ("curio_cost", 2),
                ("two_random_enhancements_cost", 3),
            ]
            .into_iter()
            .map(|(key, position)| {
                Ok(GoldAndGearsServiceStock {
                    selector: GoldAndGearsServiceOfferSelector::RespitePosition(position),
                    unit_cost: parameter_u32(&parameters, key)?,
                    maximum_uses: 1,
                })
            })
            .collect::<Result<Vec<_>, GoldAndGearsEntryError>>()?,
        ),
        GoldAndGearsServiceKind::Reviver => (
            None,
            vec![GoldAndGearsServiceStock {
                selector: GoldAndGearsServiceOfferSelector::Reviver,
                unit_cost: parameter_u32(&parameters, "cost")?,
                maximum_uses: 1,
            }],
        ),
        GoldAndGearsServiceKind::Downloader => (
            None,
            vec![GoldAndGearsServiceStock {
                selector: GoldAndGearsServiceOfferSelector::Downloader,
                unit_cost: 0,
                maximum_uses: parameter_u8(&parameters, "characters_per_device")?,
            }],
        ),
        GoldAndGearsServiceKind::Currency => {
            if parameter_u32(&parameters, "initial_amount")? != 50 {
                return Err(GoldAndGearsEntryError::InvalidServiceRuntime);
            }
            (None, Vec::new())
        }
    };
    Ok(GoldAndGearsServiceDefinition {
        id: u32::try_from(service.id)
            .ok()
            .filter(|id| *id > 0)
            .ok_or(GoldAndGearsEntryError::InvalidServiceRuntime)?,
        stable_key: service.key.as_str().into(),
        kind,
        currency: service.currency.as_ref().map(|key| key.as_str().into()),
        price_formula: service
            .price_formula
            .as_ref()
            .map(|key| key.as_str().into()),
        offer_pool,
        stock: stock.into_boxed_slice(),
    })
}

fn shop_stock(
    kind: GoldAndGearsServiceKind,
    json: &str,
) -> Result<(Option<Box<str>>, Vec<GoldAndGearsServiceStock>), GoldAndGearsEntryError> {
    let raw: RawShopOffer =
        serde_json::from_str(json).map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)?;
    let metadata_matches = match kind {
        GoldAndGearsServiceKind::BlessingShop => {
            raw.resolved_offer_pool_id.as_ref() == "gold-gears.blessing-pool.all"
                && raw.stock_modifier_id.as_deref() == Some("gold-gears.neural-network.1201")
        }
        GoldAndGearsServiceKind::CurioShop => {
            raw.resolved_offer_pool_id.as_ref() == "gold-gears.curio-pool.normal"
                && raw.stock_modifier_id.is_none()
        }
        _ => false,
    };
    if !metadata_matches {
        return Err(GoldAndGearsEntryError::InvalidServiceRuntime);
    }
    let stock = raw
        .inventory
        .iter()
        .map(|entry| {
            let selector = match kind {
                GoldAndGearsServiceKind::BlessingShop => {
                    GoldAndGearsServiceOfferSelector::BlessingRarity(parse_u8(
                        entry.rarity.as_deref(),
                    )?)
                }
                GoldAndGearsServiceKind::CurioShop => {
                    GoldAndGearsServiceOfferSelector::CurioSlot(parse_u8(entry.slot.as_deref())?)
                }
                _ => return Err(GoldAndGearsEntryError::InvalidServiceRuntime),
            };
            Ok(GoldAndGearsServiceStock {
                selector,
                unit_cost: parse_u32(Some(&entry.unit_cost))?,
                maximum_uses: entry
                    .base_stock
                    .as_deref()
                    .map_or(Ok(1), |value| parse_u8(Some(value)))?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if stock.len() != 3
        || stock
            .windows(2)
            .any(|pair| selector_code(pair[0].selector) >= selector_code(pair[1].selector))
    {
        return Err(GoldAndGearsEntryError::InvalidServiceRuntime);
    }
    Ok((Some(raw.resolved_offer_pool_id), stock))
}

fn compile_adventure(
    adventure: &AdventureOutcome,
) -> Result<GoldAndGearsAdventureDefinition, GoldAndGearsEntryError> {
    let [quality, selection, tiers] = adventure.payloads.as_ref() else {
        return Err(GoldAndGearsEntryError::InvalidAdventureRuntime);
    };
    let quality: Vec<RawQualityOverride> = serde_json::from_str(quality.as_str())
        .map_err(|_| GoldAndGearsEntryError::InvalidAdventureRuntime)?;
    let selection: RawAdventurePolicy = serde_json::from_str(selection.as_str())
        .map_err(|_| GoldAndGearsEntryError::InvalidAdventureRuntime)?;
    let tiers: Vec<RawRewardTier> = serde_json::from_str(tiers.as_str())
        .map_err(|_| GoldAndGearsEntryError::InvalidAdventureRuntime)?;
    validate_adventure_policy(&selection, &quality, &tiers)?;
    let thresholds = adventure
        .objective_thresholds
        .iter()
        .map(|value| parse_threshold(value))
        .collect::<Result<Vec<_>, GoldAndGearsEntryError>>()?;
    if !adventure.rewards_are_cumulative
        || thresholds.len() != 2
        || thresholds[0].objective != 1
        || thresholds[1].objective != 2
        || thresholds[0].minimum_value >= thresholds[1].minimum_value
    {
        return Err(GoldAndGearsEntryError::InvalidAdventureRuntime);
    }
    Ok(GoldAndGearsAdventureDefinition {
        id: adventure
            .source_id
            .parse()
            .map_err(|_| GoldAndGearsEntryError::InvalidAdventureRuntime)?,
        stable_key: adventure.key.as_str().into(),
        room: adventure.room.as_str().into(),
        adventure_type: adventure_type(&adventure.adventure_type)?,
        metric: adventure_metric(&adventure.objective_metric)?,
        thresholds: thresholds.into_boxed_slice(),
        maximum_value: adventure
            .maximum_value
            .parse()
            .map_err(|_| GoldAndGearsEntryError::InvalidAdventureRuntime)?,
        time_limit_seconds: adventure
            .time_limit_seconds
            .as_deref()
            .filter(|value| !value.is_empty())
            .map(str::parse)
            .transpose()
            .map_err(|_| GoldAndGearsEntryError::InvalidAdventureRuntime)?,
        technique_rule: technique_rule(&adventure.technique_rule)?,
    })
}

fn validate_adventure_policy(
    policy: &RawAdventurePolicy,
    quality: &[RawQualityOverride],
    tiers: &[RawRewardTier],
) -> Result<(), GoldAndGearsEntryError> {
    if policy.policy_id.as_ref() != "adventure-reward-selection-v1"
        || policy.fragment_range_selection.as_ref() != "seeded-integer-inclusive"
        || policy.candidate_order.as_ref() != "stable-source-id"
        || policy.randomness.as_ref() != "seeded-activity-stream"
        || policy.unresolved_pool_behavior.as_ref() != "FailClosed"
        || quality.len() != 1
        || quality[0].field.as_ref() != "reward_selection_policy"
        || quality[0].evidence_quality.as_ref() != "ProjectPolicy"
        || quality[0].policy_id.as_ref() != "adventure-reward-selection-v1"
        || quality[0].replacement_condition.is_empty()
        || tiers.len() != 3
        || tiers[0].operation.as_ref() != "AddCosmicFragments"
        || tiers[0].minimum_value.as_deref() != Some("100")
        || tiers[0].maximum_value.as_deref() != Some("150")
        || tiers[1].operation.as_ref() != "OfferBlessingChoice"
        || tiers[1].rarity.as_deref() != Some("2")
        || tiers[1].selected_count.as_deref() != Some("1")
        || tiers[1].offer_pool_id.as_deref() != Some("gold-gears.blessing-pool.rarity.2")
        || tiers[2].operation.as_ref() != "OfferCurioChoice"
        || tiers[2].selected_count.as_deref() != Some("1")
        || tiers[2].offer_pool_id.as_deref() != Some("gold-gears.curio-pool.normal")
        || tiers.iter().map(|tier| tier.tier).ne(1..=3)
        || tiers.iter().map(|tier| tier.minimum_objectives).ne(0..=2)
    {
        Err(GoldAndGearsEntryError::InvalidAdventureRuntime)
    } else {
        Ok(())
    }
}

fn validate_selection(policy: &RawSelectionPolicy) -> Result<(), GoldAndGearsEntryError> {
    if policy.candidate_order.as_ref() == "stable-source-id"
        && policy.randomness.as_ref() == "seeded-activity-stream"
        && policy.unresolved_pool_behavior.as_ref() == "FailClosed"
    {
        Ok(())
    } else {
        Err(GoldAndGearsEntryError::InvalidServiceRuntime)
    }
}

fn service_kind(value: &str) -> Result<GoldAndGearsServiceKind, GoldAndGearsEntryError> {
    match value {
        "BlessingShop" => Ok(GoldAndGearsServiceKind::BlessingShop),
        "CurioShop" => Ok(GoldAndGearsServiceKind::CurioShop),
        "Currency" => Ok(GoldAndGearsServiceKind::Currency),
        "Downloader" => Ok(GoldAndGearsServiceKind::Downloader),
        "EnhanceBlessing" => Ok(GoldAndGearsServiceKind::EnhanceBlessing),
        "ResetBlessing" => Ok(GoldAndGearsServiceKind::ResetBlessing),
        "RespiteOffers" => Ok(GoldAndGearsServiceKind::RespiteOffers),
        "Reviver" => Ok(GoldAndGearsServiceKind::Reviver),
        _ => Err(GoldAndGearsEntryError::InvalidServiceRuntime),
    }
}

fn adventure_type(value: &str) -> Result<GoldAndGearsAdventureType, GoldAndGearsEntryError> {
    match value {
        "RogueCaptureMonster" => Ok(GoldAndGearsAdventureType::CaptureMonster),
        "RogueDestroyProp" => Ok(GoldAndGearsAdventureType::DestroyProp),
        "RogueEscapeLaser" => Ok(GoldAndGearsAdventureType::EscapeLaser),
        "RogueTurntable" => Ok(GoldAndGearsAdventureType::Turntable),
        _ => Err(GoldAndGearsEntryError::InvalidAdventureRuntime),
    }
}

fn adventure_metric(value: &str) -> Result<GoldAndGearsAdventureMetric, GoldAndGearsEntryError> {
    match value {
        "Points" => Ok(GoldAndGearsAdventureMetric::Points),
        "DestroyedObjects" => Ok(GoldAndGearsAdventureMetric::DestroyedObjects),
        "EvadedCycles" => Ok(GoldAndGearsAdventureMetric::EvadedCycles),
        "AlignedHands" => Ok(GoldAndGearsAdventureMetric::AlignedHands),
        _ => Err(GoldAndGearsEntryError::InvalidAdventureRuntime),
    }
}

fn technique_rule(value: &str) -> Result<GoldAndGearsTechniqueRule, GoldAndGearsEntryError> {
    match value {
        "Allowed" => Ok(GoldAndGearsTechniqueRule::Allowed),
        "Disabled" => Ok(GoldAndGearsTechniqueRule::Disabled),
        "NotApplicable" => Ok(GoldAndGearsTechniqueRule::NotApplicable),
        _ => Err(GoldAndGearsEntryError::InvalidAdventureRuntime),
    }
}

fn parameter<'a>(values: &'a [RawParameter], key: &str) -> Result<&'a str, GoldAndGearsEntryError> {
    values
        .iter()
        .find(|parameter| parameter.key.as_ref() == key)
        .map(|parameter| parameter.value.as_ref())
        .ok_or(GoldAndGearsEntryError::InvalidServiceRuntime)
}

fn parse_threshold(value: &str) -> Result<GoldAndGearsAdventureThreshold, GoldAndGearsEntryError> {
    let (objective, minimum_value) = value
        .strip_prefix("{'objective': ")
        .and_then(|value| value.strip_suffix("'}"))
        .and_then(|value| value.split_once(", 'minimum_value': '"))
        .ok_or(GoldAndGearsEntryError::InvalidAdventureRuntime)?;
    Ok(GoldAndGearsAdventureThreshold {
        objective: objective
            .parse()
            .map_err(|_| GoldAndGearsEntryError::InvalidAdventureRuntime)?,
        minimum_value: minimum_value
            .parse()
            .map_err(|_| GoldAndGearsEntryError::InvalidAdventureRuntime)?,
    })
}

fn parameter_u32(values: &[RawParameter], key: &str) -> Result<u32, GoldAndGearsEntryError> {
    parameter(values, key)?
        .parse()
        .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)
}

fn parameter_u8(values: &[RawParameter], key: &str) -> Result<u8, GoldAndGearsEntryError> {
    parameter(values, key)?
        .parse()
        .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)
}

fn parse_cost_schedule(
    value: &str,
) -> Result<Vec<GoldAndGearsServiceStock>, GoldAndGearsEntryError> {
    value
        .split(',')
        .enumerate()
        .map(|(index, entry)| {
            let amount = entry
                .strip_prefix("[31:")
                .and_then(|entry| entry.strip_suffix(']'))
                .ok_or(GoldAndGearsEntryError::InvalidServiceRuntime)?
                .parse()
                .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)?;
            Ok(GoldAndGearsServiceStock {
                selector: GoldAndGearsServiceOfferSelector::UseIndex(
                    u8::try_from(index + 1)
                        .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)?,
                ),
                unit_cost: amount,
                maximum_uses: 1,
            })
        })
        .collect()
}

fn parse_u8(value: Option<&str>) -> Result<u8, GoldAndGearsEntryError> {
    value
        .ok_or(GoldAndGearsEntryError::InvalidServiceRuntime)?
        .parse()
        .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)
}

fn parse_u32(value: Option<&str>) -> Result<u32, GoldAndGearsEntryError> {
    value
        .ok_or(GoldAndGearsEntryError::InvalidServiceRuntime)?
        .parse()
        .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)
}

fn selector_code(selector: GoldAndGearsServiceOfferSelector) -> u8 {
    match selector {
        GoldAndGearsServiceOfferSelector::BlessingRarity(value)
        | GoldAndGearsServiceOfferSelector::CurioSlot(value)
        | GoldAndGearsServiceOfferSelector::UseIndex(value)
        | GoldAndGearsServiceOfferSelector::RespitePosition(value) => value,
        GoldAndGearsServiceOfferSelector::Reviver => 14,
        GoldAndGearsServiceOfferSelector::Downloader => 15,
    }
}

fn service_use_key(
    service: u32,
    selector: GoldAndGearsServiceOfferSelector,
) -> Result<u64, GoldAndGearsEntryError> {
    Ok(DEFERRED_SERVICE_USE_BASE
        + u64::from(service)
            .checked_mul(16)
            .and_then(|value| value.checked_add(u64::from(selector_code(selector))))
            .ok_or(GoldAndGearsEntryError::InvalidServiceRuntime)?)
}

fn counter(slot: u32, key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: ActivitySlotId::new(slot).expect("static slot is non-zero"),
        key,
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn require_counter(slot: u32, key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(counter(slot, key), integer(value)))
}

fn add_counter(slot: u32, key: u64, delta: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: ActivitySlotId::new(slot).expect("static slot is non-zero"),
        key,
        delta: integer(delta),
    }
}

fn program(
    id: u32,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(id).ok_or(GoldAndGearsEntryError::InvalidServiceRuntime)?,
        operations,
    )
    .map_err(|_| GoldAndGearsEntryError::InvalidServiceRuntime)
}

fn service_digest(services: &[GoldAndGearsServiceDefinition]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-gold-gears-service-runtime-v1");
    encoder.text(GOLD_AND_GEARS_SERVICE_RUNTIME_REVISION);
    encoder.u32(services.len() as u32);
    for service in services {
        encoder.u32(service.id);
        encoder.text(&service.stable_key);
        encoder.u8(service.kind as u8);
        encoder.text(service.currency.as_deref().unwrap_or(""));
        encoder.text(service.price_formula.as_deref().unwrap_or(""));
        encoder.text(service.offer_pool.as_deref().unwrap_or(""));
        encoder.u32(service.stock.len() as u32);
        for stock in &service.stock {
            encoder.u8(selector_code(stock.selector));
            encoder.u32(stock.unit_cost);
            encoder.u8(stock.maximum_uses);
        }
    }
    encoder.finish()
}

fn adventure_digest(adventures: &[GoldAndGearsAdventureDefinition]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-gold-gears-adventure-runtime-v1");
    encoder.text(GOLD_AND_GEARS_ADVENTURE_RUNTIME_REVISION);
    encoder.text(GOLD_AND_GEARS_ADVENTURE_POLICY_REVISION);
    encoder.u32(adventures.len() as u32);
    for adventure in adventures {
        encoder.u32(adventure.id);
        encoder.text(&adventure.stable_key);
        encoder.text(&adventure.room);
        encoder.u8(adventure.adventure_type as u8);
        encoder.u8(adventure.metric as u8);
        encoder.u32(adventure.thresholds.len() as u32);
        for threshold in &adventure.thresholds {
            encoder.u8(threshold.objective);
            encoder.u32(threshold.minimum_value);
        }
        encoder.u32(adventure.maximum_value);
        encoder.u32(u32::from(adventure.time_limit_seconds.unwrap_or_default()));
        encoder.u8(adventure.technique_rule as u8);
    }
    encoder.finish()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawParameter {
    key: Box<str>,
    value: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSelectionPolicy {
    candidate_order: Box<str>,
    randomness: Box<str>,
    unresolved_pool_behavior: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawShopOffer {
    inventory: Box<[RawShopStock]>,
    #[serde(default)]
    stock_modifier_id: Option<Box<str>>,
    resolved_offer_pool_id: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawShopStock {
    #[serde(default)]
    rarity: Option<Box<str>>,
    #[serde(default)]
    slot: Option<Box<str>>,
    unit_cost: Box<str>,
    #[serde(default)]
    base_stock: Option<Box<str>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAdventurePolicy {
    policy_id: Box<str>,
    fragment_range_selection: Box<str>,
    candidate_order: Box<str>,
    randomness: Box<str>,
    unresolved_pool_behavior: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRewardTier {
    tier: u8,
    minimum_objectives: u8,
    operation: Box<str>,
    #[serde(default)]
    minimum_value: Option<Box<str>>,
    #[serde(default)]
    maximum_value: Option<Box<str>>,
    #[serde(default)]
    rarity: Option<Box<str>>,
    #[serde(default)]
    selected_count: Option<Box<str>>,
    #[serde(default)]
    offer_pool_id: Option<Box<str>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawQualityOverride {
    field: Box<str>,
    evidence_quality: Box<str>,
    policy_id: Box<str>,
    replacement_condition: Box<str>,
}
