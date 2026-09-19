use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe_decisions::{DecisionCatalog, validation};
use crate::divergent_universe_domain_decks::{DomainCardKind, DomainDeckAdmission};

fn source() -> SoraConfig {
    SoraConfig::from_source(
        &SoraBundle::parse(include_bytes!(
            "../../../config/divergent-universe-decisions-generated/config.sora"
        ))
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn domain_decks_retain_source_multiplicity_and_distinct_authored_instances() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    assert_eq!(catalog.domain_decks().len(), 9);
    let mut instances = BTreeSet::new();
    for deck in catalog.domain_decks() {
        assert_eq!(
            deck.admission,
            DomainDeckAdmission::VersionedProjectPolicyExplicitSourceDeck
        );
        assert_ne!(deck.style_source.as_ref(), "901");
        for (index, card) in deck.cards.iter().enumerate() {
            assert!(instances.insert(card.instance));
            assert_eq!(usize::from(card.ordinal), index + 1);
        }
    }
    assert_eq!(instances.len(), 125);
    let gladiator = catalog
        .domain_decks()
        .iter()
        .find(|deck| deck.key.as_ref() == "du.domain-deck.gladiator")
        .unwrap();
    assert_eq!(
        gladiator
            .cards
            .iter()
            .map(|card| card.preset_source.as_ref())
            .collect::<Vec<_>>(),
        [
            "1005", "1005", "1006", "1007", "1007", "1008", "1009", "1009", "1010", "1011", "1017",
            "1020", "1025", "1023"
        ]
    );
    assert_eq!(
        (gladiator.cards[12].kind, gladiator.cards[12].level),
        (DomainCardKind::Elite, 2)
    );
    assert_ne!(gladiator.cards[0].instance, gladiator.cards[1].instance);
}

#[test]
fn domain_decks_reject_broken_order_binding_and_provenance() {
    let reference = load_divergent_universe_bundle().unwrap();
    let config = source();
    let cards =
        serde_json::to_value(config.du_domain_cards().ordered_rows().collect::<Vec<_>>()).unwrap();
    let decks =
        serde_json::to_value(config.du_domain_decks().ordered_rows().collect::<Vec<_>>()).unwrap();
    let sources = serde_json::to_value(
        config
            .du_decision_sources()
            .ordered_rows()
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let reject = |table, rows| {
        let result = SoraConfig::from_source(&EditedRows(BTreeMap::from([(table, rows)])));
        assert!(result.is_err() || validation::compile(&result.unwrap(), &reference).is_err());
    };
    for (table, baseline, field, value) in [
        ("DuDomainCards", &cards, "ordinal", json!(2)),
        ("DuDomainCards", &cards, "id", json!(0)),
        ("DuDomainCards", &cards, "deck_id", json!(999)),
        ("DuDomainCards", &cards, "level", json!(5)),
        ("DuDomainCards", &cards, "preset_source", json!("9001")),
        (
            "DuDomainCards",
            &cards,
            "stable_key",
            json!("du.domain-deck.camera.card-1"),
        ),
        (
            "DuDomainCards",
            &cards,
            "source_locator",
            json!("unreviewed"),
        ),
        ("DuDomainCards", &cards, "source_ids", json!([67])),
        ("DuDomainDecks", &decks, "style_source", json!("901")),
        ("DuDomainDecks", &decks, "card_count", json!(13)),
        ("DuDomainDecks", &decks, "policy_note", json!("")),
        ("DuDomainDecks", &decks, "replacement_condition", json!("")),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        reject(table, rows);
    }
    for (table, mut rows) in [("DuDomainCards", cards), ("DuDomainDecks", decks)] {
        rows.as_array_mut().unwrap().pop();
        reject(table, rows);
    }
    let mut rows = sources;
    let source = rows
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == json!(67))
        .unwrap();
    source["sha256"] = Value::String("0".repeat(64));
    reject("DuDecisionSources", rows);
}
