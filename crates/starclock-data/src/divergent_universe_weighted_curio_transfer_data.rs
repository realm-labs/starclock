//! Four released special-shield operands, distinct from hidden execution policy.

use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_curio_catalog::DivergentUniverseWeightedCurioId,
    divergent_universe_decisions::{DecisionDataError, key, validation::source_keys},
    divergent_universe_decisions_generated::{
        SoraConfig, du_curio_transfer_policy::DuCurioTransferPolicy,
        du_decision_evidence::DuDecisionEvidence,
    },
};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightedCurioTransferPolicy {
    VersionedProjectPolicyOtherShieldAppliedOwnerTurnExcessDecay,
}

/// Exact current operands; loading does not admit a Forge offer or prove execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedCurioTransferDefinition {
    pub key: Box<str>,
    pub weighted_curio: DivergentUniverseWeightedCurioId,
    pub maze_buff_id: Box<str>,
    pub transfer_millionths: i64,
    pub threshold_millionths: i64,
    pub decay_millionths: i64,
    pub heal_millionths: i64,
    pub policy: WeightedCurioTransferPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[WeightedCurioTransferDefinition]>, DecisionDataError> {
    let mut definitions = Vec::new();
    let mut identities = BTreeSet::new();
    for row in config.du_weighted_curio_transfers().ordered_rows() {
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
        if row.id <= 0
            || curio.id.as_str() != "divergent-universe.weighted-curio.1009"
            || row.maze_buff_id != "633409"
            || curio.maze_buff_id.as_ref() != row.maze_buff_id
            || row.character_path != "Knight"
            || eligible.character_paths.as_ref() != [Box::<str>::from("Knight")]
            || !eligible.elements.is_empty()
            || (
                row.transfer_parameter,
                row.threshold_parameter,
                row.decay_parameter,
                row.heal_parameter,
            ) != (1, 2, 3, 4)
            || !identities.insert(curio.id.clone())
        {
            return Err(DecisionDataError::InvalidReference);
        }
        let (transfer_millionths, threshold_millionths, decay_millionths, heal_millionths) = match (
            row.transfer_fraction.as_str(),
            row.threshold_fraction.as_str(),
            row.decay_fraction.as_str(),
            row.heal_fraction.as_str(),
        ) {
            ("0.75", "0.3", "0.9", "0.1") => (750_000, 300_000, 900_000, 100_000),
            _ => return Err(DecisionDataError::InvalidReward),
        };
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
        if sources.len() != 4 {
            return Err(DecisionDataError::InvalidProvenance);
        }
        for (prefix, locator, digest) in [
            (
                "ExcelOutput/RogueTournHex.json;",
                "HexID=1009;",
                "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455",
            ),
            (
                "ExcelOutput/MazeBuff.json;",
                "ID=633409;",
                "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac",
            ),
            (
                "TextMap/TextMapEN.json;",
                "hash=14114601434821166552;",
                "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789",
            ),
            (
                "TextMap/TextMapCHS.json;",
                "hash=14114601434821166552;",
                "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147",
            ),
        ] {
            if !row.source_ids.iter().any(|id| {
                config.du_decision_sources().get(id).is_some_and(|source| {
                    source.quality == DuDecisionEvidence::ExactStructured
                        && source.url == "https://gitlab.com/Dimbreath/turnbasedgamedata"
                        && source.revision == "fd978d6ef09f941fba644c731ab54abd6f7c3568"
                        && source.game_version == "4.4"
                        && source.sha256 == digest
                        && source.locator.starts_with(prefix)
                        && source.locator.contains(locator)
                })
            }) {
                return Err(DecisionDataError::InvalidProvenance);
            }
        }
        definitions.push(WeightedCurioTransferDefinition {
            key: key(&row.stable_key)?, weighted_curio: curio.id.clone(), maze_buff_id: row.maze_buff_id.clone().into(),
            transfer_millionths, threshold_millionths, decay_millionths, heal_millionths,
            policy: match row.policy { DuCurioTransferPolicy::OtherShieldAppliedOwnerTurnExcessDecay =>
                WeightedCurioTransferPolicy::VersionedProjectPolicyOtherShieldAppliedOwnerTurnExcessDecay },
            summary_en: row.summary_en.clone().into(), summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(), replacement_condition: row.replacement_condition.clone().into(), sources,
        });
    }
    definitions.sort_by(|a, b| a.weighted_curio.cmp(&b.weighted_curio));
    Ok(definitions.into_boxed_slice())
}
