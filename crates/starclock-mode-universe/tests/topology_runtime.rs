use std::sync::{Arc, OnceLock};

use starclock_activity::{
    ActivityDecisionKind, ActivityInstanceId, ActivityMasterSeed, ActivityOptionId,
    ActivityRngLabel, ActivityStateHash, ActivityTransactionRejection, ActivityValue, BuildDigest,
    GraphActivity, GraphActivityCommandError, LoadoutLockScope, OpaqueParticipantBuild,
    ParticipantId, ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};
use starclock_mode_universe::{
    catalog::UniverseCatalog,
    entry::{CompiledActivity, StandardUniverseEntry, StandardUniverseProfile},
    topology::STANDARD_UNIVERSE_TOPOLOGY_REVISION,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

fn catalog() -> Arc<UniverseCatalog> {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    Arc::clone(CATALOG.get_or_init(|| {
        let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
        UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog")
    }))
}

fn participants() -> ParticipantLock {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .expect("standard policy");
    let entries = (0_u8..4)
        .map(|index| {
            let byte = index + 1;
            ParticipantLockEntry::new(
                ParticipantId::new(u32::from(byte)).expect("participant"),
                0,
                index,
                UnitDefinitionId::new(20_001 + u32::from(index)).expect("unit"),
                OpaqueParticipantBuild::new(
                    CombatantSpecDigest::new([byte; 32]).expect("spec digest"),
                    BuildDigest::new([byte + 32; 32]).expect("build digest"),
                    "test-build-catalog-v1",
                    ParticipantSourceKind::CompiledBuild,
                )
                .expect("build"),
            )
            .expect("entry")
        })
        .collect();
    ParticipantLock::seal(policy, entries).expect("lock")
}

fn compiled() -> (Arc<UniverseCatalog>, CompiledActivity) {
    let catalog = catalog();
    let world = &catalog.worlds()[0];
    let profile = StandardUniverseProfile::new(Arc::clone(&catalog));
    let compiled = profile
        .compile(StandardUniverseEntry::new(
            world.id(),
            world.difficulties()[0],
            participants(),
            vec![],
        ))
        .expect("compiled World entry");
    (catalog, compiled)
}

fn choose_first(activity: &mut GraphActivity) {
    let view = activity.player_view();
    let decision = view.decision().expect("offered decision");
    activity
        .choose_option(view.state_hash(), decision.id(), decision.options()[0].id())
        .expect("first option");
}

#[test]
fn all_topologies_compile_to_bounded_spatial_free_hubs() {
    let (catalog, compiled) = compiled();
    let runtime = compiled.runtime_definition();
    assert_eq!(compiled.topology_candidates().len(), 37);
    assert_eq!(compiled.domain_hubs().len(), 579);
    assert_eq!(runtime.graph().nodes().len(), 582);
    assert_eq!(runtime.graph().edges().len(), 782);
    assert_eq!(runtime.graph().maximum_total_visits(), 582);
    assert_eq!(
        runtime.graph().digest().bytes(),
        [
            194, 55, 50, 132, 78, 86, 75, 151, 183, 165, 95, 85, 118, 192, 211, 224, 80, 141, 209,
            135, 134, 27, 102, 229, 27, 87, 58, 19, 210, 221, 81, 164,
        ]
    );
    assert_eq!(
        STANDARD_UNIVERSE_TOPOLOGY_REVISION,
        "standard-universe-topology-v1"
    );

    for hub in compiled.domain_hubs() {
        assert!(!hub.eligible_rooms().is_empty());
        for room_id in hub.eligible_rooms() {
            let room = catalog.room(*room_id).expect("eligible room");
            assert!(
                room.section_ids().is_empty()
                    || room.section_ids().contains(&0)
                    || room.section_ids().contains(&hub.section_index())
            );
        }
        assert!(!hub.routes().is_empty());
    }
}

#[test]
fn start_draws_one_topology_and_offers_nine_paths_without_leaking_private_state() {
    let (_, compiled) = compiled();
    let started = compiled
        .start(
            ActivityInstanceId::new(1).expect("instance"),
            ActivityMasterSeed::from_u64(7),
        )
        .expect("start");
    let view = started.view();
    assert_eq!(
        view.state_hash().bytes(),
        [
            243, 200, 165, 224, 161, 11, 61, 227, 227, 28, 22, 204, 142, 61, 14, 132, 240, 198,
            114, 230, 53, 119, 121, 131, 101, 65, 201, 74, 225, 60, 142, 144,
        ]
    );
    let decision = view.decision().expect("Path choice");
    assert_eq!(decision.kind(), ActivityDecisionKind::Choice);
    assert_eq!(decision.options().len(), 9);
    assert!(
        view.slots()
            .iter()
            .all(|slot| slot.id() != compiled.selected_topology_slot())
    );
    let activity = started.into_activity();
    let debug = activity.debug_view();
    let graph_rng = debug
        .rng()
        .iter()
        .find(|stream| stream.label() == ActivityRngLabel::Graph)
        .expect("Graph stream");
    assert_eq!(graph_rng.draw_count(), 1);
    assert!(matches!(
        debug
            .all_slots()
            .iter()
            .find(|slot| slot.id() == compiled.selected_topology_slot())
            .expect("private topology slot")
            .value(),
        ActivityValue::OptionalId(Some(_))
    ));
}

#[test]
fn mandatory_interaction_consumption_gates_routes_and_a_seeded_graph_terminates() {
    let (_, compiled) = compiled();
    let mut activity = compiled
        .start(
            ActivityInstanceId::new(2).expect("instance"),
            ActivityMasterSeed::from_u64(7),
        )
        .expect("start")
        .into_activity();
    choose_first(&mut activity);

    for _ in 0..128 {
        let view = activity.player_view();
        if view.terminal().is_some() {
            assert_eq!(
                view.terminal(),
                Some(starclock_activity::ActivityTerminalOutcome::Completed)
            );
            return;
        }
        let hub = compiled
            .domain_hubs()
            .iter()
            .find(|hub| hub.node() == view.current_node())
            .expect("current domain hub");
        let decision = view.decision().expect("hub decision");
        assert_eq!(decision.kind(), ActivityDecisionKind::Route);
        if decision
            .options()
            .iter()
            .any(|item| item.id() == hub.interaction())
        {
            assert_eq!(decision.options().len(), 1);
            activity
                .choose_option(view.state_hash(), decision.id(), hub.interaction())
                .expect("mandatory interaction");
            let after = activity.player_view();
            assert_eq!(after.current_node(), hub.node());
            assert_eq!(
                after.decision().expect("routes").options().len(),
                hub.routes().len()
            );
        } else {
            activity
                .choose_option(view.state_hash(), decision.id(), decision.options()[0].id())
                .expect("route");
        }
    }
    panic!("seeded topology did not terminate within its bound");
}

#[test]
fn stale_and_unoffered_hub_commands_preserve_exact_state_and_rng() {
    let (_, compiled) = compiled();
    let mut activity = compiled
        .start(
            ActivityInstanceId::new(3).expect("instance"),
            ActivityMasterSeed::from_u64(11),
        )
        .expect("start")
        .into_activity();
    let before = activity.canonical_state_bytes();
    let view = activity.player_view();
    let decision = view.decision().expect("Path choice");
    assert_eq!(
        activity.choose_option(
            ActivityStateHash::new([0; 32]).expect("zero hash allowed"),
            decision.id(),
            decision.options()[0].id(),
        ),
        Err(GraphActivityCommandError::StaleStateHash)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(
        activity.choose_option(
            view.state_hash(),
            decision.id(),
            ActivityOptionId::new(9_999_999).expect("forged option"),
        ),
        Err(GraphActivityCommandError::Rejected(
            ActivityTransactionRejection::UnknownOption
        ))
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn topology_draw_is_reproducible_for_the_same_seed_and_identity() {
    let (_, compiled) = compiled();
    let selected = |instance| {
        compiled
            .start(
                ActivityInstanceId::new(instance).expect("instance"),
                ActivityMasterSeed::from_u64(29),
            )
            .expect("start")
            .into_activity()
            .debug_view()
            .all_slots()
            .iter()
            .find(|slot| slot.id() == compiled.selected_topology_slot())
            .expect("topology slot")
            .value()
            .clone()
    };
    assert_eq!(selected(7), selected(7));
    assert_ne!(selected(7), selected(8));
}
