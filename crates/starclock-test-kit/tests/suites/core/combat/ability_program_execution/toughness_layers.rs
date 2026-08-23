use super::*;

#[test]
fn keyed_toughness_layer_create_and_remove_are_typed_idempotent_mutations() {
    let maximum = ValueExpr::Literal(RuleValue::Scalar(Scalar::checked_from_integer(40).unwrap()));
    let create = || {
        ProgramStep::Operation(RuleOperationTemplate::CreateToughnessLayer {
            selector: id(2),
            layer_key: "temporary-core".into(),
            maximum: maximum.clone(),
        })
    };
    let remove = || {
        ProgramStep::Operation(RuleOperationTemplate::RemoveToughnessLayer {
            selector: id(2),
            layer_key: "temporary-core".into(),
        })
    };
    let program = ProgramDefinition::new(id(1), vec![], vec![id(2)], vec![], vec![])
        .with_steps(vec![create(), create(), remove(), remove()]);
    let mut battle = battle(
        catalog(program, false, false, false, false),
        false,
        false,
        false,
    );

    let resolution = start_and_use(&mut battle).unwrap();

    assert!(resolution.fault().is_none());
    let events = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Toughness(value) => match value {
                starclock_combat::ToughnessEventData::LayerCreated {
                    layer_key, maximum, ..
                } => Some(("created", *layer_key, maximum.get(), maximum.get())),
                starclock_combat::ToughnessEventData::LayerRemoved {
                    layer_key,
                    current,
                    maximum,
                    ..
                } => Some(("removed", *layer_key, current.get(), maximum.get())),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        events,
        [
            ("created", 2, 40, 40),
            ("removed", 2, 40, 40),
            ("created", 2, 40, 40),
            ("removed", 2, 40, 40),
        ]
    );
    let target = battle.view().units_by_id().nth(1).unwrap();
    assert_eq!(target.toughness_layers().count(), 1);
}

#[test]
fn authored_toughness_layer_spec_retains_its_stable_key() {
    let layer = starclock_combat::ToughnessLayerSpec::ordinary(
        1,
        starclock_combat::RawToughness::new(30).unwrap(),
    )
    .unwrap()
    .with_stable_key("layer-1")
    .unwrap();
    assert_eq!(layer.stable_key(), Some("layer-1"));
}
