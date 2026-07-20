use std::sync::Arc;

use starclock_combat::{
    Battle, BattleEventKind, BattleSeed, BattleSpec, BattleSpecDigest, CombatantSpecDigest,
    Command, ConcedePolicy, EncounterWaveId, FormationIndex, Hp, ParticipantSource,
    ParticipantSpec, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings, Scalar, Speed,
    TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            ActionHitDefinition, ActionResourcePolicy, HitTargetGroup, TargetInvalidationPolicy,
            TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
        encounter::{EncounterWaveDefinition, WaveCarry, WaveSlotDefinition, WaveTransitionPolicy},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    formula::model::{CombatElement, DamageClass},
    rule::model::{ProgramStep, RuleOperationTemplate, RuleValue, ValueExpr},
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

fn catalog(program: ProgramDefinition) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new("ability-program-v1", [0x43; 32]);
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
        AbilityDefinition::new(id(1), id(1), id(1), vec![])
            .with_action(action)
            .with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, id(1)).unwrap(),
            ]),
    );
    builder.add_ability(
        AbilityDefinition::new(id(2), id(2), id(3), vec![]).with_action(empty_action()),
    );
    builder.add_unit(UnitDefinition::new(id(1), vec![id(1)], vec![]));
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

fn combatant(form: u32, ability: u32, digest: u8) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        id(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(if form == 1 { 100_000_000 } else { 1_000_000 }).unwrap(),
        ResolvedDefinitionBindings::new(vec![id(ability)], vec![], vec![]).unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
}

fn battle(catalog: Arc<CombatCatalog>) -> Battle {
    let spec = BattleSpec::new(
        "ability-program-rules-v1",
        BattleSpecDigest::new([0x44; 32]).unwrap(),
        id(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 1, 0x45),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(id(1)),
                combatant(2, 2, 0x46),
            ),
        ],
        TeamResourceSpec::new(0, 5).unwrap(),
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
    let mut battle = battle(catalog(program));
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
    let mut battle = battle(catalog(program));
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
