//! Admission is recomputed after a real purchase, without sampling the stock.

use super::{LEAVE_SHOP, Setup, choose, compile, mutate, options, reward_draws, stock};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, state::EQUATION_PROGRESS_DIRTY_SLOT,
};
use starclock_activity::{ActivityExpression, ActivityOperation, ActivityValue};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;

#[test]
fn shop_room_revalidates_pending_offers_and_dirty_progress_before_regenerating_admission() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for dirty in [false, true] {
        let fixture = compile(
            &source,
            DivergentUniverseRunFamily::Ordinary,
            stock(&source),
            Setup {
                budget: 250,
                ..Setup::default()
            },
        );
        let mut activity = fixture.start();
        let room = fixture.bound();
        if dirty {
            mutate(
                &mut activity,
                vec![ActivityOperation::SetSlot {
                    slot: EQUATION_PROGRESS_DIRTY_SLOT,
                    value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
                }],
            );
        } else {
            let blessings = source.factory().blessing_runtime().unwrap();
            let group = blessings
                .groups()
                .iter()
                .find(|group| !group.candidates().is_empty())
                .unwrap();
            let hash = activity.state_hash();
            blessings
                .begin_group_offer(&mut activity, hash, group.id())
                .unwrap();
        }
        // A trusted external mutation does not rewrite the old offered-ID snapshot.
        // The purchase plan must still reject stale eligibility before any draw.
        let before = activity.canonical_state_bytes();
        let draws = reward_draws(&activity);
        for id in [1, 2] {
            assert!(choose(&room, &mut activity, id).is_err());
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(reward_draws(&activity), draws);
        }
        choose(&room, &mut activity, 3).unwrap();
        // Existing Blessing offers are physical-node scoped and reset on the
        // self-menu transition. Dirty progress is Activity-scoped and persists.
        assert_eq!(
            options(&activity),
            if dirty {
                vec![LEAVE_SHOP]
            } else {
                vec![1, 2, LEAVE_SHOP]
            }
        );
        assert_eq!(reward_draws(&activity), draws);
        choose(&room, &mut activity, LEAVE_SHOP).unwrap();
    }
}
