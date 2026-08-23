use starclock_agent_api::{
    action::AgentActionKind,
    observation::{AgentBattlePhase, AgentBattleStatus, VisibilityPolicy},
    schema::{AgentUInt, EventCursor, IdempotencyKey, ScenarioId, SessionId},
    session::{AgentSeedPolicy, CreateSessionRequest, PlayActionRequest},
};
use starclock_data::standard::SCENARIOS;

const EXPECTED_FINAL_HASHES: [&str; 6] = [
    "ba4a03c81869a030cc313fd95ae1e2431baa988a1a961006a84638fc162bd9a3",
    "1d58df2ea29bc8c14a96c08d2d7d6954dca792d8e5763ebf4cbbfec3eec3e058",
    "3d5b4dc4384223c5ba42a7a54ae3d1dd9ca31680f9d42fe9ecc80e96c520b45e",
    "372ae30284fe7ce4c6d9d4753bb493ce518704c84dff8e905cbc41bc40bb7280",
    "99010b69effab03b5b9337efa23d944c495e8d4bf7b7d4a88a6e01909a490a22",
    "80309d65d116e5ca41291c4af08bf8fa2e6fb8593526b65c36f918822a1b851f",
];
const EXPECTED_EXTERNAL_STEPS: [u64; 6] = [8, 2, 6, 3, 17, 22];
const EXPECTED_REPLAY_COMMANDS: [usize; 6] = [20, 4, 14, 6, 40, 54];

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
                        .find(|action| action.kind == AgentActionKind::CommitPreparedAction)
                })
                .or_else(|| {
                    observation
                        .legal_actions
                        .iter()
                        .find(|action| action.kind == AgentActionKind::Advance)
                })
                .expect("the frozen script always has a deterministic progress action");
            let response = session
                .apply_action(PlayActionRequest {
                    session_id: session_id.clone(),
                    boundary_id: observation.boundary_id.clone().unwrap(),
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
