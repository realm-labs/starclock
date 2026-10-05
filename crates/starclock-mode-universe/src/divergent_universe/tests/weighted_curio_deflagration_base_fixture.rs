//! Formula-only command consumer, not Deflagration equipment or Burn execution.
use crate::divergent_universe::weighted_curio::deflagration::DeflagrationBaseDamagePolicy;
use starclock_combat::{
    AssemblyDigest, Battle, BattleEvent, BattleSeed, BattleSpec, CombatantSpecDigest, Command,
    ConcedePolicy, Energy, FormationIndex, Hp, ParticipantSource, ParticipantSpec,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed, TeamResourceSpec, TeamSide,
    UnitLevel,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            ActionResourcePolicy, TargetInvalidationPolicy, TargetPattern, TargetRelation,
            UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    rule::model::{ProgramStep, RuleOperationTemplate},
};

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

pub(super) fn scenario(policy: &DeflagrationBaseDamagePolicy, level: u8) -> Battle {
    let mut builder = CombatCatalogBuilder::new(policy.identity());
    builder.add_selector(SelectorDefinition::new(id(1)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    for (raw, origin) in [
        (2, RuleSelectorOrigin::Encounter),
        (3, RuleSelectorOrigin::CurrentSubject),
    ] {
        builder.add_selector(
            SelectorDefinition::new(id(raw)).with_rule_units(
                RuleUnitSelector::new(
                    origin,
                    RuleSelectorSide::Opposing,
                    RuleLifePredicate::Alive,
                    RulePresencePredicate::Present,
                    RuleSelectorReference::CurrentState,
                    RuleSelectorOrdering::StableId,
                    0,
                    1,
                    RuleEmptyPoolPolicy::NoOp,
                    RuleSelectorChoice::All,
                    None,
                    false,
                )
                .unwrap(),
            ),
        );
    }
    builder.add_program(
        ProgramDefinition::new(id(1), vec![id(2)], vec![id(2)], vec![], vec![]).with_steps(vec![
            ProgramStep::ForEach {
                selector: id(2),
                body: id(2),
                maximum: 1,
            },
        ]),
    );
    builder.add_program(
        ProgramDefinition::new(id(2), vec![], vec![id(3)], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
                code: 1012,
                value: Some(policy.expression().clone()),
            }),
            ProgramStep::Operation(RuleOperationTemplate::TrueDamage {
                selector: id(3),
                amount: policy.expression().clone(),
            }),
        ]),
    );
    builder.add_program(ProgramDefinition::new(
        id(3),
        vec![],
        vec![],
        vec![],
        vec![],
    ));
    for raw in [1, 2] {
        let action = AbilityActionDefinition::new(
            AbilityKind::Basic,
            1,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
        )
        .unwrap();
        let ability = AbilityDefinition::new(id(raw), id(3), id(1), vec![]).with_action(action);
        builder.add_ability(if raw == 1 {
            ability.with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, id(1)).unwrap(),
            ])
        } else {
            ability
        });
        builder.add_unit(UnitDefinition::new(id(raw), vec![id(raw)], vec![]));
    }
    builder.add_enemy(EnemyDefinition::new(id(1), id(2), vec![id(2)]));
    builder.add_encounter(EncounterDefinition::new(id(1), vec![id(1)], vec![]));
    let participants = [(true, 70), (false, level)]
        .map(|(player, level)| {
            let form = if player { 1 } else { 2 };
            ParticipantSpec::new(
                if player {
                    TeamSide::Player
                } else {
                    TeamSide::Enemy
                },
                FormationIndex::new(0).unwrap(),
                if player {
                    ParticipantSource::Player
                } else {
                    ParticipantSource::EncounterEnemy(id(1))
                },
                ResolvedCombatantSpec::new(
                    id(form),
                    UnitLevel::new(level).unwrap(),
                    Hp::new(10_000_000).unwrap(),
                    Speed::from_scaled(if player { 100_000_000 } else { 1_000_000 }).unwrap(),
                    ResolvedDefinitionBindings::new(vec![id(form)], vec![], vec![]).unwrap(),
                    CombatantSpecDigest::new([level; 32]).unwrap(),
                )
                .unwrap(),
            )
        })
        .to_vec();
    Battle::create(
        builder.build().unwrap(),
        BattleSpec::new(
            AssemblyDigest::new(policy.identity()).unwrap(),
            id(1),
            participants,
            TeamResourceSpec::new(0, 5).unwrap(),
            TeamResourceSpec::new(0, 0).unwrap(),
            ConcedePolicy::Allowed,
        )
        .unwrap(),
        BattleSeed::new([0x12; 32]),
    )
    .unwrap()
}

pub(super) fn cast(battle: &mut Battle) -> (Command, Vec<BattleEvent>) {
    let started = battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    assert!(started.fault().is_none());
    for _ in 0..8 {
        if let Some(command) = battle.decision().and_then(|decision| decision.legal_commands().iter()
            .find(|command| matches!(command, Command::UseAbility { actor, .. } if actor.get() == 1))).cloned() {
            let resolution = battle.apply(command.clone()).unwrap();
            assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
            return (command, resolution.events().to_vec());
        }
        let resolution = battle.apply(battle.advance_command().unwrap()).unwrap();
        assert!(resolution.fault().is_none());
    }
    panic!("formula probe must expose the player action");
}
