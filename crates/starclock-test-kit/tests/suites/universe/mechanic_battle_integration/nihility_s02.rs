use super::*;
use starclock_combat::{
    RawToughness, ToughnessLayerSpec,
    formula::model::CombatElement,
    modifier::model::StatKind,
    rule::model::{ProgramStep, RuleDotSelection, RuleOperationTemplate, RuleValue, ValueExpr},
};

const NIGHT: (&str, u32) = ("universe.blessing.612243", 2);
const HELL: (&str, u32) = ("universe.blessing.612244", 2);
const TWILIGHT: (&str, u32) = ("universe.blessing.612245", 2);
const ALL_THINGS: (&str, u32) = ("universe.blessing.612246", 2);
const IGNOSTICISM: (&str, u32) = ("universe.blessing.612250", 1);
const KAFKA_FORM: u32 = 45;
const KAFKA_ULTIMATE: u32 = 20_033;
const TWILIGHT_EFFECTS: [u32; 4] = [0x77f1_0001, 0x77f1_0002, 0x77f1_0003, 0x77f1_0004];
const WILDERNESS_ATTACK_MODIFIER: u32 = 0x77e0_000d;

#[test]
fn goal07_p2_m04_s02_materializes_all_five_mechanics_with_exact_policies() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.nihility",
        &[NIGHT, HELL, TWILIGHT, ALL_THINGS, IGNOSTICISM],
        None,
        false,
    );
    for key in [
        "StageAbility_612243",
        "StageAbility_612244",
        "StageAbility_612245",
        "StageAbility_612246",
        "StageAbility_612250",
    ] {
        assert!(
            contributions
                .rules()
                .iter()
                .any(|rule| rule.source_binding_key() == Some(key)),
            "{key} contribution is selected"
        );
    }
    let roster = kafka_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let combat = materialization.combat_catalog();

    let night = binding(&contributions, "StageAbility_612243");
    let night_modifier = first_effect_modifier(combat, night.rule());
    assert_eq!(night_modifier.stat, StatKind::ToughnessDamage);
    assert_eq!(literal_scalar(&night_modifier.value), 450_000);

    let hell = binding(&contributions, "StageAbility_612244");
    assert_eq!(
        combat
            .rule(hell.rule())
            .and_then(starclock_combat::catalog::definition::RuleDefinition::runtime)
            .unwrap()
            .triggers()
            .len(),
        7,
        "one fixed-element program preserves the triggering Break element"
    );

    let twilight = binding(&contributions, "StageAbility_612245");
    let twilight_program = first_program(combat, twilight.rule());
    assert!(matches!(
        &twilight_program.steps()[0],
        ProgramStep::Operation(RuleOperationTemplate::ApplyRandomEffect {
            effects,
            choice_rng_purpose,
            ..
        }) if effects.iter().map(|value| value.get()).eq(TWILIGHT_EFFECTS)
            && *choice_rng_purpose == starclock_combat::rng::types::DrawPurpose::BEHAVIOR_CHOICE
    ));
    assert!(matches!(
        &twilight_program.steps()[1],
        ProgramStep::Operation(RuleOperationTemplate::Cleanse {
            maximum: 1,
            order: starclock_combat::EffectRemovalOrder::NewestFirst,
            ..
        })
    ));
    let bleed_magnitude = combat
        .effect(starclock_combat::EffectDefinitionId::new(TWILIGHT_EFFECTS[0]).unwrap())
        .unwrap()
        .runtime_template()
        .unwrap()
        .magnitude_expression()
        .unwrap();
    assert!(
        expression_has_stat(bleed_magnitude, StatKind::BreakBaseDamage)
            && expression_has_scalar(bleed_magnitude, 2_000_000),
        "Twilight Bleed is capped at twice the applier's level-derived Break base damage"
    );

    let all_things = binding(&contributions, "StageAbility_612246");
    assert!(matches!(
        &first_program(combat, all_things.rule()).steps()[0],
        ProgramStep::Operation(RuleOperationTemplate::DetonateDot {
            selection: RuleDotSelection::RandomOne(purpose),
            fraction: ValueExpr::Literal(RuleValue::Scalar(value)),
            ..
        }) if *purpose == starclock_combat::rng::types::DrawPurpose::BEHAVIOR_CHOICE
            && value.scaled() == 1_500_000
    ));

    let ignosticism = binding(&contributions, "StageAbility_612250");
    let modifier = first_effect_modifier(combat, ignosticism.rule());
    assert_eq!(modifier.stat, StatKind::Hp);
    assert_eq!(literal_scalar(&modifier.value), 300_000);
}

#[test]
fn call_of_the_wilderness_assigned_levels_keep_exact_stack_coefficients() {
    let catalog = catalog();
    for (level, expected) in [(1, -3_000), (2, -4_000)] {
        let contributions = contributions_many(
            &catalog,
            "universe.path.nihility",
            &[("universe.blessing.612242", level)],
            None,
            false,
        );
        let materialization = materialize(&catalog, &contributions);
        let modifier = materialization
            .combat_catalog()
            .modifier(
                starclock_combat::ModifierDefinitionId::new(WILDERNESS_ATTACK_MODIFIER).unwrap(),
            )
            .expect("Call's Suspicion-scaled ATK modifier");
        assert!(
            expression_has_scalar(&modifier.value, expected),
            "Call level {level} retains the rounded six-decimal coefficient"
        );
    }
}

#[test]
fn night_beyond_pyre_changes_real_toughness_reduction_by_exactly_thirty_percent() {
    let catalog = catalog();
    let without = contributions_many(
        &catalog,
        "universe.path.nihility",
        &[("universe.blessing.612230", 1)],
        None,
        false,
    );
    let with = contributions_many(
        &catalog,
        "universe.path.nihility",
        &[("universe.blessing.612243", 1)],
        None,
        false,
    );
    let base = kafka_ultimate_reduction(&catalog, &without, 0x62);
    let boosted = kafka_ultimate_reduction(&catalog, &with, 0x62);
    assert_eq!(boosted.get() * 10, base.get() * 13);
}

#[test]
fn hell_spreads_the_triggering_break_and_random_dot_then_detonation_execute() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.nihility",
        &[
            ("universe.blessing.612244", 1),
            ("universe.blessing.612245", 2),
            ("universe.blessing.612246", 2),
        ],
        None,
        false,
    );
    let roster = kafka_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let spec = two_enemy_break_spec(&materialization, 0x63);
    let (mut battle, started) = start(&materialization, spec, 0x64);
    assert!(started.fault().is_none(), "{:?}", started.fault());

    let broken = use_kafka_ultimate(&mut battle);
    assert!(broken.fault().is_none(), "{:?}", broken.fault());
    let enemies = battle
        .view()
        .units_by_id()
        .filter(|unit| unit.side() == TeamSide::Enemy)
        .map(|unit| unit.id())
        .collect::<Vec<_>>();
    assert_eq!(enemies.len(), 2);
    assert!(
        enemies.iter().all(|enemy| battle
            .view()
            .units_by_id()
            .find(|unit| unit.id() == *enemy)
            .unwrap()
            .weakness_broken()),
        "the off-Lightning adjacent enemy is force-broken by Hell; events={:?}",
        broken.events()
    );
    assert!(
        !broken.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Toughness(starclock_combat::ToughnessEventData::Reduced {
                    target,
                    attempted,
                    ..
                }) if *target == enemies[0] && attempted.get() == 0
            )
        }),
        "Hell selects adjacent enemies only and does not emit a zero reduction for the primary"
    );

    for _ in 0..8 {
        let resolution = advance_targeting(&mut battle, enemies[0]);
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
        let twilight_applied = resolution.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Effect(starclock_combat::EffectEventData::Applied {
                    definition,
                    target,
                    ..
                }) if *target == enemies[0] && TWILIGHT_EFFECTS.contains(&definition.get())
            )
        });
        let detonations = resolution
            .events()
            .iter()
            .filter(|event| {
                matches!(
                    event.kind(),
                    BattleEventKind::Effect(starclock_combat::EffectEventData::Detonated {
                        target,
                        ..
                    }) if *target == enemies[0]
                )
            })
            .count();
        if twilight_applied {
            assert_eq!(detonations, 1, "All Things selects exactly one current DoT");
            return;
        }
    }
    panic!("the bounded deterministic fixture did not pass Twilight's 75% effect check");
}

pub(super) fn kafka_roster(catalog: &UniverseCatalog) -> UniverseBattleRoster {
    roster_for_forms_with_ability_kinds_and_energy(
        catalog,
        [KAFKA_FORM, 1, 2, 3],
        None,
        &[AbilityKind::Ultimate],
        true,
        120_000_000,
    )
}

fn kafka_ultimate_reduction(
    catalog: &Arc<UniverseCatalog>,
    contributions: &UniverseBattleContributionSet,
    marker: u8,
) -> RawToughness {
    let roster = kafka_roster(catalog);
    let materialization = materialize_with_roster(catalog, &roster, contributions);
    let spec = two_enemy_break_spec(&materialization, marker);
    let (mut battle, started) = start(&materialization, spec, marker.wrapping_add(1));
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = use_kafka_ultimate(&mut battle);
    resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Toughness(starclock_combat::ToughnessEventData::Reduced {
                element: CombatElement::Lightning,
                attempted,
                ..
            }) => Some(*attempted),
            _ => None,
        })
        .next()
        .expect("Kafka Ultimate attempts Lightning Toughness reduction")
}

pub(super) fn use_kafka_ultimate(battle: &mut Battle) -> starclock_combat::Resolution {
    for _ in 0..12 {
        let decision = battle.decision().expect("interrupt decision").clone();
        if let Some(command) = decision.legal_commands().iter().find(|command| {
            matches!(
                command,
                Command::UseInterrupt { ability, .. } | Command::UseAbility { ability, .. }
                    if ability.get() == KAFKA_ULTIMATE
            )
        }) {
            return battle
                .apply(command.clone())
                .expect("Kafka Ultimate is accepted");
        }
        let command = decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::PassInterruptWindow { .. }))
            .expect("Kafka interrupt is offered before the normal action")
            .clone();
        let resolution = battle.apply(command).expect("interrupt pass is accepted");
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    }
    panic!("Kafka Ultimate was not offered");
}

fn advance_targeting(
    battle: &mut Battle,
    target: starclock_combat::UnitId,
) -> starclock_combat::Resolution {
    let decision = battle.decision().expect("nonterminal fixture").clone();
    let command = decision
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseAbility {
                    primary_target: Some(primary),
                    ..
                } if *primary == target
            )
        })
        .or_else(|| {
            decision
                .legal_commands()
                .iter()
                .find(|command| matches!(command, Command::PassInterruptWindow { .. }))
        })
        .or_else(|| {
            decision
                .legal_commands()
                .iter()
                .find(|command| matches!(command, Command::UseAbility { .. }))
        })
        .expect("fixture has a progress command")
        .clone();
    battle.apply(command).expect("fixture command is accepted")
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

fn first_program(
    combat: &starclock_combat::catalog::CombatCatalog,
    rule: starclock_combat::RuleId,
) -> &starclock_combat::catalog::definition::ProgramDefinition {
    let id = combat.rule(rule).unwrap().programs()[0];
    combat.program(id).unwrap()
}

fn first_effect_modifier(
    combat: &starclock_combat::catalog::CombatCatalog,
    rule: starclock_combat::RuleId,
) -> &starclock_combat::modifier::model::ModifierDefinition {
    let program = first_program(combat, rule);
    let effect = combat.effect(program.effects()[0]).unwrap();
    combat.modifier(effect.modifiers()[0]).unwrap()
}

fn literal_scalar(value: &ValueExpr) -> i64 {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled(),
        _ => panic!("expected a literal Scalar"),
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

fn expression_has_stat(value: &ValueExpr, expected: StatKind) -> bool {
    match value {
        ValueExpr::QueryStat { stat, .. } => *stat == expected,
        ValueExpr::Multiply { lhs, rhs, .. }
        | ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Divide { lhs, rhs, .. }
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => {
            expression_has_stat(lhs, expected) || expression_has_stat(rhs, expected)
        }
        ValueExpr::Clamp {
            value,
            minimum,
            maximum,
        } => {
            expression_has_stat(value, expected)
                || expression_has_stat(minimum, expected)
                || expression_has_stat(maximum, expected)
        }
        _ => false,
    }
}

pub(super) fn two_enemy_break_spec(
    materialization: &UniverseBattleMaterialization,
    marker: u8,
) -> BattleSpec {
    let original = durable_spec_with_two_enemy_hp(
        materialization,
        marker,
        [
            Hp::new(9_000_000_000_000).unwrap(),
            Hp::new(9_000_000_000_000).unwrap(),
        ],
    );
    let mut enemy_index = 0_u32;
    let participants = original
        .participants()
        .iter()
        .enumerate()
        .map(|(index, participant)| {
            if participant.side() != TeamSide::Enemy {
                return participant.clone();
            }
            let source = match participant.source() {
                ParticipantSource::EncounterEnemy(source) => source,
                _ => panic!("fixture enemy source"),
            };
            let base = participant.combatant();
            let weakness = if enemy_index == 0 {
                CombatElement::Lightning
            } else {
                CombatElement::Fire
            };
            enemy_index += 1;
            let combatant = ResolvedCombatantSpec::new(
                base.form(),
                base.level(),
                base.maximum_hp(),
                Speed::from_scaled(50_000_000).unwrap(),
                ResolvedDefinitionBindings::new(
                    base.abilities().to_vec(),
                    base.rule_bundles().to_vec(),
                    base.modifiers().to_vec(),
                )
                .unwrap(),
                CombatantSpecDigest::new([marker.wrapping_add(index as u8); 32]).unwrap(),
            )
            .unwrap()
            .with_base_attack_defense(base.base_attack(), base.base_defense())
            .with_energy(base.current_energy(), base.maximum_energy())
            .unwrap()
            .with_sources(base.sources().to_vec())
            .unwrap()
            .with_modifier_bindings(base.modifier_bindings().to_vec())
            .unwrap()
            .with_toughness(
                base.rank(),
                vec![weakness],
                vec![ToughnessLayerSpec::ordinary(1, RawToughness::new(10).unwrap()).unwrap()],
            )
            .unwrap();
            ParticipantSpec::new(
                TeamSide::Enemy,
                participant.formation(),
                ParticipantSource::EncounterEnemy(source),
                combatant,
            )
            .with_wave(participant.wave())
            .unwrap()
        })
        .collect::<Vec<_>>();
    BattleSpec::new(
        original.rules_revision(),
        AssemblyDigest::new([marker.wrapping_add(7); 32]).unwrap(),
        original.encounter(),
        participants,
        original.resources(TeamSide::Player).clone(),
        original.resources(TeamSide::Enemy).clone(),
        original.concede_policy(),
    )
    .unwrap()
}
