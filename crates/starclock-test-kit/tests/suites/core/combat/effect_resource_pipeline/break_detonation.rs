//! Cross-store detonation through accepted commands and immutable source fixtures.

use super::{action, catalog, definition as definition_id, dot_damage};
use crate::combat_decision::{advance_boundary_if_offered, settle_ready_boundaries};
use starclock_combat::{
    AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattleSeed, BattleSpec, BreakDamageKind,
    CombatantSpecDigest, Command, ConcedePolicy, DamageAmount, DispelCategory, DotDefinition,
    DotDetonationDefinition, DotDetonationFilter, DotDetonationScope, DotDetonationSelection,
    DotFamily, DurationClock, EffectApplicationDefinition, EffectCategory, EffectChancePolicy,
    EffectEventData, EffectRuntimeDefinition, EffectStackPolicy, EffectTickPhase, FormationIndex,
    Hp, ParticipantSource, ParticipantSpec, Probability, Ratio, RawToughness,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, ResolvedModifierBinding, Rounding, Scalar,
    Speed, StatValue, TeamResourceSpec, TeamSide, ToughnessEventData, ToughnessLayerSpec,
    ToughnessReductionDefinition, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{AbilityProgramBinding, AbilityProgramTiming, HitOperationDefinition},
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EffectDefinition, ProgramDefinition, SelectorDefinition,
            UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    formula::{
        model::CombatElement,
        toughness::{BreakDamageDefinition, EnemyRank, ToughnessReductionContext},
    },
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rng::types::DrawPurpose,
    rule::model::{
        ProgramStep, RuleDotSelection, RuleOperationTemplate, RuleSource, RuleValue, SourceClass,
        ValueExpr,
    },
};
use starclock_replay::battle_event::encode_battle_event_payload;
use std::sync::Arc;

fn family(element: CombatElement) -> DotFamily {
    match element {
        CombatElement::Physical => DotFamily::Bleed,
        CombatElement::Lightning => DotFamily::Shock,
        CombatElement::Wind => DotFamily::WindShear,
        _ => DotFamily::Burn,
    }
}

fn build_catalog(
    element: CombatElement,
    definition: DotDetonationDefinition,
    ir: bool,
    ordinary: bool,
) -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::from_catalog(&catalog(), [0xb5; 32]);
    for raw in [10, 11] {
        let runtime = EffectRuntimeDefinition::new(
            EffectCategory::Dot,
            DispelCategory::DispellableDebuff,
            1,
            Some(3),
            DurationClock::TargetTurnEnd,
            EffectTickPhase::None,
            EffectStackPolicy::Refresh,
        )
        .unwrap()
        .with_dot(DotDefinition::new(
            dot_damage(i64::from(raw)),
            element,
            Some(definition_id(77)),
        ))
        .unwrap();
        builder.add_effect(
            EffectDefinition::new(definition_id(raw), vec![], vec![])
                .with_runtime(runtime)
                .with_dot_family(family(element)),
        );
    }
    builder.add_modifier_group(ModifierStackingGroup {
        id: definition_id(2),
        aggregation: ModifierAggregation::Product,
        comparator: None,
    });
    for (raw, factor) in [(2, 2), (3, 7)] {
        builder.add_modifier(ModifierDefinition {
            id: definition_id(raw),
            stat: StatKind::Atk,
            stage: FormulaStage::DamageFinalMultiply,
            purpose: FormulaPurpose::Break,
            value: ValueExpr::Literal(RuleValue::Scalar(
                Scalar::checked_from_integer(factor).unwrap(),
            )),
            stacking_group: definition_id(2),
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::DamageFinalMultiply,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        });
    }
    let break_damage = BreakDamageDefinition {
        attacker_level_multiplier: Scalar::from_scaled(1_250_000),
        ability_multiplier: Ratio::ONE,
        break_effect: Ratio::ZERO,
        break_damage_increase: Ratio::ZERO,
        defense_multiplier: Ratio::from_scaled(700_000),
        resistance_multiplier: Ratio::from_scaled(800_000),
        vulnerability_multiplier: Ratio::from_scaled(1_100_000),
        mitigation_multiplier: Ratio::from_scaled(900_000),
        unbroken_multiplier: Ratio::from_scaled(900_000),
    };
    let apply = |raw| {
        HitOperationDefinition::ApplyEffect(
            EffectApplicationDefinition::new(definition_id(raw), EffectChancePolicy::Guaranteed, 1)
                .unwrap(),
        )
    };
    let mut operations = Vec::new();
    if ordinary {
        operations.push(apply(10));
    }
    operations.push(HitOperationDefinition::ReduceToughness(
        ToughnessReductionDefinition {
            element,
            ignores_weakness: true,
            break_damage,
            break_effect_chance: Probability::ONE,
            reduction: ToughnessReductionContext {
                base: RawToughness::new(100).unwrap(),
                additive: RawToughness::new(0).unwrap(),
                reduction_increase: Ratio::ZERO,
                weakness_break_efficiency: Ratio::ZERO,
                weakness_break_efficiency_cap: Ratio::from_scaled(3_000_000),
                toughness_vulnerability: Ratio::ZERO,
                ability_multiplier: Ratio::ONE,
            },
        },
    ));
    if ordinary {
        operations.push(apply(11));
    }
    assert!(
        builder.replace_ability(
            AbilityDefinition::new(
                definition_id(1),
                definition_id(1),
                definition_id(1),
                vec![definition_id(10), definition_id(11)]
            )
            .with_action(action(operations))
        )
    );
    let mut ability =
        AbilityDefinition::new(definition_id(3), definition_id(2), definition_id(1), vec![]);
    if ir {
        builder.add_selector(
            SelectorDefinition::new(definition_id(3)).with_rule_units(
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
        builder.add_program(
            ProgramDefinition::new(
                definition_id(3),
                vec![],
                vec![definition_id(3)],
                vec![],
                vec![],
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::DetonateDot {
                    selector: definition_id(3),
                    fraction: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                        definition.fraction().scaled(),
                    ))),
                    required_tag: definition.required_tag(),
                    selection: RuleDotSelection::Filtered {
                        filter: definition.filter(),
                        selection: definition.selection(),
                        scope: definition.scope(),
                    },
                },
            )]),
        );
        ability = ability.with_action(action(vec![])).with_programs(vec![
            AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, definition_id(3)).unwrap(),
        ]);
    } else {
        ability = ability.with_action(action(vec![HitOperationDefinition::DetonateDots(
            definition,
        )]));
    }
    builder.add_ability(ability);
    builder.add_unit(UnitDefinition::new(
        definition_id(3),
        vec![definition_id(3)],
        vec![],
    ));
    builder.build().unwrap()
}

fn combatant(form: u32, ability: u32, speed: i64, modifier: Option<u32>) -> ResolvedCombatantSpec {
    let source = definition_id(100 + form);
    let mut spec = ResolvedCombatantSpec::new(
        definition_id(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000_000).unwrap(),
        Speed::from_scaled(speed).unwrap(),
        ResolvedDefinitionBindings::new(
            vec![definition_id(ability)],
            vec![],
            modifier.map(definition_id).into_iter().collect(),
        )
        .unwrap(),
        CombatantSpecDigest::new([u8::try_from(form).unwrap(); 32]).unwrap(),
    )
    .unwrap()
    .with_base_attack_defense(
        StatValue::from_scaled(100_000_000).unwrap(),
        StatValue::from_scaled(500_000_000).unwrap(),
    );
    if let Some(modifier) = modifier {
        spec = spec
            .with_sources(vec![RuleSource::new(
                source,
                SourceClass::Progression,
                vec![],
                [0x91; 32],
            )])
            .unwrap()
            .with_modifier_bindings(vec![ResolvedModifierBinding::new(
                definition_id(modifier),
                source,
            )])
            .unwrap();
    }
    spec
}

fn battle(
    element: CombatElement,
    definition: DotDetonationDefinition,
    ir: bool,
    ordinary: bool,
) -> Battle {
    let enemy = combatant(2, 2, 190_000_000, None)
        .with_toughness(
            EnemyRank::Elite,
            vec![],
            vec![ToughnessLayerSpec::ordinary(1, RawToughness::new(100).unwrap()).unwrap()],
        )
        .unwrap();
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xb5; 32]).unwrap(),
        definition_id(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 1, 200_000_000, Some(2)),
            ),
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(1).unwrap(),
                ParticipantSource::Player,
                combatant(3, 3, 180_000_000, Some(3)),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(definition_id(1)),
                enemy,
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(
        build_catalog(element, definition, ir, ordinary),
        spec,
        BattleSeed::new([0x51; 32]),
    )
    .unwrap()
}

fn act(battle: &mut Battle, ability: u32) -> Vec<BattleEvent> {
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(command,
        Command::UseAbility { ability: offered, .. } if offered.get() == ability)
        })
        .unwrap()
        .clone();
    let resolution = battle.apply(command).unwrap();
    assert!(resolution.fault().is_none());
    resolution.events().to_vec()
}

fn prepare(battle: &mut Battle) {
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    advance_boundary_if_offered(battle);
    act(battle, 1);
    settle_ready_boundaries(battle);
}

fn detonation(element: CombatElement) -> DotDetonationDefinition {
    DotDetonationDefinition::new(Ratio::from_scaled(2_500_000), None)
        .unwrap()
        .with_filter(DotDetonationFilter::family(family(element)))
        .with_scope(DotDetonationScope::OrdinaryAndBreakEffects)
}

#[test]
fn break_detonation_preserves_four_periodic_statuses_and_original_applier_formula() {
    for element in [
        CombatElement::Physical,
        CombatElement::Fire,
        CombatElement::Lightning,
        CombatElement::Wind,
    ] {
        for ir in [false, true] {
            let mut battle = battle(element, detonation(element), ir, false);
            prepare(&mut battle);
            let effect = battle.view().break_effects_by_id().next().unwrap();
            let before = (
                effect.id(),
                effect.remaining_turns(),
                effect.stacks(),
                effect.source_operation(),
                effect.plan(),
                effect.damage(),
            );
            let (applier, source) = (effect.applier(), effect.source_definition());
            let events = act(&mut battle, 3);
            let effect = battle.view().break_effects_by_id().next().unwrap();
            assert_eq!(
                before,
                (
                    effect.id(),
                    effect.remaining_turns(),
                    effect.stacks(),
                    effect.source_operation(),
                    effect.plan(),
                    effect.damage()
                )
            );
            let damage = events
                .iter()
                .find_map(|event| match event.kind() {
                    BattleEventKind::BreakDamage(data)
                        if data.kind == BreakDamageKind::EffectDetonation =>
                    {
                        assert_eq!(event.cause().applier(), Some(applier));
                        assert_eq!(event.cause().source_definition(), Some(source));
                        assert_ne!(event.cause().owner(), Some(applier));
                        Some(*data)
                    }
                    _ => None,
                })
                .unwrap();
            let detonated = events.iter().find(|event| matches!(event.kind(), BattleEventKind::Toughness(ToughnessEventData::BaseEffectDetonated { effect, .. }) if *effect == before.0)).unwrap();
            let encoded = encode_battle_event_payload(detonated).unwrap();
            assert_eq!(&encoded[encoded.len() - 8..], &2_500_000_i64.to_le_bytes());
            let ticks = settle_ready_boundaries(&mut battle);
            let tick = ticks
                .iter()
                .find_map(|event| match event.kind() {
                    BattleEventKind::BreakDamage(data) if data.kind == BreakDamageKind::Effect => {
                        Some(data)
                    }
                    _ => None,
                })
                .unwrap();
            assert_eq!(
                damage.raw,
                Ratio::from_scaled(2_500_000)
                    .checked_apply(tick.raw, Rounding::NearestTiesEven)
                    .unwrap()
            );
            assert_eq!(
                damage.calculated,
                DamageAmount::from_scalar(damage.raw, Rounding::Floor).unwrap()
            );
            if element == CombatElement::Fire {
                assert!(
                    damage.calculated.get() > tick.calculated.get() * 5 / 2,
                    "fraction must precede integral finalization"
                );
            }
            if element == CombatElement::Wind {
                assert_eq!(before.2, 3);
            }
            assert_eq!(battle.view().rng_draw_count(), 0);
        }
    }
}

fn detonated_ids(events: &[BattleEvent]) -> Vec<u64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Effect(EffectEventData::Detonated { effect, .. })
            | BattleEventKind::Toughness(ToughnessEventData::BaseEffectDetonated {
                effect, ..
            }) => Some(effect.get()),
            _ => None,
        })
        .collect()
}

#[test]
fn break_detonation_merges_stores_in_global_instance_order_and_reconstructs() {
    for ir in [false, true] {
        let run = || {
            let mut battle = battle(
                CombatElement::Fire,
                detonation(CombatElement::Fire),
                ir,
                true,
            );
            prepare(&mut battle);
            let events = act(&mut battle, 3);
            assert_eq!(detonated_ids(&events), [1, 2, 3]);
            assert_eq!(battle.view().rng_draw_count(), 0);
            let encoded = events
                .iter()
                .map(|event| encode_battle_event_payload(event).unwrap())
                .collect::<Vec<_>>();
            (events, encoded, battle.state_hash())
        };
        assert_eq!(run(), run());
    }
}

#[test]
fn break_detonation_scope_tags_exclusions_and_nonperiodic_controls_are_explicit() {
    let burn = detonation(CombatElement::Fire);
    for (definition, ordinary, expected) in [
        (
            burn.with_scope(DotDetonationScope::OrdinaryEffects),
            false,
            vec![],
        ),
        (
            DotDetonationDefinition::new(Ratio::ONE, Some(definition_id(77)))
                .unwrap()
                .with_scope(DotDetonationScope::OrdinaryAndBreakEffects),
            true,
            vec![1, 3],
        ),
        (
            burn.with_filter(
                DotDetonationFilter::family(DotFamily::Burn).excluding(definition_id(10)),
            ),
            true,
            vec![2, 3],
        ),
        (
            burn.with_filter(DotDetonationFilter::family(DotFamily::Bleed)),
            true,
            vec![],
        ),
    ] {
        for ir in [false, true] {
            let mut battle = battle(CombatElement::Fire, definition, ir, ordinary);
            prepare(&mut battle);
            assert_eq!(detonated_ids(&act(&mut battle, 3)), expected);
            assert_eq!(battle.view().rng_draw_count(), 0);
        }
    }
    for element in [
        CombatElement::Ice,
        CombatElement::Quantum,
        CombatElement::Imaginary,
    ] {
        let definition = DotDetonationDefinition::new(Ratio::ONE, None)
            .unwrap()
            .with_scope(DotDetonationScope::OrdinaryAndBreakEffects)
            .with_selection(DotDetonationSelection::RandomOne(
                DrawPurpose::new(301).unwrap(),
            ));
        for ir in [false, true] {
            let run = |scope| {
                let mut battle = battle(element, definition.with_scope(scope), ir, false);
                prepare(&mut battle);
                let effect = battle.view().break_effects_by_id().next().unwrap();
                let before = (
                    effect.id(),
                    effect.remaining_turns(),
                    effect.plan(),
                    effect.damage(),
                    effect.source_operation(),
                );
                let events = act(&mut battle, 3);
                assert!(detonated_ids(&events).is_empty());
                let after = battle.view().break_effects_by_id().next().unwrap();
                assert_eq!(
                    before,
                    (
                        after.id(),
                        after.remaining_turns(),
                        after.plan(),
                        after.damage(),
                        after.source_operation()
                    )
                );
                assert_eq!(battle.view().rng_draw_count(), 0);
                // Quantum stacks respond to the attack envelope itself. Changing
                // the candidate store must not alter that independent behavior.
                (after.stacks(), events, battle.state_hash())
            };
            assert_eq!(
                run(DotDetonationScope::OrdinaryEffects),
                run(DotDetonationScope::OrdinaryAndBreakEffects)
            );
        }
    }
}

#[test]
fn break_detonation_random_selection_uses_one_joint_pool_and_skips_singleton_draws() {
    let definition = detonation(CombatElement::Fire).with_selection(
        DotDetonationSelection::RandomOne(DrawPurpose::new(301).unwrap()),
    );
    for ir in [false, true] {
        let run = || {
            let mut battle = battle(CombatElement::Fire, definition, ir, true);
            prepare(&mut battle);
            let events = act(&mut battle, 3);
            let ids = detonated_ids(&events);
            assert_eq!(ids.len(), 1);
            assert!([1, 2, 3].contains(&ids[0]));
            assert_eq!(battle.view().rng_draw_count(), 1);
            (events, battle.state_hash())
        };
        assert_eq!(run(), run());
        let mut battle = battle(CombatElement::Fire, definition, ir, false);
        prepare(&mut battle);
        assert_eq!(detonated_ids(&act(&mut battle, 3)), [1]);
        assert_eq!(battle.view().rng_draw_count(), 0);
    }
}

#[test]
fn break_detonation_event_codec_has_complete_fixed_width_goldens() {
    use sha2::{Digest, Sha256};
    let mut battle = battle(
        CombatElement::Fire,
        detonation(CombatElement::Fire),
        false,
        true,
    );
    prepare(&mut battle);
    let events = act(&mut battle, 3);
    let mut bytes = Vec::new();
    for event in &events {
        match event.kind() {
            BattleEventKind::BreakDamage(data)
                if data.kind == BreakDamageKind::EffectDetonation =>
            {
                let encoded = encode_battle_event_payload(event).unwrap();
                assert_eq!(
                    encoded[encoded.len() - 66 + 16],
                    3,
                    "Break damage discriminator"
                );
                bytes.extend_from_slice(&encoded);
            }
            BattleEventKind::Toughness(ToughnessEventData::BaseEffectDetonated { .. }) => {
                let encoded = encode_battle_event_payload(event).unwrap();
                assert_eq!(encoded[encoded.len() - 34], 12, "Toughness discriminator");
                assert_eq!(&encoded[encoded.len() - 8..], &2_500_000_i64.to_le_bytes());
                bytes.extend_from_slice(&encoded);
            }
            _ => {}
        }
    }
    assert!(!bytes.is_empty());
    let digest = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(
        digest,
        "4727b1539edbc7eabde849da4be8bd22fca10a03f8863df12185c7f51cf17ee0"
    );
}

#[test]
fn break_detonation_rejected_command_preserves_prepared_pool_and_rng() {
    let mut battle = battle(
        CombatElement::Fire,
        detonation(CombatElement::Fire),
        false,
        true,
    );
    prepare(&mut battle);
    let hash = battle.state_hash();
    let rng = battle.view().rng_draw_count();
    let invalid = Command::StartBattle {
        decision: battle.decision().unwrap().id(),
    };
    assert!(battle.apply(invalid).is_err());
    assert_eq!(battle.state_hash(), hash);
    assert_eq!(battle.view().rng_draw_count(), rng);
    assert_eq!(detonated_ids(&act(&mut battle, 3)), [1, 2, 3]);
}
