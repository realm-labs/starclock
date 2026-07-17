use core::fmt;

use rand::{Rng, SeedableRng, rngs::ChaCha8Rng};

use super::{
    mapping,
    types::{DrawPurpose, DrawSample, RangeSelection, RngError, RngSeed, WeightedSelection},
};

/// One non-cloneable authoritative ChaCha8 stream with monotonic raw-draw count.
pub struct DeterministicRng {
    seed: RngSeed,
    pub(super) draws: u64,
    inner: ChaCha8Rng,
}

impl fmt::Debug for DeterministicRng {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DeterministicRng")
            .field("seed", &self.seed)
            .field("draws", &self.draws)
            .finish_non_exhaustive()
    }
}

impl DeterministicRng {
    /// Creates a fresh stream at draw index zero.
    #[must_use]
    pub fn from_seed(seed: RngSeed) -> Self {
        Self {
            seed,
            draws: 0,
            inner: ChaCha8Rng::from_seed(seed.bytes()),
        }
    }

    /// Returns the original stream seed for canonical state identity.
    #[must_use]
    pub const fn seed(&self) -> RngSeed {
        self.seed
    }
    /// Returns the number of raw `u64` words consumed.
    #[must_use]
    pub const fn draw_count(&self) -> u64 {
        self.draws
    }

    /// Copies authoritative stream semantics into existing private storage.
    ///
    /// This deliberately does not implement `Clone`: callers cannot fork a
    /// live stream, while the battle transaction can reuse owned scratch.
    pub(crate) fn clone_from_authoritative(&mut self, source: &Self) {
        self.seed = source.seed;
        self.draws = source.draws;
        self.inner = ChaCha8Rng::from_seed(source.seed.bytes());
        self.inner.set_word_pos(source.inner.get_word_pos());
    }

    /// Consumes and returns one raw generator word.
    pub fn draw_raw(&mut self, purpose: DrawPurpose) -> Result<DrawSample, RngError> {
        let next_count = self
            .draws
            .checked_add(1)
            .ok_or(RngError::DrawCounterExhausted)?;
        let sample = DrawSample::new(self.draws, purpose, self.inner.next_u64());
        self.draws = next_count;
        Ok(sample)
    }

    /// Uses project-owned rejection sampling for `[0, upper)`.
    pub fn sample_below(
        &mut self,
        purpose: DrawPurpose,
        upper: u64,
    ) -> Result<RangeSelection, RngError> {
        let (sample, value, rejected_draws) =
            mapping::sample_below(upper, || self.draw_raw(purpose))?;
        Ok(RangeSelection {
            sample,
            upper,
            value,
            rejected_draws,
        })
    }

    /// Selects a canonical candidate index. Zero candidates consume no draw.
    pub fn choose_index(
        &mut self,
        purpose: DrawPurpose,
        candidate_count: u32,
    ) -> Result<Option<RangeSelection>, RngError> {
        if candidate_count == 0 {
            return Ok(None);
        }
        self.sample_below(purpose, u64::from(candidate_count))
            .map(Some)
    }

    /// Selects by cumulative authored integer weights.
    ///
    /// Empty/all-zero candidates and validation failures consume no draw.
    pub fn choose_weighted(
        &mut self,
        purpose: DrawPurpose,
        weights: &[u64],
    ) -> Result<Option<WeightedSelection>, RngError> {
        u32::try_from(weights.len()).map_err(|_| RngError::TooManyCandidates)?;
        let total = mapping::weight_total(weights)?;
        if total == 0 {
            return Ok(None);
        }
        let range = self.sample_below(purpose, total)?;
        let index =
            mapping::weighted_index(weights, range.value()).ok_or(RngError::MappingInvariant)?;
        Ok(Some(WeightedSelection { range, index }))
    }
}
