use super::*;

use starclock_combat::{
    formula::model::DamageClass,
    modifier::model::{FormulaStage, StatKind},
    rule::model::{RuleEventPoint, RuleOperationTemplate},
};
use starclock_mode_universe::curio::CurioStateKind;

const ASSIGNED: [&str; 7] = [
    "universe.curio.108",
    "universe.curio.115",
    "universe.curio.17",
    "universe.curio.21",
    "universe.curio.45",
    "universe.curio.47",
    "universe.curio.49",
];

#[test]
fn goal07_p3_m12_s01_executes_assigned_states_without_native_handlers() {
    let catalog = catalog();
    let runtime = CurioRuntimeCatalog::compile(&catalog).unwrap();
    for stable_key in ASSIGNED {
        let state = if stable_key == "universe.curio.49" {
            CurioStateKind::Fixed
        } else {
            runtime
                .definitions()
                .iter()
                .find(|definition| definition.stable_key() == stable_key)
                .unwrap()
                .states()
                .iter()
                .find(|state| {
                    state.id()
                        == runtime
                            .definitions()
                            .iter()
                            .find(|definition| definition.stable_key() == stable_key)
                            .unwrap()
                            .initial_state()
                })
                .unwrap()
                .kind()
        };
        let snapshot = contribution_at_state(&catalog, stable_key, state);
        let materialization = materialize(&catalog, &snapshot);
        for binding in snapshot.rules().iter().filter(|binding| {
            binding.role()
                == starclock_mode_universe::battle_contribution::UniverseBattleRuleRole::CurioState
        }) {
            assert!(
                materialization
                    .combat_catalog()
                    .rule(binding.rule())
                    .is_none_or(|rule| rule
                        .runtime()
                        .is_none_or(|runtime| runtime.native_handler().is_none()))
            );
        }
    }
}

#[test]
fn odd_code_repairing_consumes_thirty_percent_current_hp_after_ultimate() {
    let catalog = catalog();
    let snapshot = contribution_at_state(&catalog, "universe.curio.47", CurioStateKind::Repairing);
    let roster = roster_for_forms_with_ability_kinds_and_energy(
        &catalog,
        [1, 2, 3, 4],
        None,
        &[AbilityKind::Ultimate],
        true,
        1_000_000_000,
    );
    let materialization = materialize_with_roster(&catalog, &roster, &snapshot);
    let (mut battle, _) = start(
        &materialization,
        durable_spec(&materialization, 0xe3, false),
        0xe4,
    );
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| match command {
            Command::UseAbility { ability, .. } | Command::UseInterrupt { ability, .. } => {
                materialization
                    .combat_catalog()
                    .ability(*ability)
                    .and_then(|definition| definition.action())
                    .is_some_and(|action| action.kind() == AbilityKind::Ultimate)
            }
            _ => false,
        })
        .unwrap()
        .clone();
    let actor = match command {
        Command::UseAbility { actor, .. } | Command::UseInterrupt { actor, .. } => actor,
        _ => unreachable!(),
    };
    battle.apply(command).unwrap();
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .find(|unit| unit.id() == actor)
            .unwrap()
            .current_hp()
            .get(),
        70_000
    );
}

#[test]
fn fission_cuckoo_clock_applies_the_exact_five_percent_attack_penalty() {
    let catalog = catalog();
    let plain = contributions(&catalog, "universe.path.erudition", None, None, false);
    let fission = contributions(
        &catalog,
        "universe.path.erudition",
        None,
        Some("universe.curio.108"),
        false,
    );
    let plain = materialize(&catalog, &plain);
    let fission = materialize(&catalog, &fission);
    let (mut plain_battle, _) = start(&plain, durable_spec(&plain, 0xe5, false), 0xe6);
    let (mut fission_battle, _) = start(&fission, durable_spec(&fission, 0xe5, false), 0xe6);
    let plain_damage = direct_damage(&first_normal_action(&mut plain_battle));
    let fission_damage = direct_damage(&first_normal_action(&mut fission_battle));
    assert!(plain_damage > 0);
    assert_eq!(fission_damage, plain_damage * 95 / 100);
}

#[test]
fn code_state_rules_retain_exact_repairing_and_fixed_operations() {
    let catalog = catalog();
    let repairing = contribution_at_state(&catalog, "universe.curio.45", CurioStateKind::Repairing);
    let fixed = contribution_at_state(&catalog, "universe.curio.45", CurioStateKind::Fixed);
    for (snapshot, expected_maximum) in [(&repairing, false), (&fixed, true)] {
        let materialization = materialize(&catalog, snapshot);
        let rule = snapshot
            .rules()
            .iter()
            .find(|binding| binding.source_binding_key() == Some("45"))
            .and_then(|binding| materialization.combat_catalog().rule(binding.rule()))
            .unwrap();
        let runtime = rule.runtime().unwrap();
        assert!(
            runtime
                .triggers()
                .iter()
                .any(|trigger| trigger.event_point == RuleEventPoint::WeaknessBroken)
        );
        let operation = rule
            .programs()
            .iter()
            .filter_map(|program| materialization.combat_catalog().program(*program))
            .flat_map(|program| program.steps())
            .find_map(|step| match step {
                starclock_combat::rule::model::ProgramStep::Operation(
                    operation @ RuleOperationTemplate::ModifyResource { .. },
                ) => Some(operation),
                _ => None,
            })
            .unwrap();
        assert_eq!(
            matches!(
                operation,
                RuleOperationTemplate::ModifyResource {
                    amount: starclock_combat::rule::model::ValueExpr::QueryMaximumEnergy(_),
                    ..
                }
            ),
            expected_maximum
        );
    }

    let reduction = contribution_at_state(&catalog, "universe.curio.49", CurioStateKind::Fixed);
    let materialization = materialize(&catalog, &reduction);
    let rule = reduction
        .rules()
        .iter()
        .find(|binding| binding.source_binding_key() == Some("49"))
        .and_then(|binding| materialization.combat_catalog().rule(binding.rule()))
        .unwrap();
    let modifiers = rule
        .programs()
        .iter()
        .filter_map(|program| materialization.combat_catalog().program(*program))
        .flat_map(|program| program.effects())
        .filter_map(|effect| materialization.combat_catalog().effect(*effect))
        .flat_map(|effect| effect.modifiers())
        .filter_map(|modifier| materialization.combat_catalog().modifier(*modifier))
        .collect::<Vec<_>>();
    assert_eq!(
        modifiers
            .iter()
            .filter(|modifier| {
                modifier.stage == FormulaStage::Mitigation
                    && modifier.value
                        == starclock_combat::rule::model::ValueExpr::Literal(
                            starclock_combat::rule::model::RuleValue::Scalar(
                                starclock_combat::Scalar::from_scaled(500_000),
                            ),
                        )
            })
            .count(),
        8
    );
    assert!(
        modifiers
            .iter()
            .any(|modifier| modifier.stat == StatKind::Hp)
    );
}

fn contribution_at_state(
    catalog: &Arc<UniverseCatalog>,
    stable_key: &str,
    kind: CurioStateKind,
) -> UniverseBattleContributionSet {
    let path_definition = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.erudition")
        .unwrap();
    let owned = path_definition
        .blessings()
        .iter()
        .take(3)
        .map(|blessing| (*blessing, 1))
        .collect::<Vec<_>>();
    let blessings = BlessingRuntimeCatalog::compile(catalog)
        .unwrap()
        .contributions_from_owned(&owned)
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
        .find(|state| state.kind() == kind)
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

fn direct_damage(resolution: &starclock_combat::Resolution) -> i64 {
    resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) if data.class == DamageClass::Direct => {
                Some(data.raw.scaled())
            }
            _ => None,
        })
        .sum()
}
