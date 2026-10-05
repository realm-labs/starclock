//! Explicit-constructor ATK queries through commands; normal equipment has separate probes.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    tests::{
        weighted_curio_overflow_fixture::{Probe, accept, cast_ability, id, scenario, start},
        weighted_curio_overflow_lifecycle_fixture::Setup,
    },
};
use starclock_build::light_cone::CombatPath;
use starclock_combat::catalog::action::TargetPattern;
use starclock_combat::{
    Battle, BattleEvent, BattleEventKind, Command, DecisionId, LinkedEntityKind, UnitEventData,
    UnitId,
};
use starclock_replay::battle_event::encode_battle_event_payload;

fn form(fixture: &DivergentUniverseBaselineFixture, path: CombatPath) -> u32 {
    let catalog = fixture.core().build_catalog();
    catalog
        .character_ids()
        .find(|id| ![2, 3].contains(&id.get()) && catalog.character(*id).unwrap().path() == path)
        .unwrap()
        .get()
}
fn probe(fixture: &DivergentUniverseBaselineFixture, path: CombatPath) -> Probe {
    Probe {
        player_form: form(fixture, path),
        attack_probe: true,
        hp: vec![100_000, 100_000, 100_000, 100_000],
        ..Probe::default()
    }
}
fn attack_effects(battle: &Battle) -> Vec<u64> {
    battle
        .view()
        .effects_by_id()
        .filter(|effect| (0x7f0e_0000..0x7f0f_0000).contains(&effect.definition().get()))
        .map(|effect| effect.target().get())
        .collect()
}
fn damage(events: &[BattleEvent]) -> Vec<i64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(data.calculated.get()),
            _ => None,
        })
        .collect()
}

#[test]
fn weighted_curio_overflow_attack_exact_paths_add_eighty_percent_and_reconstruct() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for (path, expected) in [
        (CombatPath::Destruction, 100),
        (CombatPath::Hunt, 180),
        (CombatPath::Erudition, 180),
        (CombatPath::Harmony, 100),
        (CombatPath::Nihility, 100),
        (CombatPath::Preservation, 100),
        (CombatPath::Abundance, 100),
        (CombatPath::Remembrance, 100),
        (CombatPath::Elation, 100),
    ] {
        let input = probe(&fixture, path);
        let mut a = scenario(&fixture, &input);
        let mut b = scenario(&fixture, &input);
        start(&mut a);
        start(&mut b);
        assert_eq!(
            attack_effects(&a),
            if expected == 180 { vec![1] } else { vec![] }
        );
        let before = a.state_hash();
        assert!(
            a.apply(Command::UseAbility {
                decision: DecisionId::new(999_999).unwrap(),
                actor: UnitId::new(1).unwrap(),
                ability: id(7),
                primary_target: UnitId::new(2),
            })
            .is_err()
        );
        assert_eq!(a.state_hash(), before);
        assert_eq!(a.view().rng_draw_count(), 0);
        let x = cast_ability(&mut a, 1, 7, Some(2));
        let y = cast_ability(&mut b, 1, 7, Some(2));
        assert_eq!(damage(&x), [expected]);
        assert_eq!(x, y);
        let payloads = |events: &[BattleEvent]| {
            events
                .iter()
                .map(|event| encode_battle_event_payload(event).unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(payloads(&x), payloads(&y));
        assert_eq!(a.state_hash(), b.state_hash());
        assert_eq!(a.view().rng_draw_count(), 0);
    }
}

#[test]
fn weighted_curio_overflow_attack_keeps_original_eligibility_through_transform_and_restore() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for (path, expected) in [(CombatPath::Hunt, 180), (CombatPath::Harmony, 100)] {
        let input = Probe {
            setup: Some(Setup::Transform),
            ..probe(&fixture, path)
        };
        let mut battle = scenario(&fixture, &input);
        start(&mut battle);
        cast_ability(&mut battle, 1, 3, None);
        let changed = cast_ability(&mut battle, 1, 7, Some(2));
        assert_eq!(damage(&changed), [expected]);
        cast_ability(&mut battle, 1, 4, None);
        let restored = cast_ability(&mut battle, 1, 7, Some(2));
        assert_eq!(damage(&restored), [expected]);
        assert_eq!(
            attack_effects(&battle),
            if expected == 180 { vec![1] } else { vec![] }
        );
        assert_eq!(battle.view().rng_draw_count(), 0);
    }
}

#[test]
fn weighted_curio_overflow_attack_linked_units_cannot_inherit_original_grant() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for kind in [
        LinkedEntityKind::Summon,
        LinkedEntityKind::Memosprite,
        LinkedEntityKind::SharedActor,
    ] {
        for formation in [0, 4] {
            let input = Probe {
                setup: Some(Setup::Linked(kind, formation)),
                pattern: TargetPattern::All,
                ..probe(&fixture, CombatPath::Hunt)
            };
            let mut battle = scenario(&fixture, &input);
            start(&mut battle);
            let mut events = cast_ability(&mut battle, 1, 3, None);
            let linked = events
                .iter()
                .find_map(|event| match event.kind() {
                    BattleEventKind::Unit(UnitEventData::Summoned {
                        unit,
                        kind: observed,
                        ..
                    }) if *observed == kind => Some(unit.get()),
                    _ => None,
                })
                .unwrap();
            assert_eq!(attack_effects(&battle), [1]);
            for _ in 0..32 {
                if events.iter().any(|event| {
                    matches!(event.kind(), BattleEventKind::Damage(_))
                        && event
                            .cause()
                            .source_definition()
                            .is_some_and(|id| id.get() == 6)
                }) {
                    break;
                }
                let command = battle.advance_command().unwrap();
                events.extend(accept(&mut battle, command));
            }
            let observed = events
                .into_iter()
                .filter(|event| {
                    event
                        .cause()
                        .source_definition()
                        .is_some_and(|id| id.get() == 6)
                })
                .collect::<Vec<_>>();
            assert_eq!(
                damage(&observed),
                [100],
                "{kind:?} formation {formation} unit {linked}"
            );
            let own = cast_ability(&mut battle, 1, 7, Some(2));
            assert_eq!(damage(&own), [180]);
            assert_eq!(attack_effects(&battle), [1]);
        }
    }
}

#[test]
fn weighted_curio_overflow_attack_terminal_events_remove_original_effect() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let mut battle = scenario(&fixture, &probe(&fixture, CombatPath::Hunt));
    start(&mut battle);
    assert_eq!(attack_effects(&battle), [1]);
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::Concede { .. }))
        .unwrap()
        .clone();
    accept(&mut battle, command);
    assert!(attack_effects(&battle).is_empty());

    let input = Probe {
        hp: vec![180],
        levels: vec![95],
        ..probe(&fixture, CombatPath::Hunt)
    };
    let mut won = scenario(&fixture, &input);
    start(&mut won);
    assert_eq!(attack_effects(&won), [1]);
    assert_eq!(damage(&cast_ability(&mut won, 1, 7, Some(2))), [180]);
    assert!(attack_effects(&won).is_empty());
}
