//! Current-state Divergent Universe battle assembly.

use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex},
};

use starclock_activity::{ActivityStateHash, GraphActivity, TechniqueContributionDigest};
use starclock_combat::{
    AssemblyDigest, Battle, BattleSeed, BattleSpec, ConcedePolicy, EncounterId, ParticipantSpec,
    TeamResourceSpec,
    catalog::{CombatCatalog, builder::CombatCatalogBuilder, definition::EncounterDefinition},
};
use starclock_data::{
    catalog::SimulationCatalog, divergent_universe_catalog::DivergentUniverseDifficultyId,
    divergent_universe_encounter_catalog::DivergentUniverseEncounterGroupId,
};

use super::{
    DivergentUniverseBattleContributionSnapshot, DivergentUniverseEncounterSelection,
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory,
    battle::{
        DivergentUniverseBattleError, ENEMY_PROXY, digest, enemy_participant, parse_level,
        player_participants,
    },
};

const ENCOUNTER_ID: EncounterId =
    EncounterId::new(0x7e24_0001).expect("reserved encounter ID is non-zero");
const BATTLE_ASSEMBLY_CACHE_CAPACITY: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseBattleAssemblyAccuracy {
    ExactCurrentParticipantContributionEncounterDifficultyAndPolicyIdentity,
    VersionedProjectPolicyCalibratedSharedMinionProxyForUnavailableEnemyBindings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseBattleAssemblyPolicy {
    ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
}

impl DivergentUniverseBattleAssemblyPolicy {
    fn stable_key(self) -> &'static str {
        match self {
            Self::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy => {
                "divergent-universe.policy.explicit-weekly-display-candidate.calibrated-shared-minion-proxy.v1"
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseAssembledBattle {
    combat_catalog: Arc<CombatCatalog>,
    battle_spec: BattleSpec,
    source_state_hash: ActivityStateHash,
    contribution: TechniqueContributionDigest,
    encounter_selection: super::DivergentUniverseEncounterSelectionDigest,
    encounter_group: DivergentUniverseEncounterGroupId,
    encounter_stage: Box<str>,
    difficulty: DivergentUniverseDifficultyId,
    policy: DivergentUniverseBattleAssemblyPolicy,
    exact_enemy_bindings: u16,
    proxy_enemy_bindings: u16,
    assembly_digest: [u8; 32],
}

impl DivergentUniverseAssembledBattle {
    pub(super) fn encounter_binding(&self) -> (&DivergentUniverseEncounterGroupId, &str) {
        (&self.encounter_group, &self.encounter_stage)
    }
    #[must_use]
    pub const fn combat_catalog(&self) -> &Arc<CombatCatalog> {
        &self.combat_catalog
    }
    #[must_use]
    pub const fn battle_spec(&self) -> &BattleSpec {
        &self.battle_spec
    }
    #[must_use]
    pub const fn source_state_hash(&self) -> ActivityStateHash {
        self.source_state_hash
    }
    #[must_use]
    pub const fn contribution_digest(&self) -> TechniqueContributionDigest {
        self.contribution
    }
    #[must_use]
    pub const fn encounter_selection_digest(
        &self,
    ) -> super::DivergentUniverseEncounterSelectionDigest {
        self.encounter_selection
    }
    #[must_use]
    pub const fn difficulty(&self) -> &DivergentUniverseDifficultyId {
        &self.difficulty
    }
    #[must_use]
    pub const fn policy(&self) -> DivergentUniverseBattleAssemblyPolicy {
        self.policy
    }
    #[must_use]
    pub const fn exact_enemy_bindings(&self) -> u16 {
        self.exact_enemy_bindings
    }
    #[must_use]
    pub const fn proxy_enemy_bindings(&self) -> u16 {
        self.proxy_enemy_bindings
    }
    #[must_use]
    pub const fn assembly_digest(&self) -> [u8; 32] {
        self.assembly_digest
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DivergentUniverseBattleAssemblyCacheMetrics {
    hits: u64,
    misses: u64,
    insertions: u64,
    evictions: u64,
    entries: usize,
}

impl DivergentUniverseBattleAssemblyCacheMetrics {
    #[must_use]
    pub const fn hits(self) -> u64 {
        self.hits
    }
    #[must_use]
    pub const fn misses(self) -> u64 {
        self.misses
    }
    #[must_use]
    pub const fn insertions(self) -> u64 {
        self.insertions
    }
    #[must_use]
    pub const fn evictions(self) -> u64 {
        self.evictions
    }
    #[must_use]
    pub const fn entries(self) -> usize {
        self.entries
    }
}

#[derive(Debug, Default)]
struct DivergentUniverseBattleAssemblyCache {
    entries: BTreeMap<[u8; 32], Arc<DivergentUniverseAssembledBattle>>,
    insertion_order: VecDeque<[u8; 32]>,
    metrics: DivergentUniverseBattleAssemblyCacheMetrics,
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseBattleAssemblyRuntime {
    component_digest: [u8; 32],
    cache: Arc<Mutex<DivergentUniverseBattleAssemblyCache>>,
}

impl DivergentUniverseRuntimeFactory {
    #[must_use]
    pub fn battle_assembly_runtime(&self) -> DivergentUniverseBattleAssemblyRuntime {
        DivergentUniverseBattleAssemblyRuntime {
            component_digest: self.bundle.identity().component_digest().bytes(),
            cache: Arc::new(Mutex::new(DivergentUniverseBattleAssemblyCache::default())),
        }
    }
}

impl DivergentUniverseBattleAssemblyRuntime {
    #[must_use]
    pub const fn accuracies(&self) -> [DivergentUniverseBattleAssemblyAccuracy; 2] {
        [
            DivergentUniverseBattleAssemblyAccuracy::ExactCurrentParticipantContributionEncounterDifficultyAndPolicyIdentity,
            DivergentUniverseBattleAssemblyAccuracy::VersionedProjectPolicyCalibratedSharedMinionProxyForUnavailableEnemyBindings,
        ]
    }

    /// Resolves an immutable battle through a bounded scratch cache. Input
    /// validation runs before lookup, and the cache is excluded from Activity
    /// state, RNG, configuration identity and replay authority.
    pub fn resolve_current_battle(
        &self,
        flow: &DivergentUniverseFlowInstance,
        activity: &GraphActivity,
        core: &SimulationCatalog,
        contribution: &DivergentUniverseBattleContributionSnapshot,
        encounter: &DivergentUniverseEncounterSelection,
        policy: DivergentUniverseBattleAssemblyPolicy,
    ) -> Result<Arc<DivergentUniverseAssembledBattle>, DivergentUniverseBattleAssemblyError> {
        self.validate_inputs(flow, activity, contribution, encounter)?;
        let key = assembly_digest(
            self.component_digest,
            flow,
            activity.state_hash(),
            contribution,
            encounter,
            policy,
        );
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| DivergentUniverseBattleAssemblyError::CachePoisoned)?;
        if let Some(cached) = cache
            .entries
            .get(&key)
            .filter(|cached| cache_entry_matches(cached, activity, contribution, encounter, policy))
            .cloned()
        {
            cache.metrics.hits = cache.metrics.hits.saturating_add(1);
            return Ok(cached);
        }
        cache.metrics.misses = cache.metrics.misses.saturating_add(1);
        let assembled = Arc::new(self.materialize_current_battle(
            flow,
            activity,
            core,
            contribution,
            encounter,
            policy,
        )?);
        if cache.entries.len() == BATTLE_ASSEMBLY_CACHE_CAPACITY {
            let oldest = cache
                .insertion_order
                .pop_front()
                .ok_or(DivergentUniverseBattleAssemblyError::CacheOrderCorrupt)?;
            if cache.entries.remove(&oldest).is_none() {
                return Err(DivergentUniverseBattleAssemblyError::CacheOrderCorrupt);
            }
            cache.metrics.evictions = cache.metrics.evictions.saturating_add(1);
        }
        cache.entries.insert(key, Arc::clone(&assembled));
        cache.insertion_order.push_back(key);
        cache.metrics.insertions = cache.metrics.insertions.saturating_add(1);
        cache.metrics.entries = cache.entries.len();
        Ok(assembled)
    }

    pub fn cache_metrics(
        &self,
    ) -> Result<DivergentUniverseBattleAssemblyCacheMetrics, DivergentUniverseBattleAssemblyError>
    {
        self.cache
            .lock()
            .map(|cache| cache.metrics)
            .map_err(|_| DivergentUniverseBattleAssemblyError::CachePoisoned)
    }

    pub fn clear_cache(&self) -> Result<(), DivergentUniverseBattleAssemblyError> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| DivergentUniverseBattleAssemblyError::CachePoisoned)?;
        cache.entries.clear();
        cache.insertion_order.clear();
        cache.metrics.entries = 0;
        Ok(())
    }

    /// Builds and construction-validates one immutable battle without mutating
    /// Activity state or consuming any Activity RNG stream.
    pub fn materialize_current_battle(
        &self,
        flow: &DivergentUniverseFlowInstance,
        activity: &GraphActivity,
        core: &SimulationCatalog,
        contribution: &DivergentUniverseBattleContributionSnapshot,
        encounter: &DivergentUniverseEncounterSelection,
        policy: DivergentUniverseBattleAssemblyPolicy,
    ) -> Result<DivergentUniverseAssembledBattle, DivergentUniverseBattleAssemblyError> {
        self.validate_inputs(flow, activity, contribution, encounter)?;
        let mapping = flow
            .mapping_snapshot()
            .ok_or(DivergentUniverseBattleAssemblyError::MappingRequired)?;
        mapping
            .validate(self.component_digest, flow.definition().participants())
            .map_err(|_| DivergentUniverseBattleAssemblyError::MappingRequired)?;

        let mut participants = player_participants(flow, mapping)
            .map_err(|_| DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants)?;
        let (enemy_waves, enemy_participants, exact_enemy_bindings, proxy_enemy_bindings) =
            materialize_enemies(core, encounter)?;

        if core.combat_catalog().encounter(ENCOUNTER_ID).is_some() {
            return Err(DivergentUniverseBattleAssemblyError::IdentityCollision);
        }
        let assembly_digest = assembly_digest(
            self.component_digest,
            flow,
            activity.state_hash(),
            contribution,
            encounter,
            policy,
        );
        let definition = EncounterDefinition::new(ENCOUNTER_ID, Vec::new(), Vec::new())
            .with_waves(enemy_waves)
            .ok_or(DivergentUniverseBattleAssemblyError::InvalidEncounter)?;
        let mut builder =
            CombatCatalogBuilder::from_catalog(core.combat_catalog(), assembly_digest);
        flow.curio_battle_stats
            .as_ref()
            .ok_or(DivergentUniverseBattleAssemblyError::DefinitionMismatch)?
            .assemble(
                &mut builder,
                contribution.curios(),
                &mut participants,
                assembly_digest,
            )?;
        flow.curio_battle_reactions
            .as_ref()
            .ok_or(DivergentUniverseBattleAssemblyError::DefinitionMismatch)?
            .assemble(
                &mut builder,
                contribution.curios(),
                &mut participants,
                assembly_digest,
            )?;
        participants.extend(enemy_participants);
        builder.add_encounter(definition);
        let combat_catalog = builder
            .build()
            .map_err(|_| DivergentUniverseBattleAssemblyError::InvalidCombatCatalog)?;
        let battle_spec = battle_spec(assembly_digest, participants)?;
        Battle::create(
            Arc::clone(&combat_catalog),
            battle_spec.clone(),
            BattleSeed::new([0x24; 32]),
        )
        .map_err(|_| DivergentUniverseBattleAssemblyError::InvalidBattleConstruction)?;
        Ok(DivergentUniverseAssembledBattle {
            combat_catalog,
            battle_spec,
            source_state_hash: activity.state_hash(),
            contribution: contribution.technique_contribution_digest(),
            encounter_selection: encounter.digest(),
            encounter_group: encounter.encounter_group().clone(),
            encounter_stage: encounter.stage_id().into(),
            difficulty: flow.difficulty().clone(),
            policy,
            exact_enemy_bindings,
            proxy_enemy_bindings,
            assembly_digest,
        })
    }

    fn validate_inputs(
        &self,
        flow: &DivergentUniverseFlowInstance,
        activity: &GraphActivity,
        contribution: &DivergentUniverseBattleContributionSnapshot,
        encounter: &DivergentUniverseEncounterSelection,
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        if activity.definition().identity() != flow.definition().identity()
            || encounter.component_digest() != self.component_digest
            || flow.component_digest != self.component_digest
            || contribution.difficulty_protocol().difficulty() != flow.difficulty()
            || flow
                .mapping_snapshot()
                .is_none_or(|mapping| mapping.digest() != contribution.mapping_digest())
        {
            return Err(DivergentUniverseBattleAssemblyError::DefinitionMismatch);
        }
        if activity.player_view().terminal().is_some() {
            return Err(DivergentUniverseBattleAssemblyError::ActivityCompleted);
        }
        if contribution.source_state_hash() != activity.state_hash()
            || encounter.source_state_hash() != activity.state_hash()
        {
            return Err(DivergentUniverseBattleAssemblyError::StaleStateHash);
        }
        if encounter.waves().is_empty() {
            return Err(DivergentUniverseBattleAssemblyError::InvalidEncounter);
        }
        if let Some((group, stage)) = flow
            .offered_encounter(activity)
            .map_err(|_| DivergentUniverseBattleAssemblyError::InvalidEncounter)?
            && (group != encounter.encounter_group() || stage != encounter.stage_id())
        {
            return Err(DivergentUniverseBattleAssemblyError::InvalidEncounter);
        }
        Ok(())
    }
}

fn cache_entry_matches(
    cached: &DivergentUniverseAssembledBattle,
    activity: &GraphActivity,
    contribution: &DivergentUniverseBattleContributionSnapshot,
    encounter: &DivergentUniverseEncounterSelection,
    policy: DivergentUniverseBattleAssemblyPolicy,
) -> bool {
    cached.source_state_hash() == activity.state_hash()
        && cached.contribution_digest() == contribution.technique_contribution_digest()
        && cached.encounter_selection_digest() == encounter.digest()
        && cached.difficulty() == contribution.difficulty_protocol().difficulty()
        && cached.policy() == policy
}

type MaterializedEnemies = (
    Vec<Vec<starclock_combat::EnemyDefinitionId>>,
    Vec<ParticipantSpec>,
    u16,
    u16,
);

fn materialize_enemies(
    core: &SimulationCatalog,
    encounter: &DivergentUniverseEncounterSelection,
) -> Result<MaterializedEnemies, DivergentUniverseBattleAssemblyError> {
    let mut enemy_waves = Vec::with_capacity(encounter.waves().len());
    let mut participants = Vec::new();
    let mut exact = 0_u16;
    let mut proxy = 0_u16;
    for wave in encounter.waves() {
        let mut wave_enemies = Vec::with_capacity(wave.slots().len());
        for slot in wave.slots() {
            let (enemy, exact_binding) = match core.enemy_by_stable_key(slot.monster_id()) {
                Some(enemy) => (enemy, true),
                None => (
                    core.enemy_by_stable_key(ENEMY_PROXY)
                        .ok_or(DivergentUniverseBattleAssemblyError::MissingEnemyProxy)?,
                    false,
                ),
            };
            if exact_binding {
                exact = exact
                    .checked_add(1)
                    .ok_or(DivergentUniverseBattleAssemblyError::InvalidEncounter)?;
            } else {
                proxy = proxy
                    .checked_add(1)
                    .ok_or(DivergentUniverseBattleAssemblyError::InvalidEncounter)?;
            }
            let level = parse_level(slot.level())
                .map_err(|_| DivergentUniverseBattleAssemblyError::InvalidEnemyLevel)?;
            wave_enemies.push(enemy.id());
            participants.push(
                enemy_participant(
                    core,
                    enemy,
                    level,
                    wave.wave_index(),
                    slot.slot_index(),
                    slot.monster_id(),
                )
                .map_err(|_| DivergentUniverseBattleAssemblyError::InvalidEnemyParticipant)?,
            );
        }
        enemy_waves.push(wave_enemies);
    }
    Ok((enemy_waves, participants, exact, proxy))
}

fn battle_spec(
    assembly_digest: [u8; 32],
    participants: Vec<ParticipantSpec>,
) -> Result<BattleSpec, DivergentUniverseBattleAssemblyError> {
    BattleSpec::new(
        AssemblyDigest::new(assembly_digest)
            .ok_or(DivergentUniverseBattleAssemblyError::InvalidAssemblyDigest)?,
        ENCOUNTER_ID,
        participants,
        TeamResourceSpec::new(3, 5)
            .ok_or(DivergentUniverseBattleAssemblyError::InvalidTeamResources)?,
        TeamResourceSpec::new(0, 0)
            .ok_or(DivergentUniverseBattleAssemblyError::InvalidTeamResources)?,
        ConcedePolicy::Allowed,
    )
    .map_err(|_| DivergentUniverseBattleAssemblyError::InvalidBattleSpec)
}

fn assembly_digest(
    component_digest: [u8; 32],
    flow: &DivergentUniverseFlowInstance,
    state_hash: ActivityStateHash,
    contribution: &DivergentUniverseBattleContributionSnapshot,
    encounter: &DivergentUniverseEncounterSelection,
    policy: DivergentUniverseBattleAssemblyPolicy,
) -> [u8; 32] {
    digest(&[
        b"starclock.divergent-universe.current-battle-assembly.v1",
        &component_digest,
        &flow.definition().identity().definition_digest().bytes(),
        &state_hash.bytes(),
        &contribution.digest().bytes(),
        &encounter.digest().bytes(),
        flow.difficulty().as_str().as_bytes(),
        &contribution.difficulty_protocol().digest(),
        policy.stable_key().as_bytes(),
    ])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseBattleAssemblyError {
    DefinitionMismatch,
    ActivityCompleted,
    StaleStateHash,
    MappingRequired,
    InvalidPlayerParticipants,
    InvalidEncounter,
    IdentityCollision,
    MissingEnemyProxy,
    InvalidEnemyLevel,
    InvalidEnemyParticipant,
    InvalidCombatCatalog,
    InvalidAssemblyDigest,
    InvalidTeamResources,
    InvalidBattleSpec,
    InvalidBattleConstruction,
    CachePoisoned,
    CacheOrderCorrupt,
}

impl From<DivergentUniverseBattleError> for DivergentUniverseBattleAssemblyError {
    fn from(_: DivergentUniverseBattleError) -> Self {
        Self::InvalidBattleSpec
    }
}

impl core::fmt::Display for DivergentUniverseBattleAssemblyError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe battle assembly error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseBattleAssemblyError {}
