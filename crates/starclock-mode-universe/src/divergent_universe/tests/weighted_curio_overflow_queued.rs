//! Actor-local collector state across real queued action envelopes.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    tests::weighted_curio_overflow_fixture::{Probe, accept, cast, id, scenario, start},
};
use starclock_combat::{
    ActionEventData, Battle, BattleEvent, BattleEventKind, CauseActor, Command, DecisionId, UnitId,
    catalog::action::{ReactionBoundary, TargetPattern},
};
use starclock_replay::battle_event::encode_battle_event_payload;

fn amounts(events: &[BattleEvent]) -> Vec<i64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(data.calculated.get()),
            _ => None,
        })
        .collect()
}
fn finish_child(battle: &mut Battle, mut events: Vec<BattleEvent>) -> Vec<BattleEvent> {
    for _ in 0..32 {
        if events.iter().any(|event| matches!(event.kind(),
            BattleEventKind::Action(ActionEventData::Resolved { ability, .. }) if ability.get() == 8)) {
            return events;
        }
        let command = battle.advance_command().unwrap();
        events.extend(accept(battle, command));
    }
    panic!("same-owner follow-up did not resolve");
}

#[test]
fn weighted_curio_overflow_nested_single_target_cannot_convert_into_outer_only_marked_enemy() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let probe = Probe {
        hp: vec![400, 9_000],
        levels: vec![1, 50],
        damages: vec![250, 250],
        pattern: TargetPattern::All,
        queued: Some(ReactionBoundary::AfterHit),
        queued_single: true,
        ..Probe::default()
    };
    let mut a = scenario(&fixture, &probe);
    let mut b = scenario(&fixture, &probe);
    start(&mut a);
    start(&mut b);
    let x = cast(&mut a, 1, None);
    let y = cast(&mut b, 1, None);
    let x = finish_child(&mut a, x);
    let y = finish_child(&mut b, y);
    assert_eq!(x, y);
    assert_eq!(a.state_hash(), b.state_hash());
    // The child kills its only committed target with 100 excess. The 8,750-HP
    // enemy was marked by the suspended outer action, not by the child, and
    // must never become the child's conversion recipient. The resumed outer
    // hit deals 250 but confirms no death of its own.
    assert_eq!(amounts(&x), [250, 250, 250, 250]);
    assert_eq!(a.view().rng_draw_count(), 0);
    assert_eq!(
        a.view()
            .units_by_id()
            .map(|u| u.current_hp().get())
            .collect::<Vec<_>>(),
        [10_000, 0, 8_500]
    );
    assert_eq!(
        a.view()
            .effects_by_id()
            .filter(|e| (0x7f05_0000..0x7f06_0000).contains(&e.definition().get()))
            .count(),
        0
    );
}

#[test]
fn weighted_curio_overflow_nested_after_hit_reopens_outer_collection_without_carrying_inner_total()
{
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for boundary in [ReactionBoundary::AfterHit, ReactionBoundary::AfterAction] {
        let probe = Probe {
            hp: vec![600, 400, 9_000],
            levels: vec![1, 81, 50],
            damages: vec![250, 250],
            pattern: TargetPattern::All,
            queued: Some(boundary),
            ..Probe::default()
        };
        let mut a = scenario(&fixture, &probe);
        let mut b = scenario(&fixture, &probe);
        start(&mut a);
        start(&mut b);
        let hash = a.state_hash();
        assert!(
            a.apply(Command::UseAbility {
                decision: DecisionId::new(999_999).unwrap(),
                actor: UnitId::new(1).unwrap(),
                ability: id(1),
                primary_target: None,
            })
            .is_err()
        );
        assert_eq!(a.state_hash(), hash);
        assert_eq!(a.view().rng_draw_count(), 0);
        let x = cast(&mut a, 1, None);
        let y = cast(&mut b, 1, None);
        let x = finish_child(&mut a, x);
        let y = finish_child(&mut b, y);
        assert_eq!(x, y);
        assert_eq!(
            x.iter()
                .map(|e| encode_battle_event_payload(e).unwrap())
                .collect::<Vec<_>>(),
            y.iter()
                .map(|e| encode_battle_event_payload(e).unwrap())
                .collect::<Vec<_>>()
        );
        assert_eq!(a.state_hash(), b.state_hash());
        // Both orders kill the 400-HP enemy with 100 excess, then the 600-HP
        // enemy with 150 excess. The survivor's own level is 50, so the explicit
        // fixture base contributes 10 * (50 * 2) = 1,000 each time. Neither
        // total may become 1,250 by retaining the other envelope's excess.
        assert_eq!(
            amounts(&x),
            [250, 250, 250, 250, 250, 250, 1_100, 250, 250, 1_150]
        );
        let envelopes = x
            .iter()
            .filter_map(|e| match e.kind() {
                BattleEventKind::Action(ActionEventData::Started { ability, .. }) => {
                    Some((true, ability.get()))
                }
                BattleEventKind::Action(ActionEventData::Resolved { ability, .. }) => {
                    Some((false, ability.get()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            envelopes,
            if boundary == ReactionBoundary::AfterHit {
                vec![(true, 1), (true, 8), (false, 8), (false, 1)]
            } else {
                vec![(true, 1), (false, 1), (true, 8), (false, 8)]
            }
        );
        assert!(
            x.iter()
                .filter(|e| matches!(e.kind(), BattleEventKind::Damage(_)))
                .all(|e| e.cause().actor() == Some(CauseActor::Unit(UnitId::new(1).unwrap())))
        );
        let actions = x
            .iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::Action(ActionEventData::Started { ability, .. }) => {
                    Some((ability.get(), event.cause().action().unwrap()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(actions.len(), 2);
        assert_eq!([actions[0].0, actions[1].0], [1, 8]);
        assert_ne!(actions[0].1, actions[1].1);
        let conversions = x
            .iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::Damage(data) if data.calculated.get() > 250 => {
                    assert_eq!(event.cause().owner(), Some(UnitId::new(1).unwrap()));
                    Some(event.cause().action().unwrap())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            conversions,
            if boundary == ReactionBoundary::AfterHit {
                [actions[1].1, actions[0].1]
            } else {
                [actions[0].1, actions[1].1]
            }
        );
        assert_eq!(a.view().rng_draw_count(), 2);
        assert_eq!(
            a.view()
                .units_by_id()
                .map(|u| u.current_hp().get())
                .collect::<Vec<_>>(),
            [10_000, 0, 0, 6_000]
        );
        assert_eq!(
            a.view()
                .effects_by_id()
                .filter(|e| (0x7f05_0000..0x7f06_0000).contains(&e.definition().get()))
                .count(),
            0
        );
    }
}
