use super::types::{DrawSample, RngError};

pub(super) fn sample_below(
    upper: u64,
    mut next: impl FnMut() -> Result<DrawSample, RngError>,
) -> Result<(DrawSample, u64, u32), RngError> {
    if upper == 0 {
        return Err(RngError::EmptyRange);
    }

    let rejection_threshold = upper.wrapping_neg() % upper;
    let mut rejected = 0_u32;
    loop {
        let sample = next()?;
        if sample.raw() >= rejection_threshold {
            return Ok((sample, sample.raw() % upper, rejected));
        }
        rejected = rejected
            .checked_add(1)
            .ok_or(RngError::RejectionBudgetExhausted)?;
    }
}

pub(super) fn weight_total(weights: &[u64]) -> Result<u64, RngError> {
    weights.iter().try_fold(0_u64, |total, weight| {
        total
            .checked_add(*weight)
            .ok_or(RngError::WeightTotalOverflow)
    })
}

pub(super) fn weighted_index(weights: &[u64], selected: u64) -> Option<u32> {
    let mut cumulative = 0_u64;
    for (index, weight) in weights.iter().copied().enumerate() {
        cumulative = cumulative.checked_add(weight)?;
        if selected < cumulative {
            return u32::try_from(index).ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::types::DrawPurpose;

    fn sample(index: u64, raw: u64) -> DrawSample {
        DrawSample::new(index, DrawPurpose::CRIT, raw)
    }

    #[test]
    fn rejection_mapping_discards_the_low_incomplete_interval() {
        let mut values = [sample(0, 0), sample(1, 6)].into_iter();
        let (accepted, mapped, rejected) =
            sample_below(10, || values.next().ok_or(RngError::DrawCounterExhausted))
                .expect("script contains an accepted value");

        assert_eq!(accepted.index(), 1);
        assert_eq!(mapped, 6);
        assert_eq!(rejected, 1);
    }

    #[test]
    fn weighted_mapping_preserves_authored_candidate_order() {
        let weights = [0, 2, 0, 5];
        assert_eq!(weighted_index(&weights, 0), Some(1));
        assert_eq!(weighted_index(&weights, 1), Some(1));
        assert_eq!(weighted_index(&weights, 2), Some(3));
        assert_eq!(weighted_index(&weights, 6), Some(3));
        assert_eq!(weighted_index(&weights, 7), None);
    }
}
