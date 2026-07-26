use super::*;
use starclock_combat::{
    ParticipantInitialState,
    modifier::model::{FormulaPurpose, FormulaStage, StatKind},
    rule::model::{ProgramStep, RuleEventPoint, RuleOperationTemplate, RuleValue, ValueExpr},
};

const MITIGATION: (&str, u32) = ("universe.blessing.612542", 2);
const LOST_HP_STATS: (&str, u32) = ("universe.blessing.612543", 2);
const LOW_HP_DAMAGE: (&str, u32) = ("universe.blessing.612544", 2);
const LOW_HP_HEALING: (&str, u32) = ("universe.blessing.612545", 2);
const ULTIMATE_SHIELD: (&str, u32) = ("universe.blessing.612546", 2);
const BLESSING_ATTACK: (&str, u32) = ("universe.blessing.612550", 1);
const GRIT_EFFECT_RAW: u32 = 0x79d0_0001;
const GRIT_ENGINE_EFFECT_RAW: u32 = 0x79d0_0003;
const GRIT_STACK_SLOT_RAW: u32 = 0x79d0_0004;
const MISSING_HP_STACK_SLOT_RAW: u32 = 0x79d2_0001;

#[test]
fn goal07_p2_m07_s02_materializes_every_assigned_level_as_generic_rule_ir() {
    let catalog = catalog();
    let contributions = s02_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    for key in [
        "StageAbility_61254202",
        "StageAbility_61254302",
        "StageAbility_61254401",
        "StageAbility_61254502",
        "StageAbility_61254602",
        "StageAbility_61255001",
    ] {
        let rule = combat
            .rule(binding(&contributions, key).rule())
            .unwrap_or_else(|| panic!("{key} is executable"));
        assert!(
            rule.runtime()
                .is_some_and(|runtime| runtime.native_handler().is_none()),
            "{key} remains generic Rule IR"
        );
    }

    let grit = combat
        .effect(starclock_combat::EffectDefinitionId::new(GRIT_EFFECT_RAW).unwrap())
        .unwrap();
    assert_eq!(grit.runtime_template().unwrap().stack_limit(), 45);
    let engine = combat
        .effect(starclock_combat::EffectDefinitionId::new(GRIT_ENGINE_EFFECT_RAW).unwrap())
        .unwrap();
    let mitigation = engine
        .modifiers()
        .iter()
        .map(|id| combat.modifier(*id).unwrap())
        .filter(|modifier| modifier.stage == FormulaStage::Mitigation)
        .collect::<Vec<_>>();
    assert_eq!(mitigation.len(), 7);
    assert!(mitigation.iter().all(|modifier| {
        modifier.source_stack_slot
            == starclock_combat::StateSlotDefinitionId::new(GRIT_STACK_SLOT_RAW)
    }));
}

#[test]
fn lost_hp_stats_and_low_hp_damage_keep_exact_dynamic_modifier_semantics() {
    let catalog = catalog();
    let contributions = s02_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    let lost = combat
        .rule(binding(&contributions, "StageAbility_61254302").rule())
        .unwrap();
    let lost_runtime = lost.runtime().unwrap();
    assert!(lost_runtime.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::HpChanged && trigger.filter.target_selector.is_some()
    }));
    let lost_effect = combat
        .effect(effect_id(binding(&contributions, "StageAbility_61254302")))
        .unwrap();
    let lost_modifiers = lost_effect
        .modifiers()
        .iter()
        .filter_map(|id| combat.modifier(*id))
        .collect::<Vec<_>>();
    assert!(lost_modifiers.iter().any(|modifier| {
        modifier.stat == StatKind::Atk
            && modifier.stage == FormulaStage::PercentOfBase
            && scalar_factor(&modifier.value) == Some(8_000)
    }));
    assert!(lost_modifiers.iter().any(|modifier| {
        modifier.stat == StatKind::Def
            && modifier.stage == FormulaStage::PercentOfBase
            && scalar_factor(&modifier.value) == Some(5_000)
    }));
    assert!(lost_modifiers.iter().all(|modifier| {
        modifier.source_stack_slot
            == starclock_combat::StateSlotDefinitionId::new(MISSING_HP_STACK_SLOT_RAW)
    }));

    let low_damage_effect = combat
        .effect(effect_id(binding(&contributions, "StageAbility_61254401")))
        .unwrap();
    let damage_modifiers = low_damage_effect
        .modifiers()
        .iter()
        .filter_map(|id| combat.modifier(*id))
        .filter(|modifier| {
            modifier.stage == FormulaStage::DamageBoost
                && matches!(
                    modifier.purpose,
                    FormulaPurpose::OrdinaryDamage
                        | FormulaPurpose::Dot
                        | FormulaPurpose::Break
                        | FormulaPurpose::SuperBreak
                        | FormulaPurpose::AdditionalDamage
                        | FormulaPurpose::JointDamage
                        | FormulaPurpose::ElationDamage
                )
        })
        .collect::<Vec<_>>();
    assert!(damage_modifiers.len() >= 7);

    let attack_effect = combat
        .effect(effect_id(binding(&contributions, "StageAbility_61255001")))
        .unwrap();
    let attack = combat
        .modifier(attack_effect.modifiers()[0])
        .expect("Blessing-count ATK modifier");
    assert_eq!(attack.stat, StatKind::Atk);
    assert_eq!(attack.stage, FormulaStage::PercentOfBase);
    assert_eq!(literal_scalar(&attack.value), Some(300_000));
}

#[test]
fn healing_and_ultimate_shield_use_typed_bounded_operations() {
    let catalog = catalog();
    let contributions = s02_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    let healing = combat
        .rule(binding(&contributions, "StageAbility_61254502").rule())
        .unwrap()
        .runtime()
        .unwrap();
    assert!(healing.state_slots().iter().any(|slot| {
        slot.kind() == starclock_combat::rule::model::RuleValueKind::Scalar
            && slot.scope() == starclock_combat::rule::model::BattleRuleScope::Battle
    }));
    assert!(healing.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::DamageApplied
            && combat
                .program(trigger.program)
                .unwrap()
                .steps()
                .iter()
                .any(|step| {
                    matches!(
                        step,
                        ProgramStep::Operation(RuleOperationTemplate::Heal {
                            apply_formula_modifiers: false,
                            ..
                        })
                    )
                })
    }));

    let shield = combat
        .rule(binding(&contributions, "StageAbility_61254602").rule())
        .unwrap()
        .runtime()
        .unwrap();
    assert!(shield.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::ActionResolved
            && trigger.filter.ability_tag
                == Some(starclock_combat::catalog::action::AbilityTag::Ultimate)
            && combat
                .program(trigger.program)
                .unwrap()
                .steps()
                .iter()
                .any(|step| {
                    matches!(
                        step,
                        ProgramStep::Operation(RuleOperationTemplate::Shield { .. })
                    )
                })
    }));
}

#[test]
fn s02_rules_execute_without_fault_in_a_wounded_production_battle() {
    let catalog = catalog();
    let contributions = s02_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let spec = wounded_players(durable_spec(&materialization, 0xa1, false), 30_000, 0xa2);
    let (mut battle, started) = start(&materialization, spec, 0xa3);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = first_normal_action(&mut battle);
    assert!(
        resolution.fault().is_none(),
        "{:?} {:?}",
        resolution.fault(),
        resolution.events()
    );
}

fn s02_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many(
        catalog,
        "universe.path.destruction",
        &[
            MITIGATION,
            LOST_HP_STATS,
            LOW_HP_DAMAGE,
            LOW_HP_HEALING,
            ULTIMATE_SHIELD,
            BLESSING_ATTACK,
        ],
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

fn scalar_factor(value: &ValueExpr) -> Option<i64> {
    match value {
        ValueExpr::Multiply { rhs, .. } => match rhs.as_ref() {
            ValueExpr::Literal(RuleValue::Scalar(value)) => Some(value.scaled()),
            _ => None,
        },
        _ => None,
    }
}

fn literal_scalar(value: &ValueExpr) -> Option<i64> {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => Some(value.scaled()),
        _ => None,
    }
}

fn effect_id(
    binding: &starclock_mode_universe::battle_contribution::UniverseBattleRuleBinding,
) -> starclock_combat::EffectDefinitionId {
    starclock_combat::EffectDefinitionId::new(0x7660_0000 + binding.rule().get()).unwrap()
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
