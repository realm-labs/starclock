//! Private production Standard Universe assembly for agent sessions.

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    production_runtime::{
        StandardUniverseControllerIdentity, StandardUniverseRuntimeFactory,
        StandardUniverseRuntimeFactoryError, StandardUniverseRuntimeInstance,
    },
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

#[derive(Clone)]
pub(crate) struct ActivityRuntimeFactory {
    runtime: StandardUniverseRuntimeFactory,
}

impl ActivityRuntimeFactory {
    pub(crate) fn load() -> Result<Self, ActivityRuntimeError> {
        Ok(Self {
            runtime: StandardUniverseRuntimeFactory::load(CORE_BUNDLE, UNIVERSE_BUNDLE)
                .map_err(ActivityRuntimeError::Runtime)?,
        })
    }

    pub(crate) fn start(
        &self,
        world: u32,
        difficulty_index: usize,
        seed: u64,
        controller_digest: [u8; 32],
    ) -> Result<StandardUniverseRuntimeInstance, ActivityRuntimeError> {
        self.runtime
            .start(
                world,
                difficulty_index,
                seed,
                StandardUniverseControllerIdentity {
                    id: "agent-activity-controller",
                    digest: controller_digest,
                },
            )
            .map_err(ActivityRuntimeError::Runtime)
    }

    pub(crate) fn catalog(&self) -> &UniverseCatalog {
        self.runtime.catalog()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ActivityRuntimeError {
    Runtime(StandardUniverseRuntimeFactoryError),
}
