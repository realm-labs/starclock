//! Trailblaze Bonus, Path boost, Resonance and Extrapolation runtime.

use crate::gold_gears_unique::PathDefinition;
use std::collections::BTreeMap;

use serde::Deserialize;
use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityRngLabel, ActivityRngStreams, ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use crate::{
    digest::Encoder,
    gold_gears_unique::{
        Extrapolation, GoldAndGearsUniqueCatalog, Interplay, PathBoost, Resonance, TrailblazeBonus,
    },
};

use super::{
    GoldAndGearsEntryError,
    api::{GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance},
    dice_passive::path_boost_stacks,
    state_layout::{RESOURCE_COSMIC_FRAGMENTS_KEY, RESOURCE_DICE_CHEATS_KEY, RUN_RESOURCES_SLOT},
};

pub const GOLD_AND_GEARS_PROGRESSION_RUNTIME_REVISION: &str =
    "gold-and-gears-progression-runtime-v1";
pub const GOLD_AND_GEARS_EXTRAPOLATION_POLICY_REVISION: &str =
    "gold-and-gears-resonance-extrapolation-policy-v1";
pub const GOLD_AND_GEARS_EXTRAPOLATION_POLICY_ACCURACY: &str =
    "DeterministicProjectPolicyNotObservedParity";

const EXTRAPOLATION_PURPOSE: u16 = 0x4760;
const FORMATION_THRESHOLDS: [u8; 3] = [6, 10, 14];

/// Deferred content selection emitted by a Trailblaze Bonus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoldAndGearsTrailblazeOffer {
    Blessing {
        choice_count: u8,
        minimum_rarity: u8,
        maximum_rarity: u8,
    },
    Curio {
        choice_count: u8,
    },
    CurioCategory {
        category: Box<str>,
        count: u8,
    },
}

/// Entry boundary for one caller-selected Trailblaze Bonus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsTrailblazeBonusPlan {
    source_bonus: Box<str>,
    source_rule: Box<str>,
    event_id: u32,
    immediate: Option<ActivityProgramDefinition>,
    offers: Box<[GoldAndGearsTrailblazeOffer]>,
}

impl GoldAndGearsTrailblazeBonusPlan {
    #[must_use]
    pub fn source_bonus(&self) -> &str {
        &self.source_bonus
    }

    #[must_use]
    pub fn source_rule(&self) -> &str {
        &self.source_rule
    }

    #[must_use]
    pub const fn event_id(&self) -> u32 {
        self.event_id
    }

    #[must_use]
    pub const fn immediate_program(&self) -> Option<&ActivityProgramDefinition> {
        self.immediate.as_ref()
    }

    #[must_use]
    pub fn offers(&self) -> &[GoldAndGearsTrailblazeOffer] {
        &self.offers
    }
}

/// Closed Path-stat target set used by Custom Dice boosts.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GoldAndGearsPathBoostStat {
    ShieldGain,
    EffectHitRate,
    DamageOverTime,
    OutgoingHealing,
    CriticalDamage,
    DamageDealt,
    FollowUpAttackDamage,
    BasicAttackDamage,
    UltimateDamage,
}

/// Immutable selected-Path boost projected from current Activity stacks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsPathBoostContribution {
    source_boost: Box<str>,
    path: Box<str>,
    stat: GoldAndGearsPathBoostStat,
    ratio_scaled: i64,
    stacks: u32,
}

impl GoldAndGearsPathBoostContribution {
    #[must_use]
    pub fn source_boost(&self) -> &str {
        &self.source_boost
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub const fn stat(&self) -> GoldAndGearsPathBoostStat {
        self.stat
    }

    #[must_use]
    pub const fn ratio_scaled(&self) -> i64 {
        self.ratio_scaled
    }

    #[must_use]
    pub const fn stacks(&self) -> u32 {
        self.stacks
    }
}

/// Immutable shared Resonance/Formation or Gold Interplay binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsResonanceContribution {
    source: Box<str>,
    binding_key: Box<str>,
    parameters_scaled: Box<[i64]>,
    kind: GoldAndGearsResonanceKind,
}

impl GoldAndGearsResonanceContribution {
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    #[must_use]
    pub fn binding_key(&self) -> &str {
        &self.binding_key
    }

    pub fn parameters_scaled(&self) -> impl ExactSizeIterator<Item = i64> + '_ {
        self.parameters_scaled.iter().copied()
    }

    #[must_use]
    pub const fn kind(&self) -> GoldAndGearsResonanceKind {
        self.kind
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsResonanceKind {
    Resonance,
    Formation,
    Interplay,
}

/// Current selected-Path additions after blessing thresholds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsResonanceSet {
    resonance: Option<GoldAndGearsResonanceContribution>,
    formations: Box<[GoldAndGearsResonanceContribution]>,
    interplays: Box<[GoldAndGearsResonanceContribution]>,
}

impl GoldAndGearsResonanceSet {
    #[must_use]
    pub const fn resonance(&self) -> Option<&GoldAndGearsResonanceContribution> {
        self.resonance.as_ref()
    }

    #[must_use]
    pub fn formations(&self) -> &[GoldAndGearsResonanceContribution] {
        &self.formations
    }

    #[must_use]
    pub fn interplays(&self) -> &[GoldAndGearsResonanceContribution] {
        &self.interplays
    }
}

/// Caller-owned facts for the only Extrapolation boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldAndGearsExtrapolationContext<'a> {
    plane_layer: u8,
    boss: bool,
    offered_path: &'a str,
}

impl<'a> GoldAndGearsExtrapolationContext<'a> {
    #[must_use]
    pub const fn new(plane_layer: u8, boss: bool, offered_path: &'a str) -> Self {
        Self {
            plane_layer,
            boss,
            offered_path,
        }
    }
}

/// Relative-target policy retained for enemy-owned shared bindings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoldAndGearsExtrapolationPolarity {
    RelativeToEnemyOwner,
}

/// Selected Third Plane boss contribution set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsExtrapolationSelection {
    offered_path: Box<str>,
    contributions: Box<[GoldAndGearsResonanceContribution]>,
    polarity: GoldAndGearsExtrapolationPolarity,
    digest: [u8; 32],
}

impl GoldAndGearsExtrapolationSelection {
    #[must_use]
    pub fn offered_path(&self) -> &str {
        &self.offered_path
    }

    #[must_use]
    pub fn contributions(&self) -> &[GoldAndGearsResonanceContribution] {
        &self.contributions
    }

    #[must_use]
    pub const fn polarity(&self) -> GoldAndGearsExtrapolationPolarity {
        self.polarity
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone, Debug)]
pub(super) struct ProgressionRuntimeCatalog {
    bonuses: Box<[RuntimeBonus]>,
    paths: Box<[RuntimePath]>,
}

#[derive(Clone, Debug)]
struct RuntimeBonus {
    key: Box<str>,
    rule: Box<str>,
    event_id: u32,
    effect: RuntimeBonusEffect,
}

#[derive(Clone, Debug)]
enum RuntimeBonusEffect {
    CosmicFragments(i64),
    DiceCheats(i64),
    Offers(Box<[GoldAndGearsTrailblazeOffer]>),
}

#[derive(Clone, Debug)]
struct RuntimePath {
    key: Box<str>,
    sort: u16,
    boost: RuntimeBoost,
    resonance: GoldAndGearsResonanceContribution,
    formations: Box<[GoldAndGearsResonanceContribution]>,
    extrapolations: Box<[GoldAndGearsResonanceContribution]>,
    interplays: Box<[RuntimeInterplay]>,
}

#[derive(Clone, Debug)]
struct RuntimeBoost {
    key: Box<str>,
    stat: GoldAndGearsPathBoostStat,
    allowed_increments: Box<[i64]>,
    dice_path_value_ids: Box<[u32]>,
}

#[derive(Clone, Debug)]
struct RuntimeInterplay {
    sub_path: Box<str>,
    main_threshold: u8,
    sub_threshold: u8,
    contribution: GoldAndGearsResonanceContribution,
}

#[derive(Clone, Debug)]
pub(super) struct CompiledProgressionRuntime {
    bonus: Option<GoldAndGearsTrailblazeBonusPlan>,
    path: RuntimePath,
    path_boost_value_scaled: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BonusEffect {
    operation: Box<str>,
    scope: Box<str>,
    #[serde(default)]
    value: Option<Box<str>>,
    #[serde(default)]
    unit: Option<Box<str>>,
    #[serde(default)]
    choice_count: Option<Box<str>>,
    #[serde(default)]
    minimum_rarity: Option<Box<str>>,
    #[serde(default)]
    maximum_rarity: Option<Box<str>>,
    #[serde(default)]
    pool_binding_state: Option<Box<str>>,
    #[serde(default)]
    grants: Option<Box<[CategoryGrant]>>,
    mechanism_quality: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CategoryGrant {
    category: Box<str>,
    count: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IndexedParameter {
    index: u16,
    value: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ControllerPolicy {
    policy_id: Box<str>,
    evidence_quality: Box<str>,
    candidate_order: Box<str>,
    formation_selection: Box<str>,
    base_formation_count: Box<str>,
    auxiliary_conundrum_bonus_count: Box<str>,
    action_and_polarity_lowering: Box<str>,
}

impl ProgressionRuntimeCatalog {
    pub(super) fn compile(
        catalog: &GoldAndGearsUniqueCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let mut bonuses = catalog
            .trailblaze_bonuses
            .iter()
            .map(decode_bonus)
            .collect::<Result<Vec<_>, _>>()?;
        bonuses.sort_by(|left, right| left.key.cmp(&right.key));
        let mut paths = catalog
            .paths
            .iter()
            .map(|path| decode_path(catalog, path))
            .collect::<Result<Vec<_>, _>>()?;
        paths.sort_by_key(|path| path.sort);
        if bonuses.len() != 5
            || paths.len() != 9
            || paths
                .iter()
                .enumerate()
                .any(|(index, path)| usize::from(path.sort) != index + 1)
        {
            return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
        }
        Ok(Self {
            bonuses: bonuses.into_boxed_slice(),
            paths: paths.into_boxed_slice(),
        })
    }

    pub(super) fn resonance_contribution(
        &self,
        source: &str,
    ) -> Option<&GoldAndGearsResonanceContribution> {
        self.paths
            .iter()
            .flat_map(|path| {
                std::iter::once(&path.resonance)
                    .chain(path.formations.iter())
                    .chain(path.extrapolations.iter())
                    .chain(
                        path.interplays
                            .iter()
                            .map(|interplay| &interplay.contribution),
                    )
            })
            .find(|contribution| contribution.source() == source)
    }

    pub(super) fn select(
        &self,
        path: &str,
        bonus: Option<&str>,
        path_value_id: u32,
        path_boost_value_scaled: i64,
    ) -> Result<CompiledProgressionRuntime, GoldAndGearsEntryError> {
        let path = self
            .paths
            .iter()
            .find(|candidate| candidate.key.as_ref() == path)
            .cloned()
            .ok_or(GoldAndGearsEntryError::InvalidProgressionRuntime)?;
        if path
            .boost
            .dice_path_value_ids
            .binary_search(&path_value_id)
            .is_err()
            || path
                .boost
                .allowed_increments
                .binary_search(&path_boost_value_scaled)
                .is_err()
        {
            return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
        }
        let bonus = bonus
            .map(|key| {
                self.bonuses
                    .iter()
                    .find(|candidate| candidate.key.as_ref() == key)
                    .ok_or(GoldAndGearsEntryError::InvalidProgressionRuntime)
                    .and_then(compile_bonus_plan)
            })
            .transpose()?;
        Ok(CompiledProgressionRuntime {
            bonus,
            path,
            path_boost_value_scaled,
        })
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.bonuses.len(),
            self.paths.len(),
            self.paths
                .iter()
                .map(|path| path.formations.len() + 1)
                .sum(),
            self.paths
                .iter()
                .map(|path| path.extrapolations.len())
                .sum(),
            self.paths.iter().map(|path| path.interplays.len()).sum(),
        )
    }
}

impl GoldAndGearsRuntimeInstance {
    #[must_use]
    pub const fn trailblaze_bonus_plan(&self) -> Option<&GoldAndGearsTrailblazeBonusPlan> {
        self.progression_runtime.bonus.as_ref()
    }

    pub fn path_boost_contribution(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<GoldAndGearsPathBoostContribution, GoldAndGearsEntryError> {
        let stacks =
            path_boost_stacks(state).ok_or(GoldAndGearsEntryError::InvalidProgressionState)?;
        let ratio_scaled = self
            .progression_runtime
            .path_boost_value_scaled
            .checked_mul(i64::from(stacks))
            .ok_or(GoldAndGearsEntryError::InvalidProgressionState)?;
        Ok(GoldAndGearsPathBoostContribution {
            source_boost: self.progression_runtime.path.boost.key.clone(),
            path: self.progression_runtime.path.key.clone(),
            stat: self.progression_runtime.path.boost.stat,
            ratio_scaled,
            stacks,
        })
    }

    pub fn resonance_additions(
        &self,
        blessing_counts: &[(String, u8)],
        selected_formations: &[String],
    ) -> Result<GoldAndGearsResonanceSet, GoldAndGearsEntryError> {
        let counts = canonical_counts(blessing_counts)?;
        if counts.keys().any(|key| {
            !self
                .progression_catalog
                .paths
                .iter()
                .any(|path| path.key.as_ref() == *key)
        }) {
            return Err(GoldAndGearsEntryError::InvalidBlessingCounts);
        }
        let main_count = counts
            .get(self.progression_runtime.path.key.as_ref())
            .copied()
            .unwrap_or(0);
        let resonance = (main_count >= 3).then(|| self.progression_runtime.path.resonance.clone());
        let available_slots = FORMATION_THRESHOLDS
            .iter()
            .filter(|threshold| main_count >= **threshold)
            .count();
        let mut formation_keys = selected_formations
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        formation_keys.sort_unstable();
        if formation_keys.windows(2).any(|pair| pair[0] == pair[1])
            || formation_keys.len() > available_slots
        {
            return Err(GoldAndGearsEntryError::InvalidResonanceSelection);
        }
        let formations = formation_keys
            .iter()
            .map(|key| {
                self.progression_runtime
                    .path
                    .formations
                    .iter()
                    .find(|formation| formation.source.as_ref() == *key)
                    .cloned()
                    .ok_or(GoldAndGearsEntryError::InvalidResonanceSelection)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let interplays = self
            .progression_runtime
            .path
            .interplays
            .iter()
            .filter(|interplay| {
                main_count >= interplay.main_threshold
                    && counts
                        .get(interplay.sub_path.as_ref())
                        .copied()
                        .unwrap_or(0)
                        >= interplay.sub_threshold
            })
            .map(|interplay| interplay.contribution.clone())
            .collect::<Vec<_>>();
        Ok(GoldAndGearsResonanceSet {
            resonance,
            formations: formations.into_boxed_slice(),
            interplays: interplays.into_boxed_slice(),
        })
    }

    pub fn compile_resonance_extrapolation(
        &self,
        context: GoldAndGearsExtrapolationContext<'_>,
        rng: &mut ActivityRngStreams,
    ) -> Result<GoldAndGearsExtrapolationSelection, GoldAndGearsEntryError> {
        if context.plane_layer != 3 || !context.boss {
            return Err(GoldAndGearsEntryError::InvalidExtrapolationBoundary);
        }
        let path = self
            .progression_catalog
            .paths
            .iter()
            .find(|path| path.key.as_ref() == context.offered_path)
            .ok_or_else(|| {
                GoldAndGearsEntryError::UnknownExtrapolationPath(context.offered_path.into())
            })?;
        let [base, formations @ ..] = path.extrapolations.as_ref() else {
            return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
        };
        if formations.len() != 3 {
            return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
        }
        let maximum = 1 + usize::from(self.auxiliary_conundrum() >= 1);
        let selected = rng
            .choose_weighted_without_replacement(
                ActivityRngLabel::Encounter,
                EXTRAPOLATION_PURPOSE,
                &[1; 3],
                u16::try_from(maximum).expect("at most two formations"),
            )
            .map_err(|_| GoldAndGearsEntryError::InvalidProgressionRuntime)?;
        let mut selected_formations = selected
            .iter()
            .map(|index| {
                formations
                    .get(*index as usize)
                    .cloned()
                    .ok_or(GoldAndGearsEntryError::InvalidProgressionRuntime)
            })
            .collect::<Result<Vec<_>, _>>()?;
        selected_formations.sort_by(|left, right| left.binding_key.cmp(&right.binding_key));
        let mut contributions = vec![base.clone()];
        contributions.extend(selected_formations);
        let digest = extrapolation_digest(context.offered_path, &contributions);
        Ok(GoldAndGearsExtrapolationSelection {
            offered_path: context.offered_path.into(),
            contributions: contributions.into_boxed_slice(),
            polarity: GoldAndGearsExtrapolationPolarity::RelativeToEnemyOwner,
            digest,
        })
    }
}

impl GoldAndGearsRuntimeFactory {
    #[must_use]
    pub fn extrapolation_paths(&self) -> impl ExactSizeIterator<Item = &str> {
        self.progression.paths.iter().map(|path| path.key.as_ref())
    }
}

fn decode_bonus(bonus: &TrailblazeBonus) -> Result<RuntimeBonus, GoldAndGearsEntryError> {
    let effects: Vec<BonusEffect> = serde_json::from_str(&bonus.effect_contributions_json)
        .map_err(|_| GoldAndGearsEntryError::InvalidProgressionRuntime)?;
    let [effect] = effects.as_slice() else {
        return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
    };
    if effect.scope.as_ref() != "Activity" || effect.mechanism_quality.as_ref() != "ExactStructured"
    {
        return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
    }
    let runtime = match effect.operation.as_ref() {
        "AddCosmicFragments"
            if effect.value.as_deref() == Some("150")
                && effect.unit.as_deref() == Some("CosmicFragment") =>
        {
            RuntimeBonusEffect::CosmicFragments(150)
        }
        "AddDiceCheatAttempts"
            if effect.value.as_deref() == Some("1") && effect.unit.as_deref() == Some("Count") =>
        {
            RuntimeBonusEffect::DiceCheats(1)
        }
        "OfferRandomBlessing"
            if effect.choice_count.as_deref() == Some("1")
                && effect.minimum_rarity.as_deref() == Some("1")
                && effect.maximum_rarity.as_deref() == Some("2")
                && effect.pool_binding_state.as_deref() == Some("DeferredToG08P2B1") =>
        {
            RuntimeBonusEffect::Offers(
                vec![GoldAndGearsTrailblazeOffer::Blessing {
                    choice_count: 1,
                    minimum_rarity: 1,
                    maximum_rarity: 2,
                }]
                .into_boxed_slice(),
            )
        }
        "OfferRandomCurio"
            if effect.choice_count.as_deref() == Some("1")
                && effect.pool_binding_state.as_deref() == Some("DeferredToG08P2B2") =>
        {
            RuntimeBonusEffect::Offers(
                vec![GoldAndGearsTrailblazeOffer::Curio { choice_count: 1 }].into_boxed_slice(),
            )
        }
        "GrantCuriosByCategory"
            if effect.pool_binding_state.as_deref() == Some("DeferredToG08P2B2") =>
        {
            let offers = effect
                .grants
                .as_deref()
                .ok_or(GoldAndGearsEntryError::InvalidProgressionRuntime)?
                .iter()
                .map(|grant| {
                    Ok(GoldAndGearsTrailblazeOffer::CurioCategory {
                        category: grant.category.clone(),
                        count: parse_u8(&grant.count)?,
                    })
                })
                .collect::<Result<Vec<_>, GoldAndGearsEntryError>>()?;
            if offers
                != [
                    GoldAndGearsTrailblazeOffer::CurioCategory {
                        category: "Negative".into(),
                        count: 1,
                    },
                    GoldAndGearsTrailblazeOffer::CurioCategory {
                        category: "ErrorCode".into(),
                        count: 1,
                    },
                ]
            {
                return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
            }
            RuntimeBonusEffect::Offers(offers.into_boxed_slice())
        }
        _ => return Err(GoldAndGearsEntryError::InvalidProgressionRuntime),
    };
    Ok(RuntimeBonus {
        key: bonus.identity.stable_key.clone(),
        rule: bonus.rule_contribution.clone(),
        event_id: bonus
            .bonus_event
            .parse()
            .map_err(|_| GoldAndGearsEntryError::InvalidProgressionRuntime)?,
        effect: runtime,
    })
}

fn compile_bonus_plan(
    bonus: &RuntimeBonus,
) -> Result<GoldAndGearsTrailblazeBonusPlan, GoldAndGearsEntryError> {
    let (immediate, offers): (
        Option<ActivityProgramDefinition>,
        Box<[GoldAndGearsTrailblazeOffer]>,
    ) = match &bonus.effect {
        RuntimeBonusEffect::CosmicFragments(value) => (
            Some(bonus_program(
                bonus.event_id,
                RESOURCE_COSMIC_FRAGMENTS_KEY,
                *value,
            )?),
            Box::new([]),
        ),
        RuntimeBonusEffect::DiceCheats(value) => (
            Some(bonus_program(
                bonus.event_id,
                RESOURCE_DICE_CHEATS_KEY,
                *value,
            )?),
            Box::new([]),
        ),
        RuntimeBonusEffect::Offers(offers) => (None, offers.clone()),
    };
    Ok(GoldAndGearsTrailblazeBonusPlan {
        source_bonus: bonus.key.clone(),
        source_rule: bonus.rule.clone(),
        event_id: bonus.event_id,
        immediate,
        offers,
    })
}

fn decode_path(
    catalog: &GoldAndGearsUniqueCatalog,
    path: &PathDefinition,
) -> Result<RuntimePath, GoldAndGearsEntryError> {
    let boost = catalog
        .path_boosts
        .iter()
        .find(|boost| boost.identity.id == path.path_boost)
        .ok_or(GoldAndGearsEntryError::InvalidProgressionRuntime)
        .and_then(|boost| decode_boost(catalog, boost, path.identity.id.0))?;
    let mut resonances = catalog
        .resonances
        .iter()
        .filter(|resonance| resonance.path.0 == path.identity.id.0)
        .collect::<Vec<_>>();
    resonances.sort_by(|left, right| left.identity.stable_key.cmp(&right.identity.stable_key));
    let base = resonances
        .iter()
        .find(|resonance| resonance.resonance_kind.as_ref() == "Resonance")
        .ok_or(GoldAndGearsEntryError::InvalidProgressionRuntime)?;
    let formations = resonances
        .iter()
        .filter(|resonance| resonance.resonance_kind.as_ref() == "Formation")
        .map(|resonance| resonance_contribution(resonance))
        .collect::<Result<Vec<_>, _>>()?;
    if resonances.len() != 4
        || formations.len() != 3
        || base.threshold != 3
        || base.energy_max.0.as_ref() != "100"
        || base.initial_energy.0.as_ref() != "0"
        || resonances.iter().any(|resonance| {
            resonance.resonance_kind.as_ref() == "Formation"
                && (resonance.threshold != 0
                    || resonance.energy_max.0.as_ref() != "0"
                    || resonance.initial_energy.0.as_ref() != "0")
        })
    {
        return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
    }
    let mut extrapolations = catalog
        .extrapolations
        .iter()
        .filter(|entry| entry.path.0 == path.identity.id.0)
        .map(extrapolation_contribution)
        .collect::<Result<Vec<_>, _>>()?;
    extrapolations.sort_by(|left, right| left.source.cmp(&right.source));
    if extrapolations.len() != 4
        || extrapolations[0].kind != GoldAndGearsResonanceKind::Resonance
        || extrapolations[1..]
            .iter()
            .any(|entry| entry.kind != GoldAndGearsResonanceKind::Formation)
    {
        return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
    }
    let mut interplays = catalog
        .interplays
        .iter()
        .filter(|entry| entry.main_path.0 == path.identity.id.0)
        .map(|entry| interplay_contribution(catalog, entry))
        .collect::<Result<Vec<_>, _>>()?;
    interplays.sort_by(|left, right| left.contribution.source.cmp(&right.contribution.source));
    if interplays.len() != 2 {
        return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
    }
    Ok(RuntimePath {
        key: path.identity.stable_key.clone(),
        sort: path.sort,
        boost,
        resonance: resonance_contribution(base)?,
        formations: formations.into_boxed_slice(),
        extrapolations: extrapolations.into_boxed_slice(),
        interplays: interplays.into_boxed_slice(),
    })
}

fn decode_boost(
    catalog: &GoldAndGearsUniqueCatalog,
    boost: &PathBoost,
    path_id: u32,
) -> Result<RuntimeBoost, GoldAndGearsEntryError> {
    if boost.path.0 != path_id
        || boost.effect_type.as_ref() != "AddMazeBuff"
        || boost.target_team.as_ref() != "TeamLight"
        || boost.stacking.as_ref() != "AdditiveContribution"
        || boost.value_conversion.as_ref() != "PercentInputDividedBy100ByStageAbility"
        || boost.dice_path_value_keys.len() != 12
        || boost.allowed_increments.len() != 6
    {
        return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
    }
    let mut allowed_increments = boost
        .allowed_increments
        .iter()
        .map(|value| scalar(&value.0))
        .collect::<Result<Vec<_>, _>>()?;
    allowed_increments.sort_unstable();
    let mut dice_path_value_ids = boost
        .dice_path_value_keys
        .iter()
        .map(|key| {
            catalog
                .dice_path_values
                .iter()
                .find(|value| value.identity.stable_key == *key)
                .map(|value| value.identity.id.0)
                .ok_or(GoldAndGearsEntryError::InvalidProgressionRuntime)
        })
        .collect::<Result<Vec<_>, _>>()?;
    dice_path_value_ids.sort_unstable();
    Ok(RuntimeBoost {
        key: boost.identity.stable_key.clone(),
        stat: path_boost_stat(&boost.boost_stat)?,
        allowed_increments: allowed_increments.into_boxed_slice(),
        dice_path_value_ids: dice_path_value_ids.into_boxed_slice(),
    })
}

fn resonance_contribution(
    resonance: &Resonance,
) -> Result<GoldAndGearsResonanceContribution, GoldAndGearsEntryError> {
    if resonance.source_binding_type.as_ref() != "StageAbilityBeforeCharacterBorn" {
        return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
    }
    Ok(GoldAndGearsResonanceContribution {
        source: resonance.identity.stable_key.clone(),
        binding_key: resonance.source_binding_key.clone(),
        parameters_scaled: parameters(&resonance.parameter_values_json)?,
        kind: match resonance.resonance_kind.as_ref() {
            "Resonance" => GoldAndGearsResonanceKind::Resonance,
            "Formation" => GoldAndGearsResonanceKind::Formation,
            _ => return Err(GoldAndGearsEntryError::InvalidProgressionRuntime),
        },
    })
}

fn extrapolation_contribution(
    extrapolation: &Extrapolation,
) -> Result<GoldAndGearsResonanceContribution, GoldAndGearsEntryError> {
    let policy: ControllerPolicy = serde_json::from_str(&extrapolation.controller_policy_json)
        .map_err(|_| GoldAndGearsEntryError::InvalidProgressionRuntime)?;
    if extrapolation.battle_scope.as_ref() != "ThirdPlaneBossBattle"
        || extrapolation.source_binding_type.as_ref() != "StageAbilityBeforeCharacterBorn"
        || policy.policy_id.as_ref() != "resonance-extrapolation-controller-v1"
        || policy.evidence_quality.as_ref() != "ProjectPolicy"
        || policy.candidate_order.as_ref() != "stable-source-tag-ascending"
        || policy.formation_selection.as_ref() != "seeded-activity-stream-without-replacement"
        || policy.base_formation_count.as_ref() != "1"
        || policy.auxiliary_conundrum_bonus_count.as_ref() != "1"
        || policy.action_and_polarity_lowering.as_ref() != "UnresolvedFailClosed"
    {
        return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
    }
    Ok(GoldAndGearsResonanceContribution {
        source: extrapolation.identity.stable_key.clone(),
        binding_key: extrapolation.source_binding_key.clone(),
        parameters_scaled: parameters(&extrapolation.source_parameters_json)?,
        kind: match (
            extrapolation.enhanced,
            extrapolation.shared_resonance_kind.as_ref(),
        ) {
            (false, "Resonance") => GoldAndGearsResonanceKind::Resonance,
            (true, "Formation") => GoldAndGearsResonanceKind::Formation,
            _ => return Err(GoldAndGearsEntryError::InvalidProgressionRuntime),
        },
    })
}

fn interplay_contribution(
    catalog: &GoldAndGearsUniqueCatalog,
    interplay: &Interplay,
) -> Result<RuntimeInterplay, GoldAndGearsEntryError> {
    let sub_path = catalog
        .paths
        .iter()
        .find(|path| path.identity.id == interplay.sub_path)
        .ok_or(GoldAndGearsEntryError::InvalidProgressionRuntime)?;
    if interplay.main_threshold != 3
        || interplay.sub_threshold != 3
        || interplay.source_binding_type.as_ref() != "StageAbilityBeforeCharacterBorn"
    {
        return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
    }
    Ok(RuntimeInterplay {
        sub_path: sub_path.identity.stable_key.clone(),
        main_threshold: 3,
        sub_threshold: 3,
        contribution: GoldAndGearsResonanceContribution {
            source: interplay.identity.stable_key.clone(),
            binding_key: interplay.source_binding_key.clone(),
            parameters_scaled: parameters(&interplay.source_parameters_json)?,
            kind: GoldAndGearsResonanceKind::Interplay,
        },
    })
}

fn path_boost_stat(value: &str) -> Result<GoldAndGearsPathBoostStat, GoldAndGearsEntryError> {
    Ok(match value {
        "ShieldGain" => GoldAndGearsPathBoostStat::ShieldGain,
        "EffectHitRate" => GoldAndGearsPathBoostStat::EffectHitRate,
        "DamageOverTime" => GoldAndGearsPathBoostStat::DamageOverTime,
        "OutgoingHealing" => GoldAndGearsPathBoostStat::OutgoingHealing,
        "CriticalDamage" => GoldAndGearsPathBoostStat::CriticalDamage,
        "DamageDealt" => GoldAndGearsPathBoostStat::DamageDealt,
        "FollowUpAttackDamage" => GoldAndGearsPathBoostStat::FollowUpAttackDamage,
        "BasicAttackDamage" => GoldAndGearsPathBoostStat::BasicAttackDamage,
        "UltimateDamage" => GoldAndGearsPathBoostStat::UltimateDamage,
        _ => return Err(GoldAndGearsEntryError::InvalidProgressionRuntime),
    })
}

fn parameters(json: &str) -> Result<Box<[i64]>, GoldAndGearsEntryError> {
    let values: Vec<IndexedParameter> = serde_json::from_str(json)
        .map_err(|_| GoldAndGearsEntryError::InvalidProgressionRuntime)?;
    if values
        .iter()
        .enumerate()
        .any(|(index, value)| usize::from(value.index) != index + 1)
    {
        return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
    }
    values
        .iter()
        .map(|value| scalar(&value.value))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn scalar(value: &str) -> Result<i64, GoldAndGearsEntryError> {
    let (whole, fraction) = value.split_once('.').map_or((value, ""), |parts| parts);
    if fraction.len() > 6 {
        return Err(GoldAndGearsEntryError::InvalidProgressionRuntime);
    }
    let negative = whole.starts_with('-');
    let whole = whole
        .parse::<i64>()
        .map_err(|_| GoldAndGearsEntryError::InvalidProgressionRuntime)?;
    let fraction = if fraction.is_empty() {
        0
    } else {
        fraction
            .parse::<i64>()
            .map_err(|_| GoldAndGearsEntryError::InvalidProgressionRuntime)?
            .checked_mul(10_i64.pow(u32::try_from(6 - fraction.len()).unwrap_or(0)))
            .ok_or(GoldAndGearsEntryError::InvalidProgressionRuntime)?
    };
    whole
        .checked_mul(1_000_000)
        .and_then(|scaled| {
            if negative {
                scaled.checked_sub(fraction)
            } else {
                scaled.checked_add(fraction)
            }
        })
        .ok_or(GoldAndGearsEntryError::InvalidProgressionRuntime)
}

fn canonical_counts(counts: &[(String, u8)]) -> Result<BTreeMap<&str, u8>, GoldAndGearsEntryError> {
    let mut canonical = BTreeMap::new();
    for (path, count) in counts {
        if canonical.insert(path.as_str(), *count).is_some() {
            return Err(GoldAndGearsEntryError::InvalidBlessingCounts);
        }
    }
    Ok(canonical)
}

fn bonus_program(
    event_id: u32,
    key: u64,
    delta: i64,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(0x4900_0000 + event_id)
            .expect("Trailblaze Bonus program ID is non-zero"),
        vec![ActivityOperation::AddCounter {
            slot: ActivitySlotId::new(RUN_RESOURCES_SLOT).expect("run resources slot is non-zero"),
            key,
            delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(delta)),
        }],
    )
    .map_err(|_| GoldAndGearsEntryError::InvalidProgressionRuntime)
}

fn extrapolation_digest(
    offered_path: &str,
    contributions: &[GoldAndGearsResonanceContribution],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-gears.resonance-extrapolation.v1");
    encoder.text(GOLD_AND_GEARS_EXTRAPOLATION_POLICY_REVISION);
    encoder.text(GOLD_AND_GEARS_EXTRAPOLATION_POLICY_ACCURACY);
    encoder.text(offered_path);
    for contribution in contributions {
        encoder.text(&contribution.source);
        encoder.text(&contribution.binding_key);
        encoder.u8(contribution.kind as u8);
        for value in &contribution.parameters_scaled {
            encoder.i64(*value);
        }
    }
    encoder.finish()
}

fn parse_u8(value: &str) -> Result<u8, GoldAndGearsEntryError> {
    value
        .parse()
        .map_err(|_| GoldAndGearsEntryError::InvalidProgressionRuntime)
}
