use super::*;

use starclock_combat::{
    DurationClock, EffectCategory,
    modifier::model::{FormulaPurpose, FormulaStage, StatKind},
    rule::model::{ProgramStep, ResourceUpdateKind, RuleOperationTemplate, ValueExpr},
};
use starclock_mode_universe::{battle_contribution::UniverseBattleRuleRole, curio::CurioStateKind};

const ASSIGNED: [&str; 5] = [
    "universe.curio.65",
    "universe.curio.66",
    "universe.curio.67",
    "universe.curio.70",
    "universe.curio.71",
];

#[test]
fn goal07_p3_m12_s03_materializes_only_combat_curios_without_native_handlers() {
    let catalog = catalog();
    for stable_key in ASSIGNED {
        let snapshot = contribution(&catalog, stable_key);
        let materialization = materialize(&catalog, &snapshot);
        let state_binding = snapshot
            .rules()
            .iter()
            .find(|binding| binding.role() == UniverseBattleRuleRole::CurioState)
            .unwrap();
        let rule = materialization.combat_catalog().rule(state_binding.rule());
        if matches!(stable_key, "universe.curio.66" | "universe.curio.71") {
            assert!(
                rule.and_then(|definition| definition.runtime())
                    .is_some_and(|runtime| runtime.native_handler().is_none())
            );
        } else {
            assert!(
                rule.is_none(),
                "{stable_key} is an Activity-only contribution"
            );
        }
    }
}

#[test]
fn black_forest_applies_one_five_turn_major_aggro_effect() {
    let catalog = catalog();
    let snapshot = contribution(&catalog, "universe.curio.66");
    let materialization = materialize(&catalog, &snapshot);
    let combat = materialization.combat_catalog();
    let rule = state_rule(&snapshot, combat);
    let effect = rule
        .programs()
        .iter()
        .filter_map(|program| combat.program(*program))
        .flat_map(|program| program.effects())
        .filter_map(|effect| combat.effect(*effect))
        .find(|effect| {
            effect.modifiers().iter().any(|modifier| {
                combat.modifier(*modifier).is_some_and(|modifier| {
                    modifier.stat == StatKind::Aggro
                        && modifier.stage == FormulaStage::PercentOfBase
                        && modifier.purpose == FormulaPurpose::Aggro
                        && matches!(
                            modifier.value,
                            ValueExpr::Literal(
                                starclock_combat::rule::model::RuleValue::Scalar(value)
                            ) if value.scaled() == 5_000_000
                        )
                })
            })
        })
        .unwrap();
    assert_eq!(
        effect.runtime_template().unwrap().category(),
        EffectCategory::NeutralState
    );
    let (battle, started) = start(
        &materialization,
        durable_spec(&materialization, 0xf7, false),
        0xf8,
    );
    assert!(
        started.fault().is_none(),
        "fault={:?}; events={:#?}",
        started.fault(),
        started.events()
    );
    let applied = battle
        .view()
        .effects_by_id()
        .filter(|candidate| candidate.definition() == effect.id())
        .collect::<Vec<_>>();
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].remaining(), Some(5));
    assert_eq!(applied[0].duration_clock(), DurationClock::TargetTurnEnd);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .find(|unit| unit.id() == applied[0].target())
            .unwrap()
            .side(),
        TeamSide::Player
    );
}

#[test]
fn mechanical_consumes_two_skill_points_at_battle_start_with_zero_floor() {
    let catalog = catalog();
    let snapshot = contribution(&catalog, "universe.curio.71");
    let materialization = materialize(&catalog, &snapshot);
    let combat = materialization.combat_catalog();
    let rule = state_rule(&snapshot, combat);
    assert!(rule.programs().iter().any(|program| {
        combat
            .program(*program)
            .unwrap()
            .steps()
            .iter()
            .any(|step| {
                matches!(
                    step,
                    ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                        resource: starclock_combat::rule::model::RuleResourceKind::SkillPoints,
                        update: ResourceUpdateKind::Spend,
                        ..
                    })
                )
            })
    }));
    let (battle, started) = start(
        &materialization,
        durable_spec(&materialization, 0xf9, false),
        0xfa,
    );
    assert!(started.fault().is_none(), "{:?}", started.fault());
    assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 1);
}

fn state_rule<'a>(
    snapshot: &UniverseBattleContributionSet,
    combat: &'a starclock_combat::catalog::CombatCatalog,
) -> &'a starclock_combat::catalog::definition::RuleDefinition {
    snapshot
        .rules()
        .iter()
        .find(|binding| binding.role() == UniverseBattleRuleRole::CurioState)
        .and_then(|binding| combat.rule(binding.rule()))
        .unwrap()
}

fn contribution(catalog: &Arc<UniverseCatalog>, stable_key: &str) -> UniverseBattleContributionSet {
    let path_definition = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.erudition")
        .unwrap();
    let blessings = BlessingRuntimeCatalog::compile(catalog)
        .unwrap()
        .contributions_from_owned(&[])
        .unwrap();
    let path = PathRuntimeCatalog::compile(catalog)
        .unwrap()
        .contributions(path_definition.id(), &blessings, &[])
        .unwrap();
    let runtime = CurioRuntimeCatalog::compile(catalog).unwrap();
    let definition = runtime
        .definitions()
        .iter()
        .find(|definition| definition.stable_key() == stable_key)
        .unwrap();
    let state = definition
        .states()
        .iter()
        .find(|state| state.kind() == CurioStateKind::Active)
        .unwrap();
    let curios = runtime
        .contributions_from_owned(
            &[(definition.curio(), 1)],
            &[(definition.curio(), state.id())],
            &[(definition.curio(), state.maximum_charges().unwrap_or(0))],
        )
        .unwrap();
    let abilities = RunRuntimeCatalog::compile(catalog)
        .unwrap()
        .ability_contributions(&[])
        .unwrap();
    let projection = AbilityRuntimeCatalog::compile(catalog)
        .unwrap()
        .project(
            &[],
            AbilityExecutionContext::new(
                AbilityProjectionScope::Battle,
                AbilityBoundary::BattleStart,
                3,
                false,
            ),
        )
        .unwrap();
    UniverseBattleContributionCompiler::compile(Arc::clone(catalog))
        .unwrap()
        .compile_snapshot(&path, &blessings, &curios, &abilities, &projection)
        .unwrap()
}
