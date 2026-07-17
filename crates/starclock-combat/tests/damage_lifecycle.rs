use std::sync::Arc;

use starclock_combat::{
    Battle, BattleEventKind, BattlePhase, BattleSeed, BattleSpec, BattleSpecDigest,
    CombatantSpecDigest, Command, CommandErrorKind, ConcedePolicy, FormationIndex, Hp, LifeState,
    ParticipantSource, ParticipantSpec, PresenceState, Ratio, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, Scalar, Speed, TeamResourceSpec, TeamSide, ToughnessLayerKind,
    ToughnessLayerSpec, ToughnessReductionDefinition, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HealingDefinition, HitOperationDefinition, HpConsumptionDefinition,
            OrdinaryDamageDefinition, OrdinaryDamageMultipliers, ShieldDefinition,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
            WeaknessApplicationDefinition,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
    },
    formula::{
        model::CombatElement,
        shield::ShieldAbsorptionPolicy,
        toughness::{
            BreakDamageDefinition, EnemyRank, SuperBreakDefinition, ToughnessReductionContext,
        },
    },
};

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
            .map(ActionHitDefinition::new)
            .collect(),
    )
    .unwrap()
}

fn catalog(waves: u16) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new("damage-lifecycle-v1", [0x91; 32]);
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
    builder.add_ability(
        AbilityDefinition::new(definition(2), definition(2), definition(2), vec![]).with_action(
            action(
                AbilityKind::Basic,
                vec![
                    vec![HitOperationDefinition::Damage(all_one_damage(1_000))],
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
        vec![definition(1), definition(2), definition(4), definition(5)],
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
    let wave_rows = (0..waves).map(|_| vec![definition(1)]).collect::<Vec<_>>();
    builder.add_encounter(
        EncounterDefinition::new(definition(1), vec![definition(1)], vec![])
            .with_waves(wave_rows)
            .unwrap(),
    );
    builder.build().unwrap()
}

fn combatant(
    form: u32,
    abilities: Vec<u32>,
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
            vec![],
        )
        .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
}

fn battle(waves: u16, player_speed: i64, enemy_speed: i64) -> Battle {
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
        "damage-lifecycle-rules-v1",
        BattleSpecDigest::new([0x51; 32]).unwrap(),
        definition(1),
        participants,
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog(waves), spec, BattleSeed::new([0x61; 32])).unwrap()
}

fn toughness_battle() -> Battle {
    let player = combatant(1, vec![5], 1_000, 1_000_000_000, 0x71);
    let ordinary =
        ToughnessLayerSpec::ordinary(1, starclock_combat::RawToughness::new(50).unwrap())
            .unwrap()
            .with_break_credit(starclock_combat::BreakCreditPolicy::LayerProvider(
                definition(99),
            ));
    let exo = ToughnessLayerSpec::ordinary(2, starclock_combat::RawToughness::new(40).unwrap())
        .unwrap()
        .with_kind(ToughnessLayerKind::ExoToughness)
        .with_break_behavior(true, true, true, false);
    let enemy = combatant(2, vec![3], 10_000, 1_000_000, 0x72)
        .with_toughness(EnemyRank::Normal, vec![], vec![ordinary, exo])
        .unwrap();
    let spec = BattleSpec::new(
        "toughness-layer-rules-v1",
        BattleSpecDigest::new([0x73; 32]).unwrap(),
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
    Battle::create(catalog(1), spec, BattleSeed::new([0x74; 32])).unwrap()
}

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
        "break-recovery-rules-v1",
        BattleSpecDigest::new([0x77; 32]).unwrap(),
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
    let pass = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::PassInterruptWindow { .. }))
        .unwrap()
        .clone();
    battle.apply(pass).unwrap();
}

fn pass_interrupt(battle: &mut Battle) {
    let pass = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::PassInterruptWindow { .. }))
        .unwrap()
        .clone();
    battle.apply(pass).unwrap();
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

    pass_interrupt(&mut battle);
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

    pass_interrupt(&mut battle);
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
    assert!(resolution.events().iter().any(|event| matches!(event.kind(),
        BattleEventKind::BreakDamage(data) if data.kind == starclock_combat::BreakDamageKind::Effect)));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Toughness(starclock_combat::ToughnessEventData::BaseEffectTicked {
            remaining_turns: 1,
            ..
        })
    )));
    assert!(resolution.events().iter().any(|event| matches!(event.kind(),
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

    pass_interrupt(&mut battle);
    let _enemy_action = use_ability(&mut battle, 3);
    pass_interrupt(&mut battle);
    let expiry = use_ability(&mut battle, 4);
    assert!(expiry.events().iter().any(|event| matches!(
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
    assert!(
        resolution.events().iter().any(|event| matches!(
            event.kind(),
            BattleEventKind::BreakDamage(data)
                if data.kind == starclock_combat::BreakDamageKind::Effect && data.hp_after.get() == 0
        )),
        "{:#?}",
        resolution.events()
    );
    assert_eq!(resolution.phase(), BattlePhase::Won);
    assert!(matches!(
        resolution.events().last().unwrap().kind(),
        BattleEventKind::Battle(starclock_combat::BattleEventData::Won)
    ));
    assert!(resolution.next_decision().is_none());
}

#[test]
fn damage_and_healing_emit_calculated_and_effective_hp_facts() {
    let mut battle = battle(1, 200_000_000, 50_000_000);
    start_and_pass(&mut battle);
    let resolution = use_ability(&mut battle, 1);
    assert_eq!(
        resolution.state_hash().bytes(),
        [
            0xdf, 0x99, 0xec, 0x9d, 0x5b, 0xb1, 0x5b, 0x64, 0x5c, 0x6a, 0x8d, 0xc3, 0x5a, 0xc7,
            0x01, 0x48, 0x3a, 0xd0, 0x27, 0x99, 0x30, 0x99, 0x04, 0xe1, 0xd1, 0x1d, 0x79, 0xa2,
            0xfd, 0x50, 0xa6, 0xa1,
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

    pass_interrupt(&mut battle);
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
        vec![0, 0]
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
            0x04, 0x2a, 0xaf, 0x46, 0xd6, 0x96, 0x12, 0x07, 0x42, 0xed, 0x52, 0xc7, 0x61, 0xbc,
            0xa8, 0x1a, 0xa5, 0x82, 0xd0, 0x66, 0x3a, 0xb3, 0x0e, 0xc6, 0xc4, 0xb3, 0x96, 0xbe,
            0xd8, 0xa1, 0x25, 0xba,
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
fn after_action_wave_transition_does_not_let_later_hits_reach_reserve_units() {
    let mut battle = battle(2, 200_000_000, 50_000_000);
    start_and_pass(&mut battle);
    let first = use_ability(&mut battle, 2);
    assert_eq!(
        first.state_hash().bytes(),
        [
            0x19, 0x08, 0x22, 0x4d, 0x16, 0x92, 0x73, 0xa0, 0x66, 0x0b, 0xe1, 0xef, 0x52, 0x89,
            0x65, 0x0f, 0xbc, 0x68, 0xa1, 0x40, 0x4c, 0xde, 0x8a, 0xd6, 0x67, 0x86, 0xcf, 0x5f,
            0x84, 0x79, 0x38, 0x8a,
        ]
    );
    assert_eq!(first.phase(), BattlePhase::AwaitingCommand);
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
            0xd3, 0x83, 0x56, 0xcf, 0x5a, 0x98, 0x6a, 0x80, 0x3e, 0x06, 0xdb, 0xa7, 0xa3, 0xbc,
            0x74, 0xf8, 0x39, 0x85, 0x4e, 0x87, 0x46, 0x37, 0x67, 0xbe, 0x95, 0x7a, 0x40, 0x0b,
            0x6c, 0xda, 0x66, 0x06,
        ]
    );
    assert_eq!(second.phase(), BattlePhase::Won);
}

fn start_and_pass_current_turn(battle: &mut Battle) {
    let pass = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::PassInterruptWindow { .. }))
        .unwrap()
        .clone();
    battle.apply(pass).unwrap();
}

#[test]
fn defeating_the_last_player_settles_loss() {
    let mut battle = battle(1, 50_000_000, 200_000_000);
    start_and_pass(&mut battle);
    let resolution = use_ability(&mut battle, 3);
    assert_eq!(
        resolution.state_hash().bytes(),
        [
            0x51, 0x3f, 0x22, 0xe1, 0x42, 0xe4, 0xfb, 0x55, 0x08, 0x32, 0x4a, 0x4d, 0xcd, 0xc8,
            0x7c, 0x12, 0xfe, 0x2c, 0xd6, 0xc4, 0x43, 0x7d, 0xb5, 0x84, 0x36, 0xd9, 0x8b, 0x8b,
            0x3a, 0xd4, 0xb4, 0x1f,
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
