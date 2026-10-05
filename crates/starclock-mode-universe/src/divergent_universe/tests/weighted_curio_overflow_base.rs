//! Authored formula boundary probes; normal equipment has separate command tests.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseEntry,
    contribution_snapshot::DivergentUniverseDifficultyProtocolSnapshot,
    tests::{
        area_id, difficulty_id, input_snapshot, instance, star_pioneer, vertical_slice_inputs,
        weighted_curio_overflow_fixture::{Probe, cast, scenario_with_base, start},
    },
    weighted_curio_overflow::{OverflowBaseDamagePolicy, OverflowBaseDamagePolicyError},
};
use starclock_activity::ActivityMasterSeed;
use starclock_combat::{BattleEvent, BattleEventKind, catalog::action::TargetPattern};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_replay::battle_event::encode_battle_event_payload;
use std::sync::Arc;

fn protocol(number: u16) -> DivergentUniverseDifficultyProtocolSnapshot {
    let (factory, _, participants, mapping) = vertical_slice_inputs();
    let (area, difficulty) = match number {
        1 => ("401", "3011"),
        2 => ("402", "3021"),
        _ => unreachable!("fixture uses only Protocols 1 and 2"),
    };
    let flow = factory
        .compile(
            DivergentUniverseEntry::new(
                area_id(area),
                difficulty_id(difficulty),
                Arc::clone(&participants),
                input_snapshot(&participants),
                Vec::new(),
            )
            .unwrap()
            .with_mapping_snapshot(mapping)
            .with_astronomical(star_pioneer(number, number, 0, true)),
        )
        .unwrap();
    let activity = flow
        .start(instance(26410), ActivityMasterSeed::from_u64(26410))
        .unwrap()
        .into_activity();
    factory
        .contribution_snapshot_runtime()
        .unwrap()
        .snapshot(&flow, &activity)
        .unwrap()
        .difficulty_protocol()
        .clone()
}

#[test]
fn weighted_curio_overflow_authored_base_executes_all_levels_and_immutable_protocols() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let definition = &fixture
        .factory()
        .decision_catalog()
        .weighted_curio_overflows()[0];
    let mut snapshots = Vec::new();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let activity = flow
            .start(instance(26410), ActivityMasterSeed::from_u64(26410))
            .unwrap()
            .into_activity();
        let before = activity.canonical_state_bytes();
        snapshots.push(
            fixture
                .factory()
                .contribution_snapshot_runtime()
                .unwrap()
                .snapshot(&flow, &activity)
                .unwrap()
                .difficulty_protocol()
                .clone(),
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    snapshots.extend([protocol(1), protocol(2)]);
    assert_eq!(snapshots[0].maximum_hp_increase_millionths(), 0);
    assert_eq!(snapshots[1].maximum_hp_increase_millionths(), 0);
    assert_eq!(snapshots[2].maximum_hp_increase_millionths(), 2_000_000);
    for snapshot in snapshots {
        let policy = OverflowBaseDamagePolicy::from_authored(definition, &snapshot).unwrap();
        for level in 1..=95 {
            let probe = Probe {
                hp: vec![100, 3_000_000],
                levels: vec![95, level],
                pattern: TargetPattern::All,
                ..Probe::default()
            };
            let mut battle = scenario_with_base(&fixture, &probe, &policy);
            start(&mut battle);
            let events = cast(&mut battle, 1, None);
            let damages = events
                .iter()
                .filter_map(|event| match event.kind() {
                    BattleEventKind::Damage(data) => Some(data),
                    _ => None,
                })
                .collect::<Vec<_>>();
            // Independent integer oracle: floor at each millionths boundary,
            // then integral TrueDamage. The defeated target/owner levels differ.
            let ratio = i128::from(definition.base.hp_ratios_millionths[usize::from(level - 1)]);
            let first = 100_000_000_i128 * ratio / 1_000_000;
            let factor = 1_000_000_i128 + i128::from(snapshot.maximum_hp_increase_millionths());
            let base = first * factor / 1_000_000;
            let expected = i64::try_from((10 * base + 150_000_000) / 1_000_000).unwrap();
            assert_eq!(damages.len(), 3);
            assert_eq!(damages[2].target.get(), 3);
            assert_eq!(damages[2].calculated.get(), expected, "level {level}");
            assert_eq!(battle.view().rng_draw_count(), 1);
            if [1, 40, 80, 95].contains(&level) {
                let mut fresh = scenario_with_base(&fixture, &probe, &policy);
                start(&mut fresh);
                let replayed = cast(&mut fresh, 1, None);
                let payloads = |events: &[BattleEvent]| {
                    events
                        .iter()
                        .map(|event| encode_battle_event_payload(event).unwrap())
                        .collect::<Vec<_>>()
                };
                assert_eq!(events, replayed);
                assert_eq!(payloads(&events), payloads(&replayed));
                assert_eq!(battle.state_hash(), fresh.state_hash());
            }
        }
    }
}

#[test]
fn weighted_curio_overflow_authored_base_rejects_invalid_and_overflowing_definitions() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let original = &fixture
        .factory()
        .decision_catalog()
        .weighted_curio_overflows()[0];
    let snapshot = protocol(1);
    for change in 0..8 {
        let mut altered = original.clone();
        match change {
            0 => altered.base.fixed_damage_millionths = 0,
            1 => altered.base.hard_level_group = 2,
            2 => altered.base.hp_ratios_millionths = Box::new([]),
            3 => altered.base.hp_ratios_millionths[94] = 0,
            4 => altered.base.hp_ratios_millionths[0] = -1,
            5 => altered.base.policy_note = "false parity".into(),
            6 => altered.base.replacement_condition = " ".into(),
            7 => altered.base.hp_ratios_millionths[94] = i64::MAX,
            _ => unreachable!(),
        }
        assert_eq!(
            OverflowBaseDamagePolicy::from_authored(&altered, &snapshot).unwrap_err(),
            if change == 7 {
                OverflowBaseDamagePolicyError::Arithmetic
            } else {
                OverflowBaseDamagePolicyError::InvalidDefinition
            }
        );
    }
}

#[test]
fn weighted_curio_overflow_authored_base_binds_unselected_levels_policy_and_protocol() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let original = &fixture
        .factory()
        .decision_catalog()
        .weighted_curio_overflows()[0];
    let snapshot = protocol(1);
    let policy = OverflowBaseDamagePolicy::from_authored(original, &snapshot).unwrap();
    let reference = scenario_with_base(&fixture, &Probe::default(), &policy).state_hash();
    for change in 0..4 {
        let mut altered = original.clone();
        match change {
            0 => altered.base.hp_ratios_millionths[0] += 1,
            1 => altered.base.policy_note = format!("{} amended", altered.base.policy_note).into(),
            2 => altered.base.replacement_condition = "replace on new released evidence".into(),
            3 => (),
            _ => unreachable!(),
        }
        let input = if change == 3 {
            protocol(2)
        } else {
            snapshot.clone()
        };
        let changed = OverflowBaseDamagePolicy::from_authored(&altered, &input).unwrap();
        assert_ne!(
            reference,
            scenario_with_base(&fixture, &Probe::default(), &changed).state_hash()
        );
    }
}
