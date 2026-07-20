//! Validated Sora-row to immutable Starclock catalog boundary.
//! Generated rows and preliminary definition storage remain private.
use sha2::{Digest, Sha256};
use starclock_combat::modifier::registry::ModifierRegistry;
use starclock_combat::{
    AbilityId, DispelCategory, DurationClock, EffectCategory, EffectDefinitionId,
    EffectRuntimeTemplate, EffectSnapshotPolicy, EffectStackPolicy, EffectTeardownPolicy,
    EffectTickPhase, ModifierDefinitionId, Ratio, RuleId, SourceDefinitionId,
};
use std::{collections::BTreeMap, sync::Arc};

use crate::catalog_manifest::convert_manifest;
pub(super) use crate::catalog_support::{
    domain_fail, fail, parse_decimal, valid_date, valid_sha256,
};
use crate::coverage::{GoalCoverageCategory, GoalCoverageState};
use crate::effect_lower::{
    lower_dispel, lower_duration_clock, lower_effect_category, lower_element,
    lower_snapshot_policy, lower_stack_policy, lower_teardown, lower_tick_phase,
};
use crate::generated::{
    SoraConfig, content_kind::ContentKind, coverage_state::CoverageState,
    release_state::ReleaseState, runtime::SoraBundle,
};

mod hit_formula;
mod validation;
mod value_map;

use hit_formula::AbilityHitPlanDefinition;
use validation::{bounded_u16, identity_kind};
pub(super) use validation::{contiguous, positive, positive_u16, require_identity};
use value_map::{
    ability_kind, ability_resource_kind, crit_policy, hit_damage_class, hit_element,
    hit_scaling_stat, hit_target_group, resource_delta_kind, resource_timing, retarget_policy,
    target_pattern,
};

const METADATA_TABLES: [&str; 5] = [
    "ConfigManifest",
    "ContentEvidenceBinding",
    "ContentIdentity",
    "EvidenceRecord",
    "SourceRecord",
];
const LOWERED_TABLES: [&str; 68] = [
    "Ability",
    "AbilityHitPlanBinding",
    "AbilityLevelParameter",
    "AbilityPhase",
    "AbilityResourceDelta",
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
    "CharacterResource",
    "CharacterStat",
    "ConditionExpression",
    "CountdownDefinition",
    "Effect",
    "EffectModifierBinding",
    "EffectRuleBinding",
    "Eidolon",
    "EidolonPatch",
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
    "LinkedUnitDefinition",
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
    "TraceNode",
    "TracePatch",
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
    pub(super) kind: CatalogLoadErrorKind,
    pub(super) message: String,
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
    pub(super) combat_catalog: Arc<starclock_combat::catalog::CombatCatalog>,
    pub(super) build_catalog: starclock_build::catalog::BuildCatalog,
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
    pub(super) selectors: Box<[crate::selector_lower::SelectorDataDefinition]>,
    pub(super) programs: Box<[crate::operation_lower::RuleProgramDefinition]>,
    pub(super) effects: Box<[EffectDataDefinition]>,
    pub(super) rules: Box<[crate::rule_lower::RuleDataDefinition]>,
    pub(super) linked_units: Box<[starclock_combat::LinkedUnitCatalogDefinition]>,
    pub(super) countdowns: Box<[starclock_combat::CountdownCatalogDefinition]>,
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
    rules: Box<[RuleId]>,
    modifiers: Box<[ModifierDefinitionId]>,
    runtime_template: EffectRuntimeTemplate,
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
    #[must_use]
    pub fn rules(&self) -> &[RuleId] {
        &self.rules
    }
    #[must_use]
    pub fn modifiers(&self) -> &[ModifierDefinitionId] {
        &self.modifiers
    }
    #[must_use]
    pub const fn runtime_template(&self) -> &EffectRuntimeTemplate {
        &self.runtime_template
    }
}

#[derive(Debug)]
pub(super) struct AbilityDefinition {
    pub(super) id: AbilityId,
    pub(super) kind: u8,
    pub(super) target_pattern: u8,
    pub(super) retarget_policy: u8,
    pub(super) level_cap: u16,
    pub(super) cooldown_actions: u16,
    pub(super) semantic_tags: starclock_combat::catalog::action::AbilityTags,
    pub(super) entry_rule: Option<starclock_combat::RuleId>,
    pub(super) phases: Box<[AbilityPhaseDefinition]>,
    pub(super) hit_plan_bindings: Box<[AbilityHitPlanDefinition]>,
    pub(super) resources: Box<[AbilityResourceDefinition]>,
}

#[derive(Debug)]
pub(super) struct AbilityPhaseDefinition {
    pub(super) sequence: u16,
    pub(super) kind: u8,
    pub(super) program: Option<starclock_combat::ProgramId>,
}

#[derive(Debug)]
pub(super) struct AbilityResourceDefinition {
    pub(super) sequence: u16,
    pub(super) resource_kind: u8,
    pub(super) character_resource_key: Option<Box<str>>,
    pub(super) delta_kind: u8,
    pub(super) timing: u8,
    pub(super) amount: starclock_combat::Scalar,
}

#[derive(Debug)]
pub(super) struct HitPlanDefinition {
    pub(super) id: u32,
    pub(super) target_pattern: u8,
    pub(super) retarget_policy: u8,
    pub(super) hits: Box<[HitDefinition]>,
}

#[derive(Debug)]
pub(super) struct HitDefinition {
    pub(super) sequence: u16,
    pub(super) target_group: u8,
    pub(super) damage_ratio: Ratio,
    pub(super) toughness_ratio: Ratio,
    pub(super) crit_policy: u8,
    pub(super) damage_parameter_key_override: Option<Box<str>>,
    pub(super) damage_operation_ratio: Option<Ratio>,
    pub(super) toughness_amount: Option<starclock_combat::Scalar>,
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
    let config_digest: [u8; 32] = Sha256::digest(bytes).into();
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
    let (combat_catalog, build_catalog) = crate::domain_catalog::compile(
        &manifest.data_revision,
        config_digest,
        &identities,
        &combat,
        &builds,
        &encounters,
        mode,
    )?;
    let catalog = SimulationCatalog {
        manifest,
        identities: identities.into_boxed_slice(),
        combat,
        builds,
        encounters,
        standard,
        combat_catalog,
        build_catalog,
    };
    validate_converted_catalog(&catalog)?;
    Ok(Arc::new(catalog))
}

fn validate_converted_catalog(catalog: &SimulationCatalog) -> Result<(), CatalogLoadError> {
    if catalog.combat.abilities.iter().any(|ability| {
        ability.kind > 13
            || ability.target_pattern > 7
            || ability.retarget_policy > 3
            || ability.level_cap == 0
            || ability.cooldown_actions > 100
            || ability.phases.is_empty()
            || ability
                .phases
                .iter()
                .any(|phase| phase.program.is_some_and(|program| program.get() == 0))
            || ability.resources.iter().any(|resource| {
                resource.sequence == 0
                    || resource.resource_kind > 4
                    || resource.delta_kind > 2
                    || resource.timing > 4
                    || resource.amount.scaled() <= 0
                    || (resource.resource_kind == 3) != resource.character_resource_key.is_some()
            })
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
            // Phase 0 assigned the frozen Goal 01 denominator the permanent
            // transport IDs 1..=283. Runtime support identities may share a
            // denominator content kind (for example a transformation-owned
            // unit definition), but they are not additional manifest entries.
            goal_category: (id <= 283)
                .then(|| GoalCoverageCategory::from_content_kind(row.content_kind))
                .flatten(),
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
        let category = lower_effect_category(row.category);
        let dispel = lower_dispel(row.dispel_category);
        let tick_phase = lower_tick_phase(row.tick_phase);
        let stack_policy = lower_stack_policy(row.stack_policy);
        let snapshot_policy = lower_snapshot_policy(row.snapshot_policy);
        let teardown_policy = lower_teardown(row.teardown_policy);
        let dot_element = row.dot_element.map(lower_element);
        let detonation_tag = row
            .detonation_tag_identity_id
            .map(|id| {
                let raw = positive(id, "Effect.detonation_tag_identity_id")?;
                require_identity(identities, raw, IdentityKind::Other, mode)?;
                Ok(SourceDefinitionId::new(raw).expect("positive source definition ID"))
            })
            .transpose()?;
        if category != EffectCategory::Dot && (dot_element.is_some() || detonation_tag.is_some()) {
            return Err(fail(
                CatalogLoadErrorKind::Domain,
                format!("non-DoT effect {} declares DoT metadata", row.id),
            ));
        }
        let mut runtime_template = EffectRuntimeTemplate::new(
            category,
            dispel,
            bounded_u16(row.stack_limit, "Effect.stack_limit")?,
            duration.clone(),
            duration_clock,
            tick_phase,
            stack_policy,
        )
        .ok_or_else(|| {
            fail(
                CatalogLoadErrorKind::Domain,
                format!("effect {} has invalid runtime metadata", row.id),
            )
        })?
        .with_comparison(magnitude.clone(), row.application_priority)
        .with_snapshot(snapshot_policy)
        .with_teardown(teardown_policy);
        if category == EffectCategory::Dot {
            runtime_template = runtime_template
                .with_dot(
                    dot_element.ok_or_else(|| {
                        fail(
                            CatalogLoadErrorKind::Domain,
                            format!("DoT effect {} is missing its element", row.id),
                        )
                    })?,
                    detonation_tag,
                )
                .expect("DoT category accepts DoT metadata");
        }
        let mut rules = config
            .effect_rule_binding()
            .iter()
            .filter(|binding| binding.effect_id == row.id)
            .collect::<Vec<_>>();
        rules.sort_unstable_by_key(|binding| binding.sequence);
        contiguous(
            rules
                .iter()
                .map(|binding| positive_u16(binding.sequence, "EffectRuleBinding.sequence"))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter(),
            "effect rule bindings",
        )?;
        let rules = rules
            .into_iter()
            .map(|binding| {
                positive(binding.rule_id, "EffectRuleBinding.rule_id")
                    .map(|id| RuleId::new(id).expect("positive rule ID"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut modifiers = config
            .effect_modifier_binding()
            .iter()
            .filter(|binding| binding.effect_id == row.id)
            .collect::<Vec<_>>();
        modifiers.sort_unstable_by_key(|binding| binding.sequence);
        contiguous(
            modifiers
                .iter()
                .map(|binding| positive_u16(binding.sequence, "EffectModifierBinding.sequence"))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter(),
            "effect modifier bindings",
        )?;
        let modifiers = modifiers
            .into_iter()
            .map(|binding| {
                positive(binding.modifier_id, "EffectModifierBinding.modifier_id")
                    .map(|id| ModifierDefinitionId::new(id).expect("positive modifier ID"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        effects.push(EffectDataDefinition {
            id: EffectDefinitionId::new(raw).expect("positive effect ID"),
            category,
            dispel,
            stack_limit: bounded_u16(row.stack_limit, "Effect.stack_limit")?,
            duration,
            duration_clock,
            tick_phase,
            stack_policy,
            magnitude,
            snapshot_policy,
            teardown_policy,
            application_priority: row.application_priority,
            tags: tags
                .into_iter()
                .map(|tag| tag.tag.clone().into_boxed_str())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            rules: rules.into_boxed_slice(),
            modifiers: modifiers.into_boxed_slice(),
            runtime_template,
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
        let entry_rule = row
            .entry_rule_identity_id
            .map(|id| {
                starclock_combat::RuleId::new(positive(id, "Ability.entry_rule_identity_id")?)
                    .ok_or_else(|| domain_fail("ability entry rule ID is zero"))
            })
            .transpose()?;
        let mut phases = config
            .ability_phase()
            .iter()
            .filter(|phase| phase.ability_id == row.id)
            .map(|phase| {
                Ok(AbilityPhaseDefinition {
                    sequence: positive_u16(phase.sequence, "AbilityPhase.sequence")?,
                    kind: match phase.kind {
                        crate::generated::ability_phase_kind::AbilityPhaseKind::Entry => 0,
                        crate::generated::ability_phase_kind::AbilityPhaseKind::BeforeHits => 1,
                        crate::generated::ability_phase_kind::AbilityPhaseKind::Hits => 2,
                        crate::generated::ability_phase_kind::AbilityPhaseKind::AfterHits => 3,
                        crate::generated::ability_phase_kind::AbilityPhaseKind::Resolved => 4,
                    },
                    program: phase
                        .program_identity_id
                        .map(|id| {
                            starclock_combat::ProgramId::new(positive(
                                id,
                                "AbilityPhase.program_identity_id",
                            )?)
                            .ok_or_else(|| domain_fail("ability phase program ID is zero"))
                        })
                        .transpose()?,
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
        let mut resources = config
            .ability_resource_delta()
            .iter()
            .filter(|resource| resource.ability_id == row.id)
            .map(|resource| {
                let sequence = positive_u16(resource.sequence, "AbilityResourceDelta.sequence")?;
                let amount =
                    starclock_combat::Scalar::from_scaled(parse_decimal(&resource.amount_decimal)?);
                if amount.scaled() <= 0 {
                    return Err(domain_fail("ability resource amount must be positive"));
                }
                let resource_kind = ability_resource_kind(resource.resource_kind);
                let key = resource
                    .character_resource_key
                    .as_ref()
                    .map(|value| value.clone().into_boxed_str());
                if (resource_kind == 3) != key.is_some()
                    || key.as_ref().is_some_and(|value| value.trim().is_empty())
                {
                    return Err(domain_fail(
                        "character-resource delta requires exactly one nonempty key",
                    ));
                }
                Ok(AbilityResourceDefinition {
                    sequence,
                    resource_kind,
                    character_resource_key: key,
                    delta_kind: resource_delta_kind(resource.delta_kind),
                    timing: resource_timing(resource.timing),
                    amount,
                })
            })
            .collect::<Result<Vec<_>, CatalogLoadError>>()?;
        resources.sort_unstable_by_key(|resource| resource.sequence);
        contiguous(
            resources.iter().map(|resource| resource.sequence),
            "ability resource deltas",
        )?;
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
            entry_rule,
            phases: phases.into_boxed_slice(),
            hit_plan_bindings: Box::new([]),
            resources: resources.into_boxed_slice(),
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
                    damage_parameter_key_override: hit
                        .damage_parameter_key_override
                        .as_deref()
                        .map(Into::into),
                    damage_operation_ratio: hit
                        .damage_operation_ratio_decimal
                        .as_deref()
                        .map(parse_decimal)
                        .transpose()?
                        .map(Ratio::from_scaled),
                    toughness_amount: hit
                        .toughness_amount_decimal
                        .as_deref()
                        .map(parse_decimal)
                        .transpose()?
                        .map(starclock_combat::Scalar::from_scaled),
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

    let mut bound_plans = BTreeMap::<AbilityId, Vec<AbilityHitPlanDefinition>>::new();
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
        let ability = ability.expect("validated ability binding");
        let plan = plan.expect("validated hit-plan binding");
        let damage_fields = [
            binding.damage_parameter_key.is_some(),
            binding.damage_scaling_stat.is_some(),
            binding.damage_class.is_some(),
        ];
        let has_damage = damage_fields.iter().all(|present| *present);
        let has_hit_damage = plan.hits.iter().any(|hit| {
            hit.damage_parameter_key_override.is_some() || hit.damage_operation_ratio.is_some()
        });
        let has_hit_toughness = plan.hits.iter().any(|hit| hit.toughness_amount.is_some());
        if (damage_fields.iter().any(|present| *present) && !has_damage)
            || (has_hit_damage && !has_damage)
            || (has_damage || binding.base_toughness_decimal.is_some() || has_hit_toughness)
                && binding.element.is_none()
        {
            return Err(domain_fail("ability hit formula binding is incomplete"));
        }
        bound_plans
            .entry(ability.id)
            .or_default()
            .push(AbilityHitPlanDefinition {
                phase_sequence: phase,
                hit_plan_id: plan.id,
                damage_parameter_key: binding.damage_parameter_key.as_deref().map(Into::into),
                damage_scaling_stat: binding.damage_scaling_stat.map(hit_scaling_stat),
                damage_class: binding.damage_class.map(hit_damage_class).transpose()?,
                element: binding.element.map(hit_element),
                base_toughness: binding
                    .base_toughness_decimal
                    .as_deref()
                    .map(parse_decimal)
                    .transpose()?
                    .map(starclock_combat::Scalar::from_scaled),
            });
    }
    for ability in &mut abilities {
        let Some(mut bindings) = bound_plans.remove(&ability.id) else {
            continue;
        };
        bindings.sort_unstable_by_key(|binding| binding.phase_sequence);
        if bindings
            .windows(2)
            .any(|pair| pair[0].phase_sequence == pair[1].phase_sequence)
        {
            return Err(domain_fail("ability phase has duplicate hit-plan bindings"));
        }
        ability.hit_plan_bindings = bindings.into_boxed_slice();
    }
    let native_handlers = crate::native_handler_lower::audit(config)?;
    let modifiers = crate::modifier_lower::convert(config)?;
    let selectors = config
        .selector()
        .ordered_rows()
        .map(crate::selector_lower::lower)
        .collect::<Result<Vec<_>, _>>()?;
    let programs = crate::operation_lower::convert(config, &native_handlers)?;
    let rules = crate::rule_lower::convert(config, mode, identities, &native_handlers)?;
    let (linked_units, countdowns) = crate::lifecycle_lower::lower(config, identities, mode)?;
    for ability in &abilities {
        if ability
            .entry_rule
            .is_some_and(|id| rules.binary_search_by_key(&id, |rule| rule.id).is_err())
            || ability.phases.iter().any(|phase| {
                phase.program.is_some_and(|id| {
                    programs
                        .binary_search_by_key(&id, |program| program.id)
                        .is_err()
                })
            })
        {
            return Err(domain_fail(format!(
                "ability {} refers to a missing entry rule or phase program",
                ability.id.get()
            )));
        }
    }
    Ok(CombatDefinitions {
        abilities: abilities.into_boxed_slice(),
        hit_plans: hit_plans.into_boxed_slice(),
        modifiers,
        selectors: selectors.into_boxed_slice(),
        programs: programs.into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        rules: rules.into_boxed_slice(),
        linked_units,
        countdowns,
    })
}

#[cfg(test)]
#[path = "catalog_tests.rs"]
mod tests;
