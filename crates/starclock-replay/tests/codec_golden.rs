use starclock_replay::{
    codec::{CanonicalEncode, CanonicalSink, CodecError, Decoder, Encoder, hash_canonical},
    digest::{
        BuildCatalogDigest, CombatantBuildDigest, ConfigBundleDigest, ControllerDigest,
        DefinitionDigest, EntrySpecDigest, Sha256Digest, Sha256Sink, StateDigest,
    },
    format::{
        BuildBindings, ControllerIdentity, REPLAY_FORMAT_VERSION, ReplayEntry, ReplayHeader,
        ReplayIdentity, STATE_HASH_REVISION, decode_replay, encode_replay,
    },
    record::{RecordKind, RecordRef, ReplayFormatError},
};

fn digest(byte: u8) -> [u8; 32] {
    [byte; 32]
}

fn header(record_count: u32) -> ReplayHeader {
    let identity = ReplayIdentity::new(
        "4.4",
        "standard-rules-v1",
        "catalog-v4.4",
        ConfigBundleDigest::new(digest(0x11)),
        "fixed-i64-6dp-v1",
        "chacha8-rand-0.10.2-intmap-v1",
        STATE_HASH_REVISION,
    )
    .expect("golden compatibility identity is valid");
    let controller = ControllerIdentity::new(
        "baseline-controller-v1",
        ControllerDigest::new(digest(0x22)),
    )
    .expect("golden controller identity is valid");
    let builds = BuildBindings::new(
        "build-catalog-v1",
        BuildCatalogDigest::new(digest(0x55)),
        vec![
            CombatantBuildDigest::new(digest(0x66)),
            CombatantBuildDigest::new(digest(0x77)),
        ],
    )
    .expect("golden build bindings are valid");
    let entry = ReplayEntry::Activity {
        profile_id: "standard-v1".into(),
        definition_id: 42,
        definition_digest: DefinitionDigest::new(digest(0x33)),
        spec_digest: EntrySpecDigest::new(digest(0x44)),
        builds: Some(builds),
    };
    ReplayHeader::new(
        identity,
        controller,
        0x0123_4567_89ab_cdef,
        entry,
        record_count,
    )
    .expect("golden header is valid")
}

#[derive(Debug)]
struct SyntheticState;

impl CanonicalEncode for SyntheticState {
    fn encode<S: CanonicalSink>(&self, encoder: &mut Encoder<S>) -> Result<(), CodecError> {
        encoder.raw(b"state-v1\0");
        encoder.u8(0xab);
        encoder.boolean(false);
        encoder.boolean(true);
        encoder.u16(0x1234);
        encoder.u32(0x89ab_cdef);
        encoder.u64(0x0123_4567_89ab_cdef);
        encoder.i64(i64::MIN);
        encoder.string("星钟")?;
        encoder.bytes(&[0, 1, 0xfe, 0xff])
    }
}

#[test]
fn collecting_and_streaming_sinks_share_exact_state_bytes() {
    let mut collecting = Encoder::new(Vec::new());
    SyntheticState
        .encode(&mut collecting)
        .expect("synthetic state encodes");
    let bytes = collecting.into_inner();
    let expected_bytes = [
        0x73, 0x74, 0x61, 0x74, 0x65, 0x2d, 0x76, 0x31, 0x00, 0xab, 0x00, 0x01, 0x34, 0x12, 0xef,
        0xcd, 0xab, 0x89, 0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x80, 0x06, 0x00, 0x00, 0x00, 0xe6, 0x98, 0x9f, 0xe9, 0x92, 0x9f, 0x04,
        0x00, 0x00, 0x00, 0x00, 0x01, 0xfe, 0xff,
    ];
    assert_eq!(bytes.len(), expected_bytes.len());
    assert_eq!(bytes, expected_bytes);
    let mut decoder = Decoder::new(&bytes[9..]);
    assert_eq!(decoder.u8(), Ok(0xab));
    assert_eq!(decoder.boolean(), Ok(false));
    assert_eq!(decoder.boolean(), Ok(true));
    assert_eq!(decoder.u16(), Ok(0x1234));
    assert_eq!(decoder.u32(), Ok(0x89ab_cdef));
    assert_eq!(decoder.u64(), Ok(0x0123_4567_89ab_cdef));
    assert_eq!(decoder.i64(), Ok(i64::MIN));
    assert_eq!(decoder.string(16), Ok("星钟"));
    assert_eq!(decoder.bytes(16), Ok([0, 1, 0xfe, 0xff].as_slice()));
    assert_eq!(decoder.finish(), Ok(()));
    assert_eq!(
        hash_canonical(&SyntheticState).expect("streaming hash succeeds"),
        StateDigest::new([
            0xee, 0x9b, 0x65, 0x41, 0x6d, 0x97, 0xd2, 0x54, 0x32, 0x1e, 0xa5, 0xea, 0xea, 0x28,
            0x5a, 0xc9, 0x26, 0x30, 0x7f, 0xaa, 0x43, 0x5e, 0xa5, 0xc1, 0xac, 0xcc, 0xae, 0xbe,
            0x8d, 0x13, 0xb5, 0x22,
        ])
    );
}

#[test]
fn replay_header_records_round_trip_and_stream_hash() {
    let expected_state = digest(0xaa);
    let payloads = [
        b"command-1".as_slice(),
        expected_state.as_slice(),
        b"diag".as_slice(),
    ];
    let records = [
        RecordRef::new(RecordKind::AcceptedActivityCommand, 0, payloads[0]).expect("record valid"),
        RecordRef::new(RecordKind::ExpectedActivityState, 1, payloads[1]).expect("record valid"),
        RecordRef::new(RecordKind::ControllerDiagnostic, 2, payloads[2]).expect("record valid"),
    ];
    let header = header(records.len() as u32);
    let bytes = encode_replay(&header, &records, Vec::new()).expect("replay encodes");
    assert_eq!(&bytes[..4], b"SCRP");
    assert_eq!(
        u32::from_le_bytes(bytes[4..8].try_into().expect("four bytes")),
        REPLAY_FORMAT_VERSION
    );
    let decoded = decode_replay(&bytes).expect("golden replay decodes");
    assert_eq!(decoded.header(), &header);
    assert_eq!(decoded.records(), records);

    let streamed = encode_replay(&header, &records, Sha256Sink::new())
        .expect("replay streams")
        .finalize();
    assert_eq!(
        streamed,
        Sha256Digest::new([
            97, 112, 134, 241, 93, 84, 0, 251, 40, 245, 197, 20, 32, 39, 202, 13, 151, 98, 151, 40,
            242, 13, 252, 240, 81, 105, 111, 149, 14, 55, 142, 72,
        ])
    );
}

#[test]
fn low_level_battle_entry_round_trips_without_build_vocabulary() {
    let identity = ReplayIdentity::new(
        "4.4",
        "standard-rules-v1",
        "catalog-v4.4",
        ConfigBundleDigest::new(digest(1)),
        "fixed-i64-6dp-v1",
        "chacha8-rand-0.10.2-intmap-v1",
        STATE_HASH_REVISION,
    )
    .expect("battle identity is valid");
    let controller =
        ControllerIdentity::new("external-offered-v1", ControllerDigest::new(digest(2)))
            .expect("controller identity is valid");
    let entry = ReplayEntry::Battle {
        definition_id: 9,
        spec_digest: EntrySpecDigest::new(digest(3)),
    };
    let header =
        ReplayHeader::new(identity, controller, 5, entry, 0).expect("battle header is valid");
    let bytes = encode_replay(&header, &[], Vec::new()).expect("empty replay encodes");
    let decoded = decode_replay(&bytes).expect("empty battle replay decodes");
    assert_eq!(decoded.header(), &header);
    assert!(decoded.records().is_empty());
}

#[test]
fn zero_entry_definition_is_rejected_before_encoding() {
    let identity = ReplayIdentity::new(
        "4.4",
        "rules-v1",
        "data-v1",
        ConfigBundleDigest::new(digest(1)),
        "fixed-i64-6dp-v1",
        "chacha8-rand-0.10.2-intmap-v1",
        STATE_HASH_REVISION,
    )
    .expect("identity is valid");
    let controller = ControllerIdentity::new("external-v1", ControllerDigest::new(digest(2)))
        .expect("controller identity is valid");
    let entry = ReplayEntry::Battle {
        definition_id: 0,
        spec_digest: EntrySpecDigest::new(digest(3)),
    };
    assert_eq!(
        ReplayHeader::new(identity, controller, 1, entry, 0),
        Err(ReplayFormatError::InvalidEntryDefinition)
    );
}

#[test]
fn malformed_versions_unknown_records_and_framing_are_hard_failures() {
    let record = RecordRef::new(RecordKind::AcceptedBattleCommand, 0, b"x").expect("record valid");
    let header = header(1);
    let bytes = encode_replay(&header, &[record], Vec::new()).expect("replay encodes");
    let record_offset = bytes.len() - (1 + 8 + 4 + 1);

    let mut wrong_version = bytes.clone();
    wrong_version[4..8].copy_from_slice(&99_u32.to_le_bytes());
    assert_eq!(
        decode_replay(&wrong_version).expect_err("version must fail"),
        ReplayFormatError::UnsupportedFormatVersion(99)
    );

    let mut wrong_schema = bytes.clone();
    wrong_schema[8..12].copy_from_slice(&77_u32.to_le_bytes());
    assert_eq!(
        decode_replay(&wrong_schema).expect_err("schema must fail"),
        ReplayFormatError::UnsupportedSchemaVersion(77)
    );

    let mut wrong_policy = bytes.clone();
    wrong_policy[12] = 1;
    assert_eq!(
        decode_replay(&wrong_policy).expect_err("policy must fail"),
        ReplayFormatError::UnknownRecordPolicy(1)
    );

    let mut unknown = bytes.clone();
    unknown[record_offset] = 0xff;
    assert_eq!(
        decode_replay(&unknown).expect_err("unknown record must fail"),
        ReplayFormatError::UnknownRecordKind(0xff)
    );

    let mut bad_sequence = bytes.clone();
    bad_sequence[record_offset + 1..record_offset + 9].copy_from_slice(&1_u64.to_le_bytes());
    assert_eq!(
        decode_replay(&bad_sequence).expect_err("sequence must fail"),
        ReplayFormatError::InvalidRecordSequence
    );

    let mut oversized = bytes.clone();
    oversized[record_offset + 9..record_offset + 13]
        .copy_from_slice(&(16_u32 * 1024 * 1024 + 1).to_le_bytes());
    assert_eq!(
        decode_replay(&oversized).expect_err("oversized payload must fail"),
        ReplayFormatError::Codec(CodecError::LimitExceeded)
    );

    let mut truncated = bytes.clone();
    truncated.pop();
    assert!(matches!(
        decode_replay(&truncated),
        Err(ReplayFormatError::Codec(CodecError::UnexpectedEnd))
    ));
    let mut trailing = bytes;
    trailing.push(0);
    assert_eq!(
        decode_replay(&trailing).expect_err("trailing byte must fail"),
        ReplayFormatError::Codec(CodecError::TrailingBytes)
    );
}
