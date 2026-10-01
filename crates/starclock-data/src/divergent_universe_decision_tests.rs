use crate::divergent_universe::load_divergent_universe_bundle;
use crate::divergent_universe_decisions_generated::du_decision_reward_kind::DuDecisionRewardKind;

use crate::divergent_universe_decisions_generated::{
    SoraConfig,
    runtime::{SoraBundle, SoraReadError},
};
use std::collections::BTreeSet;
#[path = "divergent_universe_decision_test_transport.rs"]
mod transport;
use transport::EditedRows;
#[path = "divergent_universe_domain_deck_tests.rs"]
mod domain_decks;
#[path = "divergent_universe_domain_layout_tests.rs"]
mod domain_layout;

#[path = "divergent_universe_adventure_reward_tests.rs"]
mod adventure_rewards;
#[path = "divergent_universe_coin_reward_tests.rs"]
mod coin_rewards;
#[path = "divergent_universe_expansion_policy_tests.rs"]
mod expansion_policy;
#[path = "divergent_universe_decision_rejection_tests.rs"]
mod rejection;
#[path = "divergent_universe_reward_occurrence_tests.rs"]
mod reward_occurrences;
#[path = "divergent_universe_reward_policy_tests.rs"]
mod reward_policies;
#[path = "divergent_universe_shop_tests.rs"]
mod shop;
#[path = "divergent_universe_weighted_curio_attack_debuff_tests.rs"]
mod weighted_curio_attack_debuffs;
#[path = "divergent_universe_weighted_curio_prayer_tests.rs"]
mod weighted_curio_prayers;
#[path = "divergent_universe_weighted_curio_shield_tests.rs"]
mod weighted_curio_shields;
#[path = "divergent_universe_weighted_curio_splash_tests.rs"]
mod weighted_curio_splashes;
#[path = "divergent_universe_weighted_curio_support_attack_tests.rs"]
mod weighted_curio_support_attacks;

use super::{
    BUNDLE, BattleRewardDomain, CurioAcquisitionGrant, CurioAcquisitionPolicy, DecisionCatalog,
    DecisionDataError, DecisionPolicyKind, DecisionReward, reward,
    validation::{compile, ordinals},
};

#[test]
fn production_decision_workbook_lowers_three_ordered_policy_choices() {
    let reference = load_divergent_universe_bundle().unwrap();
    let catalog = DecisionCatalog::production(&reference).unwrap();
    assert_eq!(catalog.sources().len(), 92);
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
