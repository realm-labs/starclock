//! Event-to-operation bridge for triggered Super Break.

pub(super) fn seed_observed_reduction(
    scratch: &mut crate::operation::HitOperationScratch,
    target: Option<crate::UnitId>,
    effective: Option<crate::RawToughness>,
    selected: &[crate::UnitId],
) {
    if let (Some(target), Some(effective)) = (target, effective)
        && selected.contains(&target)
    {
        scratch
            .effective_reductions
            .entry(target)
            .or_insert(effective);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observed_reduction_seeds_only_its_matching_event_target() {
        let first = crate::UnitId::new(1).unwrap();
        let second = crate::UnitId::new(2).unwrap();
        let effective = crate::RawToughness::new(30).unwrap();
        let mut scratch = crate::operation::HitOperationScratch::default();
        seed_observed_reduction(&mut scratch, Some(first), Some(effective), &[first, second]);
        assert_eq!(scratch.effective_reductions.get(&first), Some(&effective));
        assert_eq!(scratch.effective_reductions.get(&second), None);

        let replacement = crate::RawToughness::new(90).unwrap();
        seed_observed_reduction(&mut scratch, Some(first), Some(replacement), &[first]);
        assert_eq!(scratch.effective_reductions.get(&first), Some(&effective));
    }
}
