//! Canonical identity for Swarm shared-content links and mode Curio lifecycle rows.

use crate::{
    blessing_runtime::BlessingRuntimeCatalog, digest::Encoder,
    swarm_disaster_content::inventory_access::InventoryRuntimeInput,
};

use super::content_runtime::{
    ReachableBlessing, RuntimeCurio, SWARM_DISASTER_CONTENT_RUNTIME_REVISION,
    SWARM_DISASTER_OFFER_POLICY_ACCURACY,
};

pub(super) fn catalog_digest(
    blessings: &BlessingRuntimeCatalog,
    reachable: &[ReachableBlessing],
    curios: &[RuntimeCurio],
    input: &InventoryRuntimeInput,
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.content-runtime.v1");
    encoder.text(SWARM_DISASTER_CONTENT_RUNTIME_REVISION);
    encoder.text(SWARM_DISASTER_OFFER_POLICY_ACCURACY);
    encoder.digest(blessings.digest());
    for row in reachable {
        encoder.u32(row.id.get());
        encoder.text(&row.key);
        encoder.text(&row.shared_key);
        encoder.text(&row.path_key);
        encoder.u8(row.rarity);
    }
    for row in curios {
        encoder.u32(row.id);
        encoder.u32(row.source_id);
        encoder.text(&row.key);
        encoder.u8(row.category as u8);
        encoder.u8(row.initial_state as u8);
        encoder.u8(row.terminal_state as u8);
        encoder.u8(row.maximum_charges.unwrap_or(0));
        encoder.text(&row.decrement_event);
        encoder.u8(row.repair_after_battles.unwrap_or(0));
        encoder.text(&row.effect_program);
        encoder.text(&row.repair_target);
        encoder.text(&row.trigger_phase);
        encoder.text(&row.trigger);
        encoder.text(&row.replacement_policy);
    }
    for row in &input.pool_memberships {
        encoder.u32(row.id);
        encoder.text(&row.pool_key);
        encoder.text(&row.member_kind);
        encoder.text(&row.member_key);
        encoder.text(&row.eligibility);
        encoder.text(&row.weight_policy);
    }
    encoder.finish()
}
