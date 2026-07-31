use std::sync::{Arc, OnceLock};

use starclock_mode_universe::{
    catalog::UniverseCatalog,
    id::ServiceId,
    progression::{ServiceKind, ServiceProfileOwner},
    run_runtime::RunRuntimeCatalog,
    service_effect_runtime::{
        RespiteOfferKind, SERVICE_EFFECT_RUNTIME_REVISION, ServiceAction,
        ServiceEffectRuntimeCatalog, TrailblazeBonusEffect, TrailblazeBonusTier,
    },
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] =
    include_bytes!("../../../../../config/universe-generated/config.sora");

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
        "standard-universe-service-effect-runtime-v2"
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
            121, 232, 229, 140, 133, 193, 178, 65, 109, 57, 62, 72, 105, 64, 5, 74, 69, 170, 255,
            172, 0, 161, 242, 198, 144, 155, 195, 157, 182, 121, 193, 123,
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
    assert!(matches!(
        bonus.action(),
        ServiceAction::GrantTrailblazeBonus {
            offer_pool_key,
            source_event_id: 100001,
            tier: TrailblazeBonusTier::Ordinary,
            position: 1,
            effect: TrailblazeBonusEffect::AddFragments { amount: 100 },
        } if offer_pool_key.as_ref() == "universe.pool.trailblaze-bonuses"
    ));
    for suffix in (101_u32..=106)
        .chain(201..=205)
        .chain(401..=432)
        .chain(501..=530)
    {
        let key = format!("universe.service.trailblaze-bonus.{suffix}");
        let definition = catalog().service(service(&key)).unwrap();
        let owner = match suffix {
            101..=106 => ServiceProfileOwner::SwarmDisaster,
            201..=205 => ServiceProfileOwner::GoldAndGears,
            401..=432 | 501..=530 => ServiceProfileOwner::DivergentUniverse,
            _ => unreachable!("closed expansion suffix set"),
        };
        assert_eq!(definition.profile_owner(), owner, "{key}");
        assert_eq!(
            runtime.execute(definition.id()).unwrap().action(),
            &ServiceAction::ProfileExcluded {
                owner,
                source_event_id: definition.source_event_id().unwrap(),
            },
            "{key}"
        );
    }
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
