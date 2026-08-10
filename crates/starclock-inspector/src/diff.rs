use starclock_combat::{
    EffectInstanceId, ModifierInstanceId, RuleInstanceId, ShieldInstanceId, TeamSide,
    TimelineActorId, UnitId,
};

use crate::BattleSnapshot;

/// Typed runtime identity reported by a snapshot comparison.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeEntityRef {
    Unit(UnitId),
    TimelineActor(TimelineActorId),
    Shield(ShieldInstanceId),
    Effect(EffectInstanceId),
    BreakEffect(EffectInstanceId),
    Rule(RuleInstanceId),
    Modifier(ModifierInstanceId),
    Team(TeamSide),
}

/// Non-entity canonical section changed between two snapshots.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SnapshotSection {
    Identity,
    Lifecycle,
    Decision,
    Encounter,
    Clock,
    Formations,
    Links,
    Timeline,
    Randomness,
    Allocators,
    Revision,
}

/// Compact identity-level difference between stable battle boundaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleSnapshotDiff {
    pub before_revision: u64,
    pub after_revision: u64,
    pub before_hash: starclock_combat::BattleStateHash,
    pub after_hash: starclock_combat::BattleStateHash,
    pub added: Box<[RuntimeEntityRef]>,
    pub removed: Box<[RuntimeEntityRef]>,
    pub changed: Box<[RuntimeEntityRef]>,
    pub changed_sections: Box<[SnapshotSection]>,
}

impl BattleSnapshotDiff {
    #[must_use]
    pub fn between(before: &BattleSnapshot, after: &BattleSnapshot) -> Self {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut changed = Vec::new();

        diff_entities(
            &before.units,
            &after.units,
            |value| value.id,
            RuntimeEntityRef::Unit,
            &mut added,
            &mut removed,
            &mut changed,
        );
        diff_entities(
            &before.timeline_actors,
            &after.timeline_actors,
            |value| value.id,
            RuntimeEntityRef::TimelineActor,
            &mut added,
            &mut removed,
            &mut changed,
        );
        diff_entities(
            &before.shields,
            &after.shields,
            |value| value.id,
            RuntimeEntityRef::Shield,
            &mut added,
            &mut removed,
            &mut changed,
        );
        diff_entities(
            &before.effects,
            &after.effects,
            |value| value.id,
            RuntimeEntityRef::Effect,
            &mut added,
            &mut removed,
            &mut changed,
        );
        diff_entities(
            &before.break_effects,
            &after.break_effects,
            |value| value.id,
            RuntimeEntityRef::BreakEffect,
            &mut added,
            &mut removed,
            &mut changed,
        );
        diff_entities(
            &before.rules,
            &after.rules,
            |value| value.id,
            RuntimeEntityRef::Rule,
            &mut added,
            &mut removed,
            &mut changed,
        );
        diff_entities(
            &before.modifiers,
            &after.modifiers,
            |value| value.id,
            RuntimeEntityRef::Modifier,
            &mut added,
            &mut removed,
            &mut changed,
        );

        for side in [TeamSide::Player, TeamSide::Enemy] {
            let index = match side {
                TeamSide::Player => 0,
                TeamSide::Enemy => 1,
            };
            if before.teams[index] != after.teams[index] {
                changed.push(RuntimeEntityRef::Team(side));
            }
        }

        added.sort_unstable();
        removed.sort_unstable();
        changed.sort_unstable();
        let mut sections = Vec::new();
        if before.identity != after.identity {
            sections.push(SnapshotSection::Identity);
        }
        if before.phase != after.phase
            || before.fault != after.fault
            || before.concede_policy != after.concede_policy
        {
            sections.push(SnapshotSection::Lifecycle);
        }
        if before.decision != after.decision {
            sections.push(SnapshotSection::Decision);
        }
        if before.encounter != after.encounter {
            sections.push(SnapshotSection::Encounter);
        }
        if before.clock != after.clock {
            sections.push(SnapshotSection::Clock);
        }
        if before.formations != after.formations {
            sections.push(SnapshotSection::Formations);
        }
        if before.links != after.links {
            sections.push(SnapshotSection::Links);
        }
        if before.active_turn != after.active_turn
            || before.action_boundary != after.action_boundary
            || before.prepared_action != after.prepared_action
            || before.action_frame != after.action_frame
            || before.pending_extra_turns != after.pending_extra_turns
            || before.pending_reactions != after.pending_reactions
        {
            sections.push(SnapshotSection::Timeline);
        }
        if before.rng_draw_count != after.rng_draw_count {
            sections.push(SnapshotSection::Randomness);
        }
        if before.sequence_cursors != after.sequence_cursors {
            sections.push(SnapshotSection::Allocators);
        }
        if before.committed_revision != after.committed_revision {
            sections.push(SnapshotSection::Revision);
        }

        Self {
            before_revision: before.committed_revision,
            after_revision: after.committed_revision,
            before_hash: before.state_hash,
            after_hash: after.state_hash,
            added: added.into_boxed_slice(),
            removed: removed.into_boxed_slice(),
            changed: changed.into_boxed_slice(),
            changed_sections: sections.into_boxed_slice(),
        }
    }
}

fn diff_entities<T: Eq, I: Copy + Ord>(
    before: &[T],
    after: &[T],
    id: impl Fn(&T) -> I,
    reference: impl Fn(I) -> RuntimeEntityRef,
    added: &mut Vec<RuntimeEntityRef>,
    removed: &mut Vec<RuntimeEntityRef>,
    changed: &mut Vec<RuntimeEntityRef>,
) {
    let mut left = 0;
    let mut right = 0;
    while left < before.len() && right < after.len() {
        let left_id = id(&before[left]);
        let right_id = id(&after[right]);
        match left_id.cmp(&right_id) {
            core::cmp::Ordering::Less => {
                removed.push(reference(left_id));
                left += 1;
            }
            core::cmp::Ordering::Greater => {
                added.push(reference(right_id));
                right += 1;
            }
            core::cmp::Ordering::Equal => {
                if before[left] != after[right] {
                    changed.push(reference(left_id));
                }
                left += 1;
                right += 1;
            }
        }
    }
    removed.extend(before[left..].iter().map(|value| reference(id(value))));
    added.extend(after[right..].iter().map(|value| reference(id(value))));
}
