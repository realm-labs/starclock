use crate::{
    codec::{CanonicalEncode, CanonicalSink, CodecError, Decoder, Encoder},
    digest::{
        BuildCatalogDigest, CombatantBuildDigest, ConfigBundleDigest, ControllerDigest,
        DefinitionDigest, EntrySpecDigest,
    },
    record::{
        MAX_RECORD_PAYLOAD_BYTES, MAX_REPLAY_RECORDS, RecordKind, RecordRef, ReplayFormatError,
        UnknownRecordPolicy,
    },
};

/// Fixed replay file magic.
pub const REPLAY_MAGIC: [u8; 4] = *b"SCRP";
/// Canonical replay envelope tag.
pub const REPLAY_ENVELOPE_TAG: u32 = 1;
/// Canonical replay schema tag.
pub const REPLAY_SCHEMA_TAG: u32 = 1;
/// Maximum bytes in any header text identity.
pub const MAX_HEADER_TEXT_BYTES: u32 = 128;
/// Maximum participant/build digests bound into one entry header.
pub const MAX_BUILD_BINDINGS: u32 = 1024;

/// Runtime identity required before replay execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayIdentity {
    game_version: Box<str>,
    config_bundle: ConfigBundleDigest,
}

impl ReplayIdentity {
    /// Creates a validated runtime identity.
    pub fn new(
        game_version: impl Into<Box<str>>,
        config_bundle: ConfigBundleDigest,
    ) -> Result<Self, ReplayFormatError> {
        let value = Self {
            game_version: game_version.into(),
            config_bundle,
        };
        validate_text(&value.game_version)?;
        Ok(value)
    }
    /// Returns the exact configuration digest.
    #[must_use]
    pub const fn config_bundle(&self) -> ConfigBundleDigest {
        self.config_bundle
    }
    /// Returns the source game version.
    #[must_use]
    pub fn game_version(&self) -> &str {
        &self.game_version
    }
}

/// Controller identity; diagnostics never affect authoritative state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControllerIdentity {
    digest: ControllerDigest,
}

impl ControllerIdentity {
    /// Creates a validated controller identity.
    pub const fn new(digest: ControllerDigest) -> Self {
        Self { digest }
    }
    /// Returns its exact configuration digest.
    #[must_use]
    pub const fn digest(&self) -> ControllerDigest {
        self.digest
    }
}

/// Optional build-aware replay binding in participant order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildBindings {
    catalog_digest: BuildCatalogDigest,
    combatants: Box<[CombatantBuildDigest]>,
}

impl BuildBindings {
    /// Creates validated build bindings.
    pub fn new(
        digest: BuildCatalogDigest,
        combatants: Vec<CombatantBuildDigest>,
    ) -> Result<Self, ReplayFormatError> {
        if combatants.len() > MAX_BUILD_BINDINGS as usize {
            return Err(CodecError::LimitExceeded.into());
        }
        Ok(Self {
            catalog_digest: digest,
            combatants: combatants.into_boxed_slice(),
        })
    }
    /// Returns the exact build catalog digest.
    #[must_use]
    pub const fn catalog_digest(&self) -> BuildCatalogDigest {
        self.catalog_digest
    }
    /// Returns build digests in participant order.
    #[must_use]
    pub fn combatants(&self) -> &[CombatantBuildDigest] {
        &self.combatants
    }
}

/// Initial replay entry identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayEntry {
    /// Low-level battle with no build vocabulary.
    Battle {
        definition_id: u32,
        spec_digest: EntrySpecDigest,
    },
    /// Activity/profile entry, optionally build-aware.
    Activity {
        profile_id: Box<str>,
        definition_id: u32,
        definition_digest: DefinitionDigest,
        spec_digest: EntrySpecDigest,
        builds: Option<BuildBindings>,
    },
}

/// Validated replay header.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayHeader {
    identity: ReplayIdentity,
    controller: ControllerIdentity,
    master_seed: u64,
    entry: ReplayEntry,
    record_count: u32,
}

impl ReplayHeader {
    /// Creates a header and validates preallocation bounds.
    pub fn new(
        identity: ReplayIdentity,
        controller: ControllerIdentity,
        master_seed: u64,
        entry: ReplayEntry,
        record_count: u32,
    ) -> Result<Self, ReplayFormatError> {
        if record_count > MAX_REPLAY_RECORDS {
            return Err(ReplayFormatError::TooManyRecords);
        }
        let definition_id = match &entry {
            ReplayEntry::Battle { definition_id, .. }
            | ReplayEntry::Activity { definition_id, .. } => *definition_id,
        };
        if definition_id == 0 {
            return Err(ReplayFormatError::InvalidEntryDefinition);
        }
        if let ReplayEntry::Activity { profile_id, .. } = &entry {
            validate_text(profile_id)?;
        }
        Ok(Self {
            identity,
            controller,
            master_seed,
            entry,
            record_count,
        })
    }
    /// Returns the declared record count.
    #[must_use]
    pub const fn record_count(&self) -> u32 {
        self.record_count
    }
    /// Returns runtime identity.
    #[must_use]
    pub const fn identity(&self) -> &ReplayIdentity {
        &self.identity
    }
    /// Returns controller identity.
    #[must_use]
    pub const fn controller(&self) -> &ControllerIdentity {
        &self.controller
    }
    /// Returns the master activity/battle seed.
    #[must_use]
    pub const fn master_seed(&self) -> u64 {
        self.master_seed
    }
    /// Returns the low-level battle or activity entry identity.
    #[must_use]
    pub const fn entry(&self) -> &ReplayEntry {
        &self.entry
    }
}

impl CanonicalEncode for ReplayHeader {
    fn encode<S: CanonicalSink>(&self, e: &mut Encoder<S>) -> Result<(), CodecError> {
        e.raw(&REPLAY_MAGIC);
        e.u32(REPLAY_ENVELOPE_TAG);
        e.u32(REPLAY_SCHEMA_TAG);
        e.u8(UnknownRecordPolicy::Reject as u8);
        encode_identity(&self.identity, e)?;
        e.raw(&self.controller.digest.bytes());
        e.u64(self.master_seed);
        match &self.entry {
            ReplayEntry::Battle {
                definition_id,
                spec_digest,
            } => {
                e.u8(1);
                e.u32(*definition_id);
                e.raw(&spec_digest.bytes());
            }
            ReplayEntry::Activity {
                profile_id,
                definition_id,
                definition_digest,
                spec_digest,
                builds,
            } => {
                e.u8(2);
                e.string(profile_id)?;
                e.u32(*definition_id);
                e.raw(&definition_digest.bytes());
                e.raw(&spec_digest.bytes());
                e.boolean(builds.is_some());
                if let Some(value) = builds {
                    encode_builds(value, e)?;
                }
            }
        }
        e.u32(self.record_count);
        Ok(())
    }
}

/// Borrowed decoded replay after full framing validation.
#[derive(Debug)]
pub struct DecodedReplay<'a> {
    header: ReplayHeader,
    records: Box<[RecordRef<'a>]>,
}

impl<'a> DecodedReplay<'a> {
    /// Returns the validated header.
    #[must_use]
    pub const fn header(&self) -> &ReplayHeader {
        &self.header
    }
    /// Returns records in exact sequence order.
    #[must_use]
    pub fn records(&self) -> &[RecordRef<'a>] {
        &self.records
    }
}

/// Encodes a complete replay through the caller's sink.
pub fn encode_replay<S: CanonicalSink>(
    header: &ReplayHeader,
    records: &[RecordRef<'_>],
    sink: S,
) -> Result<S, ReplayFormatError> {
    if records.len() != header.record_count as usize {
        return Err(ReplayFormatError::InvalidRecordSequence);
    }
    let mut encoder = Encoder::new(sink);
    header.encode(&mut encoder)?;
    for (expected, record) in records.iter().enumerate() {
        if record.sequence() != expected as u64 {
            return Err(ReplayFormatError::InvalidRecordSequence);
        }
        record.encode(&mut encoder)?;
    }
    Ok(encoder.into_inner())
}

/// Decodes and validates all lengths/sequences before allocating the record table.
pub fn decode_replay(bytes: &[u8]) -> Result<DecodedReplay<'_>, ReplayFormatError> {
    let mut decoder = Decoder::new(bytes);
    let header = decode_header(&mut decoder)?;
    let records_start = decoder.position();
    validate_records(&bytes[records_start..], header.record_count)?;
    let mut records = Vec::with_capacity(header.record_count as usize);
    let mut record_decoder = Decoder::new(&bytes[records_start..]);
    for sequence in 0..header.record_count {
        records.push(decode_record(&mut record_decoder, u64::from(sequence))?);
    }
    record_decoder.finish()?;
    Ok(DecodedReplay {
        header,
        records: records.into_boxed_slice(),
    })
}

fn encode_identity<S: CanonicalSink>(
    v: &ReplayIdentity,
    e: &mut Encoder<S>,
) -> Result<(), CodecError> {
    e.string(&v.game_version)?;
    e.raw(&v.config_bundle.bytes());
    Ok(())
}

fn encode_builds<S: CanonicalSink>(
    v: &BuildBindings,
    e: &mut Encoder<S>,
) -> Result<(), CodecError> {
    e.raw(&v.catalog_digest.bytes());
    e.u32(u32::try_from(v.combatants.len()).map_err(|_| CodecError::LengthOverflow)?);
    for digest in &v.combatants {
        e.raw(&digest.bytes());
    }
    Ok(())
}

fn validate_text(value: &str) -> Result<(), ReplayFormatError> {
    if value.is_empty()
        || value.len() > MAX_HEADER_TEXT_BYTES as usize
        || !value.bytes().all(|b| b.is_ascii_graphic())
    {
        return Err(ReplayFormatError::Codec(CodecError::LimitExceeded));
    }
    Ok(())
}

fn decode_header(d: &mut Decoder<'_>) -> Result<ReplayHeader, ReplayFormatError> {
    if d.take(4)? != REPLAY_MAGIC {
        return Err(ReplayFormatError::InvalidMagic);
    }
    let envelope_tag = d.u32()?;
    if envelope_tag != REPLAY_ENVELOPE_TAG {
        return Err(ReplayFormatError::UnexpectedEnvelopeTag(envelope_tag));
    }
    let schema = d.u32()?;
    if schema != REPLAY_SCHEMA_TAG {
        return Err(ReplayFormatError::UnexpectedSchemaTag(schema));
    }
    let policy = d.u8()?;
    if policy != UnknownRecordPolicy::Reject as u8 {
        return Err(ReplayFormatError::UnknownRecordPolicy(policy));
    }
    let identity = ReplayIdentity::new(
        d.string(MAX_HEADER_TEXT_BYTES)?,
        ConfigBundleDigest::new(d.take(32)?.try_into().expect("fixed length")),
    )?;
    let controller = ControllerIdentity::new(ControllerDigest::new(
        d.take(32)?.try_into().expect("fixed length"),
    ));
    let master_seed = d.u64()?;
    let entry = match d.u8()? {
        1 => ReplayEntry::Battle {
            definition_id: d.u32()?,
            spec_digest: EntrySpecDigest::new(d.take(32)?.try_into().expect("fixed length")),
        },
        2 => decode_activity_entry(d)?,
        other => return Err(ReplayFormatError::UnknownEntryKind(other)),
    };
    let record_count = d.u32()?;
    ReplayHeader::new(identity, controller, master_seed, entry, record_count)
}

fn decode_activity_entry(d: &mut Decoder<'_>) -> Result<ReplayEntry, ReplayFormatError> {
    let profile_id = Box::<str>::from(d.string(MAX_HEADER_TEXT_BYTES)?);
    let definition_id = d.u32()?;
    let definition_digest = DefinitionDigest::new(d.take(32)?.try_into().expect("fixed length"));
    let spec_digest = EntrySpecDigest::new(d.take(32)?.try_into().expect("fixed length"));
    let builds = match d.u8()? {
        0 => None,
        1 => Some(decode_builds(d)?),
        _ => return Err(CodecError::InvalidPresence.into()),
    };
    Ok(ReplayEntry::Activity {
        profile_id,
        definition_id,
        definition_digest,
        spec_digest,
        builds,
    })
}

fn decode_builds(d: &mut Decoder<'_>) -> Result<BuildBindings, ReplayFormatError> {
    let digest = BuildCatalogDigest::new(d.take(32)?.try_into().expect("fixed length"));
    let count = d.u32()?;
    if count > MAX_BUILD_BINDINGS {
        return Err(CodecError::LimitExceeded.into());
    }
    let mut combatants = Vec::with_capacity(count as usize);
    for _ in 0..count {
        combatants.push(CombatantBuildDigest::new(
            d.take(32)?.try_into().expect("fixed length"),
        ));
    }
    BuildBindings::new(digest, combatants)
}

fn validate_records(bytes: &[u8], count: u32) -> Result<(), ReplayFormatError> {
    let mut d = Decoder::new(bytes);
    for sequence in 0..count {
        let _ = decode_record(&mut d, u64::from(sequence))?;
    }
    d.finish()?;
    Ok(())
}

fn decode_record<'a>(
    d: &mut Decoder<'a>,
    expected: u64,
) -> Result<RecordRef<'a>, ReplayFormatError> {
    let kind = RecordKind::try_from(d.u8()?)?;
    let sequence = d.u64()?;
    if sequence != expected {
        return Err(ReplayFormatError::InvalidRecordSequence);
    }
    let payload = d.bytes(MAX_RECORD_PAYLOAD_BYTES)?;
    RecordRef::new(kind, sequence, payload)
}
