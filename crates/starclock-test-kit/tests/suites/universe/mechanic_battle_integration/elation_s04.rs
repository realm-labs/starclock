use super::*;
use starclock_combat::{
    ModifierDefinitionId,
    catalog::action::{AbilityProgramTiming, AbilityTag},
    modifier::model::{FormulaPurpose, FormulaStage, StatKind},
    rule::model::{ProgramStep, RuleOperationTemplate, RuleValue, ValueExpr},
};

const CHAMPION: (&str, u32) = ("universe.blessing.612632", 2);
const AUTO_HARMONICA: (&str, u32) = ("universe.blessing.612630", 2);
const SLAUGHTERHOUSE: (&str, u32) = ("universe.blessing.612631", 2);
const PORTRAIT: (&str, u32) = ("universe.blessing.612640", 2);
const TWILIGHT: (&str, u32) = ("universe.blessing.612641", 2);
const HOURGLASS: (&str, u32) = ("universe.blessing.612642", 2);
const TWELVE_MONKEYS: (&str, u32) = ("universe.blessing.612644", 2);
const AIDEN: (&str, u32) = ("universe.blessing.612645", 2);
const MILITARY_RULE: (&str, u32) = ("universe.blessing.612646", 2);
const EXEMPLARY: (&str, u32) = ("universe.blessing.612650", 2);
const MOSTLY_HARMFUL: (&str, u32) = ("universe.blessing.612651", 2);
const SUSPIRIA: (&str, u32) = ("universe.blessing.612652", 2);
const PLATINUM: (&str, u32) = ("universe.blessing.612656", 2);
const APPLE: (&str, u32) = ("universe.blessing.612657", 2);
const DOOMSDAY: &str = "universe.resonance.612621";
const DANCE: &str = "universe.resonance.612622";
const INSTANT: &str = "universe.resonance.612623";

#[test]
fn goal07_p2_m08_s04_materializes_all_rules_without_native_handlers() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    for key in [
        "StageAbility_612656",
        "StageAbility_612657",
        "StageAbility_612621",
        "StageAbility_612623",
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
}

#[test]
fn enhanced_platinum_age_and_clockwork_apple_are_two_turn_replace_by_caster_buffs() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    for (key, stat, value) in [
        ("StageAbility_612656", StatKind::Def, 400_000),
        ("StageAbility_612657", StatKind::Spd, 160_000),
    ] {
        let binding = binding(&contributions, key);
        let raw = binding.rule().get();
        let modifier = combat
            .modifier(ModifierDefinitionId::new(0x76c0_0000 + raw).unwrap())
            .expect("timed stat modifier");
        assert_eq!(
            (modifier.stat, modifier.stage, modifier.purpose),
            (stat, FormulaStage::PercentOfBase, FormulaPurpose::Stat)
        );
        assert!(expression_has_scalar(&modifier.value, value));
        let effect = combat
            .effect(starclock_combat::EffectDefinitionId::new(0x7660_0000 + raw).unwrap())
            .expect("timed stat effect");
        let runtime = effect.runtime_template().expect("runtime template");
        assert_eq!(runtime.stack_limit(), 1);
        assert_eq!(
            runtime.duration_expression(),
            Some(&ValueExpr::Literal(RuleValue::Integer(2)))
        );
        assert_eq!(
            runtime.duration_clock(),
            starclock_combat::DurationClock::OwnerTurnEnd
        );
        let triggers = combat
            .rule(binding.rule())
            .unwrap()
            .runtime()
            .unwrap()
            .triggers();
        assert_eq!(triggers.len(), 3);
        let tags = triggers
            .iter()
            .filter_map(|trigger| trigger.filter.ability_tag)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            tags,
            [
                AbilityTag::FollowUp,
                AbilityTag::Counter,
                AbilityTag::Ultimate
            ]
            .into_iter()
            .collect()
        );
    }
}

#[test]
fn complete_elation_resonance_encodes_random_hits_overflow_and_formation_energy() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let resonance = combat
        .ability(AbilityId::new(RESONANCE_ABILITY_RAW).unwrap())
        .expect("manual Elation Resonance");
    let action = resonance.action().expect("action");
    assert!(action.tags().contains(AbilityTag::Assist));
    assert!(action.tags().contains(AbilityTag::FollowUp));
    assert_eq!(
        resonance.programs()[0].timing(),
        AbilityProgramTiming::BeforeHits
    );
    let root = combat
        .program(resonance.programs()[0].program())
        .expect("resonance root");
    let branches = root
        .called_programs()
        .iter()
        .map(|id| combat.program(*id).expect("branch"))
        .collect::<Vec<_>>();
    assert!(branches.iter().any(|program| {
        program.steps().iter().any(|step| {
            matches!(
                step,
                ProgramStep::Operation(RuleOperationTemplate::RandomRepeatedDamage {
                    minimum_hits: 3,
                    maximum_hits: 5,
                    class: starclock_combat::formula::model::DamageClass::Elation,
                    elements,
                    ..
                }) if elements.len() == 7
            )
        })
    }));
    assert!(branches.iter().any(|program| {
        program.steps().iter().any(|step| {
            matches!(
                step,
                ProgramStep::Operation(RuleOperationTemplate::RandomRepeatedDamage {
                    minimum_hits: 8,
                    maximum_hits: 10,
                    ..
                })
            )
        }) && program.steps().iter().any(|step| {
            matches!(
                step,
                ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                    update: starclock_combat::rule::model::ResourceUpdateKind::Spend,
                    ..
                })
            )
        })
    }));

    let spec = materialization.difficulty_specs()[0].battle_spec();
    let resource = spec
        .resources(TeamSide::Player)
        .keyed()
        .iter()
        .find(|resource| resource.id().get() == RESONANCE_RESOURCE_RAW)
        .expect("path resource");
    assert_eq!((resource.initial(), resource.maximum()), (80, 200));

    let instant = combat
        .rule(binding(&contributions, "StageAbility_612623").rule())
        .unwrap()
        .runtime()
        .unwrap();
    assert_eq!(instant.triggers().len(), 3);
    let energy_program = combat.program(instant.triggers()[0].program).unwrap();
    assert!(matches!(
        &energy_program.steps()[0],
        ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
            amount: ValueExpr::Literal(RuleValue::Scalar(amount)),
            update: starclock_combat::rule::model::ResourceUpdateKind::Gain,
            ..
        }) if amount.scaled() == 10_000_000
    ));
}

#[test]
fn charged_complete_resonance_spends_all_energy_and_emits_repeated_elation_damage() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let original = durable_spec_with_two_enemies(&materialization, 0xf1);
    let spec = with_resonance_energy(original, 200, 200, 0xf2);
    let (mut battle, started) = start(&materialization, spec, 0xf3);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = use_resonance(&mut battle);
    assert!(
        resolution.fault().is_none(),
        "{:?} {:#?}",
        resolution.fault(),
        resolution.events()
    );
    assert_eq!(
        battle.view().team(TeamSide::Player).keyed_resource(
            starclock_combat::SourceDefinitionId::new(RESONANCE_RESOURCE_RAW).unwrap()
        ),
        Some((0, 200))
    );
    let damage = resolution
        .events()
        .iter()
        .filter(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Damage(data)
                    if data.class
                        == starclock_combat::formula::model::DamageClass::Elation
            ) && event.cause().source_definition()
                == Some(starclock_combat::SourceDefinitionId::new(RESONANCE_ABILITY_RAW).unwrap())
        })
        .count();
    assert!(
        (16..=20).contains(&damage),
        "8–10 all-enemy hits against two enemies, got {damage}"
    );
}

fn full_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many_with_formations(
        catalog,
        "universe.path.elation",
        &[
            AUTO_HARMONICA,
            SLAUGHTERHOUSE,
            CHAMPION,
            PORTRAIT,
            TWILIGHT,
            HOURGLASS,
            TWELVE_MONKEYS,
            AIDEN,
            MILITARY_RULE,
            EXEMPLARY,
            MOSTLY_HARMFUL,
            SUSPIRIA,
            PLATINUM,
            APPLE,
        ],
        &[DOOMSDAY, DANCE, INSTANT],
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

fn use_resonance(battle: &mut Battle) -> starclock_combat::Resolution {
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseInterrupt { ability, .. } | Command::UseAbility { ability, .. }
                    if ability.get() == RESONANCE_ABILITY_RAW
            )
        })
        .unwrap_or_else(|| panic!("charged Resonance is legal"))
        .clone();
    battle.apply(command).unwrap()
}
