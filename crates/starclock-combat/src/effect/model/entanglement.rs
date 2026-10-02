//! Ordinary resistible Entanglement's explicit shared Quantum fallback.

use super::{
    DispelCategory, DurationClock, EffectCategory, EffectRuntimeDefinition, EffectRuntimeTemplate,
    EffectStackPolicy, EffectTickPhase, EntanglementLevelSource,
};
use crate::{
    Scalar,
    formula::toughness::{self, BreakDamageDefinition},
    modifier::model::StatKind,
};

impl EffectRuntimeDefinition {
    /// Adds hit-accumulating delayed Quantum damage without applying Weakness
    /// Break. Only a finite, cleanseable, refreshing Control at target turn
    /// start is legal; action suppression and ordinary DoT are incompatible.
    ///
    /// The supplied immutable Break formula and target ordinary maximum
    /// Toughness determine the captured base. Live source Break Effect affects
    /// the initial delay; normal Break modifiers and actual broken state are
    /// evaluated on expiry. This shared fallback is not content parity evidence.
    #[must_use]
    pub fn with_entanglement(mut self, damage: BreakDamageDefinition) -> Option<Self> {
        if self.category != EffectCategory::Control
            || self.dispel != DispelCategory::CleanseableControl
            || self.duration_clock != DurationClock::TargetTurnStart
            || self.tick_phase != EffectTickPhase::None
            || self.stack_policy != EffectStackPolicy::Refresh
            || self.stack_limit != 1
            || self.dot.is_some()
            || !self.controlled_actions.is_empty()
            || self.forced_normal_action.is_some()
            || toughness::break_effect_damage(damage, Scalar::ZERO, false).is_err()
        {
            return None;
        }
        self.entanglement = Some(damage);
        self.entanglement_level_source = EntanglementLevelSource::Authored;
        self.specific_resistance_stat = Some(StatKind::ControlResistance);
        Some(self)
    }

    /// Captured formula request for ordinary Entanglement, if declared.
    #[must_use]
    pub const fn entanglement(&self) -> Option<BreakDamageDefinition> {
        self.entanglement
    }

    /// Declares ordinary Entanglement using the first successful applier's
    /// battle level from the shared Break level table. The supplied formula's
    /// level multiplier must be nonnegative but is replaced during capture;
    /// its other factors are unchanged. Unsupported levels cause a typed
    /// transactional fault, never a guessed level base. Refresh keeps the
    /// original resolved formula even when another caster has a different level.
    #[must_use]
    pub fn with_entanglement_from_applier_level(
        self,
        damage: BreakDamageDefinition,
    ) -> Option<Self> {
        let mut runtime = self.with_entanglement(damage)?;
        runtime.entanglement_level_source = EntanglementLevelSource::Applier;
        Some(runtime)
    }

    pub(crate) const fn entanglement_level_source(&self) -> EntanglementLevelSource {
        self.entanglement_level_source
    }
}

impl EffectRuntimeTemplate {
    /// Declares the same ordinary Entanglement lifecycle as the resolved
    /// runtime; duration expressions still resolve through ordinary modifiers.
    #[must_use]
    pub fn with_entanglement(mut self, damage: BreakDamageDefinition) -> Option<Self> {
        // Resolve a positive placeholder duration to validate static semantics;
        // the real authored duration is independently resolved per application.
        self.resolve(Some(1), Scalar::ZERO, None)?
            .with_entanglement(damage)?;
        self.entanglement = Some(damage);
        self.entanglement_level_source = EntanglementLevelSource::Authored;
        self.specific_resistance_stat = Some(StatKind::ControlResistance);
        Some(self)
    }

    #[must_use]
    pub const fn entanglement(&self) -> Option<BreakDamageDefinition> {
        self.entanglement
    }

    /// Resolves duration normally, then captures the first successful applier's
    /// level exactly as `EffectRuntimeDefinition::with_entanglement_from_applier_level`.
    #[must_use]
    pub fn with_entanglement_from_applier_level(
        self,
        damage: BreakDamageDefinition,
    ) -> Option<Self> {
        let mut template = self.with_entanglement(damage)?;
        template.entanglement_level_source = EntanglementLevelSource::Applier;
        Some(template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ControlledAction, ForcedNormalAction, Ratio,
        formula::model::CombatElement,
        rule::model::{RuleValue, ValueExpr},
    };

    fn formula() -> BreakDamageDefinition {
        BreakDamageDefinition {
            attacker_level_multiplier: Scalar::ONE,
            ability_multiplier: Ratio::ONE,
            break_effect: Ratio::ZERO,
            break_damage_increase: Ratio::ZERO,
            defense_multiplier: Ratio::ONE,
            resistance_multiplier: Ratio::ONE,
            vulnerability_multiplier: Ratio::ONE,
            mitigation_multiplier: Ratio::ONE,
            unbroken_multiplier: Ratio::from_scaled(900_000),
        }
    }

    fn runtime() -> EffectRuntimeDefinition {
        EffectRuntimeDefinition::new(
            EffectCategory::Control,
            DispelCategory::CleanseableControl,
            1,
            Some(1),
            DurationClock::TargetTurnStart,
            EffectTickPhase::None,
            EffectStackPolicy::Refresh,
        )
        .unwrap()
    }

    #[test]
    fn ordinary_entanglement_level_source_is_explicit_and_survives_template_resolution() {
        let template = EffectRuntimeTemplate::new(
            EffectCategory::Control,
            DispelCategory::CleanseableControl,
            1,
            Some(ValueExpr::Literal(RuleValue::Integer(1))),
            DurationClock::TargetTurnStart,
            EffectTickPhase::None,
            EffectStackPolicy::Refresh,
        )
        .unwrap()
        .with_entanglement_from_applier_level(formula())
        .unwrap();
        let resolved = template.resolve(Some(1), Scalar::ZERO, None).unwrap();
        assert_eq!(resolved.entanglement(), Some(formula()));
        assert_eq!(
            resolved.entanglement_level_source(),
            EntanglementLevelSource::Applier
        );
        let authored = template
            .with_entanglement(formula())
            .unwrap()
            .resolve(Some(1), Scalar::ZERO, None)
            .unwrap();
        assert_eq!(
            authored.entanglement_level_source(),
            EntanglementLevelSource::Authored
        );
        let authored = resolved.with_entanglement(formula()).unwrap();
        assert_eq!(
            authored.entanglement_level_source(),
            EntanglementLevelSource::Authored
        );
        let mut invalid = formula();
        invalid.break_effect = Ratio::from_scaled(-1);
        assert!(
            runtime()
                .with_entanglement_from_applier_level(invalid)
                .is_none()
        );
    }

    #[test]
    fn ordinary_entanglement_rejects_incompatible_clocks_stacks_categories_and_numeric_inputs() {
        for (category, dispel, stacks, clock, tick, policy) in [
            (
                EffectCategory::Buff,
                DispelCategory::DispellableBuff,
                1,
                DurationClock::TargetTurnStart,
                EffectTickPhase::None,
                EffectStackPolicy::Refresh,
            ),
            (
                EffectCategory::Control,
                DispelCategory::NonDispellable,
                1,
                DurationClock::TargetTurnStart,
                EffectTickPhase::None,
                EffectStackPolicy::Refresh,
            ),
            (
                EffectCategory::Control,
                DispelCategory::CleanseableControl,
                2,
                DurationClock::TargetTurnStart,
                EffectTickPhase::None,
                EffectStackPolicy::Refresh,
            ),
            (
                EffectCategory::Control,
                DispelCategory::CleanseableControl,
                1,
                DurationClock::TargetTurnEnd,
                EffectTickPhase::None,
                EffectStackPolicy::Refresh,
            ),
            (
                EffectCategory::Control,
                DispelCategory::CleanseableControl,
                1,
                DurationClock::TargetTurnStart,
                EffectTickPhase::TurnStart,
                EffectStackPolicy::Refresh,
            ),
            (
                EffectCategory::Control,
                DispelCategory::CleanseableControl,
                1,
                DurationClock::TargetTurnStart,
                EffectTickPhase::None,
                EffectStackPolicy::Replace,
            ),
        ] {
            assert!(
                EffectRuntimeDefinition::new(
                    category,
                    dispel,
                    stacks,
                    Some(1),
                    clock,
                    tick,
                    policy
                )
                .unwrap()
                .with_entanglement(formula())
                .is_none()
            );
        }
        let mut invalid = formula();
        invalid.break_effect = Ratio::from_scaled(-1);
        assert!(runtime().with_entanglement(invalid).is_none());
        assert!(
            runtime()
                .with_control(vec![ControlledAction::NormalAction])
                .unwrap()
                .with_entanglement(formula())
                .is_none()
        );
        let resolved = runtime().with_entanglement(formula()).unwrap();
        assert!(
            resolved
                .clone()
                .with_control(vec![ControlledAction::NormalAction])
                .is_none()
        );
        assert!(
            resolved
                .with_forced_normal_action(ForcedNormalAction::BasicAttackRandomAlly)
                .is_none()
        );
    }

    #[test]
    fn ordinary_entanglement_template_keeps_formula_control_resistance_and_finite_duration() {
        let template = EffectRuntimeTemplate::new(
            EffectCategory::Control,
            DispelCategory::CleanseableControl,
            1,
            Some(ValueExpr::Literal(RuleValue::Integer(2))),
            DurationClock::TargetTurnStart,
            EffectTickPhase::None,
            EffectStackPolicy::Refresh,
        )
        .unwrap()
        .with_entanglement(formula())
        .unwrap();
        let resolved = template.resolve(Some(2), Scalar::ZERO, None).unwrap();
        assert_eq!(resolved.entanglement(), Some(formula()));
        assert_eq!(
            resolved.specific_resistance_stat(),
            Some(StatKind::ControlResistance)
        );
        assert_eq!(resolved.duration(), Some(2));
        assert!(template.with_dot(CombatElement::Quantum, None).is_none());
    }
}
