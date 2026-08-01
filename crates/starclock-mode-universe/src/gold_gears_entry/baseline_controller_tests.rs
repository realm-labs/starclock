use starclock_activity::{
    ActivityDecisionId, ActivityEdgeId, ActivityInstanceId, ActivityOptionId,
    ActivityTerminalOutcome, NodeId,
};

use crate::baseline_controller::ActivityScoreComponents;

use super::{
    GoldAndGearsBaselineController, GoldAndGearsBaselineError, GoldAndGearsCommandFamily,
    GoldAndGearsOfferedAction, GoldAndGearsOfferedCommand, GoldAndGearsSeededRunRequest,
    baseline_controller::GOLD_AND_GEARS_BASELINE_CONTROLLER_REVISION,
    battle_materialization_tests::{activity_identity, seeded_matrix_roster},
};

fn id(raw: u64) -> ActivityOptionId {
    ActivityOptionId::new(raw).unwrap()
}

fn score(progress: i32) -> ActivityScoreComponents {
    ActivityScoreComponents::new(progress, 0, 0, 0, 0).unwrap()
}

fn action(family: GoldAndGearsCommandFamily, ordinal: u32) -> GoldAndGearsOfferedAction {
    let key = format!("gold-gears.baseline.{ordinal}").into_boxed_str();
    match family {
        GoldAndGearsCommandFamily::Route => GoldAndGearsOfferedAction::Traverse {
            edge: ActivityEdgeId::new(ordinal).unwrap(),
        },
        GoldAndGearsCommandFamily::BossSelection => GoldAndGearsOfferedAction::SelectBoss {
            plane: 1,
            boss: key,
        },
        GoldAndGearsCommandFamily::DiceLoadout => {
            GoldAndGearsOfferedAction::SelectDiceLoadout {
                custom_dice: key,
                faces: Vec::new().into_boxed_slice(),
            }
        }
        GoldAndGearsCommandFamily::DiceAction => GoldAndGearsOfferedAction::ActivateDice {
            face: key,
            targets: Vec::new().into_boxed_slice(),
        },
        GoldAndGearsCommandFamily::Cognition => {
            GoldAndGearsOfferedAction::AdjustCognition {
                delta: i64::from(ordinal),
            }
        }
        GoldAndGearsCommandFamily::Knowledge => {
            GoldAndGearsOfferedAction::ResolveKnowledge {
                rule: key,
                targets: vec![NodeId::new(ordinal).unwrap()].into_boxed_slice(),
            }
        }
        GoldAndGearsCommandFamily::Conundrum => {
            GoldAndGearsOfferedAction::SelectConundrum {
                stats: ordinal as u8,
                auxiliary: 0,
            }
        }
        GoldAndGearsCommandFamily::Reward => GoldAndGearsOfferedAction::SelectReward {
            source: "gold-gears.reward.fixture".into(),
            selection: key,
        },
        GoldAndGearsCommandFamily::Service => GoldAndGearsOfferedAction::PurchaseService {
            service: key,
        },
        GoldAndGearsCommandFamily::AdventureOutcome => {
            GoldAndGearsOfferedAction::SubmitAdventureOutcome {
                adventure: key,
                achieved: ordinal,
            }
        }
    }
}

#[test]
fn every_gold_family_selects_only_an_exact_offered_command() {
    assert_eq!(
        GOLD_AND_GEARS_BASELINE_CONTROLLER_REVISION,
        "gold-and-gears-baseline-controller-v1"
    );
    assert_eq!(
        hex(GoldAndGearsBaselineController::identity_digest()),
        "a84aea733d6e43bdc3528e20c2c99c79223add2874c9dea0db83e8bb21cbc420"
    );
    let controller = GoldAndGearsBaselineController::default();
    let families = [
        GoldAndGearsCommandFamily::Route,
        GoldAndGearsCommandFamily::BossSelection,
        GoldAndGearsCommandFamily::DiceLoadout,
        GoldAndGearsCommandFamily::DiceAction,
        GoldAndGearsCommandFamily::Cognition,
        GoldAndGearsCommandFamily::Knowledge,
        GoldAndGearsCommandFamily::Conundrum,
        GoldAndGearsCommandFamily::Reward,
        GoldAndGearsCommandFamily::Service,
        GoldAndGearsCommandFamily::AdventureOutcome,
    ];
    for (index, family) in families.into_iter().enumerate() {
        let low = GoldAndGearsOfferedCommand::new(id(1), 0, score(0), action(family, 1));
        let high = GoldAndGearsOfferedCommand::new(id(2), 0, score(1), action(family, 2));
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
    let controller = GoldAndGearsBaselineController::default();
    let decision = ActivityDecisionId::new(1).unwrap();
    let first = GoldAndGearsOfferedCommand::new(
        id(2),
        0,
        score(0),
        action(GoldAndGearsCommandFamily::Route, 2),
    );
    let second = GoldAndGearsOfferedCommand::new(
        id(1),
        0,
        score(0),
        action(GoldAndGearsCommandFamily::Route, 1),
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
        GoldAndGearsBaselineError::EmptyOffer
    );
    assert_eq!(
        controller
            .decide(
                decision,
                &[
                    first,
                    GoldAndGearsOfferedCommand::new(
                        id(3),
                        0,
                        score(0),
                        action(GoldAndGearsCommandFamily::Reward, 3),
                    ),
                ],
            )
            .unwrap_err(),
        GoldAndGearsBaselineError::MixedFamilies
    );
    assert!(matches!(
        controller.decide(decision, &[second.clone(), second]),
        Err(GoldAndGearsBaselineError::Hints(_))
    ));
}

#[test]
fn baseline_completes_a_real_seeded_run_through_route_and_boss_offers() {
    let factory = super::tests::shared_factory();
    let dice = &factory.unique.dice[0];
    let instance = factory
        .compile_entry(super::tests::battle_entry(
            factory,
            "gold-gears.area.401",
            "universe.path.abundance",
            dice,
        ))
        .unwrap();
    let roster = seeded_matrix_roster(&instance);
    let report = instance
        .execute_baseline_run(
            GoldAndGearsSeededRunRequest::new(
                14_001,
                activity_identity(),
                ActivityInstanceId::new(1).unwrap(),
            ),
            &roster,
        )
        .unwrap();
    assert_eq!(report.run().terminal(), ActivityTerminalOutcome::Completed);
    assert_eq!(report.run().battle_count(), 15);
    assert_eq!(report.decisions().len(), 42);
    assert!(report.decisions().iter().all(|decision| matches!(
        decision.selected().family(),
        GoldAndGearsCommandFamily::Route | GoldAndGearsCommandFamily::BossSelection
    )));
    assert_eq!(
        hex(report.run().final_state_hash().bytes()),
        "42e138d9362d55844fe18020434ed7d8609cea5e9f13e8522540be74b0088168"
    );
    assert_eq!(
        hex(report.run().transcript_digest()),
        "b27668a62d803800de9f38563f1bd9cbdc825538486126b2eead2e1ed807b854"
    );
    assert_eq!(
        hex(report.decision_digest()),
        "ca9a08325af92c40c07489a79416fb042906c58c9ce8d0a4f8f4594079e767b8"
    );
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
