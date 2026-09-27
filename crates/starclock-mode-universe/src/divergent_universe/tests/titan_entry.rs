use super::{complete, entry, instance, slot_value};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseCurrencyCommand,
    DivergentUniverseCurrencyCommandError, DivergentUniverseCurrencyKind,
    DivergentUniverseCurrencySpendRule, DivergentUniverseEntryFlowError,
    DivergentUniverseRuntimeFactory, DivergentUniverseTitanRuntime,
    DivergentUniverseTitanRuntimeError, state::CURRENCIES_SLOT,
};
use starclock_activity::{
    ActivityMasterSeed, ActivityStateHash, ActivityTransactionEventKind, ActivityValue,
    GraphActivityCommandError,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_titan_catalog::DivergentUniverseTitanTalentId,
};

fn fragment_talent() -> DivergentUniverseTitanTalentId {
    DivergentUniverseTitanTalentId::new("divergent-universe.titan-talent.12302").unwrap()
}

fn closure(runtime: &DivergentUniverseTitanRuntime) -> Vec<DivergentUniverseTitanTalentId> {
    let mut ids = Vec::new();
    let mut current = Some(fragment_talent());
    while let Some(id) = current {
        let talent = runtime
            .talents()
            .iter()
            .find(|talent| talent.id() == &id)
            .unwrap();
        current = talent.predecessor().cloned();
        ids.push(id);
    }
    ids.sort_unstable();
    ids
}

#[test]
fn titan_entry_fragments_are_eventful_once_per_run_and_spendable_in_both_families() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.titan_runtime().unwrap();
    let ids = closure(&runtime);
    let original = ids.clone();
    let without_fragment: Vec<_> = ids
        .iter()
        .filter(|id| **id != fragment_talent())
        .cloned()
        .collect();
    for area in ["401", "20401"] {
        let flow = factory
            .compile(entry(area, "3011").with_titan_talents(ids.clone()).unwrap())
            .unwrap();
        let control = factory
            .compile(
                entry(area, "3011")
                    .with_titan_talents(without_fragment.clone())
                    .unwrap(),
            )
            .unwrap();
        assert_ne!(
            flow.definition().identity(),
            control.definition().identity()
        );
        assert_eq!(flow.entry_titan_talents(), original);
        let input = flow.input_snapshot().clone();
        let started = flow
            .start(instance(1), ActivityMasterSeed::from_u64(22))
            .unwrap();
        let key = flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .key();
        let credit_events = started
            .events()
            .iter()
            .filter(|event| {
                event.kind()
                    == &ActivityTransactionEventKind::CounterChanged {
                        slot: CURRENCIES_SLOT,
                        key,
                    }
            })
            .collect::<Vec<_>>();
        // The shared gain pipeline emits its base and inactive-bonus zero
        // deltas under one command cause; that is one grant, not five grants.
        assert_eq!(
            credit_events.len(),
            1 + factory.decision_catalog().curio_fragment_gains().len()
        );
        let cause = credit_events[0].cause();
        assert_eq!(cause.program().get(), 22_535);
        assert!(credit_events.iter().all(|event| event.cause() == cause));
        let mut activity = started.into_activity();
        assert_eq!(
            slot_value(&activity.player_view(), CURRENCIES_SLOT),
            &ActivityValue::BoundedCounterMap(vec![(key, 30)].into_boxed_slice())
        );
        assert_eq!(runtime.snapshot(&activity).unwrap().talents(), original);
        let mut control_activity = control
            .start(instance(1), ActivityMasterSeed::from_u64(22))
            .unwrap()
            .into_activity();
        let spend = DivergentUniverseCurrencyCommand::Spend {
            currency: DivergentUniverseCurrencyKind::CosmicFragment,
            rule: DivergentUniverseCurrencySpendRule::CurseChest,
            amount: 30,
        };
        let before = control_activity.canonical_state_bytes();
        assert!(matches!(
            control.apply_currency_command(
                &mut control_activity,
                ActivityStateHash::new([7; 32]).unwrap(),
                spend
            ),
            Err(DivergentUniverseCurrencyCommandError::Activity(
                GraphActivityCommandError::StaleStateHash
            ))
        ));
        let hash = control_activity.state_hash();
        assert_eq!(
            control.apply_currency_command(&mut control_activity, hash, spend),
            Err(DivergentUniverseCurrencyCommandError::InsufficientBalance)
        );
        assert_eq!(control_activity.canonical_state_bytes(), before);
        let hash = activity.state_hash();
        flow.apply_currency_command(&mut activity, hash, spend)
            .unwrap();
        complete(&mut activity);
        assert_eq!(
            slot_value(&activity.player_view(), CURRENCIES_SLOT),
            &ActivityValue::BoundedCounterMap(Box::new([]))
        );
        assert_eq!(runtime.snapshot(&activity).unwrap().talents(), original);
        assert_eq!(flow.input_snapshot(), &input);
        let fresh = flow
            .start(instance(1), ActivityMasterSeed::from_u64(22))
            .unwrap()
            .into_activity();
        assert_eq!(
            slot_value(&fresh.player_view(), CURRENCIES_SLOT),
            &ActivityValue::BoundedCounterMap(vec![(key, 30)].into_boxed_slice())
        );
    }
    assert_eq!(ids, original);
}

#[test]
fn titan_entry_rejects_duplicates_unknown_ids_and_incomplete_prerequisites() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    assert!(matches!(
        entry("401", "3011").with_titan_talents(vec![fragment_talent(), fragment_talent()]),
        Err(DivergentUniverseEntryFlowError::InvalidTitanTalents)
    ));
    let unknown =
        DivergentUniverseTitanTalentId::new("divergent-universe.titan-talent.99999").unwrap();
    assert!(matches!(
        factory.compile(
            entry("401", "3011")
                .with_titan_talents(vec![unknown])
                .unwrap()
        ),
        Err(DivergentUniverseEntryFlowError::Titan(
            DivergentUniverseTitanRuntimeError::UnknownIdentity
        ))
    ));
    assert!(matches!(
        factory.compile(
            entry("401", "3011")
                .with_titan_talents(vec![fragment_talent()])
                .unwrap()
        ),
        Err(DivergentUniverseEntryFlowError::Titan(
            DivergentUniverseTitanRuntimeError::InvalidState
        ))
    ));
    let ids = closure(&factory.titan_runtime().unwrap());
    let mut reversed = ids.clone();
    reversed.reverse();
    let a = factory
        .compile(entry("401", "3011").with_titan_talents(ids).unwrap())
        .unwrap();
    let b = factory
        .compile(entry("401", "3011").with_titan_talents(reversed).unwrap())
        .unwrap();
    assert_eq!(a.definition().identity(), b.definition().identity());
    let empty = factory
        .compile(entry("401", "3011").with_titan_talents(vec![]).unwrap())
        .unwrap();
    let default = factory.compile(entry("401", "3011")).unwrap();
    assert_eq!(
        empty.definition().identity(),
        default.definition().identity()
    );
}

#[test]
fn titan_entry_late_unlock_does_not_retroactively_grant_starting_fragments() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.titan_runtime().unwrap();
    let ids = closure(&runtime)
        .into_iter()
        .filter(|id| *id != fragment_talent())
        .collect();
    let flow = factory
        .compile(entry("401", "3011").with_titan_talents(ids).unwrap())
        .unwrap();
    let mut activity = flow
        .start(instance(1), ActivityMasterSeed::from_u64(22))
        .unwrap()
        .into_activity();
    let hash = activity.state_hash();
    runtime
        .credit_talent_currency_accepted(&mut activity, hash, 75)
        .unwrap();
    let hash = activity.state_hash();
    runtime
        .unlock_talent_accepted(&mut activity, hash, &fragment_talent())
        .unwrap();
    assert_eq!(
        slot_value(&activity.player_view(), CURRENCIES_SLOT),
        &ActivityValue::BoundedCounterMap(Box::new([]))
    );
    assert!(!flow.entry_titan_talents().contains(&fragment_talent()));
    let next = factory
        .compile(
            entry("401", "3011")
                .with_titan_talents(runtime.snapshot(&activity).unwrap().talents().to_vec())
                .unwrap(),
        )
        .unwrap();
    let next_activity = next
        .start(instance(1), ActivityMasterSeed::from_u64(22))
        .unwrap()
        .into_activity();
    let key = next
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    assert_eq!(
        slot_value(&next_activity.player_view(), CURRENCIES_SLOT),
        &ActivityValue::BoundedCounterMap(vec![(key, 30)].into_boxed_slice())
    );
}

#[test]
fn titan_entry_grants_before_source_deck_and_equation_offers_without_recredit() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let ids = closure(&fixture.factory().titan_runtime().unwrap());
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let area = if family == DivergentUniverseRunFamily::Ordinary {
            "401"
        } else {
            "20401"
        };
        let flow = fixture
            .flow_for_entry_configuration(
                family,
                &format!("divergent-universe.area.{area}"),
                "divergent-universe.difficulty.3011",
                None,
                true,
                &ids,
            )
            .unwrap();
        let mut activity = flow
            .start(instance(1), ActivityMasterSeed::from_u64(24201))
            .unwrap()
            .into_activity();
        let key = flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .key();
        let expected = ActivityValue::BoundedCounterMap(vec![(key, 30)].into_boxed_slice());
        assert_eq!(
            slot_value(&activity.player_view(), CURRENCIES_SLOT),
            &expected
        );
        let before = activity.canonical_state_bytes();
        let offer = activity.player_view().decision().unwrap().clone();
        let hash = activity.state_hash();
        assert!(
            activity
                .choose_option(hash, offer.id(), offer.options()[0].id())
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        flow.choose_source_deck(&mut activity, hash, offer.id(), offer.options()[0].id())
            .unwrap();
        assert_eq!(
            slot_value(&activity.player_view(), CURRENCIES_SLOT),
            &expected
        );
        let offer = activity.player_view().decision().unwrap().clone();
        let hash = activity.state_hash();
        flow.choose_initial_equation(&mut activity, hash, offer.id(), offer.options()[0].id())
            .unwrap();
        assert_eq!(
            slot_value(&activity.player_view(), CURRENCIES_SLOT),
            &expected
        );
    }
}
