use starclock_agent_api::{
    action::AgentActionKind,
    observation::{AgentBattlePhase, AgentBattleStatus, VisibilityPolicy},
    schema::{AgentSchemaRevision, AgentUInt, EventCursor, IdempotencyKey, ScenarioId, SessionId},
    session::{AgentSeedPolicy, CreateSessionRequest, PlayActionRequest},
};
use starclock_data::standard_v1::SCENARIOS;

const EXPECTED_FINAL_HASHES: [&str; 6] = [
    "eb95d3eba8dbb2cd53258e5e174bbb8f6e744c557d4693a65951c4876d7b6178",
    "06d18777392f029c67f895fcff0dc6cbc32633cf562b3614a67425e8b02fd512",
    "328b29f00aaa8c06a679bc3fc59f9ea7e5587807ef2fd1a2ed4295c796f41227",
    "2cab1f6a0d3b7763cf59c6ce8b2b05ecf4c21e1d2d212275af693d13c7f36260",
    "91f8a8aafe068125c7e57100b159f763dd2fa6366e31ee245c45238bbb35ff90",
    "6da9c86593dbe3e2d004071108ba2f2ac959dc5a0d413a70fb9ce1faef05463b",
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
