use super::*;
use starclock_combat::{
    EffectDefinitionId, ModifierDefinitionId,
    modifier::model::{FormulaPurpose, FormulaStage, StatKind},
    rule::model::{RuleEventPoint, RuleValue, ValueExpr},
};

const ROBE_RUNTIME_KEY: u64 = u64::MAX - 4;
const LOCAL_EFFECT_BASE: u32 = 0x77d3_0000;
const ROBE_MODIFIER_BASE: u32 = 0x77e5_0000;

#[test]
fn goal07_p3_m11_s03_executes_every_assigned_curio_without_native_handlers() {
    let catalog = catalog();
    let runtime = CurioRuntimeCatalog::compile(&catalog).unwrap();
    for stable_key in [
        "universe.curio.13",
        "universe.curio.14",
        "universe.curio.15",
        "universe.curio.19",
        "universe.curio.2",
        "universe.curio.20",
        "universe.curio.211",
        "universe.curio.22",
    ] {
        let definition = runtime
            .definitions()
            .iter()
            .find(|definition| definition.stable_key() == stable_key)
            .expect("assigned Curio");
        let snapshot = contributions(
            &catalog,
            "universe.path.erudition",
            None,
            Some(stable_key),
            false,
        );
        let binding = snapshot
            .rules()
            .iter()
            .find(|binding| {
                binding.source_binding_key()
                    == definition
                        .states()
                        .iter()
                        .find(|state| state.id() == definition.initial_state())
                        .map(|state| state.source_effect_id())
            })
            .expect("Curio state binding");
        let materialization = materialize(&catalog, &snapshot);
        assert!(
            materialization
                .combat_catalog()
                .rule(binding.rule())
                .is_none_or(|rule| rule
                    .runtime()
                    .is_none_or(|runtime| runtime.native_handler().is_none())),
            "{stable_key} remains generic Rule IR"
        );
    }
}

#[test]
fn robe_of_the_beauty_snapshots_complete_fragment_hundreds() {
    let catalog = catalog();
    let contributions = contributions_many_with_curio_runtime(
        &catalog,
        "universe.path.erudition",
        &[],
        &[],
        Some("universe.curio.14"),
        false,
        0,
        &[(ROBE_RUNTIME_KEY, 250)],
    );
    let materialization = materialize(&catalog, &contributions);
    let binding = binding(&contributions, "14");
    let modifier = materialization
        .combat_catalog()
        .modifier(local_modifier(binding.rule().get(), 0))
        .expect("Robe damage modifier");
    assert_eq!(
        (
            modifier.stat,
            modifier.stage,
            modifier.purpose,
            literal_scalar(&modifier.value)
        ),
        (
            StatKind::Hp,
            FormulaStage::DamageBoost,
            FormulaPurpose::OrdinaryDamage,
            320_000
        )
    );
}

#[test]
fn record_from_beyond_the_sky_has_attack_consumed_protection_and_three_turn_resistance() {
    let catalog = catalog();
    let contributions = contributions(
        &catalog,
        "universe.path.erudition",
        None,
        Some("universe.curio.19"),
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let binding = binding(&contributions, "19");
    let raw = binding.rule().get();
    let combat = materialization.combat_catalog();
    let protection = combat
        .effect(local_effect(raw, 1))
        .expect("damage protection effect");
    let resistance = combat
        .effect(local_effect(raw, 2))
        .expect("effect-resistance effect");
    assert_eq!(protection.modifiers().len(), 7);
    assert!(protection.modifiers().iter().all(|id| {
        let modifier = combat.modifier(*id).unwrap();
        modifier.stage == FormulaStage::Mitigation
            && modifier.purpose != FormulaPurpose::Dot
            && literal_scalar(&modifier.value) == 1_000_000
    }));
    assert_eq!(
        resistance
            .runtime_template()
            .unwrap()
            .duration_expression()
            .map(literal_scalar),
        Some(3_000_000)
    );
    let rule = combat.rule(binding.rule()).unwrap().runtime().unwrap();
    assert!(rule.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::DamageApplied
            && trigger.filter.damage_class
                == Some(starclock_combat::rule::model::RuleDamageClass::Ordinary)
    }));
}

fn binding<'a>(
    contributions: &'a UniverseBattleContributionSet,
    effect: &str,
) -> &'a starclock_mode_universe::battle_contribution::UniverseBattleRuleBinding {
    contributions
        .rules()
        .iter()
        .find(|binding| binding.source_binding_key() == Some(effect))
        .unwrap()
}

fn local_modifier(raw: u32, offset: u32) -> ModifierDefinitionId {
    ModifierDefinitionId::new(ROBE_MODIFIER_BASE + (raw & 0xffff) * 16 + offset).unwrap()
}

fn local_effect(raw: u32, offset: u32) -> EffectDefinitionId {
    EffectDefinitionId::new(LOCAL_EFFECT_BASE + (raw & 0xffff) * 16 + offset).unwrap()
}

fn literal_scalar(value: &ValueExpr) -> i64 {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled(),
        ValueExpr::Literal(RuleValue::Integer(value)) => value * 1_000_000,
        _ => panic!("expected scalar literal, got {value:?}"),
    }
}
