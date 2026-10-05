//! One accepted action creates direct, detonated and later periodic facts.
use super::{catalog, combatant, definition, dot_damage, execute_probe};
use starclock_combat::{
    AssemblyDigest, Battle, BattleEventKind, BattleSeed, BattleSpec, CombatantSpecDigest, Command,
    ConcedePolicy, DamageKind, DecisionId, FormationIndex, Hp, ParticipantSource, ParticipantSpec,
    Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings, TeamResourceSpec, TeamSide,
    catalog::{
        action::{ActionHitDefinition, HitCritPolicy, HitOperationDefinition, HitTargetGroup},
        builder::CombatCatalogBuilder,
        definition::{ProgramDefinition, RuleBundle, RuleDefinition},
    },
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleDamageClass, RuleEventPoint, RuleOperationTemplate, RuleSource, SourceClass,
        TriggerDef, TriggerPhase,
    },
};
use starclock_replay::battle_event::encode_battle_event_payload;

fn scene() -> Battle {
    let base = catalog();
    let mut builder = CombatCatalogBuilder::from_catalog(&base, [0xd7; 32]);
    let ability = base.ability(definition(1)).unwrap();
    let action = ability.action().unwrap();
    let mut operations = vec![HitOperationDefinition::Damage(dot_damage(10))];
    operations.extend_from_slice(action.hits()[0].operations());
    assert!(
        builder.replace_ability(
            ability.clone().with_action(
                action
                    .clone()
                    .with_hits(vec![ActionHitDefinition::new(operations).with_profile(
                        HitTargetGroup::Selected,
                        Ratio::ONE,
                        Ratio::ONE,
                        HitCritPolicy::Never,
                    )])
                    .unwrap(),
            )
        )
    );
    let mut triggers = vec![];
    let mut programs = vec![];
    for (ordinal, point, kind, class) in [
        (
            1,
            RuleEventPoint::DamageApplied,
            Some(DamageKind::Direct),
            None,
        ),
        (
            2,
            RuleEventPoint::DamageApplied,
            Some(DamageKind::DotTick),
            None,
        ),
        (
            3,
            RuleEventPoint::DamageApplied,
            Some(DamageKind::DotDetonation),
            None,
        ),
        (
            4,
            RuleEventPoint::DamageApplied,
            None,
            Some(RuleDamageClass::Dot),
        ),
        (
            5,
            RuleEventPoint::DamageApplied,
            Some(DamageKind::DotTick),
            Some(RuleDamageClass::Ordinary),
        ),
        (
            6,
            RuleEventPoint::ActionStarted,
            Some(DamageKind::DotTick),
            None,
        ),
    ] {
        let program = definition(100 + ordinal);
        builder.add_program(
            ProgramDefinition::new(program, vec![], vec![], vec![], vec![]).with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
                    code: ordinal,
                    value: None,
                }),
            ]),
        );
        programs.push(program);
        triggers.push(TriggerDef {
            id: definition(100 + ordinal),
            event: point.kind(),
            event_point: point,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter {
                damage_kind: kind,
                damage_class: class,
                ..EventFilter::default()
            },
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Event,
            priority: ReactionPriority::new(0),
            program,
        });
    }
    builder.add_rule(
        RuleDefinition::new(definition(2), programs, vec![]).with_runtime(
            BattleRuleDefinition::new(
                RuleSource::new(definition(2), SourceClass::Synthetic, vec![], [0xd8; 32]),
                vec![],
                triggers,
                None,
            ),
        ),
    );
    builder.add_rule_bundle(RuleBundle::new(definition(2), vec![definition(2)]));
    let original = combatant(1, 1, 200_000_000, 1);
    let player = ResolvedCombatantSpec::new(
        original.form(),
        original.level(),
        Hp::new(1_000).unwrap(),
        original.speed(),
        ResolvedDefinitionBindings::new(
            vec![definition(1)],
            vec![definition(1), definition(2)],
            vec![],
        )
        .unwrap(),
        CombatantSpecDigest::new([0xd9; 32]).unwrap(),
    )
    .unwrap();
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xda; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                player,
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(2, 2, 101_000_000, 2),
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(builder.build().unwrap(), spec, BattleSeed::new([0xdb; 32])).unwrap()
}

#[test]
fn combat_effect_damage_kind_filters_distinguish_real_tick_and_external_detonation() {
    let run = || {
        let mut initial = scene();
        let before = initial.state_hash();
        assert!(
            initial
                .apply(Command::StartBattle {
                    decision: DecisionId::new(999).unwrap()
                })
                .is_err()
        );
        assert_eq!(initial.state_hash(), before);
        assert_eq!(initial.view().rng_draw_count(), 0);
        let (battle, events, hash) = execute_probe(initial);
        let kinds = events
            .iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::Damage(data) => Some(data.kind),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            [
                DamageKind::Direct,
                DamageKind::DotDetonation,
                DamageKind::DotTick
            ]
        );
        let retained = events
            .iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::Damage(data) if data.kind != DamageKind::Direct => {
                    Some((event.cause(), data.source_effect))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(retained.len(), 2);
        assert!(retained[0].1.is_some());
        assert_eq!(retained[0].1, retained[1].1);
        assert!(retained[0].0.applier().is_some());
        assert_eq!(retained[0].0.applier(), retained[1].0.applier());
        assert!(retained[0].0.source_definition().is_some());
        assert_eq!(
            retained[0].0.source_definition(),
            retained[1].0.source_definition()
        );
        let signals = events
            .iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::RuleSignal(data) => Some(data.code),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(signals, [1, 3, 4, 2, 4]);
        assert_eq!(battle.view().rng_draw_count(), 0);
        let payloads = events
            .iter()
            .map(|event| encode_battle_event_payload(event).unwrap())
            .collect::<Vec<_>>();
        (events, payloads, hash)
    };
    assert_eq!(run(), run());
}
