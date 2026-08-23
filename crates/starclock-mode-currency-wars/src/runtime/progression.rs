use super::{
    CURRENT_CHAPTER, CURRENT_SECTION, CurrencyWarsRun, CurrencyWarsRuntimeError, debug_error, error,
};
use crate::{CurrencyWarsProgressionProjection, CurrencyWarsRunPosition};

impl CurrencyWarsRun {
    /// Projects the authored season score and experience for the last entered
    /// route position. This remains queryable after terminal settlement.
    pub fn progression_projection(
        &self,
    ) -> Result<Option<CurrencyWarsProgressionProjection<'_>>, CurrencyWarsRuntimeError> {
        let position = self.current_progression_position()?;
        let difficulty = self
            .definition
            .catalog
            .difficulties()
            .binary_search_by_key(&self.definition.difficulty, |value| value.source_id)
            .ok()
            .map(|index| &self.definition.catalog.difficulties()[index])
            .ok_or_else(|| error("Currency Wars difficulty is missing"))?;
        self.definition
            .catalog
            .progression_projection(difficulty, self.definition.gambit, position)
            .map_err(debug_error)
    }

    pub(super) fn current_progression_position(
        &self,
    ) -> Result<CurrencyWarsRunPosition, CurrencyWarsRuntimeError> {
        let chapter = u8::try_from(self.integer(CURRENT_CHAPTER))
            .map_err(|_| error("Currency Wars current chapter is invalid"))?;
        let section = u8::try_from(self.integer(CURRENT_SECTION))
            .map_err(|_| error("Currency Wars current section is invalid"))?;
        CurrencyWarsRunPosition::new(chapter, section).map_err(debug_error)
    }
}
