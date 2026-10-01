//! Released Physical retaliation operands and explicit target-weight inputs.
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_curio_catalog::DivergentUniverseWeightedCurioId,
    divergent_universe_decisions::{DecisionDataError, key, validation::source_keys},
    divergent_universe_decisions_generated::{
        SoraConfig, du_curio_retaliation_policy::DuCurioRetaliationPolicy,
        du_decision_evidence::DuDecisionEvidence,
    },
};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Replaceable targeting and reaction choices; never an observed-parity claim.
pub enum WeightedCurioRetaliationPolicy {
    VersionedProjectPolicyPhysicalAggroAndOwnerAdditional,
}

/// Explicit content-owned baselines, not an inference in the shared resolver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WeightedCurioPathAggro {
    pub destruction: i64,
    pub hunt: i64,
    pub erudition: i64,
    pub harmony: i64,
    pub nihility: i64,
    pub preservation: i64,
    pub abundance: i64,
    pub remembrance: i64,
    pub elation: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Immutable released operands with separately labeled hidden-behavior policy.
///
/// Compilation rejects wrong current Hex/MazeBuff joins, noncanonical values,
/// missing definitions and provenance not bound to the released revision.
pub struct WeightedCurioRetaliationDefinition {
    pub key: Box<str>,
    pub weighted_curio: DivergentUniverseWeightedCurioId,
    pub maze_buff_id: Box<str>,
    pub additional_millionths: i64,
    pub aggro_millionths: i64,
    pub base_aggro: WeightedCurioPathAggro,
    pub policy: WeightedCurioRetaliationPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[WeightedCurioRetaliationDefinition]>, DecisionDataError> {
    let mut definitions = Vec::new();
    let mut identities = BTreeSet::new();
    for row in config.du_weighted_curio_retaliations().ordered_rows() {
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
            || curio.id.as_str() != "divergent-universe.weighted-curio.1013"
            || row.maze_buff_id != "633413"
            || curio.maze_buff_id.as_ref() != row.maze_buff_id
            || row
                .character_elements
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                != ["Physical"]
            || eligible
                .elements
                .iter()
                .map(AsRef::as_ref)
                .collect::<BTreeSet<&str>>()
                != BTreeSet::from(["Physical"])
            || !eligible.character_paths.is_empty()
            || (row.additional_parameter, row.aggro_parameter) != (1, 2)
            || !identities.insert(curio.id.clone())
        {
            return Err(DecisionDataError::InvalidReference);
        }
        let (additional_millionths, aggro_millionths) = match (
            row.additional_multiplier.as_str(),
            row.aggro_fraction.as_str(),
        ) {
            ("4", "0.3") => (4_000_000, 300_000),
            _ => return Err(DecisionDataError::InvalidReward),
        };
        let authored = [
            &row.base_aggro_destruction,
            &row.base_aggro_hunt,
            &row.base_aggro_erudition,
            &row.base_aggro_harmony,
            &row.base_aggro_nihility,
            &row.base_aggro_preservation,
            &row.base_aggro_abundance,
            &row.base_aggro_remembrance,
            &row.base_aggro_elation,
        ];
        if authored.map(String::as_str)
            != ["125", "75", "75", "100", "100", "150", "100", "100", "100"]
        {
            return Err(DecisionDataError::InvalidReward);
        }
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
        for (prefix, locator, digest) in [
            (
                "ExcelOutput/RogueTournHex.json;",
                "HexID=1013;",
                "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455",
            ),
            (
                "ExcelOutput/MazeBuff.json;",
                "ID=633413;",
                "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac",
            ),
            (
                "TextMap/TextMapEN.json;",
                "hash=6457248194266440334;",
                "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789",
            ),
            (
                "TextMap/TextMapCHS.json;",
                "hash=6457248194266440334;",
                "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147",
            ),
            (
                "ExcelOutput/AvatarConfig.json;",
                "AvatarID=1001:Knight|1002:Rogue|1003:Mage|1004:Warlock|1008:Warrior|1009:Shaman|1105:Priest|1402:Memory|1501:Elation;",
                "c14584519e2feda70e9932501f7b79d67242d700585fe44119bbce9da9678e98",
            ),
            (
                "ExcelOutput/AvatarPromotionConfig.json;",
                "AvatarID=1001:150|1002:75|1003:75|1004:100|1008:125|1009:100|1105:100|1402:100|1501:100;",
                "4453f206d6b79658128f22ce4d923e2e608f92b48175ba5c913ed2be322d24c5",
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
        definitions.push(WeightedCurioRetaliationDefinition {
            key: key(&row.stable_key)?,
            weighted_curio: curio.id.clone(),
            maze_buff_id: row.maze_buff_id.clone().into(),
            additional_millionths,
            aggro_millionths,
            base_aggro: WeightedCurioPathAggro {
                destruction: 125,
                hunt: 75,
                erudition: 75,
                harmony: 100,
                nihility: 100,
                preservation: 150,
                abundance: 100,
                remembrance: 100,
                elation: 100,
            },
            policy: match row.policy {
                DuCurioRetaliationPolicy::PhysicalAggroAndOwnerAdditional =>
                    WeightedCurioRetaliationPolicy::VersionedProjectPolicyPhysicalAggroAndOwnerAdditional,
            },
            summary_en: row.summary_en.clone().into(),
            summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(),
            replacement_condition: row.replacement_condition.clone().into(),
            sources,
        });
    }
    if definitions.len() != 1 {
        return Err(DecisionDataError::InvalidReference);
    }
    Ok(definitions.into_boxed_slice())
}
