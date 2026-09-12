use std::collections::BTreeSet;

use starclock_activity::ActivityTerminalOutcome;
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_mode_universe::divergent_universe::{
    DivergentUniverseBaselineFixture, encode_divergent_universe_replay,
    record_divergent_universe_run, verify_divergent_universe_replay,
    verify_divergent_universe_selected_replay,
};
use starclock_replay::{
    format::{decode_replay, encode_replay},
    record::RecordRef,
};

#[test]
fn divergent_universe_seed_property_is_deterministic_and_stream_distinct() {
    let fixture = DivergentUniverseBaselineFixture::production().expect("production fixture");
    let mut replay_digests = BTreeSet::new();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        for seed in 0..8 {
            let left = encode_divergent_universe_replay(
                &record_divergent_universe_run(&fixture, family, seed).expect("left run"),
            )
            .expect("left replay");
            let right = encode_divergent_universe_replay(
                &record_divergent_universe_run(&fixture, family, seed).expect("right run"),
            )
            .expect("right replay");
            assert_eq!(left, right, "{family:?} seed {seed}");
            let report =
                verify_divergent_universe_replay(&left, &fixture).expect("fresh production replay");
            assert_eq!(report.terminal(), ActivityTerminalOutcome::Completed);
            assert_eq!(report.run_family(), family);
            assert_eq!(report.battle_count(), 1);
            assert!(replay_digests.insert(left));
        }
    }
    assert_eq!(replay_digests.len(), 16);
}

#[test]
fn divergent_universe_malformed_and_corrupt_replays_fail_repeatably() {
    let fixture = DivergentUniverseBaselineFixture::production().expect("production fixture");
    let bytes = encode_divergent_universe_replay(
        &record_divergent_universe_run(&fixture, DivergentUniverseRunFamily::Ordinary, 812)
            .expect("complete run"),
    )
    .expect("canonical replay");
    let truncation_step = (bytes.len() / 31).max(1);
    for end in (0..bytes.len()).step_by(truncation_step) {
        let left = verify_divergent_universe_replay(&bytes[..end], &fixture)
            .expect_err("truncated replay rejected");
        let right = verify_divergent_universe_replay(&bytes[..end], &fixture)
            .expect_err("same truncated replay rejected");
        assert_eq!(format!("{left:?}"), format!("{right:?}"));
    }
    let mutation_step = (bytes.len() / 17).max(1);
    for index in (0..bytes.len()).step_by(mutation_step) {
        let mut corrupted = bytes.clone();
        corrupted[index] ^= 1;
        let left = verify_divergent_universe_replay(&corrupted, &fixture)
            .expect_err("corrupt replay rejected");
        let right = verify_divergent_universe_replay(&corrupted, &fixture)
            .expect_err("same corrupt replay rejected");
        assert_eq!(format!("{left:?}"), format!("{right:?}"));
    }
    verify_divergent_universe_replay(&bytes, &fixture)
        .expect("corruption probes do not mutate fixture authority");
}

#[test]
fn divergent_universe_canonical_save_load_round_trip_reconstructs_fresh() {
    let fixture = DivergentUniverseBaselineFixture::production().expect("production fixture");
    let bytes = encode_divergent_universe_replay(
        &record_divergent_universe_run(&fixture, DivergentUniverseRunFamily::Cyclical, 813)
            .expect("complete run"),
    )
    .expect("canonical replay");
    let decoded = decode_replay(&bytes).expect("canonical load");
    let records = decoded
        .records()
        .iter()
        .map(|record| {
            RecordRef::new(record.kind(), record.sequence(), record.payload())
                .expect("loaded record remains valid")
        })
        .collect::<Vec<_>>();
    let loaded = encode_replay(decoded.header(), &records, Vec::new()).expect("canonical save");
    assert_eq!(loaded, bytes);
    verify_divergent_universe_replay(&loaded, &fixture).expect("loaded replay reconstructs fresh");
}

#[test]
fn divergent_universe_stale_selection_rejects_without_poisoning_valid_replay() {
    let fixture = DivergentUniverseBaselineFixture::production().expect("production fixture");
    let bytes = encode_divergent_universe_replay(
        &record_divergent_universe_run(&fixture, DivergentUniverseRunFamily::Ordinary, 814)
            .expect("complete run"),
    )
    .expect("canonical replay");
    assert!(
        verify_divergent_universe_selected_replay(
            &bytes,
            &fixture,
            DivergentUniverseRunFamily::Ordinary,
            "divergent-universe.area.402",
            "divergent-universe.difficulty.3021",
        )
        .is_err()
    );
    assert!(
        verify_divergent_universe_selected_replay(
            &bytes,
            &fixture,
            DivergentUniverseRunFamily::Cyclical,
            "divergent-universe.area.20401",
            "divergent-universe.difficulty.3011",
        )
        .is_err()
    );
    verify_divergent_universe_replay(&bytes, &fixture)
        .expect("valid replay remains independently verifiable");
}
