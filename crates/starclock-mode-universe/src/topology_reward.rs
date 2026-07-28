//! Standard Universe Blessing reward program compilation.

use starclock_activity::{
    ActivityCondition, ActivityEdgeId, ActivityExpression, ActivityInventoryId, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivitySlotId, ActivityValue,
};

use crate::{
    ability_runtime::AbilityTarget,
    blessing_runtime::BlessingRuntimeDefinition,
    curio_activity::{CurioActivityBindings, dimension_reward_settlement},
    topology::UniverseTopologyCompileError,
    topology_identity::blessing_option,
};

const WARPING_COMPOUND_EYE_CURIO: u64 = 35;

pub(crate) struct CompiledBlessingReward {
    pub(crate) options: Vec<ActivityOptionDefinition>,
    pub(crate) weights: Vec<(ActivityOptionId, u64)>,
}

#[derive(Clone, Copy)]
pub(crate) enum BlessingRewardCompletion {
    Inline {
        reward_formation: ActivityEdgeId,
        hub_clear_slot: ActivitySlotId,
        ability_projection_slot: ActivitySlotId,
    },
    Resolution(ActivityEdgeId),
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_blessing_reward(
    source: u64,
    completion: BlessingRewardCompletion,
    path_blessing_count_slot: ActivitySlotId,
    blessing_offer_marker_slot: ActivitySlotId,
    curio_bindings: CurioActivityBindings,
    blessing_inventory: ActivityInventoryId,
    battle_reward_path_slot: ActivitySlotId,
    eligible_blessings: &[&BlessingRuntimeDefinition],
) -> Result<CompiledBlessingReward, UniverseTopologyCompileError> {
    let path_ids = eligible_blessings
        .iter()
        .map(|blessing| blessing.path())
        .collect::<std::collections::BTreeSet<_>>();
    let path_reward_constrained = ActivityCondition::Any(
        path_ids
            .iter()
            .map(|path| {
                ActivityCondition::LessThan(
                    integer(0),
                    ActivityExpression::CounterValue {
                        slot: battle_reward_path_slot,
                        key: u64::from(path.get()),
                    },
                )
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    let mut options = Vec::with_capacity(eligible_blessings.len());
    let mut weights = Vec::with_capacity(eligible_blessings.len());
    for (priority, blessing) in eligible_blessings.iter().enumerate() {
        let id = blessing_option(source, blessing.blessing());
        let ordinary_finish = match completion {
            BlessingRewardCompletion::Inline {
                reward_formation,
                hub_clear_slot,
                ability_projection_slot,
            } => {
                let bonus_available = ActivityCondition::LessThan(
                    integer(0),
                    ActivityExpression::CounterValue {
                        slot: ability_projection_slot,
                        key: AbilityTarget::FirstBattleBlessingCount.activity_key(),
                    },
                );
                vec![ActivityOperation::Conditional {
                    condition: bonus_available,
                    if_true: vec![ActivityOperation::AddCounter {
                        slot: ability_projection_slot,
                        key: AbilityTarget::FirstBattleBlessingCount.activity_key(),
                        delta: integer(-1_000_000),
                    }]
                    .into_boxed_slice(),
                    if_false: vec![
                        ActivityOperation::AddCounter {
                            slot: hub_clear_slot,
                            key: source,
                            delta: integer(1),
                        },
                        ActivityOperation::Traverse(reward_formation),
                    ]
                    .into_boxed_slice(),
                }]
            }
            BlessingRewardCompletion::Resolution(edge) => {
                vec![ActivityOperation::Traverse(edge)]
            }
        };
        let mut settlement = vec![ActivityOperation::AddCounter {
            slot: path_blessing_count_slot,
            key: u64::from(blessing.path().get()),
            delta: integer(1),
        }];
        settlement.insert(
            0,
            ActivityOperation::AddCounter {
                slot: battle_reward_path_slot,
                key: u64::from(blessing.path().get()),
                delta: ActivityExpression::Negate(Box::new(ActivityExpression::Minimum(
                    Box::new(ActivityExpression::CounterValue {
                        slot: battle_reward_path_slot,
                        key: u64::from(blessing.path().get()),
                    }),
                    Box::new(integer(1)),
                ))),
            },
        );
        settlement.extend(dimension_reward_settlement(curio_bindings, ordinary_finish));
        let content = u64::from(blessing.blessing().get());
        let one_star_bonus = if blessing.rarity() == 1 {
            ActivityExpression::InventoryCount {
                inventory: curio_bindings.inventory,
                content: WARPING_COMPOUND_EYE_CURIO,
            }
        } else {
            integer(0)
        };
        let acquisition_count = ActivityExpression::Minimum(
            Box::new(ActivityExpression::Add(
                Box::new(integer(1)),
                Box::new(ActivityExpression::Add(
                    Box::new(one_star_bonus),
                    Box::new(ActivityExpression::CounterValue {
                        slot: blessing_offer_marker_slot,
                        key: id.get(),
                    }),
                )),
            )),
            Box::new(integer(2)),
        );
        settlement.insert(
            0,
            ActivityOperation::AddInventory {
                inventory: blessing_inventory,
                content,
                count: acquisition_count,
            },
        );
        options.push(ActivityOptionDefinition::new(
            id,
            priority as i32,
            ActivityCondition::All(
                vec![
                    ActivityCondition::Equal(
                        ActivityExpression::InventoryCount {
                            inventory: blessing_inventory,
                            content,
                        },
                        integer(0),
                    ),
                    ActivityCondition::Any(
                        vec![
                            ActivityCondition::Not(Box::new(path_reward_constrained.clone())),
                            ActivityCondition::LessThan(
                                integer(0),
                                ActivityExpression::CounterValue {
                                    slot: battle_reward_path_slot,
                                    key: u64::from(blessing.path().get()),
                                },
                            ),
                        ]
                        .into_boxed_slice(),
                    ),
                ]
                .into_boxed_slice(),
            ),
            settlement,
        ));
        weights.push((id, 1));
    }
    Ok(CompiledBlessingReward { options, weights })
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
