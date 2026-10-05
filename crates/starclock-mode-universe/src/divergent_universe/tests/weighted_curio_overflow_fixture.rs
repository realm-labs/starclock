//! Real battle commands using production Sora operands and an explicit test base formula.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    weighted_curio_overflow::{OverflowBaseDamagePolicy, bind_death_conversion_policy},
};
use starclock_combat::{
    AssemblyDigest, Battle, BattleEvent, BattleSeed, BattleSpec, CombatantSpecDigest, Command,
    ConcedePolicy, DispelCategory, DurationClock, EffectCategory, EffectDamageGuard,
    EffectRuntimeDefinition, EffectStackPolicy, EffectTickPhase, Energy, FormationIndex, Hp,
    ParticipantSource, ParticipantSpec, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    Rounding, RuleBundleId, Scalar, Speed, TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            ActionHitDefinition, ActionResourcePolicy, HitCritPolicy, HitOperationDefinition,
            HitTargetGroup, OrdinaryDamageDefinition, OrdinaryDamageMultipliers,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition,
            ProgramDefinition, SelectorDefinition, UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    modifier::model::StatQuerySubject,
    rule::model::{
        ProgramStep, RuleEffectChancePolicy, RuleOperationTemplate, RuleValue, RuleValueKind,
        ValueExpr,
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
    pub(super) hp: Vec<i64>,
    pub(super) levels: Vec<u8>,
    pub(super) pattern: TargetPattern,
    pub(super) damages: Vec<i64>,
    pub(super) base_factor: i64,
    pub(super) seed: u8,
    pub(super) inherited: bool,
    pub(super) shield: i64,
    pub(super) guard: Option<EffectDamageGuard>,
}
impl Default for Probe {
    fn default() -> Self {
        Self {
            hp: vec![100, 80, 3_000, 9_000],
            levels: vec![1, 81, 95, 50],
            pattern: TargetPattern::Blast,
            damages: vec![250],
            base_factor: 2,
            seed: 0x81,
            inherited: false,
            shield: 0,
            guard: None,
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
        ResolvedDefinitionBindings::new(vec![id(if player { 1 } else { 2 })], bundles, vec![])
            .unwrap(),
        CombatantSpecDigest::new([level; 32]).unwrap(),
    )
    .unwrap()
}

pub(super) fn scenario(fixture: &DivergentUniverseBaselineFixture, probe: &Probe) -> Battle {
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
    builder.add_effect(
        EffectDefinition::new(id(10), vec![], vec![]).with_runtime(
            probe
                .guard
                .map_or(runtime.clone(), |guard| runtime.with_damage_guard(guard)),
        ),
    );
    builder.add_program(
        ProgramDefinition::new(id(10), vec![], vec![shield_selector], vec![id(10)], vec![])
            .with_steps(if probe.shield > 0 || probe.guard.is_some() {
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
            }),
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
    for (form, ability) in [(1, 1), (2, 2), (3, 1)] {
        builder.add_unit(UnitDefinition::new(id(form), vec![id(ability)], vec![]));
    }
    builder.add_enemy(EnemyDefinition::new(id(1), id(2), vec![id(2)]));
    builder.add_encounter(EncounterDefinition::new(id(1), vec![id(1)], vec![]));
    let player = ParticipantSpec::new(
        TeamSide::Player,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::Player,
        combatant(1, 70, 10_000, true, vec![]),
    );
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
    let player = bind_death_conversion_policy(
        &mut builder,
        &fixture
            .factory()
            .decision_catalog()
            .weighted_curio_overflows()[0],
        &player,
        &policy,
        [0x82; 32],
    )
    .unwrap();
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
            ParticipantSource::EncounterEnemy(id(1)),
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
    for _ in 0..32 {
        if let Some(command) = battle.decision().and_then(|decision| decision.legal_commands().iter().find(|command| {
            matches!(command,Command::UseAbility {actor:a,ability,primary_target,..} if a.get()==actor && ability.get()==1 && primary_target.map(|t|t.get())==target)
        })).cloned() {return accept(battle,command);}
        let command = battle.advance_command().unwrap();
        accept(battle, command);
    }
    panic!("attack was not offered");
}
