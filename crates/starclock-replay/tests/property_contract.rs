use proptest::{
    collection::vec,
    prelude::*,
    test_runner::{Config as ProptestConfig, FileFailurePersistence, RngAlgorithm, RngSeed},
};
use starclock_replay::{
    codec::{CanonicalEncode, CanonicalSink, CodecError, Decoder, Encoder, hash_canonical},
    digest::{ConfigBundleDigest, ControllerDigest, EntrySpecDigest, Sha256Sink},
    format::{
        ControllerIdentity, ReplayEntry, ReplayHeader, ReplayIdentity, STATE_HASH_REVISION,
        decode_replay, encode_replay,
    },
    record::{MAX_RECORD_PAYLOAD_BYTES, RecordKind, RecordRef, ReplayFormatError},
};

const CODEC_SEED: u64 = 0x636f_6465_632d_7631;
const REPLAY_SEED: u64 = 0x7265_706c_6179_2d31;
const MALFORMED_SEED: u64 = 0x6d61_6c66_6f72_6d31;

fn property_config(seed: u64) -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        max_shrink_iters: 4_096,
        failure_persistence: Some(Box::new(FileFailurePersistence::SourceParallel(
            "proptest-regressions",
        ))),
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(seed),
        ..ProptestConfig::default()
    }
}

fn ascii_text() -> impl Strategy<Value = String> {
    vec(0x21_u8..=0x7e, 0..65)
        .prop_map(|bytes| String::from_utf8(bytes).expect("ASCII strategy is UTF-8"))
}

fn digest(byte: u8) -> [u8; 32] {
    [byte; 32]
}

fn header(master_seed: u64, record_count: u32) -> ReplayHeader {
    let identity = ReplayIdentity::new(
        "4.4",
        "property-rules-v1",
        "property-data-v1",
        ConfigBundleDigest::new(digest(0x31)),
        "fixed-i64-6dp-v1",
        "chacha8-rand-0.10.2-intmap-v1",
        STATE_HASH_REVISION,
    )
    .expect("property identity is valid");
    let controller = ControllerIdentity::new(
        "property-controller-v1",
        ControllerDigest::new(digest(0x32)),
    )
    .expect("property controller is valid");
    ReplayHeader::new(
        identity,
        controller,
        master_seed,
        ReplayEntry::Battle {
            definition_id: 1,
            spec_digest: EntrySpecDigest::new(digest(0x33)),
        },
        record_count,
    )
    .expect("bounded property header is valid")
}

fn record_kind(raw: u8) -> RecordKind {
    match raw % 7 {
        0 => RecordKind::AcceptedBattleCommand,
        1 => RecordKind::ExpectedBattleState,
        2 => RecordKind::NestedBattleStart,
        3 => RecordKind::NestedBattleEnd,
        4 => RecordKind::AcceptedActivityCommand,
        5 => RecordKind::ExpectedActivityState,
        _ => RecordKind::ControllerDiagnostic,
    }
}

#[derive(Debug)]
struct PrimitiveFixture {
    boolean: bool,
    unsigned16: u16,
    unsigned32: u32,
    unsigned64: u64,
    signed64: i64,
    text: String,
    bytes: Vec<u8>,
}

impl CanonicalEncode for PrimitiveFixture {
    fn encode<S: CanonicalSink>(&self, encoder: &mut Encoder<S>) -> Result<(), CodecError> {
        encoder.boolean(self.boolean);
        encoder.u16(self.unsigned16);
        encoder.u32(self.unsigned32);
        encoder.u64(self.unsigned64);
        encoder.i64(self.signed64);
        encoder.string(&self.text)?;
        encoder.bytes(&self.bytes)
    }
}

proptest! {
    #![proptest_config(property_config(CODEC_SEED))]

    #[test]
    fn canonical_primitives_round_trip_and_stream_identically(
        boolean in any::<bool>(),
        unsigned16 in any::<u16>(),
        unsigned32 in any::<u32>(),
        unsigned64 in any::<u64>(),
        signed64 in any::<i64>(),
        text in ascii_text(),
        bytes in vec(any::<u8>(), 0..513),
    ) {
        let fixture = PrimitiveFixture {
            boolean,
            unsigned16,
            unsigned32,
            unsigned64,
            signed64,
            text,
            bytes,
        };
        let mut encoder = Encoder::new(Vec::new());
        fixture.encode(&mut encoder).unwrap();
        let encoded = encoder.into_inner();
        let mut decoder = Decoder::new(&encoded);
        prop_assert_eq!(decoder.boolean().unwrap(), fixture.boolean);
        prop_assert_eq!(decoder.u16().unwrap(), fixture.unsigned16);
        prop_assert_eq!(decoder.u32().unwrap(), fixture.unsigned32);
        prop_assert_eq!(decoder.u64().unwrap(), fixture.unsigned64);
        prop_assert_eq!(decoder.i64().unwrap(), fixture.signed64);
        prop_assert_eq!(decoder.string(64).unwrap(), fixture.text.as_str());
        prop_assert_eq!(decoder.bytes(512).unwrap(), fixture.bytes.as_slice());
        prop_assert_eq!(decoder.finish(), Ok(()));

        let mut streamed = Encoder::new(Sha256Sink::new());
        fixture.encode(&mut streamed).unwrap();
        prop_assert_eq!(
            hash_canonical(&fixture).unwrap().bytes(),
            streamed.into_inner().finalize().bytes()
        );
    }
}

proptest! {
    #![proptest_config(property_config(REPLAY_SEED))]

    #[test]
    fn replay_records_round_trip_with_stable_bytes(
        master_seed in any::<u64>(),
        generated in vec((0_u8..7, vec(any::<u8>(), 0..1025)), 0..257),
    ) {
        let records = generated
            .iter()
            .enumerate()
            .map(|(sequence, (kind, payload))| {
                RecordRef::new(record_kind(*kind), sequence as u64, payload).unwrap()
            })
            .collect::<Vec<_>>();
        let header = header(master_seed, records.len() as u32);
        let bytes = encode_replay(&header, &records, Vec::new()).unwrap();
        let decoded = decode_replay(&bytes).unwrap();
        prop_assert_eq!(decoded.header(), &header);
        prop_assert_eq!(decoded.records(), records.as_slice());
        prop_assert_eq!(
            encode_replay(decoded.header(), decoded.records(), Vec::new()).unwrap(),
            bytes
        );
    }
}

proptest! {
    #![proptest_config(property_config(MALFORMED_SEED))]

    #[test]
    fn malformed_replay_framing_is_always_rejected_before_payload_use(
        master_seed in any::<u64>(),
        payload in vec(any::<u8>(), 0..1025),
        mutation in 0_u8..5,
        selector in any::<usize>(),
    ) {
        let record = RecordRef::new(RecordKind::AcceptedBattleCommand, 0, &payload).unwrap();
        let mut bytes = encode_replay(&header(master_seed, 1), &[record], Vec::new()).unwrap();
        let record_offset = bytes.len() - (1 + 8 + 4 + payload.len());

        match mutation {
            0 => {
                let cut = selector % bytes.len();
                bytes.truncate(cut);
                prop_assert!(decode_replay(&bytes).is_err());
            }
            1 => {
                bytes.push(0);
                prop_assert_eq!(
                    decode_replay(&bytes).unwrap_err(),
                    ReplayFormatError::Codec(CodecError::TrailingBytes)
                );
            }
            2 => {
                bytes[record_offset] = 0xff;
                prop_assert_eq!(
                    decode_replay(&bytes).unwrap_err(),
                    ReplayFormatError::UnknownRecordKind(0xff)
                );
            }
            3 => {
                bytes[record_offset + 1..record_offset + 9]
                    .copy_from_slice(&1_u64.to_le_bytes());
                prop_assert_eq!(
                    decode_replay(&bytes).unwrap_err(),
                    ReplayFormatError::InvalidRecordSequence
                );
            }
            _ => {
                bytes[record_offset + 9..record_offset + 13]
                    .copy_from_slice(&(MAX_RECORD_PAYLOAD_BYTES + 1).to_le_bytes());
                prop_assert_eq!(
                    decode_replay(&bytes).unwrap_err(),
                    ReplayFormatError::Codec(CodecError::LimitExceeded)
                );
            }
        }
    }
}

proptest! {
    #![proptest_config(property_config(MALFORMED_SEED ^ 0x55aa_55aa_55aa_55aa))]

    #[test]
    fn arbitrary_bytes_never_escape_the_total_decoder(bytes in vec(any::<u8>(), 0..4097)) {
        let _ = decode_replay(&bytes);
    }
}
