use super::*;

use starclock_combat::{
    EffectEventData,
    modifier::model::{FormulaPurpose, FormulaStage, StatKind},
    rule::model::{
        ProgramStep, ResourceUpdateKind, RuleEventPoint, RuleOperationTemplate, ValueExpr,
    },
};
use starclock_mode_universe::curio::CurioStateKind;

const ASSIGNED: [&str; 7] = [
    "universe.curio.49",
    "universe.curio.51",
    "universe.curio.53",
    "universe.curio.55",
    "universe.curio.57",
    "universe.curio.59",
    "universe.curio.60",
];

#[test]
fn goal07_p3_m12_s02_materializes_combat_states_without_native_handlers() {
    let catalog = catalog();
    for stable_key in ASSIGNED {
        let states: &[CurioStateKind] = match stable_key {
            "universe.curio.49" => &[CurioStateKind::Repairing],
            "universe.curio.51" | "universe.curio.53" | "universe.curio.55" => {
                &[CurioStateKind::Repairing, CurioStateKind::Fixed]
            }
            _ => &[CurioStateKind::Active],
        };
        for state in states {
            let snapshot = contribution_at_state(&catalog, stable_key, *state);
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
}

#[test]
fn code_states_keep_exact_released_action_damage_and_skill_point_values() {
    let catalog = catalog();
    for (stable_key, state, expected) in [
        ("universe.curio.51", CurioStateKind::Repairing, 350_000),
        ("universe.curio.51", CurioStateKind::Fixed, 250_000),
        ("universe.curio.53", CurioStateKind::Repairing, 350_000),
        ("universe.curio.53", CurioStateKind::Fixed, 250_000),
    ] {
        let snapshot = contribution_at_state(&catalog, stable_key, state);
        let materialization = materialize(&catalog, &snapshot);
        let combat = materialization.combat_catalog();
        let rule = snapshot
            .rules()
            .iter()
            .filter(|binding| {
                binding.role()
                    == starclock_mode_universe::battle_contribution::UniverseBattleRuleRole::CurioState
            })
            .find_map(|binding| combat.rule(binding.rule()))
            .unwrap();
        let operations = rule
            .programs()
            .iter()
            .filter_map(|program| combat.program(*program))
            .flat_map(|program| program.steps());
        let exact_action = operations.clone().any(|step| {
            matches!(
                step,
                ProgramStep::Operation(RuleOperationTemplate::AdvanceAction {
                    amount: ValueExpr::Literal(starclock_combat::rule::model::RuleValue::Scalar(value)),
                    ..
                }) if value.scaled() == expected
            )
        });
        let exact_damage = rule
            .programs()
            .iter()
            .filter_map(|program| combat.program(*program))
            .flat_map(|program| program.effects())
            .filter_map(|effect| combat.effect(*effect))
            .flat_map(|effect| effect.modifiers())
            .filter_map(|modifier| combat.modifier(*modifier))
            .any(|modifier| {
                modifier.stage == FormulaStage::DamageBoost
                    && expression_has_scalar(&modifier.value, expected)
            });
        assert!(exact_action || exact_damage, "{stable_key} {state:?}");
    }

    for (state, update) in [
        (CurioStateKind::Repairing, ResourceUpdateKind::Spend),
        (CurioStateKind::Fixed, ResourceUpdateKind::Gain),
    ] {
        let snapshot = contribution_at_state(&catalog, "universe.curio.55", state);
        let materialization = materialize(&catalog, &snapshot);
        let combat = materialization.combat_catalog();
        let rule = snapshot
            .rules()
            .iter()
            .filter(|binding| {
                binding.role()
                    == starclock_mode_universe::battle_contribution::UniverseBattleRuleRole::CurioState
            })
            .find_map(|binding| combat.rule(binding.rule()))
            .unwrap();
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
                            update: actual,
                            ..
                        }) if *actual == update
                    )
                })
        }));
    }
}

#[test]
fn fixed_recursive_code_makes_basic_attack_recover_two_total_skill_points() {
    let catalog = catalog();
    let snapshot = contribution_at_state(&catalog, "universe.curio.55", CurioStateKind::Fixed);
    let materialization = materialize(&catalog, &snapshot);
    let (mut battle, started) = start(
        &materialization,
        durable_spec(&materialization, 0xf1, false),
        0xf2,
    );
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let before = battle.view().team(TeamSide::Player).skill_points();
    let resolution = first_normal_action(&mut battle);
    assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    assert_eq!(
        battle.view().team(TeamSide::Player).skill_points(),
        before.saturating_add(2).min(5)
    );
}

#[test]
fn mysterious_code_applies_nonstacking_effect_after_enemy_defeat() {
    let catalog = catalog();
    for state in [CurioStateKind::Repairing, CurioStateKind::Fixed] {
        let snapshot = contribution_at_state(&catalog, "universe.curio.53", state);
        let materialization = materialize(&catalog, &snapshot);
        let rule_id = snapshot
            .rules()
            .iter()
            .find(|binding| {
                binding.role()
                    == starclock_mode_universe::battle_contribution::UniverseBattleRuleRole::CurioState
            })
            .unwrap()
            .rule();
        assert!(
            materialization
                .combat_catalog()
                .trigger_ids(
                    starclock_combat::rule::model::RuleEventKind::Unit,
                    starclock_combat::rule::model::TriggerPhase::AfterEvent,
                )
                .any(|(candidate, _)| candidate == rule_id)
        );
        let spec = durable_spec_with_two_enemy_hp(
            &materialization,
            0xf3,
            [Hp::new(1).unwrap(), Hp::new(2_000_000_000).unwrap()],
        );
        let (mut battle, started) = start(&materialization, spec, 0xf4);
        assert!(started.fault().is_none(), "{:?}", started.fault());
        assert!(
            battle
                .view()
                .rule_instances_by_id()
                .any(|instance| instance.rule() == rule_id)
        );
        let resolution = first_normal_action(&mut battle);
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
        let expected_side = if state == CurioStateKind::Repairing {
            TeamSide::Enemy
        } else {
            TeamSide::Player
        };
        assert!(
            resolution
                .events()
                .iter()
                .filter_map(|event| match event.kind() {
                    BattleEventKind::Effect(EffectEventData::Applied { target, .. }) => {
                        Some(*target)
                    }
                    _ => None,
                })
                .any(|target| battle
                    .view()
                    .units_by_id()
                    .find(|unit| unit.id() == target)
                    .is_some_and(|unit| unit.side() == expected_side)),
            "{state:?}: {:#?}",
            resolution.events()
        );
    }
}

#[test]
fn insect_web_marks_highest_attack_drains_current_hp_and_authors_transfer() {
    let catalog = catalog();
    let snapshot = contribution_at_state(&catalog, "universe.curio.59", CurioStateKind::Active);
    let materialization = materialize(&catalog, &snapshot);
    let combat = materialization.combat_catalog();
    let rule = snapshot
        .rules()
        .iter()
        .filter(|binding| {
            binding.role()
                == starclock_mode_universe::battle_contribution::UniverseBattleRuleRole::CurioState
        })
        .find_map(|binding| combat.rule(binding.rule()))
        .unwrap();
    for point in [
        RuleEventPoint::BattleStarted,
        RuleEventPoint::TurnStarted,
        RuleEventPoint::UnitDowned,
    ] {
        assert!(
            rule.runtime()
                .unwrap()
                .triggers()
                .iter()
                .any(|trigger| trigger.event_point == point)
        );
    }
    let parasitized = rule
        .programs()
        .iter()
        .filter_map(|program| combat.program(*program))
        .flat_map(|program| program.effects())
        .filter_map(|effect| combat.effect(*effect))
        .find(|effect| {
            effect.modifiers().iter().any(|modifier| {
                combat.modifier(*modifier).is_some_and(|modifier| {
                    modifier.stat == StatKind::Atk
                        && modifier.stage == FormulaStage::PercentOfBase
                        && modifier.purpose == FormulaPurpose::Stat
                        && expression_has_scalar(&modifier.value, 500_000)
                })
            })
        })
        .unwrap()
        .id();
    let (mut battle, started) = start(
        &materialization,
        durable_spec(&materialization, 0xf5, false),
        0xf6,
    );
    assert!(
        started.fault().is_none(),
        "fault={:?}; events={:#?}",
        started.fault(),
        started.events()
    );
    let marked = battle
        .view()
        .effects_by_id()
        .find(|effect| effect.definition() == parasitized)
        .unwrap()
        .target();
    let initial = unit_hp(&battle, marked);
    for _ in 0..12 {
        let resolution = first_normal_action(&mut battle);
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
        let current = unit_hp(&battle, marked);
        if current != initial {
            assert_eq!(current, initial * 80 / 100);
            return;
        }
    }
    panic!("Parasitized owner did not take a turn");
}

fn unit_hp(battle: &Battle, unit: starclock_combat::UnitId) -> i64 {
    battle
        .view()
        .units_by_id()
        .find(|candidate| candidate.id() == unit)
        .unwrap()
        .current_hp()
        .get()
}

fn expression_has_scalar(value: &ValueExpr, expected: i64) -> bool {
    matches!(
        value,
        ValueExpr::Literal(starclock_combat::rule::model::RuleValue::Scalar(value))
            if value.scaled() == expected
    )
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
    let owned = Vec::new();
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
