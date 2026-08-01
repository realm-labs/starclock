//! Shared services, atomic purchases and externally offered Adventure outcomes.

use std::collections::BTreeMap;

use serde::Deserialize;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivityRngLabel, ActivityRngStreams, ActivitySlotId, ActivityValue,
};

use crate::{
    catalog::UniverseCatalog,
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    id::BlessingId,
    swarm_disaster_content::interaction_access::{
        AdventureInput, CurrencyInput, InteractionRuntimeInput, ServiceInput, ServiceRuleInput,
    },
};

use super::{
    SwarmDisasterRuntimeInstance,
    content_runtime::select,
    state::{DEFERRED, RESOURCES},
};

pub const SWARM_DISASTER_SERVICE_RUNTIME_REVISION: &str = "swarm-disaster-service-runtime-v1";
pub const SWARM_DISASTER_ADVENTURE_RUNTIME_REVISION: &str = "swarm-disaster-adventure-runtime-v1";
pub const SWARM_DISASTER_SERVICE_POLICY_ACCURACY: &str =
    "DeterministicProjectPolicyNotObservedParity";

const FRAGMENTS_KEY: u64 = 1;
const SERVICE_USE_BASE: u64 = 0x5344_7100_0000_0000;
const ADVENTURE_SETTLED_BASE: u64 = 0x5344_7200_0000_0000;
const SERVICE_BLESSING_PURPOSE: u16 = 0x5501;
const SERVICE_CURIO_PURPOSE: u16 = 0x5502;

#[derive(Clone, Debug)]
pub(super) struct ServiceAdventureRuntimeCatalog {
    services: Box<[RuntimeService]>,
    adventures: Box<[RuntimeAdventure]>,
    beacons: Box<[BeaconContribution]>,
    currency: RuntimeCurrency,
    service_digest: [u8; 32],
    adventure_digest: [u8; 32],
}

#[derive(Clone, Debug)]
struct RuntimeService {
    id: u32,
    key: Box<str>,
    shared_key: Box<str>,
    kind: ServiceKind,
    parameters: Box<[Parameter]>,
    eligibility: Box<str>,
    price_policy: Box<str>,
    allowed_costs: Box<[u32]>,
    rule: RuntimeServiceRule,
}

#[derive(Clone, Debug)]
struct RuntimeServiceRule {
    id: u32,
    key: Box<str>,
    conditions: Box<str>,
    costs: Box<str>,
    operations: Box<str>,
}

#[derive(Clone, Debug)]
struct RuntimeAdventure {
    source_row_id: u32,
    id: u32,
    key: Box<str>,
    kind: Box<str>,
    parameter_group: Box<str>,
    accepted_tiers: Box<[Box<str>]>,
    offered_result: Box<str>,
    reward_program: Box<str>,
}

#[derive(Clone, Debug)]
struct RuntimeCurrency {
    id: u32,
    key: Box<str>,
    resource_key: Box<str>,
    initial_value: i64,
    cap_policy: Box<str>,
}

#[derive(Clone, Debug)]
struct BeaconContribution {
    id: u32,
    key: Box<str>,
    beacon_key: Box<str>,
    block_intro_id: u32,
    boundary: Box<str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ServiceKind {
    BlessingShop,
    CurioShop,
    Currency,
    Downloader,
    EnhanceBlessing,
    ResetBlessing,
    RespiteOffers,
    Reviver,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct Parameter {
    key: Box<str>,
    value: Box<str>,
}

impl ServiceAdventureRuntimeCatalog {
    pub(super) fn compile(
        input: &InteractionRuntimeInput,
        standard: &UniverseCatalog,
    ) -> Result<Self, UniverseCatalogLoadError> {
        if input.services.len() != 15
            || input.adventures.len() != 6
            || input.currencies.len() != 1
            || input.service_rules.len() != 19
        {
            return Err(invalid("Swarm Service/Adventure denominator drift"));
        }
        let service_keys = input
            .services
            .iter()
            .map(|row| (row.key.as_ref(), row))
            .collect::<BTreeMap<_, _>>();
        if service_keys.len() != 15 {
            return Err(reference("duplicate Swarm Service key"));
        }
        let mut service_rules = BTreeMap::new();
        let mut beacons = Vec::new();
        for rule in &input.service_rules {
            if rule.service_key.starts_with("swarm-disaster.beacon.") {
                beacons.push(compile_beacon(rule)?);
            } else if service_keys.contains_key(rule.service_key.as_ref())
                && service_rules
                    .insert(rule.service_key.as_ref(), rule)
                    .is_none()
            {
            } else {
                return Err(reference("invalid Service rule ownership"));
            }
        }
        if service_rules.len() != 15 || beacons.len() != 4 {
            return Err(reference("Service and Beacon rule closure drift"));
        }
        let mut services = input
            .services
            .iter()
            .map(|row| compile_service(row, service_rules.get(row.key.as_ref()).copied(), standard))
            .collect::<Result<Vec<_>, _>>()?;
        services.sort_unstable_by_key(|row| row.id);
        if services.windows(2).any(|pair| pair[0].id == pair[1].id)
            || services
                .iter()
                .filter(|row| row.kind == ServiceKind::BlessingShop)
                .count()
                != 5
            || services
                .iter()
                .filter(|row| row.kind == ServiceKind::CurioShop)
                .count()
                != 4
        {
            return Err(reference("Swarm Service exact-once closure drift"));
        }
        let mut adventures = input
            .adventures
            .iter()
            .map(compile_adventure)
            .collect::<Result<Vec<_>, _>>()?;
        adventures.sort_unstable_by_key(|row| row.id);
        if adventures.windows(2).any(|pair| pair[0].id == pair[1].id)
            || adventures.iter().any(|row| row.source_row_id == 0)
            || adventures
                .iter()
                .map(|row| (row.source_row_id, ()))
                .collect::<BTreeMap<_, _>>()
                .len()
                != 6
            || adventures
                .iter()
                .filter(|row| row.kind.as_ref() == "RogueCaptureMonster")
                .count()
                != 3
            || adventures
                .iter()
                .filter(|row| row.kind.as_ref() == "RogueDestroyProp")
                .count()
                != 3
        {
            return Err(reference("Swarm Adventure exact-once closure drift"));
        }
        let currency = compile_currency(&input.currencies[0])?;
        beacons.sort_unstable_by_key(|row| row.id);
        let service_digest = service_digest(&services, &beacons, &currency);
        let adventure_digest = adventure_digest(&adventures);
        Ok(Self {
            services: services.into_boxed_slice(),
            adventures: adventures.into_boxed_slice(),
            beacons: beacons.into_boxed_slice(),
            currency,
            service_digest,
            adventure_digest,
        })
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize) {
        (
            self.services.len(),
            self.adventures.len(),
            self.beacons.len(),
        )
    }

    fn service(&self, key: &str) -> Result<&RuntimeService, UniverseCatalogLoadError> {
        self.services
            .iter()
            .find(|row| row.key.as_ref() == key)
            .ok_or_else(|| reference("unknown Swarm Service"))
    }

    fn adventure(&self, key: &str) -> Result<&RuntimeAdventure, UniverseCatalogLoadError> {
        self.adventures
            .iter()
            .find(|row| row.key.as_ref() == key)
            .ok_or_else(|| reference("unknown Swarm Adventure"))
    }
}

impl SwarmDisasterRuntimeInstance {
    #[must_use]
    pub fn service_runtime_digest(&self) -> [u8; 32] {
        self.service_adventure.service_digest
    }

    #[must_use]
    pub fn adventure_runtime_digest(&self) -> [u8; 32] {
        self.service_adventure.adventure_digest
    }

    #[must_use]
    pub fn service_count(&self) -> usize {
        self.service_adventure.services.len()
    }

    #[must_use]
    pub fn adventure_count(&self) -> usize {
        self.service_adventure.adventures.len()
    }

    #[must_use]
    pub fn initial_cosmic_fragments(&self) -> i64 {
        self.service_adventure.currency.initial_value
    }

    pub fn compile_service_purchase(
        &self,
        service: &str,
        offered_unit_cost: u32,
        expected_uses: u8,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let service = self.service_adventure.service(service)?;
        if service.kind == ServiceKind::Currency {
            return Err(reference("Currency initialization is not a purchase"));
        }
        let externally_offered = matches!(
            service.kind,
            ServiceKind::BlessingShop | ServiceKind::CurioShop
        );
        if (!externally_offered && service.allowed_costs.is_empty() && offered_unit_cost != 0)
            || (!service.allowed_costs.is_empty()
                && service
                    .allowed_costs
                    .binary_search(&offered_unit_cost)
                    .is_err())
        {
            return Err(reference("Service cost is not an inherited authored price"));
        }
        let use_key = SERVICE_USE_BASE + u64::from(service.id);
        let fragments = i64::from(offered_unit_cost);
        let mut operations = vec![
            require_counter(DEFERRED, use_key, i64::from(expected_uses)),
            ActivityOperation::Require(ActivityCondition::LessThan(
                integer(fragments - 1),
                counter(RESOURCES, FRAGMENTS_KEY),
            )),
        ];
        if fragments != 0 {
            operations.push(add_counter(RESOURCES, FRAGMENTS_KEY, -fragments));
        }
        operations.push(add_counter(DEFERRED, use_key, 1));
        let id = service
            .id
            .checked_mul(256)
            .and_then(|offset| offset.checked_add(u32::from(expected_uses)))
            .and_then(|offset| 0x5350_0000_u32.checked_add(offset))
            .ok_or_else(|| invalid("Service program ID overflow"))?;
        program(id, operations)
    }

    pub fn select_service_blessings(
        &self,
        service: &str,
        rarity: u8,
        owned: &[BlessingId],
        maximum: u16,
        rng: &mut ActivityRngStreams,
    ) -> Result<Box<[BlessingId]>, UniverseCatalogLoadError> {
        let service = self.service_adventure.service(service)?;
        if service.kind != ServiceKind::BlessingShop {
            return Err(reference("Service is not a Blessing shop"));
        }
        let candidates = self
            .content_runtime
            .blessing_candidates(rarity, rarity, owned)?;
        select(
            &candidates,
            maximum,
            ActivityRngLabel::Shop,
            SERVICE_BLESSING_PURPOSE,
            rng,
        )
    }

    pub fn select_service_curios(
        &self,
        service: &str,
        owned: &[u32],
        maximum: u16,
        rng: &mut ActivityRngStreams,
    ) -> Result<Box<[u32]>, UniverseCatalogLoadError> {
        let service = self.service_adventure.service(service)?;
        if service.kind != ServiceKind::CurioShop {
            return Err(reference("Service is not a Curio shop"));
        }
        let candidates = self
            .content_runtime
            .curio_candidates(Some(super::content_runtime::CurioCategory::Normal), owned)?;
        select(
            &candidates,
            maximum,
            ActivityRngLabel::Shop,
            SERVICE_CURIO_PURPOSE,
            rng,
        )
    }

    pub fn compile_adventure_settlement(
        &self,
        adventure: &str,
        tier: &str,
        cosmic_fragments: u32,
        blessing: Option<BlessingId>,
        curio: Option<u32>,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let adventure = self.service_adventure.adventure(adventure)?;
        if adventure
            .accepted_tiers
            .iter()
            .all(|candidate| candidate.as_ref() != tier)
            || cosmic_fragments > 1_000_000_000
        {
            return Err(reference("invalid external Adventure result"));
        }
        let settled = ADVENTURE_SETTLED_BASE + u64::from(adventure.id);
        let mut operations = vec![require_counter(DEFERRED, settled, 0)];
        if cosmic_fragments != 0 {
            operations.push(add_counter(
                RESOURCES,
                FRAGMENTS_KEY,
                i64::from(cosmic_fragments),
            ));
        }
        if let Some(blessing) = blessing {
            operations.extend(
                self.content_runtime
                    .blessing_acquisition_operations(blessing)?,
            );
        }
        if let Some(curio) = curio {
            operations.extend(self.content_runtime.curio_acquisition_operations(curio)?);
        }
        operations.push(add_counter(DEFERRED, settled, 1));
        let program_id = 0x5351_0000_u32
            .checked_add(adventure.id)
            .ok_or_else(|| invalid("Adventure program ID overflow"))?;
        program(program_id, operations)
    }

    pub fn beacon_service_contribution(
        &self,
        beacon: &str,
    ) -> Result<(u32, &str), UniverseCatalogLoadError> {
        self.service_adventure
            .beacons
            .iter()
            .find(|row| row.beacon_key.as_ref() == beacon)
            .map(|row| (row.block_intro_id, row.boundary.as_ref()))
            .ok_or_else(|| reference("unknown Swarm Beacon contribution"))
    }
}

fn compile_service(
    row: &ServiceInput,
    rule: Option<&ServiceRuleInput>,
    standard: &UniverseCatalog,
) -> Result<RuntimeService, UniverseCatalogLoadError> {
    let shared = standard
        .services()
        .iter()
        .find(|candidate| candidate.stable_key() == row.shared_key.as_ref())
        .ok_or_else(|| reference("missing shared Service definition"))?;
    let parameters: Vec<Parameter> = serde_json::from_str(&row.parameters)
        .map_err(|_| reference("invalid Service parameters"))?;
    if parameters.len() != shared.parameters().len()
        || parameters
            .iter()
            .zip(shared.parameters())
            .any(|(authored, shared)| {
                authored.key.as_ref() != shared.key() || authored.value.as_ref() != shared.value()
            })
        || row
            .resource_key
            .as_deref()
            .is_some_and(|key| key != "universe.currency.cosmic-fragments")
        || !service_eligibility(&row.eligibility)
        || !price_policy(&row.price_policy, &parameters)
    {
        return Err(reference("shared Service link drift"));
    }
    let rule = rule.ok_or_else(|| reference("missing Swarm Service rule"))?;
    for value in [&rule.conditions, &rule.costs, &rule.operations] {
        serde_json::from_str::<serde_json::Value>(value)
            .map_err(|_| reference("invalid Swarm Service rule JSON"))?;
    }
    let allowed_costs = allowed_service_costs(&rule.costs)?;
    Ok(RuntimeService {
        id: row.id,
        key: row.key.clone(),
        shared_key: row.shared_key.clone(),
        kind: service_kind(&row.service_kind)?,
        parameters: parameters.into_boxed_slice(),
        eligibility: row.eligibility.clone(),
        price_policy: row.price_policy.clone(),
        allowed_costs,
        rule: RuntimeServiceRule {
            id: rule.id,
            key: rule.key.clone(),
            conditions: rule.conditions.clone(),
            costs: rule.costs.clone(),
            operations: rule.operations.clone(),
        },
    })
}

fn allowed_service_costs(value: &str) -> Result<Box<[u32]>, UniverseCatalogLoadError> {
    let rows: Vec<ServiceCost> =
        serde_json::from_str(value).map_err(|_| reference("invalid Swarm Service costs"))?;
    let mut costs = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        if usize::from(row.order) != index
            || row.resource_id.as_ref() != "universe.currency.cosmic-fragments"
            || row.key.is_empty()
        {
            return Err(reference("invalid inherited Service cost binding"));
        }
        if let Ok(cost) = row.value.parse::<u32>() {
            costs.push(cost);
            continue;
        }
        if row.key.as_ref() != "source_cost_schedule" {
            return Err(reference("invalid inherited Service cost value"));
        }
        for item in row.value.split(',') {
            let cost = item
                .trim_matches(['[', ']'])
                .split_once(':')
                .and_then(|(_, cost)| cost.parse::<u32>().ok())
                .ok_or_else(|| reference("invalid inherited Service cost schedule"))?;
            costs.push(cost);
        }
    }
    costs.sort_unstable();
    costs.dedup();
    Ok(costs.into_boxed_slice())
}

fn compile_adventure(row: &AdventureInput) -> Result<RuntimeAdventure, UniverseCatalogLoadError> {
    let offered: OfferedResult = serde_json::from_str(&row.offered_result)
        .map_err(|_| reference("invalid Adventure offered result"))?;
    let reward: RewardProgram = serde_json::from_str(&row.reward_program)
        .map_err(|_| reference("invalid Adventure reward program"))?;
    let id = row
        .key
        .rsplit('.')
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| reference("invalid Adventure room ID"))?;
    if row.tier.as_ref() != "ExternalTieredResult"
        || offered
            .accepted_values
            .iter()
            .map(Box::as_ref)
            .ne(["Tier1", "Tier2", "Tier3"])
        || !offered.cumulative
        || offered.input_simulation.as_ref() != "Excluded"
        || reward.operation.as_ref() != "ApplyValidatedExternalAdventureReward"
        || reward.payload_schema.as_ref() != "swarm-disaster.external-adventure-reward.v1"
        || reward.unresolved_payload.as_ref() != "RejectWithoutMutation"
        || reward.blessing_pool_id.as_ref() != "swarm-disaster.pool.blessings"
        || reward.curio_pool_prefix.as_ref() != "swarm-disaster.curio-pool."
    {
        return Err(reference("Adventure external-result policy drift"));
    }
    Ok(RuntimeAdventure {
        source_row_id: row.id,
        id,
        key: row.key.clone(),
        kind: row.adventure_type.clone(),
        parameter_group: row.parameter_group.clone(),
        accepted_tiers: offered.accepted_values,
        offered_result: row.offered_result.clone(),
        reward_program: row.reward_program.clone(),
    })
}

fn compile_currency(row: &CurrencyInput) -> Result<RuntimeCurrency, UniverseCatalogLoadError> {
    let policy: CurrencyPolicy = serde_json::from_str(&row.cap_policy)
        .map_err(|_| reference("invalid Swarm Currency policy"))?;
    let initial_value = row
        .initial_value
        .parse::<i64>()
        .ok()
        .filter(|value| *value == 50)
        .ok_or_else(|| reference("Swarm initial Currency drift"))?;
    if row.resource_key.as_ref() != "universe.currency.cosmic-fragments"
        || policy.maximum.as_ref() != ""
        || policy.overflow.as_ref() != "CheckedUnboundedDomainValue"
        || policy.reset_boundary.as_ref() != "RunStart"
        || policy.scope.as_ref() != "ActivityRun"
    {
        return Err(reference("Swarm Currency contract drift"));
    }
    Ok(RuntimeCurrency {
        id: row.id,
        key: row.key.clone(),
        resource_key: row.resource_key.clone(),
        initial_value,
        cap_policy: row.cap_policy.clone(),
    })
}

fn compile_beacon(rule: &ServiceRuleInput) -> Result<BeaconContribution, UniverseCatalogLoadError> {
    let conditions: Vec<BeaconCondition> = serde_json::from_str(&rule.conditions)
        .map_err(|_| reference("invalid Beacon condition"))?;
    let operations: Vec<BeaconOperation> = serde_json::from_str(&rule.operations)
        .map_err(|_| reference("invalid Beacon contribution"))?;
    if conditions.len() != 1
        || operations.len() != 1
        || conditions[0].beacon_id != rule.service_key
        || conditions[0].kind.as_ref() != "NodeHasBeaconAtAcceptedDomainEntry"
        || operations[0].operation.as_ref() != "ApplyBeaconContribution"
        || operations[0].boundary.as_ref() != "TopologyMutationResolution"
        || operations[0].order != 0
        || serde_json::from_str::<Vec<serde_json::Value>>(&rule.costs)
            .map_or(true, |costs| !costs.is_empty())
    {
        return Err(reference("Beacon contribution policy drift"));
    }
    Ok(BeaconContribution {
        id: rule.id,
        key: rule.key.clone(),
        beacon_key: rule.service_key.clone(),
        block_intro_id: operations[0]
            .block_intro_id
            .parse()
            .map_err(|_| reference("invalid Beacon block intro ID"))?,
        boundary: operations[0].boundary.clone(),
    })
}

fn service_kind(value: &str) -> Result<ServiceKind, UniverseCatalogLoadError> {
    match value {
        "BlessingShop" => Ok(ServiceKind::BlessingShop),
        "CurioShop" => Ok(ServiceKind::CurioShop),
        "Currency" => Ok(ServiceKind::Currency),
        "Downloader" => Ok(ServiceKind::Downloader),
        "EnhanceBlessing" => Ok(ServiceKind::EnhanceBlessing),
        "ResetBlessing" => Ok(ServiceKind::ResetBlessing),
        "RespiteOffers" => Ok(ServiceKind::RespiteOffers),
        "Reviver" => Ok(ServiceKind::Reviver),
        _ => Err(reference("unknown Swarm Service kind")),
    }
}

fn service_eligibility(value: &str) -> bool {
    serde_json::from_str::<Eligibility>(value).is_ok_and(|policy| {
        policy.rule.as_ref() == "ServicePresentAndAcceptedActivityCommand"
            && policy.unresolved_offer_behavior.as_ref() == "FailClosed"
    })
}

fn price_policy(value: &str, parameters: &[Parameter]) -> bool {
    serde_json::from_str::<PricePolicy>(value).is_ok_and(|policy| {
        policy.insufficient_resource.as_ref() == "RejectWithoutMutation"
            && policy.source.as_ref() == "InheritedSharedServiceParameters"
            && policy.parameter_values.as_ref() == parameters
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Eligibility {
    rule: Box<str>,
    unresolved_offer_behavior: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PricePolicy {
    insufficient_resource: Box<str>,
    parameter_values: Box<[Parameter]>,
    source: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceCost {
    order: u16,
    resource_id: Box<str>,
    key: Box<str>,
    value: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OfferedResult {
    accepted_values: Box<[Box<str>]>,
    cumulative: bool,
    input_simulation: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RewardProgram {
    blessing_pool_id: Box<str>,
    curio_pool_prefix: Box<str>,
    operation: Box<str>,
    payload_schema: Box<str>,
    unresolved_payload: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CurrencyPolicy {
    maximum: Box<str>,
    overflow: Box<str>,
    reset_boundary: Box<str>,
    scope: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BeaconCondition {
    beacon_id: Box<str>,
    kind: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BeaconOperation {
    block_intro_id: Box<str>,
    boundary: Box<str>,
    operation: Box<str>,
    order: u16,
}

fn require_counter(slot: u32, key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(counter(slot, key), integer(value)))
}

fn add_counter(slot: u32, key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: ActivitySlotId::new(slot).expect("static Swarm slot is non-zero"),
        key,
        delta: integer(value),
    }
}

fn counter(slot: u32, key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: ActivitySlotId::new(slot).expect("static Swarm slot is non-zero"),
        key,
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn program(
    id: u32,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(id).ok_or_else(|| invalid("zero Service program ID"))?,
        operations,
    )
    .map_err(|_| invalid("invalid Service Activity program"))
}

fn service_digest(
    services: &[RuntimeService],
    beacons: &[BeaconContribution],
    currency: &RuntimeCurrency,
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.service-runtime.v1");
    encoder.text(SWARM_DISASTER_SERVICE_RUNTIME_REVISION);
    encoder.text(SWARM_DISASTER_SERVICE_POLICY_ACCURACY);
    for row in services {
        encoder.u32(row.id);
        encoder.text(&row.key);
        encoder.text(&row.shared_key);
        encoder.u8(row.kind as u8);
        for parameter in &row.parameters {
            encoder.text(&parameter.key);
            encoder.text(&parameter.value);
        }
        encoder.text(&row.eligibility);
        encoder.text(&row.price_policy);
        encoder.u32(row.rule.id);
        encoder.text(&row.rule.key);
        encoder.text(&row.rule.conditions);
        encoder.text(&row.rule.costs);
        encoder.text(&row.rule.operations);
    }
    for row in beacons {
        encoder.u32(row.id);
        encoder.text(&row.key);
        encoder.text(&row.beacon_key);
        encoder.u32(row.block_intro_id);
        encoder.text(&row.boundary);
    }
    encoder.u32(currency.id);
    encoder.text(&currency.key);
    encoder.text(&currency.resource_key);
    encoder.i64(currency.initial_value);
    encoder.text(&currency.cap_policy);
    encoder.finish()
}

fn adventure_digest(adventures: &[RuntimeAdventure]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.adventure-runtime.v1");
    encoder.text(SWARM_DISASTER_ADVENTURE_RUNTIME_REVISION);
    for row in adventures {
        encoder.u32(row.source_row_id);
        encoder.u32(row.id);
        encoder.text(&row.key);
        encoder.text(&row.kind);
        encoder.text(&row.parameter_group);
        for tier in &row.accepted_tiers {
            encoder.text(tier);
        }
        encoder.text(&row.offered_result);
        encoder.text(&row.reward_program);
    }
    encoder.finish()
}

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}

fn reference(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}

#[cfg(test)]
#[path = "service_adventure_runtime_tests.rs"]
mod tests;
