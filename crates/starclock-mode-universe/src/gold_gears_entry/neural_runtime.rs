//! Typed Neural Network acquisition, Activity effects and battle projections.

use std::collections::BTreeSet;

use serde::Deserialize;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use crate::{
    digest::Encoder,
    gold_gears_unique::{GoldAndGearsUniqueCatalog, NeuralNode},
};

use super::{
    GoldAndGearsEntryError,
    api::{GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance},
    state_layout::{
        PLANE_ACTION_POINTS_KEY, PLANE_STATE_SLOT, PROGRESSION_NEURAL_REBOOT_BATTLES_KEY,
        PROGRESSION_SLOT, RESOURCE_DICE_REROLLS_KEY, RUN_RESOURCES_SLOT,
    },
};

pub const GOLD_AND_GEARS_NEURAL_RUNTIME_REVISION: &str = "gold-and-gears-neural-runtime-v1";

const NEURAL_CURRENCY_SOURCE: &str = "281013";
const NEURAL_PLANE_START_PROGRAM_BASE: u32 = 0x4800_0000;
const NEURAL_BATTLE_ENTRY_PROGRAM_ID: u32 = 0x4800_0010;
const SLOT_UPGRADE_POLICY: &str = "neural-network-slot-upgrade-target-v1";
const REROLL_POLICY: &str = "neural-network-reroll-empty-candidate-v1";
const BASELINE_TRAILBLAZE_BONUSES: [&str; 3] = [
    "gold-gears.trailblaze-bonus.201",
    "gold-gears.trailblaze-bonus.202",
    "gold-gears.trailblaze-bonus.203",
];

/// Closed battle-stat target set authored by the 40 Neural nodes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsNeuralBattleStat {
    PartyAttackRatio = 0,
    PathResonanceDamageRatio = 1,
    PartyMaximumHpRatio = 2,
    PartyDefenseRatio = 3,
    PartySpeedRatio = 4,
    PartyEffectHitRateRatio = 5,
    PartyEffectResistanceRatio = 6,
    PartyCriticalRateRatio = 7,
    PartyCriticalDamageRatio = 8,
    PartyDamageTakenReductionRatio = 9,
    PartyDamageDealtRatio = 10,
}

impl GoldAndGearsNeuralBattleStat {
    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "party.attack_ratio" => Self::PartyAttackRatio,
            "path-resonance.damage_ratio" => Self::PathResonanceDamageRatio,
            "party.max_hp_ratio" => Self::PartyMaximumHpRatio,
            "party.defense_ratio" => Self::PartyDefenseRatio,
            "party.speed_ratio" => Self::PartySpeedRatio,
            "party.effect_hit_rate_ratio" => Self::PartyEffectHitRateRatio,
            "party.effect_resistance_ratio" => Self::PartyEffectResistanceRatio,
            "party.critical_rate_ratio" => Self::PartyCriticalRateRatio,
            "party.critical_damage_ratio" => Self::PartyCriticalDamageRatio,
            "party.damage_taken_reduction_ratio" => Self::PartyDamageTakenReductionRatio,
            "party.damage_dealt_ratio" => Self::PartyDamageDealtRatio,
            _ => return None,
        })
    }

    #[must_use]
    pub const fn stable_key(self) -> &'static str {
        match self {
            Self::PartyAttackRatio => "party.attack_ratio",
            Self::PathResonanceDamageRatio => "path-resonance.damage_ratio",
            Self::PartyMaximumHpRatio => "party.max_hp_ratio",
            Self::PartyDefenseRatio => "party.defense_ratio",
            Self::PartySpeedRatio => "party.speed_ratio",
            Self::PartyEffectHitRateRatio => "party.effect_hit_rate_ratio",
            Self::PartyEffectResistanceRatio => "party.effect_resistance_ratio",
            Self::PartyCriticalRateRatio => "party.critical_rate_ratio",
            Self::PartyCriticalDamageRatio => "party.critical_damage_ratio",
            Self::PartyDamageTakenReductionRatio => "party.damage_taken_reduction_ratio",
            Self::PartyDamageDealtRatio => "party.damage_dealt_ratio",
        }
    }
}

/// One immutable additive battle-stat contribution with source provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsNeuralStatContribution {
    source_node: Box<str>,
    stat: GoldAndGearsNeuralBattleStat,
    ratio_scaled: i64,
}

/// One frozen Neural mechanic rule bound to its production executor.
///
/// Bindings retain the exact source identity and evidence classification while
/// the actual values flow through the typed Activity and battle projections
/// exposed by [`GoldAndGearsRuntimeInstance`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsNeuralRuleBinding {
    rule_id: Box<str>,
    owner_node: Box<str>,
    operation: &'static str,
    accuracy: GoldAndGearsNeuralRuleAccuracy,
}

impl GoldAndGearsNeuralRuleBinding {
    #[must_use]
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }

    #[must_use]
    pub fn owner_node(&self) -> &str {
        &self.owner_node
    }

    #[must_use]
    pub const fn operation(&self) -> &'static str {
        self.operation
    }

    #[must_use]
    pub const fn accuracy(&self) -> GoldAndGearsNeuralRuleAccuracy {
        self.accuracy
    }

    /// The frozen partition dispatches every rule through ordinary Activity
    /// programs or immutable combat inputs.
    #[must_use]
    pub const fn executor(&self) -> &'static str {
        "ActivityAndCombatPrograms"
    }
}

/// Truthful runtime accuracy attached to one Neural mechanic rule.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GoldAndGearsNeuralRuleAccuracy {
    ExactPublic,
    ProjectPolicy,
}

impl GoldAndGearsNeuralStatContribution {
    #[must_use]
    pub fn source_node(&self) -> &str {
        &self.source_node
    }

    #[must_use]
    pub const fn stat(&self) -> GoldAndGearsNeuralBattleStat {
        self.stat
    }

    /// Exact six-decimal ratio in millionths.
    #[must_use]
    pub const fn ratio_scaled(&self) -> i64 {
        self.ratio_scaled
    }
}

/// Caller-owned facts needed by the conditional First Plane entry effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldAndGearsNeuralBattleEntryContext {
    plane_layer: u8,
    boss: bool,
    previous_first_plane_completed: bool,
}

impl GoldAndGearsNeuralBattleEntryContext {
    #[must_use]
    pub const fn new(plane_layer: u8, boss: bool, previous_first_plane_completed: bool) -> Self {
        Self {
            plane_layer,
            boss,
            previous_first_plane_completed,
        }
    }
}

/// Immutable battle input paired with its pre-battle Activity accounting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsNeuralBattleEntry {
    source_node: Box<str>,
    target_max_hp_ratio_scaled: i64,
    accounting: ActivityProgramDefinition,
}

impl GoldAndGearsNeuralBattleEntry {
    #[must_use]
    pub fn source_node(&self) -> &str {
        &self.source_node
    }

    /// Exact six-decimal TargetMaxHp ratio in millionths.
    #[must_use]
    pub const fn target_max_hp_ratio_scaled(&self) -> i64 {
        self.target_max_hp_ratio_scaled
    }

    #[must_use]
    pub const fn accounting_program(&self) -> &ActivityProgramDefinition {
        &self.accounting
    }
}

/// Account-progression purchase result. It does not mutate live run state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsNeuralAcquisition {
    node: Box<str>,
    source_item_id: u32,
    cost: u32,
    remaining: u32,
}

impl GoldAndGearsNeuralAcquisition {
    #[must_use]
    pub fn node(&self) -> &str {
        &self.node
    }

    #[must_use]
    pub const fn source_item_id(&self) -> u32 {
        self.source_item_id
    }

    #[must_use]
    pub const fn cost(&self) -> u32 {
        self.cost
    }

    #[must_use]
    pub const fn remaining(&self) -> u32 {
        self.remaining
    }
}

#[derive(Clone, Debug)]
pub(super) struct NeuralRuntimeCatalog {
    nodes: Box<[RuntimeNeuralNode]>,
}

#[derive(Clone, Debug)]
struct RuntimeNeuralNode {
    id: u32,
    source_id: Box<str>,
    topological_index: u16,
    key: Box<str>,
    prerequisites: Box<[Box<str>]>,
    cost_item: u32,
    cost: u32,
    effect: NeuralEffect,
}

#[derive(Clone, Debug)]
enum NeuralEffect {
    BattleStat {
        stat: GoldAndGearsNeuralBattleStat,
        ratio_scaled: i64,
    },
    FixedEntryDamage {
        ratio_scaled: i64,
        eligible_battle_limit: u32,
    },
    DiceSlotUpgrade {
        target: Box<str>,
        from: u8,
        to: u8,
    },
    TrailblazeBonusUnlock(Box<str>),
    InitialCountdown(i64),
    BlessingStoreOffers(i64),
    NextPlaneRerolls(i64),
    RerollExclusion,
}

#[derive(Clone, Debug)]
pub(super) struct CompiledNeuralRuntime {
    rule_bindings: Box<[GoldAndGearsNeuralRuleBinding]>,
    battle_stats: Box<[GoldAndGearsNeuralStatContribution]>,
    fixed_entry: Option<RuntimeFixedEntry>,
    trailblaze_bonus_unlocks: Box<[Box<str>]>,
    initial_countdown: i64,
    blessing_store_offers: i64,
    next_plane_rerolls: i64,
    digest: [u8; 32],
}

#[derive(Clone, Debug)]
struct RuntimeFixedEntry {
    source_node: Box<str>,
    ratio_scaled: i64,
    eligible_battle_limit: u32,
}

impl NeuralRuntimeCatalog {
    pub(super) fn compile(
        catalog: &GoldAndGearsUniqueCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let mut nodes = Vec::with_capacity(catalog.neural_nodes.len());
        for node in &catalog.neural_nodes {
            let costs = serde_json::from_str::<Vec<NeuralCost>>(&node.costs_json)
                .map_err(|_| GoldAndGearsEntryError::InvalidNeuralRuntime)?;
            let [cost] = costs.as_slice() else {
                return Err(GoldAndGearsEntryError::InvalidNeuralRuntime);
            };
            let cost_item = parse_positive_u32(&cost.source_item_id)?;
            let cost_value = parse_positive_u32(&cost.amount)?;
            if cost.source_item_id.as_ref() != NEURAL_CURRENCY_SOURCE
                || node.disposition.as_ref() != "MechanicallyRelevant"
                || !node.external_unlocks.is_empty()
            {
                return Err(GoldAndGearsEntryError::InvalidNeuralRuntime);
            }
            let contributions =
                serde_json::from_str::<Vec<NeuralContribution>>(&node.effect_contributions_json)
                    .map_err(|_| GoldAndGearsEntryError::InvalidNeuralRuntime)?;
            let [contribution] = contributions.as_slice() else {
                return Err(GoldAndGearsEntryError::InvalidNeuralRuntime);
            };
            let effect = decode_effect(node, contribution)?;
            nodes.push(RuntimeNeuralNode {
                id: node.identity.id.0,
                source_id: node.identity.source_id.clone(),
                topological_index: node.topological_index,
                key: node.identity.stable_key.clone(),
                prerequisites: node.prerequisites.clone(),
                cost_item,
                cost: cost_value,
                effect,
            });
        }
        nodes.sort_unstable_by_key(|node| node.topological_index);
        validate_runtime_nodes(&nodes)?;
        Ok(Self {
            nodes: nodes.into_boxed_slice(),
        })
    }

    pub(super) fn select(
        &self,
        selected: &[&NeuralNode],
    ) -> Result<CompiledNeuralRuntime, GoldAndGearsEntryError> {
        let mut rule_bindings = Vec::with_capacity(selected.len());
        let mut battle_stats = Vec::new();
        let mut fixed_entry = None;
        let mut trailblaze_bonus_unlocks = BTreeSet::new();
        let mut initial_countdown = 0_i64;
        let mut blessing_store_offers = 0_i64;
        let mut next_plane_rerolls = 0_i64;
        let mut encoder = Encoder::new(b"starclock.gold-and-gears.neural-selected.v1");
        encoder.u32(
            u32::try_from(selected.len())
                .map_err(|_| GoldAndGearsEntryError::InvalidNeuralRuntime)?,
        );
        for authored in selected {
            let node = self
                .nodes
                .iter()
                .find(|node| node.id == authored.identity.id.0)
                .ok_or(GoldAndGearsEntryError::InvalidNeuralRuntime)?;
            encoder.text(&node.key);
            encoder.u32(node.cost_item);
            encoder.u32(node.cost);
            encode_effect(&mut encoder, &node.effect);
            rule_bindings.push(rule_binding(node));
            match &node.effect {
                NeuralEffect::BattleStat { stat, ratio_scaled } => {
                    battle_stats.push(GoldAndGearsNeuralStatContribution {
                        source_node: node.key.clone(),
                        stat: *stat,
                        ratio_scaled: *ratio_scaled,
                    });
                }
                NeuralEffect::FixedEntryDamage {
                    ratio_scaled,
                    eligible_battle_limit,
                } => {
                    if fixed_entry.is_some() {
                        return Err(GoldAndGearsEntryError::InvalidNeuralRuntime);
                    }
                    fixed_entry = Some(RuntimeFixedEntry {
                        source_node: node.key.clone(),
                        ratio_scaled: *ratio_scaled,
                        eligible_battle_limit: *eligible_battle_limit,
                    });
                }
                NeuralEffect::TrailblazeBonusUnlock(target) => {
                    if !trailblaze_bonus_unlocks.insert(target.clone()) {
                        return Err(GoldAndGearsEntryError::InvalidNeuralRuntime);
                    }
                }
                NeuralEffect::InitialCountdown(value) => {
                    initial_countdown = checked_add(initial_countdown, *value)?;
                }
                NeuralEffect::BlessingStoreOffers(value) => {
                    blessing_store_offers = checked_add(blessing_store_offers, *value)?;
                }
                NeuralEffect::NextPlaneRerolls(value) => {
                    next_plane_rerolls = checked_add(next_plane_rerolls, *value)?;
                }
                NeuralEffect::DiceSlotUpgrade { .. } | NeuralEffect::RerollExclusion => {}
            }
        }
        Ok(CompiledNeuralRuntime {
            rule_bindings: rule_bindings.into_boxed_slice(),
            battle_stats: battle_stats.into_boxed_slice(),
            fixed_entry,
            trailblaze_bonus_unlocks: trailblaze_bonus_unlocks.into_iter().collect(),
            initial_countdown,
            blessing_store_offers,
            next_plane_rerolls,
            digest: encoder.finish(),
        })
    }

    fn acquisition(
        &self,
        unlocked: &[String],
        target: &str,
        available: u32,
    ) -> Result<GoldAndGearsNeuralAcquisition, GoldAndGearsEntryError> {
        let mut selected = BTreeSet::new();
        for key in unlocked {
            let node = self
                .nodes
                .iter()
                .find(|node| node.key.as_ref() == key)
                .ok_or_else(|| GoldAndGearsEntryError::UnknownNeuralNode(key.clone().into()))?;
            if !selected.insert(node.key.as_ref()) {
                return Err(GoldAndGearsEntryError::DuplicateNeuralNode(
                    node.key.clone(),
                ));
            }
        }
        for node in self
            .nodes
            .iter()
            .filter(|node| selected.contains(node.key.as_ref()))
        {
            if let Some(prerequisite) = node
                .prerequisites
                .iter()
                .find(|prerequisite| !selected.contains(prerequisite.as_ref()))
            {
                return Err(GoldAndGearsEntryError::MissingNeuralPrerequisite {
                    node: node.key.clone(),
                    prerequisite: prerequisite.clone(),
                });
            }
        }
        let target = self
            .nodes
            .iter()
            .find(|node| node.key.as_ref() == target)
            .ok_or_else(|| GoldAndGearsEntryError::UnknownNeuralNode(target.into()))?;
        if selected.contains(target.key.as_ref()) {
            return Err(GoldAndGearsEntryError::NeuralAlreadyAcquired(
                target.key.clone(),
            ));
        }
        if let Some(prerequisite) = target
            .prerequisites
            .iter()
            .find(|prerequisite| !selected.contains(prerequisite.as_ref()))
        {
            return Err(GoldAndGearsEntryError::MissingNeuralPrerequisite {
                node: target.key.clone(),
                prerequisite: prerequisite.clone(),
            });
        }
        let remaining = available.checked_sub(target.cost).ok_or(
            GoldAndGearsEntryError::InsufficientNeuralCurrency {
                required: target.cost,
                available,
            },
        )?;
        Ok(GoldAndGearsNeuralAcquisition {
            node: target.key.clone(),
            source_item_id: target.cost_item,
            cost: target.cost,
            remaining,
        })
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, u32) {
        let total_cost = self
            .nodes
            .iter()
            .try_fold(0_u32, |total, node| total.checked_add(node.cost))
            .expect("validated 40-node Neural cost total fits u32");
        (
            self.nodes.len(),
            self.nodes
                .iter()
                .filter(|node| matches!(node.effect, NeuralEffect::BattleStat { .. }))
                .count(),
            total_cost,
        )
    }
}

impl CompiledNeuralRuntime {
    pub(super) fn allows_trailblaze_bonus(&self, key: &str) -> bool {
        BASELINE_TRAILBLAZE_BONUSES.contains(&key)
            || self
                .trailblaze_bonus_unlocks
                .binary_search_by(|candidate| candidate.as_ref().cmp(key))
                .is_ok()
    }

    fn compile_plane_start(
        &self,
        plane_layer: u8,
    ) -> Result<Option<ActivityProgramDefinition>, GoldAndGearsEntryError> {
        if !(1..=3).contains(&plane_layer) {
            return Err(GoldAndGearsEntryError::InvalidPlaneLayer);
        }
        let mut operations = Vec::new();
        if self.initial_countdown != 0 {
            operations.push(add_counter(
                PLANE_STATE_SLOT,
                PLANE_ACTION_POINTS_KEY,
                self.initial_countdown,
            ));
        }
        if plane_layer > 1 && self.next_plane_rerolls != 0 {
            operations.push(add_counter(
                RUN_RESOURCES_SLOT,
                RESOURCE_DICE_REROLLS_KEY,
                self.next_plane_rerolls,
            ));
        }
        if operations.is_empty() {
            return Ok(None);
        }
        program(
            NEURAL_PLANE_START_PROGRAM_BASE + u32::from(plane_layer),
            operations,
        )
        .map(Some)
    }

    fn compile_battle_entry(
        &self,
        state: &ActivityTransactionState,
        context: GoldAndGearsNeuralBattleEntryContext,
    ) -> Result<Option<GoldAndGearsNeuralBattleEntry>, GoldAndGearsEntryError> {
        if !(1..=3).contains(&context.plane_layer) {
            return Err(GoldAndGearsEntryError::InvalidPlaneLayer);
        }
        let Some(effect) = &self.fixed_entry else {
            return Ok(None);
        };
        if context.plane_layer != 1 || context.boss || !context.previous_first_plane_completed {
            return Ok(None);
        }
        let consumed = counter_value(
            state,
            PROGRESSION_SLOT,
            PROGRESSION_NEURAL_REBOOT_BATTLES_KEY,
        )?;
        if consumed >= i64::from(effect.eligible_battle_limit) {
            return Ok(None);
        }
        let operations = vec![
            ActivityOperation::Require(ActivityCondition::LessThan(
                counter(PROGRESSION_SLOT, PROGRESSION_NEURAL_REBOOT_BATTLES_KEY),
                integer(i64::from(effect.eligible_battle_limit)),
            )),
            add_counter(PROGRESSION_SLOT, PROGRESSION_NEURAL_REBOOT_BATTLES_KEY, 1),
        ];
        Ok(Some(GoldAndGearsNeuralBattleEntry {
            source_node: effect.source_node.clone(),
            target_max_hp_ratio_scaled: effect.ratio_scaled,
            accounting: program(NEURAL_BATTLE_ENTRY_PROGRAM_ID, operations)?,
        }))
    }
}

impl GoldAndGearsRuntimeFactory {
    /// Compiles one account-owned Neural purchase without mutating run state.
    ///
    /// The caller commits the returned exact item/cost plan in the account
    /// progression owner. Existing unlocks must be prerequisite-closed.
    pub fn compile_neural_acquisition(
        &self,
        unlocked: &[String],
        target: &str,
        available: u32,
    ) -> Result<GoldAndGearsNeuralAcquisition, GoldAndGearsEntryError> {
        self.neural.acquisition(unlocked, target, available)
    }
}

impl GoldAndGearsRuntimeInstance {
    /// Selected Neural rules in canonical source-node order, each bound to its
    /// production executor and truthful accuracy classification.
    #[must_use]
    pub fn neural_rule_bindings(&self) -> &[GoldAndGearsNeuralRuleBinding] {
        &self.neural_runtime.rule_bindings
    }

    /// Immutable additive Neural stat projections in source-node order.
    #[must_use]
    pub fn neural_battle_stat_contributions(&self) -> &[GoldAndGearsNeuralStatContribution] {
        &self.neural_runtime.battle_stats
    }

    /// Canonical digest of all selected Neural effects and exact source costs.
    #[must_use]
    pub const fn neural_contribution_digest(&self) -> [u8; 32] {
        self.neural_runtime.digest
    }

    /// Selected Neural unlocks beyond the three baseline Trailblaze Bonuses.
    pub fn neural_trailblaze_bonus_unlocks(&self) -> impl ExactSizeIterator<Item = &str> {
        self.neural_runtime
            .trailblaze_bonus_unlocks
            .iter()
            .map(Box::as_ref)
    }

    /// Extra purchasable Blessings offered on Transaction-domain entry.
    #[must_use]
    pub const fn neural_blessing_store_offer_count(&self) -> i64 {
        self.neural_runtime.blessing_store_offers
    }

    /// Applies selected Neural plane-start effects through ordinary counters.
    pub fn compile_neural_plane_start(
        &self,
        plane_layer: u8,
    ) -> Result<Option<ActivityProgramDefinition>, GoldAndGearsEntryError> {
        self.neural_runtime.compile_plane_start(plane_layer)
    }

    /// Projects conditional First Plane entry damage and its bounded Activity
    /// accounting. The battle receives only the immutable ratio.
    pub fn compile_neural_battle_entry(
        &self,
        state: &ActivityTransactionState,
        context: GoldAndGearsNeuralBattleEntryContext,
    ) -> Result<Option<GoldAndGearsNeuralBattleEntry>, GoldAndGearsEntryError> {
        self.neural_runtime.compile_battle_entry(state, context)
    }
}

fn rule_binding(node: &RuntimeNeuralNode) -> GoldAndGearsNeuralRuleBinding {
    let (operation, accuracy) = match node.effect {
        NeuralEffect::BattleStat { .. } => (
            "AddBattleStatRatio",
            GoldAndGearsNeuralRuleAccuracy::ExactPublic,
        ),
        NeuralEffect::FixedEntryDamage { .. } => (
            "ApplyFixedEntryDamage",
            GoldAndGearsNeuralRuleAccuracy::ExactPublic,
        ),
        NeuralEffect::DiceSlotUpgrade { .. } => (
            "UpgradeDiceFaceSlot",
            GoldAndGearsNeuralRuleAccuracy::ProjectPolicy,
        ),
        NeuralEffect::TrailblazeBonusUnlock(_) => (
            "UnlockTrailblazeBonus",
            GoldAndGearsNeuralRuleAccuracy::ExactPublic,
        ),
        NeuralEffect::InitialCountdown(_) => (
            "AddInitialCountdown",
            GoldAndGearsNeuralRuleAccuracy::ExactPublic,
        ),
        NeuralEffect::BlessingStoreOffers(_) => (
            "AddBlessingStoreOfferCount",
            GoldAndGearsNeuralRuleAccuracy::ExactPublic,
        ),
        NeuralEffect::NextPlaneRerolls(_) => (
            "AddRerollAttempts",
            GoldAndGearsNeuralRuleAccuracy::ExactPublic,
        ),
        NeuralEffect::RerollExclusion => (
            "ExcludePreviousRerollResult",
            GoldAndGearsNeuralRuleAccuracy::ProjectPolicy,
        ),
    };
    GoldAndGearsNeuralRuleBinding {
        rule_id: format!("gold-gears.rule.neural-network.{}", node.source_id).into(),
        owner_node: node.key.clone(),
        operation,
        accuracy,
    }
}

fn decode_effect(
    node: &NeuralNode,
    contribution: &NeuralContribution,
) -> Result<NeuralEffect, GoldAndGearsEntryError> {
    if contribution.mechanism_quality.as_ref() != "ExactStructured" {
        return Err(GoldAndGearsEntryError::InvalidNeuralRuntime);
    }
    let effect = match contribution.operation.as_ref() {
        "AddBattleStatRatio"
            if node.effect_domain.as_ref() == "Battle"
                && contribution.scope.as_deref() == Some("Battle")
                && contribution.unit.as_deref() == Some("Ratio")
                && contribution.stacking.as_deref() == Some("AdditiveContribution") =>
        {
            NeuralEffect::BattleStat {
                stat: GoldAndGearsNeuralBattleStat::parse(required(&contribution.target)?)
                    .ok_or(GoldAndGearsEntryError::InvalidNeuralRuntime)?,
                ratio_scaled: positive_scaled(required(&contribution.value)?)?,
            }
        }
        "ApplyFixedEntryDamage"
            if node.effect_domain.as_ref() == "ActivityAndBattle"
                && contribution.scope.as_deref() == Some("ActivityAndBattle")
                && contribution.target.as_deref() == Some("all-enemies")
                && contribution.timing.as_deref() == Some("BattleEntry")
                && contribution.damage_basis.as_deref() == Some("TargetMaxHpRatio")
                && contribution.eligible_section.as_deref() == Some("FirstPlane")
                && contribution.excluded_battle_kind.as_deref() == Some("Boss")
                && contribution.condition.as_deref()
                    == Some("previous-challenge-first-plane-completed") =>
        {
            NeuralEffect::FixedEntryDamage {
                ratio_scaled: positive_scaled(required(&contribution.value)?)?,
                eligible_battle_limit: parse_positive_u32(required(
                    &contribution.eligible_battle_limit,
                )?)?,
            }
        }
        "UpgradeDiceFaceSlot"
            if node.effect_domain.as_ref() == "Activity"
                && contribution.scope.as_deref() == Some("Activity")
                && contribution.unit.as_deref() == Some("Rarity")
                && valid_slot_policy(contribution.target_policy.as_ref()) =>
        {
            NeuralEffect::DiceSlotUpgrade {
                target: contribution
                    .target
                    .clone()
                    .ok_or(GoldAndGearsEntryError::InvalidNeuralRuntime)?,
                from: contribution
                    .from_max_rarity
                    .ok_or(GoldAndGearsEntryError::InvalidNeuralRuntime)?,
                to: contribution
                    .to_max_rarity
                    .ok_or(GoldAndGearsEntryError::InvalidNeuralRuntime)?,
            }
        }
        "UnlockTrailblazeBonus"
            if node.effect_domain.as_ref() == "Activity"
                && contribution.scope.as_deref() == Some("Activity") =>
        {
            let target = contribution
                .target
                .clone()
                .ok_or(GoldAndGearsEntryError::InvalidNeuralRuntime)?;
            if !matches!(
                target.as_ref(),
                "gold-gears.trailblaze-bonus.204" | "gold-gears.trailblaze-bonus.205"
            ) {
                return Err(GoldAndGearsEntryError::InvalidNeuralRuntime);
            }
            NeuralEffect::TrailblazeBonusUnlock(target)
        }
        "AddInitialCountdown"
            if activity_count(contribution, "section.countdown.initial", None) =>
        {
            NeuralEffect::InitialCountdown(integer_text(required(&contribution.value)?)?)
        }
        "AddBlessingStoreOfferCount"
            if activity_count(
                contribution,
                "transaction.blessing-store.purchasable-blessings",
                Some("EnterTransactionDomain"),
            ) =>
        {
            NeuralEffect::BlessingStoreOffers(integer_text(required(&contribution.value)?)?)
        }
        "AddRerollAttempts"
            if activity_count(contribution, "dice.reroll-attempts", Some("EnterNextPlane")) =>
        {
            NeuralEffect::NextPlaneRerolls(integer_text(required(&contribution.value)?)?)
        }
        "ExcludePreviousRerollResult"
            if node.effect_domain.as_ref() == "Activity"
                && contribution.scope.as_deref() == Some("Activity")
                && contribution.target.as_deref() == Some("dice-face-result")
                && contribution.exclusion.as_deref() == Some("PreviousResult")
                && valid_reroll_policy(contribution.selection_policy.as_ref()) =>
        {
            NeuralEffect::RerollExclusion
        }
        _ => return Err(GoldAndGearsEntryError::InvalidNeuralRuntime),
    };
    Ok(effect)
}

fn validate_runtime_nodes(nodes: &[RuntimeNeuralNode]) -> Result<(), GoldAndGearsEntryError> {
    let mut counts = [0_usize; 8];
    for node in nodes {
        let index = match node.effect {
            NeuralEffect::BattleStat { .. } => 0,
            NeuralEffect::FixedEntryDamage { .. } => 1,
            NeuralEffect::DiceSlotUpgrade { .. } => 2,
            NeuralEffect::TrailblazeBonusUnlock(_) => 3,
            NeuralEffect::InitialCountdown(_) => 4,
            NeuralEffect::BlessingStoreOffers(_) => 5,
            NeuralEffect::NextPlaneRerolls(_) => 6,
            NeuralEffect::RerollExclusion => 7,
        };
        counts[index] = counts[index]
            .checked_add(1)
            .ok_or(GoldAndGearsEntryError::InvalidNeuralRuntime)?;
    }
    let total_cost = nodes.iter().try_fold(0_u32, |total, node| {
        total
            .checked_add(node.cost)
            .ok_or(GoldAndGearsEntryError::InvalidNeuralRuntime)
    })?;
    if nodes.len() != 40
        || counts != [30, 1, 3, 2, 1, 1, 1, 1]
        || nodes
            .iter()
            .enumerate()
            .any(|(index, node)| usize::from(node.topological_index) != index + 1)
        || nodes.windows(2).any(|pair| pair[0].id == pair[1].id)
        || total_cost != 31_250
    {
        return Err(GoldAndGearsEntryError::InvalidNeuralRuntime);
    }
    Ok(())
}

fn valid_slot_policy(policy: Option<&UpgradeTargetPolicy>) -> bool {
    policy.is_some_and(|policy| {
        policy.policy_id.as_ref() == SLOT_UPGRADE_POLICY
            && policy.evidence_quality.as_ref() == "ProjectPolicy"
            && policy.mapping_basis.as_ref() == "released-slot-capability-plus-stable-slot-order"
            && !policy.replacement_condition.is_empty()
    })
}

fn valid_reroll_policy(policy: Option<&RerollSelectionPolicy>) -> bool {
    policy.is_some_and(|policy| {
        policy.policy_id.as_ref() == REROLL_POLICY
            && policy.evidence_quality.as_ref() == "ProjectPolicy"
            && policy.candidate_order.as_ref() == "stable-dice-face-id-ascending"
            && policy.draw_mode.as_ref() == "seeded-from-eligible-candidates"
            && policy.empty_candidate_behavior.as_ref() == "KeepPreviousAndConsumeAttempt"
            && !policy.replacement_condition.is_empty()
    })
}

fn activity_count(contribution: &NeuralContribution, target: &str, timing: Option<&str>) -> bool {
    contribution.scope.as_deref() == Some("Activity")
        && contribution.target.as_deref() == Some(target)
        && contribution.unit.as_deref() == Some("Count")
        && contribution.timing.as_deref() == timing
}

fn encode_effect(encoder: &mut Encoder, effect: &NeuralEffect) {
    match effect {
        NeuralEffect::BattleStat { stat, ratio_scaled } => {
            encoder.u8(0);
            encoder.u8(*stat as u8);
            encoder.i64(*ratio_scaled);
        }
        NeuralEffect::FixedEntryDamage {
            ratio_scaled,
            eligible_battle_limit,
        } => {
            encoder.u8(1);
            encoder.i64(*ratio_scaled);
            encoder.u32(*eligible_battle_limit);
        }
        NeuralEffect::DiceSlotUpgrade { target, from, to } => {
            encoder.u8(2);
            encoder.text(target);
            encoder.u8(*from);
            encoder.u8(*to);
        }
        NeuralEffect::TrailblazeBonusUnlock(target) => {
            encoder.u8(3);
            encoder.text(target);
        }
        NeuralEffect::InitialCountdown(value) => {
            encoder.u8(4);
            encoder.i64(*value);
        }
        NeuralEffect::BlessingStoreOffers(value) => {
            encoder.u8(5);
            encoder.i64(*value);
        }
        NeuralEffect::NextPlaneRerolls(value) => {
            encoder.u8(6);
            encoder.i64(*value);
        }
        NeuralEffect::RerollExclusion => encoder.u8(7),
    }
}

fn parse_positive_u32(value: &str) -> Result<u32, GoldAndGearsEntryError> {
    value
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or(GoldAndGearsEntryError::InvalidNeuralRuntime)
}

fn integer_text(value: &str) -> Result<i64, GoldAndGearsEntryError> {
    value
        .parse::<i64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or(GoldAndGearsEntryError::InvalidNeuralRuntime)
}

fn positive_scaled(value: &str) -> Result<i64, GoldAndGearsEntryError> {
    let (whole, fraction) = value.split_once('.').map_or((value, ""), |parts| parts);
    if value.starts_with('-') || fraction.len() > 6 {
        return Err(GoldAndGearsEntryError::InvalidNeuralRuntime);
    }
    let whole = whole
        .parse::<i64>()
        .map_err(|_| GoldAndGearsEntryError::InvalidNeuralRuntime)?;
    let mut fraction = fraction.to_owned();
    fraction.extend(core::iter::repeat_n('0', 6 - fraction.len()));
    let fraction = fraction
        .parse::<i64>()
        .map_err(|_| GoldAndGearsEntryError::InvalidNeuralRuntime)?;
    whole
        .checked_mul(1_000_000)
        .and_then(|value| value.checked_add(fraction))
        .filter(|value| *value > 0)
        .ok_or(GoldAndGearsEntryError::InvalidNeuralRuntime)
}

fn required(value: &Option<Box<str>>) -> Result<&str, GoldAndGearsEntryError> {
    value
        .as_deref()
        .ok_or(GoldAndGearsEntryError::InvalidNeuralRuntime)
}

fn checked_add(left: i64, right: i64) -> Result<i64, GoldAndGearsEntryError> {
    left.checked_add(right)
        .ok_or(GoldAndGearsEntryError::InvalidNeuralRuntime)
}

fn add_counter(slot_id: u32, key: u64, delta: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(slot_id),
        key,
        delta: integer(delta),
    }
}

fn counter(slot_id: u32, key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: slot(slot_id),
        key,
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("static Gold and Gears slot is non-zero")
}

fn counter_value(
    state: &ActivityTransactionState,
    slot_id: u32,
    key: u64,
) -> Result<i64, GoldAndGearsEntryError> {
    match state.slot(slot(slot_id)) {
        Some(ActivityValue::BoundedCounterMap(values)) => Ok(values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1)),
        _ => Err(GoldAndGearsEntryError::InvalidNeuralRuntime),
    }
}

fn program(
    raw: u32,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(raw).expect("static Gold and Gears program ID is non-zero"),
        operations,
    )
    .map_err(|_| GoldAndGearsEntryError::InvalidNeuralRuntime)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NeuralCost {
    source_item_id: Box<str>,
    amount: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NeuralContribution {
    operation: Box<str>,
    #[serde(default)]
    scope: Option<Box<str>>,
    #[serde(default)]
    target: Option<Box<str>>,
    #[serde(default)]
    value: Option<Box<str>>,
    #[serde(default)]
    unit: Option<Box<str>>,
    #[serde(default)]
    stacking: Option<Box<str>>,
    #[serde(default)]
    timing: Option<Box<str>>,
    #[serde(default)]
    damage_basis: Option<Box<str>>,
    #[serde(default)]
    eligible_battle_limit: Option<Box<str>>,
    #[serde(default)]
    eligible_section: Option<Box<str>>,
    #[serde(default)]
    excluded_battle_kind: Option<Box<str>>,
    #[serde(default)]
    condition: Option<Box<str>>,
    #[serde(default)]
    from_max_rarity: Option<u8>,
    #[serde(default)]
    to_max_rarity: Option<u8>,
    #[serde(default)]
    exclusion: Option<Box<str>>,
    #[serde(default)]
    target_policy: Option<UpgradeTargetPolicy>,
    #[serde(default)]
    selection_policy: Option<RerollSelectionPolicy>,
    mechanism_quality: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpgradeTargetPolicy {
    policy_id: Box<str>,
    evidence_quality: Box<str>,
    mapping_basis: Box<str>,
    replacement_condition: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RerollSelectionPolicy {
    policy_id: Box<str>,
    evidence_quality: Box<str>,
    candidate_order: Box<str>,
    draw_mode: Box<str>,
    empty_candidate_behavior: Box<str>,
    replacement_condition: Box<str>,
}
