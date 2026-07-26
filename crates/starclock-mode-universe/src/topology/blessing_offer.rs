//! Curio-aware postcombat Blessing offer policy.

use super::*;
use crate::id::CurioId;

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
const FORTUNE_GLUE_CURIO: u32 = 7;
const BEACON_COLORING_PASTE_CURIO: u32 = 69;

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_blessing_offer_policy(
    catalog: &UniverseCatalog,
    node: NodeId,
    source: u64,
    weights: Vec<(ActivityOptionId, u64)>,
    reroll_slot: ActivitySlotId,
    blessing_offer_marker_slot: ActivitySlotId,
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
    let three_star_options = eligible
        .iter()
        .filter(|blessing| blessing.rarity() == 3)
        .map(|blessing| blessing_option(source, blessing.blessing()))
        .collect::<Vec<_>>();
    let fortune_glue = CurioId::new(FORTUNE_GLUE_CURIO).expect("Fortune Glue Curio ID is non-zero");
    offer = offer
        .with_conditional_candidate_filter(
            crate::curio_activity::active_condition(fortune_glue, curio_bindings),
            three_star_options,
        )
        .ok_or(UniverseTopologyCompileError::InvalidProgram)?;
    offer = offer
        .with_selection_prefix(vec![ActivityOperation::Conditional {
            condition: crate::curio_activity::active_condition(fortune_glue, curio_bindings),
            if_true: crate::curio_activity::destroy_and_count_operations(
                fortune_glue,
                curio_bindings,
            )
            .into_boxed_slice(),
            if_false: Box::new([]),
        }])
        .ok_or(UniverseTopologyCompileError::InvalidProgram)?;
    let beacon = CurioId::new(BEACON_COLORING_PASTE_CURIO)
        .expect("Beacon Coloring Paste Curio ID is non-zero");
    offer = offer
        .with_selected_option_marker(
            crate::curio_activity::active_condition(beacon, curio_bindings),
            blessing_offer_marker_slot,
            ActivityRngLabel::Reward,
            BLESSING_ENHANCEMENT_DRAW_PURPOSE,
            1,
        )
        .ok_or(UniverseTopologyCompileError::InvalidProgram)?;
    Ok(offer)
}
