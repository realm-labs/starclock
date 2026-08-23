//! Line-limit exception: this test-only damage lifecycle corpus shares one deterministic battle fixture.
use crate::combat_decision::{advance_boundary_if_offered, settle_ready_boundaries};
use std::sync::Arc;

use starclock_combat::{
    ActionValue, ActionValueClockSpec, AssemblyDigest, Battle, BattleClockExpiry, BattleClockSpec,
    BattleEventKind, BattlePhase, BattleSeed, BattleSpec, CombatantSpecDigest, Command,
    CommandErrorKind, ConcedePolicy, EncounterWaveId, Energy, FormationIndex, Hp,
    LethalRescueHpPolicy, LifeState, ParticipantSource, ParticipantSpec, PlayerLethalRescueSpec,
    PresenceState, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    ResolvedModifierBinding, Scalar, Speed, StatValue, TeamResourceSpec, TeamSide,
    ToughnessLayerKind, ToughnessLayerSpec, ToughnessReductionDefinition, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HealingDefinition, HitOperationDefinition, HpConsumptionDefinition,
            OrdinaryDamageDefinition, OrdinaryDamageMultipliers, ScalingDamageDefinition,
            ShieldDefinition, TargetInvalidationPolicy, TargetPattern, TargetRelation,
            UnitTargetSelector, WeaknessApplicationDefinition,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
        encounter::{EncounterWaveDefinition, WaveCarry, WaveSlotDefinition, WaveTransitionPolicy},
    },
    formula::{
        model::CombatElement,
        shield::ShieldAbsorptionPolicy,
        toughness::{
            BreakDamageDefinition, EnemyRank, SuperBreakDefinition, ToughnessReductionContext,
        },
    },
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition, ModifierFilter,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{RuleSource, RuleValue, SourceClass, ValueExpr},
};

#[path = "damage_lifecycle/source_resistance.rs"]
mod source_resistance;

fn definition<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn runtime<I: TryFrom<u64>>(raw: u64) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn all_one_damage(amount: i64) -> OrdinaryDamageDefinition {
    OrdinaryDamageDefinition::new(
        Scalar::checked_from_integer(amount).unwrap(),
        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
    )
    .unwrap()
}

fn action(
    kind: AbilityKind,
    operations: Vec<Vec<HitOperationDefinition>>,
    invalidation: TargetInvalidationPolicy,
) -> AbilityActionDefinition {
    AbilityActionDefinition::new(
        kind,
        u16::try_from(operations.len()).unwrap(),
        invalidation,
        ActionResourcePolicy::new(
            0,
            0,
            starclock_combat::Energy::ZERO,
            starclock_combat::Energy::ZERO,
        ),
    )
    .unwrap()
    .with_hits(
        operations
            .into_iter()
            .map(|operations| {
                ActionHitDefinition::new(operations).with_profile(
                    starclock_combat::catalog::action::HitTargetGroup::Selected,
                    Ratio::ONE,
                    Ratio::ONE,
                    starclock_combat::catalog::action::HitCritPolicy::Never,
                )
            })
            .collect(),
    )
    .unwrap()
}

fn catalog(waves: u16) -> Arc<CombatCatalog> {
    catalog_with_policy(waves, WaveTransitionPolicy::AfterAction)
}

fn catalog_with_policy(waves: u16, transition: WaveTransitionPolicy) -> Arc<CombatCatalog> {
    catalog_with_spawn(waves, transition, None)
}

fn catalog_with_spawn(
    waves: u16,
    transition: WaveTransitionPolicy,
    spawn: Option<starclock_combat::catalog::encounter::SpawnProgramDefinition>,
) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new([0x91; 32]);
    builder.add_modifier_group(ModifierStackingGroup {
        id: definition(1),
        aggregation: ModifierAggregation::Sum,
        comparator: None,
    });
    for (id, value, snapshot) in [
        (1, 100_000, SnapshotPolicy::OnActionStart),
        (2, 200_000, SnapshotPolicy::OnPhaseStart),
        (3, 300_000, SnapshotPolicy::OnHitStart),
        (4, 400_000, SnapshotPolicy::OnApplication),
        (5, 500_000, SnapshotPolicy::RecomputeOnStackChange),
    ] {
        builder.add_modifier(ModifierDefinition {
            id: definition(id),
            stat: StatKind::Atk,
            stage: FormulaStage::DamageBoost,
            purpose: FormulaPurpose::OrdinaryDamage,
            value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(value))),
            stacking_group: definition(1),
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::DamageBoost,
            snapshot,
            source_stack_slot: None,
            filters: Box::new([]),
        });
    }
    for (id, purpose) in [(6, FormulaPurpose::Break), (7, FormulaPurpose::SuperBreak)] {
        builder.add_modifier(ModifierDefinition {
            id: definition(id),
            stat: StatKind::Hp,
            stage: FormulaStage::Mitigation,
            purpose,
            value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(500_000))),
            stacking_group: definition(1),
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::Mitigation,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: vec![ModifierFilter::DamageTag(
                match purpose {
                    FormulaPurpose::Break => "break",
                    FormulaPurpose::SuperBreak => "super_break",
                    _ => unreachable!(),
                }
                .into(),
            )]
            .into_boxed_slice(),
        });
    }
    builder.add_modifier(ModifierDefinition {
        id: definition(8),
        stat: StatKind::Atk,
        stage: FormulaStage::Resistance,
        purpose: FormulaPurpose::OrdinaryDamage,
        value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(250_000))),
        stacking_group: definition(1),
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Resistance,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: vec![ModifierFilter::FormulaSubject(
            starclock_combat::modifier::model::FormulaSubject::Source,
        )]
        .into_boxed_slice(),
    });
    builder.add_modifier(ModifierDefinition {
        id: definition(9),
        stat: StatKind::Atk,
        stage: FormulaStage::DamageOverride,
        purpose: FormulaPurpose::OrdinaryDamage,
        value: ValueExpr::Literal(RuleValue::Scalar(Scalar::checked_from_integer(1).unwrap())),
        stacking_group: definition(1),
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::DamageOverride,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: vec![ModifierFilter::FormulaSubject(
            starclock_combat::modifier::model::FormulaSubject::Source,
        )]
        .into_boxed_slice(),
    });
    for (raw, relation) in [
        (1, TargetRelation::SelfUnit),
        (2, TargetRelation::Opposing),
        (3, TargetRelation::Opposing),
    ] {
        builder.add_selector(
            SelectorDefinition::new(definition(raw)).with_unit_targets(
                UnitTargetSelector::new(relation, TargetPattern::Single).unwrap(),
            ),
        );
        builder.add_program(ProgramDefinition::new(
            definition(raw),
            vec![],
            vec![definition(raw)],
            vec![],
            vec![],
        ));
    }
    for raw in [90, 91, 92] {
        builder.add_program(ProgramDefinition::new(
            definition(raw),
            vec![],
            vec![],
            vec![],
            vec![],
        ));
    }
    let healing = HealingDefinition::new(
        Scalar::checked_from_integer(600).unwrap(),
        Ratio::from_scaled(200_000),
        Ratio::ZERO,
        Ratio::ZERO,
    )
    .unwrap();
    builder.add_ability(
        AbilityDefinition::new(definition(1), definition(1), definition(1), vec![]).with_action(
            action(
                AbilityKind::Basic,
                vec![vec![
                    HitOperationDefinition::Damage(all_one_damage(600)),
                    HitOperationDefinition::Heal(healing),
                ]],
                TargetInvalidationPolicy::KeepIfPresent,
            ),
        ),
    );
    let break_formula = BreakDamageDefinition {
        attacker_level_multiplier: Scalar::ONE,
        ability_multiplier: Ratio::ONE,
        break_effect: Ratio::ZERO,
        break_damage_increase: Ratio::ZERO,
        defense_multiplier: Ratio::ONE,
        resistance_multiplier: Ratio::ONE,
        vulnerability_multiplier: Ratio::ONE,
        mitigation_multiplier: Ratio::ONE,
        unbroken_multiplier: Ratio::from_scaled(900_000),
    };
    builder.add_ability(
        AbilityDefinition::new(definition(5), definition(2), definition(2), vec![]).with_action(
            action(
                AbilityKind::Basic,
                vec![vec![
                    HitOperationDefinition::AddWeakness(
                        WeaknessApplicationDefinition::timed(CombatElement::Fire, 2).unwrap(),
                    ),
                    HitOperationDefinition::ReduceToughness(ToughnessReductionDefinition {
                        element: CombatElement::Fire,
                        ignores_weakness: false,
                        reduction: ToughnessReductionContext {
                            base: starclock_combat::RawToughness::new(90).unwrap(),
                            additive: starclock_combat::RawToughness::new(0).unwrap(),
                            reduction_increase: Ratio::ZERO,
                            weakness_break_efficiency: Ratio::ZERO,
                            weakness_break_efficiency_cap: Ratio::from_scaled(3_000_000),
                            toughness_vulnerability: Ratio::ZERO,
                            ability_multiplier: Ratio::ONE,
                        },
                        break_damage: break_formula,
                        break_effect_chance: starclock_combat::Probability::ONE,
                    }),
                    HitOperationDefinition::SuperBreak(SuperBreakDefinition {
                        element: CombatElement::Fire,
                        attacker_level_multiplier: Scalar::ONE,
                        ability_multiplier: Ratio::from_scaled(500_000),
                        break_effect: Ratio::ZERO,
                        break_damage_increase: Ratio::ZERO,
                        super_break_increase: Ratio::ZERO,
                        defense_multiplier: Ratio::ONE,
                        resistance_multiplier: Ratio::ONE,
                        vulnerability_multiplier: Ratio::ONE,
                        mitigation_multiplier: Ratio::ONE,
                        broken_multiplier: Ratio::ONE,
                    }),
                ]],
                TargetInvalidationPolicy::KeepIfPresent,
            ),
        ),
    );
    builder.add_ability(
        AbilityDefinition::new(definition(6), definition(2), definition(2), vec![]).with_action(
            action(
                AbilityKind::Basic,
                vec![vec![HitOperationDefinition::ScalingDamage(
                    ScalingDamageDefinition::new(
                        starclock_combat::modifier::model::StatKind::Atk,
                        Ratio::from_scaled(500_000),
                        starclock_combat::formula::model::DamageClass::Direct,
                        CombatElement::Fire,
                    )
                    .unwrap(),
                )]],
                TargetInvalidationPolicy::KeepIfPresent,
            ),
        ),
    );
    let concurrent = ShieldAbsorptionPolicy::ConcurrentLargest;
    builder.add_ability(
        AbilityDefinition::new(definition(4), definition(1), definition(1), vec![]).with_action(
            action(
                AbilityKind::Basic,
                vec![vec![
                    HitOperationDefinition::Shield(
                        ShieldDefinition::new(
                            Scalar::checked_from_integer(300).unwrap(),
                            Ratio::ZERO,
                            concurrent,
                        )
                        .unwrap(),
                    ),
                    HitOperationDefinition::Shield(
                        ShieldDefinition::new(
                            Scalar::checked_from_integer(500).unwrap(),
                            Ratio::ZERO,
                            concurrent,
                        )
                        .unwrap(),
                    ),
                    HitOperationDefinition::ConsumeHp(HpConsumptionDefinition::new(
                        Hp::new(400).unwrap(),
                        Hp::new(1).unwrap(),
                    )),
                ]],
                TargetInvalidationPolicy::KeepIfPresent,
            ),
        ),
    );
    let mut first_hit = vec![HitOperationDefinition::Damage(all_one_damage(1_000))];
    if transition == WaveTransitionPolicy::Explicit {
        first_hit.push(HitOperationDefinition::RequestWaveTransition);
    }
    builder.add_ability(
        AbilityDefinition::new(definition(2), definition(2), definition(2), vec![]).with_action(
            action(
                AbilityKind::Basic,
                vec![
                    first_hit,
                    vec![HitOperationDefinition::Damage(all_one_damage(1_000))],
                ],
                TargetInvalidationPolicy::CancelRemainingForTarget,
            ),
        ),
    );
    builder.add_ability(
        AbilityDefinition::new(definition(3), definition(3), definition(3), vec![]).with_action(
            action(
                AbilityKind::Basic,
                vec![vec![HitOperationDefinition::Damage(all_one_damage(1_000))]],
                TargetInvalidationPolicy::CancelRemainingForTarget,
            ),
        ),
    );
    builder.add_unit(UnitDefinition::new(
        definition(1),
        vec![
            definition(1),
            definition(2),
            definition(4),
            definition(5),
            definition(6),
        ],
        vec![],
    ));
    builder.add_unit(UnitDefinition::new(
        definition(2),
        vec![definition(3)],
        vec![],
    ));
    builder.add_enemy(EnemyDefinition::new(
        definition(1),
        definition(2),
        vec![definition(3)],
    ));
    let wave_rows = (1..=waves)
        .map(|number| {
            let entry_program = (number > 1).then(|| definition(91));
            let exit_program = (number < waves).then(|| definition(90));
            let carry = if number > 1 {
                WaveCarry {
                    hp: starclock_combat::catalog::encounter::WaveCarryPolicy::ExplicitProgram(
                        definition(92),
                    ),
                    ..WaveCarry::CARRY_ALL
                }
            } else {
                WaveCarry::CARRY_ALL
            };
            let wave = EncounterWaveDefinition::new(
                definition::<EncounterWaveId>(u32::from(number)),
                number,
                entry_program,
                exit_program,
                carry,
                vec![
                    WaveSlotDefinition::new(
                        1,
                        FormationIndex::new(4).unwrap(),
                        definition(1),
                        None,
                        None,
                        spawn.is_none(),
                    )
                    .unwrap(),
                ],
            )
            .unwrap();
            if number == 1 {
                spawn
                    .clone()
                    .map_or(Some(wave.clone()), |program| {
                        wave.with_spawn_program(program)
                    })
                    .unwrap()
            } else {
                wave
            }
        })
        .collect::<Vec<_>>();
    builder.add_encounter(
        EncounterDefinition::new(definition(1), vec![definition(1)], vec![])
            .with_authored_waves(transition, wave_rows)
            .unwrap(),
    );
    builder.build().unwrap()
}

fn spawn_battle(quota: u16) -> Battle {
    use starclock_combat::catalog::encounter::{
        SpawnEndPolicy, SpawnOrdering, SpawnProgramDefinition, SpawnRefillTiming,
    };

    let program = SpawnProgramDefinition::new(
        SpawnRefillTiming::AfterDefeatSettlement,
        SpawnOrdering::AuthoredSlot,
        1,
        vec![FormationIndex::new(4).unwrap()],
        SpawnEndPolicy::DefeatQuota(quota),
    )
    .unwrap();
    let participants = vec![
        ParticipantSpec::new(
            TeamSide::Player,
            FormationIndex::new(0).unwrap(),
            ParticipantSource::Player,
            combatant(1, vec![1, 2, 4], 1_000, 100_000_000, 0x31),
        ),
        ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(4).unwrap(),
            ParticipantSource::EncounterEnemy(definition(1)),
            combatant(2, vec![3], 600, 50_000_000, 0x41),
        ),
    ];
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x52; 32]).unwrap(),
        definition(1),
        participants,
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(
        catalog_with_spawn(1, WaveTransitionPolicy::AfterAction, Some(program)),
        spec,
        BattleSeed::new([0x62; 32]),
    )
    .unwrap()
}

fn combatant(
    form: u32,
    abilities: Vec<u32>,
    hp: i64,
    speed: i64,
    digest: u8,
) -> ResolvedCombatantSpec {
    combatant_with_modifiers(form, abilities, vec![], hp, speed, digest)
}

fn combatant_with_modifiers(
    form: u32,
    abilities: Vec<u32>,
    modifiers: Vec<u32>,
    hp: i64,
    speed: i64,
    digest: u8,
) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        definition(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(hp).unwrap(),
        Speed::from_scaled(speed).unwrap(),
        ResolvedDefinitionBindings::new(
            abilities.into_iter().map(definition).collect(),
            vec![],
            modifiers.into_iter().map(definition).collect(),
        )
        .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
}

fn combatant_with_formula_modifier() -> ResolvedCombatantSpec {
    let source = definition(90);
    ResolvedCombatantSpec::new(
        definition(1),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(1_000_000_000).unwrap(),
        ResolvedDefinitionBindings::new(
            vec![definition(6)],
            vec![],
            vec![
                definition(1),
                definition(2),
                definition(3),
                definition(4),
                definition(5),
            ],
        )
        .unwrap(),
        CombatantSpecDigest::new([0x79; 32]).unwrap(),
    )
    .unwrap()
    .with_base_attack_defense(
        StatValue::from_scaled(2_000_000_000).unwrap(),
        StatValue::from_scaled(0).unwrap(),
    )
    .with_sources(vec![RuleSource::new(
        source,
        SourceClass::Progression,
        vec![],
        [0x77; 32],
    )])
    .unwrap()
    .with_modifier_bindings(
        [1, 2, 3, 4, 5]
            .map(|id| ResolvedModifierBinding::new(definition(id), source))
            .to_vec(),
    )
    .unwrap()
}

fn combatant_with_damage_override() -> ResolvedCombatantSpec {
    let source = definition(93);
    combatant_with_modifiers(1, vec![2], vec![9], 1_000, 1_000_000_000, 0x7b)
        .with_sources(vec![RuleSource::new(
            source,
            SourceClass::Progression,
            vec![],
            [0x7b; 32],
        )])
        .unwrap()
        .with_modifier_bindings(vec![ResolvedModifierBinding::new(definition(9), source)])
        .unwrap()
}

fn battle(waves: u16, player_speed: i64, enemy_speed: i64) -> Battle {
    battle_with_policy(
        waves,
        player_speed,
        enemy_speed,
        WaveTransitionPolicy::AfterAction,
    )
}

fn battle_with_policy(
    waves: u16,
    player_speed: i64,
    enemy_speed: i64,
    transition: WaveTransitionPolicy,
) -> Battle {
    let mut participants = vec![ParticipantSpec::new(
        TeamSide::Player,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::Player,
        combatant(1, vec![1, 2, 4], 1_000, player_speed, 0x31),
    )];
    for wave in 1..=waves {
        participants.push(
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(
                    2,
                    vec![3],
                    600,
                    enemy_speed,
                    u8::try_from(0x40 + wave).unwrap(),
                ),
            )
            .with_wave(wave)
            .unwrap(),
        );
    }
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x51; 32]).unwrap(),
        definition(1),
        participants,
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(
        catalog_with_policy(waves, transition),
        spec,
        BattleSeed::new([0x61; 32]),
    )
    .unwrap()
}

fn battle_with_enemy_defeat_energy() -> Battle {
    let player = combatant(1, vec![1, 2, 4], 1_000, 200_000_000, 0x32)
        .with_energy(Energy::ZERO, Energy::from_scaled(100_000_000).unwrap())
        .unwrap();
    let participants = vec![
        ParticipantSpec::new(
            TeamSide::Player,
            FormationIndex::new(0).unwrap(),
            ParticipantSource::Player,
            player,
        ),
        ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(4).unwrap(),
            ParticipantSource::EncounterEnemy(definition(1)),
            combatant(2, vec![3], 600, 50_000_000, 0x42),
        ),
    ];
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x53; 32]).unwrap(),
        definition(1),
        participants,
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap()
    .with_enemy_defeat_energy(Energy::from_scaled(5_000_000).unwrap())
    .unwrap();
    Battle::create(catalog(1), spec, BattleSeed::new([0x63; 32])).unwrap()
}

fn battle_with_player_lethal_rescue(action_value_loss: i64) -> Battle {
    let participants = vec![
        ParticipantSpec::new(
            TeamSide::Player,
            FormationIndex::new(0).unwrap(),
            ParticipantSource::Player,
            combatant(1, vec![1], 1_000, 50_000_000, 0x33),
        ),
        ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(4).unwrap(),
            ParticipantSource::EncounterEnemy(definition(1)),
            combatant(2, vec![3], 1_000, 200_000_000, 0x43),
        ),
    ];
    let clock = BattleClockSpec::ActionValue(
        ActionValueClockSpec::new(
            ActionValue::from_scaled(100_000_000).unwrap(),
            BattleClockExpiry::Lose,
        )
        .unwrap(),
    );
    let rescue = PlayerLethalRescueSpec::new(
        LethalRescueHpPolicy::MaximumHp,
        Some(ActionValue::from_scaled(action_value_loss).unwrap()),
    )
    .unwrap();
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x54; 32]).unwrap(),
        definition(1),
        participants,
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap()
    .with_clock(clock)
    .with_player_lethal_rescue(rescue)
    .unwrap();
    Battle::create(catalog(1), spec, BattleSeed::new([0x64; 32])).unwrap()
}

fn toughness_battle() -> Battle {
    damage_mitigation::toughness_battle_with_mitigation(false)
}

#[path = "damage_lifecycle/damage_mitigation.rs"]
mod damage_mitigation;

fn break_recovery_battle() -> Battle {
    break_recovery_battle_with_enemy_hp(10_000)
}

fn break_recovery_battle_with_enemy_hp(enemy_hp: i64) -> Battle {
    let player = combatant(1, vec![4, 5], 10_000, 200_000_000, 0x75);
    let layer =
        ToughnessLayerSpec::ordinary(1, starclock_combat::RawToughness::new(50).unwrap()).unwrap();
    let enemy = combatant(2, vec![3], enemy_hp, 190_000_000, 0x76)
        .with_toughness(EnemyRank::Normal, vec![], vec![layer])
        .unwrap();
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x77; 32]).unwrap(),
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
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                enemy,
            ),
        ],
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog(1), spec, BattleSeed::new([0x78; 32])).unwrap()
}

fn start_and_pass(battle: &mut Battle) {
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    advance_boundary_if_offered(battle);
}

fn advance_boundary(battle: &mut Battle) {
    advance_boundary_if_offered(battle);
}

fn use_ability(battle: &mut Battle, ability: u32) -> starclock_combat::Resolution {
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(command, Command::UseAbility { ability: offered, .. } if offered.get() == ability)
        })
        .unwrap_or_else(|| {
            panic!(
                "ability {ability} was not offered: {:?}",
                battle.decision().unwrap().legal_commands()
            )
        })
        .clone();
    battle.apply(command).unwrap()
}

#[test]
fn scaling_hit_damage_resolves_the_actors_live_stat() {
    let player = combatant(1, vec![6], 1_000, 1_000_000_000, 0x31).with_base_attack_defense(
        StatValue::from_scaled(2_000_000_000).unwrap(),
        StatValue::from_scaled(0).unwrap(),
    );
    let enemy = combatant(2, vec![3], 2_000, 1_000_000, 0x41);
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x51; 32]).unwrap(),
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
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                enemy,
            ),
        ],
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle = Battle::create(catalog(1), spec, BattleSeed::new([0x61; 32])).unwrap();
    start_and_pass(&mut battle);
    let resolution = use_ability(&mut battle, 6);
    let damage = resolution
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(data),
            _ => None,
        })
        .expect("scaling operation must emit damage");
    assert_eq!(damage.raw.scaled(), 1_000_000_000);
    assert_eq!(damage.calculated.get(), 1_000);
}

#[test]
fn application_action_phase_hit_and_stack_snapshots_change_damage() {
    let player = combatant_with_formula_modifier();
    let enemy = combatant(2, vec![3], 2_000, 1_000_000, 0x41);
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x51; 32]).unwrap(),
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
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                enemy,
            ),
        ],
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle = Battle::create(catalog(1), spec, BattleSeed::new([0x7A; 32])).unwrap();
    start_and_pass(&mut battle);
    let resolution = use_ability(&mut battle, 6);
    let damage = resolution
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(data.calculated),
            _ => None,
        })
        .unwrap_or_else(|| panic!("damage event missing: {:?}", resolution.events()));
    assert_eq!(damage.get(), 2_500);
}

#[test]
fn damage_override_sets_each_hit_without_collapsing_the_action() {
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x7c; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant_with_damage_override(),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(2, vec![3], 1_000, 1_000_000, 0x7d),
            ),
        ],
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle = Battle::create(catalog(1), spec, BattleSeed::new([0x7e; 32])).unwrap();
    start_and_pass(&mut battle);
    let resolution = use_ability(&mut battle, 2);
    let damage = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(data.applied.get()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(damage, [1, 1]);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .nth(1)
            .unwrap()
            .current_hp()
            .get(),
        998
    );
}

#[test]
fn weakness_precedes_reduction_and_super_break_uses_effective_layer_sample() {
    let mut battle = toughness_battle();
    start_and_pass(&mut battle);
    let first = use_ability(&mut battle, 5);
    let first_reduction = first
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Toughness(starclock_combat::ToughnessEventData::Reduced {
                layer_key,
                attempted,
                effective,
                ..
            }) => Some((*layer_key, attempted.get(), effective.get())),
            _ => None,
        })
        .unwrap();
    assert_eq!(first_reduction, (Some(1), 90, 50));
    let initial_break = first
        .events()
        .iter()
        .find(|event| {
            matches!(
                event.kind(),
                BattleEventKind::BreakDamage(data)
                    if data.kind == starclock_combat::BreakDamageKind::Initial
            )
        })
        .unwrap();
    assert_eq!(
        initial_break.cause().source_definition(),
        Some(definition(99))
    );
    assert_eq!(
        battle
            .view()
            .break_effects_by_id()
            .map(|effect| (
                effect.element(),
                effect.remaining_turns(),
                effect.source_definition()
            ))
            .collect::<Vec<_>>(),
        vec![(CombatElement::Fire, 2, definition(99))]
    );
    let first_kinds = first
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Toughness(starclock_combat::ToughnessEventData::WeaknessAdded {
                ..
            }) => Some("weakness"),
            BattleEventKind::Toughness(starclock_combat::ToughnessEventData::Reduced {
                ..
            }) => Some("reduction"),
            BattleEventKind::BreakDamage(data)
                if data.kind == starclock_combat::BreakDamageKind::SuperBreak =>
            {
                Some("super-break")
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(first_kinds, vec!["weakness", "reduction", "super-break"]);
    let enemy = battle.view().units_by_id().nth(1).unwrap();
    assert!(enemy.weakness_broken());
    assert_eq!(
        enemy
            .toughness_layers()
            .map(|layer| layer.current().get())
            .collect::<Vec<_>>(),
        vec![0, 40]
    );

    advance_boundary(&mut battle);
    let second = use_ability(&mut battle, 5);
    let second_reduction = second
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Toughness(starclock_combat::ToughnessEventData::Reduced {
                layer_key,
                effective,
                ..
            }) => Some((*layer_key, effective.get())),
            _ => None,
        })
        .unwrap();
    assert_eq!(second_reduction, (Some(2), 40));
    assert!(second.events().iter().any(|event| matches!(event.kind(),
        BattleEventKind::BreakDamage(data) if data.kind == starclock_combat::BreakDamageKind::SuperBreak)));

    advance_boundary(&mut battle);
    let third = use_ability(&mut battle, 5);
    assert!(third.events().iter().any(|event| matches!(event.kind(),
        BattleEventKind::Toughness(starclock_combat::ToughnessEventData::Reduced { layer_key: None, effective, .. }) if effective.get() == 0)));
    assert!(third.events().iter().any(|event| matches!(event.kind(),
        BattleEventKind::Toughness(starclock_combat::ToughnessEventData::SuperBreakSkipped { effective_reduction, .. }) if effective_reduction.get() == 0)));
}

#[test]
fn fire_break_dot_ticks_and_recovery_turn_restores_the_layer() {
    let mut battle = break_recovery_battle();
    start_and_pass(&mut battle);
    let resolution = use_ability(&mut battle, 5);
    let mut events = resolution.events().to_vec();
    events.extend(settle_ready_boundaries(&mut battle));
    assert!(events.iter().any(|event| matches!(event.kind(),
        BattleEventKind::BreakDamage(data) if data.kind == starclock_combat::BreakDamageKind::Effect)));
    assert!(events.iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Toughness(starclock_combat::ToughnessEventData::BaseEffectTicked {
            remaining_turns: 1,
            ..
        })
    )));
    assert!(events.iter().any(|event| matches!(event.kind(),
        BattleEventKind::Toughness(starclock_combat::ToughnessEventData::Recovered { before, after, exited_global_broken: true, .. })
            if before.get() == 0 && after.get() == 50)));
    let enemy = battle.view().units_by_id().nth(1).unwrap();
    assert!(!enemy.weakness_broken());
    assert_eq!(enemy.toughness_layers().next().unwrap().current().get(), 50);
    assert_eq!(
        battle
            .view()
            .break_effects_by_id()
            .next()
            .unwrap()
            .remaining_turns(),
        1
    );

    advance_boundary(&mut battle);
    let enemy_action = use_ability(&mut battle, 3);
    let mut expiry = enemy_action.events().to_vec();
    expiry.extend(settle_ready_boundaries(&mut battle));
    advance_boundary(&mut battle);
    let player_action = use_ability(&mut battle, 4);
    expiry.extend_from_slice(player_action.events());
    expiry.extend(settle_ready_boundaries(&mut battle));
    assert!(expiry.iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Toughness(starclock_combat::ToughnessEventData::WeaknessRemoved {
            element: CombatElement::Fire,
            ..
        })
    )));
    assert!(
        !battle
            .view()
            .units_by_id()
            .nth(1)
            .unwrap()
            .weaknesses()
            .contains(&CombatElement::Fire)
    );
}

#[test]
fn lethal_turn_start_break_effect_settles_before_selecting_another_actor() {
    let mut battle = break_recovery_battle_with_enemy_hp(6);
    start_and_pass(&mut battle);
    let resolution = use_ability(&mut battle, 5);
    let mut events = resolution.events().to_vec();
    events.extend(settle_ready_boundaries(&mut battle));
    assert!(
        events.iter().any(|event| matches!(
            event.kind(),
            BattleEventKind::BreakDamage(data)
                if data.kind == starclock_combat::BreakDamageKind::Effect && data.hp_after.get() == 0
        )),
        "{:#?}",
        events
    );
    assert_eq!(battle.view().phase(), BattlePhase::Won);
    assert!(matches!(
        events.last().unwrap().kind(),
        BattleEventKind::Battle(starclock_combat::BattleEventData::Won)
    ));
    assert!(battle.decision().is_none());
}

#[test]
fn credited_enemy_defeat_grants_the_explicit_energy_reward() {
    let mut battle = battle_with_enemy_defeat_energy();
    start_and_pass(&mut battle);
    let resolution = use_ability(&mut battle, 2);

    assert!(
        resolution.events().iter().any(|event| matches!(
            event.kind(),
            BattleEventKind::Resource(starclock_combat::ResourceEventData::Energy {
                before,
                after,
                overflow,
                ..
            }) if *before == Energy::ZERO
                && after.scaled() == 5_000_000
                && *overflow == Energy::ZERO
        )),
        "{:#?}",
        resolution.events()
    );
    assert_eq!(
        battle.view().units_by_id().next().unwrap().current_energy(),
        Energy::from_scaled(5_000_000).unwrap()
    );
}

#[test]
fn lethal_player_damage_restores_hp_and_deducts_the_action_value_clock() {
    let mut battle = battle_with_player_lethal_rescue(25_000_000);
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    let resolution = use_ability(&mut battle, 3);

    let rescue = resolution.events().iter().position(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Unit(starclock_combat::UnitEventData::LethalRescued {
                hp,
                ..
            }) if hp.get() == 1_000
        )
    });
    let deduction = resolution.events().iter().position(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Clock(starclock_combat::BattleClockEventData::Advanced {
                delta_scaled: 25_000_000,
                before_scaled: 50_000_000,
                after_scaled: 25_000_000,
            })
        )
    });
    assert!(
        rescue.is_some_and(|rescue| deduction.is_some_and(|deduction| rescue < deduction)),
        "{:#?}",
        resolution.events()
    );
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .next()
            .unwrap()
            .current_hp()
            .get(),
        1_000
    );
    assert_eq!(
        battle
            .view()
            .clock()
            .unwrap()
            .remaining_action_value_scaled(),
        Some(25_000_000)
    );
    assert_eq!(battle.view().phase(), BattlePhase::ReadyToAdvance);
}

#[test]
fn lethal_rescue_clock_exhaustion_loses_at_the_action_boundary() {
    let mut battle = battle_with_player_lethal_rescue(50_000_000);
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    let resolution = use_ability(&mut battle, 3);

    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Unit(starclock_combat::UnitEventData::LethalRescued { .. })
    )));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Clock(starclock_combat::BattleClockEventData::Expired {
            expiry: BattleClockExpiry::Lose,
        })
    )));
    assert!(!resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Unit(starclock_combat::UnitEventData::Downed { .. })
    )));
    assert_eq!(battle.view().phase(), BattlePhase::Lost);
}

#[test]
fn damage_and_healing_emit_calculated_and_effective_hp_facts() {
    let mut battle = battle(1, 200_000_000, 50_000_000);
    start_and_pass(&mut battle);
    let resolution = use_ability(&mut battle, 1);
    assert_eq!(
        resolution.state_hash().bytes(),
        [
            140, 10, 199, 175, 149, 86, 40, 43, 54, 12, 138, 41, 78, 159, 55, 104, 17, 1, 114, 215,
            39, 34, 128, 197, 138, 136, 14, 185, 227, 31, 100, 210,
        ]
    );
    let damage = resolution
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some((event.cause(), *data)),
            _ => None,
        })
        .unwrap();
    assert_eq!(damage.1.calculated.get(), 600);
    assert_eq!(damage.1.applied.get(), 600);
    assert_eq!(damage.1.hp_before.get(), 1_000);
    assert_eq!(damage.1.hp_after.get(), 400);
    assert_eq!(
        damage.0.applier(),
        Some(runtime::<starclock_combat::UnitId>(1))
    );
    let healing = resolution
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Heal(data) => Some(*data),
            _ => None,
        })
        .unwrap();
    assert_eq!(healing.calculated.get(), 720);
    assert_eq!(healing.effective.get(), 600);
    assert_eq!(healing.overheal.get(), 120);
    assert_eq!(healing.hp_after.get(), 1_000);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .next()
            .unwrap()
            .current_hp()
            .get(),
        1_000
    );
}

#[test]
fn hp_consumption_and_concurrent_shields_flow_through_authoritative_state() {
    let mut battle = battle(1, 100_000_000, 100_000_000);
    start_and_pass(&mut battle);
    let applied = use_ability(&mut battle, 4);
    let consumed = applied
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::HpConsumption(data) => Some(*data),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        (consumed.effective.get(), consumed.overflow.get()),
        (400, 0)
    );
    assert_eq!(
        battle
            .view()
            .shields_by_id()
            .map(|shield| shield.remaining().get())
            .collect::<Vec<_>>(),
        vec![300, 500]
    );

    advance_boundary(&mut battle);
    let damaged = use_ability(&mut battle, 3);
    let shield_events = damaged
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Shield(starclock_combat::ShieldEventData::Absorbed {
                shield,
                before,
                after,
                ..
            }) => Some((shield.get(), before.get(), after.get())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(shield_events, vec![(1, 300, 0), (2, 500, 0)]);
    let damage = damaged
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(*data),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        (damage.calculated.get(), damage.absorbed.get()),
        (1_000, 500)
    );
    assert_eq!((damage.applied.get(), damage.hp_after.get()), (500, 100));
    assert_eq!(
        battle
            .view()
            .shields_by_id()
            .map(|shield| shield.remaining().get())
            .collect::<Vec<_>>(),
        Vec::<i64>::new()
    );
}

#[test]
fn single_wave_defeat_settles_to_victory_and_terminal_rejection_is_immutable() {
    let mut battle = battle(1, 200_000_000, 50_000_000);
    start_and_pass(&mut battle);
    let resolution = use_ability(&mut battle, 2);
    assert_eq!(
        resolution.state_hash().bytes(),
        [
            98, 77, 1, 126, 147, 37, 37, 47, 90, 2, 221, 164, 193, 3, 230, 233, 63, 250, 191, 94,
            232, 130, 78, 19, 208, 220, 233, 15, 214, 147, 254, 43,
        ]
    );
    assert_eq!(resolution.phase(), BattlePhase::Won);
    assert!(resolution.next_decision().is_none());
    let enemy = battle.view().units_by_id().nth(1).unwrap();
    assert_eq!(enemy.current_hp().get(), 0);
    assert_eq!(enemy.life(), LifeState::Defeated);
    assert!(matches!(
        resolution.events().last().unwrap().kind(),
        BattleEventKind::Battle(starclock_combat::BattleEventData::Won)
    ));
    let before = battle.state_hash();
    let draws = battle.view().rng_draw_count();
    let error = battle
        .apply(Command::StartBattle {
            decision: runtime(999),
        })
        .unwrap_err();
    assert_eq!(error.kind(), CommandErrorKind::TerminalBattle);
    assert_eq!(battle.state_hash(), before);
    assert_eq!(battle.view().rng_draw_count(), draws);
}

#[test]
fn continuous_spawn_refills_in_slot_order_until_the_authored_quota() {
    let mut battle = spawn_battle(2);
    start_and_pass(&mut battle);
    let first = use_ability(&mut battle, 2);
    assert_eq!(first.phase(), BattlePhase::ReadyToAdvance);
    assert_eq!(battle.view().encounter().spawn_defeats(), 1);
    let enemy = battle.view().units_by_id().nth(1).unwrap();
    assert_eq!(enemy.life(), LifeState::Alive);
    assert_eq!(enemy.current_hp().get(), 600);
    assert_eq!(enemy.spawn_sequence().get(), 3);
    assert!(first.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Unit(starclock_combat::UnitEventData::Refilled {
            wave_defeats: 1,
            ..
        })
    )));

    start_and_pass_current_turn(&mut battle);
    let second = use_ability(&mut battle, 2);
    assert_eq!(second.phase(), BattlePhase::Won);
    assert_eq!(battle.view().encounter().spawn_defeats(), 2);
    assert!(!second.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Unit(starclock_combat::UnitEventData::Refilled { .. })
    )));
}

#[test]
fn after_action_wave_transition_does_not_let_later_hits_reach_reserve_units() {
    let mut battle = battle(2, 200_000_000, 50_000_000);
    start_and_pass(&mut battle);
    let first = use_ability(&mut battle, 2);
    assert_eq!(
        first.state_hash().bytes(),
        [
            126, 251, 219, 38, 193, 97, 135, 172, 21, 156, 116, 38, 123, 235, 71, 147, 246, 59,
            248, 250, 84, 133, 119, 251, 156, 250, 255, 22, 34, 68, 227, 251,
        ]
    );
    assert_eq!(first.phase(), BattlePhase::ReadyToAdvance);
    assert_eq!(battle.view().encounter().number(), 2);
    assert_eq!(battle.view().encounter().total_waves(), 2);
    let units = battle.view().units_by_id().collect::<Vec<_>>();
    assert_eq!(units[1].life(), LifeState::Defeated);
    assert_eq!(units[1].presence(), PresenceState::Departed);
    assert_eq!(units[2].current_hp().get(), 600);
    assert_eq!(units[2].life(), LifeState::Alive);
    assert_eq!(units[2].presence(), PresenceState::Present);
    let hit_end_positions = first
        .events()
        .iter()
        .enumerate()
        .filter_map(|(index, event)| {
            matches!(
                event.kind(),
                BattleEventKind::Hit(starclock_combat::HitEventData::Ended { .. })
            )
            .then_some(index)
        })
        .collect::<Vec<_>>();
    let wave_started = first
        .events()
        .iter()
        .position(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Wave(starclock_combat::WaveEventData::Started { number: 2, .. })
            )
        })
        .unwrap();
    assert_eq!(hit_end_positions.len(), 2);
    assert!(wave_started > *hit_end_positions.last().unwrap());

    start_and_pass_current_turn(&mut battle);
    let second = use_ability(&mut battle, 2);
    assert_eq!(
        second.state_hash().bytes(),
        [
            255, 97, 244, 252, 60, 197, 209, 8, 201, 247, 40, 222, 157, 13, 127, 251, 7, 101, 121,
            243, 66, 136, 80, 83, 238, 39, 143, 88, 84, 139, 213, 89,
        ]
    );
    assert_eq!(second.phase(), BattlePhase::Won);
}

#[test]
fn nondefault_wave_boundaries_emit_at_the_authored_lifecycle_point() {
    for policy in [
        WaveTransitionPolicy::AfterHit,
        WaveTransitionPolicy::AfterPhase,
        WaveTransitionPolicy::Explicit,
    ] {
        let mut battle = battle_with_policy(2, 200_000_000, 50_000_000, policy);
        start_and_pass(&mut battle);
        let resolution = use_ability(&mut battle, 2);
        assert_eq!(battle.view().encounter().number(), 2);
        let position = |predicate: &dyn Fn(&BattleEventKind) -> bool| {
            resolution
                .events()
                .iter()
                .position(|event| predicate(event.kind()))
                .unwrap()
        };
        let wave = position(&|kind| {
            matches!(
                kind,
                BattleEventKind::Wave(starclock_combat::WaveEventData::Started { number: 2, .. })
            )
        });
        let first_hit_end = position(&|kind| {
            matches!(kind,
            BattleEventKind::Hit(starclock_combat::HitEventData::Ended { hit, .. }) if hit.get() == 1)
        });
        let phase_end = position(&|kind| {
            matches!(
                kind,
                BattleEventKind::Phase(starclock_combat::PhaseEventData::Ended { .. })
            )
        });
        match policy {
            WaveTransitionPolicy::AfterHit => assert!(first_hit_end < wave && wave < phase_end),
            WaveTransitionPolicy::AfterPhase => assert!(phase_end < wave),
            WaveTransitionPolicy::Explicit => assert!(wave < first_hit_end),
            WaveTransitionPolicy::AfterAction => unreachable!(),
        }
    }
}

fn start_and_pass_current_turn(battle: &mut Battle) {
    advance_boundary_if_offered(battle);
}

#[test]
fn defeating_the_last_player_settles_loss() {
    let mut battle = battle(1, 50_000_000, 200_000_000);
    start_and_pass(&mut battle);
    let resolution = use_ability(&mut battle, 3);
    assert_eq!(
        resolution.state_hash().bytes(),
        [
            174, 126, 81, 169, 12, 105, 97, 219, 118, 155, 242, 29, 0, 99, 224, 64, 116, 253, 218,
            176, 245, 85, 197, 142, 178, 33, 198, 42, 147, 56, 209, 187,
        ]
    );
    assert_eq!(resolution.phase(), BattlePhase::Lost);
    assert!(resolution.next_decision().is_none());
    let player = battle.view().units_by_id().next().unwrap();
    assert_eq!(player.current_hp().get(), 0);
    assert_eq!(player.life(), LifeState::Defeated);
    assert!(matches!(
        resolution.events().last().unwrap().kind(),
        BattleEventKind::Battle(starclock_combat::BattleEventData::Lost)
    ));
}
