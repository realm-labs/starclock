//! Hostile definitions deliberately retain the original claimed identity.

use std::sync::Arc;

use super::{Fixture, LEAVE_SHOP, ROOT, SLOTS, Setup, compile, instance, program, slot, stock};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    domain_route::DomainRoomComposition,
    shop_purchase::room::{ShopRoomError, ShopRoomSlots},
};
use starclock_activity::{
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId, ActivityGraphDefinition,
    ActivityMasterSeed, ActivityOperation, ActivityOptionId, ActivityRandomOffer,
    ActivityRandomPolicies, ActivityRngLabel, ActivityStateDefinition, GraphActivity,
    GraphActivityDefinition, GraphActivityDefinitionError, GraphActivityNodeProgram,
    LogicalScopeAddress, LogicalScopeDefinitions, LogicalScopeNodeBinding, NodeId,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_domain_decks::DomainCardKind,
};

fn scenario() -> Fixture {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    compile(
        &source,
        DivergentUniverseRunFamily::Ordinary,
        stock(&source),
        Setup {
            budget: 250,
            ..Setup::default()
        },
    )
}

fn replace(
    fixture: &Fixture,
    graph: ActivityGraphDefinition,
    state: ActivityStateDefinition,
    programs: Vec<GraphActivityNodeProgram>,
) -> Arc<GraphActivityDefinition> {
    Arc::new(
        GraphActivityDefinition::new(
            fixture.definition.identity(),
            graph,
            state,
            Arc::clone(fixture.definition.participants()),
            programs,
            None,
            ActivityRandomPolicies::new(Vec::new(), Vec::new()),
        )
        .unwrap(),
    )
}

#[test]
fn shop_room_rejects_invalid_slots_contexts_and_binds_stock_price_identity() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = source.factory();
    for slots in [
        ShopRoomSlots {
            purchased: slot(69),
            accepted: slot(71),
        },
        ShopRoomSlots {
            purchased: slot(70),
            accepted: slot(69),
        },
        ShopRoomSlots {
            purchased: slot(70),
            accepted: slot(70),
        },
    ] {
        assert!(factory.shop_room_compiler(stock(&source), slots).is_err());
    }
    let fixture = scenario();
    let compiler = factory.shop_room_compiler(stock(&source), SLOTS).unwrap();
    let mut context = fixture.room.context().clone();
    context.composition = DomainRoomComposition::Card(DomainCardKind::Reward);
    assert!(matches!(
        compiler.compile(&context),
        Err(ShopRoomError::InvalidContext)
    ));
    let mut context = fixture.room.context().clone();
    context.position_key = "fabricated-source-position".into();
    assert!(matches!(
        compiler.compile(&context),
        Err(ShopRoomError::InvalidContext)
    ));
    let mut context = fixture.room.context().clone();
    context.preset_source = "fabricated-source-shop".into();
    assert!(matches!(
        compiler.compile(&context),
        Err(ShopRoomError::InvalidContext)
    ));
    let same = compiler.compile(fixture.room.context()).unwrap();
    assert_eq!(
        same.configuration_digest(),
        fixture.room.configuration_digest()
    );
    same.bind(Arc::clone(&fixture.definition)).unwrap();
    let mut items = stock(&source);
    items[0].price += 1;
    let changed = factory
        .shop_room_compiler(items, SLOTS)
        .unwrap()
        .compile(fixture.room.context())
        .unwrap();
    assert_ne!(changed.configuration_digest(), same.configuration_digest());
    assert!(matches!(
        changed.bind(Arc::clone(&fixture.definition)),
        Err(ShopRoomError::DefinitionMismatch)
    ));
}

#[test]
fn shop_room_binding_rejects_changed_program_missing_entry_prefix_or_slot_declaration() {
    let fixture = scenario();
    let definition = &fixture.definition;
    for target in [
        fixture.room.menu_node(),
        fixture.room.context().entry_node(),
    ] {
        let mut programs = definition.programs().to_vec();
        let record = programs
            .iter_mut()
            .find(|record| record.node() == target)
            .unwrap();
        *record = if target == fixture.room.context().entry_node() {
            fixture
                .room
                .fragment()
                .programs
                .iter()
                .find(|record| record.node() == target)
                .unwrap()
                .clone()
        } else {
            let mut operations = record.program().operations().to_vec();
            operations.remove(0);
            program(target, operations)
        };
        let hostile = replace(
            &fixture,
            definition.graph().clone(),
            definition.state_definition().clone(),
            programs,
        );
        assert!(matches!(
            fixture.room.bind(hostile),
            Err(ShopRoomError::DefinitionMismatch)
        ));
    }
    let state = definition.state_definition();
    let slots = state
        .slots()
        .iter()
        .filter(|declaration| declaration.id() != SLOTS.purchased)
        .cloned()
        .collect();
    // Missing declaration prevents graph construction itself or capability binding.
    let missing = ActivityStateDefinition::new(slots, Vec::new(), Vec::new())
        .unwrap()
        .with_logical_scopes(state.logical_scopes().clone());
    let hostile = GraphActivityDefinition::new(
        definition.identity(),
        definition.graph().clone(),
        missing,
        Arc::clone(definition.participants()),
        definition.programs().to_vec(),
        None,
        ActivityRandomPolicies::new(Vec::new(), Vec::new()),
    );
    if let Ok(hostile) = hostile {
        assert!(matches!(
            fixture.room.bind(Arc::new(hostile)),
            Err(ShopRoomError::DefinitionMismatch)
        ));
    }
}

#[test]
fn shop_room_binding_rejects_extra_exit_entry_bypass_and_wrong_logical_room() {
    let fixture = scenario();
    let definition = &fixture.definition;
    let original = definition.graph();
    for (from, to) in [
        (
            fixture.room.context().entry_node(),
            fixture.room.context().successor(),
        ),
        (NodeId::new(ROOT).unwrap(), fixture.room.menu_node()),
    ] {
        let mut edges = original.edges().to_vec();
        edges.push(
            ActivityEdgeDefinition::new(
                ActivityEdgeId::new(ROOT + 1).unwrap(),
                from,
                to,
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .unwrap(),
        );
        let graph = ActivityGraphDefinition::new(
            original.entry(),
            original.nodes().to_vec(),
            edges,
            original.maximum_total_visits(),
        )
        .unwrap();
        let hostile = replace(
            &fixture,
            graph,
            definition.state_definition().clone(),
            definition.programs().to_vec(),
        );
        assert!(matches!(
            fixture.room.bind(hostile),
            Err(ShopRoomError::DefinitionMismatch)
        ));
    }
    let state = definition.state_definition();
    let scopes = state.logical_scopes();
    let mut bindings = scopes.bindings().to_vec();
    let binding = bindings
        .iter_mut()
        .find(|binding| binding.node() == fixture.room.menu_node())
        .unwrap();
    let mut path = binding.path().to_vec();
    path[2] = LogicalScopeAddress::new(path[2].class(), path[2].key() + 1).unwrap();
    *binding = LogicalScopeNodeBinding::new(binding.node(), path).unwrap();
    let hostile = replace(
        &fixture,
        original.clone(),
        state.clone().with_logical_scopes(
            LogicalScopeDefinitions::new(scopes.classes().to_vec(), bindings).unwrap(),
        ),
        definition.programs().to_vec(),
    );
    assert!(matches!(
        fixture.room.bind(hostile),
        Err(ShopRoomError::DefinitionMismatch)
    ));
}

#[test]
fn shop_room_bound_capability_rejects_foreign_external_program_with_same_claimed_identity() {
    let fixture = scenario();
    let definition = &fixture.definition;
    let mut programs = definition.programs().to_vec();
    let root = programs
        .iter_mut()
        .find(|record| record.node().get() == ROOT)
        .unwrap();
    let mut operations = root.program().operations().to_vec();
    operations.insert(
        0,
        ActivityOperation::SetCounterMap {
            slot: SLOTS.purchased,
            values: Box::new([]),
        },
    );
    *root = program(root.node(), operations);
    let hostile = replace(
        &fixture,
        definition.graph().clone(),
        definition.state_definition().clone(),
        programs,
    );
    // The fragment still binds, but the old whole-definition capability is foreign.
    let foreign = fixture.room.bind(Arc::clone(&hostile)).unwrap();
    let mut activity = GraphActivity::start(
        hostile,
        instance(26311),
        ActivityMasterSeed::from_u64(26311),
    )
    .unwrap()
    .into_activity();
    let original = fixture.bound();
    assert!(!original.offered(&activity));
    assert!(foreign.offered(&activity));
    let before = activity.canonical_state_bytes();
    let offer = activity.player_view().decision().unwrap().clone();
    let hash = activity.state_hash();
    assert!(
        original
            .choose(
                &mut activity,
                hash,
                offer.id(),
                ActivityOptionId::new(1).unwrap()
            )
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn shop_room_sampling_injection_fails_closed_in_graph_validation_or_room_binding() {
    let fixture = scenario();
    let definition = &fixture.definition;
    let sampled = ActivityRandomOffer::new(
        fixture.room.menu_node(),
        ActivityRngLabel::Reward,
        26313,
        1,
        vec![
            (ActivityOptionId::new(1).unwrap(), 1),
            (ActivityOptionId::new(2).unwrap(), 1),
            (ActivityOptionId::new(3).unwrap(), 1),
            (ActivityOptionId::new(LEAVE_SHOP).unwrap(), 1),
        ],
        None,
    )
    .unwrap();
    let build = |programs| {
        GraphActivityDefinition::new(
            definition.identity(),
            definition.graph().clone(),
            definition.state_definition().clone(),
            Arc::clone(definition.participants()),
            programs,
            None,
            ActivityRandomPolicies::new(Vec::new(), vec![sampled.clone()]),
        )
    };
    // Shared random-offer validation requires a single Offer operation; the
    // exact menu's explicit acceptance reset already prevents this injection.
    assert!(matches!(
        build(definition.programs().to_vec()),
        Err(GraphActivityDefinitionError::InvalidRandomOffer)
    ));
    let mut programs = definition.programs().to_vec();
    let record = programs
        .iter_mut()
        .find(|record| record.node() == fixture.room.menu_node())
        .unwrap();
    let mut operations = record.program().operations().to_vec();
    operations.remove(0);
    *record = program(record.node(), operations);
    let hostile = Arc::new(build(programs).unwrap());
    assert!(matches!(
        fixture.room.bind(hostile),
        Err(ShopRoomError::DefinitionMismatch)
    ));
}
