use crate::divergent_universe::load_divergent_universe_bundle;
use crate::divergent_universe_decisions_generated::du_decision_reward_kind::DuDecisionRewardKind;

use crate::divergent_universe_decisions_generated::{
    SoraConfig,
    runtime::{SoraBundle, SoraReadError},
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
#[path = "divergent_universe_decision_test_transport.rs"]
mod transport;
use transport::EditedRows;
#[path = "divergent_universe_domain_deck_tests.rs"]
mod domain_decks;
#[path = "divergent_universe_domain_layout_tests.rs"]
mod domain_layout;

#[path = "divergent_universe_expansion_policy_tests.rs"]
mod expansion_policy;
#[path = "divergent_universe_reward_policy_tests.rs"]
mod reward_policies;

use super::{
    BUNDLE, BattleRewardDomain, CurioAcquisitionGrant, CurioAcquisitionPolicy, DecisionCatalog,
    DecisionDataError, DecisionPolicyKind, DecisionReward, reward,
    validation::{compile, ordinals},
};

#[test]
fn production_decision_workbook_lowers_three_ordered_policy_choices() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    assert_eq!(catalog.sources().len(), 69);
    expansion_policy::production(&catalog);
    reward_policies::production_battle_stats(&catalog);
    assert_eq!(
        catalog
            .curio_domain_expiries()
            .iter()
            .map(|row| (row.state.as_str(), row.domain_limit))
            .collect::<Vec<_>>(),
        [
            ("divergent-universe.curio-state.9070", 3),
            ("divergent-universe.curio-state.9071", 3),
            ("divergent-universe.curio-state.9072", 3),
            ("divergent-universe.curio-state.9079", 5)
        ]
    );
    assert_eq!(
        catalog
            .battle_fragments()
            .iter()
            .map(|row| (row.domain, row.amount))
            .collect::<Vec<_>>(),
        [
            (BattleRewardDomain::Combat, 40),
            (BattleRewardDomain::Elite, 100),
            (BattleRewardDomain::Aberration, 40),
            (BattleRewardDomain::Boss, 100)
        ]
    );
    assert_eq!(catalog.domain_choices().len(), 3);
    assert_eq!(catalog.curio_victory_blessings().len(), 4);
    for (index, definition) in catalog.curio_victory_blessings()[1..].iter().enumerate() {
        assert_eq!(usize::from(definition.count), index + 1);
        assert_eq!(
            (definition.rarity.minimum(), definition.rarity.maximum()),
            (1, 3)
        );
        assert_eq!(
            definition.domains.as_ref(),
            &[BattleRewardDomain::Elite, BattleRewardDomain::Aberration]
        );
    }
    assert_eq!(catalog.evolution_events().len(), 4);
    assert_eq!(
        catalog
            .evolution_events()
            .iter()
            .map(|event| event.key.as_ref())
            .collect::<Vec<_>>(),
        [
            "du.evolution-event.green.ii",
            "du.evolution-event.sage.ii",
            "du.evolution-event.green.iii",
            "du.evolution-event.sage.iii"
        ]
    );
    assert_eq!(
        catalog
            .evolution_events()
            .iter()
            .map(|event| event.options.len())
            .sum::<usize>(),
        12
    );
    assert_eq!(catalog.curio_battle_grants().len(), 3);
    assert_eq!(
        catalog
            .curio_battle_grants()
            .iter()
            .map(|row| row.amount_per_full_hp)
            .collect::<BTreeSet<_>>(),
        [8, 16, 32].into()
    );
    assert_eq!(catalog.curio_evolutions().len(), 4);
    for edge in catalog.curio_evolutions() {
        assert!(
            [
                "divergent-universe.curio.9154",
                "divergent-universe.curio.9155"
            ]
            .contains(&edge.owner.as_str())
        );
        assert!(
            reference
                .curio_catalog()
                .states()
                .iter()
                .find(|state| state.id == edge.to)
                .unwrap()
                .curio
                .is_none()
        );
    }
    assert_eq!(catalog.encounter_pool().candidate_stages.len(), 5);
    assert_eq!(catalog.encounter_pool().first_stage.as_ref(), "83002081");
    assert_eq!(catalog.initial_equations().offer_width, 3);
    assert_eq!(catalog.equation_grants().len(), 1);
    assert_eq!(catalog.equation_grants()[0].count, 3);
    assert_eq!(
        catalog.equation_grants()[0].state.as_str(),
        "divergent-universe.curio-state.9187"
    );
    assert_eq!(catalog.curio_battle_weights().len(), 8);
    assert!(
        catalog
            .curio_battle_weights()
            .iter()
            .all(|row| row.bonus_weight == 3)
    );
    let [battle] = catalog.battle_blessings() else {
        panic!("one normal battle policy");
    };
    assert_eq!(battle.offer_width, 3);
    assert_eq!((battle.rarity.minimum(), battle.rarity.maximum()), (1, 2));
    assert_eq!(
        battle.suppression_states[0].as_str(),
        "divergent-universe.curio-state.9055"
    );
    assert_eq!(
        catalog
            .curio_fragment_gains()
            .iter()
            .map(|gain| (gain.state.as_str(), gain.numerator, gain.denominator))
            .collect::<Vec<_>>(),
        [
            ("divergent-universe.curio-state.9055", 5, 10),
            ("divergent-universe.curio-state.9070", 5, 10),
            ("divergent-universe.curio-state.9079", 3, 10),
            ("divergent-universe.curio-state.9159", 3, 10),
        ]
    );
    assert_eq!(catalog.curio_acquisitions().len(), 18);
    assert_eq!(
        catalog
            .curio_acquisitions()
            .iter()
            .filter(|effect| matches!(
                effect.grant,
                CurioAcquisitionGrant::FixedFragments(_)
                    | CurioAcquisitionGrant::BalanceFraction { .. }
            ))
            .map(|effect| effect.grant.clone())
            .collect::<Vec<_>>(),
        [
            CurioAcquisitionGrant::FixedFragments(300),
            CurioAcquisitionGrant::BalanceFraction {
                numerator: 4,
                denominator: 10
            },
            CurioAcquisitionGrant::FixedFragments(500),
            CurioAcquisitionGrant::FixedFragments(150),
            CurioAcquisitionGrant::FixedFragments(300),
            CurioAcquisitionGrant::FixedFragments(600),
        ]
    );
    assert_eq!(
        catalog
            .curio_acquisitions()
            .iter()
            .filter(|effect| reference
                .curio_catalog()
                .states()
                .iter()
                .any(|state| state.id == effect.state && state.curio.is_some()))
            .count(),
        14
    );
    let mut path_widths = Vec::new();
    for effect in catalog.curio_acquisitions() {
        match &effect.grant {
            CurioAcquisitionGrant::PathBlessings { count, paths } => {
                assert_eq!(*count, 1);
                assert!(paths.windows(2).all(|pair| pair[0] < pair[1]));
                path_widths.push(paths.len());
                assert_eq!(effect.policy, CurioAcquisitionPolicy::VersionedProjectPolicyStableStateOrderUniformUnownedPathRejectExhaustion);
            }
            CurioAcquisitionGrant::RarityBlessings { count, rarity } => {
                assert_eq!(u16::from(rarity.minimum()), *count);
                assert_eq!(rarity.minimum(), rarity.maximum());
                assert_eq!(
                    effect.policy,
                    CurioAcquisitionPolicy::FeasibleRarityAssignments
                );
            }
            _ => assert_eq!(
                effect.policy,
                CurioAcquisitionPolicy::VersionedProjectPolicyStableStateOrderFloorBeforeEachGrant
            ),
        }
    }
    path_widths.sort_unstable();
    assert_eq!(path_widths, [1, 1, 1, 1, 1, 1, 1, 1, 8]);
    assert_eq!(catalog.occurrences().len(), 1);
    let event = &catalog.occurrences()[0];
    assert_eq!(
        event.occurrence.as_str(),
        "divergent-universe.occurrence.108"
    );
    assert_eq!(
        event.variant.as_str(),
        "divergent-universe.occurrence-variant.722601"
    );
    assert_eq!(
        event.policy.kind,
        DecisionPolicyKind::VersionedProjectPolicyUniformUnownedCurrentCatalog
    );
    assert_eq!(event.choices.len(), 3);
    assert_eq!(
        event.choices[0].outcomes[0].reward,
        DecisionReward::Fragments(200)
    );
    assert!(
        event
            .choices
            .iter()
            .all(|choice| choice.fragment_cost == 0 && choice.outcomes.len() == 1)
    );
    assert_eq!(
        event
            .choices
            .iter()
            .map(|choice| choice.ordinal)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    match event.choices[1].outcomes[0].reward {
        DecisionReward::Curios { count, rarity } => {
            assert_eq!(count, 2);
            assert_eq!((rarity.minimum(), rarity.maximum()), (1, 2));
        }
        other => panic!("expected Curio draw, got {other:?}"),
    }
    match event.choices[2].outcomes[0].reward {
        DecisionReward::Blessings { count, rarity } => {
            assert_eq!(count, 2);
            assert_eq!((rarity.minimum(), rarity.maximum()), (1, 2));
        }
        other => panic!("expected Blessing draw, got {other:?}"),
    }
    assert_eq!(catalog, DecisionCatalog::production(&reference).unwrap());
    assert_ne!(catalog.digest(), [0; 32]);
}

#[test]
fn decision_transport_rejects_truncation_and_non_sora_inputs() {
    let reference = load_divergent_universe_bundle().unwrap();
    for bytes in [b"{}".as_slice(), &BUNDLE[..16], &BUNDLE[..BUNDLE.len() / 2]] {
        assert_eq!(
            DecisionCatalog::from_bytes(bytes, &reference),
            Err(DecisionDataError::Transport)
        );
    }
}

#[test]
fn typed_rewards_reject_invalid_amounts_ranges_and_currency_rarity() {
    let invalid = [
        (DuDecisionRewardKind::Fragments, 0, None, None),
        (DuDecisionRewardKind::Fragments, -1, None, None),
        (DuDecisionRewardKind::Fragments, 200, Some(1), Some(2)),
        (DuDecisionRewardKind::Curios, 2, None, Some(2)),
        (DuDecisionRewardKind::Blessings, 2, Some(2), Some(1)),
        (DuDecisionRewardKind::Curios, 2, Some(0), Some(2)),
        (DuDecisionRewardKind::Blessings, 2, Some(1), Some(4)),
        (DuDecisionRewardKind::Curios, 65_536, Some(1), Some(2)),
    ];
    for (kind, amount, minimum, maximum) in invalid {
        assert_eq!(
            reward(kind, amount, minimum, maximum),
            Err(DecisionDataError::InvalidReward)
        );
    }
    assert_eq!(
        reward(DuDecisionRewardKind::Fragments, i32::MAX, None, None),
        Ok(DecisionReward::Fragments(2_147_483_647))
    );
}

#[test]
fn decision_child_ordinals_must_be_nonempty_contiguous_and_bounded() {
    for values in [
        vec![],
        vec![0],
        vec![1, 1],
        vec![1, 3],
        vec![2, 1],
        (1..=65).collect(),
    ] {
        assert_eq!(
            ordinals(values.into_iter()),
            Err(DecisionDataError::InvalidOrder)
        );
    }
    assert_eq!(ordinals(1..=64), Ok(()));
}

#[test]
fn semantic_decision_validation_rejects_bad_joins_provenance_and_orphan_rows() {
    let reference = load_divergent_universe_bundle().unwrap();
    let bundle = SoraBundle::parse(BUNDLE).unwrap();
    let config = SoraConfig::from_source(&bundle).unwrap();
    let baseline = BTreeMap::from([
        expansion_policy::table(&config),
        reward_policies::battle_stats_table(&config),
        reward_policies::battle_reactions_table(&config),
        reward_policies::tawot_services_table(&config),
        reward_policies::domain_grants_table(&config),
        (
            "DuCurioDomainExpiries",
            serde_json::to_value(
                config
                    .du_curio_domain_expiries()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuBattleFragments",
            serde_json::to_value(
                config
                    .du_battle_fragments()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuEvolutionEvents",
            serde_json::to_value(
                config
                    .du_evolution_events()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuEvolutionOptions",
            serde_json::to_value(
                config
                    .du_evolution_options()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuCurioEvolutions",
            serde_json::to_value(
                config
                    .du_curio_evolutions()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuCurioBattleGrants",
            serde_json::to_value(
                config
                    .du_curio_battle_grants()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuEncounterPools",
            serde_json::to_value(
                config
                    .du_encounter_pools()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuDomainChoices",
            serde_json::to_value(
                config
                    .du_domain_choices()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuCurioVictoryBlessings",
            serde_json::to_value(
                config
                    .du_curio_victory_blessings()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuBattleRoutes",
            serde_json::to_value(config.du_battle_routes().ordered_rows().collect::<Vec<_>>())
                .unwrap(),
        ),
        (
            "DuInitialEquations",
            serde_json::to_value(
                config
                    .du_initial_equations()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuEquationGrants",
            serde_json::to_value(
                config
                    .du_equation_grants()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuCurioBattleWeights",
            serde_json::to_value(
                config
                    .du_curio_battle_weights()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuBattleBlessings",
            serde_json::to_value(
                config
                    .du_battle_blessings()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuCurioFragmentGains",
            serde_json::to_value(
                config
                    .du_curio_fragment_gains()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuCurioAcquisitions",
            serde_json::to_value(
                config
                    .du_curio_acquisitions()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuDecisionSources",
            serde_json::to_value(
                config
                    .du_decision_sources()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuDecisionPolicies",
            serde_json::to_value(
                config
                    .du_decision_policies()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuDecisionOccurrences",
            serde_json::to_value(
                config
                    .du_decision_occurrences()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuDecisionChoices",
            serde_json::to_value(
                config
                    .du_decision_choices()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        (
            "DuDecisionOutcomes",
            serde_json::to_value(
                config
                    .du_decision_outcomes()
                    .ordered_rows()
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
    ]);
    let edits = [
        (
            "DuCurioVictoryBlessings",
            "count",
            Value::from(0),
            DecisionDataError::InvalidReward,
        ),
        (
            "DuCurioVictoryBlessings",
            "count",
            Value::from(2),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioVictoryBlessings",
            "count_parameter",
            Value::from(0),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioVictoryBlessings",
            "effect_id",
            Value::from("2194"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioVictoryBlessings",
            "minimum_rarity",
            Value::from(0),
            DecisionDataError::InvalidReward,
        ),
        (
            "DuCurioVictoryBlessings",
            "maximum_rarity",
            Value::from(4),
            DecisionDataError::InvalidReward,
        ),
        (
            "DuCurioVictoryBlessings",
            "domains",
            Value::from(Vec::<String>::new()),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuCurioVictoryBlessings",
            "domains",
            Value::from(vec!["Elite", "Elite"]),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuCurioVictoryBlessings",
            "policy_note",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuCurioVictoryBlessings",
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuEvolutionEvents",
            "layer_ordinal",
            Value::from(3),
            DecisionDataError::InvalidIdentity,
        ),
        (
            "DuEvolutionEvents",
            "evolution_id",
            Value::from(999),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuEvolutionEvents",
            "layer_ordinal",
            Value::from(1),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuEvolutionEvents",
            "policy_note",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuEvolutionOptions",
            "event_id",
            Value::from(999),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuEvolutionOptions",
            "fragment_cost",
            Value::from(-1),
            DecisionDataError::InvalidReward,
        ),
        (
            "DuEvolutionOptions",
            "curio_count",
            Value::from(2),
            DecisionDataError::InvalidReward,
        ),
        (
            "DuEvolutionOptions",
            "success_denominator",
            Value::from(0),
            DecisionDataError::InvalidReward,
        ),
        (
            "DuEvolutionOptions",
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioEvolutions",
            "from_state",
            Value::from("divergent-universe.curio-state.9197"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioEvolutions",
            "to_state",
            Value::from("divergent-universe.curio-state.9197"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioEvolutions",
            "owner",
            Value::from("divergent-universe.curio.9001"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioEvolutions",
            "occurrence",
            Value::from("divergent-universe.occurrence.0"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioEvolutions",
            "policy_note",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuCurioEvolutions",
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioBattleGrants",
            "state_key",
            Value::from("divergent-universe.curio-state.9196"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioBattleGrants",
            "effect_id",
            Value::from("2196"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioBattleGrants",
            "amount_parameter",
            Value::from(1),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioBattleGrants",
            "amount_per_full_hp",
            Value::from(16),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioBattleGrants",
            "amount_per_full_hp",
            Value::from(0),
            DecisionDataError::InvalidReward,
        ),
        (
            "DuCurioBattleGrants",
            "policy_note",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuCurioBattleGrants",
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuEncounterPools",
            "encounter_group",
            Value::from("divergent-universe.encounter-group.0"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuEncounterPools",
            "first_stage",
            Value::from("0"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuEncounterPools",
            "candidate_stages",
            Value::from(vec!["0"]),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuEncounterPools",
            "candidate_stages",
            Value::from(Vec::<String>::new()),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuEncounterPools",
            "candidate_stages",
            Value::from(vec!["83002081", "83002081"]),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuEncounterPools",
            "candidate_stages",
            Value::from(vec!["83002111", "83002081"]),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuEncounterPools",
            "policy_note",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuEncounterPools",
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuBattleRoutes",
            "battles_per_layer",
            Value::from(0),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuBattleRoutes",
            "battles_per_layer",
            Value::from(2),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuBattleRoutes",
            "policy_note",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuBattleRoutes",
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuInitialEquations",
            "offer_width",
            Value::from(0),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuInitialEquations",
            "offer_width",
            Value::from(9),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuInitialEquations",
            "policy_note",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuInitialEquations",
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuEquationGrants",
            "count",
            Value::from(2),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuEquationGrants",
            "count_parameter",
            Value::from(1),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuEquationGrants",
            "domain_limit",
            Value::from(2),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuEquationGrants",
            "effect_id",
            Value::from("2043"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuEquationGrants",
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioBattleWeights",
            "bonus_weight",
            Value::from(0),
            DecisionDataError::InvalidReward,
        ),
        (
            "DuCurioBattleWeights",
            "bonus_weight",
            Value::from(1_000_001),
            DecisionDataError::InvalidReward,
        ),
        (
            "DuCurioBattleWeights",
            "state_key",
            Value::from("divergent-universe.curio-state.9001"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioBattleWeights",
            "effect_id",
            Value::from("2044"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioBattleWeights",
            "path_type",
            Value::from("124"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioBattleWeights",
            "policy_note",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuCurioBattleWeights",
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuBattleBlessings",
            "offer_width",
            Value::from(0),
            DecisionDataError::InvalidReward,
        ),
        (
            "DuBattleBlessings",
            "maximum_rarity",
            Value::from(0),
            DecisionDataError::InvalidReward,
        ),
        (
            "DuBattleBlessings",
            "suppression_states",
            Value::from(vec!["divergent-universe.curio-state.9001"]),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuBattleBlessings",
            "suppression_effects",
            Value::from(vec!["2001"]),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuBattleBlessings",
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuBattleBlessings",
            "policy_note",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuBattleBlessings",
            "replacement_condition",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuCurioFragmentGains",
            "state_key",
            Value::from("divergent-universe.curio-state.9001"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioFragmentGains",
            "effect_id",
            Value::from("2079"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioFragmentGains",
            "bonus_fraction",
            Value::from("0.3"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioFragmentGains",
            "parameter_index",
            Value::from(0),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioFragmentGains",
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioFragmentGains",
            "replacement_condition",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuCurioAcquisitions",
            "state_key",
            Value::from("divergent-universe.curio-state.9001"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioAcquisitions",
            "effect_id",
            Value::from("2053"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioAcquisitions",
            "amount",
            Value::from("301"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioAcquisitions",
            "parameter_index",
            Value::from(0),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuCurioAcquisitions",
            "replacement_condition",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuCurioAcquisitions",
            "source_ids",
            Value::from(vec![999]),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuDecisionOccurrences",
            "reference_variant",
            Value::from("divergent-universe.occurrence-variant.700103"),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuDecisionChoices",
            "occurrence_id",
            Value::from(999),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuDecisionOutcomes",
            "choice_id",
            Value::from(999),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuDecisionOutcomes",
            "source_id",
            Value::from(999),
            DecisionDataError::InvalidReference,
        ),
        (
            "DuDecisionChoices",
            "ordinal",
            Value::from(2),
            DecisionDataError::InvalidOrder,
        ),
        (
            "DuDecisionChoices",
            "fragment_cost",
            Value::from(-1),
            DecisionDataError::InvalidReward,
        ),
        (
            "DuDecisionPolicies",
            "replacement_condition",
            Value::from(""),
            DecisionDataError::InvalidPolicy,
        ),
        (
            "DuDecisionSources",
            "sha256",
            Value::from("not-a-digest"),
            DecisionDataError::InvalidProvenance,
        ),
        (
            "DuDecisionPolicies",
            "source_ids",
            Value::from(vec![1, 1]),
            DecisionDataError::InvalidProvenance,
        ),
    ];
    reward_policies::validate(&baseline, &reference);
    expansion_policy::validate(&baseline, &reference);
    for (table, field, value, expected) in edits {
        let mut rows = EditedRows(baseline.clone());
        rows.0.get_mut(table).unwrap()[0][field] = value;
        let edited = SoraConfig::from_source(&rows).unwrap();
        assert_eq!(
            compile(&edited, &reference),
            Err(expected),
            "{table}.{field}"
        );
    }
    for (index, field, value, expected) in [
        (
            0,
            "minimum_rarity",
            Value::from(1),
            DecisionDataError::InvalidReward,
        ),
        (
            15,
            "minimum_rarity",
            Value::Null,
            DecisionDataError::InvalidReward,
        ),
        (
            15,
            "maximum_rarity",
            Value::Null,
            DecisionDataError::InvalidReward,
        ),
        (
            15,
            "minimum_rarity",
            Value::from(0),
            DecisionDataError::InvalidReward,
        ),
        (
            15,
            "minimum_rarity",
            Value::from(2),
            DecisionDataError::InvalidReward,
        ),
        (
            15,
            "maximum_rarity",
            Value::from(4),
            DecisionDataError::InvalidReward,
        ),
        (
            15,
            "path_types",
            serde_json::json!(["126"]),
            DecisionDataError::InvalidReward,
        ),
        (
            15,
            "amount",
            Value::from("2"),
            DecisionDataError::InvalidReference,
        ),
        (
            15,
            "policy",
            Value::from("StableStateOrderFloorBeforeEachGrant"),
            DecisionDataError::InvalidPolicy,
        ),
        (
            0,
            "path_types",
            serde_json::json!(["126"]),
            DecisionDataError::InvalidReward,
        ),
        (
            6,
            "path_types",
            Value::Null,
            DecisionDataError::InvalidReference,
        ),
        (
            6,
            "path_types",
            serde_json::json!([]),
            DecisionDataError::InvalidReference,
        ),
        (
            6,
            "path_types",
            serde_json::json!(["126", "126"]),
            DecisionDataError::InvalidReference,
        ),
        (
            6,
            "path_types",
            serde_json::json!(["123"]),
            DecisionDataError::InvalidReference,
        ),
        (
            6,
            "policy",
            Value::from("StableStateOrderFloorBeforeEachGrant"),
            DecisionDataError::InvalidPolicy,
        ),
    ] {
        let mut rows = EditedRows(baseline.clone());
        rows.0.get_mut("DuCurioAcquisitions").unwrap()[index][field] = value;
        let edited = SoraConfig::from_source(&rows).unwrap();
        assert_eq!(
            compile(&edited, &reference),
            Err(expected),
            "acquisition[{index}].{field}"
        );
    }
}
