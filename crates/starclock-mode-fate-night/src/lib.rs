//! Fate/Star Rail NIGHT activity definitions.
//!
//! Case Boards compile to the shared Activity graph. The 4.4 tactical-card
//! catalog remains identity-exact and fail-closed until its custom battle
//! programs lower into the shared Combat aggregate.

#![forbid(unsafe_code)]

mod board;
mod catalog;
mod deck;

pub use board::{FateBoard, FateBoardEdge, FateBoardError, FateBoardNode, FateBoardNodeKind};
pub use catalog::{
    FateCard, FateCardId, FateCardOwner, FateCardRarity, FateCatalog, FateCatalogError,
    FateCatalogParts, FateChallengeFight, FateChallengeFightId, FateDeck, FateDeckId,
    FateDeckRecommendation, FateDeckRecommendationId, FateDeckRecommendationKind, FateMapFight,
    FateMapFightId, FateRuntimePolicy, FateStoryFight, FateStoryFightId,
};
pub use deck::{FateCardLoadout, FateCardLoadoutError};
