use starclock_agent_api::{
    action::AgentActionKind,
    observation::{AgentBattlePhase, AgentBattleStatus, VisibilityPolicy},
    schema::{AgentSchemaRevision, AgentUInt, EventCursor, IdempotencyKey, ScenarioId, SessionId},
    session::{AgentSeedPolicy, AgentSessionFactory, CreateSessionRequest, PlayActionRequest},
};
use starclock_data::standard_v1::SCENARIOS;

const EXPECTED_FINAL_HASHES: [&str; 6] = [
    "1720715d5d784dc533905aa1fb74b8633b85df2cd039e7254e7a010d5b75b475",
    "0e12ee3c7ffca5b5815444bb4cbbe1b1adbcbdabf5de09c2feda2e0f08bde17c",
    "5f53e4b9e9586761e1563787c67a58d5cf871346396ecc8898295785e8cff588",
    "d3e28d8c884ee073bcad8e97dc2e92b846d96b12af7e3017228a156e2ae5961b",
    "0725ca202c4b14a6a8831c5f78b0ee50116944f8e11def996924934c70c904a6",
    "12c2649faccc4c0cfe084cbe5deea68780490b169f0df0a412b9f34192aedb66",
];
const EXPECTED_EXTERNAL_STEPS: [u64; 6] = [8, 2, 6, 2, 22, 22];
const EXPECTED_REPLAY_COMMANDS: [usize; 6] = [9, 3, 7, 3, 23, 23];

#[test]
fn every_frozen_standard_scenario_finishes_through_agent_values_only() {
    let factory = AgentSessionFactory::load_production().unwrap();
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
        assert_eq!(state_hashes.len(), export.diagnostics().len());
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
