//! Closed lowering for run currencies and simulation-visible common constants.

#[path = "fragment_gains.rs"]
pub(super) mod fragment_gains;

use fragment_gains::FragmentGainRuntime;

use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityStateHash, ActivityTransactionEvent, ActivityValue, GraphActivity,
    GraphActivityCommandError,
};
use starclock_data::{
    divergent_universe_curio_catalog::DivergentUniverseCurioCatalog,
    divergent_universe_decisions::DecisionCatalog,
    divergent_universe_progression_catalog::{
        DivergentUniverseConstantId, DivergentUniverseConstantValue,
        DivergentUniverseProgressionCatalog,
    },
    divergent_universe_service_catalog::{
        DivergentUniverseCurrencyDefinition, DivergentUniverseCurrencyId,
        DivergentUniverseServiceCatalog,
    },
};

use super::entry_flow::DivergentUniverseFlowInstance;
use super::state::CURRENCIES_SLOT;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseCurrencyKind {
    CosmicFragment,
    WorkbenchHeat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseCurrencyScope {
    Run,
    Workbench,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseCurrencyGainRule {
    CurseChest,
    GambleCoinUnit,
    Unspecified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseCurrencySpendRule {
    CurseChest,
    OtherServicesDeferredToP2B4,
    EnhanceBlessing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseCurrencyResetRule {
    RunEnd,
    EachWorkbench,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseCurrencyCommand {
    Credit {
        currency: DivergentUniverseCurrencyKind,
        rule: DivergentUniverseCurrencyGainRule,
        amount: u64,
    },
    Spend {
        currency: DivergentUniverseCurrencyKind,
        rule: DivergentUniverseCurrencySpendRule,
        amount: u64,
    },
    ResetWorkbenchHeat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurrencyRuntime {
    id: DivergentUniverseCurrencyId,
    kind: DivergentUniverseCurrencyKind,
    key: u64,
    scope: DivergentUniverseCurrencyScope,
    gain_rules: Box<[DivergentUniverseCurrencyGainRule]>,
    spend_rules: Box<[DivergentUniverseCurrencySpendRule]>,
    reset_rule: DivergentUniverseCurrencyResetRule,
    fragment_gains: Option<FragmentGainRuntime>,
}

impl DivergentUniverseCurrencyRuntime {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseCurrencyId {
        &self.id
    }

    #[must_use]
    pub const fn kind(&self) -> DivergentUniverseCurrencyKind {
        self.kind
    }

    #[must_use]
    pub const fn key(&self) -> u64 {
        self.key
    }

    #[must_use]
    pub const fn scope(&self) -> DivergentUniverseCurrencyScope {
        self.scope
    }

    #[must_use]
    pub fn gain_rules(&self) -> &[DivergentUniverseCurrencyGainRule] {
        &self.gain_rules
    }

    #[must_use]
    pub fn spend_rules(&self) -> &[DivergentUniverseCurrencySpendRule] {
        &self.spend_rules
    }

    #[must_use]
    pub const fn reset_rule(&self) -> DivergentUniverseCurrencyResetRule {
        self.reset_rule
    }

    /// Generates one atomic credit including reviewed gain modifiers. The
    /// caller must commit the complete ordered program, never a prefix.
    pub fn credit_operations(
        &self,
        amount: u64,
    ) -> Result<Vec<ActivityOperation>, DivergentUniverseEconomyError> {
        let amount = checked_amount(amount)?;
        Ok(self.credit_expression_operations(integer(amount)))
    }

    pub(super) fn credit_expression_operations(
        &self,
        base: ActivityExpression,
    ) -> Vec<ActivityOperation> {
        if let Some(gains) = &self.fragment_gains {
            return gains.operations(self.key, base);
        }
        vec![ActivityOperation::AddCounter {
            slot: CURRENCIES_SLOT,
            key: self.key,
            delta: base,
        }]
    }

    pub fn spend_operation(
        &self,
        amount: u64,
    ) -> Result<ActivityOperation, DivergentUniverseEconomyError> {
        let amount = checked_amount(amount)?;
        Ok(ActivityOperation::AddCounter {
            slot: CURRENCIES_SLOT,
            key: self.key,
            delta: integer(-amount),
        })
    }

    /// Resets Workbench Heat at a workbench boundary. The enclosing accepted
    /// service command remains responsible for applying this typed operation.
    pub fn workbench_reset_operation(
        &self,
    ) -> Result<ActivityOperation, DivergentUniverseEconomyError> {
        if self.reset_rule != DivergentUniverseCurrencyResetRule::EachWorkbench {
            return Err(DivergentUniverseEconomyError::WrongResetBoundary);
        }
        Ok(ActivityOperation::SetCounter {
            slot: CURRENCIES_SLOT,
            key: self.key,
            value: integer(0),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DivergentUniverseRuntimeConstantValue {
    Integer(i64),
    IntegerArray(Box<[i64]>),
    OpaqueText(Box<str>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseRuntimeConstant {
    id: DivergentUniverseConstantId,
    value: DivergentUniverseRuntimeConstantValue,
    consumers: Box<[Box<str>]>,
}

impl DivergentUniverseRuntimeConstant {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseConstantId {
        &self.id
    }

    #[must_use]
    pub const fn value(&self) -> &DivergentUniverseRuntimeConstantValue {
        &self.value
    }

    #[must_use]
    pub fn consumers(&self) -> &[Box<str>] {
        &self.consumers
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEconomyProjection {
    currencies: Box<[DivergentUniverseCurrencyRuntime]>,
    constants: Box<[DivergentUniverseRuntimeConstant]>,
    excluded_constant_count: usize,
}

impl DivergentUniverseEconomyProjection {
    #[must_use]
    pub fn currencies(&self) -> &[DivergentUniverseCurrencyRuntime] {
        &self.currencies
    }

    #[must_use]
    pub fn constants(&self) -> &[DivergentUniverseRuntimeConstant] {
        &self.constants
    }

    #[must_use]
    pub const fn excluded_constant_count(&self) -> usize {
        self.excluded_constant_count
    }

    #[must_use]
    pub fn currency(
        &self,
        kind: DivergentUniverseCurrencyKind,
    ) -> &DivergentUniverseCurrencyRuntime {
        self.currencies
            .iter()
            .find(|currency| currency.kind == kind)
            .expect("closed projection contains both currency kinds")
    }
}

impl DivergentUniverseFlowInstance {
    /// Applies one validated currency command through the generic Activity
    /// transaction owner. Service batches remain responsible for deciding when
    /// an authored gain/spend rule is offered.
    pub fn apply_currency_command(
        &self,
        activity: &mut GraphActivity,
        expected_state_hash: ActivityStateHash,
        command: DivergentUniverseCurrencyCommand,
    ) -> Result<Box<[ActivityTransactionEvent]>, DivergentUniverseCurrencyCommandError> {
        if expected_state_hash != activity.state_hash() {
            return Err(DivergentUniverseCurrencyCommandError::Activity(
                GraphActivityCommandError::StaleStateHash,
            ));
        }
        if activity.definition().identity() != self.definition.identity() {
            return Err(DivergentUniverseCurrencyCommandError::DefinitionMismatch);
        }
        if activity.player_view().terminal().is_some() {
            return Err(DivergentUniverseCurrencyCommandError::ActivityCompleted);
        }
        let (program_id, operations) = match command {
            DivergentUniverseCurrencyCommand::Credit {
                currency,
                rule,
                amount,
            } => {
                let definition = self.economy().currency(currency);
                if !definition.gain_rules.contains(&rule) {
                    return Err(DivergentUniverseCurrencyCommandError::RuleMismatch);
                }
                let checked = checked_amount(amount)?;
                currency_balance(activity, definition.key)?
                    .checked_add(checked)
                    .ok_or(DivergentUniverseCurrencyCommandError::BalanceOutOfRange)?;
                (20_001, definition.credit_operations(amount)?)
            }
            DivergentUniverseCurrencyCommand::Spend {
                currency,
                rule,
                amount,
            } => {
                let definition = self.economy().currency(currency);
                if !definition.spend_rules.contains(&rule) {
                    return Err(DivergentUniverseCurrencyCommandError::RuleMismatch);
                }
                let checked = checked_amount(amount)?;
                if checked > currency_balance(activity, definition.key)? {
                    return Err(DivergentUniverseCurrencyCommandError::InsufficientBalance);
                }
                (20_002, vec![definition.spend_operation(amount)?])
            }
            DivergentUniverseCurrencyCommand::ResetWorkbenchHeat => (
                20_003,
                vec![
                    self.economy()
                        .currency(DivergentUniverseCurrencyKind::WorkbenchHeat)
                        .workbench_reset_operation()?,
                ],
            ),
        };
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(program_id).expect("non-zero currency program ID"),
            operations,
        )
        .map_err(|_| DivergentUniverseCurrencyCommandError::InvalidProgram)?;
        activity
            .apply_boundary_program(expected_state_hash, &program)
            .map_err(DivergentUniverseCurrencyCommandError::Activity)
    }
}

fn currency_balance(
    activity: &GraphActivity,
    key: u64,
) -> Result<i64, DivergentUniverseCurrencyCommandError> {
    let view = activity.player_view();
    let value = view
        .slots()
        .iter()
        .find(|slot| slot.id() == CURRENCIES_SLOT)
        .ok_or(DivergentUniverseCurrencyCommandError::InvalidCurrencyState)?
        .value();
    let ActivityValue::BoundedCounterMap(values) = value else {
        return Err(DivergentUniverseCurrencyCommandError::InvalidCurrencyState);
    };
    Ok(values
        .binary_search_by_key(&key, |entry| entry.0)
        .ok()
        .map_or(0, |index| values[index].1))
}

pub(super) fn compile(
    services: &DivergentUniverseServiceCatalog,
    progression: &DivergentUniverseProgressionCatalog,
    curios: &DivergentUniverseCurioCatalog,
    decisions: &DecisionCatalog,
) -> Result<DivergentUniverseEconomyProjection, DivergentUniverseEconomyError> {
    let gains = FragmentGainRuntime::compile(curios, decisions.curio_fragment_gains())?;
    let currencies = services
        .currencies()
        .iter()
        .map(|definition| {
            let mut currency = lower_currency(definition)?;
            if currency.kind == DivergentUniverseCurrencyKind::CosmicFragment {
                currency.fragment_gains = Some(gains.clone());
            }
            Ok(currency)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if currencies.len() != 2
        || !currencies
            .iter()
            .any(|value| value.kind == DivergentUniverseCurrencyKind::CosmicFragment)
        || !currencies
            .iter()
            .any(|value| value.kind == DivergentUniverseCurrencyKind::WorkbenchHeat)
    {
        return Err(DivergentUniverseEconomyError::CurrencyClosureMismatch);
    }

    let excluded_constant_count = progression
        .constants()
        .iter()
        .filter(|value| value.exclusion_reason.is_some())
        .count();
    let constants = progression
        .constants()
        .iter()
        .filter(|value| value.exclusion_reason.is_none())
        .map(|definition| {
            let value = match (&definition.canonical_value, definition.value_kind.as_ref()) {
                (DivergentUniverseConstantValue::Scalar(value), "Integer") => {
                    DivergentUniverseRuntimeConstantValue::Integer(parse_integer(value)?)
                }
                (DivergentUniverseConstantValue::Scalar(value), "String") => {
                    if value.is_empty() {
                        return Err(DivergentUniverseEconomyError::InvalidConstant);
                    }
                    DivergentUniverseRuntimeConstantValue::OpaqueText(value.clone())
                }
                (DivergentUniverseConstantValue::Array(values), "Array") => {
                    let values = values
                        .iter()
                        .map(|value| parse_integer(value))
                        .collect::<Result<Vec<_>, _>>()?;
                    DivergentUniverseRuntimeConstantValue::IntegerArray(values.into_boxed_slice())
                }
                _ => return Err(DivergentUniverseEconomyError::InvalidConstant),
            };
            Ok(DivergentUniverseRuntimeConstant {
                id: definition.id.clone(),
                value,
                consumers: definition.consumers.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if constants.len() != 18 || excluded_constant_count != 16 {
        return Err(DivergentUniverseEconomyError::ConstantClosureMismatch);
    }
    Ok(DivergentUniverseEconomyProjection {
        currencies: currencies.into_boxed_slice(),
        constants: constants.into_boxed_slice(),
        excluded_constant_count,
    })
}

fn lower_currency(
    definition: &DivergentUniverseCurrencyDefinition,
) -> Result<DivergentUniverseCurrencyRuntime, DivergentUniverseEconomyError> {
    let (kind, scope, reset_rule) = match definition.id.as_str() {
        "divergent-universe.currency.cosmic-fragment" => (
            DivergentUniverseCurrencyKind::CosmicFragment,
            DivergentUniverseCurrencyScope::Run,
            DivergentUniverseCurrencyResetRule::RunEnd,
        ),
        "divergent-universe.currency.workbench-heat" => (
            DivergentUniverseCurrencyKind::WorkbenchHeat,
            DivergentUniverseCurrencyScope::Workbench,
            DivergentUniverseCurrencyResetRule::EachWorkbench,
        ),
        _ => return Err(DivergentUniverseEconomyError::UnknownCurrency),
    };
    if definition.scope.as_ref()
        != match scope {
            DivergentUniverseCurrencyScope::Run => "Run",
            DivergentUniverseCurrencyScope::Workbench => "Workbench",
        }
        || definition.reset_rule.as_ref()
            != match reset_rule {
                DivergentUniverseCurrencyResetRule::RunEnd => "RunEnd",
                DivergentUniverseCurrencyResetRule::EachWorkbench => "ResetAtEachWorkbench",
            }
    {
        return Err(DivergentUniverseEconomyError::CurrencyBoundaryMismatch);
    }
    let gain_rules = definition
        .gain_rules
        .iter()
        .map(|rule| match rule.as_ref() {
            "CurseChest" => Ok(DivergentUniverseCurrencyGainRule::CurseChest),
            "GambleCoinUnit" => Ok(DivergentUniverseCurrencyGainRule::GambleCoinUnit),
            "Unspecified" => Ok(DivergentUniverseCurrencyGainRule::Unspecified),
            _ => Err(DivergentUniverseEconomyError::UnknownCurrencyRule),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let spend_rules = definition
        .spend_rules
        .iter()
        .map(|rule| match rule.as_ref() {
            "CurseChest" => Ok(DivergentUniverseCurrencySpendRule::CurseChest),
            "OtherServicesDeferredToP2B4" => {
                Ok(DivergentUniverseCurrencySpendRule::OtherServicesDeferredToP2B4)
            }
            "EnhanceBlessing" => Ok(DivergentUniverseCurrencySpendRule::EnhanceBlessing),
            _ => Err(DivergentUniverseEconomyError::UnknownCurrencyRule),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DivergentUniverseCurrencyRuntime {
        id: definition.id.clone(),
        kind,
        key: stable_text(definition.id.as_str()),
        scope,
        gain_rules: gain_rules.into_boxed_slice(),
        spend_rules: spend_rules.into_boxed_slice(),
        reset_rule,
        fragment_gains: None,
    })
}

fn parse_integer(value: &str) -> Result<i64, DivergentUniverseEconomyError> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(DivergentUniverseEconomyError::InvalidConstant);
    }
    value
        .parse()
        .map_err(|_| DivergentUniverseEconomyError::InvalidConstant)
}

fn checked_amount(amount: u64) -> Result<i64, DivergentUniverseEconomyError> {
    if amount == 0 {
        return Err(DivergentUniverseEconomyError::ZeroAmount);
    }
    i64::try_from(amount).map_err(|_| DivergentUniverseEconomyError::AmountOutOfRange)
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn stable_text(value: &str) -> u64 {
    use crate::digest::CanonicalDigestBuilder;

    let mut hash = CanonicalDigestBuilder::new();
    hash.update(b"starclock.divergent-universe.currency-key.v1");
    hash.update(
        u64::try_from(value.len())
            .expect("text length fits u64")
            .to_le_bytes(),
    );
    hash.update(value.as_bytes());
    let bytes = hash.finalize();
    let mut raw = [0_u8; 8];
    raw.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(raw).max(1)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseEconomyError {
    UnknownCurrency,
    CurrencyClosureMismatch,
    CurrencyBoundaryMismatch,
    UnknownCurrencyRule,
    ConstantClosureMismatch,
    InvalidConstant,
    ZeroAmount,
    AmountOutOfRange,
    WrongResetBoundary,
    InvalidFragmentGain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseCurrencyCommandError {
    DefinitionMismatch,
    ActivityCompleted,
    RuleMismatch,
    InvalidProgram,
    InvalidCurrencyState,
    InsufficientBalance,
    BalanceOutOfRange,
    Economy(DivergentUniverseEconomyError),
    Activity(GraphActivityCommandError),
}

impl From<DivergentUniverseEconomyError> for DivergentUniverseCurrencyCommandError {
    fn from(value: DivergentUniverseEconomyError) -> Self {
        Self::Economy(value)
    }
}

impl std::fmt::Display for DivergentUniverseCurrencyCommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid Divergent Universe currency command: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseCurrencyCommandError {}

impl std::fmt::Display for DivergentUniverseEconomyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid Divergent Universe economy runtime: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseEconomyError {}
