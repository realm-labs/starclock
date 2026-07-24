use sha2::{Digest, Sha256};
use starclock_replay::{
    component::{
        ComponentIdentityError, ConfigurationComponentIdentity, ConfigurationComponentKind,
        ConfigurationComponentSet,
    },
    digest::{ComponentDigest, EntrySpecDigest},
    format::{ReplayEntry, decode_replay},
    format_v2::{
        REPLAY_FORMAT_VERSION_V2, ReplayCompatibilityV2, ReplayHeaderV2, decode_replay_v2,
        encode_replay_v2,
    },
    format_v3::{REPLAY_FORMAT_VERSION_V3, decode_replay_v3, encode_replay_v3},
};

fn component(
    kind: ConfigurationComponentKind,
    id: &str,
    revision: &str,
    byte: u8,
) -> ConfigurationComponentIdentity {
    ConfigurationComponentIdentity::new(kind, id, revision, ComponentDigest::new([byte; 32]))
        .unwrap()
}

#[test]
fn v3_uses_a_distinct_envelope_and_rejects_unknown_records() {
    let header = ReplayHeaderV2::new(
        ReplayCompatibilityV2::new("4.4", "fixed-6-v1", "chacha8-v1", "sha256-v3").unwrap(),
        component_set(0x44),
        42,
        ReplayEntry::Battle {
            definition_id: 7,
            spec_digest: EntrySpecDigest::new([0x77; 32]),
        },
        0,
    )
    .unwrap();
    let bytes = encode_replay_v3(&header, &[], Vec::new()).unwrap();
    assert_eq!(
        u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
        REPLAY_FORMAT_VERSION_V3
    );
    assert_eq!(decode_replay_v3(&bytes).unwrap().header(), &header);
    assert!(decode_replay_v2(&bytes).is_err());

    let mut unknown = encode_replay_v3(
        &ReplayHeaderV2::new(
            header.compatibility().clone(),
            header.components().clone(),
            header.master_seed(),
            header.entry().clone(),
            1,
        )
        .unwrap(),
        &[starclock_replay::record::RecordRef::new(
            starclock_replay::record::RecordKind::ControllerDiagnostic,
            0,
            &[],
        )
        .unwrap()],
        Vec::new(),
    )
    .unwrap();
    let record_kind_offset = unknown.len() - 13;
    unknown[record_kind_offset] = 0xff;
    assert!(matches!(
        decode_replay_v3(&unknown),
        Err(starclock_replay::format_v3::ReplayV3Error::Format(
            starclock_replay::record::ReplayFormatError::UnknownRecordKind(0xff)
        ))
    ));
}

fn component_set(controller_byte: u8) -> ConfigurationComponentSet {
    ConfigurationComponentSet::new(vec![
        component(
            ConfigurationComponentKind::CombatCatalog,
            "combat-v4.4",
            "2026-07-17",
            0x11,
        ),
        component(
            ConfigurationComponentKind::ActivityCore,
            "activity-core",
            "logical-scope-v1",
            0x22,
        ),
        component(
            ConfigurationComponentKind::ModeContent,
            "standard-universe",
            "v4.4",
            0x33,
        ),
        component(
            ConfigurationComponentKind::Controller,
            "baseline",
            "v2",
            controller_byte,
        ),
    ])
    .unwrap()
}

#[test]
fn component_root_is_canonical_and_reports_the_first_mismatch() {
    let expected = component_set(0x44);
    assert_eq!(
        expected.root().bytes(),
        [
            122, 237, 94, 177, 68, 36, 68, 84, 178, 122, 230, 129, 205, 47, 204, 28, 237, 229, 167,
            252, 163, 60, 88, 111, 68, 77, 76, 163, 201, 102, 201, 249,
        ]
    );
    let actual = component_set(0x45);
    let divergence = expected.verify_exact(&actual).unwrap_err();
    assert_eq!(divergence.index, 3);
    assert_eq!(divergence.expected.unwrap().digest().bytes(), [0x44; 32]);
    assert_eq!(divergence.actual.unwrap().digest().bytes(), [0x45; 32]);
}

#[test]
fn component_set_rejects_duplicate_or_unsorted_keys() {
    let duplicate = component(
        ConfigurationComponentKind::CombatCatalog,
        "combat-v4.4",
        "one",
        1,
    );
    assert_eq!(
        ConfigurationComponentSet::new(vec![duplicate.clone(), duplicate]).unwrap_err(),
        ComponentIdentityError::NonCanonicalOrder
    );
    assert_eq!(
        ConfigurationComponentSet::new(vec![
            component(
                ConfigurationComponentKind::Controller,
                "controller",
                "one",
                1,
            ),
            component(
                ConfigurationComponentKind::CombatCatalog,
                "catalog",
                "one",
                2,
            ),
        ])
        .unwrap_err(),
        ComponentIdentityError::NonCanonicalOrder
    );
}

#[test]
fn v2_round_trip_binds_components_without_changing_legacy_decoder() {
    let header = ReplayHeaderV2::new(
        ReplayCompatibilityV2::new("4.4", "fixed-6-v1", "chacha8-v1", "sha256-v3").unwrap(),
        component_set(0x44),
        42,
        ReplayEntry::Battle {
            definition_id: 7,
            spec_digest: EntrySpecDigest::new([0x77; 32]),
        },
        0,
    )
    .unwrap();
    let bytes = encode_replay_v2(&header, &[], Vec::new()).unwrap();
    let frozen_digest: [u8; 32] = Sha256::digest(&bytes).into();
    assert_eq!(
        frozen_digest,
        [
            234, 149, 123, 1, 147, 51, 12, 39, 35, 100, 77, 57, 7, 10, 179, 250, 188, 168, 1, 171,
            165, 220, 65, 140, 10, 54, 26, 226, 248, 58, 231, 172,
        ]
    );
    assert_eq!(
        u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
        REPLAY_FORMAT_VERSION_V2
    );
    let decoded = decode_replay_v2(&bytes).unwrap();
    assert_eq!(decoded.header(), &header);
    assert!(decoded.records().is_empty());
    assert!(matches!(
        decode_replay(&bytes),
        Err(starclock_replay::record::ReplayFormatError::UnsupportedFormatVersion(2))
    ));

    let root_offset = bytes.len() - 8 - 1 - 4 - 32 - 4 - 32;
    let mut corrupt = bytes;
    corrupt[root_offset] ^= 0x80;
    assert!(decode_replay_v2(&corrupt).is_err());
}
