//! Production lowering, simultaneous entry captures and actual Break damage.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    tests::{curio_battle_grants::ready, curio_battle_stats::assemble},
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_combat::{
    AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattleSeed, BattleSpec, BreakDamageKind,
    CombatantSpecDigest, Command, ConcedePolicy, DecisionId, DispelCategory, DurationClock,
    EffectApplicationDefinition, EffectCategory, EffectChancePolicy, EffectRuntimeDefinition,
    EffectStackPolicy, EffectTickPhase, Energy, FormationIndex, Hp, LifeState,
    ParticipantInitialState, ParticipantSource, ParticipantSpec, PresenceState, Probability, Ratio,
    RawToughness, ResolvedBuildBonuses, ResolvedCombatantSpec, ResolvedDefinitionBindings, Scalar,
    Speed, TeamResourceSpec, TeamSide, ToughnessLayerSpec, ToughnessReductionDefinition, UnitLevel,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HitOperationDefinition, TargetInvalidationPolicy, TargetPattern, TargetRelation,
            UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition,
            ProgramDefinition, SelectorDefinition, UnitDefinition,
        },
    },
    formula::{
        model::CombatElement,
        toughness::{self, BreakDamageDefinition, EnemyRank, ToughnessReductionContext},
    },
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{RuleValue, ValueExpr},
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
const PARTY: [u32; 4] = [1001, 1002, 1103, 1005];
const EFFECT: u32 = 0x7ea5_0001;
const MODIFIER: u32 = 0x7ea8_0001;

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

fn equipped(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> DivergentUniverseAssembledBattle {
    let (flow, mut activity) = ready(fixture, family);
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    let selected = fixture
        .factory()
        .decision_catalog()
        .weighted_curio_break_effects()[0]
        .weighted_curio
        .clone();
    let hash = activity.state_hash();
    runtime
        .replace_accepted_loadout(
            &flow,
            &mut activity,
            hash,
            WeightedCurioSlotLimit::new(1).unwrap(),
            &[selected],
        )
        .unwrap();
    assemble(fixture, &flow, &activity)
}

fn break_formula(break_effect: i64) -> BreakDamageDefinition {
    BreakDamageDefinition {
        attacker_level_multiplier: Scalar::checked_from_integer(100).unwrap(),
        ability_multiplier: Ratio::ONE,
        break_effect: Ratio::from_scaled(break_effect),
        break_damage_increase: Ratio::ZERO,
        defense_multiplier: Ratio::ONE,
        resistance_multiplier: Ratio::ONE,
        vulnerability_multiplier: Ratio::ONE,
        mitigation_multiplier: Ratio::ONE,
        unbroken_multiplier: Ratio::ONE,
    }
}

#[derive(Clone, Copy, Default)]
struct Probe {
    values: [i64; 4],
    down_first: bool,
    absent: Option<u8>,
    actor: u8,
    later_bonus: bool,
}

fn probe(assembled: &DivergentUniverseAssembledBattle, input: Probe) -> Battle {
    let mut builder = CombatCatalogBuilder::from_catalog(assembled.combat_catalog(), [0xc1; 32]);
    let target_selector = id(0x7dbe_0001);
    let program = id(0x7dbf_0001);
    let form = id(0x7dc0_0001);
    let enemy = id(0x7dc1_0001);
    let encounter = id(0x7dc2_0001);
    builder.add_selector(SelectorDefinition::new(target_selector).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_program(ProgramDefinition::new(
        program,
        vec![],
        vec![],
        vec![],
        vec![],
    ));
    let late_effect = id(0x7dc3_0001);
    if input.later_bonus {
        let modifier = id(0x7dc4_0001);
        let group = id(0x7dc5_0001);
        builder.add_modifier_group(ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        });
        builder.add_modifier(ModifierDefinition {
            id: modifier,
            stat: StatKind::BreakEffect,
            stage: FormulaStage::Flat,
            purpose: FormulaPurpose::Stat,
            value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(500_000))),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::Flat,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        });
        builder.add_effect(
            EffectDefinition::new(late_effect, vec![], vec![modifier]).with_runtime(
                EffectRuntimeDefinition::new(
                    EffectCategory::Buff,
                    DispelCategory::NonDispellable,
                    1,
                    None,
                    DurationClock::Permanent,
                    EffectTickPhase::None,
                    EffectStackPolicy::Replace,
                )
                .unwrap(),
            ),
        );
        // Its separate SelfUnit Ultimate applies the later buff to the actor,
        // without advancing that actor's normal turn or recapturing entry stats.
    }
    let mut participants = Vec::new();
    for original in assembled
        .battle_spec()
        .participants()
        .iter()
        .filter(|p| p.side() == TeamSide::Player)
    {
        let index = original.formation().get();
        let base = original.combatant();
        let ability = id(0x7dc6_0001 + u32::from(index));
        let element = if index == 1 {
            CombatElement::Wind
        } else {
            CombatElement::Lightning
        };
        let reduction = HitOperationDefinition::ReduceToughness(ToughnessReductionDefinition {
            element,
            ignores_weakness: false,
            reduction: ToughnessReductionContext {
                base: RawToughness::new(40).unwrap(),
                additive: RawToughness::new(0).unwrap(),
                reduction_increase: Ratio::ZERO,
                weakness_break_efficiency: Ratio::ZERO,
                weakness_break_efficiency_cap: Ratio::from_scaled(3_000_000),
                toughness_vulnerability: Ratio::ZERO,
                ability_multiplier: Ratio::ONE,
            },
            break_damage: break_formula(0),
            break_effect_chance: Probability::ONE,
        });
        let action = AbilityActionDefinition::new(
            AbilityKind::Basic,
            1,
            TargetInvalidationPolicy::KeepIfPresent,
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
        )
        .unwrap()
        .with_hits(vec![ActionHitDefinition::new(vec![reduction])])
        .unwrap();
        builder.add_ability(
            AbilityDefinition::new(ability, program, target_selector, vec![]).with_action(action),
        );
        let mut abilities = vec![ability];
        if input.later_bonus && index == input.actor {
            let allied = id(0x7dc7_0001);
            let buff_ability = id(0x7dc8_0001);
            builder.add_selector(SelectorDefinition::new(allied).with_unit_targets(
                UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single).unwrap(),
            ));
            let action = AbilityActionDefinition::new(
                AbilityKind::Ultimate,
                1,
                TargetInvalidationPolicy::KeepIfPresent,
                ActionResourcePolicy::new(
                    0,
                    0,
                    Energy::from_scaled(1_000_000).unwrap(),
                    Energy::ZERO,
                ),
            )
            .unwrap()
            .with_hits(vec![ActionHitDefinition::new(vec![
                HitOperationDefinition::ApplyEffect(
                    EffectApplicationDefinition::new(
                        late_effect,
                        EffectChancePolicy::Guaranteed,
                        1,
                    )
                    .unwrap(),
                ),
            ])])
            .unwrap();
            builder.add_ability(
                AbilityDefinition::new(buff_ability, program, allied, vec![late_effect])
                    .with_action(action),
            );
            abilities.push(buff_ability);
            abilities.sort_unstable();
        }
        let maximum_energy = if input.later_bonus && index == input.actor {
            Energy::from_scaled(2_000_000).unwrap()
        } else {
            base.maximum_energy()
        };
        let initial_energy = if input.later_bonus && index == input.actor {
            maximum_energy
        } else {
            Energy::ZERO
        };
        let spec = ResolvedCombatantSpec::new(
            base.form(),
            base.level(),
            Hp::new(1_000).unwrap(),
            Speed::from_scaled(if index == input.actor {
                500_000_000
            } else {
                100_000_000
            })
            .unwrap(),
            ResolvedDefinitionBindings::new(
                abilities,
                base.rule_bundles().to_vec(),
                base.modifiers().to_vec(),
            )
            .unwrap(),
            CombatantSpecDigest::new([index + 1; 32]).unwrap(),
        )
        .unwrap()
        .with_build_bonuses(ResolvedBuildBonuses::new(
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::from_scaled(input.values[usize::from(index)]),
            Scalar::ZERO,
            Scalar::ZERO,
            [Scalar::ZERO; 7],
        ))
        .with_energy(initial_energy, maximum_energy)
        .unwrap()
        .with_sources(base.sources().to_vec())
        .unwrap()
        .with_modifier_bindings(base.modifier_bindings().to_vec())
        .unwrap();
        let down = input.down_first && index == 0;
        participants.push(
            ParticipantSpec::new(
                original.side(),
                original.formation(),
                original.source(),
                spec,
            )
            .with_initial_state(
                ParticipantInitialState::new(
                    Hp::new(if down { 0 } else { 1_000 }).unwrap(),
                    Hp::new(1_000).unwrap(),
                    initial_energy,
                    maximum_energy,
                    if down {
                        LifeState::Defeated
                    } else {
                        LifeState::Alive
                    },
                    if input.absent == Some(index) {
                        PresenceState::Linked
                    } else {
                        PresenceState::Present
                    },
                )
                .unwrap(),
            )
            .unwrap(),
        );
    }
    let empty_ability = id(0x7dc9_0001);
    let empty = AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::KeepIfPresent,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap();
    builder.add_ability(
        AbilityDefinition::new(empty_ability, program, target_selector, vec![]).with_action(empty),
    );
    builder.add_unit(UnitDefinition::new(form, vec![empty_ability], vec![]));
    builder.add_enemy(EnemyDefinition::new(enemy, form, vec![empty_ability]));
    builder.add_encounter(EncounterDefinition::new(encounter, vec![enemy], vec![]));
    let enemy_spec = ResolvedCombatantSpec::new(
        form,
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000_000).unwrap(),
        Speed::from_scaled(50_000_000).unwrap(),
        ResolvedDefinitionBindings::new(vec![empty_ability], vec![], vec![]).unwrap(),
        CombatantSpecDigest::new([0xc2; 32]).unwrap(),
    )
    .unwrap()
    .with_toughness(
        EnemyRank::Normal,
        vec![CombatElement::Lightning, CombatElement::Wind],
        vec![ToughnessLayerSpec::ordinary(1, RawToughness::new(40).unwrap()).unwrap()],
    )
    .unwrap()
    .with_build_bonuses(ResolvedBuildBonuses::new(
        Scalar::ZERO,
        Scalar::ZERO,
        Scalar::from_scaled(99_000_000),
        Scalar::ZERO,
        Scalar::ZERO,
        [Scalar::ZERO; 7],
    ));
    participants.push(ParticipantSpec::new(
        TeamSide::Enemy,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::EncounterEnemy(enemy),
        enemy_spec,
    ));
    Battle::create(
        builder.build().unwrap(),
        BattleSpec::new(
            AssemblyDigest::new([0xc3; 32]).unwrap(),
            encounter,
            participants,
            TeamResourceSpec::new(3, 5).unwrap(),
            TeamResourceSpec::new(0, 0).unwrap(),
            ConcedePolicy::Allowed,
        )
        .unwrap(),
        BattleSeed::new([0xc4; 32]),
    )
    .unwrap()
}

fn apply(running: &mut Battle, fresh: &mut Battle, command: Command) -> Vec<BattleEvent> {
    let actual = running.apply(command.clone()).unwrap();
    let expected = fresh.apply(command).unwrap();
    assert!(actual.fault().is_none(), "{:?}", actual.fault());
    assert_eq!(actual.events(), expected.events());
    assert_eq!(running.state_hash(), fresh.state_hash());
    assert_eq!(
        running.view().rng_draw_count(),
        fresh.view().rng_draw_count()
    );
    actual.events().to_vec()
}
fn start(running: &mut Battle, fresh: &mut Battle) {
    let hash = running.state_hash();
    let rng = running.view().rng_draw_count();
    assert!(
        running
            .apply(Command::StartBattle {
                decision: DecisionId::new(999).unwrap()
            })
            .is_err()
    );
    assert_eq!(running.state_hash(), hash);
    assert_eq!(running.view().rng_draw_count(), rng);
    let command = Command::StartBattle {
        decision: running.decision().unwrap().id(),
    };
    apply(running, fresh, command);
}
fn captures(battle: &Battle) -> Vec<(u8, i64)> {
    let mut values = battle
        .view()
        .modifier_instances_by_id()
        .filter(|m| m.definition().get() == MODIFIER)
        .map(|m| {
            let unit = battle
                .view()
                .units_by_id()
                .find(|u| u.id() == m.subject())
                .unwrap();
            assert_eq!(
                m.slots().collect::<Vec<_>>(),
                [(
                    id(0x7ea9_0001),
                    &RuleValue::Scalar(m.captured_value().unwrap())
                )]
            );
            let effect = battle
                .view()
                .effects_by_id()
                .find(|e| Some(e.id()) == m.source_effect())
                .unwrap();
            assert_eq!(effect.definition().get(), EFFECT);
            assert_eq!(effect.target(), unit.id());
            assert_eq!(effect.magnitude(), m.captured_value().unwrap());
            (unit.formation().get(), m.captured_value().unwrap().scaled())
        })
        .collect::<Vec<_>>();
    values.sort_unstable();
    values
}

#[test]
fn weighted_curio_break_effect_captures_one_team_maximum_for_both_elements_without_compounding() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        for values in [
            [2_000_000, 500_000, 1_000_000, 750_000],
            [0, 2_000_000, 500_000, 100_000],
            [500_000; 4],
            [0; 4],
        ] {
            let input = Probe {
                values,
                ..Probe::default()
            };
            let mut running = probe(&assembled, input);
            let mut fresh = probe(&assembled, input);
            start(&mut running, &mut fresh);
            let highest = values.into_iter().max().unwrap();
            assert_eq!(
                captures(&running),
                (1..4)
                    .map(|n| (n, highest * 6 / 5 - values[usize::from(n)]))
                    .collect::<Vec<_>>()
            );
            assert_eq!(running.view().rng_draw_count(), 0);
        }
    }
}

#[test]
fn weighted_curio_break_effect_skips_inactive_members_and_uses_next_living_producer() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        for absent in [None, Some(2)] {
            let input = Probe {
                values: [9_000_000, 500_000, 8_000_000, 2_000_000],
                down_first: true,
                absent,
                actor: 1,
                later_bonus: false,
            };
            let mut running = probe(&assembled, input);
            let mut fresh = probe(&assembled, input);
            start(&mut running, &mut fresh);
            let expected = if absent.is_some() {
                vec![(1, 1_900_000), (3, 400_000)]
            } else {
                vec![(1, 9_100_000), (2, 1_600_000), (3, 7_600_000)]
            };
            assert_eq!(captures(&running), expected);
            assert_eq!(running.view().rng_draw_count(), 0);
        }
    }
}

#[test]
fn weighted_curio_break_effect_respects_reordered_roster_and_no_eligible_party() {
    for family in FAMILIES {
        let reordered =
            DivergentUniverseBaselineFixture::production_for_source_party([1005, 1103, 1002, 1001])
                .unwrap();
        let assembled = equipped(&reordered, family);
        let input = Probe {
            values: [750_000, 1_000_000, 500_000, 2_000_000],
            ..Probe::default()
        };
        let mut running = probe(&assembled, input);
        let mut fresh = probe(&assembled, input);
        start(&mut running, &mut fresh);
        assert_eq!(
            captures(&running),
            [(0, 1_650_000), (1, 1_400_000), (2, 1_900_000)]
        );
        let excluded =
            DivergentUniverseBaselineFixture::production_for_source_party([1001, 1003, 1004, 8001])
                .unwrap();
        let assembled = equipped(&excluded, family);
        assert!(assembled.combat_catalog().effect(id(EFFECT)).is_none());
        let mut running = probe(&assembled, input);
        let mut fresh = probe(&assembled, input);
        start(&mut running, &mut fresh);
        assert!(captures(&running).is_empty());
        assert_eq!(running.view().rng_draw_count(), 0);
    }
}

#[test]
fn weighted_curio_break_effect_changes_real_break_damage_and_keeps_later_buffs_live() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        for actor in [1, 2] {
            for later_bonus in [false, true] {
                let input = Probe {
                    values: [2_000_000, 500_000, 1_000_000, 750_000],
                    actor,
                    later_bonus,
                    ..Probe::default()
                };
                let mut running = probe(&assembled, input);
                let mut fresh = probe(&assembled, input);
                start(&mut running, &mut fresh);
                let before = captures(&running);
                if later_bonus {
                    let option = running
                        .available_ultimates()
                        .into_iter()
                        .find(|option| option.ability().get() == 0x7dc8_0001)
                        .unwrap();
                    let request = running.request_ultimate_command(option).unwrap();
                    apply(&mut running, &mut fresh, request);
                    let command = running
                        .decision()
                        .unwrap()
                        .legal_commands()
                        .iter()
                        .find(|c| matches!(c, Command::CommitPreparedAction { .. }))
                        .unwrap()
                        .clone();
                    apply(&mut running, &mut fresh, command);
                }
                for _ in 0..3 {
                    if let Some(command) = running.advance_command() {
                        apply(&mut running, &mut fresh, command);
                    } else {
                        break;
                    }
                }
                let command = running.decision().unwrap().legal_commands().iter().find(|c| matches!(c, Command::UseAbility { ability, .. } if ability.get() == 0x7dc6_0001 + u32::from(actor))).unwrap().clone();
                let events = apply(&mut running, &mut fresh, command);
                let damage = events
                    .iter()
                    .filter_map(|e| match e.kind() {
                        BattleEventKind::BreakDamage(d) if d.kind == BreakDamageKind::Initial => {
                            Some(d.calculated)
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                let element = if actor == 1 {
                    CombatElement::Wind
                } else {
                    CombatElement::Lightning
                };
                let expected = toughness::break_damage(
                    break_formula(2_400_000 + if later_bonus { 500_000 } else { 0 }),
                    element,
                    RawToughness::new(40).unwrap(),
                    false,
                )
                .unwrap()
                .finalized;
                assert_eq!(damage, [expected]);
                assert_eq!(captures(&running), before);
                assert_eq!(running.view().rng_draw_count(), 0);
            }
        }
    }
}

#[test]
fn weighted_curio_break_effect_unequip_preserves_activity_and_reconstructs_production_handoffs() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    let selected = fixture
        .factory()
        .decision_catalog()
        .weighted_curio_break_effects()[0]
        .weighted_curio
        .clone();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                std::slice::from_ref(&selected),
            )
            .unwrap();
        let before = activity.canonical_state_bytes();
        let rng = activity.debug_view();
        let assembled = assemble(&fixture, &flow, &activity);
        assert!(assembled.combat_catalog().effect(id(EFFECT)).is_some());
        assert_eq!(
            assemble(&fixture, &flow, &activity).battle_spec(),
            assembled.battle_spec()
        );
        let mut running = Battle::create(
            assembled.combat_catalog().clone(),
            assembled.battle_spec().clone(),
            BattleSeed::new([0xc5; 32]),
        )
        .unwrap();
        let mut fresh = Battle::create(
            assembled.combat_catalog().clone(),
            assembled.battle_spec().clone(),
            BattleSeed::new([0xc5; 32]),
        )
        .unwrap();
        start(&mut running, &mut fresh);
        assert_eq!(captures(&running).len(), 3);
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.debug_view(), rng);
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                &[],
            )
            .unwrap();
        let plain = assemble(&fixture, &flow, &activity);
        assert!(plain.combat_catalog().effect(id(EFFECT)).is_none());
    }
}
