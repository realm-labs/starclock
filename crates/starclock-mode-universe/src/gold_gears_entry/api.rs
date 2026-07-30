//! Public Gold and Gears entry and entry-compiled instance types.

use std::sync::Arc;

use starclock_activity::{ActivityStateDefinition, ParticipantLock};

use super::{
    EXPECTED_PROFILE_KEY, GoldAndGearsEntryError,
    state::compile_state,
    validate::{
        canonical_completed_areas, canonical_neural_network, canonical_unlocked_dice,
        parse_integer, validate_conundrum, validate_loadout, validate_participants,
    },
};
use crate::{
    gold_gears_structural::{AreaDefinition, AreaGroup, GoldAndGearsStructuralCatalog},
    gold_gears_unique::GoldAndGearsUniqueCatalog,
};

/// Entry-policy revision that resolves `G14-R01`.
pub const GOLD_AND_GEARS_ENTRY_REVISION: &str = "gold-and-gears-entry-policy-v1";

/// Caller-owned selections for one Gold and Gears run.
///
/// The six dice faces retain slot order. Neural nodes, completed formal areas,
/// and unlocked dice are sets: compilation canonicalizes them and rejects
/// duplicates. No omitted selection is filled with a random or hidden default.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsEntry {
    area: Box<str>,
    path: Box<str>,
    custom_dice: Box<str>,
    dice_faces: Box<[Box<str>]>,
    participants: ParticipantLock,
    neural_network: Box<[Box<str>]>,
    stats_conundrum: u8,
    auxiliary_conundrum: u8,
    completed_formal_areas: Box<[Box<str>]>,
    trailblaze_bonus: Option<Box<str>>,
    unlocked_dice: Box<[Box<str>]>,
}

impl GoldAndGearsEntry {
    #[must_use]
    pub fn new(
        area: impl Into<Box<str>>,
        path: impl Into<Box<str>>,
        custom_dice: impl Into<Box<str>>,
        dice_faces: Vec<String>,
        participants: ParticipantLock,
    ) -> Self {
        Self {
            area: area.into(),
            path: path.into(),
            custom_dice: custom_dice.into(),
            dice_faces: boxed_strings(dice_faces),
            participants,
            neural_network: Box::new([]),
            stats_conundrum: 0,
            auxiliary_conundrum: 0,
            completed_formal_areas: Box::new([]),
            trailblaze_bonus: None,
            unlocked_dice: Box::new([]),
        }
    }

    #[must_use]
    pub fn with_neural_network(mut self, nodes: Vec<String>) -> Self {
        self.neural_network = boxed_strings(nodes);
        self
    }

    #[must_use]
    pub fn with_conundrum(
        mut self,
        stats: u8,
        auxiliary: u8,
        completed_formal_areas: Vec<String>,
    ) -> Self {
        self.stats_conundrum = stats;
        self.auxiliary_conundrum = auxiliary;
        self.completed_formal_areas = boxed_strings(completed_formal_areas);
        self
    }

    #[must_use]
    pub fn with_trailblaze_bonus(mut self, bonus: impl Into<Box<str>>) -> Self {
        self.trailblaze_bonus = Some(bonus.into());
        self
    }

    #[must_use]
    pub fn with_unlocked_dice(mut self, dice: Vec<String>) -> Self {
        self.unlocked_dice = boxed_strings(dice);
        self
    }

    #[must_use]
    pub fn area(&self) -> &str {
        &self.area
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn custom_dice(&self) -> &str {
        &self.custom_dice
    }

    #[must_use]
    pub fn dice_faces(&self) -> impl ExactSizeIterator<Item = &str> {
        self.dice_faces.iter().map(Box::as_ref)
    }

    #[must_use]
    pub const fn participants(&self) -> &ParticipantLock {
        &self.participants
    }

    #[must_use]
    pub fn neural_network(&self) -> impl ExactSizeIterator<Item = &str> {
        self.neural_network.iter().map(Box::as_ref)
    }

    #[must_use]
    pub const fn stats_conundrum(&self) -> u8 {
        self.stats_conundrum
    }

    #[must_use]
    pub const fn auxiliary_conundrum(&self) -> u8 {
        self.auxiliary_conundrum
    }

    #[must_use]
    pub fn trailblaze_bonus(&self) -> Option<&str> {
        self.trailblaze_bonus.as_deref()
    }
}

/// Shared immutable catalog facade and the only Gold entry compiler.
#[derive(Clone, Debug)]
pub struct GoldAndGearsRuntimeFactory {
    pub(super) structural: Arc<GoldAndGearsStructuralCatalog>,
    pub(super) unique: Arc<GoldAndGearsUniqueCatalog>,
}

impl GoldAndGearsRuntimeFactory {
    /// Loads and validates the exact Candidate component used by this phase.
    ///
    /// Core combat and shared Universe components are added to this same
    /// factory before its production release boundary is enabled.
    pub fn load_candidate(bytes: &[u8]) -> Result<Self, GoldAndGearsEntryError> {
        let structural = GoldAndGearsStructuralCatalog::load(bytes)
            .map_err(|_| GoldAndGearsEntryError::InvalidCatalog)?;
        let unique = GoldAndGearsUniqueCatalog::load(bytes)
            .map_err(|_| GoldAndGearsEntryError::InvalidCatalog)?;
        if structural.bundle != unique.bundle
            || !structural
                .profiles
                .iter()
                .any(|profile| profile.stable_key.as_ref() == EXPECTED_PROFILE_KEY)
        {
            return Err(GoldAndGearsEntryError::InvalidCatalog);
        }
        Ok(Self {
            structural: Arc::new(structural),
            unique: Arc::new(unique),
        })
    }

    /// Validates every selected input and compiles exactly one generic
    /// Activity state profile. It performs no random draw.
    pub fn compile_entry(
        &self,
        entry: GoldAndGearsEntry,
    ) -> Result<GoldAndGearsRuntimeInstance, GoldAndGearsEntryError> {
        validate_participants(entry.participants.policy())?;
        let area = self.formal_area(&entry.area)?;
        let path = self
            .unique
            .paths
            .iter()
            .find(|path| path.identity.stable_key.as_ref() == entry.path.as_ref())
            .ok_or_else(|| GoldAndGearsEntryError::UnknownPath(entry.path.clone()))?;
        let unlocked_dice = canonical_unlocked_dice(&self.unique, &entry.unlocked_dice)?;
        let dice = self
            .unique
            .dice
            .iter()
            .find(|dice| dice.identity.stable_key.as_ref() == entry.custom_dice.as_ref())
            .ok_or_else(|| GoldAndGearsEntryError::UnknownDice(entry.custom_dice.clone()))?;
        if !dice.available_by_default
            && unlocked_dice
                .binary_search_by(|candidate| candidate.as_ref().cmp(&entry.custom_dice))
                .is_err()
        {
            return Err(GoldAndGearsEntryError::LockedDice(
                entry.custom_dice.clone(),
            ));
        }
        let faces = validate_loadout(&self.unique, dice, &entry.dice_faces)?;
        let neural = canonical_neural_network(&self.unique, &entry.neural_network)?;
        let completed_areas =
            canonical_completed_areas(&self.structural, &entry.completed_formal_areas)?;
        validate_conundrum(
            &self.unique,
            area,
            entry.stats_conundrum,
            entry.auxiliary_conundrum,
            &completed_areas,
        )?;
        let trailblaze_bonus = entry
            .trailblaze_bonus
            .as_deref()
            .map(|key| {
                self.unique
                    .trailblaze_bonuses
                    .iter()
                    .find(|bonus| bonus.identity.stable_key.as_ref() == key)
                    .ok_or_else(|| GoldAndGearsEntryError::UnknownTrailblazeBonus(key.into()))
            })
            .transpose()?;
        let cognition = self
            .unique
            .cognition_ranges
            .iter()
            .find(|range| range.area_key.as_ref() == area.stable_key.as_ref())
            .ok_or(GoldAndGearsEntryError::MissingCognitionRange)?;
        let cognition_minimum = parse_integer(&cognition.minimum.0)?;
        let cognition_maximum = parse_integer(&cognition.maximum.0)?;
        let state = compile_state(
            area,
            path.identity.id.0,
            dice.identity.id.0,
            &faces,
            &neural,
            entry.stats_conundrum,
            entry.auxiliary_conundrum,
            trailblaze_bonus.map(|bonus| bonus.identity.id.0),
            cognition_minimum,
            cognition_maximum,
        )?;

        Ok(GoldAndGearsRuntimeInstance {
            area: area.stable_key.clone(),
            difficulty: area.difficulty,
            path: path.identity.stable_key.clone(),
            custom_dice: dice.identity.stable_key.clone(),
            dice_faces: faces
                .iter()
                .map(|face| face.identity.stable_key.clone())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            participants: Arc::new(entry.participants),
            neural_network: neural
                .iter()
                .map(|node| node.identity.stable_key.clone())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            stats_conundrum: entry.stats_conundrum,
            auxiliary_conundrum: entry.auxiliary_conundrum,
            trailblaze_bonus: trailblaze_bonus.map(|bonus| bonus.identity.stable_key.clone()),
            state,
        })
    }

    fn formal_area(&self, key: &str) -> Result<&AreaDefinition, GoldAndGearsEntryError> {
        let area = self
            .structural
            .areas
            .iter()
            .find(|area| area.stable_key.as_ref() == key)
            .ok_or_else(|| GoldAndGearsEntryError::UnknownArea(key.into()))?;
        if area.group != AreaGroup::Formal {
            return Err(GoldAndGearsEntryError::GuideArea(key.into()));
        }
        Ok(area)
    }
}

/// Entry-compiled immutable Activity profile.
///
/// Later batches attach the bounded graph and generic runtime to this value;
/// all entry-owned state already uses the final seventeen slot families.
#[derive(Clone, Debug)]
pub struct GoldAndGearsRuntimeInstance {
    area: Box<str>,
    difficulty: u8,
    path: Box<str>,
    custom_dice: Box<str>,
    dice_faces: Box<[Box<str>]>,
    participants: Arc<ParticipantLock>,
    neural_network: Box<[Box<str>]>,
    stats_conundrum: u8,
    auxiliary_conundrum: u8,
    trailblaze_bonus: Option<Box<str>>,
    state: ActivityStateDefinition,
}

impl GoldAndGearsRuntimeInstance {
    #[must_use]
    pub fn area(&self) -> &str {
        &self.area
    }

    #[must_use]
    pub const fn difficulty(&self) -> u8 {
        self.difficulty
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn custom_dice(&self) -> &str {
        &self.custom_dice
    }

    #[must_use]
    pub fn dice_faces(&self) -> impl ExactSizeIterator<Item = &str> {
        self.dice_faces.iter().map(Box::as_ref)
    }

    #[must_use]
    pub const fn participants(&self) -> &Arc<ParticipantLock> {
        &self.participants
    }

    #[must_use]
    pub fn neural_network(&self) -> impl ExactSizeIterator<Item = &str> {
        self.neural_network.iter().map(Box::as_ref)
    }

    #[must_use]
    pub const fn stats_conundrum(&self) -> u8 {
        self.stats_conundrum
    }

    #[must_use]
    pub const fn auxiliary_conundrum(&self) -> u8 {
        self.auxiliary_conundrum
    }

    #[must_use]
    pub fn trailblaze_bonus(&self) -> Option<&str> {
        self.trailblaze_bonus.as_deref()
    }

    #[must_use]
    pub const fn state_definition(&self) -> &ActivityStateDefinition {
        &self.state
    }
}

fn boxed_strings(values: Vec<String>) -> Box<[Box<str>]> {
    values
        .into_iter()
        .map(String::into_boxed_str)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}
