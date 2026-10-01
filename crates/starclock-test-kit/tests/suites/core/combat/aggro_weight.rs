//! Authored Aggro expressions drive the existing integer target sampler.

use starclock_combat::{
    AssemblyDigest, Battle, BattleEventKind, BattleSeed, BattleSpec, CombatantSpecDigest, Command,
    ConcedePolicy, DecisionId, Energy, FormationIndex, Hp, ParticipantSource, ParticipantSpec,
    Resolution, ResolvedCombatantSpec, ResolvedDefinitionBindings, ResolvedModifierBinding,
    Rounding, Scalar, Speed, TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionResourcePolicy, TargetInvalidationPolicy,
            TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition, RuleBundle,
            RuleDefinition, SelectorDefinition, UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    formula::model::{CombatElement, DamageClass},
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind, StatQuerySubject,
    },
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleEventPoint, RuleOperationTemplate, RuleSource, RuleValue, SourceClass, TriggerDef,
        TriggerPhase, ValueExpr,
    },
};
use std::sync::Arc;

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn aggro(subject: StatQuerySubject) -> ValueExpr {
    ValueExpr::QueryStat {
        subject,
        stat: StatKind::Aggro,
        purpose: FormulaPurpose::Aggro,
    }
}

fn catalog(
    factors: [i64; 2],
    reference: RuleSelectorReference,
    snapshot: SnapshotPolicy,
) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new([0xba; 32]);
    builder.add_selector(SelectorDefinition::new(id(1)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    for (raw, choice, maximum) in [
        (2, RuleSelectorChoice::All, 2),
        (3, RuleSelectorChoice::RngWeighted, 1),
    ] {
        let weighted = choice == RuleSelectorChoice::RngWeighted;
        let selector = RuleUnitSelector::new(
            RuleSelectorOrigin::Encounter,
            RuleSelectorSide::Opposing,
            RuleLifePredicate::Alive,
            RulePresencePredicate::Present,
            reference,
            RuleSelectorOrdering::Formation,
            0,
            maximum,
            RuleEmptyPoolPolicy::NoOp,
            choice,
            weighted.then(|| "aggro-target".into()),
            false,
        )
        .unwrap()
        .with_weight(weighted.then(|| aggro(StatQuerySubject::CurrentTarget)));
        builder.add_selector(SelectorDefinition::new(id(raw)).with_rule_units(selector));
    }
    builder.add_program(ProgramDefinition::new(
        id(1),
        vec![],
        vec![],
        vec![],
        vec![],
    ));
    builder.add_program(
        ProgramDefinition::new(id(2), vec![], vec![id(2), id(3)], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
                code: 100,
                value: Some(ValueExpr::SelectorSum {
                    selector: id(2),
                    value: Box::new(aggro(StatQuerySubject::CurrentTarget)),
                }),
            }),
            ProgramStep::Operation(RuleOperationTemplate::Damage {
                selector: id(3),
                amount: ValueExpr::Literal(RuleValue::Scalar(Scalar::ONE)),
                class: DamageClass::Additional,
                element: CombatElement::Physical,
                can_crit: false,
                can_defeat: false,
            }),
        ]),
    );
    builder.add_rule(
        RuleDefinition::new(id(1), vec![id(2)], vec![id(2), id(3)]).with_runtime(
            BattleRuleDefinition::new(
                RuleSource::new(id(90), SourceClass::Synthetic, vec![], [0x90; 32]),
                vec![],
                vec![TriggerDef {
                    id: id(1),
                    event: RuleEventPoint::BattleStarted.kind(),
                    event_point: RuleEventPoint::BattleStarted,
                    phase: TriggerPhase::AfterEvent,
                    filter: EventFilter::default(),
                    condition: ConditionExpr::Literal(true),
                    once_scope: OnceScope::Battle,
                    priority: ReactionPriority::new(0),
                    program: id(2),
                }],
                None,
            ),
        ),
    );
    builder.add_rule_bundle(RuleBundle::new(id(1), vec![id(1)]));
    for (index, fraction) in factors.into_iter().enumerate() {
        let raw = u32::try_from(index).unwrap() + 1;
        builder.add_modifier_group(ModifierStackingGroup {
            id: id(raw),
            aggregation: ModifierAggregation::Sum,
            comparator: None,
        });
        let literal = ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(fraction)));
        let value = if snapshot == SnapshotPolicy::OnApplication {
            ValueExpr::Multiply {
                lhs: Box::new(literal),
                rhs: Box::new(aggro(StatQuerySubject::Owner)),
                rounding: Rounding::NearestTiesEven,
            }
        } else {
            literal
        };
        builder.add_modifier(ModifierDefinition {
            id: id(raw),
            stat: StatKind::Aggro,
            stage: FormulaStage::PercentOfBase,
            purpose: if raw == 1 {
                FormulaPurpose::Aggro
            } else {
                FormulaPurpose::Stat
            },
            value,
            stacking_group: id(raw),
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::PercentOfBase,
            snapshot,
            source_stack_slot: None,
            filters: Box::new([]),
        });
    }
    builder.add_ability(
        AbilityDefinition::new(id(1), id(1), id(1), vec![]).with_action(
            AbilityActionDefinition::new(
                AbilityKind::Basic,
                1,
                TargetInvalidationPolicy::KeepIfPresent,
                ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
            )
            .unwrap(),
        ),
    );
    for raw in [1, 2] {
        builder.add_unit(UnitDefinition::new(id(raw), vec![id(1)], vec![]));
    }
    builder.add_enemy(EnemyDefinition::new(id(1), id(2), vec![id(1)]));
    builder.add_encounter(EncounterDefinition::new(id(1), vec![id(1)], vec![]));
    builder.build().unwrap()
}

fn combatant(player: bool, index: u32) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        id(if player { 1 } else { 2 }),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(100_000_000).unwrap(),
        ResolvedDefinitionBindings::new(
            vec![id(1)],
            (!player).then(|| id(1)).into_iter().collect(),
            player.then(|| id(index)).into_iter().collect(),
        )
        .unwrap(),
        CombatantSpecDigest::new([u8::try_from(index).unwrap(); 32]).unwrap(),
    )
    .unwrap()
    .with_sources(vec![RuleSource::new(
        id(80),
        SourceClass::Synthetic,
        vec![],
        [0x80; 32],
    )])
    .unwrap()
    .with_modifier_bindings(
        player
            .then(|| ResolvedModifierBinding::new(id(index), id(80)))
            .into_iter()
            .collect(),
    )
    .unwrap()
}

fn battle(
    factors: [i64; 2],
    reference: RuleSelectorReference,
    snapshot: SnapshotPolicy,
    seed: u8,
) -> Battle {
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xbb; 32]).unwrap(),
        id(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(true, 1),
            ),
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(1).unwrap(),
                ParticipantSource::Player,
                combatant(true, 2),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(id(1)),
                combatant(false, 3),
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(
        catalog(factors, reference, snapshot),
        spec,
        BattleSeed::new([seed; 32]),
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

fn selected(resolution: &Resolution) -> Option<u64> {
    resolution
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(data.target.get()),
            _ => None,
        })
}

fn sum(resolution: &Resolution) -> i64 {
    resolution
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::RuleSignal(data) if data.code == 100 => match data.value {
                Some(RuleValue::Scalar(value)) => Some(value.scaled()),
                _ => None,
            },
            _ => None,
        })
        .unwrap()
}

#[test]
fn aggro_weights_apply_normalized_factors_in_current_and_event_snapshots() {
    for reference in [
        RuleSelectorReference::CurrentState,
        RuleSelectorReference::EventSnapshot,
    ] {
        for (factors, expected) in [
            ([0, 0], 2_000_000),
            ([300_000, 0], 2_300_000),
            ([0, 300_000], 2_300_000),
            ([5_000_000, 0], 7_000_000),
        ] {
            let mut battle = battle(factors, reference, SnapshotPolicy::Dynamic, 1);
            let result = start(&mut battle);
            assert!(result.fault().is_none(), "{:?}", result.fault());
            assert_eq!(sum(&result), expected);
            assert!(matches!(selected(&result), Some(1 | 2)));
            assert!(battle.view().rng_draw_count() > 0);
        }
    }
}

#[test]
fn aggro_weights_are_available_to_initial_modifier_capture() {
    let mut battle = battle(
        [300_000, 0],
        RuleSelectorReference::EventSnapshot,
        SnapshotPolicy::OnApplication,
        1,
    );
    let result = start(&mut battle);
    assert!(result.fault().is_none());
    assert_eq!(sum(&result), 2_300_000);
}

#[test]
fn aggro_zero_weight_excludes_candidates_and_all_zero_consumes_no_draw() {
    for seed in 0..16 {
        let mut excluded = battle(
            [-1_000_000, 0],
            RuleSelectorReference::EventSnapshot,
            SnapshotPolicy::Dynamic,
            seed,
        );
        let result = start(&mut excluded);
        assert!(result.fault().is_none());
        assert_eq!(selected(&result), Some(2));
    }
    let mut empty = battle(
        [-1_000_000; 2],
        RuleSelectorReference::CurrentState,
        SnapshotPolicy::Dynamic,
        1,
    );
    let result = start(&mut empty);
    assert!(result.fault().is_none());
    assert_eq!(sum(&result), 0);
    assert_eq!(selected(&result), None);
    assert_eq!(empty.view().rng_draw_count(), 0);
}

#[test]
fn aggro_negative_weight_is_a_deterministic_fault_not_a_clamped_candidate() {
    let mut first = battle(
        [-2_000_000, 0],
        RuleSelectorReference::CurrentState,
        SnapshotPolicy::Dynamic,
        1,
    );
    let mut second = battle(
        [-2_000_000, 0],
        RuleSelectorReference::CurrentState,
        SnapshotPolicy::Dynamic,
        1,
    );
    let actual = start(&mut first);
    let replayed = start(&mut second);
    assert!(actual.fault().is_some());
    assert_eq!(actual.events(), replayed.events());
    assert_eq!(actual.state_hash(), replayed.state_hash());
    assert_eq!(first.view().rng_draw_count(), 0);
}

#[test]
fn aggro_seeded_selection_changes_with_factors_and_reconstructs_exactly() {
    let mut plain_mask = 0_u64;
    let mut boosted_mask = 0_u64;
    for seed in 0..64 {
        let mut plain = battle(
            [0, 0],
            RuleSelectorReference::CurrentState,
            SnapshotPolicy::Dynamic,
            seed,
        );
        let plain_result = start(&mut plain);
        let mut boosted = battle(
            [300_000, 0],
            RuleSelectorReference::CurrentState,
            SnapshotPolicy::Dynamic,
            seed,
        );
        let mut fresh = battle(
            [300_000, 0],
            RuleSelectorReference::CurrentState,
            SnapshotPolicy::Dynamic,
            seed,
        );
        let before = fresh.state_hash();
        assert!(
            fresh
                .apply(Command::StartBattle {
                    decision: DecisionId::new(999).unwrap()
                })
                .is_err()
        );
        assert_eq!(fresh.state_hash(), before);
        assert_eq!(fresh.view().rng_draw_count(), 0);
        let actual = start(&mut boosted);
        let replayed = start(&mut fresh);
        assert!(actual.fault().is_none());
        assert_eq!(actual.events(), replayed.events());
        assert_eq!(actual.state_hash(), replayed.state_hash());
        assert_eq!(
            boosted.view().rng_draw_count(),
            fresh.view().rng_draw_count()
        );
        if selected(&plain_result) == Some(1) {
            plain_mask |= 1_u64 << seed;
        }
        if selected(&actual) == Some(1) {
            boosted_mask |= 1_u64 << seed;
        }
    }
    assert_ne!(plain_mask, boosted_mask);
    assert_eq!(plain_mask, 0x8a6f_6779_4991_e3a2);
    assert_eq!(boosted_mask, 0x8d09_bcd4_7e8d_cf5e);
    assert!(boosted_mask.count_ones() > plain_mask.count_ones());
}
