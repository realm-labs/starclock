use super::*;
use starclock_combat::{
    modifier::model::{FormulaStage, StatKind},
    rule::model::{ProgramStep, RuleEventPoint, RuleOperationTemplate, RuleValue, ValueExpr},
};

const VERMEIL: (&str, u32) = ("universe.blessing.612450", 2);
const CRIT_RATE: (&str, u32) = ("universe.blessing.612451", 2);
const CRIT_DAMAGE: (&str, u32) = ("universe.blessing.612452", 2);
const BREAK_DELAY: (&str, u32) = ("universe.blessing.612453", 2);
const ENTRY_SPEED: (&str, u32) = ("universe.blessing.612454", 2);
const TURN_ADVANCE: (&str, u32) = ("universe.blessing.612455", 2);

#[test]
fn goal07_p2_m06_s03_materializes_every_selected_level_without_native_handlers() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    for key in [
        "StageAbility_61245001",
        "StageAbility_61245101",
        "StageAbility_61245201",
        "StageAbility_61245301",
        "StageAbility_61245401",
        "StageAbility_61245501",
    ] {
        let rule = materialization
            .combat_catalog()
            .rule(binding(&contributions, key).rule())
            .unwrap();
        assert!(
            rule.runtime()
                .is_some_and(|runtime| runtime.native_handler().is_none()),
            "{key} remains generic Rule IR"
        );
    }
}

#[test]
fn passive_critical_and_blessing_count_speed_keep_exact_level_two_values() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    assert_eq!(
        persistent_modifier(combat, &contributions, "StageAbility_61245001"),
        (StatKind::Spd, FormulaStage::PercentOfBase, 240_000),
        "six selected Hunt Blessings at 4% each produce 24% SPD"
    );
    assert_eq!(
        persistent_modifier(combat, &contributions, "StageAbility_61245101"),
        (StatKind::CritRate, FormulaStage::Flat, 160_000)
    );
    assert_eq!(
        persistent_modifier(combat, &contributions, "StageAbility_61245201"),
        (StatKind::CritDamage, FormulaStage::Flat, 300_000)
    );
}

#[test]
fn thundering_chariot_and_astral_menace_use_exact_timeline_operations() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let delay = operation(
        combat,
        &contributions,
        "StageAbility_61245301",
        RuleEventPoint::WeaknessBroken,
    );
    assert!(matches!(
        delay,
        RuleOperationTemplate::DelayAction { amount, .. }
            if literal_scalar(amount) == 300_000
    ));
    let advance = operation(
        combat,
        &contributions,
        "StageAbility_61245501",
        RuleEventPoint::TurnEnded,
    );
    assert!(matches!(
        advance,
        RuleOperationTemplate::AdvanceAction { amount, .. }
            if literal_scalar(amount) == 120_000
    ));
}

#[test]
fn constellation_surge_is_removed_by_the_first_damage_event() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let binding = binding(&contributions, "StageAbility_61245401");
    let rule = combat.rule(binding.rule()).unwrap();
    let runtime = rule.runtime().unwrap();
    assert!(runtime.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::BattleStarted
            && combat
                .program(trigger.program)
                .unwrap()
                .steps()
                .iter()
                .any(|step| {
                    matches!(
                        step,
                        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect { .. })
                    )
                })
    }));
    let remove = runtime
        .triggers()
        .iter()
        .find(|trigger| trigger.event_point == RuleEventPoint::DamageApplied)
        .expect("damage removes entry speed");
    let effect = match &combat.program(remove.program).unwrap().steps()[0] {
        ProgramStep::Operation(RuleOperationTemplate::RemoveEffect { effect, .. }) => *effect,
        _ => panic!("expected effect removal"),
    };
    let modifier = combat
        .modifier(combat.effect(effect).unwrap().modifiers()[0])
        .unwrap();
    assert_eq!(
        (
            modifier.stat,
            modifier.stage,
            literal_scalar(&modifier.value)
        ),
        (StatKind::Spd, FormulaStage::PercentOfBase, 450_000)
    );
}

fn full_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many(
        catalog,
        "universe.path.hunt",
        &[
            VERMEIL,
            CRIT_RATE,
            CRIT_DAMAGE,
            BREAK_DELAY,
            ENTRY_SPEED,
            TURN_ADVANCE,
        ],
        None,
        false,
    )
}

fn binding<'a>(
    contributions: &'a UniverseBattleContributionSet,
    key: &str,
) -> &'a starclock_mode_universe::battle_contribution::UniverseBattleRuleBinding {
    contributions
        .rules()
        .iter()
        .find(|binding| binding.source_binding_key() == Some(key))
        .unwrap_or_else(|| panic!("{key} selected"))
}

fn persistent_modifier(
    combat: &starclock_combat::catalog::CombatCatalog,
    contributions: &UniverseBattleContributionSet,
    key: &str,
) -> (StatKind, FormulaStage, i64) {
    let rule = combat.rule(binding(contributions, key).rule()).unwrap();
    let modifier = rule
        .programs()
        .iter()
        .flat_map(|program| combat.program(*program).unwrap().effects())
        .filter_map(|effect| combat.effect(*effect))
        .flat_map(|effect| effect.modifiers())
        .filter_map(|modifier| combat.modifier(*modifier))
        .find(|modifier| modifier.source_stack_slot.is_none())
        .expect("persistent modifier");
    (
        modifier.stat,
        modifier.stage,
        literal_scalar(&modifier.value),
    )
}

fn operation<'a>(
    combat: &'a starclock_combat::catalog::CombatCatalog,
    contributions: &UniverseBattleContributionSet,
    key: &str,
    point: RuleEventPoint,
) -> &'a RuleOperationTemplate {
    let rule = combat.rule(binding(contributions, key).rule()).unwrap();
    let trigger = rule
        .runtime()
        .unwrap()
        .triggers()
        .iter()
        .find(|trigger| trigger.event_point == point)
        .unwrap();
    match &combat.program(trigger.program).unwrap().steps()[0] {
        ProgramStep::Operation(operation) => operation,
        _ => panic!("expected operation"),
    }
}

fn literal_scalar(value: &ValueExpr) -> i64 {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled(),
        _ => panic!("expected scalar literal, got {value:?}"),
    }
}
