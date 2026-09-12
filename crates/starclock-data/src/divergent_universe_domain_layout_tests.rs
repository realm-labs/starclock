use super::{EditedRows, SoraBundle, SoraConfig, SoraReadError, load_divergent_universe_bundle};
use crate::divergent_universe_decisions::{DecisionCatalog, validation};
use crate::divergent_universe_domain_layout::{DomainPositionKind, FixedDomainKind};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn domain_layout_covers_current_profile_exactly_without_random_membership() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    assert_eq!(catalog.domain_layout().len(), 11);
    assert_eq!(
        catalog
            .domain_layout()
            .iter()
            .map(|layer| layer.positions.len())
            .sum::<usize>(),
        60
    );
    let totals = reference
        .catalog()
        .areas()
        .iter()
        .map(|area| {
            let count = area
                .layers
                .iter()
                .map(|id| {
                    catalog
                        .domain_layout()
                        .iter()
                        .find(|layer| &layer.layer == id)
                        .unwrap()
                        .positions
                        .len()
                })
                .sum::<usize>();
            (area.id.as_str(), count)
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(totals.len(), 28);
    for (area, expected) in [
        ("103", 5),
        ("104", 5),
        ("401", 13),
        ("406", 17),
        ("409", 20),
        ("20401", 13),
        ("20406", 17),
        ("20409", 20),
    ] {
        assert_eq!(
            totals[format!("divergent-universe.area.{area}").as_str()],
            expected
        );
    }
    let mut fixed = 0;
    let mut unspecified = 0;
    for layer in catalog.domain_layout() {
        for position in &layer.positions {
            match position.kind {
                DomainPositionKind::Unspecified => unspecified += 1,
                DomainPositionKind::Fixed { .. } => fixed += 1,
            }
        }
        assert!(matches!(
            layer.positions.last().unwrap().kind,
            DomainPositionKind::Fixed {
                kind: FixedDomainKind::Boss,
                level: 1,
                ..
            }
        ));
    }
    assert_eq!((fixed, unspecified), (21, 39));
}

#[test]
fn domain_layout_rejects_historical_layers_missing_positions_and_preset_substitutions() {
    let reference = load_divergent_universe_bundle().unwrap();
    let bundle = SoraBundle::parse(include_bytes!(
        "../../../config/divergent-universe-decisions-generated/config.sora"
    ))
    .unwrap();
    let config = SoraConfig::from_source(&bundle).unwrap();
    let baseline =
        serde_json::to_value(config.du_domain_layout().ordered_rows().collect::<Vec<_>>()).unwrap();
    for (field, value) in [
        ("layer_key", json!("divergent-universe.layer.1101")),
        ("ordinal", json!(0)),
        ("ordinal", json!(2)),
        ("preset_source", json!("1001")),
        ("level", json!(0)),
        ("kind", json!("Boss")),
        ("source_locator", json!("wrong")),
        ("interpretation_note", json!("")),
        ("replacement_condition", json!("")),
        ("source_ids", json!([18, 39])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        let source = EditedRows(BTreeMap::from([("DuDomainLayout", rows)]));
        let result = SoraConfig::from_source(&source).and_then(|config| {
            validation::compile(&config, &reference)
                .map(|_| ())
                .map_err(|error| SoraReadError::new(error.to_string()))
        });
        assert!(result.is_err(), "accepted invalid layout {field}");
    }
    for missing in [0, 4, 59] {
        let mut rows = baseline.clone();
        rows.as_array_mut().unwrap().remove(missing);
        let config =
            SoraConfig::from_source(&EditedRows(BTreeMap::from([("DuDomainLayout", rows)])))
                .unwrap();
        assert!(validation::compile(&config, &reference).is_err());
    }
}
