//! Production-passive assembly and public paid acquisition, not catalog credit.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    encode_divergent_universe_replay, record_divergent_universe_transcript,
    tests::{
        curio_battle_grants::{project, ready, result},
        curio_battle_stats::assemble,
        domain_choices::advance,
        instance, reward_draws,
    },
    verify_divergent_universe_replay,
};
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityMasterSeed, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivityTerminalOutcome, ActivityValue,
    BattleOutcome, BattleResult, GraphActivity, ProjectedValue,
};
use starclock_combat::{
    AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattleFault, BattleSeed, BattleSpec,
    BreakDamageKind, CombatantSpecDigest, Command, ConcedePolicy, Energy, FaultBoundary, FaultKind,
    FaultPolicy, FormationIndex, Hp, ParticipantSource, ParticipantSpec, Probability, Ratio,
    RawToughness, ResolvedCombatantSpec, ResolvedDefinitionBindings, Scalar, Speed,
    TeamResourceSpec, TeamSide, ToughnessLayerSpec, ToughnessReductionDefinition, UnitLevel,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HitCritPolicy, HitOperationDefinition, HitTargetGroup, OrdinaryDamageDefinition,
            OrdinaryDamageMultipliers, ShieldDefinition, TargetInvalidationPolicy, TargetPattern,
            TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
    },
    formula::{
        model::{CombatElement, DamageClass},
        shield::ShieldAbsorptionPolicy,
        toughness::{
            BreakDamageDefinition, EnemyRank, SuperBreakDefinition, ToughnessReductionContext,
        },
    },
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
};

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
fn state() -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9073").unwrap()
}
fn id<I: TryFrom<u32>>(value: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(value).unwrap()
}
fn charges(fixture: &DivergentUniverseBaselineFixture, activity: &GraphActivity) -> Option<u16> {
    fixture
        .factory()
        .curio_runtime()
        .unwrap()
        .owned(activity)
        .unwrap()
        .iter()
        .find(|held| held.state() == &state())
        .map(|held| held.charges())
}

fn probe(
    assembled: &DivergentUniverseAssembledBattle,
    class: Option<DamageClass>,
    shield: bool,
) -> Vec<BattleEvent> {
    let mut builder = CombatCatalogBuilder::from_catalog(assembled.combat_catalog(), [41; 32]);
    let selector = id(0x7d11_0001);
    let program = id(0x7d12_0001);
    let ability = id(0x7d13_0001);
    let form = id(0x7d14_0001);
    let enemy = id(0x7d15_0001);
    let encounter = id(0x7d16_0001);
    builder.add_selector(SelectorDefinition::new(selector).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_program(ProgramDefinition::new(
        program,
        vec![],
        vec![selector],
        vec![],
        vec![],
    ));
    let mut operations = Vec::new();
    if shield {
        operations.push(HitOperationDefinition::Shield(
            ShieldDefinition::new(
                Scalar::checked_from_integer(100).unwrap(),
                Ratio::ZERO,
                ShieldAbsorptionPolicy::ConcurrentLargest,
            )
            .unwrap(),
        ));
    }
    if let Some(class) = class {
        operations.push(HitOperationDefinition::Damage(
            OrdinaryDamageDefinition::new(
                Scalar::from_scaled(1_900_000),
                OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
            )
            .unwrap()
            .with_class(class),
        ));
    } else {
        operations.push(HitOperationDefinition::ReduceToughness(
            ToughnessReductionDefinition {
                element: CombatElement::Fire,
                ignores_weakness: false,
                reduction: ToughnessReductionContext {
                    base: RawToughness::new(50).unwrap(),
                    additive: RawToughness::new(0).unwrap(),
                    reduction_increase: Ratio::ZERO,
                    weakness_break_efficiency: Ratio::ZERO,
                    weakness_break_efficiency_cap: Ratio::from_scaled(3_000_000),
                    toughness_vulnerability: Ratio::ZERO,
                    ability_multiplier: Ratio::ONE,
                },
                break_damage: BreakDamageDefinition {
                    attacker_level_multiplier: Scalar::ONE,
                    ability_multiplier: Ratio::ONE,
                    break_effect: Ratio::ZERO,
                    break_damage_increase: Ratio::ZERO,
                    defense_multiplier: Ratio::ONE,
                    resistance_multiplier: Ratio::ONE,
                    vulnerability_multiplier: Ratio::ONE,
                    mitigation_multiplier: Ratio::ONE,
                    unbroken_multiplier: Ratio::ONE,
                },
                break_effect_chance: Probability::ONE,
            },
        ));
        operations.push(HitOperationDefinition::SuperBreak(SuperBreakDefinition {
            element: CombatElement::Fire,
            attacker_level_multiplier: Scalar::ONE,
            ability_multiplier: Ratio::ONE,
            break_effect: Ratio::ZERO,
            break_damage_increase: Ratio::ZERO,
            super_break_increase: Ratio::ZERO,
            defense_multiplier: Ratio::ONE,
            resistance_multiplier: Ratio::ONE,
            vulnerability_multiplier: Ratio::ONE,
            mitigation_multiplier: Ratio::ONE,
            broken_multiplier: Ratio::ONE,
        }));
    }
    let action = AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::KeepIfPresent,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap()
    .with_hits(vec![ActionHitDefinition::new(operations).with_profile(
        HitTargetGroup::Selected,
        Ratio::ONE,
        Ratio::ONE,
        HitCritPolicy::Never,
    )])
    .unwrap();
    builder.add_ability(
        AbilityDefinition::new(ability, program, selector, vec![]).with_action(action),
    );
    builder.add_unit(UnitDefinition::new(form, vec![ability], vec![]));
    builder.add_enemy(EnemyDefinition::new(enemy, form, vec![ability]));
    builder.add_encounter(EncounterDefinition::new(encounter, vec![enemy], vec![]));
    let original = assembled
        .battle_spec()
        .participants()
        .iter()
        .find(|p| p.side() == TeamSide::Player)
        .unwrap()
        .combatant();
    let mut participants = Vec::new();
    for (side, formation, speed) in [
        (TeamSide::Player, 0, 200_000_000),
        (TeamSide::Enemy, 4, 190_000_000),
    ] {
        let player = side == TeamSide::Player;
        let modifiers = if player {
            original.modifiers().to_vec()
        } else {
            vec![]
        };
        let mut combatant = ResolvedCombatantSpec::new(
            form,
            UnitLevel::new(80).unwrap(),
            Hp::new(10_000).unwrap(),
            Speed::from_scaled(speed).unwrap(),
            ResolvedDefinitionBindings::new(vec![ability], vec![], modifiers).unwrap(),
            CombatantSpecDigest::new([42; 32]).unwrap(),
        )
        .unwrap();
        if player {
            combatant = combatant
                .with_sources(original.sources().to_vec())
                .unwrap()
                .with_modifier_bindings(original.modifier_bindings().to_vec())
                .unwrap();
        } else {
            combatant = combatant
                .with_toughness(
                    EnemyRank::Normal,
                    vec![CombatElement::Fire],
                    vec![ToughnessLayerSpec::ordinary(1, RawToughness::new(50).unwrap()).unwrap()],
                )
                .unwrap();
        }
        participants.push(ParticipantSpec::new(
            side,
            FormationIndex::new(formation).unwrap(),
            if player {
                ParticipantSource::Player
            } else {
                ParticipantSource::EncounterEnemy(enemy)
            },
            combatant,
        ));
    }
    let spec = BattleSpec::new(
        AssemblyDigest::new([43; 32]).unwrap(),
        encounter,
        participants,
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle =
        Battle::create(builder.build().unwrap(), spec, BattleSeed::new([44; 32])).unwrap();
    let mut events = Vec::new();
    for _ in 0..20 {
        let command = battle
            .decision()
            .and_then(|d| {
                d.legal_commands()
                    .iter()
                    .find(|command| !matches!(command, Command::Concede { .. }))
            })
            .cloned()
            .unwrap_or_else(|| Command::Advance {
                boundary: battle.view().action_boundary().unwrap().id(),
            });
        let result = battle.apply(command).unwrap();
        assert!(result.fault().is_none());
        events.extend_from_slice(result.events());
        if class.is_some()
            && events
                .iter()
                .any(|e| matches!(e.kind(), BattleEventKind::Damage(_)))
        {
            return events;
        }
        if class.is_none()
            && events.iter().any(|e| matches!(e.kind(), BattleEventKind::BreakDamage(d) if d.kind == BreakDamageKind::Effect))
        {
            return events;
        }
    }
    panic!("controlled attack did not execute");
}

#[test]
fn curio_final_damage_production_binding_amplifies_all_six_purposes_and_preserves_build() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let plain = assemble(&fixture, &flow, &activity);
        let hash = activity.state_hash();
        fixture
            .factory()
            .curio_runtime()
            .unwrap()
            .acquire_accepted_state(&mut activity, hash, &state())
            .unwrap();
        let boosted = assemble(&fixture, &flow, &activity);
        assert_eq!(charges(&fixture, &activity), Some(5));
        for (before, after) in plain
            .battle_spec()
            .participants()
            .iter()
            .zip(boosted.battle_spec().participants())
        {
            assert_eq!(
                before.locked_combatant_digest(),
                after.locked_combatant_digest()
            );
            assert_eq!(
                before.combatant().base_attack(),
                after.combatant().base_attack()
            );
            assert_eq!(before.combatant().speed(), after.combatant().speed());
            if before.side() == TeamSide::Enemy {
                assert_eq!(before, after);
            } else {
                assert_eq!(
                    after.combatant().modifiers().len(),
                    before.combatant().modifiers().len() + 6
                );
            }
        }
        for class in [
            DamageClass::Direct,
            DamageClass::Dot,
            DamageClass::Additional,
            DamageClass::Elation,
        ] {
            for shield in [false, true] {
                let events = probe(&boosted, Some(class), shield);
                assert_eq!(events, probe(&boosted, Some(class), shield));
                let damage = events
                    .iter()
                    .find_map(|e| match e.kind() {
                        BattleEventKind::Damage(d) => Some(d),
                        _ => None,
                    })
                    .unwrap();
                assert_eq!(damage.raw.scaled(), 2_850_000);
                assert_eq!(damage.calculated.get(), 2);
                assert_eq!(
                    damage.hp_before.get() - damage.hp_after.get(),
                    if shield { 0 } else { 2 }
                );
            }
        }
        let control = probe(&plain, None, false);
        let actual = probe(&boosted, None, false);
        for kind in [
            BreakDamageKind::Initial,
            BreakDamageKind::SuperBreak,
            BreakDamageKind::Effect,
        ] {
            let raw = |events: &[BattleEvent]| {
                events
                    .iter()
                    .find_map(|e| match e.kind() {
                        BattleEventKind::BreakDamage(d) if d.kind == kind => Some(d.raw.scaled()),
                        _ => None,
                    })
                    .unwrap()
            };
            assert_eq!(raw(&actual), raw(&control) * 3 / 2, "{kind:?}");
        }
    }
}

#[test]
fn curio_final_damage_lifecycle_pauses_resumes_refills_and_discards_after_five_results() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let hash = activity.state_hash();
        curios
            .acquire_accepted_state(&mut activity, hash, &state())
            .unwrap();
        for (destroy, expected) in [(true, 1_900_000), (false, 2_850_000)] {
            let hash = activity.state_hash();
            if destroy {
                curios
                    .destroy_accepted(&mut activity, hash, &state())
                    .unwrap();
                assert!(
                    curios
                        .battle_lifetime_operations(&activity.player_view())
                        .unwrap()
                        .is_empty()
                );
            } else {
                curios
                    .repair_accepted(&mut activity, hash, &state())
                    .unwrap();
            }
            let bytes = activity.canonical_state_bytes();
            let assembled = assemble(&fixture, &flow, &activity);
            let events = probe(&assembled, Some(DamageClass::Direct), false);
            assert!(events.iter().any(
                |e| matches!(e.kind(),BattleEventKind::Damage(d) if d.raw.scaled()==expected)
            ));
            assert_eq!(charges(&fixture, &activity), Some(5));
            assert_eq!(bytes, activity.canonical_state_bytes());
        }
        assert!(
            curios
                .domain_entry_operations(&activity.player_view())
                .unwrap()
                .is_empty()
        );
        for expected in (1..=5).rev() {
            assert_eq!(charges(&fixture, &activity), Some(expected));
            // This is the five-result operation vector, not a claim of five public battles.
            let program = ActivityProgramDefinition::new(
                ActivityProgramId::new(24110).unwrap(),
                curios
                    .battle_lifetime_operations(&activity.player_view())
                    .unwrap(),
            )
            .unwrap();
            activity
                .apply_boundary_program(activity.state_hash(), &program)
                .unwrap();
        }
        assert_eq!(charges(&fixture, &activity), None);
        let hash = activity.state_hash();
        curios
            .acquire_accepted_state(&mut activity, hash, &state())
            .unwrap();
        assert_eq!(charges(&fixture, &activity), Some(5));
        let hash = activity.state_hash();
        curios
            .set_accepted_charges(&mut activity, hash, &state(), 0)
            .unwrap();
        let events = probe(
            &assemble(&fixture, &flow, &activity),
            Some(DamageClass::Direct),
            false,
        );
        assert!(
            events.iter().any(
                |e| matches!(e.kind(),BattleEventKind::Damage(d) if d.raw.scaled()==1_900_000)
            )
        );
        let other =
            DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9069").unwrap();
        let hash = activity.state_hash();
        curios
            .replace_accepted(&mut activity, hash, &state(), &other)
            .unwrap();
        let hash = activity.state_hash();
        curios
            .replace_accepted(&mut activity, hash, &other, &state())
            .unwrap();
        assert_eq!(charges(&fixture, &activity), Some(5));
    }
}

#[test]
fn curio_final_damage_paid_service_acquisition_completes_and_replays_actual_battles() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let option = curios
        .states()
        .iter()
        .find(|s| s.id() == &state())
        .unwrap()
        .state_key();
    for family in FAMILIES {
        let flow = fixture.flow_with_tawot_service(family, 2).unwrap();
        let mut found = None;
        for seed in 0..96 {
            let mut activity = flow
                .start(instance(1), ActivityMasterSeed::from_u64(seed))
                .unwrap()
                .into_activity();
            let mut steps = Vec::new();
            for _ in 0..3 {
                steps.push(advance(&fixture, &flow, &mut activity, None));
            }
            steps.push(advance(&fixture, &flow, &mut activity, Some(1)));
            if activity
                .player_view()
                .decision()
                .unwrap()
                .options()
                .iter()
                .any(|candidate| candidate.id().get() == option)
            {
                found = Some((seed, activity, steps));
                break;
            }
        }
        let (seed, mut activity, mut steps) =
            found.expect("bounded public service offers final-damage card");
        steps.push(advance(&fixture, &flow, &mut activity, Some(option)));
        assert_eq!(charges(&fixture, &activity), Some(5));
        steps.push(advance(&fixture, &flow, &mut activity, Some(2)));
        let actual = assemble(&fixture, &flow, &activity);
        assert!(
            probe(&actual, Some(DamageClass::Direct), false)
                .iter()
                .any(|e| matches!(e.kind(),BattleEventKind::Damage(d) if d.calculated.get()==2))
        );
        while activity.player_view().terminal().is_none() {
            assert!(steps.len() < 24);
            steps.push(advance(&fixture, &flow, &mut activity, None));
            assert_eq!(
                charges(&fixture, &activity),
                if activity.player_view().terminal().is_some() {
                    // The accepted finalization clears all run-owned inventory.
                    None
                } else {
                    Some(
                        5 - u16::try_from(activity.player_view().completed_battle_count()).unwrap(),
                    )
                }
            );
        }
        assert_eq!(activity.player_view().completed_battle_count(), 3);
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
        let recorded =
            record_divergent_universe_transcript(&fixture, &flow, &activity, seed, steps).unwrap();
        let bytes = encode_divergent_universe_replay(&recorded).unwrap();
        let verified = verify_divergent_universe_replay(&bytes, &fresh).unwrap();
        assert_eq!(
            verified.final_state_hash().bytes(),
            activity.state_hash().bytes()
        );
        assert_eq!(verified.battle_count(), 3);
    }
}

#[test]
fn curio_final_damage_verified_results_count_once_and_faults_preserve_allowance() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        for outcome in [
            BattleOutcome::Won,
            BattleOutcome::Lost,
            BattleOutcome::Faulted,
        ] {
            let (flow, mut activity) = ready(&fixture, family);
            let hash = activity.state_hash();
            curios
                .acquire_accepted_state(&mut activity, hash, &state())
                .unwrap();
            let hash = activity.state_hash();
            curios
                .set_accepted_charges(&mut activity, hash, &state(), 1)
                .unwrap();
            let actual = result(&fixture, &flow, &mut activity);
            // Counterfactual terminal-result contract vector, not an observed encounter outcome.
            let projected = project(&actual, 4, outcome);
            let projected = BattleResult::seal(
                projected.identity(),
                projected
                    .values()
                    .iter()
                    .map(|value| {
                        if outcome == BattleOutcome::Faulted
                            && matches!(value, ProjectedValue::TerminalFault(_))
                        {
                            ProjectedValue::TerminalFault(Some(BattleFault::from_parts(
                                FaultKind::Numeric,
                                FaultBoundary::Command,
                                FaultPolicy::Rollback,
                                1,
                                None,
                            )))
                        } else {
                            value.clone()
                        }
                    })
                    .collect(),
            );
            let hash = activity.state_hash();
            let settled = fixture
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&flow, &mut activity, hash, projected.clone(), None)
                .unwrap();
            assert_eq!(
                charges(&fixture, &activity),
                if outcome == BattleOutcome::Faulted {
                    Some(1)
                } else {
                    None
                }
            );
            assert_eq!(
                settled
                    .events()
                    .iter()
                    .any(|event| event.cause().program().get() == 24110),
                outcome != BattleOutcome::Faulted
            );
            let before = activity.canonical_state_bytes();
            let hash = activity.state_hash();
            assert!(
                fixture
                    .factory()
                    .battle_settlement_runtime()
                    .settle_started_result(&flow, &mut activity, hash, projected, None)
                    .is_err()
            );
            assert_eq!(before, activity.canonical_state_bytes());
            if outcome == BattleOutcome::Won {
                advance(&fixture, &flow, &mut activity, None);
                advance(&fixture, &flow, &mut activity, None);
                let next = probe(
                    &assemble(&fixture, &flow, &activity),
                    Some(DamageClass::Direct),
                    false,
                );
                assert!(next.iter().any(|e| matches!(e.kind(), BattleEventKind::Damage(d) if d.raw.scaled() == 1_900_000)));
            }
        }
    }
}

#[test]
fn curio_final_damage_late_reward_failure_restores_discard_carry_and_rng() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let hash = activity.state_hash();
        curios
            .acquire_accepted_state(&mut activity, hash, &state())
            .unwrap();
        let hash = activity.state_hash();
        curios
            .set_accepted_charges(&mut activity, hash, &state(), 1)
            .unwrap();
        let actual = result(&fixture, &flow, &mut activity);
        let before = activity.canonical_state_bytes();
        let draws = reward_draws(&activity);
        let hash = activity.state_hash();
        let expiry = ActivityProgramId::new(24110).unwrap();
        let offer = ActivityProgramId::new(23702).unwrap();
        let mut reached_discard = false;
        assert!(
            activity
                .submit_pending_battle_result_with_generated_boundary(
                    hash,
                    actual.clone(),
                    &[expiry, offer],
                    |program, view, _, rng| {
                        if program == expiry {
                            return Ok(curios.battle_lifetime_operations(view).unwrap());
                        }
                        reached_discard = curios
                            .owned_from_view(view)
                            .unwrap()
                            .iter()
                            .all(|held| held.state() != &state());
                        let mut operations = flow
                            .battle_blessings
                            .as_ref()
                            .unwrap()
                            .generate(view, rng)?;
                        operations.push(ActivityOperation::Require(ActivityCondition::Boolean(
                            ActivityExpression::Literal(ActivityValue::Boolean(false)),
                        )));
                        Ok(operations)
                    },
                )
                .is_err()
        );
        assert!(reached_discard);
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&activity), draws);
        assert_eq!(charges(&fixture, &activity), Some(1));
        fixture
            .factory()
            .battle_settlement_runtime()
            .settle_started_result(&flow, &mut activity, hash, actual, None)
            .unwrap();
        assert_eq!(charges(&fixture, &activity), None);
    }
}
