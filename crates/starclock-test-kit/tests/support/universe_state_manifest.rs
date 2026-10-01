//! Current canonical vectors for production-input-sensitive Universe fixtures.
//! Replace these values when the exact current inputs change; no legacy vectors.
use sha2::{Digest, Sha256};

pub(crate) struct UniverseStateManifest {
    core_bundle: [u8; 32],
    universe_bundle: [u8; 32],
    pub(crate) topology_start: [u8; 32],
    pub(crate) encounter_settlement: [u8; 32],
    pub(crate) encounter_blessing_contributions: [u8; 32],
    pub(crate) baseline_final: [u8; 32],
    pub(crate) baseline_steps: usize,
    pub(crate) baseline_battles: usize,
    pub(crate) nested_final: [u8; 32],
    pub(crate) nested_first_events: [u8; 32],
    pub(crate) nested_battles: usize,
    pub(crate) nested_commands: usize,
}

pub(crate) const MANIFEST: UniverseStateManifest = UniverseStateManifest {
    core_bundle: [
        106, 126, 40, 227, 3, 84, 59, 209, 151, 2, 125, 142, 230, 160, 188, 106, 35, 54, 178, 42,
        26, 111, 206, 129, 150, 174, 85, 194, 90, 223, 224, 185,
    ],
    universe_bundle: [
        129, 6, 7, 43, 75, 184, 214, 226, 83, 156, 251, 58, 122, 251, 160, 87, 16, 76, 52, 7, 7,
        106, 250, 29, 69, 164, 240, 15, 51, 35, 180, 15,
    ],
    topology_start: [
        159, 52, 226, 215, 217, 165, 191, 105, 15, 53, 220, 239, 29, 243, 224, 91, 244, 13, 37, 76,
        200, 142, 157, 136, 14, 8, 13, 40, 255, 17, 93, 113,
    ],
    encounter_settlement: [
        64, 134, 239, 223, 244, 77, 239, 175, 213, 193, 51, 18, 125, 133, 86, 209, 178, 70, 88,
        205, 48, 140, 81, 62, 121, 129, 99, 143, 0, 28, 105, 105,
    ],
    encounter_blessing_contributions: [
        104, 216, 195, 46, 168, 159, 169, 235, 131, 128, 233, 100, 100, 230, 181, 155, 4, 225, 176,
        85, 23, 116, 247, 167, 253, 147, 117, 122, 209, 92, 175, 197,
    ],
    baseline_final: [
        177, 230, 221, 180, 142, 13, 108, 119, 58, 227, 34, 228, 232, 81, 163, 63, 240, 51, 88,
        246, 83, 242, 139, 170, 202, 207, 232, 4, 154, 83, 76, 86,
    ],
    baseline_steps: 65,
    baseline_battles: 6,
    nested_final: [
        234, 9, 83, 61, 174, 150, 146, 251, 186, 234, 112, 161, 20, 22, 193, 232, 28, 126, 105,
        129, 211, 36, 247, 148, 128, 224, 24, 243, 113, 253, 149, 85,
    ],
    nested_first_events: [
        94, 210, 198, 41, 148, 58, 210, 129, 166, 159, 11, 245, 102, 66, 138, 200, 181, 113, 70,
        35, 18, 163, 116, 69, 109, 119, 5, 235, 2, 205, 120, 161,
    ],
    nested_battles: 4,
    nested_commands: 24,
};

impl UniverseStateManifest {
    pub(crate) fn assert_current_inputs(&self) {
        let core: [u8; 32] =
            Sha256::digest(include_bytes!("../../../../config/generated/config.sora")).into();
        let universe: [u8; 32] = Sha256::digest(include_bytes!(
            "../../../../config/universe-generated/config.sora"
        ))
        .into();
        assert_eq!(core, self.core_bundle, "current core input identity");
        assert_eq!(
            universe, self.universe_bundle,
            "current Universe input identity"
        );
    }
}
