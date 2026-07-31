//! Shared immutable fixtures for Starclock's responsibility-sharded test suites.
//!
//! Production crates never depend on this crate. Heavy cross-crate tests use
//! these process-local fixtures and create fresh mutable battle or Activity
//! state for each test.

use std::sync::{Arc, OnceLock};

use starclock_agent_api::{
    activity_session::ActivityAgentSessionFactory, session::AgentSessionFactory,
};
use starclock_mode_universe::{
    catalog::UniverseCatalog, gold_gears_entry::GoldAndGearsRuntimeFactory,
    production_runtime::StandardUniverseRuntimeFactory,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");
const GOLD_GEARS_BUNDLE: &[u8] =
    include_bytes!("../../../config/gold-and-gears-generated/config.sora");

/// Returns the process-local immutable Standard Universe catalog.
pub fn universe_catalog() -> &'static Arc<UniverseCatalog> {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let core = starclock_data::catalog::load(CORE_BUNDLE)
            .expect("the checked-in core test bundle must load");
        UniverseCatalog::load(UNIVERSE_BUNDLE, core)
            .expect("the checked-in Universe test bundle must load")
    })
}

/// Returns the process-local immutable Standard Universe runtime factory.
///
/// Tests that assert assembly-cache metrics should construct an isolated
/// factory instead of sharing this fixture.
pub fn standard_universe_factory() -> &'static StandardUniverseRuntimeFactory {
    static FACTORY: OnceLock<StandardUniverseRuntimeFactory> = OnceLock::new();
    FACTORY.get_or_init(|| {
        StandardUniverseRuntimeFactory::load(CORE_BUNDLE, UNIVERSE_BUNDLE)
            .expect("the checked-in Standard Universe test bundles must compile")
    })
}

/// Returns the process-local immutable Gold and Gears runtime factory.
pub fn gold_gears_factory() -> &'static GoldAndGearsRuntimeFactory {
    static FACTORY: OnceLock<GoldAndGearsRuntimeFactory> = OnceLock::new();
    FACTORY.get_or_init(|| {
        GoldAndGearsRuntimeFactory::load_candidate(GOLD_GEARS_BUNDLE)
            .expect("the checked-in Gold and Gears Candidate bundle must compile")
    })
}

/// Returns the process-local immutable Standard battle agent factory.
pub fn agent_session_factory() -> &'static AgentSessionFactory {
    static FACTORY: OnceLock<AgentSessionFactory> = OnceLock::new();
    FACTORY.get_or_init(|| {
        AgentSessionFactory::load_production()
            .expect("the checked-in Standard battle agent fixtures must compile")
    })
}

/// Returns the process-local immutable Standard Universe agent factory.
pub fn activity_agent_session_factory() -> &'static ActivityAgentSessionFactory {
    static FACTORY: OnceLock<ActivityAgentSessionFactory> = OnceLock::new();
    FACTORY.get_or_init(|| {
        ActivityAgentSessionFactory::load_production()
            .expect("the checked-in Standard Universe agent fixtures must compile")
    })
}
