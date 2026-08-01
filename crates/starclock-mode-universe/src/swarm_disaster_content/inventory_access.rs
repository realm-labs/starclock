//! Narrow immutable input boundary for the Swarm content runtime.

use super::SwarmDisasterContentCatalog;

#[derive(Clone, Debug)]
pub(crate) struct InventoryRuntimeInput {
    pub(crate) blessings: Box<[BlessingInput]>,
    pub(crate) blessing_levels: Box<[BlessingLevelInput]>,
    pub(crate) pool_memberships: Box<[PoolMembershipInput]>,
    pub(crate) curios: Box<[CurioInput]>,
    pub(crate) curio_states: Box<[CurioStateInput]>,
    pub(crate) curio_rules: Box<[CurioRuleInput]>,
}

#[derive(Clone, Debug)]
pub(crate) struct BlessingInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) shared_key: Box<str>,
    pub(crate) path_key: Box<str>,
    pub(crate) rarity: u8,
    pub(crate) level_keys: Box<[Box<str>]>,
}

#[derive(Clone, Debug)]
pub(crate) struct BlessingLevelInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) blessing: u32,
    pub(crate) shared_blessing_key: Box<str>,
    pub(crate) shared_level_key: Box<str>,
    pub(crate) level: u8,
    pub(crate) parameters: Box<[Box<str>]>,
    pub(crate) effect_program: Box<str>,
}

#[derive(Clone, Debug)]
pub(crate) struct PoolMembershipInput {
    pub(crate) id: u32,
    pub(crate) pool_key: Box<str>,
    pub(crate) member_kind: Box<str>,
    pub(crate) member_key: Box<str>,
    pub(crate) eligibility: Box<str>,
    pub(crate) weight_policy: Box<str>,
}

#[derive(Clone, Debug)]
pub(crate) struct CurioInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) mode_copy_key: Box<str>,
    pub(crate) pool_category: Box<str>,
    pub(crate) pool_rules: Box<str>,
    pub(crate) initial_state: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct CurioStateInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) curio: u32,
    pub(crate) state: Box<str>,
    pub(crate) charges: Option<Box<str>>,
    pub(crate) effect_program: Box<str>,
    pub(crate) lifecycle: Box<str>,
    pub(crate) repair_target: Box<str>,
}

#[derive(Clone, Debug)]
pub(crate) struct CurioRuleInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) curio: u32,
    pub(crate) state: u32,
    pub(crate) trigger_phase: Box<str>,
    pub(crate) trigger: Box<str>,
    pub(crate) lifecycle: Box<str>,
    pub(crate) replacement_policy: Box<str>,
}

impl SwarmDisasterContentCatalog {
    pub(crate) fn inventory_runtime_input(&self) -> InventoryRuntimeInput {
        let blessings = self
            .blessings
            .iter()
            .map(|row| BlessingInput {
                id: row.id.0,
                key: row.key.clone(),
                shared_key: row.shared_key.clone(),
                path_key: row.path_key.clone(),
                rarity: row.rarity,
                level_keys: row.level_keys.clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let blessing_levels = self
            .blessing_levels
            .iter()
            .map(|row| BlessingLevelInput {
                id: row.id.0,
                key: row.key.clone(),
                blessing: row.blessing.0,
                shared_blessing_key: row.shared_blessing_key.clone(),
                shared_level_key: row.shared_level_key.clone(),
                level: row.level,
                parameters: row.parameters.clone(),
                effect_program: row.effect_program.clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let pool_memberships = self
            .pool_memberships
            .iter()
            .map(|row| PoolMembershipInput {
                id: row.id.0,
                pool_key: row.pool_key.clone(),
                member_kind: row.member_kind.clone(),
                member_key: row.member_key.clone(),
                eligibility: row.eligibility.clone(),
                weight_policy: row.weight_policy.clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let curios = self
            .curios
            .iter()
            .map(|row| CurioInput {
                id: row.id.0,
                key: row.key.clone(),
                mode_copy_key: row.mode_copy_key.clone(),
                pool_category: row.pool_category.clone(),
                pool_rules: row.pool_rules.clone(),
                initial_state: row.initial_state.0,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let curio_states = self
            .curio_states
            .iter()
            .map(|row| CurioStateInput {
                id: row.id.0,
                key: row.key.clone(),
                curio: row.curio.0,
                state: row.state.clone(),
                charges: row.charges.clone(),
                effect_program: row.effect_program.clone(),
                lifecycle: row.lifecycle.clone(),
                repair_target: row.repair_target.clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let curio_rules = self
            .curio_rules
            .iter()
            .map(|row| CurioRuleInput {
                id: row.id.0,
                key: row.key.clone(),
                curio: row.curio.0,
                state: row.state.0,
                trigger_phase: row.trigger_phase.clone(),
                trigger: row.trigger.clone(),
                lifecycle: row.lifecycle.clone(),
                replacement_policy: row.replacement_policy.clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        InventoryRuntimeInput {
            blessings,
            blessing_levels,
            pool_memberships,
            curios,
            curio_states,
            curio_rules,
        }
    }
}
