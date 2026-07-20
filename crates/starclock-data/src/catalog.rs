//! Validated Sora-row to immutable Starclock catalog boundary.
//! Generated rows and preliminary definition storage remain private.
use starclock_combat::modifier::registry::ModifierRegistry;
use starclock_combat::{
    AbilityId, DispelCategory, DurationClock, EffectCategory, EffectDefinitionId,
    EffectSnapshotPolicy, EffectStackPolicy, EffectTeardownPolicy, EffectTickPhase, Ratio,
};
use std::{collections::BTreeMap, sync::Arc};

use crate::catalog_manifest::convert_manifest;
use crate::coverage::{GoalCoverageCategory, GoalCoverageState};
use crate::effect_lower::{
    lower_dispel, lower_duration_clock, lower_effect_category, lower_snapshot_policy,
    lower_stack_policy, lower_teardown, lower_tick_phase,
};
use crate::generated::{
    SoraConfig, content_kind::ContentKind, coverage_state::CoverageState,
    release_state::ReleaseState, runtime::SoraBundle,
};

const METADATA_TABLES: [&str; 5] = [
    "ConfigManifest",
    "ContentEvidenceBinding",
    "ContentIdentity",
    "EvidenceRecord",
    "SourceRecord",
];
const LOWERED_TABLES: [&str; 57] = [
    "Ability",
    "AbilityHitPlanBinding",
    "AbilityPhase",
    "ActivityDefinition",
    "ActivityEdge",
    "ActivityNode",
    "ActivitySection",
    "ActivitySlot",
    "ActivitySlotReset",
    "AiCandidate",
    "AiGraph",
    "AiState",
    "AiTransition",
    "BattleBinding",
    "BattleBindingRule",
    "BattleParticipantSlot",
    "BattleResultProjection",
    "BattleResultProjectionField",
    "Character",
    "CharacterAbilityBinding",
    "CharacterStat",
    "ConditionExpression",
    "Effect",
    "Encounter",
    "EncounterRuleBinding",
    "EncounterWave",
    "EnemyAbility",
    "EnemyDebuffResistance",
    "EnemyLink",
    "EnemyPhase",
    "EnemyResistance",
    "EnemyStat",
    "EnemyTemplate",
    "EnemyToughnessLayer",
    "EnemyVariant",
    "EnemyVariantAbility",
    "EnemyWeakness",
    "EventFilter",
    "HitPlan",
    "HitPlanHit",
    "ModifierDefinition",
    "ModifierFilter",
    "ModifierStackingGroup",
    "NativeHandler",
    "Operation",
    "ParticipantPolicy",
    "Program",
    "ProgramStep",
    "RuleDefinition",
    "RuleTrigger",
    "Selector",
    "StandardProfile",
    "StandardScenario",
    "StateSlot",
    "StateSlotReset",
    "ValueExpression",
    "WaveSlot",
];

/// Stable category for a catalog-load failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogLoadErrorKind {
    /// The bytes are not a readable bundle for the generated schema.
    Bundle,
    /// Manifest compatibility or format metadata is invalid.
    Manifest,
    /// Shared identity, provenance or evidence metadata is invalid.
    Metadata,
    /// A row cannot be converted to a valid immutable domain definition.
    Domain,
    /// A populated table has no reviewed lowering implementation yet.
    UnsupportedTable,
}

/// Stable load error that never exposes a generated Sora error type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogLoadError {
    kind: CatalogLoadErrorKind,
    message: String,
}

impl CatalogLoadError {
    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(&self) -> CatalogLoadErrorKind {
        self.kind
    }
}

impl std::fmt::Display for CatalogLoadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CatalogLoadError {}

/// Starclock-owned compatibility metadata copied from the singleton manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogManifest {
    /// Authored game-version snapshot.
    pub game_version: String,
    /// Source snapshot date in `YYYY-MM-DD` form.
    pub snapshot_date: String,
    /// Stable data revision.
    pub data_revision: String,
    /// Rules compatibility revision required by the data.
    pub required_rules_revision: String,
    /// Pinned Sora authoring-tool version.
    pub sora_cli_version: String,
    /// Authoritative numeric policy revision.
    pub numeric_policy_revision: String,
    /// RNG mapping revision.
    pub rng_algorithm_revision: String,
    /// Canonical state-hash revision.
    pub state_hash_revision: String,
    /// Replay envelope revision.
    pub replay_format_version: String,
    /// Frozen coverage-manifest SHA-256 digest.
    pub coverage_manifest_sha256: String,
}

/// Counts from the validated metadata and private domain partitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogSummary {
    /// All metadata identities represented by the bundle.
    pub identity_count: usize,
    /// Identities enabled for production use.
    pub enabled_identity_count: usize,
    /// Lowered combat ability definitions.
    pub ability_count: usize,
    /// Lowered ordered hit-plan definitions.
    pub hit_plan_count: usize,
    /// Lowered build-side character definitions.
    pub character_count: usize,
    /// Lowered generic effect definitions.
    pub effect_count: usize,
    /// Lowered deterministic enemy AI graphs.
    pub ai_graph_count: usize,
    /// Lowered mechanically distinct enemy variants.
    pub enemy_count: usize,
    /// Lowered ordered encounter definitions.
    pub encounter_count: usize,
    /// Lowered ordinary Standard profiles.
    pub standard_profile_count: usize,
    /// Lowered reproducible Standard scenario descriptors.
    pub standard_scenario_count: usize,
}

/// Generated-row-free binding data for one reproducible Standard scenario.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandardScenarioDefinition {
    pub(super) id: starclock_mode_standard::StandardScenarioId,
    pub(super) profile: starclock_mode_standard::StandardProfileId,
    pub(super) activity: starclock_activity::ActivityDefinitionId,
    pub(super) binding: starclock_mode_standard::StandardBindingId,
    pub(super) master_seed: u64,
    pub(super) expected_outcome: starclock_mode_standard::StandardExpectedOutcome,
}

impl StandardScenarioDefinition {
    #[must_use]
    pub const fn id(self) -> starclock_mode_standard::StandardScenarioId {
        self.id
    }
    #[must_use]
    pub const fn profile(self) -> starclock_mode_standard::StandardProfileId {
        self.profile
    }
    #[must_use]
    pub const fn activity(self) -> starclock_activity::ActivityDefinitionId {
        self.activity
    }
    #[must_use]
    pub const fn binding(self) -> starclock_mode_standard::StandardBindingId {
        self.binding
    }
    #[must_use]
    pub const fn master_seed(self) -> u64 {
        self.master_seed
    }
    #[must_use]
    pub const fn expected_outcome(self) -> starclock_mode_standard::StandardExpectedOutcome {
        self.expected_outcome
    }
}

/// Immutable application/data-layer catalog aggregate.
///
/// It is safe to share this value between isolated jobs. Its internal rows are
/// Starclock-owned values and contain no generated-reader types.
#[derive(Debug)]
pub struct SimulationCatalog {
    pub(super) manifest: CatalogManifest,
    pub(super) identities: Box<[IdentityDefinition]>,
    pub(super) combat: CombatDefinitions,
    pub(super) builds: crate::build_lower::BuildDefinitions,
    pub(super) encounters: crate::encounter_lower::EncounterDefinitions,
    pub(super) standard: crate::standard_lower::StandardDefinitions,
}

/// Loads and validates a production bundle into an immutable shared catalog.
pub fn load(bytes: &[u8]) -> Result<Arc<SimulationCatalog>, CatalogLoadError> {
    load_with_mode(bytes, LoadMode::Production)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LoadMode {
    Production,
    #[cfg(test)]
    Fixture,
}

#[derive(Debug)]
pub(super) struct IdentityDefinition {
    pub(super) id: u32,
    pub(super) stable_key: Box<str>,
    pub(super) kind: IdentityKind,
    pub(super) enabled: bool,
    pub(super) goal_category: Option<GoalCoverageCategory>,
    pub(super) coverage_state: GoalCoverageState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IdentityKind {
    Character,
    Ability,
    Program,
    Other,
}

#[derive(Debug)]
pub(super) struct CombatDefinitions {
    pub(super) abilities: Box<[AbilityDefinition]>,
    pub(super) hit_plans: Box<[HitPlanDefinition]>,
    pub(super) modifiers: ModifierRegistry,
    pub(super) programs: Box<[crate::operation_lower::RuleProgramDefinition]>,
    pub(super) effects: Box<[EffectDataDefinition]>,
    pub(super) rules: Box<[crate::rule_lower::RuleDataDefinition]>,
}

/// Generated-row-free authored effect data retained for build compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectDataDefinition {
    id: EffectDefinitionId,
    category: EffectCategory,
    dispel: DispelCategory,
    stack_limit: u16,
    duration: Option<starclock_combat::rule::model::ValueExpr>,
    duration_clock: DurationClock,
    tick_phase: EffectTickPhase,
    stack_policy: EffectStackPolicy,
    magnitude: Option<starclock_combat::rule::model::ValueExpr>,
    snapshot_policy: EffectSnapshotPolicy,
    teardown_policy: EffectTeardownPolicy,
    application_priority: i32,
    tags: Box<[Box<str>]>,
}

impl EffectDataDefinition {
    #[must_use]
    pub const fn id(&self) -> EffectDefinitionId {
        self.id
    }
    #[must_use]
    pub const fn category(&self) -> EffectCategory {
        self.category
    }
    #[must_use]
    pub const fn dispel(&self) -> DispelCategory {
        self.dispel
    }
    #[must_use]
    pub const fn stack_limit(&self) -> u16 {
        self.stack_limit
    }
    #[must_use]
    pub const fn duration(&self) -> Option<&starclock_combat::rule::model::ValueExpr> {
        self.duration.as_ref()
    }
    #[must_use]
    pub const fn duration_clock(&self) -> DurationClock {
        self.duration_clock
    }
    #[must_use]
    pub const fn tick_phase(&self) -> EffectTickPhase {
        self.tick_phase
    }
    #[must_use]
    pub const fn stack_policy(&self) -> EffectStackPolicy {
        self.stack_policy
    }
    #[must_use]
    pub const fn magnitude(&self) -> Option<&starclock_combat::rule::model::ValueExpr> {
        self.magnitude.as_ref()
    }
    #[must_use]
    pub const fn snapshot_policy(&self) -> EffectSnapshotPolicy {
        self.snapshot_policy
    }
    #[must_use]
    pub const fn teardown_policy(&self) -> EffectTeardownPolicy {
        self.teardown_policy
    }
    #[must_use]
    pub const fn application_priority(&self) -> i32 {
        self.application_priority
    }
    #[must_use]
    pub fn tags(&self) -> &[Box<str>] {
        &self.tags
    }
}

#[derive(Debug)]
pub(super) struct AbilityDefinition {
    pub(super) id: AbilityId,
    kind: u8,
    target_pattern: u8,
    retarget_policy: u8,
    level_cap: u16,
    cooldown_actions: u16,
    pub(super) semantic_tags: starclock_combat::catalog::action::AbilityTags,
    phases: Box<[AbilityPhaseDefinition]>,
}

#[derive(Debug)]
struct AbilityPhaseDefinition {
    sequence: u16,
}

#[derive(Debug)]
pub(super) struct HitPlanDefinition {
    id: u32,
    target_pattern: u8,
    retarget_policy: u8,
    hits: Box<[HitDefinition]>,
}

#[derive(Debug)]
struct HitDefinition {
    sequence: u16,
    target_group: u8,
    damage_ratio: Ratio,
    toughness_ratio: Ratio,
    crit_policy: u8,
}

impl CombatDefinitions {
    pub(super) fn ability_level_cap(&self, id: AbilityId) -> Option<u16> {
        self.abilities
            .iter()
            .find(|ability| ability.id == id)
            .map(|ability| ability.level_cap)
    }
}

pub(super) fn load_with_mode(
    bytes: &[u8],
    mode: LoadMode,
) -> Result<Arc<SimulationCatalog>, CatalogLoadError> {
    let bundle =
        SoraBundle::parse(bytes).map_err(|error| fail(CatalogLoadErrorKind::Bundle, error))?;
    let config = SoraConfig::from_source(&bundle)
        .map_err(|error| fail(CatalogLoadErrorKind::Bundle, error))?;
    validate_populated_tables(&config)?;
    let manifest = convert_manifest(&config)?;
    let identities = convert_metadata(&config, mode, &manifest)?;
    let identity_by_id = identities
        .iter()
        .map(|identity| (identity.id, identity))
        .collect::<BTreeMap<_, _>>();
    let combat = convert_combat(&config, mode, &identity_by_id)?;
    let builds = crate::build_lower::convert(&config, mode, &identity_by_id, &combat)?;
    let encounters = crate::encounter_lower::convert(&config, mode, &identity_by_id, &combat)?;
    let standard = crate::standard_lower::convert(&config, mode, &identity_by_id, &encounters)?;
    let catalog = SimulationCatalog {
        manifest,
        identities: identities.into_boxed_slice(),
        combat,
        builds,
        encounters,
        standard,
    };
    validate_converted_catalog(&catalog)?;
    Ok(Arc::new(catalog))
}

fn validate_converted_catalog(catalog: &SimulationCatalog) -> Result<(), CatalogLoadError> {
    if catalog.combat.abilities.iter().any(|ability| {
        ability.kind > 12
            || ability.target_pattern > 7
            || ability.retarget_policy > 3
            || ability.level_cap == 0
            || ability.cooldown_actions > 100
            || ability.phases.is_empty()
    }) || catalog.combat.hit_plans.iter().any(|plan| {
        plan.target_pattern > 7
            || plan.retarget_policy > 3
            || plan.hits.is_empty()
            || plan.hits.iter().any(|hit| {
                hit.target_group > 5
                    || hit.crit_policy > 2
                    || hit.damage_ratio.scaled() < 0
                    || hit.toughness_ratio.scaled() < 0
            })
    }) || catalog.builds.violates_invariants(&catalog.combat)
    {
        return Err(fail(
            CatalogLoadErrorKind::Domain,
            "converted catalog violates an immutable definition invariant",
        ));
    }
    Ok(())
}

fn validate_populated_tables(config: &SoraConfig) -> Result<(), CatalogLoadError> {
    for table in config.tables() {
        let name = table.info().name;
        if table.is_empty()
            || METADATA_TABLES.binary_search(&name).is_ok()
            || LOWERED_TABLES.binary_search(&name).is_ok()
        {
            continue;
        }
        return Err(fail(
            CatalogLoadErrorKind::UnsupportedTable,
            format!(
                "table {name} has {} row(s) but no reviewed domain lowering",
                table.len()
            ),
        ));
    }
    Ok(())
}

fn convert_metadata(
    config: &SoraConfig,
    mode: LoadMode,
    manifest: &CatalogManifest,
) -> Result<Vec<IdentityDefinition>, CatalogLoadError> {
    let sources = config.source_record();
    for row in sources.ordered_rows() {
        positive(row.id, "SourceRecord.id")?;
        if row.stable_key.trim().is_empty()
            || row.publisher.trim().is_empty()
            || !row.url.starts_with("https://")
            || !valid_date(&row.accessed_on)
            || !valid_sha256(&row.evidence_sha256)
        {
            return Err(fail(
                CatalogLoadErrorKind::Metadata,
                format!("source {} has invalid required metadata", row.id),
            ));
        }
        if mode == LoadMode::Production
            && (matches!(
                row.category,
                crate::generated::source_category::SourceCategory::SyntheticFixture
            ) || matches!(
                row.confidence,
                crate::generated::confidence::Confidence::SyntheticFixture
            ))
        {
            return Err(fail(
                CatalogLoadErrorKind::Metadata,
                format!("production source {} is labeled synthetic", row.id),
            ));
        }
    }
    for row in config.evidence_record().ordered_rows() {
        positive(row.id, "EvidenceRecord.id")?;
        if row.stable_key.trim().is_empty() || !valid_sha256(&row.sha256) {
            return Err(fail(
                CatalogLoadErrorKind::Metadata,
                format!("evidence {} has invalid identity or digest", row.id),
            ));
        }
        if let Some(source) = row.source_record_id
            && sources.get(&source).is_none()
        {
            return Err(fail(
                CatalogLoadErrorKind::Metadata,
                format!("evidence {} refers to missing source {source}", row.id),
            ));
        }
    }

    let mut identities = Vec::with_capacity(config.content_identity().len());
    for row in config.content_identity().ordered_rows() {
        let id = positive(row.id, "ContentIdentity.id")?;
        if row.stable_key.trim().is_empty()
            || row.name_en.trim().is_empty()
            || row.name_zh_cn.trim().is_empty()
            || row.summary_en.trim().is_empty()
            || row.summary_zh_cn.trim().is_empty()
            || row.game_version_snapshot != manifest.game_version
            || row.source_record_ids.is_empty()
        {
            return Err(fail(
                CatalogLoadErrorKind::Metadata,
                format!("identity {} has incomplete shared metadata", row.id),
            ));
        }
        if row
            .source_record_ids
            .iter()
            .any(|source| sources.get(source).is_none())
        {
            return Err(fail(
                CatalogLoadErrorKind::Metadata,
                format!("identity {} refers to a missing source", row.id),
            ));
        }
        validate_release_coverage(row, mode)?;
        identities.push(IdentityDefinition {
            id,
            stable_key: row.stable_key.clone().into_boxed_str(),
            kind: identity_kind(row.content_kind),
            enabled: row.enabled,
            goal_category: GoalCoverageCategory::from_content_kind(row.content_kind),
            coverage_state: GoalCoverageState::from_generated(row.coverage_state),
        });
    }
    identities.sort_unstable_by_key(|identity| identity.id);

    let bindings = config.content_evidence_binding();
    let mut binding_counts = BTreeMap::<u32, usize>::new();
    for row in bindings.iter() {
        let content = positive(row.content_id, "ContentEvidenceBinding.content_id")?;
        positive(row.sequence, "ContentEvidenceBinding.sequence")?;
        if row.fact_key.trim().is_empty()
            || config.content_identity().get(&row.content_id).is_none()
            || sources.get(&row.source_record_id).is_none()
            || config
                .evidence_record()
                .get(&row.evidence_record_id)
                .is_none()
        {
            return Err(fail(
                CatalogLoadErrorKind::Metadata,
                format!("evidence binding for content {content} is incomplete"),
            ));
        }
        *binding_counts.entry(content).or_default() += 1;
    }
    if identities
        .iter()
        .any(|identity| binding_counts.get(&identity.id).copied().unwrap_or(0) == 0)
    {
        return Err(fail(
            CatalogLoadErrorKind::Metadata,
            "every content identity requires at least one evidence binding",
        ));
    }
    Ok(identities)
}

fn validate_release_coverage(
    row: &crate::generated::content_identity::ContentIdentity,
    mode: LoadMode,
) -> Result<(), CatalogLoadError> {
    if mode == LoadMode::Production
        && (row.release_state == ReleaseState::ProjectFixture
            || row.content_kind == ContentKind::SyntheticFixture)
    {
        return Err(fail(
            CatalogLoadErrorKind::Metadata,
            format!("production identity {} is labeled as a fixture", row.id),
        ));
    }
    let ready = matches!(
        row.coverage_state,
        CoverageState::DataReady | CoverageState::GoldenVerified
    );
    match row.release_state {
        ReleaseState::Released if row.enabled != ready => Err(fail(
            CatalogLoadErrorKind::Metadata,
            format!(
                "released identity {} has inconsistent enabled/coverage state",
                row.id
            ),
        )),
        ReleaseState::Announced if row.enabled => Err(fail(
            CatalogLoadErrorKind::Metadata,
            format!("announced identity {} cannot be enabled", row.id),
        )),
        ReleaseState::ProjectFixture if mode == LoadMode::Production => unreachable!(),
        _ => Ok(()),
    }
}

fn convert_combat(
    config: &SoraConfig,
    mode: LoadMode,
    identities: &BTreeMap<u32, &IdentityDefinition>,
) -> Result<CombatDefinitions, CatalogLoadError> {
    let mut effects = Vec::new();
    for row in config.effect().ordered_rows() {
        let raw = positive(row.id, "Effect.id")?;
        require_identity(identities, raw, IdentityKind::Other, mode)?;
        let mut tags = config
            .effect_tag()
            .iter()
            .filter(|tag| tag.effect_id == row.id)
            .collect::<Vec<_>>();
        tags.sort_unstable_by_key(|tag| tag.sequence);
        contiguous(
            tags.iter()
                .map(|tag| positive_u16(tag.sequence, "EffectTag.sequence"))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter(),
            "effect tags",
        )?;
        let mut visiting = std::collections::BTreeSet::new();
        let duration = row
            .duration_expression_id
            .map(|id| crate::modifier_lower::expression(config, id, &mut visiting))
            .transpose()?;
        visiting.clear();
        let magnitude = row
            .magnitude_comparator_expression_id
            .map(|id| crate::modifier_lower::expression(config, id, &mut visiting))
            .transpose()?;
        let duration_clock = lower_duration_clock(row.duration_clock);
        if (duration_clock == DurationClock::Permanent) != duration.is_none() {
            return Err(fail(
                CatalogLoadErrorKind::Domain,
                format!("effect {} duration/clock disagree", row.id),
            ));
        }
        effects.push(EffectDataDefinition {
            id: EffectDefinitionId::new(raw).expect("positive effect ID"),
            category: lower_effect_category(row.category),
            dispel: lower_dispel(row.dispel_category),
            stack_limit: bounded_u16(row.stack_limit, "Effect.stack_limit")?,
            duration,
            duration_clock,
            tick_phase: lower_tick_phase(row.tick_phase),
            stack_policy: lower_stack_policy(row.stack_policy),
            magnitude,
            snapshot_policy: lower_snapshot_policy(row.snapshot_policy),
            teardown_policy: lower_teardown(row.teardown_policy),
            application_priority: row.application_priority,
            tags: tags
                .into_iter()
                .map(|tag| tag.tag.clone().into_boxed_str())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        });
    }
    effects.sort_unstable_by_key(|effect| effect.id);
    let mut abilities = Vec::new();
    for row in config.ability().ordered_rows() {
        let raw = positive(row.id, "Ability.id")?;
        require_identity(identities, raw, IdentityKind::Ability, mode)?;
        let level_cap = bounded_u16(row.level_cap, "Ability.level_cap")?;
        if level_cap == 0 {
            return Err(fail(
                CatalogLoadErrorKind::Domain,
                "ability level cap must be positive",
            ));
        }
        if row.entry_rule_identity_id.is_some() {
            return Err(fail(
                CatalogLoadErrorKind::UnsupportedTable,
                format!("ability {} requires Rule IR lowering", row.id),
            ));
        }
        let mut phases = config
            .ability_phase()
            .iter()
            .filter(|phase| phase.ability_id == row.id)
            .map(|phase| {
                if phase.program_identity_id.is_some() {
                    return Err(fail(
                        CatalogLoadErrorKind::UnsupportedTable,
                        format!("ability {} phase requires program lowering", row.id),
                    ));
                }
                Ok(AbilityPhaseDefinition {
                    sequence: positive_u16(phase.sequence, "AbilityPhase.sequence")?,
                })
            })
            .collect::<Result<Vec<_>, CatalogLoadError>>()?;
        phases.sort_unstable_by_key(|phase| phase.sequence);
        contiguous(phases.iter().map(|phase| phase.sequence), "ability phases")?;
        if phases.is_empty() {
            return Err(fail(
                CatalogLoadErrorKind::Domain,
                format!("ability {} has no authored phase", row.id),
            ));
        }
        abilities.push(AbilityDefinition {
            id: AbilityId::new(raw).expect("positive u32 is a valid AbilityId"),
            kind: ability_kind(row.kind),
            target_pattern: target_pattern(row.target_pattern),
            retarget_policy: retarget_policy(row.retarget_policy),
            level_cap,
            cooldown_actions: bounded_u16(row.cooldown_actions, "Ability.cooldown_actions")?,
            semantic_tags: starclock_combat::catalog::action::AbilityTags::from_bits(
                u32::try_from(row.semantic_tags_mask).map_err(|_| {
                    fail(
                        CatalogLoadErrorKind::Domain,
                        "negative ability semantic tag mask",
                    )
                })?,
            )
            .ok_or_else(|| {
                fail(
                    CatalogLoadErrorKind::Domain,
                    "unknown ability semantic tag bit",
                )
            })?,
            phases: phases.into_boxed_slice(),
        });
    }
    abilities.sort_unstable_by_key(|ability| ability.id);

    let mut hit_plans = Vec::new();
    for row in config.hit_plan().ordered_rows() {
        let id = positive(row.id, "HitPlan.id")?;
        require_identity(identities, id, IdentityKind::Program, mode)?;
        let declared = positive(row.declared_hit_count, "HitPlan.declared_hit_count")? as usize;
        let mut hits = config
            .hit_plan_hit()
            .iter()
            .filter(|hit| hit.hit_plan_id == row.id)
            .map(|hit| {
                Ok(HitDefinition {
                    sequence: positive_u16(hit.sequence, "HitPlanHit.sequence")?,
                    target_group: hit_target_group(hit.target_group),
                    damage_ratio: Ratio::from_scaled(parse_decimal(&hit.damage_ratio_decimal)?),
                    toughness_ratio: Ratio::from_scaled(parse_decimal(
                        &hit.toughness_ratio_decimal,
                    )?),
                    crit_policy: crit_policy(hit.crit_policy),
                })
            })
            .collect::<Result<Vec<_>, CatalogLoadError>>()?;
        hits.sort_unstable_by_key(|hit| hit.sequence);
        contiguous(hits.iter().map(|hit| hit.sequence), "hit-plan hits")?;
        if hits.len() != declared
            || hits
                .iter()
                .map(|hit| i128::from(hit.damage_ratio.scaled()))
                .sum::<i128>()
                != 1_000_000
            || hits
                .iter()
                .map(|hit| i128::from(hit.toughness_ratio.scaled()))
                .sum::<i128>()
                != 1_000_000
        {
            return Err(fail(
                CatalogLoadErrorKind::Domain,
                format!("hit plan {} has an invalid count or ratio sum", row.id),
            ));
        }
        hit_plans.push(HitPlanDefinition {
            id,
            target_pattern: target_pattern(row.target_pattern),
            retarget_policy: retarget_policy(row.retarget_policy),
            hits: hits.into_boxed_slice(),
        });
    }
    hit_plans.sort_unstable_by_key(|plan| plan.id);

    for binding in config.ability_hit_plan_binding().iter() {
        let ability = abilities
            .iter()
            .find(|ability| ability.id.get() == u32::try_from(binding.ability_id).unwrap_or(0));
        let plan = hit_plans
            .iter()
            .find(|plan| plan.id == u32::try_from(binding.hit_plan_id).unwrap_or(0));
        let phase = positive_u16(
            binding.phase_sequence,
            "AbilityHitPlanBinding.phase_sequence",
        )?;
        if ability
            .is_none_or(|definition| !definition.phases.iter().any(|item| item.sequence == phase))
            || plan.is_none_or(|plan| {
                ability.is_none_or(|ability| {
                    plan.target_pattern != ability.target_pattern
                        || plan.retarget_policy != ability.retarget_policy
                })
            })
        {
            return Err(fail(
                CatalogLoadErrorKind::Domain,
                "ability/hit-plan binding refers to a missing definition or phase",
            ));
        }
    }
    let native_handlers = crate::native_handler_lower::audit(config)?;
    let modifiers = crate::modifier_lower::convert(config)?;
    let programs = crate::operation_lower::convert(config, &native_handlers)?;
    let rules = crate::rule_lower::convert(config, mode, identities, &native_handlers)?;
    Ok(CombatDefinitions {
        abilities: abilities.into_boxed_slice(),
        hit_plans: hit_plans.into_boxed_slice(),
        modifiers,
        programs: programs.into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        rules: rules.into_boxed_slice(),
    })
}

pub(super) fn require_identity(
    identities: &BTreeMap<u32, &IdentityDefinition>,
    id: u32,
    kind: IdentityKind,
    mode: LoadMode,
) -> Result<(), CatalogLoadError> {
    let identity = identities.get(&id).ok_or_else(|| {
        fail(
            CatalogLoadErrorKind::Domain,
            format!("definition {id} has no content identity"),
        )
    })?;
    if identity.kind != kind {
        return Err(fail(
            CatalogLoadErrorKind::Domain,
            format!(
                "identity {} ({}) has the wrong content kind",
                id, identity.stable_key
            ),
        ));
    }
    if mode == LoadMode::Production && !identity.enabled {
        return Err(fail(
            CatalogLoadErrorKind::Domain,
            format!(
                "disabled identity {} has executable rows",
                identity.stable_key
            ),
        ));
    }
    Ok(())
}

fn identity_kind(kind: ContentKind) -> IdentityKind {
    match kind {
        ContentKind::CharacterForm => IdentityKind::Character,
        ContentKind::Ability => IdentityKind::Ability,
        ContentKind::Program => IdentityKind::Program,
        _ => IdentityKind::Other,
    }
}

fn ability_kind(value: crate::generated::ability_kind::AbilityKind) -> u8 {
    use crate::generated::ability_kind::AbilityKind as V;
    match value {
        V::Basic => 0,
        V::Skill => 1,
        V::Ultimate => 2,
        V::Talent => 3,
        V::Technique => 4,
        V::EnhancedBasic => 5,
        V::EnhancedSkill => 6,
        V::FollowUp => 7,
        V::Counter => 8,
        V::Summon => 9,
        V::Memosprite => 10,
        V::Passive => 11,
        V::Entry => 12,
    }
}

fn target_pattern(value: crate::generated::target_pattern::TargetPattern) -> u8 {
    use crate::generated::target_pattern::TargetPattern as V;
    match value {
        V::SingleTarget => 0,
        V::Blast => 1,
        V::Aoe => 2,
        V::Bounce => 3,
        V::Support => 4,
        V::Enhance => 5,
        V::None => 6,
        V::ContentDefined => 7,
    }
}

fn retarget_policy(value: crate::generated::retarget_policy::RetargetPolicy) -> u8 {
    use crate::generated::retarget_policy::RetargetPolicy as V;
    match value {
        V::Locked => 0,
        V::CancelRemaining => 1,
        V::RetargetSameSide => 2,
        V::RecomputeEachHit => 3,
    }
}

fn hit_target_group(value: crate::generated::hit_target_group::HitTargetGroup) -> u8 {
    use crate::generated::hit_target_group::HitTargetGroup as V;
    match value {
        V::Primary => 0,
        V::Adjacent => 1,
        V::Selected => 2,
        V::All => 3,
        V::BounceDraw => 4,
        V::SelfTarget => 5,
    }
}

fn crit_policy(value: crate::generated::crit_policy::CritPolicy) -> u8 {
    use crate::generated::crit_policy::CritPolicy as V;
    match value {
        V::PerTarget => 0,
        V::Shared => 1,
        V::Never => 2,
    }
}

pub(super) fn positive(value: i32, field: &str) -> Result<u32, CatalogLoadError> {
    u32::try_from(value)
        .ok()
        .filter(|value| *value != 0)
        .ok_or_else(|| {
            fail(
                CatalogLoadErrorKind::Domain,
                format!("{field} must be positive"),
            )
        })
}

pub(super) fn positive_u16(value: i32, field: &str) -> Result<u16, CatalogLoadError> {
    let value = bounded_u16(value, field)?;
    if value == 0 {
        return Err(fail(
            CatalogLoadErrorKind::Domain,
            format!("{field} must be positive"),
        ));
    }
    Ok(value)
}

fn bounded_u16(value: i32, field: &str) -> Result<u16, CatalogLoadError> {
    u16::try_from(value).map_err(|_| {
        fail(
            CatalogLoadErrorKind::Domain,
            format!("{field} is outside the domain range"),
        )
    })
}

pub(super) fn contiguous(
    values: impl Iterator<Item = u16>,
    description: &str,
) -> Result<(), CatalogLoadError> {
    for (index, value) in values.enumerate() {
        if value as usize != index + 1 {
            return Err(fail(
                CatalogLoadErrorKind::Domain,
                format!("{description} are not contiguous from one"),
            ));
        }
    }
    Ok(())
}

pub(super) fn parse_decimal(source: &str) -> Result<i64, CatalogLoadError> {
    let (negative, unsigned) = source
        .strip_prefix('-')
        .map_or((false, source), |rest| (true, rest));
    if unsigned.is_empty() || (negative && unsigned == "0") {
        return Err(decimal_error(source));
    }
    let mut parts = unsigned.split('.');
    let integer = parts.next().expect("split always has one part");
    let fraction = parts.next();
    if parts.next().is_some()
        || integer.is_empty()
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || (integer.len() > 1 && integer.starts_with('0'))
        || fraction.is_some_and(|value| {
            value.is_empty()
                || value.len() > 6
                || !value.bytes().all(|byte| byte.is_ascii_digit())
                || value.ends_with('0')
        })
    {
        return Err(decimal_error(source));
    }
    let integer = integer.parse::<i128>().map_err(|_| decimal_error(source))?;
    let fraction_text = fraction.unwrap_or("");
    let fraction_value = if fraction_text.is_empty() {
        0
    } else {
        fraction_text
            .parse::<i128>()
            .map_err(|_| decimal_error(source))?
            * 10_i128.pow(6 - u32::try_from(fraction_text.len()).expect("length is at most six"))
    };
    let magnitude = integer
        .checked_mul(1_000_000)
        .and_then(|value| value.checked_add(fraction_value))
        .ok_or_else(|| decimal_error(source))?;
    let scaled = if negative {
        magnitude
            .checked_neg()
            .ok_or_else(|| decimal_error(source))?
    } else {
        magnitude
    };
    i64::try_from(scaled).map_err(|_| decimal_error(source))
}

fn decimal_error(source: &str) -> CatalogLoadError {
    fail(
        CatalogLoadErrorKind::Domain,
        format!("{source:?} is not a canonical six-place decimal"),
    )
}

pub(super) fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    let number =
        |range: std::ops::Range<usize>| value.get(range).and_then(|part| part.parse::<u16>().ok());
    matches!(number(0..4), Some(1..=9999))
        && matches!(number(5..7), Some(1..=12))
        && matches!(number(8..10), Some(1..=31))
}

pub(super) fn fail(
    kind: CatalogLoadErrorKind,
    message: impl std::fmt::Display,
) -> CatalogLoadError {
    CatalogLoadError {
        kind,
        message: message.to_string(),
    }
}

pub(super) fn domain_fail(message: impl std::fmt::Display) -> CatalogLoadError {
    fail(CatalogLoadErrorKind::Domain, message)
}

#[cfg(test)]
#[path = "catalog_tests.rs"]
mod tests;
