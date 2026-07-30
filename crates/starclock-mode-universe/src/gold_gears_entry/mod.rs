//! Gold and Gears entry validation and generic Activity-state compilation.

mod api;
mod error;
mod state;
mod state_layout;
mod validate;

pub use api::{
    GOLD_AND_GEARS_ENTRY_REVISION, GoldAndGearsEntry, GoldAndGearsRuntimeFactory,
    GoldAndGearsRuntimeInstance,
};
pub use error::GoldAndGearsEntryError;

const EXPECTED_PROFILE_KEY: &str = "gold-gears.profile.v1";
const CONUNDRUM_AREA_KEY: &str = "gold-gears.area.405";

#[cfg(test)]
mod tests;
