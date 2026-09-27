//! Mandatory accepted-entry payload for the current DU replay, not a legacy decoder.
use super::{DivergentUniverseReplayDivergenceKind, DivergentUniverseReplayError, divergence};
use crate::divergent_universe::DivergentUniverseFlowInstance;
use starclock_data::divergent_universe_titan_catalog::DivergentUniverseTitanTalentId;
use starclock_replay::{format::DecodedReplay, record::RecordKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct EntryInputs {
    pub(super) area: Box<str>,
    pub(super) difficulty: Box<str>,
    pub(super) tawot: Option<u16>,
    pub(super) source_deck_selection: bool,
    pub(super) titan_talents: Box<[DivergentUniverseTitanTalentId]>,
}
impl EntryInputs {
    pub(super) fn from_flow(flow: &DivergentUniverseFlowInstance) -> Self {
        Self {
            area: flow.area().as_str().into(),
            difficulty: flow.difficulty().as_str().into(),
            tawot: flow.initial_tawot_service_level(),
            source_deck_selection: flow.has_source_deck_selection(),
            titan_talents: flow.entry_titan_talents().into(),
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
        bytes.extend_from_slice(
            &u16::try_from(self.titan_talents.len())
                .expect("validated at most 36 entry talents")
                .to_le_bytes(),
        );
        for talent in &self.titan_talents {
            let value = talent.as_str().as_bytes();
            bytes.extend_from_slice(
                &u32::try_from(value.len())
                    .expect("validated talent identity fits u32")
                    .to_le_bytes(),
            );
            bytes.extend_from_slice(value);
        }
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
        let [low, high, selection] = <[u8; 3]>::try_from(rest.get(..3)?).ok()?;
        rest = rest.get(3..)?;
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
        let count = u16::from_le_bytes(rest.get(..2)?.try_into().ok()?);
        if count > 36 {
            return None;
        }
        rest = rest.get(2..)?;
        let mut titan_talents = Vec::with_capacity(usize::from(count));
        for _ in 0..count {
            titan_talents.push(DivergentUniverseTitanTalentId::new(text(&mut rest)?).ok()?);
        }
        if !rest.is_empty() || titan_talents.windows(2).any(|pair| pair[0] >= pair[1]) {
            return None;
        }
        Some(Self {
            area,
            difficulty,
            tawot,
            source_deck_selection,
            titan_talents: titan_talents.into_boxed_slice(),
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
    use crate::divergent_universe::DivergentUniverseBaselineFixture;
    use starclock_activity::{ActivityInstanceId, ActivityMasterSeed};
    use starclock_data::{
        divergent_universe_catalog::DivergentUniverseRunFamily,
        divergent_universe_titan_catalog::DivergentUniverseTitanTalentId,
    };

    #[test]
    fn titan_entry_payload_requires_bounded_canonical_talent_ids() {
        let mut entry = EntryInputs {
            area: "divergent-universe.area.401".into(),
            difficulty: "divergent-universe.difficulty.3011".into(),
            tawot: None,
            source_deck_selection: false,
            titan_talents: Box::new([]),
        };
        let empty = entry.encode();
        for count in [37_u16, u16::MAX] {
            let mut invalid = empty.clone();
            let offset = invalid.len() - 2;
            invalid[offset..].copy_from_slice(&count.to_le_bytes());
            assert!(EntryInputs::parse(&invalid).is_none());
        }
        let a =
            DivergentUniverseTitanTalentId::new("divergent-universe.titan-talent.12002").unwrap();
        let b =
            DivergentUniverseTitanTalentId::new("divergent-universe.titan-talent.12302").unwrap();
        entry.titan_talents = vec![a.clone(), b.clone()].into_boxed_slice();
        let valid = entry.encode();
        assert_eq!(EntryInputs::parse(&valid), Some(entry.clone()));
        for length in 0..valid.len() {
            assert!(EntryInputs::parse(&valid[..length]).is_none());
        }
        let mut trailing = valid;
        trailing.push(0);
        assert!(EntryInputs::parse(&trailing).is_none());
        for invalid in [vec![a.clone(), a], vec![b, entry.titan_talents[0].clone()]] {
            entry.titan_talents = invalid.into_boxed_slice();
            assert!(EntryInputs::parse(&entry.encode()).is_none());
        }
    }

    #[test]
    fn titan_entry_payload_reconstructs_fresh_production_entry_state_and_events() {
        let original = DivergentUniverseBaselineFixture::production().unwrap();
        let fresh = DivergentUniverseBaselineFixture::production().unwrap();
        let runtime = original.factory().titan_runtime().unwrap();
        let mut id = Some(
            DivergentUniverseTitanTalentId::new("divergent-universe.titan-talent.12302").unwrap(),
        );
        let mut talents = Vec::new();
        while let Some(selected) = id {
            let talent = runtime
                .talents()
                .iter()
                .find(|talent| talent.id() == &selected)
                .unwrap();
            id = talent.predecessor().cloned();
            talents.push(selected);
        }
        for (family, area) in [
            (
                DivergentUniverseRunFamily::Ordinary,
                "divergent-universe.area.401",
            ),
            (
                DivergentUniverseRunFamily::Cyclical,
                "divergent-universe.area.20401",
            ),
        ] {
            let flow = original
                .flow_for_entry_configuration(
                    family,
                    area,
                    "divergent-universe.difficulty.3011",
                    None,
                    true,
                    &talents,
                )
                .unwrap();
            let inputs = EntryInputs::from_flow(&flow);
            let decoded = EntryInputs::parse(&inputs.encode()).unwrap();
            assert_eq!(inputs, decoded);
            let rebuilt = fresh
                .flow_for_entry_configuration(
                    family,
                    &decoded.area,
                    &decoded.difficulty,
                    decoded.tawot,
                    decoded.source_deck_selection,
                    &decoded.titan_talents,
                )
                .unwrap();
            assert_eq!(
                flow.definition().identity(),
                rebuilt.definition().identity()
            );
            let instance = ActivityInstanceId::new(1).unwrap();
            let seed = ActivityMasterSeed::from_u64(24201);
            let a = flow.start(instance, seed).unwrap();
            let b = rebuilt.start(instance, seed).unwrap();
            assert_eq!(a.events(), b.events());
            assert_eq!(
                a.into_activity().canonical_state_bytes(),
                b.into_activity().canonical_state_bytes()
            );
        }
    }

    #[test]
    fn source_deck_entry_flag_is_required_boolean_and_round_trips() {
        for source_deck_selection in [false, true] {
            for tawot in [None, Some(2), Some(5)] {
                let entry = EntryInputs {
                    area: "divergent-universe.area.401".into(),
                    difficulty: "divergent-universe.difficulty.3011".into(),
                    tawot,
                    source_deck_selection,
                    titan_talents: Box::new([]),
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
                let flag = invalid.len() - 3;
                invalid[flag] = 2;
                assert!(EntryInputs::parse(&invalid).is_none());
            }
        }
    }
}
