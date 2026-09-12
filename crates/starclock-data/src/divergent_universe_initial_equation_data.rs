//! Initial Equation pool policy, distinct from exact category membership.

use super::source_keys;
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_decisions::{DecisionDataError, InitialEquationPolicy, key},
    divergent_universe_decisions_generated::{
        SoraConfig, du_initial_equation_category::DuInitialEquationCategory,
        du_initial_equation_policy::DuInitialEquationPolicy,
    },
    divergent_universe_equation_catalog::DivergentUniverseEquationCategory,
};

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<InitialEquationPolicy, DecisionDataError> {
    let rows = config
        .du_initial_equations()
        .ordered_rows()
        .collect::<Vec<_>>();
    let [row] = rows.as_slice() else {
        return Err(DecisionDataError::InvalidPolicy);
    };
    match row.policy {
        DuInitialEquationPolicy::UniformCurrentCategorySingleSelection => {}
    }
    let category = match row.category {
        DuInitialEquationCategory::Epic => DivergentUniverseEquationCategory::Epic,
    };
    let count = reference
        .equation_catalog()
        .equations()
        .iter()
        .filter(|equation| equation.category == category)
        .count();
    if row.id <= 0
        || !(1..=8).contains(&row.offer_width)
        || usize::try_from(row.offer_width).map_or(true, |width| width > count)
        || row.summary_en.is_empty()
        || row.summary_zh_cn.is_empty()
        || row.policy_note.is_empty()
        || row.replacement_condition.is_empty()
    {
        return Err(DecisionDataError::InvalidPolicy);
    }
    Ok(InitialEquationPolicy {
        key: key(&row.stable_key)?,
        offer_width: u16::try_from(row.offer_width)
            .map_err(|_| DecisionDataError::InvalidPolicy)?,
        category,
        summary_en: row.summary_en.clone().into(),
        summary_zh_cn: row.summary_zh_cn.clone().into(),
        policy_note: row.policy_note.clone().into(),
        replacement_condition: row.replacement_condition.clone().into(),
        sources: source_keys(config, &row.source_ids)?,
    })
}
