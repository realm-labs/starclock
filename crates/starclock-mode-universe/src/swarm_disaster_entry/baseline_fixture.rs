//! Fixed synthetic roster fixture for deterministic external-surface runs.

use starclock_activity::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, BuildDigest, LoadoutLockScope, OpaqueParticipantBuild,
    ParticipantId, ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope,
};
use starclock_combat::{
    CombatantSpecDigest, Energy, Hp, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    StatValue, UnitDefinitionId, UnitLevel, catalog::action::AbilityKind,
};
use starclock_replay::component::ConfigurationComponentSet;

use crate::{
    battle_materialization::UniverseBattleRoster, digest::Encoder,
    swarm_disaster_components::swarm_disaster_component_set,
};

use super::{
    SwarmDisasterControllerIdentity, SwarmDisasterEntry, SwarmDisasterRuntimeFactory,
    SwarmDisasterRuntimeInstance,
};

/// Version of the deterministic CLI/agent/MCP fixture.
pub const SWARM_DISASTER_BASELINE_FIXTURE_REVISION: &str =
    "swarm-disaster-synthetic-baseline-fixture-v1";
/// Accuracy label that prevents the synthetic high-stat roster from claiming observed parity.
pub const SWARM_DISASTER_BASELINE_FIXTURE_ACCURACY: &str =
    "SyntheticBalanceIndependentNotObservedNumericParity";
/// Build-catalog revision bound into fixture replays.
pub const SWARM_DISASTER_BASELINE_BUILD_CATALOG_REVISION: &str =
    "swarm-disaster-synthetic-baseline-build-catalog-v1";
/// Profile identity emitted by headless Swarm clients.
pub const SWARM_DISASTER_BASELINE_PROFILE: &str = "swarm-disaster.profile.v1";
/// Real nested-battle executor bound into headless diagnostics and replay actions.
pub const SWARM_DISASTER_BASELINE_BATTLE_EXECUTION_REVISION: &str =
    super::battle_execution::SWARM_DISASTER_BATTLE_EXECUTION_REVISION;

const AREA: &str = "swarm-disaster.area.201";
const PATH: &str = "universe.path.preservation";
const AUDIENCE_DIE: &str = "swarm-disaster.audience-die.1";
const BUNDLE: &[u8] = include_bytes!("../../../../config/swarm-disaster-generated/config.sora");

/// Complete immutable inputs used by headless Swarm baseline clients.
#[derive(Debug)]
pub struct SwarmDisasterBaselineFixture {
    instance: SwarmDisasterRuntimeInstance,
    roster: UniverseBattleRoster,
    activity_identity: ActivityDefinitionIdentity,
    components: ConfigurationComponentSet,
}

impl SwarmDisasterBaselineFixture {
    /// Compiled immutable Swarm runtime.
    #[must_use]
    pub const fn instance(&self) -> &SwarmDisasterRuntimeInstance {
        &self.instance
    }

    /// Synthetic balance-independent participant roster.
    #[must_use]
    pub const fn roster(&self) -> &UniverseBattleRoster {
        &self.roster
    }

    /// Activity definition/config identity bound into replay.
    #[must_use]
    pub const fn activity_identity(&self) -> ActivityDefinitionIdentity {
        self.activity_identity
    }

    /// Exact ten-component baseline set.
    #[must_use]
    pub const fn components(&self) -> &ConfigurationComponentSet {
        &self.components
    }

    /// Re-composes the exact fixture inputs for a caller-owned controller.
    pub fn components_for_controller(
        &self,
        controller: SwarmDisasterControllerIdentity<'_>,
    ) -> Result<ConfigurationComponentSet, SwarmDisasterBaselineFixtureError> {
        let combat = self.instance.battle_catalog.combat();
        swarm_disaster_component_set(
            BUNDLE,
            combat.digest().bytes(),
            build_catalog_digest(&self.roster),
            self.activity_identity.definition_digest().bytes(),
            self.instance.battle_catalog.digest(),
            self.instance.graph_definition().digest().bytes(),
            (controller.id, controller.digest),
        )
        .map_err(|_| SwarmDisasterBaselineFixtureError::Component)
    }

    /// Fixed Formal difficulty-one area.
    #[must_use]
    pub const fn area(&self) -> &'static str {
        AREA
    }

    /// Fixed released Path.
    #[must_use]
    pub const fn path(&self) -> &'static str {
        PATH
    }

    /// Fixed matching Audience Die.
    #[must_use]
    pub const fn audience_die(&self) -> &'static str {
        AUDIENCE_DIE
    }
}

impl SwarmDisasterRuntimeFactory {
    /// Compiles the fixed P6-approved high-stat roster fixture for headless clients.
    pub fn compile_synthetic_baseline_fixture(
        &self,
    ) -> Result<SwarmDisasterBaselineFixture, SwarmDisasterBaselineFixtureError> {
        let participants = synthetic_participant_lock()?;
        let progression = self
            .unique
            .trail_runtime_input()
            .nodes
            .iter()
            .map(|node| node.key.to_string())
            .chain(
                self.unique
                    .communing_runtime_input()
                    .cabinets
                    .iter()
                    .map(|cabinet| cabinet.key.to_string()),
            )
            .chain(
                self.unique
                    .path_runtime_input()
                    .interplays
                    .iter()
                    .map(|interplay| interplay.key.to_string()),
            )
            .collect();
        let communing = (1..=7)
            .map(|id| (format!("swarm-disaster.communing-dimension.{id}"), 20))
            .collect();
        let entry = SwarmDisasterEntry::new(AREA, PATH, AUDIENCE_DIE, participants)
            .with_audience_unlocks(
                [
                    "1000008", "1000013", "1000014", "1000015", "1000016", "1000017", "1000018",
                ]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            )
            .with_dice_control_unlocks(vec!["1000022".into()])
            .with_progression(communing, progression, None);
        let instance = self
            .compile_entry(entry)
            .map_err(|_| SwarmDisasterBaselineFixtureError::Entry)?;
        let roster = synthetic_roster(&instance)?;
        let activity_identity = activity_identity(&instance);
        let combat = instance.battle_catalog.combat();
        let controller = SwarmDisasterControllerIdentity::baseline();
        let components = swarm_disaster_component_set(
            BUNDLE,
            combat.digest().bytes(),
            build_catalog_digest(&roster),
            activity_identity.definition_digest().bytes(),
            instance.battle_catalog.digest(),
            instance.graph_definition().digest().bytes(),
            (controller.id, controller.digest),
        )
        .map_err(|_| SwarmDisasterBaselineFixtureError::Component)?;
        Ok(SwarmDisasterBaselineFixture {
            instance,
            roster,
            activity_identity,
            components,
        })
    }
}

fn synthetic_participant_lock() -> Result<ParticipantLock, SwarmDisasterBaselineFixtureError> {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .ok_or(SwarmDisasterBaselineFixtureError::Participant)?;
    let entries = (0_u8..4)
        .map(|index| {
            let byte = index + 1;
            let build = OpaqueParticipantBuild::new(
                CombatantSpecDigest::new([byte; 32])
                    .ok_or(SwarmDisasterBaselineFixtureError::Participant)?,
                BuildDigest::new([byte + 32; 32])
                    .ok_or(SwarmDisasterBaselineFixtureError::Participant)?,
                SWARM_DISASTER_BASELINE_BUILD_CATALOG_REVISION,
                ParticipantSourceKind::CompiledBuild,
            )
            .map_err(|_| SwarmDisasterBaselineFixtureError::Participant)?;
            ParticipantLockEntry::new(
                ParticipantId::new(u32::from(index) + 1)
                    .ok_or(SwarmDisasterBaselineFixtureError::Participant)?,
                0,
                index,
                UnitDefinitionId::new(u32::from(index) + 1)
                    .ok_or(SwarmDisasterBaselineFixtureError::Participant)?,
                build,
            )
            .map_err(|_| SwarmDisasterBaselineFixtureError::Participant)
        })
        .collect::<Result<Vec<_>, _>>()?;
    ParticipantLock::seal(policy, entries)
        .map_err(|_| SwarmDisasterBaselineFixtureError::Participant)
}

fn synthetic_roster(
    instance: &SwarmDisasterRuntimeInstance,
) -> Result<UniverseBattleRoster, SwarmDisasterBaselineFixtureError> {
    let combat = instance
        .content_runtime
        .standard
        .simulation_catalog()
        .combat_catalog();
    let combatants = instance
        .participants()
        .entries()
        .iter()
        .map(|locked| {
            let unit = combat
                .unit(locked.character())
                .ok_or(SwarmDisasterBaselineFixtureError::Combatant)?;
            let basic = unit
                .abilities()
                .iter()
                .copied()
                .find(|ability| {
                    combat
                        .ability(*ability)
                        .and_then(|definition| definition.action())
                        .is_some_and(|action| action.kind() == AbilityKind::Basic)
                })
                .ok_or(SwarmDisasterBaselineFixtureError::Combatant)?;
            let spec = ResolvedCombatantSpec::new(
                locked.character(),
                UnitLevel::new(80).ok_or(SwarmDisasterBaselineFixtureError::Combatant)?,
                Hp::new(1_000_000_000).map_err(|_| SwarmDisasterBaselineFixtureError::Combatant)?,
                Speed::from_scaled(1_000_000_000)
                    .map_err(|_| SwarmDisasterBaselineFixtureError::Combatant)?,
                ResolvedDefinitionBindings::new(vec![basic], Vec::new(), Vec::new())
                    .map_err(|_| SwarmDisasterBaselineFixtureError::Combatant)?,
                CombatantSpecDigest::new(locked.build().resolved_spec_digest().bytes())
                    .ok_or(SwarmDisasterBaselineFixtureError::Combatant)?,
            )
            .map_err(|_| SwarmDisasterBaselineFixtureError::Combatant)?
            .with_base_attack_defense(
                StatValue::from_scaled(1_000_000_000_000)
                    .map_err(|_| SwarmDisasterBaselineFixtureError::Combatant)?,
                StatValue::from_scaled(1_000_000_000_000)
                    .map_err(|_| SwarmDisasterBaselineFixtureError::Combatant)?,
            )
            .with_energy(
                Energy::ZERO,
                Energy::from_scaled(100_000_000)
                    .map_err(|_| SwarmDisasterBaselineFixtureError::Combatant)?,
            )
            .map_err(|_| SwarmDisasterBaselineFixtureError::Combatant)?;
            Ok((locked.participant(), spec))
        })
        .collect::<Result<Vec<_>, _>>()?;
    UniverseBattleRoster::new(instance.participants(), combatants)
        .map_err(|_| SwarmDisasterBaselineFixtureError::Roster)
}

fn activity_identity(instance: &SwarmDisasterRuntimeInstance) -> ActivityDefinitionIdentity {
    let mut definition = Encoder::new(b"starclock.swarm-disaster.baseline-activity-definition.v1");
    definition.text(SWARM_DISASTER_BASELINE_FIXTURE_REVISION);
    definition.digest(instance.graph_definition().digest().bytes());
    definition.digest(instance.participants().digest().bytes());
    definition.digest(instance.battle_catalog.digest());
    let mut config = Encoder::new(b"starclock.swarm-disaster.baseline-entry-spec.v1");
    config.text(AREA);
    config.text(PATH);
    config.text(AUDIENCE_DIE);
    for face in instance.audience_die_faces() {
        config.text(face);
    }
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(20).expect("the Swarm profile identity is non-zero"),
        ActivityDefinitionDigest::new(definition.finish())
            .expect("SHA-256 is a valid Activity definition digest"),
        ActivityConfigDigest::new(config.finish())
            .expect("SHA-256 is a valid Activity configuration digest"),
    )
}

fn build_catalog_digest(roster: &UniverseBattleRoster) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.synthetic-build-catalog.v1");
    encoder.text(SWARM_DISASTER_BASELINE_BUILD_CATALOG_REVISION);
    encoder.u32(u32::try_from(roster.entries().len()).expect("the roster is bounded"));
    for entry in roster.entries() {
        encoder.u32(entry.participant().get());
        encoder.digest(entry.build_digest().bytes());
        encoder.digest(entry.combatant().digest().bytes());
    }
    encoder.finish()
}

/// Failures while compiling the fixed headless fixture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SwarmDisasterBaselineFixtureError {
    /// Entry compilation rejected the frozen fixture selections.
    Entry,
    /// Participant identity or policy construction failed.
    Participant,
    /// A synthetic combatant could not bind its released basic attack.
    Combatant,
    /// The roster did not match the locked participant set.
    Roster,
    /// The exact ten-component set could not be composed.
    Component,
}
