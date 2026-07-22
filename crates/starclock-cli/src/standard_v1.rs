//! CLI compatibility facade over the shared production Standard-v1 factory.

pub(crate) use starclock_data::standard_v1::{
    CATALOG_REVISION, CONFIG_DIGEST, RULES_REVISION, SCENARIOS, instantiate,
};
