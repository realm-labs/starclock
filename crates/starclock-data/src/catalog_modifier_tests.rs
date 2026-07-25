use super::*;

#[test]
fn production_rows_execute_formula_stage_and_authored_comparator() {
    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let comparator_group = catalog
        .combat
        .modifiers
        .group(starclock_combat::ModifierStackingGroupId::new(970_001).unwrap())
        .expect("Goal 07 formal comparator probe");
    assert_eq!(
        comparator_group.aggregation,
        starclock_combat::modifier::model::ModifierAggregation::StrongestByComparator
    );
    assert!(comparator_group.comparator.is_some());

    let subject = starclock_combat::UnitId::new(1).unwrap();
    let instances = [starclock_combat::modifier::model::ActiveModifier {
        instance: starclock_combat::ModifierInstanceId::new(1).unwrap(),
        definition: starclock_combat::ModifierDefinitionId::new(24_701).unwrap(),
        owner: subject,
        subject,
        source: starclock_combat::SourceDefinitionId::new(1).unwrap(),
        source_class: starclock_combat::rule::model::SourceClass::Progression,
        insertion_sequence: 1,
        application_action: None,
        source_effect: None,
        slots: Box::new([]),
        captured_value: None,
        captured_stats: Box::new([]),
    }];
    let bases = std::collections::BTreeMap::new();
    let context = starclock_combat::modifier::model::ModifierQueryContext {
        element: Some(starclock_combat::formula::model::CombatElement::Lightning as u8),
        ..starclock_combat::modifier::model::ModifierQueryContext::default()
    };
    let value = starclock_combat::modifier::resolve::StatResolver::new(
        &catalog.combat.modifiers,
        &bases,
        &instances,
    )
    .query_formula(
        starclock_combat::modifier::model::FormulaModifierQuery {
            subject,
            stage: starclock_combat::modifier::model::FormulaStage::DamageBoost,
            purpose: starclock_combat::modifier::model::FormulaPurpose::OrdinaryDamage,
        },
        &context,
    )
    .unwrap();
    assert_eq!(value, starclock_combat::Scalar::from_scaled(32_000));
}

#[test]
fn production_state_slot_reset_survives_excel_and_sora_lowering() {
    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let rule = catalog
        .battle_rule(starclock_combat::RuleId::new(24_002).unwrap())
        .expect("Asta charging rule");
    let slot = rule
        .state_slots()
        .iter()
        .find(|slot| slot.id().get() == 24_003)
        .expect("Asta charging slot");
    assert_eq!(
        slot.reset_points(),
        &[starclock_combat::rule::model::SlotResetPoint::BattleStart]
    );
}
