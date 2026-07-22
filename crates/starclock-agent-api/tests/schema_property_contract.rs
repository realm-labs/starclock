use std::{collections::BTreeMap, str::FromStr};

use proptest::prelude::*;
use proptest::test_runner::{RngAlgorithm, RngSeed};
use serde_json::Value;
use sha2::{Digest, Sha256};
use starclock_agent_api::{
    error::{AgentError, AgentErrorCode},
    observation::{MAX_EFFECTS, MAX_EVENTS_PER_PAGE, MAX_TIMELINE_ENTRIES, MAX_UNITS},
    schema::{
        ACTION_SCHEMA_JSON, AGENT_SCHEMA_BUNDLE_SHA256, AgentHash, AgentSInt, AgentSchemaRevision,
        AgentUInt, AgentValueError, ERROR_SCHEMA_JSON, OBSERVATION_SCHEMA_JSON,
        ORDINARY_OBSERVATION_GOLDEN_JSON, STALE_DECISION_ERROR_GOLDEN_JSON, SessionId,
        TRIGGER_HEAVY_ACTION_GOLDEN_JSON,
    },
};

const PROPERTY_CASES: u32 = 512;

fn property_config() -> ProptestConfig {
    ProptestConfig {
        cases: PROPERTY_CASES,
        max_shrink_iters: 4_096,
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(0x6167_656e_742d_7631),
        failure_persistence: None,
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(property_config())]

    #[test]
    fn exact_integer_json_round_trips_every_generated_backing_value(unsigned in any::<u64>(), signed in any::<i64>()) {
        let unsigned_value = AgentUInt::from_u64(unsigned);
        let signed_value = AgentSInt::from_i64(signed);
        let unsigned_json = serde_json::to_string(&unsigned_value).unwrap();
        let signed_json = serde_json::to_string(&signed_value).unwrap();
        prop_assert_eq!(serde_json::from_str::<AgentUInt>(&unsigned_json).unwrap(), unsigned_value);
        prop_assert_eq!(serde_json::from_str::<AgentSInt>(&signed_json).unwrap(), signed_value);
        prop_assert!(serde_json::from_str::<Value>(&unsigned_json).unwrap().is_string());
        prop_assert!(serde_json::from_str::<Value>(&signed_json).unwrap().is_string());
    }

    #[test]
    fn every_unknown_printable_revision_is_rejected(value in "[ -~]{0,64}") {
        prop_assume!(value != "agent-api-v1");
        prop_assert_eq!(AgentSchemaRevision::from_str(&value), Err(AgentValueError::UnknownRevision));
    }

    #[test]
    fn detail_serialization_is_independent_of_generated_insertion_order(
        entries in proptest::collection::btree_map("[a-z]{1,12}", "[a-z0-9]{1,24}", 0..=16),
        reverse in any::<bool>(),
    ) {
        let mut ordered = entries.iter().collect::<Vec<_>>();
        if reverse { ordered.reverse(); }
        let mut error = AgentError::new(AgentErrorCode::InvalidRequest, "invalid", false, false).unwrap();
        for (key, value) in ordered { error.insert_detail(key.as_str(), value.as_str()).unwrap(); }
        let serialized: Value = serde_json::to_value(error).unwrap();
        let expected = entries.into_iter().collect::<BTreeMap<_, _>>();
        let actual = serialized.get("details").map_or_else(BTreeMap::new, |details| {
            details.as_object().unwrap().iter().map(|(key, value)| (key.clone(), value.as_str().unwrap().to_owned())).collect()
        });
        prop_assert_eq!(actual, expected);
    }
}

#[test]
fn published_schema_and_golden_bytes_match_the_frozen_bundle_digest() {
    let files = [
        (
            "schemas/agent-api-v1/action.schema.json",
            ACTION_SCHEMA_JSON,
        ),
        ("schemas/agent-api-v1/error.schema.json", ERROR_SCHEMA_JSON),
        (
            "schemas/agent-api-v1/goldens/ordinary-observation.json",
            ORDINARY_OBSERVATION_GOLDEN_JSON,
        ),
        (
            "schemas/agent-api-v1/goldens/stale-decision-error.json",
            STALE_DECISION_ERROR_GOLDEN_JSON,
        ),
        (
            "schemas/agent-api-v1/goldens/trigger-heavy-action-response.json",
            TRIGGER_HEAVY_ACTION_GOLDEN_JSON,
        ),
        (
            "schemas/agent-api-v1/observation.schema.json",
            OBSERVATION_SCHEMA_JSON,
        ),
    ];
    let mut digest = Sha256::new();
    for (path, contents) in files {
        serde_json::from_str::<Value>(contents).expect("published artifact is valid JSON");
        digest.update(path.as_bytes());
        digest.update(b"\0");
        digest.update(contents.as_bytes());
        digest.update(b"\0");
    }
    let digest: [u8; 32] = digest.finalize().into();
    assert_eq!(
        AgentHash::from_bytes(digest).as_str(),
        AGENT_SCHEMA_BUNDLE_SHA256
    );
}

#[test]
fn implementation_bounds_equal_the_frozen_schema_limits() {
    let policy: Value =
        serde_json::from_str(include_str!("../../../policy/agent-api-v1.json")).unwrap();
    assert_eq!(policy["limits"]["max_units"], MAX_UNITS);
    assert_eq!(policy["limits"]["max_effects"], MAX_EFFECTS);
    assert_eq!(
        policy["limits"]["max_timeline_entries"],
        MAX_TIMELINE_ENTRIES
    );
    assert_eq!(policy["limits"]["max_events_per_page"], MAX_EVENTS_PER_PAGE);
}

#[test]
fn error_details_enforce_schema_cardinality_and_value_bounds() {
    let mut error =
        AgentError::new(AgentErrorCode::InvalidRequest, "invalid", false, false).unwrap();
    for index in 0..16 {
        error
            .insert_detail(format!("key_{index:02}"), "value")
            .unwrap();
    }
    assert!(error.insert_detail("key_16", "value").is_err());
    assert!(error.insert_detail("key_00", "x".repeat(513)).is_err());
    assert_eq!(error.details().len(), 16);
}

#[test]
fn stable_error_value_matches_the_published_golden_shape() {
    let mut error = AgentError::new(
        AgentErrorCode::StaleDecision,
        "The requested decision is no longer current.",
        true,
        false,
    )
    .unwrap();
    error.session_id = Some(SessionId::parse("session_01").unwrap());
    error.decision_id = Some(AgentUInt::from_u64(11));
    error.state_hash = Some(
        AgentHash::parse("5021cdd60c41fd953f6d567905a079255a434248c504c03d95a30d6f7c63625e")
            .unwrap(),
    );
    error.insert_detail("current_decision_id", "12").unwrap();
    assert_eq!(
        serde_json::to_value(error).unwrap(),
        serde_json::from_str::<Value>(STALE_DECISION_ERROR_GOLDEN_JSON).unwrap()
    );
}
