use starclock_activity::{
    ActivityDecisionId, ActivityEdgeId, ActivityOptionId, ActivityTerminalOutcome, NodeId,
};

use crate::baseline_controller::ActivityScoreComponents;

use super::baseline_controller::{
    SwarmBaselineController, SwarmBaselineError, SwarmCommandFamily, SwarmOfferedAction,
    SwarmOfferedCommand,
};

fn id(raw: u64) -> ActivityOptionId {
    ActivityOptionId::new(raw).unwrap()
}

fn score(progress: i32) -> ActivityScoreComponents {
    ActivityScoreComponents::new(progress, 0, 0, 0, 0).unwrap()
}

fn action(family: SwarmCommandFamily, ordinal: u32) -> SwarmOfferedAction {
    let key = format!("swarm-disaster.baseline.{ordinal}").into_boxed_str();
    match family {
        SwarmCommandFamily::Route => SwarmOfferedAction::Traverse {
            edge: ActivityEdgeId::new(ordinal).unwrap(),
            target: NodeId::new(ordinal).unwrap(),
        },
        SwarmCommandFamily::BossSelection => SwarmOfferedAction::SelectBoss {
            plane: 1,
            boss: key,
        },
        SwarmCommandFamily::DiceControl => SwarmOfferedAction::SelectDiceControl { control: key },
        SwarmCommandFamily::DiceTarget => SwarmOfferedAction::SelectDiceTarget {
            target: NodeId::new(ordinal),
        },
        SwarmCommandFamily::Countdown => SwarmOfferedAction::AdjustCountdown {
            delta: -i64::from(ordinal),
        },
        SwarmCommandFamily::Communing => SwarmOfferedAction::SelectCommuning { choice: key },
        SwarmCommandFamily::Progression => {
            SwarmOfferedAction::SelectProgression { objective: key }
        }
        SwarmCommandFamily::Reward => SwarmOfferedAction::SelectReward {
            source: "swarm-disaster.reward.fixture".into(),
            selection: key,
        },
        SwarmCommandFamily::Service => SwarmOfferedAction::PurchaseService { service: key },
        SwarmCommandFamily::AdventureOutcome => SwarmOfferedAction::SubmitAdventureOutcome {
            adventure: key,
            achieved: ordinal,
        },
    }
}

#[test]
fn every_swarm_family_selects_only_an_exact_offered_command() {
    let identity = super::SwarmDisasterControllerIdentity::baseline();
    assert_eq!(
        hex(identity.digest),
        "ad2a905b94942b0be7d6cd93293a976c14a859dc553cefa34d98d49d6de4d817"
    );
    let controller = SwarmBaselineController::default();
    let families = [
        SwarmCommandFamily::Route,
        SwarmCommandFamily::BossSelection,
        SwarmCommandFamily::DiceControl,
        SwarmCommandFamily::DiceTarget,
        SwarmCommandFamily::Countdown,
        SwarmCommandFamily::Communing,
        SwarmCommandFamily::Progression,
        SwarmCommandFamily::Reward,
        SwarmCommandFamily::Service,
        SwarmCommandFamily::AdventureOutcome,
    ];
    for (index, family) in families.into_iter().enumerate() {
        let low = SwarmOfferedCommand::new(id(1), 0, score(0), action(family, 1));
        let high = SwarmOfferedCommand::new(id(2), 0, score(1), action(family, 2));
        let offers = [low, high.clone()];
        let selected = controller
            .decide(
                ActivityDecisionId::new(u64::try_from(index + 1).unwrap()).unwrap(),
                &offers,
            )
            .unwrap();
        assert_eq!(selected.selected(), &high);
        assert!(offers.contains(selected.selected()));
    }
}

#[test]
fn ordering_is_inert_and_malformed_offer_sets_fail_closed() {
    let controller = SwarmBaselineController::default();
    let decision = ActivityDecisionId::new(1).unwrap();
    let first = SwarmOfferedCommand::new(
        id(2),
        0,
        score(0),
        action(SwarmCommandFamily::Route, 2),
    );
    let second = SwarmOfferedCommand::new(
        id(1),
        0,
        score(0),
        action(SwarmCommandFamily::Route, 1),
    );
    let left = controller
        .decide(decision, &[first.clone(), second.clone()])
        .unwrap();
    let right = controller
        .decide(decision, &[second.clone(), first.clone()])
        .unwrap();
    assert_eq!(left, right);
    assert_eq!(left.selected(), &second);
    assert_eq!(
        controller.decide(decision, &[]).unwrap_err(),
        SwarmBaselineError::EmptyOffer
    );
    assert_eq!(
        controller
            .decide(
                decision,
                &[
                    first,
                    SwarmOfferedCommand::new(
                        id(3),
                        0,
                        score(0),
                        action(SwarmCommandFamily::Reward, 3),
                    ),
                ],
            )
            .unwrap_err(),
        SwarmBaselineError::MixedFamilies
    );
    assert!(matches!(
        controller.decide(decision, &[second.clone(), second]),
        Err(SwarmBaselineError::Hints(_))
    ));
}

#[test]
fn baseline_completes_a_real_seeded_run_through_route_and_boss_offers() {
    let (instance, roster) = super::seeded_run_tests::representative_runtime();
    let report = instance
        .execute_baseline_run(
            super::seeded_run_tests::representative_request(),
            &roster,
        )
        .unwrap();
    assert_eq!(report.run().terminal, ActivityTerminalOutcome::Completed);
    assert_eq!(report.run().battle_count, 12);
    assert_eq!(report.decisions().len(), 27);
    assert!(report.decisions().iter().all(|decision| matches!(
        decision.selected().family(),
        SwarmCommandFamily::Route | SwarmCommandFamily::BossSelection
    )));
    assert_eq!(
        report
            .decisions()
            .iter()
            .filter(|decision| decision.selected().family() == SwarmCommandFamily::Route)
            .count(),
        24
    );
    assert_eq!(
        report
            .decisions()
            .iter()
            .filter(|decision| decision.selected().family() == SwarmCommandFamily::BossSelection)
            .count(),
        3
    );
    assert_eq!(
        hex(report.run().final_state_hash.bytes()),
        "caabb6cdfdcc827b592ce1fe3576600d84e06c0a68adbafb1bf663bd83b820db"
    );
    assert_eq!(
        hex(report.run().transcript_digest),
        "062c9917ef543c21bb8f5ec18a708b1c7f638cc7ef7e4297d6239844b511f3f8"
    );
    assert_eq!(
        hex(report.decision_digest()),
        "1eee69825daca55a94b89db9004379d56139a23f9960d1a1ee6e1a1035e7735c"
    );
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
