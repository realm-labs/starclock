use std::sync::Arc;

use starclock_combat::{
    Battle, BattleEventKind, BattlePhase, BattleSeed, BattleSpec, BattleSpecDigest,
    CombatantSpecDigest, Command, CommandErrorKind, ConcedePolicy, FormationIndex, Hp, LifeState,
    ParticipantSource, ParticipantSpec, PresenceState, Ratio, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, Scalar, Speed, TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HealingDefinition, HitOperationDefinition, HpConsumptionDefinition,
            OrdinaryDamageDefinition, OrdinaryDamageMultipliers, ShieldDefinition,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
    },
    formula::shield::ShieldAbsorptionPolicy,
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
        vec![definition(1), definition(2), definition(4)],
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
fn damage_and_healing_emit_calculated_and_effective_hp_facts() {
    let mut battle = battle(1, 200_000_000, 50_000_000);
    start_and_pass(&mut battle);
    let resolution = use_ability(&mut battle, 1);
    assert_eq!(
        resolution.state_hash().bytes(),
        [
            0x1d, 0xad, 0xeb, 0xce, 0xac, 0xf8, 0x4c, 0x69, 0xf6, 0x7f, 0x9c, 0xea, 0xc7, 0x88,
            0x6b, 0x4c, 0xf5, 0x46, 0x79, 0x3d, 0x2d, 0x4c, 0xe4, 0xb1, 0x57, 0x50, 0xdf, 0xfa,
            0x61, 0x4d, 0xc5, 0x32,
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
            0x67, 0x83, 0x95, 0x67, 0x04, 0x17, 0x01, 0x2b, 0x13, 0xe6, 0xb7, 0x1d, 0x72, 0xd4,
            0xc8, 0x3b, 0x0e, 0xcd, 0x63, 0x81, 0xe2, 0x8d, 0x32, 0x50, 0x63, 0x8b, 0xe6, 0xa0,
            0xd6, 0x5b, 0x82, 0x62,
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
            0xdb, 0xdb, 0x25, 0xea, 0x64, 0x03, 0xa7, 0x08, 0x0e, 0x89, 0x05, 0xb4, 0x46, 0x09,
            0xb4, 0x84, 0x54, 0xed, 0x27, 0xe8, 0xdf, 0x31, 0x7f, 0x1f, 0x15, 0x84, 0xcf, 0xb1,
            0xeb, 0x36, 0xdb, 0xd8,
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
            0xf0, 0xc4, 0xb9, 0xe6, 0xbd, 0x40, 0x1d, 0x61, 0x00, 0xea, 0xa9, 0x03, 0x3a, 0x60,
            0x6e, 0x3a, 0x85, 0x50, 0xbd, 0x9a, 0x93, 0xf3, 0x1a, 0x00, 0x69, 0xe0, 0xb9, 0x87,
            0xd9, 0xb2, 0xea, 0x24,
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
            0x23, 0xdf, 0xaf, 0x1a, 0xd9, 0xc3, 0x7a, 0x77, 0xe3, 0x8d, 0xdf, 0x7a, 0x62, 0xfc,
            0x26, 0x70, 0x6c, 0x6a, 0x57, 0xc1, 0x0f, 0xd8, 0x68, 0x0a, 0xcd, 0x45, 0x03, 0xbe,
            0x77, 0x4a, 0xde, 0x90,
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
