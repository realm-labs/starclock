use super::*;
use starclock_combat::{
    ParticipantInitialState,
    modifier::model::{FormulaStage, StatKind},
    rule::model::{
        ProgramStep, RuleEventPoint, RuleOperationTemplate, RuleValue, StateSlotDef, ValueExpr,
    },
};

const ADEPTS_BOW: (&str, u32) = ("universe.blessing.612442", 2);
const MISTWRAITH: (&str, u32) = ("universe.blessing.612443", 2);
const STARLIT: (&str, u32) = ("universe.blessing.612444", 2);
const BORISIN: (&str, u32) = ("universe.blessing.612445", 2);
const RAINBOW: (&str, u32) = ("universe.blessing.612446", 2);
const VERMEIL: (&str, u32) = ("universe.blessing.612450", 2);
const CRITICAL_BOOST_RAW: u32 = 0x7940_0001;
const MISTWRAITH_EFFECT_RAW: u32 = 0x7950_0001;

#[test]
fn goal07_p2_m06_s02_materializes_all_selected_levels_as_generic_rule_ir() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.hunt",
        &[ADEPTS_BOW, MISTWRAITH, STARLIT, BORISIN, RAINBOW, VERMEIL],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    for key in [
        "StageAbility_61244202",
        "StageAbility_61244302",
        "StageAbility_61244402",
        "StageAbility_61244502",
        "StageAbility_61244602",
        "StageAbility_61245001",
    ] {
        let rule = combat.rule(binding(&contributions, key).rule()).unwrap();
        assert!(
            rule.runtime()
                .is_some_and(|runtime| runtime.native_handler().is_none()),
            "{key} remains generic Rule IR"
        );
    }
    assert!(
        combat
            .effect(starclock_combat::EffectDefinitionId::new(CRITICAL_BOOST_RAW).unwrap())
            .is_some(),
        "Adept's Bow receives the one shared Critical Boost definition"
    );
}

#[test]
fn enhanced_starlit_and_rainbow_execute_exact_kill_energy_and_healing() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.hunt",
        &[STARLIT, RAINBOW],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let original = durable_spec_with_two_enemy_hp(
        &materialization,
        0x91,
        [Hp::new(1).unwrap(), Hp::new(2_000_000_000).unwrap()],
    );
    let spec = wounded_players(original, 20_000, 0x92);
    let (mut battle, started) = start(&materialization, spec, 0x93);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = first_normal_action(&mut battle);
    assert!(
        resolution.fault().is_none(),
        "{:?} {:?}",
        resolution.fault(),
        resolution.events()
    );
    let beneficiary = battle
        .view()
        .units_by_id()
        .filter(|unit| unit.side() == TeamSide::Player)
        .find(|unit| unit.current_energy().scaled() > 0)
        .expect("killer receives energy");
    assert_eq!(
        beneficiary.current_energy().scaled(),
        beneficiary.maximum_energy().scaled(),
        "enhanced Starlit Hunt restores 100% Max Energy"
    );
    assert_eq!(
        beneficiary.current_hp().get(),
        68_000,
        "Rainbow Fang restores exactly 48% Max HP on defeat"
    );
}

#[test]
fn enhanced_mistwraith_authors_two_stack_attack_and_fixed_skill_point_roll() {
    let catalog = catalog();
    let contributions =
        contributions_many(&catalog, "universe.path.hunt", &[MISTWRAITH], None, false);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let effect = combat
        .effect(starclock_combat::EffectDefinitionId::new(MISTWRAITH_EFFECT_RAW).unwrap())
        .expect("Mistwraith effect");
    assert_eq!(effect.runtime_template().unwrap().stack_limit(), 2);
    let modifier = combat.modifier(effect.modifiers()[0]).unwrap();
    assert_eq!(modifier.stat, StatKind::Atk);
    assert_eq!(modifier.stage, FormulaStage::PercentOfBase);
    assert!(expression_has_scalar(&modifier.value, 400_000));
    let rule = combat
        .rule(binding(&contributions, "StageAbility_61244302").rule())
        .unwrap();
    assert!(rule.runtime().unwrap().triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::EffectApplied
            && combat
                .program(trigger.program)
                .unwrap()
                .steps()
                .iter()
                .any(|step| {
                    matches!(
                        step,
                        ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                            resource: starclock_combat::rule::model::RuleResourceKind::SkillPoints,
                            ..
                        })
                    )
                })
    }));
}

#[test]
fn enhanced_borisin_starts_at_five_and_advances_after_the_sixth_ally_turn() {
    let catalog = catalog();
    let contributions = contributions_many(&catalog, "universe.path.hunt", &[BORISIN], None, false);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let rule = combat
        .rule(binding(&contributions, "StageAbility_61244502").rule())
        .unwrap();
    let runtime = rule.runtime().unwrap();
    assert!(runtime.state_slots().iter().any(is_initial_five));
    assert!(runtime.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::TurnEnded
            && combat
                .program(trigger.program)
                .unwrap()
                .steps()
                .iter()
                .any(|step| {
                    matches!(
                        step,
                        ProgramStep::Operation(RuleOperationTemplate::AdvanceAction { .. })
                    )
                })
    }));
}

#[test]
fn enhanced_adepts_bow_inherits_on_ultimate_and_follow_up_only_when_boost_exists() {
    let catalog = catalog();
    let contributions =
        contributions_many(&catalog, "universe.path.hunt", &[ADEPTS_BOW], None, false);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let rule = combat
        .rule(binding(&contributions, "StageAbility_61244202").rule())
        .unwrap();
    let runtime = rule.runtime().unwrap();
    assert_eq!(
        runtime
            .triggers()
            .iter()
            .filter(|trigger| trigger.event_point == RuleEventPoint::ActionStarted)
            .count(),
        2
    );
    let program = combat.program(rule.programs()[0]).unwrap();
    assert!(program.steps().iter().any(|step| matches!(
        step,
        ProgramStep::Operation(RuleOperationTemplate::RemoveEffect { effect, .. })
            if effect.get() == CRITICAL_BOOST_RAW
    )));
    assert!(program.steps().iter().any(|step| matches!(
        step,
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect { effect, stacks, .. })
            if effect.get() == CRITICAL_BOOST_RAW && expression_has_integer(stacks, 1)
    )));
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

fn is_initial_five(slot: &StateSlotDef) -> bool {
    slot.initial() == &RuleValue::Integer(5)
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
        _ => false,
    }
}

fn expression_has_integer(value: &ValueExpr, expected: i64) -> bool {
    match value {
        ValueExpr::Literal(RuleValue::Integer(value)) => *value == expected,
        ValueExpr::Add(lhs, rhs) => {
            expression_has_integer(lhs, expected) || expression_has_integer(rhs, expected)
        }
        _ => false,
    }
}

fn wounded_players(original: BattleSpec, current_hp: i64, marker: u8) -> BattleSpec {
    let participants = original
        .participants()
        .iter()
        .map(|participant| {
            if participant.side() != TeamSide::Player {
                return participant.clone();
            }
            let combatant = participant.combatant();
            participant
                .clone()
                .with_initial_state(
                    ParticipantInitialState::new(
                        Hp::new(current_hp).unwrap(),
                        combatant.maximum_hp(),
                        combatant.current_energy(),
                        combatant.maximum_energy(),
                        starclock_combat::LifeState::Alive,
                        starclock_combat::PresenceState::Present,
                    )
                    .unwrap(),
                )
                .unwrap()
        })
        .collect();
    BattleSpec::new(
        original.rules_revision(),
        AssemblyDigest::new([marker; 32]).unwrap(),
        original.encounter(),
        participants,
        original.resources(TeamSide::Player).clone(),
        original.resources(TeamSide::Enemy).clone(),
        original.concede_policy(),
    )
    .unwrap()
}
