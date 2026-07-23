use std::sync::{Arc, OnceLock};

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    id::ServiceId,
    progression::ServiceKind,
    run_runtime::RunRuntimeCatalog,
    service_effect_runtime::{
        RespiteOfferKind, SERVICE_EFFECT_RUNTIME_REVISION, ServiceAction,
        ServiceEffectRuntimeCatalog,
    },
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

fn catalog() -> &'static UniverseCatalog {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    CATALOG
        .get_or_init(|| {
            let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core");
            UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe")
        })
        .as_ref()
}

fn runtime() -> ServiceEffectRuntimeCatalog {
    let run = RunRuntimeCatalog::compile(catalog()).expect("run");
    ServiceEffectRuntimeCatalog::compile(&run).expect("services")
}

fn service(key: &str) -> ServiceId {
    catalog()
        .services()
        .iter()
        .find(|value| value.stable_key() == key)
        .unwrap()
        .id()
}

#[test]
fn complete_service_partition_compiles() {
    let runtime = runtime();
    assert_eq!(
        SERVICE_EFFECT_RUNTIME_REVISION,
        "standard-universe-service-effect-runtime-v1"
    );
    assert_eq!(
        (
            runtime.content_count(),
            runtime.rule_count(),
            runtime.semantic_fixture_count()
        ),
        (94, 94, 9)
    );
    assert_eq!(runtime.service_ids().count(), 94);
    assert_eq!(
        runtime.digest(),
        [
            78, 82, 111, 239, 161, 29, 188, 122, 202, 235, 16, 119, 217, 43, 116, 145, 165, 139,
            251, 147, 78, 78, 23, 236, 214, 17, 135, 92, 216, 38, 136, 171,
        ]
    );
}

#[test]
fn every_service_executes_to_one_source_attributed_plan() {
    let runtime = runtime();
    for id in runtime.service_ids() {
        let effect = runtime.execute(id).unwrap();
        assert_eq!(effect.service(), id);
        assert!(!effect.source_key().is_empty());
        assert!(!effect.rule_key().is_empty());
    }
}

#[test]
fn currency_reset_reviver_and_downloader_values_are_exact() {
    let runtime = runtime();
    assert_eq!(
        runtime
            .execute(service("universe.currency.cosmic-fragments"))
            .unwrap()
            .action(),
        &ServiceAction::InitializeCurrency { amount: 50 }
    );
    let reset = runtime
        .execute(service("universe.service.reset-blessing-choice"))
        .unwrap();
    let ServiceAction::ResetBlessingOffer { cost_schedule, .. } = reset.action() else {
        panic!("reset action");
    };
    assert_eq!(
        cost_schedule
            .iter()
            .map(|step| (step.use_index(), step.amount()))
            .collect::<Vec<_>>(),
        [(1, 30), (2, 50), (3, 100)]
    );
    assert_eq!(
        reset.currency_key(),
        Some("universe.currency.cosmic-fragments")
    );
    assert_eq!(
        runtime
            .execute(service("universe.service.reviver"))
            .unwrap()
            .action(),
        &ServiceAction::ReviveCharacter {
            cost: 80,
            restored_hp_percent: 100
        }
    );
    assert_eq!(
        runtime
            .execute(service("universe.service.downloader"))
            .unwrap()
            .action(),
        &ServiceAction::AddReserveCharacter { amount: 1 }
    );
}

#[test]
fn respite_and_enhancement_choices_preserve_authored_prices() {
    let runtime = runtime();
    let respite = runtime
        .execute(service("universe.service.respite-offers"))
        .unwrap();
    let ServiceAction::OfferRespiteChoices { offers } = respite.action() else {
        panic!("respite action");
    };
    assert_eq!(offers.len(), 3);
    assert_eq!(
        offers
            .iter()
            .map(|offer| (offer.kind(), offer.amount(), offer.cost()))
            .collect::<Vec<_>>(),
        [
            (RespiteOfferKind::OneStarBlessing, 1, 80),
            (RespiteOfferKind::Curio, 1, 120),
            (RespiteOfferKind::EnhanceRandomBlessings, 2, 180)
        ]
    );
    assert_eq!(
        runtime
            .execute(service("universe.service.enhance-blessing"))
            .unwrap()
            .action(),
        &ServiceAction::EnhanceBlessing {
            maximum_enhancements: 1,
            rarity_costs: [100, 130, 160]
        }
    );
}

#[test]
fn all_shop_and_trailblaze_rows_retain_authored_external_bindings() {
    let runtime = runtime();
    let blessing = runtime
        .execute(service("universe.service.shop.100011"))
        .unwrap();
    assert!(matches!(
        blessing.action(),
        ServiceAction::OpenBlessingShop { price_formula_key, offer_pool_key }
            if price_formula_key.as_ref() == "universe.price.shop.100011"
                && offer_pool_key.as_ref() == "universe.pool.shop.100011"
    ));
    let curio = runtime
        .execute(service("universe.service.shop.100021"))
        .unwrap();
    assert!(matches!(
        curio.action(),
        ServiceAction::OpenCurioShop { price_formula_key, offer_pool_key }
            if price_formula_key.as_ref() == "universe.price.shop.100021"
                && offer_pool_key.as_ref() == "universe.pool.shop.100021"
    ));
    let bonus = runtime
        .execute(service("universe.service.trailblaze-bonus.1"))
        .unwrap();
    assert_eq!(
        bonus.action(),
        &ServiceAction::GrantTrailblazeBonus {
            offer_pool_key: "universe.pool.trailblaze-bonuses".into()
        }
    );
}

#[test]
fn nine_frozen_service_kind_fixtures_are_runtime_backed() {
    let runtime = runtime();
    let fixtures = [
        ("universe.currency.cosmic-fragments", ServiceKind::Currency),
        (
            "universe.service.reset-blessing-choice",
            ServiceKind::ResetBlessing,
        ),
        ("universe.service.reviver", ServiceKind::Reviver),
        ("universe.service.downloader", ServiceKind::Downloader),
        (
            "universe.service.respite-offers",
            ServiceKind::RespiteOffers,
        ),
        (
            "universe.service.enhance-blessing",
            ServiceKind::EnhanceBlessing,
        ),
        ("universe.service.shop.100011", ServiceKind::BlessingShop),
        ("universe.service.shop.100021", ServiceKind::CurioShop),
        (
            "universe.service.trailblaze-bonus.1",
            ServiceKind::TrailblazeBonus,
        ),
    ];
    for (key, kind) in fixtures {
        let definition = catalog().service(service(key)).unwrap();
        assert_eq!(definition.kind(), kind);
        assert_eq!(
            runtime.execute(definition.id()).unwrap().service(),
            definition.id()
        );
    }
}
