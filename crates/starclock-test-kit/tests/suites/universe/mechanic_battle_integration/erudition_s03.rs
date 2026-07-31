use super::*;
use starclock_combat::{
    EffectDefinitionId,
    modifier::model::{FormulaPurpose, FormulaStage, FormulaSubject, ModifierFilter, StatKind},
    rule::model::{RuleValue, ValueExpr},
};

const CRIT_RATE: (&str, u32) = ("universe.blessing.612851", 1);
const CRIT_DAMAGE: (&str, u32) = ("universe.blessing.612852", 2);
const NEXT_ATTACK: (&str, u32) = ("universe.blessing.612853", 2);
const AOE_ATTACK: (&str, u32) = ("universe.blessing.612854", 2);
const AOE_DEFENSE: (&str, u32) = ("universe.blessing.612855", 2);
const GEARS: (&str, u32) = ("universe.blessing.612850", 2);

const LOCAL_EFFECT_BASE: u32 = 0x7e90_0000;
const LOCAL_MODIFIER_BASE: u32 = 0x7ec0_0000;

#[test]
fn goal07_p2_m10_s03_materializes_all_assigned_rules_without_native_handlers() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    for key in [
        "StageAbility_612850",
        "StageAbility_612851",
        "StageAbility_612852",
        "StageAbility_612853",
        "StageAbility_612854",
        "StageAbility_612855",
    ] {
        let rule = combat
            .rule(binding(&contributions, key).rule())
            .unwrap_or_else(|| panic!("{key} is executable"));
        assert!(
            rule.runtime()
                .is_some_and(|runtime| runtime.native_handler().is_none()),
            "{key} remains generic Rule IR"
        );
    }
}

#[test]
fn ultimate_critical_rules_use_exact_source_filtered_stats() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    for (key, stat, value) in [
        ("StageAbility_612851", StatKind::CritRate, 180_000),
        ("StageAbility_612852", StatKind::CritDamage, 450_000),
    ] {
        let raw = binding(&contributions, key).rule().get();
        let modifier = combat
            .modifier(
                starclock_combat::ModifierDefinitionId::new(
                    LOCAL_MODIFIER_BASE + (raw & 0xffff) * 16,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(
            (modifier.stat, modifier.stage, modifier.purpose),
            (stat, FormulaStage::Flat, FormulaPurpose::Stat)
        );
        assert!(expression_has_scalar(&modifier.value, value));
        assert!(modifier.filters.iter().any(|filter| {
            matches!(
                filter,
                ModifierFilter::FormulaSubject(FormulaSubject::Source)
            )
        }));
        assert!(modifier.filters.iter().any(
            |filter| matches!(filter, ModifierFilter::AbilityTag(tag) if tag.as_ref() == "ultimate")
        ));
    }
}

#[test]
fn ultimate_arms_exact_next_attack_boost_and_the_attack_consumes_it() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.erudition",
        &[NEXT_ATTACK],
        None,
        false,
    );
    let roster = roster_for_forms_with_ability_kinds(
        &catalog,
        [19, 2, 3, 4],
        None,
        &[AbilityKind::Ultimate],
        true,
    );
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let combat = materialization.combat_catalog();
    let binding = binding(&contributions, "StageAbility_612853");
    let effect = local_effect(binding);
    let modifier = combat
        .modifier(combat.effect(effect).unwrap().modifiers()[0])
        .unwrap();
    assert!(expression_has_scalar(&modifier.value, 750_000));
    assert!(modifier.filters.iter().any(
        |filter| matches!(filter, ModifierFilter::AbilityTag(tag) if tag.as_ref() == "attack")
    ));

    let (mut battle, started) = start(
        &materialization,
        durable_spec(&materialization, 0xd3, false),
        0xd4,
    );
    assert!(started.fault().is_none(), "{:#?}", started.events());
    let ultimate = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseInterrupt { .. }))
        .expect("an Ultimate is legal")
        .clone();
    let actor = match &ultimate {
        Command::UseInterrupt { actor, .. } => *actor,
        _ => unreachable!("selected Ultimate"),
    };
    let resolution = battle.apply(ultimate).unwrap();
    assert!(resolution.fault().is_none(), "{:#?}", resolution.events());
    assert!(has_effect(&battle, actor, effect));

    close_interrupt_window(&mut battle);
    let basic = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseAbility {
                    actor: command_actor,
                    ability,
                    ..
                } if *command_actor == actor
                    && combat
                        .ability(*ability)
                        .and_then(|definition| definition.action())
                        .is_some_and(|action| action.kind() == AbilityKind::Basic)
            )
        })
        .expect("the same actor can make its normal Basic attack")
        .clone();
    let resolution = battle.apply(basic).unwrap();
    assert!(resolution.fault().is_none(), "{:#?}", resolution.events());
    assert!(!has_effect(&battle, actor, effect));
}

#[test]
fn aoe_attack_applies_exact_three_turn_attack_and_defense_effects() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.erudition",
        &[AOE_ATTACK, AOE_DEFENSE],
        None,
        false,
    );
    let roster = roster_for_forms_with_ability_kinds(
        &catalog,
        [18, 2, 3, 4],
        None,
        &[AbilityKind::Skill],
        false,
    );
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let (mut battle, started) = start(
        &materialization,
        durable_spec(&materialization, 0xd5, false),
        0xd6,
    );
    assert!(started.fault().is_none(), "{:#?}", started.events());
    close_interrupt_window(&mut battle);
    let aoe = AbilityId::new(20019).unwrap();
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { ability, .. } if *ability == aoe))
        .expect("form 18 AoE Skill is legal")
        .clone();
    let actor = match &command {
        Command::UseAbility { actor, .. } => *actor,
        _ => unreachable!("selected ability"),
    };
    let resolution = battle.apply(command).unwrap();
    assert!(resolution.fault().is_none(), "{:#?}", resolution.events());

    for (key, stat) in [
        ("StageAbility_612854", StatKind::Atk),
        ("StageAbility_612855", StatKind::Def),
    ] {
        let binding = binding(&contributions, key);
        let effect = local_effect(binding);
        let state = battle
            .view()
            .effects_by_id()
            .find(|state| state.target() == actor && state.definition() == effect)
            .unwrap_or_else(|| panic!("{key} effect applied"));
        assert_eq!(state.remaining(), Some(3));
        assert_eq!(
            state.duration_clock(),
            starclock_combat::DurationClock::OwnerTurnEnd
        );
        let modifier = materialization
            .combat_catalog()
            .modifier(
                materialization
                    .combat_catalog()
                    .effect(effect)
                    .unwrap()
                    .modifiers()[0],
            )
            .unwrap();
        assert_eq!(
            (modifier.stat, modifier.stage, modifier.purpose),
            (stat, FormulaStage::PercentOfBase, FormulaPurpose::Stat)
        );
        assert!(expression_has_scalar(&modifier.value, 400_000));
    }
}

fn full_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many(
        catalog,
        "universe.path.erudition",
        &[
            GEARS,
            CRIT_RATE,
            CRIT_DAMAGE,
            NEXT_ATTACK,
            AOE_ATTACK,
            AOE_DEFENSE,
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

fn local_effect(
    binding: &starclock_mode_universe::battle_contribution::UniverseBattleRuleBinding,
) -> EffectDefinitionId {
    EffectDefinitionId::new(LOCAL_EFFECT_BASE + (binding.rule().get() & 0xffff) * 16).unwrap()
}

fn close_interrupt_window(battle: &mut Battle) {
    while battle
        .decision()
        .is_some_and(|decision| decision.kind() == starclock_combat::DecisionKind::InterruptWindow)
    {
        let decision = battle.decision().unwrap().id();
        let resolution = battle
            .apply(Command::PassInterruptWindow { decision })
            .unwrap();
        assert!(resolution.fault().is_none(), "{:#?}", resolution.events());
    }
}

fn has_effect(
    battle: &Battle,
    target: starclock_combat::UnitId,
    effect: EffectDefinitionId,
) -> bool {
    battle
        .view()
        .effects_by_id()
        .any(|state| state.target() == target && state.definition() == effect)
}

fn expression_has_scalar(value: &ValueExpr, expected: i64) -> bool {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled() == expected,
        ValueExpr::Multiply { lhs, rhs, .. }
        | ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Divide { lhs, rhs, .. }
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => {
            expression_has_scalar(lhs, expected) || expression_has_scalar(rhs, expected)
        }
        ValueExpr::Negate(value) | ValueExpr::Convert { value, .. } => {
            expression_has_scalar(value, expected)
        }
        _ => false,
    }
}
