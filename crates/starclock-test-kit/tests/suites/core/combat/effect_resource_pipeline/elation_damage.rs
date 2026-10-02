//! Real accepted-command coverage of the explicit native Elation operation.
#[path = "damage_semantics.rs"]
mod damage_semantics;
#[path = "elation_rule_ir.rs"]
mod elation_rule_ir;
use super::{catalog, combatant, definition};
use crate::combat_decision::advance_boundary_if_offered;
use starclock_combat::{
    AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattlePhase, BattleSeed, BattleSpec,
    Command, ConcedePolicy, DamageEventData, DecisionId, DispelCategory, DurationClock,
    EffectApplicationDefinition, EffectCategory, EffectChancePolicy, EffectDamageGuard,
    EffectEventData, EffectRuntimeDefinition, EffectStackPolicy, EffectTickPhase, Energy,
    FormationIndex, LifeState, ParticipantSource, ParticipantSpec, Ratio, ResolvedBuildBonuses,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, ResolvedModifierBinding, Scalar, StatValue,
    TEAM_DEFEAT_GUARDED_SIGNAL, TeamResourceSpec, TeamSide,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HitCritPolicy, HitOperationDefinition, HitTargetGroup, OrdinaryDamageDefinition,
            OrdinaryDamageMultipliers, ShieldDefinition, TargetInvalidationPolicy, TargetPattern,
            TargetRelation, UnitTargetSelector, elation::ElationDamageDefinition,
        },
        builder::CombatCatalogBuilder,
        definition::{AbilityDefinition, EffectDefinition, ProgramDefinition, SelectorDefinition},
    },
    formula::{
        model::{CombatElement, DamageClass, ResistanceInput},
        shield::ShieldAbsorptionPolicy,
    },
    modifier::model::{
        FormulaPurpose, FormulaStage, FormulaSubject, ModifierAggregation, ModifierDefinition,
        ModifierFilter, ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{RuleSource, RuleValue, SourceClass, ValueExpr},
};
use std::sync::Arc;

struct Contribution {
    side: TeamSide,
    stat: StatKind,
    stage: FormulaStage,
    purpose: FormulaPurpose,
    value: i64,
    filters: Vec<ModifierFilter>,
}
impl Contribution {
    fn stat(side: TeamSide, stat: StatKind, value: i64) -> Self {
        Self {
            side,
            stat,
            stage: FormulaStage::Flat,
            purpose: FormulaPurpose::Stat,
            value,
            filters: vec![],
        }
    }
    fn factor(side: TeamSide, stage: FormulaStage, value: i64) -> Self {
        Self {
            side,
            stat: StatKind::Atk,
            stage,
            purpose: FormulaPurpose::ElationDamage,
            value,
            filters: vec![ModifierFilter::FormulaSubject(
                if side == TeamSide::Player {
                    FormulaSubject::Source
                } else {
                    FormulaSubject::Target
                },
            )],
        }
    }
}
struct Scenario {
    hits: Vec<ActionHitDefinition>,
    contributions: Vec<Contribution>,
    enemies: u8,
    target_defense: i64,
    crit_rate: i64,
    element_boost: i64,
}
impl Scenario {
    fn new(hits: Vec<ActionHitDefinition>) -> Self {
        Self {
            hits,
            contributions: vec![],
            enemies: 1,
            target_defense: 0,
            crit_rate: 0,
            element_boost: 0,
        }
    }
}
fn elation(base: i64) -> HitOperationDefinition {
    HitOperationDefinition::ElationDamage(
        ElationDamageDefinition::new(
            Scalar::from_scaled(base),
            Ratio::ONE,
            Ratio::ONE,
            Ratio::ZERO,
            ResistanceInput {
                target_resistance: Ratio::ZERO,
                penetration: Ratio::ZERO,
                minimum: Ratio::from_scaled(-1_000_000),
                maximum: Ratio::from_scaled(900_000),
            },
            Ratio::ONE,
            CombatElement::Fire,
        )
        .unwrap(),
    )
}
fn ordinary_elation(base: i64) -> HitOperationDefinition {
    HitOperationDefinition::Damage(
        OrdinaryDamageDefinition::new(
            Scalar::from_scaled(base),
            OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
        )
        .unwrap()
        .with_class(DamageClass::Elation),
    )
}
fn hit(operations: Vec<HitOperationDefinition>, policy: HitCritPolicy) -> ActionHitDefinition {
    ActionHitDefinition::new(operations).with_profile(
        HitTargetGroup::Selected,
        Ratio::ONE,
        Ratio::ONE,
        policy,
    )
}
fn apply_effect(raw: u32) -> HitOperationDefinition {
    HitOperationDefinition::ApplyEffect(
        EffectApplicationDefinition::new(definition(raw), EffectChancePolicy::Guaranteed, 1)
            .unwrap(),
    )
}
fn fixture(scenario: &Scenario) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::from_catalog(&catalog(), [0xe1; 32]);
    builder.add_selector(SelectorDefinition::new(definition(3)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::All).unwrap(),
    ));
    builder.add_program(ProgramDefinition::new(
        definition(3),
        vec![],
        vec![definition(3)],
        vec![definition(10), definition(11), definition(12)],
        vec![],
    ));
    for (index, row) in scenario.contributions.iter().enumerate() {
        let raw = 100 + u32::try_from(index).unwrap();
        builder.add_modifier_group(ModifierStackingGroup {
            id: definition(raw),
            aggregation: if row.stage == FormulaStage::DamageFinalMultiply {
                ModifierAggregation::Product
            } else {
                ModifierAggregation::Sum
            },
            comparator: None,
        });
        builder.add_modifier(ModifierDefinition {
            id: definition(raw),
            stat: row.stat,
            stage: row.stage,
            purpose: row.purpose,
            value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(row.value))),
            stacking_group: definition(raw),
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: row.stage,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: row.filters.clone().into_boxed_slice(),
        });
    }
    builder.add_modifier_group(ModifierStackingGroup {
        id: definition(99),
        aggregation: ModifierAggregation::Sum,
        comparator: None,
    });
    builder.add_modifier(ModifierDefinition {
        id: definition(99),
        stat: StatKind::Elation,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::Stat,
        value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(500_000))),
        stacking_group: definition(99),
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Flat,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::default(),
    });
    for raw in [10, 11, 12] {
        let runtime = EffectRuntimeDefinition::new(
            EffectCategory::Buff,
            DispelCategory::DispellableBuff,
            1,
            if raw == 12 { Some(1) } else { None },
            if raw == 12 {
                DurationClock::TargetTurnEnd
            } else {
                DurationClock::Permanent
            },
            EffectTickPhase::None,
            EffectStackPolicy::Replace,
        )
        .unwrap();
        let runtime = if raw == 10 {
            runtime.with_hp_floor(Ratio::from_scaled(250_000)).unwrap()
        } else if raw == 11 {
            runtime.with_damage_guard(EffectDamageGuard::TeamDefeatOnce)
        } else {
            runtime
        };
        builder.add_effect(
            EffectDefinition::new(
                definition(raw),
                vec![],
                if raw == 12 {
                    vec![definition(99)]
                } else {
                    vec![]
                },
            )
            .with_runtime(runtime),
        );
    }
    let action = AbilityActionDefinition::new(
        AbilityKind::Basic,
        u16::try_from(scenario.hits.len()).unwrap(),
        TargetInvalidationPolicy::KeepIfPresent,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap()
    .with_hits(scenario.hits.clone())
    .unwrap();
    assert!(
        builder.replace_ability(
            AbilityDefinition::new(
                definition(1),
                definition(3),
                definition(3),
                vec![definition(10), definition(11), definition(12)]
            )
            .with_action(action)
        )
    );
    builder.build().unwrap()
}
fn bound(
    base: ResolvedCombatantSpec,
    side: TeamSide,
    scenario: &Scenario,
) -> ResolvedCombatantSpec {
    let modifiers: Vec<ResolvedModifierBinding> = scenario
        .contributions
        .iter()
        .enumerate()
        .filter(|(_, row)| row.side == side)
        .map(|(index, _)| {
            ResolvedModifierBinding::new(
                definition(100 + u32::try_from(index).unwrap()),
                definition(if side == TeamSide::Player { 10 } else { 11 }),
            )
        })
        .collect();
    ResolvedCombatantSpec::new(
        base.form(),
        base.level(),
        base.maximum_hp(),
        base.speed(),
        ResolvedDefinitionBindings::new(
            base.abilities().to_vec(),
            base.rule_bundles().to_vec(),
            modifiers
                .iter()
                .map(|binding| binding.definition())
                .collect(),
        )
        .unwrap(),
        base.digest(),
    )
    .unwrap()
    .with_base_attack_defense(base.base_attack(), base.base_defense())
    .with_build_bonuses(base.build_bonuses())
    .with_sources(vec![RuleSource::new(
        definition(if side == TeamSide::Player { 10 } else { 11 }),
        SourceClass::Ability,
        vec![],
        [0xe2; 32],
    )])
    .unwrap()
    .with_modifier_bindings(modifiers)
    .unwrap()
}
fn battle(catalog: Arc<CombatCatalog>, scenario: &Scenario) -> Battle {
    let bonuses = ResolvedBuildBonuses::new(
        // Normalize the core player's 5% / 50% neutral CRIT inputs explicitly.
        Scalar::from_scaled(scenario.crit_rate - 50_000),
        Scalar::ZERO,
        Scalar::ZERO,
        Scalar::ZERO,
        Scalar::ZERO,
        [Scalar::from_scaled(scenario.element_boost); 7],
    );
    let player = bound(
        combatant(1, 1, 200_000_000, 1).with_build_bonuses(bonuses),
        TeamSide::Player,
        scenario,
    );
    let mut participants = vec![ParticipantSpec::new(
        TeamSide::Player,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::Player,
        player,
    )];
    for index in 0..scenario.enemies {
        let enemy = bound(
            combatant(2, 2, 101_000_000, 2).with_base_attack_defense(
                StatValue::from_scaled(0).unwrap(),
                StatValue::from_scaled(scenario.target_defense).unwrap(),
            ),
            TeamSide::Enemy,
            scenario,
        );
        participants.push(ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(index).unwrap(),
            ParticipantSource::EncounterEnemy(definition(1)),
            enemy,
        ));
    }
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xe3; 32]).unwrap(),
        definition(1),
        participants,
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog, spec, BattleSeed::new([0xe4; 32])).unwrap()
}
fn apply(running: &mut Battle, fresh: &mut Battle, command: Command) -> Vec<BattleEvent> {
    let a = running.apply(command.clone()).unwrap();
    let b = fresh.apply(command).unwrap();
    assert_eq!(a.fault(), b.fault());
    assert!(a.fault().is_none(), "{:?}", a.fault());
    assert_eq!(a.events(), b.events());
    assert_eq!(running.state_hash(), fresh.state_hash());
    assert_eq!(
        running.view().rng_draw_count(),
        fresh.view().rng_draw_count()
    );
    a.events().to_vec()
}
fn start(running: &mut Battle, fresh: &mut Battle) {
    let before = running.state_hash();
    let draws = running.view().rng_draw_count();
    assert!(
        running
            .apply(Command::StartBattle {
                decision: DecisionId::new(900).unwrap()
            })
            .is_err()
    );
    assert_eq!(running.state_hash(), before);
    assert_eq!(running.view().rng_draw_count(), draws);
    apply(
        running,
        fresh,
        Command::StartBattle {
            decision: running.decision().unwrap().id(),
        },
    );
    advance_boundary_if_offered(running);
    advance_boundary_if_offered(fresh);
}
fn attack(running: &mut Battle, fresh: &mut Battle) -> Vec<BattleEvent> {
    let command = running
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|c| matches!(c, Command::UseAbility { ability, .. } if ability.get() == 1))
        .unwrap()
        .clone();
    apply(running, fresh, command)
}
fn damages(events: &[BattleEvent]) -> Vec<DamageEventData> {
    events
        .iter()
        .filter_map(|e| {
            if let BattleEventKind::Damage(d) = e.kind() {
                Some(*d)
            } else {
                None
            }
        })
        .collect()
}
fn execute(scenario: &Scenario) -> (Battle, Vec<BattleEvent>) {
    let catalog = fixture(scenario);
    let mut running = battle(Arc::clone(&catalog), scenario);
    let mut fresh = battle(catalog, scenario);
    start(&mut running, &mut fresh);
    let events = attack(&mut running, &mut fresh);
    (running, events)
}

#[test]
fn dedicated_operation_excludes_ordinary_boost_weaken_and_build_element_bonus() {
    let mut scenario = Scenario::new(vec![hit(
        vec![elation(100_900_000), ordinary_elation(100_900_000)],
        HitCritPolicy::Never,
    )]);
    scenario.element_boost = 9_000_000;
    scenario.contributions = vec![
        Contribution::stat(TeamSide::Player, StatKind::Elation, 500_000),
        Contribution::factor(TeamSide::Player, FormulaStage::DamageBoost, 1_000_000),
        Contribution::factor(TeamSide::Player, FormulaStage::Weaken, 750_000),
    ];
    let (battle, events) = execute(&scenario);
    let damage = damages(&events);
    assert_eq!(damage.len(), 2);
    assert_eq!(
        (damage[0].raw.scaled(), damage[0].calculated.get()),
        (151_350_000, 151)
    );
    assert_eq!(
        (damage[1].raw.scaled(), damage[1].calculated.get()),
        (50_450_000, 50)
    );
    assert_eq!(damage[0].class, DamageClass::Elation);
    assert_eq!(damage[0].element, Some(CombatElement::Fire));
    assert!(damage[0].source_effect.is_none());
    assert_eq!(battle.view().rng_draw_count(), 0);
}

#[test]
fn dedicated_operation_resolves_hit_share_and_explicit_meter_merrymaking() {
    let operation = HitOperationDefinition::ElationDamage(
        ElationDamageDefinition::new(
            Scalar::from_scaled(100_900_000),
            Ratio::from_scaled(800_000),
            Ratio::from_scaled(2_000_000),
            Ratio::from_scaled(200_000),
            ResistanceInput {
                target_resistance: Ratio::from_scaled(200_000),
                penetration: Ratio::ZERO,
                minimum: Ratio::from_scaled(-1_000_000),
                maximum: Ratio::from_scaled(900_000),
            },
            Ratio::from_scaled(900_000),
            CombatElement::Fire,
        )
        .unwrap(),
    );
    let mut scenario = Scenario::new(vec![
        hit(vec![operation], HitCritPolicy::Never).with_profile(
            HitTargetGroup::Selected,
            Ratio::from_scaled(500_000),
            Ratio::ONE,
            HitCritPolicy::Never,
        ),
    ]);
    scenario.contributions.push(Contribution::stat(
        TeamSide::Player,
        StatKind::Elation,
        500_000,
    ));
    let (_, events) = execute(&scenario);
    let damage = damages(&events);
    assert_eq!(
        (damage[0].raw.scaled(), damage[0].calculated.get()),
        (104_613_120, 104)
    );
}

#[test]
fn native_target_and_source_stage_projection_retains_direction_element_and_final_floor() {
    let mut scenario = Scenario::new(vec![hit(vec![elation(100_900_000)], HitCritPolicy::Never)]);
    scenario.target_defense = 1_000_000_000;
    scenario.contributions = vec![
        Contribution::factor(TeamSide::Player, FormulaStage::Flat, 900_000),
        Contribution::factor(TeamSide::Player, FormulaStage::Crit, 100_000),
        Contribution::factor(TeamSide::Enemy, FormulaStage::Defense, 250_000),
        Contribution::factor(TeamSide::Player, FormulaStage::Resistance, 100_000),
        Contribution::factor(TeamSide::Enemy, FormulaStage::Resistance, -200_000),
        Contribution::factor(TeamSide::Enemy, FormulaStage::Vulnerability, 200_000),
        Contribution::factor(TeamSide::Enemy, FormulaStage::Mitigation, 250_000),
        Contribution::factor(TeamSide::Enemy, FormulaStage::Broken, 100_000),
        Contribution::factor(
            TeamSide::Player,
            FormulaStage::DamageFinalMultiply,
            1_500_000,
        ),
        // A source-only incoming contribution must not act as a target debuff.
        Contribution::factor(TeamSide::Enemy, FormulaStage::Defense, 9_000_000),
        Contribution::factor(TeamSide::Enemy, FormulaStage::Vulnerability, 9_000_000),
    ];
    scenario.contributions[9].filters =
        vec![ModifierFilter::FormulaSubject(FormulaSubject::Source)];
    scenario.contributions[10]
        .filters
        .push(ModifierFilter::Element(CombatElement::Ice as u8));
    scenario.contributions[5]
        .filters
        .push(ModifierFilter::Element(CombatElement::Fire as u8));
    let (_, events) = execute(&scenario);
    let d = damages(&events)[0];
    // 101.8 * 1.1 * .75 * .9 * 1.2 * .75 * 1.1 * 1.5, floor only at the end.
    assert_eq!(d.raw.scaled(), 112_245_952);
    assert_eq!(d.calculated.get(), 112);
}

#[test]
fn dedicated_final_multiplier_and_override_use_unfloored_raw() {
    for (stage, value, raw, applied) in [
        (FormulaStage::DamageFinalMultiply, 1_500_000, 2_850_000, 2),
        (FormulaStage::DamageFinalMultiply, 0, 0, 0),
        (FormulaStage::DamageOverride, 2_900_000, 2_900_000, 2),
    ] {
        let mut scenario = Scenario::new(vec![hit(vec![elation(1_900_000)], HitCritPolicy::Never)]);
        scenario
            .contributions
            .push(Contribution::factor(TeamSide::Player, stage, value));
        let (_, events) = execute(&scenario);
        let d = damages(&events)[0];
        assert_eq!((d.raw.scaled(), d.calculated.get()), (raw, applied));
    }
}

#[test]
fn mixed_operations_share_existing_hit_crit_samples_without_extra_rng() {
    for (policy, rate, draws) in [
        (HitCritPolicy::Never, 500_000, 0),
        (HitCritPolicy::PerTarget, 0, 0),
        (HitCritPolicy::PerTarget, 1_000_000, 0),
        (HitCritPolicy::PerTarget, 500_000, 2),
        (HitCritPolicy::Shared, 500_000, 1),
    ] {
        let mut scenario = Scenario::new(vec![hit(
            vec![ordinary_elation(10_000_000), elation(10_000_000)],
            policy,
        )]);
        scenario.enemies = 2;
        scenario.crit_rate = rate;
        let (battle, events) = execute(&scenario);
        let d = damages(&events);
        assert_eq!(d.len(), 4);
        assert_eq!(d[0].target, d[2].target);
        assert_eq!(d[1].target, d[3].target);
        assert_eq!(d[0].calculated, d[2].calculated);
        assert_eq!(d[1].calculated, d[3].calculated);
        if policy == HitCritPolicy::Shared {
            assert_eq!(d[0].calculated, d[1].calculated);
        }
        if policy == HitCritPolicy::Never || rate == 0 {
            assert_eq!(d[0].calculated.get(), 10);
        }
        if policy != HitCritPolicy::Never && rate == 1_000_000 {
            assert_eq!(d[0].calculated.get(), 15);
        }
        assert_eq!(battle.view().rng_draw_count(), draws);
    }
}

#[test]
fn guaranteed_below_hp_uses_current_hp_strict_threshold_without_rng() {
    let scenario = Scenario::new(vec![
        hit(vec![elation(400_000_000)], HitCritPolicy::Never),
        hit(
            vec![elation(100_000_000)],
            HitCritPolicy::GuaranteedBelowHpRatio(Ratio::from_scaled(600_000)),
        ),
        hit(
            vec![elation(100_000_000)],
            HitCritPolicy::GuaranteedBelowHpRatio(Ratio::from_scaled(600_000)),
        ),
    ]);
    let (battle, events) = execute(&scenario);
    assert_eq!(
        damages(&events)
            .iter()
            .map(|d| d.calculated.get())
            .collect::<Vec<_>>(),
        [400, 100, 150]
    );
    assert_eq!(battle.view().rng_draw_count(), 0);
}

#[test]
fn dedicated_damage_uses_shared_shields_hp_floor_team_guard_and_defeat() {
    let shield = HitOperationDefinition::Shield(
        ShieldDefinition::new(
            Scalar::from_scaled(100_000_000),
            Ratio::ZERO,
            ShieldAbsorptionPolicy::ConcurrentLargest,
        )
        .unwrap(),
    );
    let scenario = Scenario::new(vec![hit(
        vec![
            shield,
            elation(150_000_000),
            apply_effect(10),
            elation(2_000_000_000),
        ],
        HitCritPolicy::Never,
    )]);
    let (battle, events) = execute(&scenario);
    let d = damages(&events);
    assert_eq!(
        (d[0].absorbed.get(), d[0].applied.get(), d[0].hp_after.get()),
        (100, 50, 950)
    );
    assert_eq!(
        (
            d[1].calculated.get(),
            d[1].applied.get(),
            d[1].hp_after.get()
        ),
        (700, 700, 250)
    );
    assert!(
        battle
            .view()
            .units_by_id()
            .all(|u| u.life() == LifeState::Alive)
    );
    let scenario = Scenario::new(vec![hit(
        vec![
            apply_effect(11),
            elation(2_000_000_000),
            elation(2_000_000_000),
        ],
        HitCritPolicy::Never,
    )]);
    let (battle, events) = execute(&scenario);
    let d = damages(&events);
    assert_eq!((d[0].applied.get(), d[0].hp_after.get()), (999, 1));
    assert_eq!((d[1].applied.get(), d[1].hp_after.get()), (1, 0));
    assert!(events.iter().any(|e| matches!(e.kind(), BattleEventKind::RuleSignal(s) if s.code == TEAM_DEFEAT_GUARDED_SIGNAL)));
    assert!(
        battle
            .view()
            .units_by_id()
            .any(|u| u.life() == LifeState::Defeated)
    );
}

#[test]
fn dedicated_damage_reads_effect_property_after_commit_and_expiry() {
    let scenario = Scenario::new(vec![
        hit(vec![elation(100_000_000)], HitCritPolicy::Never),
        hit(vec![apply_effect(12)], HitCritPolicy::Never).with_profile(
            HitTargetGroup::SelfTarget,
            Ratio::ONE,
            Ratio::ONE,
            HitCritPolicy::Never,
        ),
        hit(vec![elation(100_000_000)], HitCritPolicy::Never),
    ]);
    let catalog = fixture(&scenario);
    let mut running = battle(Arc::clone(&catalog), &scenario);
    let mut fresh = battle(catalog, &scenario);
    start(&mut running, &mut fresh);
    let events = attack(&mut running, &mut fresh);
    assert_eq!(
        damages(&events)
            .iter()
            .map(|d| d.calculated.get())
            .collect::<Vec<_>>(),
        [100, 150]
    );
    let command = running.advance_command().unwrap();
    let events = apply(&mut running, &mut fresh, command);
    assert!(events.iter().any(|e| matches!(e.kind(), BattleEventKind::Effect(EffectEventData::Removed { definition, .. }) if definition.get() == 12)));
    assert!(
        running
            .view()
            .modifier_instances_by_id()
            .all(|m| m.definition().get() != 99)
    );
    assert_eq!(running.view().rng_draw_count(), 0);
}

#[test]
fn dedicated_critical_stat_queries_use_authored_element_filters() {
    let mut scenario = Scenario::new(vec![hit(
        vec![elation(100_000_000)],
        HitCritPolicy::PerTarget,
    )]);
    scenario.contributions = vec![
        Contribution::stat(TeamSide::Player, StatKind::CritRate, 1_000_000),
        Contribution::stat(TeamSide::Player, StatKind::CritDamage, 200_000),
        Contribution::stat(TeamSide::Player, StatKind::CritDamage, 9_000_000),
    ];
    scenario.contributions[0]
        .filters
        .push(ModifierFilter::Element(CombatElement::Fire as u8));
    scenario.contributions[1]
        .filters
        .push(ModifierFilter::Element(CombatElement::Fire as u8));
    scenario.contributions[2]
        .filters
        .push(ModifierFilter::Element(CombatElement::Ice as u8));
    let (battle, events) = execute(&scenario);
    assert_eq!(damages(&events)[0].calculated.get(), 170);
    assert_eq!(battle.view().rng_draw_count(), 0);
}

#[test]
fn dedicated_operation_queries_effective_target_def_instead_of_only_base_def() {
    let mut scenario = Scenario::new(vec![hit(vec![elation(100_000_000)], HitCritPolicy::Never)]);
    scenario.target_defense = 1_000_000_000;
    scenario.contributions.push(Contribution::stat(
        TeamSide::Enemy,
        StatKind::Def,
        1_000_000_000,
    ));
    let (_, events) = execute(&scenario);
    let d = damages(&events)[0];
    assert_eq!((d.raw.scaled(), d.calculated.get()), (33_333_300, 33));
}

#[test]
fn dedicated_numeric_overflow_enters_reproducible_fault_without_damage_mutation() {
    let operation = HitOperationDefinition::ElationDamage(
        ElationDamageDefinition::new(
            Scalar::MAX,
            Ratio::ONE,
            Ratio::from_scaled(2_000_000),
            Ratio::ZERO,
            ResistanceInput {
                target_resistance: Ratio::ZERO,
                penetration: Ratio::ZERO,
                minimum: Ratio::from_scaled(-1_000_000),
                maximum: Ratio::ONE,
            },
            Ratio::ONE,
            CombatElement::Fire,
        )
        .unwrap(),
    );
    let scenario = Scenario::new(vec![hit(vec![operation], HitCritPolicy::Never)]);
    let catalog = fixture(&scenario);
    let mut running = battle(Arc::clone(&catalog), &scenario);
    let mut fresh = battle(catalog, &scenario);
    start(&mut running, &mut fresh);
    let command = running
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|c| matches!(c, Command::UseAbility { .. }))
        .unwrap()
        .clone();
    let a = running.apply(command.clone()).unwrap();
    let b = fresh.apply(command).unwrap();
    assert!(a.fault().is_some());
    assert_eq!(a.fault(), b.fault());
    assert_eq!(a.events(), b.events());
    assert!(damages(a.events()).is_empty());
    assert_eq!(running.view().phase(), BattlePhase::Faulted);
    assert_eq!(running.state_hash(), fresh.state_hash());
    assert_eq!(running.view().rng_draw_count(), 0);
    assert!(
        running
            .view()
            .units_by_id()
            .all(|u| u.current_hp().get() == 1000)
    );
}
