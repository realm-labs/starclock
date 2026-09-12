//! Distinct mandatory rewards, including overlapping Path and rarity pools.

use std::collections::BTreeMap;

use starclock_activity::{
    ActivityPlayerView, ActivityRngLabel, ActivityRngStreams, GraphActivityCommandError,
};
use starclock_data::{
    divergent_universe_blessing_catalog::{
        DivergentUniverseBlessingCategory, DivergentUniverseBlessingId,
    },
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::CurioAcquisitionGrant,
};

use super::{DivergentUniverseCurioRuntime, DivergentUniverseCurioRuntimeError};

const ACQUISITION_BLESSING_PURPOSE: u16 = 23_651;

pub(super) struct BlessingPlan {
    candidates: Vec<DivergentUniverseBlessingId>,
    owners: Vec<DivergentUniverseCurioStateId>,
    requests: Vec<Vec<usize>>,
}

impl DivergentUniverseCurioRuntime {
    pub(super) fn blessing_plan(
        &self,
        view: &ActivityPlayerView,
        states: &[DivergentUniverseCurioStateId],
    ) -> Result<BlessingPlan, DivergentUniverseCurioRuntimeError> {
        let mut states = states.iter().collect::<Vec<_>>();
        states.sort_unstable();
        let effects = states
            .iter()
            .filter_map(|state| {
                self.acquisition_effects
                    .iter()
                    .find(|effect| &effect.state == *state)
            })
            .filter(|effect| {
                matches!(
                    effect.grant,
                    CurioAcquisitionGrant::PathBlessings { .. }
                        | CurioAcquisitionGrant::RarityBlessings { .. }
                )
            })
            .collect::<Vec<_>>();
        if effects.is_empty() {
            return Ok(BlessingPlan {
                candidates: Vec::new(),
                owners: Vec::new(),
                requests: Vec::new(),
            });
        }
        let owned = self
            .acquisition_blessings
            .reward_owned(view)
            .map_err(DivergentUniverseCurioRuntimeError::Blessing)?;
        let mut candidates = self
            .acquisition_blessings
            .blessings()
            .iter()
            .filter(|blessing| !owned.iter().any(|held| held.blessing() == blessing.id()))
            .collect::<Vec<_>>();
        candidates.sort_unstable_by(|left, right| left.id().cmp(right.id()));
        let mut plan = BlessingPlan {
            candidates: candidates
                .iter()
                .map(|candidate| candidate.id().clone())
                .collect(),
            owners: Vec::new(),
            requests: Vec::new(),
        };
        for effect in effects {
            let (count, pools) = match &effect.grant {
                CurioAcquisitionGrant::PathBlessings { count, paths } => (
                    *count,
                    paths
                        .iter()
                        .map(|path| {
                            candidates
                                .iter()
                                .enumerate()
                                .filter_map(|(index, blessing)| {
                                    (blessing.path() == path).then_some(index)
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>(),
                ),
                CurioAcquisitionGrant::RarityBlessings { count, rarity } => (
                    *count,
                    vec![
                        candidates
                            .iter()
                            .enumerate()
                            .filter_map(|(index, blessing)| {
                                let value = match blessing.category() {
                                    DivergentUniverseBlessingCategory::Common => 1,
                                    DivergentUniverseBlessingCategory::Rare => 2,
                                    DivergentUniverseBlessingCategory::Legendary => 3,
                                };
                                (rarity.minimum()..=rarity.maximum())
                                    .contains(&value)
                                    .then_some(index)
                            })
                            .collect::<Vec<_>>(),
                    ],
                ),
                _ => unreachable!("only mandatory Blessing grants were selected"),
            };
            for pool in pools {
                for _ in 0..count {
                    if plan.requests.len() >= candidates.len() {
                        return Err(DivergentUniverseCurioRuntimeError::NoLegalCandidate);
                    }
                    plan.owners.push(effect.state.clone());
                    plan.requests.push(pool.clone());
                }
            }
        }
        if !feasible(&plan.requests, &vec![false; plan.candidates.len()]) {
            return Err(DivergentUniverseCurioRuntimeError::NoLegalCandidate);
        }
        Ok(plan)
    }
}

impl BlessingPlan {
    pub(super) fn select(
        self,
        rng: &mut ActivityRngStreams,
    ) -> Result<
        BTreeMap<DivergentUniverseCurioStateId, Vec<DivergentUniverseBlessingId>>,
        DivergentUniverseCurioRuntimeError,
    > {
        let mut used = vec![false; self.candidates.len()];
        let mut selected = BTreeMap::<_, Vec<_>>::new();
        for (ordinal, pool) in self.requests.iter().enumerate() {
            let mut eligible = Vec::new();
            for &candidate in pool {
                if used[candidate] {
                    continue;
                }
                used[candidate] = true;
                if feasible(&self.requests[ordinal + 1..], &used) {
                    eligible.push(candidate);
                }
                used[candidate] = false;
            }
            let length = u32::try_from(eligible.len())
                .map_err(|_| DivergentUniverseCurioRuntimeError::InvalidState)?;
            let draw = rng
                .choose_index(
                    ActivityRngLabel::Reward,
                    ACQUISITION_BLESSING_PURPOSE,
                    length,
                )
                .map_err(|error| {
                    DivergentUniverseCurioRuntimeError::Activity(GraphActivityCommandError::Rng(
                        error,
                    ))
                })?
                .ok_or(DivergentUniverseCurioRuntimeError::NoLegalCandidate)?;
            let index = usize::try_from(draw.value())
                .map_err(|_| DivergentUniverseCurioRuntimeError::InvalidState)?;
            let candidate = eligible[index];
            used[candidate] = true;
            selected
                .entry(self.owners[ordinal].clone())
                .or_default()
                .push(self.candidates[candidate].clone());
        }
        Ok(selected)
    }
}

// Deterministic augmenting paths prove existence, not sampling weights. Indices
// are ephemeral lookup positions and never serialized, hashed or used as IDs.
fn feasible(requests: &[Vec<usize>], used: &[bool]) -> bool {
    if requests.len() > used.iter().filter(|used| !**used).count() {
        return false;
    }
    let mut assigned = vec![None; used.len()];
    for request in 0..requests.len() {
        if !augment(
            request,
            requests,
            used,
            &mut assigned,
            &mut vec![false; used.len()],
        ) {
            return false;
        }
    }
    true
}

fn augment(
    request: usize,
    requests: &[Vec<usize>],
    used: &[bool],
    assigned: &mut [Option<usize>],
    visited: &mut [bool],
) -> bool {
    for &candidate in &requests[request] {
        if used[candidate] || visited[candidate] {
            continue;
        }
        visited[candidate] = true;
        if assigned[candidate]
            .is_none_or(|previous| augment(previous, requests, used, assigned, visited))
        {
            assigned[candidate] = Some(request);
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::feasible;

    #[test]
    fn overlapping_demands_require_distinct_assignments_and_preserve_reserved_rewards() {
        assert!(feasible(&[vec![0, 1], vec![0]], &[false, false]));
        assert!(!feasible(&[vec![0], vec![0]], &[false, false]));
        assert!(!feasible(&[vec![0]], &[true, false]));
        assert!(feasible(&[vec![0]], &[false, true]));
        assert!(feasible(&[], &[]));
        assert!(!feasible(&[vec![]], &[]));
    }

    #[test]
    fn three_candidate_matching_agrees_with_complete_distinct_assignment_vectors() {
        for first in 0..8 {
            for second in 0..8 {
                for third in 0..8 {
                    let requests = [first, second, third].map(|mask| {
                        (0..3)
                            .filter(|candidate| mask & (1 << candidate) != 0)
                            .collect::<Vec<_>>()
                    });
                    for mask in 0..8 {
                        let used = [0, 1, 2].map(|candidate| mask & (1 << candidate) != 0);
                        let expected = requests[0].iter().any(|&a| {
                            requests[1].iter().any(|&b| {
                                requests[2].iter().any(|&c| {
                                    a != b && a != c && b != c && !used[a] && !used[b] && !used[c]
                                })
                            })
                        });
                        assert_eq!(feasible(&requests, &used), expected);
                    }
                }
            }
        }
    }
}
