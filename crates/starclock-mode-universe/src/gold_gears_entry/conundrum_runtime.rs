//! Typed Conundrum composition and versioned battle-policy projections.

use serde::Deserialize;

use crate::{
    digest::Encoder,
    gold_gears_unique::{ConundrumLevel, GoldAndGearsUniqueCatalog},
};

use super::{
    GoldAndGearsEntryError,
    api::GoldAndGearsRuntimeInstance,
    conundrum_policy::{
        GOLD_AND_GEARS_CONUNDRUM_POLICY_ACCURACY,
        GOLD_AND_GEARS_CONUNDRUM_POLICY_REPLACEMENT_CONDITION,
        GOLD_AND_GEARS_CONUNDRUM_POLICY_REVISION, GoldAndGearsBerserkPolicy,
        GoldAndGearsEliteBossResponsePolicy, GoldAndGearsEnemyStatPolicy, berserk_policy,
        elite_boss_response_policy, enemy_stat_policy,
    },
};

/// Runtime revision for both Conundrum tracks.
pub const GOLD_AND_GEARS_CONUNDRUM_RUNTIME_REVISION: &str = "gold-and-gears-conundrum-runtime-v1";

const SOURCE_POLICY: &str = "conundrum-unreleased-numeric-bindings-v1";

/// Runtime owner of a Conundrum contribution.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GoldAndGearsConundrumScope {
    Activity,
    Battle,
    ActivityAndBattle,
}

/// Closed runtime effect set produced by the 12 released definitions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoldAndGearsConundrumEffect {
    EnemyStat(GoldAndGearsEnemyStatPolicy),
    EnhancedBerserk,
    EliteBossResponse(GoldAndGearsEliteBossResponsePolicy),
    FormationExtrapolationCount(u8),
    SecondPlaneBossPhaseThree(Box<[Box<str>]>),
    BlessingResetCost(i64),
    InitialResources {
        countdown_delta: i64,
        dice_reroll_delta: i64,
        cosmic_fragment_delta: i64,
    },
    NegativeCuriosPerPlane(u8),
    EffectiveBlessingsPerPath {
        delta: i64,
        minimum: u8,
    },
}

/// One selected, source-ordered Conundrum contribution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsConundrumContribution {
    source_level: Box<str>,
    scope: GoldAndGearsConundrumScope,
    effect: GoldAndGearsConundrumEffect,
}

impl GoldAndGearsConundrumContribution {
    #[must_use]
    pub fn source_level(&self) -> &str {
        &self.source_level
    }

    #[must_use]
    pub const fn scope(&self) -> GoldAndGearsConundrumScope {
        self.scope
    }

    #[must_use]
    pub const fn effect(&self) -> &GoldAndGearsConundrumEffect {
        &self.effect
    }
}

#[derive(Clone, Debug)]
pub(super) struct ConundrumRuntimeCatalog {
    levels: Box<[RuntimeLevel]>,
}

#[derive(Clone, Debug)]
struct RuntimeLevel {
    track: Track,
    level: u8,
    active: Box<[Box<str>]>,
    rule: Box<str>,
    contribution: GoldAndGearsConundrumContribution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Track {
    Stats,
    Auxiliary,
}

#[derive(Clone, Debug)]
pub(super) struct CompiledConundrumRuntime {
    contributions: Box<[GoldAndGearsConundrumContribution]>,
    berserk: GoldAndGearsBerserkPolicy,
    activity: ConundrumActivityEffects,
    digest: [u8; 32],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ConundrumActivityEffects {
    blessing_reset_cost: i64,
    initial_countdown_delta: i64,
    initial_dice_reroll_delta: i64,
    initial_cosmic_fragment_delta: i64,
    negative_curios_per_plane: u8,
    effective_blessings_per_path_delta: i64,
}

#[derive(Deserialize)]
struct UnlockRequirement {
    operation: Box<str>,
    target: Box<str>,
}

#[derive(Deserialize)]
struct RawEffect {
    operation: Box<str>,
    scope: Box<str>,
    target: Box<str>,
    mechanism_quality: Box<str>,
    value: Option<Box<str>>,
    unit: Option<Box<str>>,
    stacking: Option<Box<str>>,
    qualitative_tier: Option<Box<str>>,
    numeric_binding: Option<NumericBinding>,
    encounter_binding_state: Option<Box<str>>,
    encounter_group_ids: Option<Box<[Box<str>]>>,
    countdown_delta: Option<Box<str>>,
    dice_reroll_delta: Option<Box<str>>,
    cosmic_fragment_delta: Option<Box<str>>,
    timing: Option<Box<str>>,
    pool_binding_state: Option<Box<str>>,
    selection_pool_id: Option<Box<str>>,
    unresolved_pool_behavior: Option<Box<str>>,
    minimum_effective_count: Option<Box<str>>,
    toughness_change: Option<Box<str>>,
    berserk_trigger: Option<Box<str>>,
    berserk_response: Option<Box<str>>,
    timing_change: Option<Box<str>>,
    effect_change: Option<Box<str>>,
}

#[derive(Deserialize)]
struct NumericBinding {
    policy_id: Box<str>,
    evidence_quality: Box<str>,
    resolution_state: Box<str>,
    authoritative_behavior: Box<str>,
    unresolved_fields: Box<[Box<str>]>,
    replacement_condition: Box<str>,
}

impl ConundrumRuntimeCatalog {
    pub(super) fn compile(
        catalog: &GoldAndGearsUniqueCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let mut levels = catalog
            .conundrum_levels
            .iter()
            .map(decode_level)
            .collect::<Result<Vec<_>, _>>()?;
        levels.sort_by_key(|entry| {
            (
                match entry.track {
                    Track::Stats => 0_u8,
                    Track::Auxiliary => 1,
                },
                entry.level,
            )
        });
        if levels.len() != 12
            || levels
                .iter()
                .enumerate()
                .any(|(index, entry)| entry.level != u8::try_from(index % 6 + 1).unwrap_or(0))
            || levels[..6].iter().any(|entry| entry.track != Track::Stats)
            || levels[6..]
                .iter()
                .any(|entry| entry.track != Track::Auxiliary)
        {
            return Err(GoldAndGearsEntryError::InvalidConundrumRuntime);
        }
        Ok(Self {
            levels: levels.into_boxed_slice(),
        })
    }

    pub(super) fn select(
        &self,
        stats: u8,
        auxiliary: u8,
    ) -> Result<CompiledConundrumRuntime, GoldAndGearsEntryError> {
        if stats > 6 || auxiliary > 6 {
            return Err(GoldAndGearsEntryError::InvalidConundrumRuntime);
        }
        let mut active_rules = Vec::new();
        for (track, level) in [(Track::Stats, stats), (Track::Auxiliary, auxiliary)] {
            if level == 0 {
                continue;
            }
            let selected = self
                .levels
                .iter()
                .find(|entry| entry.track == track && entry.level == level)
                .ok_or(GoldAndGearsEntryError::InvalidConundrumRuntime)?;
            active_rules.extend(selected.active.iter().cloned());
        }
        if active_rules
            .iter()
            .enumerate()
            .any(|(index, rule)| active_rules[..index].contains(rule))
        {
            return Err(GoldAndGearsEntryError::InvalidConundrumRuntime);
        }
        let contributions = active_rules
            .iter()
            .map(|rule| {
                self.levels
                    .iter()
                    .find(|entry| entry.rule == *rule)
                    .map(|entry| entry.contribution.clone())
                    .ok_or(GoldAndGearsEntryError::InvalidConundrumRuntime)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let activity = activity_effects(&contributions)?;
        let berserk = berserk_policy(stats >= 3);
        let digest = contribution_digest(stats, auxiliary, &contributions, berserk, activity);
        Ok(CompiledConundrumRuntime {
            contributions: contributions.into_boxed_slice(),
            berserk,
            activity,
            digest,
        })
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize) {
        (
            self.levels.len(),
            self.levels
                .iter()
                .filter(|entry| entry.track == Track::Stats)
                .count(),
            self.levels
                .iter()
                .filter(|entry| entry.track == Track::Auxiliary)
                .count(),
        )
    }
}

impl GoldAndGearsRuntimeInstance {
    /// Canonical active contributions after Stats replacement and Auxiliary
    /// cumulative composition.
    #[must_use]
    pub fn conundrum_contributions(&self) -> &[GoldAndGearsConundrumContribution] {
        &self.conundrum_runtime.contributions
    }

    /// Base Berserk for levels 0–2 and enhanced Berserk from Stats level 3.
    #[must_use]
    pub const fn conundrum_berserk_policy(&self) -> GoldAndGearsBerserkPolicy {
        self.conundrum_runtime.berserk
    }

    #[must_use]
    pub const fn conundrum_blessing_reset_cost_delta(&self) -> i64 {
        self.conundrum_runtime.activity.blessing_reset_cost
    }

    #[must_use]
    pub const fn conundrum_initial_countdown_delta(&self) -> i64 {
        self.conundrum_runtime.activity.initial_countdown_delta
    }

    #[must_use]
    pub const fn conundrum_negative_curios_per_plane(&self) -> u8 {
        self.conundrum_runtime.activity.negative_curios_per_plane
    }

    #[must_use]
    pub const fn conundrum_effective_blessings_per_path_delta(&self) -> i64 {
        self.conundrum_runtime
            .activity
            .effective_blessings_per_path_delta
    }

    #[must_use]
    pub const fn conundrum_contribution_digest(&self) -> [u8; 32] {
        self.conundrum_runtime.digest
    }
}

impl CompiledConundrumRuntime {
    pub(super) fn initial_cosmic_fragments(
        &self,
        baseline: i64,
    ) -> Result<i64, GoldAndGearsEntryError> {
        checked_add(baseline, self.activity.initial_cosmic_fragment_delta)
    }

    pub(super) fn initial_dice_rerolls(
        &self,
        baseline: i64,
    ) -> Result<i64, GoldAndGearsEntryError> {
        checked_add(baseline, self.activity.initial_dice_reroll_delta)
    }

    pub(super) const fn berserk_state(&self) -> i64 {
        if self.berserk.enhanced() { 1 } else { 0 }
    }
}

fn decode_level(level: &ConundrumLevel) -> Result<RuntimeLevel, GoldAndGearsEntryError> {
    let track = match level.track.as_ref() {
        "Stats" => Track::Stats,
        "Auxiliary" => Track::Auxiliary,
        _ => return Err(GoldAndGearsEntryError::InvalidConundrumRuntime),
    };
    let expected_mode = match track {
        Track::Stats => "LatestContributionPerSourceTagAtOrBelowSelectedLevel",
        Track::Auxiliary => "AllContributionsAtOrBelowSelectedLevel",
    };
    let unlock: UnlockRequirement = serde_json::from_str(&level.unlock_requirement_json)
        .map_err(|_| GoldAndGearsEntryError::InvalidConundrumRuntime)?;
    let effects: Vec<RawEffect> = serde_json::from_str(&level.effect_contributions_json)
        .map_err(|_| GoldAndGearsEntryError::InvalidConundrumRuntime)?;
    if level.track_cap != 6
        || level.total_cap != 12
        || level.total_formula.as_ref() != "stats_level + auxiliary_level"
        || level.composition_mode.as_ref() != expected_mode
        || unlock.operation.as_ref() != "ClearFormalDifficulty"
        || unlock.target.as_ref() != "gold-gears.area.405"
        || effects.len() != 1
    {
        return Err(GoldAndGearsEntryError::InvalidConundrumRuntime);
    }
    validate_composition(level, track)?;
    let effect = decode_effect(
        level,
        effects
            .into_iter()
            .next()
            .ok_or(GoldAndGearsEntryError::InvalidConundrumRuntime)?,
    )?;
    Ok(RuntimeLevel {
        track,
        level: level.level,
        active: level.active_contributions.clone(),
        rule: level.rule_contribution.clone(),
        contribution: GoldAndGearsConundrumContribution {
            source_level: level.identity.stable_key.clone(),
            scope: effect.0,
            effect: effect.1,
        },
    })
}

fn decode_effect(
    level: &ConundrumLevel,
    raw: RawEffect,
) -> Result<(GoldAndGearsConundrumScope, GoldAndGearsConundrumEffect), GoldAndGearsEntryError> {
    let scope = parse_scope(&raw.scope)?;
    let effect = match raw.operation.as_ref() {
        "ApplyEnemyStatTier" if scope == GoldAndGearsConundrumScope::Battle => {
            require_quality(&raw, "ProjectPolicy")?;
            validate_numeric_binding(
                raw.numeric_binding.as_ref(),
                &["attack_ratio", "max_hp_ratio", "speed_ratio"],
            )?;
            GoldAndGearsConundrumEffect::EnemyStat(enemy_stat_policy(required(
                raw.qualitative_tier.as_deref(),
            )?)?)
        }
        "EnhanceBerserk" if scope == GoldAndGearsConundrumScope::Battle => {
            require_quality(&raw, "ProjectPolicy")?;
            validate_numeric_binding(
                raw.numeric_binding.as_ref(),
                &[
                    "base_trigger_cycle",
                    "enhanced_trigger_cycle",
                    "attack_ratio_per_stack",
                    "speed_ratio_per_stack",
                    "stack_interval",
                    "stack_cap",
                ],
            )?;
            if raw.timing_change.as_deref() != Some("EarlierThanBase")
                || raw.effect_change.as_deref() != Some("EnhancedFromBase")
            {
                return Err(GoldAndGearsEntryError::InvalidConundrumRuntime);
            }
            GoldAndGearsConundrumEffect::EnhancedBerserk
        }
        "EnhanceEliteAndBossToughnessAndBerserkResponse"
            if scope == GoldAndGearsConundrumScope::Battle =>
        {
            require_quality(&raw, "ProjectPolicy")?;
            validate_numeric_binding(
                raw.numeric_binding.as_ref(),
                &["toughness_ratio", "action_advance_ratio"],
            )?;
            if raw.toughness_change.as_deref() != Some("SlightIncrease")
                || raw.berserk_trigger.as_deref() != Some("AfterEachReceivedAttack")
                || raw.berserk_response.as_deref() != Some("AdvanceOwnAction")
            {
                return Err(GoldAndGearsEntryError::InvalidConundrumRuntime);
            }
            GoldAndGearsConundrumEffect::EliteBossResponse(elite_boss_response_policy())
        }
        "AddFormationExtrapolation" if scope == GoldAndGearsConundrumScope::Battle => {
            require_quality(&raw, "ExactStructured")?;
            exact_count(&raw, "third-plane-boss-resonance-extrapolation", "1")?;
            GoldAndGearsConundrumEffect::FormationExtrapolationCount(1)
        }
        "EnableSecondPlaneBossPhaseThreeEnhancement"
            if scope == GoldAndGearsConundrumScope::Battle =>
        {
            require_quality(&raw, "ExactPublicText")?;
            let groups = raw
                .encounter_group_ids
                .ok_or(GoldAndGearsEntryError::InvalidConundrumRuntime)?;
            if raw.target.as_ref() != "second-plane-boss-phase-three"
                || raw.encounter_binding_state.as_deref() != Some("DataReady")
                || groups.len() != 12
                || groups.windows(2).any(|pair| pair[0] >= pair[1])
            {
                return Err(GoldAndGearsEntryError::InvalidConundrumRuntime);
            }
            GoldAndGearsConundrumEffect::SecondPlaneBossPhaseThree(groups)
        }
        "AddBlessingResetCost" if scope == GoldAndGearsConundrumScope::Activity => {
            require_quality(&raw, "ExactStructured")?;
            if raw.target.as_ref() != "blessing-reset.cosmic-fragment-cost"
                || raw.value.as_deref() != Some("20")
                || raw.unit.as_deref() != Some("CosmicFragment")
                || raw.stacking.as_deref() != Some("AdditiveContribution")
            {
                return Err(GoldAndGearsEntryError::InvalidConundrumRuntime);
            }
            GoldAndGearsConundrumEffect::BlessingResetCost(20)
        }
        "ReduceInitialRunResources" if scope == GoldAndGearsConundrumScope::Activity => {
            require_quality(&raw, "ExactStructured")?;
            if raw.target.as_ref() != "run-initial-resources" {
                return Err(GoldAndGearsEntryError::InvalidConundrumRuntime);
            }
            GoldAndGearsConundrumEffect::InitialResources {
                countdown_delta: integer(raw.countdown_delta.as_deref())?,
                dice_reroll_delta: integer(raw.dice_reroll_delta.as_deref())?,
                cosmic_fragment_delta: integer(raw.cosmic_fragment_delta.as_deref())?,
            }
        }
        "GrantNegativeCuriosOnPlaneEntry" if scope == GoldAndGearsConundrumScope::Activity => {
            require_quality(&raw, "ExactStructured")?;
            if raw.target.as_ref() != "party-curio-inventory"
                || raw.timing.as_deref() != Some("EnterEachPlane")
                || raw.pool_binding_state.as_deref() != Some("DataReady")
                || raw.selection_pool_id.as_deref() != Some("gold-gears.curio-pool.negative")
                || raw.unresolved_pool_behavior.as_deref() != Some("FailClosed")
            {
                return Err(GoldAndGearsEntryError::InvalidConundrumRuntime);
            }
            GoldAndGearsConundrumEffect::NegativeCuriosPerPlane(1)
        }
        "ReduceEffectiveBlessingCountPerPath"
            if scope == GoldAndGearsConundrumScope::ActivityAndBattle =>
        {
            require_quality(&raw, "ExactStructured")?;
            if raw.target.as_ref() != "all-path-blessing-counts"
                || raw.value.as_deref() != Some("-1")
                || raw.unit.as_deref() != Some("CountPerPath")
            {
                return Err(GoldAndGearsEntryError::InvalidConundrumRuntime);
            }
            GoldAndGearsConundrumEffect::EffectiveBlessingsPerPath {
                delta: -1,
                minimum: required(raw.minimum_effective_count.as_deref())?
                    .parse()
                    .map_err(|_| GoldAndGearsEntryError::InvalidConundrumRuntime)?,
            }
        }
        _ => return Err(GoldAndGearsEntryError::InvalidConundrumRuntime),
    };
    let _ = level;
    Ok((scope, effect))
}

fn activity_effects(
    contributions: &[GoldAndGearsConundrumContribution],
) -> Result<ConundrumActivityEffects, GoldAndGearsEntryError> {
    let mut result = ConundrumActivityEffects::default();
    for contribution in contributions {
        match contribution.effect {
            GoldAndGearsConundrumEffect::BlessingResetCost(value) => {
                result.blessing_reset_cost = checked_add(result.blessing_reset_cost, value)?;
            }
            GoldAndGearsConundrumEffect::InitialResources {
                countdown_delta,
                dice_reroll_delta,
                cosmic_fragment_delta,
            } => {
                result.initial_countdown_delta =
                    checked_add(result.initial_countdown_delta, countdown_delta)?;
                result.initial_dice_reroll_delta =
                    checked_add(result.initial_dice_reroll_delta, dice_reroll_delta)?;
                result.initial_cosmic_fragment_delta =
                    checked_add(result.initial_cosmic_fragment_delta, cosmic_fragment_delta)?;
            }
            GoldAndGearsConundrumEffect::NegativeCuriosPerPlane(value) => {
                result.negative_curios_per_plane = result
                    .negative_curios_per_plane
                    .checked_add(value)
                    .ok_or(GoldAndGearsEntryError::InvalidConundrumRuntime)?;
            }
            GoldAndGearsConundrumEffect::EffectiveBlessingsPerPath { delta, .. } => {
                result.effective_blessings_per_path_delta =
                    checked_add(result.effective_blessings_per_path_delta, delta)?;
            }
            _ => {}
        }
    }
    Ok(result)
}

fn contribution_digest(
    stats: u8,
    auxiliary: u8,
    contributions: &[GoldAndGearsConundrumContribution],
    berserk: GoldAndGearsBerserkPolicy,
    activity: ConundrumActivityEffects,
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-gears.conundrum-runtime.v1");
    encoder.text(GOLD_AND_GEARS_CONUNDRUM_POLICY_REVISION);
    encoder.text(GOLD_AND_GEARS_CONUNDRUM_POLICY_ACCURACY);
    encoder.text(GOLD_AND_GEARS_CONUNDRUM_POLICY_REPLACEMENT_CONDITION);
    encoder.u8(stats);
    encoder.u8(auxiliary);
    encoder.u8(u8::from(berserk.enhanced()));
    encoder.u8(berserk.trigger_cycle());
    encoder.i64(berserk.attack_ratio_per_stack_scaled());
    encoder.i64(berserk.speed_ratio_per_stack_scaled());
    encoder.u8(berserk.stack_interval_cycles());
    encoder.u8(berserk.stack_cap());
    encoder.i64(activity.blessing_reset_cost);
    encoder.i64(activity.initial_countdown_delta);
    encoder.i64(activity.initial_dice_reroll_delta);
    encoder.i64(activity.initial_cosmic_fragment_delta);
    encoder.u8(activity.negative_curios_per_plane);
    encoder.i64(activity.effective_blessings_per_path_delta);
    for contribution in contributions {
        encoder.text(&contribution.source_level);
        encode_effect(&mut encoder, &contribution.effect);
    }
    encoder.finish()
}

fn encode_effect(encoder: &mut Encoder, effect: &GoldAndGearsConundrumEffect) {
    match effect {
        GoldAndGearsConundrumEffect::EnemyStat(policy) => {
            encoder.u8(0);
            encoder.u8(policy.tier() as u8);
            encoder.i64(policy.attack_ratio_scaled());
            encoder.i64(policy.maximum_hp_ratio_scaled());
            encoder.i64(policy.speed_ratio_scaled());
        }
        GoldAndGearsConundrumEffect::EnhancedBerserk => encoder.u8(1),
        GoldAndGearsConundrumEffect::EliteBossResponse(policy) => {
            encoder.u8(2);
            encoder.i64(policy.toughness_ratio_scaled());
            encoder.i64(policy.action_advance_ratio_scaled());
        }
        GoldAndGearsConundrumEffect::FormationExtrapolationCount(value)
        | GoldAndGearsConundrumEffect::NegativeCuriosPerPlane(value) => {
            encoder.u8(3);
            encoder.u8(*value);
        }
        GoldAndGearsConundrumEffect::SecondPlaneBossPhaseThree(groups) => {
            encoder.u8(4);
            for group in groups {
                encoder.text(group);
            }
        }
        GoldAndGearsConundrumEffect::BlessingResetCost(value) => {
            encoder.u8(5);
            encoder.i64(*value);
        }
        GoldAndGearsConundrumEffect::InitialResources {
            countdown_delta,
            dice_reroll_delta,
            cosmic_fragment_delta,
        } => {
            encoder.u8(6);
            encoder.i64(*countdown_delta);
            encoder.i64(*dice_reroll_delta);
            encoder.i64(*cosmic_fragment_delta);
        }
        GoldAndGearsConundrumEffect::EffectiveBlessingsPerPath { delta, minimum } => {
            encoder.u8(7);
            encoder.i64(*delta);
            encoder.u8(*minimum);
        }
    }
}

fn validate_numeric_binding(
    binding: Option<&NumericBinding>,
    unresolved_fields: &[&str],
) -> Result<(), GoldAndGearsEntryError> {
    let binding = binding.ok_or(GoldAndGearsEntryError::InvalidConundrumRuntime)?;
    if binding.policy_id.as_ref() != SOURCE_POLICY
        || binding.evidence_quality.as_ref() != "ProjectPolicy"
        || binding.resolution_state.as_ref() != "UnresolvedFailClosed"
        || binding.authoritative_behavior.as_ref() != "RejectBattleCompilation"
        || binding
            .unresolved_fields
            .iter()
            .map(Box::as_ref)
            .ne(unresolved_fields.iter().copied())
        || binding.replacement_condition.is_empty()
    {
        return Err(GoldAndGearsEntryError::InvalidConundrumRuntime);
    }
    Ok(())
}

fn validate_composition(
    level: &ConundrumLevel,
    track: Track,
) -> Result<(), GoldAndGearsEntryError> {
    let (expected_source_type, expected_tag, expected_sort) = match (track, level.level) {
        (Track::Stats, 1 | 2 | 4 | 6) => ("AttributeDifficulty", 1_u16, 1_u16),
        (Track::Stats, 3) => ("AttributeDifficulty", 2, 2),
        (Track::Stats, 5) => ("AttributeDifficulty", 3, 3),
        (Track::Auxiliary, value @ 1..=6) => (
            "AdditionalDifficulty",
            u16::from(value) + 3,
            7_u16 - u16::from(value),
        ),
        _ => return Err(GoldAndGearsEntryError::InvalidConundrumRuntime),
    };
    let expected_active = match (track, level.level) {
        (Track::Stats, 1) => vec![stats_rule(1)],
        (Track::Stats, 2) => vec![stats_rule(2)],
        (Track::Stats, 3) => vec![stats_rule(2), stats_rule(3)],
        (Track::Stats, 4) => vec![stats_rule(3), stats_rule(4)],
        (Track::Stats, 5) => vec![stats_rule(3), stats_rule(4), stats_rule(5)],
        (Track::Stats, 6) => vec![stats_rule(3), stats_rule(5), stats_rule(6)],
        (Track::Auxiliary, value @ 1..=6) => (1..=value).map(auxiliary_rule).collect::<Vec<_>>(),
        _ => return Err(GoldAndGearsEntryError::InvalidConundrumRuntime),
    };
    let expected_replaced = match (track, level.level) {
        (Track::Stats, 2) => vec!["gold-gears.conundrum-level.stats.1".to_owned()],
        (Track::Stats, 4) => vec!["gold-gears.conundrum-level.stats.2".to_owned()],
        (Track::Stats, 6) => vec!["gold-gears.conundrum-level.stats.4".to_owned()],
        _ => Vec::new(),
    };
    if level.source_type.as_ref() != expected_source_type
        || level.source_tag != expected_tag
        || level.source_sort != expected_sort
        || level
            .active_contributions
            .iter()
            .map(Box::as_ref)
            .ne(expected_active.iter().map(String::as_str))
        || level
            .replaces_levels
            .iter()
            .map(Box::as_ref)
            .ne(expected_replaced.iter().map(String::as_str))
    {
        return Err(GoldAndGearsEntryError::InvalidConundrumRuntime);
    }
    Ok(())
}

fn stats_rule(level: u8) -> String {
    format!("gold-gears.rule.conundrum.stats.{level}")
}

fn auxiliary_rule(level: u8) -> String {
    format!("gold-gears.rule.conundrum.auxiliary.{level}")
}

fn parse_scope(value: &str) -> Result<GoldAndGearsConundrumScope, GoldAndGearsEntryError> {
    match value {
        "Activity" => Ok(GoldAndGearsConundrumScope::Activity),
        "Battle" => Ok(GoldAndGearsConundrumScope::Battle),
        "ActivityAndBattle" => Ok(GoldAndGearsConundrumScope::ActivityAndBattle),
        _ => Err(GoldAndGearsEntryError::InvalidConundrumRuntime),
    }
}

fn exact_count(raw: &RawEffect, target: &str, value: &str) -> Result<(), GoldAndGearsEntryError> {
    if raw.target.as_ref() != target
        || raw.value.as_deref() != Some(value)
        || raw.unit.as_deref() != Some("Count")
    {
        return Err(GoldAndGearsEntryError::InvalidConundrumRuntime);
    }
    Ok(())
}

fn require_quality(raw: &RawEffect, expected: &str) -> Result<(), GoldAndGearsEntryError> {
    if raw.mechanism_quality.as_ref() != expected {
        return Err(GoldAndGearsEntryError::InvalidConundrumRuntime);
    }
    Ok(())
}

fn required(value: Option<&str>) -> Result<&str, GoldAndGearsEntryError> {
    value.ok_or(GoldAndGearsEntryError::InvalidConundrumRuntime)
}

fn integer(value: Option<&str>) -> Result<i64, GoldAndGearsEntryError> {
    required(value)?
        .parse()
        .map_err(|_| GoldAndGearsEntryError::InvalidConundrumRuntime)
}

fn checked_add(left: i64, right: i64) -> Result<i64, GoldAndGearsEntryError> {
    left.checked_add(right)
        .ok_or(GoldAndGearsEntryError::InvalidConundrumRuntime)
}
