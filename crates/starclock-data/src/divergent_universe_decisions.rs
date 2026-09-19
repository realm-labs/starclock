//! Typed executable decision authoring, separate from the reference-only pack.
//!
//! Loading proves data validity, not execution of event or acquisition effects.
//! Callers must bind this catalog's digest into the Activity configuration before
//! exposing its choices. No generated transport row crosses this boundary.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use sha2::{Digest, Sha256};

use crate::divergent_universe::DivergentUniverseBundleCandidate;
use crate::divergent_universe_curio_catalog::{
    DivergentUniverseCurioId, DivergentUniverseCurioStateId,
};
use crate::divergent_universe_decisions_generated::{
    SCHEMA_FINGERPRINT, SoraConfig,
    du_decision_reward_kind::DuDecisionRewardKind,
    runtime::{SoraBundle, SoraTableSource},
};
use crate::divergent_universe_domain_decks::DomainDeckDefinition;
use crate::divergent_universe_domain_layout::DomainLayerLayout;
use crate::divergent_universe_encounter_catalog::DivergentUniverseEncounterGroupId;
use crate::divergent_universe_equation_catalog::{
    DivergentUniverseEquationCategory, DivergentUniversePathType,
};
use crate::divergent_universe_service_catalog::{
    DivergentUniverseOccurrenceId, DivergentUniverseOccurrenceVariantId,
};

#[cfg(test)]
#[path = "divergent_universe_decision_tests.rs"]
mod tests;
#[path = "divergent_universe_decision_validation.rs"]
mod validation;

const BUNDLE: &[u8] =
    include_bytes!("../../../config/divergent-universe-decisions-generated/config.sora");
const SCHEMA: &[u8] =
    include_bytes!("../../../config/divergent-universe-decisions-generated/schema.lock");

/// Stable project-owned choice identity; transport row numbers are not identity.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DecisionChoiceId(Box<str>);

impl DecisionChoiceId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionEvidence {
    ExactStructured,
    ObservedCommunity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionSource {
    pub key: Box<str>,
    pub url: Box<str>,
    pub revision: Box<str>,
    pub game_version: Box<str>,
    pub access_date: Box<str>,
    pub locator: Box<str>,
    pub sha256: Box<str>,
    pub quality: DecisionEvidence,
    pub note: Box<str>,
}

/// Explicit policy; neither the current-event binding nor hidden draw rules are
/// represented as observed parity. Selection draws distinct unowned identities
/// from the current catalog in stable-key order, each with integer weight one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionPolicyKind {
    VersionedProjectPolicyUniformUnownedCurrentCatalog,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionExhaustion {
    DisableChoice,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionPolicy {
    pub key: Box<str>,
    pub kind: DecisionPolicyKind,
    pub exhaustion: DecisionExhaustion,
    pub note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RewardRarityRange {
    minimum: u8,
    maximum: u8,
}

impl RewardRarityRange {
    #[must_use]
    pub const fn minimum(self) -> u8 {
        self.minimum
    }
    #[must_use]
    pub const fn maximum(self) -> u8 {
        self.maximum
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionReward {
    Fragments(u64),
    Curios {
        count: u16,
        rarity: RewardRarityRange,
    },
    Blessings {
        count: u16,
        rarity: RewardRarityRange,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionOutcome {
    pub key: Box<str>,
    pub reward: DecisionReward,
    pub source: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionChoice {
    pub id: DecisionChoiceId,
    pub ordinal: u16,
    pub name_en: Box<str>,
    pub name_zh_cn: Box<str>,
    pub fragment_cost: u64,
    pub source: Box<str>,
    pub outcomes: Box<[DecisionOutcome]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionOccurrence {
    pub key: Box<str>,
    pub occurrence: DivergentUniverseOccurrenceId,
    pub variant: DivergentUniverseOccurrenceVariantId,
    pub policy: DecisionPolicy,
    pub sources: Box<[Box<str>]>,
    pub choices: Box<[DecisionChoice]>,
}

/// Reviewed immediate acquisition effect, not a complete Curio lifecycle.
/// A state/effect join does not establish handbook ownership or pool eligibility.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurioAcquisitionGrant {
    FixedFragments(u64),
    BalanceFraction {
        numerator: u32,
        denominator: u32,
    },
    PathBlessings {
        count: u16,
        paths: Box<[DivergentUniversePathType]>,
    },
    RarityBlessings {
        count: u16,
        rarity: RewardRarityRange,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioAcquisitionPolicy {
    VersionedProjectPolicyStableStateOrderFloorBeforeEachGrant,
    VersionedProjectPolicyStableStateOrderUniformUnownedPathRejectExhaustion,
    /// Replaceable project policy: sequential unit-weight draws conditioned on
    /// feasibility of all remaining mandatory Path/rarity rewards, not parity.
    FeasibleRarityAssignments,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioAcquisitionDefinition {
    pub key: Box<str>,
    pub state: DivergentUniverseCurioStateId,
    pub effect_id: Box<str>,
    pub grant: CurioAcquisitionGrant,
    pub policy: CurioAcquisitionPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioFragmentGainPolicy {
    VersionedProjectPolicyActiveStateAdditiveOriginalBaseFloorEachBonus,
}

/// Reviewed global gain component, not a complete Curio lifecycle definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioFragmentGainDefinition {
    pub key: Box<str>,
    pub state: DivergentUniverseCurioStateId,
    pub effect_id: Box<str>,
    pub numerator: u32,
    pub denominator: u32,
    pub policy: CurioFragmentGainPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

/// Immutable validated decisions. The digest binds the exact decision schema,
/// binary and reference component; equal IDs cannot hide different parameters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionCatalog {
    domain_decks: Box<[DomainDeckDefinition]>,
    domain_layout: Box<[DomainLayerLayout]>,
    digest: [u8; 32],
    sources: Box<[DecisionSource]>,
    occurrences: Box<[DecisionOccurrence]>,
    curio_acquisitions: Box<[CurioAcquisitionDefinition]>,
    curio_fragment_gains: Box<[CurioFragmentGainDefinition]>,
    curio_domain_expiries: Box<[CurioDomainExpiryDefinition]>,
    curio_domain_grants: Box<[CurioDomainGrantDefinition]>,
    curio_battle_stats: Box<[CurioBattleStatDefinition]>,
    curio_battle_reactions: Box<[CurioBattleReactionDefinition]>,
    tawot_services: Box<[TawotServiceDefinition]>,
    battle_blessings: Box<[BattleBlessingPolicy]>,
    curio_battle_weights: Box<[CurioBattleWeightDefinition]>,
    equation_grants: Box<[EquationGrantDefinition]>,
    equation_expansion_rewards: Box<[EquationExpansionRewardDefinition]>,
    initial_equations: InitialEquationPolicy,
    battle_route: BattleRoutePolicy,
    encounter_pool: EncounterPoolPolicy,
    curio_battle_grants: Box<[CurioBattleGrantDefinition]>,
    curio_victory_blessings: Box<[CurioVictoryBlessingDefinition]>,
    domain_choices: Box<[DomainChoiceDefinition]>,
    battle_fragments: Box<[BattleFragmentDefinition]>,
    curio_evolutions: Box<[CurioEvolutionDefinition]>,
    evolution_events: Box<[EvolutionEventDefinition]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TawotServicePolicy {
    VersionedProjectPolicyUniformDistinctCurrentStatesPaidSelection,
}

/// Explicit service admission operands, not an original Forge placement claim.
/// Candidate membership and purchase semantics are policy; individual Curio
/// effect execution remains a separate requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TawotServiceDefinition {
    pub key: Box<str>,
    pub occurrence: DivergentUniverseOccurrenceId,
    pub variant: DivergentUniverseOccurrenceVariantId,
    pub curio: DivergentUniverseCurioId,
    pub states: Box<[DivergentUniverseCurioStateId]>,
    pub forge_level: u16,
    pub fragment_cost: u32,
    pub offer_width: u16,
    pub purchase_limit: u16,
    pub policy: TawotServicePolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

/// Exact domain-entry allowance; scheduling and lifecycle interaction are policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioDomainExpiryDefinition {
    pub key: Box<str>,
    pub state: DivergentUniverseCurioStateId,
    pub effect_id: Box<str>,
    pub domain_limit: u16,
    pub policy: CurioDomainExpiryPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioDomainExpiryPolicy {
    VersionedProjectPolicyActiveFutureSelectedDomainsDiscard,
    VersionedProjectPolicyActiveFutureSelectedDomainsGrantThenDiscard,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioDomainGrantPolicy {
    VersionedProjectPolicyActivePositiveAllowanceFragmentsBeforeDiscard,
}

/// Exact entry income joined to an independently validated expiry definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioDomainGrantDefinition {
    pub key: Box<str>,
    pub state: DivergentUniverseCurioStateId,
    pub amount: u32,
    pub policy: CurioDomainGrantPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioBattleStat {
    Speed,
    FinalDamage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioBattleStatPolicy {
    VersionedProjectPolicyBasePercentVerifiedBattleLifetime,
    VersionedProjectPolicyOutgoingFinalMultiplierVerifiedBattleLifetime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioBattleReactionPolicy {
    VersionedProjectPolicyFirstSurvivingOrdinaryAttackPerTargetAction,
}

/// Reviewed fixed healing operand, reaction policy and battle-counted allowance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioBattleReactionDefinition {
    pub key: Box<str>,
    pub state: DivergentUniverseCurioStateId,
    pub effect_id: Box<str>,
    pub heal_fraction: Box<str>,
    pub battle_limit: u16,
    pub policy: CurioBattleReactionPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

/// Source-validated passive magnitude and a battle-counted lifetime, not domains.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioBattleStatDefinition {
    pub key: Box<str>,
    pub state: DivergentUniverseCurioStateId,
    pub effect_id: Box<str>,
    pub stat: CurioBattleStat,
    pub bonus_fraction: Box<str>,
    pub battle_limit: u16,
    pub policy: CurioBattleStatPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvolutionEventPolicy {
    VersionedProjectPolicyActiveTreasuresAtLayerEntryStableSequence,
}

/// Base run-currency reward, separate from Curio and occurrence grants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleFragmentDefinition {
    pub key: Box<str>,
    pub domain: BattleRewardDomain,
    pub amount: u64,
    pub policy: BattleFragmentPolicy,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFragmentPolicy {
    VersionedProjectPolicyFixedVerifiedDomainCredit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvolutionOptionEffect {
    Evolve,
    Chance {
        numerator: u32,
        denominator: u32,
    },
    Curios {
        count: u16,
        rarity: RewardRarityRange,
    },
    Sacrifice {
        count: u16,
        rarity: RewardRarityRange,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvolutionOptionDefinition {
    pub key: Box<str>,
    pub ordinal: u16,
    pub name_en: Box<str>,
    pub name_zh_cn: Box<str>,
    pub fragment_cost: u64,
    pub fragment_grant: u64,
    pub effect: EvolutionOptionEffect,
    pub sources: Box<[Box<str>]>,
}

/// Public choices with explicit layer/iteration policy, not exact room membership.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvolutionEventDefinition {
    pub key: Box<str>,
    pub evolution: CurioEvolutionDefinition,
    pub layer_ordinal: u16,
    pub options: Box<[EvolutionOptionDefinition]>,
    pub policy: EvolutionEventPolicy,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioEvolutionPolicy {
    VersionedProjectPolicyActiveSameOwnerResetAndAcquire,
}

/// Reviewed transition/ownership overlay, not ordinary-pool or player eligibility.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioEvolutionDefinition {
    pub key: Box<str>,
    pub from: DivergentUniverseCurioStateId,
    pub to: DivergentUniverseCurioStateId,
    pub owner: DivergentUniverseCurioId,
    pub occurrence: DivergentUniverseOccurrenceId,
    pub policy: CurioEvolutionPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioBattleGrantPolicy {
    VersionedProjectPolicyFullHpPresentRosterAfterCarry,
}

/// Mode-authenticated domain classification, never inferred from enemy stages.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BattleRewardDomain {
    Combat,
    Elite,
    Aberration,
    Boss,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioVictoryBlessingPolicy {
    VersionedProjectPolicyBoundDomainUniformUnownedAvailableSubset,
    VersionedProjectPolicyPositiveDomainAllowanceUniformUnownedAvailableSubset,
}

/// Immediate victory grant, separate from acquisition rewards and player offers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioVictoryBlessingDefinition {
    pub key: Box<str>,
    pub state: DivergentUniverseCurioStateId,
    pub effect_id: Box<str>,
    pub count: u16,
    pub rarity: RewardRarityRange,
    pub domains: Box<[BattleRewardDomain]>,
    pub policy: CurioVictoryBlessingPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

/// Reviewed victory reward; does not implement base drops or Curio upgrades.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioBattleGrantDefinition {
    pub key: Box<str>,
    pub state: DivergentUniverseCurioStateId,
    pub effect_id: Box<str>,
    pub amount_per_full_hp: u32,
    pub policy: CurioBattleGrantPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncounterPoolPolicyKind {
    VersionedProjectPolicyFixedFirstUniformLaterLayersWithReplacement,
}

/// Explicit baseline pool. Reference group membership does not establish room eligibility.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterPoolPolicy {
    pub key: Box<str>,
    pub kind: EncounterPoolPolicyKind,
    pub encounter_group: DivergentUniverseEncounterGroupId,
    pub first_stage: Box<str>,
    pub candidate_stages: Box<[Box<str>]>,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleRoutePolicyKind {
    VersionedProjectPolicyOneBattlePerLayerWithDomainChoices,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomainChoicePolicy {
    VersionedProjectPolicyExplicitLaterLayerChoices,
}

/// Explicit proxy-domain choice; no source room identity or original pool claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainChoiceDefinition {
    pub key: Box<str>,
    pub ordinal: u16,
    pub domain: BattleRewardDomain,
    pub name_en: Box<str>,
    pub name_zh_cn: Box<str>,
    pub policy: DomainChoicePolicy,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

/// Explicit provisional topology, not original room or encounter membership.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleRoutePolicy {
    pub key: Box<str>,
    pub kind: BattleRoutePolicyKind,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

/// Replaceable starting-choice policy; category records do not prove pool membership.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitialEquationPolicy {
    pub key: Box<str>,
    pub offer_width: u16,
    pub category: DivergentUniverseEquationCategory,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

impl DecisionCatalog {
    /// Explicitly selected source decks, not a released mask offer pool.
    pub fn domain_decks(&self) -> &[DomainDeckDefinition] {
        &self.domain_decks
    }
    /// Current-profile position records, not executable room or deck membership.
    pub fn domain_layout(&self) -> &[DomainLayerLayout] {
        &self.domain_layout
    }
    #[must_use]
    pub fn domain_choices(&self) -> &[DomainChoiceDefinition] {
        &self.domain_choices
    }
    #[must_use]
    pub fn battle_fragments(&self) -> &[BattleFragmentDefinition] {
        &self.battle_fragments
    }
    #[must_use]
    pub fn curio_victory_blessings(&self) -> &[CurioVictoryBlessingDefinition] {
        &self.curio_victory_blessings
    }
    #[must_use]
    pub fn evolution_events(&self) -> &[EvolutionEventDefinition] {
        &self.evolution_events
    }
    #[must_use]
    pub fn curio_evolutions(&self) -> &[CurioEvolutionDefinition] {
        &self.curio_evolutions
    }
    #[must_use]
    pub fn curio_battle_grants(&self) -> &[CurioBattleGrantDefinition] {
        &self.curio_battle_grants
    }
    #[must_use]
    pub const fn encounter_pool(&self) -> &EncounterPoolPolicy {
        &self.encounter_pool
    }
    #[must_use]
    pub const fn battle_route(&self) -> &BattleRoutePolicy {
        &self.battle_route
    }
    #[must_use]
    pub const fn initial_equations(&self) -> &InitialEquationPolicy {
        &self.initial_equations
    }
    #[must_use]
    pub fn equation_grants(&self) -> &[EquationGrantDefinition] {
        &self.equation_grants
    }

    /// Reviewed expansion trigger operands; compilation alone does not execute them.
    #[must_use]
    pub fn equation_expansion_rewards(&self) -> &[EquationExpansionRewardDefinition] {
        &self.equation_expansion_rewards
    }
    #[must_use]
    pub fn curio_battle_weights(&self) -> &[CurioBattleWeightDefinition] {
        &self.curio_battle_weights
    }
    pub fn production(
        reference: &DivergentUniverseBundleCandidate,
    ) -> Result<Self, DecisionDataError> {
        Self::from_bytes(BUNDLE, reference)
    }

    /// Decodes only the current Sora schema and validates all reference joins,
    /// ordinals, numeric bounds and provenance. Failure returns no partial catalog.
    pub fn from_bytes(
        bytes: &[u8],
        reference: &DivergentUniverseBundleCandidate,
    ) -> Result<Self, DecisionDataError> {
        let bundle = SoraBundle::parse(bytes).map_err(|_| DecisionDataError::Transport)?;
        if bundle
            .schema_fingerprint()
            .map_err(|_| DecisionDataError::Transport)?
            != SCHEMA_FINGERPRINT
        {
            return Err(DecisionDataError::Transport);
        }
        let config = SoraConfig::from_source(&bundle).map_err(|_| DecisionDataError::Transport)?;
        let mut result = validation::compile(&config, reference)?;
        let mut digest = Sha256::new();
        digest.update(Sha256::digest(SCHEMA));
        digest.update(Sha256::digest(bytes));
        digest.update(reference.identity().component_digest().bytes());
        result.digest = digest.finalize().into();
        Ok(result)
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    #[must_use]
    pub fn sources(&self) -> &[DecisionSource] {
        &self.sources
    }
    #[must_use]
    pub fn occurrences(&self) -> &[DecisionOccurrence] {
        &self.occurrences
    }

    #[must_use]
    pub fn curio_acquisitions(&self) -> &[CurioAcquisitionDefinition] {
        &self.curio_acquisitions
    }

    #[must_use]
    pub fn curio_fragment_gains(&self) -> &[CurioFragmentGainDefinition] {
        &self.curio_fragment_gains
    }

    #[must_use]
    pub fn curio_domain_expiries(&self) -> &[CurioDomainExpiryDefinition] {
        &self.curio_domain_expiries
    }

    /// Exact intrinsic entry grants; execution and transaction timing are mode-owned.
    #[must_use]
    pub fn curio_domain_grants(&self) -> &[CurioDomainGrantDefinition] {
        &self.curio_domain_grants
    }

    #[must_use]
    pub fn curio_battle_stats(&self) -> &[CurioBattleStatDefinition] {
        &self.curio_battle_stats
    }

    #[must_use]
    pub fn curio_battle_reactions(&self) -> &[CurioBattleReactionDefinition] {
        &self.curio_battle_reactions
    }

    #[must_use]
    pub fn tawot_services(&self) -> &[TawotServiceDefinition] {
        &self.tawot_services
    }

    #[must_use]
    pub fn battle_blessings(&self) -> &[BattleBlessingPolicy] {
        &self.battle_blessings
    }
}

/// Source-backed Path preference with independently replaceable numeric weights.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioBattleWeightDefinition {
    pub key: Box<str>,
    pub state: DivergentUniverseCurioStateId,
    pub effect_id: Box<str>,
    pub path: DivergentUniversePathType,
    pub bonus_weight: u32,
    pub policy: CurioBattleWeightPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioBattleWeightPolicy {
    VersionedProjectPolicyActiveStateAdditiveCandidateWeight,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquationGrantDefinition {
    pub key: Box<str>,
    pub state: DivergentUniverseCurioStateId,
    pub effect_id: Box<str>,
    pub count: u16,
    pub policy: EquationGrantPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EquationGrantPolicy {
    VersionedProjectPolicyUniformMissingRecipeAvailableSubsetOnceLogicalDomain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EquationExpansionRewardPolicy {
    VersionedProjectPolicyActivePreStateUniformUnownedBoundedCascade,
}

/// Exact trigger counts with explicit replaceable reward-pool and timing policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquationExpansionRewardDefinition {
    pub key: Box<str>,
    pub state: DivergentUniverseCurioStateId,
    pub effect_id: Box<str>,
    pub count: u16,
    pub trigger_limit: u16,
    pub policy: EquationExpansionRewardPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

/// Normal-battle policy, not an observed current-version reward distribution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleBlessingPolicy {
    pub key: Box<str>,
    pub kind: BattleBlessingPolicyKind,
    pub offer_width: u16,
    pub rarity: RewardRarityRange,
    pub suppression_states: Box<[DivergentUniverseCurioStateId]>,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleBlessingPolicyKind {
    VersionedProjectPolicyUniformUnownedSingleSelectionAvailableSubset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionDataError {
    Transport,
    InvalidIdentity,
    InvalidReference,
    InvalidPolicy,
    InvalidProvenance,
    InvalidOrder,
    InvalidReward,
}

impl Display for DecisionDataError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        write!(formatter, "Divergent Universe decision data: {self:?}")
    }
}
impl Error for DecisionDataError {}

fn key(value: &str) -> Result<Box<str>, DecisionDataError> {
    if value.is_empty()
        || value.len() > 300
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
    {
        return Err(DecisionDataError::InvalidIdentity);
    }
    Ok(value.into())
}

fn unique<'a>(values: impl Iterator<Item = &'a str>) -> Result<(), DecisionDataError> {
    let mut seen = BTreeSet::new();
    for value in values {
        key(value)?;
        if !seen.insert(value) {
            return Err(DecisionDataError::InvalidIdentity);
        }
    }
    Ok(())
}

fn reward(
    kind: DuDecisionRewardKind,
    amount: i32,
    minimum: Option<i32>,
    maximum: Option<i32>,
) -> Result<DecisionReward, DecisionDataError> {
    if amount <= 0 {
        return Err(DecisionDataError::InvalidReward);
    }
    if kind == DuDecisionRewardKind::Fragments {
        if minimum.is_some() || maximum.is_some() {
            return Err(DecisionDataError::InvalidReward);
        }
        return Ok(DecisionReward::Fragments(
            u64::try_from(amount).map_err(|_| DecisionDataError::InvalidReward)?,
        ));
    }
    let (Some(minimum), Some(maximum)) = (minimum, maximum) else {
        return Err(DecisionDataError::InvalidReward);
    };
    if !(1..=3).contains(&minimum) || !(minimum..=3).contains(&maximum) {
        return Err(DecisionDataError::InvalidReward);
    }
    let rarity = RewardRarityRange {
        minimum: u8::try_from(minimum).map_err(|_| DecisionDataError::InvalidReward)?,
        maximum: u8::try_from(maximum).map_err(|_| DecisionDataError::InvalidReward)?,
    };
    let count = u16::try_from(amount).map_err(|_| DecisionDataError::InvalidReward)?;
    Ok(match kind {
        DuDecisionRewardKind::Curios => DecisionReward::Curios { count, rarity },
        DuDecisionRewardKind::Blessings => DecisionReward::Blessings { count, rarity },
        DuDecisionRewardKind::Fragments => return Err(DecisionDataError::InvalidReward),
    })
}
