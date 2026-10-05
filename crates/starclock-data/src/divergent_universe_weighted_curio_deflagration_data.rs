//! Deflagration facts and authored policies; no native battle admission.
use crate::{
    catalog::parse_decimal,
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_curio_catalog::DivergentUniverseWeightedCurioId,
    divergent_universe_decisions::{DecisionDataError, key, validation::source_keys},
    divergent_universe_decisions_generated::{
        SoraConfig, du_decision_evidence::DuDecisionEvidence,
        du_decision_sources::DuDecisionSources,
        du_deflagration_base_policy::DuDeflagrationBasePolicy,
        du_deflagration_policy::DuDeflagrationPolicy, du_deflagration_status::DuDeflagrationStatus,
    },
};
use sha2::{Digest, Sha256};

/// Data readiness only; this status must not admit an equipped battle effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightedCurioDeflagrationStatus {
    AuthoredOperandsPendingNative,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightedCurioDeflagrationPolicy {
    VersionedProjectPolicyOriginalFireAfterActionNaturalTickBurns,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightedCurioDeflagrationBasePolicy {
    VersionedProjectPolicyTargetGroupOneHpRatioProtocolHpFloor,
}

/// Immutable exact operands plus independently replaceable hidden semantics.
/// Loading validates production data, not equipment or command execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedCurioDeflagrationDefinition {
    pub key: Box<str>,
    pub weighted_curio: DivergentUniverseWeightedCurioId,
    pub maze_buff_id: Box<str>,
    pub burn_fractions_millionths: [i64; 4],
    pub damage_multiplier_millionths: i64,
    pub duration_turns: u16,
    pub fixed_base_damage_millionths: i64,
    pub hard_level_group: u16,
    pub hp_ratios_millionths: Box<[i64]>,
    pub status: WeightedCurioDeflagrationStatus,
    pub policy: WeightedCurioDeflagrationPolicy,
    pub base_policy: WeightedCurioDeflagrationBasePolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub runtime_policy_note: Box<str>,
    pub runtime_replacement_condition: Box<str>,
    pub base_policy_note: Box<str>,
    pub base_replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[WeightedCurioDeflagrationDefinition]>, DecisionDataError> {
    let rows = config
        .du_weighted_curio_deflagrations()
        .ordered_rows()
        .collect::<Vec<_>>();
    if rows.len() != 1 {
        return Err(DecisionDataError::InvalidReference);
    }
    let row = rows[0];
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
    if row.id != 1
        || row.stable_key != "du.weighted-curio-deflagration.most-raucous"
        || curio.id.as_str() != "divergent-universe.weighted-curio.1012"
        || row.maze_buff_id != "633412"
        || curio.maze_buff_id.as_ref() != row.maze_buff_id
        || row.elements != ["Fire"]
        || !eligible.character_paths.is_empty()
        || eligible
            .elements
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<&str>>()
            != ["Fire"]
        || (row.damage_parameter, row.duration_parameter) != (5, 6)
    {
        return Err(DecisionDataError::InvalidReference);
    }
    if row.burn_fractions != ["0.5", "1", "1.5", "2"]
        || row.damage_multiplier != "2"
        || row.duration_turns != 2
        || row.base_fixed_damage != "100"
        || row.base_hard_level_group != 1
    {
        return Err(DecisionDataError::InvalidReward);
    }
    if [
        &row.summary_en,
        &row.summary_zh_cn,
        &row.runtime_replacement_condition,
        &row.base_replacement_condition,
    ]
    .iter()
    .any(|value| value.trim().is_empty())
        || !row
            .runtime_policy_note
            .starts_with("VersionedProjectPolicy:")
        || !row.base_policy_note.starts_with("VersionedProjectPolicy:")
    {
        return Err(DecisionDataError::InvalidPolicy);
    }
    if row.source_ids != [141, 142, 143, 144, 145, 146, 147, 148] {
        return Err(DecisionDataError::InvalidProvenance);
    }
    let sources = source_keys(config, &row.source_ids)?;
    for (id, suffix, prefix, locator, digest) in [
        (
            141,
            "hex",
            "ExcelOutput/RogueTournHex.json;",
            "HexID=1012;",
            "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455",
        ),
        (
            142,
            "maze-buff",
            "ExcelOutput/MazeBuff.json;",
            "ID=633412;",
            "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac",
        ),
        (
            143,
            "text-en",
            "TextMap/TextMapEN.json;",
            "hash=8602529829111077351;",
            "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789",
        ),
        (
            144,
            "text-zh",
            "TextMap/TextMapCHS.json;",
            "hash=8602529829111077351;",
            "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147",
        ),
        (
            145,
            "program",
            "Config/ConfigAbility/Level/Rogue_S7/Level_RogueBuff_Ability_HEX_S7.json;",
            "Name=StageAbility_633412;",
            "5ad6c48751023184f5572e0069c5d02c841aef4de3cec092d0c2bc4913d17e47",
        ),
        (
            146,
            "hard-level",
            "ExcelOutput/HardLevelGroup.json;",
            "HardLevelGroup=1; Level=1..95;",
            "d185c09b5388f4eeb368199276ee8a815b406fd9902744027b72a5011b962978",
        ),
    ] {
        let source = config
            .du_decision_sources()
            .get(&id)
            .ok_or(DecisionDataError::InvalidProvenance)?;
        if !context(source)
            || source.quality != DuDecisionEvidence::ExactStructured
            || source.stable_key != format!("du.source.weighted-curio-deflagration.{suffix}")
            || !source.locator.starts_with(prefix)
            || !source.locator.contains(locator)
            || source.sha256 != digest
        {
            return Err(DecisionDataError::InvalidProvenance);
        }
    }
    for (id, suffix, note, anchor) in [
        (
            147,
            "base-policy",
            &row.base_policy_note,
            "base-damage-policy",
        ),
        (
            148,
            "runtime-policy",
            &row.runtime_policy_note,
            "execution-policy",
        ),
    ] {
        let source = config
            .du_decision_sources()
            .get(&id)
            .ok_or(DecisionDataError::InvalidProvenance)?;
        let digest = Sha256::digest(note.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        if !context(source)
            || source.quality != DuDecisionEvidence::ProjectPolicy
            || source.stable_key != format!("du.source.weighted-curio-deflagration.{suffix}")
            || source.locator
                != format!("docs/divergent-universe-weighted-curio-deflagration.md#{anchor}")
            || source.sha256 != digest
            || !source.note.contains("not an upstream blob")
        {
            return Err(DecisionDataError::InvalidProvenance);
        }
    }
    let levels = config
        .du_weighted_curio_deflagration_levels()
        .ordered_rows()
        .collect::<Vec<_>>();
    if levels.len() != 95 {
        return Err(DecisionDataError::InvalidReference);
    }
    let hp_ratios_millionths = levels
        .iter()
        .enumerate()
        .map(|(index, level)| {
            let expected =
                i32::try_from(index + 1).map_err(|_| DecisionDataError::InvalidReference)?;
            if level.id != expected
                || level.unit_level != expected
                || level.stable_key != format!("du.weighted-curio-deflagration.level.{expected:03}")
                || level.weighted_curio_key != row.weighted_curio_key
                || level.hard_level_group != 1
                || level.source_ids != [146]
            {
                return Err(DecisionDataError::InvalidReference);
            }
            let ratio =
                parse_decimal(&level.hp_ratio).map_err(|_| DecisionDataError::InvalidReward)?;
            if ratio <= 0 {
                return Err(DecisionDataError::InvalidReward);
            }
            Ok(ratio)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    Ok(vec![WeightedCurioDeflagrationDefinition {
        key: key(&row.stable_key)?, weighted_curio: curio.id.clone(), maze_buff_id: row.maze_buff_id.clone().into(),
        burn_fractions_millionths: [500_000, 1_000_000, 1_500_000, 2_000_000],
        damage_multiplier_millionths: 2_000_000, duration_turns: 2,
        fixed_base_damage_millionths: 100_000_000, hard_level_group: 1, hp_ratios_millionths,
        status: match row.status { DuDeflagrationStatus::AuthoredOperandsPendingNative => WeightedCurioDeflagrationStatus::AuthoredOperandsPendingNative },
        policy: match row.policy { DuDeflagrationPolicy::OriginalFireAfterActionNaturalTickBurns => WeightedCurioDeflagrationPolicy::VersionedProjectPolicyOriginalFireAfterActionNaturalTickBurns },
        base_policy: match row.base_policy { DuDeflagrationBasePolicy::TargetGroupOneHpRatioProtocolHpFloor => WeightedCurioDeflagrationBasePolicy::VersionedProjectPolicyTargetGroupOneHpRatioProtocolHpFloor },
        summary_en: row.summary_en.clone().into(), summary_zh_cn: row.summary_zh_cn.clone().into(),
        runtime_policy_note: row.runtime_policy_note.clone().into(), runtime_replacement_condition: row.runtime_replacement_condition.clone().into(),
        base_policy_note: row.base_policy_note.clone().into(), base_replacement_condition: row.base_replacement_condition.clone().into(), sources,
    }].into_boxed_slice())
}

fn context(source: &DuDecisionSources) -> bool {
    source.url == "https://gitlab.com/Dimbreath/turnbasedgamedata"
        && source.revision == "fd978d6ef09f941fba644c731ab54abd6f7c3568"
        && source.game_version == "4.4"
        && source.access_date == "2026-10-05"
}
