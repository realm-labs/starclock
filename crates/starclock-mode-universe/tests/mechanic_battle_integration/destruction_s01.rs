use super::*;
use starclock_combat::{
    ParticipantInitialState,
    modifier::model::{FormulaStage, StatKind},
    rule::model::{ProgramStep, RuleEventPoint, RuleOperationTemplate},
};

const VIRTUAL: (&str, u32) = ("universe.blessing.612530", 2);
const ON_HIT: (&str, u32) = ("universe.blessing.612531", 2);
const SHARE: (&str, u32) = ("universe.blessing.612532", 2);
const RETALIATION: (&str, u32) = ("universe.blessing.612540", 2);
const CONSUMPTION: (&str, u32) = ("universe.blessing.612541", 2);
const GRIT_EFFECT_RAW: u32 = 0x79d0_0001;
const VIRTUAL_GRIT_EFFECT_RAW: u32 = 0x79d0_0002;
const GRIT_ENGINE_EFFECT_RAW: u32 = 0x79d0_0003;

#[test]
fn goal07_p2_m07_s01_materializes_all_selected_levels_as_generic_rule_ir() {
    let catalog = catalog();
    let selected = [VIRTUAL, ON_HIT, SHARE, RETALIATION, CONSUMPTION];
    let contributions = contributions_many(
        &catalog,
        "universe.path.destruction",
        &selected,
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    for key in [
        "StageAbility_61253002",
        "StageAbility_61253102",
        "StageAbility_61253202",
        "StageAbility_61254002",
        "StageAbility_61254102",
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
    for raw in [
        GRIT_EFFECT_RAW,
        VIRTUAL_GRIT_EFFECT_RAW,
        GRIT_ENGINE_EFFECT_RAW,
    ] {
        assert!(
            combat
                .effect(starclock_combat::EffectDefinitionId::new(raw).unwrap())
                .is_some(),
            "shared Fighting Spirit effect {raw:#x}"
        );
    }
    assert_eq!(
        combat
            .effect(
                starclock_combat::EffectDefinitionId::new(GRIT_EFFECT_RAW)
                    .expect("reserved Grit effect"),
            )
            .unwrap()
            .runtime_template()
            .unwrap()
            .stack_limit(),
        35,
        "S01 retains the released Fighting Spirit cap"
    );
}

#[test]
fn grit_engine_authors_attack_and_defense_from_effective_stacks() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.destruction",
        &[VIRTUAL],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let engine = combat
        .effect(starclock_combat::EffectDefinitionId::new(GRIT_ENGINE_EFFECT_RAW).unwrap())
        .unwrap();
    let modifiers = engine
        .modifiers()
        .iter()
        .map(|id| combat.modifier(*id).unwrap())
        .collect::<Vec<_>>();
    assert!(modifiers.iter().any(|modifier| {
        modifier.stat == StatKind::Atk && modifier.stage == FormulaStage::PercentOfBase
    }));
    assert!(modifiers.iter().any(|modifier| {
        modifier.stat == StatKind::Def && modifier.stage == FormulaStage::PercentOfBase
    }));
    assert_eq!(modifiers.len(), 2);
}

#[test]
fn enhanced_virtual_grit_and_hp_consumption_execute_in_a_real_battle() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.destruction",
        &[VIRTUAL, CONSUMPTION],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let spec = wounded_players(durable_spec(&materialization, 0x91, false), 30_000, 0x92);
    let (mut battle, started) = start(&materialization, spec, 0x93);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let virtual_grit = battle
        .view()
        .effects_by_id()
        .find(|effect| effect.definition().get() == VIRTUAL_GRIT_EFFECT_RAW)
        .expect("30% HP grants enhanced virtual Grit");
    assert_eq!(virtual_grit.stacks(), 24);
    if battle
        .decision()
        .is_some_and(|decision| decision.kind() == starclock_combat::DecisionKind::InterruptWindow)
    {
        battle
            .apply(Command::PassInterruptWindow {
                decision: battle.decision().unwrap().id(),
            })
            .unwrap();
    }
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { .. }))
        .cloned()
        .expect("player action command");
    let actor = match command {
        Command::UseAbility { actor, .. } => actor,
        _ => unreachable!("filtered command"),
    };
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|candidate| matches!(candidate, Command::UseAbility { actor: value, .. } if *value == actor))
        .cloned()
        .expect("player action actor");
    let before = battle
        .view()
        .units_by_id()
        .find(|unit| unit.id() == actor)
        .unwrap()
        .current_hp()
        .get();
    let resolution = battle.apply(command).unwrap();
    assert!(
        resolution.fault().is_none(),
        "{:?} {:?}",
        resolution.fault(),
        resolution.events()
    );
    let after = battle
        .view()
        .units_by_id()
        .find(|unit| unit.id() == actor)
        .unwrap()
        .current_hp()
        .get();
    assert_eq!(before, 30_000);
    assert_eq!(after, 27_000, "attack start consumes 10% current HP");
}

#[test]
fn share_retaliation_and_consumption_preserve_typed_operation_boundaries() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.destruction",
        &[SHARE, RETALIATION, CONSUMPTION],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    let share = combat
        .rule(binding(&contributions, "StageAbility_61253202").rule())
        .unwrap()
        .runtime()
        .unwrap();
    assert!(share.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::DamageApplied
            && combat
                .program(trigger.program)
                .unwrap()
                .steps()
                .iter()
                .any(|step| {
                    matches!(
                        step,
                        ProgramStep::Operation(RuleOperationTemplate::TrueDamage { .. })
                    )
                })
    }));

    let retaliation = combat
        .rule(binding(&contributions, "StageAbility_61254002").rule())
        .unwrap()
        .runtime()
        .unwrap();
    assert!(retaliation.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::DamageApplied
            && combat
                .program(trigger.program)
                .unwrap()
                .steps()
                .iter()
                .any(|step| {
                    matches!(
                        step,
                        ProgramStep::Operation(RuleOperationTemplate::DamageFromEventElement {
                            can_defeat: false,
                            ..
                        })
                    )
                })
    }));

    let consumption = combat
        .rule(binding(&contributions, "StageAbility_61254102").rule())
        .unwrap()
        .runtime()
        .unwrap();
    assert!(consumption.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::ActionStarted
            && combat
                .program(trigger.program)
                .unwrap()
                .steps()
                .iter()
                .any(|step| match step {
                    ProgramStep::If { then_program, .. } => combat
                        .program(*then_program)
                        .unwrap()
                        .steps()
                        .iter()
                        .any(|step| {
                            matches!(
                                step,
                                ProgramStep::Operation(RuleOperationTemplate::ConsumeHp { .. })
                            )
                        }),
                    _ => false,
                })
    }));

    let spec = durable_spec_with_enemy_speed(
        &materialization,
        0x94,
        false,
        Some(Speed::from_scaled(500_000_000).unwrap()),
    );
    let (mut battle, started) = start(&materialization, spec, 0x95);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = first_normal_action(&mut battle);
    assert!(
        resolution.fault().is_none(),
        "incoming damage sharing executes without recursion: {:?} {:?}",
        resolution.fault(),
        resolution.events()
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
