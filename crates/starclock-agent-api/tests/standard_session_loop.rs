use starclock_agent_api::{
    action::AgentActionKind,
    observation::{AgentBattlePhase, AgentBattleStatus, VisibilityPolicy},
    schema::{AgentSchemaRevision, AgentUInt, EventCursor, IdempotencyKey, ScenarioId, SessionId},
    session::{AgentSeedPolicy, AgentSessionFactory, CreateSessionRequest, PlayActionRequest},
};
use starclock_data::standard_v1::SCENARIOS;

const EXPECTED_FINAL_HASHES: [&str; 6] = [
    "ef7b5d60ca5f5d76c4addfaeac087898ea6354e17c4054f5b4a0d2dce703d033",
    "9c14dace6f72cfc267277b35bf0096df6851e71df2ac6fc3755da5fdb0dd859a",
    "49561416dd657bf1ab1defd15c17ca6de16a787e4efe91f7b0e1538d3a397cbf",
    "194d97c4cc3a2b96985f9fee52ff31ae297eb061b586c0d78069caf2f7eea6d4",
    "93f4294a127823dd4f39a5146d7279bde746f4ca0776f274c2bdd28fa310ce85",
    "ba9597a7b64e2837ec6e2f48db2f6986ac6796a2e084283d9e7167d0542b750b",
];
const EXPECTED_EXTERNAL_STEPS: [u64; 6] = [8, 4, 6, 6, 22, 22];
const EXPECTED_REPLAY_COMMANDS: [usize; 6] = [9, 5, 7, 7, 23, 23];

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
