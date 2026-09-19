//! Policy-selected source decks; no automatic mask eligibility or room effects.

use std::collections::BTreeSet;
use std::num::NonZeroU64;

use crate::divergent_universe_decisions::DecisionDataError;
use crate::divergent_universe_decisions_generated::{
    SoraConfig, du_domain_card_kind::DuDomainCardKind, du_domain_deck_policy::DuDomainDeckPolicy,
};

/// Starclock-authored instance identity, not the upstream preset ID.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DomainCardInstanceId(NonZeroU64);

impl DomainCardInstanceId {
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomainCardKind {
    Battle,
    Encounter,
    Event,
    Coin,
    Shop,
    Reward,
    Adventure,
    Reforge,
    Elite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomainDeckAdmission {
    /// Caller-selected source record. Does not establish released profile membership.
    VersionedProjectPolicyExplicitSourceDeck,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainCardDefinition {
    pub instance: DomainCardInstanceId,
    pub key: Box<str>,
    pub ordinal: u16,
    pub preset_source: Box<str>,
    pub kind: DomainCardKind,
    pub level: u16,
    pub source_locator: Box<str>,
}

/// Exact source order and multiplicity under a replaceable field-role policy.
/// Absence from this catalog is not exclusion evidence for another source row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainDeckDefinition {
    pub key: Box<str>,
    pub style_source: Box<str>,
    pub admission: DomainDeckAdmission,
    pub cards: Box<[DomainCardDefinition]>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub source_locator: Box<str>,
    pub sources: Box<[Box<str>]>,
}

pub(crate) fn compile(
    config: &SoraConfig,
) -> Result<Box<[DomainDeckDefinition]>, DecisionDataError> {
    let sources = [
        (
            67,
            "RoguePersonaStyle",
            "49b1c63d1f9dcad5201ae6f6446c01b9f401506b5625c39fc4da4b62378c6834",
        ),
        (
            68,
            "RoguePersonaRoomPreset",
            "a4cc8bbd6e3a4db5fecb62b83a0c5de7ad2f4a9820ad5ae9180b30ae33c346b5",
        ),
        (
            69,
            "RoguePersonaRoomCompType",
            "c0223603c6e5252278dac5224d766f1c4ed60ebe5845b9839b9e3533a8ad76a5",
        ),
    ]
    .into_iter()
    .map(|(id, name, hash)| {
        let source = config
            .du_decision_sources()
            .get(&id)
            .ok_or(DecisionDataError::InvalidProvenance)?;
        if source.stable_key != format!("du.source.domain-deck.{}", name.to_ascii_lowercase())
            || source.sha256 != hash
            || source.revision != "fd978d6ef09f941fba644c731ab54abd6f7c3568"
            || source.game_version != "4.4"
        {
            return Err(DecisionDataError::InvalidProvenance);
        }
        Ok(source.stable_key.clone().into_boxed_str())
    })
    .collect::<Result<Box<[_]>, _>>()?;
    let mut used = BTreeSet::new();
    let mut styles = BTreeSet::new();
    let mut keys = BTreeSet::new();
    let mut result = Vec::new();
    for row in config.du_domain_decks().ordered_rows() {
        let (slug, count) = match row.style_source.as_str() {
            "101" => ("gladiator", 14),
            "102" => ("camera", 13),
            "103" => ("mechatron", 18),
            "104" => ("horse", 13),
            "105" => ("fortune-cat", 14),
            "106" => ("cringe", 15),
            "107" => ("slipshod", 12),
            "108" => ("chariot", 13),
            "110" => ("two", 13),
            _ => return Err(DecisionDataError::InvalidReference),
        };
        if row.stable_key != format!("du.domain-deck.{slug}")
            || !keys.insert(row.stable_key.as_str())
            || !styles.insert(row.style_source.as_str())
            || row.card_count != count
            || row.source_ids.as_slice() != [67, 68, 69]
            || row.policy_note.is_empty()
            || row.replacement_condition.is_empty()
            || row.source_locator
                != format!(
                    "RoguePersonaStyle:KLOEJIMMPJM={};JEHDKAKMCGC",
                    row.style_source
                )
        {
            return Err(DecisionDataError::InvalidProvenance);
        }
        let admission = match row.policy {
            DuDomainDeckPolicy::ExplicitSourceDeck => {
                DomainDeckAdmission::VersionedProjectPolicyExplicitSourceDeck
            }
        };
        let mut cards = Vec::new();
        for card in config
            .du_domain_cards()
            .ordered_rows()
            .filter(|card| card.deck_id == row.id)
        {
            let (kind, level) = preset(&card.preset_source)?;
            if kind != card.kind
                || level != card.level
                || card.ordinal < 1
                || card.stable_key != format!("{}.card-{}", row.stable_key, card.ordinal)
                || card.source_ids.as_slice() != [67, 68, 69]
                || card.source_locator
                    != format!(
                        "RoguePersonaStyle:KLOEJIMMPJM={};JEHDKAKMCGC[{}];RoguePersonaRoomPreset:LIIPLGLNPGB={}",
                        row.style_source,
                        card.ordinal - 1,
                        card.preset_source
                    )
                || !used.insert(card.id)
                || !keys.insert(card.stable_key.as_str())
            {
                return Err(DecisionDataError::InvalidReference);
            }
            let raw = u64::try_from(card.id).map_err(|_| DecisionDataError::InvalidIdentity)?;
            let instance = DomainCardInstanceId(
                NonZeroU64::new(raw).ok_or(DecisionDataError::InvalidIdentity)?,
            );
            let kind = match kind {
                DuDomainCardKind::Battle => DomainCardKind::Battle,
                DuDomainCardKind::Encounter => DomainCardKind::Encounter,
                DuDomainCardKind::Event => DomainCardKind::Event,
                DuDomainCardKind::Coin => DomainCardKind::Coin,
                DuDomainCardKind::Shop => DomainCardKind::Shop,
                DuDomainCardKind::Reward => DomainCardKind::Reward,
                DuDomainCardKind::Adventure => DomainCardKind::Adventure,
                DuDomainCardKind::Reforge => DomainCardKind::Reforge,
                DuDomainCardKind::Elite => DomainCardKind::Elite,
            };
            cards.push(DomainCardDefinition {
                instance,
                key: card.stable_key.clone().into(),
                ordinal: u16::try_from(card.ordinal)
                    .map_err(|_| DecisionDataError::InvalidOrder)?,
                preset_source: card.preset_source.clone().into(),
                kind,
                level: u16::try_from(level).map_err(|_| DecisionDataError::InvalidPolicy)?,
                source_locator: card.source_locator.clone().into(),
            });
        }
        cards.sort_by_key(|card| card.ordinal);
        if cards.len() != usize::try_from(count).map_err(|_| DecisionDataError::InvalidOrder)?
            || cards
                .iter()
                .enumerate()
                .any(|(index, card)| usize::from(card.ordinal) != index + 1)
        {
            return Err(DecisionDataError::InvalidOrder);
        }
        result.push(DomainDeckDefinition {
            key: row.stable_key.clone().into(),
            style_source: row.style_source.clone().into(),
            admission,
            cards: cards.into_boxed_slice(),
            policy_note: row.policy_note.clone().into(),
            replacement_condition: row.replacement_condition.clone().into(),
            source_locator: row.source_locator.clone().into(),
            sources: sources.clone(),
        });
    }
    if result.len() != 9 || used.len() != config.du_domain_cards().ordered_rows().count() {
        return Err(DecisionDataError::InvalidReference);
    }
    result.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(result.into_boxed_slice())
}

fn preset(source: &str) -> Result<(DuDomainCardKind, i32), DecisionDataError> {
    Ok(match source {
        "1005" => (DuDomainCardKind::Battle, 1),
        "1006" => (DuDomainCardKind::Encounter, 1),
        "1007" => (DuDomainCardKind::Event, 1),
        "1008" => (DuDomainCardKind::Coin, 1),
        "1009" => (DuDomainCardKind::Shop, 1),
        "1010" => (DuDomainCardKind::Reward, 1),
        "1011" => (DuDomainCardKind::Adventure, 1),
        "1017" => (DuDomainCardKind::Reforge, 1),
        "1020" => (DuDomainCardKind::Elite, 1),
        "1021" => (DuDomainCardKind::Event, 2),
        "1022" => (DuDomainCardKind::Encounter, 2),
        "1023" => (DuDomainCardKind::Battle, 2),
        "1024" => (DuDomainCardKind::Coin, 2),
        "1025" => (DuDomainCardKind::Elite, 2),
        "1026" => (DuDomainCardKind::Shop, 2),
        _ => return Err(DecisionDataError::InvalidReference),
    })
}
