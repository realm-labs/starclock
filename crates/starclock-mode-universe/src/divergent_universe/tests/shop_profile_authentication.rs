//! Attachment validation and the controller's public command authority.

use super::{FAMILIES, LEAVE_SHOP, SLOTS, advance, compile, policy, ready, start, stock};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    shop_purchase::room::ShopRoomSlots, tests::battle_room::base,
};
use starclock_activity::{ActivityOptionId, ActivitySlotId, GraphActivityCommandError};

#[test]
fn shop_controller_rejects_empty_duplicate_repeated_foreign_price_and_slot_attachments() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = source.factory();
    let profile = compile(&source, FAMILIES[0], 250);
    assert!(
        factory
            .bind_position_shop_rooms(base(&source, FAMILIES[0]), &profile.rooms)
            .is_err()
    );
    assert!(
        factory
            .bind_position_shop_rooms(profile.unbound.clone(), &[])
            .is_err()
    );
    assert!(
        factory
            .bind_position_shop_rooms(
                profile.unbound.clone(),
                &[profile.rooms[0].clone(), profile.rooms[0].clone()]
            )
            .is_err()
    );
    assert!(
        factory
            .bind_position_shop_rooms(profile.flow.clone(), &profile.rooms)
            .is_err()
    );
    let foreign = compile(&source, FAMILIES[1], 250);
    assert!(
        factory
            .bind_position_shop_rooms(profile.unbound.clone(), &foreign.rooms)
            .is_err()
    );
    for (price, slots) in [
        (51, SLOTS),
        (
            50,
            ShopRoomSlots {
                accepted: ActivitySlotId::new(72).unwrap(),
                ..SLOTS
            },
        ),
    ] {
        let mut items = stock(&source);
        items[0].price = price;
        let changed = factory
            .shop_room_compiler(items, slots)
            .unwrap()
            .compile(profile.rooms[0].context())
            .unwrap();
        assert!(
            factory
                .bind_position_shop_rooms(profile.unbound.clone(), &[changed])
                .is_err()
        );
    }
    let mut reversed = profile.rooms.clone();
    reversed.reverse();
    let reordered = factory
        .bind_position_shop_rooms(profile.unbound.clone(), &reversed)
        .unwrap();
    let activity = ready(&source, &profile);
    assert!(reordered.offered_shop(&activity));
    assert_eq!(
        reordered.definition().identity(),
        profile.flow.definition().identity()
    );
    assert_eq!(
        start(&reordered).canonical_state_bytes(),
        start(&profile.flow).canonical_state_bytes()
    );
}

#[test]
fn shop_controller_raw_stale_hidden_foreign_and_unbound_choices_preserve_bytes_in_every_menu() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let profile = compile(&source, family, 250);
        let foreign = compile(&source, family, 251);
        let stale = start(&profile.flow).state_hash();
        let mut activity = ready(&source, &profile);
        let mut checked = 0;
        for _ in 0..4 {
            if !profile.flow.offered_shop(&activity) {
                break;
            }
            checked += 1;
            let before = activity.canonical_state_bytes();
            let offer = activity.player_view().decision().unwrap().clone();
            let hash = activity.state_hash();
            let option = offer.options()[0].id();
            assert_ne!(hash, stale);
            assert_eq!(
                profile
                    .flow
                    .choose_shop_option(&mut activity, stale, offer.id(), option),
                Err(GraphActivityCommandError::StaleStateHash)
            );
            assert!(
                profile
                    .flow
                    .choose_shop_option(
                        &mut activity,
                        hash,
                        offer.id(),
                        ActivityOptionId::new(64).unwrap()
                    )
                    .is_err()
            );
            for flow in [&profile.unbound, &foreign.flow] {
                assert!(!flow.offered_shop(&activity));
                assert_eq!(
                    flow.choose_shop_option(&mut activity, hash, offer.id(), option),
                    Err(GraphActivityCommandError::DecisionNotOffered)
                );
            }
            for raw in [option, ActivityOptionId::new(LEAVE_SHOP).unwrap()] {
                assert!(activity.choose_option(hash, offer.id(), raw).is_err());
            }
            assert!(
                DivergentUniverseBaselineRunner::default()
                    .advance(
                        source.factory(),
                        &profile.unbound,
                        &mut activity,
                        source.core(),
                        &policy(&source)
                    )
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            advance(&source, &profile.flow, &mut activity);
        }
        assert_eq!(checked, 4);
    }
}
