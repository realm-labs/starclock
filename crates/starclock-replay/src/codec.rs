use core::fmt;

use crate::digest::{Sha256Sink, StateDigest};

/// Destination for the exact canonical byte stream.
pub trait CanonicalSink {
    /// Appends bytes without changing their boundaries or contents.
    fn write(&mut self, bytes: &[u8]);
}

impl CanonicalSink for Vec<u8> {
    fn write(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

/// Domain value that writes itself through the shared canonical encoder.
pub trait CanonicalEncode {
    /// Writes every field in its versioned canonical order.
    fn encode<S: CanonicalSink>(&self, encoder: &mut Encoder<S>) -> Result<(), CodecError>;
}

/// Little-endian canonical primitive encoder over one caller-selected sink.
#[derive(Debug)]
pub struct Encoder<S> {
    sink: S,
}

impl<S: CanonicalSink> Encoder<S> {
    /// Wraps a byte sink.
    #[must_use]
    pub const fn new(sink: S) -> Self {
        Self { sink }
    }
    /// Returns the completed sink.
    #[must_use]
    pub fn into_inner(self) -> S {
        self.sink
    }
    /// Writes unframed bytes owned by a higher-level fixed layout.
    pub fn raw(&mut self, value: &[u8]) {
        self.sink.write(value);
    }
    /// Writes one byte.
    pub fn u8(&mut self, value: u8) {
        self.raw(&[value]);
    }
    /// Writes an explicit canonical Boolean discriminant.
    pub fn boolean(&mut self, value: bool) {
        self.u8(u8::from(value));
    }
    /// Writes a little-endian `u16`.
    pub fn u16(&mut self, value: u16) {
        self.raw(&value.to_le_bytes());
    }
    /// Writes a little-endian `u32`.
    pub fn u32(&mut self, value: u32) {
        self.raw(&value.to_le_bytes());
    }
    /// Writes a little-endian `u64`.
    pub fn u64(&mut self, value: u64) {
        self.raw(&value.to_le_bytes());
    }
    /// Writes a little-endian `i64`.
    pub fn i64(&mut self, value: i64) {
        self.raw(&value.to_le_bytes());
    }
    /// Writes `u32` length-prefixed bytes.
    pub fn bytes(&mut self, value: &[u8]) -> Result<(), CodecError> {
        self.u32(u32::try_from(value.len()).map_err(|_| CodecError::LengthOverflow)?);
        self.raw(value);
        Ok(())
    }
    /// Writes a `u32` length-prefixed UTF-8 string.
    pub fn string(&mut self, value: &str) -> Result<(), CodecError> {
        self.bytes(value.as_bytes())
    }
}

/// Encodes directly into SHA-256 without building a complete byte vector.
pub fn hash_canonical(value: &impl CanonicalEncode) -> Result<StateDigest, CodecError> {
    let mut encoder = Encoder::new(Sha256Sink::new());
    value.encode(&mut encoder)?;
    Ok(StateDigest::new(encoder.into_inner().finalize().bytes()))
}

/// Borrowed canonical primitive decoder with checked cursor movement.
#[derive(Clone, Debug)]
pub struct Decoder<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Decoder<'a> {
    /// Starts decoding at byte zero.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }
    /// Returns the current byte offset.
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }
    /// Returns a checked borrowed fixed-length field.
    pub fn take(&mut self, length: usize) -> Result<&'a [u8], CodecError> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(CodecError::LengthOverflow)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(CodecError::UnexpectedEnd)?;
        self.position = end;
        Ok(value)
    }
    /// Reads one byte.
    pub fn u8(&mut self) -> Result<u8, CodecError> {
        Ok(self.take(1)?[0])
    }
    /// Reads an explicit canonical Boolean discriminant.
    pub fn boolean(&mut self) -> Result<bool, CodecError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(CodecError::InvalidPresence),
        }
    }
    /// Reads a little-endian `u16`.
    pub fn u16(&mut self) -> Result<u16, CodecError> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("fixed length"),
        ))
    }
    /// Reads a little-endian `u32`.
    pub fn u32(&mut self) -> Result<u32, CodecError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("fixed length"),
        ))
    }
    /// Reads a little-endian `u64`.
    pub fn u64(&mut self) -> Result<u64, CodecError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("fixed length"),
        ))
    }
    /// Reads a little-endian `i64`.
    pub fn i64(&mut self) -> Result<i64, CodecError> {
        Ok(i64::from_le_bytes(
            self.take(8)?.try_into().expect("fixed length"),
        ))
    }
    /// Reads borrowed `u32` length-prefixed bytes with a preallocation limit.
    pub fn bytes(&mut self, limit: u32) -> Result<&'a [u8], CodecError> {
        let length = self.u32()?;
        if length > limit {
            return Err(CodecError::LimitExceeded);
        }
        self.take(length as usize)
    }
    /// Reads a bounded UTF-8 string.
    pub fn string(&mut self, limit: u32) -> Result<&'a str, CodecError> {
        core::str::from_utf8(self.bytes(limit)?).map_err(|_| CodecError::InvalidUtf8)
    }
    /// Rejects trailing bytes.
    pub fn finish(self) -> Result<(), CodecError> {
        if self.position == self.bytes.len() {
            Ok(())
        } else {
            Err(CodecError::TrailingBytes)
        }
    }
}

/// Stable canonical encoding/decoding failure category.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecError {
    /// A collection/string length does not fit its fixed-width prefix.
    LengthOverflow,
    /// Input ended before a declared fixed field or payload.
    UnexpectedEnd,
    /// A Boolean or option presence byte is not zero or one.
    InvalidPresence,
    /// Text bytes are not valid UTF-8.
    InvalidUtf8,
    /// A configured bounded field exceeds its policy limit.
    LimitExceeded,
    /// Bytes remain after the declared structure ends.
    TrailingBytes,
}

impl fmt::Display for CodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "canonical codec error: {self:?}")
    }
}

impl std::error::Error for CodecError {}
