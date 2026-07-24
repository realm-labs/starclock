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
    topology::{STANDARD_UNIVERSE_DOMAIN_VISIT_CLASS, STANDARD_UNIVERSE_TOPOLOGY_REVISION},
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
    assert_eq!(compiled.abstract_interactions().len(), 12_051);
    assert_eq!(runtime.graph().nodes().len(), 4_058);
    assert_eq!(runtime.graph().edges().len(), 5_993);
    assert_eq!(runtime.graph().maximum_total_visits(), 4_058);
    assert_eq!(
        runtime.state_definition().logical_scopes().classes().len(),
        1
    );
    assert_eq!(
        runtime.state_definition().logical_scopes().classes()[0]
            .id()
            .get(),
        STANDARD_UNIVERSE_DOMAIN_VISIT_CLASS
    );
    assert_eq!(
        runtime.state_definition().logical_scopes().bindings().len(),
        579 * 7
    );
    assert_eq!(
        runtime.graph().digest().bytes(),
        [
            22, 164, 10, 14, 40, 127, 89, 137, 20, 25, 170, 12, 102, 143, 84, 10, 176, 36, 19, 5,
            162, 123, 169, 127, 168, 89, 190, 72, 211, 59, 15, 187,
        ]
    );
    assert_eq!(
        STANDARD_UNIVERSE_TOPOLOGY_REVISION,
        "standard-universe-topology-v5"
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
            26, 150, 45, 209, 112, 17, 162, 83, 132, 226, 152, 165, 254, 5, 94, 167, 122, 167, 45,
            74, 65, 237, 130, 118, 60, 132, 116, 127, 28, 214, 246, 133,
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
fn room_content_and_reward_nodes_gate_routes_without_spatial_state() {
    let (_, compiled) = compiled();
    let mut activity = compiled
        .start(
            ActivityInstanceId::new(2).expect("instance"),
            ActivityMasterSeed::from_u64(7),
        )
        .expect("start")
        .into_activity();
    choose_first(&mut activity);

    let content = activity.player_view();
    let hub = compiled
        .domain_hubs()
        .iter()
        .find(|hub| hub.content_node() == content.current_node())
        .expect("resolved room content hub");
    let decision = content.decision().expect("content interaction");
    assert_eq!(decision.kind(), ActivityDecisionKind::ExternalOutcome);
    assert_eq!(decision.options().len(), 1);
    activity
        .submit_external_outcome(
            content.state_hash(),
            decision.id(),
            starclock_activity::ActivityExternalOutcomeId::new(decision.options()[0].id().get())
                .unwrap(),
        )
        .expect("consume room content");
    let after = activity.player_view();
    assert_ne!(after.current_node(), hub.route_node());
    match after.decision().expect("battle or reward").kind() {
        ActivityDecisionKind::Encounter => assert_eq!(after.current_node(), hub.battle_node()),
        ActivityDecisionKind::Choice => {
            assert_eq!(after.current_node(), hub.formation_node());
            choose_first(&mut activity);
            let routes = activity.player_view();
            assert_eq!(routes.current_node(), hub.route_node());
            assert_eq!(
                routes.decision().expect("routes").options().len(),
                hub.routes().len()
            );
        }
        ActivityDecisionKind::Reward => {
            assert_eq!(after.current_node(), hub.reward_node());
            activity
                .choose_option(
                    after.state_hash(),
                    after.decision().expect("reward").id(),
                    after.decision().expect("reward").options()[0].id(),
                )
                .expect("claim post-content reward");
            choose_first(&mut activity);
            let routes = activity.player_view();
            assert_eq!(routes.current_node(), hub.route_node());
            assert_eq!(
                routes.decision().expect("routes").options().len(),
                hub.routes().len()
            );
        }
        other => panic!("unexpected post-content decision: {other:?}"),
    }
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
    let reference = selected(7);
    assert_eq!(reference, selected(7));
    assert!((8..=16).any(|instance| selected(instance) != reference));
}
