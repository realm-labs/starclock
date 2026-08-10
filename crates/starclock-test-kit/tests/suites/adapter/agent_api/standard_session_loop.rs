use starclock_agent_api::{
    action::AgentActionKind,
    observation::{AgentBattlePhase, AgentBattleStatus, VisibilityPolicy},
    schema::{AgentUInt, EventCursor, IdempotencyKey, ScenarioId, SessionId},
    session::{AgentSeedPolicy, CreateSessionRequest, PlayActionRequest},
};
use starclock_data::standard::SCENARIOS;

const EXPECTED_FINAL_HASHES: [&str; 6] = [
    "25c066d087f0c807c5454cdab688799697bbafaab8d92dd563f17716751d602c",
    "ca2d4bc3b9d0b5a0a6e32ee5bb27550180f8b3c3c76f932cc41c67b88583dca5",
    "f129ebd3ddea5877324533fa6390c4d5f65761c8de6fa84b29635a2fce57e7ed",
    "70f68ece3d724b47df613b6a28609da844128f26a8ebda78133a0852ec3ece20",
    "71a785f6a12a571b48df0fa9042edb3a3e00a621d4a86641c97882ec7a60a1ea",
    "97c06c924a87a5b28922cce75a7029c5927e70f7fe97ecfebd49b9a5d013d94a",
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
