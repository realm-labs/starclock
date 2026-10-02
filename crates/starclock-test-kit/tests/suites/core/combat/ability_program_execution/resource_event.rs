//! Committed resource variants drive generic typed filters through real commands.

use super::{battle_spec, catalog, id, start_and_use};
use starclock_combat::{
    Battle, BattleEvent, BattleEventKind, BattleSeed, BattleSpec, Command, ConcedePolicy, Energy,
    EventId, KeyedTeamResourceSpec, ParticipantSpec, ResourceEventData, Rounding, Scalar,
    TeamResourceSpec, TeamResourceWavePolicy, TeamSide,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityProgramBinding, AbilityProgramTiming,
            ActionResourcePolicy,
        },
        builder::CombatCatalogBuilder,
        definition::{ProgramDefinition, RuleBundle, RuleDefinition},
    },
    rule::model::{
        BattleRuleDefinition, Comparison, ConditionExpr, EventFilter, EventValueProperty,
        OnceScope, ProgramStep, ReactionPriority, ResourceMaximumUpdateKind, ResourceUpdateKind,
        RuleEventKind, RuleEventPoint, RuleOperationTemplate, RuleResourceEventKind,
        RuleResourceKind, RuleSource, RuleValue, SourceClass, TriggerDef, TriggerPhase, ValueExpr,
    },
};

fn scalar(amount: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(
        Scalar::checked_from_integer(amount).unwrap(),
    ))
}

fn balance(resource: RuleResourceKind, update: ResourceUpdateKind, amount: i64) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
        selector: id(4),
        resource,
        update,
        amount: scalar(amount),
        scales_with_regeneration: false,
        rounding: Rounding::Floor,
    })
}

fn maximum(update: ResourceMaximumUpdateKind, amount: i64) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ModifySkillPointMaximum {
        selector: id(4),
        update,
        amount: scalar(amount),
    })
}

fn resource_battle(steps: Vec<ProgramStep>, skill_point_gain: u16) -> Battle {
    let program =
        ProgramDefinition::new(id(1), vec![], vec![id(4)], vec![], vec![]).with_steps(steps);
    // The form declares a personal charge resource; the resolved build binds our observer rule.
    let base = catalog(program, false, false, false, true);
    let mut builder = CombatCatalogBuilder::from_catalog(&base, [0x81; 32]);
    let base_action = base.ability(id(1)).unwrap().action().unwrap();
    let action = AbilityActionDefinition::new(
        base_action.kind(),
        base_action.hit_count(),
        base_action.invalidation(),
        ActionResourcePolicy::new(0, skill_point_gain, Energy::ZERO, Energy::ZERO),
    )
    .unwrap()
    .with_hits(base_action.hits().to_vec())
    .unwrap();
    assert!(
        builder.replace_ability(
            base.ability(id(1))
                .unwrap()
                .clone()
                .with_action(action)
                .with_programs(vec![
                    AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, id(1)).unwrap()
                ])
        )
    );

    let mut triggers = Vec::new();
    for (raw, axis) in [
        (3, None),
        (4, Some(RuleResourceEventKind::BalanceChanged)),
        (5, Some(RuleResourceEventKind::MaximumChanged)),
        (6, Some(RuleResourceEventKind::BalanceChanged)),
    ] {
        builder.add_program(
            ProgramDefinition::new(id(raw), vec![], vec![], vec![], vec![]).with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
                    code: raw * 10,
                    value: Some(ValueExpr::ReadEventProperty(
                        EventValueProperty::ResourceDelta,
                    )),
                }),
                ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
                    code: raw * 10 + 1,
                    value: Some(ValueExpr::ReadEventProperty(
                        EventValueProperty::ResourceOverflow,
                    )),
                }),
            ]),
        );
        triggers.push(TriggerDef {
            id: id(raw),
            event: RuleEventKind::Resource,
            event_point: RuleEventPoint::ResourceChanged,
            phase: TriggerPhase::AfterMutation,
            filter: EventFilter {
                resource_event: axis,
                resource: (raw == 6).then_some(RuleResourceKind::SkillPoints),
                ..EventFilter::default()
            },
            condition: if raw == 6 {
                ConditionExpr::Compare {
                    operator: Comparison::Greater,
                    lhs: Box::new(ValueExpr::ReadEventProperty(
                        EventValueProperty::ResourceDelta,
                    )),
                    rhs: Box::new(scalar(0)),
                }
            } else {
                ConditionExpr::Literal(true)
            },
            once_scope: OnceScope::Event,
            priority: ReactionPriority::new(0),
            program: id(raw),
        });
    }
    builder.add_rule(
        RuleDefinition::new(id(1), vec![id(3), id(4), id(5), id(6)], vec![]).with_runtime(
            BattleRuleDefinition::new(
                RuleSource::new(id(80), SourceClass::Synthetic, vec![], [0x80; 32]),
                vec![],
                triggers,
                None,
            ),
        ),
    );
    builder.add_rule_bundle(RuleBundle::new(id(1), vec![id(1)]));
    let base_spec = battle_spec(false, true, false);
    let mut participants = base_spec.participants().to_vec();
    let player = &participants[0];
    participants[0] = ParticipantSpec::new(
        player.side(),
        player.formation(),
        player.source(),
        player
            .combatant()
            .clone()
            .with_energy(Energy::ZERO, Energy::from_scaled(10_000_000).unwrap())
            .unwrap(),
    );
    let resources = TeamResourceSpec::new(3, 5)
        .unwrap()
        .with_keyed(vec![
            KeyedTeamResourceSpec::new(id(90), 1, 5, TeamResourceWavePolicy::Persist)
                .unwrap()
                .with_stable_key("shared.meter")
                .unwrap(),
        ])
        .unwrap();
    let spec = BattleSpec::new(
        base_spec.assembly_digest(),
        base_spec.encounter(),
        participants,
        resources,
        base_spec.resources(TeamSide::Enemy).clone(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(builder.build().unwrap(), spec, BattleSeed::new([0x82; 32])).unwrap()
}

fn signals(events: &[BattleEvent], code: u32) -> Vec<(EventId, Scalar)> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::RuleSignal(data) if data.code == code => {
                let Some(RuleValue::Scalar(value)) = &data.value else {
                    panic!("typed resource fact")
                };
                // Reactions and consecutive emissions preserve their ordered parent chain.
                let mut parent = event.cause().parent_event().unwrap();
                loop {
                    let ancestor = events.iter().find(|event| event.id() == parent).unwrap();
                    if matches!(ancestor.kind(), BattleEventKind::Resource(_)) {
                        break;
                    }
                    parent = ancestor.cause().parent_event().unwrap();
                }
                Some((parent, *value))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn resource_event_dispatch_distinguishes_all_balance_variants_from_both_maximum_variants() {
    let run = || {
        let mut battle = resource_battle(
            vec![
                maximum(ResourceMaximumUpdateKind::Add, 2),
                balance(RuleResourceKind::SkillPoints, ResourceUpdateKind::Gain, 1),
                maximum(ResourceMaximumUpdateKind::Subtract, 4),
                maximum(ResourceMaximumUpdateKind::Set, 6),
                balance(RuleResourceKind::SkillPoints, ResourceUpdateKind::Spend, 1),
                balance(
                    RuleResourceKind::SkillPoints,
                    ResourceUpdateKind::Reserve,
                    1,
                ),
                balance(RuleResourceKind::SkillPoints, ResourceUpdateKind::Set, 6),
                balance(RuleResourceKind::SkillPoints, ResourceUpdateKind::Gain, 1),
                balance(RuleResourceKind::Energy, ResourceUpdateKind::Gain, 1),
                balance(
                    RuleResourceKind::Character("enhanced-counter-charges".into()),
                    ResourceUpdateKind::Gain,
                    1,
                ),
                balance(
                    RuleResourceKind::Team("shared.meter".into()),
                    ResourceUpdateKind::Gain,
                    1,
                ),
                ProgramStep::Operation(RuleOperationTemplate::ReduceMaximumHp {
                    selector: id(4),
                    amount: scalar(20),
                    minimum_ratio: scalar(0),
                }),
            ],
            0,
        );
        let result = start_and_use(&mut battle).unwrap();
        assert!(result.fault().is_none(), "{:?}", result.fault());
        let resources = result
            .events()
            .iter()
            .filter(|event| matches!(event.kind(), BattleEventKind::Resource(_)))
            .collect::<Vec<_>>();
        // Operation order, including zero effective gain and a maximum-induced balance clamp.
        let axes = [
            true, false, true, true, false, false, false, false, false, false, false, true,
        ];
        let deltas = [2, 1, -4, 3, -1, -1, 5, 0, 1, 1, 1, -20];
        assert_eq!(resources.len(), axes.len());
        let expected = resources
            .iter()
            .zip(deltas)
            .map(|(event, delta)| (event.id(), Scalar::checked_from_integer(delta).unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(signals(result.events(), 30), expected);
        for (code, maximum) in [(40, false), (50, true)] {
            assert_eq!(
                signals(result.events(), code),
                expected
                    .iter()
                    .zip(axes)
                    .filter_map(|(value, axis)| (axis == maximum).then_some(*value))
                    .collect::<Vec<_>>()
            );
        }
        assert_eq!(signals(result.events(), 60), [expected[1], expected[6]]);
        assert!(result.events().iter().any(|event| matches!(
            event.kind(),
            BattleEventKind::Resource(ResourceEventData::SkillPointMaximum {
                before: 7,
                after: 3,
                current_before: 4,
                current_after: 3,
                ..
            })
        )));
        assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 6);
        assert_eq!(battle.view().rng_draw_count(), 0);
        (result.events().to_vec(), battle.state_hash())
    };
    assert_eq!(run(), run());
}

#[test]
fn resource_event_effective_gain_excludes_cap_only_and_overflow_only_changes() {
    let mut battle = resource_battle(
        vec![
            maximum(ResourceMaximumUpdateKind::Add, 2),
            balance(RuleResourceKind::SkillPoints, ResourceUpdateKind::Set, 7),
        ],
        2,
    );
    let result = start_and_use(&mut battle).unwrap();
    assert!(result.fault().is_none(), "{:?}", result.fault());
    let gain = signals(result.events(), 60);
    assert_eq!(gain.len(), 1);
    assert_eq!(gain[0].1, Scalar::checked_from_integer(4).unwrap());
    let overflow = result
        .events()
        .iter()
        .find(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Resource(ResourceEventData::SkillPoints {
                    before: 7,
                    after: 7,
                    overflow: 2,
                    ..
                })
            )
        })
        .unwrap();
    assert!(signals(result.events(), 40).contains(&(overflow.id(), Scalar::ZERO)));
    assert!(
        signals(result.events(), 41)
            .contains(&(overflow.id(), Scalar::checked_from_integer(2).unwrap()))
    );
    assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 7);
    assert_eq!(battle.view().rng_draw_count(), 0);
}

#[test]
fn resource_event_rejected_command_leaves_resource_and_rng_state_unchanged() {
    let mut battle = resource_battle(vec![maximum(ResourceMaximumUpdateKind::Add, 2)], 0);
    let stale = Command::StartBattle {
        decision: battle.decision().unwrap().id(),
    };
    let result = start_and_use(&mut battle).unwrap();
    assert!(result.fault().is_none());
    let before = battle.state_hash();
    assert!(battle.apply(stale).is_err());
    assert_eq!(battle.state_hash(), before);
    assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 3);
    assert_eq!(battle.view().rng_draw_count(), 0);
}
