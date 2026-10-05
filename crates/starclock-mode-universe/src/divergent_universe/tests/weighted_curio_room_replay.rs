//! Encoded current commands on caller-rebuilt profiles, never serialized probes.
use super::fixture::{FAMILIES, Profile, advance, compile, select};
use crate::baseline_controller::{
    ActivityBaselineHints, ActivityOptionHint, ActivityScoreComponents,
};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselinePolicy,
    DivergentUniverseBaselineStep, DivergentUniverseRecordedRun,
    DivergentUniverseReplayDivergenceKind, encode_divergent_universe_replay,
    tests::weighted_curio_deflagration_native::{applications, fixture as fire_fixture},
    verify_divergent_universe_replay,
    weighted_curio::room::CLEAR_EQUIPMENT,
};
use starclock_activity::{ActivityInstanceId, ActivityMasterSeed, ActivityOptionId};
use starclock_data::divergent_universe_encounter_catalog::DivergentUniverseEncounterGroupId;
use starclock_replay::{
    component::ConfigurationComponentKind,
    format::{ReplayHeader, decode_replay, encode_replay},
    record::{RecordKind, RecordRef},
};

const SEED: u64 = 26_801;
fn record(
    source: &DivergentUniverseBaselineFixture,
    profile: &Profile,
    exhaust: bool,
) -> DivergentUniverseRecordedRun {
    let mut activity = profile
        .flow
        .start(
            ActivityInstanceId::new(1).unwrap(),
            ActivityMasterSeed::from_u64(SEED),
        )
        .unwrap()
        .into_activity();
    let id = &source
        .factory()
        .decision_catalog()
        .weighted_curio_deflagrations()[0]
        .weighted_curio;
    let runtime = source.factory().weighted_curio_runtime().unwrap();
    let key = u64::try_from(
        runtime
            .candidates()
            .iter()
            .position(|candidate| candidate == id)
            .unwrap(),
    )
    .unwrap()
        + 1;
    let mut changes = if exhaust {
        vec![key; 64]
    } else {
        vec![key, CLEAR_EQUIPMENT, key]
    }
    .into_iter();
    let mut steps = Vec::new();
    let mut equipped_rooms = 0;
    for _ in 0..profile.policy.max_steps() {
        if activity.player_view().terminal().is_some() {
            break;
        }
        let equipment = profile.flow.offered_weighted_curio_equipment(&activity);
        let step = if equipment && let Some(option) = changes.next() {
            equipped_rooms += 1;
            select(source, profile, &mut activity, option)
        } else {
            advance(source, profile, &mut activity)
        };
        steps.push(step);
    }
    assert!(
        activity.player_view().terminal().is_some(),
        "controlled profile must reach a real terminal"
    );
    assert_eq!(equipped_rooms, if exhaust { 64 } else { 3 });
    assert_eq!(activity.player_view().completed_battle_count(), 3);
    if !exhaust {
        let burns = steps
            .iter()
            .filter_map(|step| match step {
                DivergentUniverseBaselineStep::Battle { execution, .. } => Some(execution),
                _ => None,
            })
            .flat_map(|execution| execution.trace())
            .flat_map(|entry| entry.events().iter().cloned())
            .collect::<Vec<_>>();
        assert!(
            applications(&burns) > 0,
            "encoded menu selection must execute real Burn"
        );
    }
    profile
        .flow
        .record_bound_transcript(source, &activity, SEED, steps, &profile.policy)
        .unwrap()
}
fn payloads(bytes: &[u8]) -> Vec<(RecordKind, Vec<u8>)> {
    decode_replay(bytes)
        .unwrap()
        .records()
        .iter()
        .map(|record| (record.kind(), record.payload().to_vec()))
        .collect()
}
fn encode(header: &ReplayHeader, records: &[(RecordKind, Vec<u8>)]) -> Vec<u8> {
    let header = ReplayHeader::new(
        header.environment().clone(),
        header.components().clone(),
        header.master_seed(),
        header.entry().clone(),
        u32::try_from(records.len()).unwrap(),
    )
    .unwrap();
    let refs = records
        .iter()
        .enumerate()
        .map(|(index, (kind, payload))| {
            RecordRef::new(*kind, u64::try_from(index).unwrap(), payload).unwrap()
        })
        .collect::<Vec<_>>();
    encode_replay(&header, &refs, Vec::new()).unwrap()
}

#[test]
fn weighted_curio_bound_replay_reconstructs_menu_choices_and_real_battles_in_both_families() {
    let source = fire_fixture(1);
    let fresh_source = fire_fixture(1);
    for family in FAMILIES {
        let profile = compile(&source, family, 0, 1, false);
        let fresh = compile(&fresh_source, family, 0, 1, false);
        let recorded = record(&source, &profile, false);
        let bytes = encode_divergent_universe_replay(&recorded).unwrap();
        let verified = fresh
            .flow
            .verify_bound_replay(&bytes, &fresh_source, &fresh.policy)
            .unwrap();
        assert_eq!(verified.run_family(), family);
        assert_eq!(
            verified.action_count(),
            u32::try_from(recorded.action_count()).unwrap()
        );
        assert_eq!(verified.battle_count(), 3);
        assert!(verified.battle_command_count() > 0);
        assert_eq!(
            verified.final_state_hash().bytes(),
            recorded.report().final_state_hash().bytes()
        );
        assert_eq!(verified.terminal(), recorded.report().terminal());
        assert_eq!(
            bytes,
            encode_divergent_universe_replay(&record(&fresh_source, &fresh, false)).unwrap()
        );
        // The default reconstructor does not silently substitute a different graph.
        assert!(verify_divergent_universe_replay(&bytes, &fresh_source).is_err());
        assert!(
            fresh
                .unbound
                .verify_bound_replay(&bytes, &fresh_source, &fresh.policy)
                .is_err()
        );
    }
}

#[test]
fn weighted_curio_bound_replay_binds_policy_capacity_deck_and_family_before_commands() {
    let source = fire_fixture(1);
    let profile = compile(&source, FAMILIES[0], 0, 1, false);
    let recorded = record(&source, &profile, false);
    let bytes = encode_divergent_universe_replay(&recorded).unwrap();
    for (family, deck, capacity) in [
        (FAMILIES[0], 0, 2),
        (FAMILIES[0], 1, 1),
        (FAMILIES[1], 0, 1),
    ] {
        let foreign = compile(&source, family, deck, capacity, false);
        let rejected = foreign
            .flow
            .verify_bound_replay(&bytes, &source, &foreign.policy)
            .unwrap_err();
        assert_eq!(rejected.record_index(), Some(0));
        assert!(rejected.component_divergence().is_some());
    }
    let base = &profile.policy;
    let changed = [
        DivergentUniverseBaselinePolicy::new(
            base.hints().clone(),
            DivergentUniverseEncounterGroupId::new(format!(
                "{}.different",
                base.encounter_group().as_str()
            ))
            .unwrap(),
            base.encounter_stage(),
            base.max_steps(),
        )
        .unwrap(),
        DivergentUniverseBaselinePolicy::new(
            base.hints().clone(),
            base.encounter_group().clone(),
            base.encounter_stage(),
            base.max_steps() + 1,
        )
        .unwrap(),
        DivergentUniverseBaselinePolicy::new(
            base.hints().clone(),
            base.encounter_group().clone(),
            "different-fallback",
            base.max_steps(),
        )
        .unwrap(),
        DivergentUniverseBaselinePolicy::new(
            ActivityBaselineHints::new(vec![ActivityOptionHint::new(
                ActivityOptionId::new(1).unwrap(),
                ActivityScoreComponents::new(1, 2, 3, 4, 5).unwrap(),
            )])
            .unwrap(),
            base.encounter_group().clone(),
            base.encounter_stage(),
            base.max_steps(),
        )
        .unwrap(),
    ];
    for policy in changed {
        let error = profile
            .flow
            .verify_bound_replay(&bytes, &source, &policy)
            .unwrap_err();
        assert_eq!(error.record_index(), Some(0));
        let component = error.component_divergence().unwrap();
        assert_eq!(
            component.expected.as_ref().unwrap().kind(),
            ConfigurationComponentKind::Controller
        );
    }
    assert!(
        source
            .flow(FAMILIES[0])
            .unwrap()
            .verify_bound_replay(&bytes, &source, base)
            .is_err()
    );
}

#[test]
fn weighted_curio_bound_replay_detects_first_command_score_state_and_event_divergence() {
    let source = fire_fixture(1);
    let profile = compile(&source, FAMILIES[0], 0, 1, false);
    let bytes = encode_divergent_universe_replay(&record(&source, &profile, false)).unwrap();
    let decoded = decode_replay(&bytes).unwrap();
    let original = payloads(&bytes);
    let equipment = original
        .iter()
        .position(|(kind, payload)| {
            *kind == RecordKind::AcceptedActivityCommand
                && payload.starts_with(b"DUA1")
                && payload[4] == 6
                && u32::from_le_bytes(payload[21..25].try_into().unwrap()) == 18
        })
        .unwrap();
    // Scores are embedded in the accepted command, not separate diagnostics.
    let state = equipment + 1;
    assert_eq!(original[state].0, RecordKind::ExpectedActivityState);
    for (index, offset, divergence) in [
        (
            equipment,
            13,
            DivergentUniverseReplayDivergenceKind::ActivityCommand,
        ),
        (
            equipment,
            25,
            DivergentUniverseReplayDivergenceKind::ActivityCommand,
        ),
        (
            equipment,
            37,
            DivergentUniverseReplayDivergenceKind::ActivityCommand,
        ),
        (
            state,
            0,
            DivergentUniverseReplayDivergenceKind::ActivityState,
        ),
    ] {
        let mut changed = original.clone();
        changed[index].1[offset] ^= 0x40;
        // Corrupt a later state as well; first-boundary evidence must win.
        changed[state + 1].1[0] ^= 1;
        let tampered = encode(decoded.header(), &changed);
        let error = profile
            .flow
            .verify_bound_replay(&tampered, &source, &profile.policy)
            .unwrap_err();
        assert_eq!(error.record_index(), Some(u32::try_from(index).unwrap()));
        assert_eq!(error.first_divergence(), Some(divergence));
    }
    let event = original
        .iter()
        .position(|(kind, payload)| {
            *kind == RecordKind::ExpectedBattleState
                && u32::from_le_bytes(payload[32..36].try_into().unwrap()) > 0
        })
        .unwrap();
    let mut changed = original.clone();
    // Keep the 32-byte battle state, count and event length intact; change only
    // the actual first event payload after its u32 length prefix.
    changed[event].1[40] ^= 1;
    let error = profile
        .flow
        .verify_bound_replay(
            &encode(decoded.header(), &changed),
            &source,
            &profile.policy,
        )
        .unwrap_err();
    assert_eq!(error.record_index(), Some(u32::try_from(event).unwrap()));
    assert_eq!(
        error.first_divergence(),
        Some(DivergentUniverseReplayDivergenceKind::BattleEvent)
    );
    let mut truncated = original;
    truncated.pop();
    assert!(
        profile
            .flow
            .verify_bound_replay(
                &encode(decoded.header(), &truncated),
                &source,
                &profile.policy
            )
            .is_err()
    );
    let mut trailing = bytes;
    trailing.push(0);
    assert!(
        profile
            .flow
            .verify_bound_replay(&trailing, &source, &profile.policy)
            .is_err()
    );
}

#[test]
fn weighted_curio_bound_replay_covers_exact_menu_budget_and_clear_unequip() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let profile = compile(&source, family, 0, 1, false);
        let recorded = record(&source, &profile, true);
        let bytes = encode_divergent_universe_replay(&recorded).unwrap();
        let fresh = compile(&source, family, 0, 1, false);
        let report = fresh
            .flow
            .verify_bound_replay(&bytes, &source, &fresh.policy)
            .unwrap();
        assert_eq!(
            report.final_state_hash().bytes(),
            recorded.report().final_state_hash().bytes()
        );
        assert_eq!(report.battle_count(), 3);
        assert!(recorded.action_count() > 64);
    }
}
