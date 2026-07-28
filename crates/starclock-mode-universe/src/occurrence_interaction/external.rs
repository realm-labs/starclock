use super::{
    OccurrenceExternalResult, OccurrenceInteractionError, PayloadOperation, encode_operations,
};

pub(super) fn single_selection(
    operations: &[PayloadOperation],
) -> Result<Vec<OccurrenceExternalResult>, OccurrenceInteractionError> {
    let selection = operations
        .iter()
        .enumerate()
        .filter_map(|(index, operation)| match operation {
            PayloadOperation::Inventory {
                quantity: 1,
                candidates,
                ..
            } => Some((index, candidates.clone())),
            PayloadOperation::CurioInventory {
                quantity: 1,
                candidates,
                ..
            } => Some((
                index,
                candidates
                    .iter()
                    .map(|value| u64::from(value.id().get()))
                    .collect(),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    if selection.len() != 1 {
        return Ok(Vec::new());
    }
    let (selection_index, candidates) = &selection[0];
    candidates
        .iter()
        .map(|candidate| {
            let mut concrete = operations.to_vec();
            match &mut concrete[*selection_index] {
                PayloadOperation::Inventory { candidates, .. } => {
                    candidates.clear();
                    candidates.push(*candidate);
                }
                PayloadOperation::CurioInventory { candidates, .. } => {
                    candidates.retain(|value| u64::from(value.id().get()) == *candidate);
                }
                _ => return Err(OccurrenceInteractionError::InvalidChoice),
            }
            let (payload, immediate_operations, deferred_operations) = encode_operations(concrete)?;
            Ok(OccurrenceExternalResult {
                content: *candidate,
                payload: payload.into_boxed_slice(),
                random_candidate_count: None,
                immediate_operations,
                deferred_operations,
            })
        })
        .collect()
}
