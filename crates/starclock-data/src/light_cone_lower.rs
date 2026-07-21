//! Light Cone Excel rows to generated-type-free build definitions.

use std::collections::{BTreeMap, BTreeSet};

use starclock_build::{
    id::LightConeId,
    light_cone::{
        CombatPath, LightConeApplicability, LightConeDefinition, LightConeLevel,
        LightConePassiveRank, LightConeStatRow, Superimposition,
    },
    patch::BuildPatch,
    spec::PromotionStage,
};
use starclock_combat::{Hp, ModifierDefinitionId, Rounding, RuleBundleId, Scalar, StatValue};

use crate::{
    catalog::{
        CatalogLoadError, CombatDefinitions, IdentityDefinition, IdentityKind, LoadMode,
        domain_fail, parse_decimal, positive, require_identity,
    },
    generated,
};

#[derive(Debug)]
pub(super) struct LightConeDataDefinition {
    id: LightConeId,
    rarity: u8,
    path: CombatPath,
    applicability: LightConeApplicability,
    passive_rule: RuleBundleId,
    stats: Box<[LightConeStatDefinition]>,
    ranks: Box<[LightConeRankDefinition]>,
}

#[derive(Debug)]
struct LightConeStatDefinition {
    level: u8,
    promotion: u8,
    hp: Scalar,
    attack: Scalar,
    defense: Scalar,
}

#[derive(Debug)]
struct LightConeRankDefinition {
    rank: u8,
    modifiers: Box<[ModifierDefinitionId]>,
}

impl LightConeDataDefinition {
    pub(super) fn violates_invariants(&self) -> bool {
        !(3..=5).contains(&self.rarity)
            || self.stats.len() != 86
            || self.stats.iter().any(|row| {
                row.level == 0
                    || row.level > 80
                    || row.promotion > 6
                    || row.hp.scaled() <= 0
                    || row.attack.scaled() <= 0
                    || row.defense.scaled() <= 0
            })
            || self.ranks.len() != 5
            || self
                .ranks
                .iter()
                .enumerate()
                .any(|(index, rank)| usize::from(rank.rank) != index + 1)
    }

    pub(super) fn compile(
        &self,
        digest: [u8; 32],
    ) -> Result<LightConeDefinition, CatalogLoadError> {
        let stats = self
            .stats
            .iter()
            .map(|row| {
                Ok(LightConeStatRow::new(
                    LightConeLevel::new(row.level)
                        .ok_or_else(|| domain_fail("Light Cone level exceeds build domain"))?,
                    PromotionStage::new(row.promotion)
                        .ok_or_else(|| domain_fail("Light Cone promotion exceeds build domain"))?,
                    Hp::from_scalar(row.hp, Rounding::NearestTiesEven).map_err(domain_fail)?,
                    StatValue::from_scaled(row.attack.scaled()).map_err(domain_fail)?,
                    StatValue::from_scaled(row.defense.scaled()).map_err(domain_fail)?,
                ))
            })
            .collect::<Result<Vec<_>, CatalogLoadError>>()?;
        let ranks = self
            .ranks
            .iter()
            .map(|rank| {
                let mut patches = Vec::with_capacity(rank.modifiers.len() + 1);
                patches.push(BuildPatch::AddRuleBundle(self.passive_rule));
                patches.extend(rank.modifiers.iter().copied().map(BuildPatch::AddModifier));
                Ok(LightConePassiveRank::new(
                    Superimposition::new(rank.rank)
                        .ok_or_else(|| domain_fail("Light Cone rank exceeds build domain"))?,
                    patches,
                ))
            })
            .collect::<Result<Vec<_>, CatalogLoadError>>()?;
        Ok(LightConeDefinition::new(
            self.id,
            crate::domain_catalog::source(
                self.id.get(),
                starclock_combat::rule::model::SourceClass::Equipment,
                digest,
            )?,
            self.path,
            self.applicability,
            stats,
            ranks,
        ))
    }
}

pub(super) fn convert(
    config: &generated::SoraConfig,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
    combat: &CombatDefinitions,
) -> Result<Vec<LightConeDataDefinition>, CatalogLoadError> {
    let mut definitions = config
        .light_cone()
        .ordered_rows()
        .map(|row| lower_one(config, row, mode, identities, combat))
        .collect::<Result<Vec<_>, _>>()?;
    definitions.sort_unstable_by_key(|definition| definition.id);
    Ok(definitions)
}

fn lower_one(
    config: &generated::SoraConfig,
    row: &generated::light_cone::LightCone,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
    combat: &CombatDefinitions,
) -> Result<LightConeDataDefinition, CatalogLoadError> {
    let raw_id = positive(row.id, "LightCone.id")?;
    require_identity(identities, raw_id, IdentityKind::LightCone, mode)?;
    let rarity = u8::try_from(row.rarity)
        .ok()
        .filter(|value| (3..=5).contains(value))
        .ok_or_else(|| domain_fail("invalid Light Cone rarity"))?;
    let passive_raw = positive(
        row.passive_rule_identity_id,
        "LightCone.passive_rule_identity_id",
    )?;
    require_identity(identities, passive_raw, IdentityKind::Other, mode)?;
    if combat
        .rules
        .binary_search_by_key(&passive_raw, |rule| rule.id.get())
        .is_err()
    {
        return Err(domain_fail("Light Cone passive refers to a missing rule"));
    }
    let stats = lower_stats(config, row.id)?;
    let ranks = lower_ranks(config, row.id, combat)?;
    Ok(LightConeDataDefinition {
        id: LightConeId::new(raw_id).expect("positive Light Cone ID"),
        rarity,
        path: path(row.path),
        applicability: applicability(row.applicability),
        passive_rule: RuleBundleId::new(passive_raw).expect("positive passive rule ID"),
        stats: stats.into_boxed_slice(),
        ranks: ranks.into_boxed_slice(),
    })
}

fn lower_stats(
    config: &generated::SoraConfig,
    light_cone_id: i32,
) -> Result<Vec<LightConeStatDefinition>, CatalogLoadError> {
    let mut stats = config
        .light_cone_stat()
        .iter()
        .filter(|stat| stat.light_cone_id == light_cone_id)
        .map(|stat| {
            Ok(LightConeStatDefinition {
                level: u8::try_from(stat.level)
                    .map_err(|_| domain_fail("Light Cone level does not fit u8"))?,
                promotion: u8::try_from(stat.promotion)
                    .map_err(|_| domain_fail("Light Cone promotion does not fit u8"))?,
                hp: Scalar::from_scaled(parse_decimal(&stat.hp_decimal)?),
                attack: Scalar::from_scaled(parse_decimal(&stat.atk_decimal)?),
                defense: Scalar::from_scaled(parse_decimal(&stat.def_decimal)?),
            })
        })
        .collect::<Result<Vec<_>, CatalogLoadError>>()?;
    stats.sort_unstable_by_key(|stat| (stat.level, stat.promotion));
    let expected = expected_stat_keys();
    if stats.len() != expected.len()
        || stats
            .iter()
            .zip(expected)
            .any(|(row, key)| (row.level, row.promotion) != key)
        || stats.iter().any(|row| {
            row.hp.scaled() <= 0 || row.attack.scaled() <= 0 || row.defense.scaled() <= 0
        })
    {
        return Err(domain_fail(format!(
            "Light Cone {light_cone_id} lacks the complete level/promotion stat curve"
        )));
    }
    Ok(stats)
}

fn expected_stat_keys() -> Vec<(u8, u8)> {
    let mut output = Vec::with_capacity(86);
    for promotion in 0_u8..=6 {
        let first = if promotion == 0 {
            1
        } else {
            promotion * 10 + 10
        };
        let last = promotion * 10 + 20;
        output.extend((first..=last).map(|level| (level, promotion)));
    }
    output.sort_unstable();
    output
}

fn lower_ranks(
    config: &generated::SoraConfig,
    light_cone_id: i32,
    combat: &CombatDefinitions,
) -> Result<Vec<LightConeRankDefinition>, CatalogLoadError> {
    let mut rows = config
        .light_cone_superimposition()
        .iter()
        .filter(|value| value.light_cone_id == light_cone_id)
        .collect::<Vec<_>>();
    rows.sort_unstable_by(|left, right| {
        (left.rank, left.parameter_key.as_str()).cmp(&(right.rank, right.parameter_key.as_str()))
    });
    if rows.is_empty() {
        return Err(domain_fail(format!(
            "Light Cone {light_cone_id} has no superimposition parameters"
        )));
    }
    let mut by_rank = BTreeMap::<u8, BTreeMap<&str, (&str, bool)>>::new();
    let mut modifiers = BTreeMap::<u8, BTreeSet<ModifierDefinitionId>>::new();
    for row in rows {
        let rank = u8::try_from(row.rank)
            .ok()
            .filter(|value| (1..=5).contains(value))
            .ok_or_else(|| domain_fail("invalid Light Cone superimposition rank"))?;
        if row.parameter_key.trim().is_empty() {
            return Err(domain_fail("empty Light Cone parameter key"));
        }
        parse_decimal(&row.value_decimal)?;
        if by_rank
            .entry(rank)
            .or_default()
            .insert(
                row.parameter_key.as_str(),
                (row.value_decimal.as_str(), row.constant_across_ranks),
            )
            .is_some()
        {
            return Err(domain_fail("duplicate Light Cone parameter rank"));
        }
        for raw in row.modifier_identity_ids.as_deref().unwrap_or(&[]) {
            let id = ModifierDefinitionId::new(positive(
                *raw,
                "LightConeSuperimposition.modifier_identity_ids",
            )?)
            .expect("positive modifier ID");
            if combat.modifiers.definition(id).is_none()
                || !modifiers.entry(rank).or_default().insert(id)
            {
                return Err(domain_fail("invalid Light Cone rank modifier binding"));
            }
        }
    }
    if by_rank.len() != 5 || !(1..=5).all(|rank| by_rank.contains_key(&rank)) {
        return Err(domain_fail("Light Cone lacks exact S1-S5 parameter rows"));
    }
    let keys = by_rank[&1].keys().copied().collect::<Vec<_>>();
    if keys.is_empty()
        || (2..=5).any(|rank| by_rank[&rank].keys().copied().collect::<Vec<_>>() != keys)
    {
        return Err(domain_fail("Light Cone S1-S5 parameter vectors differ"));
    }
    for key in keys {
        let values = (1..=5).map(|rank| by_rank[&rank][key]).collect::<Vec<_>>();
        let constant = values.iter().all(|value| value.0 == values[0].0);
        if values.iter().any(|value| value.1 != constant) {
            return Err(domain_fail("Light Cone constant-rank policy is incorrect"));
        }
    }
    Ok((1..=5)
        .map(|rank| LightConeRankDefinition {
            rank,
            modifiers: modifiers
                .remove(&rank)
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        })
        .collect())
}

fn path(value: generated::combat_path::CombatPath) -> CombatPath {
    use generated::combat_path::CombatPath as V;
    match value {
        V::Destruction => CombatPath::Destruction,
        V::Hunt => CombatPath::Hunt,
        V::Erudition => CombatPath::Erudition,
        V::Harmony => CombatPath::Harmony,
        V::Nihility => CombatPath::Nihility,
        V::Preservation => CombatPath::Preservation,
        V::Abundance => CombatPath::Abundance,
        V::Remembrance => CombatPath::Remembrance,
        V::Elation => CombatPath::Elation,
    }
}

fn applicability(
    value: generated::light_cone_applicability::LightConeApplicability,
) -> LightConeApplicability {
    use generated::light_cone_applicability::LightConeApplicability as V;
    match value {
        V::MatchingPath => LightConeApplicability::MatchingPath,
        V::Always => LightConeApplicability::Always,
        V::BaseStatsOnly => LightConeApplicability::BaseStatsOnly,
    }
}
