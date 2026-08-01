//! Closed Pathstrider cabinet graph and denominator validation.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::error::UniverseCatalogLoadError;

use super::{CabinetKind, RuntimeCabinet, invalid};

pub(super) fn validate_cabinets(
    cabinets: &[RuntimeCabinet],
) -> Result<(), UniverseCatalogLoadError> {
    let by_id = cabinets
        .iter()
        .map(|cabinet| (cabinet.id, cabinet))
        .collect::<BTreeMap<_, _>>();
    if cabinets.len() != 31
        || by_id.len() != 31
        || cabinets
            .iter()
            .filter(|cabinet| cabinet.kind == CabinetKind::Normal)
            .count()
            != 24
        || cabinets
            .iter()
            .filter(|cabinet| cabinet.kind == CabinetKind::Hidden)
            .count()
            != 7
        || cabinets
            .iter()
            .map(|cabinet| cabinet.adjustments.len())
            .sum::<usize>()
            != 55
        || cabinets
            .iter()
            .map(|cabinet| cabinet.prerequisites.len())
            .sum::<usize>()
            != 33
        || cabinets
            .iter()
            .map(|cabinet| cabinet.unlocks.len())
            .sum::<usize>()
            != 33
        || cabinets
            .iter()
            .map(|cabinet| cabinet.description_parameters.len())
            .sum::<usize>()
            != 34
        || cabinets
            .iter()
            .map(|cabinet| cabinet.source_id)
            .collect::<BTreeSet<_>>()
            .len()
            != 31
        || cabinets
            .iter()
            .map(|cabinet| cabinet.sort)
            .collect::<BTreeSet<_>>()
            .len()
            != 31
        || cabinets
            .iter()
            .map(|cabinet| cabinet.objective_id.as_ref())
            .collect::<BTreeSet<_>>()
            .len()
            != 31
    {
        return Err(invalid("Swarm cabinet denominator drift"));
    }
    for cabinet in cabinets {
        for prerequisite in &cabinet.prerequisites {
            if by_id
                .get(prerequisite)
                .is_none_or(|required| !required.unlocks.contains(&cabinet.id))
            {
                return Err(invalid("Swarm cabinet inverted-edge drift"));
            }
        }
        for unlocked in &cabinet.unlocks {
            if by_id
                .get(unlocked)
                .is_none_or(|target| !target.prerequisites.contains(&cabinet.id))
            {
                return Err(invalid("Swarm cabinet outgoing-edge drift"));
            }
        }
        if cabinet
            .adjustments
            .iter()
            .map(|adjustment| adjustment.dimension_id)
            .collect::<BTreeSet<_>>()
            .len()
            != cabinet.adjustments.len()
        {
            return Err(invalid("duplicate dimension in Swarm cabinet reward"));
        }
    }
    validate_normal_reachability(cabinets, &by_id)
}

fn validate_normal_reachability(
    cabinets: &[RuntimeCabinet],
    by_id: &BTreeMap<u32, &RuntimeCabinet>,
) -> Result<(), UniverseCatalogLoadError> {
    let roots = cabinets
        .iter()
        .filter(|cabinet| cabinet.kind == CabinetKind::Normal && cabinet.prerequisites.is_empty())
        .map(|cabinet| cabinet.id)
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(invalid("Swarm normal cabinet root drift"));
    }
    let mut reached = BTreeSet::new();
    let mut queue = VecDeque::from(roots);
    while let Some(id) = queue.pop_front() {
        if !reached.insert(id) {
            continue;
        }
        let cabinet = by_id
            .get(&id)
            .ok_or_else(|| invalid("unknown Swarm cabinet graph node"))?;
        queue.extend(cabinet.unlocks.iter().copied());
    }
    if cabinets
        .iter()
        .filter(|cabinet| cabinet.kind == CabinetKind::Normal)
        .any(|cabinet| !reached.contains(&cabinet.id))
    {
        return Err(invalid("unreachable Swarm normal cabinet"));
    }
    Ok(())
}
