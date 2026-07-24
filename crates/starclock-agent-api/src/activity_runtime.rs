//! Private production Standard Universe assembly for agent sessions.

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    nested_battle_executor::UNIVERSE_NESTED_BATTLE_EXECUTOR_REVISION,
    production_runtime::{
        StandardUniverseControllerIdentity, StandardUniverseRuntimeFactory,
        StandardUniverseRuntimeFactoryError, StandardUniverseRuntimeInstance,
    },
};

use crate::activity_session::ACTIVITY_AGENT_CONTROLLER_REVISION;

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");
pub(crate) const BATTLE_EXECUTOR_REVISION: &str = UNIVERSE_NESTED_BATTLE_EXECUTOR_REVISION;

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
                    revision: ACTIVITY_AGENT_CONTROLLER_REVISION,
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
