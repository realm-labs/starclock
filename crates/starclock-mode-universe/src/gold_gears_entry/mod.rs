//! Gold and Gears entry validation and generic Activity-state compilation.

mod api;
mod cognition;
mod dice_face;
mod dice_loadout;
mod dice_passive;
mod dice_resolution;
mod error;
mod map_overlay;
mod plane_transition;
mod state;
mod state_layout;
mod topology;
mod validate;

pub use api::{
    GOLD_AND_GEARS_ENTRY_REVISION, GOLD_AND_GEARS_TOPOLOGY_REVISION, GoldAndGearsEntry,
    GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance,
};
pub use cognition::GOLD_AND_GEARS_COGNITION_REVISION;
pub use dice_face::GOLD_AND_GEARS_DICE_FACE_REVISION;
pub use dice_loadout::GOLD_AND_GEARS_DICE_LOADOUT_REVISION;
pub use dice_passive::{GoldAndGearsDiceDomain, GoldAndGearsDicePassiveEvent};
pub use dice_resolution::GOLD_AND_GEARS_DICE_RUNTIME_REVISION;
pub use error::GoldAndGearsEntryError;
pub use plane_transition::GOLD_AND_GEARS_PLANE_COMPLETION_REVISION;

const EXPECTED_PROFILE_KEY: &str = "gold-gears.profile.v1";
const CONUNDRUM_AREA_KEY: &str = "gold-gears.area.405";

#[cfg(test)]
mod cognition_tests;
#[cfg(test)]
mod dice_face_tests;
#[cfg(test)]
mod dice_loadout_tests;
#[cfg(test)]
mod dice_resolution_tests;
#[cfg(test)]
mod map_overlay_tests;
#[cfg(test)]
mod phase2_hardening_tests;
#[cfg(test)]
mod tests;
