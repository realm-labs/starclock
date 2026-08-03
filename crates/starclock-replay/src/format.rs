//! Component-addressed replay format.
//!
//! The header binds the exact runtime components consumed by a replay.

use crate::digest::BuildCatalogDigest;
use crate::digest::CombatantBuildDigest;
use crate::digest::DefinitionDigest;
use crate::digest::EntrySpecDigest;
use core::fmt;

use crate::{
    codec::{CanonicalEncode, CanonicalSink, CodecError, Decoder, Encoder},
    component::{
        ComponentIdentityError, ConfigurationComponentIdentity, ConfigurationComponentKind,
        ConfigurationComponentSet, MAX_COMPONENT_TEXT_BYTES, MAX_REPLAY_COMPONENTS,
    },
    digest::{ComponentDigest, ComponentRootDigest},
    entry::{BuildBindings, MAX_BUILD_BINDINGS, ReplayEntry},
    record::{
        MAX_RECORD_PAYLOAD_BYTES, MAX_REPLAY_RECORDS, RecordKind, RecordRef, ReplayFormatError,
        UnknownRecordPolicy,
    },
};

pub const REPLAY_MAGIC: [u8; 4] = *b"SCRP";
pub const MAX_HEADER_TEXT_BYTES: u32 = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayEnvironment {
    game_version: Box<str>,
}

impl ReplayEnvironment {
    pub fn new(game_version: impl Into<Box<str>>) -> Result<Self, ReplayError> {
        let value = Self {
            game_version: game_version.into(),
        };
        validate_header_text(&value.game_version)?;
        Ok(value)
    }

    #[must_use]
    pub fn game_version(&self) -> &str {
        &self.game_version
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayHeader {
    environment: ReplayEnvironment,
    components: ConfigurationComponentSet,
    master_seed: u64,
    entry: ReplayEntry,
    record_count: u32,
}

impl ReplayHeader {
    pub fn new(
        environment: ReplayEnvironment,
        components: ConfigurationComponentSet,
        master_seed: u64,
        entry: ReplayEntry,
        record_count: u32,
    ) -> Result<Self, ReplayError> {
        if record_count > MAX_REPLAY_RECORDS {
            return Err(ReplayError::Format(ReplayFormatError::TooManyRecords));
        }
        validate_entry(&entry)?;
        Ok(Self {
            environment,
            components,
            master_seed,
            entry,
            record_count,
        })
    }

    #[must_use]
    pub const fn environment(&self) -> &ReplayEnvironment {
        &self.environment
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

impl CanonicalEncode for ReplayHeader {
    fn encode<S: CanonicalSink>(&self, e: &mut Encoder<S>) -> Result<(), CodecError> {
        encode_header(self, e)
    }
}

#[derive(Debug)]
pub struct DecodedReplay<'a> {
    header: ReplayHeader,
    records: Box<[RecordRef<'a>]>,
}

impl<'a> DecodedReplay<'a> {
    #[must_use]
    pub const fn header(&self) -> &ReplayHeader {
        &self.header
    }

    #[must_use]
    pub fn records(&self) -> &[RecordRef<'a>] {
        &self.records
    }
}

pub fn encode_replay<S: CanonicalSink>(
    header: &ReplayHeader,
    records: &[RecordRef<'_>],
    sink: S,
) -> Result<S, ReplayError> {
    if records.len() != header.record_count as usize {
        return Err(ReplayError::Format(
            ReplayFormatError::InvalidRecordSequence,
        ));
    }
    let mut encoder = Encoder::new(sink);
    encode_header(header, &mut encoder)?;
    for (expected, record) in records.iter().enumerate() {
        if record.sequence() != expected as u64 {
            return Err(ReplayError::Format(
                ReplayFormatError::InvalidRecordSequence,
            ));
        }
        record.encode(&mut encoder)?;
    }
    Ok(encoder.into_inner())
}

pub fn decode_replay(bytes: &[u8]) -> Result<DecodedReplay<'_>, ReplayError> {
    let mut decoder = Decoder::new(bytes);
    let header = decode_header(&mut decoder)?;
    let records_start = decoder.position();
    let mut record_decoder = Decoder::new(&bytes[records_start..]);
    let mut records = Vec::with_capacity(header.record_count as usize);
    for sequence in 0..header.record_count {
        records.push(decode_record(&mut record_decoder, u64::from(sequence))?);
    }
    record_decoder.finish()?;
    Ok(DecodedReplay {
        header,
        records: records.into_boxed_slice(),
    })
}

fn encode_header<S: CanonicalSink>(
    header: &ReplayHeader,
    e: &mut Encoder<S>,
) -> Result<(), CodecError> {
    e.raw(&REPLAY_MAGIC);
    e.u8(UnknownRecordPolicy::Reject as u8);
    encode_environment(&header.environment, e)?;
    header.components.encode(e)?;
    e.u64(header.master_seed);
    encode_entry(&header.entry, e)?;
    e.u32(header.record_count);
    Ok(())
}

fn encode_environment<S: CanonicalSink>(
    value: &ReplayEnvironment,
    e: &mut Encoder<S>,
) -> Result<(), CodecError> {
    e.string(&value.game_version)?;
    Ok(())
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

fn decode_header(d: &mut Decoder<'_>) -> Result<ReplayHeader, ReplayError> {
    if d.take(4)? != REPLAY_MAGIC {
        return Err(ReplayError::Format(ReplayFormatError::InvalidMagic));
    }
    let policy = d.u8()?;
    if policy != UnknownRecordPolicy::Reject as u8 {
        return Err(ReplayError::Format(ReplayFormatError::UnknownRecordPolicy(
            policy,
        )));
    }
    let environment = ReplayEnvironment::new(d.string(MAX_HEADER_TEXT_BYTES)?)?;
    let components = decode_components(d)?;
    let master_seed = d.u64()?;
    let entry = decode_entry(d)?;
    let record_count = d.u32()?;
    ReplayHeader::new(environment, components, master_seed, entry, record_count)
}

fn decode_components(d: &mut Decoder<'_>) -> Result<ConfigurationComponentSet, ReplayError> {
    let count = d.u32()? as usize;
    if count == 0 || count > MAX_REPLAY_COMPONENTS {
        return Err(ReplayError::Component(
            ComponentIdentityError::ComponentCount,
        ));
    }
    let mut components = Vec::with_capacity(count);
    for _ in 0..count {
        components.push(ConfigurationComponentIdentity::new(
            ConfigurationComponentKind::try_from(d.u8()?)?,
            d.string(MAX_COMPONENT_TEXT_BYTES as u32)?,
            ComponentDigest::new(d.take(32)?.try_into().expect("fixed length")),
        )?);
    }
    let encoded_root = ComponentRootDigest::new(d.take(32)?.try_into().expect("fixed length"));
    let components = ConfigurationComponentSet::new(components)?;
    if components.root() != encoded_root {
        return Err(ReplayError::Component(ComponentIdentityError::RootMismatch));
    }
    Ok(components)
}

fn decode_entry(d: &mut Decoder<'_>) -> Result<ReplayEntry, ReplayError> {
    match d.u8()? {
        1 => Ok(ReplayEntry::Battle {
            definition_id: d.u32()?,
            spec_digest: EntrySpecDigest::new(d.take(32)?.try_into().expect("fixed length")),
        }),
        2 => {
            let profile_id = Box::<str>::from(d.string(MAX_HEADER_TEXT_BYTES)?);
            let definition_id = d.u32()?;
            let definition_digest =
                DefinitionDigest::new(d.take(32)?.try_into().expect("fixed length"));
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
        other => Err(ReplayError::Format(ReplayFormatError::UnknownEntryKind(
            other,
        ))),
    }
}

fn decode_builds(d: &mut Decoder<'_>) -> Result<BuildBindings, ReplayError> {
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
    BuildBindings::new(digest, combatants).map_err(ReplayError::Format)
}

fn decode_record<'a>(d: &mut Decoder<'a>, expected: u64) -> Result<RecordRef<'a>, ReplayError> {
    let kind = RecordKind::try_from(d.u8()?).map_err(ReplayError::Format)?;
    let sequence = d.u64()?;
    if sequence != expected {
        return Err(ReplayError::Format(
            ReplayFormatError::InvalidRecordSequence,
        ));
    }
    RecordRef::new(kind, sequence, d.bytes(MAX_RECORD_PAYLOAD_BYTES)?).map_err(ReplayError::Format)
}

fn validate_entry(entry: &ReplayEntry) -> Result<(), ReplayError> {
    let definition_id = match entry {
        ReplayEntry::Battle { definition_id, .. } | ReplayEntry::Activity { definition_id, .. } => {
            *definition_id
        }
    };
    if definition_id == 0 {
        return Err(ReplayError::Format(
            ReplayFormatError::InvalidEntryDefinition,
        ));
    }
    if let ReplayEntry::Activity { profile_id, .. } = entry {
        validate_header_text(profile_id)?;
    }
    Ok(())
}

fn validate_header_text(value: &str) -> Result<(), ReplayError> {
    if value.is_empty()
        || value.len() > MAX_HEADER_TEXT_BYTES as usize
        || !value.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(CodecError::LimitExceeded.into());
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayError {
    Format(ReplayFormatError),
    Component(ComponentIdentityError),
}

impl From<CodecError> for ReplayError {
    fn from(value: CodecError) -> Self {
        Self::Format(ReplayFormatError::Codec(value))
    }
}

impl From<ComponentIdentityError> for ReplayError {
    fn from(value: ComponentIdentityError) -> Self {
        Self::Component(value)
    }
}

impl fmt::Display for ReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "replay format error: {self:?}")
    }
}

impl std::error::Error for ReplayError {}
