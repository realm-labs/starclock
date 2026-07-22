use std::str::FromStr;

use starclock_agent_api::{
    error::{AgentError, AgentErrorCode},
    observation::{
        AgentBattlePhase, AgentBattleStatus, AgentBattleView, AgentEffectCategory, AgentEffectView,
        AgentLifeState, AgentPresenceState, AgentTeamSide, AgentTeamView, AgentTimelineView,
        AgentUnitView, AgentWaveView, VisibilityPolicy,
    },
    schema::{
        ActionToken, AgentHash, AgentSInt, AgentSchemaRevision, AgentUInt, AgentValueError,
        EventCursor, IdempotencyKey, ScenarioId, SessionId,
    },
};

#[test]
fn revisions_ids_hashes_and_exact_integers_reject_noncanonical_values() {
    assert_eq!(
        AgentSchemaRevision::from_str("agent-api-v1"),
        Ok(AgentSchemaRevision::V1)
    );
    assert_eq!(
        AgentSchemaRevision::from_str("agent-api-v2"),
        Err(AgentValueError::UnknownRevision)
    );
    for invalid in ["", "00", "01", "+1", "-1", "18446744073709551616"] {
        assert!(AgentUInt::parse(invalid).is_err(), "accepted {invalid}");
    }
    for invalid in ["", "-0", "00", "+1", "01", "9223372036854775808"] {
        assert!(AgentSInt::parse(invalid).is_err(), "accepted {invalid}");
    }
    assert_eq!(
        AgentUInt::from_u64(u64::MAX).as_str(),
        "18446744073709551615"
    );
    assert_eq!(
        AgentSInt::from_i64(i64::MIN).as_str(),
        "-9223372036854775808"
    );
    assert_eq!(AgentHash::from_bytes([0xab; 32]).as_str(), "ab".repeat(32));
    assert!(AgentHash::parse(&"AB".repeat(32)).is_err());
    assert!(ScenarioId::parse("scenario.standard-v1.basic-single-wave").is_ok());
    assert!(ScenarioId::parse("scenario.challenge-v1.not-allowed").is_err());
    assert!(SessionId::parse("session_01-safe").is_ok());
    assert!(SessionId::parse("session/unsafe").is_err());
    assert!(EventCursor::parse("event_1").is_ok());
    assert!(IdempotencyKey::parse("retry_1").is_ok());
    assert!(serde_json::from_str::<AgentUInt>(r#""01""#).is_err());
    assert!(serde_json::from_str::<AgentSInt>(r#""-0""#).is_err());
    assert!(serde_json::from_str::<SessionId>(r#""session/unsafe""#).is_err());
}

#[test]
fn opaque_secrets_serialize_exactly_but_debug_is_redacted() {
    let token = ActionToken::parse("action_secret_01").unwrap();
    assert_eq!(
        serde_json::to_string(&token).unwrap(),
        r#""action_secret_01""#
    );
    assert_eq!(format!("{token:?}"), "ActionToken([redacted])");
    let session = SessionId::parse("session_secret_01").unwrap();
    assert_eq!(format!("{session:?}"), "SessionId([redacted])");
}

#[test]
fn owned_views_have_stable_field_order_and_string_only_authoritative_numbers() {
    let view = AgentBattleView {
        phase: AgentBattlePhase::AwaitingCommand,
        committed_revision: AgentUInt::from_u64(7),
        rng_draw_count: AgentUInt::from_u64(3),
        wave: AgentWaveView {
            number: AgentUInt::from_u64(1),
            total: AgentUInt::from_u64(2),
        },
        teams: vec![AgentTeamView {
            side: AgentTeamSide::Player,
            skill_points: AgentUInt::from_u64(3),
            maximum_skill_points: AgentUInt::from_u64(5),
        }]
        .into_boxed_slice(),
        units: vec![AgentUnitView {
            unit_id: AgentUInt::from_u64(1),
            side: AgentTeamSide::Player,
            formation: AgentUInt::from_u64(0),
            life: AgentLifeState::Alive,
            presence: AgentPresenceState::Present,
            current_hp: AgentUInt::from_u64(999),
            maximum_hp: AgentUInt::from_u64(1000),
            current_energy_scaled: AgentSInt::from_i64(50_000_000),
            maximum_energy_scaled: AgentSInt::from_i64(120_000_000),
            weakness_broken: false,
            public_intent: None,
        }]
        .into_boxed_slice(),
        effects: vec![AgentEffectView {
            effect_id: AgentUInt::from_u64(2),
            target_unit_id: AgentUInt::from_u64(1),
            category: AgentEffectCategory::Buff,
            stacks: AgentUInt::from_u64(1),
            remaining: None,
        }]
        .into_boxed_slice(),
        timeline: vec![AgentTimelineView {
            actor_id: AgentUInt::from_u64(1),
            owner_unit_id: AgentUInt::from_u64(1),
            active: true,
            action_gauge_scaled: AgentSInt::from_i64(0),
            speed_scaled: AgentSInt::from_i64(100_000_000),
        }]
        .into_boxed_slice(),
    };
    let json = serde_json::to_string(&view).unwrap();
    assert!(json.starts_with(
        r#"{"phase":"awaiting_command","committed_revision":"7","rng_draw_count":"3""#
    ));
    assert!(json.contains(r#""current_hp":"999""#));
    assert!(!json.contains(":999"));
    assert_eq!(
        VisibilityPolicy::PlayerVisible,
        VisibilityPolicy::PlayerVisible
    );
    assert_eq!(
        AgentBattleStatus::AwaitingPlayer,
        AgentBattleStatus::AwaitingPlayer
    );
}

#[test]
fn errors_are_stable_bounded_and_context_order_is_canonical() {
    let mut error =
        AgentError::new(AgentErrorCode::StaleDecision, "stale decision", true, false).unwrap();
    error.insert_detail("z", "last").unwrap();
    error.insert_detail("a", "first").unwrap();
    let json = serde_json::to_string(&error).unwrap();
    assert!(json.contains(r#""details":{"a":"first","z":"last"}"#));
    assert_eq!(error.to_string(), "agent request failed: StaleDecision");
    assert!(AgentError::new(AgentErrorCode::InvalidRequest, "", false, false).is_err());
    assert!(
        AgentError::new(
            AgentErrorCode::InvalidRequest,
            "x".repeat(1025),
            false,
            false
        )
        .is_err()
    );
}
