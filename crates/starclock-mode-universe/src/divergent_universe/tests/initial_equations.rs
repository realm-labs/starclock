//! Public starting choices bind authored policy to ordinary Activity and replay.

use super::{instance, reward_draws, slot_value};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseFlowInstance, DivergentUniverseOfferedSelection,
    encode_divergent_universe_replay, record_divergent_universe_transcript,
    state::{EQUATION_GRANT_DOMAIN_VISITS_SLOT, EQUATIONS_SLOT},
    verify_divergent_universe_replay,
};
use starclock_activity::{
    ActivityDecisionKind, ActivityMasterSeed, ActivityOperation, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityTerminalOutcome, ActivityValue,
    GraphActivity,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_equation_catalog::DivergentUniverseEquationCategory,
};

pub(super) fn accept_initial(flow: &DivergentUniverseFlowInstance, activity: &mut GraphActivity) {
    let view = activity.player_view();
    let decision = view.decision().unwrap();
    assert_eq!(decision.kind(), ActivityDecisionKind::Preparation);
    flow.choose_initial_equation(
        activity,
        view.state_hash(),
        decision.id(),
        decision.options()[0].id(),
    )
    .unwrap();
}

#[test]
fn every_public_initial_equation_choice_completes_a_fresh_verified_replay() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    let runner = DivergentUniverseBaselineRunner::default();
    let policy = fixture.policy().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        for ordinal in 0..3 {
            let seed = 23_802;
            let mut activity = flow
                .start(instance(1), ActivityMasterSeed::from_u64(seed))
                .unwrap()
                .into_activity();
            let view = activity.player_view();
            let decision = view.decision().unwrap();
            let selected = decision.options()[ordinal].id();
            let hidden = fixture
                .factory()
                .bundle
                .equation_catalog()
                .equations()
                .iter()
                .enumerate()
                .find(|(index, equation)| {
                    equation.category == DivergentUniverseEquationCategory::Epic
                        && decision
                            .options()
                            .iter()
                            .all(|option| option.id().get() != (*index as u64 + 1))
                })
                .map(|(index, _)| ActivityOptionId::new(index as u64 + 1).unwrap())
                .unwrap();
            let before = activity.canonical_state_bytes();
            assert!(
                flow.choose_initial_equation(
                    &mut activity,
                    view.state_hash(),
                    decision.id(),
                    hidden
                )
                .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            let mut steps = vec![
                runner
                    .advance_selected(
                        fixture.factory(),
                        &flow,
                        &mut activity,
                        fixture.core(),
                        &policy,
                        DivergentUniverseOfferedSelection::new(decision.id(), selected),
                    )
                    .unwrap(),
            ];
            assert!(
                matches!(slot_value(&activity.player_view(), EQUATIONS_SLOT), ActivityValue::OrderedIdSet(values) if values.as_ref() == [selected.get()])
            );
            while activity.player_view().terminal().is_none() {
                assert!(steps.len() < policy.max_steps() as usize);
                steps.push(
                    runner
                        .advance(
                            fixture.factory(),
                            &flow,
                            &mut activity,
                            fixture.core(),
                            &policy,
                        )
                        .unwrap(),
                );
            }
            let recorded =
                record_divergent_universe_transcript(&fixture, &flow, &activity, seed, steps)
                    .unwrap();
            let bytes = encode_divergent_universe_replay(&recorded).unwrap();
            let replay = verify_divergent_universe_replay(&bytes, &fresh).unwrap();
            assert_eq!(replay.action_count(), 10);
            assert_eq!(replay.terminal(), ActivityTerminalOutcome::Completed);
        }
    }
}

#[test]
fn public_initial_grant_retains_domain_budget_and_rolls_back_late_capacity_failure() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let equations = factory.equation_offer_runtime().unwrap();
    let wax = DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9187").unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        for capacity_failure in [false, true] {
            let mut activity = flow
                .start(instance(1), ActivityMasterSeed::from_u64(23_803))
                .unwrap()
                .into_activity();
            let hash = activity.state_hash();
            factory
                .curio_runtime()
                .unwrap()
                .acquire_accepted_state(&mut activity, hash, &wax)
                .unwrap();
            if capacity_failure {
                let program = ActivityProgramDefinition::new(
                    ActivityProgramId::new(23_803).unwrap(),
                    vec![ActivityOperation::SetCounterMap {
                        slot: EQUATION_GRANT_DOMAIN_VISITS_SLOT,
                        values: (1000..1512).map(|key| (key, 1)).collect(),
                    }],
                )
                .unwrap();
                activity
                    .apply_boundary_program(activity.state_hash(), &program)
                    .unwrap();
            }
            let view = activity.player_view();
            let decision = view.decision().unwrap();
            let before = activity.canonical_state_bytes();
            let draws = reward_draws(&activity);
            let result = flow.choose_initial_equation(
                &mut activity,
                view.state_hash(),
                decision.id(),
                decision.options()[0].id(),
            );
            if capacity_failure {
                assert!(result.is_err());
                assert_eq!(activity.canonical_state_bytes(), before);
                continue;
            }
            result.unwrap();
            assert!((1..=3).contains(&(reward_draws(&activity) - draws)));
            let receipt =
                slot_value(&activity.player_view(), EQUATION_GRANT_DOMAIN_VISITS_SLOT).clone();
            assert!(
                matches!(&receipt, ActivityValue::BoundedCounterMap(values) if values.len() == 1)
            );
            let hash = activity.state_hash();
            let offer = equations
                .begin_offer(&mut activity, hash, equations.offers()[0].id())
                .unwrap();
            let draws = reward_draws(&activity);
            let hash = activity.state_hash();
            equations
                .acquire(&mut activity, hash, &offer.candidates()[0])
                .unwrap();
            assert_eq!(reward_draws(&activity), draws);
            assert_eq!(
                slot_value(&activity.player_view(), EQUATION_GRANT_DOMAIN_VISITS_SLOT),
                &receipt
            );
        }
    }
}

#[test]
fn public_initial_choice_cannot_overwrite_an_internal_equation_offer() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let flow = fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap();
    let mut activity = flow
        .start(instance(1), ActivityMasterSeed::from_u64(23_804))
        .unwrap()
        .into_activity();
    let equations = fixture.factory().equation_offer_runtime().unwrap();
    let hash = activity.state_hash();
    equations
        .begin_offer(&mut activity, hash, equations.offers()[0].id())
        .unwrap();
    let view = activity.player_view();
    let decision = view.decision().unwrap();
    let before = activity.canonical_state_bytes();
    assert!(
        flow.choose_initial_equation(
            &mut activity,
            view.state_hash(),
            decision.id(),
            decision.options()[0].id()
        )
        .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn initial_equation_is_a_required_authored_public_choice_in_both_baselines() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        assert_eq!(
            flow.initial_equation_policy().unwrap().category,
            DivergentUniverseEquationCategory::Epic
        );
        let mut activity = flow
            .start(instance(1), ActivityMasterSeed::from_u64(23_801))
            .unwrap()
            .into_activity();
        let fresh = flow
            .start(instance(1), ActivityMasterSeed::from_u64(23_801))
            .unwrap()
            .into_activity();
        assert_eq!(
            activity.canonical_state_bytes(),
            fresh.canonical_state_bytes()
        );
        let view = activity.player_view();
        let decision = view.decision().unwrap();
        assert_eq!(decision.kind(), ActivityDecisionKind::Preparation);
        assert_eq!(decision.options().len(), 3);
        assert_eq!(reward_draws(&activity), 3);
        for option in decision.options() {
            let equation = &fixture.factory().bundle.equation_catalog().equations()
                [usize::try_from(option.id().get() - 1).unwrap()];
            assert_eq!(equation.category, DivergentUniverseEquationCategory::Epic);
        }
        let selected = decision.options()[0].id();
        let before = activity.canonical_state_bytes();
        assert!(
            activity
                .choose_option(view.state_hash(), decision.id(), selected)
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert!(
            flow.choose_initial_equation(
                &mut activity,
                view.state_hash(),
                decision.id(),
                ActivityOptionId::new(u64::MAX).unwrap()
            )
            .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        accept_initial(&flow, &mut activity);
        assert!(
            matches!(slot_value(&activity.player_view(), EQUATIONS_SLOT), ActivityValue::OrderedIdSet(values) if values.as_ref() == [selected.get()])
        );
        assert_eq!(
            activity.player_view().decision().unwrap().kind(),
            ActivityDecisionKind::Choice
        );
        let after = activity.canonical_state_bytes();
        assert!(
            flow.choose_initial_equation(&mut activity, view.state_hash(), decision.id(), selected)
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), after);
        let report = DivergentUniverseBaselineRunner::default()
            .run_to_terminal(
                fixture.factory(),
                &flow,
                &mut activity,
                fixture.core(),
                &fixture.policy().unwrap(),
            )
            .unwrap();
        assert_eq!(report.terminal(), ActivityTerminalOutcome::Completed);
        assert_eq!(report.completed_battles(), 3);
    }
}
