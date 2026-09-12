//! Semantic policy corruption vectors share the production table fixture.

use super::{EditedRows, SoraConfig, compile};
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_decisions::{
        BattleRewardDomain, CurioBattleStat, CurioVictoryBlessingPolicy, DecisionCatalog,
        DecisionDataError,
    },
};
use serde_json::Value;
use std::collections::BTreeMap;

pub(super) fn battle_stats_table(config: &SoraConfig) -> (&'static str, Value) {
    (
        "DuCurioBattleStats",
        serde_json::to_value(
            config
                .du_curio_battle_stats()
                .ordered_rows()
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    )
}

pub(super) fn production_battle_stats(catalog: &DecisionCatalog) {
    let victory = &catalog.curio_victory_blessings()[0];
    assert_eq!(
        victory.state.as_str(),
        "divergent-universe.curio-state.9072"
    );
    assert_eq!(victory.effect_id.as_ref(), "2072");
    assert_eq!(victory.count, 1);
    assert_eq!((victory.rarity.minimum(), victory.rarity.maximum()), (1, 3));
    assert_eq!(
        victory.domains.as_ref(),
        &[
            BattleRewardDomain::Combat,
            BattleRewardDomain::Elite,
            BattleRewardDomain::Aberration,
            BattleRewardDomain::Boss
        ]
    );
    assert_eq!(victory.policy, CurioVictoryBlessingPolicy::VersionedProjectPolicyPositiveDomainAllowanceUniformUnownedAvailableSubset);
    let [grant] = catalog.curio_domain_grants() else {
        panic!("one reviewed domain grant")
    };
    assert_eq!(grant.state.as_str(), "divergent-universe.curio-state.9071");
    assert_eq!(grant.amount, 60);
    assert_eq!(catalog.tawot_services().len(), 4);
    for (service, (level, limit)) in
        catalog
            .tawot_services()
            .iter()
            .zip([(2, 1), (3, 1), (4, 2), (5, 2)])
    {
        assert_eq!(service.forge_level, level);
        assert_eq!(service.purchase_limit, limit);
        assert_eq!(service.fragment_cost, 100);
        assert_eq!(service.offer_width, 3);
        assert_eq!(
            service.occurrence.as_str(),
            "divergent-universe.occurrence.181"
        );
        assert_eq!(
            service.variant.as_str(),
            "divergent-universe.occurrence-variant.700108"
        );
        assert_eq!(service.states.len(), 12);
        assert_eq!(
            service.states.first().unwrap().as_str(),
            "divergent-universe.curio-state.9068"
        );
        assert_eq!(
            service.states.last().unwrap().as_str(),
            "divergent-universe.curio-state.9079"
        );
    }
    let [reaction] = catalog.curio_battle_reactions() else {
        panic!("one reviewed reaction")
    };
    assert_eq!(
        reaction.state.as_str(),
        "divergent-universe.curio-state.9069"
    );
    assert_eq!(reaction.effect_id.as_ref(), "2069");
    assert_eq!(reaction.heal_fraction.as_ref(), "0.2");
    assert_eq!(reaction.battle_limit, 5);
    let [stat, damage] = catalog.curio_battle_stats() else {
        panic!("two reviewed passives")
    };
    assert_eq!(stat.state.as_str(), "divergent-universe.curio-state.9068");
    assert_eq!(stat.effect_id.as_ref(), "2068");
    assert_eq!(stat.stat, CurioBattleStat::Speed);
    assert_eq!(stat.bonus_fraction.as_ref(), "0.35");
    assert_eq!(stat.battle_limit, 5);
    assert_eq!(damage.state.as_str(), "divergent-universe.curio-state.9073");
    assert_eq!(damage.effect_id.as_ref(), "2073");
    assert_eq!(damage.stat, CurioBattleStat::FinalDamage);
    assert_eq!(damage.bonus_fraction.as_ref(), "0.5");
    assert_eq!(damage.battle_limit, 5);
}

pub(super) fn battle_reactions_table(config: &SoraConfig) -> (&'static str, Value) {
    (
        "DuCurioBattleReactions",
        serde_json::to_value(
            config
                .du_curio_battle_reactions()
                .ordered_rows()
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    )
}

pub(super) fn tawot_services_table(config: &SoraConfig) -> (&'static str, Value) {
    (
        "DuTawotServices",
        serde_json::to_value(
            config
                .du_tawot_services()
                .ordered_rows()
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    )
}

pub(super) fn validate(
    baseline: &BTreeMap<&'static str, Value>,
    reference: &DivergentUniverseBundleCandidate,
) {
    for (field, value, error) in [
        ("count", Value::from(2), DecisionDataError::InvalidReference),
        (
            "policy",
            Value::from("BoundDomainUniformUnownedAvailableSubset"),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "count_parameter",
            Value::from(2),
            DecisionDataError::InvalidReference,
        ),
        (
            "minimum_rarity",
            Value::from(2),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "domains",
            serde_json::json!(["Combat", "Elite", "Aberration"]),
            DecisionDataError::InvalidPolicy,
        ),
    ] {
        let mut rows = EditedRows(baseline.clone());
        rows.0.get_mut("DuCurioVictoryBlessings").unwrap()[3][field] = value;
        assert_eq!(
            compile(&SoraConfig::from_source(&rows).unwrap(), reference),
            Err(error)
        );
    }
    let mut missing_expiry = EditedRows(baseline.clone());
    missing_expiry
        .0
        .get_mut("DuCurioDomainExpiries")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .pop();
    assert_eq!(
        compile(
            &SoraConfig::from_source(&missing_expiry).unwrap(),
            reference
        ),
        Err(DecisionDataError::InvalidReference)
    );
    for (field, value, error) in [
        (
            "amount",
            Value::from(61),
            DecisionDataError::InvalidReference,
        ),
        ("amount", Value::from(0), DecisionDataError::InvalidReward),
        (
            "amount_parameter",
            Value::from(2),
            DecisionDataError::InvalidReference,
        ),
        (
            "expiry_id",
            Value::from(1),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "policy_note",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "replacement_condition",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "source_ids",
            Value::from(Vec::<i32>::new()),
            DecisionDataError::InvalidProvenance,
        ),
    ] {
        let mut rows = EditedRows(baseline.clone());
        rows.0.get_mut("DuCurioDomainGrants").unwrap()[0][field] = value;
        assert_eq!(
            compile(&SoraConfig::from_source(&rows).unwrap(), reference),
            Err(error)
        );
    }
    let mut missing = EditedRows(baseline.clone());
    missing
        .0
        .insert("DuCurioDomainGrants", Value::from(Vec::<i32>::new()));
    assert_eq!(
        compile(&SoraConfig::from_source(&missing).unwrap(), reference),
        Err(DecisionDataError::InvalidReference)
    );
    for (row, field, value) in [
        (0, "stat", Value::from("FinalDamage")),
        (1, "stat", Value::from("Speed")),
        (
            1,
            "policy",
            Value::from("BasePercentVerifiedBattleLifetime"),
        ),
    ] {
        let mut rows = EditedRows(baseline.clone());
        rows.0.get_mut("DuCurioBattleStats").unwrap()[row][field] = value;
        assert_eq!(
            compile(&SoraConfig::from_source(&rows).unwrap(), reference),
            Err(DecisionDataError::InvalidPolicy)
        );
    }
    for (field, value, error) in [
        ("id", Value::from(0), DecisionDataError::InvalidIdentity),
        (
            "forge_level",
            Value::from(1),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "forge_level",
            Value::from(6),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "forge_level",
            Value::from(3),
            DecisionDataError::InvalidIdentity,
        ),
        (
            "fragment_cost",
            Value::from(0),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "offer_width",
            Value::from(0),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "offer_width",
            Value::from(13),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "purchase_limit",
            Value::from(0),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "purchase_limit",
            Value::from(9),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "occurrence_key",
            Value::from("divergent-universe.occurrence.108"),
            DecisionDataError::InvalidReference,
        ),
        (
            "variant_key",
            Value::from("divergent-universe.occurrence-variant.722601"),
            DecisionDataError::InvalidReference,
        ),
        (
            "curio_key",
            Value::from("divergent-universe.curio.missing"),
            DecisionDataError::InvalidReference,
        ),
        (
            "summary_en",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "summary_zh_cn",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "policy_note",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "replacement_condition",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
    ] {
        reject(baseline, reference, "DuTawotServices", field, value, error);
    }
    for (field, value, error) in [
        ("id", Value::from(0), DecisionDataError::InvalidReference),
        (
            "state_key",
            Value::from("divergent-universe.curio-state.9068"),
            DecisionDataError::InvalidReference,
        ),
        (
            "effect_id",
            Value::from("2068"),
            DecisionDataError::InvalidReference,
        ),
        (
            "heal_parameter",
            Value::from(2),
            DecisionDataError::InvalidReference,
        ),
        (
            "heal_fraction",
            Value::from("0.35"),
            DecisionDataError::InvalidReference,
        ),
        (
            "limit_parameter",
            Value::from(1),
            DecisionDataError::InvalidReference,
        ),
        (
            "battle_limit",
            Value::from(3),
            DecisionDataError::InvalidReference,
        ),
        (
            "battle_limit",
            Value::from(0),
            DecisionDataError::InvalidReward,
        ),
        (
            "battle_limit",
            Value::from(65536),
            DecisionDataError::InvalidReward,
        ),
        (
            "summary_en",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "summary_zh_cn",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "policy_note",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "replacement_condition",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
    ] {
        reject(
            baseline,
            reference,
            "DuCurioBattleReactions",
            field,
            value,
            error,
        );
    }
    for (field, value, error) in [
        ("id", Value::from(0), DecisionDataError::InvalidReference),
        (
            "state_key",
            Value::from("divergent-universe.curio-state.9070"),
            DecisionDataError::InvalidReference,
        ),
        (
            "effect_id",
            Value::from("2070"),
            DecisionDataError::InvalidReference,
        ),
        (
            "bonus_parameter",
            Value::from(2),
            DecisionDataError::InvalidReference,
        ),
        (
            "bonus_fraction",
            Value::from("0.5"),
            DecisionDataError::InvalidReference,
        ),
        (
            "limit_parameter",
            Value::from(1),
            DecisionDataError::InvalidReference,
        ),
        (
            "battle_limit",
            Value::from(3),
            DecisionDataError::InvalidReference,
        ),
        (
            "battle_limit",
            Value::from(0),
            DecisionDataError::InvalidReward,
        ),
        (
            "battle_limit",
            Value::from(65536),
            DecisionDataError::InvalidReward,
        ),
        (
            "policy_note",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "replacement_condition",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "summary_en",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "summary_zh_cn",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
    ] {
        reject(
            baseline,
            reference,
            "DuCurioBattleStats",
            field,
            value,
            error,
        );
    }
    for (field, value, error) in [
        ("id", Value::from(0), DecisionDataError::InvalidReference),
        (
            "state_key",
            Value::from("divergent-universe.curio-state.9079"),
            DecisionDataError::InvalidReference,
        ),
        (
            "effect_id",
            Value::from("2079"),
            DecisionDataError::InvalidReference,
        ),
        (
            "limit_parameter",
            Value::from(1),
            DecisionDataError::InvalidReference,
        ),
        (
            "domain_limit",
            Value::from(5),
            DecisionDataError::InvalidReference,
        ),
        (
            "domain_limit",
            Value::from(0),
            DecisionDataError::InvalidReward,
        ),
        (
            "domain_limit",
            Value::from(65536),
            DecisionDataError::InvalidReward,
        ),
        (
            "policy_note",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "replacement_condition",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "summary_en",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "summary_zh_cn",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
    ] {
        reject(
            baseline,
            reference,
            "DuCurioDomainExpiries",
            field,
            value,
            error,
        );
    }
    let mut duplicate = EditedRows(baseline.clone());
    let mut copied = duplicate.0["DuCurioDomainExpiries"][0].clone();
    copied["id"] = Value::from(3);
    copied["stable_key"] = Value::from("du.curio-domain-expiry.duplicate");
    duplicate
        .0
        .get_mut("DuCurioDomainExpiries")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .push(copied);
    assert_eq!(
        compile(&SoraConfig::from_source(&duplicate).unwrap(), reference),
        Err(DecisionDataError::InvalidIdentity)
    );
    for table in ["DuDomainChoices", "DuBattleFragments"] {
        for (field, value, expected) in [
            (
                "domain",
                Value::from("Boss"),
                DecisionDataError::InvalidPolicy,
            ),
            (
                "domain",
                Value::from("Elite"),
                DecisionDataError::InvalidIdentity,
            ),
            ("id", Value::from(0), DecisionDataError::InvalidIdentity),
            (
                "policy_note",
                Value::from(""),
                DecisionDataError::InvalidPolicy,
            ),
            (
                "replacement_condition",
                Value::from(""),
                DecisionDataError::InvalidPolicy,
            ),
            (
                "source_ids",
                Value::from(vec![999]),
                DecisionDataError::InvalidReference,
            ),
        ] {
            reject(baseline, reference, table, field, value, expected);
        }
        let mut rows = EditedRows(baseline.clone());
        rows.0.get_mut(table).unwrap().as_array_mut().unwrap().pop();
        assert_eq!(
            compile(&SoraConfig::from_source(&rows).unwrap(), reference),
            Err(DecisionDataError::InvalidPolicy)
        );
    }
    for field in ["name_en", "name_zh_cn"] {
        reject(
            baseline,
            reference,
            "DuDomainChoices",
            field,
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        );
    }
    reject(
        baseline,
        reference,
        "DuDomainChoices",
        "id",
        Value::from(4),
        DecisionDataError::InvalidIdentity,
    );
    for amount in [0, -1] {
        reject(
            baseline,
            reference,
            "DuBattleFragments",
            "amount",
            Value::from(amount),
            DecisionDataError::InvalidReward,
        );
    }
}

pub(super) fn domain_grants_table(config: &SoraConfig) -> (&'static str, Value) {
    (
        "DuCurioDomainGrants",
        serde_json::to_value(
            config
                .du_curio_domain_grants()
                .ordered_rows()
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    )
}

fn reject(
    baseline: &BTreeMap<&'static str, Value>,
    reference: &DivergentUniverseBundleCandidate,
    table: &'static str,
    field: &str,
    value: Value,
    expected: DecisionDataError,
) {
    let mut rows = EditedRows(baseline.clone());
    rows.0.get_mut(table).unwrap()[0][field] = value;
    assert_eq!(
        compile(&SoraConfig::from_source(&rows).unwrap(), reference),
        Err(expected),
        "{table}.{field}"
    );
}
