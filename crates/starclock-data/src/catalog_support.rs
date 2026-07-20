//! Shared validation helpers kept outside catalog orchestration.

use crate::catalog::{CatalogLoadError, CatalogLoadErrorKind};

pub(super) fn parse_decimal(source: &str) -> Result<i64, CatalogLoadError> {
    let (negative, unsigned) = source
        .strip_prefix('-')
        .map_or((false, source), |rest| (true, rest));
    if unsigned.is_empty() || (negative && unsigned == "0") {
        return Err(decimal_error(source));
    }
    let mut parts = unsigned.split('.');
    let integer = parts.next().expect("split always has one part");
    let fraction = parts.next();
    if parts.next().is_some()
        || integer.is_empty()
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || (integer.len() > 1 && integer.starts_with('0'))
        || fraction.is_some_and(|value| {
            value.is_empty()
                || value.len() > 6
                || !value.bytes().all(|byte| byte.is_ascii_digit())
                || value.ends_with('0')
        })
    {
        return Err(decimal_error(source));
    }
    let integer = integer.parse::<i128>().map_err(|_| decimal_error(source))?;
    let fraction_text = fraction.unwrap_or("");
    let fraction_value = if fraction_text.is_empty() {
        0
    } else {
        fraction_text
            .parse::<i128>()
            .map_err(|_| decimal_error(source))?
            * 10_i128.pow(6 - u32::try_from(fraction_text.len()).expect("length is at most six"))
    };
    let magnitude = integer
        .checked_mul(1_000_000)
        .and_then(|value| value.checked_add(fraction_value))
        .ok_or_else(|| decimal_error(source))?;
    let scaled = if negative {
        magnitude
            .checked_neg()
            .ok_or_else(|| decimal_error(source))?
    } else {
        magnitude
    };
    i64::try_from(scaled).map_err(|_| decimal_error(source))
}

fn decimal_error(source: &str) -> CatalogLoadError {
    fail(
        CatalogLoadErrorKind::Domain,
        format!("{source:?} is not a canonical six-place decimal"),
    )
}

pub(super) fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    let number =
        |range: std::ops::Range<usize>| value.get(range).and_then(|part| part.parse::<u16>().ok());
    matches!(number(0..4), Some(1..=9999))
        && matches!(number(5..7), Some(1..=12))
        && matches!(number(8..10), Some(1..=31))
}

pub(super) fn fail(
    kind: CatalogLoadErrorKind,
    message: impl std::fmt::Display,
) -> CatalogLoadError {
    CatalogLoadError {
        kind,
        message: message.to_string(),
    }
}

pub(super) fn domain_fail(message: impl std::fmt::Display) -> CatalogLoadError {
    fail(CatalogLoadErrorKind::Domain, message)
}
