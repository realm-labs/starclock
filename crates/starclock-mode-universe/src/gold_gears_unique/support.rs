use super::{
    GoldAndGearsUniqueError, GoldAndGearsUniqueErrorKind,
    types::{CanonicalScalar, Identity},
};

const ROW_REVISION: &str = "starclock.gold-and-gears-row.v1";

pub(super) fn identity<I>(
    id: i32,
    stable_key: &str,
    source_id: &str,
    constructor: impl FnOnce(u32) -> I,
) -> Result<Identity<I>, GoldAndGearsUniqueError> {
    metadata(stable_key)?;
    Ok(Identity {
        id: constructor(positive_u32(id, stable_key)?),
        stable_key: stable(stable_key)?,
        source_id: text(source_id, stable_key)?,
    })
}

pub(super) fn row(
    stable_key: &str,
    revision: &str,
    kind: &str,
    expected_kind: &str,
) -> Result<(), GoldAndGearsUniqueError> {
    metadata(stable_key)?;
    if revision != ROW_REVISION || kind != expected_kind {
        return invalid(stable_key);
    }
    Ok(())
}

pub(super) fn stable(value: &str) -> Result<Box<str>, GoldAndGearsUniqueError> {
    if value.is_empty() || value.len() > 256 || !value.bytes().all(|byte| byte.is_ascii_graphic()) {
        return fail(GoldAndGearsUniqueErrorKind::Identifier, value);
    }
    Ok(value.into())
}

pub(super) fn text(value: &str, key: &str) -> Result<Box<str>, GoldAndGearsUniqueError> {
    if value.trim().is_empty() || value.trim() != value {
        return invalid(key);
    }
    Ok(value.into())
}

pub(super) fn optional_text(
    value: Option<&str>,
    key: &str,
) -> Result<Option<Box<str>>, GoldAndGearsUniqueError> {
    value.map(|value| text(value, key)).transpose()
}

pub(super) fn texts(
    values: &[String],
    key: &str,
) -> Result<Box<[Box<str>]>, GoldAndGearsUniqueError> {
    values
        .iter()
        .map(|value| text(value, key))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

pub(super) fn optional_texts(
    values: Option<&[String]>,
    key: &str,
) -> Result<Box<[Box<str>]>, GoldAndGearsUniqueError> {
    values.map_or_else(
        || Ok(Vec::<Box<str>>::new().into_boxed_slice()),
        |values| texts(values, key),
    )
}

pub(super) fn json_text(value: &str, key: &str) -> Result<Box<str>, GoldAndGearsUniqueError> {
    let value = text(value, key)?;
    let bytes = value.as_bytes();
    if bytes.len() < 2
        || !matches!(
            (bytes.first(), bytes.last()),
            (Some(b'{'), Some(b'}')) | (Some(b'['), Some(b']'))
        )
    {
        return invalid(key);
    }
    Ok(value)
}

pub(super) fn scalar(value: &str, key: &str) -> Result<CanonicalScalar, GoldAndGearsUniqueError> {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let (integer, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, None), |(integer, fraction)| {
            (integer, Some(fraction))
        });
    let canonical_integer = integer == "0"
        || (!integer.starts_with('0') && integer.bytes().all(|byte| byte.is_ascii_digit()));
    let canonical_fraction = fraction.is_none_or(|fraction| {
        !fraction.is_empty()
            && fraction.bytes().all(|byte| byte.is_ascii_digit())
            && !fraction.ends_with('0')
    });
    if value.is_empty()
        || value == "-0"
        || !canonical_integer
        || !canonical_fraction
        || unsigned.matches('.').count() > 1
    {
        return invalid(key);
    }
    Ok(CanonicalScalar(value.into()))
}

pub(super) fn scalars(
    values: &[String],
    key: &str,
) -> Result<Box<[CanonicalScalar]>, GoldAndGearsUniqueError> {
    values
        .iter()
        .map(|value| scalar(value, key))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

pub(super) fn positive_u32(value: i32, key: &str) -> Result<u32, GoldAndGearsUniqueError> {
    u32::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| error(GoldAndGearsUniqueErrorKind::Identifier, key))
}

pub(super) fn positive_u16(value: i32, key: &str) -> Result<u16, GoldAndGearsUniqueError> {
    u16::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| error(GoldAndGearsUniqueErrorKind::Identifier, key))
}

pub(super) fn nonnegative_u16(value: i32, key: &str) -> Result<u16, GoldAndGearsUniqueError> {
    u16::try_from(value).map_err(|_| error(GoldAndGearsUniqueErrorKind::Identifier, key))
}

pub(super) fn positive_u8(value: i32, key: &str) -> Result<u8, GoldAndGearsUniqueError> {
    u8::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| error(GoldAndGearsUniqueErrorKind::Identifier, key))
}

pub(super) fn nonnegative_u8(value: i32, key: &str) -> Result<u8, GoldAndGearsUniqueError> {
    u8::try_from(value).map_err(|_| error(GoldAndGearsUniqueErrorKind::Identifier, key))
}

pub(super) fn optional_u8(
    value: Option<i32>,
    key: &str,
) -> Result<Option<u8>, GoldAndGearsUniqueError> {
    value.map(|value| positive_u8(value, key)).transpose()
}

pub(super) fn invalid<T>(key: &str) -> Result<T, GoldAndGearsUniqueError> {
    fail(GoldAndGearsUniqueErrorKind::Metadata, key)
}

pub(super) fn fail<T>(
    kind: GoldAndGearsUniqueErrorKind,
    key: &str,
) -> Result<T, GoldAndGearsUniqueError> {
    Err(error(kind, key))
}

pub(super) fn error(kind: GoldAndGearsUniqueErrorKind, key: &str) -> GoldAndGearsUniqueError {
    GoldAndGearsUniqueError {
        kind,
        key: key.into(),
    }
}

fn metadata(key: &str) -> Result<(), GoldAndGearsUniqueError> {
    stable(key).map(|_| ())
}
