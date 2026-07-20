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
/// Canonical replay envelope/codec revision.
pub const REPLAY_FORMAT_VERSION: u32 = 1;
/// Initial domain-payload schema revision.
pub const REPLAY_SCHEMA_VERSION: u32 = 1;
/// Full canonical state hash policy revision.
pub const STATE_HASH_REVISION: &str = "sha256-v2";
/// Maximum bytes in any compatibility/header text identity.
pub const MAX_HEADER_TEXT_BYTES: u32 = 128;
/// Maximum participant/build digests bound into one entry header.
pub const MAX_BUILD_BINDINGS: u32 = 1024;

/// Compatibility identities required before replay execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayIdentity {
    game_version: Box<str>,
    rules_revision: Box<str>,
    data_revision: Box<str>,
    config_bundle: ConfigBundleDigest,
    numeric_policy_revision: Box<str>,
    rng_algorithm_revision: Box<str>,
    state_hash_revision: Box<str>,
}

impl ReplayIdentity {
    /// Creates validated compatibility identity fields.
    pub fn new(
        game_version: impl Into<Box<str>>,
        rules_revision: impl Into<Box<str>>,
        data_revision: impl Into<Box<str>>,
        config_bundle: ConfigBundleDigest,
        numeric_policy_revision: impl Into<Box<str>>,
        rng_algorithm_revision: impl Into<Box<str>>,
        state_hash_revision: impl Into<Box<str>>,
    ) -> Result<Self, ReplayFormatError> {
        let value = Self {
            game_version: game_version.into(),
            rules_revision: rules_revision.into(),
            data_revision: data_revision.into(),
            config_bundle,
            numeric_policy_revision: numeric_policy_revision.into(),
            rng_algorithm_revision: rng_algorithm_revision.into(),
            state_hash_revision: state_hash_revision.into(),
        };
        for text in [
            &value.game_version,
            &value.rules_revision,
            &value.data_revision,
            &value.numeric_policy_revision,
            &value.rng_algorithm_revision,
            &value.state_hash_revision,
        ] {
            validate_text(text)?;
        }
        Ok(value)
    }
    /// Returns the exact configuration digest.
    #[must_use]
    pub const fn config_bundle(&self) -> ConfigBundleDigest {
        self.config_bundle
    }
    /// Returns the compatibility-target game version.
    #[must_use]
    pub fn game_version(&self) -> &str {
        &self.game_version
    }
    /// Returns the combat/activity rules revision.
    #[must_use]
    pub fn rules_revision(&self) -> &str {
        &self.rules_revision
    }
    /// Returns the domain catalog revision.
    #[must_use]
    pub fn data_revision(&self) -> &str {
        &self.data_revision
    }
    /// Returns the numeric compatibility revision.
    #[must_use]
    pub fn numeric_policy_revision(&self) -> &str {
        &self.numeric_policy_revision
    }
    /// Returns the RNG compatibility revision.
    #[must_use]
    pub fn rng_algorithm_revision(&self) -> &str {
        &self.rng_algorithm_revision
    }
    /// Returns the canonical state-hash revision.
    #[must_use]
    pub fn state_hash_revision(&self) -> &str {
        &self.state_hash_revision
    }
}

/// Controller compatibility identity; diagnostics never affect authoritative state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControllerIdentity {
    revision: Box<str>,
    digest: ControllerDigest,
}

impl ControllerIdentity {
    /// Creates a validated controller identity.
    pub fn new(
        revision: impl Into<Box<str>>,
        digest: ControllerDigest,
    ) -> Result<Self, ReplayFormatError> {
        let revision = revision.into();
        validate_text(&revision)?;
        Ok(Self { revision, digest })
    }
    /// Returns the controller implementation/policy revision.
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
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
    catalog_revision: Box<str>,
    catalog_digest: BuildCatalogDigest,
    combatants: Box<[CombatantBuildDigest]>,
}

impl BuildBindings {
    /// Creates validated build bindings.
    pub fn new(
        revision: impl Into<Box<str>>,
        digest: BuildCatalogDigest,
        combatants: Vec<CombatantBuildDigest>,
    ) -> Result<Self, ReplayFormatError> {
        let catalog_revision = revision.into();
        validate_text(&catalog_revision)?;
        if combatants.len() > MAX_BUILD_BINDINGS as usize {
            return Err(CodecError::LimitExceeded.into());
        }
        Ok(Self {
            catalog_revision,
            catalog_digest: digest,
            combatants: combatants.into_boxed_slice(),
        })
    }
    /// Returns the build catalog revision.
    #[must_use]
    pub fn catalog_revision(&self) -> &str {
        &self.catalog_revision
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

/// Validated version-1 replay header.
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
    /// Returns compatibility identities.
    #[must_use]
    pub const fn identity(&self) -> &ReplayIdentity {
        &self.identity
    }
    /// Returns controller compatibility identity.
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
        e.u32(REPLAY_FORMAT_VERSION);
        e.u32(REPLAY_SCHEMA_VERSION);
        e.u8(UnknownRecordPolicy::Reject as u8);
        encode_identity(&self.identity, e)?;
        e.string(&self.controller.revision)?;
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
    e.string(&v.rules_revision)?;
    e.string(&v.data_revision)?;
    e.raw(&v.config_bundle.bytes());
    e.string(&v.numeric_policy_revision)?;
    e.string(&v.rng_algorithm_revision)?;
    e.string(&v.state_hash_revision)
}

fn encode_builds<S: CanonicalSink>(
    v: &BuildBindings,
    e: &mut Encoder<S>,
) -> Result<(), CodecError> {
    e.string(&v.catalog_revision)?;
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
    let version = d.u32()?;
    if version != REPLAY_FORMAT_VERSION {
        return Err(ReplayFormatError::UnsupportedFormatVersion(version));
    }
    let schema = d.u32()?;
    if schema != REPLAY_SCHEMA_VERSION {
        return Err(ReplayFormatError::UnsupportedSchemaVersion(schema));
    }
    let policy = d.u8()?;
    if policy != UnknownRecordPolicy::Reject as u8 {
        return Err(ReplayFormatError::UnknownRecordPolicy(policy));
    }
    let identity = ReplayIdentity::new(
        d.string(MAX_HEADER_TEXT_BYTES)?,
        d.string(MAX_HEADER_TEXT_BYTES)?,
        d.string(MAX_HEADER_TEXT_BYTES)?,
        ConfigBundleDigest::new(d.take(32)?.try_into().expect("fixed length")),
        d.string(MAX_HEADER_TEXT_BYTES)?,
        d.string(MAX_HEADER_TEXT_BYTES)?,
        d.string(MAX_HEADER_TEXT_BYTES)?,
    )?;
    let controller = ControllerIdentity::new(
        d.string(MAX_HEADER_TEXT_BYTES)?,
        ControllerDigest::new(d.take(32)?.try_into().expect("fixed length")),
    )?;
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
    let revision = Box::<str>::from(d.string(MAX_HEADER_TEXT_BYTES)?);
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
    BuildBindings::new(revision, digest, combatants)
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
