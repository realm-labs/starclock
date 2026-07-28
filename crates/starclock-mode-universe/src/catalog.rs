//! Exact isolated-bundle loading and composed catalog identity.

use std::sync::Arc;

use crate::curio::{CurioDefinition, CurioStateDefinition};
use crate::definition::{
    DifficultyDefinition, DomainDefinition, RoomDefinition, TopologyDefinition,
    UniverseActivityBindingDefinition, UniverseDefinitions, UniverseProfileDefinition,
    WorldDefinition,
};
use crate::digest::{
    ActivityConfigurationDigest, Encoder, UniverseBundleDigest, UniverseCurioDefinitionsDigest,
    UniverseDefinitionsDigest, UniverseEncounterDefinitionsDigest, UniversePathDefinitionsDigest,
    UniverseProfileDigest, UniverseRunDefinitionsDigest, bundle_digest,
};
use crate::encounter::{
    ContentPoolDefinition, DifficultyEnemyBinding, EncounterGroupDefinition,
    EncounterPoolDefinition, RoomContentBinding,
};
use crate::error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind};
use crate::generated::{SoraConfig, runtime::SoraBundle};
use crate::id::{
    AbilityTreeNodeId, BlessingId, BlessingLevelId, ContentPoolId, CurioId, CurioStateId,
    DifficultyId, DomainId, EncounterGroupId, EncounterPoolId, MechanicRuleId, OccurrenceChoiceId,
    OccurrenceId, OccurrenceVariantId, PathId, ResonanceId, RoomId, ServiceId, TopologyId, WorldId,
};
use crate::occurrence::{
    OccurrenceChoiceDefinition, OccurrenceDefinition, OccurrenceVariantDefinition,
};
use crate::path::{
    BlessingDefinition, BlessingLevelDefinition, PathDefinition, ResonanceDefinition,
};
use crate::progression::{AbilityTreeNodeDefinition, ServiceDefinition};
use crate::rule::MechanicRuleDefinition;

pub const UNIVERSE_CATALOG_REVISION: &str = "standard-universe-v4.4-runtime-v3";
pub const STANDARD_UNIVERSE_PROFILE_REVISION: &str = "standard-universe-main-world-v1";
pub const ACTIVITY_CONFIGURATION_REVISION: &str = "starclock-activity-config-v1";

const EXPECTED_PROFILE_KEY: &str = "universe.profile.standard-main-world.v4.4";
const EXPECTED_GAME_VERSION: &str = "4.4";
const EXPECTED_SNAPSHOT_DATE: &str = "2026-07-22";
const EXPECTED_CONTENT_MANIFEST: &str =
    "1dac0f8102a8c2a77717a37d206e2288f38fda8d428e490cdd91177190bce216";
const EXPECTED_PACK_DIGEST: &str =
    "e620b9e4371f41afb43967a0537f8e705a33ae70f6e8ede0468b80f3444cd933";
const EXPECTED_CORE_DATA_REVISION: &str = "core-combat-v1-phase7-l11";
const EXPECTED_CORE_RULES_REVISION: &str = "core-combat-rules-v1";
const EXPECTED_NUMERIC_REVISION: &str = "fixed-i64-6dp-v1";
const EXPECTED_RNG_REVISION: &str = "chacha8-rand-0.10.2-intmap-v1";
const EXPECTED_STATE_HASH_REVISION: &str = "sha256-v5";

const EXPECTED_CORE_BUNDLE: [u8; 32] = [
    0x67, 0x69, 0xf2, 0xf6, 0x6c, 0x22, 0x13, 0x58, 0x2e, 0x58, 0x15, 0x1f, 0x72, 0x9f, 0x8c, 0xb5,
    0x14, 0x41, 0x76, 0x58, 0xa5, 0x51, 0x37, 0x9e, 0x1b, 0xc4, 0x1b, 0x62, 0xec, 0x5b, 0x66, 0x15,
];
const EXPECTED_UNIVERSE_BUNDLE: UniverseBundleDigest = UniverseBundleDigest::new([
    0x5e, 0x52, 0x34, 0xee, 0x39, 0x77, 0xf7, 0x94, 0xae, 0x9b, 0x1b, 0x83, 0x33, 0x72, 0xf5, 0x1c,
    0x38, 0x40, 0x8c, 0x20, 0x51, 0x05, 0xc4, 0x64, 0xf1, 0x18, 0x27, 0xe9, 0xe9, 0xae, 0x6a, 0x75,
]);

/// Generated-row-free compatibility identity for one catalog composition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniverseCatalogIdentity {
    game_version: Box<str>,
    snapshot_date: Box<str>,
    core_data_revision: Box<str>,
    catalog_revision: Box<str>,
    profile_revision: Box<str>,
    core_bundle: [u8; 32],
    build_catalog: [u8; 32],
    universe_bundle: UniverseBundleDigest,
    profile: UniverseProfileDigest,
    definitions: UniverseDefinitionsDigest,
    path_definitions: UniversePathDefinitionsDigest,
    curio_definitions: UniverseCurioDefinitionsDigest,
    run_definitions: UniverseRunDefinitionsDigest,
    encounter_definitions: UniverseEncounterDefinitionsDigest,
    configuration: ActivityConfigurationDigest,
}

impl UniverseCatalogIdentity {
    #[must_use]
    pub fn game_version(&self) -> &str {
        &self.game_version
    }
    #[must_use]
    pub fn snapshot_date(&self) -> &str {
        &self.snapshot_date
    }
    #[must_use]
    pub fn core_data_revision(&self) -> &str {
        &self.core_data_revision
    }
    #[must_use]
    pub fn catalog_revision(&self) -> &str {
        &self.catalog_revision
    }
    #[must_use]
    pub fn profile_revision(&self) -> &str {
        &self.profile_revision
    }
    #[must_use]
    pub const fn core_bundle_digest(&self) -> [u8; 32] {
        self.core_bundle
    }
    #[must_use]
    pub const fn build_catalog_digest(&self) -> [u8; 32] {
        self.build_catalog
    }
    #[must_use]
    pub const fn universe_bundle_digest(&self) -> UniverseBundleDigest {
        self.universe_bundle
    }
    #[must_use]
    pub const fn profile_digest(&self) -> UniverseProfileDigest {
        self.profile
    }
    #[must_use]
    pub const fn definitions_digest(&self) -> UniverseDefinitionsDigest {
        self.definitions
    }
    #[must_use]
    pub const fn path_definitions_digest(&self) -> UniversePathDefinitionsDigest {
        self.path_definitions
    }
    #[must_use]
    pub const fn curio_definitions_digest(&self) -> UniverseCurioDefinitionsDigest {
        self.curio_definitions
    }
    #[must_use]
    pub const fn run_definitions_digest(&self) -> UniverseRunDefinitionsDigest {
        self.run_definitions
    }
    #[must_use]
    pub const fn encounter_definitions_digest(&self) -> UniverseEncounterDefinitionsDigest {
        self.encounter_definitions
    }
    #[must_use]
    pub const fn configuration_digest(&self) -> ActivityConfigurationDigest {
        self.configuration
    }
}

/// Aggregate counts validated before later domain lowering batches.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UniverseCatalogSummary {
    pub worlds: usize,
    pub paths: usize,
    pub difficulties: usize,
    pub content_records: usize,
    pub mechanic_rules: usize,
    pub semantic_fixtures: usize,
    pub source_records: usize,
}

/// Immutable validated Universe transport/catalog aggregate.
#[derive(Debug)]
pub struct UniverseCatalog {
    identity: UniverseCatalogIdentity,
    summary: UniverseCatalogSummary,
    definitions: UniverseDefinitions,
    transport: SoraConfig,
    core: Arc<starclock_data::catalog::SimulationCatalog>,
}

impl UniverseCatalog {
    /// Loads only the exact Goal 03 bundle and composes it with the exact Goal 01 catalog.
    pub fn load(
        universe_bundle: &[u8],
        core: Arc<starclock_data::catalog::SimulationCatalog>,
    ) -> Result<Arc<Self>, UniverseCatalogLoadError> {
        let actual_digest = bundle_digest(universe_bundle);
        if actual_digest != EXPECTED_UNIVERSE_BUNDLE {
            return Err(error(
                UniverseCatalogLoadErrorKind::UniverseBundleDigest,
                "Universe bundle SHA-256 does not match the reviewed Goal 07 catalog revision",
            ));
        }
        validate_core(&core)?;
        let transport = decode(universe_bundle)?;
        let profile = only_profile(&transport)?;
        validate_profile(profile)?;
        let summary = validate_counts(&transport, profile)?;
        let definitions = crate::lowering::lower(&transport)?;
        let profile_digest = profile_digest(profile);
        let build_digest = core.build_catalog().digest().bytes();
        let configuration = compose_configuration(
            core.combat_catalog().digest().bytes(),
            core.combat_catalog().revision().as_str(),
            build_digest,
            core.build_catalog().revision().as_str(),
            actual_digest,
            profile_digest,
        );
        let identity = UniverseCatalogIdentity {
            game_version: profile.game_version.as_str().into(),
            snapshot_date: profile.snapshot_date.as_str().into(),
            core_data_revision: core.manifest().data_revision.as_str().into(),
            catalog_revision: UNIVERSE_CATALOG_REVISION.into(),
            profile_revision: STANDARD_UNIVERSE_PROFILE_REVISION.into(),
            core_bundle: core.combat_catalog().digest().bytes(),
            build_catalog: build_digest,
            universe_bundle: actual_digest,
            profile: profile_digest,
            definitions: definitions.digest,
            path_definitions: definitions.path_digest,
            curio_definitions: definitions.curio_digest,
            run_definitions: definitions.run_digest,
            encounter_definitions: definitions.encounter_digest,
            configuration,
        };
        Ok(Arc::new(Self {
            identity,
            summary,
            definitions,
            transport,
            core,
        }))
    }

    #[must_use]
    pub const fn identity(&self) -> &UniverseCatalogIdentity {
        &self.identity
    }

    #[must_use]
    pub const fn summary(&self) -> UniverseCatalogSummary {
        self.summary
    }

    #[must_use]
    pub fn simulation_catalog(&self) -> &starclock_data::catalog::SimulationCatalog {
        &self.core
    }

    #[must_use]
    pub const fn profile(&self) -> &UniverseProfileDefinition {
        &self.definitions.profile
    }

    #[must_use]
    pub fn worlds(&self) -> &[WorldDefinition] {
        &self.definitions.worlds
    }

    #[must_use]
    pub fn world(&self, id: WorldId) -> Option<&WorldDefinition> {
        lookup(&self.definitions.worlds, id, WorldDefinition::id)
    }

    #[must_use]
    pub fn difficulties(&self) -> &[DifficultyDefinition] {
        &self.definitions.difficulties
    }

    #[must_use]
    pub fn difficulty(&self, id: DifficultyId) -> Option<&DifficultyDefinition> {
        lookup(&self.definitions.difficulties, id, DifficultyDefinition::id)
    }

    #[must_use]
    pub fn domains(&self) -> &[DomainDefinition] {
        &self.definitions.domains
    }

    #[must_use]
    pub fn domain(&self, id: DomainId) -> Option<&DomainDefinition> {
        lookup(&self.definitions.domains, id, DomainDefinition::id)
    }

    #[must_use]
    pub fn topologies(&self) -> &[TopologyDefinition] {
        &self.definitions.topologies
    }

    #[must_use]
    pub fn topology(&self, id: TopologyId) -> Option<&TopologyDefinition> {
        lookup(&self.definitions.topologies, id, TopologyDefinition::id)
    }

    #[must_use]
    pub fn rooms(&self) -> &[RoomDefinition] {
        &self.definitions.rooms
    }

    #[must_use]
    pub fn room(&self, id: RoomId) -> Option<&RoomDefinition> {
        lookup(&self.definitions.rooms, id, RoomDefinition::id)
    }

    #[must_use]
    pub const fn activity_binding(&self) -> &UniverseActivityBindingDefinition {
        &self.definitions.activity
    }

    #[must_use]
    pub fn paths(&self) -> &[PathDefinition] {
        &self.definitions.paths
    }

    #[must_use]
    pub fn path(&self, id: PathId) -> Option<&PathDefinition> {
        lookup(&self.definitions.paths, id, PathDefinition::id)
    }

    #[must_use]
    pub fn blessings(&self) -> &[BlessingDefinition] {
        &self.definitions.blessings
    }

    #[must_use]
    pub fn blessing(&self, id: BlessingId) -> Option<&BlessingDefinition> {
        lookup(&self.definitions.blessings, id, BlessingDefinition::id)
    }

    #[must_use]
    pub fn blessing_levels(&self) -> &[BlessingLevelDefinition] {
        &self.definitions.blessing_levels
    }

    #[must_use]
    pub fn blessing_level(&self, id: BlessingLevelId) -> Option<&BlessingLevelDefinition> {
        lookup(
            &self.definitions.blessing_levels,
            id,
            BlessingLevelDefinition::id,
        )
    }

    #[must_use]
    pub fn resonances(&self) -> &[ResonanceDefinition] {
        &self.definitions.resonances
    }

    #[must_use]
    pub fn resonance(&self, id: ResonanceId) -> Option<&ResonanceDefinition> {
        lookup(&self.definitions.resonances, id, ResonanceDefinition::id)
    }

    #[must_use]
    pub fn curios(&self) -> &[CurioDefinition] {
        &self.definitions.curios
    }

    #[must_use]
    pub fn curio(&self, id: CurioId) -> Option<&CurioDefinition> {
        lookup(&self.definitions.curios, id, CurioDefinition::id)
    }

    #[must_use]
    pub fn curio_states(&self) -> &[CurioStateDefinition] {
        &self.definitions.curio_states
    }

    #[must_use]
    pub fn curio_state(&self, id: CurioStateId) -> Option<&CurioStateDefinition> {
        lookup(&self.definitions.curio_states, id, CurioStateDefinition::id)
    }

    #[must_use]
    pub fn occurrences(&self) -> &[OccurrenceDefinition] {
        &self.definitions.occurrences
    }
    #[must_use]
    pub fn occurrence(&self, id: OccurrenceId) -> Option<&OccurrenceDefinition> {
        lookup(&self.definitions.occurrences, id, OccurrenceDefinition::id)
    }
    #[must_use]
    pub fn occurrence_variants(&self) -> &[OccurrenceVariantDefinition] {
        &self.definitions.occurrence_variants
    }
    #[must_use]
    pub fn occurrence_variant(
        &self,
        id: OccurrenceVariantId,
    ) -> Option<&OccurrenceVariantDefinition> {
        lookup(
            &self.definitions.occurrence_variants,
            id,
            OccurrenceVariantDefinition::id,
        )
    }
    #[must_use]
    pub fn occurrence_choices(&self) -> &[OccurrenceChoiceDefinition] {
        &self.definitions.occurrence_choices
    }
    #[must_use]
    pub fn occurrence_choice(&self, id: OccurrenceChoiceId) -> Option<&OccurrenceChoiceDefinition> {
        lookup(
            &self.definitions.occurrence_choices,
            id,
            OccurrenceChoiceDefinition::id,
        )
    }
    #[must_use]
    pub fn services(&self) -> &[ServiceDefinition] {
        &self.definitions.services
    }
    #[must_use]
    pub fn service(&self, id: ServiceId) -> Option<&ServiceDefinition> {
        lookup(&self.definitions.services, id, ServiceDefinition::id)
    }
    #[must_use]
    pub fn ability_tree_nodes(&self) -> &[AbilityTreeNodeDefinition] {
        &self.definitions.ability_tree_nodes
    }
    #[must_use]
    pub fn ability_tree_node(&self, id: AbilityTreeNodeId) -> Option<&AbilityTreeNodeDefinition> {
        lookup(
            &self.definitions.ability_tree_nodes,
            id,
            AbilityTreeNodeDefinition::id,
        )
    }

    #[must_use]
    pub fn encounter_groups(&self) -> &[EncounterGroupDefinition] {
        &self.definitions.encounter_groups
    }
    #[must_use]
    pub fn encounter_group(&self, id: EncounterGroupId) -> Option<&EncounterGroupDefinition> {
        lookup(
            &self.definitions.encounter_groups,
            id,
            EncounterGroupDefinition::id,
        )
    }
    #[must_use]
    pub fn difficulty_enemy_bindings(&self) -> &[DifficultyEnemyBinding] {
        &self.definitions.difficulty_enemies
    }
    #[must_use]
    pub fn encounter_pools(&self) -> &[EncounterPoolDefinition] {
        &self.definitions.encounter_pools
    }
    #[must_use]
    pub fn encounter_pool(&self, id: EncounterPoolId) -> Option<&EncounterPoolDefinition> {
        lookup(
            &self.definitions.encounter_pools,
            id,
            EncounterPoolDefinition::id,
        )
    }
    #[must_use]
    pub fn room_content(&self) -> &[RoomContentBinding] {
        &self.definitions.room_content
    }
    #[must_use]
    pub fn content_pools(&self) -> &[ContentPoolDefinition] {
        &self.definitions.content_pools
    }
    #[must_use]
    pub fn content_pool(&self, id: ContentPoolId) -> Option<&ContentPoolDefinition> {
        lookup(
            &self.definitions.content_pools,
            id,
            ContentPoolDefinition::id,
        )
    }
    #[must_use]
    pub fn mechanic_rules(&self) -> &[MechanicRuleDefinition] {
        &self.definitions.mechanic_rules
    }
    #[must_use]
    pub fn mechanic_rule(&self, id: MechanicRuleId) -> Option<&MechanicRuleDefinition> {
        lookup(
            &self.definitions.mechanic_rules,
            id,
            MechanicRuleDefinition::id,
        )
    }

    /// Returns the number of privately loaded Sora tables without exposing them.
    #[must_use]
    pub fn transport_table_count(&self) -> usize {
        self.transport.tables().count()
    }
}

fn lookup<T, I: Ord + Copy>(values: &[T], id: I, key: impl Fn(&T) -> I) -> Option<&T> {
    values
        .binary_search_by_key(&id, key)
        .ok()
        .map(|index| &values[index])
}

fn decode(bytes: &[u8]) -> Result<SoraConfig, UniverseCatalogLoadError> {
    let bundle = SoraBundle::parse(bytes).map_err(|value| {
        error(
            UniverseCatalogLoadErrorKind::BundleFormat,
            format!("Universe Sora envelope rejected: {value}"),
        )
    })?;
    SoraConfig::from_source(&bundle).map_err(|value| {
        error(
            UniverseCatalogLoadErrorKind::BundleFormat,
            format!("Universe Sora schema rejected: {value}"),
        )
    })
}

fn only_profile(
    config: &SoraConfig,
) -> Result<&crate::generated::universe_profile::UniverseProfile, UniverseCatalogLoadError> {
    let mut rows = config.universe_profile().ordered_rows();
    let profile = rows.next().ok_or_else(|| {
        error(
            UniverseCatalogLoadErrorKind::UniverseRevision,
            "Universe bundle has no profile",
        )
    })?;
    if rows.next().is_some() {
        return Err(error(
            UniverseCatalogLoadErrorKind::UniverseRevision,
            "Universe bundle has multiple profiles",
        ));
    }
    Ok(profile)
}

fn validate_profile(
    profile: &crate::generated::universe_profile::UniverseProfile,
) -> Result<(), UniverseCatalogLoadError> {
    let valid = profile.id == 1
        && profile.stable_key == EXPECTED_PROFILE_KEY
        && profile.game_version == EXPECTED_GAME_VERSION
        && profile.snapshot_date == EXPECTED_SNAPSHOT_DATE
        && profile.content_manifest_sha256 == EXPECTED_CONTENT_MANIFEST
        && profile.pack_sha256 == EXPECTED_PACK_DIGEST
        && profile.world_count == 9
        && profile.path_count == 9
        && profile.runtime_loading == "ForbiddenStagingOnly";
    if valid {
        Ok(())
    } else {
        Err(error(
            UniverseCatalogLoadErrorKind::UniverseRevision,
            "Universe profile identity/revision differs from the frozen Goal 03 release",
        ))
    }
}

fn validate_core(
    core: &starclock_data::catalog::SimulationCatalog,
) -> Result<(), UniverseCatalogLoadError> {
    let manifest = core.manifest();
    let valid = core.combat_catalog().digest().bytes() == EXPECTED_CORE_BUNDLE
        && manifest.game_version == EXPECTED_GAME_VERSION
        && manifest.data_revision == EXPECTED_CORE_DATA_REVISION
        && manifest.required_rules_revision == EXPECTED_CORE_RULES_REVISION
        && manifest.numeric_policy_revision == EXPECTED_NUMERIC_REVISION
        && manifest.rng_algorithm_revision == EXPECTED_RNG_REVISION
        && manifest.state_hash_revision == EXPECTED_STATE_HASH_REVISION
        && core.build_catalog().compatible_combat_digest().bytes() == EXPECTED_CORE_BUNDLE;
    if valid {
        Ok(())
    } else {
        Err(error(
            UniverseCatalogLoadErrorKind::CoreCompatibility,
            format!(
                "combat/build catalog identity is incompatible with Standard Universe v1: \
                 combat={:02x?}, build={:02x?}, state={}",
                core.combat_catalog().digest().bytes(),
                core.build_catalog().compatible_combat_digest().bytes(),
                manifest.state_hash_revision,
            ),
        ))
    }
}

fn validate_counts(
    config: &SoraConfig,
    profile: &crate::generated::universe_profile::UniverseProfile,
) -> Result<UniverseCatalogSummary, UniverseCatalogLoadError> {
    let coverage = config.universe_coverage();
    let content_records = coverage.ordered_rows().try_fold(0usize, |total, row| {
        if row.required != row.accounted
            || row.required != row.data_ready
            || row.coverage_percent_decimal != "100"
        {
            return Err(error(
                UniverseCatalogLoadErrorKind::Coverage,
                format!("Universe coverage category {} is incomplete", row.category),
            ));
        }
        let count = usize::try_from(row.required).map_err(|_| {
            error(
                UniverseCatalogLoadErrorKind::Coverage,
                "Universe coverage has a negative count",
            )
        })?;
        total.checked_add(count).ok_or_else(|| {
            error(
                UniverseCatalogLoadErrorKind::Coverage,
                "Universe coverage count overflow",
            )
        })
    })?;
    let summary = UniverseCatalogSummary {
        worlds: config.universe_world().len(),
        paths: config.universe_path().len(),
        difficulties: config.universe_difficulty().len(),
        content_records,
        mechanic_rules: config.universe_mechanic_rule().len(),
        semantic_fixtures: config.universe_review_fixture().len(),
        source_records: config.universe_source_record().len(),
    };
    let valid = summary.worlds == usize::try_from(profile.world_count).unwrap_or_default()
        && summary.paths == usize::try_from(profile.path_count).unwrap_or_default()
        && summary.difficulties == 33
        && summary.content_records == 2_201
        && config.universe_content_audit().len() == 2_201
        && summary.mechanic_rules == 786
        && summary.semantic_fixtures == 78
        && summary.source_records == 2_645
        && config.universe_pack_file().len() == 24;
    if valid {
        Ok(summary)
    } else {
        Err(error(
            UniverseCatalogLoadErrorKind::Coverage,
            "Universe release denominator differs from Goal 03",
        ))
    }
}

fn profile_digest(
    profile: &crate::generated::universe_profile::UniverseProfile,
) -> UniverseProfileDigest {
    let mut encoder = Encoder::new(b"starclock-standard-universe-profile-v1");
    encoder.u32(u32::try_from(profile.id).expect("validated positive profile ID"));
    for value in [
        &profile.stable_key,
        &profile.game_version,
        &profile.snapshot_date,
        &profile.content_manifest_sha256,
        &profile.pack_sha256,
        &profile.runtime_loading,
    ] {
        encoder.text(value);
    }
    encoder.u32(u32::try_from(profile.world_count).expect("validated world count"));
    encoder.u32(u32::try_from(profile.path_count).expect("validated path count"));
    UniverseProfileDigest::new(encoder.finish())
}

fn compose_configuration(
    combat_digest: [u8; 32],
    combat_revision: &str,
    build_digest: [u8; 32],
    build_revision: &str,
    universe_digest: UniverseBundleDigest,
    profile_digest: UniverseProfileDigest,
) -> ActivityConfigurationDigest {
    let mut encoder = Encoder::new(ACTIVITY_CONFIGURATION_REVISION.as_bytes());
    for (label, revision, digest) in [
        ("combat", combat_revision, combat_digest),
        ("build", build_revision, build_digest),
        (
            "universe",
            UNIVERSE_CATALOG_REVISION,
            universe_digest.bytes(),
        ),
        (
            "activity-profile",
            STANDARD_UNIVERSE_PROFILE_REVISION,
            profile_digest.bytes(),
        ),
    ] {
        encoder.text(label);
        encoder.text(revision);
        encoder.optional_digest(Some(digest));
    }
    ActivityConfigurationDigest::new(encoder.finish())
}

fn error(
    kind: UniverseCatalogLoadErrorKind,
    message: impl Into<Box<str>>,
) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(kind, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
    const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

    #[test]
    fn exact_isolated_bundle_composes_with_exact_core_catalog() {
        let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
        let catalog = UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog");
        assert_eq!(catalog.identity().game_version(), "4.4");
        assert_eq!(
            catalog.identity().catalog_revision(),
            UNIVERSE_CATALOG_REVISION
        );
        assert_eq!(
            catalog.identity().core_bundle_digest(),
            EXPECTED_CORE_BUNDLE
        );
        assert_eq!(
            catalog.identity().universe_bundle_digest(),
            EXPECTED_UNIVERSE_BUNDLE
        );
        assert_eq!(
            catalog.summary(),
            UniverseCatalogSummary {
                worlds: 9,
                paths: 9,
                difficulties: 33,
                content_records: 2_201,
                mechanic_rules: 786,
                semantic_fixtures: 78,
                source_records: 2_645,
            }
        );
        assert_eq!(catalog.transport_table_count(), 49);
        assert_eq!(catalog.worlds().len(), 9);
        assert_eq!(catalog.difficulties().len(), 33);
        assert_eq!(catalog.domains().len(), 9);
        assert_eq!(catalog.topologies().len(), 37);
        assert_eq!(catalog.rooms().len(), 163);
        assert_eq!(
            catalog
                .topologies()
                .iter()
                .map(|topology| topology.nodes().len())
                .sum::<usize>(),
            579
        );
        assert_eq!(
            catalog
                .topologies()
                .iter()
                .flat_map(|topology| topology.nodes())
                .map(|node| node.outgoing().len())
                .sum::<usize>(),
            707
        );
        for world in catalog.worlds() {
            assert_eq!(world.profile(), catalog.profile().id());
            for difficulty in world.difficulties() {
                assert_eq!(
                    catalog.difficulty(*difficulty).expect("difficulty").world(),
                    world.id()
                );
            }
        }
        for room in catalog.rooms() {
            assert!(catalog.domain(room.domain()).is_some());
        }
        assert_eq!(catalog.activity_binding().domains().len(), 9);
        assert_eq!(catalog.paths().len(), 9);
        assert_eq!(catalog.blessings().len(), 162);
        assert_eq!(catalog.blessing_levels().len(), 324);
        assert_eq!(catalog.resonances().len(), 36);
        assert_eq!(catalog.curios().len(), 61);
        assert_eq!(catalog.curio_states().len(), 67);
        assert_eq!(
            catalog
                .curio_states()
                .iter()
                .map(|value| value.parameters().len())
                .sum::<usize>(),
            89
        );
        assert_eq!(
            catalog
                .curio_states()
                .iter()
                .filter(|value| value.next_state().is_some())
                .count(),
            6
        );
        for curio in catalog.curios() {
            let initial = catalog
                .curio_state(curio.initial_state())
                .expect("Curio initial state");
            assert_eq!(initial.curio(), curio.id());
            assert!(!curio.states().is_empty());
        }
        assert_eq!(catalog.occurrences().len(), 59);
        assert_eq!(catalog.occurrence_variants().len(), 67);
        assert_eq!(catalog.occurrence_choices().len(), 321);
        assert_eq!(
            catalog
                .occurrence_choices()
                .iter()
                .map(|value| value.costs().len())
                .sum::<usize>(),
            23
        );
        assert_eq!(
            catalog
                .occurrence_choices()
                .iter()
                .map(|value| value.outcomes().len())
                .sum::<usize>(),
            321
        );
        assert_eq!(
            catalog
                .occurrence_choices()
                .iter()
                .flat_map(|value| value.outcomes())
                .filter(|value| value.random_policy().is_some())
                .count(),
            127
        );
        assert_eq!(catalog.services().len(), 94);
        assert_eq!(
            catalog
                .services()
                .iter()
                .map(|value| value.parameters().len())
                .sum::<usize>(),
            41
        );
        assert_eq!(catalog.ability_tree_nodes().len(), 42);
        assert_eq!(
            catalog
                .ability_tree_nodes()
                .iter()
                .map(|value| value.prerequisites().len())
                .sum::<usize>(),
            55
        );
        assert_eq!(
            catalog
                .ability_tree_nodes()
                .iter()
                .map(|value| value.effects().len())
                .sum::<usize>(),
            50
        );
        assert_eq!(
            catalog
                .ability_tree_nodes()
                .iter()
                .map(|value| value.parameters().len())
                .sum::<usize>(),
            43
        );
        assert_eq!(catalog.encounter_groups().len(), 74);
        assert_eq!(
            catalog
                .encounter_groups()
                .iter()
                .map(|value| value.members().len())
                .sum::<usize>(),
            173
        );
        assert_eq!(
            catalog
                .encounter_groups()
                .iter()
                .flat_map(|value| value.members())
                .flat_map(|value| value.waves())
                .map(|value| value.enemies().len())
                .sum::<usize>(),
            538
        );
        assert_eq!(catalog.difficulty_enemy_bindings().len(), 182);
        assert_eq!(catalog.encounter_pools().len(), 92);
        assert_eq!(
            catalog
                .encounter_pools()
                .iter()
                .map(|value| value.fixed().len())
                .sum::<usize>(),
            36
        );
        assert_eq!(
            catalog
                .encounter_pools()
                .iter()
                .map(|value| value.weighted().len())
                .sum::<usize>(),
            174
        );
        assert_eq!(catalog.room_content().len(), 380);
        assert_eq!(catalog.content_pools().len(), 23);
        assert_eq!(
            catalog
                .content_pools()
                .iter()
                .map(|value| value.entries().len())
                .sum::<usize>(),
            1_578
        );
        assert_eq!(catalog.mechanic_rules().len(), 786);
        assert_eq!(
            catalog
                .mechanic_rules()
                .iter()
                .map(|value| value.parameters().len())
                .sum::<usize>(),
            1_049
        );
        assert_eq!(
            catalog
                .blessing_levels()
                .iter()
                .map(|value| value.parameters().len())
                .sum::<usize>(),
            638
        );
        assert_eq!(
            catalog
                .resonances()
                .iter()
                .map(|value| value.parameters().len())
                .sum::<usize>(),
            238
        );
        assert_eq!(
            catalog
                .blessings()
                .iter()
                .map(|value| value.prerequisite_keys().len())
                .sum::<usize>(),
            72
        );
        assert!(
            catalog
                .blessing_levels()
                .iter()
                .flat_map(|value| value.parameters())
                .any(|value| value.scale() == 10)
        );
        for path in catalog.paths() {
            assert_eq!(path.blessings().len(), 18);
            assert_eq!(path.formations().len(), 3);
            assert!(catalog.resonance(path.resonance()).is_some());
        }
        assert_eq!(
            catalog.identity().path_definitions_digest().bytes(),
            [
                0x49, 0x1d, 0x76, 0xe6, 0xbd, 0xbb, 0x0a, 0x93, 0x2d, 0x20, 0x40, 0xa6, 0x52, 0xc0,
                0x84, 0xc8, 0x2b, 0x48, 0x86, 0x0f, 0xd8, 0x4f, 0xcf, 0xde, 0xa9, 0xd2, 0x14, 0x04,
                0xf9, 0xfb, 0x4b, 0xb5,
            ]
        );
        assert_eq!(
            catalog.identity().curio_definitions_digest().bytes(),
            [
                0xe9, 0xf1, 0xea, 0x70, 0xed, 0x8f, 0x85, 0x6f, 0xac, 0x31, 0xd1, 0x00, 0xeb, 0x5b,
                0x0e, 0xa1, 0x73, 0x7f, 0x63, 0x2d, 0x65, 0xbc, 0xec, 0x8d, 0x93, 0xee, 0x60, 0x95,
                0xb8, 0x53, 0x70, 0x72,
            ]
        );
        assert_eq!(
            catalog.identity().run_definitions_digest().bytes(),
            [
                177, 201, 219, 41, 126, 113, 135, 229, 217, 199, 16, 6, 127, 9, 190, 174, 85, 5,
                131, 151, 186, 98, 51, 201, 18, 45, 54, 160, 166, 42, 2, 110,
            ]
        );
        assert_eq!(
            catalog.identity().encounter_definitions_digest().bytes(),
            [
                24, 8, 19, 18, 29, 124, 171, 245, 125, 174, 194, 92, 122, 235, 253, 2, 66, 246,
                175, 124, 80, 75, 49, 186, 24, 229, 17, 171, 29, 178, 35, 26,
            ]
        );
        assert_eq!(
            catalog.identity().definitions_digest().bytes(),
            [
                120, 190, 196, 196, 192, 213, 164, 29, 203, 115, 129, 13, 40, 28, 227, 231, 104,
                126, 149, 212, 102, 134, 210, 15, 137, 244, 199, 104, 237, 30, 183, 242,
            ]
        );
        assert_eq!(
            catalog.identity().configuration_digest().bytes(),
            [
                19, 206, 53, 175, 240, 199, 253, 161, 133, 173, 3, 4, 29, 90, 48, 56, 66, 86, 154,
                169, 237, 112, 25, 253, 56, 35, 206, 230, 177, 178, 192, 93,
            ]
        );
        assert_eq!(
            catalog.identity().profile_digest().bytes(),
            [
                20, 52, 22, 185, 103, 86, 246, 218, 188, 110, 246, 17, 14, 147, 183, 141, 126, 27,
                228, 181, 134, 70, 233, 227, 34, 232, 22, 63, 73, 192, 164, 174,
            ]
        );
    }

    #[test]
    fn wrong_and_tampered_bundles_are_rejected_before_generated_rows_escape() {
        let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
        let wrong = UniverseCatalog::load(CORE_BUNDLE, Arc::clone(&core)).unwrap_err();
        assert_eq!(
            wrong.kind(),
            UniverseCatalogLoadErrorKind::UniverseBundleDigest
        );
        let mut tampered = UNIVERSE_BUNDLE.to_vec();
        *tampered.last_mut().expect("non-empty bundle") ^= 1;
        let tampered = UniverseCatalog::load(&tampered, core).unwrap_err();
        assert_eq!(
            tampered.kind(),
            UniverseCatalogLoadErrorKind::UniverseBundleDigest
        );
        let format = decode(br#"{"table":{}}"#).unwrap_err();
        assert_eq!(format.kind(), UniverseCatalogLoadErrorKind::BundleFormat);
    }

    #[test]
    fn revision_validation_is_explicit_after_transport_decoding() {
        let config = decode(UNIVERSE_BUNDLE).expect("Universe bundle");
        let profile = only_profile(&config).expect("profile");
        let mut changed = profile.clone();
        changed.game_version = "4.5".to_owned();
        let error = validate_profile(&changed).unwrap_err();
        assert_eq!(error.kind(), UniverseCatalogLoadErrorKind::UniverseRevision);
    }
}
