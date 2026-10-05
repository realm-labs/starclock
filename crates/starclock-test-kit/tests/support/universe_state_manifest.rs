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
        118, 190, 186, 18, 157, 232, 5, 41, 243, 173, 136, 69, 200, 33, 124, 107, 227, 38, 163,
        209, 226, 226, 145, 113, 24, 254, 242, 144, 27, 35, 177, 154,
    ],
    universe_bundle: [
        129, 6, 7, 43, 75, 184, 214, 226, 83, 156, 251, 58, 122, 251, 160, 87, 16, 76, 52, 7, 7,
        106, 250, 29, 69, 164, 240, 15, 51, 35, 180, 15,
    ],
    topology_start: [
        16, 182, 207, 80, 205, 210, 219, 42, 211, 62, 142, 146, 21, 17, 91, 171, 4, 164, 142, 12,
        194, 84, 175, 172, 150, 122, 169, 69, 176, 195, 37, 29,
    ],
    encounter_settlement: [
        194, 191, 160, 39, 20, 41, 236, 248, 168, 127, 251, 190, 139, 162, 5, 16, 131, 215, 5, 159,
        40, 1, 241, 213, 192, 29, 167, 163, 24, 183, 41, 140,
    ],
    encounter_blessing_contributions: [
        33, 113, 61, 160, 81, 30, 26, 94, 132, 187, 187, 196, 37, 254, 185, 126, 53, 254, 40, 239,
        171, 52, 190, 152, 215, 246, 75, 156, 16, 30, 221, 25,
    ],
    baseline_final: [
        236, 203, 58, 130, 247, 134, 112, 106, 63, 137, 207, 233, 41, 78, 107, 146, 101, 244, 131,
        146, 122, 165, 27, 25, 207, 208, 35, 189, 82, 248, 42, 89,
    ],
    baseline_steps: 53,
    baseline_battles: 3,
    nested_final: [
        81, 150, 177, 75, 245, 192, 166, 28, 28, 210, 173, 60, 219, 69, 101, 89, 220, 33, 47, 144,
        28, 152, 63, 198, 233, 148, 225, 88, 25, 174, 89, 145,
    ],
    nested_first_events: [
        153, 24, 22, 78, 176, 214, 224, 252, 53, 24, 255, 213, 38, 155, 30, 152, 98, 136, 137, 240,
        101, 237, 124, 157, 185, 4, 91, 147, 65, 210, 151, 177,
    ],
    nested_battles: 6,
    nested_commands: 32,
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
