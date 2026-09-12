//! Production passive bindings with controlled attacks; not a public acquisition proof.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::{ready, result},
        curio_battle_stats::assemble,
    },
};
use starclock_activity::{ActivityProgramDefinition, ActivityProgramId};
use starclock_combat::{
    AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattleSeed, BattleSpec,
    CombatantSpecDigest, Command, ConcedePolicy, Energy, FormationIndex, Hp, LifeState,
    ParticipantInitialState, ParticipantSource, ParticipantSpec, PresenceState, Ratio,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, Scalar, Speed, TeamResourceSpec, TeamSide,
    UnitLevel,
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
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
};

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
fn state() -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9069").unwrap()
}
fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

#[derive(Clone, Copy)]
struct Attack {
    hits: u16,
    damage: i64,
    class: DamageClass,
    shield: bool,
    anchor_down: bool,
    linked_presence: bool,
}
impl Default for Attack {
    fn default() -> Self {
        Self {
            hits: 2,
            damage: 10,
            class: DamageClass::Direct,
            shield: false,
            anchor_down: false,
            linked_presence: false,
        }
    }
}

fn probe(assembled: &DivergentUniverseAssembledBattle, attack: Attack) -> Battle {
    let mut builder = CombatCatalogBuilder::from_catalog(assembled.combat_catalog(), [31; 32]);
    let selector = id(0x7d01_0001);
    let program = id(0x7d02_0001);
    let ability = id(0x7d03_0001);
    let form = id(0x7d04_0001);
    let enemy = id(0x7d05_0001);
    let encounter = id(0x7d06_0001);
    builder.add_selector(SelectorDefinition::new(selector).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::All).unwrap(),
    ));
    builder.add_program(ProgramDefinition::new(
        program,
        Vec::new(),
        vec![selector],
        Vec::new(),
        Vec::new(),
    ));
    let hits = (0..attack.hits)
        .map(|index| {
            let mut operations = Vec::new();
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
    .unwrap();
    builder.add_ability(
        AbilityDefinition::new(ability, program, selector, Vec::new()).with_action(action),
    );
    builder.add_unit(UnitDefinition::new(form, vec![ability], Vec::new()));
    builder.add_enemy(EnemyDefinition::new(enemy, form, vec![ability]));
    builder.add_encounter(EncounterDefinition::new(encounter, vec![enemy], Vec::new()));
    let mut participants = assembled
        .battle_spec()
        .participants()
        .iter()
        .filter(|entry| entry.side() == TeamSide::Player)
        .enumerate()
        .map(|(index, original)| {
            let base = original.combatant();
            let down = attack.anchor_down && index == 0;
            ParticipantSpec::new(
                original.side(),
                original.formation(),
                original.source(),
                base.clone(),
            )
            .with_locked_combatant_digest(original.locked_combatant_digest())
            .with_initial_state(
                ParticipantInitialState::new(
                    Hp::new(if down { 0 } else { base.maximum_hp().get() / 2 }).unwrap(),
                    base.maximum_hp(),
                    Energy::ZERO,
                    base.maximum_energy(),
                    if down {
                        LifeState::Defeated
                    } else {
                        LifeState::Alive
                    },
                    if attack.linked_presence && index == 1 {
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
    let enemy_spec = ResolvedCombatantSpec::new(
        form,
        UnitLevel::new(80).unwrap(),
        Hp::new(100_000).unwrap(),
        Speed::from_scaled(1_000_000_000).unwrap(),
        ResolvedDefinitionBindings::new(vec![ability], Vec::new(), Vec::new()).unwrap(),
        CombatantSpecDigest::new([32; 32]).unwrap(),
    )
    .unwrap();
    participants.push(ParticipantSpec::new(
        TeamSide::Enemy,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::EncounterEnemy(enemy),
        enemy_spec,
    ));
    let spec = BattleSpec::new(
        AssemblyDigest::new([33; 32]).unwrap(),
        encounter,
        participants,
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(builder.build().unwrap(), spec, BattleSeed::new([34; 32])).unwrap()
}

fn first_attack(battle: &mut Battle) -> Vec<BattleEvent> {
    let mut events = Vec::new();
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
            .unwrap_or_else(|| Command::Advance {
                boundary: battle.view().action_boundary().unwrap().id(),
            });
        let resolution = battle.apply(command).unwrap();
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
        let attacked = resolution
            .events()
            .iter()
            .any(|event| matches!(event.kind(), BattleEventKind::Damage(_)));
        events.extend_from_slice(resolution.events());
        if attacked {
            return events;
        }
        assert!(
            !events
                .iter()
                .any(|event| matches!(event.kind(), BattleEventKind::Heal(_))),
            "no entry heal"
        );
    }
    panic!("fixture must execute an actual attack")
}

#[test]
fn curio_battle_reactions_heal_each_attacked_ally_once_per_action_and_replay() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let hash = activity.state_hash();
        curios
            .acquire_accepted_state(&mut activity, hash, &state())
            .unwrap();
        let assembled = assemble(&fixture, &flow, &activity);
        for attack in [
            Attack::default(),
            Attack {
                shield: true,
                ..Attack::default()
            },
            Attack {
                anchor_down: true,
                ..Attack::default()
            },
            // A controlled linked-presence target tests the selector, not a
            // production summon/linked-entity construction claim.
            Attack {
                linked_presence: true,
                ..Attack::default()
            },
        ] {
            let mut battle = probe(&assembled, attack);
            let mut fresh = probe(&assembled, attack);
            let before = battle
                .view()
                .units_by_id()
                .filter(|unit| unit.side() == TeamSide::Player && unit.life() == LifeState::Alive)
                .map(|unit| (unit.id(), unit.current_hp().get(), unit.maximum_hp().get()))
                .collect::<Vec<_>>();
            let events = first_attack(&mut battle);
            assert_eq!(events, first_attack(&mut fresh));
            assert_eq!(battle.state_hash(), fresh.state_hash());
            for (unit, hp, maximum) in before {
                let heals = events
                    .iter()
                    .filter_map(|event| match event.kind() {
                        BattleEventKind::Heal(heal) if heal.target == unit => Some((event, heal)),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                assert_eq!(heals.len(), 1, "multi-hit attacks heal each target once");
                let (event, heal) = heals[0];
                assert_eq!(heal.calculated.get(), maximum / 5);
                assert_eq!(
                    heal.hp_before.get(),
                    hp - if attack.shield { 0 } else { attack.damage },
                    "heal occurs after the first eligible hit, not after the complete multi-hit action"
                );
                assert_eq!(
                    event.cause().source_definition().unwrap().get(),
                    0x7e34_0001
                );
                assert_eq!(
                    battle
                        .view()
                        .units_by_id()
                        .find(|entry| entry.id() == unit)
                        .unwrap()
                        .current_hp()
                        .get(),
                    hp + maximum / 5
                        - if attack.shield {
                            0
                        } else {
                            i64::from(attack.hits) * attack.damage
                        }
                );
            }
            let next = first_attack(&mut battle);
            assert_eq!(next, first_attack(&mut fresh));
            assert_eq!(
                next.iter()
                    .filter(|event| matches!(event.kind(), BattleEventKind::Heal(_)))
                    .count(),
                if attack.anchor_down { 3 } else { 4 },
                "the next action receives fresh once-per-target allowance"
            );
        }
    }
}

#[test]
fn curio_battle_reactions_do_not_heal_dot_additional_or_lethal_damage() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let (flow, mut activity) = ready(&fixture, FAMILIES[0]);
    let curios = fixture.factory().curio_runtime().unwrap();
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(&mut activity, hash, &state())
        .unwrap();
    let assembled = assemble(&fixture, &flow, &activity);
    for attack in [
        Attack {
            class: DamageClass::Dot,
            ..Attack::default()
        },
        Attack {
            class: DamageClass::Additional,
            ..Attack::default()
        },
        Attack {
            damage: 1_000_000,
            ..Attack::default()
        },
    ] {
        let events = first_attack(&mut probe(&assembled, attack));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event.kind(), BattleEventKind::Heal(_)))
        );
    }
}

#[test]
fn curio_battle_reactions_destroy_repair_and_verified_lifetime_teardown() {
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
            .destroy_accepted(&mut activity, hash, &state())
            .unwrap();
        assert!(
            curios
                .battle_lifetime_operations(&activity.player_view())
                .unwrap()
                .is_empty()
        );
        assert!(
            !first_attack(&mut probe(
                &assemble(&fixture, &flow, &activity),
                Attack::default()
            ))
            .iter()
            .any(|event| matches!(event.kind(), BattleEventKind::Heal(_)))
        );
        let hash = activity.state_hash();
        curios
            .repair_accepted(&mut activity, hash, &state())
            .unwrap();
        assert!(
            first_attack(&mut probe(
                &assemble(&fixture, &flow, &activity),
                Attack::default()
            ))
            .iter()
            .any(|event| matches!(event.kind(), BattleEventKind::Heal(_)))
        );
        let hash = activity.state_hash();
        curios
            .set_accepted_charges(&mut activity, hash, &state(), 0)
            .unwrap();
        assert!(
            !first_attack(&mut probe(
                &assemble(&fixture, &flow, &activity),
                Attack::default()
            ))
            .iter()
            .any(|event| matches!(event.kind(), BattleEventKind::Heal(_)))
        );
        let hash = activity.state_hash();
        curios
            .set_accepted_charges(&mut activity, hash, &state(), 1)
            .unwrap();
        let actual = result(&fixture, &flow, &mut activity);
        let hash = activity.state_hash();
        fixture
            .factory()
            .battle_settlement_runtime()
            .settle_started_result(&flow, &mut activity, hash, actual, None)
            .unwrap();
        assert!(
            !curios
                .owned(&activity)
                .unwrap()
                .iter()
                .any(|held| held.state() == &state())
        );
    }
}

#[test]
fn curio_battle_reactions_five_count_vector_excludes_domains_and_reacquires() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let (_, mut activity) = ready(&fixture, FAMILIES[0]);
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(&mut activity, hash, &state())
        .unwrap();
    assert!(
        curios
            .domain_entry_operations(&activity.player_view())
            .unwrap()
            .is_empty()
    );
    for remaining in (1..=5).rev() {
        let held = curios.owned(&activity).unwrap();
        assert_eq!(
            held.iter()
                .find(|held| held.state() == &state())
                .unwrap()
                .charges(),
            remaining
        );
        // Controlled operation vector, not five battles through a public producer.
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
    assert!(
        !curios
            .owned(&activity)
            .unwrap()
            .iter()
            .any(|held| held.state() == &state())
    );
    let hash = activity.state_hash();
    assert!(
        curios
            .repair_accepted(&mut activity, hash, &state())
            .is_err()
    );
    curios
        .acquire_accepted_state(&mut activity, hash, &state())
        .unwrap();
    assert_eq!(
        curios
            .owned(&activity)
            .unwrap()
            .iter()
            .find(|held| held.state() == &state())
            .unwrap()
            .charges(),
        5
    );
}
