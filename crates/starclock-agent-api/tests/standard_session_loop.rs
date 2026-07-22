use serde_json::Value;
use starclock_agent_api::{
    action::AgentActionKind,
    observation::{AgentBattlePhase, AgentBattleStatus, VisibilityPolicy},
    schema::{AgentSchemaRevision, AgentUInt, EventCursor, IdempotencyKey, ScenarioId, SessionId},
    session::{AgentSeedPolicy, AgentSessionFactory, CreateSessionRequest, PlayActionRequest},
};
use starclock_data::standard_v1::SCENARIOS;

const EXPECTED_FINAL_HASHES: [&str; 6] = [
    "5021cdd6019e0a100ad35e36ffb69fdb4860600db472c77fb8b33a9571b507ec",
    "87d2523332871b19cf4773373d031c6473bac29a48d17e796e0584cda296b344",
    "c6c1a62d408e6c31f45624440802e64d79cbc359faf9ffb58b62b25be3879603",
    "d3459759678910e92a719341a837a2ceca24a05bc1f5abbfa2190556e21e9c06",
    "c89ee783c91ce046d6b3b07ee0b29376417dc34ccc2f6935510bab180254a588",
    "413356b9d452876c51b269e62703072eef916ef866fd1420cfd7164e7383356b",
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
        if index == 0 {
            let frozen: Value = serde_json::from_str(include_str!(
                "../../../evidence/agent-control-mcp-v1/protocol/basic-transport-trace.json"
            ))
            .unwrap();
            assert_eq!(
                serde_json::to_value(&state_hashes).unwrap(),
                frozen["state_hashes"]
            );
            assert_eq!(hex(export.bytes()), frozen["replay_hex"].as_str().unwrap());
        }
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

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}
