//! Workbench transformations and Curse Chest operations with explicit policy boundaries.

use std::sync::Arc;

use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityStateHash, ActivityTransactionEvent, ActivityValue, GraphActivity,
    GraphActivityCommandError,
};
use starclock_data::{
    divergent_universe_blessing_catalog::DivergentUniverseBlessingId,
    divergent_universe_service_catalog::{
        DivergentUniverseCurseChestId, DivergentUniverseServiceCatalog,
        DivergentUniverseWorkbenchFunctionId, DivergentUniverseWorkbenchId,
    },
};

use super::economy::DivergentUniverseCurrencyRuntime;
use super::{
    DivergentUniverseBlessingRuntime, DivergentUniverseBlessingRuntimeError,
    DivergentUniverseCurrencyKind, DivergentUniverseEconomyError,
    DivergentUniverseEconomyProjection, DivergentUniverseRuntimeFactory,
    state::{CURRENCIES_SLOT, SERVICE_RECEIPTS_SLOT, WORKBENCH_SLOT},
};

const ENTER_WORKBENCH_PROGRAM: u32 = 22_551;
const CURSE_CHEST_PROGRAM: u32 = 22_552;
const RECEIPT_KEY_BASE: u64 = 0x2256_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseWorkbenchCurseAccuracy {
    ExactReleasedWorkbenchFunctionMembershipAndChoiceParameters,
    VersionedProjectPolicyAcceptedWorkbenchEntryAndExplicitPrice,
    VersionedProjectPolicyRejectUnpublishedTransformationCandidates,
    VersionedProjectPolicyExplicitCurseChestAmountWithinReleasedBounds,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseWorkbenchFunctionKind {
    BlessingEnhance,
    BlessingReforge,
    EquationReforge,
    CurioCompose,
    CurioReforge,
    HexEquipment,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseWorkbenchFunctionDisposition {
    ExecutableAcceptedBlessingEnhancement,
    RejectUnpublishedPriceOrCandidateProgram,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseWorkbenchRuntimeDefinition {
    id: DivergentUniverseWorkbenchId,
    state_key: u64,
    functions: Box<[DivergentUniverseWorkbenchFunctionId]>,
    currency_count: usize,
    availability: Box<str>,
}

impl DivergentUniverseWorkbenchRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseWorkbenchId {
        &self.id
    }
    #[must_use]
    pub fn functions(&self) -> &[DivergentUniverseWorkbenchFunctionId] {
        &self.functions
    }
    #[must_use]
    pub const fn currency_count(&self) -> usize {
        self.currency_count
    }
    #[must_use]
    pub fn availability(&self) -> &str {
        &self.availability
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseWorkbenchFunctionRuntime {
    id: DivergentUniverseWorkbenchFunctionId,
    kind: DivergentUniverseWorkbenchFunctionKind,
    input_policy: Box<str>,
    output_policy: Box<str>,
    price_currency: Box<str>,
    price_formula: Box<str>,
    price_reset: Box<str>,
    disposition: DivergentUniverseWorkbenchFunctionDisposition,
}

impl DivergentUniverseWorkbenchFunctionRuntime {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseWorkbenchFunctionId {
        &self.id
    }
    #[must_use]
    pub const fn kind(&self) -> DivergentUniverseWorkbenchFunctionKind {
        self.kind
    }
    #[must_use]
    pub fn input_policy(&self) -> &str {
        &self.input_policy
    }
    #[must_use]
    pub fn output_policy(&self) -> &str {
        &self.output_policy
    }
    #[must_use]
    pub fn price_currency(&self) -> &str {
        &self.price_currency
    }
    #[must_use]
    pub fn price_formula(&self) -> &str {
        &self.price_formula
    }
    #[must_use]
    pub fn price_reset(&self) -> &str {
        &self.price_reset
    }
    #[must_use]
    pub const fn disposition(&self) -> DivergentUniverseWorkbenchFunctionDisposition {
        self.disposition
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DivergentUniverseCurseChestOperation {
    GainCosmicFragments { minimum: u64, maximum: u64 },
    LoseCosmicFragments { minimum: u64, maximum: u64 },
    LeaveWithoutMutation,
    UnresolvedCandidateOperation { operation: Box<str> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCurseChestRuntimeDefinition {
    id: DivergentUniverseCurseChestId,
    receipt_key: u64,
    chest_type: Box<str>,
    operations: Box<[DivergentUniverseCurseChestOperation]>,
    fallback: Box<str>,
}

impl DivergentUniverseCurseChestRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseCurseChestId {
        &self.id
    }
    #[must_use]
    pub fn chest_type(&self) -> &str {
        &self.chest_type
    }
    #[must_use]
    pub fn operations(&self) -> &[DivergentUniverseCurseChestOperation] {
        &self.operations
    }
    #[must_use]
    pub fn fallback(&self) -> &str {
        &self.fallback
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseWorkbenchCurseResolution {
    operation: Box<str>,
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}

impl DivergentUniverseWorkbenchCurseResolution {
    #[must_use]
    pub fn operation(&self) -> &str {
        &self.operation
    }
    #[must_use]
    pub fn events(&self) -> &[ActivityTransactionEvent] {
        &self.events
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseWorkbenchCurseRuntime {
    workbenches: Arc<[DivergentUniverseWorkbenchRuntimeDefinition]>,
    functions: Arc<[DivergentUniverseWorkbenchFunctionRuntime]>,
    curse_chests: Arc<[DivergentUniverseCurseChestRuntimeDefinition]>,
    blessing: Arc<DivergentUniverseBlessingRuntime>,
    heat_key: u64,
    fragment_key: u64,
    fragments: DivergentUniverseCurrencyRuntime,
}

impl DivergentUniverseRuntimeFactory {
    pub fn workbench_curse_runtime(
        &self,
    ) -> Result<DivergentUniverseWorkbenchCurseRuntime, DivergentUniverseWorkbenchCurseError> {
        let economy = super::economy::compile(
            self.bundle.service_catalog(),
            self.bundle.progression_catalog(),
            self.bundle.curio_catalog(),
            self.decision_catalog(),
        )?;
        let blessing = self.blessing_runtime()?;
        DivergentUniverseWorkbenchCurseRuntime::compile(
            self.bundle.service_catalog(),
            &economy,
            Arc::new(blessing),
        )
    }
}

impl DivergentUniverseWorkbenchCurseRuntime {
    fn compile(
        catalog: &DivergentUniverseServiceCatalog,
        economy: &DivergentUniverseEconomyProjection,
        blessing: Arc<DivergentUniverseBlessingRuntime>,
    ) -> Result<Self, DivergentUniverseWorkbenchCurseError> {
        let workbenches = catalog
            .workbenches()
            .iter()
            .enumerate()
            .map(|(index, value)| {
                if value.availability.as_ref() != "Unspecified" || value.runtime_lowered {
                    return Err(DivergentUniverseWorkbenchCurseError::InvalidCatalog);
                }
                Ok(DivergentUniverseWorkbenchRuntimeDefinition {
                    id: value.id.clone(),
                    state_key: ordinal(index)?,
                    functions: value.functions.clone(),
                    currency_count: value.currencies.len(),
                    availability: value.availability.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let functions = catalog
            .functions()
            .iter()
            .map(|value| {
                if !value.candidates.is_empty()
                    || !value.weights.is_empty()
                    || value.fallback.as_ref() != "RejectWithoutMutation"
                    || value.runtime_lowered
                {
                    return Err(DivergentUniverseWorkbenchCurseError::InvalidCatalog);
                }
                let kind = function_kind(&value.function_type)?;
                Ok(DivergentUniverseWorkbenchFunctionRuntime {
                    id: value.id.clone(),
                    kind,
                    input_policy: value.input_policy.clone(),
                    output_policy: value.output_policy.clone(),
                    price_currency: value.price_rule.currency.clone(),
                    price_formula: value.price_rule.formula.clone(),
                    price_reset: value.price_rule.reset.clone(),
                    disposition: if kind == DivergentUniverseWorkbenchFunctionKind::BlessingEnhance {
                        DivergentUniverseWorkbenchFunctionDisposition::ExecutableAcceptedBlessingEnhancement
                    } else {
                        DivergentUniverseWorkbenchFunctionDisposition::RejectUnpublishedPriceOrCandidateProgram
                    },
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let curse_chests = catalog
            .curse_chests()
            .iter()
            .enumerate()
            .map(|(index, value)| {
                if value.fallback.as_ref() != "LeaveWithoutMutation"
                    || value.runtime_lowered
                    || value.choices.len() != 3
                {
                    return Err(DivergentUniverseWorkbenchCurseError::InvalidCatalog);
                }
                let operations = value
                    .choices
                    .iter()
                    .map(lower_curse_operation)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(DivergentUniverseCurseChestRuntimeDefinition {
                    id: value.id.clone(),
                    receipt_key: RECEIPT_KEY_BASE
                        .checked_add(ordinal(index)?)
                        .ok_or(DivergentUniverseWorkbenchCurseError::InvalidCatalog)?,
                    chest_type: value.chest_type.clone(),
                    operations: operations.into_boxed_slice(),
                    fallback: value.fallback.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if workbenches.len() != 11 || functions.len() != 6 || curse_chests.len() != 29 {
            return Err(DivergentUniverseWorkbenchCurseError::InvalidCatalog);
        }
        for workbench in &workbenches {
            if workbench
                .functions
                .iter()
                .any(|id| lookup(&functions, |value| &value.id, id).is_err())
            {
                return Err(DivergentUniverseWorkbenchCurseError::InvalidCatalog);
            }
        }
        Ok(Self {
            workbenches: workbenches.into(),
            functions: functions.into(),
            curse_chests: curse_chests.into(),
            blessing,
            heat_key: economy
                .currency(DivergentUniverseCurrencyKind::WorkbenchHeat)
                .key(),
            fragments: economy
                .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                .clone(),
            fragment_key: economy
                .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                .key(),
        })
    }

    #[must_use]
    pub const fn accuracies(&self) -> [DivergentUniverseWorkbenchCurseAccuracy; 4] {
        [
            DivergentUniverseWorkbenchCurseAccuracy::ExactReleasedWorkbenchFunctionMembershipAndChoiceParameters,
            DivergentUniverseWorkbenchCurseAccuracy::VersionedProjectPolicyAcceptedWorkbenchEntryAndExplicitPrice,
            DivergentUniverseWorkbenchCurseAccuracy::VersionedProjectPolicyRejectUnpublishedTransformationCandidates,
            DivergentUniverseWorkbenchCurseAccuracy::VersionedProjectPolicyExplicitCurseChestAmountWithinReleasedBounds,
        ]
    }
    #[must_use]
    pub fn workbenches(&self) -> &[DivergentUniverseWorkbenchRuntimeDefinition] {
        &self.workbenches
    }
    #[must_use]
    pub fn functions(&self) -> &[DivergentUniverseWorkbenchFunctionRuntime] {
        &self.functions
    }
    #[must_use]
    pub fn curse_chests(&self) -> &[DivergentUniverseCurseChestRuntimeDefinition] {
        &self.curse_chests
    }

    pub fn enter_workbench_policy_accepted(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        workbench: &DivergentUniverseWorkbenchId,
    ) -> Result<DivergentUniverseWorkbenchCurseResolution, DivergentUniverseWorkbenchCurseError>
    {
        validate_hash(activity, expected)?;
        let workbench = self.workbench(workbench)?;
        self.apply(
            activity,
            expected,
            ENTER_WORKBENCH_PROGRAM,
            "EnterWorkbench",
            vec![
                ActivityOperation::SetSlot {
                    slot: WORKBENCH_SLOT,
                    value: literal(ActivityValue::OptionalId(Some(workbench.state_key))),
                },
                ActivityOperation::SetCounter {
                    slot: CURRENCIES_SLOT,
                    key: self.heat_key,
                    value: literal(ActivityValue::BoundedInteger(0)),
                },
            ],
        )
    }

    pub fn enhance_blessing_policy_accepted(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        workbench: &DivergentUniverseWorkbenchId,
        blessing: &DivergentUniverseBlessingId,
        explicit_heat_price: u64,
    ) -> Result<DivergentUniverseWorkbenchCurseResolution, DivergentUniverseWorkbenchCurseError>
    {
        validate_hash(activity, expected)?;
        let workbench = self.require_current_workbench(activity, workbench)?;
        let function = self
            .functions
            .iter()
            .find(|value| value.kind == DivergentUniverseWorkbenchFunctionKind::BlessingEnhance)
            .ok_or(DivergentUniverseWorkbenchCurseError::InvalidCatalog)?;
        if !workbench.functions.contains(&function.id) {
            return Err(DivergentUniverseWorkbenchCurseError::FunctionUnavailable);
        }
        let price = checked_amount(explicit_heat_price)?;
        if balance(activity, self.heat_key)? < price {
            return Err(DivergentUniverseWorkbenchCurseError::InsufficientBalance);
        }
        let receipt_key = function_receipt_key(function)?;
        let resolution = self.blessing.enhance_accepted_identity_with_operations(
            activity,
            expected,
            blessing,
            vec![
                ActivityOperation::AddCounter {
                    slot: CURRENCIES_SLOT,
                    key: self.heat_key,
                    delta: literal(ActivityValue::BoundedInteger(-price)),
                },
                ActivityOperation::AddCounter {
                    slot: SERVICE_RECEIPTS_SLOT,
                    key: receipt_key,
                    delta: literal(ActivityValue::BoundedInteger(1)),
                },
            ],
        )?;
        Ok(DivergentUniverseWorkbenchCurseResolution {
            operation: "EnhanceBlessing".into(),
            events: resolution.events().to_vec().into_boxed_slice(),
            state_hash: resolution.state_hash(),
        })
    }

    pub fn reject_unresolved_workbench_transformation(
        &self,
        activity: &GraphActivity,
        expected: ActivityStateHash,
        workbench: &DivergentUniverseWorkbenchId,
        function: &DivergentUniverseWorkbenchFunctionId,
        input_ids: &[&str],
        output_ids: &[&str],
    ) -> Result<(), DivergentUniverseWorkbenchCurseError> {
        validate_hash(activity, expected)?;
        let workbench = self.require_current_workbench(activity, workbench)?;
        let function = self.function(function)?;
        if !workbench.functions.contains(&function.id) {
            return Err(DivergentUniverseWorkbenchCurseError::FunctionUnavailable);
        }
        if input_ids.is_empty()
            || output_ids.is_empty()
            || input_ids.iter().any(|value| value.is_empty())
            || output_ids.iter().any(|value| value.is_empty())
        {
            return Err(DivergentUniverseWorkbenchCurseError::InvalidSelection);
        }
        if function.disposition
            == DivergentUniverseWorkbenchFunctionDisposition::ExecutableAcceptedBlessingEnhancement
        {
            return Err(DivergentUniverseWorkbenchCurseError::WrongFunctionBoundary);
        }
        Err(DivergentUniverseWorkbenchCurseError::UnpublishedTransformationProgram)
    }

    pub fn execute_curse_chest_choice_policy_accepted(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        chest: &DivergentUniverseCurseChestId,
        choice_index: usize,
        explicit_amount: Option<u64>,
    ) -> Result<DivergentUniverseWorkbenchCurseResolution, DivergentUniverseWorkbenchCurseError>
    {
        validate_hash(activity, expected)?;
        let chest = self.curse_chest(chest)?;
        if receipt_count(activity, chest.receipt_key)? != 0 {
            return Err(DivergentUniverseWorkbenchCurseError::ChestAlreadyResolved);
        }
        let operation = chest
            .operations
            .get(choice_index)
            .ok_or(DivergentUniverseWorkbenchCurseError::UnknownChoice)?;
        let mut operations = Vec::new();
        let operation_name =
            match operation {
                DivergentUniverseCurseChestOperation::GainCosmicFragments { minimum, maximum } => {
                    let amount = bounded_amount(explicit_amount, *minimum, *maximum)?;
                    balance(activity, self.fragment_key)?
                        .checked_add(amount)
                        .ok_or(DivergentUniverseWorkbenchCurseError::BalanceOutOfRange)?;
                    operations.extend(self.fragments.credit_expression_operations(literal(
                        ActivityValue::BoundedInteger(amount),
                    )));
                    "GainCosmicFragments"
                }
                DivergentUniverseCurseChestOperation::LoseCosmicFragments { minimum, maximum } => {
                    let amount = bounded_amount(explicit_amount, *minimum, *maximum)?;
                    if balance(activity, self.fragment_key)? < amount {
                        return Err(DivergentUniverseWorkbenchCurseError::InsufficientBalance);
                    }
                    operations.push(ActivityOperation::AddCounter {
                        slot: CURRENCIES_SLOT,
                        key: self.fragment_key,
                        delta: literal(ActivityValue::BoundedInteger(-amount)),
                    });
                    "LoseCosmicFragments"
                }
                DivergentUniverseCurseChestOperation::LeaveWithoutMutation => {
                    if explicit_amount.is_some() {
                        return Err(DivergentUniverseWorkbenchCurseError::UnexpectedAmount);
                    }
                    "LeaveWithoutMutation"
                }
                DivergentUniverseCurseChestOperation::UnresolvedCandidateOperation { .. } => {
                    return Err(DivergentUniverseWorkbenchCurseError::UnpublishedCandidatePool);
                }
            };
        operations.push(ActivityOperation::AddCounter {
            slot: SERVICE_RECEIPTS_SLOT,
            key: chest.receipt_key,
            delta: literal(ActivityValue::BoundedInteger(1)),
        });
        self.apply(
            activity,
            expected,
            CURSE_CHEST_PROGRAM,
            operation_name,
            operations,
        )
    }

    fn require_current_workbench<'a>(
        &'a self,
        activity: &GraphActivity,
        id: &DivergentUniverseWorkbenchId,
    ) -> Result<&'a DivergentUniverseWorkbenchRuntimeDefinition, DivergentUniverseWorkbenchCurseError>
    {
        let workbench = self.workbench(id)?;
        if optional_key(activity, WORKBENCH_SLOT)? != Some(workbench.state_key) {
            return Err(DivergentUniverseWorkbenchCurseError::WorkbenchNotActive);
        }
        Ok(workbench)
    }

    fn workbench(
        &self,
        id: &DivergentUniverseWorkbenchId,
    ) -> Result<&DivergentUniverseWorkbenchRuntimeDefinition, DivergentUniverseWorkbenchCurseError>
    {
        lookup(&self.workbenches, |value| &value.id, id)
    }
    fn function(
        &self,
        id: &DivergentUniverseWorkbenchFunctionId,
    ) -> Result<&DivergentUniverseWorkbenchFunctionRuntime, DivergentUniverseWorkbenchCurseError>
    {
        lookup(&self.functions, |value| &value.id, id)
    }
    fn curse_chest(
        &self,
        id: &DivergentUniverseCurseChestId,
    ) -> Result<&DivergentUniverseCurseChestRuntimeDefinition, DivergentUniverseWorkbenchCurseError>
    {
        lookup(&self.curse_chests, |value| &value.id, id)
    }
    fn apply(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        raw_program: u32,
        operation: &str,
        operations: Vec<ActivityOperation>,
    ) -> Result<DivergentUniverseWorkbenchCurseResolution, DivergentUniverseWorkbenchCurseError>
    {
        let program = ActivityProgramDefinition::new(program_id(raw_program), operations)
            .map_err(|_| DivergentUniverseWorkbenchCurseError::InvalidProgram)?;
        let events = activity.apply_boundary_program(expected, &program)?;
        Ok(DivergentUniverseWorkbenchCurseResolution {
            operation: operation.into(),
            events,
            state_hash: activity.state_hash(),
        })
    }
}

fn function_kind(
    value: &str,
) -> Result<DivergentUniverseWorkbenchFunctionKind, DivergentUniverseWorkbenchCurseError> {
    match value {
        "BuffEnhance" => Ok(DivergentUniverseWorkbenchFunctionKind::BlessingEnhance),
        "BuffReforge" => Ok(DivergentUniverseWorkbenchFunctionKind::BlessingReforge),
        "FormulaReforge" => Ok(DivergentUniverseWorkbenchFunctionKind::EquationReforge),
        "MiracleCompose" => Ok(DivergentUniverseWorkbenchFunctionKind::CurioCompose),
        "MiracleReforge" => Ok(DivergentUniverseWorkbenchFunctionKind::CurioReforge),
        "HexEquipment" => Ok(DivergentUniverseWorkbenchFunctionKind::HexEquipment),
        _ => Err(DivergentUniverseWorkbenchCurseError::InvalidCatalog),
    }
}

fn lower_curse_operation(
    value: &starclock_data::divergent_universe_service_catalog::DivergentUniverseCurseChoice,
) -> Result<DivergentUniverseCurseChestOperation, DivergentUniverseWorkbenchCurseError> {
    match value.operation.as_ref() {
        "GainCosmicFragments" => Ok(DivergentUniverseCurseChestOperation::GainCosmicFragments {
            minimum: parse_required(value.minimum.as_deref())?,
            maximum: parse_required(value.maximum.as_deref())?,
        }),
        "LoseCosmicFragments" => Ok(DivergentUniverseCurseChestOperation::LoseCosmicFragments {
            minimum: parse_required(value.minimum.as_deref())?,
            maximum: parse_required(value.maximum.as_deref())?,
        }),
        "LeaveWithoutMutation" => Ok(DivergentUniverseCurseChestOperation::LeaveWithoutMutation),
        operation => Ok(
            DivergentUniverseCurseChestOperation::UnresolvedCandidateOperation {
                operation: operation.into(),
            },
        ),
    }
}

fn parse_required(value: Option<&str>) -> Result<u64, DivergentUniverseWorkbenchCurseError> {
    let value = value.ok_or(DivergentUniverseWorkbenchCurseError::InvalidCatalog)?;
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(DivergentUniverseWorkbenchCurseError::InvalidCatalog);
    }
    value
        .parse()
        .map_err(|_| DivergentUniverseWorkbenchCurseError::InvalidCatalog)
}

fn bounded_amount(
    amount: Option<u64>,
    minimum: u64,
    maximum: u64,
) -> Result<i64, DivergentUniverseWorkbenchCurseError> {
    let amount = amount.ok_or(DivergentUniverseWorkbenchCurseError::MissingAmount)?;
    if minimum == 0 || minimum > maximum || amount < minimum || amount > maximum {
        return Err(DivergentUniverseWorkbenchCurseError::AmountOutsideReleasedBounds);
    }
    checked_amount(amount)
}

fn checked_amount(value: u64) -> Result<i64, DivergentUniverseWorkbenchCurseError> {
    if value == 0 {
        return Err(DivergentUniverseWorkbenchCurseError::MissingAmount);
    }
    i64::try_from(value).map_err(|_| DivergentUniverseWorkbenchCurseError::BalanceOutOfRange)
}

fn balance(
    activity: &GraphActivity,
    key: u64,
) -> Result<i64, DivergentUniverseWorkbenchCurseError> {
    let values = counter(activity, CURRENCIES_SLOT)?;
    Ok(values
        .binary_search_by_key(&key, |value| value.0)
        .ok()
        .map_or(0, |index| values[index].1))
}

fn receipt_count(
    activity: &GraphActivity,
    key: u64,
) -> Result<i64, DivergentUniverseWorkbenchCurseError> {
    let values = counter(activity, SERVICE_RECEIPTS_SLOT)?;
    Ok(values
        .binary_search_by_key(&key, |value| value.0)
        .ok()
        .map_or(0, |index| values[index].1))
}

fn counter(
    activity: &GraphActivity,
    slot: starclock_activity::ActivitySlotId,
) -> Result<Box<[(u64, i64)]>, DivergentUniverseWorkbenchCurseError> {
    let view = activity.player_view();
    let value = view
        .slots()
        .iter()
        .find(|value| value.id() == slot)
        .ok_or(DivergentUniverseWorkbenchCurseError::InvalidState)?
        .value();
    let ActivityValue::BoundedCounterMap(values) = value else {
        return Err(DivergentUniverseWorkbenchCurseError::InvalidState);
    };
    Ok(values.clone())
}

fn optional_key(
    activity: &GraphActivity,
    slot: starclock_activity::ActivitySlotId,
) -> Result<Option<u64>, DivergentUniverseWorkbenchCurseError> {
    let view = activity.player_view();
    let value = view
        .slots()
        .iter()
        .find(|value| value.id() == slot)
        .ok_or(DivergentUniverseWorkbenchCurseError::InvalidState)?
        .value();
    let ActivityValue::OptionalId(value) = value else {
        return Err(DivergentUniverseWorkbenchCurseError::InvalidState);
    };
    Ok(*value)
}

fn function_receipt_key(
    function: &DivergentUniverseWorkbenchFunctionRuntime,
) -> Result<u64, DivergentUniverseWorkbenchCurseError> {
    let raw = function
        .id
        .as_str()
        .rsplit('.')
        .next()
        .ok_or(DivergentUniverseWorkbenchCurseError::InvalidCatalog)?
        .parse::<u64>()
        .map_err(|_| DivergentUniverseWorkbenchCurseError::InvalidCatalog)?;
    RECEIPT_KEY_BASE
        .checked_add(100)
        .and_then(|value| value.checked_add(raw))
        .ok_or(DivergentUniverseWorkbenchCurseError::InvalidCatalog)
}

fn ordinal(index: usize) -> Result<u64, DivergentUniverseWorkbenchCurseError> {
    u64::try_from(index)
        .ok()
        .and_then(|value| value.checked_add(1))
        .ok_or(DivergentUniverseWorkbenchCurseError::InvalidCatalog)
}

fn lookup<'a, T, I: Ord>(
    values: &'a [T],
    id: impl Fn(&T) -> &I,
    expected: &I,
) -> Result<&'a T, DivergentUniverseWorkbenchCurseError> {
    values
        .binary_search_by(|value| id(value).cmp(expected))
        .ok()
        .map(|index| &values[index])
        .ok_or(DivergentUniverseWorkbenchCurseError::UnknownIdentity)
}

fn validate_hash(
    activity: &GraphActivity,
    expected: ActivityStateHash,
) -> Result<(), DivergentUniverseWorkbenchCurseError> {
    if activity.state_hash() != expected {
        return Err(DivergentUniverseWorkbenchCurseError::Activity(
            GraphActivityCommandError::StaleStateHash,
        ));
    }
    if activity.player_view().terminal().is_some() {
        return Err(DivergentUniverseWorkbenchCurseError::ActivityCompleted);
    }
    Ok(())
}

fn literal(value: ActivityValue) -> ActivityExpression {
    ActivityExpression::Literal(value)
}

fn program_id(raw: u32) -> ActivityProgramId {
    ActivityProgramId::new(raw).expect("non-zero Workbench/Curse Chest program ID")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseWorkbenchCurseError {
    InvalidCatalog,
    UnknownIdentity,
    UnknownChoice,
    InvalidSelection,
    WorkbenchNotActive,
    FunctionUnavailable,
    WrongFunctionBoundary,
    UnpublishedTransformationProgram,
    UnpublishedCandidatePool,
    MissingAmount,
    UnexpectedAmount,
    AmountOutsideReleasedBounds,
    InsufficientBalance,
    BalanceOutOfRange,
    ChestAlreadyResolved,
    InvalidState,
    InvalidProgram,
    ActivityCompleted,
    Economy(DivergentUniverseEconomyError),
    Blessing(DivergentUniverseBlessingRuntimeError),
    Activity(GraphActivityCommandError),
}

impl From<DivergentUniverseEconomyError> for DivergentUniverseWorkbenchCurseError {
    fn from(value: DivergentUniverseEconomyError) -> Self {
        Self::Economy(value)
    }
}
impl From<DivergentUniverseBlessingRuntimeError> for DivergentUniverseWorkbenchCurseError {
    fn from(value: DivergentUniverseBlessingRuntimeError) -> Self {
        Self::Blessing(value)
    }
}
impl From<GraphActivityCommandError> for DivergentUniverseWorkbenchCurseError {
    fn from(value: GraphActivityCommandError) -> Self {
        Self::Activity(value)
    }
}
impl core::fmt::Display for DivergentUniverseWorkbenchCurseError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe Workbench/Curse Chest error: {self:?}"
        )
    }
}
impl std::error::Error for DivergentUniverseWorkbenchCurseError {}
