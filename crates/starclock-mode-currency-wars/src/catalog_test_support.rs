use super::*;
use crate::{
    CurrencyWarsAugmentDefinition, CurrencyWarsAugmentLifecycle, CurrencyWarsAugmentQuality,
    CurrencyWarsConsumableDefinition, CurrencyWarsConsumableKind, CurrencyWarsContentKind,
    CurrencyWarsContentRecord, CurrencyWarsDecimal, CurrencyWarsDifficultyEnemyScaling,
    CurrencyWarsEnhancement, CurrencyWarsEnhancementSelectCondition, CurrencyWarsEquipmentCategory,
    CurrencyWarsEquipmentId, CurrencyWarsEquipmentRecipe, CurrencyWarsEquipmentUpgrade,
    CurrencyWarsForgeService, CurrencyWarsForgeTarget, CurrencyWarsItemDefinition,
    CurrencyWarsItemId, CurrencyWarsManagedFunction, CurrencyWarsMazeBuffEnhancement,
    CurrencyWarsModuleRoleBan, CurrencyWarsOrbDefinition, CurrencyWarsOrbDisplay,
    CurrencyWarsOrbType, CurrencyWarsPortalDefinition, CurrencyWarsProgressionCatalogParts,
    CurrencyWarsProjectionDefinition, CurrencyWarsRewardDefinition, CurrencyWarsRewardKind,
    CurrencyWarsRewardPool, CurrencyWarsRewardPoolCandidate, CurrencyWarsRoleCostAvailability,
    CurrencyWarsRoleReferenceScore, CurrencyWarsRunPosition, CurrencyWarsSeasonProgressionRule,
    CurrencyWarsSeasonRolePool, CurrencyWarsSeasonTraitRolePool, CurrencyWarsSelectedEnhancement,
    CurrencyWarsSelectedEnhancementId, CurrencyWarsServiceCatalog, CurrencyWarsServiceCatalogParts,
    CurrencyWarsServiceConstant, CurrencyWarsServiceConstantValue, CurrencyWarsSpecialGood,
    CurrencyWarsSpecialGoodAcquisition, CurrencyWarsTalentDefinition, CurrencyWarsTalentKind,
};

pub fn catalog() -> CurrencyWarsCatalog {
    catalog_with_role_cost_threshold(CurrencyWarsRunPosition::new(1, 1).unwrap())
}

pub fn catalog_with_role_cost_threshold(
    cost_one_standard: CurrencyWarsRunPosition,
) -> CurrencyWarsCatalog {
    let role = CurrencyWarsRoleId::new(1001).unwrap();
    let node_ids = (1..=6)
        .map(|raw| CurrencyWarsNodeId::new(raw).unwrap())
        .collect::<Vec<_>>();
    let routes = vec![CurrencyWarsRoute {
        id: CurrencyWarsRouteId::new(100).unwrap(),
        stable_key: "route.100".into(),
        map_entry_id: 100,
        difficulty_ids: Box::new([]),
        layer_ids: Box::new([
            "layer.100.1".into(),
            "layer.100.2".into(),
            "layer.100.3".into(),
        ]),
        nodes: (1..=3)
            .flat_map(|plane| {
                let battle_index = usize::from((plane - 1) * 2);
                let supply_index = battle_index + 1;
                [
                    test_node(
                        node_ids[battle_index],
                        plane,
                        1,
                        CurrencyWarsNodeKind::Monster,
                        Some(node_ids[supply_index]),
                    ),
                    test_node(
                        node_ids[supply_index],
                        plane,
                        2,
                        CurrencyWarsNodeKind::Supply,
                        None,
                    ),
                ]
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    }];
    let difficulties = (1..=2)
        .map(|division_level| CurrencyWarsDifficulty {
            source_id: u32::from(division_level),
            stable_key: format!("difficulty.{division_level}").into(),
            season_id: 1,
            division_level,
            progress: u16::from(division_level),
            standard_score_rule: 1,
            overclock_score_rule: 2,
            weekly_score_modifier: starclock_combat::Ratio::ONE,
            experience_modifier: starclock_combat::Ratio::ONE,
            enemy_scaling_refs: Box::new([]),
            enemy_scaling: CurrencyWarsDifficultyEnemyScaling {
                enemy_difficulty_level: 0,
                level_base_hp_ratio: starclock_combat::Scalar::ONE,
                level_base_attack_ratio: starclock_combat::Scalar::ONE,
            },
            enemy_affix_choice_counts: Box::new([]),
            binary_difficulty_rule: None,
        })
        .collect();
    CurrencyWarsCatalog::new(CurrencyWarsCatalogParts {
        flow: CurrencyWarsFlowCatalog::test_fixture(routes, difficulties),
        economy: CurrencyWarsEconomyCatalog::test_fixture(role),
        build: CurrencyWarsBuildCatalog::test_fixture(role),
        empowerment: CurrencyWarsEmpowermentCatalog::test_fixture(role.get()),
        content: CurrencyWarsContentCatalog::new(test_content_records()).unwrap(),
        encounter: CurrencyWarsEncounterCatalog::test_fixture(),
        roles: vec![CurrencyWarsRole {
            id: role,
            stable_key: "role.1001".into(),
            avatar_id: 1001,
            rarity: 1,
            build_mapping_id: "build.role.1001".into(),
            maximum_star: 3,
            positions: Box::new([CurrencyWarsPositionKind::Front]),
            trait_ids: Box::new([1001]),
            backend_rank_ids: Box::new([100_101, 100_102, 100_103, 100_104, 100_105, 100_106]),
        }],
        bonds: CurrencyWarsBondCatalog::test_fixture(role),
        blessing_formula: CurrencyWarsBlessingFormulaCatalog::new(vec![
            CurrencyWarsMazeBuffEnhancement {
                stable_key: "maze-buff-enhancement.1".into(),
                source_id: 1,
                parameters: Box::new([CurrencyWarsDecimal::new(1, 0).unwrap()]),
                effect_ids: Box::new(["ability:fixture".into()]),
            },
        ])
        .unwrap(),
        occurrences: CurrencyWarsOccurrenceCatalog::test_fixture(),
        services: CurrencyWarsServiceCatalog::new(CurrencyWarsServiceCatalogParts {
            items: [350_101, 350_103, 350_104, 350_107, 99_999]
                .into_iter()
                .map(|raw| CurrencyWarsItemDefinition {
                    id: CurrencyWarsItemId::new(raw).unwrap(),
                    stable_key: format!("item.{raw}").into(),
                    priority: 1,
                })
                .collect(),
            special_goods: vec![
                CurrencyWarsSpecialGood {
                    id: 101,
                    stable_key: "special-good.101".into(),
                    group_id: 1,
                    quality: 1,
                    acquisition: CurrencyWarsSpecialGoodAcquisition::ShopPurchase { price: 1 },
                    config_path: "fixture/special-good/101".into(),
                    parameters: Box::new([]),
                },
                CurrencyWarsSpecialGood {
                    id: 107,
                    stable_key: "special-good.107".into(),
                    group_id: 1,
                    quality: 1,
                    acquisition: CurrencyWarsSpecialGoodAcquisition::ShopPurchase { price: 0 },
                    config_path: "fixture/special-good/107".into(),
                    parameters: Box::new([CurrencyWarsDecimal::new(3, 0).unwrap()]),
                },
                CurrencyWarsSpecialGood {
                    id: 201,
                    stable_key: "special-good.201".into(),
                    group_id: 1,
                    quality: 4,
                    acquisition: CurrencyWarsSpecialGoodAcquisition::CyreneThreeStar,
                    config_path: "fixture/special-good/201".into(),
                    parameters: Box::new([CurrencyWarsDecimal::new(100, 0).unwrap()]),
                },
            ],
            season_items: [350_101, 350_103, 350_104, 350_107, 99_999]
                .into_iter()
                .map(|raw| CurrencyWarsItemId::new(raw).unwrap())
                .collect(),
            consumables: vec![
                CurrencyWarsConsumableDefinition {
                    item: CurrencyWarsItemId::new(350_101).unwrap(),
                    stable_key: "consumable.350101".into(),
                    kind: CurrencyWarsConsumableKind::RemoveEquipment,
                    consume: true,
                    stack: true,
                    parameters: Box::new([]),
                },
                CurrencyWarsConsumableDefinition {
                    item: CurrencyWarsItemId::new(350_103).unwrap(),
                    stable_key: "consumable.350103".into(),
                    kind: CurrencyWarsConsumableKind::RerollEquipment,
                    consume: true,
                    stack: true,
                    parameters: Box::new([]),
                },
                CurrencyWarsConsumableDefinition {
                    item: CurrencyWarsItemId::new(350_104).unwrap(),
                    stable_key: "consumable.350104".into(),
                    kind: CurrencyWarsConsumableKind::UpgradeEquipment,
                    consume: true,
                    stack: false,
                    parameters: Box::new([]),
                },
                CurrencyWarsConsumableDefinition {
                    item: CurrencyWarsItemId::new(350_107).unwrap(),
                    stable_key: "consumable.350107".into(),
                    kind: CurrencyWarsConsumableKind::GainRecommendedEquipment,
                    consume: true,
                    stack: false,
                    parameters: Box::new([1]),
                },
            ],
            managed_functions: vec![CurrencyWarsManagedFunction {
                stable_key: "workbench.fixture".into(),
                function_id: "Fixture".into(),
                unlock_id: 1,
                hidden_while_locked: true,
            }],
            rewards: vec![
                CurrencyWarsRewardDefinition {
                    id: 1,
                    stable_key: "reward.1".into(),
                    budget_cost: Some(1),
                    scalar_parameter: Some(1),
                    kind: CurrencyWarsRewardKind::DefaultCurrency,
                },
                CurrencyWarsRewardDefinition {
                    id: 2,
                    stable_key: "reward.2".into(),
                    budget_cost: Some(2),
                    scalar_parameter: Some(1),
                    kind: CurrencyWarsRewardKind::DefaultCurrency,
                },
                CurrencyWarsRewardDefinition {
                    id: 3,
                    stable_key: "reward.3".into(),
                    budget_cost: Some(1),
                    scalar_parameter: Some(1),
                    kind: CurrencyWarsRewardKind::Refresh,
                },
                CurrencyWarsRewardDefinition {
                    id: 4,
                    stable_key: "reward.4".into(),
                    budget_cost: None,
                    scalar_parameter: Some(5),
                    kind: CurrencyWarsRewardKind::Experience,
                },
                CurrencyWarsRewardDefinition {
                    id: 5,
                    stable_key: "reward.5".into(),
                    budget_cost: None,
                    scalar_parameter: None,
                    kind: CurrencyWarsRewardKind::Item {
                        item: CurrencyWarsItemId::new(350_101).unwrap(),
                        count: 1,
                    },
                },
                CurrencyWarsRewardDefinition {
                    id: 6,
                    stable_key: "reward.6".into(),
                    budget_cost: None,
                    scalar_parameter: None,
                    kind: CurrencyWarsRewardKind::Orb(100),
                },
                CurrencyWarsRewardDefinition {
                    id: 7,
                    stable_key: "reward.7".into(),
                    budget_cost: None,
                    scalar_parameter: None,
                    kind: CurrencyWarsRewardKind::RandomRole { rarity: 1, star: 1 },
                },
                CurrencyWarsRewardDefinition {
                    id: 8,
                    stable_key: "reward.8".into(),
                    budget_cost: None,
                    scalar_parameter: None,
                    kind: CurrencyWarsRewardKind::SpecificAvatar {
                        avatar_id: 1001,
                        star: 1,
                    },
                },
                CurrencyWarsRewardDefinition {
                    id: 9,
                    stable_key: "reward.9".into(),
                    budget_cost: None,
                    scalar_parameter: None,
                    kind: CurrencyWarsRewardKind::RandomEquipmentByCategory(1),
                },
                CurrencyWarsRewardDefinition {
                    id: 10,
                    stable_key: "reward.10".into(),
                    budget_cost: None,
                    scalar_parameter: None,
                    kind: CurrencyWarsRewardKind::RandomEquipmentByFunction(10),
                },
                CurrencyWarsRewardDefinition {
                    id: 11,
                    stable_key: "reward.11".into(),
                    budget_cost: None,
                    scalar_parameter: None,
                    kind: CurrencyWarsRewardKind::SpecificAvatarWithEquipment {
                        avatar_id: 1001,
                        star: 1,
                        equipment: Box::new([CurrencyWarsEquipmentId::new(1).unwrap()]),
                    },
                },
                CurrencyWarsRewardDefinition {
                    id: 12,
                    stable_key: "reward.12".into(),
                    budget_cost: None,
                    scalar_parameter: None,
                    kind: CurrencyWarsRewardKind::SpecificAvatarWithRandomEquipment {
                        avatar_id: 1001,
                        star: 1,
                        category_selector: 1,
                        count: 1,
                    },
                },
                CurrencyWarsRewardDefinition {
                    id: 13,
                    stable_key: "reward.13".into(),
                    budget_cost: None,
                    scalar_parameter: None,
                    kind: CurrencyWarsRewardKind::RandomEquipmentByCategory(6),
                },
            ],
            reward_pools: vec![
                CurrencyWarsRewardPool {
                    id: 1,
                    stable_key: "reward-pool.1".into(),
                    total_value: 1,
                    candidates: Box::new([CurrencyWarsRewardPoolCandidate {
                        reward_id: 1,
                        maximum: 1,
                        weight: 1,
                    }]),
                },
                CurrencyWarsRewardPool {
                    id: 2,
                    stable_key: "reward-pool.2".into(),
                    total_value: 1,
                    candidates: Box::new([CurrencyWarsRewardPoolCandidate {
                        reward_id: 2,
                        maximum: 1,
                        weight: 1,
                    }]),
                },
                CurrencyWarsRewardPool {
                    id: 3,
                    stable_key: "reward-pool.3".into(),
                    total_value: 1,
                    candidates: Box::new([CurrencyWarsRewardPoolCandidate {
                        reward_id: 3,
                        maximum: 1,
                        weight: 1,
                    }]),
                },
            ],
            recipes: vec![CurrencyWarsEquipmentRecipe {
                id: 1,
                stable_key: "recipe.1".into(),
                season_id: 1,
                output: CurrencyWarsEquipmentId::new(1).unwrap(),
                inputs: Box::new([
                    CurrencyWarsEquipmentId::new(1).unwrap(),
                    CurrencyWarsEquipmentId::new(1).unwrap(),
                ]),
            }],
            upgrades: vec![CurrencyWarsEquipmentUpgrade {
                source: CurrencyWarsEquipmentId::new(1).unwrap(),
                output: CurrencyWarsEquipmentId::new(2).unwrap(),
            }],
            forge_services: vec![CurrencyWarsForgeService {
                item: CurrencyWarsItemId::new(99_999).unwrap(),
                stable_key: "forge.99999".into(),
                category: CurrencyWarsEquipmentCategory::Basic,
                offer_count: 1,
                target: CurrencyWarsForgeTarget::Equipment,
            }],
            constants: [
                ("fixture", 1),
                ("GridFight_OCSeasonExpRatio", 60),
                ("GridFight_OCSeasonWeeklyScoreRatio", 100),
                ("GridFight_OCTalentPointRatio", 60),
                ("GridFight_SpecialAvatarWorldLevel", 6),
            ]
            .into_iter()
            .map(|(name, value)| CurrencyWarsServiceConstant {
                name: name.into(),
                value: CurrencyWarsServiceConstantValue::Integer(value),
            })
            .collect(),
            gamble_group_count: 0,
            gamble_unit_count: 0,
            curse_chest_count: 0,
            hex_state_count: 0,
            hex_eligibility_count: 0,
            curio_count: 0,
            curio_group_count: 0,
            curio_state_count: 0,
            curio_lifecycle_count: 0,
        })
        .unwrap(),
        augments: CurrencyWarsAugmentCatalog::new(
            (1..=5)
                .map(|raw| CurrencyWarsAugmentDefinition {
                    investment: CurrencyWarsInvestmentId::new(u64::from(raw)).unwrap(),
                    source_id: raw,
                    stable_key: format!("augment.{raw}").into(),
                    category_id: u16::try_from(raw).unwrap(),
                    quality: CurrencyWarsAugmentQuality::Silver,
                    chapter_limits: Box::new([]),
                    season_ids: Box::new([1]),
                    effect_ids: Box::new([format!("augment:{raw}").into()]),
                    config_path: format!("fixture/augment/{raw}").into(),
                    lifecycle: CurrencyWarsAugmentLifecycle {
                        saved_values: Box::new([]),
                        overclock_effective: true,
                        effect_parameters: Box::new([]),
                        description_parameters: Box::new([]),
                    },
                    remark: None,
                    banned_module_ids: Box::new([]),
                })
                .collect(),
            vec![
                CurrencyWarsSelectedEnhancement {
                    id: CurrencyWarsSelectedEnhancementId::new(1).unwrap(),
                    stable_key: "selected-enhancement.1".into(),
                    trait_effect_id: 30_021,
                    gold_cost: Some(5),
                    condition: CurrencyWarsEnhancementSelectCondition::Always,
                    parameters: Box::new([]),
                    effects: Box::new([]),
                    effect_ids: Box::new(["trait-effect:30021".into()]),
                },
                CurrencyWarsSelectedEnhancement {
                    id: CurrencyWarsSelectedEnhancementId::new(2).unwrap(),
                    stable_key: "selected-enhancement.2".into(),
                    trait_effect_id: 30_021,
                    gold_cost: None,
                    condition: CurrencyWarsEnhancementSelectCondition::MaximumStar,
                    parameters: Box::new([]),
                    effects: Box::new([]),
                    effect_ids: Box::new(["trait-effect:30021".into()]),
                },
            ],
            vec![CurrencyWarsEnhancement {
                investment: CurrencyWarsInvestmentId::new(2_000_001).unwrap(),
                id: CurrencyWarsSelectedEnhancementId::new(1).unwrap(),
                stable_key: "enhancement.1".into(),
                trait_effect_id: 30_021,
                gold_cost: Some(5),
                condition: CurrencyWarsEnhancementSelectCondition::Always,
                effects: Box::new([]),
                effect_ids: Box::new(["enhancement-group:30021".into()]),
            }],
            vec![],
            vec![],
        )
        .unwrap(),
        cross_investments: fixture_cross_investments(role),
        progression: CurrencyWarsProgressionCatalog::new(CurrencyWarsProgressionCatalogParts {
            cost_availability: (1..=5)
                .map(|cost| CurrencyWarsRoleCostAvailability {
                    stable_key: format!("role-cost.{cost}").into(),
                    cost,
                    standard: if cost == 1 {
                        cost_one_standard
                    } else {
                        CurrencyWarsRunPosition::new(1, 1).unwrap()
                    },
                    overclock: CurrencyWarsRunPosition::new(1, 1).unwrap(),
                })
                .collect(),
            season_rules: (1..=2)
                .flat_map(|division| {
                    (1..=2).flat_map(move |score_rule| {
                        (1..=3).flat_map(move |chapter| {
                            (1..=2).map(move |section| CurrencyWarsSeasonProgressionRule {
                                stable_key: format!(
                                    "season-progress.{division}.{score_rule}.{chapter}.{section}"
                                )
                                .into(),
                                division,
                                score_rule,
                                position: CurrencyWarsRunPosition::new(chapter, section).unwrap(),
                                weekly_score: Some(10),
                                experience: Some(20),
                            })
                        })
                    })
                })
                .collect(),
            module_role_bans: vec![CurrencyWarsModuleRoleBan {
                stable_key: "module-role-ban.2.1001".into(),
                module: 2,
                role,
            }],
            season_role_pools: vec![CurrencyWarsSeasonRolePool {
                stable_key: "season-role-pool.1".into(),
                season: 1,
                roles: Box::new([role]),
            }],
            season_trait_role_pools: vec![CurrencyWarsSeasonTraitRolePool {
                stable_key: "season-trait-role-pool.1.1001".into(),
                season: 1,
                trait_id: 1001,
                roles: Box::new([role]),
            }],
            role_reference_scores: vec![CurrencyWarsRoleReferenceScore {
                stable_key: "role-reference-score.1.1001".into(),
                season: 1,
                role,
                score: 3,
            }],
        })
        .unwrap(),
        role_overrides: CurrencyWarsRoleOverrideCatalog::test_fixture(role),
        investments: (1..=5)
            .map(|raw| CurrencyWarsInvestment {
                id: CurrencyWarsInvestmentId::new(raw).unwrap(),
                stable_key: format!("augment.{raw}").into(),
                kind: CurrencyWarsInvestmentKind::Augment,
                effect_ids: Box::new([]),
                source_id: raw.to_string().into(),
                references: Box::new([]),
                attributes_json: "[]".into(),
                runtime_binding_exact: true,
            })
            .chain(fixture_investments())
            .collect(),
        policies: vec![],
        front_cap: 4,
        back_cap: 9,
    })
    .unwrap()
}

fn fixture_cross_investments(role: CurrencyWarsRoleId) -> CurrencyWarsCrossInvestmentCatalog {
    CurrencyWarsCrossInvestmentCatalog::new(
        vec![CurrencyWarsPortalDefinition {
            investment: CurrencyWarsInvestmentId::new(4_000_001).unwrap(),
            source_id: 1001,
            stable_key: "portal.1001".into(),
            season_ids: Box::new([1]),
            config_path: "fixture/portal/1001".into(),
            effect_ids: Box::new([]),
            bonus_ids: Box::new([40_001]),
            overclock_effective: true,
            in_index: true,
            delayed_bonus_ids: Box::new([]),
            effect_parameters: Box::new([]),
            npc_ids: Box::new([]),
            remark: None,
            banned_module_ids: Box::new([]),
            maze_buffs: Box::new([]),
        }],
        vec![CurrencyWarsOrbDefinition {
            investment: CurrencyWarsInvestmentId::new(3_000_001).unwrap(),
            source_id: "100.20001.0".into(),
            stable_key: "orb.100.20001.0".into(),
            bonus_id: 20_001,
            orb_type: CurrencyWarsOrbType::White,
            effect_ids: Box::new(["bonus:20001".into()]),
            display: CurrencyWarsOrbDisplay {
                orb_type: CurrencyWarsOrbType::White,
                icon_path: "fixture/orb.png".into(),
                prefab_path: "".into(),
            },
        }],
        vec![CurrencyWarsProjectionDefinition {
            investment: CurrencyWarsInvestmentId::new(5_000_001).unwrap(),
            source_id: role.get(),
            stable_key: "projection.1001".into(),
            role,
            unlock_type: "SpecialGoods".into(),
            trait_ids: Box::new([]),
            effect_ids: Box::new([]),
            maze_buffs: Box::new([]),
        }],
        vec![
            fixture_talent(
                6_000_001,
                1011,
                CurrencyWarsTalentKind::Permanent,
                Box::new([]),
            ),
            fixture_talent(
                6_000_002,
                1021,
                CurrencyWarsTalentKind::Permanent,
                Box::new([1011]),
            ),
            fixture_talent(0, 2011, CurrencyWarsTalentKind::Season, Box::new([])),
            fixture_talent(0, 2021, CurrencyWarsTalentKind::Season, Box::new([2011])),
        ],
        vec![],
    )
    .unwrap()
}

fn fixture_talent(
    investment: u64,
    source_id: u32,
    kind: CurrencyWarsTalentKind,
    prerequisites: Box<[u32]>,
) -> CurrencyWarsTalentDefinition {
    CurrencyWarsTalentDefinition {
        investment: CurrencyWarsInvestmentId::new(investment),
        source_id,
        stable_key: format!("talent.{source_id}").into(),
        kind,
        season_id: (kind == CurrencyWarsTalentKind::Season).then_some(1),
        cost: 20,
        prerequisites,
        successors: Box::new([]),
        effect_ids: Box::new([]),
        config_path: format!("fixture/talent/{source_id}").into(),
        maze_buffs: Box::new([]),
    }
}

fn fixture_investments() -> impl Iterator<Item = CurrencyWarsInvestment> {
    [
        (2_000_001, CurrencyWarsInvestmentKind::Enhancement),
        (3_000_001, CurrencyWarsInvestmentKind::Orb),
        (4_000_001, CurrencyWarsInvestmentKind::Portal),
        (5_000_001, CurrencyWarsInvestmentKind::Projection),
        (6_000_001, CurrencyWarsInvestmentKind::Talent),
        (6_000_002, CurrencyWarsInvestmentKind::Talent),
    ]
    .into_iter()
    .map(|(raw, kind)| CurrencyWarsInvestment {
        id: CurrencyWarsInvestmentId::new(raw).unwrap(),
        stable_key: format!("investment.{raw}").into(),
        kind,
        effect_ids: Box::new([]),
        source_id: raw.to_string().into(),
        references: Box::new([]),
        attributes_json: "[]".into(),
        runtime_binding_exact: true,
    })
}

fn test_node(
    id: CurrencyWarsNodeId,
    plane: u8,
    ordinal: u8,
    kind: CurrencyWarsNodeKind,
    next: Option<CurrencyWarsNodeId>,
) -> CurrencyWarsNode {
    let kind_name = match kind {
        CurrencyWarsNodeKind::Monster => "monster",
        CurrencyWarsNodeKind::Supply => "supply",
        _ => unreachable!("Currency Wars test fixture uses Monster and Supply only"),
    };
    CurrencyWarsNode {
        id,
        stable_key: format!("node.{}", id.get()).into(),
        plane,
        ordinal,
        kind,
        layer_id: format!("layer.100.{plane}").into(),
        domain_composition_id: format!("domain.{kind_name}").into(),
        room_id: format!("room.{kind_name}").into(),
        node_template_id: 10_000 + id.get(),
        encounter: EncounterId::new(70_000_000 + id.get()).unwrap(),
        parameter_ids: Box::new([]),
        penalty_bonus_rule_id: kind.battle().then_some(90_301),
        basic_gold_reward: kind.battle().then_some(3),
        next,
    }
}

fn test_content_records() -> Vec<CurrencyWarsContentRecord> {
    (0..1)
        .map(|index| CurrencyWarsContentRecord {
            stable_key: format!("content.fixture.{index}").into(),
            source_id: Some(index.to_string().into()),
            kind: CurrencyWarsContentKind::AugmentMonsterRule,
            references: Box::new([]),
            effect_ids: Box::new([]),
            attributes_json: "[]".into(),
        })
        .collect()
}
