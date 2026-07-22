use std::collections::BTreeMap;
use std::fmt;

use rand::{Rng, SeedableRng, rngs::ChaCha8Rng};
use sha2::{Digest, Sha256};

use crate::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId, ActivityGraphDigest,
    ActivityInstanceId, ActivityMasterSeed, AttemptId, NodeId, SectionId,
};

pub const ACTIVITY_RNG_REVISION: &str = "starclock-activity-rng-v2";
const MAX_REJECTIONS: u32 = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ActivityRngLabel {
    Graph = 0,
    Encounter = 1,
    Reward = 2,
    Shop = 3,
    Occurrence = 4,
    Spawn = 5,
    ExternalOutcomeTest = 6,
    Battle = 7,
}

impl ActivityRngLabel {
    pub const ALL: [Self; 8] = [
        Self::Graph,
        Self::Encounter,
        Self::Reward,
        Self::Shop,
        Self::Occurrence,
        Self::Spawn,
        Self::ExternalOutcomeTest,
        Self::Battle,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityRngContext {
    master: ActivityMasterSeed,
    definition_id: ActivityDefinitionId,
    definition_digest: ActivityDefinitionDigest,
    config_digest: ActivityConfigDigest,
    graph_digest: ActivityGraphDigest,
    instance: ActivityInstanceId,
    section: Option<SectionId>,
    node: Option<NodeId>,
    attempt: Option<AttemptId>,
    battle_sequence: u32,
}

impl ActivityRngContext {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        master: ActivityMasterSeed,
        definition_id: ActivityDefinitionId,
        definition_digest: ActivityDefinitionDigest,
        config_digest: ActivityConfigDigest,
        graph_digest: ActivityGraphDigest,
        instance: ActivityInstanceId,
        section: Option<SectionId>,
        node: Option<NodeId>,
        attempt: Option<AttemptId>,
        battle_sequence: u32,
    ) -> Self {
        Self {
            master,
            definition_id,
            definition_digest,
            config_digest,
            graph_digest,
            instance,
            section,
            node,
            attempt,
            battle_sequence,
        }
    }

    fn derive(self, label: ActivityRngLabel) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"SCAR");
        hash.update(2_u32.to_le_bytes());
        write_text(&mut hash, ACTIVITY_RNG_REVISION);
        hash.update(self.master.bytes());
        hash.update(self.definition_id.get().to_le_bytes());
        hash.update(self.definition_digest.bytes());
        hash.update(self.config_digest.bytes());
        hash.update(self.graph_digest.bytes());
        hash.update(self.instance.get().to_le_bytes());
        write_optional_u32(&mut hash, self.section.map(SectionId::get));
        write_optional_u32(&mut hash, self.node.map(NodeId::get));
        write_optional_u32(&mut hash, self.attempt.map(AttemptId::get));
        hash.update(self.battle_sequence.to_le_bytes());
        hash.update([label as u8]);
        hash.finalize().into()
    }
}

struct ActivityRngStream {
    seed: [u8; 32],
    draws: u64,
    inner: ChaCha8Rng,
}

impl ActivityRngStream {
    fn new(seed: [u8; 32]) -> Self {
        Self {
            seed,
            draws: 0,
            inner: ChaCha8Rng::from_seed(seed),
        }
    }

    fn raw(&mut self) -> Result<(u64, u64), ActivityRngError> {
        let next = self
            .draws
            .checked_add(1)
            .ok_or(ActivityRngError::DrawCounterExhausted)?;
        let index = self.draws;
        let raw = self.inner.next_u64();
        self.draws = next;
        Ok((index, raw))
    }
}

pub struct ActivityRngStreams {
    streams: BTreeMap<ActivityRngLabel, ActivityRngStream>,
}

impl core::fmt::Debug for ActivityRngStreams {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ActivityRngStreams")
            .field("snapshots", &self.snapshots())
            .finish()
    }
}

impl ActivityRngStreams {
    #[must_use]
    pub fn new(context: ActivityRngContext) -> Self {
        let streams = ActivityRngLabel::ALL
            .into_iter()
            .map(|label| (label, ActivityRngStream::new(context.derive(label))))
            .collect();
        Self { streams }
    }

    pub fn choose_index(
        &mut self,
        label: ActivityRngLabel,
        purpose: u16,
        candidate_count: u32,
    ) -> Result<Option<ActivityRngDraw>, ActivityRngError> {
        if purpose == 0 {
            return Err(ActivityRngError::InvalidPurpose);
        }
        if candidate_count == 0 {
            return Ok(None);
        }
        self.sample_below(label, purpose, u64::from(candidate_count))
            .map(Some)
    }

    pub fn choose_weighted(
        &mut self,
        label: ActivityRngLabel,
        purpose: u16,
        weights: &[u64],
    ) -> Result<Option<(u32, ActivityRngDraw)>, ActivityRngError> {
        if purpose == 0 {
            return Err(ActivityRngError::InvalidPurpose);
        }
        let _ = u32::try_from(weights.len()).map_err(|_| ActivityRngError::TooManyCandidates)?;
        let total = weights
            .iter()
            .try_fold(0_u64, |sum, weight| sum.checked_add(*weight))
            .ok_or(ActivityRngError::WeightOverflow)?;
        if total == 0 {
            return Ok(None);
        }
        let draw = self.sample_below(label, purpose, total)?;
        let mut cumulative = 0_u64;
        for (index, weight) in weights.iter().copied().enumerate() {
            cumulative = cumulative
                .checked_add(weight)
                .ok_or(ActivityRngError::WeightOverflow)?;
            if draw.value < cumulative {
                return Ok(Some((index as u32, draw)));
            }
        }
        Err(ActivityRngError::MappingInvariant)
    }

    #[must_use]
    pub fn snapshots(&self) -> Box<[ActivityRngStreamSnapshot]> {
        self.streams
            .iter()
            .map(|(label, stream)| ActivityRngStreamSnapshot {
                label: *label,
                seed: stream.seed,
                draws: stream.draws,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn sample_below(
        &mut self,
        label: ActivityRngLabel,
        purpose: u16,
        upper: u64,
    ) -> Result<ActivityRngDraw, ActivityRngError> {
        if purpose == 0 {
            return Err(ActivityRngError::InvalidPurpose);
        }
        if upper == 0 {
            return Err(ActivityRngError::EmptyRange);
        }
        let stream = self
            .streams
            .get_mut(&label)
            .ok_or(ActivityRngError::MissingStream)?;
        let threshold = upper.wrapping_neg() % upper;
        let mut rejected = 0_u32;
        loop {
            let (index, raw) = stream.raw()?;
            if raw >= threshold {
                return Ok(ActivityRngDraw {
                    label,
                    purpose,
                    index,
                    raw,
                    upper,
                    value: raw % upper,
                    rejected,
                });
            }
            rejected = rejected
                .checked_add(1)
                .ok_or(ActivityRngError::RejectionBudgetExhausted)?;
            if rejected > MAX_REJECTIONS {
                return Err(ActivityRngError::RejectionBudgetExhausted);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityRngStreamSnapshot {
    label: ActivityRngLabel,
    seed: [u8; 32],
    draws: u64,
}
impl ActivityRngStreamSnapshot {
    #[must_use]
    pub const fn label(self) -> ActivityRngLabel {
        self.label
    }
    #[must_use]
    pub const fn seed(self) -> [u8; 32] {
        self.seed
    }
    #[must_use]
    pub const fn draw_count(self) -> u64 {
        self.draws
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityRngDraw {
    label: ActivityRngLabel,
    purpose: u16,
    index: u64,
    raw: u64,
    upper: u64,
    value: u64,
    rejected: u32,
}
impl ActivityRngDraw {
    #[must_use]
    pub const fn label(self) -> ActivityRngLabel {
        self.label
    }
    #[must_use]
    pub const fn purpose(self) -> u16 {
        self.purpose
    }
    #[must_use]
    pub const fn index(self) -> u64 {
        self.index
    }
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.raw
    }
    #[must_use]
    pub const fn upper(self) -> u64 {
        self.upper
    }
    #[must_use]
    pub const fn value(self) -> u64 {
        self.value
    }
    #[must_use]
    pub const fn rejected_draws(self) -> u32 {
        self.rejected
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityRngError {
    InvalidPurpose,
    EmptyRange,
    DrawCounterExhausted,
    RejectionBudgetExhausted,
    WeightOverflow,
    TooManyCandidates,
    MissingStream,
    MappingInvariant,
}

impl fmt::Display for ActivityRngError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "activity RNG error: {self:?}")
    }
}

impl std::error::Error for ActivityRngError {}

fn write_text(hash: &mut Sha256, value: &str) {
    hash.update((value.len() as u32).to_le_bytes());
    hash.update(value.as_bytes());
}
fn write_optional_u32(hash: &mut Sha256, value: Option<u32>) {
    hash.update([u8::from(value.is_some())]);
    if let Some(value) = value {
        hash.update(value.to_le_bytes());
    }
}
