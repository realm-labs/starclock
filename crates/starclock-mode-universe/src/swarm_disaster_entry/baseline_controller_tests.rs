use starclock_activity::{
    ActivityDecisionId, ActivityEdgeId, ActivityOptionId, ActivityTerminalOutcome, NodeId,
};

use crate::baseline_controller::ActivityScoreComponents;

use super::baseline_controller::{
    SWARM_DISASTER_BASELINE_CONTROLLER_REVISION, SwarmBaselineController, SwarmBaselineError,
    SwarmCommandFamily, SwarmOfferedAction, SwarmOfferedCommand,
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
    assert_eq!(
        SWARM_DISASTER_BASELINE_CONTROLLER_REVISION,
        "swarm-disaster-baseline-controller-v1"
    );
    let identity = super::SwarmDisasterControllerIdentity::baseline();
    assert_eq!(identity.revision, SWARM_DISASTER_BASELINE_CONTROLLER_REVISION);
    assert_eq!(
        hex(identity.digest),
        "0fb602397c52be5020b1053f1df9f610adeb3594007db3ab2813e3c41c42e618"
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
        "059710ea6ac74f7ae919a5f066b17fed91e13b249621eaba30e876126a207c11"
    );
    assert_eq!(
        hex(report.run().transcript_digest),
        "6cffe30e7476f330d63569264aaa22a6fe035e73a65658d8b683ded26aa3e703"
    );
    assert_eq!(
        hex(report.decision_digest()),
        "1bd51006cb09262b557177a69f5a74937eb1f5dfef191846e36c2bbed464b45f"
    );
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
