//! Real command regressions for independent outgoing final damage factors.
use super::*;
use starclock_combat::{BreakDamageKind, RawToughness, formula::model::DamageClass};
use starclock_combat::{
    catalog::{
        action::{AbilityProgramBinding, AbilityProgramTiming},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    rule::model::{ProgramStep, RuleOperationTemplate},
};

pub(super) fn extend_catalog(builder: &mut CombatCatalogBuilder) {
    builder.add_selector(
        SelectorDefinition::new(definition(200)).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::CurrentSubject,
                RuleSelectorSide::Opposing,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Present,
                RuleSelectorReference::CurrentState,
                RuleSelectorOrdering::StableId,
                1,
                1,
                RuleEmptyPoolPolicy::NoOp,
                RuleSelectorChoice::First,
                None,
                false,
            )
            .unwrap(),
        ),
    );
    builder.add_program(
        ProgramDefinition::new(
            definition(200),
            vec![],
            vec![definition(200)],
            vec![],
            vec![],
        )
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::TrueDamage {
                selector: definition(200),
                amount: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(1_900_000))),
            },
        )]),
    );
    builder.add_ability(
        AbilityDefinition::new(definition(104), definition(2), definition(2), vec![])
            .with_action(action(
                AbilityKind::Basic,
                vec![vec![]],
                TargetInvalidationPolicy::KeepIfPresent,
            ))
            .with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, definition(200)).unwrap(),
            ]),
    );
    for group in [100, 101] {
        builder.add_modifier_group(ModifierStackingGroup {
            id: definition(group),
            aggregation: ModifierAggregation::Product,
            comparator: None,
        });
    }
    for (id, purpose, value, group, stage) in [
        (
            100,
            FormulaPurpose::OrdinaryDamage,
            1_500_000,
            100,
            FormulaStage::DamageFinalMultiply,
        ),
        (
            101,
            FormulaPurpose::Dot,
            1_500_000,
            100,
            FormulaStage::DamageFinalMultiply,
        ),
        (
            102,
            FormulaPurpose::AdditionalDamage,
            1_500_000,
            100,
            FormulaStage::DamageFinalMultiply,
        ),
        (
            103,
            FormulaPurpose::ElationDamage,
            1_500_000,
            100,
            FormulaStage::DamageFinalMultiply,
        ),
        (
            104,
            FormulaPurpose::Break,
            1_500_000,
            100,
            FormulaStage::DamageFinalMultiply,
        ),
        (
            105,
            FormulaPurpose::SuperBreak,
            1_500_000,
            100,
            FormulaStage::DamageFinalMultiply,
        ),
        (
            106,
            FormulaPurpose::OrdinaryDamage,
            2_000_000,
            101,
            FormulaStage::DamageFinalMultiply,
        ),
        (
            107,
            FormulaPurpose::Stat,
            2_000_000,
            101,
            FormulaStage::FinalMultiply,
        ),
        (
            108,
            FormulaPurpose::OrdinaryDamage,
            -1,
            100,
            FormulaStage::DamageFinalMultiply,
        ),
        (
            109,
            FormulaPurpose::OrdinaryDamage,
            0,
            100,
            FormulaStage::DamageFinalMultiply,
        ),
        (
            110,
            FormulaPurpose::OrdinaryDamage,
            -1_000_000,
            101,
            FormulaStage::DamageFinalMultiply,
        ),
    ] {
        builder.add_modifier(ModifierDefinition {
            id: definition(id),
            stat: StatKind::Atk,
            stage,
            purpose,
            value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(value))),
            stacking_group: definition(group),
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: stage,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        });
    }
    for (id, class) in [
        (100, DamageClass::Direct),
        (101, DamageClass::Dot),
        (102, DamageClass::Additional),
        (103, DamageClass::Elation),
    ] {
        let damage = OrdinaryDamageDefinition::new(
            Scalar::from_scaled(1_900_000),
            OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
        )
        .unwrap()
        .with_class(class);
        builder.add_ability(
            AbilityDefinition::new(definition(id), definition(2), definition(2), vec![])
                .with_action(action(
                    AbilityKind::Basic,
                    vec![vec![HitOperationDefinition::Damage(damage)]],
                    TargetInvalidationPolicy::KeepIfPresent,
                )),
        );
    }
}

fn bound(form: u32, abilities: Vec<u32>, modifiers: &[u32], speed: i64) -> ResolvedCombatantSpec {
    let source = definition(150);
    combatant_with_modifiers(form, abilities, modifiers.to_vec(), 20_000, speed, 0x81)
        .with_base_attack_defense(
            StatValue::from_scaled(2_000_000_000).unwrap(),
            StatValue::from_scaled(0).unwrap(),
        )
        .with_sources(vec![RuleSource::new(
            source,
            SourceClass::Mode,
            vec![],
            [0x82; 32],
        )])
        .unwrap()
        .with_modifier_bindings(
            modifiers
                .iter()
                .map(|id| ResolvedModifierBinding::new(definition(*id), source))
                .collect(),
        )
        .unwrap()
}

fn make_battle(player: &[u32], enemy: &[u32]) -> Battle {
    let layer = ToughnessLayerSpec::ordinary(1, RawToughness::new(50).unwrap()).unwrap();
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x83; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                bound(
                    1,
                    vec![2, 4, 5, 6, 100, 101, 102, 103, 104],
                    player,
                    200_000_000,
                ),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                bound(2, vec![3], enemy, 190_000_000)
                    .with_toughness(EnemyRank::Normal, vec![], vec![layer])
                    .unwrap(),
            ),
        ],
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog(1), spec, BattleSeed::new([0x84; 32])).unwrap()
}

fn damages(player: &[u32], enemy: &[u32], ability: u32) -> Vec<(i64, i64)> {
    let mut battle = make_battle(player, enemy);
    start_and_pass(&mut battle);
    let result = use_ability(&mut battle, ability);
    assert!(result.fault().is_none());
    result
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some((data.raw.scaled(), data.calculated.get())),
            _ => None,
        })
        .collect()
}

#[test]
fn final_damage_scales_raw_once_for_each_ordinary_class_without_stat_leakage() {
    for id in 100..=103 {
        assert_eq!(damages(&[id], &[], id), [(2_850_000, 2)]);
        assert_eq!(
            damages(&[], &[id], id),
            [(1_900_000, 1)],
            "incoming ownership must not amplify outgoing damage"
        );
    }
    assert_eq!(
        damages(&[101], &[], 100),
        [(1_900_000, 1)],
        "formula purposes remain isolated"
    );
    assert_eq!(damages(&[100], &[], 6), [(1_500_000_000, 1500)]);
    assert_eq!(damages(&[100, 106], &[], 6), [(3_000_000_000, 3000)]);
    assert_eq!(
        damages(&[100, 107], &[], 6),
        [(3_000_000_000, 3000)],
        "stat final multiplier changes base ATK only once"
    );
    assert_eq!(
        damages(&[1, 100], &[], 6),
        [(1_650_000_000, 1650)],
        "separate from additive damage boosts"
    );
    assert_eq!(
        damages(&[9, 100], &[], 2),
        [(1_000_000, 1), (1_000_000, 1)],
        "absolute overrides remain final"
    );
    assert_eq!(damages(&[109], &[], 100), [(0, 0)]);
    assert_eq!(
        damages(&[100, 106], &[], 104),
        [(1_900_000, 1)],
        "true damage bypasses source amplification"
    );
}

#[test]
fn final_damage_break_super_break_and_later_effect_use_original_source() {
    let run = |modifiers: &[u32], target_modifiers: &[u32]| {
        let mut battle = make_battle(modifiers, target_modifiers);
        start_and_pass(&mut battle);
        let mut events = use_ability(&mut battle, 5).events().to_vec();
        events.extend(settle_ready_boundaries(&mut battle));
        events
            .into_iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::BreakDamage(data) => {
                    Some((data.kind, data.raw.scaled(), data.calculated.get()))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let plain = run(&[], &[]);
    let boosted = run(&[104, 105], &[]);
    assert_eq!(
        run(&[], &[104, 105]),
        plain,
        "the victim's turn never grants its outgoing factor to an earlier applier"
    );
    for kind in [
        BreakDamageKind::Initial,
        BreakDamageKind::SuperBreak,
        BreakDamageKind::Effect,
    ] {
        let expected = plain.iter().find(|row| row.0 == kind).unwrap();
        let actual = boosted.iter().find(|row| row.0 == kind).unwrap();
        assert_eq!(actual.1, expected.1 * 3 / 2, "{kind:?}");
        assert_eq!(actual.2, actual.1 / 1_000_000);
    }
}

#[test]
fn final_damage_negative_factor_enters_deterministic_fault_before_hp_mutation() {
    for modifiers in [&[108][..], &[108, 110][..]] {
        let mut first = make_battle(modifiers, &[]);
        let mut fresh = make_battle(modifiers, &[]);
        for battle in [&mut first, &mut fresh] {
            start_and_pass(battle);
            let result = use_ability(battle, 100);
            assert!(result.fault().is_some());
            assert!(
                !result
                    .events()
                    .iter()
                    .any(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
            );
            assert_eq!(
                battle
                    .view()
                    .units_by_id()
                    .nth(1)
                    .unwrap()
                    .current_hp()
                    .get(),
                20_000
            );
        }
        assert_eq!(first.state_hash(), fresh.state_hash());
    }
}
