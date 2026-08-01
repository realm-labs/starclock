//! Trailblaze Bonus execution and immutable Path/Resonance contributions.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_unique::runtime_access::{
        SwarmBonusRuntimeInput, SwarmInterplayRuntimeInput, SwarmPathBoostRuntimeInput,
        SwarmPathRuntimeDefinitionInput, SwarmPathRuntimeInput, SwarmResonanceRuntimeInput,
    },
};

use super::{
    SwarmDisasterRuntimeInstance,
    pathstrider_progress::PathstriderRuntimeCatalog,
    state::{COUNTDOWN, DEFERRED, PROGRESSION, RESOURCES},
};

const REVISION: &str = "swarm-disaster-path-resonance-runtime-v1";
const BONUS_APPLIED_KEY: u64 = 0x4000_0001;
const INTERPLAY_KEY_BASE: u64 = 0x3000_0000;
const DEFERRED_BLESSING_BASE: u64 = 0x5344_5100_0000_0000;
const DEFERRED_CURIO_ANY_BASE: u64 = 0x5344_5200_0000_0000;
const DEFERRED_CURIO_ERROR_BASE: u64 = 0x5344_5300_0000_0000;
const DEFERRED_CURIO_NEGATIVE_BASE: u64 = 0x5344_5400_0000_0000;
const BONUS_PROGRAM_BASE: u32 = 0x5344_4600;
const INTERPLAY_PROGRAM_BASE: u32 = 0x5344_4700;
const COSMIC_FRAGMENTS_KEY: u64 = 1;

#[derive(Clone, Debug)]
pub(super) struct PathRuntimeCatalog {
    bonuses: Box<[RuntimeBonus]>,
    paths: Box<[RuntimePath]>,
    digest: [u8; 32],
}

#[derive(Clone, Debug)]
pub(super) struct CompiledPathRuntime {
    path: RuntimePath,
    bonus: Option<RuntimeBonus>,
    digest: [u8; 32],
}

#[derive(Clone, Debug)]
struct RuntimeBonus {
    id: u32,
    key: Box<str>,
    immediate_fragments: i64,
    countdown_delta: i64,
    requests: Box<[DeferredRequest]>,
}

#[derive(Clone, Debug)]
struct DeferredRequest {
    kind: DeferredKind,
    count: u16,
    minimum_rarity: u8,
    maximum_rarity: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeferredKind {
    Blessing,
    CurioAny,
    CurioErrorCode,
    CurioNegative,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PendingContentKind {
    Blessing,
    CurioAny,
    CurioErrorCode,
    CurioNegative,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PendingContentRequest {
    pub(super) key: u64,
    pub(super) kind: PendingContentKind,
    pub(super) count: u16,
    pub(super) minimum_rarity: u8,
    pub(super) maximum_rarity: u8,
}

#[derive(Clone, Debug)]
struct RuntimePath {
    id: u32,
    key: Box<str>,
    shared_path: Box<str>,
    mode_unlock: Option<Box<str>>,
    propagation: bool,
    boost: RuntimeBinding,
    resonances: Box<[RuntimeResonance]>,
    interplays: Box<[RuntimeInterplay]>,
    battle_event_groups: Box<[Box<str>]>,
    extra_effect_keys: Box<[Box<str>]>,
}

#[derive(Clone, Debug)]
struct RuntimeBinding {
    key: Box<str>,
    binding_key: Box<str>,
    parameters: Box<[Box<str>]>,
}

#[derive(Clone, Debug)]
struct RuntimeResonance {
    key: Box<str>,
    shared_key: Box<str>,
    threshold: u16,
    energy_max: i64,
    initial_energy: i64,
    parameters: Box<[Box<str>]>,
    mechanic_tags: Box<[Box<str>]>,
    binding_key: Box<str>,
    rule_key: Box<str>,
}

#[derive(Clone, Debug)]
struct RuntimeInterplay {
    id: u32,
    key: Box<str>,
    sub_path: Box<str>,
    main_threshold: u16,
    sub_threshold: u16,
    binding_key: Box<str>,
    parameters: Box<[Box<str>]>,
}

impl PathRuntimeCatalog {
    pub(super) fn compile(
        input: SwarmPathRuntimeInput,
        pathstrider: &PathstriderRuntimeCatalog,
    ) -> Result<Self, UniverseCatalogLoadError> {
        if input.bonuses.len() != 6
            || input.paths.len() != 8
            || input.boosts.len() != 8
            || input.resonances.len() != 32
            || input.interplays.len() != 16
        {
            return Err(invalid("Swarm Path runtime denominator drift"));
        }
        let mut bonuses = input
            .bonuses
            .iter()
            .map(compile_bonus)
            .collect::<Result<Vec<_>, _>>()?;
        bonuses.sort_unstable_by_key(|bonus| bonus.id);
        if bonuses
            .iter()
            .enumerate()
            .any(|(index, bonus)| u32::try_from(index + 1).ok() != Some(bonus.id))
        {
            return Err(invalid("Swarm Trailblaze Bonus identity drift"));
        }

        let path_keys = input
            .paths
            .iter()
            .map(|path| (path.id, path.shared_path.as_ref()))
            .collect::<BTreeMap<_, _>>();
        if path_keys.len() != 8 {
            return Err(invalid("duplicate Swarm Path identity"));
        }
        let boosts = unique_by_path(&input.boosts, |row| row.path_id)?;
        let resonances = input.resonances.iter().fold(
            BTreeMap::<u32, Vec<&SwarmResonanceRuntimeInput>>::new(),
            |mut grouped, row| {
                grouped.entry(row.path_id).or_default().push(row);
                grouped
            },
        );
        let interplays = input.interplays.iter().fold(
            BTreeMap::<u32, Vec<&SwarmInterplayRuntimeInput>>::new(),
            |mut grouped, row| {
                grouped.entry(row.main_path_id).or_default().push(row);
                grouped
            },
        );
        let mut paths = input
            .paths
            .iter()
            .map(|path| {
                compile_path(
                    path,
                    boosts.get(&path.id).copied(),
                    resonances.get(&path.id).map(Vec::as_slice).unwrap_or(&[]),
                    interplays.get(&path.id).map(Vec::as_slice).unwrap_or(&[]),
                    &path_keys,
                    pathstrider,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        paths.sort_unstable_by_key(|path| path.id);
        if paths
            .iter()
            .map(|path| path.boost.key.as_ref())
            .collect::<BTreeSet<_>>()
            .len()
            != 8
            || paths.iter().flat_map(|path| &path.resonances).count() != 32
            || paths.iter().flat_map(|path| &path.interplays).count() != 16
            || paths.iter().filter(|path| path.propagation).count() != 1
        {
            return Err(invalid("Swarm Path runtime exact-once closure drift"));
        }
        let digest = catalog_digest(&bonuses, &paths);
        Ok(Self {
            bonuses: bonuses.into_boxed_slice(),
            paths: paths.into_boxed_slice(),
            digest,
        })
    }

    pub(super) fn select(
        &self,
        shared_path: &str,
        audience_unlock: Option<&str>,
        bonus: Option<&str>,
    ) -> Result<CompiledPathRuntime, UniverseCatalogLoadError> {
        let path = self
            .paths
            .iter()
            .find(|path| path.shared_path.as_ref() == shared_path)
            .ok_or_else(|| reference("unknown Swarm Path runtime selection"))?;
        let expected_numeric = path
            .mode_unlock
            .as_deref()
            .map(|unlock| {
                unlock
                    .strip_prefix("swarm-disaster.pathstrider-unlock.")
                    .ok_or_else(|| invalid("invalid Path mode unlock identity"))
            })
            .transpose()?;
        if expected_numeric != audience_unlock {
            return Err(reference(
                "Path mode unlock and Audience authorization differ",
            ));
        }
        let bonus = bonus
            .map(|key| {
                self.bonuses
                    .iter()
                    .find(|bonus| bonus.key.as_ref() == key)
                    .cloned()
                    .ok_or_else(|| reference("unknown Trailblaze Bonus runtime selection"))
            })
            .transpose()?;
        let mut encoder = Encoder::new(b"starclock.swarm-disaster.path-runtime.instance.v1");
        encoder.digest(self.digest);
        encoder.text(&path.shared_path);
        encoder.optional_text(bonus.as_ref().map(|bonus| bonus.key.as_ref()));
        Ok(CompiledPathRuntime {
            path: path.clone(),
            bonus,
            digest: encoder.finish(),
        })
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.bonuses.len(),
            self.paths.len(),
            self.paths
                .iter()
                .map(|path| usize::from(!path.boost.key.is_empty()))
                .sum(),
            self.paths.iter().map(|path| path.resonances.len()).sum(),
            self.paths.iter().map(|path| path.interplays.len()).sum(),
        )
    }

    pub(super) fn profile_bonus_keys(&self) -> impl ExactSizeIterator<Item = &str> {
        self.bonuses.iter().map(|bonus| bonus.key.as_ref())
    }
}

impl CompiledPathRuntime {
    pub(super) fn pending_content_requests(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<Box<[PendingContentRequest]>, UniverseCatalogLoadError> {
        let Some(bonus) = &self.bonus else {
            return Ok(Box::new([]));
        };
        bonus
            .requests
            .iter()
            .enumerate()
            .filter_map(|(ordinal, request)| {
                let key = match request_key(bonus.id, ordinal, request.kind) {
                    Ok(key) => key,
                    Err(error) => return Some(Err(error)),
                };
                match counter_value(state, DEFERRED, key) {
                    Ok(0) => None,
                    Ok(value) if value == i64::from(request.count) => {
                        Some(Ok(PendingContentRequest {
                            key,
                            kind: match request.kind {
                                DeferredKind::Blessing => PendingContentKind::Blessing,
                                DeferredKind::CurioAny => PendingContentKind::CurioAny,
                                DeferredKind::CurioErrorCode => PendingContentKind::CurioErrorCode,
                                DeferredKind::CurioNegative => PendingContentKind::CurioNegative,
                            },
                            count: request.count,
                            minimum_rarity: request.minimum_rarity,
                            maximum_rarity: request.maximum_rarity,
                        }))
                    }
                    Ok(_) => Some(Err(invalid("invalid partial deferred content request"))),
                    Err(error) => Some(Err(error)),
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    fn compile_bonus(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        let Some(bonus) = &self.bonus else {
            return Ok(None);
        };
        if counter_value(state, PROGRESSION, BONUS_APPLIED_KEY)? != 0 {
            return Err(invalid("Trailblaze Bonus already applied"));
        }
        let fragments = counter_value(state, RESOURCES, COSMIC_FRAGMENTS_KEY)?;
        let updated = fragments
            .checked_add(bonus.immediate_fragments)
            .filter(|value| *value >= 0)
            .ok_or_else(|| invalid("Trailblaze Bonus fragment cost is not payable"))?;
        let countdown = integer_value(state, COUNTDOWN)?;
        countdown
            .checked_add(bonus.countdown_delta)
            .ok_or_else(|| invalid("Trailblaze Bonus Countdown overflow"))?;
        let mut operations = vec![
            require_counter(PROGRESSION, BONUS_APPLIED_KEY, 0),
            require_counter(RESOURCES, COSMIC_FRAGMENTS_KEY, fragments),
            require_integer(COUNTDOWN, countdown),
        ];
        if updated != fragments {
            operations.push(add_counter(
                RESOURCES,
                COSMIC_FRAGMENTS_KEY,
                bonus.immediate_fragments,
            ));
        }
        if bonus.countdown_delta != 0 {
            operations.push(ActivityOperation::AddToSlot {
                slot: slot(COUNTDOWN),
                delta: integer(bonus.countdown_delta),
            });
        }
        for (ordinal, request) in bonus.requests.iter().enumerate() {
            let key = request_key(bonus.id, ordinal, request.kind)?;
            operations.push(require_counter(DEFERRED, key, 0));
            operations.push(add_counter(DEFERRED, key, i64::from(request.count)));
        }
        operations.push(add_counter(PROGRESSION, BONUS_APPLIED_KEY, 1));
        Ok(Some(program(BONUS_PROGRAM_BASE + bonus.id, operations)?))
    }

    fn compile_interplays(
        &self,
        state: &ActivityTransactionState,
        blessing_counts: &[(String, u16)],
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        let counts = canonical_counts(blessing_counts)?;
        let main = counts
            .get(self.path.shared_path.as_ref())
            .copied()
            .unwrap_or(0);
        let mut operations = Vec::new();
        for interplay in &self.path.interplays {
            let key = INTERPLAY_KEY_BASE + u64::from(interplay.id);
            let current = counter_value(state, PROGRESSION, key)?;
            if current > 1 {
                return Err(invalid("invalid Resonance Interplay state"));
            }
            if current == 0
                && main >= interplay.main_threshold
                && counts
                    .get(interplay.sub_path.as_ref())
                    .copied()
                    .unwrap_or(0)
                    >= interplay.sub_threshold
            {
                operations.push(require_counter(PROGRESSION, key, 0));
                operations.push(add_counter(PROGRESSION, key, 1));
            }
        }
        if operations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(program(
                INTERPLAY_PROGRAM_BASE + self.path.id,
                operations,
            )?))
        }
    }

    fn active_interplays<'a>(
        &'a self,
        state: &ActivityTransactionState,
    ) -> Result<Vec<(&'a str, &'a str, &'a str)>, UniverseCatalogLoadError> {
        self.path
            .interplays
            .iter()
            .filter_map(|interplay| {
                match counter_value(
                    state,
                    PROGRESSION,
                    INTERPLAY_KEY_BASE + u64::from(interplay.id),
                ) {
                    Ok(0) => None,
                    Ok(1) => Some(Ok((
                        interplay.key.as_ref(),
                        interplay.sub_path.as_ref(),
                        interplay.binding_key.as_ref(),
                    ))),
                    Ok(_) => Some(Err(invalid("invalid Resonance Interplay state"))),
                    Err(error) => Some(Err(error)),
                }
            })
            .collect()
    }
}

impl SwarmDisasterRuntimeInstance {
    /// Applies the selected run-start bonus once. Random content requests are
    /// committed as deterministic P4-B4-owned pool work without consuming RNG.
    pub fn compile_trailblaze_bonus_run_start(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        self.path_runtime.compile_bonus(state)
    }

    /// The selected Path's exact mode unlock, including Propagation `1000008`.
    pub fn path_progression_unlock_id(&self) -> Option<&str> {
        self.path_runtime.path.mode_unlock.as_deref()
    }

    #[must_use]
    pub const fn path_is_propagation(&self) -> bool {
        self.path_runtime.path.propagation
    }

    /// Selected Path boost as `(row, StageAbility)`.
    pub fn path_boost_binding(&self) -> (&str, &str) {
        (
            &self.path_runtime.path.boost.key,
            &self.path_runtime.path.boost.binding_key,
        )
    }

    /// Base Resonance followed by its three Formations in stable binding order.
    pub fn path_resonance_bindings(
        &self,
    ) -> impl ExactSizeIterator<Item = (&str, &str, u16, &str)> {
        self.path_runtime.path.resonances.iter().map(|resonance| {
            (
                resonance.key.as_ref(),
                resonance.shared_key.as_ref(),
                resonance.threshold,
                resonance.binding_key.as_ref(),
            )
        })
    }

    pub fn path_resonance_parameters(
        &self,
        key: &str,
    ) -> Option<impl ExactSizeIterator<Item = &str>> {
        self.path_runtime
            .path
            .resonances
            .iter()
            .find(|resonance| resonance.key.as_ref() == key)
            .map(|resonance| resonance.parameters.iter().map(AsRef::as_ref))
    }

    /// Activates every newly satisfied main/sub Path `3 + 3` threshold once.
    pub fn compile_resonance_interplays(
        &self,
        state: &ActivityTransactionState,
        distinct_blessing_counts: &[(String, u16)],
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        self.path_runtime
            .compile_interplays(state, distinct_blessing_counts)
    }

    /// Active immutable Interplay contributions as `(row, sub Path, binding)`.
    pub fn active_resonance_interplays<'a>(
        &'a self,
        state: &ActivityTransactionState,
    ) -> Result<Vec<(&'a str, &'a str, &'a str)>, UniverseCatalogLoadError> {
        self.path_runtime.active_interplays(state)
    }

    #[must_use]
    pub const fn path_runtime_digest(&self) -> [u8; 32] {
        self.path_runtime.digest
    }
}

fn compile_bonus(input: &SwarmBonusRuntimeInput) -> Result<RuntimeBonus, UniverseCatalogLoadError> {
    let program = serde_json::from_str::<BonusProgram>(&input.effect_program)
        .map_err(|_| invalid("invalid Trailblaze Bonus effect program"))?;
    if program.transaction.as_ref() != "AtomicAcceptedActivityOperations"
        || program.random_stream.as_ref() != input.key.as_ref()
        || !sha256_text(&program.source_description_sha256_en)
        || !sha256_text(&program.source_description_sha256_zh_cn)
        || program.operations.is_empty()
        || program
            .operations
            .iter()
            .enumerate()
            .any(|(index, operation)| usize::from(operation.order) != index)
    {
        return Err(invalid("Trailblaze Bonus execution contract drift"));
    }
    let mut immediate_fragments = 0_i64;
    let mut countdown_delta = 0_i64;
    let mut requests = Vec::new();
    for operation in &program.operations {
        match operation.operation.as_ref() {
            "AddCosmicFragments" => {
                immediate_fragments = add_exact(
                    immediate_fragments,
                    positive_i64(operation.value.as_deref())?,
                )?;
            }
            "SpendCosmicFragments" => {
                immediate_fragments = add_exact(
                    immediate_fragments,
                    -positive_i64(operation.value.as_deref())?,
                )?;
            }
            "AdjustCountdown" => {
                countdown_delta =
                    add_exact(countdown_delta, signed_i64(operation.value.as_deref())?)?;
            }
            "GrantRandomBlessings" => requests.push(DeferredRequest {
                kind: DeferredKind::Blessing,
                count: positive_u16(operation.count.as_deref())?,
                minimum_rarity: positive_u8(operation.minimum_rarity.as_deref())?,
                maximum_rarity: positive_u8(operation.maximum_rarity.as_deref())?,
            }),
            "GrantRandomCurios" => requests.push(DeferredRequest {
                kind: match operation.category.as_deref() {
                    Some("AnyEligible") => DeferredKind::CurioAny,
                    Some("ErrorCode") => DeferredKind::CurioErrorCode,
                    Some("Negative") => DeferredKind::CurioNegative,
                    _ => return Err(invalid("invalid Trailblaze Bonus Curio category")),
                },
                count: positive_u16(operation.count.as_deref())?,
                minimum_rarity: 0,
                maximum_rarity: 0,
            }),
            _ => return Err(invalid("unsupported Trailblaze Bonus operation")),
        }
        let is_request = matches!(
            operation.operation.as_ref(),
            "GrantRandomBlessings" | "GrantRandomCurios"
        );
        let recognized_pool = operation
            .pool_binding_state
            .as_deref()
            .is_some_and(|state| matches!(state, "DeferredToG09P2B1" | "DeferredToG09P2B2"));
        if is_request != operation.pool_binding_state.is_some()
            || (is_request && !recognized_pool)
            || !valid_bonus_operation_shape(operation)
        {
            return Err(invalid("Trailblaze Bonus pool boundary drift"));
        }
    }
    if requests
        .iter()
        .any(|request| request.minimum_rarity > request.maximum_rarity)
    {
        return Err(invalid("invalid Trailblaze Bonus rarity interval"));
    }
    Ok(RuntimeBonus {
        id: input.id,
        key: input.key.clone(),
        immediate_fragments,
        countdown_delta,
        requests: requests.into_boxed_slice(),
    })
}

fn compile_path(
    input: &SwarmPathRuntimeDefinitionInput,
    boost: Option<&SwarmPathBoostRuntimeInput>,
    resonance_rows: &[&SwarmResonanceRuntimeInput],
    interplay_rows: &[&SwarmInterplayRuntimeInput],
    path_keys: &BTreeMap<u32, &str>,
    pathstrider: &PathstriderRuntimeCatalog,
) -> Result<RuntimePath, UniverseCatalogLoadError> {
    let unlock = serde_json::from_str::<PropagationUnlock>(&input.propagation_unlock)
        .map_err(|_| invalid("invalid Propagation unlock program"))?;
    let propagation = input.shared_path.as_ref() == "universe.path.propagation";
    if unlock.is_propagation != propagation
        || propagation != (unlock.unlock_state.as_ref() == "ReleasedUnlockRowBound")
        || propagation
            != (unlock.required_unlock_id.as_ref() == "swarm-disaster.pathstrider-unlock.1000008")
        || (propagation
            && input.mode_unlock.as_deref().unwrap_or("") != unlock.required_unlock_id.as_ref())
        || input
            .mode_unlock
            .as_deref()
            .is_some_and(|key| !pathstrider.contains_known_unlock(key))
    {
        return Err(invalid("Path/Propagation unlock binding drift"));
    }
    let boost = boost.ok_or_else(|| invalid("Path boost is missing"))?;
    let boost_program = serde_json::from_str::<BoostProgram>(&boost.effect_program)
        .map_err(|_| invalid("invalid Path boost program"))?;
    if boost.id == 0
        || boost_program.operation.as_ref() != "AddMazeBuff"
        || boost_program.level_parameters.as_ref() != [Box::<str>::from("0")]
        || !sha256_text(&boost_program.source_program_sha256)
    {
        return Err(invalid("Path boost execution contract drift"));
    }
    let mut resonances = resonance_rows
        .iter()
        .map(|row| compile_resonance(row))
        .collect::<Result<Vec<_>, _>>()?;
    resonances.sort_unstable_by_key(|row| (u16::from(row.threshold == 0), row.shared_key.clone()));
    let base = resonances
        .first()
        .ok_or_else(|| invalid("Path base Resonance is missing"))?;
    if resonances.len() != 4
        || base.threshold != 3
        || base.energy_max != 100
        || base.initial_energy != 0
        || resonance_rows
            .iter()
            .find(|row| row.id == input.resonance_id)
            .map(|row| row.shared_resonance.as_ref())
            != Some(base.shared_key.as_ref())
        || input
            .formation_keys
            .iter()
            .map(AsRef::as_ref)
            .collect::<BTreeSet<_>>()
            != resonances
                .iter()
                .skip(1)
                .map(|row| row.shared_key.as_ref())
                .collect::<BTreeSet<_>>()
    {
        return Err(invalid("Path Resonance/Formation membership drift"));
    }
    let mut interplays = interplay_rows
        .iter()
        .map(|row| compile_interplay(row, path_keys))
        .collect::<Result<Vec<_>, _>>()?;
    interplays.sort_unstable_by_key(|row| row.id);
    if interplays.len() != 2 {
        return Err(invalid("Path Interplay denominator drift"));
    }
    let battle_event_groups = serde_json::from_str::<Box<[Box<str>]>>(&input.battle_event_groups)
        .map_err(|_| invalid("invalid Path battle event groups"))?;
    if battle_event_groups.len() != 2 || input.extra_effect_keys.is_empty() {
        return Err(invalid("Path battle contribution membership drift"));
    }
    Ok(RuntimePath {
        id: input.id,
        key: input.key.clone(),
        shared_path: input.shared_path.clone(),
        mode_unlock: input.mode_unlock.clone(),
        propagation,
        boost: RuntimeBinding {
            key: boost.key.clone(),
            binding_key: boost_program.stage_ability,
            parameters: boost_program.level_parameters,
        },
        resonances: resonances.into_boxed_slice(),
        interplays: interplays.into_boxed_slice(),
        battle_event_groups,
        extra_effect_keys: input.extra_effect_keys.clone(),
    })
}

fn compile_resonance(
    input: &SwarmResonanceRuntimeInput,
) -> Result<RuntimeResonance, UniverseCatalogLoadError> {
    let program = serde_json::from_str::<ResonanceProgram>(&input.effect_program)
        .map_err(|_| invalid("invalid Resonance binding program"))?;
    let [rule_key] = program.rule_ids.as_ref() else {
        return Err(invalid("Resonance must bind one rule"));
    };
    if program.binding_type.as_ref() != "StageAbilityBeforeCharacterBorn"
        || program.modifier_name.as_ref() != format!("ADV_{}", program.binding_key)
        || rule_key.as_ref()
            != format!(
                "universe.rule.resonance.{}",
                program.binding_key.trim_start_matches("StageAbility_")
            )
        || input.shared_resonance.as_ref()
            != format!(
                "universe.resonance.{}",
                program.binding_key.trim_start_matches("StageAbility_")
            )
    {
        return Err(invalid("Resonance shared binding drift"));
    }
    let parameters = input
        .parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| parse_indexed_parameter(parameter, index + 1))
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    Ok(RuntimeResonance {
        key: input.key.clone(),
        shared_key: input.shared_resonance.clone(),
        threshold: input.threshold,
        energy_max: signed_i64(Some(&input.energy_max))?,
        initial_energy: signed_i64(Some(&input.initial_energy))?,
        parameters,
        mechanic_tags: input.mechanic_tags.clone(),
        binding_key: program.binding_key,
        rule_key: rule_key.clone(),
    })
}

fn compile_interplay(
    input: &SwarmInterplayRuntimeInput,
    path_keys: &BTreeMap<u32, &str>,
) -> Result<RuntimeInterplay, UniverseCatalogLoadError> {
    let threshold = serde_json::from_str::<InterplayThreshold>(&input.thresholds)
        .map_err(|_| invalid("invalid Resonance Interplay threshold"))?;
    let program = serde_json::from_str::<InterplayProgram>(&input.effect_program)
        .map_err(|_| invalid("invalid Resonance Interplay binding"))?;
    if threshold.comparison.as_ref() != "GreaterEqual"
        || threshold.counting_policy.as_ref() != "DistinctOwnedBlessingIdentity"
        || program.binding_type.as_ref() != "StageAbilityBeforeCharacterBorn"
        || program.modifier_name.as_ref() != format!("ADV_{}", program.binding_key)
        || program.maze_buff_id.as_ref() != program.binding_key.trim_start_matches("StageAbility_")
        || program.buff_group_id.is_empty()
        || !sha256_text(&program.source_description_sha256_en)
        || !sha256_text(&program.source_description_sha256_zh_cn)
        || program
            .parameters
            .iter()
            .enumerate()
            .any(|(index, parameter)| {
                usize::from(parameter.index) != index + 1
                    || !canonical_scalar_text(&parameter.value)
            })
    {
        return Err(invalid("Resonance Interplay execution contract drift"));
    }
    let sub_path = path_keys
        .get(&input.sub_path_id)
        .ok_or_else(|| invalid("Interplay sub Path is missing"))?;
    Ok(RuntimeInterplay {
        id: input.id,
        key: input.key.clone(),
        sub_path: (*sub_path).into(),
        main_threshold: positive_u16(Some(&threshold.main_path_blessings))?,
        sub_threshold: positive_u16(Some(&threshold.sub_path_blessings))?,
        binding_key: program.binding_key,
        parameters: program
            .parameters
            .into_iter()
            .map(|parameter| parameter.value)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    })
}

fn unique_by_path<T>(
    rows: &[T],
    key: impl Fn(&T) -> u32,
) -> Result<BTreeMap<u32, &T>, UniverseCatalogLoadError> {
    let mut output = BTreeMap::new();
    for row in rows {
        if output.insert(key(row), row).is_some() {
            return Err(invalid("duplicate Path-owned runtime row"));
        }
    }
    Ok(output)
}

fn canonical_counts(
    input: &[(String, u16)],
) -> Result<BTreeMap<&str, u16>, UniverseCatalogLoadError> {
    let mut counts = BTreeMap::new();
    for (path, count) in input {
        if !path.starts_with("universe.path.") || counts.insert(path.as_str(), *count).is_some() {
            return Err(reference("invalid or duplicate blessing Path count"));
        }
    }
    Ok(counts)
}

fn request_key(
    id: u32,
    ordinal: usize,
    kind: DeferredKind,
) -> Result<u64, UniverseCatalogLoadError> {
    let base = match kind {
        DeferredKind::Blessing => DEFERRED_BLESSING_BASE,
        DeferredKind::CurioAny => DEFERRED_CURIO_ANY_BASE,
        DeferredKind::CurioErrorCode => DEFERRED_CURIO_ERROR_BASE,
        DeferredKind::CurioNegative => DEFERRED_CURIO_NEGATIVE_BASE,
    };
    base.checked_add(u64::from(id) * 16)
        .and_then(|value| value.checked_add(u64::try_from(ordinal).ok()?))
        .ok_or_else(|| invalid("Trailblaze Bonus deferred key overflow"))
}

fn catalog_digest(bonuses: &[RuntimeBonus], paths: &[RuntimePath]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.path-runtime.catalog.v1");
    encoder.text(REVISION);
    for bonus in bonuses {
        encoder.u32(bonus.id);
        encoder.text(&bonus.key);
        encoder.i64(bonus.immediate_fragments);
        encoder.i64(bonus.countdown_delta);
        for request in &bonus.requests {
            encoder.u8(request.kind as u8);
            encoder.u32(u32::from(request.count));
            encoder.u8(request.minimum_rarity);
            encoder.u8(request.maximum_rarity);
        }
    }
    for path in paths {
        encoder.u32(path.id);
        encoder.text(&path.key);
        encoder.text(&path.shared_path);
        encoder.optional_text(path.mode_unlock.as_deref());
        encoder.bool(path.propagation);
        encode_binding(&mut encoder, &path.boost);
        for resonance in &path.resonances {
            encoder.text(&resonance.key);
            encoder.text(&resonance.shared_key);
            encoder.u32(u32::from(resonance.threshold));
            encoder.i64(resonance.energy_max);
            encoder.i64(resonance.initial_energy);
            encoder.text(&resonance.binding_key);
            encoder.text(&resonance.rule_key);
            for value in &resonance.parameters {
                encoder.text(value);
            }
            for tag in &resonance.mechanic_tags {
                encoder.text(tag);
            }
        }
        for interplay in &path.interplays {
            encoder.u32(interplay.id);
            encoder.text(&interplay.key);
            encoder.text(&interplay.sub_path);
            encoder.u32(u32::from(interplay.main_threshold));
            encoder.u32(u32::from(interplay.sub_threshold));
            encoder.text(&interplay.binding_key);
            for parameter in &interplay.parameters {
                encoder.text(parameter);
            }
        }
        for group in &path.battle_event_groups {
            encoder.text(group);
        }
        for effect in &path.extra_effect_keys {
            encoder.text(effect);
        }
    }
    encoder.finish()
}

fn encode_binding(encoder: &mut Encoder, binding: &RuntimeBinding) {
    encoder.text(&binding.key);
    encoder.text(&binding.binding_key);
    for parameter in &binding.parameters {
        encoder.text(parameter);
    }
}

fn sha256_text(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn parse_indexed_parameter(
    input: &str,
    expected_index: usize,
) -> Result<Box<str>, UniverseCatalogLoadError> {
    let body = input
        .strip_prefix("{'index': ")
        .and_then(|value| value.strip_suffix("'}"))
        .ok_or_else(|| invalid("invalid Resonance indexed parameter"))?;
    let (index, value) = body
        .split_once(", 'value': '")
        .ok_or_else(|| invalid("invalid Resonance indexed parameter"))?;
    if index.parse::<usize>().ok() != Some(expected_index) || !canonical_scalar_text(value) {
        return Err(invalid("invalid Resonance parameter order or scalar"));
    }
    Ok(value.into())
}

fn canonical_scalar_text(value: &str) -> bool {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let (integer, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, None), |(integer, fraction)| {
            (integer, Some(fraction))
        });
    let valid_integer = integer == "0"
        || (!integer.starts_with('0') && integer.bytes().all(|byte| byte.is_ascii_digit()));
    let valid_fraction = fraction.is_none_or(|fraction| {
        !fraction.is_empty()
            && !fraction.ends_with('0')
            && fraction.bytes().all(|byte| byte.is_ascii_digit())
    });
    !value.is_empty() && value != "-0" && !value.starts_with('+') && valid_integer && valid_fraction
}

fn valid_bonus_operation_shape(operation: &BonusOperation) -> bool {
    match operation.operation.as_ref() {
        "AddCosmicFragments" | "SpendCosmicFragments" | "AdjustCountdown" => {
            operation.value.is_some()
                && operation.count.is_none()
                && operation.minimum_rarity.is_none()
                && operation.maximum_rarity.is_none()
                && operation.category.is_none()
        }
        "GrantRandomBlessings" => {
            operation.value.is_none()
                && operation.count.is_some()
                && operation.minimum_rarity.is_some()
                && operation.maximum_rarity.is_some()
                && operation.category.is_none()
        }
        "GrantRandomCurios" => {
            operation.value.is_none()
                && operation.count.is_some()
                && operation.minimum_rarity.is_none()
                && operation.maximum_rarity.is_none()
                && operation.category.is_some()
        }
        _ => false,
    }
}

fn add_exact(left: i64, right: i64) -> Result<i64, UniverseCatalogLoadError> {
    left.checked_add(right)
        .ok_or_else(|| invalid("Path runtime arithmetic overflow"))
}

fn signed_i64(value: Option<&str>) -> Result<i64, UniverseCatalogLoadError> {
    value
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| invalid("invalid Path runtime integer"))
}

fn positive_i64(value: Option<&str>) -> Result<i64, UniverseCatalogLoadError> {
    signed_i64(value).and_then(|value| {
        if value > 0 {
            Ok(value)
        } else {
            Err(invalid("expected positive Path runtime integer"))
        }
    })
}

fn positive_u16(value: Option<&str>) -> Result<u16, UniverseCatalogLoadError> {
    value
        .and_then(|value| value.parse().ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| invalid("expected positive Path runtime u16"))
}

fn positive_u8(value: Option<&str>) -> Result<u8, UniverseCatalogLoadError> {
    value
        .and_then(|value| value.parse().ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| invalid("expected positive Path runtime u8"))
}

fn counter_value(
    state: &ActivityTransactionState,
    slot_id: u32,
    key: u64,
) -> Result<i64, UniverseCatalogLoadError> {
    let Some(ActivityValue::BoundedCounterMap(values)) = state.slot(slot(slot_id)) else {
        return Err(invalid("invalid Path runtime counter slot"));
    };
    Ok(values
        .binary_search_by_key(&key, |(candidate, _)| *candidate)
        .ok()
        .map_or(0, |index| values[index].1))
}

fn integer_value(
    state: &ActivityTransactionState,
    slot_id: u32,
) -> Result<i64, UniverseCatalogLoadError> {
    match state.slot(slot(slot_id)) {
        Some(ActivityValue::BoundedInteger(value)) => Ok(*value),
        _ => Err(invalid("invalid Path runtime integer slot")),
    }
}

fn require_counter(slot_id: u32, key: u64, expected: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(
        ActivityExpression::CounterValue {
            slot: slot(slot_id),
            key,
        },
        integer(expected),
    ))
}

fn require_integer(slot_id: u32, expected: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(
        ActivityExpression::Slot(slot(slot_id)),
        integer(expected),
    ))
}

fn add_counter(slot_id: u32, key: u64, delta: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(slot_id),
        key,
        delta: integer(delta),
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("static Swarm slot ID is non-zero")
}

fn program(
    raw: u32,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(raw).expect("static Swarm program ID is non-zero"),
        operations,
    )
    .map_err(|_| invalid("invalid Path runtime Activity program"))
}

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}

fn reference(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BonusProgram {
    operations: Box<[BonusOperation]>,
    random_stream: Box<str>,
    source_description_sha256_en: Box<str>,
    source_description_sha256_zh_cn: Box<str>,
    transaction: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BonusOperation {
    operation: Box<str>,
    order: u16,
    #[serde(default)]
    value: Option<Box<str>>,
    #[serde(default)]
    count: Option<Box<str>>,
    #[serde(default)]
    minimum_rarity: Option<Box<str>>,
    #[serde(default)]
    maximum_rarity: Option<Box<str>>,
    #[serde(default)]
    category: Option<Box<str>>,
    #[serde(default)]
    pool_binding_state: Option<Box<str>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PropagationUnlock {
    is_propagation: bool,
    required_unlock_id: Box<str>,
    unlock_state: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BoostProgram {
    level_parameters: Box<[Box<str>]>,
    operation: Box<str>,
    source_program_sha256: Box<str>,
    stage_ability: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResonanceProgram {
    binding_key: Box<str>,
    binding_type: Box<str>,
    modifier_name: Box<str>,
    rule_ids: Box<[Box<str>]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InterplayThreshold {
    comparison: Box<str>,
    counting_policy: Box<str>,
    main_path_blessings: Box<str>,
    sub_path_blessings: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InterplayProgram {
    binding_key: Box<str>,
    binding_type: Box<str>,
    buff_group_id: Box<str>,
    maze_buff_id: Box<str>,
    modifier_name: Box<str>,
    parameters: Box<[InterplayParameter]>,
    source_description_sha256_en: Box<str>,
    source_description_sha256_zh_cn: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InterplayParameter {
    index: u16,
    value: Box<str>,
}

#[cfg(test)]
#[path = "path_runtime_tests.rs"]
mod tests;
