macro_rules! unique_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub(super) struct $name(pub(super) u32);
    };
}

unique_id!(CountdownId);
unique_id!(BossDecayId);
unique_id!(AudiencePathId);
unique_id!(AudienceDieId);
unique_id!(DiceRarityId);
unique_id!(DiceFaceId);
unique_id!(DiceTargetId);
unique_id!(DiceControlId);
unique_id!(CommuningChoiceId);
unique_id!(CommuningDimensionId);
unique_id!(PointAdjustmentId);
unique_id!(TrailNodeId);
unique_id!(TrailPrerequisiteId);
unique_id!(TrailEffectId);
unique_id!(CabinetId);
unique_id!(ObjectiveId);
unique_id!(FinishId);
unique_id!(UnlockId);
unique_id!(ChapterId);
unique_id!(BonusId);
unique_id!(PathId);
unique_id!(PathBoostId);
unique_id!(ResonanceId);
unique_id!(InterplayId);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CountdownDefinition {
    pub(super) id: CountdownId,
    pub(super) key: Box<str>,
    pub(super) initial: Box<str>,
    pub(super) warning: Box<str>,
    pub(super) movement_delta: Box<str>,
    pub(super) tiers: Box<str>,
    pub(super) source_constants: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BossDecayDefinition {
    pub(super) id: BossDecayId,
    pub(super) key: Box<str>,
    pub(super) threshold: Box<str>,
    pub(super) tier: Box<str>,
    pub(super) effect_program: Box<str>,
    pub(super) enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct AudiencePathDefinition {
    pub(super) id: AudiencePathId,
    pub(super) key: Box<str>,
    pub(super) source_id: Box<str>,
    pub(super) audience_die: AudienceDieId,
    pub(super) shared_path: Box<str>,
    pub(super) sort: u16,
    pub(super) unlock_id: Option<Box<str>>,
    pub(super) unlock_policy: Box<str>,
    pub(super) initial_program: Box<str>,
    pub(super) passive_program: Box<str>,
    pub(super) description_parameters: Box<[Box<str>]>,
    pub(super) rogue_buff_type: Box<str>,
    pub(super) battle_event_buff_group: Box<str>,
    pub(super) battle_event_enhance_buff_group: Box<str>,
    pub(super) extra_effect_refs: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct AudienceDieDefinition {
    pub(super) id: AudienceDieId,
    pub(super) key: Box<str>,
    pub(super) source_id: Box<str>,
    pub(super) audience_path: AudiencePathId,
    pub(super) shared_path: Box<str>,
    pub(super) face_keys: Box<[Box<str>]>,
    pub(super) roll_policy: Box<str>,
    pub(super) unlock_id: Option<Box<str>>,
    pub(super) initial_effect_parameters: Box<[Box<str>]>,
    pub(super) passive_description_parameters: Box<[Box<str>]>,
    pub(super) extra_effect_refs: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DiceRarityDefinition {
    pub(super) id: DiceRarityId,
    pub(super) key: Box<str>,
    pub(super) rank: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DiceFaceDefinition {
    pub(super) id: DiceFaceId,
    pub(super) key: Box<str>,
    pub(super) source_id: Box<str>,
    pub(super) audience_die: AudienceDieId,
    pub(super) rarity: DiceRarityId,
    pub(super) target: DiceTargetId,
    pub(super) sort: u16,
    pub(super) activation_stage: u8,
    pub(super) effect_program: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DiceTargetDefinition {
    pub(super) id: DiceTargetId,
    pub(super) key: Box<str>,
    pub(super) source_id: Box<str>,
    pub(super) candidate_filter: Box<str>,
    pub(super) ordering: Box<str>,
    pub(super) cardinality: Box<str>,
    pub(super) no_legal_target: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DiceControlDefinition {
    pub(super) id: DiceControlId,
    pub(super) key: Box<str>,
    pub(super) operation: Box<str>,
    pub(super) resource_cost: Box<str>,
    pub(super) result_order: Box<str>,
    pub(super) fallback_policy: Box<str>,
    pub(super) abandon_reward: Box<str>,
    pub(super) unlock_id: Option<Box<str>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CommuningChoiceDefinition {
    pub(super) id: CommuningChoiceId,
    pub(super) key: Box<str>,
    pub(super) source_id: Box<str>,
    pub(super) aeon_id: Box<str>,
    pub(super) shared_path: Box<str>,
    pub(super) story_stage: u16,
    pub(super) eligibility: Box<str>,
    pub(super) point_deltas: Box<str>,
    pub(super) operations: Box<str>,
    pub(super) rogue_npc_id: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CommuningDimensionDefinition {
    pub(super) id: CommuningDimensionId,
    pub(super) key: Box<str>,
    pub(super) shared_path: Box<str>,
    pub(super) maximum: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PointAdjustmentDefinition {
    pub(super) id: PointAdjustmentId,
    pub(super) key: Box<str>,
    pub(super) dimension: CommuningDimensionId,
    pub(super) source_id: Box<str>,
    pub(super) source_kind: Box<str>,
    pub(super) ordinal: u16,
    pub(super) delta: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TrailNodeDefinition {
    pub(super) id: TrailNodeId,
    pub(super) key: Box<str>,
    pub(super) dimension: CommuningDimensionId,
    pub(super) effect_keys: Box<[Box<str>]>,
    pub(super) prerequisite_keys: Box<[Box<str>]>,
    pub(super) threshold: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TrailPrerequisiteDefinition {
    pub(super) id: TrailPrerequisiteId,
    pub(super) key: Box<str>,
    pub(super) node: TrailNodeId,
    pub(super) required_node: TrailNodeId,
    pub(super) ordinal: u16,
    pub(super) required_points: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TrailEffectDefinition {
    pub(super) id: TrailEffectId,
    pub(super) key: Box<str>,
    pub(super) node: TrailNodeId,
    pub(super) ordinal: u16,
    pub(super) domain: Box<str>,
    pub(super) operations: Box<str>,
    pub(super) battle_projection: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CabinetDefinition {
    pub(super) id: CabinetId,
    pub(super) key: Box<str>,
    pub(super) source_id: Box<str>,
    pub(super) sort: u16,
    pub(super) cabinet_type: Box<str>,
    pub(super) objective_id: Box<str>,
    pub(super) prerequisite_keys: Box<[Box<str>]>,
    pub(super) unlock_keys: Box<[Box<str>]>,
    pub(super) point_deltas: Box<str>,
    pub(super) description_parameters: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ObjectiveDefinition {
    pub(super) id: ObjectiveId,
    pub(super) key: Box<str>,
    pub(super) cabinet: CabinetId,
    pub(super) finish_key: Box<str>,
    pub(super) progress_policy: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FinishDefinition {
    pub(super) id: FinishId,
    pub(super) key: Box<str>,
    pub(super) enabled: bool,
    pub(super) finish_type: Box<str>,
    pub(super) comparison: Box<str>,
    pub(super) parameters: Box<str>,
    pub(super) target: Box<str>,
    pub(super) unlock_keys: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct UnlockDefinition {
    pub(super) id: UnlockId,
    pub(super) key: Box<str>,
    pub(super) finish: FinishId,
    pub(super) consequence: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ChapterDefinition {
    pub(super) id: ChapterId,
    pub(super) key: Box<str>,
    pub(super) dimension: Option<CommuningDimensionId>,
    pub(super) layer: u8,
    pub(super) threshold: Option<Box<str>>,
    pub(super) mechanical_unlock: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BonusDefinition {
    pub(super) id: BonusId,
    pub(super) key: Box<str>,
    pub(super) effect_program: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PathDefinition {
    pub(super) id: PathId,
    pub(super) key: Box<str>,
    pub(super) shared_path: Box<str>,
    pub(super) audience_die: AudienceDieId,
    pub(super) resonance: ResonanceId,
    pub(super) sort: u16,
    pub(super) mode_unlock: Option<Box<str>>,
    pub(super) propagation_unlock: Box<str>,
    pub(super) formation_keys: Box<[Box<str>]>,
    pub(super) battle_event_groups: Box<str>,
    pub(super) extra_effect_keys: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PathBoostDefinition {
    pub(super) id: PathBoostId,
    pub(super) key: Box<str>,
    pub(super) path: PathId,
    pub(super) effect_program: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ResonanceDefinition {
    pub(super) id: ResonanceId,
    pub(super) key: Box<str>,
    pub(super) path: PathId,
    pub(super) shared_resonance: Box<str>,
    pub(super) threshold: u16,
    pub(super) energy_max: Box<str>,
    pub(super) initial_energy: Box<str>,
    pub(super) parameters: Box<[Box<str>]>,
    pub(super) mechanic_tags: Box<[Box<str>]>,
    pub(super) effect_program: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct InterplayDefinition {
    pub(super) id: InterplayId,
    pub(super) key: Box<str>,
    pub(super) main_path: PathId,
    pub(super) sub_path: PathId,
    pub(super) thresholds: Box<str>,
    pub(super) effect_program: Box<str>,
}
