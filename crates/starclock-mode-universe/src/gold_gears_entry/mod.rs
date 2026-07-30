//! Gold and Gears entry validation and generic Activity-state compilation.

mod api;
mod error;
mod map_overlay;
mod state;
mod state_layout;
mod topology;
mod validate;

pub use api::{
    GOLD_AND_GEARS_ENTRY_REVISION, GOLD_AND_GEARS_TOPOLOGY_REVISION, GoldAndGearsEntry,
    GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance,
};
pub use error::GoldAndGearsEntryError;

const EXPECTED_PROFILE_KEY: &str = "gold-gears.profile.v1";
const CONUNDRUM_AREA_KEY: &str = "gold-gears.area.405";

#[cfg(test)]
mod map_overlay_tests;
#[cfg(test)]
mod tests;
