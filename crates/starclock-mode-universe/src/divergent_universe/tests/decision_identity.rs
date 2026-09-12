//! Configuration dependency tests; these do not execute authored event choices.

use std::sync::Arc;

use starclock_activity::{
    ActivityInstanceId, ActivityMasterSeed, ActivityRandomPolicies, GraphActivity,
    GraphActivityDefinition,
};
use starclock_data::divergent_universe_catalog::{
    DivergentUniverseAreaId, DivergentUniverseDifficultyId, DivergentUniverseRunFamily,
};
use starclock_replay::{
    component::{
        ConfigurationComponentIdentity, ConfigurationComponentKind, ConfigurationComponentSet,
    },
    digest::ComponentDigest,
};

use super::compile_identity;
use crate::divergent_universe::{
    DivergentUniverseAccountSnapshotDigest, DivergentUniverseBaselineFixture,
    DivergentUniverseEntry, DivergentUniverseInputSnapshot, DivergentUniverseLoadoutSnapshotDigest,
};

#[test]
fn decision_digest_changes_config_and_canonical_activity_state() {
    let fixture = DivergentUniverseBaselineFixture::production().expect("production fixture");
    let factory = fixture.factory();
    let snapshot = DivergentUniverseInputSnapshot::seal(
        DivergentUniverseAccountSnapshotDigest::new([0x22; 32]).expect("account digest"),
        DivergentUniverseLoadoutSnapshotDigest::new([0x44; 32]).expect("loadout digest"),
        fixture.participants(),
    );
    let entry = DivergentUniverseEntry::new(
        DivergentUniverseAreaId::new("divergent-universe.area.401").expect("area ID"),
        DivergentUniverseDifficultyId::new("divergent-universe.difficulty.3011")
            .expect("difficulty ID"),
        Arc::clone(fixture.participants()),
        snapshot,
        Vec::new(),
    )
    .expect("entry");
    let flow = factory
        .compile(entry.clone())
        .expect("compile production entry");
    let area = factory
        .bundle
        .catalog()
        .area(&entry.area)
        .expect("production area");
    let original = compile_identity(
        &factory.bundle,
        factory.decision_catalog().digest(),
        area,
        &entry,
    );
    assert_eq!(flow.definition().identity(), original);
    let mut changed_digest = factory.decision_catalog().digest();
    changed_digest[0] ^= 1;
    let changed = compile_identity(&factory.bundle, changed_digest, area, &entry);
    assert_eq!(original.definition_digest(), changed.definition_digest());
    assert_ne!(original.config_digest(), changed.config_digest());

    // Hold graph, initial values, participants, instance and seed constant.
    let definition = flow.definition();
    let changed_definition = GraphActivityDefinition::new(
        changed,
        definition.graph().clone(),
        definition.state_definition().clone(),
        Arc::clone(definition.participants()),
        definition.programs().to_vec(),
        definition.bootstrap().cloned(),
        ActivityRandomPolicies::new(
            definition.random_checkpoints().to_vec(),
            definition.random_offers().to_vec(),
        ),
    )
    .expect("same graph with changed config dependency");
    let instance = ActivityInstanceId::new(23_600).expect("instance");
    let seed = ActivityMasterSeed::from_u64(23_600);
    let original_activity = flow
        .start(instance, seed)
        .expect("original start")
        .into_activity();
    let changed_activity = GraphActivity::start(Arc::new(changed_definition), instance, seed)
        .expect("changed start")
        .into_activity();
    assert_ne!(
        original_activity.state_hash(),
        changed_activity.state_hash()
    );
    assert_ne!(
        original_activity.canonical_state_bytes(),
        changed_activity.canonical_state_bytes()
    );
    let reconstructed = flow
        .start(instance, seed)
        .expect("fresh start")
        .into_activity();
    assert_eq!(
        original_activity.canonical_state_bytes(),
        reconstructed.canonical_state_bytes()
    );
}

#[test]
fn both_run_families_bind_decisions_in_exact_replay_components() {
    let fixture = DivergentUniverseBaselineFixture::production().expect("production fixture");
    let factory = fixture.factory();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).expect("production flow");
        let components = fixture.components(&flow).expect("component manifest");
        let content = components
            .components()
            .iter()
            .find(|item| item.kind() == ConfigurationComponentKind::ModeContent)
            .expect("mode content");
        assert_eq!(
            content.digest().bytes(),
            factory.decision_catalog().digest()
        );
        let reference_only = ConfigurationComponentIdentity::new(
            content.kind(),
            content.id(),
            ComponentDigest::new(factory.bundle_identity().component_digest().bytes()),
        )
        .expect("reference-only component");
        let mismatched = ConfigurationComponentSet::new(
            components
                .components()
                .iter()
                .map(|item| {
                    if item.kind() == ConfigurationComponentKind::ModeContent {
                        reference_only.clone()
                    } else {
                        item.clone()
                    }
                })
                .collect(),
        )
        .expect("ordered components");
        let mismatch = components
            .verify_exact(&mismatched)
            .expect_err("must bind decision inputs");
        assert_eq!(mismatch.expected.as_ref(), Some(content));
        assert_ne!(components.root(), mismatched.root());
    }
}
