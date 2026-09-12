//! Provisional logical-layer battle topology with explicit evidence boundaries.

use super::source_keys;
use crate::{
    divergent_universe_decisions::{
        BattleRoutePolicy, BattleRoutePolicyKind, DecisionDataError, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_battle_route_policy::DuBattleRoutePolicy,
    },
};

pub(super) fn compile(config: &SoraConfig) -> Result<BattleRoutePolicy, DecisionDataError> {
    let rows = config.du_battle_routes().ordered_rows().collect::<Vec<_>>();
    let [row] = rows.as_slice() else {
        return Err(DecisionDataError::InvalidPolicy);
    };
    if row.id <= 0
        || row.battles_per_layer != 1
        || row.summary_en.is_empty()
        || row.summary_zh_cn.is_empty()
        || row.policy_note.is_empty()
        || row.replacement_condition.is_empty()
    {
        return Err(DecisionDataError::InvalidPolicy);
    }
    Ok(BattleRoutePolicy {
        key: key(&row.stable_key)?,
        kind: match row.policy {
            DuBattleRoutePolicy::OneBattlePerLayerWithDomainChoices => {
                BattleRoutePolicyKind::VersionedProjectPolicyOneBattlePerLayerWithDomainChoices
            }
        },
        summary_en: row.summary_en.clone().into(),
        summary_zh_cn: row.summary_zh_cn.clone().into(),
        policy_note: row.policy_note.clone().into(),
        replacement_condition: row.replacement_condition.clone().into(),
        sources: source_keys(config, &row.source_ids)?,
    })
}
