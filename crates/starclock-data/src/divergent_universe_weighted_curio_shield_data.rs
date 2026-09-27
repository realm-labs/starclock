//! Released ally-directed Harmony shields with explicit hidden-lifecycle policy.

use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_curio_catalog::DivergentUniverseWeightedCurioId,
    divergent_universe_decisions::{DecisionDataError, key, validation::source_keys},
    divergent_universe_decisions_generated::{
        SoraConfig, du_decision_evidence::DuDecisionEvidence,
        du_weighted_curio_shield_policy::DuWeightedCurioShieldPolicy,
    },
};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightedCurioShieldPolicy {
    VersionedProjectPolicyAllyActionResolvedReplaceTargetTurnShield,
}

/// Immutable released operands, never a complete Forge offer or reference-program receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedCurioShieldDefinition {
    pub key: Box<str>,
    pub weighted_curio: DivergentUniverseWeightedCurioId,
    pub maze_buff_id: Box<str>,
    pub fraction_millionths: i64,
    pub duration_turns: u16,
    pub policy: WeightedCurioShieldPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[WeightedCurioShieldDefinition]>, DecisionDataError> {
    let mut definitions = Vec::new();
    let mut identities = BTreeSet::new();
    for row in config.du_weighted_curio_shields().ordered_rows() {
        let curio = reference
            .curio_catalog()
            .weighted_curios()
            .iter()
            .find(|curio| curio.id.as_str() == row.weighted_curio_key)
            .ok_or(DecisionDataError::InvalidReference)?;
        let eligible = reference
            .curio_catalog()
            .weighted_curio_eligibility()
            .iter()
            .find(|eligible| curio.eligibility_rules.contains(&eligible.id))
            .ok_or(DecisionDataError::InvalidReference)?;
        if row.id <= 0
            || curio.maze_buff_id.as_ref() != row.maze_buff_id
            || row.character_path != "Shaman"
            || row.fraction_parameter != 1
            || row.duration_parameter != 2
            || eligible.character_paths.as_ref() != [Box::<str>::from("Shaman")]
            || !eligible.elements.is_empty()
            || !identities.insert(curio.id.clone())
        {
            return Err(DecisionDataError::InvalidReference);
        }
        let digits = row
            .shield_fraction
            .strip_prefix("0.")
            .filter(|digits| {
                !digits.is_empty()
                    && digits.len() <= 6
                    && !digits.ends_with('0')
                    && digits.bytes().all(|byte| byte.is_ascii_digit())
            })
            .ok_or(DecisionDataError::InvalidReward)?;
        let fraction = digits
            .parse::<i64>()
            .ok()
            .and_then(|value| {
                value.checked_mul(10_i64.checked_pow(6 - u32::try_from(digits.len()).ok()?)?)
            })
            .filter(|value| *value > 0)
            .ok_or(DecisionDataError::InvalidReward)?;
        let duration_turns = u16::try_from(row.duration_turns)
            .ok()
            .filter(|turns| *turns > 0)
            .ok_or(DecisionDataError::InvalidPolicy)?;
        if [
            &row.summary_en,
            &row.summary_zh_cn,
            &row.policy_note,
            &row.replacement_condition,
        ]
        .iter()
        .any(|value| value.trim().is_empty())
        {
            return Err(DecisionDataError::InvalidPolicy);
        }
        let sources = source_keys(config, &row.source_ids)?;
        for prefix in [
            "ExcelOutput/RogueTournHex.json;",
            "ExcelOutput/MazeBuff.json;",
            "TextMap/TextMapEN.json;",
            "TextMap/TextMapCHS.json;",
        ] {
            if !row.source_ids.iter().any(|id| {
                config.du_decision_sources().get(id).is_some_and(|source| {
                    source.quality == DuDecisionEvidence::ExactStructured
                        && source.revision == "fd978d6ef09f941fba644c731ab54abd6f7c3568"
                        && source.locator.starts_with(prefix)
                })
            }) {
                return Err(DecisionDataError::InvalidProvenance);
            }
        }
        definitions.push(WeightedCurioShieldDefinition {
            key: key(&row.stable_key)?,
            weighted_curio: curio.id.clone(),
            maze_buff_id: row.maze_buff_id.clone().into(),
            fraction_millionths: fraction,
            duration_turns,
            policy: match row.policy {
                DuWeightedCurioShieldPolicy::AllyActionResolvedReplaceTargetTurnShield => {
                    WeightedCurioShieldPolicy::VersionedProjectPolicyAllyActionResolvedReplaceTargetTurnShield
                }
            },
            summary_en: row.summary_en.clone().into(),
            summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(),
            replacement_condition: row.replacement_condition.clone().into(),
            sources,
        });
    }
    definitions.sort_by(|a, b| a.weighted_curio.cmp(&b.weighted_curio));
    Ok(definitions.into_boxed_slice())
}
