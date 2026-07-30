//! Knowledge binding lowering and deterministic target policy.

use serde::Deserialize;

use crate::gold_gears_unique::{GoldAndGearsUniqueCatalog, KnowledgeRule};

use super::GoldAndGearsEntryError;

pub const GOLD_AND_GEARS_KNOWLEDGE_REVISION: &str = "gold-and-gears-knowledge-policy-v1";

const TARGET_POLICY_ID: &str = "knowledge-target-selection-v1";
const SIMULTANEOUS_POLICY_ID: &str = "knowledge-simultaneous-resolution-v1";

#[derive(Clone, Debug)]
pub(super) struct KnowledgeRuntimeCatalog {
    rules: Box<[RuntimeKnowledgeRule]>,
}

#[derive(Clone, Debug)]
pub(super) struct RuntimeKnowledgeRule {
    pub(super) id: u32,
    pub(super) face_id: u32,
    pub(super) operation: KnowledgeOperation,
    pub(super) trigger: KnowledgeTrigger,
    pub(super) scope: KnowledgeTargetScope,
    pub(super) selection: KnowledgeSelection,
    pub(super) access: KnowledgeAccess,
    pub(super) parameters_scaled: Box<[i64]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum KnowledgeOperation {
    CopyCurrentDomainAndApply,
    CopySelectedDomainToAdjacentAndApply,
    CopySelectedDomainToPlaneAndApply,
    CopyCurrentDomainToPlaneAndApply,
    GenerateBeaconOnKnowledgeDomain,
    ApplyToUnmarkedDomains,
    PropagatePerKnowledgeDomain,
    PropagateFromSelectedDomain,
    ProtectCollapsingDomains,
    ApplyAdjacentToCurrentDomain,
    RewardPerKnowledgeDomainType,
    ApplyAfterEnteringKnowledgeDomain,
    OverrideMovementToKnowledgeDomain,
    TransformKnowledgeDomainToAdventure,
    ApplyToSelectedDomain,
    RemoveKnowledgeAndReward,
    RewardPerKnowledgeDomain,
    TransformToBlankAndPreserveKnowledge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum KnowledgeTrigger {
    Immediate,
    AfterMovement,
    AfterMovementBeforeCollapse,
    DuringMovementSelection,
    OnEnterDuringMovement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum KnowledgeSelection {
    All,
    CountAll,
    Random,
    RandomPerSource,
    Selected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum KnowledgeAccess {
    Apply,
    Preserve,
    Query,
    Remove,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum KnowledgeTargetScope {
    AdjacentDomainPerKnowledgeDomain,
    AdjacentNonBossDomain,
    AllAboutToCollapseDomains,
    AllAdjacentToCurrentDomain,
    AllAdjacentToSelectedKnowledgeDomain,
    AllKnowledgeDomains,
    AnyKnowledgeDomain,
    DistinctKnowledgeDomainTypes,
    RandomAboutToCollapseDomain,
    RandomAdjacentToCurrentDomain,
    RandomAdjacentToSelectedKnowledgeDomain,
    RandomKnowledgeDomain,
    RandomNonBossKnowledgeDomain,
    RandomNonBossPlaneDomain,
    RandomPlaneDomain,
    RandomUnmarkedPlaneDomain,
    SelectedDomain,
    SelectedDomainAndAllAdjacent,
    SelectedNonBlankNonBossKnowledgeDomain,
    SelectedNonBossDomain,
}

impl KnowledgeRuntimeCatalog {
    pub(super) fn compile(
        catalog: &GoldAndGearsUniqueCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let mut rules = catalog
            .knowledge_rules
            .iter()
            .map(runtime_rule)
            .collect::<Result<Vec<_>, _>>()?;
        rules.sort_by_key(|rule| rule.id);
        if rules.len() != 22
            || rules.windows(2).any(|pair| pair[0].id == pair[1].id)
            || rules
                .windows(2)
                .any(|pair| pair[0].face_id == pair[1].face_id)
        {
            return Err(GoldAndGearsEntryError::InvalidKnowledgeRuntime);
        }
        Ok(Self {
            rules: rules.into_boxed_slice(),
        })
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, [usize; 4], [usize; 5]) {
        let mut access = [0; 4];
        let mut selection = [0; 5];
        for rule in &self.rules {
            access[match rule.access {
                KnowledgeAccess::Apply => 0,
                KnowledgeAccess::Preserve => 1,
                KnowledgeAccess::Query => 2,
                KnowledgeAccess::Remove => 3,
            }] += 1;
            selection[match rule.selection {
                KnowledgeSelection::All => 0,
                KnowledgeSelection::CountAll => 1,
                KnowledgeSelection::Random => 2,
                KnowledgeSelection::RandomPerSource => 3,
                KnowledgeSelection::Selected => 4,
            }] += 1;
        }
        (self.rules.len(), access, selection)
    }

    pub(super) fn rule_for_face(&self, face_id: u32) -> Option<&RuntimeKnowledgeRule> {
        self.rules.iter().find(|rule| rule.face_id == face_id)
    }
}

impl RuntimeKnowledgeRule {
    pub(super) const fn trigger_name(&self) -> &'static str {
        match self.trigger {
            KnowledgeTrigger::Immediate => "Immediate",
            KnowledgeTrigger::AfterMovement => "AfterMovement",
            KnowledgeTrigger::AfterMovementBeforeCollapse => "AfterMovementBeforeCollapse",
            KnowledgeTrigger::DuringMovementSelection => "DuringMovementSelection",
            KnowledgeTrigger::OnEnterDuringMovement => "OnEnterDuringMovement",
        }
    }

    pub(super) const fn scope_name(&self) -> &'static str {
        match self.scope {
            KnowledgeTargetScope::AdjacentDomainPerKnowledgeDomain => {
                "AdjacentDomainPerKnowledgeDomain"
            }
            KnowledgeTargetScope::AdjacentNonBossDomain => "AdjacentNonBossDomain",
            KnowledgeTargetScope::AllAboutToCollapseDomains => "AllAboutToCollapseDomains",
            KnowledgeTargetScope::AllAdjacentToCurrentDomain => "AllAdjacentToCurrentDomain",
            KnowledgeTargetScope::AllAdjacentToSelectedKnowledgeDomain => {
                "AllAdjacentToSelectedKnowledgeDomain"
            }
            KnowledgeTargetScope::AllKnowledgeDomains => "AllKnowledgeDomains",
            KnowledgeTargetScope::AnyKnowledgeDomain => "AnyKnowledgeDomain",
            KnowledgeTargetScope::DistinctKnowledgeDomainTypes => "DistinctKnowledgeDomainTypes",
            KnowledgeTargetScope::RandomAboutToCollapseDomain => "RandomAboutToCollapseDomain",
            KnowledgeTargetScope::RandomAdjacentToCurrentDomain => "RandomAdjacentToCurrentDomain",
            KnowledgeTargetScope::RandomAdjacentToSelectedKnowledgeDomain => {
                "RandomAdjacentToSelectedKnowledgeDomain"
            }
            KnowledgeTargetScope::RandomKnowledgeDomain => "RandomKnowledgeDomain",
            KnowledgeTargetScope::RandomNonBossKnowledgeDomain => "RandomNonBossKnowledgeDomain",
            KnowledgeTargetScope::RandomNonBossPlaneDomain => "RandomNonBossPlaneDomain",
            KnowledgeTargetScope::RandomPlaneDomain => "RandomPlaneDomain",
            KnowledgeTargetScope::RandomUnmarkedPlaneDomain => "RandomUnmarkedPlaneDomain",
            KnowledgeTargetScope::SelectedDomain => "SelectedDomain",
            KnowledgeTargetScope::SelectedDomainAndAllAdjacent => "SelectedDomainAndAllAdjacent",
            KnowledgeTargetScope::SelectedNonBlankNonBossKnowledgeDomain => {
                "SelectedNonBlankNonBossKnowledgeDomain"
            }
            KnowledgeTargetScope::SelectedNonBossDomain => "SelectedNonBossDomain",
        }
    }
}

fn runtime_rule(rule: &KnowledgeRule) -> Result<RuntimeKnowledgeRule, GoldAndGearsEntryError> {
    validate_policies(rule)?;
    Ok(RuntimeKnowledgeRule {
        id: rule.identity.id.0,
        face_id: rule.dice_face.0,
        operation: parse_operation(&rule.operation)?,
        trigger: parse_trigger(&rule.trigger_boundary)?,
        scope: parse_scope(&rule.target_scope)?,
        selection: parse_selection(&rule.selection_mode)?,
        access: parse_access(&rule.knowledge_access)?,
        parameters_scaled: rule
            .parameters
            .iter()
            .map(|value| scaled(value))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
    })
}

fn validate_policies(rule: &KnowledgeRule) -> Result<(), GoldAndGearsEntryError> {
    let target = serde_json::from_str::<TargetPolicy>(&rule.target_policy_json)
        .map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)?;
    let simultaneous = serde_json::from_str::<SimultaneousPolicy>(&rule.simultaneous_policy_json)
        .map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)?;
    let interactions = serde_json::from_str::<DiceInteractions>(&rule.dice_interactions_json)
        .map_err(|_| GoldAndGearsEntryError::InvalidKnowledgeRuntime)?;
    if target.policy_id.as_ref() != TARGET_POLICY_ID
        || target.evidence_quality.as_ref() != "ProjectPolicy"
        || target.candidate_order.as_ref() != "stable-node-id-ascending"
        || target.random_selection.as_ref() != "seeded-without-replacement"
        || target.selected_validation.as_ref() != "reject-outside-exact-selector"
        || target.empty_candidate_behavior.as_ref() != "NoEffect"
        || target.replacement_condition.is_empty()
        || simultaneous.policy_id.as_ref() != SIMULTANEOUS_POLICY_ID
        || simultaneous.evidence_quality.as_ref() != "ProjectPolicy"
        || simultaneous
            .tiers
            .iter()
            .map(Box::as_ref)
            .collect::<Vec<_>>()
            != [
                "resolve-movement-destination",
                "resolve-after-movement-face-effects",
                "apply-knowledge-state-mutations",
                "apply-active-custom-dice-entry-or-collapse-callbacks",
                "resolve-domain-collapse",
                "award-derived-resources",
            ]
        || simultaneous
            .tie_breakers
            .iter()
            .map(Box::as_ref)
            .collect::<Vec<_>>()
            != ["dice-face-id", "target-node-id"]
        || simultaneous.replacement_condition.is_empty()
        || interactions.countdown_dice_id.as_ref() != "gold-gears.custom-dice.301"
        || interactions.collapse_prevention_dice_id.as_ref() != "gold-gears.custom-dice.302"
        || interactions.collapse_reward_dice_id.as_ref() != "gold-gears.custom-dice.303"
        || interactions.evidence_quality.as_ref() != "ExactStructured"
        || interactions.countdown_behavior.is_empty()
        || interactions.collapse_prevention_behavior.is_empty()
        || interactions.collapse_reward_behavior.is_empty()
    {
        return Err(GoldAndGearsEntryError::InvalidKnowledgeRuntime);
    }
    Ok(())
}

fn parse_operation(value: &str) -> Result<KnowledgeOperation, GoldAndGearsEntryError> {
    match value {
        "CopyCurrentDomainAndApplyKnowledge" => Ok(KnowledgeOperation::CopyCurrentDomainAndApply),
        "CopySelectedDomainToAdjacentAndApplyKnowledge" => {
            Ok(KnowledgeOperation::CopySelectedDomainToAdjacentAndApply)
        }
        "CopySelectedDomainToPlaneAndApplyKnowledge" => {
            Ok(KnowledgeOperation::CopySelectedDomainToPlaneAndApply)
        }
        "CopyCurrentDomainToPlaneAndApplyKnowledge" => {
            Ok(KnowledgeOperation::CopyCurrentDomainToPlaneAndApply)
        }
        "GenerateBeaconOnKnowledgeDomain" => {
            Ok(KnowledgeOperation::GenerateBeaconOnKnowledgeDomain)
        }
        "ApplyKnowledgeToUnmarkedDomains" => Ok(KnowledgeOperation::ApplyToUnmarkedDomains),
        "PropagateKnowledgePerKnowledgeDomain" => {
            Ok(KnowledgeOperation::PropagatePerKnowledgeDomain)
        }
        "PropagateKnowledgeFromSelectedDomain" => {
            Ok(KnowledgeOperation::PropagateFromSelectedDomain)
        }
        "ProtectCollapsingDomainsWithKnowledge" => Ok(KnowledgeOperation::ProtectCollapsingDomains),
        "ApplyKnowledgeAdjacentToCurrentDomain" => {
            Ok(KnowledgeOperation::ApplyAdjacentToCurrentDomain)
        }
        "RewardPerKnowledgeDomainType" => Ok(KnowledgeOperation::RewardPerKnowledgeDomainType),
        "ApplyKnowledgeAfterEnteringKnowledgeDomain" => {
            Ok(KnowledgeOperation::ApplyAfterEnteringKnowledgeDomain)
        }
        "OverrideMovementToKnowledgeDomain" => {
            Ok(KnowledgeOperation::OverrideMovementToKnowledgeDomain)
        }
        "TransformKnowledgeDomainToAdventure" => {
            Ok(KnowledgeOperation::TransformKnowledgeDomainToAdventure)
        }
        "ApplyKnowledgeToSelectedDomain" => Ok(KnowledgeOperation::ApplyToSelectedDomain),
        "RemoveKnowledgeAndRewardPerRemoval" => Ok(KnowledgeOperation::RemoveKnowledgeAndReward),
        "RewardPerKnowledgeDomain" => Ok(KnowledgeOperation::RewardPerKnowledgeDomain),
        "TransformToBlankAndPreserveKnowledge" => {
            Ok(KnowledgeOperation::TransformToBlankAndPreserveKnowledge)
        }
        _ => Err(GoldAndGearsEntryError::InvalidKnowledgeRuntime),
    }
}

fn parse_trigger(value: &str) -> Result<KnowledgeTrigger, GoldAndGearsEntryError> {
    match value {
        "Immediate" => Ok(KnowledgeTrigger::Immediate),
        "AfterMovement" => Ok(KnowledgeTrigger::AfterMovement),
        "AfterMovementBeforeCollapse" => Ok(KnowledgeTrigger::AfterMovementBeforeCollapse),
        "DuringMovementSelection" => Ok(KnowledgeTrigger::DuringMovementSelection),
        "OnEnterDuringMovement" => Ok(KnowledgeTrigger::OnEnterDuringMovement),
        _ => Err(GoldAndGearsEntryError::InvalidKnowledgeRuntime),
    }
}

fn parse_selection(value: &str) -> Result<KnowledgeSelection, GoldAndGearsEntryError> {
    match value {
        "All" => Ok(KnowledgeSelection::All),
        "CountAll" => Ok(KnowledgeSelection::CountAll),
        "Random" => Ok(KnowledgeSelection::Random),
        "RandomPerSource" => Ok(KnowledgeSelection::RandomPerSource),
        "Selected" => Ok(KnowledgeSelection::Selected),
        _ => Err(GoldAndGearsEntryError::InvalidKnowledgeRuntime),
    }
}

fn parse_access(value: &str) -> Result<KnowledgeAccess, GoldAndGearsEntryError> {
    match value {
        "Apply" => Ok(KnowledgeAccess::Apply),
        "Preserve" => Ok(KnowledgeAccess::Preserve),
        "Query" => Ok(KnowledgeAccess::Query),
        "Remove" => Ok(KnowledgeAccess::Remove),
        _ => Err(GoldAndGearsEntryError::InvalidKnowledgeRuntime),
    }
}

fn parse_scope(value: &str) -> Result<KnowledgeTargetScope, GoldAndGearsEntryError> {
    match value {
        "AdjacentDomainPerKnowledgeDomain" => {
            Ok(KnowledgeTargetScope::AdjacentDomainPerKnowledgeDomain)
        }
        "AdjacentNonBossDomain" => Ok(KnowledgeTargetScope::AdjacentNonBossDomain),
        "AllAboutToCollapseDomains" => Ok(KnowledgeTargetScope::AllAboutToCollapseDomains),
        "AllAdjacentToCurrentDomain" => Ok(KnowledgeTargetScope::AllAdjacentToCurrentDomain),
        "AllAdjacentToSelectedKnowledgeDomain" => {
            Ok(KnowledgeTargetScope::AllAdjacentToSelectedKnowledgeDomain)
        }
        "AllKnowledgeDomains" => Ok(KnowledgeTargetScope::AllKnowledgeDomains),
        "AnyKnowledgeDomain" => Ok(KnowledgeTargetScope::AnyKnowledgeDomain),
        "DistinctKnowledgeDomainTypes" => Ok(KnowledgeTargetScope::DistinctKnowledgeDomainTypes),
        "RandomAboutToCollapseDomain" => Ok(KnowledgeTargetScope::RandomAboutToCollapseDomain),
        "RandomAdjacentToCurrentDomain" => Ok(KnowledgeTargetScope::RandomAdjacentToCurrentDomain),
        "RandomAdjacentToSelectedKnowledgeDomain" => {
            Ok(KnowledgeTargetScope::RandomAdjacentToSelectedKnowledgeDomain)
        }
        "RandomKnowledgeDomain" => Ok(KnowledgeTargetScope::RandomKnowledgeDomain),
        "RandomNonBossKnowledgeDomain" => Ok(KnowledgeTargetScope::RandomNonBossKnowledgeDomain),
        "RandomNonBossPlaneDomain" => Ok(KnowledgeTargetScope::RandomNonBossPlaneDomain),
        "RandomPlaneDomain" => Ok(KnowledgeTargetScope::RandomPlaneDomain),
        "RandomUnmarkedPlaneDomain" => Ok(KnowledgeTargetScope::RandomUnmarkedPlaneDomain),
        "SelectedDomain" => Ok(KnowledgeTargetScope::SelectedDomain),
        "SelectedDomainAndAllAdjacent" => Ok(KnowledgeTargetScope::SelectedDomainAndAllAdjacent),
        "SelectedNonBlankNonBossKnowledgeDomain" => {
            Ok(KnowledgeTargetScope::SelectedNonBlankNonBossKnowledgeDomain)
        }
        "SelectedNonBossDomain" => Ok(KnowledgeTargetScope::SelectedNonBossDomain),
        _ => Err(GoldAndGearsEntryError::InvalidKnowledgeRuntime),
    }
}

fn scaled(value: &str) -> Result<i64, GoldAndGearsEntryError> {
    let (integer, fraction) = value.split_once('.').map_or((value, ""), |parts| parts);
    if fraction.len() > 6 {
        return Err(GoldAndGearsEntryError::InvalidKnowledgeRuntime);
    }
    let mut fraction_text = fraction.to_owned();
    fraction_text.extend(core::iter::repeat_n('0', 6 - fraction.len()));
    integer
        .parse::<i64>()
        .ok()
        .and_then(|whole| whole.checked_mul(1_000_000))
        .and_then(|whole| {
            fraction_text
                .parse::<i64>()
                .ok()
                .and_then(|fraction| whole.checked_add(fraction))
        })
        .ok_or(GoldAndGearsEntryError::InvalidKnowledgeRuntime)
}

#[derive(Deserialize)]
struct TargetPolicy {
    policy_id: Box<str>,
    evidence_quality: Box<str>,
    candidate_order: Box<str>,
    random_selection: Box<str>,
    selected_validation: Box<str>,
    empty_candidate_behavior: Box<str>,
    replacement_condition: Box<str>,
}

#[derive(Deserialize)]
struct SimultaneousPolicy {
    policy_id: Box<str>,
    evidence_quality: Box<str>,
    tiers: Box<[Box<str>]>,
    tie_breakers: Box<[Box<str>]>,
    replacement_condition: Box<str>,
}

#[derive(Deserialize)]
struct DiceInteractions {
    countdown_dice_id: Box<str>,
    countdown_behavior: Box<str>,
    collapse_prevention_dice_id: Box<str>,
    collapse_prevention_behavior: Box<str>,
    collapse_reward_dice_id: Box<str>,
    collapse_reward_behavior: Box<str>,
    evidence_quality: Box<str>,
}
