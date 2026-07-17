use std::collections::BTreeMap;

use starclock_combat::{
    ActionId, ModifierDefinitionId, ModifierInstanceId, Scalar, SourceDefinitionId,
    StateSlotDefinitionId, UnitId,
    modifier::model::{
        ActionTargetLedger, ActiveModifier, FormulaPurpose, ModifierQueryContext, StatKind,
        StatQuery,
    },
    modifier::resolve::StatResolver,
    rule::model::{RuleValue, SourceClass},
};

use crate::catalog::{LoadMode, load_with_mode};

const ASTA_PROBE: &[u8] = include_bytes!("../../../config/probes/v1a/asta-modifier/config.sora");

#[test]
fn asta_dynamic_team_aura_tracks_one_charging_instance() {
    let catalog = load_with_mode(ASTA_PROBE, LoadMode::Fixture).expect("Asta probe must lower");
    let asta = unit(1);
    let ally = unit(2);
    let slot = StateSlotDefinitionId::new(3).unwrap();
    let mut instances = vec![ActiveModifier {
        instance: ModifierInstanceId::new(1).unwrap(),
        definition: ModifierDefinitionId::new(4).unwrap(),
        owner: asta,
        subject: ally,
        source: SourceDefinitionId::new(1).unwrap(),
        source_class: SourceClass::Ability,
        insertion_sequence: 1,
        application_action: None,
        slots: vec![(slot, RuleValue::Integer(0))].into_boxed_slice(),
        captured_value: None,
        captured_stats: Box::new([]),
    }];
    let bases = BTreeMap::from([((ally, StatKind::Atk), Scalar::from_scaled(1_000_000_000))]);
    let query = StatQuery {
        subject: ally,
        stat: StatKind::Atk,
        purpose: FormulaPurpose::Stat,
    };
    for (stacks, expected) in [
        (0, 1_000_000_000),
        (1, 1_140_000_000),
        (5, 1_700_000_000),
        (3, 1_420_000_000),
    ] {
        assert!(instances[0].set_slot(slot, RuleValue::Integer(stacks)));
        let actual = StatResolver::new(catalog.modifiers(), &bases, &instances)
            .query(query, &ModifierQueryContext::default())
            .unwrap();
        assert_eq!(actual.scaled(), expected);
        assert_eq!(
            instances.len(),
            1,
            "stack changes must not replace the aura instance"
        );
    }
}

#[test]
fn production_loader_rejects_the_nonproduction_asta_probe() {
    let error = crate::catalog::load(ASTA_PROBE).expect_err("probe cannot enter production");
    assert_eq!(error.kind(), crate::catalog::CatalogLoadErrorKind::Metadata);
}

#[test]
fn asta_credit_is_distinct_per_action_and_uses_hit_time_weakness() {
    let _catalog = load_with_mode(ASTA_PROBE, LoadMode::Fixture).expect("Asta probe must lower");
    let action = ActionId::new(41).unwrap();
    let [first, second, third] = [unit(11), unit(12), unit(13)];
    let mut ledger = ActionTargetLedger::default();
    let hits = [
        (first, false),
        (second, true),
        (first, true),
        (third, false),
    ];
    let credits = hits.map(|(target, fire_weak_at_hit)| {
        ledger.credit(action, target, 1, u16::from(fire_weak_at_hit), 5)
    });
    assert_eq!(credits, [1, 2, 0, 1]);
    assert_eq!(ledger.len(), 3);
    ledger.clear_action(action);
    assert_eq!(ledger.credit(action, first, 1, 1, 5), 2);
}

fn unit(value: u64) -> UnitId {
    UnitId::new(value).unwrap()
}
