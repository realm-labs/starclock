//! Shared primitives introduced by Goal 07 Occurrence partition S02.

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityHandlerFault, ActivityHandlerInput,
    ActivityOperation, ActivityValue,
};
use starclock_combat::{Hp, LifeState, Ratio};

use crate::{
    catalog::UniverseCatalog,
    occurrence::{AuthoredScalar, AuthoredScalarUnit, OccurrenceOutcome},
};

use super::{
    OccurrenceInteractionError,
    support::{Decoder, invalid_payload, invalid_state, inventory},
};

pub(super) fn decode_participant_hp_loss(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let scaled = decoder.i64()?;
    if !(1..=1_000_000).contains(&scaled) {
        return Err(invalid_payload());
    }
    let ratio = Ratio::from_scaled(scaled);
    operations.extend(
        input
            .view()
            .participant_carry()
            .iter()
            .filter(|state| state.life() == LifeState::Alive)
            .map(|state| ActivityOperation::LoseParticipantCurrentHpRatio {
                participant: state.participant(),
                hp_ratio: ratio,
                minimum_hp: Hp::new(1).expect("one HP is valid"),
            }),
    );
    Ok(())
}

pub(super) fn decode_ensure_inventory_group(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let inventory = inventory(decoder.u32()?)?;
    let group_count = usize::from(decoder.u16()?);
    if group_count == 0
        || !input
            .view()
            .inventories()
            .iter()
            .any(|value| value.id() == inventory)
    {
        return Err(invalid_state());
    }
    let selected = input
        .random_index()
        .map_or(0, |index| index as usize % group_count);
    for group_index in 0..group_count {
        let member_count = usize::from(decoder.u16()?);
        if member_count == 0 {
            return Err(invalid_payload());
        }
        for _ in 0..member_count {
            let content = decoder.u64()?;
            if group_index == selected {
                operations.push(ActivityOperation::Conditional {
                    condition: ActivityCondition::Equal(
                        ActivityExpression::InventoryCount { inventory, content },
                        ActivityExpression::Literal(ActivityValue::BoundedInteger(0)),
                    ),
                    if_true: vec![ActivityOperation::AddInventory {
                        inventory,
                        content,
                        count: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
                    }]
                    .into_boxed_slice(),
                    if_false: Vec::new().into_boxed_slice(),
                });
            }
        }
    }
    Ok(())
}

pub(super) fn referenced_blessing_groups(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
) -> Result<Vec<Vec<u64>>, OccurrenceInteractionError> {
    let references = outcome
        .parameter_refs()
        .iter()
        .filter(|value| value.starts_with("universe.path."))
        .map(AsRef::as_ref)
        .collect::<Vec<_>>();
    let mut groups = Vec::with_capacity(references.len());
    for reference in references {
        let path = catalog
            .paths()
            .iter()
            .find(|value| value.stable_key() == reference)
            .ok_or(OccurrenceInteractionError::InvalidChoice)?;
        let mut group = catalog
            .blessings()
            .iter()
            .filter(|value| value.path() == path.id())
            .map(|value| u64::from(value.id().get()))
            .collect::<Vec<_>>();
        group.sort_unstable();
        if group.is_empty() {
            return Err(OccurrenceInteractionError::InvalidChoice);
        }
        groups.push(group);
    }
    groups.sort();
    groups.dedup();
    Ok(groups)
}

pub(super) fn percent_ratio_scaled(
    value: AuthoredScalar,
) -> Result<i64, OccurrenceInteractionError> {
    if value.unit() != AuthoredScalarUnit::Percent {
        return Err(OccurrenceInteractionError::InvalidChoice);
    }
    let scale = u32::from(value.value().scale());
    let denominator = 100_i128
        .checked_mul(10_i128.pow(scale))
        .ok_or(OccurrenceInteractionError::Arithmetic)?;
    let numerator = i128::from(value.value().coefficient())
        .checked_mul(1_000_000)
        .ok_or(OccurrenceInteractionError::Arithmetic)?;
    if numerator % denominator != 0 {
        return Err(OccurrenceInteractionError::NonIntegerScalar);
    }
    let scaled = i64::try_from(numerator / denominator)
        .map_err(|_| OccurrenceInteractionError::Arithmetic)?;
    if !(1..=1_000_000).contains(&scaled) {
        return Err(OccurrenceInteractionError::InvalidChoice);
    }
    Ok(scaled)
}
