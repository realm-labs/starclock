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
pub(crate) struct JsonPayload(Box<str>);

impl JsonPayload {
    pub(super) fn new(value: &str) -> Self {
        Self(value.into())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub(crate) struct Blessing {
    pub(crate) id: i32,
    pub(crate) key: StableKey,
    pub(crate) path: StableKey,
    pub(crate) levels: Box<[StableKey]>,
    pub(crate) inherited_rules: Box<[StableKey]>,
}

#[derive(Debug)]
pub(crate) struct BlessingLevel {
    pub(crate) id: i32,
    pub(crate) key: StableKey,
    pub(crate) blessing_id: i32,
    pub(crate) inherited_rules: Box<[StableKey]>,
    pub(crate) parameters: JsonPayload,
}

#[derive(Debug)]
pub(crate) struct Curio {
    pub(crate) id: i32,
    pub(crate) key: StableKey,
    pub(crate) source_id: Box<str>,
    pub(crate) mode_copy_id: Box<str>,
    pub(crate) handbook_order: i32,
    pub(crate) pool_category: Box<str>,
    pub(crate) selection_pool: StableKey,
    pub(crate) random_offer_eligibility: Box<str>,
    pub(crate) initial_state_id: i32,
    pub(crate) states: Box<[StableKey]>,
    pub(crate) rule: StableKey,
    pub(crate) shared: bool,
}

#[derive(Debug)]
pub(crate) struct CurioState {
    pub(crate) id: i32,
    pub(crate) key: StableKey,
    pub(crate) curio_id: i32,
    pub(crate) state_kind: Box<str>,
    pub(crate) pool_category: Box<str>,
    pub(crate) lifecycle: JsonPayload,
    pub(crate) parameters: JsonPayload,
    pub(crate) repair_target: JsonPayload,
    pub(crate) source_effect_id: Box<str>,
    pub(crate) selection_policy: JsonPayload,
    pub(crate) rule: StableKey,
    pub(crate) payloads: Box<[JsonPayload]>,
}

#[derive(Debug)]
pub(crate) struct Occurrence {
    pub(crate) id: i32,
    pub(crate) key: StableKey,
    pub(crate) variants: Box<[StableKey]>,
    pub(crate) rule: StableKey,
}

#[derive(Debug)]
pub(crate) struct OccurrenceVariant {
    pub(crate) id: i32,
    pub(crate) key: StableKey,
    pub(crate) occurrence_id: i32,
    pub(crate) occurrence_keys: Box<[StableKey]>,
    pub(crate) entry_node: StableKey,
    pub(crate) conditions: Box<[Box<str>]>,
    pub(crate) choices: Box<[StableKey]>,
    pub(crate) rule: StableKey,
}

#[derive(Debug)]
pub(crate) struct OccurrenceChoice {
    pub(crate) id: i32,
    pub(crate) key: StableKey,
    pub(crate) source_id: Box<str>,
    pub(crate) variant_id: i32,
    pub(crate) node_index: i32,
    pub(crate) choice_index: i32,
    pub(crate) option_index: i32,
    pub(crate) conditions: Box<[Box<str>]>,
    pub(crate) next_node: Option<StableKey>,
    pub(crate) rule: StableKey,
    pub(crate) payloads: Box<[JsonPayload]>,
}

#[derive(Debug)]
pub(crate) struct Service {
    pub(crate) id: i32,
    pub(crate) key: StableKey,
    pub(crate) kind: Box<str>,
    pub(crate) currency: Option<StableKey>,
    pub(crate) price_formula: Option<StableKey>,
    pub(crate) rule: StableKey,
    pub(crate) shared: bool,
    pub(crate) payloads: Box<[JsonPayload]>,
}

#[derive(Debug)]
pub(crate) struct AdventureOutcome {
    pub(crate) id: i32,
    pub(crate) key: StableKey,
    pub(crate) source_id: Box<str>,
    pub(crate) adventure_type: Box<str>,
    pub(crate) objective_metric: Box<str>,
    pub(crate) objective_thresholds: Box<[Box<str>]>,
    pub(crate) maximum_value: Box<str>,
    pub(crate) time_limit_seconds: Option<Box<str>>,
    pub(crate) technique_rule: Box<str>,
    pub(crate) rewards_are_cumulative: bool,
    pub(crate) downloader_service_id: i32,
    pub(crate) room: StableKey,
    pub(crate) rule: StableKey,
    pub(crate) payloads: Box<[JsonPayload]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EncounterRole {
    GuideBoss,
    CombatPool,
    ElitePool,
    FirstPlaneBossAlternative,
    SecondPlaneBossAlternative,
    FinalBoss,
}

#[derive(Debug)]
pub(crate) struct EncounterMember {
    pub(crate) source_rogue_monster_id: Box<str>,
    pub(crate) source_primary_monster_id: Box<str>,
    pub(crate) source_stage_id: Box<str>,
    pub(crate) weight: u64,
    pub(crate) waves: Box<[StableKey]>,
}

#[derive(Debug)]
pub(crate) struct EncounterGroup {
    pub(crate) id: i32,
    pub(crate) key: StableKey,
    pub(crate) source_group_id: Box<str>,
    pub(crate) source_namespace: Box<str>,
    pub(crate) role: EncounterRole,
    pub(crate) parent_room: Option<StableKey>,
    pub(crate) areas: Box<[StableKey]>,
    pub(crate) members: Box<[EncounterMember]>,
    pub(crate) payloads: Box<[JsonPayload]>,
}

#[derive(Debug)]
pub(crate) struct EncounterWave {
    pub(crate) id: i32,
    pub(crate) key: StableKey,
    pub(crate) group_id: i32,
    pub(crate) source_rogue_monster_id: Box<str>,
    pub(crate) source_stage_id: Box<str>,
    pub(crate) wave_index: u16,
    pub(crate) slots: Box<[StableKey]>,
    pub(crate) stage_type: Box<str>,
    pub(crate) authored_stage_level: u16,
    pub(crate) hard_level_group: u16,
    pub(crate) stage_ability_ids: Box<[Box<str>]>,
    pub(crate) payload: JsonPayload,
}

#[derive(Debug)]
pub(crate) struct EnemySlot {
    pub(crate) id: i32,
    pub(crate) key: StableKey,
    pub(crate) wave_id: i32,
    pub(crate) slot_index: u16,
    pub(crate) source_slot: Box<str>,
    pub(crate) source_monster_id: Box<str>,
    pub(crate) enemy: StableKey,
    pub(crate) boss_choices: Box<[StableKey]>,
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
