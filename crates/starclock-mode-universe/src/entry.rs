//! Standard Universe entry validation and generic Activity-state compilation.

use std::sync::Arc;

use starclock_activity::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityScope,
    ActivitySlotDefinition, ActivitySlotId, ActivityStateDefinition, ActivityStateSource,
    ActivityStateVisibility, ActivityValue, GraphActivity, GraphActivityDefinition,
    GraphActivityResolution, GraphActivityStartError, LoadoutLockScope, ParticipantLock,
    ParticipantPolicy, ParticipantUniquenessScope, SlotCarryPolicy, SlotResetPoint,
};

use crate::{
    catalog::UniverseCatalog,
    digest::Encoder,
    id::{AbilityTreeNodeId, DifficultyId, PathId, WorldId},
};

pub const STANDARD_UNIVERSE_ENTRY_REVISION: &str = "standard-universe-entry-v1";

const WORLD_SLOT: u32 = 1;
const DIFFICULTY_SLOT: u32 = 2;
const PATH_SLOT: u32 = 3;
const ABILITY_TREE_SLOT: u32 = 4;
const TOPOLOGY_SLOT: u32 = 5;
const HUB_CLEAR_SLOT: u32 = 6;
const WORLD_SOURCE: u64 = 0x5355_0001;
const DIFFICULTY_SOURCE: u64 = 0x5355_0002;
const PATH_SOURCE: u64 = 0x5355_0003;
const ABILITY_TREE_SOURCE: u64 = 0x5355_0004;
const TOPOLOGY_SOURCE: u64 = 0x5355_0005;
const HUB_CLEAR_SOURCE: u64 = 0x5355_0006;

/// Validated caller-owned inputs for one Standard Universe run.
///
/// Path selection is deliberately not an entry argument. It is the first
/// authoritative Activity decision and begins as an empty optional slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardUniverseEntry {
    world: WorldId,
    difficulty: DifficultyId,
    participants: ParticipantLock,
    ability_tree: Box<[AbilityTreeNodeId]>,
}

impl StandardUniverseEntry {
    #[must_use]
    pub fn new(
        world: WorldId,
        difficulty: DifficultyId,
        participants: ParticipantLock,
        ability_tree: Vec<AbilityTreeNodeId>,
    ) -> Self {
        Self {
            world,
            difficulty,
            participants,
            ability_tree: ability_tree.into_boxed_slice(),
        }
    }

    #[must_use]
    pub const fn world(&self) -> WorldId {
        self.world
    }

    #[must_use]
    pub const fn difficulty(&self) -> DifficultyId {
        self.difficulty
    }

    #[must_use]
    pub const fn participants(&self) -> &ParticipantLock {
        &self.participants
    }

    #[must_use]
    pub fn ability_tree(&self) -> &[AbilityTreeNodeId] {
        &self.ability_tree
    }
}

/// Immutable compiler facade over one validated shared Universe catalog.
#[derive(Clone, Debug)]
pub struct StandardUniverseProfile {
    catalog: Arc<UniverseCatalog>,
}

impl StandardUniverseProfile {
    #[must_use]
    pub const fn new(catalog: Arc<UniverseCatalog>) -> Self {
        Self { catalog }
    }

    #[must_use]
    pub const fn catalog(&self) -> &Arc<UniverseCatalog> {
        &self.catalog
    }

    pub fn compile(
        &self,
        entry: StandardUniverseEntry,
    ) -> Result<CompiledActivity, StandardUniverseCompileError> {
        let world = self
            .catalog
            .world(entry.world)
            .ok_or(StandardUniverseCompileError::UnknownWorld(entry.world))?;
        let difficulty = self.catalog.difficulty(entry.difficulty).ok_or(
            StandardUniverseCompileError::UnknownDifficulty(entry.difficulty),
        )?;
        if difficulty.world() != world.id() || !world.difficulties().contains(&difficulty.id()) {
            return Err(StandardUniverseCompileError::DifficultyWorldMismatch {
                world: world.id(),
                difficulty: difficulty.id(),
            });
        }
        validate_participants(entry.participants.policy())?;

        let ability_tree = canonical_ability_tree(&self.catalog, &entry.ability_tree)?;
        let path_options = self
            .catalog
            .paths()
            .iter()
            .map(|path| path.id())
            .collect::<Vec<_>>();
        if path_options.is_empty() {
            return Err(StandardUniverseCompileError::NoAvailablePaths);
        }

        let state = compile_state(world.id(), difficulty.id(), &ability_tree)?;
        let participant_digest = entry.participants.digest();
        let identity = compile_identity(
            &self.catalog,
            world.id(),
            difficulty.id(),
            participant_digest.bytes(),
            &ability_tree,
            &path_options,
        )?;
        let participants = Arc::new(entry.participants);
        let topology = crate::topology::compile(
            &self.catalog,
            identity,
            state.clone(),
            Arc::clone(&participants),
            slot(PATH_SLOT),
            slot(TOPOLOGY_SLOT),
            slot(HUB_CLEAR_SLOT),
        )
        .map_err(StandardUniverseCompileError::Topology)?;

        Ok(CompiledActivity {
            catalog: Arc::clone(&self.catalog),
            identity,
            world: world.id(),
            difficulty: difficulty.id(),
            participants,
            ability_tree: ability_tree.into_boxed_slice(),
            path_options: path_options.into_boxed_slice(),
            state,
            runtime: topology.runtime,
            hubs: topology.hubs,
            topology_candidates: topology.candidates,
        })
    }
}

/// Immutable mode-compiled Activity entry contract.
///
/// P3-B2 attaches the topology graph and generic runtime start operation to
/// this same type. This batch owns only entry selections and Activity state.
#[derive(Clone, Debug)]
pub struct CompiledActivity {
    catalog: Arc<UniverseCatalog>,
    identity: ActivityDefinitionIdentity,
    world: WorldId,
    difficulty: DifficultyId,
    participants: Arc<ParticipantLock>,
    ability_tree: Box<[AbilityTreeNodeId]>,
    path_options: Box<[PathId]>,
    state: ActivityStateDefinition,
    runtime: Arc<GraphActivityDefinition>,
    hubs: Box<[crate::topology::DomainHubDefinition]>,
    topology_candidates: Box<[crate::id::TopologyId]>,
}

impl CompiledActivity {
    #[must_use]
    pub const fn catalog(&self) -> &Arc<UniverseCatalog> {
        &self.catalog
    }

    #[must_use]
    pub const fn identity(&self) -> ActivityDefinitionIdentity {
        self.identity
    }

    #[must_use]
    pub const fn world(&self) -> WorldId {
        self.world
    }

    #[must_use]
    pub const fn difficulty(&self) -> DifficultyId {
        self.difficulty
    }

    #[must_use]
    pub const fn participants(&self) -> &Arc<ParticipantLock> {
        &self.participants
    }

    #[must_use]
    pub fn ability_tree(&self) -> &[AbilityTreeNodeId] {
        &self.ability_tree
    }

    /// Canonical Path candidates for the first runtime choice.
    #[must_use]
    pub fn path_options(&self) -> &[PathId] {
        &self.path_options
    }

    #[must_use]
    pub const fn state_definition(&self) -> &ActivityStateDefinition {
        &self.state
    }

    #[must_use]
    pub const fn runtime_definition(&self) -> &Arc<GraphActivityDefinition> {
        &self.runtime
    }

    #[must_use]
    pub fn domain_hubs(&self) -> &[crate::topology::DomainHubDefinition] {
        &self.hubs
    }

    #[must_use]
    pub fn topology_candidates(&self) -> &[crate::id::TopologyId] {
        &self.topology_candidates
    }

    pub fn start(
        &self,
        instance: ActivityInstanceId,
        master_seed: ActivityMasterSeed,
    ) -> Result<GraphActivityResolution, GraphActivityStartError> {
        GraphActivity::start(Arc::clone(&self.runtime), instance, master_seed)
    }

    #[must_use]
    pub const fn world_slot(&self) -> ActivitySlotId {
        slot(WORLD_SLOT)
    }

    #[must_use]
    pub const fn difficulty_slot(&self) -> ActivitySlotId {
        slot(DIFFICULTY_SLOT)
    }

    #[must_use]
    pub const fn selected_path_slot(&self) -> ActivitySlotId {
        slot(PATH_SLOT)
    }

    #[must_use]
    pub const fn ability_tree_slot(&self) -> ActivitySlotId {
        slot(ABILITY_TREE_SLOT)
    }

    #[must_use]
    pub const fn selected_topology_slot(&self) -> ActivitySlotId {
        slot(TOPOLOGY_SLOT)
    }

    #[must_use]
    pub const fn hub_clear_slot(&self) -> ActivitySlotId {
        slot(HUB_CLEAR_SLOT)
    }
}

fn validate_participants(actual: ParticipantPolicy) -> Result<(), StandardUniverseCompileError> {
    let expected = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .expect("static Standard Universe participant policy is valid");
    if actual != expected {
        return Err(StandardUniverseCompileError::ParticipantPolicyMismatch);
    }
    Ok(())
}

fn canonical_ability_tree(
    catalog: &UniverseCatalog,
    input: &[AbilityTreeNodeId],
) -> Result<Vec<AbilityTreeNodeId>, StandardUniverseCompileError> {
    let mut selected = input.to_vec();
    selected.sort_unstable();
    if let Some(pair) = selected.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(StandardUniverseCompileError::DuplicateAbilityTreeNode(
            pair[0],
        ));
    }
    for node in &selected {
        let definition = catalog
            .ability_tree_node(*node)
            .ok_or(StandardUniverseCompileError::UnknownAbilityTreeNode(*node))?;
        if let Some(prerequisite) = definition
            .prerequisites()
            .iter()
            .find(|prerequisite| selected.binary_search(prerequisite).is_err())
        {
            return Err(
                StandardUniverseCompileError::MissingAbilityTreePrerequisite {
                    node: *node,
                    prerequisite: *prerequisite,
                },
            );
        }
    }
    Ok(selected)
}

fn compile_state(
    world: WorldId,
    difficulty: DifficultyId,
    ability_tree: &[AbilityTreeNodeId],
) -> Result<ActivityStateDefinition, StandardUniverseCompileError> {
    let slots = vec![
        activity_slot(
            WORLD_SLOT,
            ActivityValue::StableId(u64::from(world.get())),
            None,
            WORLD_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        activity_slot(
            DIFFICULTY_SLOT,
            ActivityValue::StableId(u64::from(difficulty.get())),
            None,
            DIFFICULTY_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        activity_slot(
            PATH_SLOT,
            ActivityValue::OptionalId(None),
            None,
            PATH_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        activity_slot(
            ABILITY_TREE_SLOT,
            ActivityValue::OrderedIdSet(
                ability_tree
                    .iter()
                    .map(|id| u64::from(id.get()))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            Some(4_096),
            ABILITY_TREE_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        activity_slot(
            TOPOLOGY_SLOT,
            ActivityValue::OptionalId(None),
            None,
            TOPOLOGY_SOURCE,
            ActivityStateVisibility::Private,
        )?,
        activity_slot(
            HUB_CLEAR_SLOT,
            ActivityValue::BoundedCounterMap(Box::new([])),
            Some(4_096),
            HUB_CLEAR_SOURCE,
            ActivityStateVisibility::Private,
        )?,
    ];
    ActivityStateDefinition::new(slots, vec![], vec![])
        .map_err(|_| StandardUniverseCompileError::InvalidActivityState)
}

fn activity_slot(
    id: u32,
    initial: ActivityValue,
    maximum_entries: Option<u32>,
    source: u64,
    visibility: ActivityStateVisibility,
) -> Result<ActivitySlotDefinition, StandardUniverseCompileError> {
    let bounds = matches!(&initial, ActivityValue::BoundedCounterMap(_)).then_some((0, 1));
    ActivitySlotDefinition::new_with_policy(
        slot(id),
        ActivityScope::Activity,
        initial,
        bounds,
        maximum_entries,
        vec![SlotResetPoint::ActivityStart],
        SlotCarryPolicy::CarryExact,
        visibility,
        ActivityStateSource::new(source).expect("static state source is non-zero"),
    )
    .map_err(|_| StandardUniverseCompileError::InvalidActivityState)
}

fn compile_identity(
    catalog: &UniverseCatalog,
    world: WorldId,
    difficulty: DifficultyId,
    participant_digest: [u8; 32],
    ability_tree: &[AbilityTreeNodeId],
    path_options: &[PathId],
) -> Result<ActivityDefinitionIdentity, StandardUniverseCompileError> {
    let catalog_identity = catalog.identity();
    let mut encoder = Encoder::new(b"starclock-standard-universe-entry-definition-v1");
    encoder.text(STANDARD_UNIVERSE_ENTRY_REVISION);
    encoder.digest(catalog_identity.configuration_digest().bytes());
    encoder.digest(catalog_identity.definitions_digest().bytes());
    encoder.digest(catalog_identity.path_definitions_digest().bytes());
    encoder.digest(catalog_identity.run_definitions_digest().bytes());
    encoder.u32(world.get());
    encoder.u32(difficulty.get());
    encoder.digest(participant_digest);
    encoder.u32(ability_tree.len() as u32);
    for node in ability_tree {
        encoder.u32(node.get());
    }
    encoder.u32(path_options.len() as u32);
    for path in path_options {
        encoder.u32(path.get());
    }
    let definition_digest = ActivityDefinitionDigest::new(encoder.finish())
        .ok_or(StandardUniverseCompileError::InvalidCatalogIdentity)?;
    let config_digest = ActivityConfigDigest::new(catalog_identity.configuration_digest().bytes())
        .ok_or(StandardUniverseCompileError::InvalidCatalogIdentity)?;
    let definition_id = ActivityDefinitionId::new(catalog.activity_binding().id().get())
        .ok_or(StandardUniverseCompileError::InvalidCatalogIdentity)?;
    Ok(ActivityDefinitionIdentity::new(
        definition_id,
        definition_digest,
        config_digest,
    ))
}

const fn slot(raw: u32) -> ActivitySlotId {
    match ActivitySlotId::new(raw) {
        Some(value) => value,
        None => panic!("static Activity slot ID must be non-zero"),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseCompileError {
    UnknownWorld(WorldId),
    UnknownDifficulty(DifficultyId),
    DifficultyWorldMismatch {
        world: WorldId,
        difficulty: DifficultyId,
    },
    ParticipantPolicyMismatch,
    DuplicateAbilityTreeNode(AbilityTreeNodeId),
    UnknownAbilityTreeNode(AbilityTreeNodeId),
    MissingAbilityTreePrerequisite {
        node: AbilityTreeNodeId,
        prerequisite: AbilityTreeNodeId,
    },
    NoAvailablePaths,
    InvalidActivityState,
    InvalidCatalogIdentity,
    Topology(crate::topology::UniverseTopologyCompileError),
}

impl core::fmt::Display for StandardUniverseCompileError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "Standard Universe entry rejected: {self:?}")
    }
}

impl std::error::Error for StandardUniverseCompileError {}
