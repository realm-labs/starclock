use std::sync::Arc;

use starclock_combat::{
    ActionGauge, ActionOrigin, Battle, BattleEventKind, BattleSeed, BattleSpec, BattleSpecDigest,
    CombatantSpecDigest, ConcedePolicy, CountdownCatalogDefinition, CountdownDefinition,
    FormationIndex, Hp, LifeState, LinkedEntity, LinkedEntityKind, LinkedOwnerScaling,
    LinkedStatScaling, LinkedUnitDefinition, OwnerLinkPolicy, ParticipantSource, ParticipantSpec,
    PresenceState, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings, ReviveDefinition,
    ReviveGaugePolicy, Scalar, Speed, StatValue, TeamResourceSpec, TeamSide, TransformEndPolicy,
    TransformationDefinition, UnitEventData, UnitLevel, WaveEventData, WaveLinkPolicy,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            ActionHitDefinition, ActionResourcePolicy, HitOperationDefinition,
            OrdinaryDamageDefinition, OrdinaryDamageMultipliers, TargetInvalidationPolicy,
            TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
    },
};

fn definition<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn scalar(raw: i64) -> Scalar {
    Scalar::from_scaled(raw * 1_000_000)
}

fn all_one_damage(raw: i64) -> OrdinaryDamageDefinition {
    OrdinaryDamageDefinition::new(
        scalar(raw),
        OrdinaryDamageMultipliers::new([starclock_combat::Ratio::ONE; 9]).unwrap(),
    )
    .unwrap()
}

fn bindings(abilities: Vec<u32>) -> ResolvedDefinitionBindings {
    ResolvedDefinitionBindings::new(
        abilities.into_iter().map(definition).collect(),
        vec![],
        vec![],
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
    ResolvedCombatantSpec::new(
        definition(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(hp).unwrap(),
        Speed::from_scaled(speed * 1_000_000).unwrap(),
        bindings(abilities),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
    .with_base_attack_defense(
        StatValue::from_scaled(100_000_000).unwrap(),
        StatValue::from_scaled(50_000_000).unwrap(),
    )
}

fn linked(
    owner_defeat: OwnerLinkPolicy,
    wave: WaveLinkPolicy,
    initial_gauge: i64,
) -> LinkedUnitDefinition {
    LinkedUnitDefinition::new(
        combatant(2, vec![2], 700, 300, 0x72),
        definition(2),
        FormationIndex::new(8).unwrap(),
        LinkedEntityKind::Memosprite,
        PresenceState::Linked,
        Some(definition(2)),
        ActionGauge::from_scaled(initial_gauge * 1_000_000).unwrap(),
        owner_defeat,
        OwnerLinkPolicy::Depart,
        wave,
    )
    .unwrap()
}

fn owner_scaled_linked() -> LinkedUnitDefinition {
    linked(OwnerLinkPolicy::Persist, WaveLinkPolicy::Persist, 0).with_owner_scaling(
        LinkedOwnerScaling::new(
            LinkedStatScaling::new(Ratio::from_scaled(500_000), scalar(100)),
            LinkedStatScaling::new(Ratio::from_scaled(1_500_000), scalar(10)),
            LinkedStatScaling::new(Ratio::from_scaled(2_000_000), scalar(5)),
            LinkedStatScaling::new(Ratio::from_scaled(250_000), scalar(20)),
        ),
    )
}

fn action(
    kind: AbilityKind,
    invalidation: TargetInvalidationPolicy,
    operations: Vec<HitOperationDefinition>,
) -> AbilityActionDefinition {
    AbilityActionDefinition::new(
        kind,
        1,
        invalidation,
        ActionResourcePolicy::new(
            0,
            0,
            starclock_combat::Energy::ZERO,
            starclock_combat::Energy::ZERO,
        ),
    )
    .unwrap()
    .with_hits(vec![ActionHitDefinition::new(operations)])
    .unwrap()
}

fn ability(
    id: u32,
    selector: u32,
    kind: AbilityKind,
    invalidation: TargetInvalidationPolicy,
    operations: Vec<HitOperationDefinition>,
) -> AbilityDefinition {
    AbilityDefinition::new(
        definition(id),
        definition(selector),
        definition(selector),
        vec![],
    )
    .with_action(action(kind, invalidation, operations))
}

fn fixture_catalog() -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new("linked-lifecycle-v1", [0x41; 32]);
    builder.add_selector(SelectorDefinition::new(definition(1)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single).unwrap(),
    ));
    builder.add_selector(SelectorDefinition::new(definition(2)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_program(ProgramDefinition::new(
        definition(1),
        vec![],
        vec![definition(1)],
        vec![],
        vec![],
    ));
    builder.add_program(
        ProgramDefinition::new(definition(9), vec![], vec![], vec![], vec![]).with_steps(vec![
            starclock_combat::rule::model::ProgramStep::Operation(
                starclock_combat::rule::model::RuleOperationTemplate::CreateCountdown { code: 11 },
            ),
        ]),
    );
    builder.add_program(ProgramDefinition::new(
        definition(2),
        vec![],
        vec![definition(2)],
        vec![],
        vec![],
    ));

    builder.add_ability(ability(
        1,
        1,
        AbilityKind::Basic,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        vec![HitOperationDefinition::SummonLinked(linked(
            OwnerLinkPolicy::Persist,
            WaveLinkPolicy::Persist,
            0,
        ))],
    ));
    builder.add_ability(ability(
        2,
        1,
        AbilityKind::Memosprite,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        vec![],
    ));
    builder.add_ability(
        ability(
            9,
            1,
            AbilityKind::Basic,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            vec![HitOperationDefinition::Transform(
                TransformationDefinition::new(
                    definition(3),
                    vec![definition(3)],
                    None,
                    TransformEndPolicy::End,
                    TransformEndPolicy::End,
                )
                .unwrap(),
            )],
        )
        .with_programs(vec![
            AbilityProgramBinding::new(1, AbilityProgramTiming::AfterHits, definition(9)).unwrap(),
        ]),
    );
    builder.add_ability(ability(
        10,
        1,
        AbilityKind::Countdown,
        TargetInvalidationPolicy::KeepIfPresent,
        vec![],
    ));
    builder.add_countdown(
        CountdownCatalogDefinition::new(
            11,
            CountdownDefinition::new(
                definition(10),
                ActionGauge::from_scaled(0).unwrap(),
                Speed::from_scaled(400_000_000).unwrap(),
                OwnerLinkPolicy::Depart,
                OwnerLinkPolicy::Depart,
                WaveLinkPolicy::Persist,
            )
            .with_end_transformation(),
        )
        .unwrap(),
    );
    builder.add_ability(ability(
        11,
        1,
        AbilityKind::Basic,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        vec![HitOperationDefinition::SummonLinked(owner_scaled_linked())],
    ));
    builder.add_ability(ability(
        3,
        1,
        AbilityKind::Basic,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        vec![],
    ));
    builder.add_ability(ability(
        4,
        1,
        AbilityKind::Countdown,
        TargetInvalidationPolicy::KeepIfPresent,
        vec![HitOperationDefinition::EndTransformation],
    ));
    let countdown = CountdownDefinition::new(
        definition(4),
        ActionGauge::from_scaled(0).unwrap(),
        Speed::from_scaled(400_000_000).unwrap(),
        OwnerLinkPolicy::Depart,
        OwnerLinkPolicy::Depart,
        WaveLinkPolicy::Persist,
    );
    builder.add_ability(ability(
        5,
        1,
        AbilityKind::Basic,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        vec![HitOperationDefinition::Transform(
            TransformationDefinition::new(
                definition(3),
                vec![definition(3)],
                Some(countdown),
                TransformEndPolicy::End,
                TransformEndPolicy::End,
            )
            .unwrap(),
        )],
    ));
    builder.add_ability(ability(
        6,
        1,
        AbilityKind::Basic,
        TargetInvalidationPolicy::KeepIfPresent,
        vec![
            HitOperationDefinition::SummonLinked(linked(
                OwnerLinkPolicy::Depart,
                WaveLinkPolicy::Persist,
                10_000,
            )),
            HitOperationDefinition::Damage(all_one_damage(2_000)),
            HitOperationDefinition::Revive(
                ReviveDefinition::new(
                    Hp::new(500).unwrap(),
                    PresenceState::Present,
                    ReviveGaugePolicy::Reset,
                )
                .unwrap(),
            ),
        ],
    ));
    builder.add_ability(ability(
        7,
        2,
        AbilityKind::Basic,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        vec![
            HitOperationDefinition::SummonLinked(linked(
                OwnerLinkPolicy::Persist,
                WaveLinkPolicy::Depart,
                0,
            )),
            HitOperationDefinition::Damage(all_one_damage(2_000)),
        ],
    ));
    builder.add_ability(ability(
        8,
        1,
        AbilityKind::Basic,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        vec![],
    ));

    builder.add_unit(UnitDefinition::new(
        definition(1),
        vec![
            definition(1),
            definition(5),
            definition(6),
            definition(7),
            definition(9),
            definition(11),
        ],
        vec![],
    ));
    builder.add_unit(UnitDefinition::new(
        definition(2),
        vec![definition(2)],
        vec![],
    ));
    builder.add_unit(UnitDefinition::new(
        definition(3),
        vec![definition(3), definition(4), definition(10)],
        vec![],
    ));
    builder.add_unit(UnitDefinition::new(
        definition(4),
        vec![definition(8)],
        vec![],
    ));
    for enemy in 1..=2 {
        builder.add_enemy(EnemyDefinition::new(
            definition(enemy),
            definition(4),
            vec![definition(8)],
        ));
    }
    builder.add_encounter(
        EncounterDefinition::new(definition(1), vec![definition(1), definition(2)], vec![])
            .with_waves(vec![vec![definition(1)], vec![definition(2)]])
            .unwrap(),
    );
    builder.build().unwrap()
}

fn fixture_battle() -> Battle {
    let spec = BattleSpec::new(
        "linked-lifecycle-rules-v1",
        BattleSpecDigest::new([0x51; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, vec![1, 5, 6, 7, 9, 11], 1_000, 200, 0x61),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(4, vec![8], 1_000, 50, 0x62),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(definition(2)),
                combatant(4, vec![8], 1_000, 50, 0x63),
            )
            .with_wave(2)
            .unwrap(),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(fixture_catalog(), spec, BattleSeed::new([0x71; 32])).unwrap()
}

fn open_normal_action(battle: &mut Battle) {
    let start = battle.decision().unwrap().legal_commands()[0].clone();
    battle.apply(start).unwrap();
    let pass = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                starclock_combat::Command::PassInterruptWindow { .. }
            )
        })
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
        .find(|command| match command {
            starclock_combat::Command::UseAbility {
                ability: offered, ..
            } => *offered == definition(ability),
            _ => false,
        })
        .unwrap()
        .clone();
    battle.apply(command).unwrap()
}

#[test]
fn memosprite_has_distinct_owner_unit_actor_and_automatic_turn() {
    let mut battle = fixture_battle();
    open_normal_action(&mut battle);
    let resolution = use_ability(&mut battle, 1);
    let units = battle.view().units_by_id().collect::<Vec<_>>();
    let memo = units
        .iter()
        .copied()
        .find(|unit| unit.form() == definition(2))
        .unwrap();
    assert_eq!(memo.source(), ParticipantSource::Linked(definition(2)));
    let link = battle
        .view()
        .links()
        .find(|link| link.entity() == LinkedEntity::Unit(memo.id()))
        .unwrap();
    assert_eq!(link.owner(), units[0].id());
    assert_eq!(link.kind(), LinkedEntityKind::Memosprite);
    let actor = battle
        .view()
        .timeline_actors()
        .find(|actor| actor.unit() == Some(memo.id()))
        .unwrap();
    assert_eq!(actor.owner(), units[0].id());
    assert_eq!(actor.linked_kind(), Some(LinkedEntityKind::Memosprite));
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Action(starclock_combat::ActionEventData::Declared { actor, origin: ActionOrigin::MemospriteAction, .. }) if *actor == memo.id()
    ) && event.cause().owner() == Some(units[0].id())));
}

#[test]
fn linked_combatant_stats_are_resolved_from_its_current_owner() {
    let mut battle = fixture_battle();
    open_normal_action(&mut battle);
    let resolution = use_ability(&mut battle, 11);
    assert!(resolution.fault().is_none());
    let memo = battle
        .view()
        .units_by_id()
        .find(|unit| unit.form() == definition(2))
        .unwrap();

    assert_eq!(memo.maximum_hp(), Hp::new(600).unwrap());
    assert_eq!(memo.current_hp(), Hp::new(600).unwrap());
    assert_eq!(
        memo.base_attack(),
        StatValue::from_scaled(160_000_000).unwrap()
    );
    assert_eq!(
        memo.base_defense(),
        StatValue::from_scaled(105_000_000).unwrap()
    );
    assert_eq!(memo.base_speed(), Speed::from_scaled(70_000_000).unwrap());
}

#[test]
fn countdown_ends_transformation_once_and_restores_original_abilities() {
    let mut battle = fixture_battle();
    open_normal_action(&mut battle);
    let resolution = use_ability(&mut battle, 5);
    let owner = battle.view().units_by_id().next().unwrap();
    assert_eq!(owner.form(), definition(1));
    assert_eq!(
        owner.abilities(),
        [
            definition(1),
            definition(5),
            definition(6),
            definition(7),
            definition(9),
            definition(11),
        ]
    );
    assert!(!owner.is_transformed());
    let countdowns = battle
        .view()
        .timeline_actors()
        .filter(|actor| actor.linked_kind() == Some(LinkedEntityKind::Countdown))
        .collect::<Vec<_>>();
    assert_eq!(countdowns.len(), 1);
    assert!(!countdowns[0].is_active());
    let unit_events = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Unit(unit) => Some(unit),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(matches!(unit_events[0], UnitEventData::Transformed { .. }));
    assert_eq!(
        unit_events
            .iter()
            .filter(|event| matches!(event, UnitEventData::TransformationEnded { .. }))
            .count(),
        1
    );
}

#[test]
fn separately_created_countdown_ends_the_active_transformation() {
    let mut battle = fixture_battle();
    open_normal_action(&mut battle);
    let resolution = use_ability(&mut battle, 9);
    let owner = battle.view().units_by_id().next().unwrap();

    assert_eq!(owner.form(), definition(1));
    assert_eq!(
        owner.abilities(),
        [
            definition(1),
            definition(5),
            definition(6),
            definition(7),
            definition(9),
            definition(11),
        ]
    );
    assert!(!owner.is_transformed());
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Unit(UnitEventData::CountdownCreated { ability, .. })
            if *ability == definition(10)
    )));
    assert_eq!(
        resolution
            .events()
            .iter()
            .filter(|event| matches!(
                event.kind(),
                BattleEventKind::Unit(UnitEventData::TransformationEnded { .. })
            ))
            .count(),
        1
    );
    let countdown = battle
        .view()
        .timeline_actors()
        .find(|actor| actor.linked_kind() == Some(LinkedEntityKind::Countdown))
        .unwrap();
    assert!(!countdown.is_active());
}

#[test]
fn owner_defeat_settles_link_before_explicit_revival() {
    let mut battle = fixture_battle();
    open_normal_action(&mut battle);
    let resolution = use_ability(&mut battle, 6);
    let owner = battle.view().units_by_id().next().unwrap();
    assert_eq!(owner.life(), LifeState::Alive);
    assert_eq!(owner.current_hp(), Hp::new(500).unwrap());
    let linked = battle
        .view()
        .units_by_id()
        .find(|unit| unit.form() == definition(2))
        .unwrap();
    assert_eq!(linked.presence(), PresenceState::Departed);
    let order = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Unit(UnitEventData::Downed { .. }) => Some(0),
            BattleEventKind::Unit(UnitEventData::Defeated { .. }) => Some(1),
            BattleEventKind::Unit(UnitEventData::LinkSettled { .. }) => Some(2),
            BattleEventKind::Unit(UnitEventData::Revived { .. }) => Some(3),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(order, [0, 1, 2, 3]);
}

#[test]
fn wave_departure_policy_runs_after_wave_end_before_wave_start() {
    let mut battle = fixture_battle();
    open_normal_action(&mut battle);
    let resolution = use_ability(&mut battle, 7);
    assert_eq!(battle.view().encounter().number(), 2);
    let linked = battle
        .view()
        .units_by_id()
        .find(|unit| unit.form() == definition(2))
        .unwrap();
    assert_eq!(linked.presence(), PresenceState::Departed);
    assert!(
        !battle
            .view()
            .links()
            .find(|link| link.entity() == LinkedEntity::Unit(linked.id()))
            .unwrap()
            .is_active()
    );
    let order = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Wave(WaveEventData::Ended { .. }) => Some(0),
            BattleEventKind::Unit(UnitEventData::LinkSettled { .. }) => Some(1),
            BattleEventKind::Wave(WaveEventData::Started { .. }) => Some(2),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(order, [0, 1, 2]);
}

#[test]
fn lifecycle_replay_is_deterministic() {
    let mut first = fixture_battle();
    let mut second = fixture_battle();
    open_normal_action(&mut first);
    open_normal_action(&mut second);
    let first_resolution = use_ability(&mut first, 5);
    let second_resolution = use_ability(&mut second, 5);
    assert_eq!(first_resolution, second_resolution);
    assert_eq!(first.state_hash(), second.state_hash());
}
