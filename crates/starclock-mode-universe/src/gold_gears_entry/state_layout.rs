//! Stable state identities for the Gold and Gears Activity profile.

pub(super) const ENTRY_SELECTION_SLOT: u32 = 0x4747_0001;
pub(super) const DICE_LOADOUT_SLOT: u32 = 0x4747_0002;
pub(super) const NEURAL_NETWORK_SLOT: u32 = 0x4747_0003;
pub(super) const CONUNDRUM_SLOT: u32 = 0x4747_0004;
pub(super) const COGNITION_SLOT: u32 = 0x4747_0005;
pub(super) const SECRETS_SLOT: u32 = 0x4747_0006;
pub(super) const PROGRESSION_SLOT: u32 = 0x4747_0007;
pub(super) const RUN_RESOURCES_SLOT: u32 = 0x4747_0008;
pub(super) const CONTENT_LIFECYCLE_SLOT: u32 = 0x4747_0009;
pub(super) const DEFERRED_EFFECTS_SLOT: u32 = 0x4747_000A;
pub(super) const PLANE_STATE_SLOT: u32 = 0x4747_000B;
pub(super) const BOARD_NODE_STATE_SLOT: u32 = 0x4747_000C;
pub(super) const BOARD_NODE_DOMAIN_SLOT: u32 = 0x4747_000D;
pub(super) const BOARD_NODE_BEACON_SLOT: u32 = 0x4747_000E;
pub(super) const KNOWLEDGE_SLOT: u32 = 0x4747_000F;
pub(super) const NODE_VISIT_SLOT: u32 = 0x4747_0010;
pub(super) const DICE_RESOLUTION_SLOT: u32 = 0x4747_0011;

pub(super) const ENTRY_SELECTION_SOURCE: u64 = 0x4747_1400_0000_0001;
pub(super) const DICE_LOADOUT_SOURCE: u64 = 0x4747_1400_0000_0002;
pub(super) const NEURAL_NETWORK_SOURCE: u64 = 0x4747_1400_0000_0003;
pub(super) const CONUNDRUM_SOURCE: u64 = 0x4747_1400_0000_0004;
pub(super) const COGNITION_SOURCE: u64 = 0x4747_1400_0000_0005;
pub(super) const SECRETS_SOURCE: u64 = 0x4747_1400_0000_0006;
pub(super) const PROGRESSION_SOURCE: u64 = 0x4747_1400_0000_0007;
pub(super) const RUN_RESOURCES_SOURCE: u64 = 0x4747_1400_0000_0008;
pub(super) const CONTENT_LIFECYCLE_SOURCE: u64 = 0x4747_1400_0000_0009;
pub(super) const DEFERRED_EFFECTS_SOURCE: u64 = 0x4747_1400_0000_000A;
pub(super) const PLANE_STATE_SOURCE: u64 = 0x4747_1400_0000_000B;
pub(super) const BOARD_NODE_STATE_SOURCE: u64 = 0x4747_1400_0000_000C;
pub(super) const BOARD_NODE_DOMAIN_SOURCE: u64 = 0x4747_1400_0000_000D;
pub(super) const BOARD_NODE_BEACON_SOURCE: u64 = 0x4747_1400_0000_000E;
pub(super) const KNOWLEDGE_SOURCE: u64 = 0x4747_1400_0000_000F;
pub(super) const NODE_VISIT_SOURCE: u64 = 0x4747_1400_0000_0010;
pub(super) const DICE_RESOLUTION_SOURCE: u64 = 0x4747_1400_0000_0011;

pub(super) const ENTRY_AREA_KEY: u64 = 1;
pub(super) const ENTRY_DIFFICULTY_KEY: u64 = 2;
pub(super) const ENTRY_PATH_KEY: u64 = 3;
pub(super) const ENTRY_DICE_KEY: u64 = 4;
pub(super) const ENTRY_STATS_CONUNDRUM_KEY: u64 = 5;
pub(super) const ENTRY_AUXILIARY_CONUNDRUM_KEY: u64 = 6;

pub(super) const DICE_LOADOUT_FACE_KEY_BASE: u64 = 0;
pub(super) const DICE_LOADOUT_MAX_RARITY_KEY_BASE: u64 = 0x100;

pub(super) const CONUNDRUM_STATS_KEY: u64 = 1;
pub(super) const CONUNDRUM_AUXILIARY_KEY: u64 = 2;
pub(super) const CONUNDRUM_BERSERK_KEY: u64 = 3;

pub(super) const PROGRESSION_TRAILBLAZE_BONUS_KEY: u64 = 1;
pub(super) const PROGRESSION_DICE_PATH_VALUE_KEY: u64 = 2;
pub(super) const PROGRESSION_DICE_PATH_INTERVAL_KEY: u64 = 3;
pub(super) const PROGRESSION_DICE_PATH_SCALED_VALUE_KEY: u64 = 4;
pub(super) const PROGRESSION_DICE_PATH_TRIGGER_PROGRESS_KEY: u64 = 5;
pub(super) const PROGRESSION_DICE_PATH_BOOST_STACKS_KEY: u64 = 6;

pub(super) const RESOURCE_COSMIC_FRAGMENTS_KEY: u64 = 1;
pub(super) const RESOURCE_DICE_REROLLS_KEY: u64 = 2;
pub(super) const RESOURCE_DICE_CHEATS_KEY: u64 = 3;

pub(super) const DICE_RESOLUTION_FACE_KEY: u64 = 1;
pub(super) const DICE_RESOLUTION_PREVIOUS_FACE_KEY: u64 = 2;
pub(super) const DICE_RESOLUTION_KIND_KEY: u64 = 3;
pub(super) const DICE_RESOLUTION_CANDIDATE_COUNT_KEY: u64 = 4;
pub(super) const DICE_RESOLUTION_DRAW_INDEX_KEY: u64 = 5;

pub(super) const DEFERRED_DICE_INITIAL_BASE: u64 = 0x4747_3000_0000_0000;
pub(super) const DEFERRED_DICE_PASSIVE_BASE: u64 = 0x4747_3100_0000_0000;

pub(super) const INITIAL_COSMIC_FRAGMENTS: i64 = 100;
pub(super) const INITIAL_DICE_REROLLS: i64 = 1;
pub(super) const INITIAL_DICE_CHEATS: i64 = 0;
