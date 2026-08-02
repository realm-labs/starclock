use super::*;
use starclock_combat::{
    EffectDefinitionId, ProgramId, RuleBundleId, SelectorId,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityProgramBinding, AbilityProgramTiming,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EffectDefinition, ProgramDefinition, SelectorDefinition,
            UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    rule::model::{ProgramStep, RuleOperationTemplate, RuleValue, ValueExpr},
};

const FIXTURE_ABILITY: u32 = 0x7f00_0001;
const FIXTURE_UNIT: u32 = 0x7f00_0002;
const FIXTURE_ACTION_SELECTOR: u32 = 0x7f00_0003;
const FIXTURE_RULE_SELECTOR: u32 = 0x7f00_0004;
const FIXTURE_PROGRAM: u32 = 0x7f00_0005;
const FIXTURE_EFFECT: u32 = 0x7f00_0006;

#[test]
fn goal07_p2_m02_s02_executes_dynamic_stat_and_directional_shield_rules() {
    let catalog = catalog();
    let direct = |level, marker| {
        let contributions = contributions_many(
            &catalog,
            "universe.path.preservation",
            &[
                ("universe.blessing.612030", 1),
                ("universe.blessing.612032", 2),
                ("universe.blessing.612043", level),
            ],
            None,
            false,
        );
        let materialization = materialize(&catalog, &contributions);
        let (mut battle, start) = start(
            &materialization,
            durable_spec(&materialization, marker, false),
            marker.wrapping_add(1),
        );
        assert!(start.fault().is_none(), "{:?}", start.fault());
        first_normal_action(&mut battle)
            .events()
            .iter()
            .find_map(|event| match event.kind() {
                BattleEventKind::Damage(data)
                    if data.class == starclock_combat::formula::model::DamageClass::Direct =>
                {
                    Some(data.applied.get())
                }
                _ => None,
            })
            .expect("the selected basic action deals direct damage")
    };
    let safe_load_l1 = direct(1, 0xc1);
    let safe_load_l2 = direct(2, 0xc2);
    assert_eq!((safe_load_l1, safe_load_l2), (110, 130));

    let turn_end_shield = |sanctuary_level, capacity_level, marker| {
        let contributions = contributions_many(
            &catalog,
            "universe.path.preservation",
            &[
                ("universe.blessing.612044", sanctuary_level),
                ("universe.blessing.612045", capacity_level),
                ("universe.blessing.612042", 1),
            ],
            None,
            false,
        );
        let materialization = materialize(&catalog, &contributions);
        let (mut battle, start) = start(
            &materialization,
            durable_spec(&materialization, marker, false),
            marker.wrapping_add(1),
        );
        assert!(start.fault().is_none(), "{:?}", start.fault());
        first_normal_action(&mut battle)
            .events()
            .iter()
            .find_map(|event| match event.kind() {
                BattleEventKind::Shield(starclock_combat::ShieldEventData::Applied {
                    amount,
                    ..
                }) => Some(amount.get()),
                _ => None,
            })
    };
    assert_eq!(turn_end_shield(1, 1, 0xc3), Some(15_600));
    assert_eq!(turn_end_shield(2, 2, 0xc4), Some(20_250));
}

#[test]
fn goal07_p2_m02_s02_executes_defense_count_and_provider_shield_rules() {
    let catalog = catalog();
    let quake = |defense_level, marker| {
        let contributions = contributions_many(
            &catalog,
            "universe.path.preservation",
            &[
                ("universe.blessing.612030", 1),
                ("universe.blessing.612032", 2),
                ("universe.blessing.612042", defense_level),
                ("universe.blessing.612050", 1),
            ],
            None,
            false,
        );
        let materialization = materialize(&catalog, &contributions);
        let (mut battle, start) = start(
            &materialization,
            durable_spec(&materialization, marker, false),
            marker.wrapping_add(1),
        );
        assert!(start.fault().is_none(), "{:?}", start.fault());
        first_normal_action(&mut battle)
            .events()
            .iter()
            .find_map(|event| match event.kind() {
                BattleEventKind::Damage(data)
                    if data.class == starclock_combat::formula::model::DamageClass::Additional =>
                {
                    Some(data.applied.get())
                }
                _ => None,
            })
            .expect("Quake emits additional damage")
    };
    assert_eq!((quake(1, 0xc5), quake(2, 0xc6)), (10_099, 10_148));

    assert_eq!(provider_shields(&catalog, 1, 0xc7), vec![1_000, 240]);
    assert_eq!(provider_shields(&catalog, 2, 0xc8), vec![1_000, 360]);
}

fn provider_shields(catalog: &Arc<UniverseCatalog>, level: u32, marker: u8) -> Vec<i64> {
    let contributions = contributions_many(
        catalog,
        "universe.path.preservation",
        &[
            ("universe.blessing.612046", level),
            ("universe.blessing.612042", 1),
            ("universe.blessing.612043", 1),
        ],
        None,
        false,
    );
    let materialization = materialize(catalog, &contributions);
    let original = durable_spec(&materialization, marker, false);
    let first = original
        .participants()
        .iter()
        .position(|participant| participant.side() == TeamSide::Player)
        .unwrap();
    let base = original.participants()[first].combatant();
    let rule_bundles = base.rule_bundles().to_vec();
    let combat_catalog = fixture_catalog(materialization.combat_catalog(), &rule_bundles, marker);
    let mut participants = original.participants().to_vec();
    let fixture = ResolvedCombatantSpec::new(
        UnitDefinitionId::new(FIXTURE_UNIT).unwrap(),
        base.level(),
        base.maximum_hp(),
        base.speed(),
        ResolvedDefinitionBindings::new(
            vec![AbilityId::new(FIXTURE_ABILITY).unwrap()],
            rule_bundles,
            base.modifiers().to_vec(),
        )
        .unwrap(),
        CombatantSpecDigest::new([marker.wrapping_add(10); 32]).unwrap(),
    )
    .unwrap()
    .with_base_attack_defense(base.base_attack(), base.base_defense())
    .with_energy(base.current_energy(), base.maximum_energy())
    .unwrap()
    .with_sources(base.sources().to_vec())
    .unwrap()
    .with_modifier_bindings(base.modifier_bindings().to_vec())
    .unwrap();
    let original_player = &participants[first];
    participants[first] = ParticipantSpec::new(
        TeamSide::Player,
        original_player.formation(),
        original_player.source(),
        fixture,
    )
    .with_wave(original_player.wave())
    .unwrap();
    let spec = BattleSpec::new(
        AssemblyDigest::new([marker.wrapping_add(11); 32]).unwrap(),
        original.encounter(),
        participants,
        original.resources(TeamSide::Player).clone(),
        original.resources(TeamSide::Enemy).clone(),
        original.concede_policy(),
    )
    .unwrap();
    let mut battle = Battle::create(
        combat_catalog,
        spec,
        BattleSeed::new([marker.wrapping_add(12); 32]),
    )
    .unwrap();
    let start = battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    assert!(start.fault().is_none(), "{:?}", start.fault());
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
        .find(|command| {
            matches!(
                command,
                Command::UseAbility {
                    ability,
                    primary_target: Some(target),
                    ..
                } if ability.get() == FIXTURE_ABILITY
                    && *target != battle.decision().unwrap().legal_commands().iter().find_map(
                        |candidate| match candidate {
                            Command::UseAbility { actor, ability, .. }
                                if ability.get() == FIXTURE_ABILITY => Some(actor),
                            _ => None,
                        }
                    ).copied().unwrap()
            )
        })
        .expect("fixture shield ability can target an ally")
        .clone();
    battle
        .apply(command)
        .unwrap()
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Shield(starclock_combat::ShieldEventData::Applied {
                amount, ..
            }) => Some(amount.get()),
            _ => None,
        })
        .collect()
}

fn fixture_catalog(
    base: &Arc<starclock_combat::catalog::CombatCatalog>,
    rule_bundles: &[RuleBundleId],
    marker: u8,
) -> Arc<starclock_combat::catalog::CombatCatalog> {
    let action_selector = SelectorId::new(FIXTURE_ACTION_SELECTOR).unwrap();
    let rule_selector = SelectorId::new(FIXTURE_RULE_SELECTOR).unwrap();
    let program = ProgramId::new(FIXTURE_PROGRAM).unwrap();
    let effect = EffectDefinitionId::new(FIXTURE_EFFECT).unwrap();
    let ability = AbilityId::new(FIXTURE_ABILITY).unwrap();
    let mut builder = CombatCatalogBuilder::from_catalog(
        base,
        [marker; 32],
    );
    builder.add_selector(SelectorDefinition::new(action_selector).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Allied, TargetPattern::Single).unwrap(),
    ));
    builder.add_selector(
        SelectorDefinition::new(rule_selector).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::PrimaryTarget,
                RuleSelectorSide::Same,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Present,
                RuleSelectorReference::CurrentState,
                RuleSelectorOrdering::Formation,
                1,
                1,
                RuleEmptyPoolPolicy::Fault,
                RuleSelectorChoice::First,
                None,
                false,
            )
            .unwrap(),
        ),
    );
    builder.add_effect(EffectDefinition::new(effect, Vec::new(), Vec::new()));
    builder.add_program(
        ProgramDefinition::new(
            program,
            Vec::new(),
            vec![rule_selector],
            vec![effect],
            Vec::new(),
        )
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::Shield {
                selector: rule_selector,
                amount: ValueExpr::Literal(RuleValue::Scalar(
                    starclock_combat::Scalar::checked_from_integer(1_000).unwrap(),
                )),
                effect,
            },
        )]),
    );
    let action = AbilityActionDefinition::new(
        AbilityKind::Skill,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        starclock_combat::catalog::action::ActionResourcePolicy::new(
            0,
            0,
            Energy::ZERO,
            Energy::ZERO,
        ),
    )
    .unwrap();
    builder.add_ability(
        AbilityDefinition::new(ability, program, action_selector, vec![effect])
            .with_action(action)
            .with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::BeforeHits, program).unwrap(),
            ]),
    );
    builder.add_unit(UnitDefinition::new(
        UnitDefinitionId::new(FIXTURE_UNIT).unwrap(),
        vec![ability],
        rule_bundles.to_vec(),
    ));
    builder.build().unwrap()
}
