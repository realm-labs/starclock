//! Caller-admitted equipped Weighted Curios, separate from ordinary holdings.
//! Equipment mutation is executable; unresolved battle effects fail closed.

use crate::digest::CanonicalDigestBuilder;
use starclock_data::divergent_universe_decisions::weighted_curio_attack_debuffs::WeightedCurioAttackDebuffDefinition;
use starclock_data::divergent_universe_decisions::weighted_curio_prayers::WeightedCurioPrayerDefinition;
use starclock_data::divergent_universe_decisions::weighted_curio_shields::WeightedCurioShieldDefinition;
use starclock_data::divergent_universe_decisions::weighted_curio_splashes::WeightedCurioSplashDefinition;
use starclock_data::divergent_universe_decisions::weighted_curio_support_attacks::WeightedCurioSupportAttackDefinition;
use std::sync::Arc;

use crate::divergent_universe::{
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory,
    state::WEIGHTED_CURIO_REFERENCES_SLOT,
};
use starclock_activity::{
    ActivityOperation, ActivityProgramDefinition, ActivityProgramId, ActivityStateHash,
    ActivityTransactionEvent, ActivityValue, GraphActivity, GraphActivityCommandError,
};
use starclock_data::divergent_universe_curio_catalog::{
    DivergentUniverseHexContentKind, DivergentUniverseWeightedCurioId,
};

const REPLACE_PROGRAM: u32 = 22_560;
pub(super) const POLICY_IDENTITY: &[u8] =
    b"weighted-curio.accepted-current-catalog-set.one-through-three.slots-count-one.unlowered-effects-reject";

/// Explicit caller-selected number of available equipment slots, not an inferred
/// Forge-level selector. A profile must bind this input and admit its service
/// before invoking the accepted boundary. Current bounded composition allows
/// one through three; this does not implement domain enhancement or divination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WeightedCurioSlotLimit(u16);

impl WeightedCurioSlotLimit {
    pub fn new(value: u16) -> Result<Self, WeightedCurioError> {
        if !(1..=3).contains(&value) {
            return Err(WeightedCurioError::InvalidSlotLimit);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug)]
pub enum WeightedCurioError {
    InvalidCatalog,
    InvalidSlotLimit,
    UnknownSelection,
    DuplicateSelection,
    CapacityExceeded,
    InvalidState,
    UnchangedLoadout,
    DefinitionMismatch,
    ActivityCompleted,
    UnsupportedBattleEffect(DivergentUniverseWeightedCurioId),
    Command(GraphActivityCommandError),
}

impl std::fmt::Display for WeightedCurioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Divergent Universe Weighted Curio loadout: {self:?}")
    }
}
impl std::error::Error for WeightedCurioError {}

/// Immutable current Sora identities and their canonical equipment keys.
/// Inline path/element rows remain recommendations, not selection restrictions:
/// no unproven eligible pool, weight or effect activation is invented here.
#[derive(Clone, Debug)]
pub struct WeightedCurioRuntime {
    ids: Arc<[DivergentUniverseWeightedCurioId]>,
    component: [u8; 32],
    pub(super) splashes: Box<[WeightedCurioSplashDefinition]>,
    pub(super) shields: Box<[WeightedCurioShieldDefinition]>,
    pub(super) attack_debuffs: Box<[WeightedCurioAttackDebuffDefinition]>,
    pub(super) support_attacks: Box<[WeightedCurioSupportAttackDefinition]>,
    pub(super) prayers: Box<[WeightedCurioPrayerDefinition]>,
    decision_digest: [u8; 32],
}

/// Validated equipped effects at an immutable battle handoff. Unsupported
/// identities reject construction, rather than becoming digest-only effects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedCurioSnapshot {
    equipped: Box<[DivergentUniverseWeightedCurioId]>,
    digest: [u8; 32],
}

impl WeightedCurioSnapshot {
    #[must_use]
    pub fn equipped(&self) -> &[DivergentUniverseWeightedCurioId] {
        &self.equipped
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

impl DivergentUniverseRuntimeFactory {
    pub fn weighted_curio_runtime(&self) -> Result<WeightedCurioRuntime, WeightedCurioError> {
        let definitions = self.bundle.curio_catalog().weighted_curios();
        if definitions.len() != 17
            || definitions.iter().any(|row| {
                row.content_kind != DivergentUniverseHexContentKind::WeightedCurio
                    || row.runtime_lowered
            })
        {
            return Err(WeightedCurioError::InvalidCatalog);
        }
        let mut ids = definitions
            .iter()
            .map(|row| row.id.clone())
            .collect::<Vec<_>>();
        ids.sort_unstable();
        if ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(WeightedCurioError::InvalidCatalog);
        }
        Ok(WeightedCurioRuntime {
            ids: ids.into(),
            component: self.bundle_identity().component_digest().bytes(),
            splashes: self.decision_catalog().weighted_curio_splashes().into(),
            shields: self.decision_catalog().weighted_curio_shields().into(),
            attack_debuffs: self
                .decision_catalog()
                .weighted_curio_attack_debuffs()
                .into(),
            decision_digest: self.decision_catalog().digest(),
            prayers: self.decision_catalog().weighted_curio_prayers().into(),
            support_attacks: self
                .decision_catalog()
                .weighted_curio_support_attacks()
                .into(),
        })
    }
}

impl WeightedCurioRuntime {
    #[must_use]
    pub fn candidates(&self) -> &[DivergentUniverseWeightedCurioId] {
        &self.ids
    }

    /// Observes the equipped set in stable current catalog order without RNG.
    /// Unknown keys, zero/duplicate counts and more than three holdings reject.
    /// This does not return ordinary Curios or claim battle-effect activation.
    pub fn equipped(
        &self,
        activity: &GraphActivity,
    ) -> Result<Vec<DivergentUniverseWeightedCurioId>, WeightedCurioError> {
        let view = activity.player_view();
        let slot = view
            .slots()
            .iter()
            .find(|slot| slot.id() == WEIGHTED_CURIO_REFERENCES_SLOT)
            .ok_or(WeightedCurioError::InvalidState)?;
        let ActivityValue::BoundedCounterMap(values) = slot.value() else {
            return Err(WeightedCurioError::InvalidState);
        };
        if values.len() > 3 {
            return Err(WeightedCurioError::InvalidState);
        }
        values
            .iter()
            .map(|(key, count)| {
                if *count != 1 {
                    return Err(WeightedCurioError::InvalidState);
                }
                let index = key
                    .checked_sub(1)
                    .and_then(|value| usize::try_from(value).ok())
                    .ok_or(WeightedCurioError::InvalidState)?;
                self.ids
                    .get(index)
                    .cloned()
                    .ok_or(WeightedCurioError::InvalidState)
            })
            .collect()
    }

    /// Trusted accepted-service boundary, not an unbound player command.
    /// The owner must authenticate its offered choice and declared slot limit.
    /// Replaces the complete equipped set atomically through shared Activity
    /// operations; an empty selection unequips. This never grants ordinary
    /// Curios, charges currency, draws RNG or executes a battle effect.
    /// Stale, foreign, repeated, invalid and over-capacity requests are inert.
    pub fn replace_accepted_loadout(
        &self,
        flow: &DivergentUniverseFlowInstance,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        limit: WeightedCurioSlotLimit,
        selection: &[DivergentUniverseWeightedCurioId],
    ) -> Result<Vec<ActivityTransactionEvent>, WeightedCurioError> {
        if expected != activity.state_hash() {
            return Err(WeightedCurioError::Command(
                GraphActivityCommandError::StaleStateHash,
            ));
        }
        let definition = activity.definition();
        let owned = flow.definition();
        let matches = Arc::ptr_eq(definition, owned)
            || (definition.identity() == owned.identity()
                && definition.graph() == owned.graph()
                && definition.programs() == owned.programs()
                && definition.state_definition() == owned.state_definition()
                && definition.participants().digest() == owned.participants().digest()
                && definition.bootstrap() == owned.bootstrap()
                && definition.random_checkpoints() == owned.random_checkpoints()
                && definition.random_offers() == owned.random_offers()
                && definition.interactions().is_none()
                && owned.interactions().is_none());
        if flow.component_digest != self.component || !matches {
            return Err(WeightedCurioError::DefinitionMismatch);
        }
        if activity.player_view().terminal().is_some() {
            return Err(WeightedCurioError::ActivityCompleted);
        }
        let current = self.equipped(activity)?;
        if selection.len() > usize::from(limit.get()) {
            return Err(WeightedCurioError::CapacityExceeded);
        }
        let mut selected = selection.to_vec();
        selected.sort_unstable();
        if selected.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(WeightedCurioError::DuplicateSelection);
        }
        let values = selected
            .iter()
            .map(|id| {
                let index = self
                    .ids
                    .binary_search(id)
                    .map_err(|_| WeightedCurioError::UnknownSelection)?;
                let key = u64::try_from(index)
                    .ok()
                    .and_then(|value| value.checked_add(1))
                    .ok_or(WeightedCurioError::InvalidCatalog)?;
                Ok((key, 1))
            })
            .collect::<Result<Vec<_>, WeightedCurioError>>()?;
        if current == selected {
            return Err(WeightedCurioError::UnchangedLoadout);
        }
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(REPLACE_PROGRAM).expect("fixed nonzero equipment program"),
            vec![ActivityOperation::SetCounterMap {
                slot: WEIGHTED_CURIO_REFERENCES_SLOT,
                values: values.into_boxed_slice(),
            }],
        )
        .map_err(|_| WeightedCurioError::InvalidCatalog)?;
        activity
            .apply_boundary_program(expected, &program)
            .map(|events| events.into_vec())
            .map_err(WeightedCurioError::Command)
    }

    /// Validate equipped state and actual current effect admission before
    /// assembly/cache lookup. Unlowered identities cannot be silently omitted.
    pub(super) fn snapshot(
        &self,
        activity: &GraphActivity,
    ) -> Result<WeightedCurioSnapshot, WeightedCurioError> {
        let equipped = self.equipped(activity)?;
        for id in &equipped {
            if !self
                .splashes
                .iter()
                .any(|definition| &definition.weighted_curio == id)
                && !self
                    .shields
                    .iter()
                    .any(|definition| &definition.weighted_curio == id)
                && !self
                    .attack_debuffs
                    .iter()
                    .any(|definition| &definition.weighted_curio == id)
                && !self
                    .support_attacks
                    .iter()
                    .any(|definition| &definition.weighted_curio == id)
                && !self
                    .prayers
                    .iter()
                    .any(|definition| &definition.weighted_curio == id)
            {
                return Err(WeightedCurioError::UnsupportedBattleEffect(id.clone()));
            }
        }
        let mut hash = CanonicalDigestBuilder::new();
        hash.update(b"starclock.divergent-universe.weighted-curio-snapshot");
        hash.update(self.component);
        hash.update(self.decision_digest);
        hash.update(activity.state_hash().bytes());
        for id in &equipped {
            hash.update(id.as_str().as_bytes());
            hash.update([0]);
        }
        Ok(WeightedCurioSnapshot {
            equipped: equipped.into_boxed_slice(),
            digest: hash.finalize(),
        })
    }
}
