//! Released Walkie-Talkie operands with separately authored native policies.
use crate::{
    catalog::parse_decimal,
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_curio_catalog::DivergentUniverseWeightedCurioId,
    divergent_universe_decisions::{DecisionDataError, key, validation::source_keys},
    divergent_universe_decisions_generated::{
        SoraConfig, du_curio_overflow_base_policy::DuCurioOverflowBasePolicy,
        du_curio_overflow_status::DuCurioOverflowStatus, du_decision_evidence::DuDecisionEvidence,
        du_weighted_curio_overflows::DuWeightedCurioOverflows,
    },
};
use sha2::{Digest, Sha256};

/// Group selection, Protocol mapping and rounding are not observed parity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightedCurioOverflowBasePolicy {
    VersionedProjectPolicyEnemyGroupOneHpRatioProtocolHpMultiplierFloor,
}

/// Exact released curve values with separately authored formula decisions.
/// Ratios are positive canonical millionths ordered by target level 1..=95.
/// Loading does not execute this formula or admit the Curio into a battle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedCurioOverflowBaseDefinition {
    pub fixed_damage_millionths: i64,
    pub hard_level_group: u16,
    pub hp_ratios_millionths: Box<[i64]>,
    pub policy: WeightedCurioOverflowBasePolicy,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
}

/// Native equipment policy, not an observed-parity or complete-run disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightedCurioOverflowStatus {
    NativeProjectPolicyHitEnded,
}

/// Exact ratios plus replaceable timing/ownership policy. Loading alone does
/// not execute the effect; the normal immutable battle assembly owns binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedCurioOverflowDefinition {
    pub key: Box<str>,
    pub weighted_curio: DivergentUniverseWeightedCurioId,
    pub maze_buff_id: Box<str>,
    pub base_multiplier_millionths: i64,
    pub overflow_multiplier_millionths: i64,
    pub attack_increase_millionths: i64,
    pub status: WeightedCurioOverflowStatus,
    pub base: WeightedCurioOverflowBaseDefinition,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub source_semantics: Box<str>,
    pub runtime_policy_note: Box<str>,
    pub runtime_replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[WeightedCurioOverflowDefinition]>, DecisionDataError> {
    let rows = config
        .du_weighted_curio_overflows()
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
        .find(|eligible| eligible.weighted_curio.as_ref() == Some(&curio.id))
        .ok_or(DecisionDataError::InvalidReference)?;
    let paths = [Box::<str>::from("Mage"), Box::<str>::from("Rogue")];
    if row.id != 1
        || row.stable_key != "du.weighted-curio-overflow.parallel-universe-walkie-talkie"
        || curio.id.as_str() != "divergent-universe.weighted-curio.1016"
        || row.maze_buff_id != "633416"
        || curio.maze_buff_id.as_ref() != row.maze_buff_id
        || row.character_paths != ["Mage", "Rogue"]
        || eligible.character_paths.as_ref() != paths
        || !eligible.elements.is_empty()
        || (
            row.base_parameter,
            row.overflow_parameter,
            row.attack_parameter,
        ) != (1, 2, 3)
    {
        return Err(DecisionDataError::InvalidReference);
    }
    let (base_multiplier_millionths, overflow_multiplier_millionths, attack_increase_millionths) =
        match (
            row.base_multiplier.as_str(),
            row.overflow_multiplier.as_str(),
            row.attack_increase.as_str(),
        ) {
            ("10", "1", "0.8") => (10_000_000, 1_000_000, 800_000),
            _ => return Err(DecisionDataError::InvalidReward),
        };
    if [
        &row.summary_en,
        &row.summary_zh_cn,
        &row.source_semantics,
        &row.runtime_policy_note,
        &row.runtime_replacement_condition,
    ]
    .iter()
    .any(|value| value.trim().is_empty())
        || !row
            .runtime_policy_note
            .starts_with("VersionedProjectPolicy:")
    {
        return Err(DecisionDataError::InvalidPolicy);
    }
    let sources = source_keys(config, &row.source_ids)?;
    if sources.len() != 8 {
        return Err(DecisionDataError::InvalidProvenance);
    }
    for (suffix, prefix, locator, digest) in [
        (
            "hex",
            "ExcelOutput/RogueTournHex.json;",
            "HexID=1016;",
            "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455",
        ),
        (
            "maze-buff",
            "ExcelOutput/MazeBuff.json;",
            "ID=633416;",
            "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac",
        ),
        (
            "text-en",
            "TextMap/TextMapEN.json;",
            "hash=12104045893670599658;",
            "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789",
        ),
        (
            "text-zh",
            "TextMap/TextMapCHS.json;",
            "hash=12104045893670599658;",
            "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147",
        ),
        (
            "program",
            "Config/ConfigAbility/Level/Rogue_S7/Level_RogueBuff_Ability_HEX_S7.json;",
            "Name=StageAbility_633416;",
            "5ad6c48751023184f5572e0069c5d02c841aef4de3cec092d0c2bc4913d17e47",
        ),
    ] {
        let expected_key = format!("du.source.weighted-curio-overflow.{suffix}");
        if !row.source_ids.iter().any(|id| {
            config.du_decision_sources().get(id).is_some_and(|source| {
                source.stable_key == expected_key
                    && source.quality == DuDecisionEvidence::ExactStructured
                    && source.url == "https://gitlab.com/Dimbreath/turnbasedgamedata"
                    && source.revision == "fd978d6ef09f941fba644c731ab54abd6f7c3568"
                    && source.game_version == "4.4"
                    && source.access_date == "2026-10-03"
                    && source.sha256 == digest
                    && source.locator.starts_with(prefix)
                    && source.locator.contains(locator)
            })
        }) {
            return Err(DecisionDataError::InvalidProvenance);
        }
    }
    validate_runtime_policy_source(config, row)?;
    Ok(vec![WeightedCurioOverflowDefinition {
        key: key(&row.stable_key)?,
        weighted_curio: curio.id.clone(),
        maze_buff_id: row.maze_buff_id.clone().into(),
        base_multiplier_millionths,
        overflow_multiplier_millionths,
        attack_increase_millionths,
        base: compile_base(config, row)?,
        status: match row.status {
            DuCurioOverflowStatus::NativeProjectPolicyHitEnded => {
                WeightedCurioOverflowStatus::NativeProjectPolicyHitEnded
            }
        },
        summary_en: row.summary_en.clone().into(),
        summary_zh_cn: row.summary_zh_cn.clone().into(),
        source_semantics: row.source_semantics.clone().into(),
        runtime_policy_note: row.runtime_policy_note.clone().into(),
        runtime_replacement_condition: row.runtime_replacement_condition.clone().into(),
        sources,
    }]
    .into_boxed_slice())
}

fn validate_runtime_policy_source(
    config: &SoraConfig,
    row: &DuWeightedCurioOverflows,
) -> Result<(), DecisionDataError> {
    let source = config
        .du_decision_sources()
        .get(&133)
        .ok_or(DecisionDataError::InvalidProvenance)?;
    let digest = Sha256::digest(row.runtime_policy_note.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if !row.source_ids.contains(&133)
        || source.stable_key != "du.source.weighted-curio-overflow.runtime-policy"
        || source.quality != DuDecisionEvidence::ProjectPolicy
        || source.url != "https://gitlab.com/Dimbreath/turnbasedgamedata"
        || source.revision != "fd978d6ef09f941fba644c731ab54abd6f7c3568"
        || source.game_version != "4.4"
        || source.access_date != "2026-10-05"
        || source.locator
            != "docs/divergent-universe-weighted-curio-overflow.md#native-equipment-policy"
        || source.sha256 != digest
        || source.note != row.runtime_policy_note
    {
        return Err(DecisionDataError::InvalidProvenance);
    }
    Ok(())
}

fn compile_base(
    config: &SoraConfig,
    row: &DuWeightedCurioOverflows,
) -> Result<WeightedCurioOverflowBaseDefinition, DecisionDataError> {
    if row.base_fixed_damage != "100"
        || row.base_hard_level_group != 1
        || !row.base_policy_note.starts_with("VersionedProjectPolicy:")
        || row.base_replacement_condition.trim().is_empty()
    {
        return Err(DecisionDataError::InvalidPolicy);
    }
    for (id, suffix, quality, locator, digest) in [
        (
            131,
            "hard-level",
            DuDecisionEvidence::ExactStructured,
            "ExcelOutput/HardLevelGroup.json; HardLevelGroup=1; Level=1..95; HPRatio.Value",
            "d185c09b5388f4eeb368199276ee8a815b406fd9902744027b72a5011b962978".to_owned(),
        ),
        (
            132,
            "base-policy",
            DuDecisionEvidence::ProjectPolicy,
            "docs/divergent-universe-weighted-curio-overflow.md#base-damage-policy",
            Sha256::digest(row.base_policy_note.as_bytes())
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
        ),
    ] {
        let source = config
            .du_decision_sources()
            .get(&id)
            .ok_or(DecisionDataError::InvalidProvenance)?;
        if !row.source_ids.contains(&id)
            || source.stable_key != format!("du.source.weighted-curio-overflow.{suffix}")
            || source.quality != quality
            || source.locator != locator
            || source.sha256 != digest
            || source.url != "https://gitlab.com/Dimbreath/turnbasedgamedata"
            || source.revision != "fd978d6ef09f941fba644c731ab54abd6f7c3568"
            || source.game_version != "4.4"
            || source.access_date != "2026-10-05"
            || (id == 132 && source.note != row.base_policy_note)
        {
            return Err(DecisionDataError::InvalidProvenance);
        }
    }
    let levels = config
        .du_weighted_curio_overflow_levels()
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
                || level.stable_key != format!("du.weighted-curio-overflow.level.{expected:03}")
                || level.weighted_curio_key != row.weighted_curio_key
                || level.hard_level_group != 1
                || level.source_ids != [131]
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
    Ok(WeightedCurioOverflowBaseDefinition {
        fixed_damage_millionths: 100_000_000,
        hard_level_group: 1,
        hp_ratios_millionths,
        policy: match row.base_policy {
            DuCurioOverflowBasePolicy::EnemyGroupOneHpRatioProtocolHpMultiplierFloor =>
                WeightedCurioOverflowBasePolicy::VersionedProjectPolicyEnemyGroupOneHpRatioProtocolHpMultiplierFloor,
        },
        policy_note: row.base_policy_note.clone().into(),
        replacement_condition: row.base_replacement_condition.clone().into(),
    })
}
