//! Encoded service entries reconstruct public choices and reject forged inputs.
use super::instance;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseOfferedSelection, DivergentUniverseReplayDivergenceKind,
    encode_divergent_universe_replay, record_divergent_universe_run_with_tawot_service,
    record_divergent_universe_transcript, verify_divergent_universe_replay,
    verify_divergent_universe_selected_replay,
};
use starclock_activity::{ActivityMasterSeed, ActivityOptionId};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_replay::{
    format::{ReplayHeader, decode_replay, encode_replay},
    record::{RecordKind, RecordRef},
};

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];

#[test]
fn tawot_replay_restores_all_explicit_levels_in_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        for level in 2..=5 {
            let recorded =
                record_divergent_universe_run_with_tawot_service(&fixture, family, 24121, level)
                    .unwrap();
            let bytes = encode_divergent_universe_replay(&recorded).unwrap();
            let verified = verify_divergent_universe_replay(&bytes, &fresh).unwrap();
            assert_eq!(verified.run_family(), family);
            assert_eq!(verified.action_count() as usize, recorded.action_count());
            assert_eq!(verified.battle_count(), 3);
            assert_eq!(
                verified.final_state_hash().bytes(),
                recorded.report().final_state_hash().bytes()
            );
            assert!(recorded.action_count() > 10);
        }
    }
}

#[test]
fn tawot_replay_preserves_full_cancel_budget_without_controller_step_limit() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let flow = fixture.flow_with_tawot_service(family, 5).unwrap();
        let mut activity = flow
            .start(instance(1), ActivityMasterSeed::from_u64(24121))
            .unwrap()
            .into_activity();
        let policy = fixture.policy().unwrap();
        let runner = DivergentUniverseBaselineRunner::default();
        let mut steps = Vec::new();
        for _ in 0..3 {
            steps.push(
                runner
                    .advance(
                        fixture.factory(),
                        &flow,
                        &mut activity,
                        fixture.core(),
                        &policy,
                    )
                    .unwrap(),
            );
        }
        for _ in 0..64 {
            for option in [1, 0x7e42_0001] {
                let decision = activity.player_view().decision().unwrap().id();
                steps.push(
                    runner
                        .advance_selected(
                            fixture.factory(),
                            &flow,
                            &mut activity,
                            fixture.core(),
                            &policy,
                            DivergentUniverseOfferedSelection::new(
                                decision,
                                ActivityOptionId::new(option).unwrap(),
                            ),
                        )
                        .unwrap(),
                );
            }
        }
        assert_eq!(
            activity.player_view().decision().unwrap().options().len(),
            1
        );
        while activity.player_view().terminal().is_none() {
            assert!(steps.len() < 150);
            steps.push(
                runner
                    .advance(
                        fixture.factory(),
                        &flow,
                        &mut activity,
                        fixture.core(),
                        &policy,
                    )
                    .unwrap(),
            );
        }
        let recorded =
            record_divergent_universe_transcript(&fixture, &flow, &activity, 24121, steps).unwrap();
        assert!(recorded.action_count() > 128);
        let bytes = encode_divergent_universe_replay(&recorded).unwrap();
        let fresh = DivergentUniverseBaselineFixture::production().unwrap();
        let verified = verify_divergent_universe_replay(&bytes, &fresh).unwrap();
        assert_eq!(
            verified.final_state_hash().bytes(),
            activity.state_hash().bytes()
        );
        assert_eq!(verified.action_count() as usize, recorded.action_count());
    }
}

#[test]
fn tawot_replay_rejects_missing_malformed_or_changed_entry_before_player_actions() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let recorded = record_divergent_universe_run_with_tawot_service(
        &fixture,
        DivergentUniverseRunFamily::Ordinary,
        24121,
        2,
    )
    .unwrap();
    let bytes = encode_divergent_universe_replay(&recorded).unwrap();
    let decoded = decode_replay(&bytes).unwrap();
    let original = decoded
        .records()
        .iter()
        .map(|record| (record.kind(), record.payload().to_vec()))
        .collect::<Vec<_>>();
    for mutation in 0..12 {
        let mut changed = original.clone();
        let payload = &mut changed[0].1;
        let size = payload.len();
        match mutation {
            0 => payload[0] ^= 1,
            1 => payload.truncate(size - 1),
            2 => payload.push(0),
            3 => payload[3..7].copy_from_slice(&0_u32.to_le_bytes()),
            4 => payload[3..7].copy_from_slice(&u32::MAX.to_le_bytes()),
            5 => payload[7] = 255,
            6 => payload[size - 3..size - 1].copy_from_slice(&1_u16.to_le_bytes()),
            7 => payload[size - 3..size - 1].copy_from_slice(&6_u16.to_le_bytes()),
            8 => {
                changed.remove(0);
            }
            9 => changed[0].0 = RecordKind::ExpectedActivityState,
            10 => payload[size - 1] = 2,
            _ => payload.clear(),
        }
        let error = verify_divergent_universe_replay(&encode(decoded.header(), &changed), &fixture)
            .unwrap_err();
        assert_eq!(
            error.first_divergence(),
            Some(DivergentUniverseReplayDivergenceKind::ActivityCommand)
        );
        assert_eq!(error.record_index(), Some(0));
    }
    // Well-formed but different configurations cannot reuse the old identities.
    for level in [0_u16, 3, 4, 5] {
        let mut changed = original.clone();
        let size = changed[0].1.len();
        changed[0].1[size - 3..size - 1].copy_from_slice(&level.to_le_bytes());
        assert!(
            verify_divergent_universe_replay(&encode(decoded.header(), &changed), &fixture)
                .is_err()
        );
    }
    let mut changed = original.clone();
    *changed[0].1.last_mut().unwrap() = 1;
    assert!(
        verify_divergent_universe_replay(&encode(decoded.header(), &changed), &fixture).is_err()
    );
    let mut changed = original;
    changed[1].1[0] ^= 1;
    let error = verify_divergent_universe_replay(&encode(decoded.header(), &changed), &fixture)
        .unwrap_err();
    assert_eq!(
        error.first_divergence(),
        Some(DivergentUniverseReplayDivergenceKind::Activity)
    );
    assert_eq!(error.record_index(), Some(1));
    assert!(
        verify_divergent_universe_selected_replay(
            &bytes,
            &fixture,
            DivergentUniverseRunFamily::Ordinary,
            "incorrect-area",
            "3011"
        )
        .is_err()
    );
}

fn encode(header: &ReplayHeader, payloads: &[(RecordKind, Vec<u8>)]) -> Vec<u8> {
    let header = ReplayHeader::new(
        header.environment().clone(),
        header.components().clone(),
        header.master_seed(),
        header.entry().clone(),
        u32::try_from(payloads.len()).unwrap(),
    )
    .unwrap();
    let records = payloads
        .iter()
        .enumerate()
        .map(|(index, (kind, bytes))| RecordRef::new(*kind, index as u64, bytes).unwrap())
        .collect::<Vec<_>>();
    encode_replay(&header, &records, Vec::new()).unwrap()
}
