use super::*;
use starclock_combat::{
    EffectEventData,
    catalog::action::{AbilityProgramTiming, AbilityTag, TargetPattern, TargetRelation},
    formula::model::DamageClass,
    modifier::model::{FormulaPurpose, FormulaStage},
    rule::model::{ProgramStep, RuleEventPoint, RuleOperationTemplate, RuleValue, ValueExpr},
};

const SPORANGIUM: (&str, u32) = ("universe.blessing.612756", 2);
const VESICLE: (&str, u32) = ("universe.blessing.612757", 2);
const PROBOSCIS: &str = "universe.resonance.612721";
const PHENOL: &str = "universe.resonance.612722";
const CRYSTAL: &str = "universe.resonance.612723";
const CRYSTAL_ABILITY_RAW: u32 = 0x7ca0_0003;

#[test]
fn goal07_p2_m09_s04_materializes_all_exact_rules_without_native_handlers() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    for key in [
        "StageAbility_612756",
        "StageAbility_612757",
        "StageAbility_612721",
        "StageAbility_612722",
        "StageAbility_612723",
    ] {
        let runtime = combat
            .rule(binding(&contributions, key).rule())
            .unwrap_or_else(|| panic!("{key} is executable"))
            .runtime()
            .expect("generic runtime");
        assert!(runtime.native_handler().is_none());
    }

    for key in ["StageAbility_612756", "StageAbility_612757"] {
        let runtime = combat
            .rule(binding(&contributions, key).rule())
            .unwrap()
            .runtime()
            .unwrap();
        assert_eq!(runtime.triggers().len(), 1);
        assert_eq!(
            runtime.triggers()[0].event_point,
            RuleEventPoint::ResourceChanged
        );
        assert_eq!(
            runtime.triggers()[0].filter.resource,
            Some(starclock_combat::rule::model::RuleResourceKind::SkillPoints)
        );
    }
}

#[test]
fn propagation_resonance_and_formations_keep_exact_target_effect_and_energy_contracts() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let resonance = combat
        .ability(AbilityId::new(RESONANCE_ABILITY_RAW).unwrap())
        .expect("manual Propagation Resonance");
    let action = resonance.action().unwrap();
    assert!(action.tags().contains(AbilityTag::Assist));
    assert!(!action.tags().contains(AbilityTag::Attack));
    let selector = combat.selector(resonance.selector()).unwrap();
    let target = selector.unit_targets().unwrap();
    assert_eq!(target.relation(), TargetRelation::Allied);
    assert_eq!(target.pattern(), TargetPattern::Single);
    assert_eq!(
        resonance.programs()[0].timing(),
        AbilityProgramTiming::BeforeHits
    );
    let main = combat.program(resonance.programs()[0].program()).unwrap();
    assert!(main.steps().iter().any(|step| {
        matches!(
            step,
            ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                resource: starclock_combat::rule::model::RuleResourceKind::SkillPoints,
                amount: ValueExpr::Literal(RuleValue::Scalar(value)),
                ..
            }) if value.scaled() == 2_000_000
        )
    }));
    assert!(main.steps().iter().any(|step| {
        matches!(
            step,
            ProgramStep::Operation(RuleOperationTemplate::AdvanceAction {
                amount: ValueExpr::Literal(RuleValue::Scalar(value)),
                ..
            }) if value.scaled() == 1_000_000
        )
    }));
    let metamorphosis = main.effects()[0];
    let runtime = combat
        .effect(metamorphosis)
        .unwrap()
        .runtime_template()
        .unwrap();
    assert_eq!(runtime.stack_limit(), 1);
    assert_eq!(
        runtime.duration_expression(),
        Some(&ValueExpr::Literal(RuleValue::Integer(2)))
    );
    assert_eq!(
        runtime.duration_clock(),
        starclock_combat::DurationClock::OwnerTurnEnd
    );
    assert!(
        combat
            .effect(metamorphosis)
            .unwrap()
            .modifiers()
            .iter()
            .any(
                |modifier| combat.modifier(*modifier).is_some_and(|modifier| {
                    modifier.stage == FormulaStage::DamageBoost
                        && modifier.purpose == FormulaPurpose::OrdinaryDamage
                        && expression_has_scalar(&modifier.value, 400_000)
                })
            )
    );

    let crystal = combat
        .ability(AbilityId::new(CRYSTAL_ABILITY_RAW).unwrap())
        .expect("Crystal Pincers auxiliary Basic-damage action");
    assert_ne!(
        crystal.action().unwrap().kind(),
        starclock_combat::catalog::action::AbilityKind::Basic
    );
    assert!(crystal.action().unwrap().tags().contains(AbilityTag::Basic));
    assert!(
        !crystal
            .action()
            .unwrap()
            .tags()
            .contains(AbilityTag::Attack)
    );
    assert!(crystal.programs().iter().any(|binding| {
        combat.program(binding.program()).is_some_and(|program| {
            program.steps().iter().any(|step| {
                matches!(
                    step,
                    ProgramStep::Operation(RuleOperationTemplate::DamageFromActorBasicElement {
                        class: DamageClass::Direct,
                        can_crit: true,
                        ..
                    })
                )
            })
        })
    }));

    let resource = materialization.difficulty_specs()[0]
        .battle_spec()
        .resources(TeamSide::Player)
        .keyed()
        .iter()
        .find(|resource| resource.id().get() == RESONANCE_RESOURCE_RAW)
        .unwrap();
    assert_eq!(resource.maximum(), 200);
}

#[test]
fn charged_resonance_advances_one_ally_applies_latest_metamorphosis_and_recharges_from_sp_gain() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let original = durable_spec(&materialization, 0xe1, false);
    let spec = with_resonance_energy(original, 200, 200, 0xe2);
    let (mut battle, started) = start(&materialization, spec, 0xe3);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let before_skill_points = battle.view().team(TeamSide::Player).skill_points();
    let resolution = use_resonance(&mut battle);
    assert!(
        resolution.fault().is_none(),
        "{:?} {:#?}",
        resolution.fault(),
        resolution.events()
    );
    assert_eq!(
        battle.view().team(TeamSide::Player).skill_points(),
        before_skill_points.saturating_add(2).min(5)
    );
    assert_eq!(
        battle.view().team(TeamSide::Player).keyed_resource(
            starclock_combat::SourceDefinitionId::new(RESONANCE_RESOURCE_RAW).unwrap()
        ),
        Some((104, 200)),
        "100 is spent, then two recovered Skill Points each restore 2 Resonance Energy"
    );
    let applied = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Effect(EffectEventData::Applied {
                definition, target, ..
            }) if event.cause().source_definition()
                == Some(
                    starclock_combat::SourceDefinitionId::new(RESONANCE_ABILITY_RAW).unwrap(),
                ) =>
            {
                Some((*definition, *target))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(applied.len(), 1, "{:#?}", resolution.events());
    let active = battle
        .view()
        .effects_by_id()
        .filter(|effect| effect.definition() == applied[0].0)
        .collect::<Vec<_>>();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].target(), applied[0].1);
    assert_eq!(active[0].stacks(), 1);
}

fn full_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many_with_formations(
        catalog,
        "universe.path.propagation",
        &[SPORANGIUM, VESICLE],
        &[PROBOSCIS, PHENOL, CRYSTAL],
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
        .unwrap_or_else(|| panic!("charged targeted Resonance is legal"))
        .clone();
    battle.apply(command).unwrap()
}
