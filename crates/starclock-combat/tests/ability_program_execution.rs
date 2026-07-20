use std::sync::Arc;

use starclock_combat::{
    ActionGauge, Battle, BattleEventKind, BattleSeed, BattleSpec, BattleSpecDigest,
    CombatantSpecDigest, Command, ConcedePolicy, CountdownCatalogDefinition, CountdownDefinition,
    DispelCategory, DurationClock, EffectCategory, EffectRuntimeDefinition, EffectRuntimeTemplate,
    EffectStackPolicy, EffectTickPhase, EncounterWaveId, FormationIndex, Hp, KeyedTeamResourceSpec,
    LinkedEntityKind, LinkedUnitCatalogDefinition, LinkedUnitDefinition, OwnerLinkPolicy,
    ParticipantSource, ParticipantSpec, PresenceState, Ratio, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, ResolvedModifierBinding, Rounding, Scalar, SourceDefinitionId,
    Speed, StatValue, TeamResourceSpec, TeamResourceWavePolicy, TeamSide, UnitLevel,
    WaveLinkPolicy,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            ActionHitDefinition, ActionResourcePolicy, HitOperationDefinition, HitTargetGroup,
            OrdinaryDamageDefinition, OrdinaryDamageMultipliers, TargetInvalidationPolicy,
            TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, CharacterResourceDefinition, EffectDefinition, EncounterDefinition,
            EnemyDefinition, ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition,
            UnitDefinition,
        },
        encounter::{EncounterWaveDefinition, WaveCarry, WaveSlotDefinition, WaveTransitionPolicy},
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
        BattleRuleDefinition, BattleRuleScope, ConditionExpr, EventFilter, OnceScope, ProgramStep,
        ReactionPriority, ResourceUpdateKind, RuleActionOwner, RuleActionPaymentPolicy,
        RuleEffectChancePolicy, RuleEventKind, RuleOperationTemplate, RuleResourceKind, RuleSource,
        RuleValue, RuleValueKind, SourceClass, StateSlotDef, StateSlotUpdateKind, TriggerDef,
        TriggerPhase, ValueExpr,
    },
};

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn empty_action() -> AbilityActionDefinition {
    AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(
            0,
            0,
            starclock_combat::Energy::ZERO,
            starclock_combat::Energy::ZERO,
        ),
    )
    .unwrap()
}

fn catalog(
    program: ProgramDefinition,
    with_modifier: bool,
    with_rule: bool,
    recursive_rule: bool,
    mechanics_rule: bool,
) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new("ability-program-v1", [0x43; 32]);
    let authored_effects = program.effects().to_vec();
    builder.add_selector(SelectorDefinition::new(id(1)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_selector(
        SelectorDefinition::new(id(2)).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::PrimaryTarget,
                RuleSelectorSide::Opposing,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Present,
                RuleSelectorReference::CurrentState,
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
    builder.add_selector(SelectorDefinition::new(id(3)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_program(program);
    if authored_effects.binary_search(&id(1)).is_ok() {
        let template = EffectRuntimeTemplate::new(
            EffectCategory::Dot,
            DispelCategory::DispellableDebuff,
            1,
            Some(ValueExpr::Literal(RuleValue::Integer(2))),
            DurationClock::TargetTurnStart,
            EffectTickPhase::TurnStart,
            EffectStackPolicy::Refresh,
        )
        .unwrap()
        .with_comparison(
            Some(ValueExpr::QueryStat {
                subject: StatQuerySubject::Actor,
                stat: StatKind::Atk,
                purpose: FormulaPurpose::Stat,
            }),
            0,
        )
        .with_dot(CombatElement::Lightning, None)
        .unwrap();
        builder.add_effect(
            EffectDefinition::new(
                id(1),
                with_rule.then(|| id(1)).into_iter().collect(),
                with_modifier.then(|| id(1)).into_iter().collect(),
            )
            .with_runtime_template(template),
        );
    }
    if with_rule {
        if mechanics_rule {
            add_mechanics_definitions(&mut builder);
        }
        let steps = if mechanics_rule {
            mechanics_steps()
        } else {
            vec![ProgramStep::Operation(RuleOperationTemplate::Damage {
                selector: id(2),
                amount: ValueExpr::Literal(RuleValue::Scalar(
                    Scalar::checked_from_integer(if recursive_rule { 0 } else { 50 }).unwrap(),
                )),
                class: DamageClass::Additional,
                element: CombatElement::Physical,
                can_crit: false,
            })]
        };
        builder.add_program(
            ProgramDefinition::new(
                id(3),
                vec![],
                if mechanics_rule {
                    vec![id(2), id(4)]
                } else {
                    vec![id(2)]
                },
                mechanics_rule.then(|| id(1)).into_iter().collect(),
                vec![],
            )
            .with_steps(steps),
        );
        let source = RuleSource::new(
            SourceDefinitionId::new(60).unwrap(),
            SourceClass::Progression,
            vec![],
            [0x60; 32],
        );
        builder.add_rule(
            RuleDefinition::new(
                id(1),
                vec![id(3)],
                if mechanics_rule {
                    vec![id(2), id(4)]
                } else {
                    vec![id(2)]
                },
            )
            .with_runtime(BattleRuleDefinition::new(
                source,
                mechanics_rule.then(mechanics_slot).into_iter().collect(),
                vec![TriggerDef {
                    id: id(1),
                    event: if recursive_rule {
                        RuleEventKind::Damage
                    } else {
                        RuleEventKind::Hit
                    },
                    phase: TriggerPhase::AfterEvent,
                    filter: if mechanics_rule {
                        EventFilter {
                            source: Some(SourceDefinitionId::new(1).unwrap()),
                            ..EventFilter::default()
                        }
                    } else {
                        EventFilter::default()
                    },
                    condition: ConditionExpr::Literal(true),
                    once_scope: if recursive_rule {
                        OnceScope::Event
                    } else {
                        OnceScope::Action
                    },
                    priority: ReactionPriority::new(0),
                    program: id(3),
                }],
                None,
            )),
        );
        builder.add_rule_bundle(RuleBundle::new(id(1), vec![id(1)]));
    }
    if with_modifier {
        builder.add_modifier_group(ModifierStackingGroup {
            id: id(1),
            aggregation: ModifierAggregation::Sum,
        });
        builder.add_modifier(ModifierDefinition {
            id: id(1),
            stat: StatKind::Atk,
            stage: FormulaStage::Flat,
            purpose: FormulaPurpose::Stat,
            value: ValueExpr::Literal(RuleValue::Scalar(
                Scalar::checked_from_integer(200).unwrap(),
            )),
            stacking_group: id(1),
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::Flat,
            snapshot: SnapshotPolicy::Dynamic,
            filters: Box::new([]),
        });
    }
    builder.add_program(ProgramDefinition::new(
        id(2),
        vec![],
        vec![],
        vec![],
        vec![],
    ));
    let hits = vec![
        ActionHitDefinition::new(vec![]).with_profile(
            HitTargetGroup::Primary,
            Ratio::from_scaled(250_000),
            Ratio::ONE,
            starclock_combat::catalog::action::HitCritPolicy::Never,
        ),
        ActionHitDefinition::new(vec![]).with_profile(
            HitTargetGroup::Primary,
            Ratio::from_scaled(750_000),
            Ratio::ONE,
            starclock_combat::catalog::action::HitCritPolicy::Never,
        ),
    ];
    let action = AbilityActionDefinition::new(
        AbilityKind::Basic,
        2,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(
            0,
            0,
            starclock_combat::Energy::ZERO,
            starclock_combat::Energy::ZERO,
        ),
    )
    .unwrap()
    .with_hits(hits)
    .unwrap();
    builder.add_ability(
        AbilityDefinition::new(id(1), id(1), id(1), authored_effects)
            .with_action(action)
            .with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, id(1)).unwrap(),
            ]),
    );
    builder.add_ability(
        AbilityDefinition::new(id(2), id(2), id(3), vec![]).with_action(empty_action()),
    );
    let mut player = UnitDefinition::new(
        id(1),
        vec![id(1)],
        with_rule.then(|| id(1)).into_iter().collect(),
    );
    if mechanics_rule {
        player = player.with_resources(vec![
            CharacterResourceDefinition::new(
                "enhanced-counter-charges",
                Scalar::ZERO,
                Scalar::checked_from_integer(2).unwrap(),
            )
            .unwrap(),
        ]);
    }
    builder.add_unit(player);
    builder.add_unit(UnitDefinition::new(id(2), vec![id(2)], vec![]));
    builder.add_enemy(EnemyDefinition::new(id(1), id(2), vec![id(2)]));
    builder.add_encounter(
        EncounterDefinition::new(id(1), vec![id(1)], vec![])
            .with_authored_waves(
                WaveTransitionPolicy::AfterAction,
                vec![
                    EncounterWaveDefinition::new(
                        id::<EncounterWaveId>(1),
                        1,
                        None,
                        None,
                        WaveCarry::CARRY_ALL,
                        vec![
                            WaveSlotDefinition::new(
                                1,
                                FormationIndex::new(4).unwrap(),
                                id(1),
                                None,
                                None,
                                true,
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
    );
    builder.build().unwrap()
}

fn mechanics_slot() -> StateSlotDef {
    StateSlotDef::new(
        id(1),
        RuleValueKind::Integer,
        BattleRuleScope::Battle,
        RuleValue::Integer(1),
    )
    .with_bounds(RuleValue::Integer(0), RuleValue::Integer(1))
}

fn mechanics_steps() -> Vec<ProgramStep> {
    vec![
        ProgramStep::Operation(RuleOperationTemplate::ModifyStateSlot {
            slot: id(1),
            update: StateSlotUpdateKind::Subtract,
            value: ValueExpr::Literal(RuleValue::Integer(1)),
        }),
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: id(2),
            effect: id(1),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }),
        ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
            selector: id(2),
            effect: id(1),
        }),
        ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
            selector: id(4),
            resource: RuleResourceKind::Character("enhanced-counter-charges".into()),
            update: ResourceUpdateKind::Gain,
            amount: ValueExpr::Literal(RuleValue::Scalar(Scalar::checked_from_integer(1).unwrap())),
            scales_with_regeneration: false,
            rounding: Rounding::Floor,
        }),
        ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
            code: 77,
            value: Some(ValueExpr::Literal(RuleValue::Integer(3))),
        }),
        ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
            selector: id(4),
            resource: RuleResourceKind::Team("shared.punchline".into()),
            update: ResourceUpdateKind::Gain,
            amount: ValueExpr::Literal(RuleValue::Scalar(Scalar::checked_from_integer(2).unwrap())),
            scales_with_regeneration: false,
            rounding: Rounding::Floor,
        }),
        ProgramStep::Operation(RuleOperationTemplate::QueueAction {
            actor_selector: id(4),
            target_selector: id(2),
            ability: id(3),
            priority: ReactionPriority::new(-10),
            forced_use: true,
            boundary: starclock_combat::catalog::action::ReactionBoundary::AfterAction,
            owner: RuleActionOwner::Actor,
            payment: Some(RuleActionPaymentPolicy::TeamResource(
                "shared.punchline".into(),
            )),
        }),
        ProgramStep::Operation(RuleOperationTemplate::Summon {
            owner_selector: id(4),
            unit_definition: id(3),
        }),
        ProgramStep::Operation(RuleOperationTemplate::CreateCountdown { code: 7 }),
    ]
}

fn add_mechanics_definitions(builder: &mut CombatCatalogBuilder) {
    builder.add_selector(
        SelectorDefinition::new(id(4)).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::Owner,
                RuleSelectorSide::Same,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Present,
                RuleSelectorReference::CurrentState,
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
    builder.add_selector(SelectorDefinition::new(id(5)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single).unwrap(),
    ));
    let effect = EffectRuntimeDefinition::new(
        EffectCategory::Buff,
        DispelCategory::DispellableBuff,
        1,
        Some(2),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .unwrap();
    builder.add_effect(EffectDefinition::new(id(1), vec![], vec![]).with_runtime(effect));
    let counter_damage = OrdinaryDamageDefinition::new(
        Scalar::checked_from_integer(25).unwrap(),
        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
    )
    .unwrap();
    let counter = AbilityActionDefinition::new(
        AbilityKind::ExtraAction,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(
            0,
            0,
            starclock_combat::Energy::ZERO,
            starclock_combat::Energy::ZERO,
        ),
    )
    .unwrap()
    .with_hits(vec![ActionHitDefinition::new(vec![
        HitOperationDefinition::Damage(counter_damage),
    ])])
    .unwrap();
    builder.add_ability(AbilityDefinition::new(id(3), id(2), id(1), vec![]).with_action(counter));
    for (ability, kind, selector) in [
        (4, AbilityKind::Countdown, 5),
        (5, AbilityKind::Memosprite, 1),
    ] {
        let action = AbilityActionDefinition::new(
            kind,
            1,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            ActionResourcePolicy::new(
                0,
                0,
                starclock_combat::Energy::ZERO,
                starclock_combat::Energy::ZERO,
            ),
        )
        .unwrap();
        builder.add_ability(
            AbilityDefinition::new(id(ability), id(2), id(selector), vec![]).with_action(action),
        );
    }
    builder.add_unit(UnitDefinition::new(id(3), vec![id(5)], vec![]));
    let linked = LinkedUnitDefinition::new(
        combatant(3, 5, 0x48, false, false, false),
        SourceDefinitionId::new(61).unwrap(),
        FormationIndex::new(1).unwrap(),
        LinkedEntityKind::Memosprite,
        PresenceState::Linked,
        Some(id(5)),
        ActionGauge::from_scaled(100_000_000).unwrap(),
        OwnerLinkPolicy::Depart,
        OwnerLinkPolicy::Depart,
        WaveLinkPolicy::Depart,
    )
    .unwrap();
    builder.add_linked_unit(LinkedUnitCatalogDefinition::new(id(3), linked).unwrap());
    builder.add_countdown(
        CountdownCatalogDefinition::new(
            7,
            CountdownDefinition::new(
                id(4),
                ActionGauge::from_scaled(100_000_000).unwrap(),
                Speed::from_scaled(1_000_000).unwrap(),
                OwnerLinkPolicy::Depart,
                OwnerLinkPolicy::Depart,
                WaveLinkPolicy::Depart,
            ),
        )
        .unwrap(),
    );
}

fn combatant(
    form: u32,
    ability: u32,
    digest: u8,
    with_modifier: bool,
    with_rule: bool,
    with_mechanics: bool,
) -> ResolvedCombatantSpec {
    let modifiers = with_modifier.then(|| id(1)).into_iter().collect();
    let mut abilities = vec![id(ability)];
    if with_mechanics {
        abilities.push(id(3));
    }
    let mut combatant = ResolvedCombatantSpec::new(
        id(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(if form == 1 { 100_000_000 } else { 1_000_000 }).unwrap(),
        ResolvedDefinitionBindings::new(
            abilities,
            with_rule.then(|| id(1)).into_iter().collect(),
            modifiers,
        )
        .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
    .with_base_attack_defense(
        StatValue::from_scaled(200_000_000).unwrap(),
        StatValue::from_scaled(100_000_000).unwrap(),
    );
    if with_modifier {
        let source = SourceDefinitionId::new(50).unwrap();
        combatant = combatant
            .with_sources(vec![RuleSource::new(
                source,
                SourceClass::Progression,
                vec![],
                [0x50; 32],
            )])
            .unwrap()
            .with_modifier_bindings(vec![ResolvedModifierBinding::new(id(1), source)])
            .unwrap();
    }
    combatant
}

fn battle(
    catalog: Arc<CombatCatalog>,
    with_modifier: bool,
    with_rule: bool,
    with_mechanics: bool,
) -> Battle {
    let spec = BattleSpec::new(
        "ability-program-rules-v1",
        BattleSpecDigest::new([0x44; 32]).unwrap(),
        id(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 1, 0x45, with_modifier, with_rule, with_mechanics),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(id(1)),
                combatant(2, 2, 0x46, false, false, false),
            ),
        ],
        if with_mechanics {
            TeamResourceSpec::new(0, 5)
                .unwrap()
                .with_keyed(vec![
                    KeyedTeamResourceSpec::new(
                        SourceDefinitionId::new(90).unwrap(),
                        1,
                        5,
                        TeamResourceWavePolicy::Persist,
                    )
                    .unwrap()
                    .with_stable_key("shared.punchline")
                    .unwrap(),
                ])
                .unwrap()
        } else {
            TeamResourceSpec::new(0, 5).unwrap()
        },
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog, spec, BattleSeed::new([0x47; 32])).unwrap()
}

fn start_and_use(
    battle: &mut Battle,
) -> Result<starclock_combat::Resolution, starclock_combat::CommandError> {
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    let pass = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::PassInterruptWindow { .. }))
        .unwrap()
        .clone();
    battle.apply(pass).unwrap();
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
    battle.apply(use_ability)
}

#[test]
fn hit_programs_use_authored_selector_order_and_exact_hit_shares() {
    let program =
        ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::Damage {
                selector: id(2),
                amount: ValueExpr::Literal(RuleValue::Scalar(
                    Scalar::checked_from_integer(200).unwrap(),
                )),
                class: DamageClass::Direct,
                element: CombatElement::Physical,
                can_crit: false,
            }),
        ]);
    let mut battle = battle(
        catalog(program, false, false, false, false),
        false,
        false,
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
    assert_eq!(damage, [50, 150]);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .nth(1)
            .unwrap()
            .current_hp()
            .get(),
        800
    );
}

#[test]
fn selected_build_modifier_changes_rule_ir_stat_query_inside_transaction() {
    let program =
        ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::Damage {
                selector: id(2),
                amount: ValueExpr::QueryStat {
                    subject: StatQuerySubject::Actor,
                    stat: StatKind::Atk,
                    purpose: FormulaPurpose::Stat,
                },
                class: DamageClass::Direct,
                element: CombatElement::Physical,
                can_crit: false,
            }),
        ]);
    let mut battle = battle(
        catalog(program, true, false, false, false),
        true,
        false,
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

    assert_eq!(damage, [100, 300]);
    assert!(resolution.fault().is_none());
    let actor = battle.view().units_by_id().next().unwrap();
    assert_eq!(actor.base_attack().scaled(), 200_000_000);
    assert_eq!(actor.base_defense().scaled(), 100_000_000);
    assert_eq!(actor.base_speed().scaled(), 100_000_000);
    let modifier = battle.view().modifier_instances_by_id().next().unwrap();
    assert_eq!(modifier.id().get(), 1);
    assert_eq!(modifier.definition().get(), 1);
    assert_eq!(modifier.owner(), modifier.subject());
    assert_eq!(modifier.source().get(), 50);
    assert_eq!(modifier.source_class(), SourceClass::Progression);
}

#[test]
fn expression_backed_dot_runtime_resolves_per_application_target() {
    let program = ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![id(1)], vec![])
        .with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                selector: id(2),
                effect: id(1),
                chance: RuleEffectChancePolicy::Guaranteed,
                base_chance: None,
                rng_purpose: None,
            }),
            ProgramStep::Operation(RuleOperationTemplate::DetonateDot {
                selector: id(2),
                fraction: ValueExpr::Literal(RuleValue::Scalar(Scalar::ONE)),
                required_tag: None,
            }),
        ]);
    let mut battle = battle(
        catalog(program, true, true, false, false),
        true,
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

    assert_eq!(damage, [400, 50, 50, 400]);
    let retained = battle.view().effects_by_id().next().unwrap();
    assert_eq!(retained.category(), EffectCategory::Dot);
    assert_eq!(retained.remaining(), Some(2));
    let attachments = battle
        .view()
        .modifier_instances_by_id()
        .filter(|modifier| modifier.source_effect() == Some(retained.id()))
        .collect::<Vec<_>>();
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].subject(), retained.target());
    assert_eq!(battle.view().rule_instances_by_id().count(), 2);
    assert!(resolution.fault().is_none());
}

#[test]
fn removing_an_effect_tears_down_its_modifier_attachments() {
    let program = ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![id(1)], vec![])
        .with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                selector: id(2),
                effect: id(1),
                chance: RuleEffectChancePolicy::Guaranteed,
                base_chance: None,
                rng_purpose: None,
            }),
            ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                selector: id(2),
                effect: id(1),
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
    assert_eq!(battle.view().modifier_instances_by_id().count(), 1);
    assert_eq!(battle.view().rule_instances_by_id().count(), 1);
    assert!(
        battle
            .view()
            .modifier_instances_by_id()
            .all(|modifier| modifier.source_effect().is_none())
    );
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

#[test]
fn recursively_emitting_rule_faults_at_the_dispatch_budget_and_rolls_back() {
    let program =
        ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::Damage {
                selector: id(2),
                amount: ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO)),
                class: DamageClass::Direct,
                element: CombatElement::Physical,
                can_crit: false,
            }),
        ]);
    let mut battle = battle(
        catalog(program, false, true, true, false),
        false,
        true,
        false,
    );
    let resolution = start_and_use(&mut battle).unwrap();

    assert!(resolution.fault().is_some());
    assert_eq!(
        battle.view().phase(),
        starclock_combat::BattlePhase::Faulted
    );
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .nth(1)
            .unwrap()
            .current_hp()
            .get(),
        1_000
    );
}

#[test]
fn representative_rule_emissions_use_authoritative_runtime_services() {
    let program = ProgramDefinition::new(id(1), vec![], vec![], vec![], vec![]);
    let mut battle = battle(
        catalog(program, false, true, false, true),
        false,
        true,
        true,
    );
    let resolution = start_and_use(&mut battle).unwrap();

    assert!(
        resolution.fault().is_none(),
        "unexpected mechanics fault: {:?}",
        resolution.fault()
    );
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Action(starclock_combat::ActionEventData::Queued {
            ability,
            origin: starclock_combat::ActionOrigin::Forced,
            ..
        }) if ability.get() == 3
    )));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::RuleSignal(starclock_combat::RuleSignalEventData {
            code: 77,
            value: Some(RuleValue::Integer(3)),
            ..
        })
    )));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Unit(starclock_combat::UnitEventData::Summoned {
            kind: LinkedEntityKind::Memosprite,
            ..
        })
    )));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Unit(starclock_combat::UnitEventData::CountdownCreated {
            ability,
            ..
        }) if ability.get() == 4
    )));
    let effect_events = resolution
        .events()
        .iter()
        .filter(|event| matches!(event.kind(), BattleEventKind::Effect(_)))
        .count();
    assert_eq!(effect_events, 2);
    assert_eq!(battle.view().effects_by_id().count(), 0);
    assert_eq!(battle.view().units_by_id().count(), 3);
    assert_eq!(battle.view().links().count(), 2);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .next()
            .unwrap()
            .character_resource("enhanced-counter-charges"),
        Some((
            Scalar::checked_from_integer(1).unwrap(),
            Scalar::checked_from_integer(2).unwrap()
        ))
    );
    assert_eq!(
        battle.view().team(TeamSide::Player).keyed_resource(id(90)),
        Some((3, 5))
    );
    assert_eq!(
        battle
            .view()
            .rule_instances_by_id()
            .next()
            .unwrap()
            .slots()
            .next()
            .unwrap()
            .1,
        &RuleValue::Integer(0)
    );
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .find(|unit| unit.form().get() == 2)
            .unwrap()
            .current_hp()
            .get(),
        975
    );
}

#[test]
fn unsupported_program_emission_rolls_back_the_whole_command() {
    let program =
        ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::TrueDamage {
                selector: id(2),
                amount: ValueExpr::Literal(RuleValue::Scalar(
                    Scalar::checked_from_integer(200).unwrap(),
                )),
            }),
        ]);
    let mut battle = battle(
        catalog(program, false, false, false, false),
        false,
        false,
        false,
    );
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    let pass = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::PassInterruptWindow { .. }))
        .unwrap()
        .clone();
    battle.apply(pass).unwrap();
    let before_revision = battle.view().committed_revision();
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(
            |command| matches!(command, Command::UseAbility { ability, .. } if ability.get() == 1),
        )
        .unwrap()
        .clone();
    let resolution = battle.apply(command).unwrap();
    assert!(resolution.fault().is_some());
    assert_eq!(
        battle.view().phase(),
        starclock_combat::BattlePhase::Faulted
    );
    assert_eq!(battle.view().committed_revision(), before_revision + 1);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .nth(1)
            .unwrap()
            .current_hp()
            .get(),
        1_000
    );
}
