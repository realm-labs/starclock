use super::{
    DivergentUniverseTitanContributionScope, DivergentUniverseTitanRuntimeError,
};
use std::collections::BTreeSet;

#[test]
fn titan_catalog_compiles_all_exact_rows_and_policy_offer_shapes() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.titan_runtime().expect("Titan runtime");
    assert_eq!(runtime.types().len(), 12);
    assert_eq!(runtime.boons().len(), 84);
    assert_eq!(runtime.talents().len(), 36);
    assert_eq!(runtime.choices().len(), 36);
    assert_eq!(runtime.all_contributions().len(), 120);
    assert!(runtime.types().iter().all(|value| {
        value.boon_ids().len() == 7 && value.talent_ids().len() == 3
    }));
    assert!(runtime.choices().iter().all(|choice| {
        choice.eligibility()
            == match choice.level() {
                1 => "TitanTypeActivated",
                2 => "PriorBoonLevel1Accepted",
                3 => "PriorBoonLevel2Accepted",
                _ => "Invalid",
            }
            && choice.fallback() == "RejectWithoutMutation"
            && choice.candidates().len() == if choice.level() == 1 { 1 } else { 3 }
    }));
    assert!(runtime.talents().iter().all(|talent| {
        talent.cost_item_id() == "281020"
            && talent.cost() == match talent.level() {
                1 => 50,
                2 => 75,
                3 => 100,
                _ => 0,
            }
    }));
    assert_eq!(
        runtime
            .all_contributions()
            .iter()
            .filter(|value| value.scope() == DivergentUniverseTitanContributionScope::Battle)
            .count(),
        110,
    );
    assert_eq!(
        runtime
            .all_contributions()
            .iter()
            .filter(|value| value.scope() == DivergentUniverseTitanContributionScope::Activity)
            .count(),
        10,
    );
    assert!(runtime
        .all_contributions()
        .iter()
        .all(|value| value.effects().len() == 1));
}

#[test]
fn every_golden_blood_boon_executes_through_its_exact_level_offer() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.titan_runtime().expect("Titan runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    for (index, target) in runtime.boons().iter().enumerate() {
        let mut activity = flow
            .start(
                instance(5300 + u64::try_from(index).expect("small fixture index")),
                ActivityMasterSeed::from_u64(0x22_05_03),
            )
            .expect("start flow")
            .into_activity();
        let before_draws = reward_draws(&activity);
        let hash = activity.state_hash();
        runtime
            .activate_type_accepted(&mut activity, hash, target.titan_type())
            .expect("accepted Titan activation");
        for level in 1..=target.level() {
            let offer = runtime.next_offer(&activity).expect("next exact offer");
            assert_eq!(offer.level(), level);
            let selected = if level == target.level() {
                target.id()
            } else {
                &offer.candidates()[0]
            };
            assert!(offer.candidates().contains(selected));
            let hash = activity.state_hash();
            runtime
                .accept_boon_accepted(&mut activity, hash, selected)
                .expect("accepted exact offered Boon");
        }
        let snapshot = runtime.snapshot(&activity).expect("Titan snapshot");
        assert_eq!(snapshot.selected_type(), Some(target.titan_type()));
        assert!(snapshot.boons().contains(target.id()));
        assert_eq!(snapshot.contributions().len(), usize::from(target.level()));
        assert!(snapshot.contributions().iter().all(|contribution| {
            contribution.scope() == DivergentUniverseTitanContributionScope::Battle
                && contribution.activation() == "AcceptedGoldenBloodBoon"
                && contribution.teardown() == "BattleEnd"
        }));
        assert_eq!(reward_draws(&activity), before_draws);
    }
}

#[test]
fn all_permanent_titan_talents_enforce_costs_prerequisites_and_stack_contributions() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.titan_runtime().expect("Titan runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let mut activity = flow
        .start(instance(5390), ActivityMasterSeed::from_u64(0x22_05_03))
        .expect("start flow")
        .into_activity();
    let before_draws = reward_draws(&activity);
    let level_two = runtime
        .talents()
        .iter()
        .find(|value| value.level() == 2)
        .expect("level-two talent");
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert_eq!(
        runtime.unlock_talent_accepted(&mut activity, hash, level_two.id()),
        Err(DivergentUniverseTitanRuntimeError::MissingTalentPrerequisite),
    );
    assert_eq!(activity.canonical_state_bytes(), before);

    let total_cost = runtime.talents().iter().map(|value| value.cost()).sum();
    let hash = activity.state_hash();
    runtime
        .credit_talent_currency_accepted(&mut activity, hash, total_cost)
        .expect("accepted exact talent currency credit");
    let mut remaining = runtime.talents().iter().collect::<Vec<_>>();
    let mut unlocked = BTreeSet::new();
    while !remaining.is_empty() {
        let index = remaining
            .iter()
            .position(|talent| {
                talent
                    .predecessor()
                    .is_none_or(|predecessor| unlocked.contains(predecessor))
            })
            .expect("exact Titan talent graph is acyclic");
        let talent = remaining.remove(index);
        let hash = activity.state_hash();
        runtime
            .unlock_talent_accepted(&mut activity, hash, talent.id())
            .expect("accepted permanent talent unlock");
        unlocked.insert(talent.id().clone());
    }
    let snapshot = runtime.snapshot(&activity).expect("Titan snapshot");
    assert_eq!(snapshot.talents().len(), 36);
    assert_eq!(snapshot.contributions().len(), 36);
    assert_eq!(
        snapshot
            .contributions()
            .iter()
            .filter(|value| value.scope() == DivergentUniverseTitanContributionScope::Activity)
            .count(),
        10,
    );
    assert_eq!(
        snapshot
            .contributions()
            .iter()
            .filter(|value| value.scope() == DivergentUniverseTitanContributionScope::Battle)
            .count(),
        26,
    );
    assert!(snapshot
        .contributions()
        .windows(2)
        .all(|pair| pair[0].id() < pair[1].id()));
    assert!(snapshot
        .contributions()
        .iter()
        .filter_map(|value| value.effects()[0].metric())
        .filter(|metric| *metric == "attack")
        .count()
        >= 2);
    assert_eq!(
        titan_talent_currency(&activity),
        0,
        "all exact costs are atomically consumed",
    );
    assert_eq!(reward_draws(&activity), before_draws);
}

#[test]
fn titan_offer_rejections_and_replay_are_state_hash_and_rng_inert() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.titan_runtime().expect("Titan runtime");
    let flow = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    let seed = ActivityMasterSeed::from_u64(0x22_05_03);
    let mut first = flow
        .start(instance(5391), seed)
        .expect("first flow")
        .into_activity();
    let mut replay = flow
        .start(instance(5391), seed)
        .expect("replay flow")
        .into_activity();
    assert_eq!(
        runtime.next_offer(&first),
        Err(DivergentUniverseTitanRuntimeError::TitanTypeNotActivated),
    );
    let titan_type = runtime.types()[0].id();
    for activity in [&mut first, &mut replay] {
        let hash = activity.state_hash();
        runtime
            .activate_type_accepted(activity, hash, titan_type)
            .expect("accepted Titan activation");
        let offer = runtime.next_offer(activity).expect("first offer");
        let hash = activity.state_hash();
        runtime
            .accept_boon_accepted(activity, hash, &offer.candidates()[0])
            .expect("accepted first Boon");
    }
    assert_eq!(first.canonical_state_bytes(), replay.canonical_state_bytes());
    assert_eq!(runtime.snapshot(&first), runtime.snapshot(&replay));

    let wrong = runtime
        .boons()
        .iter()
        .find(|value| value.titan_type() != titan_type && value.level() == 2)
        .expect("wrong-type candidate");
    let before = first.canonical_state_bytes();
    let before_draws = reward_draws(&first);
    let hash = first.state_hash();
    assert_eq!(
        runtime.accept_boon_accepted(&mut first, hash, wrong.id()),
        Err(DivergentUniverseTitanRuntimeError::BoonNotOffered),
    );
    assert_eq!(first.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&first), before_draws);

    let stale = ActivityStateHash::new([0x53; 32]).expect("stale hash");
    assert_eq!(
        runtime.credit_talent_currency_accepted(&mut first, stale, 50),
        Err(DivergentUniverseTitanRuntimeError::Activity(
            GraphActivityCommandError::StaleStateHash,
        )),
    );
    assert_eq!(first.canonical_state_bytes(), before);
}

fn titan_talent_currency(activity: &GraphActivity) -> i64 {
    let view = activity.player_view();
    let value = view
        .slots()
        .iter()
        .find(|slot| slot.id() == super::state::TITAN_TALENT_CURRENCY_SLOT)
        .expect("Titan talent currency slot")
        .value();
    let ActivityValue::BoundedInteger(value) = value else {
        panic!("Titan talent currency slot kind");
    };
    *value
}
