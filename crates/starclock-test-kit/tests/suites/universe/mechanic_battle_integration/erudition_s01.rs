use super::*;
use starclock_combat::{
    EffectDefinitionId, ResourceEventData,
    modifier::model::{FormulaPurpose, FormulaStage, FormulaSubject, StatKind},
    rule::model::{
        EventValueProperty, ProgramStep, RuleEventPoint, RuleOperationTemplate, ValueExpr,
    },
};

const GRAY: (&str, u32) = ("universe.blessing.612830", 2);
const AMYGDALA: (&str, u32) = ("universe.blessing.612831", 2);
const OCCIPITAL: (&str, u32) = ("universe.blessing.612832", 2);
const VESTIBULAR: (&str, u32) = ("universe.blessing.612840", 2);
const TRANSMITTER: (&str, u32) = ("universe.blessing.612841", 2);
const MEMORY: (&str, u32) = ("universe.blessing.612842", 2);
const BRAIN_RAW: u32 = 0x7d00_0001;
const BRAIN_ULTIMATE_RAW: u32 = 0x7d00_0002;
const LOCAL_EFFECT_BASE: u32 = 0x7d30_0000;

#[test]
fn goal07_p2_m10_s01_materializes_all_six_blessings_as_generic_rule_ir() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    for key in [
        "StageAbility_612830",
        "StageAbility_612831",
        "StageAbility_612832",
        "StageAbility_612840",
        "StageAbility_612841",
        "StageAbility_612842",
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

    let brain = combat
        .effect(EffectDefinitionId::new(BRAIN_RAW).unwrap())
        .expect("shared Brain in a Vat charge");
    assert_eq!(brain.runtime_template().unwrap().stack_limit(), 1_000);
    let first = combat
        .rule(binding(&contributions, "StageAbility_612830").rule())
        .unwrap()
        .runtime()
        .unwrap();
    assert!(first.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::ActionResolved
            && trigger.filter.action_kind
                == Some(starclock_combat::rule::model::RuleActionKind::Ultimate)
    }));
}

#[test]
fn gray_matter_grants_full_entry_charge_and_one_non_recursive_extra_ultimate() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let roster = roster_for_forms_with_ability_kinds(
        &catalog,
        [19, 2, 3, 4],
        None,
        &[AbilityKind::Ultimate],
        true,
    );
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let (mut battle, started) = start(
        &materialization,
        durable_spec(&materialization, 0xc1, false),
        0xc2,
    );
    assert!(started.fault().is_none(), "{:#?}", started.events());
    let brain = EffectDefinitionId::new(BRAIN_RAW).unwrap();
    assert_eq!(
        battle
            .view()
            .effects_by_id()
            .filter(|effect| effect.definition() == brain && effect.stacks() == 1_000)
            .count(),
        4
    );

    let first = ultimate_command(&battle, None);
    let (actor, ability) = command_actor_ability(&first);
    let first_resolution = apply_action_command(&mut battle, first);
    assert!(
        first_resolution.fault().is_none(),
        "{:#?}",
        first_resolution.events()
    );
    let owner = battle
        .view()
        .units_by_id()
        .find(|unit| unit.id() == actor)
        .unwrap();
    assert_eq!(owner.current_energy(), owner.maximum_energy());
    assert!(has_effect(
        &battle,
        actor,
        EffectDefinitionId::new(BRAIN_ULTIMATE_RAW).unwrap()
    ));
    assert!(!has_effect(&battle, actor, brain));

    let second = ultimate_command(&battle, Some((actor, ability)));
    let second_resolution = apply_action_command(&mut battle, second);
    assert!(
        second_resolution.fault().is_none(),
        "{:#?}",
        second_resolution.events()
    );
    let owner = battle
        .view()
        .units_by_id()
        .find(|unit| unit.id() == actor)
        .unwrap();
    assert!(owner.current_energy() < owner.maximum_energy());
    assert!(!has_effect(
        &battle,
        actor,
        EffectDefinitionId::new(BRAIN_ULTIMATE_RAW).unwrap()
    ));
    assert!(!has_effect(&battle, actor, brain));

    let vestibular_effect = local_effect(&contributions, "StageAbility_612840");
    assert!(
        has_effect(&battle, actor, vestibular_effect),
        "enhanced Vestibular System persists through the Brain-powered Ultimate"
    );
    let memory_effect = local_effect(&contributions, "StageAbility_612842");
    let shield = battle
        .view()
        .shields_by_id()
        .find(|shield| shield.owner() == actor && shield.source_effect() == Some(memory_effect))
        .expect("enhanced Explicit Memory shield");
    assert_eq!(shield.remaining().get(), 45_000);
}

#[test]
fn transmitter_converts_actual_energy_overflow_to_exact_brain_charge() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.erudition",
        &[
            ("universe.blessing.612841", 1),
            ("universe.blessing.612843", 1),
            ("universe.blessing.612844", 1),
        ],
        None,
        false,
    );
    let roster = roster_for_forms_with_ability_kinds(&catalog, [8, 2, 3, 4], None, &[], true);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let (mut battle, started) = start(
        &materialization,
        durable_spec(&materialization, 0xc3, false),
        0xc4,
    );
    assert!(started.fault().is_none(), "{:#?}", started.events());
    while battle.view().phase() == starclock_combat::BattlePhase::ReadyToAdvance {
        battle.advance().unwrap();
    }
    let resolution = first_normal_action(&mut battle);
    assert!(resolution.fault().is_none(), "{:#?}", resolution.events());
    let (unit, overflow) = resolution
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Resource(ResourceEventData::Energy { unit, overflow, .. })
                if *overflow > Energy::ZERO =>
            {
                Some((*unit, *overflow))
            }
            _ => None,
        })
        .expect("a full-energy Basic Attack reports overflow");
    let brain_stacks = battle
        .view()
        .effects_by_id()
        .find(|effect| {
            effect.target() == unit
                && effect.definition() == EffectDefinitionId::new(BRAIN_RAW).unwrap()
        })
        .expect("Transmitter creates Brain charge")
        .stacks();
    assert_eq!(
        i64::from(brain_stacks),
        overflow.scaled() * 8 / 1_000_000,
        "released 0.8% charge per one overflow Energy"
    );
}

#[test]
fn occipital_and_transmitter_author_exact_formula_inputs() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    let occipital_effect = local_effect(&contributions, "StageAbility_612832");
    let modifiers = combat
        .effect(occipital_effect)
        .expect("Occipital Lobe state")
        .modifiers()
        .iter()
        .map(|id| combat.modifier(*id).unwrap())
        .collect::<Vec<_>>();
    assert!(modifiers.iter().any(|modifier| {
        modifier.stat == StatKind::Atk
            && modifier.stage == FormulaStage::Resistance
            && modifier.purpose == FormulaPurpose::OrdinaryDamage
            && modifier.filters.contains(
                &starclock_combat::modifier::model::ModifierFilter::FormulaSubject(
                    FormulaSubject::Source,
                ),
            )
            && expression_has_scalar(&modifier.value, 250_000)
    }));
    assert!(modifiers.iter().any(|modifier| {
        modifier.source_stack_slot.is_some() && expression_has_scalar(&modifier.value, 30_000)
    }));

    let transmitter = combat
        .rule(binding(&contributions, "StageAbility_612841").rule())
        .unwrap();
    assert!(transmitter.programs().iter().any(|program| {
        combat
            .program(*program)
            .unwrap()
            .steps()
            .iter()
            .any(|step| {
                matches!(
                        step,
                        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect { stacks, .. })
                            if expression_has_event(stacks, EventValueProperty::ResourceOverflow)
                                && expression_has_scalar(stacks, 12_000)
                )
            })
    }));

    let vestibular = combat
        .effect(local_effect(&contributions, "StageAbility_612840"))
        .expect("Vestibular System state");
    let modifier = combat.modifier(vestibular.modifiers()[0]).unwrap();
    assert_eq!(
        (modifier.stat, modifier.stage, modifier.purpose),
        (
            StatKind::CritDamage,
            FormulaStage::Flat,
            FormulaPurpose::Stat
        )
    );
    assert!(expression_has_scalar(&modifier.value, 900_000));
}

fn full_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many(
        catalog,
        "universe.path.erudition",
        &[GRAY, AMYGDALA, OCCIPITAL, VESTIBULAR, TRANSMITTER, MEMORY],
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

fn local_effect(contributions: &UniverseBattleContributionSet, key: &str) -> EffectDefinitionId {
    let raw = binding(contributions, key).rule().get();
    EffectDefinitionId::new(LOCAL_EFFECT_BASE + (raw & 0xffff) * 16).unwrap()
}

fn ultimate_command(
    battle: &Battle,
    exact: Option<(starclock_combat::UnitId, AbilityId)>,
) -> Command {
    let option = battle
        .available_ultimates()
        .into_iter()
        .find(|option| {
            exact.is_none_or(|(actor, ability)| {
                option.actor() == actor && option.ability() == ability
            })
        })
        .expect("requested Ultimate is legal");
    battle.request_ultimate_command(option).unwrap()
}

fn command_actor_ability(command: &Command) -> (starclock_combat::UnitId, AbilityId) {
    match command {
        Command::RequestUltimate { actor, ability, .. } => (*actor, *ability),
        _ => panic!("Ultimate command"),
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
        ValueExpr::Literal(starclock_combat::rule::model::RuleValue::Scalar(value)) => {
            value.scaled() == expected
        }
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

fn expression_has_event(value: &ValueExpr, expected: EventValueProperty) -> bool {
    match value {
        ValueExpr::ReadEventProperty(property) => *property == expected,
        ValueExpr::Multiply { lhs, rhs, .. }
        | ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Divide { lhs, rhs, .. }
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => {
            expression_has_event(lhs, expected) || expression_has_event(rhs, expected)
        }
        ValueExpr::Negate(value) | ValueExpr::Convert { value, .. } => {
            expression_has_event(value, expected)
        }
        _ => false,
    }
}
