use super::*;
use starclock_combat::{
    ParticipantInitialState,
    catalog::action::{AbilityKind, AbilityProgramTiming, ReactionBoundary},
    modifier::model::{FormulaPurpose, FormulaStage, StatKind},
    rule::model::{
        ProgramStep, RuleActionPaymentPolicy, RuleEventPoint, RuleOperationTemplate, RuleValue,
        ValueExpr,
    },
};
use super::{destruction_s03};

const LOST_HP_DEFENSE: (&str, u32) = ("universe.blessing.612556", 2);
const LOST_HP_RESISTANCE: (&str, u32) = ("universe.blessing.612557", 2);
const CATACLYSMIC: &str = "universe.resonance.612521";
const ENTROPIC: &str = "universe.resonance.612522";
const EVENT_HORIZON: &str = "universe.resonance.612523";
const CATACLYSMIC_EFFECT_RAW: u32 = 0x79f0_0001;
const ENTROPIC_EFFECT_RAW: u32 = 0x79f0_0002;
const AUTO_RESONANCE_RAW: u32 = 0x79f0_0005;
const MISSING_HP_STACK_SLOT_RAW: u32 = 0x79d2_0001;

#[test]
fn goal07_p2_m07_s04_materializes_all_exact_rules_without_native_handlers() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    for key in [
        "StageAbility_61255601",
        "StageAbility_61255701",
        "StageAbility_612521",
        "StageAbility_612522",
        "StageAbility_612523",
    ] {
        let binding = binding(&contributions, key);
        let rule = combat
            .rule(binding.rule())
            .unwrap_or_else(|| panic!("{key} is executable"));
        assert!(
            rule.runtime()
                .is_some_and(|runtime| runtime.native_handler().is_none()),
            "{key} is generic Rule IR"
        );
    }

    assert_missing_hp_modifier(
        combat,
        binding(&contributions, "StageAbility_61255601"),
        StatKind::Def,
        6_000,
    );
    assert_missing_hp_modifier(
        combat,
        binding(&contributions, "StageAbility_61255701"),
        StatKind::EffectResistance,
        4_500,
    );

    let entropic = combat
        .effect(starclock_combat::EffectDefinitionId::new(ENTROPIC_EFFECT_RAW).expect("effect ID"))
        .expect("Entropic Retribution effect");
    let runtime = entropic.runtime_template().expect("timed debuff");
    assert_eq!(runtime.category(), starclock_combat::EffectCategory::Debuff);
    assert!(matches!(
        runtime.duration_expression(),
        Some(ValueExpr::Literal(RuleValue::Integer(2)))
    ));
    let defense = combat
        .modifier(entropic.modifiers()[0])
        .expect("20% DEF reduction");
    assert_eq!(
        (defense.stat, defense.stage, defense.purpose),
        (
            StatKind::Def,
            FormulaStage::PercentOfBase,
            FormulaPurpose::Stat
        )
    );
    assert!(expression_has_scalar(&defense.value, 200_000));
    assert_eq!(
        combat
            .effect(
                starclock_combat::EffectDefinitionId::new(CATACLYSMIC_EFFECT_RAW)
                    .expect("effect ID")
            )
            .unwrap()
            .runtime_template()
            .unwrap()
            .category(),
        starclock_combat::EffectCategory::Shield
    );
}

#[test]
fn resonance_programs_encode_hp_conversion_entropic_application_and_auto_release() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let resonance = combat
        .ability(AbilityId::new(RESONANCE_ABILITY_RAW).unwrap())
        .expect("manual Destruction Resonance");
    assert_eq!(resonance.action().unwrap().kind(), AbilityKind::Ultimate);
    let automatic = combat
        .ability(AbilityId::new(AUTO_RESONANCE_RAW).unwrap())
        .expect("Event Horizon automatic Resonance");
    assert_eq!(automatic.action().unwrap().kind(), AbilityKind::ExtraAction);

    let before = resonance
        .programs()
        .iter()
        .filter(|binding| binding.timing() == AbilityProgramTiming::BeforeHits)
        .map(|binding| combat.program(binding.program()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(before.len(), 2);
    let all_before_steps = before
        .iter()
        .flat_map(|program| program.steps())
        .collect::<Vec<_>>();
    assert!(all_before_steps.iter().any(|step| {
        matches!(
            step,
            ProgramStep::ForEach {
                body,
                maximum: 16,
                ..
            } if combat.program(*body).unwrap().steps().iter().any(|body_step| {
                matches!(
                    body_step,
                    ProgramStep::Operation(RuleOperationTemplate::ConsumeHp { .. })
                )
            })
        )
    }));
    assert!(all_before_steps.iter().any(|step| {
        matches!(
            step,
            ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                effect,
                base_chance: Some(ValueExpr::Literal(RuleValue::Scalar(chance))),
                ..
            }) if effect.get() == ENTROPIC_EFFECT_RAW && chance.scaled() == 1_500_000
        )
    }));

    let horizon = combat
        .rule(binding(&contributions, "StageAbility_612523").rule())
        .unwrap()
        .runtime()
        .unwrap();
    let trigger = horizon
        .triggers()
        .iter()
        .find(|trigger| trigger.event_point == RuleEventPoint::DamageApplied)
        .expect("attacked ally trigger");
    assert_eq!(
        trigger.once_scope,
        starclock_combat::rule::model::OnceScope::Action
    );
    let queue = combat
        .program(trigger.program)
        .unwrap()
        .steps()
        .iter()
        .find_map(|step| match step {
            ProgramStep::Operation(RuleOperationTemplate::QueueAction {
                ability,
                boundary,
                payment,
                ..
            }) => Some((*ability, *boundary, payment.clone())),
            _ => None,
        })
        .expect("automatic queued action");
    assert_eq!(queue.0.get(), AUTO_RESONANCE_RAW);
    assert_eq!(queue.1, ReactionBoundary::AfterAction);
    assert_eq!(queue.2, Some(RuleActionPaymentPolicy::Suppressed));
}

#[test]
fn charged_resonance_consumes_to_forty_percent_then_shields_and_damages() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let original = durable_spec(&materialization, 0xe1, true);
    let spec = wounded_players(original, 80_000, 0xe2);
    let (mut battle, started) = start(&materialization, spec, 0xe3);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = use_resonance(&mut battle);
    assert!(
        resolution.fault().is_none(),
        "{:?} {:#?}",
        resolution.fault(),
        resolution.events()
    );
    let consumed = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::HpConsumption(data) => Some(data.effective.get()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(consumed, vec![40_000; 4]);
    let shields = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Shield(starclock_combat::ShieldEventData::Applied {
                amount, ..
            }) => Some(amount.get()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        shields.iter().filter(|amount| **amount == 40_000).count(),
        4,
        "Cataclysmic Variable converts each effective 40% HP consumption into an equal shield"
    );
    assert!(resolution.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Effect(starclock_combat::EffectEventData::Applied {
                definition,
                ..
            }) if definition.get() == ENTROPIC_EFFECT_RAW
        )
    }));
    let fire_damage = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data)
                if data.element == Some(starclock_combat::formula::model::CombatElement::Fire) =>
            {
                Some(data.raw.scaled())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(fire_damage.len(), 1);
    assert!(
        fire_damage[0] >= 720_000_000_000,
        "240k current party HP loss × 250% × 120%, before other selected Blessing modifiers"
    );
}

#[test]
fn entropic_retribution_ticks_from_current_party_hp_loss_on_enemy_turn() {
    let catalog = catalog();
    let contributions = contributions_many_with_formations(
        &catalog,
        "universe.path.destruction",
        &[LOST_HP_DEFENSE, LOST_HP_RESISTANCE],
        &[ENTROPIC],
        None,
        false,
    );
    let source = binding(&contributions, "StageAbility_612522")
        .source()
        .definition();
    let materialization = materialize(&catalog, &contributions);
    let original = durable_spec(&materialization, 0xe4, true);
    let spec = wounded_players(original, 50_000, 0xe5);
    let (mut battle, started) = start(&materialization, spec, 0xe6);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let applied = use_resonance(&mut battle);
    assert!(applied.fault().is_none(), "{:?}", applied.fault());

    let mut observed = None;
    for _ in 0..48 {
        if battle.decision().is_none() {
            break;
        }
        let resolution = first_normal_action(&mut battle);
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
        let events = complete_action_events(&mut battle, &resolution);
        observed = events
            .iter()
            .find_map(|event| match event.kind() {
                BattleEventKind::Damage(data)
                    if event.cause().source_definition() == Some(source)
                        && data.element
                            == Some(starclock_combat::formula::model::CombatElement::Fire) =>
                {
                    Some(data.raw.scaled())
                }
                _ => None,
            });
        if observed.is_some() {
            break;
        }
    }
    assert!(
        observed.is_some_and(|amount| amount > 0),
        "Entropic Retribution emits source-attributed Fire Additional DMG at an afflicted enemy turn"
    );
}

#[test]
fn event_horizon_queues_a_free_resonance_after_a_low_hp_ally_is_attacked() {
    let catalog = catalog();
    let contributions = contributions_many_with_formations(
        &catalog,
        "universe.path.destruction",
        &[LOST_HP_DEFENSE, LOST_HP_RESISTANCE],
        &[EVENT_HORIZON],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let spec = destruction_s03::enemy_duel_spec(&materialization, 30_000, 0xe8);
    let (mut battle, started) = start(&materialization, spec, 0xe9);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = first_normal_action(&mut battle);
    assert!(
        resolution.fault().is_none(),
        "{:?} {:#?}",
        resolution.fault(),
        resolution.events()
    );
    assert!(
        resolution.events().iter().any(|event| {
            matches!(
                event.kind(),
                starclock_combat::BattleEventKind::Action(
                    starclock_combat::ActionEventData::Resolved {
                        ability,
                        origin: starclock_combat::ActionOrigin::Forced,
                        ..
                    }
                ) if ability.get() == AUTO_RESONANCE_RAW
            )
        }),
        "{:#?}",
        resolution.events()
    );
    assert_eq!(
        battle.view().team(TeamSide::Player).keyed_resource(
            starclock_combat::SourceDefinitionId::new(RESONANCE_RESOURCE_RAW).unwrap()
        ),
        Some((0, 100)),
        "automatic Resonance neither requires nor consumes Energy"
    );
}

fn full_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many_with_formations(
        catalog,
        "universe.path.destruction",
        &[LOST_HP_DEFENSE, LOST_HP_RESISTANCE],
        &[CATACLYSMIC, ENTROPIC, EVENT_HORIZON],
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

fn assert_missing_hp_modifier(
    combat: &starclock_combat::catalog::CombatCatalog,
    binding: &starclock_mode_universe::battle_contribution::UniverseBattleRuleBinding,
    stat: StatKind,
    ratio: i64,
) {
    let effect = combat
        .effect(
            starclock_combat::EffectDefinitionId::new(0x7660_0000 + binding.rule().get()).unwrap(),
        )
        .unwrap();
    let modifier = combat.modifier(effect.modifiers()[0]).unwrap();
    assert_eq!(
        (modifier.stat, modifier.stage),
        (stat, FormulaStage::PercentOfBase)
    );
    assert_eq!(
        modifier.source_stack_slot,
        starclock_combat::StateSlotDefinitionId::new(MISSING_HP_STACK_SLOT_RAW)
    );
    assert!(expression_has_scalar(&modifier.value, ratio));
}

fn expression_has_scalar(value: &ValueExpr, expected: i64) -> bool {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled() == expected,
        ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => {
            expression_has_scalar(lhs, expected) || expression_has_scalar(rhs, expected)
        }
        ValueExpr::Multiply { lhs, rhs, .. } | ValueExpr::Divide { lhs, rhs, .. } => {
            expression_has_scalar(lhs, expected) || expression_has_scalar(rhs, expected)
        }
        ValueExpr::Negate(value) | ValueExpr::Convert { value, .. } => {
            expression_has_scalar(value, expected)
        }
        _ => false,
    }
}

fn use_resonance(battle: &mut Battle) -> starclock_combat::Resolution {
    use_ready_ability(battle, RESONANCE_ABILITY_RAW)
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
        AssemblyDigest::new([marker; 32]).unwrap(),
        original.encounter(),
        participants,
        original.resources(TeamSide::Player).clone(),
        original.resources(TeamSide::Enemy).clone(),
        original.concede_policy(),
    )
    .unwrap()
}
