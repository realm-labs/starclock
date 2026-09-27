//! Actual production lowering and controlled damage probes, not Forge parity.

use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    DivergentUniverseBattleAssemblyError, DivergentUniverseBattleAssemblyPolicy,
    tests::{
        curio_battle_grants::{ready, result},
        curio_battle_stats::assemble,
    },
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_activity::ProjectedValue;
use starclock_combat::{
    ActionEventData, AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattleSeed, BattleSpec,
    CombatantSpecDigest, Command, ConcedePolicy, Energy, FormationIndex, Hp, ParticipantSource,
    ParticipantSpec, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings, Scalar, Speed,
    TeamResourceSpec, TeamSide, UnitEventData, UnitLevel,
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
    formula::{model::DamageClass, shield::ShieldAbsorptionPolicy},
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use std::sync::Arc;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

#[derive(Clone, Copy)]
struct Probe {
    damage: i64,
    hits: u16,
    class: DamageClass,
    shield: bool,
    primary: u8,
    primary_hp: i64,
    hunt: bool,
    pattern: TargetPattern,
    neighbor_hp: i64,
}
impl Default for Probe {
    fn default() -> Self {
        Self {
            damage: 19,
            hits: 2,
            class: DamageClass::Direct,
            shield: false,
            primary: 1,
            primary_hp: 10_000,
            hunt: true,
            pattern: TargetPattern::Single,
            neighbor_hp: 10_000,
        }
    }
}

fn probe(assembled: &DivergentUniverseAssembledBattle, input: Probe) -> Vec<BattleEvent> {
    let mut builder = CombatCatalogBuilder::from_catalog(assembled.combat_catalog(), [61; 32]);
    let selector = id(0x7d21_0001);
    let program = id(0x7d22_0001);
    let ability = id(0x7d23_0001);
    let form = id(0x7d24_0001);
    let encounter = id(0x7d26_0001);
    builder.add_selector(SelectorDefinition::new(selector).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, input.pattern).unwrap(),
    ));
    builder.add_program(ProgramDefinition::new(
        program,
        vec![],
        vec![selector],
        vec![],
        vec![],
    ));
    let hits = (0..input.hits)
        .map(|_| {
            let mut operations = Vec::new();
            if input.shield {
                operations.push(HitOperationDefinition::Shield(
                    ShieldDefinition::new(
                        Scalar::checked_from_integer(100).unwrap(),
                        Ratio::ZERO,
                        ShieldAbsorptionPolicy::ConcurrentLargest,
                    )
                    .unwrap(),
                ));
            }
            operations.push(HitOperationDefinition::Damage(
                OrdinaryDamageDefinition::new(
                    Scalar::checked_from_integer(input.damage).unwrap(),
                    OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                )
                .unwrap()
                .with_class(input.class),
            ));
            ActionHitDefinition::new(operations).with_profile(
                HitTargetGroup::Selected,
                Ratio::ONE,
                Ratio::ONE,
                HitCritPolicy::Never,
            )
        })
        .collect();
    let action = AbilityActionDefinition::new(
        AbilityKind::Basic,
        input.hits,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap()
    .with_hits(hits)
    .unwrap();
    builder.add_ability(
        AbilityDefinition::new(ability, program, selector, vec![]).with_action(action),
    );
    builder.add_unit(UnitDefinition::new(form, vec![ability], vec![]));
    let original = assembled
        .battle_spec()
        .participants()
        .iter()
        .filter(|participant| participant.side() == TeamSide::Player)
        .find(|participant| (participant.formation().get() == 1) == input.hunt)
        .unwrap()
        .combatant();
    let player = ResolvedCombatantSpec::new(
        form,
        UnitLevel::new(80).unwrap(),
        Hp::new(10_000).unwrap(),
        Speed::from_scaled(200_000_000).unwrap(),
        ResolvedDefinitionBindings::new(
            vec![ability],
            original.rule_bundles().to_vec(),
            original.modifiers().to_vec(),
        )
        .unwrap(),
        CombatantSpecDigest::new([62; 32]).unwrap(),
    )
    .unwrap()
    .with_sources(original.sources().to_vec())
    .unwrap()
    .with_modifier_bindings(original.modifier_bindings().to_vec())
    .unwrap();
    let mut participants = vec![ParticipantSpec::new(
        TeamSide::Player,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::Player,
        player,
    )];
    let mut enemies = Vec::new();
    for index in [0_u8, 1, 2, 4] {
        let enemy = id(0x7d25_0001 + u32::from(index));
        enemies.push(enemy);
        builder.add_enemy(EnemyDefinition::new(enemy, form, vec![ability]));
        let combatant = ResolvedCombatantSpec::new(
            form,
            UnitLevel::new(80).unwrap(),
            Hp::new(if index == input.primary {
                input.primary_hp
            } else {
                input.neighbor_hp
            })
            .unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
            ResolvedDefinitionBindings::new(vec![ability], vec![], vec![]).unwrap(),
            CombatantSpecDigest::new([63; 32]).unwrap(),
        )
        .unwrap();
        participants.push(ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(index).unwrap(),
            ParticipantSource::EncounterEnemy(enemy),
            combatant,
        ));
    }
    builder.add_encounter(EncounterDefinition::new(encounter, enemies, vec![]));
    let spec = BattleSpec::new(
        AssemblyDigest::new([64; 32]).unwrap(),
        encounter,
        participants,
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle =
        Battle::create(builder.build().unwrap(), spec, BattleSeed::new([65; 32])).unwrap();
    let target = battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Enemy && unit.formation().get() == input.primary)
        .unwrap()
        .id();
    let mut events = Vec::new();
    for _ in 0..24 {
        let command = battle.decision().and_then(|decision| decision.legal_commands().iter()
            .find(|command| matches!(command, Command::UseAbility { primary_target, .. } if *primary_target == Some(target)))
            .or_else(|| decision.legal_commands().iter().find(|command| matches!(command, Command::StartBattle { .. }))))
            .cloned().unwrap_or_else(|| Command::Advance { boundary: battle.view().action_boundary().unwrap().id() });
        let result = battle.apply(command).unwrap();
        assert!(result.fault().is_none(), "{:?}", result.fault());
        events.extend_from_slice(result.events());
        if events.iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Action(ActionEventData::Resolved { .. })
            )
        }) {
            return events;
        }
    }
    panic!("controlled player attack did not finish");
}

fn copies(events: &[BattleEvent]) -> Vec<(u64, i64)> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(damage)
                if event
                    .cause()
                    .source_definition()
                    .is_some_and(|source| source.get() == 0x7e44_0001) =>
            {
                assert_eq!(damage.class, DamageClass::Additional);
                assert_eq!(damage.element, None);
                Some((damage.target.get(), damage.calculated.get()))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn weighted_curio_splash_preserves_build_and_executes_each_hit_adjacent_copy_in_both_families() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1308, 1002, 1009, 1105])
            .unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let plain = assemble(&fixture, &flow, &activity);
        let runtime = fixture.factory().weighted_curio_runtime().unwrap();
        let selected = fixture
            .factory()
            .decision_catalog()
            .weighted_curio_splashes()[0]
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
        let before = activity.canonical_state_bytes();
        let equipped = assemble(&fixture, &flow, &activity);
        assert_ne!(plain.assembly_digest(), equipped.assembly_digest());
        for (before, after) in plain
            .battle_spec()
            .participants()
            .iter()
            .zip(equipped.battle_spec().participants())
        {
            assert_eq!(
                before.locked_combatant_digest(),
                after.locked_combatant_digest()
            );
            assert_eq!(
                before.combatant().base_attack(),
                after.combatant().base_attack()
            );
            assert_eq!(
                before.combatant().build_bonuses(),
                after.combatant().build_bonuses()
            );
            if before.side() == TeamSide::Enemy {
                assert_eq!(before.combatant(), after.combatant());
            }
        }
        let events = probe(&equipped, Probe::default());
        assert_eq!(
            copies(&events).len(),
            4,
            "damage facts: {:?}",
            events
                .iter()
                .filter(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
                .collect::<Vec<_>>()
        );
        assert!(copies(&events).iter().all(|(_, amount)| *amount == 5));
        assert_eq!(events, probe(&equipped, Probe::default()));
        let fresh =
            DivergentUniverseBaselineFixture::production_for_source_party([1308, 1002, 1009, 1105])
                .unwrap();
        let (fresh_flow, mut fresh_activity) = ready(&fresh, family);
        let fresh_hash = fresh_activity.state_hash();
        fresh
            .factory()
            .weighted_curio_runtime()
            .unwrap()
            .replace_accepted_loadout(
                &fresh_flow,
                &mut fresh_activity,
                fresh_hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                &[
                    fresh.factory().decision_catalog().weighted_curio_splashes()[0]
                        .weighted_curio
                        .clone(),
                ],
            )
            .unwrap();
        let rebuilt = assemble(&fresh, &fresh_flow, &fresh_activity);
        assert_eq!(equipped.assembly_digest(), rebuilt.assembly_digest());
        assert_eq!(events, probe(&rebuilt, Probe::default()));
        let blast = probe(
            &equipped,
            Probe {
                pattern: TargetPattern::Blast,
                ..Probe::default()
            },
        );
        assert_eq!(copies(&blast).len(), 8);
        assert!(copies(&blast).iter().all(|(_, amount)| *amount == 5));
        assert!(copies(&probe(&plain, Probe::default())).is_empty());
        assert_eq!(activity.canonical_state_bytes(), before);
        for primary in [0, 2, 4] {
            let copies = copies(&probe(
                &equipped,
                Probe {
                    primary,
                    ..Probe::default()
                },
            ));
            assert_eq!(copies.len(), if primary == 4 { 0 } else { 2 });
        }
        for class in [
            DamageClass::Dot,
            DamageClass::Additional,
            DamageClass::Elation,
        ] {
            let events = probe(
                &equipped,
                Probe {
                    class,
                    ..Probe::default()
                },
            );
            assert!(copies(&events).is_empty());
        }
        assert!(
            copies(&probe(
                &equipped,
                Probe {
                    hunt: false,
                    ..Probe::default()
                }
            ))
            .is_empty()
        );
        assert!(
            copies(&probe(
                &equipped,
                Probe {
                    damage: 0,
                    ..Probe::default()
                }
            ))
            .is_empty()
        );
        let shielded = probe(
            &equipped,
            Probe {
                shield: true,
                ..Probe::default()
            },
        );
        assert_eq!(copies(&shielded).len(), 4);
        assert!(shielded.iter().any(|event| matches!(event.kind(), BattleEventKind::Damage(damage)
            if damage.class == DamageClass::Direct && damage.absorbed.get() == 19 && damage.applied.get() == 0)));
        let killed = probe(
            &equipped,
            Probe {
                primary_hp: 1,
                ..Probe::default()
            },
        );
        assert_eq!(copies(&killed).len(), 2);
        assert!(copies(&killed).iter().all(|(_, amount)| *amount == 5));
        let neighbors_killed = probe(
            &equipped,
            Probe {
                neighbor_hp: 1,
                ..Probe::default()
            },
        );
        assert_eq!(copies(&neighbors_killed).len(), 2);
        assert_eq!(
            neighbors_killed
                .iter()
                .filter(|event| matches!(
                    event.kind(),
                    BattleEventKind::Unit(UnitEventData::Defeated { .. })
                ))
                .count(),
            2
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        // This goes through the actual production handoff, not a synthetic result.
        assert_eq!(
            result(&fixture, &flow, &mut activity)
                .values()
                .iter()
                .filter(|value| matches!(value, ProjectedValue::ParticipantState(_)))
                .count(),
            4
        );
    }
}

#[test]
fn weighted_curio_splash_unequip_removes_binding_and_stale_snapshot_is_rejected() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1308, 1002, 1009, 1105])
            .unwrap();
    let (flow, mut activity) = ready(&fixture, FAMILIES[0]);
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    let selected = fixture
        .factory()
        .decision_catalog()
        .weighted_curio_splashes()[0]
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
    let equipped = assemble(&fixture, &flow, &activity);
    let contribution = fixture
        .factory()
        .contribution_snapshot_runtime()
        .unwrap()
        .snapshot(&flow, &activity)
        .unwrap();
    let (group, stage) = flow.offered_encounter(&activity).unwrap().unwrap();
    let encounter = fixture
        .factory()
        .encounter_reachability_runtime()
        .unwrap()
        .select_stage_candidate(&activity, activity.state_hash(), group, stage)
        .unwrap();
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
    let empty = assemble(&fixture, &flow, &activity);
    let before = activity.canonical_state_bytes();
    let stale = fixture.factory().battle_assembly_runtime().materialize_current_battle(&flow, &activity,
        fixture.core(), &contribution, &encounter,
        DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy);
    assert_eq!(
        stale.unwrap_err(),
        DivergentUniverseBattleAssemblyError::StaleStateHash
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_ne!(equipped.assembly_digest(), empty.assembly_digest());
    assert!(copies(&probe(&empty, Probe::default())).is_empty());
    let mut battle = Battle::create(
        Arc::clone(equipped.combat_catalog()),
        equipped.battle_spec().clone(),
        BattleSeed::new([66; 32]),
    )
    .unwrap();
    let result = battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    assert!(result.fault().is_none());
}
