use starclock_activity::{
    ActivityScope, ActivitySlotDefinition, ActivitySlotId, ActivityStateDefinition,
    ActivityStateSource, ActivityStateVisibility, ActivityValue, SlotCarryPolicy, SlotResetPoint,
};

use crate::{
    gold_gears_structural::AreaDefinition,
    gold_gears_unique::{DiceFace, NeuralNode},
};

use super::{
    GoldAndGearsEntryError,
    state_layout::{
        BOARD_NODE_BEACON_SLOT, BOARD_NODE_BEACON_SOURCE, BOARD_NODE_DOMAIN_SLOT,
        BOARD_NODE_DOMAIN_SOURCE, BOARD_NODE_STATE_SLOT, BOARD_NODE_STATE_SOURCE, COGNITION_SLOT,
        COGNITION_SOURCE, CONTENT_LIFECYCLE_SLOT, CONTENT_LIFECYCLE_SOURCE,
        CONUNDRUM_AUXILIARY_KEY, CONUNDRUM_BERSERK_KEY, CONUNDRUM_SLOT, CONUNDRUM_SOURCE,
        CONUNDRUM_STATS_KEY, DEFERRED_EFFECTS_SLOT, DEFERRED_EFFECTS_SOURCE,
        DICE_LOADOUT_FACE_KEY_BASE, DICE_LOADOUT_MAX_RARITY_KEY_BASE, DICE_LOADOUT_SLOT,
        DICE_LOADOUT_SOURCE, DICE_RESOLUTION_SLOT, DICE_RESOLUTION_SOURCE, ENTRY_AREA_KEY,
        ENTRY_AUXILIARY_CONUNDRUM_KEY, ENTRY_DICE_KEY, ENTRY_DIFFICULTY_KEY, ENTRY_PATH_KEY,
        ENTRY_SELECTION_SLOT, ENTRY_SELECTION_SOURCE, ENTRY_STATS_CONUNDRUM_KEY,
        INITIAL_COSMIC_FRAGMENTS, INITIAL_DICE_CHEATS, INITIAL_DICE_REROLLS, KNOWLEDGE_SLOT,
        KNOWLEDGE_SOURCE, NEURAL_NETWORK_SLOT, NEURAL_NETWORK_SOURCE, NODE_VISIT_SLOT,
        NODE_VISIT_SOURCE, PLANE_STATE_SLOT, PLANE_STATE_SOURCE, PROGRESSION_SLOT,
        PROGRESSION_SOURCE, PROGRESSION_TRAILBLAZE_BONUS_KEY, RESOURCE_COSMIC_FRAGMENTS_KEY,
        RESOURCE_DICE_CHEATS_KEY, RESOURCE_DICE_REROLLS_KEY, RUN_RESOURCES_SLOT,
        RUN_RESOURCES_SOURCE, SECRETS_SLOT, SECRETS_SOURCE,
    },
};

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_state(
    area: &AreaDefinition,
    path_id: u32,
    dice_id: u32,
    faces: &[&DiceFace],
    dice_slot_max_rarities: &[u8],
    neural: &[&NeuralNode],
    stats: u8,
    auxiliary: u8,
    trailblaze_bonus: Option<u32>,
    cognition_initial: i64,
    cognition_minimum: i64,
    cognition_maximum: i64,
) -> Result<ActivityStateDefinition, GoldAndGearsEntryError> {
    let entry = vec![
        (ENTRY_AREA_KEY, i64::from(area.id.0)),
        (ENTRY_DIFFICULTY_KEY, i64::from(area.difficulty)),
        (ENTRY_PATH_KEY, i64::from(path_id)),
        (ENTRY_DICE_KEY, i64::from(dice_id)),
        (ENTRY_STATS_CONUNDRUM_KEY, i64::from(stats)),
        (ENTRY_AUXILIARY_CONUNDRUM_KEY, i64::from(auxiliary)),
    ];
    let mut loadout = faces
        .iter()
        .enumerate()
        .map(|(index, face)| {
            (
                DICE_LOADOUT_FACE_KEY_BASE
                    + u64::try_from(index + 1).expect("six slot indices fit u64"),
                i64::from(face.identity.id.0),
            )
        })
        .collect::<Vec<_>>();
    loadout.extend(
        dice_slot_max_rarities
            .iter()
            .enumerate()
            .map(|(index, rarity)| {
                (
                    DICE_LOADOUT_MAX_RARITY_KEY_BASE
                        + u64::try_from(index + 1).expect("six slot indices fit u64"),
                    i64::from(*rarity),
                )
            }),
    );
    let neural_ids = neural
        .iter()
        .map(|node| u64::from(node.identity.id.0))
        .collect::<Vec<_>>();
    let conundrum = vec![
        (CONUNDRUM_STATS_KEY, i64::from(stats)),
        (CONUNDRUM_AUXILIARY_KEY, i64::from(auxiliary)),
        (CONUNDRUM_BERSERK_KEY, 0),
    ];
    let progression = trailblaze_bonus.map_or_else(Vec::new, |bonus| {
        vec![(PROGRESSION_TRAILBLAZE_BONUS_KEY, i64::from(bonus))]
    });
    let resources = vec![
        (RESOURCE_COSMIC_FRAGMENTS_KEY, INITIAL_COSMIC_FRAGMENTS),
        (RESOURCE_DICE_REROLLS_KEY, INITIAL_DICE_REROLLS),
        (RESOURCE_DICE_CHEATS_KEY, INITIAL_DICE_CHEATS),
    ];
    let slots = vec![
        map_slot(
            ENTRY_SELECTION_SLOT,
            ActivityScope::Activity,
            entry,
            6,
            0,
            i64::from(u32::MAX),
            ENTRY_SELECTION_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        map_slot(
            DICE_LOADOUT_SLOT,
            ActivityScope::Activity,
            loadout,
            18,
            0,
            i64::from(u32::MAX),
            DICE_LOADOUT_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        set_slot(
            NEURAL_NETWORK_SLOT,
            ActivityScope::Activity,
            neural_ids,
            40,
            NEURAL_NETWORK_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        map_slot(
            CONUNDRUM_SLOT,
            ActivityScope::Activity,
            conundrum,
            3,
            0,
            12,
            CONUNDRUM_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        integer_slot(
            COGNITION_SLOT,
            cognition_initial,
            cognition_minimum,
            cognition_maximum,
            COGNITION_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        set_slot(
            SECRETS_SLOT,
            ActivityScope::Activity,
            vec![],
            20,
            SECRETS_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        map_slot(
            PROGRESSION_SLOT,
            ActivityScope::Activity,
            progression,
            64,
            0,
            i64::from(u32::MAX),
            PROGRESSION_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        map_slot(
            RUN_RESOURCES_SLOT,
            ActivityScope::Activity,
            resources,
            16,
            0,
            1_000_000_000,
            RUN_RESOURCES_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        empty_map_slot(
            CONTENT_LIFECYCLE_SLOT,
            ActivityScope::Activity,
            160,
            CONTENT_LIFECYCLE_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        empty_map_slot(
            DEFERRED_EFFECTS_SLOT,
            ActivityScope::Activity,
            4_096,
            DEFERRED_EFFECTS_SOURCE,
            ActivityStateVisibility::DebugOnly,
        )?,
        empty_map_slot(
            PLANE_STATE_SLOT,
            ActivityScope::Section,
            16,
            PLANE_STATE_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        empty_map_slot(
            BOARD_NODE_STATE_SLOT,
            ActivityScope::Section,
            2_502,
            BOARD_NODE_STATE_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        empty_map_slot(
            BOARD_NODE_DOMAIN_SLOT,
            ActivityScope::Section,
            2_502,
            BOARD_NODE_DOMAIN_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        empty_map_slot(
            BOARD_NODE_BEACON_SLOT,
            ActivityScope::Section,
            2_502,
            BOARD_NODE_BEACON_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        empty_map_slot(
            KNOWLEDGE_SLOT,
            ActivityScope::Section,
            2_502,
            KNOWLEDGE_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        empty_map_slot(
            NODE_VISIT_SLOT,
            ActivityScope::Node,
            32,
            NODE_VISIT_SOURCE,
            ActivityStateVisibility::DebugOnly,
        )?,
        empty_map_slot(
            DICE_RESOLUTION_SLOT,
            ActivityScope::Attempt,
            128,
            DICE_RESOLUTION_SOURCE,
            ActivityStateVisibility::Player,
        )?,
    ];
    ActivityStateDefinition::new(slots, vec![], vec![])
        .map_err(|_| GoldAndGearsEntryError::InvalidActivityState)
}

#[allow(clippy::too_many_arguments)]
fn map_slot(
    id: u32,
    owner: ActivityScope,
    initial: Vec<(u64, i64)>,
    maximum_entries: u32,
    minimum: i64,
    maximum: i64,
    source: u64,
    visibility: ActivityStateVisibility,
) -> Result<ActivitySlotDefinition, GoldAndGearsEntryError> {
    slot_with_policy(
        id,
        owner,
        ActivityValue::BoundedCounterMap(initial.into_boxed_slice()),
        Some((minimum, maximum)),
        Some(maximum_entries),
        source,
        visibility,
    )
}

fn empty_map_slot(
    id: u32,
    owner: ActivityScope,
    maximum_entries: u32,
    source: u64,
    visibility: ActivityStateVisibility,
) -> Result<ActivitySlotDefinition, GoldAndGearsEntryError> {
    map_slot(
        id,
        owner,
        vec![],
        maximum_entries,
        0,
        i64::MAX,
        source,
        visibility,
    )
}

fn set_slot(
    id: u32,
    owner: ActivityScope,
    initial: Vec<u64>,
    maximum_entries: u32,
    source: u64,
    visibility: ActivityStateVisibility,
) -> Result<ActivitySlotDefinition, GoldAndGearsEntryError> {
    slot_with_policy(
        id,
        owner,
        ActivityValue::OrderedIdSet(initial.into_boxed_slice()),
        None,
        Some(maximum_entries),
        source,
        visibility,
    )
}

fn integer_slot(
    id: u32,
    initial: i64,
    minimum: i64,
    maximum: i64,
    source: u64,
    visibility: ActivityStateVisibility,
) -> Result<ActivitySlotDefinition, GoldAndGearsEntryError> {
    slot_with_policy(
        id,
        ActivityScope::Activity,
        ActivityValue::BoundedInteger(initial),
        Some((minimum, maximum)),
        None,
        source,
        visibility,
    )
}

#[allow(clippy::too_many_arguments)]
fn slot_with_policy(
    id: u32,
    owner: ActivityScope,
    initial: ActivityValue,
    bounds: Option<(i64, i64)>,
    maximum_entries: Option<u32>,
    source: u64,
    visibility: ActivityStateVisibility,
) -> Result<ActivitySlotDefinition, GoldAndGearsEntryError> {
    let (reset, carry) = match owner {
        ActivityScope::Activity => (SlotResetPoint::ActivityStart, SlotCarryPolicy::CarryExact),
        ActivityScope::Section => (SlotResetPoint::SectionStart, SlotCarryPolicy::Reset),
        ActivityScope::Node => (SlotResetPoint::NodeStart, SlotCarryPolicy::Reset),
        ActivityScope::Attempt => (SlotResetPoint::AttemptStart, SlotCarryPolicy::Reset),
    };
    ActivitySlotDefinition::new_with_policy(
        ActivitySlotId::new(id).expect("static Gold and Gears slot ID is non-zero"),
        owner,
        initial,
        bounds,
        maximum_entries,
        vec![reset],
        carry,
        visibility,
        ActivityStateSource::new(source).expect("static Gold and Gears source is non-zero"),
    )
    .map_err(|_| GoldAndGearsEntryError::InvalidActivityState)
}
