//! Private bounded decoder and Activity identity validation for service payloads.

use starclock_activity::{
    ActivityHandlerFault, ActivityHandlerFaultKind, ActivityInventoryId, ActivitySlotId,
};

pub(super) fn slot(value: u32) -> Result<ActivitySlotId, ActivityHandlerFault> {
    ActivitySlotId::new(value).ok_or_else(invalid_payload)
}

pub(super) fn inventory(value: u32) -> Result<ActivityInventoryId, ActivityHandlerFault> {
    ActivityInventoryId::new(value).ok_or_else(invalid_payload)
}

pub(super) struct Decoder<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Decoder<'a> {
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], ActivityHandlerFault> {
        let end = self.cursor.checked_add(count).ok_or_else(invalid_payload)?;
        let value = self
            .bytes
            .get(self.cursor..end)
            .ok_or_else(invalid_payload)?;
        self.cursor = end;
        Ok(value)
    }

    pub(super) fn u8(&mut self) -> Result<u8, ActivityHandlerFault> {
        Ok(self.take(1)?[0])
    }

    pub(super) fn u16(&mut self) -> Result<u16, ActivityHandlerFault> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    pub(super) fn u32(&mut self) -> Result<u32, ActivityHandlerFault> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub(super) fn u64(&mut self) -> Result<u64, ActivityHandlerFault> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    pub(super) fn i64(&mut self) -> Result<i64, ActivityHandlerFault> {
        Ok(i64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    pub(super) fn finish(self) -> Result<(), ActivityHandlerFault> {
        if self.cursor == self.bytes.len() {
            Ok(())
        } else {
            Err(invalid_payload())
        }
    }
}

pub(super) fn invalid_payload() -> ActivityHandlerFault {
    ActivityHandlerFault::new(ActivityHandlerFaultKind::InvalidPayload)
}

pub(super) fn invalid_state() -> ActivityHandlerFault {
    ActivityHandlerFault::new(ActivityHandlerFaultKind::InvalidState)
}
