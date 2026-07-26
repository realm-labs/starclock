use super::*;
use starclock_combat::{
    EffectDefinitionId,
    rule::model::{RuleEventPoint, RuleOperationTemplate, RuleValue, ValueExpr},
};

const ULTIMATE_HEALING: (&str, u32) = ("universe.blessing.612856", 2);
const LETHAL_ENERGY_HEALING: (&str, u32) = ("universe.blessing.612857", 2);
const MELT_CORE: &str = "universe.resonance.612821";
const CHAIN_CONTAGION: &str = "universe.resonance.612822";
const MEMETIC_INVERSION: &str = "universe.resonance.612823";

const SYNAPSE_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x7df0_0001).expect("reserved Erudition effect ID");
const LOCAL_EFFECT_BASE: u32 = 0x7ef0_0000;

#[test]
fn goal07_p2_m10_s04_materializes_every_assigned_rule_as_generic_ir() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    for key in [
        "StageAbility_612856",
        "StageAbility_612857",
        "StageAbility_612820",
        "StageAbility_612821",
        "StageAbility_612822",
        "StageAbility_612823",
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
fn healing_and_team_lethal_guard_retain_exact_energy_contracts() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    let healing = combat
        .rule(binding(&contributions, "StageAbility_612856").rule())
        .unwrap()
        .runtime()
        .unwrap();
    assert!(healing.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::ActionResolved
            && combat.program(trigger.program).is_some_and(|program| {
                program.steps().iter().any(|step| {
                    matches!(
                        step,
                        starclock_combat::rule::model::ProgramStep::Operation(
                            RuleOperationTemplate::Heal { amount, .. }
                        ) if expression_has_scalar(amount, 240_000)
                    )
                })
            })
    }));

    let guard_rule = combat
        .rule(binding(&contributions, "StageAbility_612857").rule())
        .unwrap()
        .runtime()
        .unwrap();
    assert!(
        guard_rule
            .triggers()
            .iter()
            .any(|trigger| trigger.event_point == RuleEventPoint::InformationalRule)
    );
    let raw = binding(&contributions, "StageAbility_612857").rule().get();
    let guard = combat
        .effect(
            EffectDefinitionId::new(LOCAL_EFFECT_BASE + (raw & 0xffff) * 16)
                .expect("local effect ID"),
        )
        .expect("team lethal guard effect");
    assert_eq!(
        guard.runtime_template().unwrap().damage_guard(),
        starclock_combat::EffectDamageGuard::TeamDefeatOnce
    );
}

#[test]
fn complete_erudition_resonance_applies_fifteen_shared_synapse_triggers() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let resonance = combat
        .ability(AbilityId::new(RESONANCE_ABILITY_RAW).unwrap())
        .expect("Erudition resonance");
    assert_eq!(resonance.action().unwrap().kind(), AbilityKind::Ultimate);
    let synapse = combat.effect(SYNAPSE_EFFECT).expect("Synapse effect");
    assert_eq!(synapse.runtime_template().unwrap().stack_limit(), 15);

    let (mut battle, started) = start(
        &materialization,
        durable_spec(&materialization, 0xe4, true),
        0xe5,
    );
    assert!(started.fault().is_none(), "{:#?}", started.events());
    let applied = use_resonance(&mut battle);
    assert!(applied.fault().is_none(), "{:#?}", applied.events());
    let enemy = battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Enemy)
        .expect("enemy")
        .id();
    assert_eq!(synapse_stacks(&battle, enemy), Some(15));

    close_interrupt_window(&mut battle);
    let attack = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseAbility { actor, ability, .. }
                    if battle
                        .view()
                        .units_by_id()
                        .find(|unit| unit.id() == *actor)
                        .is_some_and(|unit| unit.side() == TeamSide::Player)
                    && combat
                        .ability(*ability)
                        .and_then(|definition| definition.action())
                        .is_some_and(|action| action.tags().contains(starclock_combat::catalog::action::AbilityTag::Attack))
            )
        })
        .expect("a character attack is legal")
        .clone();
    let resolution = battle.apply(attack).unwrap();
    assert!(resolution.fault().is_none(), "{:#?}", resolution.events());
    assert_eq!(synapse_stacks(&battle, enemy), Some(14));
}

#[test]
fn formations_author_exact_melt_chain_and_appearance_energy_values() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    for (key, expected, event) in [
        (
            "StageAbility_612821",
            500_000,
            RuleEventPoint::ActionResolved,
        ),
        ("StageAbility_612822", 300_000, RuleEventPoint::UnitDefeated),
        ("StageAbility_612823", 50_000, RuleEventPoint::BattleStarted),
    ] {
        let runtime = combat
            .rule(binding(&contributions, key).rule())
            .unwrap()
            .runtime()
            .unwrap();
        assert!(runtime.triggers().iter().any(|trigger| {
            trigger.event_point == event
                && combat
                    .program(trigger.program)
                    .is_some_and(|program| expression_in_steps(program.steps(), expected))
        }));
    }
    let chain = combat
        .rule(binding(&contributions, "StageAbility_612822").rule())
        .unwrap()
        .runtime()
        .unwrap();
    let chain_program = combat.program(chain.triggers()[0].program).unwrap();
    assert_eq!(
        chain_program
            .steps()
            .iter()
            .filter(|step| {
                matches!(
                    step,
                    starclock_combat::rule::model::ProgramStep::Operation(
                        RuleOperationTemplate::UltimateDamageFromActorBasicElement { .. }
                    )
                )
            })
            .count(),
        2
    );

    let memetic = combat
        .rule(binding(&contributions, "StageAbility_612823").rule())
        .unwrap()
        .runtime()
        .unwrap();
    for event in [
        RuleEventPoint::BattleStarted,
        RuleEventPoint::WaveStarted,
        RuleEventPoint::UnitSummoned,
    ] {
        assert!(
            memetic
                .triggers()
                .iter()
                .any(|trigger| trigger.event_point == event),
            "Memetic Inversion observes {event:?}"
        );
    }

    let (battle, started) = start(
        &materialization,
        durable_spec(&materialization, 0xe6, false),
        0xe7,
    );
    assert!(started.fault().is_none(), "{:#?}", started.events());
    let combined_maximum_energy = battle
        .view()
        .units_by_id()
        .filter(|unit| unit.side() == TeamSide::Player)
        .map(|unit| unit.maximum_energy().scaled())
        .sum::<i64>();
    let expected = u16::try_from(combined_maximum_energy / 1_000_000 * 5 / 100)
        .expect("fixture Energy is integral and bounded");
    assert_eq!(
        battle
            .view()
            .team(TeamSide::Player)
            .keyed_resource(
                starclock_combat::SourceDefinitionId::new(RESONANCE_RESOURCE_RAW).unwrap(),
            )
            .unwrap()
            .0,
        expected,
        "battle entry grants exactly 5% of combined maximum Energy per enemy"
    );
}

fn full_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many_with_formations(
        catalog,
        "universe.path.erudition",
        &[ULTIMATE_HEALING, LETHAL_ENERGY_HEALING],
        &[MELT_CORE, CHAIN_CONTAGION, MEMETIC_INVERSION],
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

fn use_resonance(battle: &mut Battle) -> starclock_combat::Resolution {
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseAbility { ability, .. } | Command::UseInterrupt { ability, .. }
                    if ability.get() == RESONANCE_ABILITY_RAW
            )
        })
        .expect("charged resonance is legal")
        .clone();
    battle.apply(command).unwrap()
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

fn synapse_stacks(battle: &Battle, target: starclock_combat::UnitId) -> Option<u16> {
    battle
        .view()
        .effects_by_id()
        .find(|state| state.target() == target && state.definition() == SYNAPSE_EFFECT)
        .map(|state| state.stacks())
}

fn expression_in_steps(
    steps: &[starclock_combat::rule::model::ProgramStep],
    expected: i64,
) -> bool {
    steps.iter().any(|step| match step {
        starclock_combat::rule::model::ProgramStep::Operation(
            RuleOperationTemplate::UltimateDamageFromActorBasicElement { amount, .. }
            | RuleOperationTemplate::ModifyResource { amount, .. }
            | RuleOperationTemplate::Heal { amount, .. },
        ) => expression_has_scalar(amount, expected),
        _ => false,
    })
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
        ValueExpr::SelectorSum { value, .. } => expression_has_scalar(value, expected),
        _ => false,
    }
}
