use starclock_activity::{
    ActivityHandlerBundle, ActivityHandlerFault, ActivityHandlerId, ActivityHandlerInput,
    ActivityHandlerOutput, ActivityHandlerRegistration, ActivityHandlerRegistry,
    ActivityHandlerRegistryError,
};

fn empty(_input: ActivityHandlerInput<'_>) -> Result<ActivityHandlerOutput, ActivityHandlerFault> {
    Ok(ActivityHandlerOutput::new(Vec::new()))
}

fn registration(id: u32, key: &'static str, schema: u8) -> ActivityHandlerRegistration {
    ActivityHandlerRegistration::new(
        ActivityHandlerId::new(id).unwrap(),
        key,
        "v1",
        [schema; 32],
        "ordered-input-no-rng",
        "test",
        empty,
    )
}

#[test]
fn composed_registry_is_canonical_and_lookup_is_stable() {
    let mode = ActivityHandlerBundle::new(
        "starclock.mode.test",
        "mode-v1",
        vec!["starclock.core"],
        vec![registration(20, "test.second", 2)],
    )
    .unwrap();
    let core = ActivityHandlerBundle::new(
        "starclock.core",
        "core-v1",
        vec![],
        vec![registration(10, "test.first", 1)],
    )
    .unwrap();

    let registry = ActivityHandlerRegistry::compose(vec![mode.clone(), core.clone()]).unwrap();
    let reversed = ActivityHandlerRegistry::compose(vec![core, mode]).unwrap();
    assert_eq!(registry.digest(), reversed.digest());
    assert_eq!(
        registry
            .handler(ActivityHandlerId::new(20).unwrap())
            .unwrap()
            .stable_key(),
        "test.second"
    );
    assert_eq!(registry.bundles()[0].id(), "starclock.core");
}

#[test]
fn composition_rejects_duplicates_and_invalid_dependency_direction() {
    let duplicate = ActivityHandlerBundle::new(
        "starclock.core",
        "core-v1",
        vec![],
        vec![registration(10, "test.first", 1)],
    )
    .unwrap();
    assert_eq!(
        ActivityHandlerRegistry::compose(vec![duplicate.clone(), duplicate]).unwrap_err(),
        ActivityHandlerRegistryError::DuplicateBundle
    );

    let first =
        ActivityHandlerBundle::new("a", "v1", vec!["z"], vec![registration(1, "test.a", 1)])
            .unwrap();
    let last =
        ActivityHandlerBundle::new("z", "v1", vec![], vec![registration(2, "test.z", 2)]).unwrap();
    assert_eq!(
        ActivityHandlerRegistry::compose(vec![first, last]).unwrap_err(),
        ActivityHandlerRegistryError::InvalidDependency
    );
}

#[test]
fn digest_covers_schema_revision_and_bundle_dependencies() {
    let base = ActivityHandlerRegistry::compose(vec![
        ActivityHandlerBundle::new(
            "starclock.core",
            "core-v1",
            vec![],
            vec![registration(10, "test.first", 1)],
        )
        .unwrap(),
    ])
    .unwrap();
    let changed = ActivityHandlerRegistry::compose(vec![
        ActivityHandlerBundle::new(
            "starclock.core",
            "core-v2",
            vec![],
            vec![registration(10, "test.first", 2)],
        )
        .unwrap(),
    ])
    .unwrap();
    assert_ne!(base.digest(), changed.digest());
}
