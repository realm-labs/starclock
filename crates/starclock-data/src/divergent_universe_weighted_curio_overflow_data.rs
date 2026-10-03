//! Released Walkie-Talkie operands; native death and base-damage admission is pending.
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_curio_catalog::DivergentUniverseWeightedCurioId,
    divergent_universe_decisions::{DecisionDataError, key, validation::source_keys},
    divergent_universe_decisions_generated::{
        SoraConfig, du_curio_overflow_status::DuCurioOverflowStatus,
        du_decision_evidence::DuDecisionEvidence,
    },
};

/// Incomplete native admission, not a terminal execution or parity disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightedCurioOverflowStatus {
    PendingNativeDeathCallbackAndBaseDamage,
}

/// Exact ratios and source facts do not establish an executable battle effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedCurioOverflowDefinition {
    pub key: Box<str>,
    pub weighted_curio: DivergentUniverseWeightedCurioId,
    pub maze_buff_id: Box<str>,
    pub base_multiplier_millionths: i64,
    pub overflow_multiplier_millionths: i64,
    pub attack_increase_millionths: i64,
    pub status: WeightedCurioOverflowStatus,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub source_semantics: Box<str>,
    pub unresolved_runtime: Box<str>,
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
        &row.unresolved_runtime,
    ]
    .iter()
    .any(|value| value.trim().is_empty())
        || !row.unresolved_runtime.starts_with("Unimplemented:")
    {
        return Err(DecisionDataError::InvalidPolicy);
    }
    let sources = source_keys(config, &row.source_ids)?;
    if sources.len() != 5 {
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
    Ok(vec![WeightedCurioOverflowDefinition {
        key: key(&row.stable_key)?,
        weighted_curio: curio.id.clone(),
        maze_buff_id: row.maze_buff_id.clone().into(),
        base_multiplier_millionths,
        overflow_multiplier_millionths,
        attack_increase_millionths,
        status: match row.status {
            DuCurioOverflowStatus::PendingNativeDeathCallbackAndBaseDamage => {
                WeightedCurioOverflowStatus::PendingNativeDeathCallbackAndBaseDamage
            }
        },
        summary_en: row.summary_en.clone().into(),
        summary_zh_cn: row.summary_zh_cn.clone().into(),
        source_semantics: row.source_semantics.clone().into(),
        unresolved_runtime: row.unresolved_runtime.clone().into(),
        sources,
    }]
    .into_boxed_slice())
}
