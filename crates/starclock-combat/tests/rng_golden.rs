use starclock_combat::rng::{
    RNG_ALGORITHM_REVISION, derive::StreamPath, engine::DeterministicRng, types::DrawPurpose,
};

const DERIVED_SEED: [u8; 32] = [
    0x7c, 0x9f, 0x0d, 0x7f, 0x05, 0xc0, 0xef, 0xda, 0xd9, 0xad, 0x14, 0xf6, 0xe6, 0x6e, 0xd9, 0xe5,
    0x61, 0x08, 0xdc, 0x25, 0xb9, 0x91, 0xd7, 0x45, 0xdd, 0x11, 0x3d, 0x6a, 0x20, 0x76, 0x5b, 0xbd,
];

#[test]
fn sha256_derivation_chacha8_words_and_integer_mappings_are_golden() {
    assert_eq!(RNG_ALGORITHM_REVISION, "chacha8-rand-0.10.2-intmap-v1");
    let path = StreamPath::new("standard-v1", 42, 3, 7, 2, 11, "battle")
        .expect("golden stream path is valid");
    let seed = path.derive_seed(0x0123_4567_89ab_cdef);
    assert_eq!(seed.bytes(), DERIVED_SEED);

    let mut rng = DeterministicRng::from_seed(seed);
    let raw = (0..8)
        .map(|_| {
            rng.draw_raw(DrawPurpose::CRIT)
                .expect("golden draw counter is available")
                .raw()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        raw,
        [
            2_825_180_620_894_282_070,
            1_898_966_477_154_481_968,
            1_189_069_528_429_638_169,
            14_828_129_876_552_261_426,
            3_311_180_893_435_138_928,
            17_857_464_126_714_966_064,
            13_502_502_199_665_507_364,
            18_440_358_510_317_432_650,
        ]
    );

    let ranges = [1_u64, 2, 3, 10, 65_537]
        .into_iter()
        .map(|upper| {
            let selection = rng
                .sample_below(DrawPurpose::BOUNCE_TARGET, upper)
                .expect("golden range is valid");
            (
                selection.sample().index(),
                selection.sample().raw(),
                selection.value(),
                selection.rejected_draws(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        ranges,
        [
            (8, 10_567_628_820_862_452_970, 0, 0),
            (9, 3_266_522_941_374_366_179, 1, 0),
            (10, 11_548_020_681_798_072_329, 2, 0),
            (11, 13_385_547_836_026_660_601, 1, 0),
            (12, 3_343_423_649_238_797_288, 38_166, 0),
        ]
    );

    let weighted = rng
        .choose_weighted(DrawPurpose::AGGRO_TARGET, &[0, 5, 9, 1])
        .expect("golden weights are valid")
        .expect("positive total selects a candidate");
    assert_eq!(weighted.index(), 2);
    assert_eq!(weighted.range().sample().index(), 13);
    assert_eq!(weighted.range().sample().raw(), 2_412_161_003_938_087_781);
    assert_eq!(weighted.range().value(), 11);
    assert_eq!(weighted.range().rejected_draws(), 0);
    assert_eq!(rng.draw_count(), 14);
}

#[test]
fn stream_path_components_isolate_future_activity_substreams() {
    let battle = StreamPath::new("standard-v1", 42, 3, 7, 2, 11, "battle")
        .expect("battle stream path is valid")
        .derive_seed(123);
    let spawn = StreamPath::new("standard-v1", 42, 3, 7, 2, 11, "spawn")
        .expect("spawn stream path is valid")
        .derive_seed(123);
    let next_attempt = StreamPath::new("standard-v1", 42, 3, 7, 3, 11, "battle")
        .expect("next-attempt stream path is valid")
        .derive_seed(123);

    assert_ne!(battle, spawn);
    assert_ne!(battle, next_attempt);
}
