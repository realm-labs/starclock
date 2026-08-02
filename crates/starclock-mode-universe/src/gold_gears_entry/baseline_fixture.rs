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
use starclock_replay::{
    component::{
        ConfigurationComponentIdentity, ConfigurationComponentKind, ConfigurationComponentSet,
    },
    digest::ComponentDigest,
};

use crate::{
    battle_materialization::UniverseBattleRoster, digest::Encoder,
    gold_gears_components::gold_and_gears_component_set,
    gold_gears_identity::GoldAndGearsCatalogIdentity,
};

use super::{
    GoldAndGearsEntry, GoldAndGearsEntryError, GoldAndGearsRuntimeFactory,
    GoldAndGearsRuntimeInstance,
    baseline_controller::{GoldAndGearsBaselineController, GoldAndGearsControllerIdentity},
};

pub const GOLD_AND_GEARS_BASELINE_FIXTURE_REVISION: &str =
    "gold-and-gears-synthetic-baseline-fixture-v1";
pub const GOLD_AND_GEARS_BASELINE_FIXTURE_ACCURACY: &str =
    "SyntheticBalanceIndependentNotObservedNumericParity";
pub const GOLD_AND_GEARS_BASELINE_BUILD_CATALOG_REVISION: &str =
    "gold-and-gears-synthetic-baseline-build-catalog-v1";

const AREA: &str = "gold-gears.area.401";
const PATH: &str = "universe.path.abundance";
const DICE: &str = "gold-gears.custom-dice.101";

/// Complete immutable inputs used by headless baseline clients.
#[derive(Debug)]
pub struct GoldAndGearsBaselineFixture {
    instance: GoldAndGearsRuntimeInstance,
    roster: UniverseBattleRoster,
    activity_identity: ActivityDefinitionIdentity,
    components: ConfigurationComponentSet,
}

impl GoldAndGearsBaselineFixture {
    #[must_use]
    pub const fn instance(&self) -> &GoldAndGearsRuntimeInstance {
        &self.instance
    }

    #[must_use]
    pub const fn roster(&self) -> &UniverseBattleRoster {
        &self.roster
    }

    #[must_use]
    pub const fn activity_identity(&self) -> ActivityDefinitionIdentity {
        self.activity_identity
    }

    #[must_use]
    pub const fn components(&self) -> &ConfigurationComponentSet {
        &self.components
    }

    /// Rebinds only the caller-owned controller component while preserving
    /// every catalog, registry, profile and encounter identity.
    pub fn components_for_controller(
        &self,
        controller: GoldAndGearsControllerIdentity<'_>,
    ) -> Result<ConfigurationComponentSet, GoldAndGearsBaselineFixtureError> {
        let mut components = self
            .components
            .components()
            .iter()
            .filter(|component| component.kind() != ConfigurationComponentKind::Controller)
            .cloned()
            .collect::<Vec<_>>();
        components.push(
            ConfigurationComponentIdentity::new(
                ConfigurationComponentKind::Controller,
                controller.id,
                ComponentDigest::new(controller.digest),
            )
            .map_err(|_| GoldAndGearsBaselineFixtureError::Component)?,
        );
        ConfigurationComponentSet::new(components)
            .map_err(|_| GoldAndGearsBaselineFixtureError::Component)
    }

    #[must_use]
    pub const fn area(&self) -> &'static str {
        AREA
    }

    #[must_use]
    pub const fn path(&self) -> &'static str {
        PATH
    }

    #[must_use]
    pub const fn custom_dice(&self) -> &'static str {
        DICE
    }
}

impl GoldAndGearsRuntimeFactory {
    /// Compiles the fixed P6-approved high-stat roster fixture for headless
    /// orchestration tests. Its accuracy label explicitly disclaims balance
    /// or observed numeric parity.
    pub fn compile_synthetic_baseline_fixture(
        &self,
        identity: &GoldAndGearsCatalogIdentity,
    ) -> Result<GoldAndGearsBaselineFixture, GoldAndGearsBaselineFixtureError> {
        let dice = self
            .unique
            .dice
            .iter()
            .find(|dice| dice.identity.stable_key.as_ref() == DICE)
            .ok_or(GoldAndGearsBaselineFixtureError::Catalog)?;
        let faces = dice
            .default_face_sources
            .iter()
            .map(|source| {
                self.unique
                    .dice_faces
                    .iter()
                    .find(|face| face.identity.source_id == *source)
                    .map(|face| face.identity.stable_key.to_string())
                    .ok_or(GoldAndGearsBaselineFixtureError::Catalog)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let unlocked_dice = self
            .unique
            .dice
            .iter()
            .map(|dice| dice.identity.stable_key.to_string())
            .collect();
        let participants = synthetic_participant_lock()?;
        let instance = self
            .compile_entry(
                GoldAndGearsEntry::new(AREA, PATH, DICE, faces, participants)
                    .with_unlocked_dice(unlocked_dice),
            )
            .map_err(GoldAndGearsBaselineFixtureError::Entry)?;
        let roster = synthetic_roster(&instance)?;
        let activity_identity = activity_identity(&instance);
        let combat = instance.battle_catalog.combat();
        let components = gold_and_gears_component_set(
            identity,
            combat.digest().bytes(),
            build_catalog_digest(&roster),
            activity_identity.definition_digest().bytes(),
            instance.battle_catalog.digest(),
            instance.graph_definition().digest().bytes(),
            (
                "gold-and-gears-baseline-controller",
                GoldAndGearsBaselineController::identity_digest(),
            ),
        )
        .map_err(|_| GoldAndGearsBaselineFixtureError::Component)?;
        Ok(GoldAndGearsBaselineFixture {
            instance,
            roster,
            activity_identity,
            components,
        })
    }
}

fn synthetic_participant_lock() -> Result<ParticipantLock, GoldAndGearsBaselineFixtureError> {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .ok_or(GoldAndGearsBaselineFixtureError::Participant)?;
    let entries = (0_u8..4)
        .map(|index| {
            let byte = index + 1;
            let build = OpaqueParticipantBuild::new(
                CombatantSpecDigest::new([byte; 32])
                    .ok_or(GoldAndGearsBaselineFixtureError::Participant)?,
                BuildDigest::new([byte + 32; 32])
                    .ok_or(GoldAndGearsBaselineFixtureError::Participant)?,
                GOLD_AND_GEARS_BASELINE_BUILD_CATALOG_REVISION,
                ParticipantSourceKind::CompiledBuild,
            )
            .map_err(|_| GoldAndGearsBaselineFixtureError::Participant)?;
            ParticipantLockEntry::new(
                ParticipantId::new(u32::from(index) + 1)
                    .ok_or(GoldAndGearsBaselineFixtureError::Participant)?,
                0,
                index,
                UnitDefinitionId::new(u32::from(index) + 1)
                    .ok_or(GoldAndGearsBaselineFixtureError::Participant)?,
                build,
            )
            .map_err(|_| GoldAndGearsBaselineFixtureError::Participant)
        })
        .collect::<Result<Vec<_>, _>>()?;
    ParticipantLock::seal(policy, entries)
        .map_err(|_| GoldAndGearsBaselineFixtureError::Participant)
}

fn synthetic_roster(
    instance: &GoldAndGearsRuntimeInstance,
) -> Result<UniverseBattleRoster, GoldAndGearsBaselineFixtureError> {
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
                .ok_or(GoldAndGearsBaselineFixtureError::Combatant)?;
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
                .ok_or(GoldAndGearsBaselineFixtureError::Combatant)?;
            let spec = ResolvedCombatantSpec::new(
                locked.character(),
                UnitLevel::new(80).ok_or(GoldAndGearsBaselineFixtureError::Combatant)?,
                Hp::new(1_000_000_000).map_err(|_| GoldAndGearsBaselineFixtureError::Combatant)?,
                Speed::from_scaled(1_000_000_000)
                    .map_err(|_| GoldAndGearsBaselineFixtureError::Combatant)?,
                ResolvedDefinitionBindings::new(vec![basic], Vec::new(), Vec::new())
                    .map_err(|_| GoldAndGearsBaselineFixtureError::Combatant)?,
                CombatantSpecDigest::new(locked.build().resolved_spec_digest().bytes())
                    .ok_or(GoldAndGearsBaselineFixtureError::Combatant)?,
            )
            .map_err(|_| GoldAndGearsBaselineFixtureError::Combatant)?
            .with_base_attack_defense(
                StatValue::from_scaled(1_000_000_000_000)
                    .map_err(|_| GoldAndGearsBaselineFixtureError::Combatant)?,
                StatValue::from_scaled(1_000_000_000_000)
                    .map_err(|_| GoldAndGearsBaselineFixtureError::Combatant)?,
            )
            .with_energy(
                Energy::ZERO,
                Energy::from_scaled(100_000_000)
                    .map_err(|_| GoldAndGearsBaselineFixtureError::Combatant)?,
            )
            .map_err(|_| GoldAndGearsBaselineFixtureError::Combatant)?;
            Ok((locked.participant(), spec))
        })
        .collect::<Result<Vec<_>, _>>()?;
    UniverseBattleRoster::new(instance.participants(), combatants)
        .map_err(|_| GoldAndGearsBaselineFixtureError::Roster)
}

fn activity_identity(instance: &GoldAndGearsRuntimeInstance) -> ActivityDefinitionIdentity {
    let mut definition = Encoder::new(b"starclock.gold-and-gears.baseline-activity-definition.v1");
    definition.text(GOLD_AND_GEARS_BASELINE_FIXTURE_REVISION);
    definition.digest(instance.graph_definition().digest().bytes());
    definition.digest(instance.participants().digest().bytes());
    definition.digest(instance.battle_catalog.digest());
    let mut config = Encoder::new(b"starclock.gold-and-gears.baseline-entry-spec.v1");
    config.text(AREA);
    config.text(PATH);
    config.text(DICE);
    for face in instance.dice_faces() {
        config.text(face);
    }
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(14).expect("the Gold profile identity is non-zero"),
        ActivityDefinitionDigest::new(definition.finish())
            .expect("SHA-256 is a valid Activity definition digest"),
        ActivityConfigDigest::new(config.finish())
            .expect("SHA-256 is a valid Activity configuration digest"),
    )
}

fn build_catalog_digest(roster: &UniverseBattleRoster) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-and-gears.synthetic-build-catalog.v1");
    encoder.text(GOLD_AND_GEARS_BASELINE_BUILD_CATALOG_REVISION);
    encoder.u32(u32::try_from(roster.entries().len()).expect("the roster is bounded"));
    for entry in roster.entries() {
        encoder.u32(entry.participant().get());
        encoder.digest(entry.build_digest().bytes());
        encoder.digest(entry.combatant().digest().bytes());
    }
    encoder.finish()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoldAndGearsBaselineFixtureError {
    Catalog,
    Entry(GoldAndGearsEntryError),
    Participant,
    Combatant,
    Roster,
    Component,
}
