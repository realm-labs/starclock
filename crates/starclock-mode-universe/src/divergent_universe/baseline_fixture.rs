//! Production-backed immutable fixture for headless complete-run adapters.

use std::sync::Arc;

use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{
    BuildDigest, LoadoutLockScope, OpaqueParticipantBuild, ParticipantId, ParticipantLock,
    ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
};
use starclock_build::{
    ability::AbilityInvestment,
    compiler::LoadoutCompiler,
    spec::{CombatantBuildSpec, EidolonLevel, PromotionStage},
};
use starclock_combat::UnitLevel;
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_catalog::{
        DivergentUniverseAreaId, DivergentUniverseCyclicalChallengeId,
        DivergentUniverseDifficultyId, DivergentUniverseRunFamily,
    },
    divergent_universe_mapping_catalog::DivergentUniverseAvatarLocator,
};
use starclock_replay::{
    component::{
        ConfigurationComponentIdentity, ConfigurationComponentKind, ConfigurationComponentSet,
    },
    digest::ComponentDigest,
};

use crate::baseline_controller::ActivityBaselineHints;

use super::{
    DivergentUniverseAccountSnapshotDigest, DivergentUniverseBaselinePolicy,
    DivergentUniverseCyclicalRefresh, DivergentUniverseEntry, DivergentUniverseEntryFlowError,
    DivergentUniverseFlowInstance, DivergentUniverseInputSnapshot,
    DivergentUniverseLoadoutSnapshotDigest, DivergentUniverseMappingInput,
    DivergentUniverseMappingRuntimeError, DivergentUniverseMappingSnapshot,
    DivergentUniverseRuntimeFactory,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../config/generated/config.sora");
const ORDINARY_AREA: &str = "divergent-universe.area.401";
const CYCLICAL_AREA: &str = "divergent-universe.area.20401";
const DIFFICULTY: &str = "divergent-universe.difficulty.3011";

/// Exact production catalogs, participant lock and Arithmetic Mapping used by
/// CLI and later bounded adapters.
#[derive(Clone, Debug)]
pub struct DivergentUniverseBaselineFixture {
    factory: DivergentUniverseRuntimeFactory,
    core: Arc<SimulationCatalog>,
    participants: Arc<ParticipantLock>,
    mapping: Arc<DivergentUniverseMappingSnapshot>,
}

impl DivergentUniverseBaselineFixture {
    pub fn production() -> Result<Self, DivergentUniverseBaselineFixtureError> {
        let factory = DivergentUniverseRuntimeFactory::production()
            .map_err(DivergentUniverseBaselineFixtureError::Entry)?;
        let core = starclock_data::catalog::load(CORE_BUNDLE)
            .map_err(|_| DivergentUniverseBaselineFixtureError::CoreCatalog)?;
        let builds = [1308_u32, 1005, 1009, 1105]
            .into_iter()
            .map(|avatar| {
                let form = core
                    .character_form_for_source_avatar(avatar)
                    .ok_or(DivergentUniverseBaselineFixtureError::Build)?;
                Ok((avatar, mapped_build(&core, form)?))
            })
            .collect::<Result<Vec<_>, DivergentUniverseBaselineFixtureError>>()?;
        let participants = Arc::new(participant_lock(&core, &builds)?);
        let inputs = builds
            .into_iter()
            .enumerate()
            .map(|(index, (avatar, build))| {
                Ok(DivergentUniverseMappingInput::new(
                    participant(index)?,
                    DivergentUniverseAvatarLocator::new(avatar.to_string())
                        .map_err(|_| DivergentUniverseBaselineFixtureError::Build)?,
                    None,
                    build,
                ))
            })
            .collect::<Result<Vec<_>, DivergentUniverseBaselineFixtureError>>()?;
        let mapping = Arc::new(
            factory
                .compile_mapping(&participants, &core, inputs)
                .map_err(DivergentUniverseBaselineFixtureError::Mapping)?,
        );
        Ok(Self {
            factory,
            core,
            participants,
            mapping,
        })
    }

    #[must_use]
    pub const fn factory(&self) -> &DivergentUniverseRuntimeFactory {
        &self.factory
    }
    #[must_use]
    pub const fn core(&self) -> &Arc<SimulationCatalog> {
        &self.core
    }
    #[must_use]
    pub const fn participants(&self) -> &Arc<ParticipantLock> {
        &self.participants
    }
    #[must_use]
    pub const fn mapping(&self) -> &Arc<DivergentUniverseMappingSnapshot> {
        &self.mapping
    }

    pub fn flow(
        &self,
        family: DivergentUniverseRunFamily,
    ) -> Result<DivergentUniverseFlowInstance, DivergentUniverseBaselineFixtureError> {
        let area = match family {
            DivergentUniverseRunFamily::Ordinary => ORDINARY_AREA,
            DivergentUniverseRunFamily::Cyclical => CYCLICAL_AREA,
        };
        self.flow_for_selection(family, area, DIFFICULTY)
    }

    pub fn flow_for_selection(
        &self,
        family: DivergentUniverseRunFamily,
        area: &str,
        difficulty: &str,
    ) -> Result<DivergentUniverseFlowInstance, DivergentUniverseBaselineFixtureError> {
        self.flow_for_configuration(family, area, difficulty, None)
    }

    /// Explicit optional service admission for headless execution. This does not
    /// change the default baseline or claim original Forge-domain generation.
    pub fn flow_with_tawot_service(
        &self,
        family: DivergentUniverseRunFamily,
        forge_level: u16,
    ) -> Result<DivergentUniverseFlowInstance, DivergentUniverseBaselineFixtureError> {
        let area = match family {
            DivergentUniverseRunFamily::Ordinary => ORDINARY_AREA,
            DivergentUniverseRunFamily::Cyclical => CYCLICAL_AREA,
        };
        self.flow_for_configuration(family, area, DIFFICULTY, Some(forge_level))
    }

    /// Rebuilds one exact current entry; service admission is explicit and
    /// validated against authored levels, never inferred from a replay hash.
    pub fn flow_for_configuration(
        &self,
        family: DivergentUniverseRunFamily,
        area: &str,
        difficulty: &str,
        tawot: Option<u16>,
    ) -> Result<DivergentUniverseFlowInstance, DivergentUniverseBaselineFixtureError> {
        self.flow_for_entry_configuration(family, area, difficulty, tawot, false)
    }

    /// Optional explicit source-deck choice, without original mask-pool claims.
    pub fn flow_with_source_deck_selection(
        &self,
        family: DivergentUniverseRunFamily,
    ) -> Result<DivergentUniverseFlowInstance, DivergentUniverseBaselineFixtureError> {
        let area = match family {
            DivergentUniverseRunFamily::Ordinary => ORDINARY_AREA,
            DivergentUniverseRunFamily::Cyclical => CYCLICAL_AREA,
        };
        self.flow_for_entry_configuration(family, area, DIFFICULTY, None, true)
    }

    pub(super) fn flow_for_entry_configuration(
        &self,
        family: DivergentUniverseRunFamily,
        area: &str,
        difficulty: &str,
        tawot: Option<u16>,
        source_deck_selection: bool,
    ) -> Result<DivergentUniverseFlowInstance, DivergentUniverseBaselineFixtureError> {
        let snapshot = DivergentUniverseInputSnapshot::seal(
            DivergentUniverseAccountSnapshotDigest::new([0x22; 32])
                .map_err(|_| DivergentUniverseBaselineFixtureError::Snapshot)?,
            DivergentUniverseLoadoutSnapshotDigest::new([0x44; 32])
                .map_err(|_| DivergentUniverseBaselineFixtureError::Snapshot)?,
            &self.participants,
        );
        let mut entry = DivergentUniverseEntry::new(
            DivergentUniverseAreaId::new(area)
                .map_err(|_| DivergentUniverseBaselineFixtureError::EntryIdentity)?,
            DivergentUniverseDifficultyId::new(difficulty)
                .map_err(|_| DivergentUniverseBaselineFixtureError::EntryIdentity)?,
            Arc::clone(&self.participants),
            snapshot,
            Vec::new(),
        )
        .map_err(DivergentUniverseBaselineFixtureError::Entry)?
        .with_mapping_snapshot(Arc::clone(&self.mapping))
        .with_layer_battle_route()
        .with_initial_equation();
        // Versioned project policy: expose the first authored occurrence at the
        // initial logical checkpoint. This is not a released room-pool claim.
        let occurrence = self
            .factory
            .decision_catalog()
            .occurrences()
            .iter()
            .min_by(|left, right| left.variant.as_str().cmp(right.variant.as_str()))
            .ok_or(DivergentUniverseBaselineFixtureError::EntryIdentity)?;
        entry = entry.with_initial_occurrence(occurrence.variant.clone());
        if let Some(level) = tawot {
            entry = entry.with_initial_tawot_service(level);
        }
        if source_deck_selection {
            entry = entry.with_source_deck_selection();
        }
        if family == DivergentUniverseRunFamily::Cyclical {
            let area_source = area
                .strip_prefix("divergent-universe.area.")
                .ok_or(DivergentUniverseBaselineFixtureError::EntryIdentity)?;
            entry = entry.with_cyclical_refresh(
                DivergentUniverseCyclicalRefresh::new(
                    1,
                    DivergentUniverseCyclicalChallengeId::new(format!(
                        "divergent-universe.cyclical-area.{area_source}"
                    ))
                    .map_err(|_| DivergentUniverseBaselineFixtureError::EntryIdentity)?,
                )
                .map_err(|_| DivergentUniverseBaselineFixtureError::EntryIdentity)?,
            );
        }
        let flow = self
            .factory
            .compile(entry)
            .map_err(DivergentUniverseBaselineFixtureError::Entry)?;
        if flow.run_family() != family {
            return Err(DivergentUniverseBaselineFixtureError::EntryIdentity);
        }
        Ok(flow)
    }

    pub fn policy(
        &self,
    ) -> Result<DivergentUniverseBaselinePolicy, DivergentUniverseBaselineFixtureError> {
        let pool = self.factory.decision_catalog().encounter_pool();
        DivergentUniverseBaselinePolicy::new(
            ActivityBaselineHints::default(),
            pool.encounter_group.clone(),
            pool.first_stage.clone(),
            32,
        )
        .map_err(|_| DivergentUniverseBaselineFixtureError::Encounter)
    }

    pub fn components(
        &self,
        flow: &DivergentUniverseFlowInstance,
    ) -> Result<ConfigurationComponentSet, DivergentUniverseBaselineFixtureError> {
        let identity = flow.definition().identity();
        // The decision digest already binds its schema/binary and the exact
        // reference component. Replay must reject either input changing.
        let content = self.factory.decision_catalog().digest();
        let encounter = digest(&[
            b"starclock.divergent-universe.baseline-encounter.v1",
            &content,
        ]);
        component_set([
            (
                ConfigurationComponentKind::CombatCatalog,
                "core-combat",
                self.core.combat_catalog().digest().bytes(),
            ),
            (
                ConfigurationComponentKind::BuildCatalog,
                "core-build",
                self.core.build_catalog().digest().bytes(),
            ),
            (
                ConfigurationComponentKind::ActivityCore,
                "graph-activity",
                identity.definition_digest().bytes(),
            ),
            (
                ConfigurationComponentKind::ModeProfile,
                "divergent-universe-entry",
                identity.config_digest().bytes(),
            ),
            (
                ConfigurationComponentKind::ModeContent,
                "divergent-universe-4.4",
                content,
            ),
            (
                ConfigurationComponentKind::ActivityHandlerRegistry,
                "divergent-universe-activity-handlers",
                digest(&[b"starclock.divergent-universe.empty-activity-handler-registry.v1"]),
            ),
            (
                ConfigurationComponentKind::CombatRuleRegistry,
                "core-combat-rules",
                digest(&[
                    b"starclock.divergent-universe.core-combat-rule-registry.v1",
                    &self.core.combat_catalog().digest().bytes(),
                ]),
            ),
            (
                ConfigurationComponentKind::EncounterOverlay,
                "divergent-universe-baseline-encounter",
                encounter,
            ),
            (
                ConfigurationComponentKind::Controller,
                "divergent-universe-baseline-controller",
                digest(&[b"starclock.divergent-universe.baseline-controller.v1"]),
            ),
        ])
    }
}

fn mapped_build(
    core: &SimulationCatalog,
    form: starclock_combat::UnitDefinitionId,
) -> Result<CombatantBuildSpec, DivergentUniverseBaselineFixtureError> {
    let character = core
        .build_catalog()
        .character(form)
        .ok_or(DivergentUniverseBaselineFixtureError::Build)?;
    let abilities = character
        .ability_levels()
        .iter()
        .map(|table| AbilityInvestment::new(table.family(), table.invested_cap()))
        .collect();
    CombatantBuildSpec::new(
        form,
        UnitLevel::new(80).ok_or(DivergentUniverseBaselineFixtureError::Build)?,
        PromotionStage::new(6).ok_or(DivergentUniverseBaselineFixtureError::Build)?,
    )
    .with_ability_levels(abilities)
    .map_err(|_| DivergentUniverseBaselineFixtureError::Build)
    .map(|build| build.with_eidolon(EidolonLevel::E0))
}

fn participant_lock(
    core: &SimulationCatalog,
    builds: &[(u32, CombatantBuildSpec)],
) -> Result<ParticipantLock, DivergentUniverseBaselineFixtureError> {
    let policy = ParticipantPolicy::new(
        1,
        4,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .ok_or(DivergentUniverseBaselineFixtureError::Participant)?;
    let entries = builds
        .iter()
        .enumerate()
        .map(|(index, (_, build))| {
            let compiled = LoadoutCompiler
                .compile(core.build_catalog(), core.combat_catalog(), build)
                .map_err(|_| DivergentUniverseBaselineFixtureError::Build)?;
            let opaque = OpaqueParticipantBuild::new(
                compiled.combatant().digest(),
                BuildDigest::new(compiled.build_digest().bytes())
                    .ok_or(DivergentUniverseBaselineFixtureError::Participant)?,
                ParticipantSourceKind::CompiledBuild,
            )
            .map_err(|_| DivergentUniverseBaselineFixtureError::Participant)?;
            ParticipantLockEntry::new(
                participant(index)?,
                0,
                u8::try_from(index)
                    .map_err(|_| DivergentUniverseBaselineFixtureError::Participant)?,
                build.form(),
                opaque,
            )
            .map_err(|_| DivergentUniverseBaselineFixtureError::Participant)
        })
        .collect::<Result<Vec<_>, _>>()?;
    ParticipantLock::seal(policy, entries)
        .map_err(|_| DivergentUniverseBaselineFixtureError::Participant)
}

fn participant(index: usize) -> Result<ParticipantId, DivergentUniverseBaselineFixtureError> {
    u32::try_from(index + 1)
        .ok()
        .and_then(ParticipantId::new)
        .ok_or(DivergentUniverseBaselineFixtureError::Participant)
}

fn component_set<const N: usize>(
    values: [(ConfigurationComponentKind, &'static str, [u8; 32]); N],
) -> Result<ConfigurationComponentSet, DivergentUniverseBaselineFixtureError> {
    let components = values
        .into_iter()
        .map(|(kind, id, digest)| {
            ConfigurationComponentIdentity::new(kind, id, ComponentDigest::new(digest))
                .map_err(|_| DivergentUniverseBaselineFixtureError::Component)
        })
        .collect::<Result<Vec<_>, _>>()?;
    ConfigurationComponentSet::new(components)
        .map_err(|_| DivergentUniverseBaselineFixtureError::Component)
}

fn digest(parts: &[&[u8]]) -> [u8; 32] {
    let mut digest = CanonicalDigestBuilder::new();
    for part in parts {
        digest.update(
            u64::try_from(part.len())
                .expect("slice length fits u64")
                .to_le_bytes(),
        );
        digest.update(part);
    }
    digest.finalize()
}

#[derive(Debug)]
pub enum DivergentUniverseBaselineFixtureError {
    CoreCatalog,
    Build,
    Participant,
    Snapshot,
    EntryIdentity,
    Encounter,
    Component,
    Entry(DivergentUniverseEntryFlowError),
    Mapping(DivergentUniverseMappingRuntimeError),
}
