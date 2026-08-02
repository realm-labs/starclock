use super::{OccurrenceExternalResult, OccurrenceInteractionError, PAYLOAD_TAG, PayloadOperation};

pub(super) struct Choice {
    pub(super) content: u64,
    pub(super) operations: Vec<PayloadOperation>,
    pub(super) random_candidate_count: Option<u32>,
}

pub(super) struct Lowering {
    pub(super) choices: Vec<Choice>,
    pub(super) repeat_key: Option<u64>,
}

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

pub(super) fn encode_operations(
    operations: Vec<PayloadOperation>,
) -> Result<(Vec<u8>, u16, u16), OccurrenceInteractionError> {
    let deferred_operations = u16::try_from(
        operations
            .iter()
            .filter(|operation| operation.is_deferred())
            .count(),
    )
    .map_err(|_| OccurrenceInteractionError::TooManyOperations)?;
    let immediate_operations = u16::try_from(operations.len())
        .map_err(|_| OccurrenceInteractionError::TooManyOperations)?
        .saturating_sub(deferred_operations);
    let mut payload = Vec::new();
    payload.push(PAYLOAD_TAG);
    payload.extend_from_slice(
        &u16::try_from(operations.len())
            .map_err(|_| OccurrenceInteractionError::TooManyOperations)?
            .to_le_bytes(),
    );
    for operation in operations {
        operation.encode(&mut payload)?;
    }
    Ok((payload, immediate_operations, deferred_operations))
}
