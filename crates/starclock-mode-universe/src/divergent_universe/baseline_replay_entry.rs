//! Mandatory accepted-entry payload for the current DU replay, not a legacy decoder.
use super::{DivergentUniverseReplayDivergenceKind, DivergentUniverseReplayError, divergence};
use crate::divergent_universe::DivergentUniverseFlowInstance;
use starclock_replay::{format::DecodedReplay, record::RecordKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct EntryInputs {
    pub(super) area: Box<str>,
    pub(super) difficulty: Box<str>,
    pub(super) tawot: Option<u16>,
    pub(super) source_deck_selection: bool,
}
impl EntryInputs {
    pub(super) fn from_flow(flow: &DivergentUniverseFlowInstance) -> Self {
        Self {
            area: flow.area().as_str().into(),
            difficulty: flow.difficulty().as_str().into(),
            tawot: flow.initial_tawot_service_level(),
            source_deck_selection: flow.has_source_deck_selection(),
        }
    }
    pub(super) fn encode(&self) -> Vec<u8> {
        let mut bytes = b"DUE".to_vec();
        for value in [&self.area, &self.difficulty] {
            bytes.extend_from_slice(
                &u32::try_from(value.len())
                    .expect("validated mode identity length fits u32")
                    .to_le_bytes(),
            );
            bytes.extend_from_slice(value.as_bytes());
        }
        bytes.extend_from_slice(&self.tawot.unwrap_or(0).to_le_bytes());
        bytes.push(u8::from(self.source_deck_selection));
        bytes
    }
    pub(super) fn decode(
        decoded: &DecodedReplay<'_>,
    ) -> Result<Self, DivergentUniverseReplayError> {
        let invalid = || divergence(DivergentUniverseReplayDivergenceKind::ActivityCommand, 0);
        let record = decoded.records().first().ok_or_else(invalid)?;
        if record.kind() != RecordKind::AcceptedActivityCommand {
            return Err(invalid());
        }
        Self::parse(record.payload()).ok_or_else(invalid)
    }
    fn parse(bytes: &[u8]) -> Option<Self> {
        let mut rest = bytes.strip_prefix(b"DUE")?;
        let area = text(&mut rest)?;
        let difficulty = text(&mut rest)?;
        let [low, high, selection] = <[u8; 3]>::try_from(rest).ok()?;
        let source_deck_selection = match selection {
            0 => false,
            1 => true,
            _ => return None,
        };
        let value = u16::from_le_bytes([low, high]);
        let tawot = match value {
            0 => None,
            2..=5 => Some(value),
            _ => return None,
        };
        Some(Self {
            area,
            difficulty,
            tawot,
            source_deck_selection,
        })
    }
}
fn text(rest: &mut &[u8]) -> Option<Box<str>> {
    let size = u32::from_le_bytes(rest.get(..4)?.try_into().ok()?);
    if !(1..=128).contains(&size) {
        return None;
    }
    let length = usize::try_from(size).ok()?;
    *rest = rest.get(4..)?;
    let value = core::str::from_utf8(rest.get(..length)?).ok()?;
    *rest = rest.get(length..)?;
    Some(value.into())
}

#[cfg(test)]
mod tests {
    use super::EntryInputs;

    #[test]
    fn source_deck_entry_flag_is_required_boolean_and_round_trips() {
        for source_deck_selection in [false, true] {
            for tawot in [None, Some(2), Some(5)] {
                let entry = EntryInputs {
                    area: "divergent-universe.area.401".into(),
                    difficulty: "divergent-universe.difficulty.3011".into(),
                    tawot,
                    source_deck_selection,
                };
                let bytes = entry.encode();
                assert_eq!(EntryInputs::parse(&bytes), Some(entry));
                let mut missing = bytes.clone();
                missing.pop();
                assert!(EntryInputs::parse(&missing).is_none());
                let mut extra = bytes.clone();
                extra.push(0);
                assert!(EntryInputs::parse(&extra).is_none());
                let mut invalid = bytes;
                *invalid.last_mut().unwrap() = 2;
                assert!(EntryInputs::parse(&invalid).is_none());
            }
        }
    }
}
