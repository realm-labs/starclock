//! Current Persona position joins. Unspecified positions are not random pools.

use std::collections::{BTreeMap, BTreeSet};

use crate::divergent_universe::DivergentUniverseBundleCandidate;
use crate::divergent_universe_catalog::DivergentUniverseLayerId;
use crate::divergent_universe_decisions::DecisionDataError;
use crate::divergent_universe_decisions_generated::{
    SoraConfig, du_domain_slot_kind::DuDomainSlotKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixedDomainKind {
    Battle,
    Boss,
    Respite,
    Conversion,
    Blank,
    Coin,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainPositionKind {
    /// Source omits a preset. No deck, replacement or candidate rule is implied.
    Unspecified,
    Fixed {
        kind: FixedDomainKind,
        level: u16,
        /// Upstream locator only; never a runtime content identity.
        preset_source: Box<str>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainPosition {
    pub key: Box<str>,
    pub ordinal: u16,
    pub kind: DomainPositionKind,
    pub source_locator: Box<str>,
    pub interpretation_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainLayerLayout {
    pub layer: DivergentUniverseLayerId,
    pub positions: Box<[DomainPosition]>,
}

pub(crate) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[DomainLayerLayout]>, DecisionDataError> {
    let reachable = reference
        .catalog()
        .areas()
        .iter()
        .flat_map(|area| area.layers.iter().cloned())
        .collect::<BTreeSet<_>>();
    let mut layers = BTreeMap::<DivergentUniverseLayerId, Vec<DomainPosition>>::new();
    let mut keys = BTreeSet::new();
    for row in config.du_domain_layout().ordered_rows() {
        let layer = DivergentUniverseLayerId::new(row.layer_key.clone())
            .map_err(|_| DecisionDataError::InvalidIdentity)?;
        if !reachable.contains(&layer) || row.id <= 0 {
            return Err(DecisionDataError::InvalidReference);
        }
        let ordinal = u16::try_from(row.ordinal).map_err(|_| DecisionDataError::InvalidOrder)?;
        if ordinal == 0 || ordinal > 64 || !keys.insert(row.stable_key.as_str()) {
            return Err(DecisionDataError::InvalidOrder);
        }
        let raw_layer = layer
            .as_str()
            .strip_prefix("divergent-universe.layer.")
            .ok_or(DecisionDataError::InvalidIdentity)?;
        if row.stable_key != format!("du.domain-slot.layer-{raw_layer}.position-{ordinal}")
            || row.source_locator
                != format!(
                    "RoguePersonaLayerRoom:CBCHIHEOEGK={raw_layer};EEPIDJJJMAH={ordinal};BKHDBIFFIKP={}",
                    row.preset_source.as_deref().unwrap_or("absent")
                )
            || row.interpretation_note.is_empty()
            || row.replacement_condition.is_empty()
        {
            return Err(DecisionDataError::InvalidProvenance);
        }
        let fixed = match row.kind {
            DuDomainSlotKind::Unspecified => None,
            DuDomainSlotKind::Battle => Some(FixedDomainKind::Battle),
            DuDomainSlotKind::Boss => Some(FixedDomainKind::Boss),
            DuDomainSlotKind::Respite => Some(FixedDomainKind::Respite),
            DuDomainSlotKind::Conversion => Some(FixedDomainKind::Conversion),
            DuDomainSlotKind::Blank => Some(FixedDomainKind::Blank),
            DuDomainSlotKind::Coin => Some(FixedDomainKind::Coin),
        };
        let kind = match (fixed, row.preset_source.as_deref(), row.level) {
            (None, None, None) => DomainPositionKind::Unspecified,
            (Some(kind), Some(preset), Some(level)) => {
                // Reviewed source joins. Same-shape substitution is not proof of
                // a newly admitted current preset or its composition semantics.
                let expected = match preset {
                    "1001" => (FixedDomainKind::Battle, 3),
                    "1002" => (FixedDomainKind::Boss, 1),
                    "1003" => (FixedDomainKind::Respite, 1),
                    "1004" => (FixedDomainKind::Conversion, 1),
                    "9007" => (FixedDomainKind::Blank, 1),
                    "9008" => (FixedDomainKind::Coin, 1),
                    _ => return Err(DecisionDataError::InvalidReference),
                };
                if (kind, level) != expected {
                    return Err(DecisionDataError::InvalidReference);
                }
                DomainPositionKind::Fixed {
                    kind,
                    level: u16::try_from(level).map_err(|_| DecisionDataError::InvalidPolicy)?,
                    preset_source: preset.into(),
                }
            }
            _ => return Err(DecisionDataError::InvalidPolicy),
        };
        let sources = row
            .source_ids
            .iter()
            .map(|id| {
                config
                    .du_decision_sources()
                    .get(id)
                    .map(|source| source.stable_key.clone().into_boxed_str())
                    .ok_or(DecisionDataError::InvalidProvenance)
            })
            .collect::<Result<Box<[_]>, _>>()?;
        if sources.iter().map(AsRef::as_ref).collect::<Vec<&str>>()
            != [
                "du.source.layer-battle-route",
                "du.source.domain-layout.positions",
                "du.source.domain-layout.presets",
                "du.source.domain-layout.types",
            ]
        {
            return Err(DecisionDataError::InvalidProvenance);
        }
        layers.entry(layer).or_default().push(DomainPosition {
            key: row.stable_key.clone().into(),
            ordinal,
            kind,
            source_locator: row.source_locator.clone().into(),
            interpretation_note: row.interpretation_note.clone().into(),
            replacement_condition: row.replacement_condition.clone().into(),
            sources,
        });
    }
    if layers.keys().cloned().collect::<BTreeSet<_>>() != reachable {
        return Err(DecisionDataError::InvalidReference);
    }
    layers
        .into_iter()
        .map(|(layer, mut positions)| {
            positions.sort_by_key(|position| position.ordinal);
            if positions
                .iter()
                .enumerate()
                .any(|(index, position)| usize::from(position.ordinal) != index + 1)
                || !matches!(
                    positions.last().map(|position| &position.kind),
                    Some(DomainPositionKind::Fixed {
                        kind: FixedDomainKind::Boss,
                        ..
                    })
                )
            {
                return Err(DecisionDataError::InvalidOrder);
            }
            Ok(DomainLayerLayout {
                layer,
                positions: positions.into_boxed_slice(),
            })
        })
        .collect()
}
