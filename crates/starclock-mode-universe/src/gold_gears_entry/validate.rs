use starclock_activity::{LoadoutLockScope, ParticipantPolicy, ParticipantUniquenessScope};

use crate::{
    gold_gears_structural::{AreaDefinition, AreaGroup, GoldAndGearsStructuralCatalog},
    gold_gears_unique::{DiceDefinition, DiceFace, GoldAndGearsUniqueCatalog, NeuralNode},
};

use super::{CONUNDRUM_AREA_KEY, GoldAndGearsEntryError};

pub(super) fn validate_participants(
    actual: ParticipantPolicy,
) -> Result<(), GoldAndGearsEntryError> {
    let expected = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .expect("static Gold and Gears participant policy is valid");
    if actual != expected {
        return Err(GoldAndGearsEntryError::ParticipantPolicyMismatch);
    }
    Ok(())
}

pub(super) fn canonical_unlocked_dice(
    catalog: &GoldAndGearsUniqueCatalog,
    input: &[Box<str>],
) -> Result<Vec<Box<str>>, GoldAndGearsEntryError> {
    let mut selected = input.to_vec();
    selected.sort_unstable();
    reject_duplicate(&selected, GoldAndGearsEntryError::DuplicateUnlockedDice)?;
    for key in &selected {
        if !catalog
            .dice
            .iter()
            .any(|dice| dice.identity.stable_key == *key)
        {
            return Err(GoldAndGearsEntryError::UnknownDice(key.clone()));
        }
    }
    Ok(selected)
}

pub(super) fn canonical_completed_areas(
    catalog: &GoldAndGearsStructuralCatalog,
    input: &[Box<str>],
) -> Result<Vec<Box<str>>, GoldAndGearsEntryError> {
    let mut selected = input.to_vec();
    selected.sort_unstable();
    reject_duplicate(&selected, GoldAndGearsEntryError::DuplicateCompletedArea)?;
    for key in &selected {
        if !catalog
            .areas
            .iter()
            .any(|area| area.group == AreaGroup::Formal && area.stable_key == *key)
        {
            return Err(GoldAndGearsEntryError::UnknownCompletedArea(key.clone()));
        }
    }
    Ok(selected)
}

pub(super) fn canonical_neural_network<'a>(
    catalog: &'a GoldAndGearsUniqueCatalog,
    input: &[Box<str>],
) -> Result<Vec<&'a NeuralNode>, GoldAndGearsEntryError> {
    let mut selected = input
        .iter()
        .map(|key| {
            catalog
                .neural_nodes
                .iter()
                .find(|node| node.identity.stable_key == *key)
                .ok_or_else(|| GoldAndGearsEntryError::UnknownNeuralNode(key.clone()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    selected.sort_by_key(|node| node.identity.id.0);
    if let Some(pair) = selected
        .windows(2)
        .find(|pair| pair[0].identity.id == pair[1].identity.id)
    {
        return Err(GoldAndGearsEntryError::DuplicateNeuralNode(
            pair[0].identity.stable_key.clone(),
        ));
    }
    for node in &selected {
        if let Some(prerequisite) = node.prerequisites.iter().find(|prerequisite| {
            !selected
                .iter()
                .any(|candidate| candidate.identity.stable_key == **prerequisite)
        }) {
            return Err(GoldAndGearsEntryError::MissingNeuralPrerequisite {
                node: node.identity.stable_key.clone(),
                prerequisite: prerequisite.clone(),
            });
        }
    }
    Ok(selected)
}

pub(super) fn validate_loadout<'a>(
    catalog: &'a GoldAndGearsUniqueCatalog,
    dice: &DiceDefinition,
    input: &[Box<str>],
) -> Result<Vec<&'a DiceFace>, GoldAndGearsEntryError> {
    if input.len() != catalog.dice_slots.len() || input.len() != 6 {
        return Err(GoldAndGearsEntryError::InvalidDiceFaceCount);
    }
    let mut faces = Vec::with_capacity(input.len());
    for (slot, key) in catalog.dice_slots.iter().zip(input) {
        let face = catalog
            .dice_faces
            .iter()
            .find(|face| face.identity.stable_key == *key)
            .ok_or_else(|| GoldAndGearsEntryError::UnknownDiceFace(key.clone()))?;
        if faces
            .iter()
            .any(|selected: &&DiceFace| selected.identity.id == face.identity.id)
        {
            return Err(GoldAndGearsEntryError::DuplicateDiceFace(key.clone()));
        }
        if !face
            .allowed_slot_keys
            .iter()
            .any(|allowed| allowed == &slot.identity.stable_key)
        {
            return Err(GoldAndGearsEntryError::DiceFaceSlotMismatch(key.clone()));
        }
        if !face.universal_dice_eligibility
            && !face
                .allowed_dice_keys
                .iter()
                .any(|allowed| allowed == &dice.identity.stable_key)
        {
            return Err(GoldAndGearsEntryError::DiceFaceDiceMismatch(key.clone()));
        }
        if face.rarity > slot.base_max_rarity {
            return Err(GoldAndGearsEntryError::DiceFaceRarityMismatch(key.clone()));
        }
        faces.push(face);
    }
    Ok(faces)
}

pub(super) fn validate_conundrum(
    catalog: &GoldAndGearsUniqueCatalog,
    area: &AreaDefinition,
    stats: u8,
    auxiliary: u8,
    completed_areas: &[Box<str>],
) -> Result<(), GoldAndGearsEntryError> {
    if stats > 6 || auxiliary > 6 || stats.checked_add(auxiliary).is_none_or(|total| total > 12) {
        return Err(GoldAndGearsEntryError::InvalidConundrumLevel);
    }
    for (track, level) in [("Stats", stats), ("Auxiliary", auxiliary)] {
        if level > 0
            && !catalog
                .conundrum_levels
                .iter()
                .any(|entry| entry.track.as_ref() == track && entry.level == level)
        {
            return Err(GoldAndGearsEntryError::InvalidConundrumLevel);
        }
    }
    if stats == 0 && auxiliary == 0 {
        return Ok(());
    }
    if area.difficulty != 5 || area.stable_key.as_ref() != CONUNDRUM_AREA_KEY {
        return Err(GoldAndGearsEntryError::ConundrumDifficultyMismatch);
    }
    if completed_areas
        .binary_search_by(|key| key.as_ref().cmp(CONUNDRUM_AREA_KEY))
        .is_err()
    {
        return Err(GoldAndGearsEntryError::MissingConundrumPrerequisite);
    }
    Ok(())
}

pub(super) fn parse_integer(value: &str) -> Result<i64, GoldAndGearsEntryError> {
    value
        .parse()
        .map_err(|_| GoldAndGearsEntryError::InvalidCatalog)
}

fn reject_duplicate(
    values: &[Box<str>],
    error: impl FnOnce(Box<str>) -> GoldAndGearsEntryError,
) -> Result<(), GoldAndGearsEntryError> {
    if let Some(pair) = values.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(error(pair[0].clone()));
    }
    Ok(())
}
