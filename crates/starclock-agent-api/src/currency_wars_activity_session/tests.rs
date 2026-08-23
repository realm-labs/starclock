use super::*;

use crate::{
    activity_action::AgentActivityActionKind,
    activity_observation::{AgentActivityDecisionKind, AgentActivityStatus},
    schema::ActionToken,
};

fn create_request(
    session: &str,
    gambit: AgentCurrencyWarsGambit,
) -> CreateCurrencyWarsActivitySessionRequest {
    CreateCurrencyWarsActivitySessionRequest {
        session_id: SessionId::parse(session).unwrap(),
        route_id: AgentUInt::from_u64(801),
        difficulty_id: AgentUInt::from_u64(1),
        gambit,
        seed: AgentUInt::from_u64(31_000_501),
    }
}

fn play_request(
    observation: &AgentActivityObservation,
    token: ActionToken,
    key: &str,
) -> PlayActivityActionRequest {
    PlayActivityActionRequest {
        session_id: observation.session_id.clone(),
        boundary_id: observation.boundary_id.clone().unwrap(),
        expected_state_hash: observation.state_hash.clone(),
        action_token: token,
        idempotency_key: IdempotencyKey::parse(key).unwrap(),
    }
}

#[test]
fn manifest_is_bounded_summary_without_generated_rows() {
    let manifest = production_factory_for_tests().manifest();
    assert_eq!(manifest.game_version.as_ref(), "4.4");
    assert_eq!(manifest.routes.len(), 26);
    assert_eq!(manifest.difficulties.len(), 97);
    assert_eq!(manifest.fixture_role_ids.len(), 4);
    assert_eq!(manifest.gambits.len(), 2);

    let json = serde_json::to_string(&manifest).unwrap();
    assert!(json.len() < 65_536);
    for private_field in [
        "attributes_json",
        "parameter_ids",
        "enemy_scaling_refs",
        "mechanic_programs",
        "PostfixBase64",
    ] {
        assert!(!json.contains(private_field), "leaked {private_field}");
    }
}

#[test]
fn opaque_actions_cross_preparation_and_settle_one_real_battle() {
    let factory = production_factory_for_tests();
    let mut session = factory
        .create(create_request(
            "currency_wars_agent_standard",
            AgentCurrencyWarsGambit::Standard,
        ))
        .unwrap();
    let initial = session.observe().unwrap();
    assert_eq!(
        initial.decision_kind,
        Some(AgentActivityDecisionKind::Encounter)
    );
    assert_eq!(initial.legal_actions.len(), 1);
    assert_eq!(
        initial.legal_actions[0].kind,
        AgentActivityActionKind::EngageEncounter
    );

    let engage = play_request(
        &initial,
        initial.legal_actions[0].token.clone(),
        "currency_wars_engage_1",
    );
    let prepared = session.apply_action(engage).unwrap();
    assert_eq!(prepared.settlement.nested_battles.to_u64(), 0);
    assert_eq!(
        prepared.observation.decision_kind,
        Some(AgentActivityDecisionKind::Preparation)
    );
    assert_eq!(prepared.observation.legal_actions.len(), 1);
    assert_eq!(
        prepared.observation.legal_actions[0].kind,
        AgentActivityActionKind::EngageBattle
    );

    let stable_hash = session.state_hash();
    let stale = play_request(
        &initial,
        initial.legal_actions[0].token.clone(),
        "currency_wars_stale_1",
    );
    assert_eq!(
        session.apply_action(stale).unwrap_err().code,
        AgentErrorCode::StaleDecision
    );
    assert_eq!(session.state_hash(), stable_hash);

    let battle = play_request(
        &prepared.observation,
        prepared.observation.legal_actions[0].token.clone(),
        "currency_wars_battle_1",
    );
    let settled = session.apply_action(battle).unwrap();
    assert_eq!(settled.settlement.nested_battles.to_u64(), 1);
    assert_eq!(session.nested_battle_count(), 1);
    assert_eq!(session.accepted_action_count(), 2);
    assert!(!settled.observation.legal_actions.is_empty());

    let json = serde_json::to_string(&settled.observation).unwrap();
    assert!(!json.contains("debug_view"));
    assert!(!json.contains("combat_catalog"));
    assert!(!json.contains("attributes_json"));
}

#[test]
fn unknown_difficulty_is_rejected_before_session_creation() {
    let factory = production_factory_for_tests();
    let mut request = create_request(
        "currency_wars_agent_invalid",
        AgentCurrencyWarsGambit::Overclock,
    );
    request.difficulty_id = AgentUInt::from_u64(u64::MAX);
    let error = factory.create(request).err().expect("request is rejected");
    assert_eq!(error.code, AgentErrorCode::InvalidRequest);
}

#[test]
fn terminal_session_exports_the_shared_freshly_verified_replay() {
    let factory = production_factory_for_tests();
    let mut session = factory
        .create(create_request(
            "currency_wars_agent_replay",
            AgentCurrencyWarsGambit::Standard,
        ))
        .unwrap();
    assert_eq!(
        session.export_replay().unwrap_err().code,
        AgentErrorCode::ConfigurationRejected
    );

    for action_index in 0..32 {
        let observation = session.observe().unwrap();
        if observation.status != AgentActivityStatus::AwaitingAction {
            break;
        }
        let action = observation.legal_actions.first().unwrap();
        session
            .apply_action(play_request(
                &observation,
                action.token.clone(),
                &format!("currency_wars_replay_{action_index}"),
            ))
            .unwrap();
    }
    assert_eq!(
        session.observe().unwrap().status,
        AgentActivityStatus::Completed
    );
    let export = session.export_replay().expect("terminal replay exports");
    assert!(export.complete());
    assert_eq!(export.action_count().to_u64(), 14);
    let verification = session
        .verify_replay(&factory, export.bytes())
        .expect("fresh replay verifies");
    assert_eq!(verification.action_count.to_u64(), 14);
    assert_eq!(verification.nested_battles.to_u64(), 7);
}
