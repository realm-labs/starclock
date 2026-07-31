use starclock_agent_api::{
    action::AgentActionKind,
    observation::{AgentBattlePhase, AgentBattleStatus, VisibilityPolicy},
    schema::{AgentSchemaRevision, AgentUInt, EventCursor, IdempotencyKey, ScenarioId, SessionId},
    session::{AgentSeedPolicy, CreateSessionRequest, PlayActionRequest},
};
use starclock_data::standard_v1::SCENARIOS;

const EXPECTED_FINAL_HASHES: [&str; 6] = [
    "71faf56504a7ffb1f5c54b0135c68939a5973fb6b9e065217c12ae4d0e5e5b9e",
    "9c14dace6f72cfc267277b35bf0096df6851e71df2ac6fc3755da5fdb0dd859a",
    "157610fab9cd6fe5f5f04a8ba7b66bf46d28449c83465dfd41f8ee2bc9df02a4",
    "194d97c4cc3a2b96985f9fee52ff31ae297eb061b586c0d78069caf2f7eea6d4",
    "b22a1455458206a91b9d9a995536fd09c0ffc687fcbc267556f2fffcfac19a06",
    "20133e32dd1f7c1a6f4e46d498847ec567ef22b9a1eda546a72424e223535c1e",
];
const EXPECTED_EXTERNAL_STEPS: [u64; 6] = [16, 4, 12, 6, 34, 44];
const EXPECTED_REPLAY_COMMANDS: [usize; 6] = [21, 5, 15, 7, 41, 55];

#[test]
fn every_frozen_standard_scenario_finishes_through_agent_values_only() {
    let factory = starclock_test_kit::agent_session_factory();
    for (index, (scenario, _, _)) in SCENARIOS.into_iter().enumerate() {
        let session_id = SessionId::parse(&format!("session_standard_{index}")).unwrap();
        let mut session = factory
            .create(CreateSessionRequest {
                session_id: session_id.clone(),
                scenario_id: ScenarioId::parse(scenario).unwrap(),
                seed: AgentSeedPolicy::ScenarioDefault,
                visibility_policy: VisibilityPolicy::PlayerVisible,
            })
            .unwrap();
        let mut observation = session
            .observe(&EventCursor::parse("event_0").unwrap())
            .unwrap();
        let mut state_hashes = vec![observation.state_hash.as_str().to_owned()];
        let mut external_steps = 0u64;
        while observation.status == AgentBattleStatus::AwaitingPlayer {
            assert!(external_steps < 512, "{scenario} exceeded the script bound");
            let action = observation
                .legal_actions
                .iter()
                .find(|action| action.kind == AgentActionKind::UseAbility)
                .or_else(|| {
                    observation
                        .legal_actions
                        .iter()
                        .find(|action| action.kind == AgentActionKind::PassInterrupt)
                })
                .expect("the frozen script always has an ability or interrupt pass");
            let response = session
                .apply_action(PlayActionRequest {
                    schema_revision: AgentSchemaRevision::V1,
                    session_id: session_id.clone(),
                    decision_id: observation.decision_id.clone().unwrap(),
                    expected_state_hash: observation.state_hash.clone(),
                    action_token: action.token.clone(),
                    idempotency_key: IdempotencyKey::parse(&format!(
                        "script_{index}_{external_steps}"
                    ))
                    .unwrap(),
                })
                .unwrap();
            assert!(response.committed);
            observation = response.observation;
            state_hashes.push(observation.state_hash.as_str().to_owned());
            external_steps += 1;
        }

        assert_eq!(observation.status, AgentBattleStatus::Won, "{scenario}");
        assert_eq!(
            observation.state_hash.as_str(),
            EXPECTED_FINAL_HASHES[index],
            "{scenario}"
        );
        assert_eq!(external_steps, EXPECTED_EXTERNAL_STEPS[index], "{scenario}");
        let export = session.export_replay().unwrap();
        assert_eq!(
            state_hashes.len(),
            usize::try_from(external_steps).unwrap() + 1
        );
        assert!(export.diagnostics().len() >= state_hashes.len());
        assert_eq!(
            export.diagnostics().len(),
            EXPECTED_REPLAY_COMMANDS[index],
            "{scenario}"
        );
        assert_eq!(export.diagnostics().len(), session.replay_command_count());
        let verification = session.verify_replay(export.bytes()).unwrap();
        assert_eq!(verification.phase, AgentBattlePhase::Won);
        assert_eq!(verification.final_state_hash, observation.state_hash);
        assert_eq!(
            verification.command_count,
            AgentUInt::from_u64(u64::try_from(export.diagnostics().len()).unwrap())
        );
    }
}
