use crate::{
    NUMERIC_POLICY_REVISION,
    actor::{
        model::{LifeState, PresenceState},
        store::{FormationEntry, TeamState, TimelineActorState, UnitState},
    },
    catalog::CatalogDigest,
    command::model::DecisionPoint,
    id::{
        AbilityId, EncounterId, ModifierDefinitionId, RuleBundleId, SpawnSequence, TimelineActorId,
        UnitDefinitionId, UnitId, WaveInstanceId,
    },
    numeric::domain::{ActionGauge, Hp, Speed},
    rng::RNG_ALGORITHM_REVISION,
};

use super::{
    model::BattlePhase,
    spec::{
        BattleSeed, BattleSpecDigest, CombatantSpecDigest, FormationIndex, ParticipantSource,
        TeamSide, UnitLevel,
    },
    state::BattleState,
};

/// Borrowed immutable projection of one authoritative battle state.
#[derive(Clone, Copy)]
pub struct BattleView<'a> {
    pub(crate) state: &'a BattleState,
}

impl<'a> BattleView<'a> {
    /// Returns the current top-level lifecycle phase.
    #[must_use]
    pub const fn phase(self) -> BattlePhase {
        self.state.phase
    }
    /// Returns the active decision, or `None` at a terminal boundary.
    #[must_use]
    pub const fn decision(self) -> Option<&'a DecisionPoint> {
        self.state.decision.as_ref()
    }
    /// Returns the count of accepted command commits.
    #[must_use]
    pub const fn committed_revision(self) -> u64 {
        self.state.committed_revision
    }
    /// Returns the authoritative raw RNG draw count.
    #[must_use]
    pub const fn rng_draw_count(self) -> u64 {
        self.state.rng.draw_count()
    }
    /// Returns immutable catalog/spec/seed compatibility identity.
    #[must_use]
    pub const fn identity(self) -> BattleIdentityView<'a> {
        BattleIdentityView { state: self.state }
    }
    /// Returns encounter and current wave identity.
    #[must_use]
    pub const fn encounter(self) -> EncounterView {
        EncounterView {
            definition: self.state.encounter.definition,
            wave: self.state.encounter.wave,
        }
    }
    /// Iterates active unit records in stable runtime-ID order.
    pub fn units_by_id(self) -> impl Iterator<Item = UnitView<'a>> + 'a {
        self.state
            .units
            .iter_by_id()
            .map(|state| UnitView { state })
    }
    /// Iterates occupied formation slots in canonical slot order for one side.
    pub fn formation(self, side: TeamSide) -> impl Iterator<Item = FormationView> + 'a {
        self.state.formations.on_side(side).map(FormationView::from)
    }
    /// Iterates timeline actors in stable runtime-ID order.
    pub fn timeline_actors(self) -> impl Iterator<Item = TimelineActorView<'a>> + 'a {
        self.state
            .actors
            .iter_by_id()
            .map(|state| TimelineActorView { state })
    }
    /// Returns one side's team-scoped resources.
    #[must_use]
    pub fn team(self, side: TeamSide) -> TeamView<'a> {
        TeamView {
            state: self.state.teams.get(side),
        }
    }
}

/// Immutable compatibility identity included in canonical state later.
#[derive(Clone, Copy)]
pub struct BattleIdentityView<'a> {
    state: &'a BattleState,
}

impl<'a> BattleIdentityView<'a> {
    /// Returns the immutable catalog revision.
    #[must_use]
    pub fn catalog_revision(self) -> &'a str {
        self.state.identity.catalog_revision.as_str()
    }
    /// Returns the exact catalog digest.
    #[must_use]
    pub const fn catalog_digest(self) -> CatalogDigest {
        self.state.identity.catalog_digest
    }
    /// Returns the rules revision selected by the battle spec.
    #[must_use]
    pub fn rules_revision(self) -> &'a str {
        &self.state.identity.rules_revision
    }
    /// Returns the exact battle-spec digest.
    #[must_use]
    pub const fn spec_digest(self) -> BattleSpecDigest {
        self.state.identity.spec_digest
    }
    /// Returns the authoritative numeric compatibility revision.
    #[must_use]
    pub const fn numeric_policy_revision(self) -> &'static str {
        NUMERIC_POLICY_REVISION
    }
    /// Returns the authoritative RNG/mapping compatibility revision.
    #[must_use]
    pub const fn rng_algorithm_revision(self) -> &'static str {
        RNG_ALGORITHM_REVISION
    }
    /// Returns the exact isolated battle seed.
    #[must_use]
    pub const fn seed(self) -> BattleSeed {
        self.state.identity.seed
    }
}

/// Immutable combatant projection without mutable store access.
#[derive(Clone, Copy)]
pub struct UnitView<'a> {
    state: &'a UnitState,
}

impl<'a> UnitView<'a> {
    /// Returns the stable battle-local unit identity.
    #[must_use]
    pub const fn id(self) -> UnitId {
        self.state.id
    }
    /// Returns the monotonic spawn order used by deterministic tie-breaking.
    #[must_use]
    pub const fn spawn_sequence(self) -> SpawnSequence {
        self.state.spawn
    }
    /// Returns the immutable combat-form definition.
    #[must_use]
    pub const fn form(self) -> UnitDefinitionId {
        self.state.form
    }
    /// Returns the generic player/encounter source binding.
    #[must_use]
    pub const fn source(self) -> ParticipantSource {
        self.state.source
    }
    /// Returns the formation side.
    #[must_use]
    pub const fn side(self) -> TeamSide {
        self.state.side
    }
    /// Returns the occupied formation slot.
    #[must_use]
    pub const fn formation(self) -> FormationIndex {
        self.state.formation
    }
    /// Returns the checked level.
    #[must_use]
    pub const fn level(self) -> UnitLevel {
        self.state.level
    }
    /// Returns life independently from battlefield presence.
    #[must_use]
    pub const fn life(self) -> LifeState {
        self.state.life
    }
    /// Returns battlefield presence independently from life.
    #[must_use]
    pub const fn presence(self) -> PresenceState {
        self.state.presence
    }
    /// Returns current integral HP.
    #[must_use]
    pub const fn current_hp(self) -> Hp {
        self.state.current_hp
    }
    /// Returns maximum integral HP.
    #[must_use]
    pub const fn maximum_hp(self) -> Hp {
        self.state.maximum_hp
    }
    /// Returns the canonical selected ability set.
    #[must_use]
    pub fn abilities(self) -> &'a [AbilityId] {
        &self.state.abilities
    }
    /// Returns canonical selected rule bundles.
    #[must_use]
    pub fn rule_bundles(self) -> &'a [RuleBundleId] {
        &self.state.rule_bundles
    }
    /// Returns canonical selected modifiers.
    #[must_use]
    pub fn modifiers(self) -> &'a [ModifierDefinitionId] {
        &self.state.modifiers
    }
    /// Returns the generic resolved combatant digest.
    #[must_use]
    pub const fn digest(self) -> CombatantSpecDigest {
        self.state.digest
    }
}

/// Canonical occupied formation entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormationView {
    side: TeamSide,
    index: FormationIndex,
    unit: UnitId,
}

impl FormationView {
    /// Returns the formation side.
    #[must_use]
    pub const fn side(self) -> TeamSide {
        self.side
    }
    /// Returns the authored slot.
    #[must_use]
    pub const fn index(self) -> FormationIndex {
        self.index
    }
    /// Returns the stable unit occupying this slot.
    #[must_use]
    pub const fn unit(self) -> UnitId {
        self.unit
    }
}

impl From<FormationEntry> for FormationView {
    fn from(value: FormationEntry) -> Self {
        Self {
            side: value.side,
            index: value.index,
            unit: value.unit,
        }
    }
}

/// Immutable timeline-actor projection.
#[derive(Clone, Copy)]
pub struct TimelineActorView<'a> {
    state: &'a TimelineActorState,
}

impl TimelineActorView<'_> {
    /// Returns the stable timeline identity.
    #[must_use]
    pub const fn id(self) -> TimelineActorId {
        self.state.id
    }
    /// Returns the linked unit owner.
    #[must_use]
    pub const fn owner(self) -> UnitId {
        self.state.owner
    }
    /// Returns current canonical Action Gauge.
    #[must_use]
    pub const fn action_gauge(self) -> ActionGauge {
        self.state.gauge
    }
    /// Returns entry Speed.
    #[must_use]
    pub const fn speed(self) -> Speed {
        self.state.speed
    }
}

/// Immutable team-resource projection.
#[derive(Clone, Copy)]
pub struct TeamView<'a> {
    state: &'a TeamState,
}

impl TeamView<'_> {
    /// Returns the team side.
    #[must_use]
    pub const fn side(self) -> TeamSide {
        self.state.side
    }
    /// Returns current Skill Points.
    #[must_use]
    pub const fn skill_points(self) -> u16 {
        self.state.skill_points
    }
    /// Returns the team Skill Point cap.
    #[must_use]
    pub const fn maximum_skill_points(self) -> u16 {
        self.state.maximum_skill_points
    }
}

/// Immutable encounter progress projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncounterView {
    definition: EncounterId,
    wave: WaveInstanceId,
}

impl EncounterView {
    /// Returns the selected encounter definition.
    #[must_use]
    pub const fn definition(self) -> EncounterId {
        self.definition
    }
    /// Returns the stable current wave instance.
    #[must_use]
    pub const fn wave(self) -> WaveInstanceId {
        self.wave
    }
}
