//! Exact Gamble unit outcomes and fail-closed unresolved offer groups.

use starclock_activity::{ActivityStateHash, ActivityTransactionEvent, GraphActivity};
use starclock_data::divergent_universe_service_catalog::{
    DivergentUniverseGambleGroupId, DivergentUniverseGambleUnitId,
};

use super::{
    DivergentUniverseCurrencyCommand, DivergentUniverseCurrencyCommandError,
    DivergentUniverseCurrencyGainRule, DivergentUniverseCurrencyKind,
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseGambleAccuracy {
    ExactReleasedCoinOutcome,
    VersionedProjectPolicyFailClosedUnavailableGroupMembership,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseGambleGroupRuntime {
    id: DivergentUniverseGambleGroupId,
    group_type: Box<str>,
    group_level: Box<str>,
    fallback: Box<str>,
}

impl DivergentUniverseGambleGroupRuntime {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseGambleGroupId {
        &self.id
    }

    #[must_use]
    pub fn group_type(&self) -> &str {
        &self.group_type
    }

    #[must_use]
    pub fn group_level(&self) -> &str {
        &self.group_level
    }

    #[must_use]
    pub fn fallback(&self) -> &str {
        &self.fallback
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DivergentUniverseGambleOutcomeRuntime {
    GainRunCurrency {
        currency: DivergentUniverseCurrencyKind,
        amount: u64,
    },
    UnresolvedBlessingSourceGroup {
        category: Box<str>,
        source_group_id: Box<str>,
    },
    UnresolvedCurioSourceGroup {
        category: Box<str>,
        source_group_id: Box<str>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseGambleUnitRuntime {
    id: DivergentUniverseGambleUnitId,
    unit_type: Box<str>,
    outcome: DivergentUniverseGambleOutcomeRuntime,
}

impl DivergentUniverseGambleUnitRuntime {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseGambleUnitId {
        &self.id
    }

    #[must_use]
    pub fn unit_type(&self) -> &str {
        &self.unit_type
    }

    #[must_use]
    pub const fn outcome(&self) -> &DivergentUniverseGambleOutcomeRuntime {
        &self.outcome
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseGambleRuntime {
    groups: Box<[DivergentUniverseGambleGroupRuntime]>,
    units: Box<[DivergentUniverseGambleUnitRuntime]>,
}

impl DivergentUniverseRuntimeFactory {
    pub fn gamble_runtime(
        &self,
    ) -> Result<DivergentUniverseGambleRuntime, DivergentUniverseGambleRuntimeError> {
        compile(self.bundle.service_catalog())
    }
}

impl DivergentUniverseGambleRuntime {
    #[must_use]
    pub const fn accuracy(&self) -> [DivergentUniverseGambleAccuracy; 2] {
        [
            DivergentUniverseGambleAccuracy::ExactReleasedCoinOutcome,
            DivergentUniverseGambleAccuracy::VersionedProjectPolicyFailClosedUnavailableGroupMembership,
        ]
    }

    #[must_use]
    pub fn groups(&self) -> &[DivergentUniverseGambleGroupRuntime] {
        &self.groups
    }

    #[must_use]
    pub fn units(&self) -> &[DivergentUniverseGambleUnitRuntime] {
        &self.units
    }

    pub fn reject_unresolved_group_offer(
        &self,
        activity: &GraphActivity,
        expected_state_hash: ActivityStateHash,
        group: &DivergentUniverseGambleGroupId,
    ) -> Result<(), DivergentUniverseGambleRuntimeError> {
        if activity.state_hash() != expected_state_hash {
            return Err(DivergentUniverseGambleRuntimeError::StaleStateHash);
        }
        if self
            .groups
            .binary_search_by(|candidate| candidate.id.cmp(group))
            .is_err()
        {
            return Err(DivergentUniverseGambleRuntimeError::UnknownGroup);
        }
        Err(DivergentUniverseGambleRuntimeError::NoLegalCandidate)
    }

    pub fn execute_accepted_unit(
        &self,
        flow: &DivergentUniverseFlowInstance,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        unit: &DivergentUniverseGambleUnitId,
    ) -> Result<Box<[ActivityTransactionEvent]>, DivergentUniverseGambleRuntimeError> {
        let definition = self
            .units
            .binary_search_by(|candidate| candidate.id.cmp(unit))
            .ok()
            .and_then(|index| self.units.get(index))
            .ok_or(DivergentUniverseGambleRuntimeError::UnknownUnit)?;
        match definition.outcome {
            DivergentUniverseGambleOutcomeRuntime::GainRunCurrency { currency, amount } => flow
                .apply_currency_command(
                    activity,
                    expected_state_hash,
                    DivergentUniverseCurrencyCommand::Credit {
                        currency,
                        rule: DivergentUniverseCurrencyGainRule::GambleCoinUnit,
                        amount,
                    },
                )
                .map_err(DivergentUniverseGambleRuntimeError::Currency),
            DivergentUniverseGambleOutcomeRuntime::UnresolvedBlessingSourceGroup { .. }
            | DivergentUniverseGambleOutcomeRuntime::UnresolvedCurioSourceGroup { .. } => {
                if activity.state_hash() != expected_state_hash {
                    Err(DivergentUniverseGambleRuntimeError::StaleStateHash)
                } else {
                    Err(DivergentUniverseGambleRuntimeError::UnresolvedOutcome)
                }
            }
        }
    }
}

fn compile(
    catalog: &starclock_data::divergent_universe_service_catalog::DivergentUniverseServiceCatalog,
) -> Result<DivergentUniverseGambleRuntime, DivergentUniverseGambleRuntimeError> {
    let groups = catalog
        .gamble_groups()
        .iter()
        .map(|group| {
            if !matches!(group.group_type.as_ref(), "FortuneWheel" | "SlotMachine")
                || !group.units.is_empty()
                || !group.weights.is_empty()
                || group.draw_count.as_ref() != "Unspecified"
                || group.offer_policy.as_ref() != "UnavailableInReleasedGroupRow"
                || group.fallback.as_ref() != "RejectWithoutMutation"
                || group.runtime_lowered
            {
                return Err(DivergentUniverseGambleRuntimeError::InvalidCatalog);
            }
            Ok(DivergentUniverseGambleGroupRuntime {
                id: group.id.clone(),
                group_type: group.group_type.clone(),
                group_level: group.group_level.clone(),
                fallback: group.fallback.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let units = catalog
        .gamble_units()
        .iter()
        .map(|unit| {
            let outcome = match unit.outcome.operation.as_ref() {
                "GainRunCurrency" => {
                    if unit.unit_type.as_ref() != "Coin"
                        || unit.outcome.currency_id.as_ref().is_none_or(|id| {
                            id.as_str() != "divergent-universe.currency.cosmic-fragment"
                        })
                        || unit.outcome.category.is_some()
                        || unit.outcome.source_group_id.is_some()
                        || unit.outcome.resolution.is_some()
                    {
                        return Err(DivergentUniverseGambleRuntimeError::InvalidCatalog);
                    }
                    DivergentUniverseGambleOutcomeRuntime::GainRunCurrency {
                        currency: DivergentUniverseCurrencyKind::CosmicFragment,
                        amount: parse_amount(
                            unit.outcome
                                .amount
                                .as_deref()
                                .ok_or(DivergentUniverseGambleRuntimeError::InvalidCatalog)?,
                        )?,
                    }
                }
                "SelectBlessingSourceGroup" => unresolved_outcome(unit, true)?,
                "SelectCurioSourceGroup" => unresolved_outcome(unit, false)?,
                _ => return Err(DivergentUniverseGambleRuntimeError::InvalidCatalog),
            };
            if unit.runtime_lowered {
                return Err(DivergentUniverseGambleRuntimeError::InvalidCatalog);
            }
            Ok(DivergentUniverseGambleUnitRuntime {
                id: unit.id.clone(),
                unit_type: unit.unit_type.clone(),
                outcome,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if groups.len() != 126
        || units.len() != 89
        || units
            .iter()
            .filter(|unit| {
                matches!(
                    unit.outcome,
                    DivergentUniverseGambleOutcomeRuntime::GainRunCurrency { .. }
                )
            })
            .count()
            != 2
    {
        return Err(DivergentUniverseGambleRuntimeError::InvalidCatalog);
    }
    Ok(DivergentUniverseGambleRuntime {
        groups: groups.into_boxed_slice(),
        units: units.into_boxed_slice(),
    })
}

fn unresolved_outcome(
    unit: &starclock_data::divergent_universe_service_catalog::DivergentUniverseGambleUnitDefinition,
    blessing: bool,
) -> Result<DivergentUniverseGambleOutcomeRuntime, DivergentUniverseGambleRuntimeError> {
    let category = unit
        .outcome
        .category
        .clone()
        .ok_or(DivergentUniverseGambleRuntimeError::InvalidCatalog)?;
    let source_group_id = unit
        .outcome
        .source_group_id
        .clone()
        .ok_or(DivergentUniverseGambleRuntimeError::InvalidCatalog)?;
    if category.is_empty()
        || source_group_id.is_empty()
        || unit.outcome.currency_id.is_some()
        || unit.outcome.amount.is_some()
        || unit.outcome.resolution.as_deref()
            != Some(if blessing {
                "DeferredToP2B1"
            } else {
                "DeferredToP2B2"
            })
    {
        return Err(DivergentUniverseGambleRuntimeError::InvalidCatalog);
    }
    Ok(if blessing {
        DivergentUniverseGambleOutcomeRuntime::UnresolvedBlessingSourceGroup {
            category,
            source_group_id,
        }
    } else {
        DivergentUniverseGambleOutcomeRuntime::UnresolvedCurioSourceGroup {
            category,
            source_group_id,
        }
    })
}

fn parse_amount(value: &str) -> Result<u64, DivergentUniverseGambleRuntimeError> {
    value
        .parse::<u64>()
        .ok()
        .filter(|amount| *amount > 0 && amount.to_string() == value)
        .ok_or(DivergentUniverseGambleRuntimeError::InvalidCatalog)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DivergentUniverseGambleRuntimeError {
    InvalidCatalog,
    UnknownGroup,
    UnknownUnit,
    NoLegalCandidate,
    UnresolvedOutcome,
    StaleStateHash,
    Currency(DivergentUniverseCurrencyCommandError),
}

impl core::fmt::Display for DivergentUniverseGambleRuntimeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe Gamble runtime error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseGambleRuntimeError {}
