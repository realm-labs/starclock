use crate::combat_decision::pass_interrupt_if_offered;
use std::sync::Arc;

use starclock_combat::{
    ActionEventData, ActionOrigin, AssemblyDigest, Battle, BattleEventKind, BattleSeed, BattleSpec,
    CombatantSpecDigest, Command, ConcedePolicy, DispelCategory, DurationClock,
    EffectApplicationDefinition, EffectCategory, EffectChancePolicy, EffectRuntimeDefinition,
    EffectStackPolicy, EffectTickPhase, Energy, ForcedNormalAction, FormationIndex, Hp,
    ParticipantSource, ParticipantSpec, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    Scalar, Speed, TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HitOperationDefinition, OrdinaryDamageDefinition, OrdinaryDamageMultipliers,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition,
            ProgramDefinition, SelectorDefinition, UnitDefinition,
        },
    },
};

fn definition<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn action(kind: AbilityKind, operations: Vec<HitOperationDefinition>) -> AbilityActionDefinition {
    AbilityActionDefinition::new(
        kind,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap()
    .with_hits(vec![ActionHitDefinition::new(operations)])
    .unwrap()
}

fn catalog(forced_action: ForcedNormalAction) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new([0x91; 32]);
    for raw in 1..=2 {
        builder.add_selector(SelectorDefinition::new(definition(raw)).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
        ));
        builder.add_program(ProgramDefinition::new(
            definition(raw),
            vec![],
            vec![definition(raw)],
            vec![],
            vec![],
        ));
    }
    let outrage = EffectRuntimeDefinition::new(
        EffectCategory::Control,
        DispelCategory::CleanseableControl,
        1,
        Some(1),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .unwrap()
    .with_forced_normal_action(forced_action)
    .unwrap();
    builder.add_effect(EffectDefinition::new(definition(1), vec![], vec![]).with_runtime(outrage));
    let damage = HitOperationDefinition::Damage(
        OrdinaryDamageDefinition::new(
            Scalar::checked_from_integer(100).unwrap(),
            OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
        )
        .unwrap(),
    );
    builder.add_ability(
        AbilityDefinition::new(definition(1), definition(1), definition(1), vec![])
            .with_action(action(AbilityKind::Basic, vec![damage])),
    );
    builder.add_ability(
        AbilityDefinition::new(definition(2), definition(2), definition(2), vec![]).with_action(
            action(
                AbilityKind::Basic,
                vec![HitOperationDefinition::ApplyEffect(
                    EffectApplicationDefinition::new(
                        definition(1),
                        EffectChancePolicy::Guaranteed,
                        1,
                    )
                    .unwrap(),
                )],
            ),
        ),
    );
    builder.add_unit(UnitDefinition::new(
        definition(1),
        vec![definition(1)],
        vec![],
    ));
    builder.add_unit(UnitDefinition::new(
        definition(2),
        vec![definition(2)],
        vec![],
    ));
    builder.add_enemy(EnemyDefinition::new(
        definition(1),
        definition(2),
        vec![definition(2)],
    ));
    builder.add_encounter(EncounterDefinition::new(
        definition(1),
        vec![definition(1)],
        vec![],
    ));
    builder.build().unwrap()
}

fn combatant(form: u32, ability: u32, speed: i64, digest: u8) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        definition(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(speed).unwrap(),
        ResolvedDefinitionBindings::new(vec![definition(ability)], vec![], vec![]).unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
}

#[test]
fn outrage_replaces_the_turn_with_a_basic_attack_against_an_ally() {
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x92; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 1, 100_000_000, 1),
            ),
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(1).unwrap(),
                ParticipantSource::Player,
                combatant(1, 1, 90_000_000, 2),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(2, 2, 110_000_000, 3),
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle = Battle::create(
        catalog(ForcedNormalAction::BasicAttackRandomAlly),
        spec,
        BattleSeed::new([0x93; 32]),
    )
    .unwrap();
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    pass_interrupt_if_offered(&mut battle);
    let enemy_action = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseAbility {
                    ability,
                    primary_target: Some(target),
                    ..
                } if ability.get() == 2 && target.get() == 1
            )
        })
        .unwrap()
        .clone();
    let forced = battle.apply(enemy_action).unwrap();
    assert!(forced.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Action(ActionEventData::Resolved {
            actor,
            ability,
            origin: ActionOrigin::Forced,
            targets,
            ..
        }) if actor.get() == 1 && ability.get() == 1
            && targets.as_ref() == [starclock_combat::UnitId::new(2).unwrap()]
    )));
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .find(|unit| unit.id().get() == 2)
            .unwrap()
            .current_hp()
            .get(),
        900
    );
    assert_eq!(battle.view().rng_draw_count(), 2);
}

#[test]
fn taunt_replaces_the_turn_with_a_basic_attack_against_the_applier() {
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x94; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 1, 100_000_000, 1),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(2, 2, 110_000_000, 3),
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle = Battle::create(
        catalog(ForcedNormalAction::BasicAttackApplier),
        spec,
        BattleSeed::new([0x95; 32]),
    )
    .unwrap();
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    pass_interrupt_if_offered(&mut battle);
    let enemy_action = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseAbility {
                    ability,
                    primary_target: Some(target),
                    ..
                } if ability.get() == 2 && target.get() == 1
            )
        })
        .unwrap()
        .clone();
    let forced = battle.apply(enemy_action).unwrap();
    assert!(forced.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Action(ActionEventData::Resolved {
            actor,
            ability,
            origin: ActionOrigin::Forced,
            targets,
            ..
        }) if actor.get() == 1 && ability.get() == 1
            && targets.as_ref() == [starclock_combat::UnitId::new(2).unwrap()]
    )));
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .find(|unit| unit.id().get() == 2)
            .unwrap()
            .current_hp()
            .get(),
        900
    );
    assert_eq!(battle.view().rng_draw_count(), 1);
}
