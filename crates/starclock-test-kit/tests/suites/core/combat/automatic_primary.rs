//! Normal enemy commands commit a sampled primary before the shared envelope.

use crate::combat_decision::settle_ready_boundaries;
use starclock_combat::{
    AbilityId, ActionEventData, ActionOrigin, AssemblyDigest, Battle, BattleEventKind, BattleSeed,
    BattleSpec, CombatantSpecDigest, Command, ConcedePolicy, DecisionId, Energy, FormationIndex,
    HitEventData, Hp, LifeState, ParticipantInitialState, ParticipantSource, ParticipantSpec,
    PresenceState, Ratio, Resolution, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    ResolvedModifierBinding, Scalar, Speed, TeamResourceSpec, TeamSide, UnitId, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HitCritPolicy, HitOperationDefinition, HitTargetGroup, OrdinaryDamageDefinition,
            OrdinaryDamageMultipliers, TargetInvalidationPolicy, TargetPattern, TargetRelation,
            UnitTargetSelector,
        },
        builder::{CatalogBuildErrorKind, CombatCatalogBuilder},
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind, StatQuerySubject,
    },
    rule::model::{RuleSource, RuleValue, SourceClass, ValueExpr},
};
use std::sync::Arc;

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn selector(
    reference: RuleSelectorReference,
    side: RuleSelectorSide,
    maximum: u16,
    empty: RuleEmptyPoolPolicy,
    origin: RuleSelectorOrigin,
) -> RuleUnitSelector {
    RuleUnitSelector::new(
        origin,
        side,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        reference,
        RuleSelectorOrdering::Formation,
        1,
        maximum,
        empty,
        RuleSelectorChoice::RngWeighted,
        Some("aggro-target".into()),
        false,
    )
    .unwrap()
    .with_weight(Some(ValueExpr::QueryStat {
        subject: StatQuerySubject::CurrentTarget,
        stat: StatKind::Aggro,
        purpose: FormulaPurpose::Aggro,
    }))
}

fn primary() -> RuleUnitSelector {
    selector(
        RuleSelectorReference::CurrentState,
        RuleSelectorSide::Opposing,
        1,
        RuleEmptyPoolPolicy::Fault,
        RuleSelectorOrigin::Encounter,
    )
}

#[test]
fn automatic_primary_maximum_predicate_rejects_trigger_reads_and_accepts_safe_stats() {
    for value in [
        ValueExpr::EventId,
        ValueExpr::Slot(id(1)),
        ValueExpr::QueryHp {
            subject: StatQuerySubject::CurrentTarget,
        },
        ValueExpr::QueryStat {
            subject: StatQuerySubject::EventTarget,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        },
    ] {
        let plan = primary().with_predicates(vec![RuleSelectorPredicate::MaximumValue(value)]);
        assert_eq!(
            builder(true, TargetPattern::Single, Some(plan), [0; 3])
                .build()
                .unwrap_err()
                .kind(),
            CatalogBuildErrorKind::InvalidDefinition
        );
    }
    let plan = primary().with_predicates(vec![RuleSelectorPredicate::MaximumValue(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::CurrentTarget,
            stat: StatKind::Aggro,
            purpose: FormulaPurpose::Aggro,
        },
    )]);
    let catalog = builder(true, TargetPattern::Single, Some(plan), [0, 1_000_000, 0])
        .build()
        .unwrap();
    let mut battle = battle(catalog, 1, None);
    let result = attack(&mut battle);
    assert!(result.fault().is_none());
    // The inherited weighted selector keeps its existing singleton draw policy.
    assert_eq!(battle.view().rng_draw_count(), 1);
    assert!(result.events().iter().any(|event| matches!(event.kind(),
        BattleEventKind::Damage(data) if data.target == UnitId::new(2).unwrap())));
    assert!(!result.events().iter().any(|event| matches!(event.kind(),
        BattleEventKind::Damage(data) if data.target != UnitId::new(2).unwrap())));
}

fn builder(
    automatic: bool,
    pattern: TargetPattern,
    primary: Option<RuleUnitSelector>,
    factors: [i64; 3],
) -> CombatCatalogBuilder {
    let mut builder = CombatCatalogBuilder::new([0xc1; 32]);
    builder.add_selector(SelectorDefinition::new(id(1)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single).unwrap(),
    ));
    builder
        .add_selector(SelectorDefinition::new(id(2)).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::Opposing, pattern).unwrap(),
        ));
    if let Some(primary) = primary {
        builder.add_selector(SelectorDefinition::new(id(3)).with_rule_units(primary));
    }
    builder.add_program(ProgramDefinition::new(
        id(1),
        vec![],
        vec![],
        vec![],
        vec![],
    ));
    builder.add_ability(
        AbilityDefinition::new(id(1), id(1), id(1), vec![]).with_action(
            AbilityActionDefinition::new(
                AbilityKind::Basic,
                1,
                TargetInvalidationPolicy::CancelRemainingForTarget,
                ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
            )
            .unwrap(),
        ),
    );
    let hit = ActionHitDefinition::new(vec![HitOperationDefinition::Damage(
        OrdinaryDamageDefinition::new(
            Scalar::checked_from_integer(100).unwrap(),
            OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
        )
        .unwrap(),
    )])
    .with_profile(
        HitTargetGroup::Selected,
        Ratio::ONE,
        Ratio::ONE,
        HitCritPolicy::Never,
    );
    let enemy = AbilityDefinition::new(id(2), id(1), id(2), vec![]).with_action(
        AbilityActionDefinition::new(
            AbilityKind::Basic,
            2,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
        )
        .unwrap()
        .with_hits(vec![hit.clone(), hit])
        .unwrap(),
    );
    builder.add_ability(if automatic {
        enemy.with_automatic_primary_selector(id(3))
    } else {
        enemy
    });
    for (index, factor) in factors.into_iter().enumerate() {
        let raw = u32::try_from(index).unwrap() + 1;
        builder.add_modifier_group(ModifierStackingGroup {
            id: id(raw),
            aggregation: ModifierAggregation::Sum,
            comparator: None,
        });
        builder.add_modifier(ModifierDefinition {
            id: id(raw),
            stat: StatKind::Aggro,
            stage: FormulaStage::PercentOfBase,
            purpose: FormulaPurpose::Aggro,
            value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(factor))),
            stacking_group: id(raw),
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::PercentOfBase,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        });
    }
    builder.add_unit(UnitDefinition::new(id(1), vec![id(1)], vec![]));
    builder.add_unit(UnitDefinition::new(id(2), vec![id(2)], vec![]));
    builder.add_enemy(EnemyDefinition::new(id(1), id(2), vec![id(2)]));
    builder.add_encounter(EncounterDefinition::new(id(1), vec![id(1)], vec![]));
    builder
}

fn battle(
    catalog: Arc<CombatCatalog>,
    seed: u8,
    excluded: Option<(LifeState, PresenceState)>,
) -> Battle {
    battle_with_enemy_abilities(catalog, seed, excluded, vec![id(2)])
}

fn battle_with_enemy_abilities(
    catalog: Arc<CombatCatalog>,
    seed: u8,
    excluded: Option<(LifeState, PresenceState)>,
    enemy_abilities: Vec<AbilityId>,
) -> Battle {
    let maximum = Hp::new(1_000).unwrap();
    let mut participants = Vec::new();
    for index in 1..=4 {
        let enemy = index == 4;
        let combatant = ResolvedCombatantSpec::new(
            id(if enemy { 2 } else { 1 }),
            UnitLevel::new(80).unwrap(),
            maximum,
            Speed::from_scaled(if enemy { 300_000_000 } else { 100_000_000 }).unwrap(),
            ResolvedDefinitionBindings::new(
                if enemy {
                    enemy_abilities.clone()
                } else {
                    vec![id(1)]
                },
                vec![],
                if enemy { vec![] } else { vec![id(index)] },
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
        .with_modifier_bindings(if enemy {
            vec![]
        } else {
            vec![ResolvedModifierBinding::new(id(index), id(80))]
        })
        .unwrap();
        let mut participant = ParticipantSpec::new(
            if enemy {
                TeamSide::Enemy
            } else {
                TeamSide::Player
            },
            FormationIndex::new(if enemy {
                0
            } else {
                u8::try_from(index - 1).unwrap()
            })
            .unwrap(),
            if enemy {
                ParticipantSource::EncounterEnemy(id(1))
            } else {
                ParticipantSource::Player
            },
            combatant,
        );
        if index == 1
            && let Some((life, presence)) = excluded
        {
            let hp = if life == LifeState::Alive {
                maximum
            } else {
                Hp::new(0).unwrap()
            };
            participant = participant
                .with_initial_state(
                    ParticipantInitialState::new(
                        hp,
                        maximum,
                        Energy::ZERO,
                        Energy::ZERO,
                        life,
                        presence,
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        participants.push(participant);
    }
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xc2; 32]).unwrap(),
        id(1),
        participants,
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle = Battle::create(catalog, spec, BattleSeed::new([seed; 32])).unwrap();
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    settle_ready_boundaries(&mut battle);
    battle
}

fn automatic(seed: u8, factors: [i64; 3], pattern: TargetPattern) -> Battle {
    battle(
        builder(true, pattern, Some(primary()), factors)
            .build()
            .unwrap(),
        seed,
        None,
    )
}

fn attack(battle: &mut Battle) -> Resolution {
    let commands = battle.decision().unwrap().legal_commands();
    assert_eq!(commands.len(), 1);
    assert!(matches!(
        &commands[0],
        Command::UseAbility {
            actor,
            ability,
            primary_target: None,
            ..
        } if actor.get() == 4 && ability.get() == 2
    ));
    battle.apply(commands[0].clone()).unwrap()
}

fn chosen(result: &Resolution) -> u64 {
    result.events().iter().find_map(|event| matches!(event.kind(),
        BattleEventKind::Action(ActionEventData::Declared { actor, origin: ActionOrigin::NormalTurn, .. }) if actor.get() == 4)
        .then(|| event.cause().primary_target().unwrap().get())).unwrap()
}

#[test]
fn automatic_primary_weights_change_real_normal_attacks_and_replay_exactly() {
    let mut plain_mask = 0_u64;
    let mut boosted_mask = 0_u64;
    for seed in 0..64 {
        let mut plain = automatic(seed, [0, 0, -1_000_000], TargetPattern::Single);
        let mut boosted = automatic(seed, [300_000, 0, -1_000_000], TargetPattern::Single);
        let mut fresh = automatic(seed, [300_000, 0, -1_000_000], TargetPattern::Single);
        assert_eq!(boosted.view().rng_draw_count(), 0);
        let ordinary = attack(&mut plain);
        let actual = attack(&mut boosted);
        let replayed = attack(&mut fresh);
        assert!(actual.fault().is_none(), "{:?}", actual.fault());
        assert_eq!(actual.events(), replayed.events());
        assert_eq!(actual.state_hash(), replayed.state_hash());
        assert_eq!(
            boosted.view().rng_draw_count(),
            fresh.view().rng_draw_count()
        );
        assert!(boosted.view().rng_draw_count() > 0);
        let target = chosen(&actual);
        assert!(matches!(target, 1 | 2));
        let damage = actual
            .events()
            .iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::Damage(data) => Some(data),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(damage.len(), 2);
        assert!(damage.iter().all(|data| data.target.get() == target));
        assert!(
            boosted
                .view()
                .units_by_id()
                .filter(|unit| unit.side() == TeamSide::Player)
                .all(|unit| (unit.current_hp().get() < 1_000) == (unit.id().get() == target))
        );
        if chosen(&ordinary) == 1 {
            plain_mask |= 1_u64 << seed;
        }
        if target == 1 {
            boosted_mask |= 1_u64 << seed;
        }
    }
    assert_eq!(plain_mask, 0x8a6f_6779_4991_e3a2, "{plain_mask:#x}");
    assert_eq!(boosted_mask, 0x8d09_bcd4_7e8d_cf5e, "{boosted_mask:#x}");
    assert!(boosted_mask.count_ones() > plain_mask.count_ones());
}

#[test]
fn automatic_primary_blast_commits_one_envelope_and_reuses_its_primary_for_all_hits() {
    let mut battle = automatic(2, [-1_000_000, 0, -1_000_000], TargetPattern::Blast);
    let result = attack(&mut battle);
    assert!(result.fault().is_none());
    assert_eq!(chosen(&result), 2);
    let declarations = result
        .events()
        .iter()
        .filter(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Action(ActionEventData::Declared { .. })
            )
        })
        .count();
    assert_eq!(declarations, 1);
    let hits = result
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Hit(HitEventData::Started { targets, .. }) => Some((event, targets)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(hits.len(), 2);
    for (event, targets) in hits {
        assert_eq!(event.cause().primary_target().unwrap().get(), 2);
        assert_eq!(
            targets.iter().map(|id| id.get()).collect::<Vec<_>>(),
            [1, 2, 3]
        );
    }
    assert_eq!(
        result
            .events()
            .iter()
            .filter(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
            .count(),
        6
    );
    assert_eq!(battle.view().rng_draw_count(), 1);
    assert!(
        battle
            .view()
            .units_by_id()
            .filter(|unit| unit.side() == TeamSide::Player)
            .all(|unit| unit.current_hp().get() < 1_000)
    );
    let boundary = battle.view().action_boundary().unwrap();
    assert_eq!(boundary.turn().side(), TeamSide::Enemy);
    assert_eq!(boundary.turn().owner().get(), 4);
}

#[test]
fn automatic_primary_rejects_manual_and_stale_input_without_advancing_state_or_rng() {
    let mut battle = automatic(1, [0, 0, -1_000_000], TargetPattern::Single);
    let command = battle.decision().unwrap().legal_commands()[0].clone();
    let Command::UseAbility {
        decision,
        actor,
        ability,
        ..
    } = command.clone()
    else {
        panic!("enemy action required")
    };
    let before = battle.state_hash();
    for rejected in [
        Command::UseAbility {
            decision,
            actor,
            ability,
            primary_target: Some(UnitId::new(1).unwrap()),
        },
        Command::UseAbility {
            decision: DecisionId::new(999).unwrap(),
            actor,
            ability,
            primary_target: None,
        },
    ] {
        assert!(battle.apply(rejected).is_err());
        assert_eq!(battle.state_hash(), before);
        assert_eq!(battle.view().rng_draw_count(), 0);
    }
    let mut fresh = automatic(1, [0, 0, -1_000_000], TargetPattern::Single);
    let actual = battle.apply(command).unwrap();
    let expected = attack(&mut fresh);
    assert_eq!(actual.events(), expected.events());
    assert_eq!(actual.state_hash(), expected.state_hash());
}

#[test]
fn automatic_primary_empty_zero_and_negative_pools_fault_before_declaration_and_payment() {
    for (factors, filtered) in [
        ([-1_000_000; 3], false),
        ([-2_000_000, 0, 0], false),
        ([0; 3], true),
    ] {
        let selector = if filtered {
            primary().with_predicates(vec![RuleSelectorPredicate::FormationRange {
                minimum: 7,
                maximum: 7,
            }])
        } else {
            primary()
        };
        let catalog = builder(true, TargetPattern::Single, Some(selector), factors)
            .build()
            .unwrap();
        let mut first = battle(Arc::clone(&catalog), 1, None);
        let mut fresh = battle(catalog, 1, None);
        let result = attack(&mut first);
        let repeated = attack(&mut fresh);
        assert!(result.fault().is_some());
        assert_eq!(result.events(), repeated.events());
        assert_eq!(result.state_hash(), repeated.state_hash());
        assert!(result.events().iter().all(|event| !matches!(
            event.kind(),
            BattleEventKind::Action(_) | BattleEventKind::Damage(_)
        )));
        assert_eq!(first.view().rng_draw_count(), 0);
        assert!(
            first
                .view()
                .units_by_id()
                .all(|unit| unit.current_hp().get() == 1_000)
        );
    }
}

#[test]
fn automatic_primary_excludes_dead_and_departed_units_even_with_large_authored_weights() {
    for excluded in [
        (LifeState::Defeated, PresenceState::Present),
        (LifeState::Alive, PresenceState::Departed),
    ] {
        for seed in 0..8 {
            let catalog = builder(
                true,
                TargetPattern::Single,
                Some(primary()),
                [99_000_000, 0, -1_000_000],
            )
            .build()
            .unwrap();
            let mut battle = battle(catalog, seed, Some(excluded));
            let result = attack(&mut battle);
            assert!(result.fault().is_none());
            assert_eq!(chosen(&result), 2);
        }
    }
}

#[test]
fn unbound_normal_abilities_preserve_manual_targets_and_do_not_read_aggro() {
    let catalog = builder(
        false,
        TargetPattern::Single,
        Some(primary()),
        [0, 0, -1_000_000],
    )
    .build()
    .unwrap();
    let mut battle = battle(catalog, 1, None);
    let commands = battle.decision().unwrap().legal_commands();
    assert_eq!(commands.len(), 3);
    let command = commands.iter().find(|command| matches!(command, Command::UseAbility { primary_target: Some(target), .. } if target.get() == 3)).unwrap().clone();
    let result = battle.apply(command).unwrap();
    assert!(result.fault().is_none());
    assert_eq!(chosen(&result), 3);
    assert_eq!(battle.view().rng_draw_count(), 0);
}

#[test]
fn automatic_primary_catalog_rejects_unavailable_contexts_and_invalid_shape_contracts() {
    assert_eq!(
        builder(true, TargetPattern::Single, None, [0; 3])
            .build()
            .unwrap_err()
            .kind(),
        CatalogBuildErrorKind::MissingReference
    );
    let cases = [
        selector(
            RuleSelectorReference::EventSnapshot,
            RuleSelectorSide::Opposing,
            1,
            RuleEmptyPoolPolicy::Fault,
            RuleSelectorOrigin::Encounter,
        ),
        selector(
            RuleSelectorReference::ActionSnapshot,
            RuleSelectorSide::Opposing,
            1,
            RuleEmptyPoolPolicy::Fault,
            RuleSelectorOrigin::Encounter,
        ),
        selector(
            RuleSelectorReference::CurrentState,
            RuleSelectorSide::Same,
            1,
            RuleEmptyPoolPolicy::Fault,
            RuleSelectorOrigin::Encounter,
        ),
        selector(
            RuleSelectorReference::CurrentState,
            RuleSelectorSide::Opposing,
            2,
            RuleEmptyPoolPolicy::Fault,
            RuleSelectorOrigin::Encounter,
        ),
        selector(
            RuleSelectorReference::CurrentState,
            RuleSelectorSide::Opposing,
            1,
            RuleEmptyPoolPolicy::NoOp,
            RuleSelectorOrigin::Encounter,
        ),
        selector(
            RuleSelectorReference::CurrentState,
            RuleSelectorSide::Opposing,
            1,
            RuleEmptyPoolPolicy::Fault,
            RuleSelectorOrigin::PrimaryTarget,
        ),
        primary().with_predicates(vec![RuleSelectorPredicate::AdjacentToPrimary]),
        primary().with_weight(Some(ValueExpr::QueryStat {
            subject: StatQuerySubject::EventTarget,
            stat: StatKind::Aggro,
            purpose: FormulaPurpose::Aggro,
        })),
        primary().with_weight(Some(ValueExpr::EventId)),
        primary().with_candidate_union(vec![id(3)]).unwrap(),
    ];
    for selector in cases {
        assert_eq!(
            builder(true, TargetPattern::Single, Some(selector), [0; 3])
                .build()
                .unwrap_err()
                .kind(),
            CatalogBuildErrorKind::InvalidDefinition
        );
    }
    assert_eq!(
        builder(true, TargetPattern::All, Some(primary()), [0; 3])
            .build()
            .unwrap_err()
            .kind(),
        CatalogBuildErrorKind::InvalidDefinition
    );
}

#[test]
fn automatic_primary_first_and_uniform_share_the_same_commitment_boundary() {
    for choice in [RuleSelectorChoice::First, RuleSelectorChoice::RngUniform] {
        let selector = RuleUnitSelector::new(
            RuleSelectorOrigin::Team,
            RuleSelectorSide::Opposing,
            RuleLifePredicate::Alive,
            RulePresencePredicate::Present,
            RuleSelectorReference::CurrentState,
            RuleSelectorOrdering::Formation,
            1,
            1,
            RuleEmptyPoolPolicy::Fault,
            choice,
            (choice == RuleSelectorChoice::RngUniform).then(|| "aggro-target".into()),
            false,
        )
        .unwrap();
        let catalog = builder(
            true,
            TargetPattern::Single,
            Some(selector),
            [99_000_000, 0, -1_000_000],
        )
        .build()
        .unwrap();
        let mut battle = battle(catalog, 1, None);
        let result = attack(&mut battle);
        assert!(result.fault().is_none());
        assert!(matches!(chosen(&result), 1..=3));
        if choice == RuleSelectorChoice::First {
            assert_eq!(chosen(&result), 1);
            assert_eq!(battle.view().rng_draw_count(), 0);
        } else {
            assert!(battle.view().rng_draw_count() > 0);
        }
    }
}

#[test]
fn automatic_primary_unpayable_action_is_not_offered_and_cannot_draw_rng() {
    let base = builder(true, TargetPattern::Single, Some(primary()), [0; 3])
        .build()
        .unwrap();
    let mut overlay = CombatCatalogBuilder::from_catalog(&base, [0xc3; 32]);
    let ability = base.ability(id(2)).unwrap().clone();
    let action = AbilityActionDefinition::new(
        AbilityKind::Basic,
        2,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(1, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap()
    .with_hits(ability.action().unwrap().hits().to_vec())
    .unwrap();
    assert!(overlay.replace_ability(ability.with_action(action)));
    // Every combatant must retain a free normal ability at construction.
    let mut battle =
        battle_with_enemy_abilities(overlay.build().unwrap(), 1, None, vec![id(1), id(2)]);
    let decision = battle.decision().unwrap();
    assert!(
        !decision
            .legal_commands()
            .iter()
            .any(|command| matches!(command,
        Command::UseAbility { actor, ability, .. } if actor.get() == 4 && ability.get() == 2))
    );
    let rejected = Command::UseAbility {
        decision: decision.id(),
        actor: UnitId::new(4).unwrap(),
        ability: id(2),
        primary_target: None,
    };
    let before = battle.state_hash();
    assert!(battle.apply(rejected).is_err());
    assert_eq!(battle.state_hash(), before);
    assert_eq!(battle.view().rng_draw_count(), 0);
}
