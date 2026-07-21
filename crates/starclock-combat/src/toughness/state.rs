use crate::{Ratio, RawToughness, formula::model::CombatElement};

use super::model::{BreakCreditPolicy, ToughnessLayerSpec, ToughnessWeaknessPolicy};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ToughnessLayerState {
    pub(crate) spec: ToughnessLayerSpec,
    pub(crate) current: RawToughness,
}

impl ToughnessLayerState {
    pub(crate) fn from_spec(spec: ToughnessLayerSpec) -> Self {
        Self {
            current: spec.maximum(),
            spec,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RoutedReduction {
    pub(crate) layer_key: u32,
    pub(crate) attempted: RawToughness,
    pub(crate) effective: RawToughness,
    pub(crate) before: RawToughness,
    pub(crate) after: RawToughness,
    pub(crate) depleted: bool,
    pub(crate) changed_global_broken: bool,
    pub(crate) break_element: CombatElement,
    pub(crate) applies_break_damage: bool,
    pub(crate) applies_break_effect: bool,
    pub(crate) break_credit: BreakCreditPolicy,
    pub(crate) maximum: RawToughness,
}

#[cfg(test)]
pub(crate) fn route_reduction(
    layers: &mut [ToughnessLayerState],
    weaknesses: &[CombatElement],
    globally_broken: bool,
    attack_element: CombatElement,
    attempted: RawToughness,
) -> Option<RoutedReduction> {
    route_reduction_with_override(
        layers,
        weaknesses,
        globally_broken,
        attack_element,
        attempted,
        false,
    )
}

pub(crate) fn route_reduction_with_override(
    layers: &mut [ToughnessLayerState],
    weaknesses: &[CombatElement],
    globally_broken: bool,
    attack_element: CombatElement,
    attempted: RawToughness,
    ignores_weakness: bool,
) -> Option<RoutedReduction> {
    let layer = layers.iter_mut().find(|layer| {
        layer.spec.active()
            && !layer.spec.locked()
            && layer.current.get() > 0
            && (!globally_broken || layer.spec.reducible_while_broken())
            && (ignores_weakness
                || eligibility_multiplier(layer.spec.weakness_policy(), weaknesses, attack_element)
                    .scaled()
                    > 0)
    })?;
    let eligibility = if ignores_weakness {
        Ratio::ONE
    } else {
        eligibility_multiplier(layer.spec.weakness_policy(), weaknesses, attack_element)
    };
    let scaled = eligibility
        .checked_apply(
            crate::Scalar::checked_from_integer(attempted.get()).ok()?,
            crate::Rounding::Floor,
        )
        .ok()?;
    let eligible_attempted = RawToughness::from_scalar(scaled, crate::Rounding::Floor).ok()?;
    let before = layer.current;
    let effective = RawToughness::new(eligible_attempted.get().min(before.get())).ok()?;
    let after = RawToughness::new(before.get() - effective.get()).ok()?;
    layer.current = after;
    let depleted = before.get() > 0 && after.get() == 0;
    Some(RoutedReduction {
        layer_key: layer.spec.key(),
        attempted: eligible_attempted,
        effective,
        before,
        after,
        depleted,
        changed_global_broken: depleted && layer.spec.changes_global_broken(),
        break_element: layer.spec.break_element().unwrap_or(attack_element),
        applies_break_damage: layer.spec.applies_break_damage(),
        applies_break_effect: layer.spec.applies_break_effect(),
        break_credit: layer.spec.break_credit(),
        maximum: layer.spec.maximum(),
    })
}

fn eligibility_multiplier(
    policy: ToughnessWeaknessPolicy,
    weaknesses: &[CombatElement],
    element: CombatElement,
) -> Ratio {
    let matches = weaknesses.binary_search(&element).is_ok();
    match policy {
        ToughnessWeaknessPolicy::MatchingOnly if matches => Ratio::ONE,
        ToughnessWeaknessPolicy::MatchingOnly => Ratio::ZERO,
        ToughnessWeaknessPolicy::AnyElement => Ratio::ONE,
        ToughnessWeaknessPolicy::OffWeakness(multiplier) => {
            if matches {
                Ratio::ONE
            } else {
                multiplier
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toughness::model::{
        ToughnessLayerKind, ToughnessLayerSpec, ToughnessWeaknessPolicy,
    };

    fn raw(value: i64) -> RawToughness {
        RawToughness::new(value).unwrap()
    }

    #[test]
    fn routing_is_ordered_non_spilling_and_respects_locked_and_broken_layers() {
        let locked = ToughnessLayerSpec::ordinary(1, raw(30))
            .unwrap()
            .with_locked(true);
        let sequential = ToughnessLayerSpec::ordinary(2, raw(20))
            .unwrap()
            .with_kind(ToughnessLayerKind::Sequential)
            .with_weakness_policy(ToughnessWeaknessPolicy::AnyElement)
            .unwrap()
            .with_break_behavior(false, true, true, false)
            .with_break_element(CombatElement::Ice);
        let exo = ToughnessLayerSpec::ordinary(3, raw(40))
            .unwrap()
            .with_kind(ToughnessLayerKind::ExoToughness)
            .with_break_behavior(true, true, true, false);
        let mut layers = vec![locked, sequential, exo]
            .into_iter()
            .map(ToughnessLayerState::from_spec)
            .collect::<Vec<_>>();
        let first = route_reduction(&mut layers, &[], false, CombatElement::Fire, raw(90)).unwrap();
        assert_eq!(
            (
                first.layer_key,
                first.attempted.get(),
                first.effective.get(),
                first.break_element
            ),
            (2, 90, 20, CombatElement::Ice)
        );
        assert_eq!(
            layers[2].current.get(),
            40,
            "overflow is never inferred across layers"
        );
        let second = route_reduction(
            &mut layers,
            &[CombatElement::Fire],
            true,
            CombatElement::Fire,
            raw(90),
        )
        .unwrap();
        assert_eq!((second.layer_key, second.effective.get()), (3, 40));
    }

    #[test]
    fn authored_off_weakness_ratio_scales_before_remaining_toughness_bound() {
        let layer = ToughnessLayerSpec::ordinary(1, raw(50))
            .unwrap()
            .with_weakness_policy(ToughnessWeaknessPolicy::OffWeakness(Ratio::from_scaled(
                500_000,
            )))
            .unwrap();
        let mut layers = vec![ToughnessLayerState::from_spec(layer)];
        let result = route_reduction(
            &mut layers,
            &[CombatElement::Ice],
            false,
            CombatElement::Fire,
            raw(80),
        )
        .unwrap();
        assert_eq!(
            (
                result.attempted.get(),
                result.effective.get(),
                result.after.get()
            ),
            (40, 40, 10)
        );
    }
}
