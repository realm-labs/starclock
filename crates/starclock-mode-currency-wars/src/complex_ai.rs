use std::collections::{BTreeMap, BTreeSet};

use starclock_combat::{NumericError, Rounding, Scalar};

pub const COMPLEX_AI_MULTIRANGE_POLICY_ID: &str = "currency-wars.complex-ai-multirange-policy.v1";
pub const COMPLEX_AI_SOURCE_AND_MULTIRANGE_POLICY_ID: &str =
    "currency-wars.complex-ai-source-and-multirange-policy.v1";

pub type CurrencyWarsComplexAiTeamTagValues = BTreeMap<(Box<str>, Box<str>), Box<[Scalar]>>;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsComplexAiCombineOperator {
    Add,
    Multiply,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsComplexAiFactorSource {
    CurrentHpRatio,
    AiTag(Box<str>),
    ContainsModifier(Box<str>),
    BattleGlobalData(Box<str>),
    CasterAiTag(Box<str>),
    CasterContainsModifier(Box<str>),
    TeamAiTagMaximum {
        team_type: Box<str>,
        key: Box<str>,
    },
    CombatPowerWeightedTarget {
        ai_tag_key: Box<str>,
        default_ai_tag_value: Scalar,
        power_of_combat_power: Scalar,
        power_of_damage_carry: Scalar,
        sum_up_servant_damage_carry: bool,
    },
    ValueInTeamRatio(Box<str>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsComplexAiRange {
    pub minimum_input: Option<Scalar>,
    pub minimum_output: Option<Scalar>,
    pub maximum_input: Option<Scalar>,
    pub maximum_output: Option<Scalar>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsComplexAiFactor {
    pub combine_operator: CurrencyWarsComplexAiCombineOperator,
    pub source: CurrencyWarsComplexAiFactorSource,
    pub ranges: Box<[CurrencyWarsComplexAiRange]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsComplexAiFactorGroup {
    pub stable_key: Box<str>,
    pub factors: Box<[CurrencyWarsComplexAiFactor]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsComplexAiGlobalFactors {
    pub groups: Box<[CurrencyWarsComplexAiFactorGroup]>,
    pub mapper_policy_id: Box<str>,
    pub selected_behavior: Box<str>,
    pub unresolved_field: Box<str>,
    pub confidence: Box<str>,
    pub replacement_condition: Box<str>,
    pub mechanical_shape_sha256: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsComplexAiContext {
    pub current_hp_ratio: Scalar,
    pub ai_tags: BTreeMap<Box<str>, Scalar>,
    pub modifiers: BTreeSet<Box<str>>,
    pub caster_ai_tags: BTreeMap<Box<str>, Scalar>,
    pub caster_modifiers: BTreeSet<Box<str>>,
    pub battle_global_values: BTreeMap<Box<str>, Scalar>,
    pub team_ai_tag_values: CurrencyWarsComplexAiTeamTagValues,
    pub value_in_team_ratios: BTreeMap<Box<str>, Scalar>,
    pub combat_power: Scalar,
    pub damage_carry: Scalar,
    pub servant_damage_carry: Scalar,
}

impl CurrencyWarsComplexAiContext {
    #[must_use]
    pub fn new(current_hp_ratio: Scalar) -> Self {
        Self {
            current_hp_ratio,
            ai_tags: BTreeMap::new(),
            modifiers: BTreeSet::new(),
            caster_ai_tags: BTreeMap::new(),
            caster_modifiers: BTreeSet::new(),
            battle_global_values: BTreeMap::new(),
            team_ai_tag_values: BTreeMap::new(),
            value_in_team_ratios: BTreeMap::new(),
            combat_power: Scalar::ZERO,
            damage_carry: Scalar::ZERO,
            servant_damage_carry: Scalar::ZERO,
        }
    }
}

impl CurrencyWarsComplexAiGlobalFactors {
    pub fn group(&self, stable_key: &str) -> Option<&CurrencyWarsComplexAiFactorGroup> {
        self.groups
            .binary_search_by(|group| group.stable_key.as_ref().cmp(stable_key))
            .ok()
            .map(|index| &self.groups[index])
    }
}

impl CurrencyWarsComplexAiFactorGroup {
    pub fn evaluate(&self, context: &CurrencyWarsComplexAiContext) -> Result<Scalar, NumericError> {
        self.factors.iter().try_fold(Scalar::ZERO, |score, factor| {
            let input = match &factor.source {
                CurrencyWarsComplexAiFactorSource::CurrentHpRatio => context.current_hp_ratio,
                CurrencyWarsComplexAiFactorSource::AiTag(key) => {
                    context.ai_tags.get(key).copied().unwrap_or(Scalar::ZERO)
                }
                CurrencyWarsComplexAiFactorSource::ContainsModifier(modifier) => {
                    if context.modifiers.contains(modifier) {
                        Scalar::ONE
                    } else {
                        Scalar::ZERO
                    }
                }
                CurrencyWarsComplexAiFactorSource::BattleGlobalData(key) => context
                    .battle_global_values
                    .get(key)
                    .copied()
                    .unwrap_or(Scalar::ZERO),
                CurrencyWarsComplexAiFactorSource::CasterAiTag(key) => context
                    .caster_ai_tags
                    .get(key)
                    .copied()
                    .unwrap_or(Scalar::ZERO),
                CurrencyWarsComplexAiFactorSource::CasterContainsModifier(modifier) => {
                    if context.caster_modifiers.contains(modifier) {
                        Scalar::ONE
                    } else {
                        Scalar::ZERO
                    }
                }
                CurrencyWarsComplexAiFactorSource::TeamAiTagMaximum { team_type, key } => context
                    .team_ai_tag_values
                    .get(&(team_type.clone(), key.clone()))
                    .and_then(|values| values.iter().copied().max())
                    .unwrap_or(Scalar::ZERO),
                CurrencyWarsComplexAiFactorSource::CombatPowerWeightedTarget {
                    ai_tag_key,
                    default_ai_tag_value,
                    power_of_combat_power,
                    power_of_damage_carry,
                    sum_up_servant_damage_carry,
                } => {
                    let tag = context
                        .ai_tags
                        .get(ai_tag_key)
                        .copied()
                        .unwrap_or(*default_ai_tag_value);
                    let damage_carry = if *sum_up_servant_damage_carry {
                        context
                            .damage_carry
                            .checked_add(context.servant_damage_carry)?
                    } else {
                        context.damage_carry
                    };
                    let combat_power =
                        checked_integer_power(context.combat_power, *power_of_combat_power)?;
                    let damage_carry = checked_integer_power(damage_carry, *power_of_damage_carry)?;
                    tag.checked_mul(combat_power, Rounding::NearestTiesEven)?
                        .checked_mul(damage_carry, Rounding::NearestTiesEven)?
                }
                CurrencyWarsComplexAiFactorSource::ValueInTeamRatio(key) => context
                    .value_in_team_ratios
                    .get(key)
                    .copied()
                    .unwrap_or(Scalar::ZERO),
            };
            let value = evaluate_ranges(&factor.ranges, input)?;
            match factor.combine_operator {
                CurrencyWarsComplexAiCombineOperator::Add => score.checked_add(value),
                CurrencyWarsComplexAiCombineOperator::Multiply => {
                    score.checked_mul(value, Rounding::NearestTiesEven)
                }
            }
        })
    }
}

fn checked_integer_power(base: Scalar, authored_power: Scalar) -> Result<Scalar, NumericError> {
    let exponent = authored_power.rounded_integer(Rounding::NearestTiesEven)?;
    let exponent = u8::try_from(exponent).map_err(|_| NumericError::OutOfDomain)?;
    if exponent > 32 {
        return Err(NumericError::OutOfDomain);
    }
    (0..exponent).try_fold(Scalar::ONE, |value, _| {
        value.checked_mul(base, Rounding::NearestTiesEven)
    })
}

fn evaluate_ranges(
    ranges: &[CurrencyWarsComplexAiRange],
    input: Scalar,
) -> Result<Scalar, NumericError> {
    let range = ranges
        .iter()
        .find(|range| {
            let minimum = range.minimum_input.unwrap_or(Scalar::ZERO);
            let maximum = range.maximum_input.unwrap_or(Scalar::ZERO);
            input >= minimum && input <= maximum
        })
        .or_else(|| {
            ranges
                .first()
                .filter(|range| input < range.minimum_input.unwrap_or(Scalar::ZERO))
        })
        .or_else(|| ranges.last())
        .expect("validated Complex AI factor groups have at least one range");
    let minimum_input = range.minimum_input.unwrap_or(Scalar::ZERO);
    let maximum_input = range.maximum_input.unwrap_or(Scalar::ZERO);
    let minimum_output = range.minimum_output.unwrap_or(Scalar::ZERO);
    let maximum_output = range.maximum_output.unwrap_or(Scalar::ZERO);
    let clamped = input.max(minimum_input).min(maximum_input);
    let width = maximum_input.checked_sub(minimum_input)?;
    if width == Scalar::ZERO {
        return Ok(maximum_output);
    }
    let position = clamped
        .checked_sub(minimum_input)?
        .checked_div(width, Rounding::NearestTiesEven)?;
    maximum_output
        .checked_sub(minimum_output)?
        .checked_mul(position, Rounding::NearestTiesEven)?
        .checked_add(minimum_output)
}
