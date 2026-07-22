//! Revisioned, exact, transport-neutral value vocabulary.

use core::{fmt, str::FromStr};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

/// Human-readable responsibility marker used by architecture tests.
pub const RESPONSIBILITY: &str = "schema revisions and exact agent values";
/// Frozen public schema revision.
pub const AGENT_SCHEMA_REVISION: &str = "agent-api-v1";
/// Frozen schema/golden bundle identity.
pub const AGENT_SCHEMA_BUNDLE_SHA256: &str =
    "1746004f3f73ebbe6fb4cce4b850dd6813a1dc3a8584c3d191903328c0206725";
pub const OBSERVATION_SCHEMA_JSON: &str =
    include_str!("../../../schemas/agent-api-v1/observation.schema.json");
pub const ACTION_SCHEMA_JSON: &str =
    include_str!("../../../schemas/agent-api-v1/action.schema.json");
pub const ERROR_SCHEMA_JSON: &str = include_str!("../../../schemas/agent-api-v1/error.schema.json");
pub const ORDINARY_OBSERVATION_GOLDEN_JSON: &str =
    include_str!("../../../schemas/agent-api-v1/goldens/ordinary-observation.json");
pub const TRIGGER_HEAVY_ACTION_GOLDEN_JSON: &str =
    include_str!("../../../schemas/agent-api-v1/goldens/trigger-heavy-action-response.json");
pub const STALE_DECISION_ERROR_GOLDEN_JSON: &str =
    include_str!("../../../schemas/agent-api-v1/goldens/stale-decision-error.json");

/// Public schema revision accepted by this crate.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum AgentSchemaRevision {
    /// Goal 02 version-one contract.
    #[serde(rename = "agent-api-v1")]
    V1,
}

impl AgentSchemaRevision {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1 => AGENT_SCHEMA_REVISION,
        }
    }
}

/// Stable rejection while parsing exact public values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentValueError {
    Empty,
    TooLong,
    NonCanonicalInteger,
    InvalidOpaqueId,
    InvalidScenarioId,
    InvalidHash,
    UnknownRevision,
}

impl fmt::Display for AgentValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid agent value: {self:?}")
    }
}

impl std::error::Error for AgentValueError {}

/// Canonical unsigned base-ten integer transported as a JSON string.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct AgentUInt(Box<str>);

impl AgentUInt {
    #[must_use]
    pub fn from_u64(value: u64) -> Self {
        Self(value.to_string().into_boxed_str())
    }

    pub fn parse(value: &str) -> Result<Self, AgentValueError> {
        if !is_canonical_uint(value) || value.len() > 20 {
            return Err(AgentValueError::NonCanonicalInteger);
        }
        value
            .parse::<u64>()
            .map_err(|_| AgentValueError::NonCanonicalInteger)?;
        Ok(Self(value.into()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for AgentUInt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("AgentUInt").field(&self.0).finish()
    }
}

impl<'de> Deserialize<'de> for AgentUInt {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserialize_checked(deserializer, Self::parse)
    }
}

/// Canonical signed base-ten integer transported as a JSON string.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct AgentSInt(Box<str>);

impl AgentSInt {
    #[must_use]
    pub fn from_i64(value: i64) -> Self {
        Self(value.to_string().into_boxed_str())
    }

    pub fn parse(value: &str) -> Result<Self, AgentValueError> {
        if !is_canonical_sint(value) || value.len() > 20 {
            return Err(AgentValueError::NonCanonicalInteger);
        }
        value
            .parse::<i64>()
            .map_err(|_| AgentValueError::NonCanonicalInteger)?;
        Ok(Self(value.into()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for AgentSInt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("AgentSInt").field(&self.0).finish()
    }
}

impl<'de> Deserialize<'de> for AgentSInt {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserialize_checked(deserializer, Self::parse)
    }
}

macro_rules! opaque_id {
    ($name:ident) => {
        #[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(Box<str>);

        impl $name {
            pub fn parse(value: &str) -> Result<Self, AgentValueError> {
                if value.is_empty() {
                    return Err(AgentValueError::Empty);
                }
                if value.len() > 128 {
                    return Err(AgentValueError::TooLong);
                }
                if !value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
                {
                    return Err(AgentValueError::InvalidOpaqueId);
                }
                Ok(Self(value.into()))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(concat!(stringify!($name), "([redacted])"))
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                deserialize_checked(deserializer, Self::parse)
            }
        }
    };
}

opaque_id!(SessionId);
opaque_id!(ActionToken);
opaque_id!(EventCursor);
opaque_id!(IdempotencyKey);

/// Frozen Standard scenario stable identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ScenarioId(Box<str>);

impl ScenarioId {
    pub fn parse(value: &str) -> Result<Self, AgentValueError> {
        const PREFIX: &str = "scenario.standard-v1.";
        let suffix = value
            .strip_prefix(PREFIX)
            .ok_or(AgentValueError::InvalidScenarioId)?;
        if suffix.is_empty()
            || value.len() > 128
            || !suffix
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(AgentValueError::InvalidScenarioId);
        }
        Ok(Self(value.into()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ScenarioId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserialize_checked(deserializer, Self::parse)
    }
}

/// Lowercase SHA-256 identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct AgentHash(Box<str>);

impl AgentHash {
    pub fn parse(value: &str) -> Result<Self, AgentValueError> {
        if value.len() != 64
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(AgentValueError::InvalidHash);
        }
        Ok(Self(value.into()))
    }

    #[must_use]
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut encoded = String::with_capacity(64);
        for byte in bytes {
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        Self(encoded.into_boxed_str())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for AgentHash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserialize_checked(deserializer, Self::parse)
    }
}

impl FromStr for AgentSchemaRevision {
    type Err = AgentValueError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            AGENT_SCHEMA_REVISION => Ok(Self::V1),
            _ => Err(AgentValueError::UnknownRevision),
        }
    }
}

fn is_canonical_uint(value: &str) -> bool {
    value == "0" || (!value.starts_with('0') && value.bytes().all(|byte| byte.is_ascii_digit()))
}

fn is_canonical_sint(value: &str) -> bool {
    value == "0"
        || value.strip_prefix('-').map_or_else(
            || is_canonical_uint(value),
            |digits| digits != "0" && is_canonical_uint(digits),
        )
}

fn deserialize_checked<'de, D, T>(
    deserializer: D,
    parse: impl FnOnce(&str) -> Result<T, AgentValueError>,
) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Box::<str>::deserialize(deserializer)?;
    parse(&value).map_err(D::Error::custom)
}
