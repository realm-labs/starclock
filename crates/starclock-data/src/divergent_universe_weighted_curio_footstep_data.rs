//! Released Footstep operands and independently authored native policy fields.
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_curio_catalog::DivergentUniverseWeightedCurioId,
    divergent_universe_decisions::{DecisionDataError, key, validation::source_keys},
    divergent_universe_decisions_generated::{
        SoraConfig, du_curio_footstep_policy::DuCurioFootstepPolicy,
        du_decision_evidence::DuDecisionEvidence, du_decision_sources::DuDecisionSources,
    },
};
use sha2::{Digest, Sha256};

/// Independent timing/arithmetic/reach policy, never observed parity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightedCurioFootstepPolicy {
    VersionedProjectPolicyEffectiveHpLossAfterSkillOriginalDamage,
}

/// Immutable authored values. Loading alone neither equips nor executes them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedCurioFootstepDefinition {
    pub key: Box<str>,
    pub weighted_curio: DivergentUniverseWeightedCurioId,
    pub maze_buff_id: Box<str>,
    pub loss_fraction_millionths: i64,
    pub damage_per_stack_millionths: i64,
    pub maximum_stacks: u16,
    pub policy: WeightedCurioFootstepPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub hp_policy_note: Box<str>,
    pub hp_replacement_condition: Box<str>,
    pub skill_policy_note: Box<str>,
    pub skill_replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[WeightedCurioFootstepDefinition]>, DecisionDataError> {
    let rows = config
        .du_weighted_curio_footsteps()
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
        || row.stable_key != "du.weighted-curio-footstep.footstep-of-gods"
        || curio.id.as_str() != "divergent-universe.weighted-curio.1011"
        || row.maze_buff_id != "633411"
        || curio.maze_buff_id.as_ref() != row.maze_buff_id
        || row.character_paths != ["Warrior", "Memory"]
        || eligible
            .character_paths
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<&str>>()
            != ["Memory", "Warrior"]
        || !eligible.elements.is_empty()
        || (row.loss_parameter, row.damage_parameter, row.cap_parameter) != (1, 2, 3)
    {
        return Err(DecisionDataError::InvalidReference);
    }
    let (loss_fraction_millionths, damage_per_stack_millionths, maximum_stacks) = match (
        row.loss_fraction.as_str(),
        row.damage_per_stack.as_str(),
        row.maximum_stacks,
    ) {
        ("0.5", "0.08", 10) => (500_000, 80_000, 10),
        _ => return Err(DecisionDataError::InvalidReward),
    };
    if [
        &row.summary_en,
        &row.summary_zh_cn,
        &row.hp_policy_note,
        &row.hp_replacement_condition,
        &row.skill_policy_note,
        &row.skill_replacement_condition,
    ]
    .iter()
    .any(|value| value.trim().is_empty())
        || !row.hp_policy_note.starts_with("VersionedProjectPolicy:")
        || !row.skill_policy_note.starts_with("VersionedProjectPolicy:")
    {
        return Err(DecisionDataError::InvalidPolicy);
    }
    let sources = source_keys(config, &row.source_ids)?;
    if row.source_ids != [134, 135, 136, 137, 138, 139, 140] {
        return Err(DecisionDataError::InvalidProvenance);
    }
    for (id, suffix, prefix, locator, digest) in [
        (
            134,
            "hex",
            "ExcelOutput/RogueTournHex.json;",
            "HexID=1011;",
            "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455",
        ),
        (
            135,
            "maze-buff",
            "ExcelOutput/MazeBuff.json;",
            "ID=633411;",
            "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac",
        ),
        (
            136,
            "text-en",
            "TextMap/TextMapEN.json;",
            "hash=10599000866283908992;",
            "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789",
        ),
        (
            137,
            "text-zh",
            "TextMap/TextMapCHS.json;",
            "hash=10599000866283908992;",
            "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147",
        ),
        (
            138,
            "program",
            "Config/ConfigAbility/Level/Rogue_S7/Level_RogueBuff_Ability_HEX_S7.json;",
            "Name=StageAbility_633411;",
            "5ad6c48751023184f5572e0069c5d02c841aef4de3cec092d0c2bc4913d17e47",
        ),
    ] {
        let source = config
            .du_decision_sources()
            .get(&id)
            .ok_or(DecisionDataError::InvalidProvenance)?;
        if source.stable_key != format!("du.source.weighted-curio-footstep.{suffix}")
            || source.quality != DuDecisionEvidence::ExactStructured
            || !context(source)
            || source.sha256 != digest
            || !source.locator.starts_with(prefix)
            || !source.locator.contains(locator)
        {
            return Err(DecisionDataError::InvalidProvenance);
        }
    }
    for (id, suffix, note, anchor) in [
        (
            139,
            "hp-policy",
            row.hp_policy_note.as_str(),
            "independently-replaceable-execution-policy",
        ),
        (
            140,
            "skill-policy",
            row.skill_policy_note.as_str(),
            "after-skill-damage-policy",
        ),
    ] {
        let source = config
            .du_decision_sources()
            .get(&id)
            .ok_or(DecisionDataError::InvalidProvenance)?;
        if source.stable_key != format!("du.source.weighted-curio-footstep.{suffix}")
            || source.quality != DuDecisionEvidence::ProjectPolicy
            || !context(source)
            || source.locator
                != format!("docs/divergent-universe-weighted-curio-footstep.md#{anchor}")
            || source.sha256
                != Sha256::digest(note.as_bytes())
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
        {
            return Err(DecisionDataError::InvalidProvenance);
        }
    }
    Ok(vec![WeightedCurioFootstepDefinition {
        key: key(&row.stable_key)?, weighted_curio: curio.id.clone(), maze_buff_id: row.maze_buff_id.clone().into(),
        loss_fraction_millionths,damage_per_stack_millionths,maximum_stacks,
        policy: match row.policy {DuCurioFootstepPolicy::EffectiveHpLossAfterSkillOriginalDamage => WeightedCurioFootstepPolicy::VersionedProjectPolicyEffectiveHpLossAfterSkillOriginalDamage},
        summary_en: row.summary_en.clone().into(),summary_zh_cn: row.summary_zh_cn.clone().into(),
        hp_policy_note: row.hp_policy_note.clone().into(),hp_replacement_condition: row.hp_replacement_condition.clone().into(),
        skill_policy_note: row.skill_policy_note.clone().into(),skill_replacement_condition: row.skill_replacement_condition.clone().into(),sources,
    }].into_boxed_slice())
}

fn context(source: &DuDecisionSources) -> bool {
    source.url == "https://gitlab.com/Dimbreath/turnbasedgamedata"
        && source.revision == "fd978d6ef09f941fba644c731ab54abd6f7c3568"
        && source.game_version == "4.4"
        && source.access_date == "2026-10-05"
}
