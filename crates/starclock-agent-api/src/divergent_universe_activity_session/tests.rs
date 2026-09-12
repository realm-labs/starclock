use super::*;
use crate::{
    activity_action::OfferedActivityAction,
    activity_observation::AgentActivityStatus,
    error::AgentErrorCode,
    schema::{ActionToken, IdempotencyKey},
};
use starclock_mode_universe::divergent_universe::{
    DivergentUniverseCurrencyCommand, DivergentUniverseCurrencyKind,
};

#[test]
fn manifest_and_observations_are_bounded_without_generated_rows() {
    let factory = production_factory_for_tests();
    let manifest = factory.manifest().expect("manifest");
    assert_eq!(manifest.mode_id.as_ref(), "divergent-universe");
    assert_eq!(manifest.families.len(), 2);
    assert_eq!(manifest.configuration_tables.to_u64(), 80);
    assert_eq!(manifest.source_obligations.to_u64(), 6_215);
    assert_eq!(manifest.mechanic_programs.to_u64(), 669);
    let encoded = serde_json::to_string(&manifest).expect("manifest JSON");
    assert!(!encoded.contains("generated_row"));
    assert!(!encoded.contains("private"));

    for family in [
        AgentDivergentUniverseRunFamily::Ordinary,
        AgentDivergentUniverseRunFamily::Cyclical,
    ] {
        let session = factory
            .create(request(&format!("du_manifest_{family:?}"), family, 22_101))
            .expect("session");
        let observation = session.observe().expect("observation");
        assert_eq!(observation.profile_id.as_ref(), family.profile());
        assert_eq!(observation.status, AgentActivityStatus::AwaitingAction);
        assert!(!observation.legal_actions.is_empty());
        assert!(observation.legal_actions.len() <= 256);
        assert!(observation.participants.len() <= 4);
    }
}

#[test]
fn forged_stale_and_cross_session_actions_preserve_state() {
    let factory = production_factory_for_tests();
    let mut session = factory
        .create(request(
            "du_rejections",
            AgentDivergentUniverseRunFamily::Ordinary,
            22_102,
        ))
        .expect("session");
    let before = session.observe().expect("observation");
    let state = session.state_hash();
    let offered = before.legal_actions[0].clone();

    let forged = session
        .apply_action(PlayActivityActionRequest {
            session_id: session.session_id().clone(),
            boundary_id: before.boundary_id.clone().expect("boundary"),
            expected_state_hash: before.state_hash.clone(),
            action_token: ActionToken::parse("u_forged").expect("token"),
            idempotency_key: IdempotencyKey::parse("du_forged").expect("key"),
        })
        .expect_err("forged action");
    assert_eq!(forged.code, AgentErrorCode::InvalidActionToken);
    assert!(!forged.committed);
    assert_eq!(session.state_hash(), state);
    assert_eq!(session.observe().expect("unchanged"), before);

    let stale = session
        .apply_action(PlayActivityActionRequest {
            session_id: session.session_id().clone(),
            boundary_id: AgentUInt::from_u64(before.boundary_id.expect("boundary").to_u64() + 1),
            expected_state_hash: before.state_hash,
            action_token: offered.token,
            idempotency_key: IdempotencyKey::parse("du_stale").expect("key"),
        })
        .expect_err("stale action");
    assert_eq!(stale.code, AgentErrorCode::StaleDecision);
    assert!(!stale.committed);
    assert_eq!(session.state_hash(), state);
}

#[test]
fn rejected_reward_keeps_public_offer_and_does_not_consume_idempotency_key() {
    let factory = production_factory_for_tests();
    for family in [
        AgentDivergentUniverseRunFamily::Ordinary,
        AgentDivergentUniverseRunFamily::Cyclical,
    ] {
        let mut session = factory
            .create(request("du_reward_rejection", family, 22_109))
            .unwrap();
        play_public_option(&mut session, 0, None);
        // Counterfactual balance isolates the late checked-arithmetic failure.
        // Setup still uses the domain's currency command, never a private slot.
        let currency = DivergentUniverseCurrencyKind::CosmicFragment;
        let rule = session.flow.economy().currency(currency).gain_rules()[0];
        let hash = session.activity.state_hash();
        session
            .flow
            .apply_currency_command(
                &mut session.activity,
                hash,
                DivergentUniverseCurrencyCommand::Credit {
                    currency,
                    rule,
                    amount: i64::MAX as u64,
                },
            )
            .unwrap();
        session.refresh_offer().unwrap();
        let before = session.observe().unwrap();
        let bytes = session.activity.canonical_state_bytes();
        let steps = session.replay_action_count();
        let cached = session.idempotency.len();
        let action = before
            .legal_actions
            .iter()
            .find(|action| action.option_id.to_u64() == 1)
            .unwrap();
        let mut request = PlayActivityActionRequest {
            session_id: session.session_id().clone(),
            boundary_id: before.boundary_id.clone().unwrap(),
            expected_state_hash: before.state_hash.clone(),
            action_token: action.token.clone(),
            idempotency_key: IdempotencyKey::parse("du_rejected_reward").unwrap(),
        };
        for _ in 0..2 {
            let error = session.apply_action(request.clone()).unwrap_err();
            assert_eq!(error.code, AgentErrorCode::CombatRejected);
            assert!(!error.committed);
            assert_eq!(session.activity.canonical_state_bytes(), bytes);
            assert_eq!(session.observe().unwrap(), before);
            assert_eq!(session.replay_action_count(), steps);
            assert_eq!(session.idempotency.len(), cached);
        }
        // A different legal reward can use the rejected request's key. Rejection
        // must not strand the decision or reserve the key for an uncommitted action.
        request.action_token = before
            .legal_actions
            .iter()
            .find(|action| action.option_id.to_u64() == 3)
            .unwrap()
            .token
            .clone();
        let response = session.apply_action(request.clone()).unwrap();
        assert!(response.committed);
        assert_eq!(session.replay_action_count(), steps + 1);
        assert_eq!(session.idempotency.len(), cached + 1);
        assert_eq!(session.apply_action(request).unwrap(), response);
    }
}

#[test]
fn public_offers_complete_both_families_and_export_fresh_replays() {
    let factory = production_factory_for_tests();
    for (family, seed, event_option) in [
        (AgentDivergentUniverseRunFamily::Ordinary, 22_103, 1),
        (AgentDivergentUniverseRunFamily::Cyclical, 22_104, 1),
        (AgentDivergentUniverseRunFamily::Ordinary, 22_105, 2),
        (AgentDivergentUniverseRunFamily::Cyclical, 22_106, 2),
        (AgentDivergentUniverseRunFamily::Ordinary, 22_107, 3),
        (AgentDivergentUniverseRunFamily::Cyclical, 22_108, 3),
    ] {
        let mut session = factory
            .create(request(&format!("du_complete_{family:?}"), family, seed))
            .expect("session");
        let mut action_index = 0_u64;
        let mut nested_battles = 0_u64;
        while session.terminal().is_none() {
            let observation = session.observe().expect("observation");
            let selected = observation
                .legal_actions
                .iter()
                .filter(|action| action_index != 1 || action.option_id.to_u64() == event_option)
                .max_by(|left, right| {
                    priority(left)
                        .cmp(&priority(right))
                        .then_with(|| right.option_id.to_u64().cmp(&left.option_id.to_u64()))
                })
                .expect("one offered action")
                .clone();
            let request = PlayActivityActionRequest {
                session_id: session.session_id().clone(),
                boundary_id: observation.boundary_id.expect("boundary"),
                expected_state_hash: observation.state_hash,
                action_token: selected.token,
                idempotency_key: IdempotencyKey::parse(&format!("du_action_{action_index}"))
                    .expect("key"),
            };
            let response = session.apply_action(request.clone()).expect("action");
            if action_index == 0 {
                assert_eq!(session.apply_action(request).expect("retry"), response);
            }
            nested_battles += response.settlement.nested_battles.to_u64();
            action_index += 1;
        }
        assert_eq!(action_index, 10);
        assert_eq!(nested_battles, 3);
        assert_eq!(session.replay_action_count(), 10);
        assert_eq!(
            session.observe().expect("terminal").status,
            AgentActivityStatus::Completed
        );

        let replay = session.export_replay().expect("replay");
        assert!(replay.complete());
        assert_eq!(replay.action_count().to_u64(), 10);
        let verification = session
            .verify_replay(&factory, replay.bytes())
            .expect("fresh verification");
        assert_eq!(verification.action_count.to_u64(), 10);
        assert_eq!(verification.nested_battles.to_u64(), 3);
        assert_eq!(verification.final_state_hash, session.state_hash());

        let other_family = match family {
            AgentDivergentUniverseRunFamily::Ordinary => AgentDivergentUniverseRunFamily::Cyclical,
            AgentDivergentUniverseRunFamily::Cyclical => AgentDivergentUniverseRunFamily::Ordinary,
        };
        let error = factory
            .verify_replay(&AgentUInt::from_u64(seed), other_family, replay.bytes())
            .expect_err("family mismatch rejected");
        assert_eq!(error.code, AgentErrorCode::ReplayDiverged);

        let mut corrupted = replay.bytes().to_vec();
        *corrupted.last_mut().expect("replay byte") ^= 1;
        let error = factory
            .verify_replay(&AgentUInt::from_u64(seed), family, &corrupted)
            .expect_err("corruption rejected");
        assert_eq!(error.code, AgentErrorCode::ReplayDiverged);
    }
}

fn request(
    session: &str,
    family: AgentDivergentUniverseRunFamily,
    seed: u64,
) -> CreateDivergentUniverseActivitySessionRequest {
    CreateDivergentUniverseActivitySessionRequest {
        session_id: SessionId::parse(session).expect("session ID"),
        family,
        seed: AgentUInt::from_u64(seed),
        tawot_forge_level: None,
    }
}

#[test]
fn tawot_configuration_public_cancel_purchase_and_export_reconstruct_fresh() {
    let factory = production_factory_for_tests();
    for family in [
        AgentDivergentUniverseRunFamily::Ordinary,
        AgentDivergentUniverseRunFamily::Cyclical,
    ] {
        for level in [2, 5] {
            let mut inputs = request("du_tawot", family, 24121);
            inputs.tawot_forge_level = Some(AgentUInt::from_u64(level));
            let mut session = factory.create(inputs).unwrap();
            for index in 0..3 {
                play_public_option(&mut session, index, None);
            }
            assert!(
                session
                    .flow
                    .offered_tawot_service(&session.activity)
                    .is_some()
            );
            play_public_option(&mut session, 3, Some(1));
            let offered = session
                .observe()
                .unwrap()
                .legal_actions
                .iter()
                .map(|action| action.option_id.clone())
                .collect::<Vec<_>>();
            play_public_option(&mut session, 4, Some(0x7e42_0001));
            play_public_option(&mut session, 5, Some(1));
            assert_eq!(
                session
                    .observe()
                    .unwrap()
                    .legal_actions
                    .iter()
                    .map(|action| action.option_id.clone())
                    .collect::<Vec<_>>(),
                offered
            );
            let mut actions = 6;
            while session.terminal().is_none() {
                assert!(actions < 24);
                play_public_option(&mut session, actions, None);
                actions += 1;
            }
            let replay = session.export_replay().unwrap();
            let fresh = DivergentUniverseActivityAgentSessionFactory::load_production().unwrap();
            let verified = session.verify_replay(&fresh, replay.bytes()).unwrap();
            assert_eq!(verified.action_count.to_u64(), actions);
            assert_eq!(verified.nested_battles.to_u64(), 3);
            assert_eq!(verified.final_state_hash, session.state_hash());
        }
        for level in [0, 1, 6, 65536, u64::MAX] {
            let mut inputs = request("du_tawot_invalid", family, 24121);
            inputs.tawot_forge_level = Some(AgentUInt::from_u64(level));
            assert_eq!(
                factory.create(inputs).err().unwrap().code,
                AgentErrorCode::InvalidRequest
            );
        }
    }
}

fn priority(action: &OfferedActivityAction) -> i64 {
    action
        .priority
        .as_ref()
        .map_or(0, |value| value.as_str().parse().expect("priority"))
}

#[test]
fn treasure_evolution_public_actions_are_idempotent_and_fresh_replay_verified() {
    let factory = production_factory_for_tests();
    for (target, other) in [("9195", "9192"), ("9192", "9195")] {
        let target = format!("divergent-universe.curio-state.{target}");
        let other = format!("divergent-universe.curio-state.{other}");
        for family in [
            AgentDivergentUniverseRunFamily::Ordinary,
            AgentDivergentUniverseRunFamily::Cyclical,
        ] {
            let mut selected = None;
            for seed in 0..2048 {
                let mut session = factory
                    .create(request("du_treasure_evolution", family, seed))
                    .unwrap();
                play_public_option(&mut session, 0, None);
                play_public_option(&mut session, 1, Some(2));
                let owned = factory
                    .fixture
                    .factory()
                    .curio_runtime()
                    .unwrap()
                    .owned(&session.activity)
                    .unwrap();
                if owned.iter().any(|owned| owned.state().as_str() == target)
                    && !owned.iter().any(|owned| {
                        owned.state().as_str() == "divergent-universe.curio-state.9055"
                            || owned.state().as_str() == other
                    })
                {
                    selected = Some(session);
                    break;
                }
            }
            let mut session =
                selected.expect("bounded public Curio acquisition reaches the selected Treasure");
            let mut actions = 2;
            let mut evolution_choices = 0;
            let mut domain_choices = 0;
            let mut battles = 0;
            let blessings = factory.fixture.factory().blessing_runtime().unwrap();
            while session.terminal().is_none() {
                assert!(actions < 12);
                let evolution = session
                    .flow
                    .offered_evolution_event(&session.activity)
                    .is_some();
                let domain = session
                    .flow
                    .offered_battle_domain_choices(&session.activity)
                    .unwrap()
                    .is_some();
                let battle = matches!(
                    session.flow.offered_encounter(&session.activity),
                    Ok(Some(_))
                );
                let before = blessings.owned(&session.activity).unwrap().len();
                let option = if domain {
                    let observed = session.observe().unwrap();
                    assert_eq!(
                        observed
                            .legal_actions
                            .iter()
                            .map(|action| action.label.as_ref())
                            .collect::<Vec<_>>(),
                        ["Combat Domain", "Elite Domain", "Aberration Domain"]
                    );
                    domain_choices += 1;
                    // Public ordinals 2/3 select Elite/Aberration, never a
                    // free-form override of the pending battle's domain.
                    Some(domain_choices + 1)
                } else {
                    evolution.then_some(1)
                };
                play_public_option(&mut session, actions, option);
                if battle {
                    let expected = if target.ends_with("9192") {
                        [0, 2, 3][battles]
                    } else {
                        0
                    };
                    assert_eq!(
                        blessings.owned(&session.activity).unwrap().len() - before,
                        expected
                    );
                    battles += 1;
                }
                evolution_choices += u32::from(evolution);
                actions += 1;
            }
            assert_eq!((actions, evolution_choices), (12, 2));
            assert_eq!((domain_choices, battles), (2, 3));
            let replay = session.export_replay().unwrap();
            let fresh = DivergentUniverseActivityAgentSessionFactory::load_production().unwrap();
            let verified = session.verify_replay(&fresh, replay.bytes()).unwrap();
            assert_eq!(verified.action_count.to_u64(), 12);
            assert_eq!(verified.nested_battles.to_u64(), 3);
            assert_eq!(verified.final_state_hash, session.state_hash());
        }
    }
}

fn play_public_option(
    session: &mut DivergentUniverseActivityAgentSession,
    index: u64,
    option: Option<u64>,
) {
    let observation = session.observe().unwrap();
    let action = observation
        .legal_actions
        .iter()
        .filter(|action| option.is_none_or(|option| action.option_id.to_u64() == option))
        .max_by(|left, right| {
            priority(left)
                .cmp(&priority(right))
                .then_with(|| right.option_id.to_u64().cmp(&left.option_id.to_u64()))
        })
        .unwrap();
    let request = PlayActivityActionRequest {
        session_id: session.session_id().clone(),
        boundary_id: observation.boundary_id.unwrap(),
        expected_state_hash: observation.state_hash,
        action_token: action.token.clone(),
        idempotency_key: IdempotencyKey::parse(&format!("green_{index}")).unwrap(),
    };
    let response = session.apply_action(request.clone()).unwrap();
    if session.terminal().is_none() {
        assert_eq!(session.apply_action(request).unwrap(), response);
    } else {
        // Settled sessions reject further actions before idempotency lookup;
        // both evolution choices were retried while the session was live.
        assert_eq!(
            session.apply_action(request).unwrap_err().code,
            AgentErrorCode::SessionClosed
        );
    }
}
