//! Immutable Protocol, Astronomical Division and persistent progression definitions.

use crate::divergent_universe_catalog::DivergentUniverseFlowCatalog;
use std::collections::{BTreeMap, BTreeSet};

macro_rules! stable_id {
    ($name:ident,$prefix:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(Box<str>);
        impl $name {
            pub fn new(v: impl Into<Box<str>>) -> Result<Self, DivergentUniverseProgressionError> {
                let v = v.into();
                if !v.starts_with($prefix) || v.len() == $prefix.len() {
                    Err(error(concat!(stringify!($name), " namespace mismatch")))
                } else {
                    Ok(Self(v))
                }
            }
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}
stable_id!(DivergentUniverseProtocolId, "divergent-universe.protocol.");
stable_id!(
    DivergentUniverseDivisionId,
    "divergent-universe.astronomical-division."
);
stable_id!(
    DivergentUniverseAstronomicalModeId,
    "divergent-universe.astronomical-mode."
);
stable_id!(
    DivergentUniverseCognoculiId,
    "divergent-universe.cognoculi."
);
stable_id!(
    DivergentUniversePermanentTalentId,
    "divergent-universe.permanent-talent."
);
stable_id!(DivergentUniverseUnlockId, "divergent-universe.unlock.");
stable_id!(DivergentUniverseConstantId, "divergent-universe.constant.");
stable_id!(
    DivergentUniverseWeeklyModifierId,
    "divergent-universe.weekly-modifier."
);
stable_id!(DivergentUniverseRoomMarkId, "divergent-universe.room-mark.");
stable_id!(
    DivergentUniverseProgressionEffectId,
    "divergent-universe.progression-effect."
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniversePlaneScaling {
    pub attack: Box<str>,
    pub max_hp: Box<str>,
    pub speed: Box<str>,
    pub max_toughness: Option<Box<str>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseProtocolDefinition {
    pub id: DivergentUniverseProtocolId,
    pub level: u16,
    pub difficulty_changes: Box<[Box<str>]>,
    pub entry_rules: Box<[Box<str>]>,
    pub berserk_changes: Box<[Box<str>]>,
    pub boss_identity: Box<str>,
    pub plane_scaling: DivergentUniversePlaneScaling,
    pub source_parameters: Box<[Box<str>]>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseDivisionDefinition {
    pub id: DivergentUniverseDivisionId,
    pub level: u16,
    pub protocols: Box<[DivergentUniverseProtocolId]>,
    pub progress_boundary: Box<str>,
    pub cognoculi_retention: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseAstronomicalModeDefinition {
    pub id: DivergentUniverseAstronomicalModeId,
    pub mode_kind: Box<str>,
    pub available_content: Box<[Box<str>]>,
    pub entry_rules: Box<[Box<str>]>,
    pub reset_rules: Box<[Box<str>]>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCognoculiDefinition {
    pub id: DivergentUniverseCognoculiId,
    pub division: DivergentUniverseDivisionId,
    pub contribution_divisions: Box<[DivergentUniverseDivisionId]>,
    pub retention: Box<str>,
    pub gain: Box<str>,
    pub loss: Box<str>,
    pub division_floor: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseProgressionProgram {
    pub condition: Box<str>,
    pub description_hash: Option<Box<str>>,
    pub metric: Box<str>,
    pub operation: Box<str>,
    pub parameters: Box<[Box<str>]>,
    pub scope: Option<Box<str>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseTalentCost {
    pub item_id: Box<str>,
    pub amount: Box<str>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniversePermanentTalentDefinition {
    pub id: DivergentUniversePermanentTalentId,
    pub adjacent: Box<[DivergentUniversePermanentTalentId]>,
    pub prerequisites: Box<[DivergentUniversePermanentTalentId]>,
    pub prerequisite_resolution: Box<str>,
    pub cost: Box<[DivergentUniverseTalentCost]>,
    pub effects: Box<[DivergentUniverseProgressionEffectId]>,
    pub program: DivergentUniverseProgressionProgram,
    pub important: bool,
    pub scope: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseUnlockDefinition {
    pub id: DivergentUniverseUnlockId,
    pub finish_condition_id: Box<str>,
    pub scope: Box<str>,
    pub unlocked_content_ids: Box<[Box<str>]>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DivergentUniverseConstantValue {
    Scalar(Box<str>),
    Array(Box<[Box<str>]>),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseConstantDefinition {
    pub id: DivergentUniverseConstantId,
    pub canonical_value: DivergentUniverseConstantValue,
    pub consumers: Box<[Box<str>]>,
    pub exclusion_reason: Option<Box<str>>,
    pub value_kind: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseWeeklyEnemyGroup {
    pub slot: Box<str>,
    pub source_group_id: Box<str>,
    pub variant: Box<str>,
    pub resolution: Box<str>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseWeeklyModifierDefinition {
    pub id: DivergentUniverseWeeklyModifierId,
    pub content_ids: Box<[Box<str>]>,
    pub detail_ids: Box<[Box<str>]>,
    pub effect_ids: Box<[Box<str>]>,
    pub enemy_groups: Box<[DivergentUniverseWeeklyEnemyGroup]>,
    pub reachability: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseRoomMarkDefinition {
    pub id: DivergentUniverseRoomMarkId,
    pub room_type: Box<str>,
    pub mark_kind: Box<str>,
    pub transition_rules: Box<[Box<str>]>,
    pub fallback: Box<str>,
    pub runtime_lowered: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseProgressionEffectDefinition {
    pub id: DivergentUniverseProgressionEffectId,
    pub activation: Box<str>,
    pub contributions: Box<[DivergentUniverseProgressionProgram]>,
    pub scope: Box<str>,
    pub runtime_lowered: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseProgressionCatalogParts {
    pub protocols: Vec<DivergentUniverseProtocolDefinition>,
    pub divisions: Vec<DivergentUniverseDivisionDefinition>,
    pub modes: Vec<DivergentUniverseAstronomicalModeDefinition>,
    pub cognoculi: Vec<DivergentUniverseCognoculiDefinition>,
    pub talents: Vec<DivergentUniversePermanentTalentDefinition>,
    pub unlocks: Vec<DivergentUniverseUnlockDefinition>,
    pub constants: Vec<DivergentUniverseConstantDefinition>,
    pub weekly_modifiers: Vec<DivergentUniverseWeeklyModifierDefinition>,
    pub room_marks: Vec<DivergentUniverseRoomMarkDefinition>,
    pub effects: Vec<DivergentUniverseProgressionEffectDefinition>,
    pub source_obligations: usize,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseProgressionCatalog {
    parts: DivergentUniverseProgressionCatalogParts,
}
impl DivergentUniverseProgressionCatalog {
    pub fn new(
        mut p: DivergentUniverseProgressionCatalogParts,
        flow: &DivergentUniverseFlowCatalog,
    ) -> Result<Self, DivergentUniverseProgressionError> {
        let counts = [
            (p.protocols.len(), 8),
            (p.divisions.len(), 9),
            (p.modes.len(), 2),
            (p.cognoculi.len(), 9),
            (p.talents.len(), 38),
            (p.unlocks.len(), 97),
            (p.constants.len(), 34),
            (p.weekly_modifiers.len(), 103),
            (p.room_marks.len(), 24),
            (p.effects.len(), 38),
        ];
        if counts.iter().any(|(a, e)| a != e) {
            return Err(error("Protocol/progression table denominator drift"));
        }
        if p.source_obligations != 313 {
            return Err(error(
                "Protocol/progression source obligation closure drift",
            ));
        }
        p.protocols.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.divisions.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.modes.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.cognoculi.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.talents.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.unlocks.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.constants.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.weekly_modifiers.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.room_marks.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        p.effects.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        validate(&p, flow)?;
        Ok(Self { parts: p })
    }
    #[must_use]
    pub fn protocols(&self) -> &[DivergentUniverseProtocolDefinition] {
        &self.parts.protocols
    }
    #[must_use]
    pub fn divisions(&self) -> &[DivergentUniverseDivisionDefinition] {
        &self.parts.divisions
    }
    #[must_use]
    pub fn modes(&self) -> &[DivergentUniverseAstronomicalModeDefinition] {
        &self.parts.modes
    }
    #[must_use]
    pub fn cognoculi(&self) -> &[DivergentUniverseCognoculiDefinition] {
        &self.parts.cognoculi
    }
    #[must_use]
    pub fn talents(&self) -> &[DivergentUniversePermanentTalentDefinition] {
        &self.parts.talents
    }
    #[must_use]
    pub fn unlocks(&self) -> &[DivergentUniverseUnlockDefinition] {
        &self.parts.unlocks
    }
    #[must_use]
    pub fn constants(&self) -> &[DivergentUniverseConstantDefinition] {
        &self.parts.constants
    }
    #[must_use]
    pub fn weekly_modifiers(&self) -> &[DivergentUniverseWeeklyModifierDefinition] {
        &self.parts.weekly_modifiers
    }
    #[must_use]
    pub fn room_marks(&self) -> &[DivergentUniverseRoomMarkDefinition] {
        &self.parts.room_marks
    }
    #[must_use]
    pub fn effects(&self) -> &[DivergentUniverseProgressionEffectDefinition] {
        &self.parts.effects
    }
    #[must_use]
    pub const fn source_obligations(&self) -> usize {
        self.parts.source_obligations
    }
    #[cfg(test)]
    pub(crate) fn into_parts(self) -> DivergentUniverseProgressionCatalogParts {
        self.parts
    }
}
fn validate(
    p: &DivergentUniverseProgressionCatalogParts,
    flow: &DivergentUniverseFlowCatalog,
) -> Result<(), DivergentUniverseProgressionError> {
    let protocols = p
        .protocols
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let divisions = p
        .divisions
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let talents = p
        .talents
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    let effects = p
        .effects
        .iter()
        .map(|x| (&x.id, x))
        .collect::<BTreeMap<_, _>>();
    if protocols.len() != 8 || divisions.len() != 9 || talents.len() != 38 || effects.len() != 38 {
        return Err(error("duplicate Protocol/progression identity"));
    }
    if p.protocols.iter().enumerate().any(|(i, x)| {
        x.level != u16::try_from(i + 1).unwrap_or_default()
            || x.runtime_lowered
            || x.plane_scaling.attack.is_empty()
            || x.plane_scaling.max_hp.is_empty()
            || x.plane_scaling.speed.is_empty()
    }) {
        return Err(error("Threshold Protocol closure drift"));
    }
    if p.divisions.iter().enumerate().any(|(i, x)| {
        x.level != u16::try_from(i + 1).unwrap_or_default()
            || x.runtime_lowered
            || (i < 8 && (x.protocols.len() != 1 || !protocols.contains_key(&x.protocols[0])))
            || (i == 8 && (!x.protocols.is_empty() || x.progress_boundary.as_ref() != "Terminal"))
    }) {
        return Err(error("Astronomical Division closure drift"));
    }
    if p.cognoculi.iter().any(|x| {
        x.runtime_lowered
            || !divisions.contains_key(&x.division)
            || x.contribution_divisions.as_ref() != [x.division.clone()]
            || x.gain.as_ref() != "SuccessfulFinalizationLightsCognoculi"
            || x.loss.as_ref() != "UnsuccessfulFinalizationMayExtinguishCognoculi"
            || x.division_floor.as_ref() != "CurrentDivisionNeverDecreases"
    }) {
        return Err(error("Cognoculi lifecycle closure drift"));
    }
    if p.talents.iter().any(|x| {
        x.runtime_lowered
            || x.cost.len() != 1
            || x.cost[0].item_id.as_ref() != "281018"
            || !x.prerequisites.is_empty()
            || x.adjacent.is_empty()
            || x.effects.len() != 1
            || x.adjacent.iter().any(|id| !talents.contains_key(id))
            || x.effects.iter().any(|id| !effects.contains_key(id))
    }) {
        return Err(error("permanent talent closure drift"));
    }
    let areas = flow
        .areas()
        .iter()
        .map(|x| x.id.as_str())
        .collect::<BTreeSet<_>>();
    if p.unlocks.iter().any(|x| {
        x.runtime_lowered
            || x.unlocked_content_ids
                .iter()
                .any(|id| !areas.contains(id.as_ref()))
    }) {
        return Err(error("unlock area closure drift"));
    }
    if p.constants.iter().any(|x| {
        x.runtime_lowered
            || !matches!(x.value_kind.as_ref(), "Integer" | "String" | "Array")
            || matches!(
                (&x.canonical_value, x.value_kind.as_ref()),
                (
                    DivergentUniverseConstantValue::Array(_),
                    "Integer" | "String"
                ) | (DivergentUniverseConstantValue::Scalar(_), "Array")
            )
    }) {
        return Err(error("common constant boundary drift"));
    }
    if p.weekly_modifiers.iter().any(|x| {
        x.runtime_lowered
            || x.content_ids.is_empty()
            || x.detail_ids.is_empty()
            || x.enemy_groups.is_empty()
            || x.reachability.as_ref() != "UnprovenCurrentWeeklyCandidate"
            || x.enemy_groups
                .iter()
                .any(|g| g.resolution.as_ref() != "DisplayOnlyDeferredToP2B5")
    }) {
        return Err(error("weekly modifier boundary drift"));
    }
    if p.room_marks.iter().any(|x| {
        x.runtime_lowered
            || !x.transition_rules.is_empty()
            || x.fallback.as_ref() != "PreserveCurrentMark"
    }) {
        return Err(error("room mark boundary drift"));
    }
    if p.effects.iter().any(|x| {
        x.runtime_lowered
            || x.activation.as_ref() != "PermanentTalentUnlocked"
            || x.contributions.len() != 1
    }) {
        return Err(error("progression contribution closure drift"));
    }
    Ok(())
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseProgressionError {
    message: Box<str>,
}
impl std::fmt::Display for DivergentUniverseProgressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for DivergentUniverseProgressionError {}
fn error(message: &str) -> DivergentUniverseProgressionError {
    DivergentUniverseProgressionError {
        message: message.into(),
    }
}
