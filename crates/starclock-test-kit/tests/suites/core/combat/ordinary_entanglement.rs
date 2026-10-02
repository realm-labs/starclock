//! Real commands for ordinary resistible Entanglement, not a Weakness Break.

#[path = "ordinary_entanglement/fixture.rs"]
mod fixture;

use crate::combat_decision::settle_ready_boundaries;
use fixture::{Inputs, battle, play, start};
use starclock_combat::{
    ActionGaugeChangeKind, BattleEvent, BattleEventKind, BattlePhase, Command, EffectEventData,
    FaultKind, FaultPolicy, Ratio, Scalar, TurnEventData, formula::model::CombatElement,
};
use starclock_replay::battle_event::encode_battle_event_payload;

#[test]
fn ordinary_entanglement_refreshes_across_casters_caps_hits_and_expires_without_break_or_skip() {
    let mut battle = battle(Inputs::default(), 0xa3);
    start(&mut battle);
    let applied = play(&mut battle, 1, 1);
    let effect = battle.view().effects_by_id().next().unwrap();
    let effect_id = effect.id();
    assert_eq!(effect.entanglement_hits(), Some(0));
    assert_eq!(
        effect.entanglement_base(),
        Some(Scalar::checked_from_integer(120).unwrap())
    );
    assert_eq!(
        effect.entanglement_delay(),
        Some(Ratio::from_scaled(200_000))
    );
    assert!(applied.iter().any(
        |event| matches!(event.kind(), BattleEventKind::Turn(TurnEventData::ActionGaugeChanged {
        kind: ActionGaugeChangeKind::Delay, amount, before, after, ..
    }) if amount.scaled() == 200_000 && after.scaled() - before.scaled() == 2_000_000_000)
    ));
    assert!(applied.iter().all(|event| !matches!(
        event.kind(),
        BattleEventKind::BreakDamage(_) | BattleEventKind::Toughness(_)
    )));
    settle_ready_boundaries(&mut battle);
    let refreshed = play(&mut battle, 2, 1);
    let effect = battle.view().effects_by_id().next().unwrap();
    assert_eq!(effect.id(), effect_id);
    assert_eq!(effect.applier().get(), 1);
    assert_eq!(effect.entanglement_hits(), Some(0));
    assert!(refreshed.iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Effect(EffectEventData::Refreshed { .. })
    )));
    assert!(refreshed.iter().all(|event| !matches!(
        event.kind(),
        BattleEventKind::Turn(TurnEventData::ActionGaugeChanged { .. })
    )));
    settle_ready_boundaries(&mut battle);
    let hits = play(&mut battle, 1, 2);
    assert_eq!(
        battle
            .view()
            .effects_by_id()
            .next()
            .unwrap()
            .entanglement_hits(),
        Some(5)
    );
    let counters = hits
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Effect(EffectEventData::HitAccumulated { before, after, .. }) => {
                Some((*before, *after))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(counters, [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]);
    for event in &hits {
        if let BattleEventKind::Effect(EffectEventData::HitAccumulated {
            operation,
            effect,
            target,
            before,
            after,
        }) = event.kind()
        {
            let mut expected = vec![16, 6];
            expected.extend_from_slice(&operation.get().to_le_bytes());
            expected.extend_from_slice(&effect.get().to_le_bytes());
            expected.extend_from_slice(&target.get().to_le_bytes());
            expected.extend_from_slice(&[*before, *after]);
            let encoded = encode_battle_event_payload(event).unwrap();
            assert!(encoded.ends_with(&expected));
        }
    }
    let expired = settle_ready_boundaries(&mut battle);
    let damage = expired
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::BreakDamage(data) => Some((event.cause(), data)),
            _ => None,
        })
        .unwrap();
    assert_eq!(damage.0.applier().unwrap().get(), 1);
    assert_eq!(damage.1.element, CombatElement::Quantum);
    assert_eq!(damage.1.calculated.get(), 540);
    assert_eq!(battle.view().effects_by_id().count(), 0);
    assert_eq!(battle.view().retained_break_effects_by_id().count(), 0);
    let enemy = battle
        .view()
        .units_by_id()
        .find(|unit| unit.id().get() == 3)
        .unwrap();
    assert!(!enemy.weakness_broken());
    assert_eq!(
        enemy
            .toughness_layers()
            .map(|layer| layer.current().get())
            .collect::<Vec<_>>(),
        [1000, 60]
    );
    // Expiry is not action suppression: the enemy's ordinary action is offered.
    play(&mut battle, 3, 4);
}

#[test]
fn ordinary_entanglement_without_an_ordinary_bar_has_zero_maximum_not_exo_maximum() {
    let mut battle = battle(
        Inputs {
            ordinary_maximum: None,
            ..Inputs::default()
        },
        0xaa,
    );
    start(&mut battle);
    play(&mut battle, 1, 1);
    assert_eq!(
        battle
            .view()
            .effects_by_id()
            .next()
            .unwrap()
            .entanglement_base(),
        Some(Scalar::checked_from_integer(30).unwrap())
    );
    assert_eq!(battle.view().retained_break_effects_by_id().count(), 0);
}

#[test]
fn ordinary_entanglement_capture_overflow_rolls_back_before_effect_delay_or_damage() {
    let mut battle = battle(
        Inputs {
            break_effect: Scalar::from_scaled(i64::MAX),
            ..Inputs::default()
        },
        0xab,
    );
    start(&mut battle);
    let before_rng = battle.view().rng_draw_count();
    let events = play(&mut battle, 1, 1);
    assert_eq!(battle.view().phase(), BattlePhase::Faulted);
    let fault = battle.view().fault().unwrap();
    assert_eq!(fault.kind(), FaultKind::Numeric);
    assert_eq!(fault.policy(), FaultPolicy::Rollback);
    assert_eq!(battle.view().effects_by_id().count(), 0);
    assert_eq!(battle.view().rng_draw_count(), before_rng);
    assert!(events.iter().all(|event| !matches!(
        event.kind(),
        BattleEventKind::Effect(_)
            | BattleEventKind::BreakDamage(_)
            | BattleEventKind::Turn(TurnEventData::ActionGaugeChanged { .. })
    )));
}

#[test]
fn ordinary_entanglement_cleanse_removes_without_delayed_damage_and_stale_command_is_inert() {
    let mut battle = battle(Inputs::default(), 0xa4);
    start(&mut battle);
    let stale = battle.decision().unwrap().legal_commands()[0].clone();
    play(&mut battle, 1, 1);
    let before = battle.state_hash();
    assert!(battle.apply(stale).is_err());
    assert_eq!(battle.state_hash(), before);
    settle_ready_boundaries(&mut battle);
    let cleansed = play(&mut battle, 2, 3);
    assert_eq!(battle.view().effects_by_id().count(), 0);
    assert!(cleansed.iter().any(|event| matches!(
        event.kind(),
        BattleEventKind::Effect(EffectEventData::Removed { .. })
    )));
    let mut events = settle_ready_boundaries(&mut battle);
    events.extend(play(&mut battle, 1, 4));
    events.extend(settle_ready_boundaries(&mut battle));
    assert!(
        events
            .iter()
            .all(|event| !matches!(event.kind(), BattleEventKind::BreakDamage(_)))
    );
    play(&mut battle, 3, 4);
}

#[test]
fn ordinary_entanglement_rule_ir_resolves_hit_rate_effect_res_and_template_control_resistance() {
    for (hit_rate, resistance, control_resistance, expected, draws) in [
        (Scalar::ONE, Scalar::ZERO, Scalar::ZERO, true, 0),
        (Scalar::ONE, Scalar::ONE, Scalar::ZERO, false, 0),
        (Scalar::ONE, Scalar::ZERO, Scalar::ONE, false, 0),
    ] {
        let input = Inputs {
            hit_rate,
            resistance,
            control_resistance,
            ..Inputs::default()
        };
        let mut battle = battle(input, 0xa5);
        start(&mut battle);
        let before = battle.view().rng_draw_count();
        let events = play(&mut battle, 1, 1);
        assert_eq!(battle.view().effects_by_id().count(), usize::from(expected));
        assert_eq!(battle.view().rng_draw_count() - before, draws);
        if !expected {
            assert!(events.iter().any(|event| matches!(event.kind(), BattleEventKind::Effect(EffectEventData::Resisted { pre_clamp_chance, .. }) if *pre_clamp_chance == Scalar::ZERO)));
            assert!(events.iter().all(|event| !matches!(
                event.kind(),
                BattleEventKind::Turn(TurnEventData::ActionGaugeChanged { .. })
            )));
        }
    }
    let mut outcomes = [false; 2];
    for seed in 1..=16 {
        let mut battle = battle(
            Inputs {
                hit_rate: Scalar::ZERO,
                ..Inputs::default()
            },
            seed,
        );
        start(&mut battle);
        let before = battle.view().rng_draw_count();
        play(&mut battle, 1, 1);
        assert_eq!(battle.view().rng_draw_count() - before, 1);
        outcomes[battle.view().effects_by_id().count()] = true;
    }
    assert_eq!(outcomes, [true, true]);
}

#[test]
fn ordinary_entanglement_initial_hit_is_excluded_and_live_break_effect_changes_only_initial_delay_and_expiry_factors()
 {
    let mut battle = battle(
        Inputs {
            break_effect: Scalar::from_scaled(500_000),
            application_damage: true,
            ..Inputs::default()
        },
        0xa6,
    );
    start(&mut battle);
    play(&mut battle, 1, 1);
    let effect = battle.view().effects_by_id().next().unwrap();
    assert_eq!(effect.entanglement_hits(), Some(0));
    assert_eq!(
        effect.entanglement_base(),
        Some(Scalar::checked_from_integer(120).unwrap())
    );
    assert_eq!(
        effect.entanglement_delay(),
        Some(Ratio::from_scaled(300_000))
    );
    settle_ready_boundaries(&mut battle);
    play(&mut battle, 2, 2);
    settle_ready_boundaries(&mut battle);
    play(&mut battle, 1, 4);
    let expired = settle_ready_boundaries(&mut battle);
    let calculated = expired
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::BreakDamage(data) => Some(data.calculated.get()),
            _ => None,
        })
        .unwrap();
    assert_eq!(calculated, 810);
}

#[test]
fn ordinary_entanglement_fresh_command_reconstruction_preserves_events_hashes_rng_and_hits() {
    let input = Inputs::default();
    let mut original = battle(input, 0xa7);
    let mut fresh = battle(input, 0xa7);
    for (actor, ability) in [(0, 0), (1, 1), (0, 0), (2, 1), (0, 0), (1, 2), (0, 0)] {
        let command = if original.view().committed_revision() == 0 {
            Command::StartBattle {
                decision: original.decision().unwrap().id(),
            }
        } else if actor == 0 {
            original.advance_command().unwrap()
        } else {
            original
                .decision()
                .unwrap()
                .legal_commands()
                .iter()
                .find(|command| {
                    matches!(command,
                Command::UseAbility { actor: candidate, ability: candidate_ability, .. }
                if candidate.get() == actor && candidate_ability.get() == ability)
                })
                .unwrap()
                .clone()
        };
        let expected = original.apply(command.clone()).unwrap();
        let reconstructed = fresh.apply(command).unwrap();
        assert_eq!(expected.events(), reconstructed.events());
        assert_eq!(expected.state_hash(), reconstructed.state_hash());
        assert_eq!(original.state_hash(), fresh.state_hash());
        assert_eq!(
            original.view().rng_draw_count(),
            fresh.view().rng_draw_count()
        );
        let payloads = |events: &[BattleEvent]| {
            events
                .iter()
                .map(|event| encode_battle_event_payload(event).unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            payloads(expected.events()),
            payloads(reconstructed.events())
        );
    }
}

#[test]
fn ordinary_entanglement_provider_defeat_retains_captured_credit_and_damaging_expiry() {
    let input = Inputs {
        break_effect: Scalar::checked_from_integer(4).unwrap(),
        ..Inputs::default()
    };
    let mut battle = battle(input, 0xa8);
    start(&mut battle);
    play(&mut battle, 1, 1);
    settle_ready_boundaries(&mut battle);
    play(&mut battle, 2, 5);
    assert_eq!(
        battle
            .view()
            .units_by_id()
            .find(|unit| unit.id().get() == 1)
            .unwrap()
            .current_hp()
            .get(),
        0
    );
    assert_eq!(
        battle
            .view()
            .effects_by_id()
            .next()
            .unwrap()
            .applier()
            .get(),
        1
    );
    settle_ready_boundaries(&mut battle);
    play(&mut battle, 2, 2);
    let mut events = settle_ready_boundaries(&mut battle);
    // Equal-time player formation precedes the delayed enemy at 200 AV.
    if battle.view().effects_by_id().count() != 0 {
        events.extend(play(&mut battle, 2, 4));
        events.extend(settle_ready_boundaries(&mut battle));
    }
    let damage = events
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::BreakDamage(data) => Some((event.cause(), data)),
            _ => None,
        })
        .unwrap();
    assert_eq!(damage.0.applier().unwrap().get(), 1);
    assert_eq!(damage.1.calculated.get(), 2700);
    assert_eq!(battle.view().effects_by_id().count(), 0);
}

#[test]
fn ordinary_entanglement_longer_duration_ticks_without_periodic_damage_then_settles_once() {
    let mut battle = battle(
        Inputs {
            duration: 2,
            ..Inputs::default()
        },
        0xa9,
    );
    start(&mut battle);
    play(&mut battle, 1, 1);
    settle_ready_boundaries(&mut battle);
    play(&mut battle, 2, 2);
    settle_ready_boundaries(&mut battle);
    let refreshed = play(&mut battle, 1, 1);
    assert_eq!(
        battle
            .view()
            .effects_by_id()
            .next()
            .unwrap()
            .entanglement_hits(),
        Some(5)
    );
    assert!(refreshed.iter().all(|event| !matches!(
        event.kind(),
        BattleEventKind::Turn(TurnEventData::ActionGaugeChanged { .. })
    )));
    let mut events = settle_ready_boundaries(&mut battle);
    assert_eq!(
        battle.view().effects_by_id().next().unwrap().remaining(),
        Some(1)
    );
    assert!(
        events
            .iter()
            .all(|event| !matches!(event.kind(), BattleEventKind::BreakDamage(_)))
    );
    for _ in 0..16 {
        if battle.view().effects_by_id().count() == 0 {
            break;
        }
        let command = battle
            .decision()
            .unwrap()
            .legal_commands()
            .iter()
            .find(|command| {
                matches!(command,
            Command::UseAbility { ability, .. } if ability.get() == 4)
            })
            .unwrap()
            .clone();
        events.extend_from_slice(battle.apply(command).unwrap().events());
        events.extend(settle_ready_boundaries(&mut battle));
    }
    assert_eq!(battle.view().effects_by_id().count(), 0);
    let damage = events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::BreakDamage(data) => Some(data.calculated.get()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(damage, [540]);
}
