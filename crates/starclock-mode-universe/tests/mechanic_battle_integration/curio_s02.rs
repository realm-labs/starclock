use super::*;
use starclock_combat::{
    EffectDefinitionId, ModifierDefinitionId, ParticipantInitialState,
    modifier::model::{FormulaStage, StatKind, StatQuerySubject},
    rule::model::{ProgramStep, RuleEventPoint, RuleOperationTemplate, RuleValue, ValueExpr},
};

const CAVITY_RUNTIME_KEY: u64 = u64::MAX - 3;
const LOCAL_EFFECT_BASE: u32 = 0x77f3_0000;
const LOCAL_MODIFIER_BASE: u32 = 0x77f5_0000;

#[test]
fn goal07_p3_m11_s02_executes_every_assigned_curio_without_native_handlers() {
    let catalog = catalog();
    let runtime = CurioRuntimeCatalog::compile(&catalog).unwrap();
    for stable_key in [
        "universe.curio.112",
        "universe.curio.113",
        "universe.curio.118",
        "universe.curio.12",
        "universe.curio.120",
        "universe.curio.121",
        "universe.curio.122",
        "universe.curio.123",
    ] {
        let definition = runtime
            .definitions()
            .iter()
            .find(|definition| definition.stable_key() == stable_key)
            .expect("assigned Curio");
        let snapshot = contributions(
            &catalog,
            "universe.path.abundance",
            None,
            Some(stable_key),
            false,
        );
        let binding = snapshot
            .rules()
            .iter()
            .find(|binding| {
                binding.source_binding_key()
                    == definition
                        .states()
                        .iter()
                        .find(|state| state.id() == definition.initial_state())
                        .map(|state| state.source_effect_id())
            })
            .expect("Curio state binding");
        let materialization = materialize(&catalog, &snapshot);
        assert!(
            materialization
                .combat_catalog()
                .rule(binding.rule())
                .is_none_or(|rule| rule
                    .runtime()
                    .is_none_or(|runtime| runtime.native_handler().is_none())),
            "{stable_key} remains generic Rule IR"
        );
    }
}

#[test]
fn cavity_capture_materializes_exact_critical_damage_fixture() {
    let catalog = catalog();
    let contributions = contributions_many_with_curio_runtime(
        &catalog,
        "universe.path.abundance",
        &[],
        &[],
        Some("universe.curio.113"),
        false,
        0,
        0,
        &[(CAVITY_RUNTIME_KEY, 2)],
    );
    let materialization = materialize(&catalog, &contributions);
    let binding = binding(&contributions, "85");
    let modifier = materialization
        .combat_catalog()
        .modifier(local_modifier(binding.rule().get(), 1))
        .expect("Cavity CRIT DMG modifier");
    assert_eq!(
        (
            modifier.stat,
            modifier.stage,
            literal_scalar(&modifier.value)
        ),
        (StatKind::CritDamage, FormulaStage::Flat, 480_000)
    );
}

#[test]
fn illusory_automaton_heals_the_current_actor_for_twenty_percent_maximum_hp() {
    let catalog = catalog();
    let contributions = contributions(
        &catalog,
        "universe.path.abundance",
        None,
        Some("universe.curio.118"),
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let spec = wounded_players(durable_spec(&materialization, 0xc1, false), 50_000, 0xc2);
    let (battle, resolution) = start(&materialization, spec, 0xc3);
    assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    let actor = resolution
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Turn(starclock_combat::TurnEventData::Started { owner, .. }) => {
                Some(*owner)
            }
            _ => None,
        })
        .expect("first turn starts");
    let unit = battle
        .view()
        .units_by_id()
        .find(|unit| unit.id() == actor)
        .unwrap();
    assert_eq!(unit.current_hp().get(), 70_000);
}

#[test]
fn thalan_toxi_flame_uses_highest_attack_marker_hp_cost_and_five_stack_speed() {
    let catalog = catalog();
    let contributions = contributions(
        &catalog,
        "universe.path.abundance",
        None,
        Some("universe.curio.121"),
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let binding = binding(&contributions, "89");
    let raw = binding.rule().get();
    let combat = materialization.combat_catalog();
    let speed_effect = combat
        .effect(EffectDefinitionId::new(LOCAL_EFFECT_BASE + (raw & 0xffff) * 16 + 2).unwrap())
        .expect("Toxi speed effect");
    assert_eq!(speed_effect.runtime_template().unwrap().stack_limit(), 5);
    let modifier = combat.modifier(speed_effect.modifiers()[0]).unwrap();
    assert_eq!(
        (modifier.stat, modifier.stage),
        (StatKind::Spd, FormulaStage::PercentOfBase)
    );
    assert!(expression_has_scalar(&modifier.value, 50_000));
    let rule = combat.rule(binding.rule()).unwrap().runtime().unwrap();
    let turn_trigger = rule
        .triggers()
        .iter()
        .find(|trigger| trigger.event_point == RuleEventPoint::TurnStarted)
        .expect("turn trigger");
    let program = combat.program(turn_trigger.program).unwrap();
    assert!(program.steps().iter().any(|step| {
        matches!(
            step,
            ProgramStep::Operation(RuleOperationTemplate::ConsumeHp {
                amount: ValueExpr::Multiply { lhs, .. },
                floor,
                ..
            })
                if matches!(
                    lhs.as_ref(),
                    ValueExpr::QueryStat {
                        subject: StatQuerySubject::Actor,
                        stat: StatKind::Hp,
                        ..
                    }
                ) && literal_scalar(floor) == 1_000_000
        )
    }));
    assert!(program.steps().iter().any(|step| {
        matches!(
            step,
            ProgramStep::Operation(RuleOperationTemplate::ApplyEffect { effect, .. })
                if *effect == speed_effect.id()
        )
    }));
}

#[test]
fn pinkest_collision_counts_distinct_owned_blessing_paths() {
    let catalog = catalog();
    let contributions = contributions(
        &catalog,
        "universe.path.abundance",
        None,
        Some("universe.curio.122"),
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let binding = binding(&contributions, "90");
    let modifier = materialization
        .combat_catalog()
        .modifier(local_modifier(binding.rule().get(), 2))
        .expect("Break Effect modifier");
    assert_eq!(
        (
            modifier.stat,
            modifier.stage,
            literal_scalar(&modifier.value)
        ),
        (StatKind::BreakEffect, FormulaStage::Flat, 200_000)
    );
}

fn binding<'a>(
    contributions: &'a UniverseBattleContributionSet,
    effect: &str,
) -> &'a starclock_mode_universe::battle_contribution::UniverseBattleRuleBinding {
    contributions
        .rules()
        .iter()
        .find(|binding| binding.source_binding_key() == Some(effect))
        .unwrap_or_else(|| panic!("Curio effect {effect} selected"))
}

fn local_modifier(raw: u32, offset: u32) -> ModifierDefinitionId {
    ModifierDefinitionId::new(LOCAL_MODIFIER_BASE + (raw & 0xffff) * 16 + offset).unwrap()
}

fn literal_scalar(value: &ValueExpr) -> i64 {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled(),
        _ => panic!("expected scalar literal, got {value:?}"),
    }
}

fn expression_has_scalar(value: &ValueExpr, expected: i64) -> bool {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled() == expected,
        ValueExpr::Multiply { lhs, rhs, .. }
        | ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs) => {
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
