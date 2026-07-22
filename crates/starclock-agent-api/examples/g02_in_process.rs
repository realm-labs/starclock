//! Minimal protocol-neutral battle loop. No MCP runtime or network is involved.

use starclock_agent_api::{
    action::AgentActionKind,
    observation::{AgentBattleStatus, VisibilityPolicy},
    schema::{AgentSchemaRevision, EventCursor, IdempotencyKey, SessionId},
    session::{AgentSeedPolicy, AgentSessionFactory, CreateSessionRequest, PlayActionRequest},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let factory = AgentSessionFactory::load_production()?;
    let scenario = factory
        .list_scenarios()?
        .into_vec()
        .into_iter()
        .next()
        .ok_or("the frozen catalog has no Standard scenario")?;
    let session_id = SessionId::parse("session_example_in_process")?;
    let seed = AgentSeedPolicy::ScenarioDefault;
    let mut session = factory.create(CreateSessionRequest {
        session_id: session_id.clone(),
        scenario_id: scenario.scenario_id.clone(),
        seed: seed.clone(),
        visibility_policy: VisibilityPolicy::PlayerVisible,
    })?;
    let mut observation = session.observe(&EventCursor::parse("event_0")?)?;
    let mut step = 0_u64;

    while observation.status == AgentBattleStatus::AwaitingPlayer {
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
            .ok_or("the current decision has no supported example action")?;
        let response = session.apply_action(PlayActionRequest {
            schema_revision: AgentSchemaRevision::V1,
            session_id: session_id.clone(),
            decision_id: observation
                .decision_id
                .clone()
                .ok_or("awaiting-player observation has no decision")?,
            expected_state_hash: observation.state_hash.clone(),
            action_token: action.token.clone(),
            idempotency_key: IdempotencyKey::parse(&format!("example_step_{step}"))?,
        })?;
        observation = response.observation;
        step += 1;
    }

    if observation.status != AgentBattleStatus::Won {
        return Err("the example battle did not reach the frozen winning outcome".into());
    }
    let replay = session.export_replay()?;
    let verification = factory.verify_replay(&scenario.scenario_id, &seed, replay.bytes())?;
    if verification.final_state_hash != observation.state_hash {
        return Err("fresh replay verification returned another final hash".into());
    }
    println!(
        "scenario={} external_steps={} replay_commands={} final_hash={}",
        scenario.scenario_id.as_str(),
        step,
        verification.command_count.as_str(),
        verification.final_state_hash.as_str()
    );
    Ok(())
}
