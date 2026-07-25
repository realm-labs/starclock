//! Standard Universe Blessing reward program compilation.

use starclock_activity::{
    ActivityCondition, ActivityEdgeId, ActivityExpression, ActivityInventoryId, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivitySlotId, ActivityValue,
};

use crate::{
    ability_runtime::AbilityTarget,
    blessing_runtime::{BlessingRuntimeCatalog, BlessingRuntimeDefinition},
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
    selected_path_slot: ActivitySlotId,
    path_blessing_count_slot: ActivitySlotId,
    ability_projection_slot: ActivitySlotId,
    third_formation_capability_slot: ActivitySlotId,
    blessing_inventory: ActivityInventoryId,
    blessing_runtime: &BlessingRuntimeCatalog,
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
        let ordinary_settlement = ActivityOperation::Conditional {
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
        };
        let formation_unlock_due = ActivityCondition::All(
            vec![
                ActivityCondition::Boolean(ActivityExpression::Slot(
                    third_formation_capability_slot,
                )),
                ActivityCondition::Equal(
                    ActivityExpression::Slot(selected_path_slot),
                    ActivityExpression::Literal(ActivityValue::OptionalId(Some(u64::from(
                        blessing.path().get(),
                    )))),
                ),
                ActivityCondition::Not(Box::new(ActivityCondition::LessThan(
                    ActivityExpression::CounterValue {
                        slot: path_blessing_count_slot,
                        key: u64::from(blessing.path().get()),
                    },
                    integer(14),
                ))),
                ActivityCondition::Equal(
                    ActivityExpression::CounterValue {
                        slot: ability_projection_slot,
                        key: AbilityTarget::RunPathResonance.activity_key(),
                    },
                    integer(0),
                ),
            ]
            .into_boxed_slice(),
        );
        let settlement = vec![
            ActivityOperation::AddCounter {
                slot: path_blessing_count_slot,
                key: u64::from(blessing.path().get()),
                delta: integer(1),
            },
            ActivityOperation::Conditional {
                condition: formation_unlock_due,
                if_true: vec![
                    ActivityOperation::AddCounter {
                        slot: ability_projection_slot,
                        key: AbilityTarget::RunPathResonance.activity_key(),
                        delta: integer(1_000_000),
                    },
                    ordinary_settlement.clone(),
                ]
                .into_boxed_slice(),
                if_false: vec![ordinary_settlement].into_boxed_slice(),
            },
        ];
        options.push(
            blessing_runtime
                .acquisition_option(
                    blessing.blessing(),
                    id,
                    priority as i32,
                    blessing_inventory,
                    settlement,
                )
                .ok_or(UniverseTopologyCompileError::InvalidBlessingRuntime)?,
        );
        weights.push((id, 1));
    }
    Ok(CompiledBlessingReward { options, weights })
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
