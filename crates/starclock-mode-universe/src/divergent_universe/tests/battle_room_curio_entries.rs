//! Actual fixed/selected source-position entries, not manual lifetime vectors.
//! Acquisitions below are controlled accepted producers; other room payloads
//! remain probes and no original Forge admission or encoded profile is claimed.

use std::sync::Arc;

use super::{
    CANCEL, Combined, FAMILIES, advance, build_combined, choose, currency_balance, payload, ready,
};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseCurioRuntime,
    domain_route::DomainRouteError,
    economy::DivergentUniverseCurrencyKind,
    state::{CURIO_ACTIVATIONS_SLOT, CURIO_CHARGES_SLOT, CURIO_STATES_SLOT, CURRENCIES_SLOT},
    tests::battle_room::{SLOTS, base, probe},
};
use starclock_activity::{
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityExpression, ActivityNodeDefinition,
    ActivityNodeKind, ActivityOperation, ActivityOptionId, ActivityProgramDefinition,
    ActivityProgramId, ActivityRandomPolicies, ActivityValue, GraphActivity,
    GraphActivityDefinition,
};
use starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioStateId;

fn state(raw: &str) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}
fn acquire(curios: &DivergentUniverseCurioRuntime, activity: &mut GraphActivity, raw: &str) {
    curios
        .acquire_accepted_state(activity, activity.state_hash(), &state(raw))
        .unwrap();
}
fn charges(
    curios: &DivergentUniverseCurioRuntime,
    activity: &GraphActivity,
    raw: &str,
) -> Option<u16> {
    curios
        .owned(activity)
        .unwrap()
        .iter()
        .find(|holding| holding.state() == &state(raw))
        .map(|holding| holding.charges())
}
fn entered_room(combined: &Combined, activity: &GraphActivity) -> Option<(u32, u16)> {
    let node = activity.player_view().current_node().get();
    combined
        .scenario
        .route
        .rooms
        .iter()
        .find(|context| {
            let base = context.entry_node().get();
            base <= node && node < base + 64
        })
        .map(|context| (context.plane_ordinal, context.position_ordinal))
}
fn next_room(
    fixture: &DivergentUniverseBaselineFixture,
    combined: &Combined,
    activity: &mut GraphActivity,
) -> (u32, u16) {
    let before = entered_room(combined, activity).unwrap();
    for _ in 0..16 {
        // Selection staging and deck offers are outside the entry prefix.
        if combined
            .scenario
            .flow
            .offered_tawot_service(activity)
            .is_some()
        {
            choose(&combined.scenario, activity, 2);
        } else {
            advance(fixture, &combined.scenario, activity);
        }
        if let Some(current) = entered_room(combined, activity)
            && current != before
        {
            return current;
        }
    }
    panic!("bounded source route must enter the next room");
}
fn assert_absent(curios: &DivergentUniverseCurioRuntime, activity: &GraphActivity, raw: &str) {
    assert_eq!(charges(curios, activity, raw), None);
    let key = curios
        .states()
        .iter()
        .find(|row| row.id() == &state(raw))
        .unwrap()
        .state_key();
    for slot in [
        CURIO_STATES_SLOT,
        CURIO_CHARGES_SLOT,
        CURIO_ACTIVATIONS_SLOT,
    ] {
        assert!(activity.player_view().slots().iter().any(|row| {
            row.id() == slot
                && matches!(row.value(), ActivityValue::BoundedCounterMap(entries)
                if !entries.iter().any(|(candidate, _)| *candidate == key))
        }));
    }
}

#[test]
fn source_room_curio_entries_complete_all_four_lifetimes_and_pause_destroyed_holdings() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        let combined = build_combined(&fixture, family, 2, false);
        for (raw, limit) in [("9070", 3), ("9079", 5), ("9071", 3), ("9072", 3)] {
            let mut activity = ready(&fixture, &combined);
            assert_eq!(entered_room(&combined, &activity), Some((1, 2)));
            acquire(&curios, &mut activity, raw);
            acquire(&curios, &mut activity, "9159");
            assert_eq!(charges(&curios, &activity, raw), Some(limit));
            // The acquisition room is already entered. Menu/cache loops never
            // count and do not retroactively grant its entry income.
            let balance = currency_balance(
                &activity,
                combined
                    .scenario
                    .flow
                    .economy()
                    .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                    .key(),
            );
            for _ in 0..2 {
                choose(&combined.scenario, &mut activity, 1);
                choose(&combined.scenario, &mut activity, CANCEL);
                assert_eq!(charges(&curios, &activity, raw), Some(limit));
            }
            assert_eq!(
                currency_balance(
                    &activity,
                    combined
                        .scenario
                        .flow
                        .economy()
                        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                        .key()
                ),
                balance
            );
            let hash = activity.state_hash();
            curios
                .destroy_accepted(&mut activity, hash, &state(raw))
                .unwrap();
            assert_eq!(next_room(&fixture, &combined, &mut activity), (1, 3));
            assert_eq!(charges(&curios, &activity, raw), Some(limit));
            let hash = activity.state_hash();
            curios
                .repair_accepted(&mut activity, hash, &state(raw))
                .unwrap();
            for remaining in (0..limit).rev() {
                next_room(&fixture, &combined, &mut activity);
                if remaining == 0 {
                    assert_absent(&curios, &activity, raw);
                } else {
                    assert_eq!(charges(&curios, &activity, raw), Some(remaining));
                }
            }
            assert!(charges(&curios, &activity, "9159").is_some());
            acquire(&curios, &mut activity, raw);
            assert_eq!(charges(&curios, &activity, raw), Some(limit));
        }
    }
}

#[test]
fn source_room_curio_entries_grant_final_income_with_bonuses_and_reconstruct_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        let combined = build_combined(&fixture, family, 2, false);
        let fresh = build_combined(&fixture, family, 2, false);
        let mut activity = ready(&fixture, &combined);
        let mut rebuilt = ready(&fixture, &fresh);
        for raw in ["9071", "9055", "9159"] {
            acquire(&curios, &mut activity, raw);
            acquire(&curios, &mut rebuilt, raw);
        }
        assert_eq!(
            activity.canonical_state_bytes(),
            rebuilt.canonical_state_bytes()
        );
        let currency = combined
            .scenario
            .flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .key();
        for remaining in (0..3).rev() {
            let before = currency_balance(&activity, currency);
            assert_eq!(
                next_room(&fixture, &combined, &mut activity),
                next_room(&fixture, &fresh, &mut rebuilt)
            );
            assert_eq!(currency_balance(&activity, currency), before + 108);
            assert_eq!(
                activity.canonical_state_bytes(),
                rebuilt.canonical_state_bytes()
            );
            if remaining == 0 {
                assert_absent(&curios, &activity, "9071");
            } else {
                assert_eq!(charges(&curios, &activity, "9071"), Some(remaining));
            }
        }
    }
}

#[test]
fn source_room_curio_entries_zero_allowance_discards_without_intrinsic_income() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let combined = build_combined(&fixture, FAMILIES[0], 2, false);
    let curios = fixture.factory().curio_runtime().unwrap();
    let mut activity = ready(&fixture, &combined);
    acquire(&curios, &mut activity, "9071");
    let hash = activity.state_hash();
    curios
        .set_accepted_charges(&mut activity, hash, &state("9071"), 0)
        .unwrap();
    let currency = combined
        .scenario
        .flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    let before = currency_balance(&activity, currency);
    next_room(&fixture, &combined, &mut activity);
    assert_eq!(currency_balance(&activity, currency), before);
    assert_absent(&curios, &activity, "9071");
}

#[test]
fn source_room_curio_entries_late_fixed_room_rejection_restores_grant_discard_and_rng() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        let combined = build_combined(&fixture, family, 2, true);
        let mut activity = ready(&fixture, &combined);
        assert_eq!(entered_room(&combined, &activity), Some((1, 3)));
        acquire(&curios, &mut activity, "9071");
        let hash = activity.state_hash();
        curios
            .set_accepted_charges(&mut activity, hash, &state("9071"), 1)
            .unwrap();
        let bytes = activity.canonical_state_bytes();
        let rng = activity.debug_view().rng().to_vec();
        let decision = activity.player_view().decision().unwrap().id();
        for _ in 0..2 {
            let hash = activity.state_hash();
            assert!(
                combined
                    .scenario
                    .flow
                    .choose_tawot_service_option(
                        &mut activity,
                        hash,
                        decision,
                        ActivityOptionId::new(2).unwrap()
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), bytes);
            assert_eq!(activity.debug_view().rng(), rng);
            assert_eq!(charges(&curios, &activity, "9071"), Some(1));
        }
    }
}

#[test]
fn source_room_curio_entries_fragment_overflow_rejects_without_consuming_allowance() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let combined = build_combined(&fixture, FAMILIES[0], 2, true);
    let curios = fixture.factory().curio_runtime().unwrap();
    for (balance, bonus) in [(i64::MAX - 59, false), (i64::MAX - 60, true)] {
        let mut activity = ready(&fixture, &combined);
        acquire(&curios, &mut activity, "9071");
        if bonus {
            acquire(&curios, &mut activity, "9055");
        }
        let currency = combined
            .scenario
            .flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .key();
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(24100).unwrap(),
            vec![ActivityOperation::SetCounter {
                slot: CURRENCIES_SLOT,
                key: currency,
                value: ActivityExpression::Literal(ActivityValue::BoundedInteger(balance)),
            }],
        )
        .unwrap();
        activity
            .apply_boundary_program(activity.state_hash(), &program)
            .unwrap();
        let bytes = activity.canonical_state_bytes();
        let decision = activity.player_view().decision().unwrap().id();
        let hash = activity.state_hash();
        assert!(
            combined
                .scenario
                .flow
                .choose_tawot_service_option(
                    &mut activity,
                    hash,
                    decision,
                    ActivityOptionId::new(2).unwrap()
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), bytes);
        assert_eq!(charges(&curios, &activity, "9071"), Some(3));
    }
}

#[test]
fn source_room_curio_entries_binding_rejects_omitted_owned_entry_programs() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let combined = build_combined(&fixture, FAMILIES[0], 2, false);
    let scenario = &combined.scenario;
    let definition = scenario.flow.definition();
    for fragment in [
        scenario.rooms[0].fragment(),
        combined.services[0].fragment(),
    ] {
        let raw = &fragment.programs[0];
        let mut programs = definition.programs().to_vec();
        *programs
            .iter_mut()
            .find(|program| program.node() == raw.node())
            .unwrap() = raw.clone();
        let changed = Arc::new(
            GraphActivityDefinition::new(
                definition.identity(),
                definition.graph().clone(),
                definition.state_definition().clone(),
                Arc::clone(definition.participants()),
                programs,
                None,
                ActivityRandomPolicies::new(Vec::new(), definition.random_offers().to_vec()),
            )
            .unwrap(),
        );
        assert!(
            fixture
                .factory()
                .bind_position_rooms(
                    scenario.base.clone(),
                    changed,
                    &scenario.rooms,
                    &combined.services,
                    payload()
                )
                .is_err()
        );
    }
}

#[test]
fn source_room_curio_entries_compiler_rejects_missing_revisited_and_cyclic_entries() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let flow = base(&fixture, FAMILIES[0]);
    for invalid in 0..3 {
        let result = factory.compile_curio_domain_route(
            flow.area(),
            &factory.decision_catalog().domain_decks()[0].key,
            3,
            SLOTS,
            |context| {
                let mut fragment = probe(context)?;
                match invalid {
                    0 => fragment.programs.clear(),
                    1 => {
                        fragment.nodes[0] = ActivityNodeDefinition::new(
                            context.entry_node(),
                            context.section,
                            ActivityNodeKind::Choice,
                            2,
                        )
                        .unwrap()
                    }
                    2 => fragment.edges.push(
                        ActivityEdgeDefinition::new(
                            context.edge(0).unwrap(),
                            context.entry_node(),
                            context.entry_node(),
                            ActivityEdgeCondition::Always,
                            0,
                            1,
                        )
                        .unwrap(),
                    ),
                    _ => unreachable!(),
                }
                Ok(fragment)
            },
        );
        assert!(matches!(result, Err(DomainRouteError::InvalidFragment(_))));
    }
}
