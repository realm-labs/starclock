use super::*;

#[test]
fn selects_without_replacement_inside_each_group() {
    let program = ProgramDefinition::new(id(1), vec![], vec![id(105)], vec![id(1)], vec![])
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::RandomGroupedEffect {
                selector: id(105),
                effect: id(1),
                groups: ValueExpr::Literal(RuleValue::Integer(1)),
                applications_per_group: 2,
                stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                choice_rng_purpose: starclock_combat::rng::types::DrawPurpose::DAMAGE_TARGET,
                chance: RuleEffectChancePolicy::Guaranteed,
                base_chance: None,
                chance_rng_purpose: None,
            },
        )]);
    let mut battle = battle_with_two_enemies(catalog(program, false, false, false, false));
    let draws_before = battle.view().rng_draw_count();
    let resolution = start_and_use(&mut battle).unwrap();

    assert!(resolution.fault().is_none());
    let mut targets = battle
        .view()
        .effects_by_id()
        .map(|effect| effect.target())
        .collect::<Vec<_>>();
    targets.sort_unstable();
    targets.dedup();
    assert_eq!(targets.len(), 2);
    assert!(battle.view().rng_draw_count() > draws_before);
}
