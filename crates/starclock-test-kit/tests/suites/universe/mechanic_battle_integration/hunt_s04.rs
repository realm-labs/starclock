use super::*;
use starclock_combat::{
    catalog::{
        action::HitCritPolicy,
        selector::{RuleSelectorChoice, RuleSelectorOrdering},
    },
    modifier::model::{FormulaStage, SnapshotPolicy, StatKind},
    rule::model::{ProgramStep, RuleEventPoint, RuleOperationTemplate, RuleValue, ValueExpr},
};

const TURN_ENERGY: (&str, u32) = ("universe.blessing.612456", 2);
const LAST_ALLY_ATTACK: (&str, u32) = ("universe.blessing.612457", 2);
const STAR_HUNTER: &str = "universe.resonance.612421";
const BOW_AND_ARROW: &str = "universe.resonance.612422";
const PERFECT_AIM: &str = "universe.resonance.612423";

#[test]
fn goal07_p2_m06_s04_materializes_blessings_and_formations_as_generic_runtime() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    for key in [
        "StageAbility_61245601",
        "StageAbility_61245701",
        "StageAbility_612421",
        "StageAbility_612422",
        "StageAbility_612423",
    ] {
        let rule = combat.rule(binding(&contributions, key).rule()).unwrap();
        assert!(
            rule.runtime()
                .is_some_and(|runtime| runtime.native_handler().is_none()),
            "{key} remains generic Rule IR"
        );
    }
    assert_eq!(
        materialization.difficulty_specs()[0]
            .battle_spec()
            .resources(TeamSide::Player)
            .keyed()[0]
            .maximum(),
        200,
        "Perfect Aim doubles the authored resonance-energy capacity"
    );
}

#[test]
fn arboreal_and_arrow_shades_preserve_exact_values_and_snapshots() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    let arboreal = runtime(combat, &contributions, "StageAbility_61245601");
    let turn = arboreal
        .triggers()
        .iter()
        .find(|trigger| trigger.event_point == RuleEventPoint::TurnStarted)
        .unwrap();
    assert!(matches!(
        operation(combat, turn.program),
        RuleOperationTemplate::ModifyResource { amount, .. }
            if literal_scalar(amount) == 6_000_000
    ));

    let arrow_rule = combat
        .rule(binding(&contributions, "StageAbility_61245701").rule())
        .unwrap();
    let modifier = arrow_rule
        .programs()
        .iter()
        .flat_map(|program| combat.program(*program).unwrap().effects())
        .filter_map(|effect| combat.effect(*effect))
        .flat_map(|effect| effect.modifiers())
        .find_map(|modifier| combat.modifier(*modifier))
        .unwrap();
    assert_eq!(
        (modifier.stat, modifier.stage, modifier.snapshot),
        (
            StatKind::Atk,
            FormulaStage::Flat,
            SnapshotPolicy::OnApplication
        )
    );
    assert!(expression_has_scalar(&modifier.value, 150_000));
    assert_eq!(
        runtime(combat, &contributions, "StageAbility_61245701")
            .state_slots()
            .len(),
        1,
        "one owner-local slot elects the most recent allied actor"
    );
}

#[test]
fn hunt_resonance_uses_highest_attack_and_all_three_formation_values() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let resonance = combat
        .ability(AbilityId::new(RESONANCE_ABILITY_RAW).unwrap())
        .unwrap();
    assert!(matches!(
        resonance.action().unwrap().hits()[0].crit_policy(),
        HitCritPolicy::GuaranteedBelowHpRatio(value) if value.scaled() == 500_000
    ));
    let main = combat.program(resonance.program()).unwrap();
    assert!(main.steps().iter().any(|step| matches!(
        step,
        ProgramStep::Operation(RuleOperationTemplate::Damage { amount, .. })
            if expression_has_scalar(amount, 5_500_000)
    )));
    assert!(
        combat
            .selector(main.selectors()[1])
            .unwrap()
            .rule_units()
            .is_some_and(|selector| {
                selector.ordering() == RuleSelectorOrdering::StatDescending
                    && selector.choice() == RuleSelectorChoice::First
            })
    );

    let star = combat
        .rule(binding(&contributions, "StageAbility_612421").rule())
        .unwrap();
    assert!(star.runtime().unwrap().triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::ActionResolved
            && combat
                .program(trigger.program)
                .unwrap()
                .steps()
                .iter()
                .any(|step| {
                    matches!(
                        step,
                        ProgramStep::Operation(RuleOperationTemplate::GrantExtraTurn { .. })
                    )
                })
    }));
    assert_eq!(
        energy_gain(combat, &contributions, "StageAbility_612422"),
        100_000_000,
        "Bow and Arrow restores 50% of Perfect Aim's 200-point capacity"
    );
    assert_eq!(
        energy_gain(combat, &contributions, "StageAbility_612423"),
        6_000_000,
        "Perfect Aim restores 3% of its 200-point capacity per allied turn"
    );
}

#[test]
fn complete_hunt_resonance_executes_without_fault_and_spends_one_charge() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let original = durable_spec(&materialization, 0xd1, false);
    let spec = with_resonance_energy(original, 100, 200, 0xd2);
    let (mut battle, started) = start(&materialization, spec, 0xd3);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseInterrupt { ability, .. } if ability.get() == RESONANCE_ABILITY_RAW
            )
        })
        .unwrap()
        .clone();
    let resolution = battle.apply(command).unwrap();
    assert!(
        resolution.fault().is_none(),
        "{:?} {:?}",
        resolution.fault(),
        resolution.events()
    );
    assert!(resolution.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Resource(starclock_combat::ResourceEventData::TeamResource {
                resource,
                attempted: 100,
                effective: 100,
                before: 100,
                after: 0,
                ..
            }) if resource.get() == RESONANCE_RESOURCE_RAW
        )
    }));
    assert!(resolution.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Damage(data)
                if data.element
                    == Some(starclock_combat::formula::model::CombatElement::Wind)
        )
    }));
}

fn full_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many_with_formations(
        catalog,
        "universe.path.hunt",
        &[TURN_ENERGY, LAST_ALLY_ATTACK],
        &[STAR_HUNTER, BOW_AND_ARROW, PERFECT_AIM],
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

fn runtime<'a>(
    combat: &'a starclock_combat::catalog::CombatCatalog,
    contributions: &UniverseBattleContributionSet,
    key: &str,
) -> &'a starclock_combat::rule::model::BattleRuleDefinition {
    combat
        .rule(binding(contributions, key).rule())
        .unwrap()
        .runtime()
        .unwrap()
}

fn operation(
    combat: &starclock_combat::catalog::CombatCatalog,
    program: starclock_combat::ProgramId,
) -> &RuleOperationTemplate {
    match &combat.program(program).unwrap().steps()[0] {
        ProgramStep::Operation(operation) => operation,
        _ => panic!("expected operation"),
    }
}

fn energy_gain(
    combat: &starclock_combat::catalog::CombatCatalog,
    contributions: &UniverseBattleContributionSet,
    key: &str,
) -> i64 {
    let trigger = &runtime(combat, contributions, key).triggers()[0];
    match operation(combat, trigger.program) {
        RuleOperationTemplate::ModifyResource { amount, .. } => literal_scalar(amount),
        operation => panic!("expected energy operation, got {operation:?}"),
    }
}

fn literal_scalar(value: &ValueExpr) -> i64 {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled(),
        value => panic!("expected scalar literal, got {value:?}"),
    }
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

fn with_resonance_energy(
    original: BattleSpec,
    initial: u16,
    maximum: u16,
    marker: u8,
) -> BattleSpec {
    let resources = TeamResourceSpec::new(
        original.resources(TeamSide::Player).skill_points(),
        original.resources(TeamSide::Player).maximum_skill_points(),
    )
    .unwrap()
    .with_keyed(vec![
        KeyedTeamResourceSpec::new(
            starclock_combat::SourceDefinitionId::new(RESONANCE_RESOURCE_RAW).unwrap(),
            initial,
            maximum,
            TeamResourceWavePolicy::Persist,
        )
        .unwrap()
        .with_stable_key("standard-universe.path-resonance-energy")
        .unwrap(),
    ])
    .unwrap();
    BattleSpec::new(
        AssemblyDigest::new([marker; 32]).unwrap(),
        original.encounter(),
        original.participants().to_vec(),
        resources,
        original.resources(TeamSide::Enemy).clone(),
        original.concede_policy(),
    )
    .unwrap()
}
