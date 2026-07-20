use crate::{
    NUMERIC_POLICY_REVISION,
    actor::{
        model::{LifeState, PresenceState},
        store::{FormationEntry, LinkState, TeamState, TimelineActorState, UnitState},
    },
    catalog::CatalogDigest,
    command::model::DecisionPoint,
    id::{
        AbilityId, EncounterId, ModifierDefinitionId, RuleBundleId, ShieldInstanceId,
        SpawnSequence, TimelineActorId, UnitDefinitionId, UnitId, WaveInstanceId,
    },
    numeric::domain::{ActionGauge, Hp, ShieldAmount, Speed},
    rng::RNG_ALGORITHM_REVISION,
};

use super::{
    fault::BattleFault,
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
    /// Returns the persisted terminal fault, if resolution faulted.
    #[must_use]
    pub const fn fault(self) -> Option<BattleFault> {
        self.state.fault
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
            number: self.state.encounter.number,
            total_waves: self.state.encounter.total_waves,
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
    /// Iterates explicit owner/entity links in canonical insertion order.
    pub fn links(self) -> impl Iterator<Item = LinkView<'a>> + 'a {
        self.state
            .links
            .canonical_entries()
            .iter()
            .map(|state| LinkView { state })
    }
    /// Iterates retained shield instances in stable runtime-ID order.
    pub fn shields_by_id(self) -> impl Iterator<Item = ShieldView<'a>> + 'a {
        self.state
            .shields
            .iter_by_id()
            .map(|state| ShieldView { state })
    }
    /// Iterates retained base Break effects in stable instance order.
    pub fn break_effects_by_id(self) -> impl Iterator<Item = BreakEffectView<'a>> + 'a {
        self.state
            .break_effects
            .iter_by_id()
            .map(|state| BreakEffectView { state })
    }
    /// Iterates retained generic effect instances in stable instance order.
    pub fn effects_by_id(self) -> impl Iterator<Item = EffectView<'a>> + 'a {
        self.state
            .effects
            .iter_by_id()
            .map(|state| EffectView { state })
    }
    /// Returns the effective strongest-wins instance using the canonical comparator.
    #[must_use]
    pub fn strongest_effect(
        self,
        definition: crate::EffectDefinitionId,
        target: UnitId,
    ) -> Option<crate::EffectInstanceId> {
        self.state.effects.active_strongest(definition, target)
    }
    /// Iterates battle-bound rule instances in stable runtime order.
    pub fn rule_instances_by_id(self) -> impl Iterator<Item = RuleInstanceView<'a>> + 'a {
        self.state
            .rules
            .iter_by_id()
            .map(|state| RuleInstanceView { state })
    }
    /// Iterates battle-owned modifier instances in stable runtime-ID order.
    pub fn modifier_instances_by_id(self) -> impl Iterator<Item = ModifierInstanceView<'a>> + 'a {
        self.state
            .modifiers
            .iter_by_id()
            .map(|state| ModifierInstanceView { state })
    }
    /// Returns one side's team-scoped resources.
    #[must_use]
    pub fn team(self, side: TeamSide) -> TeamView<'a> {
        TeamView {
            state: self.state.teams.get(side),
        }
    }
    /// Returns the selected normal turn that persists across its decisions.
    #[must_use]
    pub fn active_turn(self) -> Option<ActiveTurnView> {
        self.state.timeline.active_turn.map(ActiveTurnView::from)
    }
    /// Returns the active external interrupt boundary, if any.
    #[must_use]
    pub fn interrupt_window(self) -> Option<InterruptWindowView> {
        self.state
            .timeline
            .interrupt
            .as_ref()
            .map(|window| InterruptWindowView {
                kind: window.kind,
                turn: ActiveTurnView::from(window.turn),
                pending_count: u64::try_from(window.pending.entries().len())
                    .expect("interrupt queue length is bounded below u64::MAX"),
            })
    }
}

/// Immutable projection of one separately retained shield instance.
#[derive(Clone, Copy)]
pub struct ShieldView<'a> {
    state: &'a crate::effect::shield::ShieldState,
}

/// Immutable projection of one retained base Break effect.
#[derive(Clone, Copy)]
pub struct BreakEffectView<'a> {
    state: &'a crate::effect::break_effect::BreakEffectState,
}

/// Immutable projection of one retained generic effect instance.
#[derive(Clone, Copy)]
pub struct EffectView<'a> {
    state: &'a crate::effect::state::EffectState,
}

/// Immutable projection of one battle-owned modifier instance.
#[derive(Clone, Copy)]
pub struct ModifierInstanceView<'a> {
    state: &'a crate::modifier::model::ActiveModifier,
}

impl ModifierInstanceView<'_> {
    #[must_use]
    pub const fn id(self) -> crate::ModifierInstanceId {
        self.state.instance
    }
    #[must_use]
    pub const fn definition(self) -> crate::ModifierDefinitionId {
        self.state.definition
    }
    #[must_use]
    pub const fn owner(self) -> UnitId {
        self.state.owner
    }
    #[must_use]
    pub const fn subject(self) -> UnitId {
        self.state.subject
    }
    #[must_use]
    pub const fn source(self) -> crate::SourceDefinitionId {
        self.state.source
    }
    #[must_use]
    pub const fn source_class(self) -> crate::rule::model::SourceClass {
        self.state.source_class
    }
}

impl EffectView<'_> {
    #[must_use]
    pub const fn id(self) -> crate::EffectInstanceId {
        self.state.id
    }
    #[must_use]
    pub const fn definition(self) -> crate::EffectDefinitionId {
        self.state.definition
    }
    #[must_use]
    pub const fn source_definition(self) -> crate::SourceDefinitionId {
        self.state.source_definition
    }
    #[must_use]
    pub const fn applier(self) -> UnitId {
        self.state.applier
    }
    #[must_use]
    pub const fn target(self) -> UnitId {
        self.state.target
    }
    #[must_use]
    pub const fn category(self) -> crate::EffectCategory {
        self.state.category
    }
    #[must_use]
    pub const fn stacks(self) -> u16 {
        self.state.stacks
    }
    #[must_use]
    pub const fn remaining(self) -> Option<u16> {
        self.state.remaining
    }
    #[must_use]
    pub const fn duration_clock(self) -> crate::DurationClock {
        self.state.duration_clock
    }
    #[must_use]
    pub const fn snapshot_policy(self) -> crate::EffectSnapshotPolicy {
        self.state.snapshot_policy
    }
}

#[derive(Clone, Copy)]
pub struct RuleInstanceView<'a> {
    state: &'a crate::rule::state::RuleInstanceState,
}

impl<'a> RuleInstanceView<'a> {
    #[must_use]
    pub const fn id(self) -> crate::RuleInstanceId {
        self.state.id
    }
    #[must_use]
    pub const fn rule(self) -> crate::RuleId {
        self.state.rule
    }
    #[must_use]
    pub const fn owner(self) -> Option<UnitId> {
        self.state.owner
    }
    pub fn slots(
        self,
    ) -> impl Iterator<
        Item = (
            crate::StateSlotDefinitionId,
            &'a crate::rule::model::RuleValue,
        ),
    > + 'a {
        self.state
            .slots
            .iter()
            .map(|(definition, value)| (definition.id(), value))
    }
}

impl BreakEffectView<'_> {
    #[must_use]
    pub const fn id(self) -> crate::EffectInstanceId {
        self.state.id
    }
    #[must_use]
    pub const fn owner(self) -> UnitId {
        self.state.owner
    }
    #[must_use]
    pub const fn applier(self) -> UnitId {
        self.state.applier
    }
    #[must_use]
    pub const fn source_definition(self) -> crate::SourceDefinitionId {
        self.state.source_definition
    }
    #[must_use]
    pub const fn element(self) -> crate::formula::model::CombatElement {
        self.state.plan.element
    }
    #[must_use]
    pub const fn remaining_turns(self) -> u8 {
        self.state.remaining_turns
    }
    #[must_use]
    pub const fn stacks(self) -> u8 {
        self.state.stacks
    }
}

impl ShieldView<'_> {
    #[must_use]
    pub const fn id(self) -> ShieldInstanceId {
        self.state.id
    }

    #[must_use]
    pub const fn owner(self) -> UnitId {
        self.state.owner
    }

    #[must_use]
    pub const fn remaining(self) -> ShieldAmount {
        self.state.remaining
    }

    #[must_use]
    pub const fn policy(self) -> crate::formula::shield::ShieldAbsorptionPolicy {
        self.state.policy
    }
}

/// Immutable selected normal-turn ownership.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveTurnView {
    actor: TimelineActorId,
    owner: UnitId,
    side: TeamSide,
    formation: FormationIndex,
    spawn: SpawnSequence,
}

impl ActiveTurnView {
    /// Returns the timeline actor whose gauge reached the boundary.
    #[must_use]
    pub const fn actor(self) -> TimelineActorId {
        self.actor
    }
    /// Returns the unit that owns the selected timeline actor.
    #[must_use]
    pub const fn owner(self) -> UnitId {
        self.owner
    }
    /// Returns the formation side that owns the selected turn.
    #[must_use]
    pub const fn side(self) -> TeamSide {
        self.side
    }
    /// Returns the selected owner's formation position.
    #[must_use]
    pub const fn formation(self) -> FormationIndex {
        self.formation
    }
    /// Returns the stable spawn-order tie breaker.
    #[must_use]
    pub const fn spawn_sequence(self) -> SpawnSequence {
        self.spawn
    }
}

impl From<crate::timeline::state::NormalTurnState> for ActiveTurnView {
    fn from(turn: crate::timeline::state::NormalTurnState) -> Self {
        Self {
            actor: turn.actor,
            owner: turn.owner,
            side: turn.side,
            formation: turn.formation,
            spawn: turn.spawn,
        }
    }
}

/// Immutable interrupt-window state; pending entries remain resolver-private.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterruptWindowView {
    kind: crate::timeline::state::InterruptWindowKind,
    turn: ActiveTurnView,
    pending_count: u64,
}

impl InterruptWindowView {
    /// Returns the stable interrupt boundary kind.
    #[must_use]
    pub const fn kind(self) -> crate::timeline::state::InterruptWindowKind {
        self.kind
    }
    /// Returns the normal turn suspended at this interrupt boundary.
    #[must_use]
    pub const fn turn(self) -> ActiveTurnView {
        self.turn
    }
    /// Returns queued automatic/out-of-order actions without exposing entries.
    #[must_use]
    pub const fn pending_count(self) -> u64 {
        self.pending_count
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
    /// Returns the canonical battle-state hash compatibility revision.
    #[must_use]
    pub const fn state_hash_revision(self) -> &'static str {
        crate::STATE_HASH_REVISION
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
    /// Returns the one-based encounter wave in which this unit enters.
    #[must_use]
    pub const fn entry_wave(self) -> u16 {
        self.state.entry_wave
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
    /// Returns the immutable authored base ATK retained for staged queries.
    #[must_use]
    pub const fn base_attack(self) -> crate::StatValue {
        self.state.base_attack
    }
    /// Returns the immutable authored base DEF retained for staged queries.
    #[must_use]
    pub const fn base_defense(self) -> crate::StatValue {
        self.state.base_defense
    }
    /// Returns the immutable authored base SPD retained for staged queries.
    #[must_use]
    pub const fn base_speed(self) -> crate::Speed {
        self.state.base_speed
    }
    /// Returns current personal Energy.
    #[must_use]
    pub const fn current_energy(self) -> crate::Energy {
        self.state.current_energy
    }
    /// Returns maximum personal Energy.
    #[must_use]
    pub const fn maximum_energy(self) -> crate::Energy {
        self.state.maximum_energy
    }
    /// Returns the canonical selected ability set.
    #[must_use]
    pub fn abilities(self) -> &'a [AbilityId] {
        &self.state.abilities
    }
    /// Returns whether the unit currently retains an authored transformation.
    #[must_use]
    pub const fn is_transformed(self) -> bool {
        self.state.transformation.is_some()
    }
    /// Returns the transform-owned countdown actor when present.
    #[must_use]
    pub const fn transformation_countdown(self) -> Option<TimelineActorId> {
        match &self.state.transformation {
            None => None,
            Some(state) => state.countdown_actor,
        }
    }
    /// Returns the bound enemy definition for authored hostile occurrences.
    #[must_use]
    pub const fn enemy_definition(self) -> Option<crate::EnemyDefinitionId> {
        match self.state.enemy {
            None => None,
            Some(enemy) => Some(enemy.definition),
        }
    }
    /// Returns the authoritative AI graph/state cursor for an executable enemy.
    #[must_use]
    pub const fn enemy_ai_state(self) -> Option<(crate::AiGraphId, crate::AiStateId, u16)> {
        match self.state.enemy {
            None => None,
            Some(enemy) => Some((enemy.graph, enemy.state, enemy.turn_counter)),
        }
    }
    /// Returns the current authored boss phase, when one is active.
    #[must_use]
    pub const fn enemy_phase(self) -> Option<crate::EnemyPhaseId> {
        match self.state.enemy {
            None => None,
            Some(enemy) => enemy.phase,
        }
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
    /// Returns active elemental weaknesses in canonical element order.
    #[must_use]
    pub fn weaknesses(self) -> &'a [crate::formula::model::CombatElement] {
        &self.state.weaknesses
    }
    /// Returns whether a layer has placed this unit in the global broken state.
    #[must_use]
    pub const fn weakness_broken(self) -> bool {
        self.state.weakness_broken
    }
    /// Iterates Toughness layers in authored routing order.
    pub fn toughness_layers(self) -> impl Iterator<Item = ToughnessLayerView<'a>> + 'a {
        self.state
            .toughness_layers
            .iter()
            .map(|state| ToughnessLayerView { state })
    }
    /// Returns the generic resolved combatant digest.
    #[must_use]
    pub const fn digest(self) -> CombatantSpecDigest {
        self.state.digest
    }
}

/// Immutable ordered Toughness-layer projection.
#[derive(Clone, Copy)]
pub struct ToughnessLayerView<'a> {
    state: &'a crate::toughness::state::ToughnessLayerState,
}

impl ToughnessLayerView<'_> {
    #[must_use]
    pub const fn key(self) -> u32 {
        self.state.spec.key()
    }
    #[must_use]
    pub const fn kind(self) -> crate::ToughnessLayerKind {
        self.state.spec.kind()
    }
    #[must_use]
    pub const fn current(self) -> crate::RawToughness {
        self.state.current
    }
    #[must_use]
    pub const fn maximum(self) -> crate::RawToughness {
        self.state.spec.maximum()
    }
    #[must_use]
    pub const fn active(self) -> bool {
        self.state.spec.active()
    }
    #[must_use]
    pub const fn locked(self) -> bool {
        self.state.spec.locked()
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
    /// Returns the target-capable action unit, or `None` for a timeline-only actor.
    #[must_use]
    pub const fn unit(self) -> Option<UnitId> {
        self.state.unit
    }
    /// Returns the linked semantic role; ordinary unit actors have no role tag.
    #[must_use]
    pub const fn linked_kind(self) -> Option<crate::LinkedEntityKind> {
        self.state.kind
    }
    /// Returns the automatically executed ability, if any.
    #[must_use]
    pub const fn automatic_ability(self) -> Option<AbilityId> {
        self.state.automatic_ability
    }
    /// Returns whether the actor participates in timeline selection.
    #[must_use]
    pub const fn is_active(self) -> bool {
        self.state.active
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

/// Immutable explicit owner/entity link projection.
#[derive(Clone, Copy)]
pub struct LinkView<'a> {
    state: &'a LinkState,
}

impl LinkView<'_> {
    #[must_use]
    pub const fn owner(self) -> UnitId {
        self.state.owner
    }
    #[must_use]
    pub const fn entity(self) -> crate::LinkedEntity {
        self.state.entity
    }
    #[must_use]
    pub const fn kind(self) -> crate::LinkedEntityKind {
        self.state.kind
    }
    #[must_use]
    pub const fn owner_defeat_policy(self) -> crate::OwnerLinkPolicy {
        self.state.owner_defeat
    }
    #[must_use]
    pub const fn owner_departure_policy(self) -> crate::OwnerLinkPolicy {
        self.state.owner_departure
    }
    #[must_use]
    pub const fn wave_policy(self) -> crate::WaveLinkPolicy {
        self.state.wave
    }
    #[must_use]
    pub const fn is_active(self) -> bool {
        self.state.active
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
    /// Returns a generic team resource and its cap by stable semantic identity.
    #[must_use]
    pub fn keyed_resource(self, id: crate::SourceDefinitionId) -> Option<(u16, u16)> {
        self.state
            .keyed(id)
            .map(|resource| (resource.current, resource.maximum))
    }
}

/// Immutable encounter progress projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncounterView {
    definition: EncounterId,
    wave: WaveInstanceId,
    number: u16,
    total_waves: u16,
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
    /// Returns the one-based current wave number.
    #[must_use]
    pub const fn number(self) -> u16 {
        self.number
    }
    /// Returns the immutable total number of encounter waves.
    #[must_use]
    pub const fn total_waves(self) -> u16 {
        self.total_waves
    }
}
