//! Component-addressed replay envelope.
//!
//! Version 1 remains in [`crate::format`] for archived Goal 04 files. New
//! runtimes use this envelope and bind the exact components they consumed.

use core::fmt;

use crate::{
    codec::{CanonicalEncode, CanonicalSink, CodecError, Decoder, Encoder},
    component::{
        ComponentIdentityError, ConfigurationComponentIdentity, ConfigurationComponentKind,
        ConfigurationComponentSet, MAX_COMPONENT_TEXT_BYTES, MAX_REPLAY_COMPONENTS,
    },
    digest::{ComponentDigest, ComponentRootDigest},
    format::{BuildBindings, MAX_BUILD_BINDINGS, MAX_HEADER_TEXT_BYTES, REPLAY_MAGIC, ReplayEntry},
    record::{
        MAX_RECORD_PAYLOAD_BYTES, MAX_REPLAY_RECORDS, RecordKind, RecordRef, ReplayFormatError,
        UnknownRecordPolicy,
    },
};

pub const REPLAY_FORMAT_VERSION_V2: u32 = 2;
pub const REPLAY_SCHEMA_VERSION_V2: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayCompatibilityV2 {
    game_version: Box<str>,
    numeric_policy_revision: Box<str>,
    rng_algorithm_revision: Box<str>,
    state_hash_revision: Box<str>,
}

impl ReplayCompatibilityV2 {
    pub fn new(
        game_version: impl Into<Box<str>>,
        numeric_policy_revision: impl Into<Box<str>>,
        rng_algorithm_revision: impl Into<Box<str>>,
        state_hash_revision: impl Into<Box<str>>,
    ) -> Result<Self, ReplayV2Error> {
        let value = Self {
            game_version: game_version.into(),
            numeric_policy_revision: numeric_policy_revision.into(),
            rng_algorithm_revision: rng_algorithm_revision.into(),
            state_hash_revision: state_hash_revision.into(),
        };
        for text in [
            &value.game_version,
            &value.numeric_policy_revision,
            &value.rng_algorithm_revision,
            &value.state_hash_revision,
        ] {
            validate_header_text(text)?;
        }
        Ok(value)
    }

    #[must_use]
    pub fn game_version(&self) -> &str {
        &self.game_version
    }

    #[must_use]
    pub fn numeric_policy_revision(&self) -> &str {
        &self.numeric_policy_revision
    }

    #[must_use]
    pub fn rng_algorithm_revision(&self) -> &str {
        &self.rng_algorithm_revision
    }

    #[must_use]
    pub fn state_hash_revision(&self) -> &str {
        &self.state_hash_revision
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayHeaderV2 {
    compatibility: ReplayCompatibilityV2,
    components: ConfigurationComponentSet,
    master_seed: u64,
    entry: ReplayEntry,
    record_count: u32,
}

impl ReplayHeaderV2 {
    pub fn new(
        compatibility: ReplayCompatibilityV2,
        components: ConfigurationComponentSet,
        master_seed: u64,
        entry: ReplayEntry,
        record_count: u32,
    ) -> Result<Self, ReplayV2Error> {
        if record_count > MAX_REPLAY_RECORDS {
            return Err(ReplayV2Error::Format(ReplayFormatError::TooManyRecords));
        }
        validate_entry(&entry)?;
        Ok(Self {
            compatibility,
            components,
            master_seed,
            entry,
            record_count,
        })
    }

    #[must_use]
    pub const fn compatibility(&self) -> &ReplayCompatibilityV2 {
        &self.compatibility
    }

    #[must_use]
    pub const fn components(&self) -> &ConfigurationComponentSet {
        &self.components
    }

    #[must_use]
    pub const fn master_seed(&self) -> u64 {
        self.master_seed
    }

    #[must_use]
    pub const fn entry(&self) -> &ReplayEntry {
        &self.entry
    }

    #[must_use]
    pub const fn record_count(&self) -> u32 {
        self.record_count
    }
}

impl CanonicalEncode for ReplayHeaderV2 {
    fn encode<S: CanonicalSink>(&self, e: &mut Encoder<S>) -> Result<(), CodecError> {
        encode_header(self, e, REPLAY_FORMAT_VERSION_V2)
    }
}

#[derive(Debug)]
pub struct DecodedReplayV2<'a> {
    header: ReplayHeaderV2,
    records: Box<[RecordRef<'a>]>,
}

impl<'a> DecodedReplayV2<'a> {
    #[must_use]
    pub const fn header(&self) -> &ReplayHeaderV2 {
        &self.header
    }

    #[must_use]
    pub fn records(&self) -> &[RecordRef<'a>] {
        &self.records
    }
}

pub fn encode_replay_v2<S: CanonicalSink>(
    header: &ReplayHeaderV2,
    records: &[RecordRef<'_>],
    sink: S,
) -> Result<S, ReplayV2Error> {
    encode_replay_with_version(header, records, sink, REPLAY_FORMAT_VERSION_V2)
}

pub(crate) fn encode_replay_with_version<S: CanonicalSink>(
    header: &ReplayHeaderV2,
    records: &[RecordRef<'_>],
    sink: S,
    format_version: u32,
) -> Result<S, ReplayV2Error> {
    if records.len() != header.record_count as usize {
        return Err(ReplayV2Error::Format(
            ReplayFormatError::InvalidRecordSequence,
        ));
    }
    let mut encoder = Encoder::new(sink);
    encode_header(header, &mut encoder, format_version)?;
    for (expected, record) in records.iter().enumerate() {
        if record.sequence() != expected as u64 {
            return Err(ReplayV2Error::Format(
                ReplayFormatError::InvalidRecordSequence,
            ));
        }
        record.encode(&mut encoder)?;
    }
    Ok(encoder.into_inner())
}

pub fn decode_replay_v2(bytes: &[u8]) -> Result<DecodedReplayV2<'_>, ReplayV2Error> {
    decode_replay_with_version(bytes, REPLAY_FORMAT_VERSION_V2)
}

pub(crate) fn decode_replay_with_version(
    bytes: &[u8],
    format_version: u32,
) -> Result<DecodedReplayV2<'_>, ReplayV2Error> {
    let mut decoder = Decoder::new(bytes);
    let header = decode_header(&mut decoder, format_version)?;
    let records_start = decoder.position();
    let mut record_decoder = Decoder::new(&bytes[records_start..]);
    let mut records = Vec::with_capacity(header.record_count as usize);
    for sequence in 0..header.record_count {
        records.push(decode_record(&mut record_decoder, u64::from(sequence))?);
    }
    record_decoder.finish()?;
    Ok(DecodedReplayV2 {
        header,
        records: records.into_boxed_slice(),
    })
}

fn encode_header<S: CanonicalSink>(
    header: &ReplayHeaderV2,
    e: &mut Encoder<S>,
    format_version: u32,
) -> Result<(), CodecError> {
    e.raw(&REPLAY_MAGIC);
    e.u32(format_version);
    e.u32(REPLAY_SCHEMA_VERSION_V2);
    e.u8(UnknownRecordPolicy::Reject as u8);
    encode_compatibility(&header.compatibility, e)?;
    header.components.encode(e)?;
    e.u64(header.master_seed);
    encode_entry(&header.entry, e)?;
    e.u32(header.record_count);
    Ok(())
}

fn encode_compatibility<S: CanonicalSink>(
    value: &ReplayCompatibilityV2,
    e: &mut Encoder<S>,
) -> Result<(), CodecError> {
    e.string(&value.game_version)?;
    e.string(&value.numeric_policy_revision)?;
    e.string(&value.rng_algorithm_revision)?;
    e.string(&value.state_hash_revision)
}

fn encode_entry<S: CanonicalSink>(
    entry: &ReplayEntry,
    e: &mut Encoder<S>,
) -> Result<(), CodecError> {
    match entry {
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
            if let Some(builds) = builds {
                e.string(builds.catalog_revision())?;
                e.raw(&builds.catalog_digest().bytes());
                e.u32(
                    u32::try_from(builds.combatants().len())
                        .map_err(|_| CodecError::LengthOverflow)?,
                );
                for digest in builds.combatants() {
                    e.raw(&digest.bytes());
                }
            }
        }
    }
    Ok(())
}

fn decode_header(
    d: &mut Decoder<'_>,
    expected_format_version: u32,
) -> Result<ReplayHeaderV2, ReplayV2Error> {
    if d.take(4)? != REPLAY_MAGIC {
        return Err(ReplayV2Error::Format(ReplayFormatError::InvalidMagic));
    }
    let version = d.u32()?;
    if version != expected_format_version {
        return Err(ReplayV2Error::Format(
            ReplayFormatError::UnsupportedFormatVersion(version),
        ));
    }
    let schema = d.u32()?;
    if schema != REPLAY_SCHEMA_VERSION_V2 {
        return Err(ReplayV2Error::Format(
            ReplayFormatError::UnsupportedSchemaVersion(schema),
        ));
    }
    let policy = d.u8()?;
    if policy != UnknownRecordPolicy::Reject as u8 {
        return Err(ReplayV2Error::Format(
            ReplayFormatError::UnknownRecordPolicy(policy),
        ));
    }
    let compatibility = ReplayCompatibilityV2::new(
        d.string(MAX_HEADER_TEXT_BYTES)?,
        d.string(MAX_HEADER_TEXT_BYTES)?,
        d.string(MAX_HEADER_TEXT_BYTES)?,
        d.string(MAX_HEADER_TEXT_BYTES)?,
    )?;
    let components = decode_components(d)?;
    let master_seed = d.u64()?;
    let entry = decode_entry(d)?;
    let record_count = d.u32()?;
    ReplayHeaderV2::new(compatibility, components, master_seed, entry, record_count)
}

fn decode_components(d: &mut Decoder<'_>) -> Result<ConfigurationComponentSet, ReplayV2Error> {
    let revision = d.u32()?;
    if revision != crate::component::COMPONENT_SET_REVISION {
        return Err(ReplayV2Error::UnsupportedComponentRevision(revision));
    }
    let count = d.u32()? as usize;
    if count == 0 || count > MAX_REPLAY_COMPONENTS {
        return Err(ReplayV2Error::Component(
            ComponentIdentityError::ComponentCount,
        ));
    }
    let mut components = Vec::with_capacity(count);
    for _ in 0..count {
        components.push(ConfigurationComponentIdentity::new(
            ConfigurationComponentKind::try_from(d.u8()?)?,
            d.string(MAX_COMPONENT_TEXT_BYTES as u32)?,
            d.string(MAX_COMPONENT_TEXT_BYTES as u32)?,
            ComponentDigest::new(d.take(32)?.try_into().expect("fixed length")),
        )?);
    }
    let encoded_root = ComponentRootDigest::new(d.take(32)?.try_into().expect("fixed length"));
    let components = ConfigurationComponentSet::new(components)?;
    if components.root() != encoded_root {
        return Err(ReplayV2Error::Component(
            ComponentIdentityError::RootMismatch,
        ));
    }
    Ok(components)
}

fn decode_entry(d: &mut Decoder<'_>) -> Result<ReplayEntry, ReplayV2Error> {
    match d.u8()? {
        1 => Ok(ReplayEntry::Battle {
            definition_id: d.u32()?,
            spec_digest: crate::digest::EntrySpecDigest::new(
                d.take(32)?.try_into().expect("fixed length"),
            ),
        }),
        2 => {
            let profile_id = Box::<str>::from(d.string(MAX_HEADER_TEXT_BYTES)?);
            let definition_id = d.u32()?;
            let definition_digest =
                crate::digest::DefinitionDigest::new(d.take(32)?.try_into().expect("fixed length"));
            let spec_digest =
                crate::digest::EntrySpecDigest::new(d.take(32)?.try_into().expect("fixed length"));
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
        other => Err(ReplayV2Error::Format(ReplayFormatError::UnknownEntryKind(
            other,
        ))),
    }
}

fn decode_builds(d: &mut Decoder<'_>) -> Result<BuildBindings, ReplayV2Error> {
    let revision = d.string(MAX_HEADER_TEXT_BYTES)?;
    let digest =
        crate::digest::BuildCatalogDigest::new(d.take(32)?.try_into().expect("fixed length"));
    let count = d.u32()?;
    if count > MAX_BUILD_BINDINGS {
        return Err(CodecError::LimitExceeded.into());
    }
    let mut combatants = Vec::with_capacity(count as usize);
    for _ in 0..count {
        combatants.push(crate::digest::CombatantBuildDigest::new(
            d.take(32)?.try_into().expect("fixed length"),
        ));
    }
    BuildBindings::new(revision, digest, combatants).map_err(ReplayV2Error::Format)
}

fn decode_record<'a>(d: &mut Decoder<'a>, expected: u64) -> Result<RecordRef<'a>, ReplayV2Error> {
    let kind = RecordKind::try_from(d.u8()?).map_err(ReplayV2Error::Format)?;
    let sequence = d.u64()?;
    if sequence != expected {
        return Err(ReplayV2Error::Format(
            ReplayFormatError::InvalidRecordSequence,
        ));
    }
    RecordRef::new(kind, sequence, d.bytes(MAX_RECORD_PAYLOAD_BYTES)?)
        .map_err(ReplayV2Error::Format)
}

fn validate_entry(entry: &ReplayEntry) -> Result<(), ReplayV2Error> {
    let definition_id = match entry {
        ReplayEntry::Battle { definition_id, .. } | ReplayEntry::Activity { definition_id, .. } => {
            *definition_id
        }
    };
    if definition_id == 0 {
        return Err(ReplayV2Error::Format(
            ReplayFormatError::InvalidEntryDefinition,
        ));
    }
    if let ReplayEntry::Activity { profile_id, .. } = entry {
        validate_header_text(profile_id)?;
    }
    Ok(())
}

fn validate_header_text(value: &str) -> Result<(), ReplayV2Error> {
    if value.is_empty()
        || value.len() > MAX_HEADER_TEXT_BYTES as usize
        || !value.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(CodecError::LimitExceeded.into());
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayV2Error {
    Format(ReplayFormatError),
    Component(ComponentIdentityError),
    UnsupportedComponentRevision(u32),
}

impl From<CodecError> for ReplayV2Error {
    fn from(value: CodecError) -> Self {
        Self::Format(ReplayFormatError::Codec(value))
    }
}

impl From<ComponentIdentityError> for ReplayV2Error {
    fn from(value: ComponentIdentityError) -> Self {
        Self::Component(value)
    }
}

impl fmt::Display for ReplayV2Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "replay v2 error: {self:?}")
    }
}

impl std::error::Error for ReplayV2Error {}
