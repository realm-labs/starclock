use super::*;
use starclock_combat::{
    ModifierDefinitionId, ParticipantInitialState,
    modifier::model::{FormulaPurpose, FormulaStage, ModifierDefinition, ModifierFilter, StatKind},
    rule::model::{ProgramStep, RuleOperationTemplate, RuleValue, ValueExpr},
};
use super::{nihility_s02};

const RANDOM: (&str, u32) = ("universe.blessing.612630", 2);
const CHAMPION: (&str, u32) = ("universe.blessing.612632", 2);
const HOURGLASS: (&str, u32) = ("universe.blessing.612642", 2);
const EXEMPLARY: (&str, u32) = ("universe.blessing.612650", 2);
const MOSTLY: (&str, u32) = ("universe.blessing.612651", 2);
const SUSPIRIA: (&str, u32) = ("universe.blessing.612652", 2);
const PALE_FIRE: (&str, u32) = ("universe.blessing.612653", 2);
const LIGHTHOUSE: (&str, u32) = ("universe.blessing.612654", 2);
const DOCTOR: (&str, u32) = ("universe.blessing.612655", 2);

#[test]
fn goal07_p2_m08_s03_materializes_all_levels_without_native_handlers() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize_with_roster(
        &catalog,
        &nihility_s02::kafka_roster(&catalog),
        &contributions,
    );
    for key in [
        "StageAbility_612650",
        "StageAbility_612651",
        "StageAbility_612652",
        "StageAbility_612653",
        "StageAbility_612654",
        "StageAbility_612655",
    ] {
        let rule = materialization
            .combat_catalog()
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
fn enhanced_exemplary_and_four_follow_up_modifiers_keep_exact_values_and_tags() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    let raw = binding(&contributions, "StageAbility_612650").rule().get();
    let exemplary = [0x76c0_0000, 0x79e7_0000, 0x79ec_0000]
        .into_iter()
        .map(|base| {
            combat
                .modifier(ModifierDefinitionId::new(base + raw).unwrap())
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(exemplary.len(), 3);
    assert!(exemplary.iter().all(|modifier| {
        modifier.stage == FormulaStage::DamageBoost
            && modifier.purpose == FormulaPurpose::OrdinaryDamage
            && expression_has_scalar(&modifier.value, 1_080_000)
    }));

    for (key, stat, stage, purpose, value) in [
        (
            "StageAbility_612651",
            StatKind::ToughnessDamage,
            FormulaStage::Flat,
            FormulaPurpose::Break,
            500_000,
        ),
        (
            "StageAbility_612652",
            StatKind::Atk,
            FormulaStage::DamageBoost,
            FormulaPurpose::OrdinaryDamage,
            390_000,
        ),
        (
            "StageAbility_612653",
            StatKind::CritRate,
            FormulaStage::Flat,
            FormulaPurpose::Stat,
            390_000,
        ),
        (
            "StageAbility_612654",
            StatKind::EnergyRegenerationRate,
            FormulaStage::Flat,
            FormulaPurpose::Stat,
            360_000,
        ),
    ] {
        let modifiers = modifiers(combat, binding(&contributions, key).rule());
        assert_eq!(
            modifiers.len(),
            3,
            "{key} has one modifier per eligible tag"
        );
        assert!(modifiers.iter().all(|modifier| {
            modifier.stat == stat
                && modifier.stage == stage
                && modifier.purpose == purpose
                && expression_has_scalar(&modifier.value, value)
        }));
        let tags = modifiers
            .iter()
            .flat_map(|modifier| modifier.filters.iter())
            .filter_map(|filter| match filter {
                ModifierFilter::AbilityTag(tag) => Some(tag.as_ref()),
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            tags,
            ["counter", "follow_up", "ultimate"].into_iter().collect()
        );
    }
}

#[test]
fn doctor_heals_and_lighthouse_scales_production_ultimate_energy_at_action_boundary() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.elation",
        &[CHAMPION, LIGHTHOUSE, DOCTOR],
        None,
        false,
    );
    let roster = nihility_s02::kafka_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let spec = wounded_players(
        durable_spec_with_two_enemy_hp(
            &materialization,
            0xee,
            [
                Hp::new(2_000_000_000).unwrap(),
                Hp::new(2_000_000_000).unwrap(),
            ],
        ),
        1,
        0xef,
    );
    let (mut battle, started) = start(&materialization, spec, 0xf0);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let actor = battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Player && unit.form().get() == 45)
        .expect("Kafka")
        .id();
    let maximum_hp = battle
        .view()
        .units_by_id()
        .find(|unit| unit.id() == actor)
        .unwrap()
        .maximum_hp()
        .get();
    let resolution = nihility_s02::use_kafka_ultimate(&mut battle);
    assert!(resolution.fault().is_none(), "{:?}", resolution.fault());

    assert_eq!(
        battle
            .view()
            .units_by_id()
            .find(|unit| unit.id() == actor)
            .unwrap()
            .current_energy()
            .scaled(),
        6_800_000,
        "5 Ultimate Energy × 1.36 Energy Regeneration Rate"
    );
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .find(|unit| unit.id() == actor)
            .unwrap()
            .current_hp()
            .get(),
        1 + maximum_hp * 15 / 100,
        "Doctor of Love restores 15% Max HP once after the complete action"
    );
    let source = binding(&contributions, "StageAbility_612655")
        .source()
        .definition();
    assert_eq!(
        resolution
            .events()
            .iter()
            .filter(|event| {
                event.cause().source_definition() == Some(source)
                    && matches!(event.kind(), BattleEventKind::Heal(_))
            })
            .count(),
        1
    );
}

#[test]
fn doctor_program_is_owner_max_hp_healing_after_one_complete_eligible_action() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let rule = combat
        .rule(binding(&contributions, "StageAbility_612655").rule())
        .unwrap();
    let runtime = rule.runtime().unwrap();
    assert_eq!(runtime.triggers().len(), 3);
    assert!(runtime.triggers().iter().all(|trigger| {
        trigger.event_point == starclock_combat::rule::model::RuleEventPoint::ActionResolved
            && trigger.once_scope == starclock_combat::rule::model::OnceScope::Action
    }));
    assert!(rule.programs().iter().any(|program| {
        combat.program(*program).is_some_and(|program| {
            program.steps().iter().any(|step| {
                matches!(
                    step,
                    ProgramStep::Operation(RuleOperationTemplate::Heal {
                        amount,
                        apply_formula_modifiers: true,
                        ..
                    }) if expression_has_scalar(amount, 150_000)
                )
            })
        })
    }));
}

fn full_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many(
        catalog,
        "universe.path.elation",
        &[
            RANDOM, CHAMPION, HOURGLASS, EXEMPLARY, MOSTLY, SUSPIRIA, PALE_FIRE, LIGHTHOUSE, DOCTOR,
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

fn modifiers(
    combat: &starclock_combat::catalog::CombatCatalog,
    rule: starclock_combat::RuleId,
) -> Vec<&ModifierDefinition> {
    combat
        .rule(rule)
        .unwrap()
        .programs()
        .iter()
        .flat_map(|program| combat.program(*program).unwrap().effects())
        .flat_map(|effect| combat.effect(*effect).unwrap().modifiers())
        .map(|modifier| combat.modifier(*modifier).unwrap())
        .collect()
}

fn expression_has_scalar(expression: &ValueExpr, expected: i64) -> bool {
    match expression {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled() == expected,
        ValueExpr::Negate(value) => expression_has_scalar(value, expected),
        ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => {
            expression_has_scalar(lhs, expected) || expression_has_scalar(rhs, expected)
        }
        ValueExpr::Multiply { lhs, rhs, .. } | ValueExpr::Divide { lhs, rhs, .. } => {
            expression_has_scalar(lhs, expected) || expression_has_scalar(rhs, expected)
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
        AssemblyDigest::new([marker; 32]).unwrap(),
        original.encounter(),
        participants,
        original.resources(TeamSide::Player).clone(),
        original.resources(TeamSide::Enemy).clone(),
        original.concede_policy(),
    )
    .unwrap()
}
