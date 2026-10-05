//! Actual periodic damage and wave commands over both authored equipment clauses.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    tests::{
        weighted_curio_footstep_damage_fixture::{BREAK_IN, CHANNELS, DOT_IN, DOT_OUT, KILL},
        weighted_curio_footstep_equipment_fixture::{
            HEAL, LOSS, cast, probe_with_consumers, progress, stacks,
        },
        weighted_curio_footstep_fixture::remainder,
        weighted_curio_footstep_lifecycle::{equipped, finish_loss, repeated, stale_is_inert},
        weighted_curio_overflow_fixture::accept,
    },
};
use starclock_combat::{
    Battle, BattleEvent, BattleEventKind, BattlePhase, BreakDamageKind, CauseActor, Command,
    DamageKind, ResourceEventData, Scalar, TeamSide, WaveEventData, formula::model::DamageClass,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];

fn targeted(battle: &mut Battle, owner: u64, raw: u32, target: u64) -> Vec<BattleEvent> {
    let mut events = vec![];
    for _ in 0..64 {
        let offered =
            battle.decision().and_then(|decision| {
                decision.legal_commands().iter().find(|command| matches!(command,
                Command::UseAbility {actor, ability, primary_target: Some(unit), ..}
                    if actor.get() == owner && ability.get() == raw && unit.get() == target
            )).cloned()
            });
        if let Some(command) = offered {
            events.extend(accept(battle, command));
            return events;
        }
        events.extend(progress(battle));
    }
    panic!("consumer command not offered: {owner}/{raw}/{target}");
}

fn ticks(events: &[BattleEvent], target: u64) -> Vec<i64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data)
                if data.kind == DamageKind::DotTick && data.target.get() == target =>
            {
                assert_eq!(data.class, DamageClass::Dot);
                assert!(data.source_effect.is_some());
                Some(data.applied.get())
            }
            _ => None,
        })
        .collect()
}

fn collect_ticks(battle: &mut Battle, events: &mut Vec<BattleEvent>, target: u64) {
    for _ in 0..64 {
        if ticks(events, target).len() >= 2 {
            return;
        }
        events.extend(progress(battle));
    }
    panic!("two real periodic ticks were not reached");
}

#[test]
fn weighted_curio_footstep_equipment_periodic_dot_uses_live_original_applier_bonus() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1107, 1402, 1009, 1002])
            .unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for owner in [1, 2, 4] {
            let run = || {
                let mut battle = probe_with_consumers(&source, false);
                // Apply before acquiring stacks: actual target-turn ticks must
                // read the live original applier, not the ticking enemy actor.
                let mut events = targeted(&mut battle, owner, DOT_OUT, 5);
                events.extend(targeted(&mut battle, owner, HEAL, owner));
                collect_ticks(&mut battle, &mut events, 5);
                let packets = events
                    .iter()
                    .filter(|event| {
                        matches!(event.kind(),
                            BattleEventKind::Damage(data) if data.kind == DamageKind::DotTick
                        )
                    })
                    .collect::<Vec<_>>();
                assert_eq!(packets.len(), 2);
                assert!(packets.iter().all(|event| {
                    event
                        .cause()
                        .applier()
                        .is_some_and(|unit| unit.get() == owner)
                }));
                assert_eq!(
                    ticks(&events, 5),
                    if owner < 3 {
                        vec![100, 108]
                    } else {
                        vec![100, 100]
                    }
                );
                assert_eq!(stacks(&battle, owner), u16::from(owner < 3));
                assert_eq!(remainder(&battle, owner), Scalar::ZERO);
                assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 0);
                assert_eq!(battle.view().rng_draw_count(), 0);
                stale_is_inert(&mut battle);
                let payload = repeated(&events);
                finish_loss(&mut battle);
                assert_eq!(stacks(&battle, owner), 0);
                (events, payload, battle.state_hash())
            };
            assert_eq!(run(), run(), "{family:?}/{owner}");
        }
    }
}

#[test]
fn weighted_curio_footstep_equipment_periodic_dot_and_real_break_losses_return_points() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1107, 1402, 1009, 1002])
            .unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for owner in [1, 2, 4] {
            for periodic in [false, true] {
                let run = || {
                    let mut battle = probe_with_consumers(&source, false);
                    let mut events = cast(&mut battle, owner, LOSS);
                    events.extend(cast(&mut battle, owner, HEAL));
                    // A noneligible original applies damage to the eligible
                    // victim; source attribution must not become the receiver.
                    events.extend(targeted(
                        &mut battle,
                        3,
                        if periodic { DOT_IN } else { BREAK_IN },
                        owner,
                    ));
                    if periodic {
                        collect_ticks(&mut battle, &mut events, owner);
                        assert_eq!(ticks(&events, owner), [20, 20]);
                        assert!(events.iter().filter(|event| matches!(event.kind(),
                            BattleEventKind::Damage(data) if data.kind == DamageKind::DotTick
                        )).all(|event| event.cause().applier().is_some_and(|unit| unit.get() == 3)));
                    } else {
                        let packets = events
                            .iter()
                            .filter_map(|event| match event.kind() {
                                BattleEventKind::BreakDamage(data) => {
                                    assert!(matches!(event.cause().actor(), Some(CauseActor::Unit(unit)) if unit.get() == 3));
                                    Some((data.kind, data.applied.get()))
                                }
                                _ => None,
                            })
                            .collect::<Vec<_>>();
                        assert_eq!(
                            packets,
                            [
                                (BreakDamageKind::Initial, 25),
                                (BreakDamageKind::SuperBreak, 15)
                            ]
                        );
                    }
                    let eligible = owner < 3;
                    let gains = events
                        .iter()
                        .filter(|event| {
                            matches!(
                                event.kind(),
                                BattleEventKind::Resource(ResourceEventData::SkillPoints {
                                    side: TeamSide::Player,
                                    attempted: 1,
                                    effective: 1,
                                    overflow: 0,
                                    ..
                                })
                            )
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(gains.len(), usize::from(eligible));
                    assert_eq!(
                        battle.view().team(TeamSide::Player).skill_points(),
                        u16::from(eligible)
                    );
                    assert_eq!(
                        remainder(&battle, owner),
                        if eligible {
                            Scalar::checked_from_integer(20).unwrap()
                        } else {
                            Scalar::ZERO
                        }
                    );
                    assert_eq!(stacks(&battle, owner), if eligible { 2 } else { 0 });
                    assert_eq!(stacks(&battle, 3), 0);
                    assert_eq!(battle.view().rng_draw_count(), 0);
                    stale_is_inert(&mut battle);
                    let payload = repeated(&events);
                    finish_loss(&mut battle);
                    assert_eq!(remainder(&battle, owner), Scalar::ZERO);
                    assert_eq!(stacks(&battle, owner), 0);
                    (events, payload, battle.state_hash())
                };
                assert_eq!(run(), run(), "{family:?}/{owner}/{periodic}");
            }
        }
    }
}

#[test]
fn weighted_curio_footstep_equipment_actual_damage_channels_keep_break_and_elation_separate() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1107, 1402, 1009, 1002])
            .unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for owner in [1, 2, 4] {
            for layered in [false, true] {
                let run = || {
                    let mut battle = probe_with_consumers(&source, false);
                    let mut events = vec![];
                    if layered {
                        for _ in 0..3 {
                            events.extend(cast(&mut battle, owner, HEAL));
                        }
                    }
                    let attack = targeted(&mut battle, owner, CHANNELS, 5);
                    let ordinary = attack
                        .iter()
                        .filter_map(|event| match event.kind() {
                            BattleEventKind::Damage(data) => Some((data.class, data.applied.get())),
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    let boosted = if layered && owner < 3 { 124 } else { 100 };
                    assert_eq!(
                        ordinary,
                        [
                            (DamageClass::Direct, boosted),
                            (DamageClass::Additional, boosted),
                            (DamageClass::Elation, 100)
                        ]
                    );
                    let breaks = attack
                        .iter()
                        .filter_map(|event| match event.kind() {
                            BattleEventKind::BreakDamage(data) => {
                                Some((data.kind, data.applied.get()))
                            }
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(
                        breaks,
                        [
                            (BreakDamageKind::Initial, 25),
                            (BreakDamageKind::SuperBreak, 15)
                        ]
                    );
                    assert_eq!(
                        stacks(&battle, owner),
                        if layered && owner < 3 { 3 } else { 0 }
                    );
                    events.extend(attack);
                    assert_eq!(battle.view().rng_draw_count(), 0);
                    stale_is_inert(&mut battle);
                    let payload = repeated(&events);
                    finish_loss(&mut battle);
                    assert_eq!(stacks(&battle, owner), 0);
                    (events, payload, battle.state_hash())
                };
                assert_eq!(run(), run(), "{family:?}/{owner}/{layered}");
            }
        }
    }
}

#[test]
fn weighted_curio_footstep_equipment_real_wave_retains_hp_residue_and_skill_stacks_until_win() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1107, 1402, 1009, 1002])
            .unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for owner in [1, 2, 4] {
            let run = || {
                let mut battle = probe_with_consumers(&source, true);
                let mut events = cast(&mut battle, owner, LOSS);
                events.extend(cast(&mut battle, owner, HEAL));
                events.extend(targeted(&mut battle, owner, KILL, 5));
                for _ in 0..64 {
                    if events.iter().any(|event| {
                        matches!(
                            event.kind(),
                            BattleEventKind::Wave(WaveEventData::Started { number: 2, .. })
                        )
                    }) {
                        break;
                    }
                    events.extend(progress(&mut battle));
                }
                assert!(events.iter().any(|event| matches!(
                    event.kind(),
                    BattleEventKind::Wave(WaveEventData::Ended { number: 1, .. })
                )));
                assert!(events.iter().any(|event| matches!(
                    event.kind(),
                    BattleEventKind::Wave(WaveEventData::Started { number: 2, .. })
                )));
                let eligible = owner < 3;
                assert_eq!(stacks(&battle, owner), if eligible { 2 } else { 0 });
                assert_eq!(
                    remainder(&battle, owner),
                    if eligible {
                        Scalar::checked_from_integer(30).unwrap()
                    } else {
                        Scalar::ZERO
                    }
                );
                events.extend(cast(&mut battle, owner, LOSS));
                assert_eq!(
                    battle.view().team(TeamSide::Player).skill_points(),
                    u16::from(eligible)
                );
                assert_eq!(
                    remainder(&battle, owner),
                    if eligible {
                        Scalar::checked_from_integer(10).unwrap()
                    } else {
                        Scalar::ZERO
                    }
                );
                assert_eq!(stacks(&battle, owner), if eligible { 3 } else { 0 });
                let target = battle
                    .view()
                    .units_by_id()
                    .find(|unit| unit.side() == TeamSide::Enemy && unit.current_hp().get() > 0)
                    .unwrap()
                    .id()
                    .get();
                let hit = targeted(&mut battle, owner, CHANNELS, target);
                assert!(hit.iter().any(|event| matches!(event.kind(),
                    BattleEventKind::Damage(data) if data.class == DamageClass::Direct && data.applied.get() == if eligible { 124 } else { 100 }
                )));
                events.extend(hit);
                stale_is_inert(&mut battle);
                events.extend(targeted(&mut battle, owner, KILL, target));
                for _ in 0..64 {
                    if battle.view().phase() == BattlePhase::Won {
                        break;
                    }
                    events.extend(progress(&mut battle));
                }
                assert_eq!(battle.view().phase(), BattlePhase::Won);
                assert_eq!(battle.view().rng_draw_count(), 0);
                assert_eq!(stacks(&battle, owner), 0);
                assert_eq!(remainder(&battle, owner), Scalar::ZERO);
                let payload = repeated(&events);
                (events, payload, battle.state_hash())
            };
            assert_eq!(run(), run(), "{family:?}/{owner}");
        }
    }
}
