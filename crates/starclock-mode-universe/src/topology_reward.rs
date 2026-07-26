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

pub(crate) struct CompiledBlessingReward {
    pub(crate) options: Vec<ActivityOptionDefinition>,
    pub(crate) weights: Vec<(ActivityOptionId, u64)>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_blessing_reward(
    source: u64,
    reward_formation: ActivityEdgeId,
    hub_clear_slot: ActivitySlotId,
    path_blessing_count_slot: ActivitySlotId,
    ability_projection_slot: ActivitySlotId,
    curio_bindings: CurioActivityBindings,
    blessing_inventory: ActivityInventoryId,
    eligible_blessings: &[&BlessingRuntimeDefinition],
) -> Result<CompiledBlessingReward, UniverseTopologyCompileError> {
    let first_battle_bonus = ActivityExpression::CounterValue {
        slot: ability_projection_slot,
        key: AbilityTarget::FirstBattleBlessingCount.activity_key(),
    };
    let bonus_available = ActivityCondition::LessThan(
        ActivityExpression::Literal(ActivityValue::BoundedInteger(0)),
        first_battle_bonus,
    );
    let mut options = Vec::with_capacity(eligible_blessings.len());
    let mut weights = Vec::with_capacity(eligible_blessings.len());
    for (priority, blessing) in eligible_blessings.iter().enumerate() {
        let id = blessing_option(source, blessing.blessing());
        let ordinary_finish = vec![ActivityOperation::Conditional {
            condition: bonus_available.clone(),
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
        }];
        let mut settlement = vec![ActivityOperation::AddCounter {
            slot: path_blessing_count_slot,
            key: u64::from(blessing.path().get()),
            delta: integer(1),
        }];
        settlement.extend(dimension_reward_settlement(curio_bindings, ordinary_finish));
        let content = u64::from(blessing.blessing().get());
        let acquisition_count = if blessing.rarity() == 1 {
            ActivityExpression::Add(
                Box::new(integer(1)),
                Box::new(ActivityExpression::InventoryCount {
                    inventory: curio_bindings.inventory,
                    content: 35,
                }),
            )
        } else {
            integer(1)
        };
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
            ActivityCondition::Equal(
                ActivityExpression::InventoryCount {
                    inventory: blessing_inventory,
                    content,
                },
                integer(0),
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
