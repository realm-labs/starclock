//! Immutable policy stock compiled from production Sora, never inferred merchants.

use std::collections::BTreeSet;

use super::{
    DecisionDataError, key,
    validation::{ordinals, source_keys},
};
use crate::divergent_universe::DivergentUniverseBundleCandidate;
use crate::divergent_universe_blessing_catalog::DivergentUniverseBlessingId;
use crate::divergent_universe_curio_catalog::{
    DivergentUniverseCurioCategory, DivergentUniverseCurioStateId,
};
use crate::divergent_universe_decisions_generated::{
    SoraConfig, du_decision_evidence::DuDecisionEvidence, du_shop_reward_kind::DuShopRewardKind,
    du_shop_stock_policy::DuShopStockPolicy,
};

/// Project stock identity; source merchant IDs are not stock identity.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ShopStockId(Box<str>);

impl ShopStockId {
    /// Reject invalid keys and empty/wrong namespaces; existence is a catalog lookup.
    pub fn new(value: impl Into<Box<str>>) -> Result<Self, DecisionDataError> {
        let value = value.into();
        if !value.starts_with("du.shop-stock.") || value.len() == "du.shop-stock.".len() {
            return Err(DecisionDataError::InvalidIdentity);
        }
        Ok(Self(key(&value)?))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShopStockPolicy {
    VersionedProjectPolicyExplicitFixedStockAndPriceBaseRewards,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShopStockReward {
    Blessing(DivergentUniverseBlessingId),
    Curio(DivergentUniverseCurioStateId),
}

/// Contiguous 1..=64 addresses, positive i64-range price, current owner-unique reward.
/// Sources establish reward identity/effects, not stock membership or price parity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShopStockItemDefinition {
    pub key: Box<str>,
    pub ordinal: u16,
    pub reward: ShopStockReward,
    pub price: u64,
    pub sources: Box<[Box<str>]>,
}

/// Immutable policy with field-level replacement notes and ordered item definitions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShopStockDefinition {
    pub id: ShopStockId,
    pub policy: ShopStockPolicy,
    pub items: Box<[ShopStockItemDefinition]>,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
}

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[ShopStockDefinition]>, DecisionDataError> {
    for item in config.du_shop_items().ordered_rows() {
        if config.du_shop_stocks().get(&item.stock_id).is_none() {
            return Err(DecisionDataError::InvalidReference);
        }
    }
    let mut result = Vec::new();
    for row in config.du_shop_stocks().ordered_rows() {
        if row.id <= 0 {
            return Err(DecisionDataError::InvalidIdentity);
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
        let mut rows = config
            .du_shop_items()
            .ordered_rows()
            .filter(|item| item.stock_id == row.id)
            .collect::<Vec<_>>();
        rows.sort_by_key(|item| item.ordinal);
        ordinals(rows.iter().map(|item| item.ordinal))?;
        let mut blessings = BTreeSet::new();
        let mut owners = BTreeSet::new();
        let mut items = Vec::new();
        for item in rows {
            if item.id <= 0 {
                return Err(DecisionDataError::InvalidIdentity);
            }
            let price = item
                .price
                .parse::<u64>()
                .map_err(|_| DecisionDataError::InvalidPolicy)?;
            if price == 0 || i64::try_from(price).is_err() || price.to_string() != item.price {
                return Err(DecisionDataError::InvalidPolicy);
            }
            let reward = match item.reward_kind {
                DuShopRewardKind::Blessing => {
                    let blessing = reference
                        .blessing_catalog()
                        .blessings()
                        .iter()
                        .find(|blessing| blessing.id.as_str() == item.reward_key)
                        .ok_or(DecisionDataError::InvalidReference)?;
                    if !blessings.insert(blessing.id.clone()) {
                        return Err(DecisionDataError::InvalidReward);
                    }
                    ShopStockReward::Blessing(blessing.id.clone())
                }
                DuShopRewardKind::Curio => {
                    let state = reference
                        .curio_catalog()
                        .states()
                        .iter()
                        .find(|state| state.id.as_str() == item.reward_key)
                        .ok_or(DecisionDataError::InvalidReference)?;
                    // Evolution-only successors are unbound in the reference catalog.
                    let owner = state
                        .curio
                        .as_ref()
                        .ok_or(DecisionDataError::InvalidReward)?;
                    if state.category == DivergentUniverseCurioCategory::Negative
                        || !owners.insert(owner.clone())
                    {
                        return Err(DecisionDataError::InvalidReward);
                    }
                    ShopStockReward::Curio(state.id.clone())
                }
            };
            let identity_table = match item.reward_kind {
                DuShopRewardKind::Blessing => "ExcelOutput/RogueTournBuff.json;",
                DuShopRewardKind::Curio => "ExcelOutput/RogueTournMiracle.json;",
            };
            let sources = source_keys(config, &item.source_ids)?;
            if !item.source_ids.iter().any(|id| {
                config.du_decision_sources().get(id).is_some_and(|source| {
                    source.quality == DuDecisionEvidence::ExactStructured
                        && source.locator.starts_with(identity_table)
                })
            }) {
                return Err(DecisionDataError::InvalidProvenance);
            }
            items.push(ShopStockItemDefinition {
                key: key(&item.stable_key)?,
                ordinal: u16::try_from(item.ordinal)
                    .map_err(|_| DecisionDataError::InvalidOrder)?,
                reward,
                price,
                sources,
            });
        }
        result.push(ShopStockDefinition {
            id: ShopStockId::new(row.stable_key.clone())?,
            policy: match row.policy {
                DuShopStockPolicy::ExplicitFixedStockAndPriceBaseRewards => {
                    ShopStockPolicy::VersionedProjectPolicyExplicitFixedStockAndPriceBaseRewards
                }
            },
            items: items.into_boxed_slice(),
            summary_en: row.summary_en.clone().into(),
            summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(),
            replacement_condition: row.replacement_condition.clone().into(),
        });
    }
    result.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(result.into_boxed_slice())
}
