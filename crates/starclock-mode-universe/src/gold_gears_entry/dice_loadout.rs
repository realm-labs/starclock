//! Immutable Custom Dice loadout rules compiled from the private Sora catalog.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use crate::gold_gears_unique::{
    DiceDefinition, DiceFace, DiceSlot, GoldAndGearsUniqueCatalog, NeuralNode,
};

use super::GoldAndGearsEntryError;

const BASELINE_FACE_UNLOCK_SOURCE: &str = "100";
const SLOT_UPGRADE_OPERATION: &str = "UpgradeDiceFaceSlot";
const SLOT_UPGRADE_POLICY: &str = "neural-network-slot-upgrade-target-v1";

#[derive(Clone, Debug)]
pub(super) struct DiceLoadoutRuntimeCatalog {
    slots: Box<[DiceSlotRule]>,
}

#[derive(Clone, Debug)]
struct DiceSlotRule {
    key: Box<str>,
    index: u8,
    base_max_rarity: u8,
    upgraded_max_rarity: u8,
    upgrade_node: Option<u32>,
}

#[derive(Debug)]
pub(super) struct CompiledDiceLoadout<'a> {
    pub(super) faces: Vec<&'a DiceFace>,
    pub(super) maximum_rarities: Box<[u8]>,
    pub(super) eligible_faces: Box<[Box<[Box<str>]>]>,
    pub(super) suggestive_faces: Box<[Box<str>]>,
    pub(super) recommended_faces: Box<[Box<str>]>,
}

impl DiceLoadoutRuntimeCatalog {
    pub(super) fn compile(
        catalog: &GoldAndGearsUniqueCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let mut slots = catalog
            .dice_slots
            .iter()
            .map(|slot| DiceSlotRule {
                key: slot.identity.stable_key.clone(),
                index: slot.index,
                base_max_rarity: slot.base_max_rarity,
                upgraded_max_rarity: slot.upgraded_max_rarity,
                upgrade_node: None,
            })
            .collect::<Vec<_>>();
        slots.sort_by_key(|slot| slot.index);
        validate_slots(&catalog.dice_slots, &slots)?;

        for node in &catalog.neural_nodes {
            for contribution in decode_contributions(node)? {
                if contribution.operation.as_ref() != SLOT_UPGRADE_OPERATION {
                    continue;
                }
                let target = contribution
                    .target
                    .as_deref()
                    .ok_or(GoldAndGearsEntryError::InvalidDiceLoadoutRuntime)?;
                let slot = slots
                    .iter_mut()
                    .find(|slot| slot.key.as_ref() == target)
                    .ok_or(GoldAndGearsEntryError::InvalidDiceLoadoutRuntime)?;
                if slot.upgrade_node.is_some()
                    || contribution.scope.as_deref() != Some("Activity")
                    || contribution.unit.as_deref() != Some("Rarity")
                    || contribution.from_max_rarity != Some(slot.base_max_rarity)
                    || contribution.to_max_rarity != Some(slot.upgraded_max_rarity)
                    || contribution.target_policy.as_ref().is_none_or(|policy| {
                        policy.policy_id.as_ref() != SLOT_UPGRADE_POLICY
                            || policy.evidence_quality.as_ref() != "ProjectPolicy"
                            || policy.mapping_basis.as_ref()
                                != "released-slot-capability-plus-stable-slot-order"
                            || policy.replacement_condition.is_empty()
                    })
                {
                    return Err(GoldAndGearsEntryError::InvalidDiceLoadoutRuntime);
                }
                slot.upgrade_node = Some(node.identity.id.0);
            }
        }
        if slots.iter().any(|slot| {
            (slot.base_max_rarity < slot.upgraded_max_rarity) != slot.upgrade_node.is_some()
        }) {
            return Err(GoldAndGearsEntryError::InvalidDiceLoadoutRuntime);
        }

        let runtime = Self {
            slots: slots.into_boxed_slice(),
        };
        runtime.validate_authored_loadouts(catalog)?;
        Ok(runtime)
    }

    pub(super) fn compile_loadout<'a>(
        &self,
        catalog: &'a GoldAndGearsUniqueCatalog,
        dice: &DiceDefinition,
        input: &[Box<str>],
        neural: &[&NeuralNode],
        unlocked_dice: &[Box<str>],
    ) -> Result<CompiledDiceLoadout<'a>, GoldAndGearsEntryError> {
        if input.len() != self.slots.len() || input.len() != 6 {
            return Err(GoldAndGearsEntryError::InvalidDiceFaceCount);
        }
        let selected_neural = neural
            .iter()
            .map(|node| node.identity.id.0)
            .collect::<BTreeSet<_>>();
        let maximum_rarities = self
            .slots
            .iter()
            .map(|slot| {
                if slot
                    .upgrade_node
                    .is_some_and(|node| selected_neural.contains(&node))
                {
                    slot.upgraded_max_rarity
                } else {
                    slot.base_max_rarity
                }
            })
            .collect::<Vec<_>>();
        let unlock_sources = unlocked_face_sources(catalog, unlocked_dice);
        let mut faces = Vec::with_capacity(input.len());
        for ((slot, maximum_rarity), key) in self
            .slots
            .iter()
            .zip(maximum_rarities.iter().copied())
            .zip(input)
        {
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
            validate_selected_face(face, slot, dice, maximum_rarity, &unlock_sources)?;
            faces.push(face);
        }

        let eligible_faces = self
            .slots
            .iter()
            .zip(maximum_rarities.iter().copied())
            .map(|(slot, maximum_rarity)| {
                let mut candidates = catalog
                    .dice_faces
                    .iter()
                    .filter(|face| {
                        face_is_eligible(face, slot, dice, maximum_rarity, &unlock_sources)
                    })
                    .collect::<Vec<_>>();
                candidates.sort_by_key(|face| face.identity.id.0);
                candidates
                    .into_iter()
                    .map(|face| face.identity.stable_key.clone())
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let suggestive_faces =
            available_recommendations(catalog, &dice.suggestive_face_sources, &eligible_faces)?;
        let recommended_faces =
            available_recommendations(catalog, &dice.recommended_face_sources, &eligible_faces)?;

        Ok(CompiledDiceLoadout {
            faces,
            maximum_rarities: maximum_rarities.into_boxed_slice(),
            eligible_faces,
            suggestive_faces,
            recommended_faces,
        })
    }

    fn validate_authored_loadouts(
        &self,
        catalog: &GoldAndGearsUniqueCatalog,
    ) -> Result<(), GoldAndGearsEntryError> {
        let faces_by_source = catalog
            .dice_faces
            .iter()
            .map(|face| (face.identity.source_id.as_ref(), face))
            .collect::<BTreeMap<_, _>>();
        let all_unlock_sources = catalog
            .dice
            .iter()
            .map(|dice| dice.identity.source_id.as_ref())
            .chain(core::iter::once(BASELINE_FACE_UNLOCK_SOURCE))
            .collect::<BTreeSet<_>>();
        for face in &catalog.dice_faces {
            if !all_unlock_sources.contains(face.unlock_display_source.as_ref()) {
                return Err(GoldAndGearsEntryError::InvalidDiceLoadoutRuntime);
            }
        }
        for dice in &catalog.dice {
            if dice.default_face_sources.len() != self.slots.len()
                || !unique(&dice.default_face_sources)
                || !unique(&dice.suggestive_face_sources)
                || !unique(&dice.recommended_face_sources)
            {
                return Err(GoldAndGearsEntryError::InvalidDiceLoadoutRuntime);
            }
            for ((slot, source), maximum_rarity) in self
                .slots
                .iter()
                .zip(dice.default_face_sources.iter())
                .zip(self.slots.iter().map(|slot| slot.base_max_rarity))
            {
                let face = faces_by_source
                    .get(source.as_ref())
                    .ok_or(GoldAndGearsEntryError::InvalidDiceLoadoutRuntime)?;
                if !face_is_eligible(face, slot, dice, maximum_rarity, &all_unlock_sources) {
                    return Err(GoldAndGearsEntryError::InvalidDiceLoadoutRuntime);
                }
            }
            for source in dice
                .suggestive_face_sources
                .iter()
                .chain(dice.recommended_face_sources.iter())
            {
                let face = faces_by_source
                    .get(source.as_ref())
                    .ok_or(GoldAndGearsEntryError::InvalidDiceLoadoutRuntime)?;
                if !self.slots.iter().any(|slot| {
                    face_is_eligible(
                        face,
                        slot,
                        dice,
                        slot.upgraded_max_rarity,
                        &all_unlock_sources,
                    )
                }) {
                    return Err(GoldAndGearsEntryError::InvalidDiceLoadoutRuntime);
                }
            }
        }
        Ok(())
    }
}

fn validate_slots(
    authored: &[DiceSlot],
    runtime: &[DiceSlotRule],
) -> Result<(), GoldAndGearsEntryError> {
    if authored.len() != 6
        || runtime
            .iter()
            .enumerate()
            .any(|(index, slot)| slot.index != u8::try_from(index + 1).unwrap_or(u8::MAX))
        || authored.iter().any(|slot| {
            let upgrades = slot.base_max_rarity < slot.upgraded_max_rarity;
            slot.extra_max_rarity.is_some() != upgrades
                || slot
                    .extra_max_rarity
                    .is_some_and(|extra| extra != slot.upgraded_max_rarity)
        })
    {
        return Err(GoldAndGearsEntryError::InvalidDiceLoadoutRuntime);
    }
    Ok(())
}

fn validate_selected_face(
    face: &DiceFace,
    slot: &DiceSlotRule,
    dice: &DiceDefinition,
    maximum_rarity: u8,
    unlock_sources: &BTreeSet<&str>,
) -> Result<(), GoldAndGearsEntryError> {
    if !unlock_sources.contains(face.unlock_display_source.as_ref()) {
        return Err(GoldAndGearsEntryError::LockedDiceFace(
            face.identity.stable_key.clone(),
        ));
    }
    if !face
        .allowed_slot_keys
        .iter()
        .any(|allowed| allowed == &slot.key)
    {
        return Err(GoldAndGearsEntryError::DiceFaceSlotMismatch(
            face.identity.stable_key.clone(),
        ));
    }
    if !face.universal_dice_eligibility
        && !face
            .allowed_dice_keys
            .iter()
            .any(|allowed| allowed == &dice.identity.stable_key)
    {
        return Err(GoldAndGearsEntryError::DiceFaceDiceMismatch(
            face.identity.stable_key.clone(),
        ));
    }
    if face.rarity > maximum_rarity {
        return Err(GoldAndGearsEntryError::DiceFaceRarityMismatch(
            face.identity.stable_key.clone(),
        ));
    }
    Ok(())
}

fn face_is_eligible(
    face: &DiceFace,
    slot: &DiceSlotRule,
    dice: &DiceDefinition,
    maximum_rarity: u8,
    unlock_sources: &BTreeSet<&str>,
) -> bool {
    unlock_sources.contains(face.unlock_display_source.as_ref())
        && face
            .allowed_slot_keys
            .iter()
            .any(|allowed| allowed == &slot.key)
        && (face.universal_dice_eligibility
            || face
                .allowed_dice_keys
                .iter()
                .any(|allowed| allowed == &dice.identity.stable_key))
        && face.rarity <= maximum_rarity
}

fn unlocked_face_sources<'a>(
    catalog: &'a GoldAndGearsUniqueCatalog,
    unlocked_dice: &[Box<str>],
) -> BTreeSet<&'a str> {
    let mut sources = BTreeSet::from([BASELINE_FACE_UNLOCK_SOURCE]);
    for dice in &catalog.dice {
        if dice.available_by_default
            || unlocked_dice
                .binary_search_by(|key| key.as_ref().cmp(&dice.identity.stable_key))
                .is_ok()
        {
            sources.insert(dice.identity.source_id.as_ref());
        }
    }
    sources
}

fn available_recommendations(
    catalog: &GoldAndGearsUniqueCatalog,
    sources: &[Box<str>],
    eligible_faces: &[Box<[Box<str>]>],
) -> Result<Box<[Box<str>]>, GoldAndGearsEntryError> {
    sources
        .iter()
        .map(|source| {
            let face = catalog
                .dice_faces
                .iter()
                .find(|face| face.identity.source_id == *source)
                .ok_or(GoldAndGearsEntryError::InvalidDiceLoadoutRuntime)?;
            Ok(face.identity.stable_key.clone())
        })
        .filter_map(|result| match result {
            Ok(key)
                if eligible_faces
                    .iter()
                    .any(|slot| slot.iter().any(|candidate| candidate == &key)) =>
            {
                Some(Ok(key))
            }
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn unique(values: &[Box<str>]) -> bool {
    let mut seen = BTreeSet::new();
    values.iter().all(|value| seen.insert(value.as_ref()))
}

fn decode_contributions(
    node: &NeuralNode,
) -> Result<Vec<NeuralContribution>, GoldAndGearsEntryError> {
    serde_json::from_str::<NeuralContributionEnvelope>(&node.effect_contributions_json)
        .map(NeuralContributionEnvelope::into_vec)
        .map_err(|_| GoldAndGearsEntryError::InvalidDiceLoadoutRuntime)
}

#[derive(Deserialize)]
#[serde(untagged)]
enum NeuralContributionEnvelope {
    Many(Vec<NeuralContribution>),
    One(NeuralContribution),
}

impl NeuralContributionEnvelope {
    fn into_vec(self) -> Vec<NeuralContribution> {
        match self {
            Self::Many(contributions) => contributions,
            Self::One(contribution) => vec![contribution],
        }
    }
}

#[derive(Deserialize)]
struct NeuralContribution {
    operation: Box<str>,
    #[serde(default)]
    scope: Option<Box<str>>,
    #[serde(default)]
    target: Option<Box<str>>,
    #[serde(default)]
    from_max_rarity: Option<u8>,
    #[serde(default)]
    to_max_rarity: Option<u8>,
    #[serde(default)]
    unit: Option<Box<str>>,
    #[serde(default)]
    target_policy: Option<UpgradeTargetPolicy>,
}

#[derive(Deserialize)]
struct UpgradeTargetPolicy {
    policy_id: Box<str>,
    evidence_quality: Box<str>,
    mapping_basis: Box<str>,
    replacement_condition: Box<str>,
}
