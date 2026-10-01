//! Sora-backed qualification, victim damage and real weighted normal targeting.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::{ready, result},
        curio_battle_stats::assemble,
    },
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_activity::ProjectedValue;
use starclock_combat::{
    ActionEventData, AssemblyDigest, Battle, BattleDiagnostics, BattleEvent, BattleEventKind,
    BattleSeed, BattleSpec, CauseActor, CombatantSpecDigest, Command, ConcedePolicy, DecisionId,
    Energy, FormationIndex, Hp, LifeState, ParticipantInitialState, ParticipantSource,
    ParticipantSpec, PresenceState, Ratio, Resolution, ResolvedBuildBonuses, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, Scalar, Speed, StatValue, TeamResourceSpec, TeamSide, UnitId,
    UnitLevel,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityTag, ActionHitDefinition,
            ActionResourcePolicy, HitCritPolicy, HitOperationDefinition, HitTargetGroup,
            OrdinaryDamageDefinition, OrdinaryDamageMultipliers, ShieldDefinition,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
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
    },
    rng::{
        engine::DeterministicRng,
        types::{DrawPurpose, RngSeed},
    },
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use std::sync::Arc;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
const PARTY: [u32; 4] = [8001, 1105, 1001, 1002];
const SOURCE: u32 = 0x7e93_0001;
const BUNDLE: u32 = 0x7e92_0001;
const PRIMARY: u32 = 0x7e97_0001;

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
    let selected = fixture
        .factory()
        .decision_catalog()
        .weighted_curio_retaliations()[0]
        .weighted_curio
        .clone();
    let hash = activity.state_hash();
    fixture
        .factory()
        .weighted_curio_runtime()
        .unwrap()
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

#[derive(Clone, Copy)]
struct Attack {
    all: bool,
    hits: u16,
    damage: i64,
    class: DamageClass,
    tagged: bool,
    shield: bool,
    enemy_hp: i64,
    dead: bool,
    linked: bool,
}
impl Default for Attack {
    fn default() -> Self {
        Self {
            all: true,
            hits: 3,
            damage: 10,
            class: DamageClass::Direct,
            tagged: true,
            shield: false,
            enemy_hp: 100_000,
            dead: false,
            linked: false,
        }
    }
}

fn probe(assembled: &DivergentUniverseAssembledBattle, attack: Attack, seed: u8) -> Battle {
    let mut builder = CombatCatalogBuilder::from_catalog(assembled.combat_catalog(), [0xb1; 32]);
    let selector = id(0x7d31_0001);
    let program = id(0x7d32_0001);
    let ability = id(0x7d33_0001);
    let form = id(0x7d34_0001);
    let enemy = id(0x7d35_0001);
    let encounter = id(0x7d36_0001);
    builder.add_selector(
        SelectorDefinition::new(selector).with_unit_targets(
            UnitTargetSelector::new(
                TargetRelation::Opposing,
                if attack.all {
                    TargetPattern::All
                } else {
                    TargetPattern::Single
                },
            )
            .unwrap(),
        ),
    );
    builder.add_program(ProgramDefinition::new(
        program,
        vec![],
        vec![],
        vec![],
        vec![],
    ));
    let hits = (0..attack.hits)
        .map(|index| {
            let mut operations = vec![];
            if index == 0 && attack.shield {
                operations.push(HitOperationDefinition::Shield(
                    ShieldDefinition::new(
                        Scalar::checked_from_integer(100_000).unwrap(),
                        Ratio::ZERO,
                        ShieldAbsorptionPolicy::ConcurrentLargest,
                    )
                    .unwrap(),
                ));
            }
            operations.push(HitOperationDefinition::Damage(
                OrdinaryDamageDefinition::new(
                    Scalar::checked_from_integer(attack.damage).unwrap(),
                    OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                )
                .unwrap()
                .with_class(attack.class),
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
        attack.hits,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap()
    .with_hits(hits)
    .unwrap()
    .with_tags(if attack.tagged {
        &[AbilityTag::Attack]
    } else {
        &[]
    });
    let definition = AbilityDefinition::new(ability, program, selector, vec![]).with_action(action);
    builder.add_ability(if attack.all {
        definition
    } else {
        definition.with_automatic_primary_selector(id(PRIMARY))
    });
    builder.add_unit(UnitDefinition::new(form, vec![ability], vec![]));
    builder.add_enemy(EnemyDefinition::new(enemy, form, vec![ability]));
    builder.add_encounter(EncounterDefinition::new(encounter, vec![enemy], vec![]));
    let mut participants = assembled
        .battle_spec()
        .participants()
        .iter()
        .filter(|entry| entry.side() == TeamSide::Player)
        .enumerate()
        .map(|(index, original)| {
            let base = original.combatant();
            let spec = ResolvedCombatantSpec::new(
                base.form(),
                base.level(),
                Hp::new(1_000).unwrap(),
                base.speed(),
                ResolvedDefinitionBindings::new(
                    base.abilities().to_vec(),
                    base.rule_bundles().to_vec(),
                    base.modifiers().to_vec(),
                )
                .unwrap(),
                CombatantSpecDigest::new([u8::try_from(index + 1).unwrap(); 32]).unwrap(),
            )
            .unwrap()
            .with_base_attack_defense(
                StatValue::from_scaled(i64::try_from(index + 1).unwrap() * 100_000_000).unwrap(),
                StatValue::from_scaled(0).unwrap(),
            )
            .with_build_bonuses(ResolvedBuildBonuses::default())
            .with_energy(Energy::ZERO, base.maximum_energy())
            .unwrap()
            .with_sources(base.sources().to_vec())
            .unwrap()
            .with_modifier_bindings(base.modifier_bindings().to_vec())
            .unwrap();
            let down = attack.dead && index == 0;
            ParticipantSpec::new(
                original.side(),
                original.formation(),
                original.source(),
                spec,
            )
            .with_locked_combatant_digest(original.locked_combatant_digest())
            .with_initial_state(
                ParticipantInitialState::new(
                    Hp::new(if down { 0 } else { 1_000 }).unwrap(),
                    Hp::new(1_000).unwrap(),
                    Energy::ZERO,
                    base.maximum_energy(),
                    if down {
                        LifeState::Defeated
                    } else {
                        LifeState::Alive
                    },
                    if attack.linked && index == 1 {
                        PresenceState::Linked
                    } else {
                        PresenceState::Present
                    },
                )
                .unwrap(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let spec = ResolvedCombatantSpec::new(
        form,
        UnitLevel::new(80).unwrap(),
        Hp::new(attack.enemy_hp).unwrap(),
        Speed::from_scaled(1_000_000_000).unwrap(),
        ResolvedDefinitionBindings::new(vec![ability], vec![], vec![]).unwrap(),
        CombatantSpecDigest::new([0xb2; 32]).unwrap(),
    )
    .unwrap()
    .with_base_attack_defense(
        StatValue::from_scaled(9_000_000_000).unwrap(),
        StatValue::from_scaled(0).unwrap(),
    );
    participants.push(ParticipantSpec::new(
        TeamSide::Enemy,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::EncounterEnemy(enemy),
        spec,
    ));
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xb3; 32]).unwrap(),
        encounter,
        participants,
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(builder.build().unwrap(), spec, BattleSeed::new([seed; 32])).unwrap()
}

fn first_attack(battle: &mut Battle) -> Resolution {
    for _ in 0..8 {
        let command = battle
            .decision()
            .and_then(|decision| {
                decision
                    .legal_commands()
                    .iter()
                    .find(|command| !matches!(command, Command::Concede { .. }))
            })
            .cloned()
            .unwrap_or_else(|| battle.advance_command().unwrap());
        let mut diagnostics = BattleDiagnostics::new();
        let result = battle
            .apply_inspected(command.clone(), &mut diagnostics)
            .unwrap();
        assert!(
            result.fault().is_none(),
            "command {command:?}: {:?}; diagnostics {:?}",
            result.fault(),
            diagnostics.records()
        );
        if result
            .events()
            .iter()
            .any(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
        {
            return result;
        }
    }
    panic!("fixture must execute an actual enemy attack");
}

fn retaliations(events: &[BattleEvent]) -> Vec<&BattleEvent> {
    events.iter().filter(|event| event.cause().source_definition().is_some_and(|id|id.get()==SOURCE)
        && matches!(event.kind(),BattleEventKind::Damage(data) if data.class==DamageClass::Additional)).collect()
}

#[test]
fn weighted_curio_retaliation_binds_only_physical_owners_and_materialized_enemy_primaries() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let (flow, activity) = ready(&fixture, family);
        let before = activity.canonical_state_bytes();
        let plain = assemble(&fixture, &flow, &activity);
        let added = equipped(&fixture, family);
        for (index, (old, new)) in plain
            .battle_spec()
            .participants()
            .iter()
            .zip(added.battle_spec().participants())
            .enumerate()
        {
            assert_eq!(old.locked_combatant_digest(), new.locked_combatant_digest());
            assert_eq!(old.combatant().maximum_hp(), new.combatant().maximum_hp());
            assert_eq!(old.combatant().base_attack(), new.combatant().base_attack());
            assert_eq!(
                new.combatant().rule_bundles().contains(&id(BUNDLE)),
                index < 2
            );
            if old.side() == TeamSide::Enemy {
                assert_eq!(old.combatant(), new.combatant());
                for ability in old.combatant().abilities() {
                    let definition = added.combat_catalog().ability(*ability).unwrap();
                    let action = definition.action().unwrap();
                    let target = added
                        .combat_catalog()
                        .selector(definition.selector())
                        .unwrap()
                        .unit_targets()
                        .unwrap();
                    if action.kind().is_normal_turn()
                        && target.relation() == TargetRelation::Opposing
                        && matches!(
                            target.pattern(),
                            TargetPattern::Single | TargetPattern::Blast
                        )
                    {
                        assert_eq!(definition.automatic_primary_selector(), Some(id(PRIMARY)));
                    }
                }
            }
        }
        let mut actual = Battle::create(
            Arc::clone(added.combat_catalog()),
            added.battle_spec().clone(),
            BattleSeed::new([0xb4; 32]),
        )
        .unwrap();
        let mut sampled = false;
        for _ in 0..64 {
            let command = actual
                .decision()
                .and_then(|decision| {
                    decision
                        .legal_commands()
                        .iter()
                        .find(|command| !matches!(command, Command::Concede { .. }))
                })
                .cloned()
                .unwrap_or_else(|| actual.advance_command().unwrap());
            let result = actual.apply(command).unwrap();
            assert!(result.fault().is_none());
            if result.events().iter().any(|event| matches!(event.kind(),BattleEventKind::Action(ActionEventData::Declared {actor,..})
                if actual.view().units_by_id().any(|unit|unit.id()==*actor&&unit.side()==TeamSide::Enemy))) { sampled=true;break; }
        }
        assert!(sampled);
        assert!(actual.view().rng_draw_count() > 0);
        assert_eq!(activity.canonical_state_bytes(), before);
    }
}

#[test]
fn weighted_curio_retaliation_uses_each_victims_atk_once_per_action_with_nonlethal_owner_credit() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        for attack in [
            Attack::default(),
            Attack {
                shield: true,
                ..Attack::default()
            },
            Attack {
                damage: 0,
                ..Attack::default()
            },
            Attack {
                enemy_hp: 7,
                ..Attack::default()
            },
        ] {
            let mut battle = probe(&assembled, attack, 1);
            let result = first_attack(&mut battle);
            let retaliation = retaliations(result.events());
            assert_eq!(retaliation.len(), 2);
            let declared = result
                .events()
                .iter()
                .find(|event| {
                    matches!(
                        event.kind(),
                        BattleEventKind::Action(ActionEventData::Declared { .. })
                    )
                })
                .unwrap();
            for (index, event) in retaliation.into_iter().enumerate() {
                assert_eq!(
                    event.cause().actor(),
                    Some(CauseActor::Unit(
                        UnitId::new(u64::try_from(index + 1).unwrap()).unwrap()
                    ))
                );
                assert_eq!(event.cause().owner(), event.cause().applier());
                assert_eq!(event.cause().action(), declared.cause().action());
                let BattleEventKind::Damage(data) = event.kind() else {
                    unreachable!()
                };
                assert_eq!(data.element, Some(CombatElement::Physical));
                assert_eq!(data.target.get(), 5);
                assert_eq!(
                    data.calculated.get(),
                    i64::try_from((index + 1) * 400).unwrap()
                );
                assert!(data.hp_after.get() >= 1);
            }
            assert_eq!(
                result
                    .events()
                    .iter()
                    .filter(|event| matches!(
                        event.kind(),
                        BattleEventKind::Action(ActionEventData::Declared { .. })
                    ))
                    .count(),
                1
            );
            assert_eq!(battle.view().rng_draw_count(), 0);
            if attack.enemy_hp == 7 {
                assert_eq!(
                    battle
                        .view()
                        .units_by_id()
                        .find(|unit| unit.side() == TeamSide::Enemy)
                        .unwrap()
                        .current_hp()
                        .get(),
                    1
                );
            }
            let next = first_attack(&mut battle);
            let repeated = retaliations(next.events());
            assert_eq!(repeated.len(), 2);
            assert_ne!(repeated[0].cause().action(), declared.cause().action());
        }
    }
}

#[test]
fn weighted_curio_retaliation_rejects_nonattack_damage_and_inactive_owners_without_recursive_damage()
 {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        for (attack, expected) in [
            (
                Attack {
                    tagged: false,
                    ..Attack::default()
                },
                0,
            ),
            (
                Attack {
                    class: DamageClass::Dot,
                    ..Attack::default()
                },
                0,
            ),
            (
                Attack {
                    class: DamageClass::Additional,
                    ..Attack::default()
                },
                0,
            ),
            (
                Attack {
                    class: DamageClass::Elation,
                    ..Attack::default()
                },
                0,
            ),
            (
                Attack {
                    dead: true,
                    ..Attack::default()
                },
                1,
            ),
            (
                Attack {
                    linked: true,
                    ..Attack::default()
                },
                1,
            ),
            (
                Attack {
                    damage: 2_000,
                    ..Attack::default()
                },
                0,
            ),
        ] {
            let mut battle = probe(&assembled, attack, 1);
            let result = first_attack(&mut battle);
            assert_eq!(retaliations(result.events()).len(), expected);
        }
    }
}

#[test]
fn weighted_curio_retaliation_sampling_has_path_weights_physical_bonus_and_fresh_hashes() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        let mut mask = 0_u64;
        let mut all_seen = 0_u8;
        for seed in 0..64 {
            let attack = Attack {
                all: false,
                ..Attack::default()
            };
            let mut actual = probe(&assembled, attack, seed);
            let mut fresh = probe(&assembled, attack, seed);
            let first = first_attack(&mut actual);
            let replayed = first_attack(&mut fresh);
            assert_eq!(first.events(), replayed.events());
            assert_eq!(first.state_hash(), replayed.state_hash());
            assert_eq!(
                actual.view().rng_draw_count(),
                fresh.view().rng_draw_count()
            );
            let target = first
                .events()
                .iter()
                .find_map(|event| {
                    matches!(
                        event.kind(),
                        BattleEventKind::Action(ActionEventData::Declared { .. })
                    )
                    .then(|| event.cause().primary_target().unwrap().get())
                })
                .unwrap();
            let mut expected = DeterministicRng::from_seed(RngSeed::new([seed; 32]));
            let chosen = expected
                .choose_weighted(
                    DrawPurpose::AGGRO_TARGET,
                    &[162_500_000, 130_000_000, 150_000_000, 75_000_000],
                )
                .unwrap()
                .unwrap();
            assert_eq!(target, u64::from(chosen.index()) + 1);
            assert_eq!(actual.view().rng_draw_count(), expected.draw_count());
            all_seen |= 1_u8 << u8::try_from(target - 1).unwrap();
            if target <= 2 {
                mask |= 1_u64 << seed;
            }
            assert_eq!(retaliations(first.events()).len(), usize::from(target <= 2));
        }
        assert_eq!(all_seen, 0b1111);
        assert_eq!(mask, 0xc0ed_a1bd_41f1_6e4a);
    }
}

#[test]
fn weighted_curio_retaliation_preserves_fresh_handoffs_rejected_commands_and_unequip() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    let fresh = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let (fresh_flow, mut fresh_activity) = ready(&fresh, family);
        let runtime = fixture.factory().weighted_curio_runtime().unwrap();
        let selected = fixture
            .factory()
            .decision_catalog()
            .weighted_curio_retaliations()[0]
            .weighted_curio
            .clone();
        for (factory, run, state) in [
            (fixture.factory(), &flow, &mut activity),
            (fresh.factory(), &fresh_flow, &mut fresh_activity),
        ] {
            let hash = state.state_hash();
            factory
                .weighted_curio_runtime()
                .unwrap()
                .replace_accepted_loadout(
                    run,
                    state,
                    hash,
                    WeightedCurioSlotLimit::new(1).unwrap(),
                    std::slice::from_ref(&selected),
                )
                .unwrap();
        }
        let bytes = activity.canonical_state_bytes();
        assert_eq!(bytes, fresh_activity.canonical_state_bytes());
        let added = assemble(&fixture, &flow, &activity);
        let rebuilt = assemble(&fresh, &fresh_flow, &fresh_activity);
        assert_eq!(added.assembly_digest(), rebuilt.assembly_digest());
        let mut battle = probe(
            &added,
            Attack {
                all: false,
                ..Attack::default()
            },
            4,
        );
        let before = battle.state_hash();
        let draws = battle.view().rng_draw_count();
        assert!(
            battle
                .apply(Command::Concede {
                    decision: DecisionId::new(999).unwrap()
                })
                .is_err()
        );
        assert_eq!(battle.state_hash(), before);
        assert_eq!(battle.view().rng_draw_count(), draws);
        first_attack(&mut battle);
        assert_eq!(activity.canonical_state_bytes(), bytes);
        let actual = result(&fixture, &flow, &mut activity);
        let reproduced = result(&fresh, &fresh_flow, &mut fresh_activity);
        assert_eq!(actual.identity(), reproduced.identity());
        assert_eq!(actual.values(), reproduced.values());
        assert_eq!(
            activity.canonical_state_bytes(),
            fresh_activity.canonical_state_bytes()
        );
        let participants = actual
            .values()
            .iter()
            .filter_map(|value| match value {
                ProjectedValue::ParticipantState(state) => Some(state),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(participants.len(), 4);
        for (state, spec) in participants.iter().zip(
            added
                .battle_spec()
                .participants()
                .iter()
                .filter(|p| p.side() == TeamSide::Player),
        ) {
            assert_eq!(state.maximum_hp(), spec.combatant().maximum_hp());
            assert!(state.current_hp() <= state.maximum_hp());
        }
        // Unequip is tested at the next independent pre-battle handoff, not by
        // mutating a live battle or pretending equipment commands have a codec.
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
        assert!(
            runtime
                .replace_accepted_loadout(
                    &flow,
                    &mut activity,
                    hash,
                    WeightedCurioSlotLimit::new(1).unwrap(),
                    &[]
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
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
        let removed = assemble(&fixture, &flow, &activity);
        assert!(
            !removed
                .battle_spec()
                .participants()
                .iter()
                .any(|p| p.combatant().rule_bundles().contains(&id(BUNDLE)))
        );
        for player in removed
            .battle_spec()
            .participants()
            .iter()
            .filter(|p| p.side() == TeamSide::Player)
        {
            assert!(
                !player
                    .combatant()
                    .sources()
                    .iter()
                    .any(|s| s.definition().get() == SOURCE)
            );
        }
        let mut plain = probe(&removed, Attack::default(), 4);
        assert!(retaliations(first_attack(&mut plain).events()).is_empty());
    }
}
