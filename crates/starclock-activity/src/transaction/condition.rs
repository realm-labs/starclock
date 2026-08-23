use super::*;

impl ActivityTransactionState {
    pub(crate) fn condition(&self, condition: &ActivityCondition) -> Result<bool, ActivityFault> {
        match condition {
            ActivityCondition::Boolean(value) => match self.evaluate(value)? {
                ActivityValue::Boolean(value) => Ok(value),
                _ => Err(ActivityFault::TypeMismatch),
            },
            ActivityCondition::Equal(a, b) => Ok(self.evaluate(a)? == self.evaluate(b)?),
            ActivityCondition::LessThan(a, b) => match (self.evaluate(a)?, self.evaluate(b)?) {
                (ActivityValue::BoundedInteger(a), ActivityValue::BoundedInteger(b))
                | (ActivityValue::FixedScalar(a), ActivityValue::FixedScalar(b)) => Ok(a < b),
                _ => Err(ActivityFault::TypeMismatch),
            },
            ActivityCondition::Compare {
                left,
                operator,
                right,
            } => compare(self.evaluate(left)?, *operator, self.evaluate(right)?),
            ActivityCondition::OrderedIdSetContains { slot, id } => {
                if *id == 0 {
                    return Err(ActivityFault::TypeMismatch);
                }
                match self
                    .slots
                    .get(slot)
                    .ok_or(ActivityFault::MissingSlot(*slot))?
                {
                    ActivityValue::OrderedIdSet(values) => Ok(values.binary_search(id).is_ok()),
                    _ => Err(ActivityFault::TypeMismatch),
                }
            }
            ActivityCondition::ParticipantDefeated(participant) => {
                Ok(self.carry.participant_defeated(*participant))
            }
            ActivityCondition::Not(value) => Ok(!self.condition(value)?),
            ActivityCondition::All(values) => {
                for value in values.iter() {
                    if !self.condition(value)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            ActivityCondition::Any(values) => {
                for value in values.iter() {
                    if self.condition(value)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }
}

fn compare(
    left: ActivityValue,
    operator: ActivityComparison,
    right: ActivityValue,
) -> Result<bool, ActivityFault> {
    match operator {
        ActivityComparison::Equal => Ok(left == right),
        ActivityComparison::NotEqual => Ok(left != right),
        ActivityComparison::Less
        | ActivityComparison::LessOrEqual
        | ActivityComparison::Greater
        | ActivityComparison::GreaterOrEqual => {
            let (left, right) = match (left, right) {
                (ActivityValue::BoundedInteger(left), ActivityValue::BoundedInteger(right))
                | (ActivityValue::FixedScalar(left), ActivityValue::FixedScalar(right)) => {
                    (left, right)
                }
                _ => return Err(ActivityFault::TypeMismatch),
            };
            Ok(match operator {
                ActivityComparison::Less => left < right,
                ActivityComparison::LessOrEqual => left <= right,
                ActivityComparison::Greater => left > right,
                ActivityComparison::GreaterOrEqual => left >= right,
                ActivityComparison::Equal | ActivityComparison::NotEqual => {
                    unreachable!("equality comparisons returned before numeric ordering")
                }
            })
        }
    }
}
