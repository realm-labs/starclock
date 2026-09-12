//! Immutable data definitions for Divergent Universe entry and topology.
//!
//! These types describe validated content only. They do not implement an
//! Activity state machine or promote policy-bound room candidates to offers.

use std::collections::{BTreeMap, BTreeSet};

macro_rules! stable_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(Box<str>);

        impl $name {
            /// Constructs a typed stable ID with its required namespace.
            pub fn new(value: impl Into<Box<str>>) -> Result<Self, DivergentUniverseCatalogError> {
                let value = value.into();
                if !value.starts_with($prefix) || value.len() == $prefix.len() {
                    return Err(error(concat!(stringify!($name), " namespace mismatch")));
                }
                Ok(Self(value))
            }

            /// Returns the canonical stable text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

stable_id!(DivergentUniverseProfileId, "divergent-universe.profile.");
stable_id!(DivergentUniverseModuleId, "divergent-universe.module.");
stable_id!(DivergentUniverseEntryId, "divergent-universe.entry.");
stable_id!(
    DivergentUniverseFinishConditionId,
    "divergent-universe.finish."
);
stable_id!(DivergentUniverseAreaId, "divergent-universe.area.");
stable_id!(
    DivergentUniverseDifficultyId,
    "divergent-universe.difficulty."
);
stable_id!(DivergentUniverseLayerId, "divergent-universe.layer.");
stable_id!(
    DivergentUniverseRoomCandidateId,
    "divergent-universe.room-candidate."
);
stable_id!(DivergentUniverseFlowId, "divergent-universe.flow.");
stable_id!(
    DivergentUniverseCyclicalChallengeId,
    "divergent-universe.cyclical-area."
);

/// Authored run family selected by the entry/area boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseRunFamily {
    Ordinary,
    Cyclical,
}

/// Released area classification.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseAreaKind {
    Guide,
    Formal,
    WeekChallenge,
}

impl DivergentUniverseAreaKind {
    /// Returns the only legal run family for this area kind.
    #[must_use]
    pub const fn run_family(self) -> DivergentUniverseRunFamily {
        match self {
            Self::Guide | Self::Formal => DivergentUniverseRunFamily::Ordinary,
            Self::WeekChallenge => DivergentUniverseRunFamily::Cyclical,
        }
    }
}

/// Released entry row role.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseEntryKind {
    ModeTitle,
    ResidentActivity,
}

/// Explicit status of layer-to-room evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseRoomPositionResolution {
    NoMatchingReleasedRow,
}

/// Policy-bound reachability of a retained shared room row.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseRoomReachability {
    UnprovenSharedCandidate,
}

/// Typed flow condition retained for later Activity lowering.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseFlowCondition {
    AcceptedEntry,
    CurrentLayerCompleted,
    NextLayerSelected,
    NextRoomSelected,
    RunFinalized,
}

/// Typed flow operation retained in authored order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseFlowOperation {
    InitializeArea,
    EnterLayer,
    ExitLayer,
    EvaluateFinish,
    FinalizeArea,
    CarryRunState,
    CarryRunInventory,
    CarryEquationProgress,
    CarryTemporaryBuilds,
    ClearRunState,
    RemoveTemporaryBuilds,
    PreservePermanentUnlocks,
}

/// Profile identity and its exact entry/finish closure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseProfileDefinition {
    pub id: DivergentUniverseProfileId,
    pub module: DivergentUniverseModuleId,
    pub entries: Box<[DivergentUniverseEntryId]>,
    pub finish_conditions: Box<[DivergentUniverseFinishConditionId]>,
    pub game_version: Box<str>,
    pub reference_runtime_enabled: bool,
}

/// Released tournament module boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseModuleDefinition {
    pub id: DivergentUniverseModuleId,
    pub source_id: Box<str>,
    pub main_tournament: u16,
    pub sub_tournament: u16,
}

/// Released mode-title or resident-activity entry identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEntryDefinition {
    pub id: DivergentUniverseEntryId,
    pub kind: DivergentUniverseEntryKind,
    pub source_id: Box<str>,
    pub module: DivergentUniverseModuleId,
    pub related_panel_id: Option<Box<str>>,
}

/// Exact source finish-condition expression retained for later execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseFinishConditionDefinition {
    pub id: DivergentUniverseFinishConditionId,
    pub source_id: Box<str>,
    pub kind: Box<str>,
    pub parameter_type: Box<str>,
    pub string_parameter: Box<str>,
    pub integer_parameters: Box<[Box<str>]>,
    pub item_parameters: Box<[Box<str>]>,
    pub progress: Box<str>,
    pub source_only: bool,
}

/// Released difficulty row. Protocol/scaling gaps remain explicit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseDifficultyDefinition {
    pub id: DivergentUniverseDifficultyId,
    pub source_id: Box<str>,
    pub levels: Box<[u16]>,
    pub protocol_id: Option<Box<str>>,
    pub enemy_scaling_refs: Box<[Box<str>]>,
    pub unresolved_fields: Box<[Box<str>]>,
}

/// Released layer identity and its explicit missing room-position boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseLayerDefinition {
    pub id: DivergentUniverseLayerId,
    pub source_id: Box<str>,
    pub number: u16,
    pub ordered_room_positions: Box<[Box<str>]>,
    pub room_position_resolution: DivergentUniverseRoomPositionResolution,
}

/// Released area with exact legal difficulty and ordered-layer joins.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseAreaDefinition {
    pub id: DivergentUniverseAreaId,
    pub source_id: Box<str>,
    pub kind: DivergentUniverseAreaKind,
    pub group: Option<Box<str>>,
    pub difficulties: Box<[DivergentUniverseDifficultyId]>,
    pub layers: Box<[DivergentUniverseLayerId]>,
    pub map_entry_id: Box<str>,
    pub initial_room_type: Box<str>,
    pub unlock_finish_source_id: Box<str>,
}

impl DivergentUniverseAreaDefinition {
    /// Returns the run family implied by the exact released area kind.
    #[must_use]
    pub const fn run_family(&self) -> DivergentUniverseRunFamily {
        self.kind.run_family()
    }
}

/// Shared Tourn2 room retained only as a non-offered candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseRoomCandidateDefinition {
    pub id: DivergentUniverseRoomCandidateId,
    pub source_id: Box<str>,
    pub room_type: Box<str>,
    pub reachability: DivergentUniverseRoomReachability,
    pub stage_refs: Box<[Box<str>]>,
    pub offered_pool_ids: Box<[Box<str>]>,
    pub replacement_condition: Box<str>,
}

/// One area transition or global carry/reset record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseFlowDefinition {
    pub id: DivergentUniverseFlowId,
    pub area: Option<DivergentUniverseAreaId>,
    pub from_state: Box<str>,
    pub condition: DivergentUniverseFlowCondition,
    pub to_state: Box<str>,
    pub operations: Box<[DivergentUniverseFlowOperation]>,
    pub policy_id: Box<str>,
}

/// Released Cyclical Extrapolation area binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCyclicalChallengeDefinition {
    pub id: DivergentUniverseCyclicalChallengeId,
    pub source_id: Box<str>,
    pub area: DivergentUniverseAreaId,
    pub modifier_ids: Box<[Box<str>]>,
    pub modifier_resolution: Box<str>,
    pub enemy_display_refs: Box<[Box<str>]>,
}

/// Complete immutable entry/topology catalog parts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseFlowCatalogParts {
    pub profile: DivergentUniverseProfileDefinition,
    pub module: DivergentUniverseModuleDefinition,
    pub entries: Vec<DivergentUniverseEntryDefinition>,
    pub finish_conditions: Vec<DivergentUniverseFinishConditionDefinition>,
    pub areas: Vec<DivergentUniverseAreaDefinition>,
    pub difficulties: Vec<DivergentUniverseDifficultyDefinition>,
    pub layers: Vec<DivergentUniverseLayerDefinition>,
    pub room_candidates: Vec<DivergentUniverseRoomCandidateDefinition>,
    pub flows: Vec<DivergentUniverseFlowDefinition>,
    pub cyclical_challenges: Vec<DivergentUniverseCyclicalChallengeDefinition>,
}

/// Validated immutable catalog for entry, area, difficulty, layer and flow data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseFlowCatalog {
    profile: DivergentUniverseProfileDefinition,
    module: DivergentUniverseModuleDefinition,
    entries: Box<[DivergentUniverseEntryDefinition]>,
    finish_conditions: Box<[DivergentUniverseFinishConditionDefinition]>,
    areas: Box<[DivergentUniverseAreaDefinition]>,
    difficulties: Box<[DivergentUniverseDifficultyDefinition]>,
    layers: Box<[DivergentUniverseLayerDefinition]>,
    room_candidates: Box<[DivergentUniverseRoomCandidateDefinition]>,
    flows: Box<[DivergentUniverseFlowDefinition]>,
    cyclical_challenges: Box<[DivergentUniverseCyclicalChallengeDefinition]>,
}

impl DivergentUniverseFlowCatalog {
    /// Validates exact denominators, unique identities and every current join.
    pub fn new(
        mut parts: DivergentUniverseFlowCatalogParts,
    ) -> Result<Self, DivergentUniverseCatalogError> {
        require_count(&parts.entries, 2, "entries")?;
        require_count(&parts.finish_conditions, 13, "finish conditions")?;
        require_count(&parts.areas, 28, "areas")?;
        require_count(&parts.difficulties, 22, "difficulties")?;
        require_count(&parts.layers, 11, "layers")?;
        require_count(&parts.room_candidates, 848, "room candidates")?;
        require_count(&parts.flows, 111, "flow records")?;
        require_count(&parts.cyclical_challenges, 13, "cyclical challenges")?;
        parts
            .entries
            .sort_unstable_by(|left, right| left.id.cmp(&right.id));
        parts
            .finish_conditions
            .sort_unstable_by(|left, right| left.id.cmp(&right.id));
        parts
            .areas
            .sort_unstable_by(|left, right| left.id.cmp(&right.id));
        parts
            .difficulties
            .sort_unstable_by(|left, right| left.id.cmp(&right.id));
        parts
            .layers
            .sort_unstable_by(|left, right| left.id.cmp(&right.id));
        parts
            .room_candidates
            .sort_unstable_by(|left, right| left.id.cmp(&right.id));
        parts
            .flows
            .sort_unstable_by(|left, right| left.id.cmp(&right.id));
        parts
            .cyclical_challenges
            .sort_unstable_by(|left, right| left.id.cmp(&right.id));
        ensure_unique(&parts.entries, |value| &value.id, "entry")?;
        ensure_unique(
            &parts.finish_conditions,
            |value| &value.id,
            "finish condition",
        )?;
        ensure_unique(&parts.areas, |value| &value.id, "area")?;
        ensure_unique(&parts.difficulties, |value| &value.id, "difficulty")?;
        ensure_unique(&parts.layers, |value| &value.id, "layer")?;
        ensure_unique(&parts.room_candidates, |value| &value.id, "room candidate")?;
        ensure_unique(&parts.flows, |value| &value.id, "flow")?;
        ensure_unique(
            &parts.cyclical_challenges,
            |value| &value.id,
            "cyclical challenge",
        )?;
        validate_joins(&parts)?;
        Ok(Self {
            profile: parts.profile,
            module: parts.module,
            entries: parts.entries.into_boxed_slice(),
            finish_conditions: parts.finish_conditions.into_boxed_slice(),
            areas: parts.areas.into_boxed_slice(),
            difficulties: parts.difficulties.into_boxed_slice(),
            layers: parts.layers.into_boxed_slice(),
            room_candidates: parts.room_candidates.into_boxed_slice(),
            flows: parts.flows.into_boxed_slice(),
            cyclical_challenges: parts.cyclical_challenges.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn profile(&self) -> &DivergentUniverseProfileDefinition {
        &self.profile
    }
    #[must_use]
    pub const fn module(&self) -> &DivergentUniverseModuleDefinition {
        &self.module
    }
    #[must_use]
    pub const fn entries(&self) -> &[DivergentUniverseEntryDefinition] {
        &self.entries
    }
    #[must_use]
    pub const fn finish_conditions(&self) -> &[DivergentUniverseFinishConditionDefinition] {
        &self.finish_conditions
    }
    #[must_use]
    pub const fn areas(&self) -> &[DivergentUniverseAreaDefinition] {
        &self.areas
    }
    #[must_use]
    pub const fn difficulties(&self) -> &[DivergentUniverseDifficultyDefinition] {
        &self.difficulties
    }
    #[must_use]
    pub const fn layers(&self) -> &[DivergentUniverseLayerDefinition] {
        &self.layers
    }
    /// Returns retained candidates, never an offered room pool.
    #[must_use]
    pub const fn room_candidates(&self) -> &[DivergentUniverseRoomCandidateDefinition] {
        &self.room_candidates
    }
    #[must_use]
    pub const fn flows(&self) -> &[DivergentUniverseFlowDefinition] {
        &self.flows
    }
    #[must_use]
    pub const fn cyclical_challenges(&self) -> &[DivergentUniverseCyclicalChallengeDefinition] {
        &self.cyclical_challenges
    }

    #[must_use]
    pub fn area(&self, id: &DivergentUniverseAreaId) -> Option<&DivergentUniverseAreaDefinition> {
        self.areas
            .binary_search_by(|value| value.id.cmp(id))
            .ok()
            .map(|index| &self.areas[index])
    }

    #[cfg(test)]
    pub(crate) fn into_parts(self) -> DivergentUniverseFlowCatalogParts {
        DivergentUniverseFlowCatalogParts {
            profile: self.profile,
            module: self.module,
            entries: self.entries.into_vec(),
            finish_conditions: self.finish_conditions.into_vec(),
            areas: self.areas.into_vec(),
            difficulties: self.difficulties.into_vec(),
            layers: self.layers.into_vec(),
            room_candidates: self.room_candidates.into_vec(),
            flows: self.flows.into_vec(),
            cyclical_challenges: self.cyclical_challenges.into_vec(),
        }
    }
}

fn validate_joins(
    parts: &DivergentUniverseFlowCatalogParts,
) -> Result<(), DivergentUniverseCatalogError> {
    if parts.profile.module != parts.module.id
        || parts.profile.game_version.as_ref() != "4.4"
        || parts.profile.reference_runtime_enabled
    {
        return Err(error("profile/module or reference runtime boundary drift"));
    }
    let entries = parts
        .entries
        .iter()
        .map(|value| &value.id)
        .collect::<BTreeSet<_>>();
    let finishes = parts
        .finish_conditions
        .iter()
        .map(|value| &value.id)
        .collect::<BTreeSet<_>>();
    if parts.profile.entries.iter().collect::<BTreeSet<_>>() != entries
        || parts
            .profile
            .finish_conditions
            .iter()
            .collect::<BTreeSet<_>>()
            != finishes
        || parts
            .entries
            .iter()
            .any(|entry| entry.module != parts.module.id)
    {
        return Err(error("profile entry/finish closure drift"));
    }
    let difficulties = parts
        .difficulties
        .iter()
        .map(|value| &value.id)
        .collect::<BTreeSet<_>>();
    let layers = parts
        .layers
        .iter()
        .map(|value| &value.id)
        .collect::<BTreeSet<_>>();
    let areas = parts
        .areas
        .iter()
        .map(|value| (&value.id, value))
        .collect::<BTreeMap<_, _>>();
    for area in &parts.areas {
        if area.difficulties.is_empty()
            || area.layers.is_empty()
            || area
                .difficulties
                .iter()
                .any(|id| !difficulties.contains(id))
            || area.layers.iter().any(|id| !layers.contains(id))
        {
            return Err(error("area difficulty/layer closure drift"));
        }
    }
    for challenge in &parts.cyclical_challenges {
        let area = areas
            .get(&challenge.area)
            .ok_or_else(|| error("unknown cyclical area"))?;
        if area.kind != DivergentUniverseAreaKind::WeekChallenge {
            return Err(error("cyclical challenge references a non-weekly area"));
        }
    }
    let cyclical_areas = parts
        .cyclical_challenges
        .iter()
        .map(|value| &value.area)
        .collect::<BTreeSet<_>>();
    let weekly_areas = parts
        .areas
        .iter()
        .filter(|value| value.kind == DivergentUniverseAreaKind::WeekChallenge)
        .map(|value| &value.id)
        .collect::<BTreeSet<_>>();
    if cyclical_areas != weekly_areas {
        return Err(error("cyclical challenge area closure drift"));
    }
    for room in &parts.room_candidates {
        if room.reachability != DivergentUniverseRoomReachability::UnprovenSharedCandidate
            || !room.stage_refs.is_empty()
            || !room.offered_pool_ids.is_empty()
        {
            return Err(error(
                "room candidate was promoted without selector evidence",
            ));
        }
    }
    if parts.flows.iter().any(|flow| {
        flow.area
            .as_ref()
            .is_some_and(|area| !areas.contains_key(area))
            || flow.operations.is_empty()
            || flow.policy_id.as_ref() != "ordered-tourn3-area-layer-flow-v1"
    }) || !valid_area_flows(parts)
        || !valid_global_flows(parts)
    {
        return Err(error("flow area/carry/reset closure drift"));
    }
    Ok(())
}

fn valid_area_flows(parts: &DivergentUniverseFlowCatalogParts) -> bool {
    for area in &parts.areas {
        let flows = parts
            .flows
            .iter()
            .filter(|flow| flow.area.as_ref() == Some(&area.id))
            .collect::<Vec<_>>();
        let Some(first_layer) = area.layers.first() else {
            return false;
        };
        let Some(last_layer) = area.layers.last() else {
            return false;
        };
        if flows.len() != area.layers.len() + 1
            || !has_flow(
                &flows,
                "AreaEntry",
                DivergentUniverseFlowCondition::AcceptedEntry,
                first_layer.as_str(),
                &[
                    DivergentUniverseFlowOperation::InitializeArea,
                    DivergentUniverseFlowOperation::EnterLayer,
                ],
            )
            || !has_flow(
                &flows,
                last_layer.as_str(),
                DivergentUniverseFlowCondition::CurrentLayerCompleted,
                "AreaTerminal",
                &[
                    DivergentUniverseFlowOperation::ExitLayer,
                    DivergentUniverseFlowOperation::EvaluateFinish,
                    DivergentUniverseFlowOperation::FinalizeArea,
                ],
            )
        {
            return false;
        }
        for layers in area.layers.windows(2) {
            if !has_flow(
                &flows,
                layers[0].as_str(),
                DivergentUniverseFlowCondition::CurrentLayerCompleted,
                layers[1].as_str(),
                &[
                    DivergentUniverseFlowOperation::CarryRunState,
                    DivergentUniverseFlowOperation::ExitLayer,
                    DivergentUniverseFlowOperation::EnterLayer,
                ],
            ) {
                return false;
            }
        }
    }
    true
}

fn valid_global_flows(parts: &DivergentUniverseFlowCatalogParts) -> bool {
    let flows = parts
        .flows
        .iter()
        .filter(|flow| flow.area.is_none())
        .collect::<Vec<_>>();
    let carry = [
        DivergentUniverseFlowOperation::CarryRunInventory,
        DivergentUniverseFlowOperation::CarryEquationProgress,
        DivergentUniverseFlowOperation::CarryTemporaryBuilds,
    ];
    flows.len() == 3
        && has_flow(
            &flows,
            "RoomTerminal",
            DivergentUniverseFlowCondition::NextRoomSelected,
            "RoomEntry",
            &carry,
        )
        && has_flow(
            &flows,
            "LayerTerminal",
            DivergentUniverseFlowCondition::NextLayerSelected,
            "LayerEntry",
            &carry,
        )
        && has_flow(
            &flows,
            "AreaTerminal",
            DivergentUniverseFlowCondition::RunFinalized,
            "ProfileReady",
            &[
                DivergentUniverseFlowOperation::ClearRunState,
                DivergentUniverseFlowOperation::RemoveTemporaryBuilds,
                DivergentUniverseFlowOperation::PreservePermanentUnlocks,
            ],
        )
}

fn has_flow(
    flows: &[&DivergentUniverseFlowDefinition],
    from: &str,
    condition: DivergentUniverseFlowCondition,
    to: &str,
    operations: &[DivergentUniverseFlowOperation],
) -> bool {
    flows.iter().any(|flow| {
        flow.from_state.as_ref() == from
            && flow.condition == condition
            && flow.to_state.as_ref() == to
            && flow.operations.as_ref() == operations
    })
}

fn require_count<T>(
    values: &[T],
    expected: usize,
    label: &str,
) -> Result<(), DivergentUniverseCatalogError> {
    if values.len() != expected {
        return Err(error(&format!(
            "expected {expected} {label}, got {}",
            values.len()
        )));
    }
    Ok(())
}

fn ensure_unique<T, K: Ord>(
    values: &[T],
    key: impl Fn(&T) -> &K,
    label: &str,
) -> Result<(), DivergentUniverseCatalogError> {
    if values.windows(2).any(|pair| key(&pair[0]) == key(&pair[1])) {
        return Err(error(&format!("duplicate {label} identity")));
    }
    Ok(())
}

/// Typed invalid-definition error; construction never returns a partial catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCatalogError {
    message: Box<str>,
}

impl std::fmt::Display for DivergentUniverseCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for DivergentUniverseCatalogError {}

fn error(message: &str) -> DivergentUniverseCatalogError {
    DivergentUniverseCatalogError {
        message: message.into(),
    }
}
