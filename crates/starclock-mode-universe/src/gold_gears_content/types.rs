#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct StableKey(Box<str>);

impl StableKey {
    pub(crate) fn new(value: &str) -> Self {
        Self(value.into())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub(super) struct JsonPayload(Box<str>);

impl JsonPayload {
    pub(super) fn new(value: &str) -> Self {
        Self(value.into())
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub(super) struct Blessing {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) path: StableKey,
    pub(super) levels: Box<[StableKey]>,
    pub(super) inherited_rules: Box<[StableKey]>,
}

#[derive(Debug)]
pub(super) struct BlessingLevel {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) blessing_id: i32,
    pub(super) inherited_rules: Box<[StableKey]>,
    pub(super) parameters: JsonPayload,
}

#[derive(Debug)]
pub(super) struct Curio {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) initial_state_id: i32,
    pub(super) states: Box<[StableKey]>,
    pub(super) rule: StableKey,
    pub(super) shared: bool,
}

#[derive(Debug)]
pub(super) struct CurioState {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) curio_id: i32,
    pub(super) rule: StableKey,
    pub(super) payloads: Box<[JsonPayload]>,
}

#[derive(Debug)]
pub(super) struct Occurrence {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) variants: Box<[StableKey]>,
    pub(super) rule: StableKey,
}

#[derive(Debug)]
pub(super) struct OccurrenceVariant {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) occurrence_id: i32,
    pub(super) occurrence_keys: Box<[StableKey]>,
    pub(super) choices: Box<[StableKey]>,
    pub(super) rule: StableKey,
}

#[derive(Debug)]
pub(super) struct OccurrenceChoice {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) variant_id: i32,
    pub(super) next_node: Option<StableKey>,
    pub(super) rule: StableKey,
    pub(super) payloads: Box<[JsonPayload]>,
}

#[derive(Debug)]
pub(super) struct Service {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) rule: StableKey,
    pub(super) shared: bool,
    pub(super) payloads: Box<[JsonPayload]>,
}

#[derive(Debug)]
pub(super) struct AdventureOutcome {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) downloader_service_id: i32,
    pub(super) room: StableKey,
    pub(super) rule: StableKey,
    pub(super) payloads: Box<[JsonPayload]>,
}

#[derive(Debug)]
pub(super) struct EncounterGroup {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) parent_room: Option<StableKey>,
    pub(super) areas: Box<[StableKey]>,
    pub(super) payloads: Box<[JsonPayload]>,
}

#[derive(Debug)]
pub(super) struct EncounterWave {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) group_id: i32,
    pub(super) slots: Box<[StableKey]>,
    pub(super) payload: JsonPayload,
}

#[derive(Debug)]
pub(super) struct EnemySlot {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) wave_id: i32,
    pub(super) enemy: StableKey,
    pub(super) boss_choices: Box<[StableKey]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MapEventTrigger {
    EnterCell,
    EnterRow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MapEventEffect {
    AddActionPoint,
    GrantCurio,
    GenerateMark,
    RandomReplace,
    Replace,
    Shuffle,
}

#[derive(Debug)]
pub(crate) struct MapEvent {
    pub(crate) id: i32,
    pub(crate) key: StableKey,
    pub(crate) chessboard_id: i32,
    pub(crate) trigger: MapEventTrigger,
    pub(crate) trigger_parameters: Box<[u32]>,
    pub(crate) effect: MapEventEffect,
    pub(crate) effect_parameters: Box<[u32]>,
    pub(crate) secondary_effect_parameters: Box<[u32]>,
    pub(crate) weight: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CreateCountWeight {
    pub(crate) count: u16,
    pub(crate) weight: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BeaconWeight {
    pub(crate) beacon: Option<StableKey>,
    pub(crate) weight: u64,
}

#[derive(Debug)]
pub(crate) struct BlockCreateRule {
    pub(crate) id: i32,
    pub(crate) key: StableKey,
    pub(crate) chessboard_id: i32,
    pub(crate) group_id: Box<str>,
    pub(crate) order: u16,
    pub(crate) domain_id: i32,
    pub(crate) create_counts: Box<[CreateCountWeight]>,
    pub(crate) beacons: Box<[BeaconWeight]>,
}

#[derive(Debug)]
pub(super) struct MechanicRule {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) owner: StableKey,
    pub(super) fixtures: Box<[StableKey]>,
    pub(super) disposition: Box<str>,
    pub(super) policy_bound: bool,
    pub(super) payloads: Box<[JsonPayload]>,
}

#[derive(Debug)]
pub(super) struct CatalogCoverage {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) category: Box<str>,
    pub(super) required: i32,
    pub(super) accounted: i32,
    pub(super) data_ready: i32,
    pub(super) blocking_gaps: Box<[StableKey]>,
}

#[derive(Debug)]
pub(super) struct StableIndexRow {
    pub(super) id: i32,
    pub(super) key: StableKey,
}
