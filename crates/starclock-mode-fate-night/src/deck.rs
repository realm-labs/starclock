use crate::{
    FateCardId, FateCardOwner, FateCatalog, FateDeckId, FateDeckRecommendationId,
    FateDeckRecommendationKind,
};

const MAX_CONFIGURABLE_CARDS: usize = 64;

/// One deterministic configurable card loadout selected before a Fate battle.
///
/// Rin's fixed battle surface is not inferred here: the released recommendation
/// rows publish only Servant-owned and Neutral card lists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FateCardLoadout {
    deck: FateDeckId,
    recommendation: FateDeckRecommendationKind,
    cards: Box<[FateCardId]>,
}

impl FateCardLoadout {
    pub fn from_recommendation(
        catalog: &FateCatalog,
        recommendation: FateDeckRecommendationId,
    ) -> Result<Self, FateCardLoadoutError> {
        let recommendation = catalog
            .recommendations()
            .iter()
            .find(|candidate| candidate.id == recommendation)
            .ok_or(FateCardLoadoutError::UnknownRecommendation)?;
        let deck = catalog
            .decks()
            .iter()
            .find(|deck| deck.owner == recommendation.owner)
            .ok_or(FateCardLoadoutError::UnknownDeck)?;
        let cards = recommendation
            .owner_cards
            .iter()
            .chain(recommendation.neutral_cards.iter())
            .copied()
            .collect::<Vec<_>>();
        Self::new(catalog, deck.id, recommendation.kind, cards)
    }

    pub fn new(
        catalog: &FateCatalog,
        deck: FateDeckId,
        recommendation: FateDeckRecommendationKind,
        cards: Vec<FateCardId>,
    ) -> Result<Self, FateCardLoadoutError> {
        let owner = catalog
            .deck(deck)
            .map(|definition| definition.owner)
            .ok_or(FateCardLoadoutError::UnknownDeck)?;
        if cards.is_empty() || cards.len() > MAX_CONFIGURABLE_CARDS {
            return Err(FateCardLoadoutError::InvalidCardCount);
        }
        if cards
            .iter()
            .enumerate()
            .any(|(index, card)| cards[index + 1..].contains(card))
        {
            return Err(FateCardLoadoutError::DuplicateCard);
        }
        for id in &cards {
            let card = catalog.card(*id).ok_or(FateCardLoadoutError::UnknownCard)?;
            if !matches!(card.owner, FateCardOwner::Rin | FateCardOwner::Neutral)
                && card.owner != owner
            {
                return Err(FateCardLoadoutError::ForeignOwnerCard);
            }
        }
        Ok(Self {
            deck,
            recommendation,
            cards: cards.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn deck(&self) -> FateDeckId {
        self.deck
    }

    #[must_use]
    pub const fn recommendation(&self) -> FateDeckRecommendationKind {
        self.recommendation
    }

    #[must_use]
    pub fn cards(&self) -> &[FateCardId] {
        &self.cards
    }

    /// Fails closed until every selected released ability program has an exact
    /// shared-Combat lowering. Identity-only cards never become no-op actions.
    pub fn exact_ability_programs<'a>(
        &'a self,
        catalog: &'a FateCatalog,
    ) -> Result<Box<[&'a str]>, FateCardLoadoutError> {
        self.cards
            .iter()
            .map(|id| {
                let card = catalog.card(*id).ok_or(FateCardLoadoutError::UnknownCard)?;
                if !card.runtime_binding_exact {
                    return Err(FateCardLoadoutError::IdentityOnlyCard(*id));
                }
                card.ability_program
                    .as_deref()
                    .ok_or(FateCardLoadoutError::MissingAbilityProgram(*id))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FateCardLoadoutError {
    UnknownRecommendation,
    UnknownDeck,
    InvalidCardCount,
    DuplicateCard,
    UnknownCard,
    ForeignOwnerCard,
    IdentityOnlyCard(FateCardId),
    MissingAbilityProgram(FateCardId),
}

#[cfg(test)]
mod tests {
    use super::{FateCardLoadout, FateCardLoadoutError};
    use crate::{
        FateCard, FateCardId, FateCardOwner, FateCardRarity, FateCatalog, FateCatalogParts,
        FateDeck, FateDeckId, FateDeckRecommendation, FateDeckRecommendationId,
        FateDeckRecommendationKind,
    };

    #[test]
    fn identity_only_recommended_card_fails_closed() {
        let catalog = catalog(false);
        let loadout = FateCardLoadout::from_recommendation(
            &catalog,
            FateDeckRecommendationId::new(1).unwrap(),
        )
        .unwrap();
        assert_eq!(
            loadout.exact_ability_programs(&catalog),
            Err(FateCardLoadoutError::IdentityOnlyCard(
                FateCardId::new(10).unwrap()
            ))
        );
    }

    #[test]
    fn exact_recommended_cards_expose_stable_program_order() {
        let catalog = catalog(true);
        let loadout = FateCardLoadout::from_recommendation(
            &catalog,
            FateDeckRecommendationId::new(1).unwrap(),
        )
        .unwrap();
        assert_eq!(
            loadout.exact_ability_programs(&catalog).unwrap().as_ref(),
            ["owner-program", "neutral-program"]
        );
    }

    fn catalog(exact: bool) -> FateCatalog {
        FateCatalog::new(FateCatalogParts {
            owners: vec![FateCardOwner::Trailblazer, FateCardOwner::Neutral],
            cards: vec![
                card(10, FateCardOwner::Trailblazer, "owner-program", exact),
                card(20, FateCardOwner::Neutral, "neutral-program", exact),
            ],
            decks: vec![FateDeck {
                id: FateDeckId::new(1).unwrap(),
                stable_key: "deck".into(),
                owner: FateCardOwner::Trailblazer,
                presentation_locator: 1,
                action_locator: 2,
            }],
            recommendations: vec![FateDeckRecommendation {
                id: FateDeckRecommendationId::new(1).unwrap(),
                owner: FateCardOwner::Trailblazer,
                kind: FateDeckRecommendationKind::Base,
                owner_cards: vec![FateCardId::new(10).unwrap()].into_boxed_slice(),
                neutral_cards: vec![FateCardId::new(20).unwrap()].into_boxed_slice(),
            }],
            ..FateCatalogParts::default()
        })
        .unwrap()
    }

    fn card(id: u32, owner: FateCardOwner, program: &str, exact: bool) -> FateCard {
        FateCard {
            id: FateCardId::new(id).unwrap(),
            stable_key: format!("card-{id}").into_boxed_str(),
            owner,
            magical_energy_cost: 1,
            rarity: FateCardRarity::R,
            ability_program: Some(program.into()),
            runtime_binding_exact: exact,
        }
    }
}
