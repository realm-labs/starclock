#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct StableKey(Box<str>);

impl StableKey {
    pub(super) fn new(value: &str) -> Self {
        Self(value.into())
    }

    pub(super) fn as_str(&self) -> &str {
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

#[derive(Debug)]
pub(super) struct MapEvent {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) chessboard_id: i32,
    pub(super) parameters: Box<[Box<str>]>,
}

#[derive(Debug)]
pub(super) struct BlockCreateRule {
    pub(super) id: i32,
    pub(super) key: StableKey,
    pub(super) chessboard_id: i32,
    pub(super) domain_id: i32,
    pub(super) payloads: Box<[JsonPayload]>,
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
