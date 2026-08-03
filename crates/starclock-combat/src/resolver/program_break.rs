//! Event-to-operation bridge for triggered Super Break.

use crate::{RawToughness, UnitId};

use crate::operation::HitOperationScratch;
pub(super) fn seed_observed_reduction(
    scratch: &mut HitOperationScratch,
    target: Option<UnitId>,
    effective: Option<RawToughness>,
    selected: &[UnitId],
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
    use crate::RawToughness;
    use crate::UnitId;
    use crate::operation::HitOperationScratch;

    #[test]
    fn observed_reduction_seeds_only_its_matching_event_target() {
        let first = UnitId::new(1).unwrap();
        let second = UnitId::new(2).unwrap();
        let effective = RawToughness::new(30).unwrap();
        let mut scratch = HitOperationScratch::default();
        seed_observed_reduction(&mut scratch, Some(first), Some(effective), &[first, second]);
        assert_eq!(scratch.effective_reductions.get(&first), Some(&effective));
        assert_eq!(scratch.effective_reductions.get(&second), None);

        let replacement = RawToughness::new(90).unwrap();
        seed_observed_reduction(&mut scratch, Some(first), Some(replacement), &[first]);
        assert_eq!(scratch.effective_reductions.get(&first), Some(&effective));
    }
}
