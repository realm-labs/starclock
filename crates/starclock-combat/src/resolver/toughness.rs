//! Transaction mutations owned by the generic Toughness subsystem.

use crate::{
    EffectInstanceId, OperationId, RawToughness, UnitId, battle::fault::BattleFault,
    formula::model::CombatElement,
};

use super::{
    journal::MutationField,
    transaction::{Transaction, action_fault},
};

impl Transaction<'_> {
    pub(super) fn add_weakness(
        &mut self,
        unit: UnitId,
        element: CombatElement,
        duration_turns: Option<u8>,
        applier: UnitId,
        source_operation: OperationId,
    ) -> Result<bool, BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(47))?;
        let already_present = state.weaknesses.binary_search(&element).is_ok();
        if let Some(turns) = duration_turns {
            if let Some(existing) = state
                .temporary_weaknesses
                .iter_mut()
                .find(|value| value.element == element && value.applier == applier)
            {
                existing.remaining_turns = turns;
                existing.source_operation = source_operation;
            } else if state.permanent_weaknesses.binary_search(&element).is_err() {
                state
                    .temporary_weaknesses
                    .push(crate::actor::store::TemporaryWeaknessState {
                        element,
                        applier,
                        source_operation,
                        remaining_turns: turns,
                    });
            }
        } else if state.permanent_weaknesses.binary_search(&element).is_err() {
            let index = state
                .permanent_weaknesses
                .binary_search(&element)
                .unwrap_err();
            let mut values = state.permanent_weaknesses.to_vec();
            values.insert(index, element);
            state.permanent_weaknesses = values.into_boxed_slice();
        }
        if !already_present {
            let index = state.weaknesses.binary_search(&element).unwrap_err();
            state.weaknesses.insert(index, element);
            self.journal
                .mutation(MutationField::Weakness, 0, element as u64 + 1);
        }
        Ok(!already_present)
    }

    pub(super) fn tick_temporary_weaknesses(
        &mut self,
        unit: UnitId,
    ) -> Result<Vec<(OperationId, CombatElement)>, BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(61))?;
        let mut expired = Vec::new();
        for effect in &mut state.temporary_weaknesses {
            if effect.remaining_turns > 0 {
                effect.remaining_turns -= 1;
                if effect.remaining_turns == 0 {
                    expired.push((effect.source_operation, effect.element));
                }
            }
        }
        for (_, element) in &expired {
            let retained = state.permanent_weaknesses.binary_search(element).is_ok()
                || state
                    .temporary_weaknesses
                    .iter()
                    .any(|value| value.element == *element && value.remaining_turns > 0);
            if !retained {
                let index = state
                    .weaknesses
                    .binary_search(element)
                    .map_err(|_| action_fault(62))?;
                state.weaknesses.remove(index);
                self.journal
                    .mutation(MutationField::Weakness, *element as u64 + 1, 0);
            }
        }
        Ok(expired)
    }

    pub(super) fn set_toughness(
        &mut self,
        unit: UnitId,
        layer_key: u32,
        value: RawToughness,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(48))?;
        let layer = state
            .toughness_layers
            .iter_mut()
            .find(|layer| layer.spec.key() == layer_key)
            .ok_or_else(|| action_fault(49))?;
        let before = layer.current;
        if before != value {
            layer.current = value;
            self.journal.mutation(
                MutationField::Toughness,
                before.get() as u64,
                value.get() as u64,
            );
        }
        Ok(())
    }

    pub(super) fn set_weakness_broken(
        &mut self,
        unit: UnitId,
        value: bool,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(50))?;
        if state.weakness_broken != value {
            let before = state.weakness_broken;
            state.weakness_broken = value;
            self.journal.mutation(
                MutationField::WeaknessBroken,
                u64::from(before),
                u64::from(value),
            );
        }
        Ok(())
    }

    pub(super) fn recover_toughness(
        &mut self,
        unit: UnitId,
    ) -> Result<Vec<(u32, RawToughness, RawToughness)>, BattleFault> {
        let changes = self
            .state
            .units
            .get(unit)
            .ok_or_else(|| action_fault(57))?
            .toughness_layers
            .iter()
            .filter_map(|layer| {
                let scaled = layer
                    .spec
                    .recovery_ratio()
                    .checked_apply(
                        crate::Scalar::checked_from_integer(layer.spec.maximum().get()).ok()?,
                        crate::Rounding::Floor,
                    )
                    .ok()?;
                let recovered = RawToughness::from_scalar(scaled, crate::Rounding::Floor).ok()?;
                let after = RawToughness::new(
                    layer
                        .current
                        .get()
                        .saturating_add(recovered.get())
                        .min(layer.spec.maximum().get()),
                )
                .ok()?;
                (after != layer.current).then_some((layer.spec.key(), layer.current, after))
            })
            .collect::<Vec<_>>();
        for (key, _, after) in &changes {
            self.set_toughness(unit, *key, *after)?;
        }
        Ok(changes)
    }

    pub(super) fn record_break_effect(
        &mut self,
        effect: crate::effect::break_effect::BreakEffectState,
    ) {
        self.state.break_effects.insert(effect);
        self.journal.mutation(MutationField::BreakEffect, 0, 1);
    }

    pub(super) fn update_break_effect(
        &mut self,
        id: EffectInstanceId,
        remaining_turns: u8,
        stacks: u8,
    ) -> Result<(), BattleFault> {
        let effect = self
            .state
            .break_effects
            .get_mut(id)
            .ok_or_else(|| action_fault(56))?;
        let before = (u64::from(effect.remaining_turns) << 8) | u64::from(effect.stacks);
        let after = (u64::from(remaining_turns) << 8) | u64::from(stacks);
        if before != after {
            effect.remaining_turns = remaining_turns;
            effect.stacks = stacks;
            self.journal
                .mutation(MutationField::BreakEffect, before, after);
        }
        Ok(())
    }

    pub(super) fn increment_entanglement_for_hit(
        &mut self,
        targets: &[UnitId],
    ) -> Result<(), BattleFault> {
        let target_set = targets
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let changes = self
            .state
            .break_effects
            .canonical_entries()
            .iter()
            .filter(|effect| {
                effect.remaining_turns > 0
                    && effect.plan.element == CombatElement::Quantum
                    && target_set.contains(&effect.owner)
                    && effect.stacks < effect.plan.maximum_stacks
            })
            .map(|effect| (effect.id, effect.remaining_turns, effect.stacks + 1))
            .collect::<Vec<_>>();
        for (id, turns, stacks) in changes {
            self.update_break_effect(id, turns, stacks)?;
        }
        Ok(())
    }
}
