//! Authored base policy through real commands; equipment remains fail-closed.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseEntry,
    contribution_snapshot::DivergentUniverseDifficultyProtocolSnapshot,
    tests::{
        area_id, difficulty_id, input_snapshot, instance, star_pioneer, vertical_slice_inputs,
        weighted_curio_deflagration_base_fixture::{cast, scenario},
    },
    weighted_curio::deflagration::{
        DeflagrationBaseDamagePolicy, DeflagrationBaseDamagePolicyError,
    },
};
use starclock_activity::ActivityMasterSeed;
use starclock_combat::{BattleEvent, BattleEventKind, Scalar, rule::model::RuleValue};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_replay::battle_event::encode_battle_event_payload;
use std::sync::Arc;

fn protocol(number: u16) -> DivergentUniverseDifficultyProtocolSnapshot {
    let (factory, _, participants, mapping) = vertical_slice_inputs();
    let (area, difficulty) = match number {
        1 => ("401", "3011"),
        2 => ("402", "3021"),
        _ => unreachable!(),
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
        .start(instance(26412), ActivityMasterSeed::from_u64(26412))
        .unwrap()
        .into_activity();
    let before = activity.canonical_state_bytes();
    let snapshot = factory
        .contribution_snapshot_runtime()
        .unwrap()
        .snapshot(&flow, &activity)
        .unwrap()
        .difficulty_protocol()
        .clone();
    assert_eq!(activity.canonical_state_bytes(), before);
    snapshot
}

fn payloads(events: &[BattleEvent]) -> Vec<Vec<u8>> {
    events
        .iter()
        .map(|event| encode_battle_event_payload(event).unwrap())
        .collect()
}

#[test]
fn deflagration_base_executes_all_95_target_levels_and_immutable_protocols() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let definition = &fixture
        .factory()
        .decision_catalog()
        .weighted_curio_deflagrations()[0];
    let mut snapshots = Vec::new();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let activity = flow
            .start(instance(26412), ActivityMasterSeed::from_u64(26412))
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
        let policy = DeflagrationBaseDamagePolicy::from_authored(definition, &snapshot).unwrap();
        for level in 1..=95 {
            let mut battle = scenario(&policy, level);
            let (accepted, events) = cast(&mut battle);
            // Independent integer oracle for both six-place floor boundaries.
            // Owner is level 70; the queried enemy's own level ranges 1..=95.
            let ratio = i128::from(definition.hp_ratios_millionths[usize::from(level - 1)]);
            let first = 100_000_000_i128 * ratio / 1_000_000;
            let factor = 1_000_000_i128 + i128::from(snapshot.maximum_hp_increase_millionths());
            let expected = i64::try_from(first * factor / 1_000_000).unwrap();
            let signals = events
                .iter()
                .filter_map(|event| match event.kind() {
                    BattleEventKind::RuleSignal(signal) if signal.code == 1012 => {
                        signal.value.clone()
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(
                signals,
                [RuleValue::Scalar(Scalar::from_scaled(expected))],
                "level {level}"
            );
            let damage = events
                .iter()
                .filter_map(|event| match event.kind() {
                    BattleEventKind::Damage(data) => Some(data),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(damage.len(), 1);
            assert_eq!(damage[0].target.get(), 2);
            assert_eq!(damage[0].calculated.get(), expected / 1_000_000);
            assert_eq!(battle.view().rng_draw_count(), 0);
            let before = battle.decision().cloned();
            let hash = battle.state_hash();
            assert!(battle.apply(accepted).is_err());
            assert_eq!(battle.decision().cloned(), before);
            assert_eq!(battle.state_hash(), hash);
            assert_eq!(battle.view().rng_draw_count(), 0);
            if [1, 40, 80, 95].contains(&level) {
                let mut fresh = scenario(&policy, level);
                let (_, repeated) = cast(&mut fresh);
                assert_eq!(events, repeated);
                assert_eq!(payloads(&events), payloads(&repeated));
                assert_eq!(battle.state_hash(), fresh.state_hash());
            }
        }
    }
}

#[test]
fn deflagration_base_rejects_invalid_and_overflowing_inputs() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let original = &fixture
        .factory()
        .decision_catalog()
        .weighted_curio_deflagrations()[0];
    let snapshot = protocol(1);
    for change in 0..8 {
        let mut altered = original.clone();
        match change {
            0 => altered.fixed_base_damage_millionths = 0,
            1 => altered.hard_level_group = 2,
            2 => altered.hp_ratios_millionths = Box::new([]),
            3 => altered.hp_ratios_millionths[94] = 0,
            4 => altered.hp_ratios_millionths[0] = -1,
            5 => altered.base_policy_note = "false parity".into(),
            6 => altered.base_replacement_condition = " ".into(),
            7 => altered.hp_ratios_millionths[94] = i64::MAX,
            _ => unreachable!(),
        }
        assert_eq!(
            DeflagrationBaseDamagePolicy::from_authored(&altered, &snapshot).unwrap_err(),
            if change == 7 {
                DeflagrationBaseDamagePolicyError::Arithmetic
            } else {
                DeflagrationBaseDamagePolicyError::InvalidDefinition
            }
        );
    }
}

#[test]
fn deflagration_base_binds_unselected_levels_and_independent_policy_inputs() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let original = &fixture
        .factory()
        .decision_catalog()
        .weighted_curio_deflagrations()[0];
    let snapshot = protocol(1);
    let policy = DeflagrationBaseDamagePolicy::from_authored(original, &snapshot).unwrap();
    let reference = scenario(&policy, 95).state_hash();
    for change in 0..4 {
        let mut altered = original.clone();
        match change {
            0 => altered.hp_ratios_millionths[0] += 1,
            1 => altered.base_policy_note = format!("{} amended", altered.base_policy_note).into(),
            2 => {
                altered.base_replacement_condition =
                    "replace independently on released evidence".into()
            }
            3 => (),
            _ => unreachable!(),
        }
        let input = if change == 3 {
            protocol(2)
        } else {
            snapshot.clone()
        };
        let changed = DeflagrationBaseDamagePolicy::from_authored(&altered, &input).unwrap();
        assert_ne!(policy.identity(), changed.identity());
        assert_ne!(reference, scenario(&changed, 95).state_hash());
    }
    let mut unrelated = original.clone();
    unrelated.runtime_policy_note =
        format!("{} independently amended", unrelated.runtime_policy_note).into();
    let unchanged = DeflagrationBaseDamagePolicy::from_authored(&unrelated, &snapshot).unwrap();
    assert_eq!(policy.identity(), unchanged.identity());
    assert_eq!(policy.expression(), unchanged.expression());
}
