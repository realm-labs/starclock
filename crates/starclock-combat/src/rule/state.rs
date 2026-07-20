//! Battle-owned rule instances and typed state-slot values.

use std::collections::BTreeMap;

use crate::{RuleId, RuleInstanceId, StateSlotDefinitionId, UnitId};

use super::model::{
    BattleRuleDefinition, RuleValue, SlotResetPoint, StateSlotDef, StateSlotUpdateKind,
};

#[derive(Clone, Debug)]
pub(crate) struct RuleInstanceState {
    pub(crate) id: RuleInstanceId,
    pub(crate) rule: RuleId,
    pub(crate) owner: Option<UnitId>,
    pub(crate) slots: Box<[(StateSlotDef, RuleValue)]>,
    pub(crate) ledger: super::evaluate::TriggerLedger,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RuleStateStore {
    entries: BTreeMap<RuleInstanceId, RuleInstanceState>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RuleStateError {
    MissingInstance,
    MissingSlot,
    TypeMismatch,
    OutOfBounds,
    Numeric,
}

impl RuleStateStore {
    pub(crate) fn insert(
        &mut self,
        id: RuleInstanceId,
        rule: RuleId,
        owner: Option<UnitId>,
        runtime: &BattleRuleDefinition,
    ) -> bool {
        let state = RuleInstanceState {
            id,
            rule,
            owner,
            slots: runtime
                .state_slots()
                .iter()
                .cloned()
                .map(|definition| {
                    let value = definition.initial().clone();
                    (definition, value)
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            ledger: super::evaluate::TriggerLedger::default(),
        };
        self.entries.insert(id, state).is_none()
    }

    pub(crate) fn iter_by_id(&self) -> impl ExactSizeIterator<Item = &RuleInstanceState> {
        self.entries.values()
    }
    pub(crate) fn instance_for(&self, owner: UnitId, rule: RuleId) -> Option<RuleInstanceId> {
        self.entries
            .values()
            .find(|state| state.owner == Some(owner) && state.rule == rule)
            .map(|state| state.id)
    }

    pub(crate) fn evaluate_trigger(
        &mut self,
        instance: RuleInstanceId,
        programs: &impl super::evaluate::ProgramLookup,
        trigger: &super::model::TriggerDef,
        input: super::model::RuleEvaluationInput<'_>,
    ) -> Result<Vec<super::model::RuleEmission>, super::evaluate::RuleEvaluationError> {
        self.entries
            .get_mut(&instance)
            .ok_or_else(|| super::evaluate::stat_query_error(0x301))?
            .ledger
            .evaluate(
                programs,
                trigger,
                input,
                super::evaluate::EvaluationBudget::STANDARD,
                4_096,
            )
    }

    pub(crate) fn update(
        &mut self,
        instance: RuleInstanceId,
        slot: StateSlotDefinitionId,
        update: StateSlotUpdateKind,
        operand: RuleValue,
    ) -> Result<(RuleValue, RuleValue), RuleStateError> {
        let state = self
            .entries
            .get_mut(&instance)
            .ok_or(RuleStateError::MissingInstance)?;
        let (definition, current) = state
            .slots
            .iter_mut()
            .find(|(definition, _)| definition.id() == slot)
            .ok_or(RuleStateError::MissingSlot)?;
        if current.kind() != operand.kind() {
            return Err(RuleStateError::TypeMismatch);
        }
        let before = current.clone();
        let next = apply_update(current, operand, update)?;
        if !within_bounds(definition, &next) {
            return Err(RuleStateError::OutOfBounds);
        }
        *current = next.clone();
        Ok((before, next))
    }

    pub(crate) fn reset(&mut self, boundary: SlotResetPoint, owner: Option<UnitId>) -> usize {
        let mut count = 0;
        for state in self.entries.values_mut() {
            if owner.is_some() && state.owner != owner {
                continue;
            }
            for (definition, value) in &mut state.slots {
                if definition.reset_points().contains(&boundary) && value != definition.initial() {
                    value.clone_from(definition.initial());
                    count += 1;
                }
            }
        }
        count
    }
}

fn apply_update(
    current: &RuleValue,
    operand: RuleValue,
    update: StateSlotUpdateKind,
) -> Result<RuleValue, RuleStateError> {
    use RuleValue as V;
    use StateSlotUpdateKind as U;
    match update {
        U::Set => Ok(operand),
        U::Add | U::Subtract => match (current, operand) {
            (V::Integer(left), V::Integer(right)) => {
                let value = if update == U::Add {
                    left.checked_add(right)
                } else {
                    left.checked_sub(right)
                };
                value.map(V::Integer).ok_or(RuleStateError::Numeric)
            }
            (V::Scalar(left), V::Scalar(right)) => {
                let value = if update == U::Add {
                    left.checked_add(right)
                } else {
                    left.checked_sub(right)
                };
                value.map(V::Scalar).map_err(|_| RuleStateError::Numeric)
            }
            _ => Err(RuleStateError::TypeMismatch),
        },
        U::Minimum | U::Maximum => match (current, operand) {
            (V::Integer(left), V::Integer(right)) => Ok(V::Integer(if update == U::Minimum {
                (*left).min(right)
            } else {
                (*left).max(right)
            })),
            (V::Scalar(left), V::Scalar(right)) => Ok(V::Scalar(if update == U::Minimum {
                (*left).min(right)
            } else {
                (*left).max(right)
            })),
            _ => Err(RuleStateError::TypeMismatch),
        },
    }
}

fn within_bounds(definition: &StateSlotDef, value: &RuleValue) -> bool {
    let minimum = definition.minimum();
    let maximum = definition.maximum();
    match value {
        RuleValue::Integer(value) => {
            minimum.is_none_or(|bound| matches!(bound, RuleValue::Integer(bound) if value >= bound))
                && maximum.is_none_or(
                    |bound| matches!(bound, RuleValue::Integer(bound) if value <= bound),
                )
        }
        RuleValue::Scalar(value) => {
            minimum.is_none_or(|bound| matches!(bound, RuleValue::Scalar(bound) if value >= bound))
                && maximum
                    .is_none_or(|bound| matches!(bound, RuleValue::Scalar(bound) if value <= bound))
        }
        _ => minimum.is_none() && maximum.is_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RuleId, SourceDefinitionId,
        rule::model::{BattleRuleScope, RuleSource, RuleValueKind, SourceClass},
    };

    fn runtime() -> BattleRuleDefinition {
        let slot = StateSlotDef::new(
            StateSlotDefinitionId::new(1).unwrap(),
            RuleValueKind::Integer,
            BattleRuleScope::Turn,
            RuleValue::Integer(0),
        )
        .with_bounds(RuleValue::Integer(0), RuleValue::Integer(1))
        .with_reset_points(vec![SlotResetPoint::TurnStart]);
        BattleRuleDefinition::new(
            RuleSource::new(
                SourceDefinitionId::new(1).unwrap(),
                SourceClass::Ability,
                vec![],
                [0x51; 32],
            ),
            vec![slot],
            vec![],
            None,
        )
    }

    #[test]
    fn turn_start_reset_is_scoped_to_the_rule_owner() {
        let mut store = RuleStateStore::default();
        let rule = RuleId::new(1).unwrap();
        let first_owner = UnitId::new(1).unwrap();
        let second_owner = UnitId::new(2).unwrap();
        let first = crate::RuleInstanceId::new(1).unwrap();
        let second = crate::RuleInstanceId::new(2).unwrap();
        assert!(store.insert(first, rule, Some(first_owner), &runtime()));
        assert!(store.insert(second, rule, Some(second_owner), &runtime()));
        for instance in [first, second] {
            store
                .update(
                    instance,
                    StateSlotDefinitionId::new(1).unwrap(),
                    StateSlotUpdateKind::Set,
                    RuleValue::Integer(1),
                )
                .unwrap();
        }
        assert_eq!(store.reset(SlotResetPoint::TurnStart, Some(first_owner)), 1);
        let values = store
            .iter_by_id()
            .map(|instance| instance.slots[0].1.clone())
            .collect::<Vec<_>>();
        assert_eq!(values, [RuleValue::Integer(0), RuleValue::Integer(1)]);
    }
}
