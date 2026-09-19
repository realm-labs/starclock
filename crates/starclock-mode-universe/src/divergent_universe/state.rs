//! Activity slot allocation and initial-state compilation for Divergent Universe.

use starclock_activity::{
    ActivityScope, ActivitySlotDefinition, ActivitySlotId, ActivityStateDefinition,
    ActivityStateSource, ActivityStateVisibility, ActivityValue, LogicalScopeDefinitions,
    SlotCarryPolicy, SlotResetPoint,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;

use super::{entry_flow::DivergentUniverseEntryFlowError, progression::CompiledProgression};

pub(super) const PROFILE_SLOT: ActivitySlotId = slot(1);
pub(super) const ENTRY_SLOT: ActivitySlotId = slot(2);
pub(super) const MODULE_SLOT: ActivitySlotId = slot(3);
pub(super) const RUN_FAMILY_SLOT: ActivitySlotId = slot(4);
pub(super) const DIFFICULTY_SLOT: ActivitySlotId = slot(5);
pub(super) const AREA_SLOT: ActivitySlotId = slot(6);
pub(super) const LAYER_SLOT: ActivitySlotId = slot(7);
pub(super) const LAYER_SEQUENCE_SLOT: ActivitySlotId = slot(8);
pub(super) const ROOM_SLOT: ActivitySlotId = slot(9);
pub(super) const CURRENCIES_SLOT: ActivitySlotId = slot(10);
pub(super) const RUN_FLAGS_SLOT: ActivitySlotId = slot(11);
pub(super) const PLANE_FLAGS_SLOT: ActivitySlotId = slot(12);
pub(super) const PERMANENT_UNLOCKS_SLOT: ActivitySlotId = slot(13);
pub(super) const DIFFICULTY_LEVELS_SLOT: ActivitySlotId = slot(14);
pub(super) const ASTRONOMICAL_MODE_SLOT: ActivitySlotId = slot(15);
pub(super) const DIVISION_SLOT: ActivitySlotId = slot(16);
pub(super) const PROTOCOL_SLOT: ActivitySlotId = slot(17);
pub(super) const COGNOCULI_SLOT: ActivitySlotId = slot(18);
pub(super) const CYCLICAL_EPOCH_SLOT: ActivitySlotId = slot(19);
pub(super) const ACCOUNT_LOADOUT_SNAPSHOT_SLOT: ActivitySlotId = slot(20);
pub(super) const PARTY_SNAPSHOT_SLOT: ActivitySlotId = slot(21);
pub(super) const MAPPING_STATE_SLOT: ActivitySlotId = slot(22);
pub(super) const EQUATIONS_SLOT: ActivitySlotId = slot(23);
pub(super) const EQUATION_PROGRESS_SLOT: ActivitySlotId = slot(24);
pub(super) const BLESSINGS_SLOT: ActivitySlotId = slot(25);
pub(super) const TITAN_BOONS_SLOT: ActivitySlotId = slot(26);
pub(super) const SERVICE_RECEIPTS_SLOT: ActivitySlotId = slot(27);
pub(super) const EQUATION_OFFERS_SLOT: ActivitySlotId = slot(28);
pub(super) const EQUATION_OFFER_SOURCE_SLOT: ActivitySlotId = slot(29);
pub(super) const EQUATION_REROLL_COUNT_SLOT: ActivitySlotId = slot(30);
pub(super) const EXPANDED_EQUATIONS_SLOT: ActivitySlotId = slot(31);
pub(super) const EQUATION_BLESSING_SNAPSHOT_SLOT: ActivitySlotId = slot(32);
pub(super) const EQUATION_PROGRESS_DIRTY_SLOT: ActivitySlotId = slot(33);
pub(super) const BLESSING_OFFERS_SLOT: ActivitySlotId = slot(34);
pub(super) const BLESSING_OFFER_SOURCE_SLOT: ActivitySlotId = slot(35);
pub(super) const CURIO_STATES_SLOT: ActivitySlotId = slot(36);
pub(super) const CURIO_CHARGES_SLOT: ActivitySlotId = slot(37);
pub(super) const CURIO_ACTIVATIONS_SLOT: ActivitySlotId = slot(38);
pub(super) const GRAND_MIRACLES_SLOT: ActivitySlotId = slot(39);
pub(super) const TITAN_TYPE_SLOT: ActivitySlotId = slot(40);
pub(super) const TITAN_TALENTS_SLOT: ActivitySlotId = slot(41);
pub(super) const TITAN_TALENT_CURRENCY_SLOT: ActivitySlotId = slot(42);
pub(super) const PERMANENT_TALENT_CURRENCY_SLOT: ActivitySlotId = slot(43);
pub(super) const WEEKLY_MODIFIER_SLOT: ActivitySlotId = slot(44);
pub(super) const ROOM_MARK_SLOT: ActivitySlotId = slot(45);
pub(super) const WORKBENCH_SLOT: ActivitySlotId = slot(46);
pub(super) const ACTIVITY_MECHANIC_LIFECYCLE_SLOT: ActivitySlotId = slot(47);
pub(super) const ROOM_DIALOGUE_PROGRAM_SLOT: ActivitySlotId = slot(48);
pub(super) const ROOM_DIALOGUE_FINISHED_SLOT: ActivitySlotId = slot(49);
pub(super) const ROOM_PREDICATE_SATISFIED_SLOT: ActivitySlotId = slot(50);
pub(super) const ROOM_CONTENT_UPDATED_SLOT: ActivitySlotId = slot(51);
pub(super) const ROOM_FINISHED_SLOT: ActivitySlotId = slot(52);
pub(super) const ROOM_DOORS_OPEN_SLOT: ActivitySlotId = slot(53);
pub(super) const ROOM_CONTENT_ENABLED_SLOT: ActivitySlotId = slot(54);
pub(super) const OCCURRENCE_REWARD_ACCEPTED_SLOT: ActivitySlotId = slot(55);
pub(super) const FRAGMENT_GAIN_BASE_SLOT: ActivitySlotId = slot(56);
pub(super) const BATTLE_BLESSING_CANDIDATES_SLOT: ActivitySlotId = slot(57);
pub(super) const BATTLE_BLESSING_ACCEPTED_SLOT: ActivitySlotId = slot(58);
pub(super) const EQUATION_GRANT_DOMAIN_VISITS_SLOT: ActivitySlotId = slot(59);
pub(super) const INITIAL_EQUATION_ACCEPTED_SLOT: ActivitySlotId = slot(60);
pub(super) const BATTLE_DOMAIN_SLOT: ActivitySlotId = slot(61);
pub(super) const TAWOT_PURCHASES_SLOT: ActivitySlotId = slot(62);
pub(super) const TAWOT_OFFER_SLOT: ActivitySlotId = slot(63);
pub(super) const TAWOT_ACCEPTED_SLOT: ActivitySlotId = slot(64);
pub(super) const TAWOT_OPENS_SLOT: ActivitySlotId = slot(65);

pub(super) struct EntryStateValues<'a> {
    pub(super) entry: u64,
    pub(super) module: u64,
    pub(super) area: u64,
    pub(super) difficulty: u64,
    pub(super) first_layer: u64,
    pub(super) permanent_unlocks: &'a [u64],
    pub(super) account_loadout_snapshot: u64,
    pub(super) party_snapshot: u64,
    pub(super) mapping_state: Box<[(u64, i64)]>,
}

pub(super) fn compile_state(
    family: DivergentUniverseRunFamily,
    values: EntryStateValues<'_>,
    progression: &CompiledProgression,
    logical_scopes: LogicalScopeDefinitions,
    additional_slots: Vec<ActivitySlotDefinition>,
) -> Result<ActivityStateDefinition, DivergentUniverseEntryFlowError> {
    let family_value = match family {
        DivergentUniverseRunFamily::Ordinary => 1,
        DivergentUniverseRunFamily::Cyclical => 2,
    };
    let mut slots = vec![
        integer_slot(
            TAWOT_PURCHASES_SLOT,
            0,
            0,
            8,
            ActivityScope::Section,
            SlotCarryPolicy::Reset,
            62,
        )?,
        set_slot_with_limit(
            TAWOT_OFFER_SLOT,
            Vec::new(),
            ActivityScope::Section,
            SlotCarryPolicy::Reset,
            63,
            8,
        )?,
        boolean_slot(
            TAWOT_ACCEPTED_SLOT,
            false,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            64,
        )?,
        integer_slot(
            TAWOT_OPENS_SLOT,
            0,
            0,
            64,
            ActivityScope::Section,
            SlotCarryPolicy::Reset,
            65,
        )?,
        optional_slot(
            BATTLE_DOMAIN_SLOT,
            // This policy has one logical domain per layer/section. Physical
            // choice, encounter, battle and reward nodes must share its label.
            ActivityScope::Section,
            SlotCarryPolicy::Reset,
            61,
        )?,
        boolean_slot(
            INITIAL_EQUATION_ACCEPTED_SLOT,
            false,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            60,
        )?,
        stable_slot(
            PROFILE_SLOT,
            1,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            1,
        )?,
        stable_slot(
            ENTRY_SLOT,
            values.entry,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            2,
        )?,
        stable_slot(
            MODULE_SLOT,
            values.module,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            3,
        )?,
        stable_slot(
            RUN_FAMILY_SLOT,
            family_value,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            4,
        )?,
        stable_slot(
            DIFFICULTY_SLOT,
            values.difficulty,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            5,
        )?,
        stable_slot(
            AREA_SLOT,
            values.area,
            ActivityScope::Section,
            SlotCarryPolicy::CarryExact,
            6,
        )?,
        stable_slot(
            LAYER_SLOT,
            values.first_layer,
            ActivityScope::Section,
            SlotCarryPolicy::Replace,
            7,
        )?,
        integer_slot(
            LAYER_SEQUENCE_SLOT,
            0,
            0,
            64,
            ActivityScope::Section,
            SlotCarryPolicy::Replace,
            8,
        )?,
        optional_slot(ROOM_SLOT, ActivityScope::Node, SlotCarryPolicy::Replace, 9)?,
        counter_slot(
            CURRENCIES_SLOT,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            10,
        )?,
        set_slot(
            RUN_FLAGS_SLOT,
            vec![1],
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            11,
        )?,
        set_slot(
            PLANE_FLAGS_SLOT,
            Vec::new(),
            ActivityScope::Section,
            SlotCarryPolicy::Reset,
            12,
        )?,
        set_slot(
            PERMANENT_UNLOCKS_SLOT,
            values.permanent_unlocks.to_vec(),
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            13,
        )?,
        set_slot(
            DIFFICULTY_LEVELS_SLOT,
            progression
                .projection
                .difficulty_levels()
                .iter()
                .copied()
                .map(u64::from)
                .collect(),
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            14,
        )?,
        optional_value_slot(
            ASTRONOMICAL_MODE_SLOT,
            progression.mode_value,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            15,
        )?,
        optional_value_slot(
            DIVISION_SLOT,
            progression.division_value,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            16,
        )?,
        optional_value_slot(
            PROTOCOL_SLOT,
            progression.protocol_value,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            17,
        )?,
        integer_slot(
            COGNOCULI_SLOT,
            i64::from(progression.projection.initial_cognoculi()),
            0,
            64,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            18,
        )?,
        optional_value_slot(
            CYCLICAL_EPOCH_SLOT,
            progression.projection.cyclical_epoch(),
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            19,
        )?,
        private_stable_slot(
            ACCOUNT_LOADOUT_SNAPSHOT_SLOT,
            values.account_loadout_snapshot,
            20,
        )?,
        private_stable_slot(PARTY_SNAPSHOT_SLOT, values.party_snapshot, 21)?,
        counter_value_slot(
            MAPPING_STATE_SLOT,
            values.mapping_state,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            22,
        )?,
        set_slot(
            EQUATIONS_SLOT,
            Vec::new(),
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            23,
        )?,
        counter_slot(
            EQUATION_PROGRESS_SLOT,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            24,
        )?,
        counter_slot_with_limit(
            BLESSINGS_SLOT,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            25,
            414,
        )?,
        set_slot(
            TITAN_BOONS_SLOT,
            Vec::new(),
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            26,
        )?,
        counter_slot(
            SERVICE_RECEIPTS_SLOT,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            27,
        )?,
        set_slot(
            EQUATION_OFFERS_SLOT,
            Vec::new(),
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            28,
        )?,
        optional_slot(
            EQUATION_OFFER_SOURCE_SLOT,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            29,
        )?,
        integer_slot(
            EQUATION_REROLL_COUNT_SLOT,
            0,
            0,
            1,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            30,
        )?,
        set_slot(
            EXPANDED_EQUATIONS_SLOT,
            Vec::new(),
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            31,
        )?,
        set_slot_with_limit(
            EQUATION_BLESSING_SNAPSHOT_SLOT,
            Vec::new(),
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            32,
            414,
        )?,
        boolean_slot(
            EQUATION_PROGRESS_DIRTY_SLOT,
            false,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            33,
        )?,
        counter_slot_with_limit(
            BLESSING_OFFERS_SLOT,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            34,
            144,
        )?,
        optional_slot(
            BLESSING_OFFER_SOURCE_SLOT,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            35,
        )?,
        counter_slot_with_limit(
            CURIO_STATES_SLOT,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            36,
            235,
        )?,
        counter_slot_with_limit(
            CURIO_CHARGES_SLOT,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            37,
            235,
        )?,
        counter_slot_with_limit(
            CURIO_ACTIVATIONS_SLOT,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            38,
            235,
        )?,
        counter_slot_with_limit(
            GRAND_MIRACLES_SLOT,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            39,
            17,
        )?,
        optional_slot(
            TITAN_TYPE_SLOT,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            40,
        )?,
        set_slot_with_limit(
            TITAN_TALENTS_SLOT,
            Vec::new(),
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            41,
            36,
        )?,
        integer_slot(
            TITAN_TALENT_CURRENCY_SLOT,
            0,
            0,
            i64::MAX,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            42,
        )?,
        integer_slot(
            PERMANENT_TALENT_CURRENCY_SLOT,
            0,
            0,
            i64::MAX,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            43,
        )?,
        optional_slot(
            WEEKLY_MODIFIER_SLOT,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            44,
        )?,
        optional_slot(
            ROOM_MARK_SLOT,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            45,
        )?,
        optional_slot(
            WORKBENCH_SLOT,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            46,
        )?,
        counter_slot_with_limit(
            ACTIVITY_MECHANIC_LIFECYCLE_SLOT,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            47,
            669,
        )?,
        optional_slot(
            ROOM_DIALOGUE_PROGRAM_SLOT,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            48,
        )?,
        boolean_slot(
            ROOM_DIALOGUE_FINISHED_SLOT,
            false,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            49,
        )?,
        boolean_slot(
            ROOM_PREDICATE_SATISFIED_SLOT,
            false,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            50,
        )?,
        boolean_slot(
            ROOM_CONTENT_UPDATED_SLOT,
            false,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            51,
        )?,
        boolean_slot(
            ROOM_FINISHED_SLOT,
            false,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            52,
        )?,
        boolean_slot(
            ROOM_DOORS_OPEN_SLOT,
            false,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            53,
        )?,
        boolean_slot(
            ROOM_CONTENT_ENABLED_SLOT,
            false,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            54,
        )?,
        optional_slot(
            OCCURRENCE_REWARD_ACCEPTED_SLOT,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            55,
        )?,
        integer_slot(
            FRAGMENT_GAIN_BASE_SLOT,
            0,
            0,
            i64::MAX,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            56,
        )?,
        set_slot_with_limit(
            BATTLE_BLESSING_CANDIDATES_SLOT,
            Vec::new(),
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            57,
            8,
        )?,
        boolean_slot(
            BATTLE_BLESSING_ACCEPTED_SLOT,
            false,
            ActivityScope::Node,
            SlotCarryPolicy::Reset,
            58,
        )?,
        counter_slot_with_limit(
            EQUATION_GRANT_DOMAIN_VISITS_SLOT,
            ActivityScope::Activity,
            SlotCarryPolicy::CarryExact,
            59,
            512,
        )?,
    ];
    slots.extend(additional_slots);
    ActivityStateDefinition::new(slots, Vec::new(), Vec::new())
        .map(|state| state.with_logical_scopes(logical_scopes))
        .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}

fn private_stable_slot(
    id: ActivitySlotId,
    value: u64,
    source: u64,
) -> Result<ActivitySlotDefinition, DivergentUniverseEntryFlowError> {
    slot_definition_with_visibility(
        id,
        ActivityScope::Activity,
        ActivityValue::StableId(value),
        None,
        None,
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Private,
        source,
    )
}

fn stable_slot(
    id: ActivitySlotId,
    value: u64,
    owner: ActivityScope,
    carry: SlotCarryPolicy,
    source: u64,
) -> Result<ActivitySlotDefinition, DivergentUniverseEntryFlowError> {
    slot_definition(
        id,
        owner,
        ActivityValue::StableId(value),
        None,
        None,
        carry,
        source,
    )
}

fn optional_slot(
    id: ActivitySlotId,
    owner: ActivityScope,
    carry: SlotCarryPolicy,
    source: u64,
) -> Result<ActivitySlotDefinition, DivergentUniverseEntryFlowError> {
    optional_value_slot(id, None, owner, carry, source)
}

fn boolean_slot(
    id: ActivitySlotId,
    value: bool,
    owner: ActivityScope,
    carry: SlotCarryPolicy,
    source: u64,
) -> Result<ActivitySlotDefinition, DivergentUniverseEntryFlowError> {
    slot_definition(
        id,
        owner,
        ActivityValue::Boolean(value),
        None,
        None,
        carry,
        source,
    )
}

fn optional_value_slot(
    id: ActivitySlotId,
    value: Option<u64>,
    owner: ActivityScope,
    carry: SlotCarryPolicy,
    source: u64,
) -> Result<ActivitySlotDefinition, DivergentUniverseEntryFlowError> {
    slot_definition(
        id,
        owner,
        ActivityValue::OptionalId(value),
        None,
        None,
        carry,
        source,
    )
}

#[allow(clippy::too_many_arguments)]
fn integer_slot(
    id: ActivitySlotId,
    value: i64,
    minimum: i64,
    maximum: i64,
    owner: ActivityScope,
    carry: SlotCarryPolicy,
    source: u64,
) -> Result<ActivitySlotDefinition, DivergentUniverseEntryFlowError> {
    slot_definition(
        id,
        owner,
        ActivityValue::BoundedInteger(value),
        Some((minimum, maximum)),
        None,
        carry,
        source,
    )
}

fn counter_slot(
    id: ActivitySlotId,
    owner: ActivityScope,
    carry: SlotCarryPolicy,
    source: u64,
) -> Result<ActivitySlotDefinition, DivergentUniverseEntryFlowError> {
    counter_slot_with_limit(id, owner, carry, source, 256)
}

fn counter_slot_with_limit(
    id: ActivitySlotId,
    owner: ActivityScope,
    carry: SlotCarryPolicy,
    source: u64,
    maximum_entries: u32,
) -> Result<ActivitySlotDefinition, DivergentUniverseEntryFlowError> {
    counter_value_slot_with_limit(id, Box::new([]), owner, carry, source, maximum_entries)
}

fn counter_value_slot(
    id: ActivitySlotId,
    values: Box<[(u64, i64)]>,
    owner: ActivityScope,
    carry: SlotCarryPolicy,
    source: u64,
) -> Result<ActivitySlotDefinition, DivergentUniverseEntryFlowError> {
    counter_value_slot_with_limit(id, values, owner, carry, source, 256)
}

fn counter_value_slot_with_limit(
    id: ActivitySlotId,
    values: Box<[(u64, i64)]>,
    owner: ActivityScope,
    carry: SlotCarryPolicy,
    source: u64,
    maximum_entries: u32,
) -> Result<ActivitySlotDefinition, DivergentUniverseEntryFlowError> {
    slot_definition(
        id,
        owner,
        ActivityValue::BoundedCounterMap(values),
        Some((0, i64::MAX)),
        Some(maximum_entries),
        carry,
        source,
    )
}

fn set_slot(
    id: ActivitySlotId,
    values: Vec<u64>,
    owner: ActivityScope,
    carry: SlotCarryPolicy,
    source: u64,
) -> Result<ActivitySlotDefinition, DivergentUniverseEntryFlowError> {
    set_slot_with_limit(id, values, owner, carry, source, 256)
}

fn set_slot_with_limit(
    id: ActivitySlotId,
    values: Vec<u64>,
    owner: ActivityScope,
    carry: SlotCarryPolicy,
    source: u64,
    maximum_entries: u32,
) -> Result<ActivitySlotDefinition, DivergentUniverseEntryFlowError> {
    slot_definition(
        id,
        owner,
        ActivityValue::OrderedIdSet(values.into_boxed_slice()),
        None,
        Some(maximum_entries),
        carry,
        source,
    )
}

#[allow(clippy::too_many_arguments)]
fn slot_definition(
    id: ActivitySlotId,
    owner: ActivityScope,
    initial: ActivityValue,
    bounds: Option<(i64, i64)>,
    maximum_entries: Option<u32>,
    carry: SlotCarryPolicy,
    source: u64,
) -> Result<ActivitySlotDefinition, DivergentUniverseEntryFlowError> {
    slot_definition_with_visibility(
        id,
        owner,
        initial,
        bounds,
        maximum_entries,
        carry,
        ActivityStateVisibility::Player,
        source,
    )
}

#[allow(clippy::too_many_arguments)]
fn slot_definition_with_visibility(
    id: ActivitySlotId,
    owner: ActivityScope,
    initial: ActivityValue,
    bounds: Option<(i64, i64)>,
    maximum_entries: Option<u32>,
    carry: SlotCarryPolicy,
    visibility: ActivityStateVisibility,
    source: u64,
) -> Result<ActivitySlotDefinition, DivergentUniverseEntryFlowError> {
    let reset = match owner {
        ActivityScope::Activity => SlotResetPoint::ActivityStart,
        ActivityScope::Section => SlotResetPoint::SectionStart,
        ActivityScope::Node => SlotResetPoint::NodeStart,
        ActivityScope::Attempt => SlotResetPoint::AttemptStart,
    };
    ActivitySlotDefinition::new_with_policy(
        id,
        owner,
        initial,
        bounds,
        maximum_entries,
        vec![reset],
        carry,
        visibility,
        ActivityStateSource::new(source).expect("non-zero source"),
    )
    .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}

const fn slot(raw: u32) -> ActivitySlotId {
    match ActivitySlotId::new(raw) {
        Some(value) => value,
        None => panic!("non-zero slot"),
    }
}
