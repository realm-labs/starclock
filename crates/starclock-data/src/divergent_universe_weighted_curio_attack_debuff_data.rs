//! Released attack advance/reduction operands, separate from execution policy.

use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_curio_catalog::DivergentUniverseWeightedCurioId,
    divergent_universe_decisions::{DecisionDataError, key, validation::source_keys},
    divergent_universe_decisions_generated::{
        SoraConfig, du_curio_attack_debuff_policy::DuCurioAttackDebuffPolicy,
        du_decision_evidence::DuDecisionEvidence,
    },
};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightedCurioAttackDebuffPolicy {
    VersionedProjectPolicyAttackResolvedAdvanceAndTargetTurnFinalReduction,
}

/// Immutable operands and replaceable lifecycle/formula policy, not a Forge offer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedCurioAttackDebuffDefinition {
    pub key: Box<str>,
    pub weighted_curio: DivergentUniverseWeightedCurioId,
    pub maze_buff_id: Box<str>,
    pub advance_millionths: i64,
    pub reduction_millionths: i64,
    pub duration_turns: u16,
    pub policy: WeightedCurioAttackDebuffPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[WeightedCurioAttackDebuffDefinition]>, DecisionDataError> {
    let mut definitions = Vec::new();
    let mut identities = BTreeSet::new();
    for row in config.du_weighted_curio_attack_debuffs().ordered_rows() {
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
        let paths: Vec<&str> = row.character_paths.iter().map(String::as_str).collect();
        if row.id <= 0
            || curio.maze_buff_id.as_ref() != row.maze_buff_id
            || paths != ["Warrior", "Warlock"]
            || eligible
                .character_paths
                .iter()
                .map(AsRef::as_ref)
                .collect::<BTreeSet<&str>>()
                != paths.iter().copied().collect()
            || !eligible.elements.is_empty()
            || (
                row.advance_parameter,
                row.reduction_parameter,
                row.duration_parameter,
            ) != (1, 2, 3)
            || !identities.insert(curio.id.clone())
        {
            return Err(DecisionDataError::InvalidReference);
        }
        let advance_millionths = fraction(&row.advance_fraction)?;
        let reduction_millionths = fraction(&row.reduction_fraction)?;
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
        let hex_locator = format!(
            "HexID={};",
            curio
                .id
                .as_str()
                .rsplit('.')
                .next()
                .ok_or(DecisionDataError::InvalidReference)?
        );
        let maze_locator = format!("ID={};", row.maze_buff_id);
        for (prefix, locator) in [
            ("ExcelOutput/RogueTournHex.json;", hex_locator.as_str()),
            ("ExcelOutput/MazeBuff.json;", maze_locator.as_str()),
            ("TextMap/TextMapEN.json;", "hash=4217722633224787979;"),
            ("TextMap/TextMapCHS.json;", "hash=4217722633224787979;"),
        ] {
            if !row.source_ids.iter().any(|id| {
                config.du_decision_sources().get(id).is_some_and(|source| {
                    source.quality == DuDecisionEvidence::ExactStructured
                        && source.revision == "fd978d6ef09f941fba644c731ab54abd6f7c3568"
                        && source.locator.starts_with(prefix)
                        && source.locator.contains(locator)
                })
            }) {
                return Err(DecisionDataError::InvalidProvenance);
            }
        }
        definitions.push(WeightedCurioAttackDebuffDefinition {
            key: key(&row.stable_key)?, weighted_curio: curio.id.clone(),
            maze_buff_id: row.maze_buff_id.clone().into(), advance_millionths,
            reduction_millionths, duration_turns,
            policy: match row.policy {
                DuCurioAttackDebuffPolicy::AttackResolvedAdvanceAndTargetTurnFinalReduction =>
                    WeightedCurioAttackDebuffPolicy::VersionedProjectPolicyAttackResolvedAdvanceAndTargetTurnFinalReduction,
            },
            summary_en: row.summary_en.clone().into(), summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(), replacement_condition: row.replacement_condition.clone().into(), sources,
        });
    }
    definitions.sort_by(|a, b| a.weighted_curio.cmp(&b.weighted_curio));
    Ok(definitions.into_boxed_slice())
}

fn fraction(value: &str) -> Result<i64, DecisionDataError> {
    let digits = value
        .strip_prefix("0.")
        .filter(|digits| {
            !digits.is_empty()
                && digits.len() <= 6
                && !digits.ends_with('0')
                && digits.bytes().all(|byte| byte.is_ascii_digit())
        })
        .ok_or(DecisionDataError::InvalidReward)?;
    digits
        .parse::<i64>()
        .ok()
        .and_then(|value| {
            value.checked_mul(
                10_i64.checked_pow(6_u32.checked_sub(u32::try_from(digits.len()).ok()?)?)?,
            )
        })
        .filter(|value| *value > 0)
        .ok_or(DecisionDataError::InvalidReward)
}
