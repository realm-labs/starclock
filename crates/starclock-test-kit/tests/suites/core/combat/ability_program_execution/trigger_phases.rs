use super::*;

#[test]
fn production_dispatches_each_supported_post_commit_phase_from_its_observed_event() {
    let cases = [
        TestTrigger {
            event: RuleEventKind::Hit,
            point: RuleEventPoint::HitStarted,
            phase: TriggerPhase::Before,
            once: OnceScope::Action,
        },
        TestTrigger {
            event: RuleEventKind::Hit,
            point: RuleEventPoint::HitEnded,
            phase: TriggerPhase::AfterMutation,
            once: OnceScope::Action,
        },
        TestTrigger {
            event: RuleEventKind::Hit,
            point: RuleEventPoint::HitEnded,
            phase: TriggerPhase::AfterEvent,
            once: OnceScope::Action,
        },
        TestTrigger {
            event: RuleEventKind::Hit,
            point: RuleEventPoint::HitEnded,
            phase: TriggerPhase::Boundary,
            once: OnceScope::Action,
        },
        TestTrigger {
            event: RuleEventKind::Action,
            point: RuleEventPoint::ActionResolved,
            phase: TriggerPhase::AfterAction,
            once: OnceScope::Action,
        },
    ];
    for trigger in cases {
        let program = ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![], vec![]);
        let mut battle = battle(
            catalog_with_trigger(program, false, true, false, false, Some(trigger), None),
            false,
            true,
            false,
        );
        let resolution = start_and_use(&mut battle).unwrap();
        let observed = resolution
            .events()
            .iter()
            .find(|event| {
                matches!(
                    (trigger.point, event.kind()),
                    (
                        RuleEventPoint::HitStarted,
                        BattleEventKind::Hit(starclock_combat::HitEventData::Started { .. }),
                    ) | (
                        RuleEventPoint::HitEnded,
                        BattleEventKind::Hit(starclock_combat::HitEventData::Ended { .. }),
                    ) | (
                        RuleEventPoint::ActionResolved,
                        BattleEventKind::Action(starclock_combat::ActionEventData::Resolved { .. }),
                    )
                )
            })
            .expect("authored observation event");
        let rule_damage = resolution
            .events()
            .iter()
            .find(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
            .expect("phase trigger emitted damage");

        assert_eq!(
            rule_damage.cause().parent_event(),
            Some(observed.id()),
            "{:?} trigger did not retain its observed-event parent",
            trigger.phase
        );
        assert_eq!(
            rule_damage.cause().root_command(),
            observed.cause().root_command()
        );
        assert_eq!(rule_damage.cause().action(), observed.cause().action());
        assert_eq!(rule_damage.cause().phase(), observed.cause().phase());
        assert_eq!(rule_damage.cause().hit(), observed.cause().hit());
        assert_eq!(
            battle
                .view()
                .units_by_id()
                .nth(1)
                .unwrap()
                .current_hp()
                .get(),
            950,
            "{:?} trigger did not execute exactly once",
            trigger.phase
        );
        assert!(resolution.fault().is_none());
    }
}

#[test]
fn after_defeat_settlement_dispatches_from_the_defeated_fact() {
    let program =
        ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::Damage {
                selector: id(2),
                amount: ValueExpr::Literal(RuleValue::Scalar(
                    Scalar::checked_from_integer(2_000).unwrap(),
                )),
                class: DamageClass::Direct,
                element: CombatElement::Physical,
                can_crit: false,
                can_defeat: true,
            }),
        ]);
    let trigger = TestTrigger {
        event: RuleEventKind::Unit,
        point: RuleEventPoint::UnitDefeated,
        phase: TriggerPhase::AfterDefeatSettlement,
        once: OnceScope::Event,
    };
    let rule_steps = vec![ProgramStep::Operation(
        RuleOperationTemplate::EmitRuleEvent {
            code: 701,
            value: Some(ValueExpr::Literal(RuleValue::Integer(1))),
        },
    )];
    let mut battle = battle(
        catalog_with_trigger(
            program,
            false,
            true,
            false,
            false,
            Some(trigger),
            Some(rule_steps),
        ),
        false,
        true,
        false,
    );
    let resolution = start_and_use(&mut battle).unwrap();
    let defeated = resolution
        .events()
        .iter()
        .find(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Unit(starclock_combat::UnitEventData::Defeated { .. })
            )
        })
        .expect("defeat settlement event");
    let signal = resolution
        .events()
        .iter()
        .find(|event| {
            matches!(
                event.kind(),
                BattleEventKind::RuleSignal(value) if value.code == 701
            )
        })
        .expect("after-defeat rule signal");

    assert_eq!(signal.cause().parent_event(), Some(defeated.id()));
    assert!(resolution.fault().is_none());
}

#[test]
fn once_per_turn_coalesces_hits_and_resets_at_the_next_turn_boundary() {
    let program = ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![], vec![]);
    let trigger = TestTrigger {
        event: RuleEventKind::Hit,
        point: RuleEventPoint::HitEnded,
        phase: TriggerPhase::AfterEvent,
        once: OnceScope::Turn,
    };
    let mut battle = battle(
        catalog_with_trigger(program, false, true, false, false, Some(trigger), None),
        false,
        true,
        false,
    );
    let first = start_and_use(&mut battle).unwrap();
    assert_eq!(
        first
            .events()
            .iter()
            .filter(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
            .count(),
        1
    );

    crate::combat_decision::pass_interrupt_if_offered(&mut battle);
    let use_ability = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(
            |command| matches!(command, Command::UseAbility { ability, .. } if ability.get() == 1),
        )
        .unwrap()
        .clone();
    let second = battle.apply(use_ability).unwrap();

    assert_eq!(
        second
            .events()
            .iter()
            .filter(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
            .count(),
        1
    );
    assert!(first.fault().is_none());
    assert!(second.fault().is_none());
}

#[test]
fn selected_rule_bundle_dispatches_once_after_the_authored_hit_event() {
    let program = ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![], vec![]);
    let mut battle = battle(
        catalog(program, false, true, false, false),
        false,
        true,
        false,
    );
    let resolution = start_and_use(&mut battle).unwrap();
    let damage = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(value) => Some(value.applied.get()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(damage, [50]);
    assert_eq!(battle.view().rule_instances_by_id().count(), 1);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .nth(1)
            .unwrap()
            .current_hp()
            .get(),
        950
    );
}
