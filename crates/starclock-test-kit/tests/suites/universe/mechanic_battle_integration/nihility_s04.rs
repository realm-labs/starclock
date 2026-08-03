use super::*;
use starclock_combat::{
    RawToughness,
    catalog::action::AbilityProgramTiming,
    modifier::model::{FormulaPurpose, StatKind},
    rule::model::{ProgramStep, RuleOperationTemplate, RuleValue, ValueExpr},
};
use super::{nihility_s02};

const OFFERINGS: (&str, u32) = ("universe.blessing.612256", 2);
const BEFORE_SUNRISE: (&str, u32) = ("universe.blessing.612257", 2);
const FOURFOLD_ROOT: &str = "universe.resonance.612221";
const SUFFERING_SUNSHINE: &str = "universe.resonance.612222";
const OUTSIDER: &str = "universe.resonance.612223";
const EFFECTS: [u32; 6] = [
    0x7920_0002,
    0x7920_0003,
    0x7920_0004,
    0x7920_0005,
    0x7920_0006,
    0x7920_0007,
];

#[test]
fn goal07_p2_m04_s04_materializes_every_assigned_exact_mechanic() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    for key in [
        "StageAbility_612256",
        "StageAbility_612257",
        "StageAbility_612220",
        "StageAbility_612221",
        "StageAbility_612222",
        "StageAbility_612223",
    ] {
        assert!(
            contributions
                .rules()
                .iter()
                .any(|rule| rule.source_binding_key() == Some(key)),
            "{key} contribution is selected"
        );
    }
    let roster = nihility_s02::kafka_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let combat = materialization.combat_catalog();
    let ability = combat
        .ability(AbilityId::new(RESONANCE_ABILITY_RAW).unwrap())
        .expect("Nihility Resonance is executable");
    let before = ability
        .programs()
        .iter()
        .find(|binding| binding.timing() == AbilityProgramTiming::BeforeHits)
        .expect("Resonance owns its effect application program");
    let steps = combat.program(before.program()).unwrap().steps();
    assert_eq!(steps.len(), 6);
    assert_apply(&steps[0], EFFECTS[0], 1, 1_800_000);
    assert_apply(&steps[1], EFFECTS[1], 1, 1_800_000);
    assert_apply(&steps[2], EFFECTS[2], 1, 1_800_000);
    assert_apply(&steps[3], EFFECTS[3], 3, 1_800_000);
    assert_apply(&steps[4], EFFECTS[4], 3, 2_000_000);
    assert_apply(&steps[5], EFFECTS[5], 3, 2_000_000);

    for raw in EFFECTS {
        let runtime = combat
            .effect(starclock_combat::EffectDefinitionId::new(raw).unwrap())
            .unwrap()
            .runtime_template()
            .unwrap();
        assert!(matches!(
            runtime.duration_expression(),
            Some(ValueExpr::Literal(RuleValue::Integer(3)))
        ));
    }
    let devoid = combat
        .effect(starclock_combat::EffectDefinitionId::new(EFFECTS[5]).unwrap())
        .unwrap();
    let modifier = combat.modifier(devoid.modifiers()[0]).unwrap();
    assert_eq!(
        (modifier.stat, modifier.purpose),
        (StatKind::ToughnessRecovery, FormulaPurpose::Stat)
    );
    assert!(expression_has_scalar(&modifier.value, 100_000));
    let before = contributions
        .rules()
        .iter()
        .find(|rule| rule.source_binding_key() == Some("StageAbility_612257"))
        .unwrap();
    let program = combat
        .program(combat.rule(before.rule()).unwrap().programs()[0])
        .unwrap();
    assert!(matches!(
        &program.steps()[0],
        ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
            amount: ValueExpr::Literal(RuleValue::Scalar(amount)),
            ..
        }) if amount.scaled() == 3_000_000
    ));
}

#[test]
fn nihility_resonance_applies_all_six_statuses_with_formation_upgrades() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let roster = nihility_s02::kafka_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let (mut battle, started) = start(
        &materialization,
        durable_spec(&materialization, 0x81, true),
        0x82,
    );
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = use_resonance(&mut battle);
    assert!(
        resolution.fault().is_none(),
        "{:?} {:?}",
        resolution.fault(),
        resolution.events()
    );
    let enemy = battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Enemy)
        .unwrap()
        .id();
    let statuses = battle
        .view()
        .effects_by_id()
        .filter(|effect| effect.target() == enemy && EFFECTS.contains(&effect.definition().get()))
        .map(|effect| {
            (
                effect.definition().get(),
                effect.stacks(),
                effect.remaining(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        statuses,
        vec![
            (EFFECTS[0], 1, Some(3)),
            (EFFECTS[1], 1, Some(3)),
            (EFFECTS[2], 1, Some(3)),
            (EFFECTS[3], 3, Some(3)),
            (EFFECTS[4], 3, Some(3)),
            (EFFECTS[5], 3, Some(3)),
        ]
    );
    assert_eq!(
        battle.view().team(TeamSide::Player).keyed_resource(
            starclock_combat::SourceDefinitionId::new(RESONANCE_RESOURCE_RAW).unwrap()
        ),
        Some((0, 100))
    );
}

#[test]
fn enemy_dot_ticks_heal_the_team_restore_random_energy_and_charge_resonance() {
    let catalog = catalog();
    let contributions = contributions_many_with_formations(
        &catalog,
        "universe.path.nihility",
        &[OFFERINGS, BEFORE_SUNRISE],
        &[OUTSIDER],
        None,
        false,
    );
    let roster = nihility_s02::kafka_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let spec = durable_spec_with_enemy_speed(
        &materialization,
        0x83,
        false,
        Some(Speed::from_scaled(400_000_000).unwrap()),
    );
    let (mut battle, started) = start(&materialization, spec, 0x84);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    assert_eq!(resonance_energy(&battle), 40);
    let applied = use_kafka_ultimate(&mut battle);
    assert!(applied.fault().is_none(), "{:?}", applied.fault());
    let energy_before_tick = resonance_energy(&battle);
    assert_eq!(
        energy_before_tick, 42,
        "Kafka's immediate DoT damage charges Outsider once"
    );

    for _ in 0..40 {
        let resolution = advance(&mut battle);
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
        let dot = resolution.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Damage(data)
                    if data.class == starclock_combat::formula::model::DamageClass::Dot
            )
        });
        if !dot {
            continue;
        }
        let dot_count = resolution
            .events()
            .iter()
            .filter(|event| {
                matches!(
                    event.kind(),
                    BattleEventKind::Damage(data)
                        if data.class == starclock_combat::formula::model::DamageClass::Dot
                )
            })
            .count();
        let heals = resolution
            .events()
            .iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::Heal(data) => Some(data.calculated.get()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(heals, vec![1_500; dot_count * 4]);
        assert!(resolution.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Resource(starclock_combat::ResourceEventData::Energy { .. })
            )
        }));
        assert_eq!(
            resonance_energy(&battle),
            energy_before_tick + u16::try_from(dot_count).unwrap() * 2
        );
        return;
    }
    panic!("production Kafka Shock did not reach its enemy turn-start tick");
}

#[test]
fn confusion_detonates_current_dots_and_devoid_reduces_toughness_recovery() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let roster = nihility_s02::kafka_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let spec = charged_resonance(
        nihility_s02::two_enemy_break_spec(&materialization, 0x85),
        0x87,
    );
    let (mut battle, started) = start(&materialization, spec, 0x86);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resonance = use_resonance(&mut battle);
    assert!(resonance.fault().is_none(), "{:?}", resonance.fault());
    let broken = nihility_s02::use_kafka_ultimate(&mut battle);
    assert!(broken.fault().is_none(), "{:?}", broken.fault());
    let enemy = battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Enemy && unit.weakness_broken())
        .unwrap()
        .id();
    assert!(
        broken.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Effect(starclock_combat::EffectEventData::Detonated {
                    target,
                    ..
                }) if *target == enemy
            )
        }),
        "Confusion detonates the Resonance DoTs: {:?}",
        broken.events()
    );
    assert_eq!(
        battle
            .view()
            .effects_by_id()
            .find(|effect| effect.target() == enemy && effect.definition().get() == EFFECTS[4])
            .unwrap()
            .stacks(),
        2
    );

    for _ in 0..80 {
        let resolution = advance(&mut battle);
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
        if let Some(after) =
            resolution
                .events()
                .iter()
                .find_map(|event| match event.kind() {
                    BattleEventKind::Toughness(
                        starclock_combat::ToughnessEventData::Recovered { target, after, .. },
                    ) if *target == enemy => Some(*after),
                    _ => None,
                })
        {
            assert_eq!(
                after,
                RawToughness::new(7).unwrap(),
                "three Devoid stacks reduce ordinary recovery from 10 to 7"
            );
            return;
        }
    }
    panic!("broken enemy did not reach its deterministic recovery turn");
}

fn full_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many_with_formations(
        catalog,
        "universe.path.nihility",
        &[OFFERINGS, BEFORE_SUNRISE],
        &[FOURFOLD_ROOT, SUFFERING_SUNSHINE, OUTSIDER],
        None,
        false,
    )
}

fn charged_resonance(original: BattleSpec, marker: u8) -> BattleSpec {
    let resources = TeamResourceSpec::new(3, 5)
        .unwrap()
        .with_keyed(vec![
            KeyedTeamResourceSpec::new(
                starclock_combat::SourceDefinitionId::new(RESONANCE_RESOURCE_RAW).unwrap(),
                100,
                100,
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

fn use_resonance(battle: &mut Battle) -> starclock_combat::Resolution {
    use_ability(battle, RESONANCE_ABILITY_RAW)
}

fn use_kafka_ultimate(battle: &mut Battle) -> starclock_combat::Resolution {
    use_ability(battle, 20_033)
}

fn use_ability(battle: &mut Battle, expected: u32) -> starclock_combat::Resolution {
    for _ in 0..40 {
        let decision = battle.decision().expect("nonterminal battle").clone();
        if let Some(command) = decision.legal_commands().iter().find(|command| {
            matches!(
                command,
                Command::UseInterrupt { ability, .. } | Command::UseAbility { ability, .. }
                    if ability.get() == expected
            )
        }) {
            return battle
                .apply(command.clone())
                .expect("requested ability is accepted");
        }
        let progress = decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::PassInterruptWindow { .. }))
            .or_else(|| {
                decision
                    .legal_commands()
                    .iter()
                    .find(|command| matches!(command, Command::UseAbility { .. }))
            })
            .expect("fixture can progress to the requested ability")
            .clone();
        let resolution = battle
            .apply(progress)
            .expect("progress command is accepted");
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    }
    panic!("ability {expected} was not offered");
}

fn advance(battle: &mut Battle) -> starclock_combat::Resolution {
    let decision = battle.decision().expect("nonterminal fixture").clone();
    let command = decision
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::PassInterruptWindow { .. }))
        .or_else(|| {
            decision
                .legal_commands()
                .iter()
                .find(|command| matches!(command, Command::UseAbility { .. }))
        })
        .expect("fixture has a deterministic progress command")
        .clone();
    battle.apply(command).expect("progress command is accepted")
}

fn resonance_energy(battle: &Battle) -> u16 {
    battle
        .view()
        .team(TeamSide::Player)
        .keyed_resource(starclock_combat::SourceDefinitionId::new(RESONANCE_RESOURCE_RAW).unwrap())
        .unwrap()
        .0
}

fn assert_apply(step: &ProgramStep, effect: u32, stacks: i64, chance: i64) {
    assert!(matches!(
        step,
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            effect: definition,
            stacks: ValueExpr::Literal(RuleValue::Integer(actual_stacks)),
            base_chance: Some(ValueExpr::Literal(RuleValue::Scalar(actual_chance))),
            ..
        }) if definition.get() == effect
            && *actual_stacks == stacks
            && actual_chance.scaled() == chance
    ));
}

fn expression_has_scalar(value: &ValueExpr, expected: i64) -> bool {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled().abs() == expected,
        ValueExpr::Negate(value) | ValueExpr::Convert { value, .. } => {
            expression_has_scalar(value, expected)
        }
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
