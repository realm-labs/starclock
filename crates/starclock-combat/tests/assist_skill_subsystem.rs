use std::sync::Arc;

use starclock_combat::{
    ActionEventData, ActionOrigin, Battle, BattleEventKind, BattleSeed, BattleSpec,
    BattleSpecDigest, CauseActor, CombatantSpecDigest, Command, ConcedePolicy, DispelCategory,
    DurationClock, EffectApplicationDefinition, EffectCategory, EffectChancePolicy,
    EffectRuntimeDefinition, EffectStackPolicy, EffectTickPhase, Energy, FormationIndex, Hp,
    KeyedTeamResourceSpec, ParticipantSource, ParticipantSpec, Ratio, RawToughness,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, ResourceEventData, Scalar, Speed, StatValue,
    TeamResourceSpec, TeamResourceWavePolicy, TeamSide, ToughnessLayerSpec,
    ToughnessReductionDefinition, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, AbilityTag, ActionHitDefinition,
            ActionResourcePolicy, HitOperationDefinition, QueueActionDefinition, QueuedActor,
            QueuedOwner, QueuedTarget, ReactionBoundary, ScalingDamageDefinition,
            SkillPointPaymentPolicy, TargetInvalidationPolicy, TargetPattern, TargetRelation,
            TeamResourceCost, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition,
            ProgramDefinition, SelectorDefinition, UnitDefinition,
        },
    },
    formula::{
        model::{CombatElement, DamageClass},
        toughness::{BreakDamageDefinition, EnemyRank, ToughnessReductionContext},
    },
    modifier::model::StatKind,
};

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn combatant(
    form: u32,
    abilities: Vec<u32>,
    attack: i64,
    speed: i64,
    digest: u8,
) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        id(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(10_000).unwrap(),
        Speed::from_scaled(speed).unwrap(),
        ResolvedDefinitionBindings::new(abilities.into_iter().map(id).collect(), vec![], vec![])
            .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
    .with_base_attack_defense(
        StatValue::from_scaled(attack).unwrap(),
        StatValue::from_scaled(0).unwrap(),
    )
}

fn action(
    kind: AbilityKind,
    tags: &[AbilityTag],
    resources: ActionResourcePolicy,
    operations: Vec<HitOperationDefinition>,
) -> AbilityActionDefinition {
    AbilityActionDefinition::new(
        kind,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        resources,
    )
    .unwrap()
    .with_tags(tags)
    .with_hits(vec![ActionHitDefinition::new(operations)])
    .unwrap()
}

fn break_damage() -> BreakDamageDefinition {
    BreakDamageDefinition {
        attacker_level_multiplier: Scalar::ONE,
        ability_multiplier: Ratio::ONE,
        break_effect: Ratio::ZERO,
        break_damage_increase: Ratio::ZERO,
        defense_multiplier: Ratio::ONE,
        resistance_multiplier: Ratio::ONE,
        vulnerability_multiplier: Ratio::ONE,
        mitigation_multiplier: Ratio::ONE,
        unbroken_multiplier: Ratio::from_scaled(900_000),
    }
}

fn catalog(expiring_grant: bool) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new("assist-skill-v1", [0xa1; 32]);
    for (raw, relation, pattern) in [
        (1, TargetRelation::Allied, TargetPattern::Single),
        (2, TargetRelation::Opposing, TargetPattern::All),
        (3, TargetRelation::Opposing, TargetPattern::Single),
    ] {
        builder.add_selector(
            SelectorDefinition::new(id(raw))
                .with_unit_targets(UnitTargetSelector::new(relation, pattern).unwrap()),
        );
        builder.add_program(ProgramDefinition::new(
            id(raw),
            vec![],
            vec![id(raw)],
            vec![],
            vec![],
        ));
    }

    let (duration, duration_clock) = if expiring_grant {
        (Some(1), DurationClock::TargetTurnStart)
    } else {
        (None, DurationClock::Permanent)
    };
    let grant = EffectRuntimeDefinition::new(
        EffectCategory::Buff,
        DispelCategory::DispellableBuff,
        1,
        duration,
        duration_clock,
        EffectTickPhase::None,
        EffectStackPolicy::UniquePerSource,
    )
    .unwrap();
    builder.add_effect(
        EffectDefinition::new(id(1), vec![], vec![])
            .with_runtime(grant)
            .with_granted_abilities(vec![id(2)]),
    );

    let assist_resources = ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO)
        .with_team_resource_costs(vec![TeamResourceCost::new("assist-use", 1).unwrap()])
        .unwrap();
    let assist = action(
        AbilityKind::Skill,
        &[AbilityTag::Attack, AbilityTag::Skill, AbilityTag::Assist],
        assist_resources,
        vec![
            HitOperationDefinition::ScalingDamage(
                ScalingDamageDefinition::new(
                    StatKind::Atk,
                    Ratio::from_scaled(500_000),
                    DamageClass::Direct,
                    CombatElement::Fire,
                )
                .unwrap(),
            ),
            HitOperationDefinition::ReduceToughness(ToughnessReductionDefinition {
                element: CombatElement::Fire,
                ignores_weakness: true,
                reduction: ToughnessReductionContext {
                    base: RawToughness::new(10).unwrap(),
                    additive: RawToughness::new(0).unwrap(),
                    reduction_increase: Ratio::ZERO,
                    weakness_break_efficiency: Ratio::ZERO,
                    weakness_break_efficiency_cap: Ratio::from_scaled(3_000_000),
                    toughness_vulnerability: Ratio::ZERO,
                    ability_multiplier: Ratio::ONE,
                },
                break_damage: break_damage(),
                break_effect_chance: starclock_combat::Probability::ONE,
            }),
        ],
    );
    let forced = QueueActionDefinition::new(
        id(2),
        ActionOrigin::Forced,
        QueuedActor::PrimaryTarget,
        QueuedTarget::None,
        ReactionBoundary::AfterHit,
        -100,
    )
    .with_envelope(
        QueuedOwner::CauseOwner,
        Some(SkillPointPaymentPolicy::Suppressed),
    );
    builder.add_ability(
        AbilityDefinition::new(id(1), id(1), id(1), vec![id(1)]).with_action(action(
            AbilityKind::Skill,
            &[AbilityTag::Skill],
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
            vec![
                HitOperationDefinition::ApplyEffect(
                    EffectApplicationDefinition::new(id(1), EffectChancePolicy::Guaranteed, 1)
                        .unwrap(),
                ),
                HitOperationDefinition::QueueAction(forced),
            ],
        )),
    );
    builder.add_ability(AbilityDefinition::new(id(2), id(2), id(2), vec![]).with_action(assist));
    for raw in [3, 4] {
        builder.add_ability(
            AbilityDefinition::new(id(raw), id(3), id(3), vec![]).with_action(action(
                AbilityKind::Basic,
                &[AbilityTag::Attack, AbilityTag::Basic],
                ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
                vec![],
            )),
        );
    }
    builder.add_unit(UnitDefinition::new(id(1), vec![id(1)], vec![]));
    builder.add_unit(UnitDefinition::new(id(2), vec![id(3)], vec![]));
    builder.add_unit(UnitDefinition::new(id(3), vec![id(4)], vec![]));
    builder.add_enemy(EnemyDefinition::new(id(1), id(3), vec![id(4)]));
    builder.add_encounter(EncounterDefinition::new(id(1), vec![id(1)], vec![]));
    builder.build().unwrap()
}

fn battle(assist_uses: u16, expiring_grant: bool) -> Battle {
    let enemy = combatant(3, vec![4], 0, 50_000_000, 3)
        .with_toughness(
            EnemyRank::Normal,
            vec![CombatElement::Ice],
            vec![ToughnessLayerSpec::ordinary(1, RawToughness::new(30).unwrap()).unwrap()],
        )
        .unwrap();
    let player_resources = TeamResourceSpec::new(0, 5)
        .unwrap()
        .with_keyed(vec![
            KeyedTeamResourceSpec::new(id(90), assist_uses, 1, TeamResourceWavePolicy::Persist)
                .unwrap()
                .with_stable_key("assist-use")
                .unwrap(),
        ])
        .unwrap();
    let spec = BattleSpec::new(
        "assist-skill-rules-v1",
        BattleSpecDigest::new([0xa2; 32]).unwrap(),
        id(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, vec![1], 2_000_000_000, 300_000_000, 1),
            ),
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(1).unwrap(),
                ParticipantSource::Player,
                combatant(2, vec![3], 100_000_000, 200_000_000, 2),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(id(1)),
                enemy,
            ),
        ],
        player_resources,
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog(expiring_grant), spec, BattleSeed::new([0xa3; 32])).unwrap()
}

fn start_and_apply_grant(battle: &mut Battle) -> starclock_combat::Resolution {
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    battle
        .apply(Command::PassInterruptWindow {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(command,
            Command::UseAbility { ability, primary_target: Some(target), .. }
                if ability.get() == 1 && target.get() == 2)
        })
        .unwrap()
        .clone();
    battle.apply(command).unwrap()
}

fn pass_interrupt(battle: &mut Battle) {
    battle
        .apply(Command::PassInterruptWindow {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
}

#[test]
fn grant_exposes_provider_owned_assist_and_normal_use_spends_shared_counter() {
    let mut battle = battle(1, false);
    let forced = start_and_apply_grant(&mut battle);

    let declared = forced
        .events()
        .iter()
        .find(|event| {
            matches!(event.kind(),
            BattleEventKind::Action(ActionEventData::Declared {
                actor, ability, origin: ActionOrigin::Forced, tags, ..
            }) if actor.get() == 2 && ability.get() == 2 && tags.contains(AbilityTag::Assist))
        })
        .expect("the granted Assist must be eligible for a forced use");
    assert_eq!(declared.cause().owner().unwrap().get(), 1);
    assert_eq!(
        declared.cause().actor(),
        Some(CauseActor::Unit(starclock_combat::UnitId::new(2).unwrap()))
    );
    assert_eq!(declared.cause().applier().unwrap().get(), 1);
    assert_eq!(
        battle.view().team(TeamSide::Player).keyed_resource(id(90)),
        Some((1, 1))
    );
    assert!(!forced.events().iter().any(|event| matches!(event.kind(),
        BattleEventKind::Resource(ResourceEventData::TeamResource { resource, .. }) if resource.get() == 90)));
    assert!(forced.events().iter().any(|event| matches!(event.kind(),
        BattleEventKind::Damage(data) if data.raw.scaled() == 1_000_000_000)));
    assert!(forced.events().iter().any(|event| matches!(event.kind(),
        BattleEventKind::Toughness(starclock_combat::ToughnessEventData::Reduced {
            effective, ..
        }) if effective.get() == 10)));

    pass_interrupt(&mut battle);
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(command,
            Command::UseAbility { actor, ability, .. }
                if actor.get() == 2 && ability.get() == 2)
        })
        .expect("the effect grant must expose Assist on the recipient's turn")
        .clone();
    let normal = battle.apply(command).unwrap();
    let declared = normal
        .events()
        .iter()
        .find(|event| {
            matches!(event.kind(),
        BattleEventKind::Action(ActionEventData::Declared {
            actor, ability, origin: ActionOrigin::NormalTurn, ..
        }) if actor.get() == 2 && ability.get() == 2)
        })
        .unwrap();
    assert_eq!(declared.cause().owner().unwrap().get(), 1);
    assert_eq!(
        declared.cause().actor(),
        Some(CauseActor::Unit(starclock_combat::UnitId::new(2).unwrap()))
    );
    assert_eq!(declared.cause().applier().unwrap().get(), 1);
    assert!(normal.events().iter().any(|event| matches!(event.kind(),
        BattleEventKind::Resource(ResourceEventData::TeamResource {
            resource, attempted: 1, effective: 1, before: 1, after: 0, ..
        }) if resource.get() == 90)));
    assert_eq!(
        battle.view().team(TeamSide::Player).keyed_resource(id(90)),
        Some((0, 1))
    );
}

#[test]
fn an_empty_shared_counter_hides_normal_assist_but_not_the_no_cost_forced_use() {
    let mut battle = battle(0, false);
    let forced = start_and_apply_grant(&mut battle);
    assert!(forced.events().iter().any(|event| matches!(event.kind(),
        BattleEventKind::Action(ActionEventData::Declared {
            ability, origin: ActionOrigin::Forced, ..
        }) if ability.get() == 2)));
    pass_interrupt(&mut battle);
    assert!(
        !battle
            .decision()
            .unwrap()
            .legal_commands()
            .iter()
            .any(|command| matches!(command,
        Command::UseAbility { ability, .. } if ability.get() == 2))
    );
}

#[test]
fn expiring_the_provider_effect_revokes_assist_before_the_recipient_decision() {
    let mut battle = battle(1, true);
    let forced = start_and_apply_grant(&mut battle);
    assert!(forced.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Action(ActionEventData::Declared {
            ability,
            origin: ActionOrigin::Forced,
            ..
        }) if ability.get() == 2
    )));
    assert_eq!(battle.view().effects_by_id().count(), 0);
    pass_interrupt(&mut battle);
    assert!(
        !battle
            .decision()
            .unwrap()
            .legal_commands()
            .iter()
            .any(|command| matches!(command,
            Command::UseAbility { ability, .. } if ability.get() == 2))
    );
}
