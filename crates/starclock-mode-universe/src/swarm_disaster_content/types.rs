macro_rules! content_id {
    ($($name:ident),+ $(,)?) => {$(
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub(super) struct $name(pub(super) u32);
    )+};
}

content_id!(
    MapEventId,
    BlockRuleId,
    TopologyConsequenceId,
    BlessingId,
    BlessingLevelId,
    PoolMembershipId,
    CurioId,
    CurioStateId,
    CurioRuleId,
    OccurrenceId,
    OccurrenceVariantId,
    OccurrenceChoiceId,
    ServiceId,
    AdventureOutcomeId,
    CurrencyId,
    ServiceRuleId,
    EncounterGroupId,
    EncounterWaveId,
    EnemySlotId,
    BossPoolId,
    MechanicRuleId,
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MapEventDefinition {
    pub(super) id: MapEventId,
    pub(super) key: Box<str>,
    pub(super) chessboard_id: u32,
    pub(super) trigger: Box<str>,
    pub(super) weight: Box<str>,
    pub(super) operations: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BlockRuleDefinition {
    pub(super) id: BlockRuleId,
    pub(super) key: Box<str>,
    pub(super) chessboard_id: u32,
    pub(super) group: Box<str>,
    pub(super) domain_id: u32,
    pub(super) order: u16,
    pub(super) count: Box<str>,
    pub(super) candidates: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TopologyConsequenceDefinition {
    pub(super) id: TopologyConsequenceId,
    pub(super) key: Box<str>,
    pub(super) trigger_kind: Box<str>,
    pub(super) scope: Box<str>,
    pub(super) operations: Box<str>,
    pub(super) audience_die_id: u32,
    pub(super) active_stage: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BlessingDefinition {
    pub(super) id: BlessingId,
    pub(super) key: Box<str>,
    pub(super) shared_key: Box<str>,
    pub(super) path_key: Box<str>,
    pub(super) rarity: u8,
    pub(super) level_keys: Box<[Box<str>]>,
    pub(super) pool_rules: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BlessingLevelDefinition {
    pub(super) id: BlessingLevelId,
    pub(super) key: Box<str>,
    pub(super) blessing: BlessingId,
    pub(super) shared_blessing_key: Box<str>,
    pub(super) shared_level_key: Box<str>,
    pub(super) level: u8,
    pub(super) parameters: Box<[Box<str>]>,
    pub(super) effect_program: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PoolMembershipDefinition {
    pub(super) id: PoolMembershipId,
    pub(super) key: Box<str>,
    pub(super) pool_key: Box<str>,
    pub(super) member_kind: Box<str>,
    pub(super) member_key: Box<str>,
    pub(super) eligibility: Box<str>,
    pub(super) weight_policy: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CurioDefinition {
    pub(super) id: CurioId,
    pub(super) key: Box<str>,
    pub(super) mode_copy_key: Box<str>,
    pub(super) pool_category: Box<str>,
    pub(super) pool_rules: Box<str>,
    pub(super) initial_state: CurioStateId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CurioStateDefinition {
    pub(super) id: CurioStateId,
    pub(super) key: Box<str>,
    pub(super) curio: CurioId,
    pub(super) state: Box<str>,
    pub(super) charges: Option<Box<str>>,
    pub(super) effect_program: Box<str>,
    pub(super) lifecycle: Box<str>,
    pub(super) repair_target: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CurioRuleDefinition {
    pub(super) id: CurioRuleId,
    pub(super) key: Box<str>,
    pub(super) curio: CurioId,
    pub(super) state: CurioStateId,
    pub(super) trigger_phase: Box<str>,
    pub(super) trigger: Box<str>,
    pub(super) lifecycle: Box<str>,
    pub(super) replacement_policy: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct OccurrenceDefinition {
    pub(super) id: OccurrenceId,
    pub(super) key: Box<str>,
    pub(super) order: u16,
    pub(super) source_event_type: Box<str>,
    pub(super) variant_keys: Box<[Box<str>]>,
    pub(super) pool_rules: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct OccurrenceVariantDefinition {
    pub(super) id: OccurrenceVariantId,
    pub(super) key: Box<str>,
    pub(super) occurrence_keys: Box<[Box<str>]>,
    pub(super) choice_keys: Box<[Box<str>]>,
    pub(super) graph: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct OccurrenceChoiceDefinition {
    pub(super) id: OccurrenceChoiceId,
    pub(super) key: Box<str>,
    pub(super) variant: OccurrenceVariantId,
    pub(super) ordinal: u16,
    pub(super) node_ordinal: u16,
    pub(super) option_ordinal: u16,
    pub(super) conditions: Box<str>,
    pub(super) costs: Box<str>,
    pub(super) outcomes: Box<str>,
    pub(super) display: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ServiceDefinition {
    pub(super) id: ServiceId,
    pub(super) key: Box<str>,
    pub(super) shared_key: Box<str>,
    pub(super) service_kind: Box<str>,
    pub(super) resource_key: Option<Box<str>>,
    pub(super) parameters: Box<str>,
    pub(super) eligibility: Box<str>,
    pub(super) price_policy: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct AdventureOutcomeDefinition {
    pub(super) id: AdventureOutcomeId,
    pub(super) key: Box<str>,
    pub(super) adventure_type: Box<str>,
    pub(super) parameter_group: Box<str>,
    pub(super) tier: Box<str>,
    pub(super) offered_result: Box<str>,
    pub(super) reward_program: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CurrencyDefinition {
    pub(super) id: CurrencyId,
    pub(super) key: Box<str>,
    pub(super) resource_key: Box<str>,
    pub(super) initial_value: Box<str>,
    pub(super) cap_policy: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ServiceRuleDefinition {
    pub(super) id: ServiceRuleId,
    pub(super) key: Box<str>,
    pub(super) service_key: Box<str>,
    pub(super) conditions: Box<str>,
    pub(super) costs: Box<str>,
    pub(super) operations: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct EncounterGroupDefinition {
    pub(super) id: EncounterGroupId,
    pub(super) key: Box<str>,
    pub(super) room_key: Option<Box<str>>,
    pub(super) area_keys: Box<[Box<str>]>,
    pub(super) boss_choice_keys: Box<[Box<str>]>,
    pub(super) role: Box<str>,
    pub(super) wave_keys: Box<[Box<str>]>,
    pub(super) difficulty_binding: Box<str>,
    pub(super) members: Box<str>,
    pub(super) weight_policy: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct EncounterWaveDefinition {
    pub(super) id: EncounterWaveId,
    pub(super) key: Box<str>,
    pub(super) group: EncounterGroupId,
    pub(super) ordinal: u16,
    pub(super) slot_keys: Box<[Box<str>]>,
    pub(super) stage_type: Box<str>,
    pub(super) authored_level: u16,
    pub(super) hard_level_group: u16,
    pub(super) level_binding: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct EnemySlotDefinition {
    pub(super) id: EnemySlotId,
    pub(super) key: Box<str>,
    pub(super) wave: EncounterWaveId,
    pub(super) wave_key: Box<str>,
    pub(super) formation_index: u8,
    pub(super) enemy_variant_key: Box<str>,
    pub(super) boss_choice_keys: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BossPoolDefinition {
    pub(super) id: BossPoolId,
    pub(super) key: Box<str>,
    pub(super) difficulty_key: Box<str>,
    pub(super) area_id: u32,
    pub(super) tier: Box<str>,
    pub(super) candidate_keys: Box<[Box<str>]>,
    pub(super) candidate_order: Box<str>,
    pub(super) consequences: Box<str>,
    pub(super) selection_policy: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MechanicRuleDefinition {
    pub(super) id: MechanicRuleId,
    pub(super) key: Box<str>,
    pub(super) family_key: Box<str>,
    pub(super) domain: Box<str>,
    pub(super) triggers: Box<[Box<str>]>,
    pub(super) slots: Box<str>,
    pub(super) program: Box<str>,
    pub(super) fixture_keys: Box<[Box<str>]>,
    pub(super) disposition: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct AuditCatalogSummary {
    pub(super) source_records: usize,
    pub(super) coverage_rows: usize,
    pub(super) research_gaps: usize,
    pub(super) affected_rows: usize,
    pub(super) fixtures: usize,
    pub(super) receipts: usize,
    pub(super) manifest_rows: usize,
    pub(super) pack_rows: usize,
    pub(super) frozen_obligations: u32,
    pub(super) mechanic_rules: u16,
    pub(super) fixture_families: u16,
}
