use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::{EditedRows, SoraBundle, SoraConfig, load_divergent_universe_bundle};
use crate::divergent_universe_curio_catalog::DivergentUniverseCurioCategory;
use crate::divergent_universe_decisions::{
    BUNDLE, DecisionCatalog,
    shop::{ShopStockId, ShopStockPolicy, ShopStockReward},
    validation,
};

fn source() -> SoraConfig {
    SoraConfig::from_source(&SoraBundle::parse(BUNDLE).unwrap()).unwrap()
}

fn items(config: &SoraConfig) -> Value {
    serde_json::to_value(config.du_shop_items().ordered_rows().collect::<Vec<_>>()).unwrap()
}

fn compile_rows(table: &'static str, rows: Value) -> Result<DecisionCatalog, String> {
    let reference = load_divergent_universe_bundle().unwrap();
    let config = SoraConfig::from_source(&EditedRows(BTreeMap::from([(table, rows)])))
        .map_err(|error| format!("{error:?}"))?;
    validation::compile(&config, &reference).map_err(|error| format!("{error:?}"))
}

#[test]
fn shop_stock_production_sora_preserves_policy_prices_rewards_and_identity_sources() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    let [stock] = catalog.shop_stocks() else {
        panic!("one authored policy stock")
    };
    assert_eq!(stock.id.as_str(), "du.shop-stock.acquisition-policy");
    assert_eq!(
        stock.policy,
        ShopStockPolicy::VersionedProjectPolicyExplicitFixedStockAndPriceBaseRewards
    );
    assert!(stock.policy_note.contains("not observed parity"));
    assert!(stock.replacement_condition.contains("merchant selectors"));
    assert_eq!(
        stock
            .items
            .iter()
            .map(|item| (item.ordinal, item.price))
            .collect::<Vec<_>>(),
        [(1, 50), (2, 70), (3, 40)]
    );
    assert!(
        matches!(&stock.items[0].reward, ShopStockReward::Blessing(id) if id.as_str() == "divergent-universe.blessing.615130")
    );
    assert!(
        matches!(&stock.items[1].reward, ShopStockReward::Curio(id) if id.as_str() == "divergent-universe.curio-state.9043")
    );
    assert!(
        matches!(&stock.items[2].reward, ShopStockReward::Curio(id) if id.as_str() == "divergent-universe.curio-state.9053")
    );
    assert_eq!(
        stock.items[0].sources[0].as_ref(),
        "du.source.battle-blessing-catalog"
    );
    assert_eq!(stock.items[1].sources.len(), 3);
    assert_eq!(stock.items[2].sources.len(), 3);
}

#[test]
fn shop_stock_prices_require_positive_canonical_i64_range_integer_text() {
    let config = source();
    for price in [
        "0",
        "-1",
        "+1",
        "01",
        "1.0",
        "1e2",
        " 1",
        "1 ",
        "9223372036854775808",
        "18446744073709551616",
    ] {
        let mut rows = items(&config);
        rows[0]["price"] = json!(price);
        assert!(compile_rows("DuShopItems", rows).is_err(), "{price}");
    }
    for price in ["1", "9223372036854775807"] {
        let mut rows = items(&config);
        rows[0]["price"] = json!(price);
        let catalog = compile_rows("DuShopItems", rows).unwrap();
        assert_eq!(catalog.shop_stocks()[0].items[0].price.to_string(), price);
    }
}

#[test]
fn shop_stock_rejects_bad_ordinals_references_owners_and_reward_provenance() {
    let config = source();
    let baseline = items(&config);
    for (field, value) in [
        ("id", json!(0)),
        ("stable_key", json!("UPPER")),
        ("ordinal", json!(2)),
        ("ordinal", json!(65)),
        ("stock_id", json!(999)),
        ("reward_key", json!("divergent-universe.blessing.missing")),
        ("source_ids", json!([])),
        ("source_ids", json!([15, 15])),
        ("source_ids", json!([999])),
        ("source_ids", json!([1])),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        assert!(compile_rows("DuShopItems", rows).is_err(), "{field}");
    }
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = reference.curio_catalog();
    let negative = catalog
        .states()
        .iter()
        .find(|state| {
            state.category == DivergentUniverseCurioCategory::Negative && state.curio.is_some()
        })
        .unwrap();
    let unbound = catalog
        .states()
        .iter()
        .find(|state| state.curio.is_none())
        .unwrap();
    for id in [
        negative.id.as_str(),
        unbound.id.as_str(),
        "divergent-universe.curio-state.9043",
        "divergent-universe.curio-state.missing",
    ] {
        let mut rows = baseline.clone();
        rows[2]["reward_key"] = json!(id);
        assert!(compile_rows("DuShopItems", rows).is_err(), "{id}");
    }
    let mut rows = baseline.clone();
    rows[1]["reward_kind"] = rows[0]["reward_kind"].clone();
    rows[1]["reward_key"] = rows[0]["reward_key"].clone();
    rows[1]["source_ids"] = rows[0]["source_ids"].clone();
    assert!(compile_rows("DuShopItems", rows).is_err());
    assert!(compile_rows("DuShopItems", json!([])).is_err());
}

#[test]
fn shop_stock_requires_policy_metadata_and_canonical_key_namespace() {
    let config = source();
    let baseline =
        serde_json::to_value(config.du_shop_stocks().ordered_rows().collect::<Vec<_>>()).unwrap();
    for (field, value) in [
        ("id", json!(0)),
        ("stable_key", json!("du.shop-stock.")),
        ("stable_key", json!("du.other-stock.example")),
        ("policy", json!("Exact")),
        ("summary_en", json!(" ")),
        ("summary_zh_cn", json!("")),
        ("policy_note", json!("")),
        ("replacement_condition", json!("")),
    ] {
        let mut rows = baseline.clone();
        rows[0][field] = value;
        assert!(compile_rows("DuShopStocks", rows).is_err(), "{field}");
    }
    for key in [
        "du.shop-stock.",
        "du.shop-stock.UPPER",
        "du.other-stock.example",
    ] {
        assert!(ShopStockId::new(key).is_err());
    }
    assert!(ShopStockId::new("du.shop-stock.not-yet-authored").is_ok());
}

#[test]
fn shop_stock_transport_row_order_does_not_change_sorted_domain_definitions() {
    let config = source();
    let baseline = items(&config);
    let mut reversed = baseline.clone();
    reversed.as_array_mut().unwrap().reverse();
    assert_eq!(
        compile_rows("DuShopItems", baseline).unwrap().shop_stocks(),
        compile_rows("DuShopItems", reversed).unwrap().shop_stocks()
    );
}
