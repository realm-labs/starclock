//! Curio-aware postcombat Blessing offer policy.

use super::*;

const SEALING_WAX_POLICIES: [(&str, u64); 9] = [
    ("universe.path.propagation", 18),
    ("universe.path.erudition", 27),
    ("universe.path.preservation", 28),
    ("universe.path.elation", 29),
    ("universe.path.hunt", 30),
    ("universe.path.destruction", 31),
    ("universe.path.remembrance", 32),
    ("universe.path.nihility", 33),
    ("universe.path.abundance", 34),
];

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_blessing_offer_policy(
    catalog: &UniverseCatalog,
    node: NodeId,
    source: u64,
    weights: Vec<(ActivityOptionId, u64)>,
    reroll_slot: ActivitySlotId,
    curio_bindings: crate::curio_activity::CurioActivityBindings,
    eligible: &[&crate::blessing_runtime::BlessingRuntimeDefinition],
) -> Result<ActivityRandomOffer, UniverseTopologyCompileError> {
    let mut offer = ActivityRandomOffer::new(
        node,
        ActivityRngLabel::Reward,
        BLESSING_DRAW_PURPOSE,
        3,
        weights,
        Some((reroll_slot, 2)),
    )
    .map_err(UniverseTopologyCompileError::RuntimeDefinition)?
    .with_maximum_options_reduction(
        crate::curio_activity::dimension_reward_condition(curio_bindings),
        1,
    )
    .ok_or(UniverseTopologyCompileError::InvalidProgram)?
    .with_inactive_condition(crate::curio_activity::domain::gossip_condition(
        curio_bindings,
    ));

    // Public Curio text says "greatly increased" but publishes no scalar.
    // Standard Universe v1 freezes x2 as one replaceable project policy.
    for (path_key, curio_content) in SEALING_WAX_POLICIES {
        let path = catalog
            .paths()
            .iter()
            .find(|path| path.stable_key() == path_key)
            .map(|path| path.id())
            .ok_or(UniverseTopologyCompileError::InvalidBlessingRuntime)?;
        let options = eligible
            .iter()
            .filter(|blessing| blessing.path() == path)
            .map(|blessing| blessing_option(source, blessing.blessing()))
            .collect::<Vec<_>>();
        if options.is_empty() {
            continue;
        }
        offer = offer
            .with_conditional_weight_multiplier(
                crate::curio_activity::domain::sealing_wax_condition(curio_bindings, curio_content),
                options,
                2,
            )
            .ok_or(UniverseTopologyCompileError::InvalidProgram)?;
    }
    Ok(offer)
}
