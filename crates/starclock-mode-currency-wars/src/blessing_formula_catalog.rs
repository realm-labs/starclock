use std::collections::BTreeSet;

use crate::CurrencyWarsDecimal;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsMazeBuffEnhancement {
    pub stable_key: Box<str>,
    pub source_id: u32,
    pub parameters: Box<[CurrencyWarsDecimal]>,
    pub effect_ids: Box<[Box<str>]>,
}

/// Exact Version 4.4 closure for Blessing- and formula-shaped content.
///
/// Currency Wars has no reachable Blessing identities, formulas, recipes,
/// progress states, randomizers or formula contributions. The only retained
/// rows are seven independently referenced MazeBuff enhancements; they are
/// deliberately not promoted to Blessings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBlessingFormulaCatalog {
    maze_buff_enhancements: Box<[CurrencyWarsMazeBuffEnhancement]>,
}

impl CurrencyWarsBlessingFormulaCatalog {
    pub fn new(
        mut maze_buff_enhancements: Vec<CurrencyWarsMazeBuffEnhancement>,
    ) -> Result<Self, CurrencyWarsBlessingFormulaCatalogError> {
        maze_buff_enhancements.sort_by_key(|row| row.source_id);
        let mut stable_keys = BTreeSet::new();
        if maze_buff_enhancements.is_empty()
            || maze_buff_enhancements.iter().any(|row| {
                row.parameters.is_empty()
                    || row.effect_ids.is_empty()
                    || !stable_keys.insert(row.stable_key.as_ref())
            })
            || maze_buff_enhancements
                .windows(2)
                .any(|rows| rows[0].source_id == rows[1].source_id)
        {
            return Err(error(
                "Currency Wars MazeBuff enhancement registry is invalid",
            ));
        }
        Ok(Self {
            maze_buff_enhancements: maze_buff_enhancements.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn maze_buff_enhancements(&self) -> &[CurrencyWarsMazeBuffEnhancement] {
        &self.maze_buff_enhancements
    }

    #[must_use]
    pub fn maze_buff_enhancement(
        &self,
        source_id: u32,
    ) -> Option<&CurrencyWarsMazeBuffEnhancement> {
        self.maze_buff_enhancements
            .binary_search_by_key(&source_id, |row| row.source_id)
            .ok()
            .map(|index| &self.maze_buff_enhancements[index])
    }

    #[must_use]
    pub const fn blessing_identity_count(&self) -> usize {
        0
    }

    #[must_use]
    pub const fn formula_identity_count(&self) -> usize {
        0
    }

    #[must_use]
    pub const fn recipe_count(&self) -> usize {
        0
    }

    #[must_use]
    pub const fn progress_state_count(&self) -> usize {
        0
    }

    #[must_use]
    pub const fn randomizer_count(&self) -> usize {
        0
    }

    #[must_use]
    pub const fn formula_contribution_count(&self) -> usize {
        0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBlessingFormulaCatalogError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsBlessingFormulaCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsBlessingFormulaCatalogError {}

fn error(message: &'static str) -> CurrencyWarsBlessingFormulaCatalogError {
    CurrencyWarsBlessingFormulaCatalogError {
        message: message.into(),
    }
}
