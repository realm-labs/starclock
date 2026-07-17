use super::{
    derive::StreamPath,
    engine::DeterministicRng,
    types::{DrawPurpose, RngError, RngSeed},
};

fn seed() -> RngSeed {
    RngSeed::new(core::array::from_fn(|index| index as u8))
}

#[test]
fn stream_path_rejects_ambiguous_text_fields() {
    assert_eq!(
        StreamPath::new("", 1, 2, 3, 4, 5, "battle"),
        Err(RngError::InvalidStreamIdentity)
    );
    assert_eq!(
        StreamPath::new("standard-v1", 1, 2, 3, 4, 5, "battle stream"),
        Err(RngError::InvalidStreamIdentity)
    );
}

#[test]
fn no_candidate_and_invalid_weight_requests_consume_no_draw() {
    let mut rng = DeterministicRng::from_seed(seed());
    assert_eq!(
        rng.sample_below(DrawPurpose::BOUNCE_TARGET, 0),
        Err(RngError::EmptyRange)
    );
    assert_eq!(rng.choose_index(DrawPurpose::BOUNCE_TARGET, 0), Ok(None));
    assert_eq!(
        rng.choose_weighted(DrawPurpose::AGGRO_TARGET, &[]),
        Ok(None)
    );
    assert_eq!(
        rng.choose_weighted(DrawPurpose::AGGRO_TARGET, &[0, 0]),
        Ok(None)
    );
    assert_eq!(
        rng.choose_weighted(DrawPurpose::AGGRO_TARGET, &[u64::MAX, 1]),
        Err(RngError::WeightTotalOverflow)
    );
    assert_eq!(rng.draw_count(), 0);
}

#[test]
fn raw_draws_are_monotonic_and_same_seed_reproduces() {
    let mut left = DeterministicRng::from_seed(seed());
    let mut right = DeterministicRng::from_seed(seed());
    for expected_index in 0..8 {
        let left_sample = left.draw_raw(DrawPurpose::CRIT).expect("counter available");
        let right_sample = right
            .draw_raw(DrawPurpose::CRIT)
            .expect("counter available");
        assert_eq!(left_sample, right_sample);
        assert_eq!(left_sample.index(), expected_index);
        assert_eq!(left_sample.purpose(), DrawPurpose::CRIT);
    }
    assert_eq!(left.draw_count(), 8);
}

#[test]
fn weighted_selection_returns_authored_index_and_range_trace() {
    let mut rng = DeterministicRng::from_seed(seed());
    let selection = rng
        .choose_weighted(DrawPurpose::AGGRO_TARGET, &[0, 10, 20, 0])
        .expect("weights are valid")
        .expect("positive total selects");
    assert!(matches!(selection.index(), 1 | 2));
    assert_eq!(selection.range().upper(), 30);
    assert_eq!(selection.range().sample().index(), 0);
    assert_eq!(
        rng.draw_count(),
        1 + u64::from(selection.range().rejected_draws())
    );
}

#[test]
fn draw_counter_exhaustion_does_not_advance_the_generator() {
    let mut exhausted = DeterministicRng::from_seed(seed());
    exhausted.draws = u64::MAX;
    assert_eq!(
        exhausted.draw_raw(DrawPurpose::CRIT),
        Err(RngError::DrawCounterExhausted)
    );
    exhausted.draws = 0;

    let mut fresh = DeterministicRng::from_seed(seed());
    assert_eq!(
        exhausted.draw_raw(DrawPurpose::CRIT),
        fresh.draw_raw(DrawPurpose::CRIT)
    );
}
