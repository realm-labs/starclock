//! Canonical lowering of concrete Standard Universe service selections.

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityHandlerFault, ActivityHandlerFaultKind,
    ActivityHandlerInput, ActivityHandlerOutput, ActivityInventoryId, ActivityOperation,
    ActivitySlotId, ActivityValue,
};

use crate::{
    catalog::UniverseCatalog,
    digest::Encoder,
    id::{BlessingId, CurioId, ServiceId},
    service_effect_runtime::{
        RespiteOfferKind, ServiceAction, ServiceEffectRuntimeCatalog, ServiceEffectRuntimeError,
    },
};

pub const SERVICE_INTERACTION_HANDLER_ID: u32 = 3;
pub const SERVICE_INTERACTION_RUNTIME_REVISION: &str =
    "standard-universe-service-interaction-runtime-v1";

const PAYLOAD_REVISION: u8 = 1;
const TAG_SET_FRAGMENTS: u8 = 1;
const TAG_DEBIT_FRAGMENTS: u8 = 2;
const TAG_SCHEDULED_DEBIT: u8 = 3;
const TAG_ADD_INVENTORY: u8 = 4;
const TAG_ENHANCE_INVENTORY: u8 = 5;
const TAG_DEFERRED_EFFECT: u8 = 6;
const TAG_INCREMENT_USE: u8 = 7;
const TAG_RANDOM_INVENTORY: u8 = 8;
const MAX_PAYLOAD_OPERATIONS: usize = 32;
const SERVICE_EFFECT_KEY_BASE: u64 = 1 << 62;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServicePurchaseContent {
    Blessing(BlessingId),
    Curio(CurioId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServiceInteractionSelection {
    Activate,
    RespiteBlessing,
    RespiteCurio,
    RespiteEnhance,
    EnhanceBlessing(BlessingId),
    ShopPurchase {
        content: ServicePurchaseContent,
        cost: u32,
        offer_digest: [u8; 32],
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledServiceInteraction {
    payload: Box<[u8]>,
    random_candidate_count: Option<u32>,
    required_fragments: Option<u32>,
    immediate_operations: u8,
    deferred_operations: u8,
}

impl CompiledServiceInteraction {
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    #[must_use]
    pub const fn immediate_operations(&self) -> u8 {
        self.immediate_operations
    }

    #[must_use]
    pub const fn deferred_operations(&self) -> u8 {
        self.deferred_operations
    }

    #[must_use]
    pub const fn random_candidate_count(&self) -> Option<u32> {
        self.random_candidate_count
    }

    #[must_use]
    pub const fn required_fragments(&self) -> Option<u32> {
        self.required_fragments
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceInteractionRuntimeCatalog {
    services: ServiceEffectRuntimeCatalog,
    blessing_rarities: Box<[(BlessingId, u8)]>,
    curio_ids: Box<[CurioId]>,
    cosmic_fragments: ActivitySlotId,
    service_uses: ActivitySlotId,
    service_effects: ActivitySlotId,
    blessing_inventory: ActivityInventoryId,
    curio_inventory: ActivityInventoryId,
    digest: [u8; 32],
}

#[derive(Clone, Copy)]
struct ServiceActivityBindings {
    cosmic_fragments: ActivitySlotId,
    service_uses: ActivitySlotId,
    service_effects: ActivitySlotId,
    blessing_inventory: ActivityInventoryId,
    curio_inventory: ActivityInventoryId,
}

impl ServiceInteractionRuntimeCatalog {
    pub(crate) fn compile(
        catalog: &UniverseCatalog,
        services: ServiceEffectRuntimeCatalog,
        cosmic_fragments: ActivitySlotId,
        service_uses: ActivitySlotId,
        service_effects: ActivitySlotId,
        blessing_inventory: ActivityInventoryId,
        curio_inventory: ActivityInventoryId,
    ) -> Result<Self, ServiceInteractionError> {
        let blessing_rarities = catalog
            .blessings()
            .iter()
            .map(|value| (value.id(), value.rarity()))
            .collect::<Vec<_>>();
        let blessing_ids = blessing_rarities
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        let curio_ids = catalog
            .curios()
            .iter()
            .map(|value| value.id())
            .collect::<Vec<_>>();
        if blessing_ids.len() != 162
            || curio_ids.len() != 61
            || blessing_rarities
                .windows(2)
                .any(|pair| pair[0].0 >= pair[1].0)
            || curio_ids.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(ServiceInteractionError::InvalidCatalog);
        }
        let bindings = ServiceActivityBindings {
            cosmic_fragments,
            service_uses,
            service_effects,
            blessing_inventory,
            curio_inventory,
        };
        let digest = catalog_digest(&services, &blessing_rarities, &curio_ids, bindings);
        Ok(Self {
            services,
            blessing_rarities: blessing_rarities.into_boxed_slice(),
            curio_ids: curio_ids.into_boxed_slice(),
            cosmic_fragments,
            service_uses,
            service_effects,
            blessing_inventory,
            curio_inventory,
            digest,
        })
    }

    #[must_use]
    pub const fn service_count(&self) -> usize {
        self.services.content_count()
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub(crate) const fn cosmic_fragments_slot(&self) -> ActivitySlotId {
        self.cosmic_fragments
    }

    pub fn compile_selection(
        &self,
        service: ServiceId,
        selection: &ServiceInteractionSelection,
    ) -> Result<CompiledServiceInteraction, ServiceInteractionError> {
        let effect = self.services.execute(service)?;
        let mut operations = Vec::new();
        match (effect.action(), selection) {
            (
                ServiceAction::InitializeCurrency { amount },
                ServiceInteractionSelection::Activate,
            ) => {
                operations.push(PayloadOperation::SetFragments(*amount));
            }
            (
                ServiceAction::ResetBlessingOffer { cost_schedule, .. },
                ServiceInteractionSelection::Activate,
            ) => {
                operations.push(PayloadOperation::ScheduledDebit {
                    service,
                    schedule: cost_schedule
                        .iter()
                        .map(|step| step.amount())
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                });
                operations.push(PayloadOperation::DeferredEffect(service));
                operations.push(PayloadOperation::IncrementUse(service));
            }
            (
                ServiceAction::ReviveCharacter { cost, .. },
                ServiceInteractionSelection::Activate,
            ) => {
                operations.push(PayloadOperation::DebitFragments(*cost));
                operations.push(PayloadOperation::DeferredEffect(service));
                operations.push(PayloadOperation::IncrementUse(service));
            }
            (ServiceAction::AddReserveCharacter { .. }, ServiceInteractionSelection::Activate)
            | (ServiceAction::GrantTrailblazeBonus { .. }, ServiceInteractionSelection::Activate) =>
            {
                operations.push(PayloadOperation::DeferredEffect(service));
                operations.push(PayloadOperation::IncrementUse(service));
            }
            (ServiceAction::OpenBlessingShop { .. }, ServiceInteractionSelection::Activate)
            | (ServiceAction::OpenCurioShop { .. }, ServiceInteractionSelection::Activate) => {
                operations.push(PayloadOperation::DeferredEffect(service));
                operations.push(PayloadOperation::IncrementUse(service));
            }
            (
                ServiceAction::OpenBlessingShop { .. },
                ServiceInteractionSelection::ShopPurchase {
                    content: ServicePurchaseContent::Blessing(blessing),
                    cost,
                    offer_digest,
                },
            ) => {
                validate_external_offer(*cost, *offer_digest)?;
                self.require_blessing(*blessing)?;
                operations.push(PayloadOperation::DebitFragments(*cost));
                operations.push(PayloadOperation::AddInventory {
                    inventory: self.blessing_inventory,
                    content: u64::from(blessing.get()),
                });
                operations.push(PayloadOperation::IncrementUse(service));
            }
            (
                ServiceAction::OpenCurioShop { .. },
                ServiceInteractionSelection::ShopPurchase {
                    content: ServicePurchaseContent::Curio(curio),
                    cost,
                    offer_digest,
                },
            ) => {
                validate_external_offer(*cost, *offer_digest)?;
                self.require_curio(*curio)?;
                operations.push(PayloadOperation::DebitFragments(*cost));
                operations.push(PayloadOperation::AddInventory {
                    inventory: self.curio_inventory,
                    content: u64::from(curio.get()),
                });
                operations.push(PayloadOperation::IncrementUse(service));
            }
            (
                ServiceAction::OfferRespiteChoices { offers },
                ServiceInteractionSelection::RespiteBlessing,
            ) => {
                operations.push(PayloadOperation::DebitFragments(respite_cost(
                    offers,
                    RespiteOfferKind::OneStarBlessing,
                )?));
                operations.push(PayloadOperation::RandomInventory {
                    inventory: self.blessing_inventory,
                    candidates: self
                        .blessing_rarities
                        .iter()
                        .filter(|(_, rarity)| *rarity == 1)
                        .map(|(id, _)| u64::from(id.get()))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    quantity: 1,
                    enhance_owned: false,
                });
                operations.push(PayloadOperation::IncrementUse(service));
            }
            (
                ServiceAction::OfferRespiteChoices { offers },
                ServiceInteractionSelection::RespiteCurio,
            ) => {
                operations.push(PayloadOperation::DebitFragments(respite_cost(
                    offers,
                    RespiteOfferKind::Curio,
                )?));
                operations.push(PayloadOperation::RandomInventory {
                    inventory: self.curio_inventory,
                    candidates: self
                        .curio_ids
                        .iter()
                        .map(|id| u64::from(id.get()))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    quantity: 1,
                    enhance_owned: false,
                });
                operations.push(PayloadOperation::IncrementUse(service));
            }
            (
                ServiceAction::OfferRespiteChoices { offers },
                ServiceInteractionSelection::RespiteEnhance,
            ) => {
                operations.push(PayloadOperation::DebitFragments(respite_cost(
                    offers,
                    RespiteOfferKind::EnhanceRandomBlessings,
                )?));
                operations.push(PayloadOperation::RandomInventory {
                    inventory: self.blessing_inventory,
                    candidates: self
                        .blessing_rarities
                        .iter()
                        .map(|(id, _)| u64::from(id.get()))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    quantity: 2,
                    enhance_owned: true,
                });
                operations.push(PayloadOperation::IncrementUse(service));
            }
            (
                ServiceAction::EnhanceBlessing { rarity_costs, .. },
                ServiceInteractionSelection::EnhanceBlessing(blessing),
            ) => {
                let rarity = self.require_blessing(*blessing)?;
                let cost = *rarity_costs
                    .get(usize::from(rarity.saturating_sub(1)))
                    .ok_or(ServiceInteractionError::InvalidSelection)?;
                operations.push(PayloadOperation::DebitFragments(cost));
                operations.push(PayloadOperation::EnhanceInventory {
                    inventory: self.blessing_inventory,
                    content: u64::from(blessing.get()),
                });
                operations.push(PayloadOperation::IncrementUse(service));
            }
            _ => return Err(ServiceInteractionError::InvalidSelection),
        }
        encode_program(
            operations,
            self.cosmic_fragments,
            self.service_uses,
            self.service_effects,
        )
    }

    fn require_blessing(&self, id: BlessingId) -> Result<u8, ServiceInteractionError> {
        self.blessing_rarities
            .binary_search_by_key(&id, |(candidate, _)| *candidate)
            .ok()
            .map(|index| self.blessing_rarities[index].1)
            .ok_or(ServiceInteractionError::InvalidSelection)
    }

    fn require_curio(&self, id: CurioId) -> Result<(), ServiceInteractionError> {
        self.curio_ids
            .binary_search(&id)
            .map(|_| ())
            .map_err(|_| ServiceInteractionError::InvalidSelection)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PayloadOperation {
    SetFragments(u32),
    DebitFragments(u32),
    ScheduledDebit {
        service: ServiceId,
        schedule: Box<[u32]>,
    },
    AddInventory {
        inventory: ActivityInventoryId,
        content: u64,
    },
    EnhanceInventory {
        inventory: ActivityInventoryId,
        content: u64,
    },
    DeferredEffect(ServiceId),
    IncrementUse(ServiceId),
    RandomInventory {
        inventory: ActivityInventoryId,
        candidates: Box<[u64]>,
        quantity: u8,
        enhance_owned: bool,
    },
}

impl PayloadOperation {
    const fn is_deferred(&self) -> bool {
        matches!(self, Self::DeferredEffect(_))
    }

    fn encode(&self, output: &mut Vec<u8>) -> Result<(), ServiceInteractionError> {
        match self {
            Self::SetFragments(amount) => {
                output.push(TAG_SET_FRAGMENTS);
                output.extend_from_slice(&amount.to_le_bytes());
            }
            Self::DebitFragments(amount) => {
                output.push(TAG_DEBIT_FRAGMENTS);
                output.extend_from_slice(&amount.to_le_bytes());
            }
            Self::ScheduledDebit { service, schedule } => {
                output.push(TAG_SCHEDULED_DEBIT);
                output.extend_from_slice(&service.get().to_le_bytes());
                output.push(
                    u8::try_from(schedule.len())
                        .map_err(|_| ServiceInteractionError::TooManyOperations)?,
                );
                for amount in schedule {
                    output.extend_from_slice(&amount.to_le_bytes());
                }
            }
            Self::AddInventory { inventory, content } => {
                output.push(TAG_ADD_INVENTORY);
                output.extend_from_slice(&inventory.get().to_le_bytes());
                output.extend_from_slice(&content.to_le_bytes());
            }
            Self::EnhanceInventory { inventory, content } => {
                output.push(TAG_ENHANCE_INVENTORY);
                output.extend_from_slice(&inventory.get().to_le_bytes());
                output.extend_from_slice(&content.to_le_bytes());
            }
            Self::DeferredEffect(service) => {
                output.push(TAG_DEFERRED_EFFECT);
                output.extend_from_slice(&service.get().to_le_bytes());
            }
            Self::IncrementUse(service) => {
                output.push(TAG_INCREMENT_USE);
                output.extend_from_slice(&service.get().to_le_bytes());
            }
            Self::RandomInventory {
                inventory,
                candidates,
                quantity,
                enhance_owned,
            } => {
                output.push(TAG_RANDOM_INVENTORY);
                output.extend_from_slice(&inventory.get().to_le_bytes());
                output.push(*quantity);
                output.push(u8::from(*enhance_owned));
                output.extend_from_slice(
                    &u16::try_from(candidates.len())
                        .map_err(|_| ServiceInteractionError::TooManyOperations)?
                        .to_le_bytes(),
                );
                for candidate in candidates {
                    output.extend_from_slice(&candidate.to_le_bytes());
                }
            }
        }
        Ok(())
    }
}

fn encode_program(
    operations: Vec<PayloadOperation>,
    fragments: ActivitySlotId,
    uses: ActivitySlotId,
    effects: ActivitySlotId,
) -> Result<CompiledServiceInteraction, ServiceInteractionError> {
    if operations.is_empty() || operations.len() > MAX_PAYLOAD_OPERATIONS {
        return Err(ServiceInteractionError::TooManyOperations);
    }
    let deferred_operations = u8::try_from(
        operations
            .iter()
            .filter(|operation| operation.is_deferred())
            .count(),
    )
    .map_err(|_| ServiceInteractionError::TooManyOperations)?;
    let immediate_operations = u8::try_from(operations.len())
        .map_err(|_| ServiceInteractionError::TooManyOperations)?
        .saturating_sub(deferred_operations);
    let random_candidate_count = operations.iter().find_map(|operation| match operation {
        PayloadOperation::RandomInventory { candidates, .. } => {
            u32::try_from(candidates.len()).ok()
        }
        _ => None,
    });
    let required_fragments = operations.iter().find_map(|operation| match operation {
        PayloadOperation::DebitFragments(amount) => Some(*amount),
        _ => None,
    });
    let mut payload = vec![PAYLOAD_REVISION];
    payload.extend_from_slice(&fragments.get().to_le_bytes());
    payload.extend_from_slice(&uses.get().to_le_bytes());
    payload.extend_from_slice(&effects.get().to_le_bytes());
    payload.push(
        u8::try_from(operations.len()).map_err(|_| ServiceInteractionError::TooManyOperations)?,
    );
    for operation in operations {
        operation.encode(&mut payload)?;
    }
    Ok(CompiledServiceInteraction {
        payload: payload.into_boxed_slice(),
        random_candidate_count,
        required_fragments,
        immediate_operations,
        deferred_operations,
    })
}

pub(crate) fn execute(
    input: ActivityHandlerInput<'_>,
) -> Result<ActivityHandlerOutput, ActivityHandlerFault> {
    let mut decoder = Decoder::new(input.payload());
    if decoder.u8()? != PAYLOAD_REVISION {
        return Err(invalid_payload());
    }
    let fragments = slot(decoder.u32()?)?;
    let uses = slot(decoder.u32()?)?;
    let effects = slot(decoder.u32()?)?;
    let count = usize::from(decoder.u8()?);
    if count == 0 || count > MAX_PAYLOAD_OPERATIONS {
        return Err(invalid_payload());
    }
    let mut operations = Vec::new();
    for _ in 0..count {
        match decoder.u8()? {
            TAG_SET_FRAGMENTS => operations.push(ActivityOperation::SetSlot {
                slot: fragments,
                value: integer(i64::from(decoder.u32()?)),
            }),
            TAG_DEBIT_FRAGMENTS => {
                debit(&mut operations, fragments, decoder.u32()?)?;
            }
            TAG_SCHEDULED_DEBIT => {
                let service = u64::from(decoder.u32()?);
                let schedule_count = usize::from(decoder.u8()?);
                if schedule_count == 0 {
                    return Err(invalid_payload());
                }
                let mut schedule = Vec::with_capacity(schedule_count);
                for _ in 0..schedule_count {
                    schedule.push(decoder.u32()?);
                }
                let use_count = counter(input, uses, service)?;
                let index = usize::try_from(use_count).map_err(|_| invalid_state())?;
                let amount = schedule
                    .get(index.min(schedule.len().saturating_sub(1)))
                    .copied()
                    .ok_or_else(invalid_payload)?;
                debit(&mut operations, fragments, amount)?;
            }
            TAG_ADD_INVENTORY => operations.push(ActivityOperation::AddInventory {
                inventory: inventory(decoder.u32()?)?,
                content: decoder.u64()?,
                count: integer(1),
            }),
            TAG_ENHANCE_INVENTORY => {
                let inventory = inventory(decoder.u32()?)?;
                let content = decoder.u64()?;
                operations.push(ActivityOperation::Require(ActivityCondition::Not(
                    Box::new(ActivityCondition::LessThan(
                        ActivityExpression::InventoryCount { inventory, content },
                        integer(1),
                    )),
                )));
                operations.push(ActivityOperation::AddInventory {
                    inventory,
                    content,
                    count: integer(1),
                });
            }
            TAG_DEFERRED_EFFECT => {
                let service = u64::from(decoder.u32()?);
                operations.push(ActivityOperation::AddCounter {
                    slot: effects,
                    key: SERVICE_EFFECT_KEY_BASE | service,
                    delta: integer(1),
                });
            }
            TAG_INCREMENT_USE => {
                let service = u64::from(decoder.u32()?);
                operations.push(ActivityOperation::AddCounter {
                    slot: uses,
                    key: service,
                    delta: integer(1),
                });
            }
            TAG_RANDOM_INVENTORY => {
                let inventory = inventory(decoder.u32()?)?;
                let quantity = usize::from(decoder.u8()?);
                let enhance_owned = decoder.u8()? != 0;
                let candidate_count = usize::from(decoder.u16()?);
                if quantity == 0 || candidate_count == 0 {
                    return Err(invalid_payload());
                }
                let mut candidates = Vec::with_capacity(candidate_count);
                for _ in 0..candidate_count {
                    candidates.push(decoder.u64()?);
                }
                let selected =
                    select_candidates(input, inventory, &candidates, quantity, enhance_owned)?;
                for content in selected {
                    if enhance_owned {
                        operations.push(ActivityOperation::Require(ActivityCondition::Not(
                            Box::new(ActivityCondition::LessThan(
                                ActivityExpression::InventoryCount { inventory, content },
                                integer(1),
                            )),
                        )));
                    }
                    operations.push(ActivityOperation::AddInventory {
                        inventory,
                        content,
                        count: integer(1),
                    });
                }
            }
            _ => return Err(invalid_payload()),
        }
    }
    decoder.finish()?;
    Ok(ActivityHandlerOutput::new(operations))
}

fn debit(
    operations: &mut Vec<ActivityOperation>,
    slot: ActivitySlotId,
    amount: u32,
) -> Result<(), ActivityHandlerFault> {
    if amount == 0 {
        return Err(invalid_payload());
    }
    let amount = i64::from(amount);
    operations.push(ActivityOperation::Require(ActivityCondition::Not(
        Box::new(ActivityCondition::LessThan(
            ActivityExpression::Slot(slot),
            integer(amount),
        )),
    )));
    operations.push(ActivityOperation::AddToSlot {
        slot,
        delta: integer(-amount),
    });
    Ok(())
}

fn counter(
    input: ActivityHandlerInput<'_>,
    id: ActivitySlotId,
    key: u64,
) -> Result<i64, ActivityHandlerFault> {
    input
        .view()
        .slots()
        .iter()
        .find(|value| value.id() == id)
        .and_then(|value| match value.value() {
            ActivityValue::BoundedCounterMap(values) => Some(
                values
                    .binary_search_by_key(&key, |entry| entry.0)
                    .ok()
                    .map_or(0, |index| values[index].1),
            ),
            _ => None,
        })
        .ok_or_else(invalid_state)
}

fn select_candidates(
    input: ActivityHandlerInput<'_>,
    inventory: ActivityInventoryId,
    candidates: &[u64],
    quantity: usize,
    enhance_owned: bool,
) -> Result<Vec<u64>, ActivityHandlerFault> {
    let entries = input
        .view()
        .inventories()
        .iter()
        .find(|value| value.id() == inventory)
        .ok_or_else(invalid_state)?
        .entries();
    let eligible = candidates
        .iter()
        .copied()
        .filter(|candidate| {
            let count = entries
                .binary_search_by_key(candidate, |entry| entry.0)
                .ok()
                .map_or(0, |index| entries[index].1);
            if enhance_owned {
                count == 1
            } else {
                count == 0
            }
        })
        .collect::<Vec<_>>();
    if eligible.len() < quantity {
        return Err(invalid_state());
    }
    let start = usize::try_from(input.random_index().ok_or_else(invalid_state)?)
        .map_err(|_| invalid_state())?
        % eligible.len();
    Ok((0..quantity)
        .map(|offset| eligible[(start + offset) % eligible.len()])
        .collect())
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn validate_external_offer(cost: u32, digest: [u8; 32]) -> Result<(), ServiceInteractionError> {
    if cost == 0 || digest == [0; 32] {
        return Err(ServiceInteractionError::InvalidExternalOffer);
    }
    Ok(())
}

fn respite_cost(
    offers: &[crate::service_effect_runtime::RespiteOffer],
    kind: RespiteOfferKind,
) -> Result<u32, ServiceInteractionError> {
    offers
        .iter()
        .find(|offer| offer.kind() == kind)
        .map(|offer| offer.cost())
        .ok_or(ServiceInteractionError::InvalidSelection)
}

fn catalog_digest(
    services: &ServiceEffectRuntimeCatalog,
    blessings: &[(BlessingId, u8)],
    curios: &[CurioId],
    bindings: ServiceActivityBindings,
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-standard-universe-service-interaction-v1");
    encoder.text(SERVICE_INTERACTION_RUNTIME_REVISION);
    encoder.digest(services.digest());
    encoder.u32(bindings.cosmic_fragments.get());
    encoder.u32(bindings.service_uses.get());
    encoder.u32(bindings.service_effects.get());
    encoder.u32(bindings.blessing_inventory.get());
    encoder.u32(bindings.curio_inventory.get());
    encoder.u32(blessings.len() as u32);
    for (id, rarity) in blessings {
        encoder.u32(id.get());
        encoder.u8(*rarity);
    }
    encoder.u32(curios.len() as u32);
    for id in curios {
        encoder.u32(id.get());
    }
    encoder.finish()
}

fn slot(value: u32) -> Result<ActivitySlotId, ActivityHandlerFault> {
    ActivitySlotId::new(value).ok_or_else(invalid_payload)
}

fn inventory(value: u32) -> Result<ActivityInventoryId, ActivityHandlerFault> {
    ActivityInventoryId::new(value).ok_or_else(invalid_payload)
}

struct Decoder<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Decoder<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], ActivityHandlerFault> {
        let end = self.cursor.checked_add(count).ok_or_else(invalid_payload)?;
        let value = self
            .bytes
            .get(self.cursor..end)
            .ok_or_else(invalid_payload)?;
        self.cursor = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, ActivityHandlerFault> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32, ActivityHandlerFault> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u16(&mut self) -> Result<u16, ActivityHandlerFault> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64, ActivityHandlerFault> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn finish(self) -> Result<(), ActivityHandlerFault> {
        if self.cursor == self.bytes.len() {
            Ok(())
        } else {
            Err(invalid_payload())
        }
    }
}

fn invalid_payload() -> ActivityHandlerFault {
    ActivityHandlerFault::new(ActivityHandlerFaultKind::InvalidPayload)
}

fn invalid_state() -> ActivityHandlerFault {
    ActivityHandlerFault::new(ActivityHandlerFaultKind::InvalidState)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceInteractionError {
    InvalidCatalog,
    InvalidSelection,
    InvalidExternalOffer,
    TooManyOperations,
    ServiceRuntime(ServiceEffectRuntimeError),
}

impl From<ServiceEffectRuntimeError> for ServiceInteractionError {
    fn from(value: ServiceEffectRuntimeError) -> Self {
        Self::ServiceRuntime(value)
    }
}
