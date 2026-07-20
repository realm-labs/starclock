//! Canonically ordered authoritative generic effect instances.

use std::collections::BTreeMap;

use crate::{EffectDefinitionId, EffectInstanceId, OperationId, SourceDefinitionId, UnitId};

use super::model::{
    ControlledAction, DispelCategory, DotDefinition, DurationClock, EffectCategory,
    EffectRuntimeDefinition, EffectSnapshotPolicy, EffectStackPolicy, EffectTeardownPolicy,
    EffectTickPhase,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EffectState {
    pub(crate) id: EffectInstanceId,
    pub(crate) definition: EffectDefinitionId,
    pub(crate) source_definition: SourceDefinitionId,
    pub(crate) source_operation: OperationId,
    pub(crate) applier: UnitId,
    pub(crate) target: UnitId,
    pub(crate) category: EffectCategory,
    pub(crate) dispel: DispelCategory,
    pub(crate) stacks: u16,
    pub(crate) stack_limit: u16,
    pub(crate) remaining: Option<u16>,
    pub(crate) duration_clock: DurationClock,
    pub(crate) tick_phase: EffectTickPhase,
    pub(crate) stack_policy: EffectStackPolicy,
    pub(crate) snapshot_policy: EffectSnapshotPolicy,
    pub(crate) teardown_policy: EffectTeardownPolicy,
    pub(crate) application_priority: i32,
    pub(crate) magnitude: crate::Scalar,
    pub(crate) tags: Box<[SourceDefinitionId]>,
    pub(crate) controlled_actions: Box<[ControlledAction]>,
    pub(crate) dot: Option<DotDefinition>,
    pub(crate) application_sequence: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EffectApplicationContext {
    pub(crate) source_definition: SourceDefinitionId,
    pub(crate) source_operation: OperationId,
    pub(crate) applier: UnitId,
    pub(crate) target: UnitId,
    pub(crate) stacks: u16,
}

impl EffectState {
    pub(crate) fn from_definition(
        id: EffectInstanceId,
        definition: EffectDefinitionId,
        runtime: &EffectRuntimeDefinition,
        context: EffectApplicationContext,
    ) -> Self {
        Self {
            id,
            definition,
            source_definition: context.source_definition,
            source_operation: context.source_operation,
            applier: context.applier,
            target: context.target,
            category: runtime.category(),
            dispel: runtime.dispel(),
            stacks: context.stacks.min(runtime.stack_limit()),
            stack_limit: runtime.stack_limit(),
            remaining: runtime.duration(),
            duration_clock: runtime.duration_clock(),
            tick_phase: runtime.tick_phase(),
            stack_policy: runtime.stack_policy(),
            snapshot_policy: runtime.snapshot_policy(),
            teardown_policy: runtime.teardown_policy(),
            application_priority: runtime.application_priority(),
            magnitude: runtime.magnitude(),
            tags: runtime.tags().into(),
            controlled_actions: runtime.controlled_actions().into(),
            dot: runtime.dot(),
            application_sequence: id.get(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EffectApplyResult {
    Inserted {
        effect: EffectInstanceId,
        removed: Box<[EffectInstanceId]>,
    },
    Refreshed {
        effect: EffectInstanceId,
        stacks_before: u16,
        stacks_after: u16,
    },
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EffectStore {
    entries: BTreeMap<EffectInstanceId, EffectState>,
}

impl EffectStore {
    pub(crate) fn apply(&mut self, candidate: EffectState) -> EffectApplyResult {
        let matching = self
            .entries
            .values()
            .filter(|entry| {
                entry.definition == candidate.definition && entry.target == candidate.target
            })
            .map(|entry| entry.id)
            .collect::<Vec<_>>();
        let source_matching = matching.iter().copied().find(|id| {
            self.entries
                .get(id)
                .is_some_and(|entry| entry.source_definition == candidate.source_definition)
        });
        match candidate.stack_policy {
            EffectStackPolicy::Refresh | EffectStackPolicy::RefreshAndAddStacks
                if !matching.is_empty() =>
            {
                let id = matching[0];
                let entry = self.entries.get_mut(&id).expect("matching ID exists");
                let before = entry.stacks;
                if candidate.stack_policy == EffectStackPolicy::RefreshAndAddStacks {
                    entry.stacks = entry
                        .stacks
                        .saturating_add(candidate.stacks)
                        .min(entry.stack_limit);
                }
                entry.remaining = candidate.remaining;
                EffectApplyResult::Refreshed {
                    effect: id,
                    stacks_before: before,
                    stacks_after: entry.stacks,
                }
            }
            EffectStackPolicy::IndependentBySource if source_matching.is_some() => {
                let id = source_matching.expect("checked");
                let entry = self.entries.get_mut(&id).expect("matching ID exists");
                let before = entry.stacks;
                entry.stacks = candidate.stacks;
                entry.remaining = candidate.remaining;
                entry.magnitude = candidate.magnitude;
                EffectApplyResult::Refreshed {
                    effect: id,
                    stacks_before: before,
                    stacks_after: entry.stacks,
                }
            }
            policy => {
                let remove = match policy {
                    EffectStackPolicy::Replace => matching,
                    EffectStackPolicy::UniqueGlobal => self
                        .entries
                        .values()
                        .filter(|entry| entry.definition == candidate.definition)
                        .map(|entry| entry.id)
                        .collect(),
                    EffectStackPolicy::UniquePerSource => self
                        .entries
                        .values()
                        .filter(|entry| {
                            entry.definition == candidate.definition
                                && entry.source_definition == candidate.source_definition
                        })
                        .map(|entry| entry.id)
                        .collect(),
                    _ => Vec::new(),
                };
                for id in &remove {
                    self.entries.remove(id);
                }
                let id = candidate.id;
                self.entries.insert(id, candidate);
                EffectApplyResult::Inserted {
                    effect: id,
                    removed: remove.into_boxed_slice(),
                }
            }
        }
    }

    pub(crate) fn remove(&mut self, id: EffectInstanceId) -> Option<EffectState> {
        self.entries.remove(&id)
    }

    pub(crate) fn get(&self, id: EffectInstanceId) -> Option<&EffectState> {
        self.entries.get(&id)
    }
    pub(crate) fn get_mut(&mut self, id: EffectInstanceId) -> Option<&mut EffectState> {
        self.entries.get_mut(&id)
    }
    pub(crate) fn iter_by_id(&self) -> impl Iterator<Item = &EffectState> {
        self.entries.values()
    }
    pub(crate) fn canonical_entries(&self) -> impl ExactSizeIterator<Item = &EffectState> {
        self.entries.values()
    }

    pub(crate) fn removable_for(
        &self,
        target: UnitId,
        category: DispelCategory,
        required_tag: Option<SourceDefinitionId>,
    ) -> Vec<EffectInstanceId> {
        self.entries
            .values()
            .filter(|entry| {
                entry.target == target
                    && entry.dispel == category
                    && required_tag.is_none_or(|tag| entry.tags.binary_search(&tag).is_ok())
            })
            .map(|entry| entry.id)
            .collect()
    }

    pub(crate) fn dots_for(
        &self,
        target: UnitId,
        required_tag: Option<SourceDefinitionId>,
    ) -> Vec<EffectState> {
        self.entries
            .values()
            .filter(|entry| {
                entry.target == target
                    && entry.category == EffectCategory::Dot
                    && entry.dot.is_some()
                    && required_tag.is_none_or(|tag| {
                        entry.tags.binary_search(&tag).is_ok()
                            || entry
                                .dot
                                .is_some_and(|dot| dot.detonation_tag() == Some(tag))
                    })
            })
            .cloned()
            .collect()
    }

    pub(crate) fn active_strongest(
        &self,
        definition: EffectDefinitionId,
        target: UnitId,
    ) -> Option<EffectInstanceId> {
        self.entries
            .values()
            .filter(|entry| {
                entry.definition == definition
                    && entry.target == target
                    && entry.stack_policy == EffectStackPolicy::StrongestWins
            })
            .max_by_key(|entry| {
                (
                    entry.magnitude,
                    entry.application_priority,
                    entry.source_definition,
                    entry.id,
                )
            })
            .map(|entry| entry.id)
    }

    pub(crate) fn blocks(&self, owner: UnitId, action: ControlledAction) -> bool {
        self.entries.values().any(|entry| {
            entry.target == owner
                && entry.category == EffectCategory::Control
                && entry.controlled_actions.binary_search(&action).is_ok()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn effect(raw: u64) -> EffectInstanceId {
        EffectInstanceId::new(raw).unwrap()
    }

    fn unit(raw: u64) -> UnitId {
        UnitId::new(raw).unwrap()
    }

    fn source_id(raw: u64) -> SourceDefinitionId {
        SourceDefinitionId::new(u32::try_from(raw).unwrap()).unwrap()
    }

    fn candidate(
        instance: u64,
        policy: EffectStackPolicy,
        source: u64,
        target: u64,
        stacks: u16,
        magnitude: i64,
    ) -> EffectState {
        let category = if policy == EffectStackPolicy::IndependentInstances {
            EffectCategory::Control
        } else {
            EffectCategory::Buff
        };
        let mut runtime = EffectRuntimeDefinition::new(
            category,
            if category == EffectCategory::Control {
                DispelCategory::CleanseableControl
            } else {
                DispelCategory::DispellableBuff
            },
            5,
            Some(2),
            DurationClock::TargetTurnEnd,
            EffectTickPhase::None,
            policy,
        )
        .unwrap()
        .with_comparison(crate::Scalar::from_scaled(magnitude), 0);
        if category == EffectCategory::Control {
            runtime = runtime
                .with_control(vec![ControlledAction::NormalAction])
                .unwrap();
        }
        EffectState::from_definition(
            effect(instance),
            EffectDefinitionId::new(1).unwrap(),
            &runtime,
            EffectApplicationContext {
                source_definition: source_id(source),
                source_operation: OperationId::new(instance).unwrap(),
                applier: unit(source),
                target: unit(target),
                stacks,
            },
        )
    }

    fn dot_candidate(instance: u64, source: u64, target: u64, tag: u32) -> EffectState {
        let formula = crate::catalog::action::OrdinaryDamageDefinition::new(
            crate::Scalar::checked_from_integer(10).unwrap(),
            crate::catalog::action::OrdinaryDamageMultipliers::new([crate::Ratio::ONE; 9]).unwrap(),
        )
        .unwrap();
        let runtime = EffectRuntimeDefinition::new(
            EffectCategory::Dot,
            DispelCategory::DispellableDebuff,
            1,
            Some(2),
            DurationClock::TargetTurnStart,
            EffectTickPhase::TurnStart,
            EffectStackPolicy::IndependentInstances,
        )
        .unwrap()
        .with_tags(vec![SourceDefinitionId::new(tag).unwrap()])
        .unwrap()
        .with_dot(DotDefinition::new(
            formula,
            crate::formula::model::CombatElement::Lightning,
            Some(SourceDefinitionId::new(tag).unwrap()),
        ))
        .unwrap();
        EffectState::from_definition(
            effect(instance),
            EffectDefinitionId::new(2).unwrap(),
            &runtime,
            EffectApplicationContext {
                source_definition: source_id(source),
                source_operation: OperationId::new(instance).unwrap(),
                applier: unit(source),
                target: unit(target),
                stacks: 1,
            },
        )
    }

    #[test]
    fn all_stack_policies_have_distinct_deterministic_identity_semantics() {
        let mut store = EffectStore::default();
        store.apply(candidate(1, EffectStackPolicy::Replace, 1, 10, 1, 1));
        let result = store.apply(candidate(2, EffectStackPolicy::Replace, 2, 10, 1, 2));
        assert!(
            matches!(result, EffectApplyResult::Inserted { effect: actual, ref removed } if actual == effect(2) && removed.as_ref() == [effect(1)])
        );

        let mut store = EffectStore::default();
        store.apply(candidate(1, EffectStackPolicy::Refresh, 1, 10, 1, 1));
        assert!(
            matches!(store.apply(candidate(2, EffectStackPolicy::Refresh, 2, 10, 3, 2)), EffectApplyResult::Refreshed { effect: actual, stacks_before: 1, stacks_after: 1 } if actual == effect(1))
        );

        let mut store = EffectStore::default();
        store.apply(candidate(
            1,
            EffectStackPolicy::RefreshAndAddStacks,
            1,
            10,
            2,
            1,
        ));
        assert!(matches!(
            store.apply(candidate(
                2,
                EffectStackPolicy::RefreshAndAddStacks,
                2,
                10,
                3,
                2
            )),
            EffectApplyResult::Refreshed {
                stacks_before: 2,
                stacks_after: 5,
                ..
            }
        ));

        let mut store = EffectStore::default();
        store.apply(candidate(1, EffectStackPolicy::StrongestWins, 1, 10, 1, 2));
        store.apply(candidate(2, EffectStackPolicy::StrongestWins, 2, 10, 1, 3));
        assert_eq!(
            store.active_strongest(EffectDefinitionId::new(1).unwrap(), unit(10)),
            Some(effect(2))
        );

        let mut store = EffectStore::default();
        store.apply(candidate(
            1,
            EffectStackPolicy::IndependentBySource,
            1,
            10,
            1,
            1,
        ));
        assert!(
            matches!(store.apply(candidate(2, EffectStackPolicy::IndependentBySource, 1, 10, 2, 2)), EffectApplyResult::Refreshed { effect: actual, .. } if actual == effect(1))
        );
        store.apply(candidate(
            3,
            EffectStackPolicy::IndependentBySource,
            2,
            10,
            1,
            1,
        ));
        assert_eq!(store.entries.len(), 2);

        let mut store = EffectStore::default();
        store.apply(candidate(
            1,
            EffectStackPolicy::IndependentInstances,
            1,
            10,
            1,
            1,
        ));
        store.apply(candidate(
            2,
            EffectStackPolicy::IndependentInstances,
            1,
            10,
            1,
            1,
        ));
        assert_eq!(store.entries.len(), 2);

        let mut store = EffectStore::default();
        store.apply(candidate(1, EffectStackPolicy::UniqueGlobal, 1, 10, 1, 1));
        store.apply(candidate(2, EffectStackPolicy::UniqueGlobal, 2, 11, 1, 1));
        assert_eq!(
            store.iter_by_id().map(|entry| entry.id).collect::<Vec<_>>(),
            [effect(2)]
        );

        let mut store = EffectStore::default();
        store.apply(candidate(
            1,
            EffectStackPolicy::UniquePerSource,
            1,
            10,
            1,
            1,
        ));
        store.apply(candidate(
            2,
            EffectStackPolicy::UniquePerSource,
            1,
            11,
            1,
            1,
        ));
        store.apply(candidate(
            3,
            EffectStackPolicy::UniquePerSource,
            2,
            10,
            1,
            1,
        ));
        assert_eq!(
            store.iter_by_id().map(|entry| entry.id).collect::<Vec<_>>(),
            [effect(2), effect(3)]
        );
    }

    #[test]
    fn cleanse_queries_and_control_are_category_scoped() {
        let mut store = EffectStore::default();
        store.apply(candidate(
            1,
            EffectStackPolicy::IndependentInstances,
            1,
            10,
            1,
            1,
        ));
        store.apply(candidate(2, EffectStackPolicy::Refresh, 1, 11, 1, 1));
        assert!(store.blocks(unit(10), ControlledAction::NormalAction));
        assert!(!store.blocks(unit(10), ControlledAction::Ultimate));
        assert_eq!(
            store.removable_for(unit(10), DispelCategory::CleanseableControl, None),
            [effect(1)]
        );
        assert_eq!(
            store.removable_for(unit(11), DispelCategory::DispellableBuff, None),
            [effect(2)]
        );
        assert!(
            store
                .removable_for(unit(10), DispelCategory::NonDispellable, None)
                .is_empty()
        );
    }

    #[test]
    fn dot_selection_is_target_local_tagged_and_keeps_original_attribution() {
        let mut store = EffectStore::default();
        store.apply(dot_candidate(1, 1, 10, 31));
        store.apply(dot_candidate(2, 2, 10, 32));
        store.apply(dot_candidate(3, 3, 11, 31));
        let selected = store.dots_for(unit(10), Some(SourceDefinitionId::new(31).unwrap()));
        assert_eq!(selected.len(), 1);
        assert_eq!(
            (
                selected[0].id,
                selected[0].applier,
                selected[0].source_definition
            ),
            (effect(1), unit(1), source_id(1))
        );
        assert_eq!(store.dots_for(unit(10), None).len(), 2);
        assert_eq!(store.dots_for(unit(11), None).len(), 1);
    }
}
