use super::*;

#[test]
fn rule_cleanse_removes_a_dispellable_negative_effect_through_the_resolver() {
    let program = ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![id(1)], vec![])
        .with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                selector: id(2),
                effect: id(1),
                stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                chance: RuleEffectChancePolicy::Guaranteed,
                base_chance: None,
                rng_purpose: None,
            }),
            ProgramStep::Operation(RuleOperationTemplate::Cleanse {
                selector: id(2),
                maximum: 1,
                order: starclock_combat::EffectRemovalOrder::OldestFirst,
            }),
        ]);
    let mut battle = battle(
        catalog(program, true, true, false, false),
        true,
        true,
        false,
    );
    let resolution = start_and_use(&mut battle).unwrap();

    assert!(resolution.fault().is_none());
    assert_eq!(battle.view().effects_by_id().count(), 0);
    assert!(resolution.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Effect(starclock_combat::EffectEventData::Removed { .. })
        )
    }));
}
