//! Titan selection, Golden Blood Boon offers and permanent Titan talents.

use std::sync::Arc;

use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityStateHash, ActivityTransactionEvent, ActivityValue, GraphActivity,
    GraphActivityCommandError,
};
use starclock_data::divergent_universe_titan_catalog::{
    DivergentUniverseTitanBoonId, DivergentUniverseTitanCatalog, DivergentUniverseTitanCategory,
    DivergentUniverseTitanChoiceId, DivergentUniverseTitanContributionId,
    DivergentUniverseTitanOrderedEffect, DivergentUniverseTitanTalentId,
    DivergentUniverseTitanTypeId,
};

use super::DivergentUniverseRuntimeFactory;
use super::state::{
    TITAN_BOONS_SLOT, TITAN_TALENT_CURRENCY_SLOT, TITAN_TALENTS_SLOT, TITAN_TYPE_SLOT,
};

const ACTIVATE_TYPE_PROGRAM: u32 = 22_531;
const ACCEPT_BOON_PROGRAM: u32 = 22_532;
const CREDIT_TALENT_CURRENCY_PROGRAM: u32 = 22_533;
const UNLOCK_TALENT_PROGRAM: u32 = 22_534;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseTitanAccuracy {
    ExactReleasedTypesLevelsCostsPrerequisitesAndContributions,
    VersionedProjectPolicyAcceptedOfferTimingAndExplicitStableIdSelectionNotObservedParity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanTypeRuntime {
    id: DivergentUniverseTitanTypeId,
    key: u64,
    category: DivergentUniverseTitanCategory,
    boon_ids: Box<[DivergentUniverseTitanBoonId]>,
    talent_ids: Box<[DivergentUniverseTitanTalentId]>,
}

impl DivergentUniverseTitanTypeRuntime {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseTitanTypeId {
        &self.id
    }
    #[must_use]
    pub const fn category(&self) -> DivergentUniverseTitanCategory {
        self.category
    }
    #[must_use]
    pub fn boon_ids(&self) -> &[DivergentUniverseTitanBoonId] {
        &self.boon_ids
    }
    #[must_use]
    pub fn talent_ids(&self) -> &[DivergentUniverseTitanTalentId] {
        &self.talent_ids
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanBoonRuntime {
    id: DivergentUniverseTitanBoonId,
    key: u64,
    titan_type: DivergentUniverseTitanTypeId,
    level: u16,
    contribution: DivergentUniverseTitanContributionId,
    binding_key: Box<str>,
    maze_buff_id: Box<str>,
    parameters: Box<[Box<str>]>,
    effect_ids: Box<[Box<str>]>,
}

impl DivergentUniverseTitanBoonRuntime {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseTitanBoonId {
        &self.id
    }
    #[must_use]
    pub const fn titan_type(&self) -> &DivergentUniverseTitanTypeId {
        &self.titan_type
    }
    #[must_use]
    pub const fn level(&self) -> u16 {
        self.level
    }
    #[must_use]
    pub const fn contribution(&self) -> &DivergentUniverseTitanContributionId {
        &self.contribution
    }
    #[must_use]
    pub fn binding_key(&self) -> &str {
        &self.binding_key
    }
    #[must_use]
    pub fn maze_buff_id(&self) -> &str {
        &self.maze_buff_id
    }
    #[must_use]
    pub fn parameters(&self) -> &[Box<str>] {
        &self.parameters
    }
    #[must_use]
    pub fn effect_ids(&self) -> &[Box<str>] {
        &self.effect_ids
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanTalentRuntime {
    id: DivergentUniverseTitanTalentId,
    key: u64,
    titan_type: DivergentUniverseTitanTypeId,
    level: u16,
    predecessor: Option<DivergentUniverseTitanTalentId>,
    contribution: DivergentUniverseTitanContributionId,
    cost_item_id: Box<str>,
    cost: u32,
    operation: Box<str>,
    metric: Box<str>,
    value: Option<Box<str>>,
    condition: Box<str>,
}

impl DivergentUniverseTitanTalentRuntime {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseTitanTalentId {
        &self.id
    }
    #[must_use]
    pub const fn titan_type(&self) -> &DivergentUniverseTitanTypeId {
        &self.titan_type
    }
    #[must_use]
    pub const fn level(&self) -> u16 {
        self.level
    }
    #[must_use]
    pub const fn predecessor(&self) -> Option<&DivergentUniverseTitanTalentId> {
        self.predecessor.as_ref()
    }
    #[must_use]
    pub const fn contribution(&self) -> &DivergentUniverseTitanContributionId {
        &self.contribution
    }
    #[must_use]
    pub fn cost_item_id(&self) -> &str {
        &self.cost_item_id
    }
    #[must_use]
    pub const fn cost(&self) -> u32 {
        self.cost
    }
    #[must_use]
    pub fn operation(&self) -> &str {
        &self.operation
    }
    #[must_use]
    pub fn metric(&self) -> &str {
        &self.metric
    }
    #[must_use]
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
    #[must_use]
    pub fn condition(&self) -> &str {
        &self.condition
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseGoldenBloodOffer {
    choice: DivergentUniverseTitanChoiceId,
    titan_type: DivergentUniverseTitanTypeId,
    level: u16,
    candidates: Box<[DivergentUniverseTitanBoonId]>,
    eligibility: Box<str>,
    fallback: Box<str>,
}

impl DivergentUniverseGoldenBloodOffer {
    #[must_use]
    pub const fn choice(&self) -> &DivergentUniverseTitanChoiceId {
        &self.choice
    }
    #[must_use]
    pub const fn titan_type(&self) -> &DivergentUniverseTitanTypeId {
        &self.titan_type
    }
    #[must_use]
    pub const fn level(&self) -> u16 {
        self.level
    }
    #[must_use]
    pub fn candidates(&self) -> &[DivergentUniverseTitanBoonId] {
        &self.candidates
    }
    #[must_use]
    pub fn eligibility(&self) -> &str {
        &self.eligibility
    }
    #[must_use]
    pub fn fallback(&self) -> &str {
        &self.fallback
    }
    #[must_use]
    pub const fn accuracy(&self) -> DivergentUniverseTitanAccuracy {
        DivergentUniverseTitanAccuracy::VersionedProjectPolicyAcceptedOfferTimingAndExplicitStableIdSelectionNotObservedParity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseTitanContributionScope {
    Activity,
    Battle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanEffect {
    operation: Box<str>,
    binding_key: Option<Box<str>>,
    condition: Option<Box<str>>,
    metric: Option<Box<str>>,
    maze_buff_id: Option<Box<str>>,
    parameters: Box<[Box<str>]>,
    extra_effect_ids: Box<[Box<str>]>,
    value: Option<Box<str>>,
}

impl DivergentUniverseTitanEffect {
    #[must_use]
    pub fn operation(&self) -> &str {
        &self.operation
    }
    #[must_use]
    pub fn binding_key(&self) -> Option<&str> {
        self.binding_key.as_deref()
    }
    #[must_use]
    pub fn condition(&self) -> Option<&str> {
        self.condition.as_deref()
    }
    #[must_use]
    pub fn metric(&self) -> Option<&str> {
        self.metric.as_deref()
    }
    #[must_use]
    pub fn maze_buff_id(&self) -> Option<&str> {
        self.maze_buff_id.as_deref()
    }
    #[must_use]
    pub fn parameters(&self) -> &[Box<str>] {
        &self.parameters
    }
    #[must_use]
    pub fn extra_effect_ids(&self) -> &[Box<str>] {
        &self.extra_effect_ids
    }
    #[must_use]
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanContribution {
    id: DivergentUniverseTitanContributionId,
    scope: DivergentUniverseTitanContributionScope,
    activation: Box<str>,
    teardown: Box<str>,
    effects: Box<[DivergentUniverseTitanEffect]>,
}

impl DivergentUniverseTitanContribution {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseTitanContributionId {
        &self.id
    }
    #[must_use]
    pub const fn scope(&self) -> DivergentUniverseTitanContributionScope {
        self.scope
    }
    #[must_use]
    pub fn activation(&self) -> &str {
        &self.activation
    }
    #[must_use]
    pub fn teardown(&self) -> &str {
        &self.teardown
    }
    #[must_use]
    pub fn effects(&self) -> &[DivergentUniverseTitanEffect] {
        &self.effects
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DivergentUniverseTitanSnapshotDigest([u8; 32]);

impl DivergentUniverseTitanSnapshotDigest {
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanSnapshot {
    source_state_hash: ActivityStateHash,
    selected_type: Option<DivergentUniverseTitanTypeId>,
    boons: Box<[DivergentUniverseTitanBoonId]>,
    talents: Box<[DivergentUniverseTitanTalentId]>,
    contributions: Box<[DivergentUniverseTitanContribution]>,
    digest: DivergentUniverseTitanSnapshotDigest,
}

impl DivergentUniverseTitanSnapshot {
    #[must_use]
    pub const fn source_state_hash(&self) -> ActivityStateHash {
        self.source_state_hash
    }
    #[must_use]
    pub const fn selected_type(&self) -> Option<&DivergentUniverseTitanTypeId> {
        self.selected_type.as_ref()
    }
    #[must_use]
    pub fn boons(&self) -> &[DivergentUniverseTitanBoonId] {
        &self.boons
    }
    #[must_use]
    pub fn talents(&self) -> &[DivergentUniverseTitanTalentId] {
        &self.talents
    }
    #[must_use]
    pub fn contributions(&self) -> &[DivergentUniverseTitanContribution] {
        &self.contributions
    }
    #[must_use]
    pub const fn digest(&self) -> DivergentUniverseTitanSnapshotDigest {
        self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanCommandResolution {
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}

impl DivergentUniverseTitanCommandResolution {
    #[must_use]
    pub fn events(&self) -> &[ActivityTransactionEvent] {
        &self.events
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTitanRuntime {
    types: Arc<[DivergentUniverseTitanTypeRuntime]>,
    boons: Arc<[DivergentUniverseTitanBoonRuntime]>,
    talents: Arc<[DivergentUniverseTitanTalentRuntime]>,
    choices: Arc<[DivergentUniverseGoldenBloodOffer]>,
    contributions: Arc<[DivergentUniverseTitanContribution]>,
    component_digest: [u8; 32],
}

impl DivergentUniverseRuntimeFactory {
    pub fn titan_runtime(
        &self,
    ) -> Result<DivergentUniverseTitanRuntime, DivergentUniverseTitanRuntimeError> {
        DivergentUniverseTitanRuntime::compile(
            self.bundle.titan_catalog(),
            self.bundle.identity().component_digest().bytes(),
        )
    }
}

impl DivergentUniverseTitanRuntime {
    fn compile(
        catalog: &DivergentUniverseTitanCatalog,
        component_digest: [u8; 32],
    ) -> Result<Self, DivergentUniverseTitanRuntimeError> {
        let types = catalog
            .types()
            .iter()
            .enumerate()
            .map(|(index, value)| {
                Ok(DivergentUniverseTitanTypeRuntime {
                    id: value.id.clone(),
                    key: ordinal(index)?,
                    category: value.category,
                    boon_ids: value.boons.clone(),
                    talent_ids: value.talents.clone(),
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseTitanRuntimeError>>()?;
        let boons = catalog
            .boons()
            .iter()
            .enumerate()
            .map(|(index, value)| {
                Ok(DivergentUniverseTitanBoonRuntime {
                    id: value.id.clone(),
                    key: ordinal(index)?,
                    titan_type: value.titan_type.clone(),
                    level: value.level,
                    contribution: value.contribution.clone(),
                    binding_key: value.binding_key.clone(),
                    maze_buff_id: value.maze_buff_id.clone(),
                    parameters: value.parameters.clone(),
                    effect_ids: value.effect_ids.clone(),
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseTitanRuntimeError>>()?;
        let talents = catalog
            .talents()
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let amount = value.cost[0]
                    .amount
                    .parse::<u32>()
                    .ok()
                    .filter(|amount| *amount > 0)
                    .ok_or(DivergentUniverseTitanRuntimeError::InvalidCatalog)?;
                Ok(DivergentUniverseTitanTalentRuntime {
                    id: value.id.clone(),
                    key: ordinal(index)?,
                    titan_type: value.titan_type.clone(),
                    level: value.level,
                    predecessor: value.predecessor.clone(),
                    contribution: value.contribution.clone(),
                    cost_item_id: value.cost[0].item_id.clone(),
                    cost: amount,
                    operation: value.effect_program.operation.clone(),
                    metric: value.effect_program.metric.clone(),
                    value: value.effect_program.value.clone(),
                    condition: value.effect_program.condition.clone(),
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseTitanRuntimeError>>()?;
        let choices = catalog
            .choices()
            .iter()
            .map(|value| DivergentUniverseGoldenBloodOffer {
                choice: value.id.clone(),
                titan_type: value.titan_type.clone(),
                level: value.level,
                candidates: value.candidates.clone(),
                eligibility: value.eligibility.clone(),
                fallback: value.fallback.clone(),
            })
            .collect::<Vec<_>>();
        let contributions = catalog
            .contributions()
            .iter()
            .map(|value| {
                let scope = match value.scope.as_ref() {
                    "Activity" => DivergentUniverseTitanContributionScope::Activity,
                    "Battle" => DivergentUniverseTitanContributionScope::Battle,
                    _ => return Err(DivergentUniverseTitanRuntimeError::InvalidCatalog),
                };
                Ok(DivergentUniverseTitanContribution {
                    id: value.id.clone(),
                    scope,
                    activation: value.activation.clone(),
                    teardown: value.teardown.clone(),
                    effects: value
                        .ordered_effects
                        .iter()
                        .map(effect)
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if types.len() != 12
            || boons.len() != 84
            || talents.len() != 36
            || choices.len() != 36
            || contributions.len() != 120
            || types.windows(2).any(|pair| pair[0].id >= pair[1].id)
            || boons.windows(2).any(|pair| pair[0].id >= pair[1].id)
            || talents.windows(2).any(|pair| pair[0].id >= pair[1].id)
            || choices
                .windows(2)
                .any(|pair| pair[0].choice >= pair[1].choice)
            || contributions
                .windows(2)
                .any(|pair| pair[0].id >= pair[1].id)
            || choices.iter().any(|choice| {
                choice.eligibility.as_ref()
                    != match choice.level {
                        1 => "TitanTypeActivated",
                        2 => "PriorBoonLevel1Accepted",
                        3 => "PriorBoonLevel2Accepted",
                        _ => "Invalid",
                    }
                    || choice.fallback.as_ref() != "RejectWithoutMutation"
                    || choice.candidates.len() != if choice.level == 1 { 1 } else { 3 }
            })
            || talents
                .iter()
                .any(|talent| talent.cost_item_id.as_ref() != "281020")
        {
            return Err(DivergentUniverseTitanRuntimeError::InvalidCatalog);
        }
        Ok(Self {
            types: types.into(),
            boons: boons.into(),
            talents: talents.into(),
            choices: choices.into(),
            contributions: contributions.into(),
            component_digest,
        })
    }

    #[must_use]
    pub const fn accuracy(&self) -> [DivergentUniverseTitanAccuracy; 2] {
        [
            DivergentUniverseTitanAccuracy::ExactReleasedTypesLevelsCostsPrerequisitesAndContributions,
            DivergentUniverseTitanAccuracy::VersionedProjectPolicyAcceptedOfferTimingAndExplicitStableIdSelectionNotObservedParity,
        ]
    }
    #[must_use]
    pub fn types(&self) -> &[DivergentUniverseTitanTypeRuntime] {
        &self.types
    }
    #[must_use]
    pub fn boons(&self) -> &[DivergentUniverseTitanBoonRuntime] {
        &self.boons
    }
    #[must_use]
    pub fn talents(&self) -> &[DivergentUniverseTitanTalentRuntime] {
        &self.talents
    }
    #[must_use]
    pub fn choices(&self) -> &[DivergentUniverseGoldenBloodOffer] {
        &self.choices
    }
    #[must_use]
    pub fn all_contributions(&self) -> &[DivergentUniverseTitanContribution] {
        &self.contributions
    }

    pub fn activate_type_accepted(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        titan_type: &DivergentUniverseTitanTypeId,
    ) -> Result<DivergentUniverseTitanCommandResolution, DivergentUniverseTitanRuntimeError> {
        validate_hash(activity, expected_state_hash)?;
        let definition = self.titan_type(titan_type)?;
        if selected_type_key(activity)?.is_some() {
            return Err(DivergentUniverseTitanRuntimeError::TypeAlreadyActivated);
        }
        self.apply(
            activity,
            expected_state_hash,
            ACTIVATE_TYPE_PROGRAM,
            vec![ActivityOperation::SetSlot {
                slot: TITAN_TYPE_SLOT,
                value: literal(ActivityValue::OptionalId(Some(definition.key))),
            }],
        )
    }

    pub fn next_offer(
        &self,
        activity: &GraphActivity,
    ) -> Result<DivergentUniverseGoldenBloodOffer, DivergentUniverseTitanRuntimeError> {
        let selected = selected_type_key(activity)?
            .ok_or(DivergentUniverseTitanRuntimeError::TitanTypeNotActivated)?;
        let titan_type = self
            .types
            .iter()
            .find(|value| value.key == selected)
            .ok_or(DivergentUniverseTitanRuntimeError::InvalidState)?;
        let owned = ordered_set(activity, TITAN_BOONS_SLOT)?;
        self.validate_owned_boons(titan_type, &owned)?;
        let next_level = u16::try_from(owned.len() + 1)
            .map_err(|_| DivergentUniverseTitanRuntimeError::InvalidState)?;
        self.choices
            .iter()
            .find(|choice| choice.titan_type == titan_type.id && choice.level == next_level)
            .cloned()
            .ok_or(DivergentUniverseTitanRuntimeError::NoFurtherBoonLevel)
    }

    pub fn accept_boon_accepted(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        boon: &DivergentUniverseTitanBoonId,
    ) -> Result<DivergentUniverseTitanCommandResolution, DivergentUniverseTitanRuntimeError> {
        validate_hash(activity, expected_state_hash)?;
        let offer = self.next_offer(activity)?;
        if !offer.candidates.contains(boon) {
            return Err(DivergentUniverseTitanRuntimeError::BoonNotOffered);
        }
        let definition = self.boon(boon)?;
        self.apply(
            activity,
            expected_state_hash,
            ACCEPT_BOON_PROGRAM,
            vec![ActivityOperation::InsertOrderedId {
                slot: TITAN_BOONS_SLOT,
                id: definition.key,
            }],
        )
    }

    pub fn credit_talent_currency_accepted(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        amount: u32,
    ) -> Result<DivergentUniverseTitanCommandResolution, DivergentUniverseTitanRuntimeError> {
        validate_hash(activity, expected_state_hash)?;
        if amount == 0 {
            return Err(DivergentUniverseTitanRuntimeError::InvalidAmount);
        }
        talent_currency(activity)?
            .checked_add(i64::from(amount))
            .ok_or(DivergentUniverseTitanRuntimeError::CurrencyOutOfRange)?;
        self.apply(
            activity,
            expected_state_hash,
            CREDIT_TALENT_CURRENCY_PROGRAM,
            vec![ActivityOperation::AddToSlot {
                slot: TITAN_TALENT_CURRENCY_SLOT,
                delta: literal(ActivityValue::BoundedInteger(i64::from(amount))),
            }],
        )
    }

    pub fn unlock_talent_accepted(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        talent: &DivergentUniverseTitanTalentId,
    ) -> Result<DivergentUniverseTitanCommandResolution, DivergentUniverseTitanRuntimeError> {
        validate_hash(activity, expected_state_hash)?;
        let definition = self.talent(talent)?;
        let owned = ordered_set(activity, TITAN_TALENTS_SLOT)?;
        self.validate_owned_talents(&owned)?;
        if owned.binary_search(&definition.key).is_ok() {
            return Err(DivergentUniverseTitanRuntimeError::TalentAlreadyUnlocked);
        }
        if let Some(predecessor) = definition.predecessor.as_ref() {
            let predecessor_key = self.talent(predecessor)?.key;
            if owned.binary_search(&predecessor_key).is_err() {
                return Err(DivergentUniverseTitanRuntimeError::MissingTalentPrerequisite);
            }
        }
        if talent_currency(activity)? < i64::from(definition.cost) {
            return Err(DivergentUniverseTitanRuntimeError::InsufficientTalentCurrency);
        }
        self.apply(
            activity,
            expected_state_hash,
            UNLOCK_TALENT_PROGRAM,
            vec![
                ActivityOperation::AddToSlot {
                    slot: TITAN_TALENT_CURRENCY_SLOT,
                    delta: literal(ActivityValue::BoundedInteger(-i64::from(definition.cost))),
                },
                ActivityOperation::InsertOrderedId {
                    slot: TITAN_TALENTS_SLOT,
                    id: definition.key,
                },
            ],
        )
    }

    pub fn snapshot(
        &self,
        activity: &GraphActivity,
    ) -> Result<DivergentUniverseTitanSnapshot, DivergentUniverseTitanRuntimeError> {
        let selected_key = selected_type_key(activity)?;
        let selected = selected_key
            .map(|key| {
                self.types
                    .iter()
                    .find(|value| value.key == key)
                    .ok_or(DivergentUniverseTitanRuntimeError::InvalidState)
            })
            .transpose()?;
        let boon_keys = ordered_set(activity, TITAN_BOONS_SLOT)?;
        if let Some(titan_type) = selected {
            self.validate_owned_boons(titan_type, &boon_keys)?;
        } else if !boon_keys.is_empty() {
            return Err(DivergentUniverseTitanRuntimeError::InvalidState);
        }
        let talent_keys = ordered_set(activity, TITAN_TALENTS_SLOT)?;
        self.validate_owned_talents(&talent_keys)?;
        let boons = boon_keys
            .iter()
            .map(|key| self.boon_by_key(*key).map(|value| value.id.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        let talents = talent_keys
            .iter()
            .map(|key| self.talent_by_key(*key).map(|value| value.id.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        let mut contribution_ids = boons
            .iter()
            .map(|id| self.boon(id).map(|value| value.contribution.clone()))
            .chain(
                talents
                    .iter()
                    .map(|id| self.talent(id).map(|value| value.contribution.clone())),
            )
            .collect::<Result<Vec<_>, _>>()?;
        contribution_ids.sort_unstable();
        let contributions = contribution_ids
            .iter()
            .map(|id| self.contribution(id).cloned())
            .collect::<Result<Vec<_>, _>>()?;
        let state_hash = activity.state_hash();
        let selected_id = selected.map(|value| value.id.clone());
        let digest = snapshot_digest(
            state_hash,
            self.component_digest,
            selected_id.as_ref(),
            &boons,
            &talents,
            &contributions,
        );
        Ok(DivergentUniverseTitanSnapshot {
            source_state_hash: state_hash,
            selected_type: selected_id,
            boons: boons.into_boxed_slice(),
            talents: talents.into_boxed_slice(),
            contributions: contributions.into_boxed_slice(),
            digest: DivergentUniverseTitanSnapshotDigest(digest),
        })
    }

    fn titan_type(
        &self,
        id: &DivergentUniverseTitanTypeId,
    ) -> Result<&DivergentUniverseTitanTypeRuntime, DivergentUniverseTitanRuntimeError> {
        lookup(&self.types, |value| &value.id, id)
    }

    fn boon(
        &self,
        id: &DivergentUniverseTitanBoonId,
    ) -> Result<&DivergentUniverseTitanBoonRuntime, DivergentUniverseTitanRuntimeError> {
        lookup(&self.boons, |value| &value.id, id)
    }

    fn talent(
        &self,
        id: &DivergentUniverseTitanTalentId,
    ) -> Result<&DivergentUniverseTitanTalentRuntime, DivergentUniverseTitanRuntimeError> {
        lookup(&self.talents, |value| &value.id, id)
    }

    fn contribution(
        &self,
        id: &DivergentUniverseTitanContributionId,
    ) -> Result<&DivergentUniverseTitanContribution, DivergentUniverseTitanRuntimeError> {
        lookup(&self.contributions, |value| &value.id, id)
    }

    fn boon_by_key(
        &self,
        key: u64,
    ) -> Result<&DivergentUniverseTitanBoonRuntime, DivergentUniverseTitanRuntimeError> {
        self.boons
            .iter()
            .find(|value| value.key == key)
            .ok_or(DivergentUniverseTitanRuntimeError::InvalidState)
    }

    fn talent_by_key(
        &self,
        key: u64,
    ) -> Result<&DivergentUniverseTitanTalentRuntime, DivergentUniverseTitanRuntimeError> {
        self.talents
            .iter()
            .find(|value| value.key == key)
            .ok_or(DivergentUniverseTitanRuntimeError::InvalidState)
    }

    fn validate_owned_boons(
        &self,
        titan_type: &DivergentUniverseTitanTypeRuntime,
        keys: &[u64],
    ) -> Result<(), DivergentUniverseTitanRuntimeError> {
        if keys.len() > 3 {
            return Err(DivergentUniverseTitanRuntimeError::InvalidState);
        }
        for (index, key) in keys.iter().enumerate() {
            let boon = self.boon_by_key(*key)?;
            if boon.titan_type != titan_type.id
                || usize::from(boon.level) != index + 1
                || !titan_type.boon_ids.contains(&boon.id)
            {
                return Err(DivergentUniverseTitanRuntimeError::InvalidState);
            }
        }
        Ok(())
    }

    fn validate_owned_talents(
        &self,
        keys: &[u64],
    ) -> Result<(), DivergentUniverseTitanRuntimeError> {
        if keys.len() > 36 {
            return Err(DivergentUniverseTitanRuntimeError::InvalidState);
        }
        for key in keys {
            let talent = self.talent_by_key(*key)?;
            if let Some(predecessor) = talent.predecessor.as_ref() {
                let predecessor_key = self.talent(predecessor)?.key;
                if keys.binary_search(&predecessor_key).is_err() {
                    return Err(DivergentUniverseTitanRuntimeError::InvalidState);
                }
            }
        }
        Ok(())
    }

    fn apply(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        raw_program: u32,
        operations: Vec<ActivityOperation>,
    ) -> Result<DivergentUniverseTitanCommandResolution, DivergentUniverseTitanRuntimeError> {
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(raw_program).expect("static Titan program ID is non-zero"),
            operations,
        )
        .map_err(|_| DivergentUniverseTitanRuntimeError::InvalidProgram)?;
        let events = activity
            .apply_boundary_program(expected_state_hash, &program)
            .map_err(DivergentUniverseTitanRuntimeError::Activity)?;
        Ok(DivergentUniverseTitanCommandResolution {
            events,
            state_hash: activity.state_hash(),
        })
    }
}

fn effect(value: &DivergentUniverseTitanOrderedEffect) -> DivergentUniverseTitanEffect {
    DivergentUniverseTitanEffect {
        operation: value.operation.clone(),
        binding_key: value.binding_key.clone(),
        condition: value.condition.clone(),
        metric: value.metric.clone(),
        maze_buff_id: value.maze_buff_id.clone(),
        parameters: value.parameters.clone(),
        extra_effect_ids: value.extra_effect_ids.clone(),
        value: value.value.clone(),
    }
}

fn lookup<'a, T, I: Ord>(
    values: &'a [T],
    id: impl Fn(&T) -> &I,
    expected: &I,
) -> Result<&'a T, DivergentUniverseTitanRuntimeError> {
    values
        .binary_search_by(|value| id(value).cmp(expected))
        .ok()
        .and_then(|index| values.get(index))
        .ok_or(DivergentUniverseTitanRuntimeError::UnknownIdentity)
}

fn selected_type_key(
    activity: &GraphActivity,
) -> Result<Option<u64>, DivergentUniverseTitanRuntimeError> {
    let value = slot_value(activity, TITAN_TYPE_SLOT)?;
    let ActivityValue::OptionalId(value) = value else {
        return Err(DivergentUniverseTitanRuntimeError::InvalidState);
    };
    Ok(value)
}

fn ordered_set(
    activity: &GraphActivity,
    slot: starclock_activity::ActivitySlotId,
) -> Result<Vec<u64>, DivergentUniverseTitanRuntimeError> {
    let value = slot_value(activity, slot)?;
    let ActivityValue::OrderedIdSet(values) = value else {
        return Err(DivergentUniverseTitanRuntimeError::InvalidState);
    };
    if values.windows(2).any(|pair| pair[0] >= pair[1]) || values.contains(&0) {
        return Err(DivergentUniverseTitanRuntimeError::InvalidState);
    }
    Ok(values.into_vec())
}

fn talent_currency(activity: &GraphActivity) -> Result<i64, DivergentUniverseTitanRuntimeError> {
    let value = slot_value(activity, TITAN_TALENT_CURRENCY_SLOT)?;
    let ActivityValue::BoundedInteger(value) = value else {
        return Err(DivergentUniverseTitanRuntimeError::InvalidState);
    };
    if value < 0 {
        return Err(DivergentUniverseTitanRuntimeError::InvalidState);
    }
    Ok(value)
}

fn slot_value(
    activity: &GraphActivity,
    id: starclock_activity::ActivitySlotId,
) -> Result<ActivityValue, DivergentUniverseTitanRuntimeError> {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|slot| slot.id() == id)
        .map(|slot| slot.value().clone())
        .ok_or(DivergentUniverseTitanRuntimeError::InvalidState)
}

fn validate_hash(
    activity: &GraphActivity,
    expected: ActivityStateHash,
) -> Result<(), DivergentUniverseTitanRuntimeError> {
    if activity.state_hash() == expected {
        Ok(())
    } else {
        Err(DivergentUniverseTitanRuntimeError::Activity(
            GraphActivityCommandError::StaleStateHash,
        ))
    }
}

fn literal(value: ActivityValue) -> ActivityExpression {
    ActivityExpression::Literal(value)
}

fn ordinal(index: usize) -> Result<u64, DivergentUniverseTitanRuntimeError> {
    u64::try_from(index + 1)
        .ok()
        .filter(|value| *value != 0)
        .ok_or(DivergentUniverseTitanRuntimeError::InvalidCatalog)
}

fn snapshot_digest(
    state_hash: ActivityStateHash,
    component_digest: [u8; 32],
    selected: Option<&DivergentUniverseTitanTypeId>,
    boons: &[DivergentUniverseTitanBoonId],
    talents: &[DivergentUniverseTitanTalentId],
    contributions: &[DivergentUniverseTitanContribution],
) -> [u8; 32] {
    let mut hash = CanonicalDigestBuilder::new();
    field(&mut hash, b"starclock.divergent-universe.titan-snapshot.v1");
    field(&mut hash, &component_digest);
    field(&mut hash, &state_hash.bytes());
    optional_field(&mut hash, selected.map(|value| value.as_str()));
    for boon in boons {
        field(&mut hash, boon.as_str().as_bytes());
    }
    for talent in talents {
        field(&mut hash, talent.as_str().as_bytes());
    }
    for contribution in contributions {
        field(&mut hash, contribution.id.as_str().as_bytes());
        field(&mut hash, &[contribution.scope as u8]);
        field(&mut hash, contribution.activation.as_bytes());
        field(&mut hash, contribution.teardown.as_bytes());
        for effect in &contribution.effects {
            field(&mut hash, effect.operation.as_bytes());
            optional_field(&mut hash, effect.binding_key.as_deref());
            optional_field(&mut hash, effect.condition.as_deref());
            optional_field(&mut hash, effect.metric.as_deref());
            optional_field(&mut hash, effect.maze_buff_id.as_deref());
            optional_field(&mut hash, effect.value.as_deref());
            for parameter in &effect.parameters {
                field(&mut hash, parameter.as_bytes());
            }
            for effect_id in &effect.extra_effect_ids {
                field(&mut hash, effect_id.as_bytes());
            }
        }
    }
    hash.finalize()
}

fn optional_field(hash: &mut CanonicalDigestBuilder, value: Option<&str>) {
    match value {
        Some(value) => {
            hash.update([1]);
            field(hash, value.as_bytes());
        }
        None => hash.update([0]),
    }
}

fn field(hash: &mut CanonicalDigestBuilder, value: &[u8]) {
    hash.update(
        u64::try_from(value.len())
            .expect("slice length fits u64")
            .to_le_bytes(),
    );
    hash.update(value);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseTitanRuntimeError {
    InvalidCatalog,
    UnknownIdentity,
    InvalidState,
    InvalidProgram,
    TypeAlreadyActivated,
    TitanTypeNotActivated,
    NoFurtherBoonLevel,
    BoonNotOffered,
    InvalidAmount,
    CurrencyOutOfRange,
    TalentAlreadyUnlocked,
    MissingTalentPrerequisite,
    InsufficientTalentCurrency,
    Activity(GraphActivityCommandError),
}

impl std::fmt::Display for DivergentUniverseTitanRuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid Divergent Universe Titan runtime: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseTitanRuntimeError {}
