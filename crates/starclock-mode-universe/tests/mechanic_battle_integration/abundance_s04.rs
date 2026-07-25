use super::*;
use starclock_combat::{
    EffectApplicationGuard, EffectDamageGuard, ParticipantInitialState,
    catalog::action::AbilityKind,
    modifier::model::{FormulaStage, StatKind},
    rule::model::{OnceScope, RuleEventPoint},
};

const FORCE_VICTOIRE: (&str, u32) = ("universe.blessing.612356", 2);
const EMPOWER: (&str, u32) = ("universe.blessing.612357", 2);
const TERMINAL_NIRVANA: &str = "universe.resonance.612321";
const ANICCA: &str = "universe.resonance.612322";
const ANATTA: &str = "universe.resonance.612323";
const AUTO_ABILITY: u32 = 0x7930_0001;
const TERMINAL_ABILITY: u32 = 0x7930_0002;
const MAX_HP_EFFECT: u32 = 0x7930_0007;
const SUBDUING_EVILS_EFFECT: u32 = 0x7930_000a;
const ANATTA_COUNTDOWN_CODE: u32 = 0x7930_000b;

#[test]
fn goal07_p2_m05_s04_materializes_every_assigned_mechanic_without_native_handlers() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    for key in [
        "StageAbility_612356",
        "StageAbility_612357",
        "StageAbility_612321",
        "StageAbility_612322",
        "StageAbility_612323",
    ] {
        let binding = contributions
            .rules()
            .iter()
            .find(|binding| binding.source_binding_key() == Some(key))
            .unwrap_or_else(|| panic!("{key} selected"));
        let rule = combat
            .rule(binding.rule())
            .unwrap_or_else(|| panic!("{key} has an executable rule"));
        assert!(
            rule.runtime()
                .is_some_and(|runtime| runtime.native_handler().is_none()),
            "{key} remains generic Rule IR"
        );
    }
    assert!(
        contributions
            .rules()
            .iter()
            .any(|binding| binding.source_binding_key() == Some("StageAbility_612320"))
    );

    let speed = binding(&contributions, "StageAbility_612356");
    let speed_effect = combat
        .effect(
            combat
                .program(combat.rule(speed.rule()).unwrap().programs()[0])
                .unwrap()
                .effects()[0],
        )
        .unwrap();
    let speed_modifier = combat.modifier(speed_effect.modifiers()[0]).unwrap();
    assert_eq!(
        (
            speed_modifier.stat,
            speed_modifier.stage,
            literal_scalar(&speed_modifier.value)
        ),
        (StatKind::Spd, FormulaStage::PercentOfBase, 150_000)
    );

    let terminal = binding(&contributions, "StageAbility_612321");
    let terminal_effect = combat
        .program(combat.rule(terminal.rule()).unwrap().programs()[0])
        .unwrap()
        .effects()[0];
    assert_eq!(
        combat
            .effect(terminal_effect)
            .unwrap()
            .runtime_template()
            .unwrap()
            .damage_guard(),
        EffectDamageGuard::TeamDefeatOnce
    );
    assert_eq!(
        combat
            .effect(starclock_combat::EffectDefinitionId::new(SUBDUING_EVILS_EFFECT).unwrap())
            .unwrap()
            .runtime_template()
            .unwrap()
            .application_guard(),
        EffectApplicationGuard::NegativeEffectOnce
    );
    assert!(
        combat
            .ability(AbilityId::new(AUTO_ABILITY).unwrap())
            .is_some_and(|ability| ability.action().unwrap().kind() == AbilityKind::Countdown)
    );
    assert!(
        combat
            .ability(AbilityId::new(TERMINAL_ABILITY).unwrap())
            .is_some_and(|ability| ability.action().unwrap().kind() == AbilityKind::ExtraAction)
    );
    assert!(combat.countdown(ANATTA_COUNTDOWN_CODE).is_some());

    let empower = binding(&contributions, "StageAbility_612357");
    let empower_triggers = combat
        .rule(empower.rule())
        .unwrap()
        .runtime()
        .unwrap()
        .triggers();
    assert!(empower_triggers.iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::HealApplied
            && trigger.once_scope == OnceScope::Action
    }));
    assert!(
        combat
            .rule(empower.rule())
            .unwrap()
            .programs()
            .iter()
            .flat_map(|program| combat.program(*program).unwrap().steps())
            .any(|step| {
                matches!(
                    step,
                    starclock_combat::rule::model::ProgramStep::Operation(
                        starclock_combat::rule::model::RuleOperationTemplate::ApplyEffect {
                            chance: starclock_combat::rule::model::RuleEffectChancePolicy::Fixed,
                            base_chance: Some(starclock_combat::rule::model::ValueExpr::Literal(
                                starclock_combat::rule::model::RuleValue::Scalar(value)
                            )),
                            ..
                        }
                    ) if value.scaled() == 450_000
                )
            })
    );
}

#[test]
fn abundance_resonance_heals_buffs_cleanses_and_installs_anatta_actor() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let spec = wounded_players(durable_spec(&materialization, 0xd0, true), 10_000, 0xd1);
    let (mut battle, started) = start(&materialization, spec, 0xd2);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = use_resonance(&mut battle);
    assert!(
        resolution.fault().is_none(),
        "{:?} {:?}",
        resolution.fault(),
        resolution.events()
    );
    assert_eq!(
        resolution
            .events()
            .iter()
            .filter(|event| {
                event
                    .cause()
                    .source_definition()
                    .is_some_and(|source| source.get() == RESONANCE_ABILITY_RAW)
                    && matches!(event.kind(), BattleEventKind::Heal(_))
            })
            .count(),
        4
    );
    assert!(resolution.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Unit(starclock_combat::UnitEventData::CountdownCreated {
                ability,
                ..
            }) if ability.get() == AUTO_ABILITY
        )
    }));
    for player in battle
        .view()
        .units_by_id()
        .filter(|unit| unit.side() == TeamSide::Player)
    {
        let effects = battle
            .view()
            .effects_by_id()
            .filter(|effect| effect.target() == player.id())
            .map(|effect| effect.definition().get())
            .collect::<Vec<_>>();
        assert!(effects.contains(&MAX_HP_EFFECT));
        assert!(effects.contains(&SUBDUING_EVILS_EFFECT));
    }
    assert_eq!(
        battle.view().team(TeamSide::Player).keyed_resource(
            starclock_combat::SourceDefinitionId::new(RESONANCE_RESOURCE_RAW).unwrap()
        ),
        Some((0, 100))
    );
}

fn full_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many_with_formations(
        catalog,
        "universe.path.abundance",
        &[FORCE_VICTOIRE, EMPOWER],
        &[TERMINAL_NIRVANA, ANICCA, ANATTA],
        None,
        false,
    )
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
                Command::UseAbility { ability, .. } | Command::UseInterrupt { ability, .. }
                    if ability.get() == RESONANCE_ABILITY_RAW
            )
        })
        .unwrap_or_else(|| panic!("charged resonance is legal"))
        .clone();
    battle.apply(command).unwrap()
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

fn literal_scalar(value: &starclock_combat::rule::model::ValueExpr) -> i64 {
    match value {
        starclock_combat::rule::model::ValueExpr::Literal(
            starclock_combat::rule::model::RuleValue::Scalar(value),
        ) => value.scaled(),
        _ => panic!("expected scalar literal"),
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
