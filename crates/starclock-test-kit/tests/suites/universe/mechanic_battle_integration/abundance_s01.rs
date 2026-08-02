use super::*;
use starclock_combat::{
    ParticipantInitialState,
    catalog::selector::{RuleSelectorChoice, RuleSelectorOrigin},
    formula::model::DamageClass,
    modifier::model::{FormulaStage, StatKind},
    rule::model::{ProgramStep, RuleOperationTemplate},
};

const HEALING_DEWDROP: (&str, u32) = ("universe.blessing.612330", 2);
const TURN_DEWDROP: (&str, u32) = ("universe.blessing.612331", 2);
const SHARED_HEALING: (&str, u32) = ("universe.blessing.612332", 2);
const RUPTURE_HEALING: (&str, u32) = ("universe.blessing.612340", 2);
const FULL_HP_EFFICIENCY: (&str, u32) = ("universe.blessing.612341", 2);
const BAILU_FORM: u32 = 10;

#[test]
fn goal07_p2_m05_s01_materializes_all_five_assigned_mechanics() {
    let catalog = catalog();
    let contributions = all_contributions(&catalog);
    let roster = bailu_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let combat = materialization.combat_catalog();
    for key in [
        "StageAbility_612330",
        "StageAbility_612331",
        "StageAbility_612332",
        "StageAbility_612340",
        "StageAbility_612341",
    ] {
        let binding = contributions
            .rules()
            .iter()
            .find(|binding| binding.source_binding_key() == Some(key))
            .expect("assigned Blessing level is selected");
        assert!(
            combat.rule(binding.rule()).is_some(),
            "{key} is an executable battle rule"
        );
    }

    let shared = binding(&contributions, "StageAbility_612332");
    let effect = starclock_combat::EffectDefinitionId::new(0x7660_0000 + shared.rule().get())
        .expect("reserved effect ID");
    let runtime = combat
        .effect(effect)
        .expect("healing-derived ATK effect")
        .runtime_template()
        .expect("effect is executable");
    assert!(matches!(
        runtime.duration_expression(),
        Some(starclock_combat::rule::model::ValueExpr::Literal(
            starclock_combat::rule::model::RuleValue::Integer(2)
        ))
    ));
    let modifier = combat
        .modifier(combat.effect(effect).unwrap().modifiers()[0])
        .unwrap();
    assert_eq!(
        (modifier.stat, modifier.stage),
        (StatKind::Atk, FormulaStage::Flat)
    );

    let host = binding(&contributions, "StageAbility_612330");
    let rupture_target = combat
        .rule(host.rule())
        .unwrap()
        .programs()
        .iter()
        .filter_map(|program| combat.program(*program))
        .flat_map(|program| program.steps())
        .find_map(|step| {
            if let ProgramStep::Operation(RuleOperationTemplate::DamageFromEventElement {
                selector,
                class: DamageClass::Additional,
                ..
            }) = step
            {
                Some(*selector)
            } else {
                None
            }
        });
    assert!(
        rupture_target.is_some(),
        "Dewdrop rupture inherits the triggering attack element"
    );
    let rupture_target = combat
        .selector(rupture_target.unwrap())
        .unwrap()
        .rule_units()
        .unwrap();
    assert_eq!(rupture_target.origin(), RuleSelectorOrigin::EventTargets);
    assert_eq!(rupture_target.choice(), RuleSelectorChoice::RngUniform);
    assert_eq!(rupture_target.rng_purpose(), Some("bounce-target"));
}

#[test]
fn turn_dewdrop_charges_ruptures_and_heals_with_exact_level_two_bounds() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.abundance",
        &[TURN_DEWDROP, RUPTURE_HEALING, SHARED_HEALING],
        None,
        false,
    );
    let roster = bailu_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let spec = wounded_players(durable_spec(&materialization, 0x91, false), 50_000);
    let (mut battle, started) = start(&materialization, spec, 0x92);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    enter_normal_turn(&mut battle);

    let host = binding(&contributions, "StageAbility_612331");
    let slot = starclock_combat::StateSlotDefinitionId::new(0x7670_0000 + host.rule().get())
        .expect("reserved Dewdrop slot");
    let charged = battle
        .view()
        .rule_instances_by_id()
        .find_map(|instance| {
            (instance.rule() == host.rule())
                .then(|| instance.slots().find(|(id, _)| *id == slot))
                .flatten()
                .map(|(_, value)| (instance.owner(), value.clone()))
        })
        .expect("the first acting character charged Dewdrop");
    assert!(matches!(
        charged.1,
        starclock_combat::rule::model::RuleValue::Scalar(value)
            if value.scaled() == 70_000_000_000
    ));

    let hp_before = battle
        .view()
        .units_by_id()
        .find(|unit| unit.id() == charged.0.expect("player rule has owner"))
        .unwrap()
        .current_hp();
    let resolution = first_normal_action(&mut battle);
    assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    let additional = resolution
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Damage(data) if data.class == DamageClass::Additional => Some(data),
            _ => None,
        });
    assert!(additional.is_some(), "{:?}", resolution.events());
    let hp_after = battle
        .view()
        .units_by_id()
        .find(|unit| unit.id() == charged.0.unwrap())
        .unwrap()
        .current_hp();
    assert_eq!(
        hp_after.get() - hp_before.get(),
        14_000,
        "20% of 70% Max HP is inside the authored 12%-24% bounds"
    );
    assert!(matches!(
        battle
            .view()
            .rule_instances_by_id()
            .find(|instance| instance.rule() == host.rule() && instance.owner() == charged.0)
            .unwrap()
            .slots()
            .find(|(id, _)| *id == slot)
            .unwrap()
            .1,
        starclock_combat::rule::model::RuleValue::Scalar(value) if value.scaled() == 0
    ));
}

#[test]
fn full_hp_efficiency_caps_turn_charge_at_maximum_hp() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.abundance",
        &[TURN_DEWDROP, FULL_HP_EFFICIENCY, SHARED_HEALING],
        None,
        false,
    );
    let roster = bailu_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let (mut battle, started) = start(
        &materialization,
        durable_spec(&materialization, 0x93, false),
        0x94,
    );
    assert!(started.fault().is_none(), "{:?}", started.fault());
    enter_normal_turn(&mut battle);
    let host = binding(&contributions, "StageAbility_612331");
    let slot =
        starclock_combat::StateSlotDefinitionId::new(0x7670_0000 + host.rule().get()).unwrap();
    let charge = battle
        .view()
        .rule_instances_by_id()
        .filter(|instance| instance.rule() == host.rule())
        .flat_map(|instance| instance.slots())
        .find(|(id, _)| *id == slot)
        .unwrap()
        .1;
    assert!(matches!(
        charge,
        starclock_combat::rule::model::RuleValue::Scalar(value)
            if value.scaled() == 100_000_000_000
    ));
}

#[test]
fn all_abundance_shares_effective_healing_without_recursion_and_stacks_attack() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.abundance",
        &[TURN_DEWDROP, RUPTURE_HEALING, SHARED_HEALING],
        None,
        false,
    );
    let roster = bailu_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let spec = wounded_players(durable_spec(&materialization, 0x95, false), 20_000);
    let (mut battle, started) = start(&materialization, spec, 0x96);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    enter_normal_turn(&mut battle);
    let binding = binding(&contributions, "StageAbility_612332");
    let resolution = first_normal_action(&mut battle);
    assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    let source = binding.source().definition();
    let shared_heals = resolution
        .events()
        .iter()
        .filter_map(|event| {
            (event.cause().source_definition() == Some(source))
                .then(|| match event.kind() {
                    BattleEventKind::Heal(data) => Some(data),
                    _ => None,
                })
                .flatten()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        shared_heals.len(),
        3,
        "one rupture heal is shared once with the other three allies: {:?}",
        resolution.events(),
    );
    assert!(
        shared_heals.iter().all(|heal| {
            heal.calculated.get() == 4_200
                && heal.effective.get() == 4_200
                && heal.overheal.get() == 0
        }),
        "already-resolved 14,000 healing is restored at exactly 30% without a second multiplier: {shared_heals:?}"
    );
    let effect =
        starclock_combat::EffectDefinitionId::new(0x7660_0000 + binding.rule().get()).unwrap();
    let buffed = battle
        .view()
        .effects_by_id()
        .filter(|state| state.definition() == effect && state.stacks() > 0)
        .count();
    assert_eq!(
        buffed, 4,
        "every ally receives the healing-derived ATK buff"
    );
}

fn all_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many(
        catalog,
        "universe.path.abundance",
        &[
            HEALING_DEWDROP,
            TURN_DEWDROP,
            SHARED_HEALING,
            RUPTURE_HEALING,
            FULL_HP_EFFICIENCY,
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
        .unwrap()
}

fn bailu_roster(catalog: &UniverseCatalog) -> UniverseBattleRoster {
    roster_for_forms(catalog, [BAILU_FORM, 1, 2, 3], None)
}

fn wounded_players(original: BattleSpec, current_hp: i64) -> BattleSpec {
    let participants = original
        .participants()
        .iter()
        .map(|participant| {
            if participant.side() != TeamSide::Player {
                return participant.clone();
            }
            let combatant = participant.combatant();
            let initial = ParticipantInitialState::new(
                Hp::new(current_hp).unwrap(),
                combatant.maximum_hp(),
                combatant.current_energy(),
                combatant.maximum_energy(),
                starclock_combat::LifeState::Alive,
                starclock_combat::PresenceState::Present,
            )
            .unwrap();
            participant
                .clone()
                .with_initial_state(initial)
                .expect("wounded carry is valid")
        })
        .collect();
    BattleSpec::new(
        AssemblyDigest::new([0x97; 32]).unwrap(),
        original.encounter(),
        participants,
        original.resources(TeamSide::Player).clone(),
        original.resources(TeamSide::Enemy).clone(),
        original.concede_policy(),
    )
    .unwrap()
}

fn enter_normal_turn(battle: &mut Battle) {
    if battle
        .decision()
        .is_some_and(|decision| decision.kind() == starclock_combat::DecisionKind::InterruptWindow)
    {
        let resolution = battle
            .apply(Command::PassInterruptWindow {
                decision: battle.decision().unwrap().id(),
            })
            .unwrap();
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    }
}
