use super::*;

pub(super) fn valid_selection_prefix(
    policy: &ActivityRandomOffer,
    state: &ActivityStateDefinition,
    graph: &ActivityGraphDefinition,
) -> bool {
    if policy.selection_prefix.is_empty() {
        return true;
    }
    ActivityProgramDefinition::new(
        ActivityProgramId::new(u32::MAX).expect("maximum program ID remains non-zero"),
        policy.selection_prefix.to_vec(),
    )
    .is_ok_and(|prefix| {
        !contains_boundary_operation(prefix.operations())
            && prefix.validate_against(state, graph).is_ok()
    })
}

pub(super) fn restrict_random_offer(
    state: &mut ActivityTransactionState,
    rng: &mut ActivityRngStreams,
    policy: &ActivityRandomOffer,
) -> Result<(), GraphActivityRuntimeError> {
    let mut offered = state
        .pending_option_ids()
        .or_else(|| {
            policy
                .inactive_condition
                .as_ref()
                .and_then(|condition| state.condition(condition).ok())
                .filter(|inactive| *inactive)
                .map(|_| Vec::new().into_boxed_slice())
        })
        .ok_or(GraphActivityRuntimeError::InvalidRandomOffer)?;
    if offered.is_empty() {
        return Ok(());
    }
    for (condition, options) in &policy.conditional_candidate_filters {
        if state
            .condition(condition)
            .map_err(|_| GraphActivityRuntimeError::InvalidRandomOffer)?
        {
            offered = offered
                .iter()
                .copied()
                .filter(|option| options.binary_search(option).is_ok())
                .collect::<Vec<_>>()
                .into_boxed_slice();
        }
    }
    if offered.is_empty() {
        return Err(GraphActivityRuntimeError::InvalidRandomOffer);
    }
    let mut weights = Vec::with_capacity(offered.len());
    for option in &offered {
        weights.push(random_offer_weight(state, policy, *option)?);
    }
    let reduce_options = policy
        .maximum_options_reduction
        .as_ref()
        .map(|(condition, _)| {
            state
                .condition(condition)
                .map_err(|_| GraphActivityRuntimeError::InvalidRandomOffer)
        })
        .transpose()?
        .unwrap_or(false);
    let maximum_options = if reduce_options {
        policy
            .maximum_options
            .checked_sub(
                policy
                    .maximum_options_reduction
                    .as_ref()
                    .expect("reduction condition was evaluated")
                    .1,
            )
            .ok_or(GraphActivityRuntimeError::InvalidRandomOffer)?
    } else {
        policy.maximum_options
    };
    let maximum_options = maximum_options.min(
        u16::try_from(offered.len()).map_err(|_| GraphActivityRuntimeError::InvalidRandomOffer)?,
    );
    let selected = rng
        .choose_weighted_without_replacement(
            policy.label,
            policy.purpose,
            &weights,
            maximum_options,
        )
        .map_err(GraphActivityRuntimeError::Rng)?;
    let ids = selected
        .iter()
        .map(|index| offered[*index as usize])
        .collect::<Vec<_>>();
    if let Some(marker) = &policy.selected_option_marker {
        let active = state
            .condition(&marker.condition)
            .map_err(|_| GraphActivityRuntimeError::InvalidRandomOffer)?;
        let marked = if active {
            let weights = vec![1_u64; ids.len()];
            rng.choose_weighted_without_replacement(
                marker.label,
                marker.purpose,
                &weights,
                marker.count.min(maximum_options),
            )
            .map_err(GraphActivityRuntimeError::Rng)?
            .iter()
            .map(|index| ids[*index as usize])
            .collect()
        } else {
            Vec::new()
        };
        state
            .replace_counter_keys(marker.slot, marked)
            .map_err(|_| GraphActivityRuntimeError::InvalidRandomOffer)?;
    }
    state
        .restrict_pending_options(ids)
        .map_err(|_| GraphActivityRuntimeError::InvalidRandomOffer)
}

fn random_offer_weight(
    state: &ActivityTransactionState,
    policy: &ActivityRandomOffer,
    option: ActivityOptionId,
) -> Result<u64, GraphActivityRuntimeError> {
    let mut weight = policy
        .weights
        .binary_search_by_key(&option, |item| item.0)
        .ok()
        .map(|index| policy.weights[index].1)
        .ok_or(GraphActivityRuntimeError::InvalidRandomOffer)?;
    for (condition, options, multiplier) in &policy.conditional_weight_multipliers {
        if options.binary_search(&option).is_ok()
            && state
                .condition(condition)
                .map_err(|_| GraphActivityRuntimeError::InvalidRandomOffer)?
        {
            weight = weight
                .checked_mul(*multiplier)
                .ok_or(GraphActivityRuntimeError::InvalidRandomOffer)?;
        }
    }
    Ok(weight)
}
