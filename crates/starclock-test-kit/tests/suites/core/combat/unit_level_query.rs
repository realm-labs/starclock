//! Resolved-level queries drive actual proposals, commands and canonical events.
use crate::combat_decision::advance_boundary_if_offered;
use starclock_combat::{
    ActionGauge, AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattleSeed, BattleSpec,
    CombatantSpecDigest, Command, ConcedePolicy, DecisionId, Energy, FaultPolicy, FormationIndex,
    Hp, LinkedEntityKind, LinkedUnitCatalogDefinition, LinkedUnitDefinition, OwnerLinkPolicy,
    ParticipantSource, ParticipantSpec, PresenceState, Resolution, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, Rounding, Scalar, Speed, TeamResourceSpec, TeamSide, UnitLevel,
    WaveLinkPolicy,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            ActionResourcePolicy, TargetInvalidationPolicy, TargetPattern, TargetRelation,
            UnitTargetSelector,
        },
        builder::{CatalogBuildError, CatalogBuildErrorKind, CombatCatalogBuilder},
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition, RuleBundle,
            RuleDefinition, SelectorDefinition, UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    modifier::model::StatQuerySubject,
    rule::model::{
        BattleRuleDefinition, BattleRuleScope, ConditionExpr, EventFilter, OnceScope, ProgramStep,
        ReactionPriority, RuleEventPoint, RuleOperationTemplate, RuleSource, RuleValue,
        RuleValueKind, SourceClass, StateSlotDef, TriggerDef, TriggerPhase, ValueExpr,
    },
};
use starclock_replay::battle_event::encode_battle_event_payload;
use std::sync::Arc;

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn level(subject: StatQuerySubject) -> ValueExpr {
    ValueExpr::QueryUnitLevel(subject)
}

fn units(origin: RuleSelectorOrigin, side: RuleSelectorSide, maximum: u16) -> RuleUnitSelector {
    RuleUnitSelector::new(
        origin,
        side,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StableId,
        0,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::All,
        None,
        false,
    )
    .unwrap()
}

fn combatant(form: u32, level: u8, player: bool, bound: bool) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        id(form),
        UnitLevel::new(level).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(if player { 100_000_000 } else { 1_000_000 }).unwrap(),
        ResolvedDefinitionBindings::new(
            vec![id(if player { 1 } else { 2 })],
            if bound { vec![id(1)] } else { vec![] },
            vec![],
        )
        .unwrap(),
        CombatantSpecDigest::new([level; 32]).unwrap(),
    )
    .unwrap()
}

fn signal(code: u32, value: ValueExpr) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
        code,
        value: Some(value),
    })
}

fn catalog(
    reference: RuleSelectorReference,
    scalar_slot: bool,
    missing_subject: bool,
    spawn: bool,
) -> Result<Arc<CombatCatalog>, CatalogBuildError> {
    let mut builder = CombatCatalogBuilder::new([0xe4; 32]);
    builder.add_selector(SelectorDefinition::new(id(1)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_selector(SelectorDefinition::new(id(2)).with_rule_units(units(
        RuleSelectorOrigin::Encounter,
        RuleSelectorSide::Opposing,
        16,
    )));
    builder.add_selector(SelectorDefinition::new(id(3)).with_rule_units(units(
        RuleSelectorOrigin::CurrentSubject,
        RuleSelectorSide::Opposing,
        1,
    )));
    builder.add_selector(SelectorDefinition::new(id(4)).with_rule_units(units(
        RuleSelectorOrigin::Encounter,
        RuleSelectorSide::Any,
        16,
    )));
    builder.add_selector(SelectorDefinition::new(id(6)).with_rule_units(units(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        1,
    )));
    let maximum = units(
        RuleSelectorOrigin::Encounter,
        RuleSelectorSide::Opposing,
        16,
    );
    builder.add_selector(
        SelectorDefinition::new(id(5)).with_rule_units(
            RuleUnitSelector::new(
                maximum.origin(),
                maximum.side(),
                maximum.life(),
                maximum.presence(),
                reference,
                maximum.ordering(),
                0,
                16,
                maximum.empty_pool(),
                maximum.choice(),
                None,
                false,
            )
            .unwrap()
            .with_predicates(vec![RuleSelectorPredicate::MaximumValue(level(
                StatQuerySubject::CurrentTarget,
            ))]),
        ),
    );
    let mut entry = vec![ProgramStep::ForEach {
        selector: id(2),
        body: id(12),
        maximum: 16,
    }];
    for (code, subject) in [
        (910, StatQuerySubject::Owner),
        (911, StatQuerySubject::Actor),
        (912, StatQuerySubject::Applier),
        (913, StatQuerySubject::EventTarget),
    ] {
        entry.push(signal(code, level(subject)));
    }
    if spawn {
        entry.push(ProgramStep::Operation(RuleOperationTemplate::Summon {
            owner_selector: id(6),
            unit_definition: id(3),
        }));
        entry.push(signal(915, ValueExpr::SelectorCount(id(4))));
    }
    if missing_subject {
        entry.push(signal(919, level(StatQuerySubject::CurrentTarget)));
    }
    builder.add_program(
        ProgramDefinition::new(
            id(11),
            vec![id(12)],
            vec![id(2), id(3), id(4), id(6)],
            vec![],
            vec![],
        )
        .with_steps(entry),
    );
    builder.add_program(
        ProgramDefinition::new(id(12), vec![], vec![id(3)], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::TrueDamage {
                selector: id(3),
                amount: ValueExpr::Convert {
                    value: Box::new(level(StatQuerySubject::CurrentTarget)),
                    target: RuleValueKind::Scalar,
                    rounding: Rounding::Floor,
                },
            }),
        ]),
    );
    builder.add_program(
        ProgramDefinition::new(
            id(13),
            vec![id(14), id(15)],
            vec![id(4), id(5)],
            vec![],
            vec![],
        )
        .with_steps(vec![
            ProgramStep::ForEach {
                selector: id(4),
                body: id(14),
                maximum: 16,
            },
            ProgramStep::ForEach {
                selector: id(5),
                body: id(15),
                maximum: 16,
            },
            ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                slot: id(1),
                value: level(StatQuerySubject::Owner),
            }),
            signal(914, level(StatQuerySubject::Owner)),
        ]),
    );
    for (program, code) in [(14, 900), (15, 902)] {
        builder.add_program(
            ProgramDefinition::new(id(program), vec![], vec![], vec![], vec![]).with_steps(vec![
                signal(code, ValueExpr::CurrentTarget),
                signal(code + 1, level(StatQuerySubject::CurrentTarget)),
            ]),
        );
    }
    let source = RuleSource::new(id(60), SourceClass::Synthetic, vec![], [0xe5; 32]);
    let triggers = [
        RuleEventPoint::BattleStarted,
        RuleEventPoint::ActionResolved,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, point)| TriggerDef {
        id: id(u32::try_from(index + 1).unwrap()),
        event: point.kind(),
        event_point: point,
        phase: TriggerPhase::AfterEvent,
        filter: EventFilter::default(),
        condition: ConditionExpr::Literal(true),
        once_scope: OnceScope::Event,
        priority: ReactionPriority::new(0),
        program: id(13),
    })
    .collect();
    let kind = if scalar_slot {
        RuleValueKind::Scalar
    } else {
        RuleValueKind::Integer
    };
    let initial = if scalar_slot {
        RuleValue::Scalar(Scalar::ZERO)
    } else {
        RuleValue::Integer(0)
    };
    builder.add_rule(
        RuleDefinition::new(id(1), vec![id(13), id(14), id(15)], vec![id(4), id(5)]).with_runtime(
            BattleRuleDefinition::new(
                source,
                vec![StateSlotDef::new(
                    id(1),
                    kind,
                    BattleRuleScope::Battle,
                    initial,
                )],
                triggers,
                None,
            ),
        ),
    );
    builder.add_rule_bundle(RuleBundle::new(id(1), vec![id(1)]));
    builder.add_program(ProgramDefinition::new(
        id(2),
        vec![],
        vec![],
        vec![],
        vec![],
    ));
    let action = AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap();
    builder.add_ability(
        AbilityDefinition::new(id(1), id(11), id(1), vec![])
            .with_action(action.clone())
            .with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, id(11)).unwrap(),
            ]),
    );
    builder.add_ability(AbilityDefinition::new(id(2), id(2), id(1), vec![]).with_action(action));
    builder.add_unit(UnitDefinition::new(id(1), vec![id(1)], vec![id(1)]));
    builder.add_unit(UnitDefinition::new(id(2), vec![id(2)], vec![]));
    builder.add_unit(UnitDefinition::new(id(3), vec![id(1)], vec![]));
    builder.add_enemy(EnemyDefinition::new(id(1), id(2), vec![id(2)]));
    builder.add_encounter(EncounterDefinition::new(id(1), vec![id(1)], vec![]));
    if spawn {
        let linked = LinkedUnitDefinition::new(
            combatant(3, 13, true, false),
            id(61),
            FormationIndex::new(1).unwrap(),
            LinkedEntityKind::Memosprite,
            PresenceState::Present,
            None,
            ActionGauge::from_scaled(0).unwrap(),
            OwnerLinkPolicy::Persist,
            OwnerLinkPolicy::Persist,
            WaveLinkPolicy::Persist,
        )
        .unwrap();
        builder.add_linked_unit(LinkedUnitCatalogDefinition::new(id(3), linked).unwrap());
    }
    builder.build()
}

fn fixture(spawn: bool, missing: bool) -> Battle {
    let participants = [70, 1, 81, 95]
        .into_iter()
        .enumerate()
        .map(|(index, level)| {
            let player = index == 0;
            ParticipantSpec::new(
                if player {
                    TeamSide::Player
                } else {
                    TeamSide::Enemy
                },
                FormationIndex::new(u8::try_from(index.saturating_sub(1)).unwrap()).unwrap(),
                if player {
                    ParticipantSource::Player
                } else {
                    ParticipantSource::EncounterEnemy(id(1))
                },
                combatant(if player { 1 } else { 2 }, level, player, player),
            )
        })
        .collect();
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xe6; 32]).unwrap(),
        id(1),
        participants,
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(
        catalog(RuleSelectorReference::CurrentState, false, missing, spawn).unwrap(),
        spec,
        BattleSeed::new([0xe7; 32]),
    )
    .unwrap()
}

fn start(battle: &mut Battle) -> Resolution {
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap()
}

fn attack(battle: &mut Battle) -> Resolution {
    advance_boundary_if_offered(battle);
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(command, Command::UseAbility {
            primary_target: Some(target), ..
        } if target.get() == 4)
        })
        .unwrap()
        .clone();
    battle.apply(command).unwrap()
}

fn values(events: &[BattleEvent], code: u32) -> Vec<RuleValue> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::RuleSignal(signal) if signal.code == code => signal.value.clone(),
            _ => None,
        })
        .collect()
}

#[test]
fn unit_level_native_for_each_damage_maximum_and_reconstruction_are_exact() {
    let mut first = fixture(false, false);
    let started = start(&mut first);
    assert!(started.fault().is_none());
    assert_eq!(
        values(started.events(), 901),
        [70, 1, 81, 95].map(RuleValue::Integer)
    );
    assert_eq!(
        values(started.events(), 902),
        [RuleValue::OptionalStableId(Some(4))]
    );
    assert_eq!(values(started.events(), 903), [RuleValue::Integer(95)]);
    assert!(started.events().iter().any(|event| matches!(event.kind(),
        BattleEventKind::RuleState(data) if data.slot == id(1)
            && data.before == RuleValue::Integer(0) && data.after == RuleValue::Integer(70))));
    let mut second = fixture(false, false);
    let repeated = start(&mut second);
    assert_eq!(started.events(), repeated.events());
    assert_eq!(first.state_hash(), second.state_hash());
    let result = attack(&mut first);
    let repeated = attack(&mut second);
    assert!(result.fault().is_none());
    assert_eq!(result.events(), repeated.events());
    assert_eq!(
        result
            .events()
            .iter()
            .map(|event| encode_battle_event_payload(event).unwrap())
            .collect::<Vec<_>>(),
        repeated
            .events()
            .iter()
            .map(|event| encode_battle_event_payload(event).unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(first.state_hash(), second.state_hash());
    let damages = result
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some((data.target.get(), data.applied.get())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(damages, [(2, 1), (3, 81), (4, 95)]);
    assert_eq!(
        first
            .view()
            .units_by_id()
            .map(|unit| unit.current_hp().get())
            .collect::<Vec<_>>(),
        [1_000, 999, 919, 905]
    );
    for code in [910, 911, 912, 914] {
        assert_eq!(values(result.events(), code), [RuleValue::Integer(70)]);
    }
    assert_eq!(values(result.events(), 913), [RuleValue::Integer(95)]);
    assert_eq!(first.view().rng_draw_count(), 0);
}

#[test]
fn unit_level_new_linked_unit_reads_its_own_level_after_committed_summon() {
    let mut battle = fixture(true, false);
    assert!(start(&mut battle).fault().is_none());
    let result = attack(&mut battle);
    assert!(result.fault().is_none());
    // This program's selection/query boundary predates its summon emission.
    assert_eq!(values(result.events(), 915), [RuleValue::Integer(4)]);
    assert_eq!(
        values(result.events(), 900),
        (1..=5)
            .map(|unit| RuleValue::OptionalStableId(Some(unit)))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        values(result.events(), 901),
        [70, 1, 81, 95, 13].map(RuleValue::Integer)
    );
    assert_eq!(battle.view().rng_draw_count(), 0);
}

#[test]
fn unit_level_rejected_commands_preserve_hash_and_draw_count() {
    let mut battle = fixture(false, false);
    start(&mut battle);
    advance_boundary_if_offered(&mut battle);
    let hash = battle.state_hash();
    let draws = battle.view().rng_draw_count();
    let mut command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { .. }))
        .unwrap()
        .clone();
    if let Command::UseAbility { decision, .. } = &mut command {
        *decision = DecisionId::new(decision.get() + 1).unwrap();
    }
    assert!(battle.apply(command).is_err());
    assert_eq!(battle.state_hash(), hash);
    assert_eq!(battle.view().rng_draw_count(), draws);
}

#[test]
fn unit_level_missing_iteration_subject_faults_before_any_damage_commits() {
    let mut battle = fixture(false, true);
    assert!(start(&mut battle).fault().is_none());
    let result = attack(&mut battle);
    assert_eq!(result.fault().unwrap().policy(), FaultPolicy::Rollback);
    assert!(
        !result
            .events()
            .iter()
            .any(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
    );
    assert!(
        battle
            .view()
            .units_by_id()
            .all(|unit| unit.current_hp().get() == 1_000)
    );
    assert_eq!(battle.view().rng_draw_count(), 0);
}

#[test]
fn unit_level_catalog_rejects_scalar_slot_and_unsupported_historical_query_contexts() {
    assert_eq!(
        catalog(RuleSelectorReference::CurrentState, true, false, false)
            .unwrap_err()
            .kind(),
        CatalogBuildErrorKind::InvalidDefinition
    );
    for reference in [
        RuleSelectorReference::ActionSnapshot,
        RuleSelectorReference::EventSnapshot,
    ] {
        assert_eq!(
            catalog(reference, false, false, false).unwrap_err().kind(),
            CatalogBuildErrorKind::InvalidDefinition
        );
    }
}
