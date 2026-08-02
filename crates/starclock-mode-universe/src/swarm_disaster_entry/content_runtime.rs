use std::{collections::BTreeMap, sync::Arc};

use serde::Deserialize;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityInventoryId, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivityRngLabel, ActivityRngStreams,
    ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use crate::{
    ability_runtime::AbilityRuntimeCatalog,
    battle_contribution::UniverseBattleContributionCompiler,
    blessing_runtime::BlessingRuntimeCatalog,
    catalog::UniverseCatalog,
    curio_runtime::CurioRuntimeCatalog,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    id::{BlessingId, CurioId},
    path::ExactParameter,
    path_runtime::PathRuntimeCatalog,
    run_runtime::RunRuntimeCatalog,
    swarm_disaster_content::inventory_access::{
        BlessingInput, BlessingLevelInput, CurioInput, CurioRuleInput, CurioStateInput,
        InventoryRuntimeInput,
    },
};

use super::{
    SwarmDisasterRuntimeInstance, content_runtime_digest,
    path_runtime::PendingContentKind,
    state::{BLESSING_INVENTORY, CONTENT, CURIO_INVENTORY, DEFERRED},
};

pub const SWARM_DISASTER_CONTENT_RUNTIME_REVISION: &str = "swarm-disaster-content-runtime-v1";
pub const SWARM_DISASTER_OFFER_POLICY_ACCURACY: &str =
    "DeterministicProjectPolicyNotObservedParity";

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../../config/universe-generated/config.sora");
const BLESSING_PURPOSE: u16 = 0x5341;
const CURIO_PURPOSE: u16 = 0x5342;
const CURIO_STATE_BASE: u64 = 0x5344_6100_0000_0000;
const CURIO_COUNTER_BASE: u64 = 0x5344_6200_0000_0000;
const DEFERRED_SETTLEMENT_PROGRAM: u32 = 0x534C_0001;

#[derive(Clone, Debug)]
pub(super) struct ContentRuntimeCatalog {
    pub(super) standard: Arc<UniverseCatalog>,
    pub(super) blessings: Arc<BlessingRuntimeCatalog>,
    pub(super) paths: Arc<PathRuntimeCatalog>,
    pub(super) shared_curios: Arc<CurioRuntimeCatalog>,
    pub(super) abilities: Arc<AbilityRuntimeCatalog>,
    pub(super) run: Arc<RunRuntimeCatalog>,
    pub(super) battle_contributions: Arc<UniverseBattleContributionCompiler>,
    reachable_blessings: Box<[ReachableBlessing]>,
    curios: Box<[RuntimeCurio]>,
    digest: [u8; 32],
}

#[derive(Clone, Debug)]
pub(super) struct ReachableBlessing {
    pub(super) id: BlessingId,
    pub(super) key: Box<str>,
    pub(super) shared_key: Box<str>,
    pub(super) path_key: Box<str>,
    pub(super) rarity: u8,
}

#[derive(Clone, Debug)]
pub(super) struct RuntimeCurio {
    pub(super) id: u32,
    pub(super) source_id: u32,
    pub(super) key: Box<str>,
    pub(super) shared_curio: Option<CurioId>,
    pub(super) category: CurioCategory,
    pub(super) initial_state: CurioState,
    pub(super) terminal_state: CurioState,
    pub(super) maximum_charges: Option<u8>,
    pub(super) decrement_event: Box<str>,
    pub(super) repair_after_battles: Option<u8>,
    pub(super) effect_program: Box<str>,
    pub(super) repair_target: Box<str>,
    pub(super) trigger_phase: Box<str>,
    pub(super) trigger: Box<str>,
    pub(super) replacement_policy: Box<str>,
    pub(super) replaces_all: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CurioCategory {
    Normal,
    Negative,
    ErrorCode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CurioState {
    Active = 1,
    Repairing = 2,
    Fixed = 3,
    Destroyed = 4,
    Replaced = 5,
}

impl ContentRuntimeCatalog {
    pub(super) fn compile(input: InventoryRuntimeInput) -> Result<Self, UniverseCatalogLoadError> {
        if input.blessings.len() != 144
            || input.blessing_levels.len() != 288
            || input.pool_memberships.len() != 184
            || input.curios.len() != 66
            || input.curio_states.len() != 66
            || input.curio_rules.len() != 66
        {
            return Err(invalid("Swarm content runtime denominator drift"));
        }
        let core = starclock_data::catalog::load(CORE_BUNDLE)
            .map_err(|_| invalid("invalid shared core bundle"))?;
        let standard = UniverseCatalog::load(UNIVERSE_BUNDLE, core)
            .map_err(|_| invalid("invalid shared Universe bundle"))?;
        let blessings = Arc::new(
            BlessingRuntimeCatalog::compile(&standard)
                .map_err(|_| invalid("invalid shared Blessing runtime"))?,
        );
        let paths = Arc::new(
            PathRuntimeCatalog::compile(&standard)
                .map_err(|_| invalid("invalid shared Path runtime"))?,
        );
        let shared_curios = Arc::new(
            CurioRuntimeCatalog::compile(&standard)
                .map_err(|_| invalid("invalid shared Curio runtime"))?,
        );
        let abilities = Arc::new(
            AbilityRuntimeCatalog::compile(&standard)
                .map_err(|_| invalid("invalid shared Ability runtime"))?,
        );
        let run = Arc::new(
            RunRuntimeCatalog::compile(&standard)
                .map_err(|_| invalid("invalid shared run runtime"))?,
        );
        let battle_contributions = Arc::new(
            UniverseBattleContributionCompiler::compile(Arc::clone(&standard))
                .map_err(|_| invalid("invalid shared battle contribution runtime"))?,
        );
        let reachable_blessings = compile_blessings(
            &standard,
            &blessings,
            &input.blessings,
            &input.blessing_levels,
            &input,
        )?;
        let curios = compile_curios(&input, &standard)?;
        let digest = content_runtime_digest::catalog_digest(
            &blessings,
            &reachable_blessings,
            &curios,
            &input,
        );
        Ok(Self {
            standard,
            blessings,
            paths,
            shared_curios,
            abilities,
            run,
            battle_contributions,
            reachable_blessings,
            curios,
            digest,
        })
    }

    pub(super) const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub(super) fn standard(&self) -> &UniverseCatalog {
        &self.standard
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize) {
        (
            self.blessings.definitions().len(),
            self.reachable_blessings.len(),
            self.curios.len(),
        )
    }

    pub(super) fn curio(&self, id: u32) -> Result<&RuntimeCurio, UniverseCatalogLoadError> {
        self.curios
            .binary_search_by_key(&id, |curio| curio.id)
            .ok()
            .map(|index| &self.curios[index])
            .ok_or_else(|| reference("unknown Swarm Curio mode-copy ID"))
    }

    pub(super) fn blessing_candidates(
        &self,
        minimum_rarity: u8,
        maximum_rarity: u8,
        owned: &[BlessingId],
    ) -> Result<Vec<BlessingId>, UniverseCatalogLoadError> {
        let owned = canonical_ids(owned)?;
        if minimum_rarity == 0 || minimum_rarity > maximum_rarity || maximum_rarity > 3 {
            return Err(reference("invalid Swarm Blessing rarity range"));
        }
        Ok(self
            .reachable_blessings
            .iter()
            .filter(|row| (minimum_rarity..=maximum_rarity).contains(&row.rarity))
            .filter(|row| owned.binary_search(&row.id).is_err())
            .map(|row| row.id)
            .collect())
    }

    pub(super) fn curio_candidates(
        &self,
        category: Option<CurioCategory>,
        owned: &[u32],
    ) -> Result<Vec<u32>, UniverseCatalogLoadError> {
        let owned = canonical_u32(owned)?;
        if owned.iter().any(|id| self.curio(*id).is_err()) {
            return Err(reference("invalid Swarm Curio inventory"));
        }
        Ok(self
            .curios
            .iter()
            .filter(|row| category.is_none_or(|category| row.category == category))
            .filter(|row| owned.binary_search(&row.id).is_err())
            .map(|row| row.id)
            .collect())
    }

    pub(super) fn blessing_acquisition_operations(
        &self,
        id: BlessingId,
    ) -> Result<Vec<ActivityOperation>, UniverseCatalogLoadError> {
        if self.reachable_blessings.iter().all(|row| row.id != id) {
            return Err(reference("Blessing is not reachable in Swarm Disaster"));
        }
        Ok(vec![
            require_inventory(blessing_inventory(), u64::from(id.get()), 0),
            ActivityOperation::AddInventory {
                inventory: blessing_inventory(),
                content: u64::from(id.get()),
                count: integer(1),
            },
        ])
    }

    pub(super) fn curio_acquisition_operations(
        &self,
        id: u32,
    ) -> Result<Vec<ActivityOperation>, UniverseCatalogLoadError> {
        self.curio(id).map(acquisition)
    }
}

impl SwarmDisasterRuntimeInstance {
    #[must_use]
    pub fn content_runtime_digest(&self) -> [u8; 32] {
        self.content_runtime.digest()
    }

    #[must_use]
    pub fn reachable_blessing_count(&self) -> usize {
        self.content_runtime.reachable_blessings.len()
    }

    #[must_use]
    pub fn swarm_curio_count(&self) -> usize {
        self.content_runtime.curios.len()
    }

    pub fn blessing_candidates(
        &self,
        minimum_rarity: u8,
        maximum_rarity: u8,
        owned: &[BlessingId],
    ) -> Result<Box<[BlessingId]>, UniverseCatalogLoadError> {
        self.content_runtime
            .blessing_candidates(minimum_rarity, maximum_rarity, owned)
            .map(Vec::into_boxed_slice)
    }

    pub fn select_blessings(
        &self,
        minimum_rarity: u8,
        maximum_rarity: u8,
        owned: &[BlessingId],
        maximum: u16,
        rng: &mut ActivityRngStreams,
    ) -> Result<Box<[BlessingId]>, UniverseCatalogLoadError> {
        let candidates =
            self.content_runtime
                .blessing_candidates(minimum_rarity, maximum_rarity, owned)?;
        select(
            &candidates,
            maximum,
            ActivityRngLabel::Reward,
            BLESSING_PURPOSE,
            rng,
        )
    }

    pub fn compile_blessing_acquisition(
        &self,
        id: BlessingId,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        if !self
            .content_runtime
            .reachable_blessings
            .iter()
            .any(|row| row.id == id)
        {
            return Err(reference("Blessing is not reachable in Swarm Disaster"));
        }
        program(
            0x5344_7000_u32
                .checked_add(id.get())
                .ok_or_else(|| invalid("Blessing program ID overflow"))?,
            vec![
                require_inventory(blessing_inventory(), u64::from(id.get()), 0),
                ActivityOperation::AddInventory {
                    inventory: blessing_inventory(),
                    content: u64::from(id.get()),
                    count: integer(1),
                },
            ],
        )
    }

    pub fn compile_blessing_enhancement(
        &self,
        id: BlessingId,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let operations = self
            .content_runtime
            .blessings
            .enhancement_operations(blessing_inventory(), id)
            .filter(|_| {
                self.content_runtime
                    .reachable_blessings
                    .iter()
                    .any(|row| row.id == id)
            })
            .ok_or_else(|| reference("Blessing is not reachable in Swarm Disaster"))?;
        program(0x5344_7400 + id.get(), operations.into_vec())
    }

    pub fn compile_blessing_replacement(
        &self,
        removed: BlessingId,
        acquired: BlessingId,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let reachable = |id| {
            self.content_runtime
                .reachable_blessings
                .iter()
                .any(|row| row.id == id)
        };
        if !reachable(removed) || !reachable(acquired) {
            return Err(reference("Blessing replacement leaves the Swarm pool"));
        }
        let operations = self
            .content_runtime
            .blessings
            .replacement_operations(blessing_inventory(), removed, acquired)
            .ok_or_else(|| reference("invalid Blessing replacement"))?;
        let id = 0x5345_0000_u32
            .checked_add(
                removed
                    .get()
                    .checked_mul(512)
                    .and_then(|value| value.checked_add(acquired.get()))
                    .ok_or_else(|| invalid("Blessing replacement ID overflow"))?,
            )
            .ok_or_else(|| invalid("Blessing replacement ID overflow"))?;
        program(id, operations.into_vec())
    }

    pub fn curio_candidates(
        &self,
        category: &str,
        owned: &[u32],
    ) -> Result<Box<[u32]>, UniverseCatalogLoadError> {
        self.curio_rules
            .content(&self.content_runtime)
            .curio_candidates(category_filter(category)?, owned)
            .map(Vec::into_boxed_slice)
    }

    pub fn select_curios(
        &self,
        category: &str,
        owned: &[u32],
        maximum: u16,
        rng: &mut ActivityRngStreams,
    ) -> Result<Box<[u32]>, UniverseCatalogLoadError> {
        let candidates = self
            .curio_rules
            .content(&self.content_runtime)
            .curio_candidates(category_filter(category)?, owned)?;
        select(
            &candidates,
            maximum,
            ActivityRngLabel::Reward,
            CURIO_PURPOSE,
            rng,
        )
    }

    pub fn compile_deferred_content_rewards(
        &self,
        state: &ActivityTransactionState,
        rng: &mut ActivityRngStreams,
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        let requests = self.path_runtime.pending_content_requests(state)?;
        if requests.is_empty() {
            return Ok(None);
        }
        let mut owned_blessings = state
            .inventory_entries(blessing_inventory())
            .ok_or_else(|| invalid("missing Swarm Blessing inventory"))?
            .map(|(id, _)| {
                u32::try_from(id)
                    .ok()
                    .and_then(BlessingId::new)
                    .ok_or_else(|| reference("invalid owned Blessing identity"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut owned_curios = state
            .inventory_entries(curio_inventory())
            .ok_or_else(|| invalid("missing Swarm Curio inventory"))?
            .map(|(id, _)| u32::try_from(id).map_err(|_| reference("invalid owned Curio identity")))
            .collect::<Result<Vec<_>, _>>()?;
        let operations = rng.transact(|working| {
            let mut operations = Vec::new();
            for request in &requests {
                operations.push(require_counter_in(
                    DEFERRED,
                    request.key,
                    i64::from(request.count),
                ));
                operations.push(add_counter_in(
                    DEFERRED,
                    request.key,
                    -i64::from(request.count),
                ));
                match request.kind {
                    PendingContentKind::Blessing => {
                        let candidates = self.content_runtime.blessing_candidates(
                            request.minimum_rarity,
                            request.maximum_rarity,
                            &owned_blessings,
                        )?;
                        let selected = select(
                            &candidates,
                            request.count,
                            ActivityRngLabel::Reward,
                            BLESSING_PURPOSE,
                            working,
                        )?;
                        if selected.len() != usize::from(request.count) {
                            return Err(reference("exhausted deferred Blessing pool"));
                        }
                        for id in selected {
                            operations.push(require_inventory(
                                blessing_inventory(),
                                u64::from(id.get()),
                                0,
                            ));
                            operations.push(ActivityOperation::AddInventory {
                                inventory: blessing_inventory(),
                                content: u64::from(id.get()),
                                count: integer(1),
                            });
                            owned_blessings.push(id);
                        }
                    }
                    kind => {
                        let category = match kind {
                            PendingContentKind::CurioAny => None,
                            PendingContentKind::CurioErrorCode => Some(CurioCategory::ErrorCode),
                            PendingContentKind::CurioNegative => Some(CurioCategory::Negative),
                            PendingContentKind::Blessing => unreachable!("handled above"),
                        };
                        let content = self.curio_rules.content(&self.content_runtime);
                        let candidates = content.curio_candidates(category, &owned_curios)?;
                        let selected = select(
                            &candidates,
                            request.count,
                            ActivityRngLabel::Reward,
                            CURIO_PURPOSE,
                            working,
                        )?;
                        if selected.len() != usize::from(request.count) {
                            return Err(reference("exhausted deferred Curio pool"));
                        }
                        for id in selected {
                            operations.extend(acquisition(content.curio(id)?));
                            owned_curios.push(id);
                        }
                    }
                }
                owned_blessings.sort_unstable();
                owned_curios.sort_unstable();
            }
            Ok(operations)
        })?;
        program(DEFERRED_SETTLEMENT_PROGRAM, operations).map(Some)
    }

    pub fn curio_stable_key(&self, id: u32) -> Result<&str, UniverseCatalogLoadError> {
        self.curio_rules
            .content(&self.content_runtime)
            .curio(id)
            .map(|curio| curio.key.as_ref())
    }

    pub fn compile_curio_acquisition(
        &self,
        id: u32,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let curio = self.curio_rules.content(&self.content_runtime).curio(id)?;
        program(0x5346_0000 + curio.source_id, acquisition(curio))
    }

    pub fn compile_curio_charge_use(
        &self,
        id: u32,
        expected_remaining: u8,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let curio = self.curio_rules.content(&self.content_runtime).curio(id)?;
        let maximum = curio
            .maximum_charges
            .ok_or_else(|| reference("Swarm Curio has no numeric charges"))?;
        if expected_remaining == 0 || expected_remaining > maximum {
            return Err(reference("invalid expected Swarm Curio charge"));
        }
        let mut operations = require_owned_state(id, CurioState::Active);
        operations.push(require_counter(
            counter_key(id),
            i64::from(expected_remaining),
        ));
        operations.push(add_counter(counter_key(id), -1));
        if expected_remaining == 1 {
            operations.push(transition(id, CurioState::Active, curio.terminal_state));
        }
        program(0x5347_0000 + curio.source_id, operations)
    }

    pub fn compile_curio_source_destruction(
        &self,
        id: u32,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let curio = self.curio_rules.content(&self.content_runtime).curio(id)?;
        if curio.decrement_event.as_ref() != "SourceConditionWithoutNumericCharges"
            || curio.terminal_state != CurioState::Destroyed
        {
            return Err(reference("Swarm Curio lacks source-condition destruction"));
        }
        let mut operations = require_owned_state(id, CurioState::Active);
        operations.push(transition(id, CurioState::Active, CurioState::Destroyed));
        program(0x5348_0000 + curio.source_id, operations)
    }

    pub fn compile_curio_repair_progress(
        &self,
        id: u32,
        expected_progress: u8,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let curio = self.curio_rules.content(&self.content_runtime).curio(id)?;
        let required = curio
            .repair_after_battles
            .ok_or_else(|| reference("Swarm Curio cannot be repaired"))?;
        if expected_progress >= required {
            return Err(reference("invalid Swarm Curio repair progress"));
        }
        let mut operations = require_owned_state(id, CurioState::Repairing);
        operations.push(require_counter(
            counter_key(id),
            i64::from(expected_progress),
        ));
        if expected_progress + 1 == required {
            operations.push(add_counter(counter_key(id), -i64::from(expected_progress)));
            operations.push(transition(id, CurioState::Repairing, CurioState::Fixed));
        } else {
            operations.push(add_counter(counter_key(id), 1));
        }
        program(
            0x5349_0000 + curio.source_id * 4 + u32::from(expected_progress),
            operations,
        )
    }

    pub fn compile_curio_replacement(
        &self,
        removed: u32,
        acquired: u32,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        if removed == acquired {
            return Err(reference("Swarm Curio replacement cycle"));
        }
        let content = self.curio_rules.content(&self.content_runtime);
        let removed_curio = content.curio(removed)?;
        let acquired_curio = content.curio(acquired)?;
        let mut operations = teardown(removed);
        operations.extend(acquisition(acquired_curio));
        program(
            0x534A_0000_u32
                .checked_add(removed_curio.source_id * 128 + acquired_curio.source_id)
                .ok_or_else(|| invalid("Curio replacement program ID overflow"))?,
            operations,
        )
    }

    pub fn compile_curio_teardown(
        &self,
        id: u32,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let curio = self.curio_rules.content(&self.content_runtime).curio(id)?;
        program(0x534B_0000 + curio.source_id, teardown(id))
    }
}

fn compile_blessings(
    standard: &UniverseCatalog,
    runtime: &BlessingRuntimeCatalog,
    rows: &[BlessingInput],
    levels: &[BlessingLevelInput],
    input: &InventoryRuntimeInput,
) -> Result<Box<[ReachableBlessing]>, UniverseCatalogLoadError> {
    let level_map = levels
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    if level_map.len() != levels.len() {
        return Err(reference("duplicate Swarm Blessing level"));
    }
    let mut output = Vec::with_capacity(rows.len());
    for row in rows {
        let shared = standard
            .blessings()
            .iter()
            .find(|candidate| candidate.stable_key() == row.shared_key.as_ref())
            .ok_or_else(|| reference("missing shared Swarm Blessing"))?;
        let definition = runtime
            .definition(shared.id())
            .ok_or_else(|| reference("missing shared Blessing runtime"))?;
        let path = standard
            .paths()
            .iter()
            .find(|path| path.id() == definition.path())
            .ok_or_else(|| reference("missing shared Blessing Path"))?;
        if row.id == 0
            || row.key.as_ref() == row.shared_key.as_ref()
            || row.path_key.as_ref() != path.stable_key()
            || row.rarity != definition.rarity()
            || row.level_keys.len() != 2
        {
            return Err(reference("Swarm Blessing link drift"));
        }
        for (index, key) in row.level_keys.iter().enumerate() {
            let level = levels
                .iter()
                .find(|level| level.key.as_ref() == key.as_ref())
                .ok_or_else(|| reference("missing Swarm Blessing level"))?;
            let shared_level = standard
                .blessing_levels()
                .iter()
                .find(|candidate| candidate.stable_key() == level.shared_level_key.as_ref())
                .ok_or_else(|| reference("missing shared Blessing level"))?;
            if level.blessing != row.id
                || level.shared_blessing_key != row.shared_key
                || usize::from(level.level) != index + 1
                || shared_level.blessing() != shared.id()
                || shared_level.level() != level.level
                || !exact_parameters(&level.parameters, shared_level.parameters())
                || level.effect_program.is_empty()
            {
                return Err(reference("Swarm Blessing level link drift"));
            }
        }
        output.push(ReachableBlessing {
            id: shared.id(),
            key: row.key.clone(),
            shared_key: row.shared_key.clone(),
            path_key: row.path_key.clone(),
            rarity: row.rarity,
        });
    }
    output.sort_unstable_by_key(|row| row.id);
    if output.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err(reference("duplicate shared Blessing reachability"));
    }
    let memberships = input
        .pool_memberships
        .iter()
        .filter(|row| row.member_kind.as_ref() == "Blessing")
        .collect::<Vec<_>>();
    if memberships.len() != 144
        || memberships.iter().any(|membership| {
            membership.pool_key.as_ref() != "swarm-disaster.pool.blessings"
                || output
                    .iter()
                    .all(|blessing| blessing.shared_key.as_ref() != membership.member_key.as_ref())
                || !equal_weight_policy(&membership.weight_policy)
        })
    {
        return Err(reference("Swarm Blessing pool closure drift"));
    }
    Ok(output.into_boxed_slice())
}

fn compile_curios(
    input: &InventoryRuntimeInput,
    standard: &UniverseCatalog,
) -> Result<Box<[RuntimeCurio]>, UniverseCatalogLoadError> {
    let states = input
        .curio_states
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let rules = input
        .curio_rules
        .iter()
        .map(|row| (row.curio, row))
        .collect::<BTreeMap<_, _>>();
    if states.len() != 66 || rules.len() != 66 {
        return Err(reference("duplicate Swarm Curio state or rule"));
    }
    let mut output = input
        .curios
        .iter()
        .map(|row| {
            compile_curio(
                row,
                states.get(&row.initial_state).copied(),
                rules.get(&row.id).copied(),
                standard,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    output.sort_unstable_by_key(|curio| curio.id);
    if output.windows(2).any(|pair| pair[0].id == pair[1].id)
        || output
            .iter()
            .filter(|row| row.category == CurioCategory::Normal)
            .count()
            != 53
        || output
            .iter()
            .filter(|row| row.category == CurioCategory::Negative)
            .count()
            != 7
        || output
            .iter()
            .filter(|row| row.category == CurioCategory::ErrorCode)
            .count()
            != 6
        || output.iter().filter(|row| row.replaces_all).count() != 1
        || output
            .iter()
            .filter(|row| row.shared_curio.is_some())
            .count()
            != 60
    {
        return Err(reference("Swarm Curio exact-once closure drift"));
    }
    Ok(output.into_boxed_slice())
}

fn compile_curio(
    row: &CurioInput,
    state: Option<&CurioStateInput>,
    rule: Option<&CurioRuleInput>,
    standard: &UniverseCatalog,
) -> Result<RuntimeCurio, UniverseCatalogLoadError> {
    let state = state.ok_or_else(|| reference("missing Swarm Curio initial state"))?;
    let rule = rule.ok_or_else(|| reference("missing Swarm Curio rule"))?;
    let lifecycle: Lifecycle = serde_json::from_str(&state.lifecycle)
        .map_err(|_| reference("invalid Swarm Curio lifecycle"))?;
    let replacement: ReplacementPolicy = serde_json::from_str(&rule.replacement_policy)
        .map_err(|_| reference("invalid Swarm Curio replacement policy"))?;
    let category = category(&row.pool_category)?;
    let initial_state = curio_state(&lifecycle.initial_state)?;
    let terminal_state = curio_state(&lifecycle.terminal_state)?;
    let maximum_charges = optional_u8(&lifecycle.charges)?;
    let repair_after_battles = optional_u8(&lifecycle.repair_after_completed_battles)?;
    let mode_copy = row
        .mode_copy_key
        .parse::<u32>()
        .ok()
        .filter(|id| *id > 0)
        .ok_or_else(|| reference("invalid Swarm Curio mode-copy ID"))?;
    let shared_key = row.key.replacen("swarm-disaster.", "universe.", 1);
    let shared_curio = standard
        .curios()
        .iter()
        .find(|candidate| candidate.stable_key() == shared_key)
        .map(crate::curio::CurioDefinition::id);
    if state.curio != row.id
        || rule.state != state.id
        || state.key.is_empty()
        || rule.id == 0
        || rule.key.is_empty()
        || state.state.as_ref() != lifecycle.initial_state.as_ref()
        || state.charges.as_deref().unwrap_or("") != lifecycle.charges.as_ref()
        || !pool_policy(&row.pool_rules, category)
        || state.effect_program.is_empty()
        || rule.trigger_phase.is_empty()
        || rule.trigger.is_empty()
        || rule.lifecycle.is_empty()
        || (repair_after_battles.is_some()) != (initial_state == CurioState::Repairing)
        || (repair_after_battles.is_some() && state.repair_target.as_ref() == "{}")
    {
        return Err(reference("Swarm Curio lifecycle link drift"));
    }
    let replaces_all =
        replacement.operation.as_ref() == "ReplaceAllPossessedCuriosIncludingSelfWithRandomCurios";
    if replaces_all != (terminal_state == CurioState::Replaced)
        || (replaces_all
            && (replacement.candidate_order.as_ref() != "StableEligibleCurioIdAscending"
                || replacement.no_legal_candidate.as_ref() != "NoOp"))
    {
        return Err(reference("Swarm Curio replacement link drift"));
    }
    Ok(RuntimeCurio {
        id: mode_copy,
        source_id: row.id,
        key: row.key.clone(),
        shared_curio,
        category,
        initial_state,
        terminal_state,
        maximum_charges,
        decrement_event: lifecycle.decrement_event,
        repair_after_battles,
        effect_program: state.effect_program.clone(),
        repair_target: state.repair_target.clone(),
        trigger_phase: rule.trigger_phase.clone(),
        trigger: rule.trigger.clone(),
        replacement_policy: rule.replacement_policy.clone(),
        replaces_all,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Lifecycle {
    initial_state: Box<str>,
    terminal_state: Box<str>,
    charges: Box<str>,
    decrement_event: Box<str>,
    #[serde(rename = "charge_parameter_index")]
    _charge_parameter_index: u8,
    repair_after_completed_battles: Box<str>,
    #[serde(rename = "repair_operation")]
    _repair_operation: Box<str>,
    #[serde(rename = "replacement_operation")]
    _replacement_operation: Box<str>,
    #[serde(rename = "post_destruction_effect")]
    _post_destruction_effect: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplacementPolicy {
    candidate_order: Box<str>,
    no_legal_candidate: Box<str>,
    operation: Box<str>,
    #[serde(rename = "random_stream")]
    _random_stream: Box<str>,
}

fn exact_parameters(authored: &[Box<str>], shared: &[ExactParameter]) -> bool {
    authored.len() == shared.len()
        && authored
            .iter()
            .zip(shared)
            .enumerate()
            .all(|(index, (authored, shared))| {
                indexed_decimal(authored, index + 1) == Some((shared.coefficient(), shared.scale()))
            })
}

fn indexed_decimal(value: &str, expected_index: usize) -> Option<(i64, u8)> {
    let body = value.strip_prefix("{'index': ")?.strip_suffix("'}")?;
    let (index, value) = body.split_once(", 'value': '")?;
    if index.parse::<usize>().ok()? != expected_index {
        return None;
    }
    exact_decimal(value)
}

fn exact_decimal(value: &str) -> Option<(i64, u8)> {
    let negative = value.starts_with('-');
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let (whole, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, ""), |parts| parts);
    let scale = u8::try_from(fraction.len()).ok()?;
    let coefficient = format!("{whole}{fraction}").parse::<i64>().ok()?;
    Some((if negative { -coefficient } else { coefficient }, scale))
}

fn pool_policy(value: &str, category: CurioCategory) -> bool {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Policy {
        candidate_order: Box<str>,
        eligibility: Box<str>,
        pool_id: Box<str>,
        unresolved_offer_behavior: Box<str>,
        weight_policy: Box<str>,
    }
    serde_json::from_str::<Policy>(value).is_ok_and(|policy| {
        policy.candidate_order.as_ref() == "StableCurioIdAscending"
            && policy.eligibility.as_ref() == "OwningOfferRuleRequired"
            && policy.pool_id.as_ref() == category.pool_key()
            && policy.unresolved_offer_behavior.as_ref() == "FailClosed"
            && policy.weight_policy.as_ref() == "OwningOfferMustProvideWeight"
    })
}

fn equal_weight_policy(value: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(value).is_ok_and(|policy| {
        policy
            .get("candidate_order")
            .and_then(serde_json::Value::as_str)
            == Some("StableMemberIdAscending")
            && policy
                .get("integer_weight")
                .and_then(serde_json::Value::as_str)
                == Some("1")
            && policy.get("selection").and_then(serde_json::Value::as_str)
                == Some("SeededUniformIntegerWeight")
    })
}

impl CurioCategory {
    const fn pool_key(self) -> &'static str {
        match self {
            Self::Normal => "swarm-disaster.curio-pool.normal",
            Self::Negative => "swarm-disaster.curio-pool.negative",
            Self::ErrorCode => "swarm-disaster.curio-pool.errorcode",
        }
    }
}

fn category(value: &str) -> Result<CurioCategory, UniverseCatalogLoadError> {
    match value {
        "Normal" => Ok(CurioCategory::Normal),
        "Negative" => Ok(CurioCategory::Negative),
        "ErrorCode" => Ok(CurioCategory::ErrorCode),
        _ => Err(reference("unknown Swarm Curio category")),
    }
}

fn category_filter(value: &str) -> Result<Option<CurioCategory>, UniverseCatalogLoadError> {
    match value {
        "Any" => Ok(None),
        value => category(value).map(Some),
    }
}

fn curio_state(value: &str) -> Result<CurioState, UniverseCatalogLoadError> {
    match value {
        "Active" => Ok(CurioState::Active),
        "Repairing" => Ok(CurioState::Repairing),
        "Fixed" => Ok(CurioState::Fixed),
        "Destroyed" => Ok(CurioState::Destroyed),
        "Replaced" => Ok(CurioState::Replaced),
        _ => Err(reference("unknown Swarm Curio state")),
    }
}

fn optional_u8(value: &str) -> Result<Option<u8>, UniverseCatalogLoadError> {
    if value.is_empty() {
        Ok(None)
    } else {
        value
            .parse::<u8>()
            .ok()
            .filter(|value| *value > 0)
            .map(Some)
            .ok_or_else(|| reference("invalid Swarm Curio lifecycle count"))
    }
}

pub(super) fn select<T: Copy>(
    candidates: &[T],
    maximum: u16,
    label: ActivityRngLabel,
    purpose: u16,
    rng: &mut ActivityRngStreams,
) -> Result<Box<[T]>, UniverseCatalogLoadError> {
    if maximum == 0 || candidates.is_empty() {
        return Ok(Box::new([]));
    }
    let selected = rng.transact(|working| {
        working
            .choose_weighted_without_replacement(
                label,
                purpose,
                &vec![1; candidates.len()],
                maximum,
            )
            .map_err(|_| invalid("Swarm content offer RNG failure"))
    })?;
    selected
        .iter()
        .map(|index| {
            candidates
                .get(*index as usize)
                .copied()
                .ok_or_else(|| invalid("Swarm content offer mapping failure"))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn canonical_ids(values: &[BlessingId]) -> Result<Vec<BlessingId>, UniverseCatalogLoadError> {
    let mut output = values.to_vec();
    output.sort_unstable();
    if output.windows(2).any(|pair| pair[0] == pair[1]) {
        Err(reference("duplicate Blessing inventory identity"))
    } else {
        Ok(output)
    }
}

fn canonical_u32(values: &[u32]) -> Result<Vec<u32>, UniverseCatalogLoadError> {
    let mut output = values.to_vec();
    output.sort_unstable();
    if output.contains(&0) || output.windows(2).any(|pair| pair[0] == pair[1]) {
        Err(reference("duplicate or zero Curio inventory identity"))
    } else {
        Ok(output)
    }
}

fn acquisition(curio: &RuntimeCurio) -> Vec<ActivityOperation> {
    let mut operations = vec![
        require_inventory(curio_inventory(), u64::from(curio.id), 0),
        require_counter(state_key(curio.id), 0),
        require_counter(counter_key(curio.id), 0),
        ActivityOperation::AddInventory {
            inventory: curio_inventory(),
            content: u64::from(curio.id),
            count: integer(1),
        },
        add_counter(state_key(curio.id), curio.initial_state as i64),
    ];
    if let Some(charges) = curio.maximum_charges {
        operations.push(add_counter(counter_key(curio.id), i64::from(charges)));
    }
    operations
}

fn teardown(id: u32) -> Vec<ActivityOperation> {
    vec![
        require_inventory(curio_inventory(), u64::from(id), 1),
        ActivityOperation::RemoveInventory {
            inventory: curio_inventory(),
            content: u64::from(id),
            count: integer(1),
        },
        add_counter(state_key(id), negate(counter(state_key(id)))),
        add_counter(counter_key(id), negate(counter(counter_key(id)))),
    ]
}

fn require_owned_state(id: u32, state: CurioState) -> Vec<ActivityOperation> {
    vec![
        require_inventory(curio_inventory(), u64::from(id), 1),
        require_counter(state_key(id), state as i64),
    ]
}

fn transition(id: u32, from: CurioState, to: CurioState) -> ActivityOperation {
    add_counter(state_key(id), to as i64 - from as i64)
}

fn require_inventory(
    inventory: ActivityInventoryId,
    content: u64,
    count: i64,
) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(
        ActivityExpression::InventoryCount { inventory, content },
        integer(count),
    ))
}

fn require_counter(key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(counter(key), integer(value)))
}

fn require_counter_in(slot: u32, key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(
        ActivityExpression::CounterValue {
            slot: ActivitySlotId::new(slot).expect("static Swarm slot is non-zero"),
            key,
        },
        integer(value),
    ))
}

fn add_counter_in(slot: u32, key: u64, delta: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: ActivitySlotId::new(slot).expect("static Swarm slot is non-zero"),
        key,
        delta: integer(delta),
    }
}

fn add_counter(key: u64, delta: impl IntoExpression) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: lifecycle_slot(),
        key,
        delta: delta.into_expression(),
    }
}

fn counter(key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: lifecycle_slot(),
        key,
    }
}

fn negate(value: ActivityExpression) -> ActivityExpression {
    ActivityExpression::Negate(Box::new(value))
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

trait IntoExpression {
    fn into_expression(self) -> ActivityExpression;
}

impl IntoExpression for i64 {
    fn into_expression(self) -> ActivityExpression {
        integer(self)
    }
}

impl IntoExpression for ActivityExpression {
    fn into_expression(self) -> ActivityExpression {
        self
    }
}

fn program(
    id: u32,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(id).ok_or_else(|| invalid("zero content runtime program ID"))?,
        operations,
    )
    .map_err(|_| invalid("invalid content runtime program"))
}

fn blessing_inventory() -> ActivityInventoryId {
    ActivityInventoryId::new(BLESSING_INVENTORY).expect("static inventory is non-zero")
}

fn curio_inventory() -> ActivityInventoryId {
    ActivityInventoryId::new(CURIO_INVENTORY).expect("static inventory is non-zero")
}

fn lifecycle_slot() -> ActivitySlotId {
    ActivitySlotId::new(CONTENT).expect("static lifecycle slot is non-zero")
}

const fn state_key(id: u32) -> u64 {
    CURIO_STATE_BASE + id as u64
}

const fn counter_key(id: u32) -> u64 {
    CURIO_COUNTER_BASE + id as u64
}

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}

fn reference(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}

#[cfg(test)]
#[path = "content_runtime_tests.rs"]
mod tests;
