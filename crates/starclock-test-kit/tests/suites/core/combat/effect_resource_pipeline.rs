use std::sync::Arc;

use starclock_combat::{
    AssemblyDigest, Battle, BattleEventKind, BattleSeed, BattleSpec, CombatantSpecDigest, Command,
    ConcedePolicy, DamageKind, DispelCategory, DotDefinition, DotDetonationDefinition,
    DurationClock, EffectApplicationDefinition, EffectCategory, EffectChancePolicy,
    EffectDefinitionId, EffectEventData, EffectRuntimeDefinition, EffectStackPolicy,
    EffectTickPhase, Energy, FormationIndex, Hp, ModifierDefinitionId, ModifierStackingGroupId,
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
            ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition, UnitDefinition,
        },
    },
    formula::model::CombatElement,
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{
        BattleRuleDefinition, BattleRuleScope, RuleSlotMutationDefinition, RuleSource, RuleValue,
        RuleValueKind, SlotResetPoint, SourceClass, StateSlotDef, StateSlotUpdateKind,
    },
};

fn definition<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn dot_damage(amount: i64) -> OrdinaryDamageDefinition {
    OrdinaryDamageDefinition::new(
        Scalar::checked_from_integer(amount).unwrap(),
        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
    )
    .unwrap()
}

fn action(operations: Vec<HitOperationDefinition>) -> AbilityActionDefinition {
    AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::KeepIfPresent,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap()
    .with_hits(vec![ActionHitDefinition::new(operations)])
    .unwrap()
}

fn catalog() -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new([0xa5; 32]);
    for (raw, relation) in [(1, TargetRelation::Opposing), (2, TargetRelation::Opposing)] {
        builder.add_selector(
            SelectorDefinition::new(definition(raw)).with_unit_targets(
                UnitTargetSelector::new(relation, TargetPattern::Single).unwrap(),
            ),
        );
        builder.add_program(ProgramDefinition::new(
            definition(raw),
            vec![],
            vec![definition(raw)],
            if raw == 1 {
                vec![definition(1)]
            } else {
                vec![]
            },
            vec![],
        ));
    }
    let runtime = EffectRuntimeDefinition::new(
        EffectCategory::Dot,
        DispelCategory::DispellableDebuff,
        5,
        Some(2),
        DurationClock::TargetTurnStart,
        EffectTickPhase::TurnStart,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .unwrap()
    .with_snapshot(starclock_combat::EffectSnapshotPolicy::OnApplication)
    .with_dot(DotDefinition::new(
        dot_damage(100),
        CombatElement::Lightning,
        None,
    ))
    .unwrap();
    builder.add_modifier_group(ModifierStackingGroup {
        id: ModifierStackingGroupId::new(1).unwrap(),
        aggregation: ModifierAggregation::Sum,
        comparator: None,
    });
    builder.add_modifier(ModifierDefinition {
        id: ModifierDefinitionId::new(1).unwrap(),
        stat: StatKind::Atk,
        stage: FormulaStage::Vulnerability,
        purpose: FormulaPurpose::Dot,
        value: starclock_combat::rule::model::ValueExpr::Slot(definition(2)),
        stacking_group: ModifierStackingGroupId::new(1).unwrap(),
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Vulnerability,
        snapshot: SnapshotPolicy::RecomputeOnStackChange,
        source_stack_slot: Some(definition(2)),
        filters: Box::new([]),
    });
    builder.add_effect(
        EffectDefinition::new(
            definition(1),
            vec![],
            vec![ModifierDefinitionId::new(1).unwrap()],
        )
        .with_runtime(runtime),
    );

    let slot = StateSlotDef::new(
        definition(1),
        RuleValueKind::Integer,
        BattleRuleScope::Turn,
        RuleValue::Integer(0),
    )
    .with_bounds(RuleValue::Integer(0), RuleValue::Integer(5))
    .with_reset_points(vec![SlotResetPoint::TurnStart]);
    let action_slot = StateSlotDef::new(
        definition(2),
        RuleValueKind::Integer,
        BattleRuleScope::Action,
        RuleValue::Integer(0),
    )
    .with_bounds(RuleValue::Integer(0), RuleValue::Integer(5))
    .with_reset_points(vec![SlotResetPoint::ActionEnd]);
    let source = RuleSource::new(definition(1), SourceClass::Ability, vec![], [0x31; 32]);
    builder.add_rule(
        RuleDefinition::new(definition(1), vec![], vec![]).with_runtime(BattleRuleDefinition::new(
            source,
            vec![slot, action_slot],
            vec![],
            None,
        )),
    );
    builder.add_rule_bundle(RuleBundle::new(definition(1), vec![definition(1)]));

    let effect = EffectApplicationDefinition::new(
        EffectDefinitionId::new(1).unwrap(),
        EffectChancePolicy::Guaranteed,
        1,
    )
    .unwrap();
    builder.add_ability(
        AbilityDefinition::new(
            definition(1),
            definition(1),
            definition(1),
            vec![definition(1)],
        )
        .with_action(action(vec![
            HitOperationDefinition::ModifyStateSlot(RuleSlotMutationDefinition {
                rule: definition(1),
                slot: definition(1),
                update: StateSlotUpdateKind::Add,
                value: RuleValue::Integer(1),
            }),
            HitOperationDefinition::ModifyStateSlot(RuleSlotMutationDefinition {
                rule: definition(1),
                slot: definition(2),
                update: StateSlotUpdateKind::Add,
                value: RuleValue::Integer(1),
            }),
            HitOperationDefinition::ApplyEffect(effect),
            HitOperationDefinition::ApplyEffect(effect),
            HitOperationDefinition::DetonateDots(
                DotDetonationDefinition::new(Ratio::from_scaled(750_000), None).unwrap(),
            ),
        ])),
    );
    builder.add_ability(
        AbilityDefinition::new(definition(2), definition(2), definition(2), vec![])
            .with_action(action(vec![])),
    );
    builder.add_unit(UnitDefinition::new(
        definition(1),
        vec![definition(1)],
        vec![definition(1)],
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
        ResolvedDefinitionBindings::new(
            vec![definition(ability)],
            if form == 1 {
                vec![definition(1)]
            } else {
                vec![]
            },
            vec![],
        )
        .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
}

fn battle() -> Battle {
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x41; 32]).unwrap(),
        definition(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 1, 200_000_000, 1),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(2, 2, 101_000_000, 2),
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog(), spec, BattleSeed::new([0x51; 32])).unwrap()
}

fn execute_probe(mut battle: Battle) -> (Battle, starclock_combat::Resolution) {
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    crate::combat_decision::pass_interrupt_if_offered(&mut battle);
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { .. }))
        .unwrap()
        .clone();
    let resolution = battle.apply(command).unwrap();
    (battle, resolution)
}

#[test]
fn kafka_style_detonation_retains_source_snapshot_duration_and_stacks() {
    let (battle, resolution) = execute_probe(battle());
    let damages = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some((event.cause(), *data)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(damages.len(), 2);
    assert_eq!(
        (damages[0].1.kind, damages[0].1.calculated.get()),
        (DamageKind::DotDetonation, 450)
    );
    assert_eq!(
        (damages[1].1.kind, damages[1].1.calculated.get()),
        (DamageKind::DotTick, 600)
    );
    let effect_id = damages[0].1.source_effect.unwrap();
    assert_eq!(damages[1].1.source_effect, Some(effect_id));
    assert_eq!(damages[0].0.applier(), damages[1].0.applier());
    assert_eq!(
        damages[0].0.source_definition(),
        damages[1].0.source_definition()
    );
    let effect = battle.view().effects_by_id().next().unwrap();
    assert_eq!(
        (effect.id(), effect.stacks(), effect.remaining()),
        (effect_id, 2, Some(1))
    );
    assert_eq!(
        effect.snapshot_policy(),
        starclock_combat::EffectSnapshotPolicy::OnApplication
    );
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .nth(1)
            .unwrap()
            .current_hp()
            .get(),
        0
    );
    assert_eq!(
        battle.view().rng_draw_count(),
        0,
        "guaranteed applications bypass RNG"
    );
    assert!(resolution.events().iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Effect(EffectEventData::Refreshed { .. })
    )));
    let slots = battle
        .view()
        .rule_instances_by_id()
        .next()
        .unwrap()
        .slots()
        .collect::<Vec<_>>();
    assert!(matches!(
        slots.as_slice(),
        [(_, RuleValue::Integer(1)), (_, RuleValue::Integer(0))]
    ));
}

#[test]
fn effect_execution_is_replay_deterministic() {
    let (_, first) = execute_probe(battle());
    let (_, second) = execute_probe(battle());
    assert_eq!(first.events(), second.events());
    assert_eq!(first.state_hash(), second.state_hash());
}

#[test]
fn chance_energy_and_aggro_goldens_use_checked_fixed_point() {
    let chance = starclock_combat::formula::effect::resistible_chance(
        Ratio::from_scaled(800_000),
        Ratio::from_scaled(500_000),
        Ratio::from_scaled(200_000),
        Ratio::from_scaled(100_000),
    )
    .unwrap();
    assert_eq!(
        (chance.pre_clamp.scaled(), chance.probability.millionths()),
        (864_000, 864_000)
    );
    let over_cap = starclock_combat::formula::effect::resistible_chance(
        Ratio::from_scaled(1_500_000),
        Ratio::from_scaled(500_000),
        Ratio::from_scaled(200_000),
        Ratio::from_scaled(100_000),
    )
    .unwrap();
    assert_eq!(
        (
            over_cap.pre_clamp.scaled(),
            over_cap.probability.millionths()
        ),
        (1_620_000, 1_000_000)
    );
    let reduced_freeze_resistance = starclock_combat::formula::effect::resistible_chance(
        Ratio::ONE,
        Ratio::ZERO,
        Ratio::ZERO,
        Ratio::from_scaled(-560_000),
    )
    .unwrap();
    assert_eq!(
        (
            reduced_freeze_resistance.pre_clamp.scaled(),
            reduced_freeze_resistance.probability.millionths()
        ),
        (1_560_000, 1_000_000)
    );
    assert_eq!(
        starclock_combat::formula::effect::energy_gain(
            Energy::from_scaled(30_000_000).unwrap(),
            Ratio::from_scaled(1_200_000),
            true,
        )
        .unwrap()
        .scaled(),
        36_000_000
    );
    let weights = starclock_combat::formula::effect::aggro_weights(&[
        (Scalar::checked_from_integer(4).unwrap(), Ratio::ZERO),
        (
            Scalar::checked_from_integer(5).unwrap(),
            Ratio::from_scaled(500_000),
        ),
        (
            Scalar::checked_from_integer(6).unwrap(),
            Ratio::from_scaled(-1_000_000),
        ),
    ])
    .unwrap();
    assert_eq!(weights, [4_000_000, 7_500_000, 0]);
    let mut first = starclock_combat::rng::engine::DeterministicRng::from_seed(
        starclock_combat::rng::types::RngSeed::new([0x91; 32]),
    );
    let selected = first
        .choose_weighted(
            starclock_combat::rng::types::DrawPurpose::AGGRO_TARGET,
            &weights,
        )
        .unwrap()
        .unwrap();
    let mut second = starclock_combat::rng::engine::DeterministicRng::from_seed(
        starclock_combat::rng::types::RngSeed::new([0x91; 32]),
    );
    assert_eq!(
        second
            .choose_weighted(
                starclock_combat::rng::types::DrawPurpose::AGGRO_TARGET,
                &weights,
            )
            .unwrap()
            .unwrap(),
        selected
    );
    assert_eq!(
        selected.range().sample().purpose(),
        starclock_combat::rng::types::DrawPurpose::AGGRO_TARGET
    );
    let draws = first.draw_count();
    assert!(
        first
            .choose_weighted(
                starclock_combat::rng::types::DrawPurpose::AGGRO_TARGET,
                &[0, 0],
            )
            .unwrap()
            .is_none()
    );
    assert_eq!(first.draw_count(), draws, "all-zero aggro consumes no RNG");
}
