use crate::{
    UnitId,
    actor::{
        model::LifeState,
        store::{FormationState, UnitStore},
    },
    battle::spec::TeamSide,
    catalog::action::{
        TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
    },
};

use super::model::TargetCommitment;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TargetError {
    MissingActor,
    InvalidPrimary,
    EmptyPool,
    Invalidated,
    ChoiceFailed,
}

pub(crate) fn legal_primary_targets(
    units: &UnitStore,
    formations: &FormationState,
    actor: UnitId,
    selector: UnitTargetSelector,
) -> Result<Vec<Option<UnitId>>, TargetError> {
    let actor_state = units.get(actor).ok_or(TargetError::MissingActor)?;
    let pool = stable_pool(units, formations, actor_state.side, selector.relation());
    match (selector.relation(), selector.pattern()) {
        (TargetRelation::SelfUnit, _) => Ok(vec![None]),
        (_, TargetPattern::All) => {
            if pool.is_empty() {
                Err(TargetError::EmptyPool)
            } else {
                Ok(vec![None])
            }
        }
        (_, TargetPattern::Single | TargetPattern::Blast) => {
            if pool.is_empty() {
                Err(TargetError::EmptyPool)
            } else {
                Ok(pool.into_iter().map(Some).collect())
            }
        }
    }
}

pub(crate) fn commit(
    units: &UnitStore,
    formations: &FormationState,
    actor: UnitId,
    selector: UnitTargetSelector,
    invalidation: TargetInvalidationPolicy,
    primary: Option<UnitId>,
) -> Result<TargetCommitment, TargetError> {
    let actor_state = units.get(actor).ok_or(TargetError::MissingActor)?;
    let pool = stable_pool(units, formations, actor_state.side, selector.relation());
    let targets = build_pattern(units, formations, actor, selector, primary, &pool)?;
    Ok(TargetCommitment {
        selector,
        invalidation,
        primary,
        targets: targets.into_boxed_slice(),
    })
}

pub(crate) fn resolve_for_hit(
    units: &UnitStore,
    formations: &FormationState,
    actor: UnitId,
    commitment: &mut TargetCommitment,
    mut choose: impl FnMut(u32) -> Result<usize, TargetError>,
) -> Result<Box<[UnitId]>, TargetError> {
    let valid = |target| ordinarily_targetable(units, target);
    if commitment.targets.iter().copied().all(valid) {
        return Ok(commitment.targets.clone());
    }
    match commitment.invalidation {
        TargetInvalidationPolicy::CancelRemainingForTarget => {
            commitment.targets = commitment
                .targets
                .iter()
                .copied()
                .filter(|target| valid(*target))
                .collect();
        }
        TargetInvalidationPolicy::KeepIfPresent => {
            commitment.targets = commitment
                .targets
                .iter()
                .copied()
                .filter(|target| {
                    units
                        .get(*target)
                        .is_some_and(|unit| unit.presence.is_active())
                })
                .collect();
        }
        TargetInvalidationPolicy::FailAction => return Err(TargetError::Invalidated),
        TargetInvalidationPolicy::RetargetSamePool => {
            let actor_side = units.get(actor).ok_or(TargetError::MissingActor)?.side;
            let pool = stable_pool(
                units,
                formations,
                actor_side,
                commitment.selector.relation(),
            );
            if pool.is_empty() {
                return Err(TargetError::EmptyPool);
            }
            for index in 0..commitment.targets.len() {
                if !valid(commitment.targets[index]) {
                    let candidates = pool
                        .iter()
                        .copied()
                        .filter(|candidate| {
                            commitment.selector.repeated_targets()
                                || !commitment.targets.contains(candidate)
                        })
                        .collect::<Vec<_>>();
                    if candidates.is_empty() {
                        return Err(TargetError::EmptyPool);
                    }
                    commitment.targets[index] = candidates[choose(
                        u32::try_from(candidates.len()).map_err(|_| TargetError::ChoiceFailed)?,
                    )?];
                }
            }
        }
        TargetInvalidationPolicy::RetargetPrimaryThenRebuildPattern => {
            let actor_side = units.get(actor).ok_or(TargetError::MissingActor)?.side;
            let pool = stable_pool(
                units,
                formations,
                actor_side,
                commitment.selector.relation(),
            );
            if pool.is_empty() {
                return Err(TargetError::EmptyPool);
            }
            let primary = match commitment.primary {
                Some(primary) if valid(primary) => primary,
                _ => {
                    pool[choose(u32::try_from(pool.len()).map_err(|_| TargetError::ChoiceFailed)?)?]
                }
            };
            commitment.primary = Some(primary);
            commitment.targets = build_pattern(
                units,
                formations,
                actor,
                commitment.selector,
                Some(primary),
                &pool,
            )?
            .into_boxed_slice();
        }
    }
    Ok(commitment.targets.clone())
}

fn build_pattern(
    units: &UnitStore,
    formations: &FormationState,
    actor: UnitId,
    selector: UnitTargetSelector,
    primary: Option<UnitId>,
    pool: &[UnitId],
) -> Result<Vec<UnitId>, TargetError> {
    if selector.relation() == TargetRelation::SelfUnit {
        return (primary.is_none())
            .then_some(vec![actor])
            .ok_or(TargetError::InvalidPrimary);
    }
    match selector.pattern() {
        TargetPattern::All => (primary.is_none() && !pool.is_empty())
            .then(|| pool.to_vec())
            .ok_or(TargetError::InvalidPrimary),
        TargetPattern::Single => {
            let target = primary
                .filter(|target| pool.contains(target))
                .ok_or(TargetError::InvalidPrimary)?;
            Ok(vec![target])
        }
        TargetPattern::Blast => {
            let target = primary
                .filter(|target| pool.contains(target))
                .ok_or(TargetError::InvalidPrimary)?;
            let primary_state = units.get(target).ok_or(TargetError::InvalidPrimary)?;
            let index = primary_state.formation.get();
            Ok(formations
                .on_side(primary_state.side)
                .filter(|entry| entry.index.get().abs_diff(index) <= 1)
                .map(|entry| entry.unit)
                .filter(|unit| pool.contains(unit))
                .collect())
        }
    }
}

pub(crate) fn stable_pool(
    units: &UnitStore,
    formations: &FormationState,
    actor_side: TeamSide,
    relation: TargetRelation,
) -> Vec<UnitId> {
    if relation == TargetRelation::SelfUnit {
        return Vec::new();
    }
    let side = match relation {
        TargetRelation::SelfUnit => actor_side,
        TargetRelation::Allied => actor_side,
        TargetRelation::Opposing => opposite(actor_side),
    };
    formations
        .on_side(side)
        .map(|entry| entry.unit)
        .filter(|unit| ordinarily_targetable(units, *unit))
        .collect()
}

fn ordinarily_targetable(units: &UnitStore, target: UnitId) -> bool {
    units
        .get(target)
        .is_some_and(|unit| unit.life == LifeState::Alive && unit.presence.is_targetable())
}

const fn opposite(side: TeamSide) -> TeamSide {
    match side {
        TeamSide::Player => TeamSide::Enemy,
        TeamSide::Enemy => TeamSide::Player,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Energy, Hp, PresenceState,
        actor::store::{FormationEntry, UnitState},
        battle::spec::{CombatantSpecDigest, FormationIndex, ParticipantSource, UnitLevel},
        id::{SpawnSequence, UnitDefinitionId},
    };

    fn runtime<I: TryFrom<u64>>(raw: u64) -> I
    where
        I::Error: core::fmt::Debug,
    {
        I::try_from(raw).unwrap()
    }

    fn definition<I: TryFrom<u32>>(raw: u32) -> I
    where
        I::Error: core::fmt::Debug,
    {
        I::try_from(raw).unwrap()
    }

    fn state(id: u64, side: TeamSide, formation: u8) -> UnitState {
        UnitState {
            id: runtime(id),
            spawn: runtime::<SpawnSequence>(id),
            form: definition::<UnitDefinitionId>(1),
            source: ParticipantSource::Player,
            side,
            formation: FormationIndex::new(formation).unwrap(),
            entry_wave: 1,
            level: UnitLevel::new(80).unwrap(),
            life: LifeState::Alive,
            presence: PresenceState::Present,
            current_hp: Hp::new(100).unwrap(),
            maximum_hp: Hp::new(100).unwrap(),
            base_attack: crate::StatValue::from_scaled(0).unwrap(),
            base_defense: crate::StatValue::from_scaled(0).unwrap(),
            base_speed: crate::Speed::from_scaled(100_000_000).unwrap(),
            current_energy: Energy::ZERO,
            maximum_energy: Energy::ZERO,
            rank: crate::formula::toughness::EnemyRank::Normal,
            weaknesses: Vec::new(),
            permanent_weaknesses: Box::new([]),
            temporary_weaknesses: Vec::new(),
            toughness_layers: Vec::new(),
            weakness_broken: false,
            abilities: Box::new([]),
            rule_bundles: Box::new([]),
            modifiers: Box::new([]),
            digest: CombatantSpecDigest::new([u8::try_from(id).unwrap(); 32]).unwrap(),
            transformation: None,
            enemy: None,
        }
    }

    fn stores() -> (UnitStore, FormationState) {
        let mut units = UnitStore::default();
        let mut formations = FormationState::default();
        for (id, side, index) in [
            (1, TeamSide::Player, 0),
            (2, TeamSide::Enemy, 3),
            (3, TeamSide::Enemy, 4),
            (4, TeamSide::Enemy, 5),
        ] {
            units.insert(state(id, side, index));
            formations.push(FormationEntry {
                side,
                index: FormationIndex::new(index).unwrap(),
                unit: runtime(id),
            });
        }
        (units, formations)
    }

    #[test]
    fn target_locks_apply_every_explicit_invalidation_policy() {
        let (mut units, formations) = stores();
        let actor = runtime(1);
        let primary = runtime(3);
        let selector =
            UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Blast).unwrap();
        let make =
            |policy| commit(&units, &formations, actor, selector, policy, Some(primary)).unwrap();
        let cancel = make(TargetInvalidationPolicy::CancelRemainingForTarget);
        let keep = make(TargetInvalidationPolicy::KeepIfPresent);
        let fail = make(TargetInvalidationPolicy::FailAction);
        let retarget = commit(
            &units,
            &formations,
            actor,
            UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
            TargetInvalidationPolicy::RetargetSamePool,
            Some(primary),
        )
        .unwrap();
        let rebuild = make(TargetInvalidationPolicy::RetargetPrimaryThenRebuildPattern);

        units.get_mut(primary).unwrap().life = LifeState::Defeated;

        let mut cancel = cancel;
        assert_eq!(
            resolve_for_hit(&units, &formations, actor, &mut cancel, |_| Ok(0))
                .unwrap()
                .as_ref(),
            [runtime(2), runtime(4)]
        );
        let mut keep = keep;
        assert_eq!(
            resolve_for_hit(&units, &formations, actor, &mut keep, |_| Ok(0))
                .unwrap()
                .as_ref(),
            [runtime(2), runtime(3), runtime(4)]
        );
        let mut fail = fail;
        assert_eq!(
            resolve_for_hit(&units, &formations, actor, &mut fail, |_| Ok(0)),
            Err(TargetError::Invalidated)
        );
        let mut retarget = retarget;
        assert_eq!(
            resolve_for_hit(&units, &formations, actor, &mut retarget, |_| Ok(0))
                .unwrap()
                .as_ref(),
            [runtime(2)]
        );
        let mut rebuild = rebuild;
        assert_eq!(
            resolve_for_hit(&units, &formations, actor, &mut rebuild, |_| Ok(1))
                .unwrap()
                .as_ref(),
            [runtime(4)]
        );
        assert_eq!(rebuild.primary, Some(runtime(4)));
    }
}
