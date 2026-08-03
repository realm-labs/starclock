//! Bounded in-memory frame retention independent from battle execution.

use std::collections::VecDeque;

use crate::InspectorFrame;

pub const MAX_RETAINED_INSPECTOR_FRAMES: usize = 4_096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InspectorHistoryError {
    InvalidCapacity,
}

impl core::fmt::Display for InspectorHistoryError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("inspector history capacity must be within its fixed bound")
    }
}

impl std::error::Error for InspectorHistoryError {}

/// Bounded oldest-first frame history owned entirely outside combat state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectorHistory {
    capacity: usize,
    dropped: u64,
    frames: VecDeque<InspectorFrame>,
}

impl InspectorHistory {
    pub fn new(capacity: usize) -> Result<Self, InspectorHistoryError> {
        if capacity == 0 || capacity > MAX_RETAINED_INSPECTOR_FRAMES {
            return Err(InspectorHistoryError::InvalidCapacity);
        }
        Ok(Self {
            capacity,
            dropped: 0,
            frames: VecDeque::with_capacity(capacity),
        })
    }

    pub fn push(&mut self, frame: InspectorFrame) {
        if self.frames.len() == self.capacity {
            self.frames.pop_front();
            self.dropped = self.dropped.saturating_add(1);
        }
        self.frames.push_back(frame);
    }

    #[must_use]
    pub fn frames(&self) -> impl ExactSizeIterator<Item = &InspectorFrame> {
        self.frames.iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    #[must_use]
    pub const fn dropped(&self) -> u64 {
        self.dropped
    }

    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }
}
