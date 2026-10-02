//! Real commands for ordinary resistible Entanglement, not a Weakness Break.

use crate::combat_decision::advance_boundary_if_offered;
use starclock_combat::{
    AssemblyDigest, Battle, BattleEvent, BattleSeed, BattleSpec, CombatantSpecDigest, Command,
    ConcedePolicy, DispelCategory, DurationClock, EffectCategory, EffectRemovalDefinition,
    EffectRuntimeTemplate, EffectStackPolicy, EffectTeardownPolicy, EffectTickPhase, Energy,
    FormationIndex, Hp, ParticipantSource, ParticipantSpec, Ratio, RawToughness,
    ResolvedBuildBonuses, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    ResolvedModifierBinding, Scalar, Speed, TeamResourceSpec, TeamSide, ToughnessLayerKind,
    ToughnessLayerSpec, UnitLevel,
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
            AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition,
            ProgramDefinition, SelectorDefinition, UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    formula::{
        model::CombatElement,
        toughness::{BreakDamageDefinition, EnemyRank},
    },
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{
        ProgramStep, RuleEffectChancePolicy, RuleOperationTemplate, RuleSource, RuleValue,
        SourceClass, ValueExpr,
    },
};
use std::sync::Arc;

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

#[derive(Clone, Copy)]
pub(super) struct Inputs {
    pub(super) base_chance: Ratio,
    pub(super) hit_rate: Scalar,
    pub(super) resistance: Scalar,
    pub(super) control_resistance: Scalar,
    pub(super) break_effect: Scalar,
    pub(super) duration: u16,
    pub(super) application_damage: bool,
    pub(super) ordinary_maximum: Option<i64>,
}

impl Default for Inputs {
    fn default() -> Self {
        Self {
            base_chance: Ratio::from_scaled(500_000),
            hit_rate: Scalar::ONE,
            resistance: Scalar::ZERO,
            control_resistance: Scalar::ZERO,
            break_effect: Scalar::ZERO,
            duration: 1,
            application_damage: false,
            ordinary_maximum: Some(60),
        }
    }
}

fn damage_formula() -> BreakDamageDefinition {
    BreakDamageDefinition {
        attacker_level_multiplier: Scalar::checked_from_integer(100).unwrap(),
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

fn damage() -> HitOperationDefinition {
    HitOperationDefinition::Damage(
        OrdinaryDamageDefinition::new(
            Scalar::checked_from_integer(100).unwrap(),
            OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
        )
        .unwrap(),
    )
}

fn action(hits: Vec<ActionHitDefinition>) -> AbilityActionDefinition {
    AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap()
    .with_hits(hits)
    .unwrap()
}

fn catalog(input: Inputs) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new([0xa1; 32]);
    builder.add_selector(SelectorDefinition::new(id(1)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_selector(SelectorDefinition::new(id(3)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Allied, TargetPattern::Single).unwrap(),
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
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Control,
        DispelCategory::CleanseableControl,
        1,
        Some(ValueExpr::Literal(RuleValue::Integer(i64::from(
            input.duration,
        )))),
        DurationClock::TargetTurnStart,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .unwrap()
    .with_teardown(EffectTeardownPolicy::PersistByScope)
    .with_entanglement(damage_formula())
    .unwrap();
    builder.add_effect(EffectDefinition::new(id(1), vec![], vec![]).with_runtime_template(runtime));
    builder.add_program(
        ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![id(1)], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                selector: id(2),
                effect: id(1),
                stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                chance: RuleEffectChancePolicy::Resistible,
                base_chance: Some(ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                    input.base_chance.scaled(),
                )))),
                rng_purpose: None,
            }),
        ]),
    );
    builder.add_ability(
        AbilityDefinition::new(id(1), id(1), id(1), vec![id(1)])
            .with_action(action(vec![ActionHitDefinition::new(
                if input.application_damage {
                    vec![damage()]
                } else {
                    vec![]
                },
            )]))
            .with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, id(1)).unwrap(),
            ]),
    );
    for raw in 2..=4 {
        builder.add_program(ProgramDefinition::new(
            id(raw),
            vec![],
            vec![id(1)],
            vec![],
            vec![],
        ));
        let hits = match raw {
            2 => (0..7)
                .map(|_| ActionHitDefinition::new(vec![damage()]))
                .collect(),
            3 => vec![ActionHitDefinition::new(vec![
                HitOperationDefinition::RemoveEffects(
                    EffectRemovalDefinition::negative(1).unwrap(),
                ),
            ])],
            _ => vec![ActionHitDefinition::new(vec![])],
        };
        builder.add_ability(
            AbilityDefinition::new(id(raw), id(raw), id(1), vec![]).with_action(action(hits)),
        );
    }
    builder.add_modifier_group(ModifierStackingGroup {
        id: id(1),
        aggregation: ModifierAggregation::Sum,
        comparator: None,
    });
    builder.add_modifier(ModifierDefinition {
        id: id(1),
        stat: StatKind::ControlResistance,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::Stat,
        value: ValueExpr::Literal(RuleValue::Scalar(input.control_resistance)),
        stacking_group: id(1),
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Flat,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::new([]),
    });
    builder.add_program(ProgramDefinition::new(
        id(5),
        vec![],
        vec![id(3)],
        vec![],
        vec![],
    ));
    let lethal = HitOperationDefinition::Damage(
        OrdinaryDamageDefinition::new(
            Scalar::checked_from_integer(40_000).unwrap(),
            OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
        )
        .unwrap(),
    );
    builder.add_ability(
        AbilityDefinition::new(id(5), id(5), id(3), vec![])
            .with_action(action(vec![ActionHitDefinition::new(vec![lethal])])),
    );
    builder.add_unit(UnitDefinition::new(
        id(1),
        (1..=5).map(id).collect(),
        vec![],
    ));
    builder.add_unit(UnitDefinition::new(id(2), vec![id(4)], vec![]));
    builder.add_enemy(EnemyDefinition::new(id(1), id(2), vec![id(4)]));
    builder.add_encounter(EncounterDefinition::new(id(1), vec![id(1)], vec![]));
    builder.build().unwrap()
}

fn spec(input: Inputs) -> BattleSpec {
    let mut participants = Vec::new();
    for (index, speed) in [(0, 200_000_000), (1, 150_000_000), (2, 100_000_000)] {
        let player = index < 2;
        let combatant = ResolvedCombatantSpec::new(
            id(if player { 1 } else { 2 }),
            UnitLevel::new(80).unwrap(),
            Hp::new(10_000).unwrap(),
            Speed::from_scaled(speed).unwrap(),
            ResolvedDefinitionBindings::new(
                if player {
                    (1..=5).map(id).collect()
                } else {
                    vec![id(4)]
                },
                vec![],
                if player { vec![] } else { vec![id(1)] },
            )
            .unwrap(),
            CombatantSpecDigest::new([index + 1; 32]).unwrap(),
        )
        .unwrap()
        .with_base_effect_stats(
            if player { input.hit_rate } else { Scalar::ZERO },
            if player {
                Scalar::ZERO
            } else {
                input.resistance
            },
        )
        .with_build_bonuses(ResolvedBuildBonuses::new(
            Scalar::ZERO,
            Scalar::ZERO,
            if player {
                input.break_effect
            } else {
                Scalar::ZERO
            },
            Scalar::ZERO,
            Scalar::ZERO,
            [Scalar::ZERO; 7],
        ));
        let combatant = if player {
            combatant
        } else {
            let mut layers = vec![
                ToughnessLayerSpec::ordinary(1, RawToughness::new(1000).unwrap())
                    .unwrap()
                    .with_kind(ToughnessLayerKind::ExoToughness),
            ];
            if let Some(maximum) = input.ordinary_maximum {
                layers.push(
                    ToughnessLayerSpec::ordinary(2, RawToughness::new(maximum).unwrap()).unwrap(),
                );
            }
            combatant
                .with_sources(vec![RuleSource::new(
                    id(501),
                    SourceClass::Synthetic,
                    vec![],
                    [0xa0; 32],
                )])
                .unwrap()
                .with_modifier_bindings(vec![ResolvedModifierBinding::new(id(1), id(501))])
                .unwrap()
                .with_toughness(EnemyRank::Normal, vec![CombatElement::Quantum], layers)
                .unwrap()
        };
        participants.push(ParticipantSpec::new(
            if player {
                TeamSide::Player
            } else {
                TeamSide::Enemy
            },
            FormationIndex::new(if player { index } else { 0 }).unwrap(),
            if player {
                ParticipantSource::Player
            } else {
                ParticipantSource::EncounterEnemy(id(1))
            },
            combatant,
        ));
    }
    BattleSpec::new(
        AssemblyDigest::new([0xa2; 32]).unwrap(),
        id(1),
        participants,
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap()
}

pub(super) fn battle(input: Inputs, seed: u8) -> Battle {
    Battle::create(catalog(input), spec(input), BattleSeed::new([seed; 32])).unwrap()
}

pub(super) fn start(battle: &mut Battle) {
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    advance_boundary_if_offered(battle);
}

pub(super) fn play(battle: &mut Battle, actor: u64, ability: u32) -> Vec<BattleEvent> {
    let command = battle.decision().unwrap().legal_commands().iter().find(|command| matches!(command,
        Command::UseAbility { actor: offered_actor, ability: offered_ability, primary_target: Some(target), .. }
        if offered_actor.get() == actor && offered_ability.get() == ability && target.get() == if actor < 3 && ability != 5 { 3 } else { 1 }))
        .unwrap().clone();
    battle.apply(command).unwrap().events().to_vec()
}
