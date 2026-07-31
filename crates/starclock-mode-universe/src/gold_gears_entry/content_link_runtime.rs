//! Released shared-content links consumed by the Gold and Gears profile.

use std::sync::Arc;

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityInventoryId, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivityRngLabel, ActivityRngStreams,
    ActivityValue,
};

use crate::{
    blessing_runtime::{BlessingContributionSet, BlessingOfferEligibility, BlessingRuntimeCatalog},
    catalog::UniverseCatalog,
    curio_runtime::CurioRuntimeCatalog,
    gold_gears_content::GoldAndGearsContentCatalog,
    id::BlessingId,
    path_runtime::PathRuntimeCatalog,
};

use super::{
    GoldAndGearsEntryError,
    api::{GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance},
    curio_runtime::GoldAndGearsCurioRuntimeCatalog,
    occurrence_runtime::GoldAndGearsOccurrenceRuntimeCatalog,
    path_boost_rule_runtime::GoldAndGearsPathBoostRuleRuntimeCatalog,
    progression_runtime::ProgressionRuntimeCatalog,
    resonance_rule_runtime::GoldAndGearsResonanceRuleRuntimeCatalog,
    service_adventure_runtime::GoldAndGearsServiceAdventureRuntimeCatalog,
    state_layout::BLESSING_INVENTORY,
};

pub const GOLD_AND_GEARS_SHARED_CONTENT_RUNTIME_REVISION: &str =
    "gold-and-gears-shared-content-runtime-v1";

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../../config/universe-generated/config.sora");
const TRAILBLAZE_BLESSING_PURPOSE: u16 = 0x4771;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldAndGearsSharedContentDigests {
    blessing: [u8; 32],
    path: [u8; 32],
    curio: [u8; 32],
}

impl GoldAndGearsSharedContentDigests {
    #[must_use]
    pub const fn blessing(self) -> [u8; 32] {
        self.blessing
    }

    #[must_use]
    pub const fn path(self) -> [u8; 32] {
        self.path
    }

    #[must_use]
    pub const fn curio(self) -> [u8; 32] {
        self.curio
    }
}

#[derive(Clone, Debug)]
pub(super) struct GoldAndGearsContentRuntimeCatalog {
    pub(super) blessings: Arc<BlessingRuntimeCatalog>,
    pub(super) paths: Arc<PathRuntimeCatalog>,
    pub(super) shared_curios: Arc<CurioRuntimeCatalog>,
    pub(super) curios: Arc<GoldAndGearsCurioRuntimeCatalog>,
    pub(super) occurrences: Arc<GoldAndGearsOccurrenceRuntimeCatalog>,
    pub(super) service_adventure: Arc<GoldAndGearsServiceAdventureRuntimeCatalog>,
    pub(super) path_boost_rules: Arc<GoldAndGearsPathBoostRuleRuntimeCatalog>,
    pub(super) resonance_rules: Arc<GoldAndGearsResonanceRuleRuntimeCatalog>,
    digests: GoldAndGearsSharedContentDigests,
}

impl GoldAndGearsContentRuntimeCatalog {
    pub(super) fn compile(
        content: &GoldAndGearsContentCatalog,
        unique: &crate::gold_gears_unique::GoldAndGearsUniqueCatalog,
        progression: &ProgressionRuntimeCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let core = starclock_data::catalog::load(CORE_BUNDLE)
            .map_err(|_| GoldAndGearsEntryError::InvalidSharedContentRuntime)?;
        let standard = UniverseCatalog::load(UNIVERSE_BUNDLE, core)
            .map_err(|_| GoldAndGearsEntryError::InvalidSharedContentRuntime)?;
        validate_blessing_links(content, &standard)?;
        let blessings = Arc::new(
            BlessingRuntimeCatalog::compile(&standard)
                .map_err(|_| GoldAndGearsEntryError::InvalidSharedContentRuntime)?,
        );
        let paths = Arc::new(
            PathRuntimeCatalog::compile(&standard)
                .map_err(|_| GoldAndGearsEntryError::InvalidSharedContentRuntime)?,
        );
        let shared_curios = Arc::new(
            CurioRuntimeCatalog::compile(&standard)
                .map_err(|_| GoldAndGearsEntryError::InvalidSharedContentRuntime)?,
        );
        let curios = Arc::new(GoldAndGearsCurioRuntimeCatalog::compile(
            content,
            &standard,
            &shared_curios,
        )?);
        let occurrences = Arc::new(GoldAndGearsOccurrenceRuntimeCatalog::compile(content)?);
        let service_adventure = Arc::new(GoldAndGearsServiceAdventureRuntimeCatalog::compile(
            content, &standard,
        )?);
        let path_boost_rules = Arc::new(GoldAndGearsPathBoostRuleRuntimeCatalog::compile(
            content, unique, &standard, &blessings,
        )?);
        let resonance_rules = Arc::new(GoldAndGearsResonanceRuleRuntimeCatalog::compile(
            unique,
            &standard,
            progression,
        )?);
        let digests = GoldAndGearsSharedContentDigests {
            blessing: blessings.digest(),
            path: paths.digest(),
            curio: shared_curios.digest(),
        };
        Ok(Self {
            blessings,
            paths,
            shared_curios,
            curios,
            occurrences,
            service_adventure,
            path_boost_rules,
            resonance_rules,
            digests,
        })
    }

    pub(super) const fn digests(&self) -> GoldAndGearsSharedContentDigests {
        self.digests
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.blessings.definitions().len(),
            self.blessings
                .definitions()
                .iter()
                .map(|definition| usize::from(definition.level(1).is_some()))
                .sum::<usize>()
                * 2,
            self.paths.len(),
            self.shared_curios.definitions().len(),
            self.curios.definitions().len(),
        )
    }
}

impl GoldAndGearsRuntimeFactory {
    #[must_use]
    pub fn shared_content_digests(&self) -> GoldAndGearsSharedContentDigests {
        self.content_runtime.digests()
    }

    #[must_use]
    pub fn shared_path_count(&self) -> usize {
        self.content_runtime.paths.len()
    }

    #[must_use]
    pub fn shared_curio_count(&self) -> usize {
        self.content_runtime.shared_curios.definitions().len()
    }
}

impl GoldAndGearsRuntimeInstance {
    #[must_use]
    pub fn shared_content_digests(&self) -> GoldAndGearsSharedContentDigests {
        self.content_runtime.digests()
    }

    #[must_use]
    pub fn shared_path_count(&self) -> usize {
        self.content_runtime.paths.len()
    }

    #[must_use]
    pub fn shared_curio_count(&self) -> usize {
        self.content_runtime.shared_curios.definitions().len()
    }

    pub fn select_trailblaze_blessing(
        &self,
        owned: &[BlessingId],
        rng: &mut ActivityRngStreams,
    ) -> Result<Option<BlessingId>, GoldAndGearsEntryError> {
        let mut owned = owned.to_vec();
        owned.sort_unstable();
        if owned.windows(2).any(|pair| pair[0] == pair[1])
            || owned
                .iter()
                .any(|id| self.content_runtime.blessings.definition(*id).is_none())
        {
            return Err(GoldAndGearsEntryError::InvalidBlessingInventory);
        }
        let policy = BlessingOfferEligibility::fully_unlocked(vec![1, 2])
            .map_err(|_| GoldAndGearsEntryError::InvalidSharedContentRuntime)?;
        let candidates = self
            .content_runtime
            .blessings
            .eligible(&policy)
            .map(|definition| definition.blessing())
            .filter(|id| owned.binary_search(id).is_err())
            .collect::<Vec<_>>();
        let Some(draw) = rng
            .choose_index(
                ActivityRngLabel::Reward,
                TRAILBLAZE_BLESSING_PURPOSE,
                u32::try_from(candidates.len())
                    .map_err(|_| GoldAndGearsEntryError::InvalidSharedContentRuntime)?,
            )
            .map_err(|_| GoldAndGearsEntryError::InvalidSharedContentRuntime)?
        else {
            return Ok(None);
        };
        candidates
            .get(draw.value() as usize)
            .copied()
            .map(Some)
            .ok_or(GoldAndGearsEntryError::InvalidSharedContentRuntime)
    }

    pub fn compile_blessing_acquisition(
        &self,
        blessing: BlessingId,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        if self
            .content_runtime
            .blessings
            .definition(blessing)
            .is_none()
        {
            return Err(GoldAndGearsEntryError::UnknownBlessing(blessing.get()));
        }
        let inventory =
            ActivityInventoryId::new(BLESSING_INVENTORY).expect("static inventory is non-zero");
        ActivityProgramDefinition::new(
            ActivityProgramId::new(0x4A00_0000 + blessing.get())
                .expect("Blessing program ID is non-zero"),
            vec![
                ActivityOperation::Require(ActivityCondition::Equal(
                    ActivityExpression::InventoryCount {
                        inventory,
                        content: u64::from(blessing.get()),
                    },
                    integer(0),
                )),
                ActivityOperation::AddInventory {
                    inventory,
                    content: u64::from(blessing.get()),
                    count: integer(1),
                },
            ],
        )
        .map_err(|_| GoldAndGearsEntryError::InvalidSharedContentRuntime)
    }

    pub fn compile_blessing_enhancement(
        &self,
        blessing: BlessingId,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let operations = self
            .content_runtime
            .blessings
            .enhancement_operations(blessing_inventory(), blessing)
            .ok_or(GoldAndGearsEntryError::UnknownBlessing(blessing.get()))?;
        ActivityProgramDefinition::new(
            ActivityProgramId::new(0x4A01_0000 + blessing.get())
                .expect("Blessing enhancement program ID is non-zero"),
            operations.into_vec(),
        )
        .map_err(|_| GoldAndGearsEntryError::InvalidSharedContentRuntime)
    }

    pub fn compile_blessing_replacement(
        &self,
        removed: BlessingId,
        acquired: BlessingId,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let operations = self
            .content_runtime
            .blessings
            .replacement_operations(blessing_inventory(), removed, acquired)
            .ok_or(GoldAndGearsEntryError::InvalidBlessingInventory)?;
        let id = 0x4A02_0000_u32
            .checked_add(
                removed
                    .get()
                    .checked_mul(512)
                    .and_then(|value| value.checked_add(acquired.get()))
                    .ok_or(GoldAndGearsEntryError::InvalidSharedContentRuntime)?,
            )
            .ok_or(GoldAndGearsEntryError::InvalidSharedContentRuntime)?;
        ActivityProgramDefinition::new(
            ActivityProgramId::new(id)
                .ok_or(GoldAndGearsEntryError::InvalidSharedContentRuntime)?,
            operations.into_vec(),
        )
        .map_err(|_| GoldAndGearsEntryError::InvalidSharedContentRuntime)
    }

    pub fn blessing_contributions(
        &self,
        owned: &[(BlessingId, u32)],
    ) -> Result<BlessingContributionSet, GoldAndGearsEntryError> {
        self.content_runtime
            .blessings
            .contributions_from_owned(owned)
            .map_err(|_| GoldAndGearsEntryError::InvalidBlessingInventory)
    }
}

fn validate_blessing_links(
    content: &GoldAndGearsContentCatalog,
    standard: &UniverseCatalog,
) -> Result<(), GoldAndGearsEntryError> {
    if content.blessings.len() != 162 || content.blessing_levels.len() != 324 {
        return Err(GoldAndGearsEntryError::InvalidSharedContentRuntime);
    }
    for blessing in &content.blessings {
        if standard
            .blessings()
            .iter()
            .all(|candidate| candidate.stable_key() != blessing.key.as_str())
        {
            return Err(GoldAndGearsEntryError::InvalidSharedContentRuntime);
        }
    }
    for level in &content.blessing_levels {
        let Some(shared) = standard
            .blessing_levels()
            .iter()
            .find(|candidate| candidate.stable_key() == level.key.as_str())
        else {
            return Err(GoldAndGearsEntryError::InvalidSharedContentRuntime);
        };
        let authored: Vec<IndexedDecimal> = serde_json::from_str(level.parameters.as_str())
            .map_err(|_| GoldAndGearsEntryError::InvalidSharedContentRuntime)?;
        if authored.len() != shared.parameters().len()
            || authored.iter().zip(shared.parameters()).enumerate().any(
                |(index, (authored, shared))| {
                    authored.index != index + 1
                        || exact_decimal(&authored.value)
                            != Some((shared.coefficient(), shared.scale()))
                },
            )
        {
            return Err(GoldAndGearsEntryError::InvalidSharedContentRuntime);
        }
    }
    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct IndexedDecimal {
    index: usize,
    value: Box<str>,
}

fn exact_decimal(value: &str) -> Option<(i64, u8)> {
    let negative = value.starts_with('-');
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let (whole, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, ""), |parts| parts);
    if fraction.len() > usize::from(u8::MAX) {
        return None;
    }
    let scale = u8::try_from(fraction.len()).ok()?;
    let coefficient = format!("{whole}{fraction}").parse::<i64>().ok()?;
    Some((if negative { -coefficient } else { coefficient }, scale))
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn blessing_inventory() -> ActivityInventoryId {
    ActivityInventoryId::new(BLESSING_INVENTORY).expect("static inventory is non-zero")
}
