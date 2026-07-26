use super::*;

pub(super) fn restrict_random_offer(
    state: &mut ActivityTransactionState,
    rng: &mut ActivityRngStreams,
    policy: &ActivityRandomOffer,
) -> Result<(), GraphActivityRuntimeError> {
    let offered = state
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
