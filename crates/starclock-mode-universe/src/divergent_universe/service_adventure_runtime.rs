//! Service entry fallbacks and abstract Adventure settlement.

use std::sync::Arc;

use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityStateHash, ActivityTransactionEvent, ActivityValue, GraphActivity,
    GraphActivityCommandError,
};
use starclock_data::divergent_universe_service_catalog::{
    DivergentUniverseAdventureOutcomeDefinition, DivergentUniverseAdventureOutcomeId,
    DivergentUniverseModeServiceId, DivergentUniverseServiceCatalog,
    DivergentUniverseServiceOfferId, DivergentUniverseServiceValue,
};

use super::{DivergentUniverseRuntimeFactory, state::SERVICE_RECEIPTS_SLOT};

const SETTLEMENT_PROGRAM: u32 = 22_621;
const RECEIPT_BASE: u64 = 0x2262_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseServiceAdventureAccuracy {
    ExactReleasedServiceOfferAndAdventureParameterShapes,
    VersionedProjectPolicyMissingServiceGraphAndEmptyOfferFallback,
    VersionedProjectPolicyAcceptedExternalAdventureResultReceiptOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseAdventureSettlementKind {
    RewardTier,
    ScoreThreshold,
    RoundScore,
    RoundThreshold,
    ExternalResult,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseAdventureRewardTier {
    Default,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseAdventureExternalResult {
    RewardTier(DivergentUniverseAdventureRewardTier),
    Score(u64),
    Rounds(u16),
    AcceptedOpaque,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseModeServiceRuntimeDefinition {
    id: DivergentUniverseModeServiceId,
    service_kind: Box<str>,
    graph_path: Box<str>,
    graph_resolution: Box<str>,
    fallback: Box<str>,
}

impl DivergentUniverseModeServiceRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseModeServiceId {
        &self.id
    }
    #[must_use]
    pub fn service_kind(&self) -> &str {
        &self.service_kind
    }
    #[must_use]
    pub fn graph_path(&self) -> &str {
        &self.graph_path
    }
    #[must_use]
    pub fn graph_resolution(&self) -> &str {
        &self.graph_resolution
    }
    #[must_use]
    pub fn fallback(&self) -> &str {
        &self.fallback
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseServiceOfferRuntimeDefinition {
    id: DivergentUniverseServiceOfferId,
    service_id: Box<str>,
    refresh_rule: Box<str>,
    fallback: Box<str>,
}

impl DivergentUniverseServiceOfferRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseServiceOfferId {
        &self.id
    }
    #[must_use]
    pub fn service_id(&self) -> &str {
        &self.service_id
    }
    #[must_use]
    pub fn refresh_rule(&self) -> &str {
        &self.refresh_rule
    }
    #[must_use]
    pub fn fallback(&self) -> &str {
        &self.fallback
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseAdventureRuntimeDefinition {
    id: DivergentUniverseAdventureOutcomeId,
    receipt_key: u64,
    adventure_type: Box<str>,
    room_id: Box<str>,
    parameter_group_id: Box<str>,
    settlement_kind: DivergentUniverseAdventureSettlementKind,
    thresholds: Box<[u64]>,
    parameter_rows: usize,
}

impl DivergentUniverseAdventureRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseAdventureOutcomeId {
        &self.id
    }
    #[must_use]
    pub fn adventure_type(&self) -> &str {
        &self.adventure_type
    }
    #[must_use]
    pub fn room_id(&self) -> &str {
        &self.room_id
    }
    #[must_use]
    pub fn parameter_group_id(&self) -> &str {
        &self.parameter_group_id
    }
    #[must_use]
    pub const fn settlement_kind(&self) -> DivergentUniverseAdventureSettlementKind {
        self.settlement_kind
    }
    #[must_use]
    pub fn thresholds(&self) -> &[u64] {
        &self.thresholds
    }
    #[must_use]
    pub const fn parameter_rows(&self) -> usize {
        self.parameter_rows
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseAdventureSettlementResolution {
    tier: u8,
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}

impl DivergentUniverseAdventureSettlementResolution {
    #[must_use]
    pub const fn tier(&self) -> u8 {
        self.tier
    }
    #[must_use]
    pub fn events(&self) -> &[ActivityTransactionEvent] {
        &self.events
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseServiceAdventureRuntime {
    services: Arc<[DivergentUniverseModeServiceRuntimeDefinition]>,
    offers: Arc<[DivergentUniverseServiceOfferRuntimeDefinition]>,
    adventures: Arc<[DivergentUniverseAdventureRuntimeDefinition]>,
}

impl DivergentUniverseRuntimeFactory {
    pub fn service_adventure_runtime(
        &self,
    ) -> Result<DivergentUniverseServiceAdventureRuntime, DivergentUniverseServiceAdventureError>
    {
        DivergentUniverseServiceAdventureRuntime::compile(self.bundle.service_catalog())
    }
}

impl DivergentUniverseServiceAdventureRuntime {
    fn compile(
        catalog: &DivergentUniverseServiceCatalog,
    ) -> Result<Self, DivergentUniverseServiceAdventureError> {
        let services = catalog
            .mode_services()
            .iter()
            .map(|value| {
                if value.runtime_lowered
                    || !value.choice_ids.is_empty()
                    || value.service_kind.as_ref() != "UnclassifiedMissingGraph"
                    || value.graph_resolution.as_ref() != "MissingAtPinnedRevision"
                    || value.fallback.as_ref() != "RejectWithoutMutation"
                {
                    return Err(DivergentUniverseServiceAdventureError::InvalidCatalog);
                }
                Ok(DivergentUniverseModeServiceRuntimeDefinition {
                    id: value.id.clone(),
                    service_kind: value.service_kind.clone(),
                    graph_path: value.graph_path.clone(),
                    graph_resolution: value.graph_resolution.clone(),
                    fallback: value.fallback.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let offers = catalog
            .offers()
            .iter()
            .map(|value| {
                if value.runtime_lowered
                    || !value.candidates.is_empty()
                    || !value.weights.is_empty()
                    || !matches!(
                        value.fallback.as_ref(),
                        "RejectWithoutMutation" | "LeaveWithoutMutation"
                    )
                {
                    return Err(DivergentUniverseServiceAdventureError::InvalidCatalog);
                }
                Ok(DivergentUniverseServiceOfferRuntimeDefinition {
                    id: value.id.clone(),
                    service_id: value.service_id.clone(),
                    refresh_rule: value.refresh_rule.clone(),
                    fallback: value.fallback.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let adventures = catalog
            .adventures()
            .iter()
            .enumerate()
            .map(|(index, value)| lower_adventure(index, value))
            .collect::<Result<Vec<_>, _>>()?;
        if services.len() != 23 || offers.len() != 161 || adventures.len() != 32 {
            return Err(DivergentUniverseServiceAdventureError::InvalidCatalog);
        }
        Ok(Self {
            services: services.into(),
            offers: offers.into(),
            adventures: adventures.into(),
        })
    }

    #[must_use]
    pub const fn accuracies(&self) -> [DivergentUniverseServiceAdventureAccuracy; 3] {
        [
            DivergentUniverseServiceAdventureAccuracy::ExactReleasedServiceOfferAndAdventureParameterShapes,
            DivergentUniverseServiceAdventureAccuracy::VersionedProjectPolicyMissingServiceGraphAndEmptyOfferFallback,
            DivergentUniverseServiceAdventureAccuracy::VersionedProjectPolicyAcceptedExternalAdventureResultReceiptOnly,
        ]
    }
    #[must_use]
    pub fn services(&self) -> &[DivergentUniverseModeServiceRuntimeDefinition] {
        &self.services
    }
    #[must_use]
    pub fn offers(&self) -> &[DivergentUniverseServiceOfferRuntimeDefinition] {
        &self.offers
    }
    #[must_use]
    pub fn adventures(&self) -> &[DivergentUniverseAdventureRuntimeDefinition] {
        &self.adventures
    }

    pub fn reject_missing_service_graph(
        &self,
        activity: &GraphActivity,
        expected: ActivityStateHash,
        service: &DivergentUniverseModeServiceId,
    ) -> Result<(), DivergentUniverseServiceAdventureError> {
        validate_activity(activity, expected)?;
        let _ = lookup(&self.services, |value| &value.id, service)?;
        Err(DivergentUniverseServiceAdventureError::MissingServiceGraph)
    }

    pub fn resolve_empty_offer_fallback(
        &self,
        activity: &GraphActivity,
        expected: ActivityStateHash,
        offer: &DivergentUniverseServiceOfferId,
    ) -> Result<(), DivergentUniverseServiceAdventureError> {
        validate_activity(activity, expected)?;
        let offer = lookup(&self.offers, |value| &value.id, offer)?;
        if offer.fallback.as_ref() == "LeaveWithoutMutation" {
            Ok(())
        } else {
            Err(DivergentUniverseServiceAdventureError::EmptyOfferRejected)
        }
    }

    pub fn settle_external_adventure_result(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        adventure: &DivergentUniverseAdventureOutcomeId,
        result: DivergentUniverseAdventureExternalResult,
    ) -> Result<
        DivergentUniverseAdventureSettlementResolution,
        DivergentUniverseServiceAdventureError,
    > {
        validate_activity(activity, expected)?;
        let adventure = lookup(&self.adventures, |value| &value.id, adventure)?;
        if receipt_count(activity, adventure.receipt_key)? != 0 {
            return Err(DivergentUniverseServiceAdventureError::AdventureAlreadySettled);
        }
        let tier = classify(adventure, result)?;
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(SETTLEMENT_PROGRAM)
                .ok_or(DivergentUniverseServiceAdventureError::InvalidProgram)?,
            vec![ActivityOperation::SetCounter {
                slot: SERVICE_RECEIPTS_SLOT,
                key: adventure.receipt_key,
                value: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::from(tier))),
            }],
        )
        .map_err(|_| DivergentUniverseServiceAdventureError::InvalidProgram)?;
        let events = activity.apply_boundary_program(expected, &program)?;
        Ok(DivergentUniverseAdventureSettlementResolution {
            tier,
            events,
            state_hash: activity.state_hash(),
        })
    }
}

fn lower_adventure(
    index: usize,
    value: &DivergentUniverseAdventureOutcomeDefinition,
) -> Result<DivergentUniverseAdventureRuntimeDefinition, DivergentUniverseServiceAdventureError> {
    if value.runtime_lowered
        || value.abstract_outcome.input.as_deref() != Some("AcceptedExternalAdventureResult")
        || value.abstract_outcome.ordered_operations.as_ref()
            != [
                Box::<str>::from("ValidateResult"),
                Box::<str>::from("ApplyAuthoredSettlement"),
            ]
        || value.action_gameplay.as_ref() != "Excluded"
        || value.fallback.as_ref() != "RejectWithoutMutation"
    {
        return Err(DivergentUniverseServiceAdventureError::InvalidCatalog);
    }
    let kind = settlement_kind(&value.abstract_outcome.kind)?;
    let thresholds = match kind {
        DivergentUniverseAdventureSettlementKind::ScoreThreshold
        | DivergentUniverseAdventureSettlementKind::RoundScore => {
            parameter_array(value, "ScoreRange")?
        }
        DivergentUniverseAdventureSettlementKind::RoundThreshold => {
            parameter_array(value, "RoundRange")?
        }
        DivergentUniverseAdventureSettlementKind::RewardTier => {
            if value.parameter_program.len() != 3 {
                return Err(DivergentUniverseServiceAdventureError::InvalidCatalog);
            }
            Box::new([])
        }
        DivergentUniverseAdventureSettlementKind::ExternalResult => {
            if !value.parameter_program.is_empty() {
                return Err(DivergentUniverseServiceAdventureError::InvalidCatalog);
            }
            Box::new([])
        }
    };
    let receipt_key = RECEIPT_BASE
        .checked_add(
            u64::try_from(index)
                .map_err(|_| DivergentUniverseServiceAdventureError::InvalidCatalog)?,
        )
        .and_then(|value| value.checked_add(1))
        .ok_or(DivergentUniverseServiceAdventureError::InvalidCatalog)?;
    Ok(DivergentUniverseAdventureRuntimeDefinition {
        id: value.id.clone(),
        receipt_key,
        adventure_type: value.adventure_type.clone(),
        room_id: value.room_id.clone(),
        parameter_group_id: value.parameter_group_id.clone(),
        settlement_kind: kind,
        thresholds,
        parameter_rows: value.parameter_program.len(),
    })
}

fn settlement_kind(
    value: &str,
) -> Result<DivergentUniverseAdventureSettlementKind, DivergentUniverseServiceAdventureError> {
    match value {
        "RewardTierSettlement" => Ok(DivergentUniverseAdventureSettlementKind::RewardTier),
        "ScoreThresholdSettlement" => Ok(DivergentUniverseAdventureSettlementKind::ScoreThreshold),
        "RoundScoreSettlement" => Ok(DivergentUniverseAdventureSettlementKind::RoundScore),
        "RoundThresholdSettlement" => Ok(DivergentUniverseAdventureSettlementKind::RoundThreshold),
        "ExternalAdventureResult" => Ok(DivergentUniverseAdventureSettlementKind::ExternalResult),
        _ => Err(DivergentUniverseServiceAdventureError::InvalidCatalog),
    }
}

fn parameter_array(
    value: &DivergentUniverseAdventureOutcomeDefinition,
    name: &str,
) -> Result<Box<[u64]>, DivergentUniverseServiceAdventureError> {
    let fields = &value
        .parameter_program
        .first()
        .ok_or(DivergentUniverseServiceAdventureError::InvalidCatalog)?
        .fields;
    let (_, source) = fields
        .iter()
        .find(|(key, _)| key.as_ref() == name)
        .ok_or(DivergentUniverseServiceAdventureError::InvalidCatalog)?;
    let DivergentUniverseServiceValue::Array(values) = source else {
        return Err(DivergentUniverseServiceAdventureError::InvalidCatalog);
    };
    let parsed = values
        .iter()
        .map(|value| parse_u64(value))
        .collect::<Result<Vec<_>, _>>()?;
    if parsed.len() != 3 || parsed.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(DivergentUniverseServiceAdventureError::InvalidCatalog);
    }
    Ok(parsed.into_boxed_slice())
}

fn parse_u64(value: &str) -> Result<u64, DivergentUniverseServiceAdventureError> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(DivergentUniverseServiceAdventureError::InvalidCatalog);
    }
    value
        .parse()
        .map_err(|_| DivergentUniverseServiceAdventureError::InvalidCatalog)
}

fn classify(
    adventure: &DivergentUniverseAdventureRuntimeDefinition,
    result: DivergentUniverseAdventureExternalResult,
) -> Result<u8, DivergentUniverseServiceAdventureError> {
    match (adventure.settlement_kind, result) {
        (
            DivergentUniverseAdventureSettlementKind::RewardTier,
            DivergentUniverseAdventureExternalResult::RewardTier(tier),
        ) => Ok(match tier {
            DivergentUniverseAdventureRewardTier::Default => 1,
            DivergentUniverseAdventureRewardTier::Medium => 2,
            DivergentUniverseAdventureRewardTier::High => 3,
        }),
        (
            DivergentUniverseAdventureSettlementKind::ScoreThreshold
            | DivergentUniverseAdventureSettlementKind::RoundScore,
            DivergentUniverseAdventureExternalResult::Score(value),
        ) => threshold_tier(&adventure.thresholds, value),
        (
            DivergentUniverseAdventureSettlementKind::RoundThreshold,
            DivergentUniverseAdventureExternalResult::Rounds(value),
        ) => threshold_tier(&adventure.thresholds, u64::from(value)),
        (
            DivergentUniverseAdventureSettlementKind::ExternalResult,
            DivergentUniverseAdventureExternalResult::AcceptedOpaque,
        ) => Ok(1),
        _ => Err(DivergentUniverseServiceAdventureError::ExternalResultKindMismatch),
    }
}

fn threshold_tier(
    thresholds: &[u64],
    value: u64,
) -> Result<u8, DivergentUniverseServiceAdventureError> {
    let count = thresholds
        .iter()
        .take_while(|threshold| value >= **threshold)
        .count();
    u8::try_from(count)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(DivergentUniverseServiceAdventureError::ExternalResultOutsideAuthoredRange)
}

fn validate_activity(
    activity: &GraphActivity,
    expected: ActivityStateHash,
) -> Result<(), DivergentUniverseServiceAdventureError> {
    if activity.state_hash() != expected {
        return Err(DivergentUniverseServiceAdventureError::StaleStateHash);
    }
    if activity.player_view().terminal().is_some() {
        return Err(DivergentUniverseServiceAdventureError::ActivityCompleted);
    }
    Ok(())
}

fn receipt_count(
    activity: &GraphActivity,
    key: u64,
) -> Result<i64, DivergentUniverseServiceAdventureError> {
    let view = activity.player_view();
    let value = view
        .slots()
        .iter()
        .find(|value| value.id() == SERVICE_RECEIPTS_SLOT)
        .ok_or(DivergentUniverseServiceAdventureError::InvalidState)?
        .value();
    let ActivityValue::BoundedCounterMap(values) = value else {
        return Err(DivergentUniverseServiceAdventureError::InvalidState);
    };
    Ok(values
        .binary_search_by_key(&key, |value| value.0)
        .ok()
        .map_or(0, |index| values[index].1))
}

fn lookup<'a, T, I: Ord>(
    values: &'a [T],
    id: impl Fn(&T) -> &I,
    expected: &I,
) -> Result<&'a T, DivergentUniverseServiceAdventureError> {
    values
        .binary_search_by(|value| id(value).cmp(expected))
        .ok()
        .map(|index| &values[index])
        .ok_or(DivergentUniverseServiceAdventureError::UnknownIdentity)
}

#[derive(Debug)]
pub enum DivergentUniverseServiceAdventureError {
    InvalidCatalog,
    UnknownIdentity,
    MissingServiceGraph,
    EmptyOfferRejected,
    StaleStateHash,
    ActivityCompleted,
    InvalidState,
    InvalidProgram,
    AdventureAlreadySettled,
    ExternalResultKindMismatch,
    ExternalResultOutsideAuthoredRange,
    Activity(GraphActivityCommandError),
}

impl From<GraphActivityCommandError> for DivergentUniverseServiceAdventureError {
    fn from(value: GraphActivityCommandError) -> Self {
        Self::Activity(value)
    }
}
impl core::fmt::Display for DivergentUniverseServiceAdventureError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe service/Adventure error: {self:?}"
        )
    }
}
impl std::error::Error for DivergentUniverseServiceAdventureError {}
