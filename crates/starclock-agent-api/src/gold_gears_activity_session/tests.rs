use sha2::{Digest, Sha256};

use super::*;
use crate::{
    activity_observation::AgentActivityStatus,
    error::AgentErrorCode,
    schema::{IdempotencyKey, SessionId},
};

const SEED: u64 = 14_001;
const FINAL_STATE: &str = "fe3c463ffeb94dabbb93d8d7347d53683573e0d3bd966b97df66c60d4c6fd1d7";
const COMPONENT_ROOT: &str = "6d0153750e5bcecbfc06aff754cd5d9df81b42b37bbda108daa3290c24d81391";
const REPLAY_BYTES: usize = 107_338;
const REPLAY_SHA256: &str = "da7887876921a80334c5f21742e9d7748cf04c4d7959d1ac8ed3d14a7b7836e1";

#[test]
fn manifest_and_first_observation_are_bounded_and_mode_explicit() {
    let factory = GoldAndGearsActivityAgentSessionFactory::load_production().expect("factory");
    let manifest = factory.manifest();
    assert_eq!(manifest.profile_id.as_ref(), PROFILE);
    assert_eq!(
        manifest.fixture_accuracy.as_ref(),
        GOLD_AND_GEARS_BASELINE_FIXTURE_ACCURACY
    );
    assert_eq!(manifest.area.to_u64(), u64::from(AREA));
    assert_eq!(manifest.path.as_ref(), "universe.path.abundance");
    assert_eq!(manifest.custom_dice.as_ref(), "gold-gears.custom-dice.101");
    assert_eq!(manifest.component_root.as_str(), COMPONENT_ROOT);

    let session = factory
        .create(create_request("gold_manifest"))
        .expect("session");
    let observation = session.observe().expect("observation");
    assert_eq!(observation.profile_id.as_ref(), PROFILE);
    assert_eq!(observation.status, AgentActivityStatus::AwaitingAction);
    assert_eq!(
        observation.decision_kind,
        Some(AgentActivityDecisionKind::Encounter)
    );
    assert!(!observation.legal_actions.is_empty());
    assert!(observation.legal_actions.len() <= MAX_OFFERED_ACTIVITY_ACTIONS);
    assert!(observation.participants.len() <= 4);
}

#[test]
fn forged_and_stale_actions_preserve_the_authoritative_boundary() {
    let factory = GoldAndGearsActivityAgentSessionFactory::load_production().expect("factory");
    let mut session = factory
        .create(create_request("gold_reject"))
        .expect("session");
    let before = session.observe().expect("observation");
    let state = session.state_hash();
    let actions = session.replay_action_count();
    let offered = before.legal_actions[0].clone();

    let forged = session
        .apply_action(PlayActivityActionRequest {
            schema_revision: AgentSchemaRevision::V1,
            session_id: session.session_id().clone(),
            boundary_id: before.boundary_id.clone().expect("boundary"),
            expected_state_hash: before.state_hash.clone(),
            action_token: ActionToken::parse("u_forged").expect("token"),
            idempotency_key: IdempotencyKey::parse("gold_forged").expect("key"),
        })
        .expect_err("forged action");
    assert_eq!(forged.code, AgentErrorCode::InvalidActionToken);
    assert!(!forged.committed);
    assert_eq!(session.state_hash(), state);
    assert_eq!(session.replay_action_count(), actions);
    assert_eq!(session.observe().expect("unchanged observation"), before);

    let stale = session
        .apply_action(PlayActivityActionRequest {
            schema_revision: AgentSchemaRevision::V1,
            session_id: session.session_id().clone(),
            boundary_id: AgentUInt::from_u64(before.boundary_id.expect("boundary").to_u64() + 1),
            expected_state_hash: before.state_hash,
            action_token: offered.token,
            idempotency_key: IdempotencyKey::parse("gold_stale").expect("key"),
        })
        .expect_err("stale action");
    assert_eq!(stale.code, AgentErrorCode::StaleDecision);
    assert!(!stale.committed);
    assert_eq!(session.state_hash(), state);
    assert_eq!(session.replay_action_count(), actions);
}

#[test]
fn public_offers_complete_real_battles_and_export_fresh_replay() {
    let factory = GoldAndGearsActivityAgentSessionFactory::load_production().expect("factory");
    let mut session = factory
        .create(create_request("gold_complete"))
        .expect("session");
    let mut external_actions = 0_u64;
    let mut accepted_actions = 0_u64;
    let mut nested_battles = 0_u64;
    let mut first_request = None;
    while session.terminal().is_none() {
        let observation = session.observe().expect("observation");
        let selected = select_public(&observation).clone();
        let request = PlayActivityActionRequest {
            schema_revision: AgentSchemaRevision::V1,
            session_id: session.session_id().clone(),
            boundary_id: observation.boundary_id.expect("boundary"),
            expected_state_hash: observation.state_hash,
            action_token: selected.token,
            idempotency_key: IdempotencyKey::parse(&format!("gold_action_{external_actions}"))
                .expect("key"),
        };
        let response = session
            .apply_action(request.clone())
            .expect("accepted action");
        if external_actions == 0 {
            assert_eq!(
                session.apply_action(request.clone()).expect("retry"),
                response
            );
            first_request = Some(request);
        }
        external_actions += 1;
        accepted_actions += response.settlement.accepted_activity_actions.to_u64();
        nested_battles += response.settlement.nested_battles.to_u64();
    }
    assert!(first_request.is_some());
    assert_eq!(external_actions, 42);
    assert_eq!(accepted_actions, 61);
    assert_eq!(session.replay_action_count(), 62);
    assert_eq!(nested_battles, 17);
    assert_eq!(session.state_hash().as_str(), FINAL_STATE);
    let terminal = session.observe().expect("terminal observation");
    assert_eq!(terminal.status, AgentActivityStatus::Completed);
    assert!(terminal.legal_actions.is_empty());

    let replay = session.export_replay().expect("replay");
    assert!(replay.complete());
    assert_eq!(replay.action_count().to_u64(), 62);
    assert_eq!(replay.bytes().len(), REPLAY_BYTES);
    assert_eq!(replay.sha256().as_str(), REPLAY_SHA256);
    assert_eq!(
        replay.sha256().as_str(),
        hex(Sha256::digest(replay.bytes()))
    );
    let verification = session
        .verify_replay(&factory, replay.bytes())
        .expect("verification");
    assert_eq!(verification.action_count.to_u64(), 62);
    assert_eq!(verification.nested_battles.to_u64(), 17);
    assert_eq!(verification.final_state_hash.as_str(), FINAL_STATE);
    let live_hash = session.state_hash();
    let mut corrupted = replay.bytes().to_vec();
    *corrupted.last_mut().expect("replay byte") ^= 1;
    let error = factory
        .verify_replay(&AgentUInt::from_u64(SEED), &corrupted)
        .expect_err("corrupted replay");
    assert_eq!(error.code, AgentErrorCode::ReplayDiverged);
    assert_eq!(session.state_hash(), live_hash);
}

fn create_request(session: &str) -> CreateGoldAndGearsActivitySessionRequest {
    CreateGoldAndGearsActivitySessionRequest {
        session_id: SessionId::parse(session).expect("session ID"),
        seed: AgentUInt::from_u64(SEED),
    }
}

fn select_public(observation: &AgentActivityObservation) -> &OfferedActivityAction {
    observation
        .legal_actions
        .iter()
        .max_by(|left, right| {
            priority(left)
                .cmp(&priority(right))
                .then_with(|| right.option_id.to_u64().cmp(&left.option_id.to_u64()))
        })
        .expect("one exact offered action")
}

fn priority(action: &OfferedActivityAction) -> i64 {
    action
        .priority
        .as_ref()
        .map_or(0, |value| value.as_str().parse().expect("priority"))
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
