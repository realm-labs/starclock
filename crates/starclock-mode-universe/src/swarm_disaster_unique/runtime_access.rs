use super::SwarmDisasterUniqueCatalog;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmCountdownRuntimeInput {
    pub(crate) initial: Box<str>,
    pub(crate) warning: Box<str>,
    pub(crate) movement_delta: Box<str>,
    pub(crate) tiers: Box<str>,
    pub(crate) source_constants: Box<str>,
    pub(crate) boss_decay: Box<[SwarmBossDecayRuntimeInput]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmBossDecayRuntimeInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) threshold: Box<str>,
    pub(crate) effect_program: Box<str>,
    pub(crate) enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmAudienceRuntimeInput {
    pub(crate) paths: Box<[SwarmAudiencePathRuntimeInput]>,
    pub(crate) dice: Box<[SwarmAudienceDieRuntimeInput]>,
    pub(crate) rarities: Box<[SwarmDiceRarityRuntimeInput]>,
    pub(crate) faces: Box<[SwarmAudienceFaceRuntimeInput]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmAudiencePathRuntimeInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) source_id: Box<str>,
    pub(crate) die_id: u32,
    pub(crate) shared_path: Box<str>,
    pub(crate) sort: u16,
    pub(crate) unlock_id: Option<Box<str>>,
    pub(crate) unlock_policy: Box<str>,
    pub(crate) initial_program: Box<str>,
    pub(crate) passive_program: Box<str>,
    pub(crate) description_parameters: Box<[Box<str>]>,
    pub(crate) rogue_buff_type: Box<str>,
    pub(crate) battle_event_buff_group: Box<str>,
    pub(crate) battle_event_enhance_buff_group: Box<str>,
    pub(crate) extra_effect_refs: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmAudienceDieRuntimeInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) source_id: Box<str>,
    pub(crate) path_id: u32,
    pub(crate) shared_path: Box<str>,
    pub(crate) face_keys: Box<[Box<str>]>,
    pub(crate) roll_policy: Box<str>,
    pub(crate) unlock_id: Option<Box<str>>,
    pub(crate) initial_effect_parameters: Box<[Box<str>]>,
    pub(crate) passive_description_parameters: Box<[Box<str>]>,
    pub(crate) extra_effect_refs: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDiceRarityRuntimeInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) rank: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmAudienceFaceRuntimeInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) source_id: Box<str>,
    pub(crate) die_id: u32,
    pub(crate) rarity_id: u32,
    pub(crate) target_id: u32,
    pub(crate) sort: u16,
    pub(crate) activation_stage: u8,
    pub(crate) effect_program: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDiceTargetRuntimeInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) source_id: Box<str>,
    pub(crate) candidate_filter: Box<str>,
    pub(crate) ordering: Box<str>,
    pub(crate) cardinality: Box<str>,
    pub(crate) no_legal_target: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmCommuningRuntimeInput {
    pub(crate) choices: Box<[SwarmCommuningChoiceRuntimeInput]>,
    pub(crate) dimensions: Box<[SwarmCommuningDimensionRuntimeInput]>,
    pub(crate) adjustments: Box<[SwarmCommuningAdjustmentRuntimeInput]>,
    pub(crate) cabinets: Box<[SwarmCabinetRuntimeInput]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmCommuningChoiceRuntimeInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) source_id: Box<str>,
    pub(crate) aeon_id: Box<str>,
    pub(crate) shared_path: Box<str>,
    pub(crate) story_stage: u16,
    pub(crate) eligibility: Box<str>,
    pub(crate) point_deltas: Box<str>,
    pub(crate) operations: Box<str>,
    pub(crate) rogue_npc_id: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmCommuningDimensionRuntimeInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) shared_path: Box<str>,
    pub(crate) maximum: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmCommuningAdjustmentRuntimeInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) dimension_id: u32,
    pub(crate) source_id: Box<str>,
    pub(crate) source_kind: Box<str>,
    pub(crate) ordinal: u16,
    pub(crate) delta: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmCabinetRuntimeInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) source_id: Box<str>,
    pub(crate) sort: u16,
    pub(crate) cabinet_type: Box<str>,
    pub(crate) objective_id: Box<str>,
    pub(crate) prerequisite_keys: Box<[Box<str>]>,
    pub(crate) unlock_keys: Box<[Box<str>]>,
    pub(crate) point_deltas: Box<str>,
    pub(crate) description_parameters: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDiceControlRuntimeInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) operation: Box<str>,
    pub(crate) resource_cost: Box<str>,
    pub(crate) result_order: Box<str>,
    pub(crate) fallback_policy: Box<str>,
    pub(crate) abandon_reward: Box<str>,
    pub(crate) unlock_id: Option<Box<str>>,
}

impl SwarmDisasterUniqueCatalog {
    pub(crate) fn countdown_runtime_input(&self) -> Option<SwarmCountdownRuntimeInput> {
        let countdown = self.countdown.first()?;
        Some(SwarmCountdownRuntimeInput {
            initial: countdown.initial.clone(),
            warning: countdown.warning.clone(),
            movement_delta: countdown.movement_delta.clone(),
            tiers: countdown.tiers.clone(),
            source_constants: countdown.source_constants.clone(),
            boss_decay: self
                .boss_decay_levels
                .iter()
                .map(|row| SwarmBossDecayRuntimeInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    threshold: row.threshold.clone(),
                    effect_program: row.effect_program.clone(),
                    enabled: row.enabled,
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        })
    }

    pub(crate) fn audience_runtime_input(&self) -> SwarmAudienceRuntimeInput {
        SwarmAudienceRuntimeInput {
            paths: self
                .audience_paths
                .iter()
                .map(|row| SwarmAudiencePathRuntimeInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    source_id: row.source_id.clone(),
                    die_id: row.audience_die.0,
                    shared_path: row.shared_path.clone(),
                    sort: row.sort,
                    unlock_id: row.unlock_id.clone(),
                    unlock_policy: row.unlock_policy.clone(),
                    initial_program: row.initial_program.clone(),
                    passive_program: row.passive_program.clone(),
                    description_parameters: row.description_parameters.clone(),
                    rogue_buff_type: row.rogue_buff_type.clone(),
                    battle_event_buff_group: row.battle_event_buff_group.clone(),
                    battle_event_enhance_buff_group: row.battle_event_enhance_buff_group.clone(),
                    extra_effect_refs: row.extra_effect_refs.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            dice: self
                .audience_dice
                .iter()
                .map(|row| SwarmAudienceDieRuntimeInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    source_id: row.source_id.clone(),
                    path_id: row.audience_path.0,
                    shared_path: row.shared_path.clone(),
                    face_keys: row.face_keys.clone(),
                    roll_policy: row.roll_policy.clone(),
                    unlock_id: row.unlock_id.clone(),
                    initial_effect_parameters: row.initial_effect_parameters.clone(),
                    passive_description_parameters: row.passive_description_parameters.clone(),
                    extra_effect_refs: row.extra_effect_refs.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            rarities: self
                .dice_rarities
                .iter()
                .map(|row| SwarmDiceRarityRuntimeInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    rank: row.rank,
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            faces: self
                .dice_faces
                .iter()
                .map(|row| SwarmAudienceFaceRuntimeInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    source_id: row.source_id.clone(),
                    die_id: row.audience_die.0,
                    rarity_id: row.rarity.0,
                    target_id: row.target.0,
                    sort: row.sort,
                    activation_stage: row.activation_stage,
                    effect_program: row.effect_program.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    pub(crate) fn dice_target_runtime_input(&self) -> Box<[SwarmDiceTargetRuntimeInput]> {
        self.dice_targets
            .iter()
            .map(|row| SwarmDiceTargetRuntimeInput {
                id: row.id.0,
                key: row.key.clone(),
                source_id: row.source_id.clone(),
                candidate_filter: row.candidate_filter.clone(),
                ordering: row.ordering.clone(),
                cardinality: row.cardinality.clone(),
                no_legal_target: row.no_legal_target.clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    pub(crate) fn communing_runtime_input(&self) -> SwarmCommuningRuntimeInput {
        SwarmCommuningRuntimeInput {
            choices: self
                .communing_choices
                .iter()
                .map(|row| SwarmCommuningChoiceRuntimeInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    source_id: row.source_id.clone(),
                    aeon_id: row.aeon_id.clone(),
                    shared_path: row.shared_path.clone(),
                    story_stage: row.story_stage,
                    eligibility: row.eligibility.clone(),
                    point_deltas: row.point_deltas.clone(),
                    operations: row.operations.clone(),
                    rogue_npc_id: row.rogue_npc_id.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            dimensions: self
                .communing_dimensions
                .iter()
                .map(|row| SwarmCommuningDimensionRuntimeInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    shared_path: row.shared_path.clone(),
                    maximum: row.maximum,
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            adjustments: self
                .point_adjustments
                .iter()
                .map(|row| SwarmCommuningAdjustmentRuntimeInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    dimension_id: row.dimension.0,
                    source_id: row.source_id.clone(),
                    source_kind: row.source_kind.clone(),
                    ordinal: row.ordinal,
                    delta: row.delta.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            cabinets: self
                .cabinets
                .iter()
                .map(|row| SwarmCabinetRuntimeInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    source_id: row.source_id.clone(),
                    sort: row.sort,
                    cabinet_type: row.cabinet_type.clone(),
                    objective_id: row.objective_id.clone(),
                    prerequisite_keys: row.prerequisite_keys.clone(),
                    unlock_keys: row.unlock_keys.clone(),
                    point_deltas: row.point_deltas.clone(),
                    description_parameters: row.description_parameters.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    pub(crate) fn dice_control_runtime_input(&self) -> Box<[SwarmDiceControlRuntimeInput]> {
        self.dice_controls
            .iter()
            .map(|row| SwarmDiceControlRuntimeInput {
                id: row.id.0,
                key: row.key.clone(),
                operation: row.operation.clone(),
                resource_cost: row.resource_cost.clone(),
                result_order: row.result_order.clone(),
                fallback_policy: row.fallback_policy.clone(),
                abandon_reward: row.abandon_reward.clone(),
                unlock_id: row.unlock_id.clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}
