use crate::{
    CurrencyWarsAreaGroup, CurrencyWarsAreaSelectionPolicy, CurrencyWarsEntry,
    CurrencyWarsEntryKind, CurrencyWarsEntryRule, CurrencyWarsFinishCondition,
    CurrencyWarsFinishRule, CurrencyWarsFlowCatalog, CurrencyWarsFlowCatalogParts,
    CurrencyWarsGambit, CurrencyWarsGambitDefinition, CurrencyWarsModule, CurrencyWarsProfile,
};

#[test]
fn unresolved_profile_reference_rejects_the_entire_flow_catalog() {
    let result = CurrencyWarsFlowCatalog::new(CurrencyWarsFlowCatalogParts {
        profile: CurrencyWarsProfile {
            stable_key: "profile".into(),
            entry_ids: Box::new(["entry".into()]),
            module_id: "missing-module".into(),
            gambits: Box::new([CurrencyWarsGambit::Standard]),
            initial_resource_ids: Box::new([]),
            finish_condition_ids: Box::new(["finish".into()]),
        },
        modules: vec![CurrencyWarsModule {
            stable_key: "module".into(),
            source_id: 1,
            season_id: 1,
            sub_season_id: 1,
        }],
        entries: vec![CurrencyWarsEntry {
            stable_key: "entry".into(),
            kind: CurrencyWarsEntryKind::GuideData,
            module_id: "module".into(),
            unlocks: Box::new([]),
            gambits: Box::new([CurrencyWarsGambit::Standard]),
        }],
        gambits: vec![CurrencyWarsGambitDefinition {
            stable_key: "gambit.standard".into(),
            gambit: CurrencyWarsGambit::Standard,
            unlocks: Box::new([]),
            entry_rules: Box::new([CurrencyWarsEntryRule::StandardDifficultyBoundedByHighestRank]),
            initial_resource_ids: Box::new([]),
        }],
        finish_conditions: vec![
            CurrencyWarsFinishCondition {
                stable_key: "finish".into(),
                rule: CurrencyWarsFinishRule::SettlementRank {
                    left_inclusive: None,
                    right_inclusive: None,
                    rank_type: None,
                },
            },
            CurrencyWarsFinishCondition {
                stable_key: "finish.b".into(),
                rule: CurrencyWarsFinishRule::SettlementRank {
                    left_inclusive: Some(1),
                    right_inclusive: Some(u32::MAX),
                    rank_type: Some("B".into()),
                },
            },
        ],
        area_group: CurrencyWarsAreaGroup {
            stable_key: "area-group".into(),
            routes: Box::new([]),
            selection_policy: CurrencyWarsAreaSelectionPolicy::CompleteGridFightStageRouteClosure,
            transition_rules: Box::new([]),
        },
        routes: vec![],
        difficulties: vec![],
        layers: vec![],
        rooms: vec![],
        domain_compositions: vec![],
        stage_flow: vec![],
        rank_progression: vec![],
    });

    assert_eq!(
        result.unwrap_err().to_string(),
        "Currency Wars profile reference is invalid"
    );
}
