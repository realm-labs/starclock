//! Difficulty, Threshold Protocol and Astronomical Division runtime lowering.

use starclock_combat::Ratio;
use starclock_data::{
    divergent_universe_catalog::{
        DivergentUniverseCyclicalChallengeDefinition, DivergentUniverseCyclicalChallengeId,
        DivergentUniverseDifficultyDefinition, DivergentUniverseRunFamily,
    },
    divergent_universe_progression_catalog::{
        DivergentUniverseDivisionDefinition, DivergentUniverseDivisionId,
        DivergentUniverseProgressionCatalog, DivergentUniverseProtocolDefinition,
        DivergentUniverseProtocolId,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseAstronomicalMode {
    StarPioneer,
    Practice,
}

/// Immutable account-derived selection used only to compile run-owned state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseAstronomicalEntry {
    pub(super) mode: DivergentUniverseAstronomicalMode,
    pub(super) division: DivergentUniverseDivisionId,
    pub(super) protocol: DivergentUniverseProtocolId,
    pub(super) cognoculi: u16,
    pub(super) ordinary_difficulty_five_complete: bool,
}

impl DivergentUniverseAstronomicalEntry {
    #[must_use]
    pub fn star_pioneer(
        division: DivergentUniverseDivisionId,
        protocol: DivergentUniverseProtocolId,
        cognoculi: u16,
        ordinary_difficulty_five_complete: bool,
    ) -> Self {
        Self {
            mode: DivergentUniverseAstronomicalMode::StarPioneer,
            division,
            protocol,
            cognoculi,
            ordinary_difficulty_five_complete,
        }
    }

    #[must_use]
    pub fn practice(
        division: DivergentUniverseDivisionId,
        protocol: DivergentUniverseProtocolId,
        cognoculi: u16,
        ordinary_difficulty_five_complete: bool,
    ) -> Self {
        Self {
            mode: DivergentUniverseAstronomicalMode::Practice,
            division,
            protocol,
            cognoculi,
            ordinary_difficulty_five_complete,
        }
    }
}

/// Caller-observed cycle identity. A changed epoch compiles a fresh Activity identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseCyclicalRefresh {
    pub(super) epoch: u64,
    pub(super) challenge: DivergentUniverseCyclicalChallengeId,
}

impl DivergentUniverseCyclicalRefresh {
    pub fn new(
        epoch: u64,
        challenge: DivergentUniverseCyclicalChallengeId,
    ) -> Result<Self, DivergentUniverseProgressionRuntimeError> {
        if epoch == 0 {
            return Err(DivergentUniverseProgressionRuntimeError::InvalidCyclicalEpoch);
        }
        Ok(Self { epoch, challenge })
    }

    #[must_use]
    pub const fn epoch(&self) -> u64 {
        self.epoch
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseProtocolContribution {
    attack_increase: Ratio,
    maximum_hp_increase: Ratio,
    speed_increase: Ratio,
    maximum_toughness_increase: Option<Ratio>,
    difficulty_changes: Box<[DivergentUniverseProtocolRule]>,
    entry_rules: Box<[DivergentUniverseProtocolRule]>,
    berserk_changes: Box<[DivergentUniverseProtocolRule]>,
    boss_identity: Box<str>,
}

impl DivergentUniverseProtocolContribution {
    #[must_use]
    pub const fn attack_increase(&self) -> Ratio {
        self.attack_increase
    }
    #[must_use]
    pub const fn maximum_hp_increase(&self) -> Ratio {
        self.maximum_hp_increase
    }
    #[must_use]
    pub const fn speed_increase(&self) -> Ratio {
        self.speed_increase
    }
    #[must_use]
    pub const fn maximum_toughness_increase(&self) -> Option<Ratio> {
        self.maximum_toughness_increase
    }
    #[must_use]
    pub fn difficulty_changes(&self) -> &[DivergentUniverseProtocolRule] {
        &self.difficulty_changes
    }
    #[must_use]
    pub fn entry_rules(&self) -> &[DivergentUniverseProtocolRule] {
        &self.entry_rules
    }
    #[must_use]
    pub fn berserk_changes(&self) -> &[DivergentUniverseProtocolRule] {
        &self.berserk_changes
    }
    #[must_use]
    pub fn boss_identity(&self) -> &str {
        &self.boss_identity
    }
}

/// Closed typed lowering of every released Threshold Protocol rule label.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseProtocolRule {
    IncreaseDomainCount,
    EnableSecondPlaneConversionDomains,
    IncreaseEquationRandomness,
    IncreaseMaskWishpowerRequirement,
    IncreaseEquationBlessingRequirement,
    ReplaceFirstAndSecondPlaneBosses,
    AdvanceBerserkOnset,
    IncreaseBerserkStackRate,
    FurtherIncreaseDomainCount,
    GrantOneGrandMiracleAtFirstPlaneEntry,
    AdvanceBerserkEnemyAfterAttacked,
    IncreaseAllyDamageAfterBerserk,
    DecreaseAllyHealingAfterBerserk,
    DecreaseAllyShieldAfterBerserk,
    IncreaseStorePrice { ratio: Ratio },
    GrantRandomLevelOneDomains { count: u8 },
    GrantRandomSpecialAbsoluteFailurePrescriptionAtFirstPlaneEntry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseProgressionProjection {
    difficulty_levels: Box<[u16]>,
    mode: Option<DivergentUniverseAstronomicalMode>,
    selected_division: Option<DivergentUniverseDivisionId>,
    selected_protocol: Option<DivergentUniverseProtocolId>,
    initial_cognoculi: u16,
    settled_division: Option<DivergentUniverseDivisionId>,
    settled_cognoculi: u16,
    protocol_contribution: Option<DivergentUniverseProtocolContribution>,
    cognoculi_retention: Option<DivergentUniverseCognoculiRetention>,
    cyclical_epoch: Option<u64>,
}

impl DivergentUniverseProgressionProjection {
    #[must_use]
    pub fn difficulty_levels(&self) -> &[u16] {
        &self.difficulty_levels
    }
    #[must_use]
    pub const fn mode(&self) -> Option<DivergentUniverseAstronomicalMode> {
        self.mode
    }
    #[must_use]
    pub const fn selected_division(&self) -> Option<&DivergentUniverseDivisionId> {
        self.selected_division.as_ref()
    }
    #[must_use]
    pub const fn selected_protocol(&self) -> Option<&DivergentUniverseProtocolId> {
        self.selected_protocol.as_ref()
    }
    #[must_use]
    pub const fn initial_cognoculi(&self) -> u16 {
        self.initial_cognoculi
    }
    #[must_use]
    pub const fn settled_division(&self) -> Option<&DivergentUniverseDivisionId> {
        self.settled_division.as_ref()
    }
    #[must_use]
    pub const fn settled_cognoculi(&self) -> u16 {
        self.settled_cognoculi
    }
    #[must_use]
    pub const fn protocol_contribution(&self) -> Option<&DivergentUniverseProtocolContribution> {
        self.protocol_contribution.as_ref()
    }
    #[must_use]
    pub const fn cyclical_epoch(&self) -> Option<u64> {
        self.cyclical_epoch
    }
    /// Deterministic replaceable policy for a future unsuccessful terminal.
    /// The current entry slice has no battle-driven failure path, so this is a
    /// projection only and cannot mutate a live Activity.
    #[must_use]
    pub fn unsuccessful_cognoculi(&self, completed_layers: u16) -> u16 {
        if self.mode == Some(DivergentUniverseAstronomicalMode::Practice) {
            return self.initial_cognoculi;
        }
        match self.cognoculi_retention {
            Some(DivergentUniverseCognoculiRetention::NeverExtinguish) => self.initial_cognoculi,
            Some(DivergentUniverseCognoculiRetention::RetainAfterFirstLayer)
                if completed_layers >= 1 =>
            {
                self.initial_cognoculi
            }
            Some(DivergentUniverseCognoculiRetention::RetainAfterSecondLayer)
                if completed_layers >= 2 =>
            {
                self.initial_cognoculi
            }
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseCognoculiRetention {
    NoPublishedRetentionHint,
    NeverExtinguish,
    RetainAfterFirstLayer,
    RetainAfterSecondLayer,
}

#[derive(Clone, Debug)]
pub(super) struct CompiledProgression {
    pub(super) projection: DivergentUniverseProgressionProjection,
    pub(super) mode_value: Option<u64>,
    pub(super) division_value: Option<u64>,
    pub(super) protocol_value: Option<u64>,
    pub(super) settled_division_value: Option<u64>,
}

pub(super) fn compile(
    catalog: &DivergentUniverseProgressionCatalog,
    difficulty: &DivergentUniverseDifficultyDefinition,
    astronomical: Option<&DivergentUniverseAstronomicalEntry>,
    run_family: DivergentUniverseRunFamily,
    challenge: Option<&DivergentUniverseCyclicalChallengeDefinition>,
    refresh: Option<&DivergentUniverseCyclicalRefresh>,
) -> Result<CompiledProgression, DivergentUniverseProgressionRuntimeError> {
    if run_family == DivergentUniverseRunFamily::Cyclical && astronomical.is_some() {
        return Err(DivergentUniverseProgressionRuntimeError::AstronomicalModeCyclicalMismatch);
    }
    let cycle_epoch = match (run_family, challenge, refresh) {
        (DivergentUniverseRunFamily::Ordinary, None, None) => None,
        (DivergentUniverseRunFamily::Cyclical, Some(expected), Some(value))
            if value.challenge == expected.id =>
        {
            Some(value.epoch)
        }
        (DivergentUniverseRunFamily::Cyclical, Some(_), None) => {
            return Err(DivergentUniverseProgressionRuntimeError::MissingCyclicalRefresh);
        }
        (DivergentUniverseRunFamily::Cyclical, Some(_), Some(_)) => {
            return Err(DivergentUniverseProgressionRuntimeError::CyclicalChallengeMismatch);
        }
        _ => return Err(DivergentUniverseProgressionRuntimeError::UnexpectedCyclicalRefresh),
    };
    let Some(entry) = astronomical else {
        return Ok(CompiledProgression {
            projection: DivergentUniverseProgressionProjection {
                difficulty_levels: difficulty.levels.clone(),
                mode: None,
                selected_division: None,
                selected_protocol: None,
                initial_cognoculi: 0,
                settled_division: None,
                settled_cognoculi: 0,
                protocol_contribution: None,
                cognoculi_retention: None,
                cyclical_epoch: cycle_epoch,
            },
            mode_value: None,
            division_value: None,
            protocol_value: None,
            settled_division_value: None,
        });
    };
    if !entry.ordinary_difficulty_five_complete {
        return Err(DivergentUniverseProgressionRuntimeError::AstronomicalModeLocked);
    }
    let division_index = catalog
        .divisions()
        .iter()
        .position(|value| value.id == entry.division)
        .ok_or(DivergentUniverseProgressionRuntimeError::UnknownDivision)?;
    let division = &catalog.divisions()[division_index];
    let protocol_index = catalog
        .protocols()
        .iter()
        .position(|value| value.id == entry.protocol)
        .ok_or(DivergentUniverseProgressionRuntimeError::UnknownProtocol)?;
    let protocol = &catalog.protocols()[protocol_index];
    let boundary = progress_boundary(division)?;
    if entry.cognoculi >= boundary {
        return Err(DivergentUniverseProgressionRuntimeError::CognoculiOutOfBounds);
    }
    let mode_index = mode_index(catalog, entry.mode)?;
    match entry.mode {
        DivergentUniverseAstronomicalMode::StarPioneer => {
            if division.protocols.as_ref() != [entry.protocol.clone()] {
                return Err(DivergentUniverseProgressionRuntimeError::ProtocolDivisionMismatch);
            }
        }
        DivergentUniverseAstronomicalMode::Practice => {
            let cap = division.level.min(8);
            if protocol.level > cap {
                return Err(DivergentUniverseProgressionRuntimeError::ProtocolAboveDivisionCap);
            }
        }
    }
    let (settled_index, settled_cognoculi) = match entry.mode {
        DivergentUniverseAstronomicalMode::Practice => (division_index, entry.cognoculi),
        DivergentUniverseAstronomicalMode::StarPioneer if entry.cognoculi + 1 >= boundary => {
            let next = division_index
                .checked_add(1)
                .filter(|index| *index < catalog.divisions().len())
                .ok_or(DivergentUniverseProgressionRuntimeError::TerminalDivisionEntry)?;
            (next, 0)
        }
        DivergentUniverseAstronomicalMode::StarPioneer => (division_index, entry.cognoculi + 1),
    };
    let settled = &catalog.divisions()[settled_index];
    Ok(CompiledProgression {
        projection: DivergentUniverseProgressionProjection {
            difficulty_levels: difficulty.levels.clone(),
            mode: Some(entry.mode),
            selected_division: Some(division.id.clone()),
            selected_protocol: Some(protocol.id.clone()),
            initial_cognoculi: entry.cognoculi,
            settled_division: Some(settled.id.clone()),
            settled_cognoculi,
            protocol_contribution: Some(protocol_contribution(protocol)?),
            cognoculi_retention: Some(retention(division)?),
            cyclical_epoch: cycle_epoch,
        },
        mode_value: Some(index_value(mode_index)?),
        division_value: Some(index_value(division_index)?),
        protocol_value: Some(index_value(protocol_index)?),
        settled_division_value: Some(index_value(settled_index)?),
    })
}

fn retention(
    division: &DivergentUniverseDivisionDefinition,
) -> Result<DivergentUniverseCognoculiRetention, DivergentUniverseProgressionRuntimeError> {
    match division.cognoculi_retention.as_ref() {
        "NoPublishedRetentionHint" => {
            Ok(DivergentUniverseCognoculiRetention::NoPublishedRetentionHint)
        }
        "NeverExtinguish" => Ok(DivergentUniverseCognoculiRetention::NeverExtinguish),
        "RetainAfterFirstPlaneClear" => {
            Ok(DivergentUniverseCognoculiRetention::RetainAfterFirstLayer)
        }
        "RetainAfterSecondPlaneClear" => {
            Ok(DivergentUniverseCognoculiRetention::RetainAfterSecondLayer)
        }
        _ => Err(DivergentUniverseProgressionRuntimeError::UnknownCognoculiRetention),
    }
}

fn mode_index(
    catalog: &DivergentUniverseProgressionCatalog,
    selected: DivergentUniverseAstronomicalMode,
) -> Result<usize, DivergentUniverseProgressionRuntimeError> {
    let kind = match selected {
        DivergentUniverseAstronomicalMode::StarPioneer => "StarPioneer",
        DivergentUniverseAstronomicalMode::Practice => "Practice",
    };
    catalog
        .modes()
        .iter()
        .position(|value| value.mode_kind.as_ref() == kind)
        .ok_or(DivergentUniverseProgressionRuntimeError::UnknownAstronomicalMode)
}

fn progress_boundary(
    division: &DivergentUniverseDivisionDefinition,
) -> Result<u16, DivergentUniverseProgressionRuntimeError> {
    if division.progress_boundary.as_ref() == "Terminal" {
        return Err(DivergentUniverseProgressionRuntimeError::TerminalDivisionEntry);
    }
    division
        .progress_boundary
        .parse::<u16>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or(DivergentUniverseProgressionRuntimeError::InvalidProgressBoundary)
}

fn protocol_contribution(
    protocol: &DivergentUniverseProtocolDefinition,
) -> Result<DivergentUniverseProtocolContribution, DivergentUniverseProgressionRuntimeError> {
    Ok(DivergentUniverseProtocolContribution {
        attack_increase: ratio(&protocol.plane_scaling.attack)?,
        maximum_hp_increase: ratio(&protocol.plane_scaling.max_hp)?,
        speed_increase: ratio(&protocol.plane_scaling.speed)?,
        maximum_toughness_increase: protocol
            .plane_scaling
            .max_toughness
            .as_deref()
            .map(ratio)
            .transpose()?,
        difficulty_changes: protocol
            .difficulty_changes
            .iter()
            .map(|value| protocol_rule(value))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        entry_rules: protocol
            .entry_rules
            .iter()
            .map(|value| protocol_rule(value))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        berserk_changes: protocol
            .berserk_changes
            .iter()
            .map(|value| protocol_rule(value))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        boss_identity: protocol.boss_identity.clone(),
    })
}

fn protocol_rule(
    value: &str,
) -> Result<DivergentUniverseProtocolRule, DivergentUniverseProgressionRuntimeError> {
    use DivergentUniverseProtocolRule as Rule;
    match value {
        "IncreaseDomainCount" => Ok(Rule::IncreaseDomainCount),
        "EnableSecondPlaneConversionDomains" => Ok(Rule::EnableSecondPlaneConversionDomains),
        "IncreaseEquationRandomness" => Ok(Rule::IncreaseEquationRandomness),
        "IncreaseMaskWishpowerRequirement" => Ok(Rule::IncreaseMaskWishpowerRequirement),
        "IncreaseEquationBlessingRequirement" => Ok(Rule::IncreaseEquationBlessingRequirement),
        "ReplaceFirstAndSecondPlaneBosses" => Ok(Rule::ReplaceFirstAndSecondPlaneBosses),
        "AdvanceBerserkOnset" | "EarlierOnset" => Ok(Rule::AdvanceBerserkOnset),
        "IncreaseBerserkStackRate" | "FasterStacking" => Ok(Rule::IncreaseBerserkStackRate),
        "FurtherIncreaseDomainCount" => Ok(Rule::FurtherIncreaseDomainCount),
        "GrantOneGrandMiracleAtFirstPlaneEntry" => Ok(Rule::GrantOneGrandMiracleAtFirstPlaneEntry),
        "AdvanceBerserkEnemyAfterAttacked" | "AdvanceAfterAttacked" => {
            Ok(Rule::AdvanceBerserkEnemyAfterAttacked)
        }
        "IncreaseAllyDamageAfterBerserk" => Ok(Rule::IncreaseAllyDamageAfterBerserk),
        "DecreaseAllyHealingAfterBerserk" => Ok(Rule::DecreaseAllyHealingAfterBerserk),
        "DecreaseAllyShieldAfterBerserk" => Ok(Rule::DecreaseAllyShieldAfterBerserk),
        "IncreaseStorePriceBy0.25" => Ok(Rule::IncreaseStorePrice {
            ratio: Ratio::from_scaled(250_000),
        }),
        "GrantTwoRandomLevelOneDomainsAtFirstPlaneEntry" => {
            Ok(Rule::GrantRandomLevelOneDomains { count: 2 })
        }
        "GrantRandomSpecialAbsoluteFailurePrescriptionAtFirstPlaneEntry" => {
            Ok(Rule::GrantRandomSpecialAbsoluteFailurePrescriptionAtFirstPlaneEntry)
        }
        _ => Err(DivergentUniverseProgressionRuntimeError::UnknownProtocolRule),
    }
}

fn ratio(value: &str) -> Result<Ratio, DivergentUniverseProgressionRuntimeError> {
    let (integer, fractional) = match value.split_once('.') {
        Some((integer, fractional)) => (integer, fractional),
        None => (value, ""),
    };
    if integer.is_empty()
        || (integer.len() > 1 && integer.starts_with('0'))
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || fractional.len() > 6
        || !fractional.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(DivergentUniverseProgressionRuntimeError::InvalidProtocolDecimal);
    }
    let whole = integer
        .parse::<i64>()
        .map_err(|_| DivergentUniverseProgressionRuntimeError::InvalidProtocolDecimal)?;
    let fraction = if fractional.is_empty() {
        0
    } else {
        let exponent = u32::try_from(6_usize - fractional.len())
            .map_err(|_| DivergentUniverseProgressionRuntimeError::InvalidProtocolDecimal)?;
        fractional
            .parse::<i64>()
            .map_err(|_| DivergentUniverseProgressionRuntimeError::InvalidProtocolDecimal)?
            * 10_i64.pow(exponent)
    };
    let scaled = whole
        .checked_mul(1_000_000)
        .and_then(|value| value.checked_add(fraction))
        .ok_or(DivergentUniverseProgressionRuntimeError::InvalidProtocolDecimal)?;
    Ok(Ratio::from_scaled(scaled))
}

fn index_value(index: usize) -> Result<u64, DivergentUniverseProgressionRuntimeError> {
    u64::try_from(index + 1)
        .map_err(|_| DivergentUniverseProgressionRuntimeError::InvalidStableIndex)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseProgressionRuntimeError {
    AstronomicalModeLocked,
    UnknownAstronomicalMode,
    UnknownDivision,
    UnknownProtocol,
    TerminalDivisionEntry,
    ProtocolDivisionMismatch,
    ProtocolAboveDivisionCap,
    CognoculiOutOfBounds,
    InvalidProgressBoundary,
    InvalidProtocolDecimal,
    UnknownProtocolRule,
    UnknownCognoculiRetention,
    InvalidStableIndex,
    InvalidCyclicalEpoch,
    MissingCyclicalRefresh,
    UnexpectedCyclicalRefresh,
    CyclicalChallengeMismatch,
    AstronomicalModeCyclicalMismatch,
}

impl std::fmt::Display for DivergentUniverseProgressionRuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid Divergent Universe progression runtime: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseProgressionRuntimeError {}
