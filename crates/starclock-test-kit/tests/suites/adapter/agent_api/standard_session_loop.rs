use starclock_agent_api::{
    action::AgentActionKind,
    observation::{AgentBattlePhase, AgentBattleStatus, VisibilityPolicy},
    schema::{AgentUInt, EventCursor, IdempotencyKey, ScenarioId, SessionId},
    session::{AgentSeedPolicy, CreateSessionRequest, PlayActionRequest},
};
use starclock_data::standard::SCENARIOS;

const EXPECTED_FINAL_HASHES: [&str; 6] = [
    "c3a887357ed05ed76e51512f9813635cbd7bea223bde32ca10570b530ef44342",
    "19067843678c8a095cbc4f1f69c7a9a6270b847240606a1be9f6fe20620bb5c6",
    "558b60771387770b389c7645ea280198304dda200f75d578d9b525c3805def30",
    "f3655371526d94f22361b11723e5db60a902b61789bc0f37efb4be132182108b",
    "dd8c895e6e22af5c51fb56d315f383ab15948d03a1dac1656800a3cdb3a2d676",
    "4d2f3410f4d1b5021db5f7b2d7cd509dbae0344b528da739576204e8b496bc56",
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
