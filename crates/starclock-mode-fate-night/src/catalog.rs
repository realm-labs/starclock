use std::num::NonZeroU32;

use crate::FateBoard;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(NonZeroU32);

        impl $name {
            #[must_use]
            pub const fn new(raw: u32) -> Option<Self> {
                match NonZeroU32::new(raw) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            #[must_use]
            pub const fn get(self) -> u32 {
                self.0.get()
            }
        }
    };
}

id_type!(FateCardId);
id_type!(FateDeckId);
id_type!(FateDeckRecommendationId);
id_type!(FateStoryFightId);
id_type!(FateChallengeFightId);
id_type!(FateMapFightId);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FateCardOwner {
    Trailblazer,
    Gilgamesh,
    Archer,
    Saber,
    Rin,
    Neutral,
}

impl FateCardOwner {
    #[must_use]
    pub const fn owns_deck(self) -> bool {
        matches!(
            self,
            Self::Trailblazer | Self::Gilgamesh | Self::Archer | Self::Saber
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FateCardRarity {
    R,
    Sr,
    Ssr,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FateDeckRecommendationKind {
    Base,
    Final,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FateCard {
    pub id: FateCardId,
    pub stable_key: Box<str>,
    pub owner: FateCardOwner,
    pub magical_energy_cost: u16,
    pub rarity: FateCardRarity,
    pub ability_program: Option<Box<str>>,
    pub runtime_binding_exact: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FateDeck {
    pub id: FateDeckId,
    pub stable_key: Box<str>,
    pub owner: FateCardOwner,
    pub presentation_locator: u32,
    pub action_locator: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FateDeckRecommendation {
    pub id: FateDeckRecommendationId,
    pub owner: FateCardOwner,
    pub kind: FateDeckRecommendationKind,
    pub owner_cards: Box<[FateCardId]>,
    pub neutral_cards: Box<[FateCardId]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FateStoryFight {
    pub id: FateStoryFightId,
    pub battle_event_id: u32,
    pub map_entrance_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FateChallengeFight {
    pub id: FateChallengeFightId,
    pub battle_event_id: u32,
    pub map_entrance_id: u32,
    pub enemy_id: u32,
    pub buff_ids: Box<[u32]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FateMapFight {
    pub id: FateMapFightId,
    pub battle_event_ids: Box<[u32]>,
    pub map_entrance_id: u32,
    pub reward_card: Option<FateCardId>,
    pub terminal: bool,
    pub enemy_id: u32,
    pub relation: Option<Box<str>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FateRuntimePolicy {
    pub id: Box<str>,
    pub unavailable_fact: Box<str>,
    pub known_facts: Box<str>,
    pub selected_behavior: Box<str>,
    pub rejected_alternatives: Box<[Box<str>]>,
    pub rationale: Box<str>,
    pub affected_tests: Box<[Box<str>]>,
    pub confidence: Box<str>,
    pub replacement_condition: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FateCatalog {
    boards: Box<[FateBoard]>,
    owners: Box<[FateCardOwner]>,
    cards: Box<[FateCard]>,
    decks: Box<[FateDeck]>,
    recommendations: Box<[FateDeckRecommendation]>,
    story_fights: Box<[FateStoryFight]>,
    challenge_fights: Box<[FateChallengeFight]>,
    map_fights: Box<[FateMapFight]>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FateCatalogParts {
    pub boards: Vec<FateBoard>,
    pub owners: Vec<FateCardOwner>,
    pub cards: Vec<FateCard>,
    pub decks: Vec<FateDeck>,
    pub recommendations: Vec<FateDeckRecommendation>,
    pub story_fights: Vec<FateStoryFight>,
    pub challenge_fights: Vec<FateChallengeFight>,
    pub map_fights: Vec<FateMapFight>,
}

impl FateCatalog {
    pub fn new(parts: FateCatalogParts) -> Result<Self, FateCatalogError> {
        let FateCatalogParts {
            mut boards,
            mut owners,
            mut cards,
            mut decks,
            mut recommendations,
            mut story_fights,
            mut challenge_fights,
            mut map_fights,
        } = parts;
        boards.sort_by_key(FateBoard::id);
        owners.sort_unstable();
        cards.sort_by_key(|item| item.id);
        decks.sort_by_key(|item| item.id);
        recommendations.sort_by_key(|item| item.id);
        story_fights.sort_by_key(|item| item.id);
        challenge_fights.sort_by_key(|item| item.id);
        map_fights.sort_by_key(|item| item.id);

        if !unique(&boards, FateBoard::id) {
            return Err(FateCatalogError::DuplicateBoard);
        }
        if owners.is_empty() || owners.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(FateCatalogError::InvalidOwners);
        }
        if !unique(&cards, |item| item.id)
            || cards.iter().any(|item| {
                item.stable_key.trim().is_empty()
                    || item.magical_energy_cost == 0
                    || item
                        .ability_program
                        .as_deref()
                        .is_some_and(|value| value.trim().is_empty())
                    || item.runtime_binding_exact && item.ability_program.is_none()
            })
        {
            return Err(FateCatalogError::InvalidCard);
        }
        if !unique(&decks, |item| item.id)
            || decks.iter().any(|item| {
                item.stable_key.trim().is_empty()
                    || !item.owner.owns_deck()
                    || item.presentation_locator == 0
                    || item.action_locator == 0
                    || owners.binary_search(&item.owner).is_err()
            })
        {
            return Err(FateCatalogError::InvalidDeck);
        }
        validate_recommendations(&recommendations, &decks, &cards)?;
        if !unique(&story_fights, |item| item.id)
            || story_fights
                .iter()
                .any(|item| item.battle_event_id == 0 || item.map_entrance_id == 0)
        {
            return Err(FateCatalogError::InvalidStoryFight);
        }
        if !unique(&challenge_fights, |item| item.id)
            || challenge_fights.iter().any(|item| {
                item.battle_event_id == 0
                    || item.map_entrance_id == 0
                    || item.enemy_id == 0
                    || item.buff_ids.is_empty()
                    || item.buff_ids.contains(&0)
                    || !strictly_increasing(&item.buff_ids)
            })
        {
            return Err(FateCatalogError::InvalidChallengeFight);
        }
        if !unique(&map_fights, |item| item.id)
            || map_fights.iter().any(|item| {
                !(1..=3).contains(&item.battle_event_ids.len())
                    || item.battle_event_ids.contains(&0)
                    || item.map_entrance_id == 0
                    || item.enemy_id == 0
                    || item
                        .relation
                        .as_deref()
                        .is_some_and(|value| value.trim().is_empty())
                    || item
                        .reward_card
                        .is_some_and(|id| cards.binary_search_by_key(&id, |card| card.id).is_err())
            })
        {
            return Err(FateCatalogError::InvalidMapFight);
        }
        Ok(Self {
            boards: boards.into_boxed_slice(),
            owners: owners.into_boxed_slice(),
            cards: cards.into_boxed_slice(),
            decks: decks.into_boxed_slice(),
            recommendations: recommendations.into_boxed_slice(),
            story_fights: story_fights.into_boxed_slice(),
            challenge_fights: challenge_fights.into_boxed_slice(),
            map_fights: map_fights.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn boards(&self) -> &[FateBoard] {
        &self.boards
    }

    #[must_use]
    pub fn owners(&self) -> &[FateCardOwner] {
        &self.owners
    }

    #[must_use]
    pub fn cards(&self) -> &[FateCard] {
        &self.cards
    }

    #[must_use]
    pub fn decks(&self) -> &[FateDeck] {
        &self.decks
    }

    #[must_use]
    pub fn recommendations(&self) -> &[FateDeckRecommendation] {
        &self.recommendations
    }

    #[must_use]
    pub fn story_fights(&self) -> &[FateStoryFight] {
        &self.story_fights
    }

    #[must_use]
    pub fn challenge_fights(&self) -> &[FateChallengeFight] {
        &self.challenge_fights
    }

    #[must_use]
    pub fn map_fights(&self) -> &[FateMapFight] {
        &self.map_fights
    }

    #[must_use]
    pub fn card(&self, id: FateCardId) -> Option<&FateCard> {
        self.cards
            .binary_search_by_key(&id, |card| card.id)
            .ok()
            .map(|index| &self.cards[index])
    }

    #[must_use]
    pub fn deck(&self, id: FateDeckId) -> Option<&FateDeck> {
        self.decks
            .binary_search_by_key(&id, |deck| deck.id)
            .ok()
            .map(|index| &self.decks[index])
    }
}

fn validate_recommendations(
    recommendations: &[FateDeckRecommendation],
    decks: &[FateDeck],
    cards: &[FateCard],
) -> Result<(), FateCatalogError> {
    if !unique(recommendations, |item| item.id)
        || recommendations.iter().enumerate().any(|(index, item)| {
            recommendations[index + 1..]
                .iter()
                .any(|other| (item.owner, item.kind) == (other.owner, other.kind))
        })
    {
        return Err(FateCatalogError::InvalidRecommendation);
    }
    for recommendation in recommendations {
        if decks.iter().all(|deck| deck.owner != recommendation.owner)
            || recommendation.owner_cards.is_empty()
            || recommendation.neutral_cards.is_empty()
            || !strictly_unique(&recommendation.owner_cards)
            || !strictly_unique(&recommendation.neutral_cards)
        {
            return Err(FateCatalogError::InvalidRecommendation);
        }
        for (ids, owner) in [
            (&recommendation.owner_cards, recommendation.owner),
            (&recommendation.neutral_cards, FateCardOwner::Neutral),
        ] {
            if ids.iter().any(|id| {
                cards
                    .binary_search_by_key(id, |card| card.id)
                    .ok()
                    .is_none_or(|index| cards[index].owner != owner)
            }) {
                return Err(FateCatalogError::InvalidRecommendation);
            }
        }
    }
    Ok(())
}

fn unique<T, K: Eq>(items: &[T], key: impl Fn(&T) -> K) -> bool {
    items.windows(2).all(|pair| key(&pair[0]) != key(&pair[1]))
}

fn strictly_unique<T: Eq>(items: &[T]) -> bool {
    items
        .iter()
        .enumerate()
        .all(|(index, item)| !items[index + 1..].contains(item))
}

fn strictly_increasing(items: &[u32]) -> bool {
    items.windows(2).all(|pair| pair[0] < pair[1])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FateCatalogError {
    DuplicateBoard,
    InvalidOwners,
    InvalidCard,
    InvalidDeck,
    InvalidRecommendation,
    InvalidStoryFight,
    InvalidChallengeFight,
    InvalidMapFight,
}
