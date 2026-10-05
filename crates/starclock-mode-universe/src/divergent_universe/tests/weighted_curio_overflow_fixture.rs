//! Real battle commands using production Sora operands and an explicit test base formula.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    tests::weighted_curio_overflow_lifecycle_fixture::{Setup, add_setup},
    tests::weighted_curio_overflow_queued_fixture::bind_queued_probe,
    weighted_curio_overflow::{
        OverflowBaseDamagePolicy, attack_increase::bind_attack_increase_policy,
        bind_death_conversion_policy,
    },
};
use starclock_combat::{
    AbilityId, AssemblyDigest, Battle, BattleEvent, BattleSeed, BattleSpec, CombatantSpecDigest,
    Command, ConcedePolicy, DispelCategory, DurationClock, EffectCategory, EffectDamageGuard,
    EffectRuntimeDefinition, EffectStackPolicy, EffectTickPhase, Energy, FormationIndex, Hp,
    ParticipantSource, ParticipantSpec, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    Rounding, RuleBundleId, Scalar, Speed, StatValue, TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            ActionHitDefinition, ActionResourcePolicy, HitCritPolicy, HitOperationDefinition,
            HitTargetGroup, OrdinaryDamageDefinition, OrdinaryDamageMultipliers, ReactionBoundary,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition,
            ProgramDefinition, SelectorDefinition, UnitDefinition,
        },
        encounter::{
            AiCandidateDefinition, AiCandidateSelection, AiGraphDefinition, AiNoTargetFallback,
            AiStateDefinition, EncounterWaveDefinition, EnemyPhaseCarry, EnemyPhaseDefinition,
            EnemyPhaseTransitionModel, PhaseCarryPolicy, WaveCarry, WaveSlotDefinition,
            WaveTransitionPolicy,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
    rule::model::{
        ConditionExpr, ProgramStep, RuleEffectChancePolicy, RuleOperationTemplate, RuleValue,
        RuleValueKind, ValueExpr,
    },
};

pub(super) fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

#[derive(Clone)]
pub(super) struct Probe {
    pub(super) player_form: u32,
    pub(super) attack_probe: bool,
    pub(super) hp: Vec<i64>,
    pub(super) levels: Vec<u8>,
    pub(super) pattern: TargetPattern,
    pub(super) damages: Vec<i64>,
    pub(super) base_factor: i64,
    pub(super) seed: u8,
    pub(super) inherited: bool,
    pub(super) shield: i64,
    pub(super) guard: Option<EffectDamageGuard>,
    pub(super) hp_floor: Option<Ratio>,
    pub(super) phase_targets: u8,
    pub(super) setup: Option<Setup>,
    pub(super) queued: Option<ReactionBoundary>,
    pub(super) queued_single: bool,
}
impl Default for Probe {
    fn default() -> Self {
        Self {
            player_form: 1,
            attack_probe: false,
            hp: vec![100, 80, 3_000, 9_000],
            levels: vec![1, 81, 95, 50],
            pattern: TargetPattern::Blast,
            damages: vec![250],
            base_factor: 2,
            seed: 0x81,
            inherited: false,
            shield: 0,
            guard: None,
            hp_floor: None,
            phase_targets: 0,
            setup: None,
            queued: None,
            queued_single: false,
        }
    }
}

fn combatant(
    form: u32,
    level: u8,
    hp: i64,
    player: bool,
    bundles: Vec<RuleBundleId>,
) -> ResolvedCombatantSpec {
    combatant_with_abilities(
        form,
        level,
        hp,
        player,
        bundles,
        vec![id(if player { 1 } else { 2 })],
    )
}

pub(super) fn combatant_with_abilities(
    form: u32,
    level: u8,
    hp: i64,
    player: bool,
    bundles: Vec<RuleBundleId>,
    abilities: Vec<AbilityId>,
) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        id(form),
        UnitLevel::new(level).unwrap(),
        Hp::new(hp).unwrap(),
        Speed::from_scaled(if form == 3 {
            200_000_000
        } else if player {
            100_000_000
        } else {
            1_000_000
        })
        .unwrap(),
        ResolvedDefinitionBindings::new(abilities, bundles, vec![]).unwrap(),
        CombatantSpecDigest::new([level; 32]).unwrap(),
    )
    .unwrap()
}

pub(super) fn scenario(fixture: &DivergentUniverseBaselineFixture, probe: &Probe) -> Battle {
    // Fixture-only base = selected enemy's own level * explicit factor. It is
    // deliberately not presented as decoded HPRatio/difficulty semantics.
    let policy = OverflowBaseDamagePolicy::new(
        ValueExpr::Multiply {
            lhs: Box::new(ValueExpr::Convert {
                value: Box::new(ValueExpr::QueryUnitLevel(StatQuerySubject::CurrentTarget)),
                target: RuleValueKind::Scalar,
                rounding: Rounding::Floor,
            }),
            rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(
                Scalar::checked_from_integer(probe.base_factor).unwrap(),
            ))),
            rounding: Rounding::Floor,
        },
        [u8::try_from(probe.base_factor).unwrap(); 32],
    );
    scenario_with_base(fixture, probe, &policy)
}

pub(super) fn scenario_with_base(
    fixture: &DivergentUniverseBaselineFixture,
    probe: &Probe,
    policy: &OverflowBaseDamagePolicy,
) -> Battle {
    let mut builder = CombatCatalogBuilder::new([0x81; 32]);
    builder.add_selector(SelectorDefinition::new(id(1)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, probe.pattern).unwrap(),
    ));
    builder.add_selector(SelectorDefinition::new(id(2)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_program(ProgramDefinition::new(
        id(1),
        vec![],
        vec![],
        vec![],
        vec![],
    ));
    let shield_selector = id(10);
    builder.add_selector(
        SelectorDefinition::new(shield_selector).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::Encounter,
                RuleSelectorSide::Opposing,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Present,
                RuleSelectorReference::CurrentState,
                RuleSelectorOrdering::Formation,
                0,
                64,
                RuleEmptyPoolPolicy::NoOp,
                RuleSelectorChoice::All,
                None,
                false,
            )
            .unwrap()
            .with_predicates(vec![RuleSelectorPredicate::FormationRange {
                minimum: 0,
                maximum: 1,
            }]),
        ),
    );
    let runtime = EffectRuntimeDefinition::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .unwrap();
    let runtime = probe
        .guard
        .map_or(runtime.clone(), |guard| runtime.with_damage_guard(guard));
    let runtime = probe.hp_floor.map_or(runtime.clone(), |floor| {
        runtime.with_hp_floor(floor).unwrap()
    });
    builder.add_effect(EffectDefinition::new(id(10), vec![], vec![]).with_runtime(runtime));
    builder.add_program(
        ProgramDefinition::new(id(10), vec![], vec![shield_selector], vec![id(10)], vec![])
            .with_steps(
                if probe.shield > 0 || probe.guard.is_some() || probe.hp_floor.is_some() {
                    vec![
                        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                            selector: shield_selector,
                            effect: id(10),
                            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                            chance: RuleEffectChancePolicy::Guaranteed,
                            base_chance: None,
                            rng_purpose: None,
                        }),
                        ProgramStep::Operation(RuleOperationTemplate::Shield {
                            selector: shield_selector,
                            effect: id(10),
                            amount: ValueExpr::Literal(RuleValue::Scalar(
                                Scalar::checked_from_integer(probe.shield).unwrap(),
                            )),
                        }),
                    ]
                } else {
                    vec![]
                },
            ),
    );
    let hits = probe
        .damages
        .iter()
        .map(|amount| {
            ActionHitDefinition::new(vec![HitOperationDefinition::Damage(
                OrdinaryDamageDefinition::new(
                    Scalar::checked_from_integer(*amount).unwrap(),
                    OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                )
                .unwrap(),
            )])
            .with_profile(
                HitTargetGroup::Selected,
                Ratio::ONE,
                Ratio::ONE,
                HitCritPolicy::Never,
            )
        })
        .collect();
    builder.add_ability(
        AbilityDefinition::new(id(1), id(1), id(1), vec![])
            .with_action(
                AbilityActionDefinition::new(
                    AbilityKind::Basic,
                    u16::try_from(probe.damages.len()).unwrap(),
                    TargetInvalidationPolicy::CancelRemainingForTarget,
                    ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
                )
                .unwrap()
                .with_hits(hits)
                .unwrap(),
            )
            .with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, id(10)).unwrap(),
            ]),
    );
    builder.add_ability(
        AbilityDefinition::new(id(2), id(1), id(2), vec![]).with_action(
            AbilityActionDefinition::new(
                AbilityKind::Basic,
                1,
                TargetInvalidationPolicy::CancelRemainingForTarget,
                ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
            )
            .unwrap(),
        ),
    );
    let mut player_abilities = if probe.setup.is_some() {
        vec![id(1), id(3)]
    } else {
        vec![id(1)]
    };
    if probe.queued.is_some() {
        player_abilities.push(id(8));
    }
    if probe.attack_probe {
        player_abilities.push(id(7));
        builder.add_selector(
            SelectorDefinition::new(id(7)).with_rule_units(
                RuleUnitSelector::new(
                    RuleSelectorOrigin::Encounter,
                    RuleSelectorSide::Opposing,
                    RuleLifePredicate::Alive,
                    RulePresencePredicate::Present,
                    RuleSelectorReference::CurrentState,
                    RuleSelectorOrdering::Formation,
                    0,
                    1,
                    RuleEmptyPoolPolicy::NoOp,
                    RuleSelectorChoice::First,
                    None,
                    false,
                )
                .unwrap(),
            ),
        );
        builder.add_program(
            ProgramDefinition::new(id(7), vec![], vec![id(7)], vec![], vec![]).with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::TrueDamage {
                    selector: id(7),
                    amount: ValueExpr::QueryStat {
                        subject: StatQuerySubject::Actor,
                        stat: StatKind::Atk,
                        purpose: FormulaPurpose::Stat,
                    },
                }),
            ]),
        );
        builder.add_ability(
            AbilityDefinition::new(id(7), id(7), id(2), vec![])
                .with_action(
                    AbilityActionDefinition::new(
                        AbilityKind::Skill,
                        1,
                        TargetInvalidationPolicy::CancelRemainingForTarget,
                        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
                    )
                    .unwrap(),
                )
                .with_programs(vec![
                    AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, id(7)).unwrap(),
                ]),
        );
    }
    for (form, abilities) in [
        (
            probe.player_form,
            if matches!(probe.setup, Some(Setup::Linked(..))) {
                vec![id(1), id(3), id(6)]
            } else {
                player_abilities.clone()
            },
        ),
        (2, vec![id(2)]),
        (
            3,
            if probe.setup.is_some() {
                vec![id(1), id(4)]
            } else {
                vec![id(1)]
            },
        ),
    ] {
        let mut abilities = abilities;
        if probe.attack_probe && form != 2 && !abilities.contains(&id(7)) {
            abilities.push(id(7));
        }
        builder.add_unit(UnitDefinition::new(id(form), abilities, vec![]));
    }
    builder.add_enemy(EnemyDefinition::new(id(1), id(2), vec![id(2)]));
    let mut encounter_enemies = vec![id(1)];
    if probe.phase_targets > 0 {
        assert!(probe.phase_targets <= 2);
        add_phase_enemy(&mut builder);
        encounter_enemies.push(id(2));
    }
    let encounter = EncounterDefinition::new(id(1), encounter_enemies, vec![]);
    let encounter = if probe.phase_targets > 0 {
        let slots = (0..probe.hp.len())
            .map(|index| {
                let phased = index < usize::from(probe.phase_targets);
                WaveSlotDefinition::new(
                    u16::try_from(index + 1).unwrap(),
                    FormationIndex::new(u8::try_from(index).unwrap()).unwrap(),
                    id(if phased { 2 } else { 1 }),
                    None,
                    phased.then(|| id(1)),
                    true,
                )
                .unwrap()
            })
            .collect();
        encounter
            .with_authored_waves(
                WaveTransitionPolicy::AfterAction,
                vec![
                    EncounterWaveDefinition::new(id(1), 1, None, None, WaveCarry::CARRY_ALL, slots)
                        .unwrap(),
                ],
            )
            .unwrap()
    } else {
        encounter
    };
    builder.add_encounter(encounter);
    let player = ParticipantSpec::new(
        TeamSide::Player,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::Player,
        combatant_with_abilities(
            probe.player_form,
            70,
            10_000,
            true,
            vec![],
            player_abilities,
        )
        .with_base_attack_defense(
            StatValue::from_scaled(if probe.attack_probe { 100_000_000 } else { 0 }).unwrap(),
            StatValue::from_scaled(0).unwrap(),
        ),
    );
    let player = if probe.attack_probe {
        bind_attack_increase_policy(
            &mut builder,
            &fixture
                .factory()
                .decision_catalog()
                .weighted_curio_overflows()[0],
            fixture.core(),
            &player,
            [0x82; 32],
        )
        .unwrap()
    } else {
        bind_death_conversion_policy(
            &mut builder,
            &fixture
                .factory()
                .decision_catalog()
                .weighted_curio_overflows()[0],
            &player,
            policy,
            [0x82; 32],
        )
        .unwrap()
    };
    let player = probe.queued.map_or_else(
        || player.clone(),
        |boundary| bind_queued_probe(&mut builder, &player, boundary, probe.queued_single),
    );
    if let Some(setup) = probe.setup {
        add_setup(&mut builder, &player, setup);
    }
    let mut participants = vec![player.clone()];
    if probe.inherited {
        let spec = combatant(
            3,
            13,
            10_000,
            true,
            player.combatant().rule_bundles().to_vec(),
        )
        .with_sources(player.combatant().sources().to_vec())
        .unwrap();
        participants.push(ParticipantSpec::new(
            TeamSide::Player,
            FormationIndex::new(1).unwrap(),
            ParticipantSource::Player,
            spec,
        ));
    }
    for (index, (hp, level)) in probe.hp.iter().zip(&probe.levels).enumerate() {
        participants.push(ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(u8::try_from(index).unwrap()).unwrap(),
            ParticipantSource::EncounterEnemy(id(if index < usize::from(probe.phase_targets) {
                2
            } else {
                1
            })),
            combatant(2, *level, *hp, false, vec![]),
        ));
    }
    Battle::create(
        builder.build().unwrap(),
        BattleSpec::new(
            AssemblyDigest::new([0x83; 32]).unwrap(),
            id(1),
            participants,
            TeamResourceSpec::new(3, 5).unwrap(),
            TeamResourceSpec::new(0, 0).unwrap(),
            ConcedePolicy::Allowed,
        )
        .unwrap(),
        BattleSeed::new([probe.seed; 32]),
    )
    .unwrap()
}

fn add_phase_enemy(builder: &mut CombatCatalogBuilder) {
    builder.add_ai_graph(
        AiGraphDefinition::new(
            id(1),
            id(1),
            4,
            vec![AiStateDefinition::new(
                id(1),
                None,
                id(2),
                false,
                vec![AiCandidateDefinition::new(
                    id(1),
                    id(2),
                    ConditionExpr::Literal(true),
                    id(2),
                    0,
                    AiCandidateSelection::FirstLegal,
                    AiNoTargetFallback::StayInState,
                )],
                vec![],
            )],
        )
        .unwrap(),
    );
    let phases = (1..=2)
        .map(|sequence| {
            EnemyPhaseDefinition::new(
                id(u32::from(sequence)),
                sequence,
                ConditionExpr::Literal(true),
                ConditionExpr::Literal(false),
                0,
                id(1),
                true,
                EnemyPhaseTransitionModel::TransformSameUnit,
                None,
                EnemyPhaseCarry {
                    hp: if sequence == 2 {
                        PhaseCarryPolicy::Reset
                    } else {
                        PhaseCarryPolicy::CarryExact
                    },
                    action_gauge: PhaseCarryPolicy::CarryExact,
                    effects: PhaseCarryPolicy::CarryExact,
                    toughness: PhaseCarryPolicy::CarryExact,
                    summons: PhaseCarryPolicy::CarryExact,
                },
            )
        })
        .collect();
    builder.add_enemy(
        EnemyDefinition::new(id(2), id(2), vec![id(2)])
            .with_orchestration(id(1), phases)
            .unwrap(),
    );
}

pub(super) fn accept(battle: &mut Battle, command: Command) -> Vec<BattleEvent> {
    let resolution = battle.apply(command).unwrap();
    assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    resolution.events().to_vec()
}
pub(super) fn start(battle: &mut Battle) {
    accept(
        battle,
        Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        },
    );
}
pub(super) fn cast(battle: &mut Battle, actor: u64, target: Option<u64>) -> Vec<BattleEvent> {
    cast_ability(battle, actor, 1, target)
}

pub(super) fn cast_ability(
    battle: &mut Battle,
    actor: u64,
    ability: u32,
    target: Option<u64>,
) -> Vec<BattleEvent> {
    for _ in 0..32 {
        if let Some(command) = battle.decision().and_then(|decision| decision.legal_commands().iter().find(|command| {
            matches!(command,Command::UseAbility {actor:a,ability:offered,primary_target,..} if a.get()==actor && offered.get()==ability && primary_target.map(|t|t.get())==target)
        })).cloned() {return accept(battle,command);}
        let command = battle.advance_command().unwrap();
        accept(battle, command);
    }
    panic!("attack was not offered");
}
