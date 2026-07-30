macro_rules! unique_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub(crate) struct $name(pub(crate) u32);
    };
}

unique_id!(CognitionRangeId);
unique_id!(SecretId);
unique_id!(ModeConstantId);
unique_id!(DiceId);
unique_id!(DiceCategoryId);
unique_id!(DicePathValueId);
unique_id!(DiceSlotId);
unique_id!(DiceFaceId);
unique_id!(DiceFaceTagId);
unique_id!(KnowledgeRuleId);
unique_id!(NeuralNodeId);
unique_id!(ConundrumLevelId);
unique_id!(TrailblazeBonusId);
unique_id!(PathId);
unique_id!(PathBoostId);
unique_id!(ResonanceId);
unique_id!(ExtrapolationId);
unique_id!(InterplayId);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Identity<I> {
    pub(crate) id: I,
    pub(crate) stable_key: Box<str>,
    pub(crate) source_id: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CanonicalScalar(pub(crate) Box<str>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CognitionRange {
    pub(crate) identity: Identity<CognitionRangeId>,
    pub(crate) area_key: Box<str>,
    pub(crate) minimum: CanonicalScalar,
    pub(crate) maximum: CanonicalScalar,
    pub(crate) global_minimum: CanonicalScalar,
    pub(crate) global_maximum: CanonicalScalar,
    pub(crate) inclusive: bool,
    pub(crate) lifecycle_json: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Secret {
    pub(crate) identity: Identity<SecretId>,
    pub(crate) area_key: Box<str>,
    pub(crate) area_source: Box<str>,
    pub(crate) plane_layer: u8,
    pub(crate) cognition_minimum: CanonicalScalar,
    pub(crate) cognition_maximum: CanonicalScalar,
    pub(crate) origin_minimum: Box<str>,
    pub(crate) origin_maximum: Box<str>,
    pub(crate) inclusive: bool,
    pub(crate) predecessors: Box<[Box<str>]>,
    pub(crate) next: Box<[Box<str>]>,
    pub(crate) evaluation_boundary: Box<str>,
    pub(crate) condition_hash: Box<str>,
    pub(crate) condition_digest: Box<str>,
    pub(crate) terminal: bool,
    pub(crate) lifecycle_policy: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModeConstant {
    pub(crate) identity: Identity<ModeConstantId>,
    pub(crate) mechanical_role: Box<str>,
    pub(crate) value_kind: Box<str>,
    pub(crate) values: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiceCategory {
    pub(crate) identity: Identity<DiceCategoryId>,
    pub(crate) sort: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiceDefinition {
    pub(crate) identity: Identity<DiceId>,
    pub(crate) sort: u16,
    pub(crate) category: DiceCategoryId,
    pub(crate) category_source: Box<str>,
    pub(crate) effect_parts_json: Box<str>,
    pub(crate) initial_effects: Box<[Box<str>]>,
    pub(crate) passive_effects: Box<[Box<str>]>,
    pub(crate) available_by_default: bool,
    pub(crate) unlock_id: Option<Box<str>>,
    pub(crate) ultra_face_source: Box<str>,
    pub(crate) common_face_sources: Box<[Box<str>]>,
    pub(crate) default_face_sources: Box<[Box<str>]>,
    pub(crate) suggestive_face_sources: Box<[Box<str>]>,
    pub(crate) recommended_face_sources: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DicePathValue {
    pub(crate) identity: Identity<DicePathValueId>,
    pub(crate) dice: DiceId,
    pub(crate) dice_source: Box<str>,
    pub(crate) path_key: Box<str>,
    pub(crate) path_source: Box<str>,
    pub(crate) boost_stat: Box<str>,
    pub(crate) trigger_interval: Box<str>,
    pub(crate) boost_value: CanonicalScalar,
    pub(crate) boost_unit: Box<str>,
    pub(crate) parameters: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiceSlot {
    pub(crate) identity: Identity<DiceSlotId>,
    pub(crate) index: u8,
    pub(crate) base_max_rarity: u8,
    pub(crate) extra_max_rarity: Option<u8>,
    pub(crate) upgraded_max_rarity: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiceFace {
    pub(crate) identity: Identity<DiceFaceId>,
    pub(crate) sort: u16,
    pub(crate) item_id: Box<str>,
    pub(crate) rarity: u8,
    pub(crate) activation_stage: u8,
    pub(crate) unlock_display_source: Box<str>,
    pub(crate) parameters: Box<[Box<str>]>,
    pub(crate) allowed_slot_keys: Box<[Box<str>]>,
    pub(crate) allowed_slot_sources: Box<[Box<str>]>,
    pub(crate) mechanical_codes: Box<[Box<str>]>,
    pub(crate) filter_tag_sources: Box<[Box<str>]>,
    pub(crate) allowed_dice_keys: Box<[Box<str>]>,
    pub(crate) allowed_dice_sources: Box<[Box<str>]>,
    pub(crate) universal_dice_eligibility: bool,
    pub(crate) no_target_behavior: Box<str>,
    pub(crate) target_policy_json: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiceFaceTag {
    pub(crate) identity: Identity<DiceFaceTagId>,
    pub(crate) sort: u16,
    pub(crate) mechanical_code: Box<str>,
    pub(crate) replacement_condition: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KnowledgeRule {
    pub(crate) identity: Identity<KnowledgeRuleId>,
    pub(crate) dice_face: DiceFaceId,
    pub(crate) operation: Box<str>,
    pub(crate) trigger_boundary: Box<str>,
    pub(crate) target_scope: Box<str>,
    pub(crate) selection_mode: Box<str>,
    pub(crate) knowledge_access: Box<str>,
    pub(crate) parameters: Box<[Box<str>]>,
    pub(crate) activation_stage: u8,
    pub(crate) target_policy_json: Box<str>,
    pub(crate) simultaneous_policy_json: Box<str>,
    pub(crate) dice_interactions_json: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NeuralNode {
    pub(crate) identity: Identity<NeuralNodeId>,
    pub(crate) topological_index: u16,
    pub(crate) prerequisites: Box<[Box<str>]>,
    pub(crate) next: Box<[Box<str>]>,
    pub(crate) external_unlocks: Box<[Box<str>]>,
    pub(crate) costs_json: Box<str>,
    pub(crate) important: bool,
    pub(crate) disposition: Box<str>,
    pub(crate) effect_domain: Box<str>,
    pub(crate) source_parameters_json: Box<str>,
    pub(crate) effect_contributions_json: Box<str>,
    pub(crate) rule_contribution: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConundrumLevel {
    pub(crate) identity: Identity<ConundrumLevelId>,
    pub(crate) source_type: Box<str>,
    pub(crate) track: Box<str>,
    pub(crate) level: u8,
    pub(crate) track_cap: u8,
    pub(crate) total_cap: u8,
    pub(crate) total_formula: Box<str>,
    pub(crate) unlock_requirement_json: Box<str>,
    pub(crate) composition_mode: Box<str>,
    pub(crate) active_contributions: Box<[Box<str>]>,
    pub(crate) replaces_levels: Box<[Box<str>]>,
    pub(crate) source_tag: u16,
    pub(crate) source_sort: u16,
    pub(crate) source_parameters_json: Box<str>,
    pub(crate) effect_contributions_json: Box<str>,
    pub(crate) rule_contribution: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TrailblazeBonus {
    pub(crate) identity: Identity<TrailblazeBonusId>,
    pub(crate) bonus_event: Box<str>,
    pub(crate) effect_contributions_json: Box<str>,
    pub(crate) rule_contribution: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PathDefinition {
    pub(crate) identity: Identity<PathId>,
    pub(crate) sort: u16,
    pub(crate) buff_type: u16,
    pub(crate) shared_resonance_id: u32,
    pub(crate) shared_formation_ids: Box<[Box<str>]>,
    pub(crate) path_boost: PathBoostId,
    pub(crate) normal_event_group: Box<str>,
    pub(crate) enhanced_event_group: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PathBoost {
    pub(crate) identity: Identity<PathBoostId>,
    pub(crate) path: PathId,
    pub(crate) aeon_source: Box<str>,
    pub(crate) effect_type: Box<str>,
    pub(crate) ability_name: Box<str>,
    pub(crate) target_team: Box<str>,
    pub(crate) target_property: Box<str>,
    pub(crate) boost_stat: Box<str>,
    pub(crate) stacking: Box<str>,
    pub(crate) value_conversion: Box<str>,
    pub(crate) dice_path_value_keys: Box<[Box<str>]>,
    pub(crate) allowed_increments: Box<[CanonicalScalar]>,
    pub(crate) rule_contribution: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Resonance {
    pub(crate) identity: Identity<ResonanceId>,
    pub(crate) path: PathId,
    pub(crate) resonance_kind: Box<str>,
    pub(crate) threshold: u16,
    pub(crate) energy_max: CanonicalScalar,
    pub(crate) initial_energy: CanonicalScalar,
    pub(crate) parameter_values_json: Box<str>,
    pub(crate) mechanic_tags: Box<[Box<str>]>,
    pub(crate) source_modifier: Box<str>,
    pub(crate) source_binding_type: Box<str>,
    pub(crate) source_binding_key: Box<str>,
    pub(crate) inherited_rule_ids: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Extrapolation {
    pub(crate) identity: Identity<ExtrapolationId>,
    pub(crate) path: PathId,
    pub(crate) aeon_source: Box<str>,
    pub(crate) buff_group: Box<str>,
    pub(crate) enhanced: bool,
    pub(crate) shared_resonance_id: u32,
    pub(crate) shared_resonance_kind: Box<str>,
    pub(crate) battle_event_type: Box<str>,
    pub(crate) source_modifier: Box<str>,
    pub(crate) source_binding_type: Box<str>,
    pub(crate) source_binding_key: Box<str>,
    pub(crate) source_parameters_json: Box<str>,
    pub(crate) battle_scope: Box<str>,
    pub(crate) controller_policy_json: Box<str>,
    pub(crate) rule_contribution: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Interplay {
    pub(crate) identity: Identity<InterplayId>,
    pub(crate) main_path: PathId,
    pub(crate) sub_path: PathId,
    pub(crate) main_threshold: u16,
    pub(crate) sub_threshold: u16,
    pub(crate) buff_group: Box<str>,
    pub(crate) shared_maze_buff: Box<str>,
    pub(crate) source_modifier: Box<str>,
    pub(crate) source_binding_type: Box<str>,
    pub(crate) source_binding_key: Box<str>,
    pub(crate) source_parameters_json: Box<str>,
    pub(crate) rule_contribution: Box<str>,
}
