//! Explicit damage attribution does not rewrite the triggering action or snapshot.

use std::sync::Arc;

use crate::combat_decision::advance_boundary_if_offered;
use starclock_combat::{
    ActionEventData, AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattleSeed, BattleSpec,
    CauseActor, CombatantSpecDigest, Command, ConcedePolicy, DamageEventData, Energy,
    FormationIndex, Hp, ParticipantSource, ParticipantSpec, Ratio, Resolution,
    ResolvedBuildBonuses, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    ResolvedModifierBinding, Rounding, Scalar, Speed, StatValue, TeamResourceSpec, TeamSide,
    UnitEventData, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HitOperationDefinition, OrdinaryDamageDefinition, OrdinaryDamageMultipliers,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
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
        RuleEventKind, RuleEventPoint, RuleOperationTemplate, RuleSource, RuleValue, SourceClass,
        TriggerDef, TriggerPhase, ValueExpr,
    },
};

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn scalar(raw: i64) -> Scalar {
    Scalar::checked_from_integer(raw).unwrap()
}

fn runtime<I: TryFrom<u64>>(raw: u64) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn owner_damage(amount: ValueExpr, can_crit: bool, can_defeat: bool) -> RuleOperationTemplate {
    RuleOperationTemplate::DamageFromOwner {
        selector: id(2),
        amount,
        class: DamageClass::Additional,
        element: CombatElement::Physical,
        can_crit,
        can_defeat,
    }
}

fn actor_damage(amount: ValueExpr, can_crit: bool) -> RuleOperationTemplate {
    RuleOperationTemplate::Damage {
        selector: id(2),
        amount,
        class: DamageClass::Additional,
        element: CombatElement::Physical,
        can_crit,
        can_defeat: false,
    }
}

fn attack(subject: StatQuerySubject) -> ValueExpr {
    ValueExpr::QueryStat {
        subject,
        stat: StatKind::Atk,
        purpose: FormulaPurpose::Stat,
    }
}

fn owner_attack() -> ValueExpr {
    ValueExpr::Multiply {
        lhs: Box::new(attack(StatQuerySubject::Owner)),
        rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(scalar(4)))),
        rounding: Rounding::Floor,
    }
}

fn catalog_builder(operations: Vec<RuleOperationTemplate>) -> CombatCatalogBuilder {
    let mut builder = CombatCatalogBuilder::new([0xad; 32]);
    builder.add_selector(SelectorDefinition::new(id(1)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_selector(
        SelectorDefinition::new(id(2)).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::Actor,
                RuleSelectorSide::Any,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Present,
                RuleSelectorReference::EventSnapshot,
                RuleSelectorOrdering::StableId,
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
    for raw in [1, 2] {
        builder.add_program(ProgramDefinition::new(
            id(raw),
            vec![],
            vec![],
            vec![],
            vec![],
        ));
    }
    builder.add_program(
        ProgramDefinition::new(id(3), vec![], vec![id(2)], vec![], vec![])
            .with_steps(operations.into_iter().map(ProgramStep::Operation).collect()),
    );
    builder.add_rule(
        RuleDefinition::new(id(1), vec![id(3)], vec![id(2)]).with_runtime(
            BattleRuleDefinition::new(
                RuleSource::new(id(70), SourceClass::Synthetic, vec![], [0x70; 32]),
                vec![],
                vec![TriggerDef {
                    id: id(1),
                    event: RuleEventKind::Damage,
                    event_point: RuleEventPoint::DamageApplied,
                    phase: TriggerPhase::AfterEvent,
                    filter: EventFilter {
                        source: Some(id(3)),
                        ..EventFilter::default()
                    },
                    condition: ConditionExpr::Literal(true),
                    once_scope: OnceScope::Action,
                    priority: ReactionPriority::new(0),
                    program: id(3),
                }],
                None,
            ),
        ),
    );
    builder.add_rule_bundle(RuleBundle::new(id(1), vec![id(1)]));
    for (raw, factor) in [(1, 2), (2, 5)] {
        builder.add_modifier_group(ModifierStackingGroup {
            id: id(raw),
            aggregation: ModifierAggregation::Product,
            comparator: None,
        });
        builder.add_modifier(ModifierDefinition {
            id: id(raw),
            stat: StatKind::Atk,
            stage: FormulaStage::DamageFinalMultiply,
            purpose: FormulaPurpose::AdditionalDamage,
            value: ValueExpr::Literal(RuleValue::Scalar(scalar(factor))),
            stacking_group: id(raw),
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::DamageFinalMultiply,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        });
    }
    for raw in [1, 3] {
        builder.add_ability(
            AbilityDefinition::new(id(raw), id(1), id(1), vec![]).with_action(
                AbilityActionDefinition::new(
                    AbilityKind::Basic,
                    1,
                    TargetInvalidationPolicy::KeepIfPresent,
                    ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
                )
                .unwrap()
                .with_hits(vec![ActionHitDefinition::new(vec![
                    HitOperationDefinition::Damage(
                        OrdinaryDamageDefinition::new(
                            scalar(10),
                            OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                        )
                        .unwrap(),
                    ),
                ])])
                .unwrap(),
            ),
        );
    }
    builder.add_unit(UnitDefinition::new(id(1), vec![id(1)], vec![]));
    builder.add_unit(UnitDefinition::new(id(2), vec![id(3)], vec![]));
    builder.add_enemy(EnemyDefinition::new(id(1), id(2), vec![id(3)]));
    builder.add_encounter(EncounterDefinition::new(id(1), vec![id(1)], vec![]));
    builder
}

fn catalog(operations: Vec<RuleOperationTemplate>) -> Arc<CombatCatalog> {
    catalog_builder(operations).build().unwrap()
}

fn combatant(form: u32, hp: i64) -> ResolvedCombatantSpec {
    let player = form == 1;
    ResolvedCombatantSpec::new(
        id(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(hp).unwrap(),
        Speed::from_scaled(if player { 100_000_000 } else { 200_000_000 }).unwrap(),
        ResolvedDefinitionBindings::new(
            vec![id(if player { 1 } else { 3 })],
            player.then(|| id(1)).into_iter().collect(),
            vec![id(form)],
        )
        .unwrap(),
        CombatantSpecDigest::new([u8::try_from(form).unwrap(); 32]).unwrap(),
    )
    .unwrap()
    .with_base_attack_defense(
        StatValue::from_scaled(if player { 100_000_000 } else { 300_000_000 }).unwrap(),
        StatValue::from_scaled(0).unwrap(),
    )
    .with_build_bonuses(ResolvedBuildBonuses::new(
        Scalar::from_scaled(if player { 950_000 } else { 0 }),
        Scalar::ZERO,
        Scalar::ZERO,
        Scalar::ZERO,
        Scalar::ZERO,
        [Scalar::ZERO; 7],
    ))
    .with_sources(vec![RuleSource::new(
        id(80),
        SourceClass::Synthetic,
        vec![],
        [0x80; 32],
    )])
    .unwrap()
    .with_modifier_bindings(vec![ResolvedModifierBinding::new(id(form), id(80))])
    .unwrap()
}

fn battle(operations: Vec<RuleOperationTemplate>, enemy_hp: i64) -> Battle {
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xae; 32]).unwrap(),
        id(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 10_000),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(id(1)),
                combatant(2, enemy_hp),
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle =
        Battle::create(catalog(operations), spec, BattleSeed::new([0xaf; 32])).unwrap();
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    advance_boundary_if_offered(&mut battle);
    battle
}

fn execute(battle: &mut Battle) -> Resolution {
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(
            |command| matches!(command, Command::UseAbility { ability, .. } if ability.get() == 3),
        )
        .unwrap()
        .clone();
    let resolution = battle.apply(command).unwrap();
    assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    resolution
}

fn emitted_damage(resolution: &Resolution) -> Vec<(&BattleEvent, &DamageEventData)> {
    resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) if event.cause().source_definition() == Some(id(70)) => {
                Some((event, data))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn owner_damage_uses_owner_formula_and_original_actor_selector() {
    let mut battle = battle(vec![owner_damage(owner_attack(), false, false)], 10_000);
    let draws = battle.view().rng_draw_count();
    let resolution = execute(&mut battle);
    let damage = emitted_damage(&resolution)[0].1;
    assert_eq!(
        damage.target.get(),
        2,
        "the original actor remains the selected target"
    );
    assert_eq!(
        (damage.raw.scaled(), damage.calculated.get()),
        (800_000_000, 800)
    );
    assert_eq!(damage.class, DamageClass::Additional);
    assert_eq!(damage.element, Some(CombatElement::Physical));
    assert_eq!(battle.view().rng_draw_count(), draws);
}

#[test]
fn owner_damage_preserves_ancestry_without_declaring_another_action() {
    let mut battle = battle(vec![owner_damage(owner_attack(), false, false)], 10_000);
    let resolution = execute(&mut battle);
    let original = resolution
        .events()
        .iter()
        .find(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
        .unwrap();
    let emitted = emitted_damage(&resolution)[0].0;
    let cause = emitted.cause();
    assert_eq!(cause.owner(), Some(runtime(1)));
    assert_eq!(cause.actor(), Some(CauseActor::Unit(runtime(1))));
    assert_eq!(cause.applier(), Some(runtime(1)));
    assert_eq!(cause.source_definition(), Some(id(70)));
    assert_eq!(cause.root_command(), original.cause().root_command());
    assert_eq!(cause.action(), original.cause().action());
    assert_eq!(cause.phase(), original.cause().phase());
    assert_eq!(cause.hit(), original.cause().hit());
    let parent = resolution
        .events()
        .iter()
        .find(|event| Some(event.id()) == cause.parent_event())
        .unwrap();
    assert_eq!(parent.cause().action(), original.cause().action());
    assert_eq!(
        resolution
            .events()
            .iter()
            .filter(|event| matches!(
                event.kind(),
                BattleEventKind::Action(ActionEventData::Declared { .. })
            ))
            .count(),
        1
    );
}

#[test]
fn expressions_still_read_the_original_actor_and_owner_separately() {
    let mut battle = battle(
        vec![
            owner_damage(attack(StatQuerySubject::Actor), false, false),
            owner_damage(attack(StatQuerySubject::Owner), false, false),
        ],
        10_000,
    );
    let resolution = execute(&mut battle);
    assert_eq!(
        emitted_damage(&resolution)
            .iter()
            .map(|(_, data)| data.calculated.get())
            .collect::<Vec<_>>(),
        [600, 200]
    );
}

#[test]
fn ordinary_damage_retains_observed_actor_attribution_and_modifiers() {
    let mut battle = battle(vec![actor_damage(owner_attack(), false)], 10_000);
    let resolution = execute(&mut battle);
    let (event, data) = emitted_damage(&resolution)[0];
    assert_eq!(data.calculated.get(), 2_000);
    assert_eq!(event.cause().owner(), Some(runtime(1)));
    assert_eq!(event.cause().actor(), Some(CauseActor::Unit(runtime(2))));
    assert_eq!(event.cause().applier(), Some(runtime(2)));
}

#[test]
fn owner_critical_group_does_not_reuse_or_pollute_actor_critical_cache() {
    let mut battle = battle(
        vec![
            actor_damage(attack(StatQuerySubject::Actor), true),
            owner_damage(attack(StatQuerySubject::Actor), true, false),
            actor_damage(attack(StatQuerySubject::Actor), true),
        ],
        10_000,
    );
    let resolution = execute(&mut battle);
    assert_eq!(
        emitted_damage(&resolution)
            .iter()
            .map(|(_, data)| data.calculated.get())
            .collect::<Vec<_>>(),
        [1_500, 900, 1_500]
    );
}

#[test]
fn owner_damage_nonlethal_floor_and_lethal_credit_are_explicit() {
    for (hp, expected_loss, expected_after) in
        [(1, 0, 1), (99, 98, 1), (801, 800, 1), (802, 800, 2)]
    {
        let mut battle = battle(vec![owner_damage(owner_attack(), false, false)], hp);
        let resolution = execute(&mut battle);
        let damage = emitted_damage(&resolution)[0].1;
        assert_eq!(
            (damage.applied.get(), damage.hp_after.get()),
            (expected_loss, expected_after)
        );
        assert!(!resolution.events().iter().any(|event| matches!(
            event.kind(),
            BattleEventKind::Unit(UnitEventData::Defeated { .. })
        )));
    }
    let mut battle = battle(vec![owner_damage(owner_attack(), false, true)], 99);
    let resolution = execute(&mut battle);
    assert_eq!(emitted_damage(&resolution)[0].1.hp_after.get(), 0);
    assert!(resolution.events().iter().any(|event| matches!(event.kind(), BattleEventKind::Unit(UnitEventData::Defeated { unit, credited_to }) if unit.get() == 2 && credited_to.get() == 1)));
}

#[test]
fn rejected_command_does_not_change_owner_damage_reconstruction() {
    let operations = vec![owner_damage(owner_attack(), false, false)];
    let mut first = battle(operations.clone(), 10_000);
    let mut fresh = battle(operations, 10_000);
    let hash = first.state_hash();
    let draws = first.view().rng_draw_count();
    let revision = first.view().committed_revision();
    assert!(
        first
            .apply(Command::StartBattle {
                decision: runtime(1)
            })
            .is_err()
    );
    assert_eq!(first.state_hash(), hash);
    assert_eq!(first.view().rng_draw_count(), draws);
    assert_eq!(first.view().committed_revision(), revision);
    let actual = execute(&mut first);
    let replayed = execute(&mut fresh);
    assert_eq!(actual.events(), replayed.events());
    assert_eq!(actual.state_hash(), replayed.state_hash());
    assert_eq!(first.state_hash(), fresh.state_hash());
}

#[test]
fn owner_damage_requires_scalar_amount_and_resolved_selector() {
    assert!(
        catalog_builder(vec![owner_damage(
            ValueExpr::Literal(RuleValue::Integer(4)),
            false,
            false,
        )])
        .build()
        .is_err()
    );
    let mut unresolved = owner_damage(owner_attack(), false, false);
    if let RuleOperationTemplate::DamageFromOwner { selector, .. } = &mut unresolved {
        *selector = id(404);
    }
    assert!(catalog_builder(vec![unresolved]).build().is_err());
}

#[test]
fn owner_damage_finalizes_fractional_amounts_only_after_source_multiplication() {
    for (raw, expected_raw, expected_integral) in [
        (1, 2, 0),
        (499_999, 999_998, 0),
        (500_000, 1_000_000, 1),
        (999_999, 1_999_998, 1),
    ] {
        let mut battle = battle(
            vec![owner_damage(
                ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(raw))),
                false,
                false,
            )],
            10_000,
        );
        let resolution = execute(&mut battle);
        let data = emitted_damage(&resolution)[0].1;
        assert_eq!(
            (data.raw.scaled(), data.calculated.get()),
            (expected_raw, expected_integral)
        );
    }
}
