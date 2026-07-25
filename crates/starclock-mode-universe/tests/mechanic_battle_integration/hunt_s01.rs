use super::*;
use starclock_combat::{
    ParticipantInitialState,
    formula::toughness::EnemyRank,
    modifier::model::{FormulaStage, StatKind},
    rule::model::{ConditionExpr, ProgramStep, RuleEventPoint, RuleValue, ValueExpr},
};

const EMPYREAN: (&str, u32) = ("universe.blessing.612430", 2);
const RADIANT: (&str, u32) = ("universe.blessing.612431", 2);
const SKYBREAKER: (&str, u32) = ("universe.blessing.612432", 2);
const VENDETTA: (&str, u32) = ("universe.blessing.612440", 2);
const ARCHERY: (&str, u32) = ("universe.blessing.612441", 2);
const CRITICAL_BOOST_RAW: u32 = 0x7940_0001;

#[test]
fn goal07_p2_m06_s01_materializes_every_selected_level_as_generic_rule_ir() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.hunt",
        &[EMPYREAN, RADIANT, SKYBREAKER, VENDETTA, ARCHERY],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    for key in [
        "StageAbility_61243002",
        "StageAbility_61243101",
        "StageAbility_61243201",
        "StageAbility_61244002",
        "StageAbility_61244102",
    ] {
        let binding = binding(&contributions, key);
        let rule = combat
            .rule(binding.rule())
            .unwrap_or_else(|| panic!("{key} is executable"));
        assert!(
            rule.runtime()
                .is_some_and(|runtime| runtime.native_handler().is_none()),
            "{key} remains generic Rule IR"
        );
    }

    let critical = combat
        .effect(
            starclock_combat::EffectDefinitionId::new(CRITICAL_BOOST_RAW)
                .expect("reserved Critical Boost ID"),
        )
        .expect("shared Critical Boost definition");
    assert_eq!(
        critical.runtime_template().unwrap().stack_limit(),
        12,
        "enhanced Empyrean Imperium raises the shared cap"
    );
    let stats = critical
        .modifiers()
        .iter()
        .map(|modifier| combat.modifier(*modifier).unwrap())
        .map(|modifier| (modifier.stat, modifier.stage))
        .collect::<Vec<_>>();
    assert!(stats.contains(&(StatKind::CritRate, FormulaStage::Flat)));
    assert_eq!(
        stats
            .iter()
            .filter(|entry| **entry == (StatKind::CritDamage, FormulaStage::Flat))
            .count(),
        2,
        "Critical Boost and enhanced Skyward Vendetta both contribute CRIT DMG"
    );
}

#[test]
fn enhanced_empyrean_and_archery_execute_on_the_first_production_turn() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.hunt",
        &[EMPYREAN, ARCHERY],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let spec = wounded_players(durable_spec(&materialization, 0x81, false), 50_000, 0x82);
    let (battle, started) = start(&materialization, spec, 0x83);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let critical = battle
        .view()
        .effects_by_id()
        .find(|effect| effect.definition().get() == CRITICAL_BOOST_RAW)
        .expect("turn start grants Critical Boost");
    assert_eq!(critical.stacks(), 1);
    let actor = battle
        .view()
        .units_by_id()
        .find(|unit| unit.id() == critical.target())
        .unwrap();
    assert_eq!(
        actor.current_hp().get(),
        55_000,
        "Archery Duel heals 5% MaxHP for the one current Critical Boost stack"
    );
}

#[test]
fn enhanced_radiant_advances_the_killer_and_grants_seven_stacks_next_turn() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.hunt",
        &[RADIANT, SKYBREAKER, VENDETTA],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let spec = durable_spec_with_two_enemy_hp(
        &materialization,
        0x84,
        [Hp::new(1).unwrap(), Hp::new(2_000_000_000).unwrap()],
    );
    let (mut battle, started) = start(&materialization, spec, 0x85);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = first_normal_action(&mut battle);
    assert!(
        resolution.fault().is_none(),
        "{:?} {:?}",
        resolution.fault(),
        resolution.events()
    );
    assert!(
        resolution.events().iter().any(|event| matches!(
            event.kind(),
            BattleEventKind::Unit(starclock_combat::UnitEventData::Defeated { .. })
        )),
        "the production action defeats the one-HP target"
    );
    assert!(
        battle.view().effects_by_id().any(|effect| {
            effect.definition().get() == CRITICAL_BOOST_RAW && effect.stacks() == 7
        }),
        "the advanced next turn grants seven Critical Boost stacks"
    );
}

#[test]
fn enhanced_skybreaker_authors_elite_team_advance_and_next_attack_timing() {
    let catalog = catalog();
    let contributions =
        contributions_many(&catalog, "universe.path.hunt", &[SKYBREAKER], None, false);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let rule = combat
        .rule(binding(&contributions, "StageAbility_61243201").rule())
        .unwrap();
    let runtime = rule.runtime().unwrap();
    assert!(runtime.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::WeaknessBroken
            && program_has_elite_rank(combat, trigger.program)
    }));
    assert!(runtime.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::ActionResolved
            && trigger.filter.ability_tag
                == Some(starclock_combat::catalog::action::AbilityTag::Attack)
    }));
    assert!(runtime.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::ActionStarted
            && trigger.filter.ability_tag
                == Some(starclock_combat::catalog::action::AbilityTag::Attack)
    }));
    let damage_modifier = rule
        .programs()
        .iter()
        .flat_map(|program| combat.program(*program).unwrap().effects())
        .filter_map(|effect| combat.effect(*effect))
        .flat_map(|effect| effect.modifiers())
        .filter_map(|modifier| combat.modifier(*modifier))
        .find(|modifier| modifier.stage == FormulaStage::DamageBoost)
        .expect("next-attack damage effect");
    assert_eq!(literal_scalar(&damage_modifier.value), 750_000);
}

#[test]
fn enhanced_vendetta_clamps_conversion_to_two_hundred_percent() {
    let catalog = catalog();
    let contributions =
        contributions_many(&catalog, "universe.path.hunt", &[VENDETTA], None, false);
    let materialization = materialize(&catalog, &contributions);
    let critical = materialization
        .combat_catalog()
        .effect(starclock_combat::EffectDefinitionId::new(CRITICAL_BOOST_RAW).unwrap())
        .unwrap();
    let modifier = critical
        .modifiers()
        .iter()
        .filter_map(|id| materialization.combat_catalog().modifier(*id))
        .find(|modifier| {
            modifier.stat == StatKind::CritDamage
                && expression_has_scalar(&modifier.value, 2_000_000)
        })
        .expect("enhanced overflow conversion modifier");
    assert!(
        expression_has_scalar(&modifier.value, 30_000)
            && expression_has_scalar(&modifier.value, 2_000)
            && expression_has_scalar(&modifier.value, 10_000),
        "expression preserves 3% per excess 1% plus 0.2% per stack"
    );
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

fn program_has_elite_rank(
    combat: &starclock_combat::catalog::CombatCatalog,
    program: starclock_combat::ProgramId,
) -> bool {
    combat.program(program).unwrap().steps().iter().any(|step| {
        matches!(
            step,
            ProgramStep::If {
                condition: ConditionExpr::EnemyRank(_, EnemyRank::EliteOrBoss),
                ..
            }
        )
    })
}

fn literal_scalar(value: &ValueExpr) -> i64 {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled(),
        _ => panic!("expected scalar literal"),
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
        ValueExpr::Clamp {
            value,
            minimum,
            maximum,
        } => {
            expression_has_scalar(value, expected)
                || expression_has_scalar(minimum, expected)
                || expression_has_scalar(maximum, expected)
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
