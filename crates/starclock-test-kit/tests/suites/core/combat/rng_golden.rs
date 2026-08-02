use starclock_combat::rng::{derive::StreamPath, engine::DeterministicRng, types::DrawPurpose};

const DERIVED_SEED: [u8; 32] = [
    74, 68, 127, 21, 98, 178, 250, 70, 93, 236, 190, 11, 157, 49, 21, 140, 114, 155, 20, 51, 107,
    53, 237, 189, 217, 31, 186, 150, 186, 129, 171, 214,
];

#[test]
fn sha256_derivation_chacha8_words_and_integer_mappings_are_golden() {
    let path = StreamPath::new("standard", 42, 3, 7, 2, 11, "battle")
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
            13_688_018_998_735_943_848,
            16_958_743_438_719_903_442,
            689_129_106_957_067_895,
            33_912_604_313_134_972,
            6_395_911_296_259_864,
            12_628_346_939_270_799_171,
            3_133_752_112_535_164_657,
            18_247_536_753_784_664_816,
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
            (8, 1_087_277_417_912_598_104, 0, 0),
            (9, 13_113_051_527_326_443_542, 0, 0),
            (10, 7_722_818_192_692_745_428, 1, 0),
            (11, 7_374_993_817_069_586_039, 9, 0),
            (12, 11_035_618_088_890_844_675, 53_206, 0),
        ]
    );

    let weighted = rng
        .choose_weighted(DrawPurpose::AGGRO_TARGET, &[0, 5, 9, 1])
        .expect("golden weights are valid")
        .expect("positive total selects a candidate");
    assert_eq!(weighted.index(), 1);
    assert_eq!(weighted.range().sample().index(), 13);
    assert_eq!(weighted.range().sample().raw(), 3_848_029_865_741_063_569);
    assert_eq!(weighted.range().value(), 4);
    assert_eq!(weighted.range().rejected_draws(), 0);
    assert_eq!(rng.draw_count(), 14);
}

#[test]
fn stream_path_components_isolate_future_activity_substreams() {
    let battle = StreamPath::new("standard", 42, 3, 7, 2, 11, "battle")
        .expect("battle stream path is valid")
        .derive_seed(123);
    let spawn = StreamPath::new("standard", 42, 3, 7, 2, 11, "spawn")
        .expect("spawn stream path is valid")
        .derive_seed(123);
    let next_attempt = StreamPath::new("standard", 42, 3, 7, 3, 11, "battle")
        .expect("next-attempt stream path is valid")
        .derive_seed(123);

    assert_ne!(battle, spawn);
    assert_ne!(battle, next_attempt);
}
