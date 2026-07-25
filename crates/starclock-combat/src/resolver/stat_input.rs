//! Shared immutable inputs for staged stat and formula queries.

use std::collections::BTreeMap;

use crate::Scalar;

use super::transaction::Transaction;

pub(super) fn shield_values(txn: &Transaction<'_>) -> BTreeMap<crate::UnitId, Scalar> {
    txn.state
        .units
        .iter_by_id()
        .map(|unit| {
            let value = txn
                .state
                .shields
                .effective_remaining(unit.id)
                .ok()
                .and_then(|value| Scalar::checked_from_integer(value.get()).ok())
                .unwrap_or(Scalar::ZERO);
            (unit.id, value)
        })
        .collect()
}
