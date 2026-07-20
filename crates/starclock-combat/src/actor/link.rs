use crate::{
    AbilityId, ActionGauge, FormationIndex, Hp, PresenceState, ResolvedCombatantSpec,
    SourceDefinitionId, Speed, TimelineActorId, UnitDefinitionId, UnitId,
};

/// Shared semantic role of an entity linked to a combat unit.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum LinkedEntityKind {
    /// Independently scheduled non-memosprite summon.
    Summon = 0,
    /// Target-capable Remembrance memosprite.
    Memosprite = 1,
    /// Timeline-only state-ending or encounter countdown.
    Countdown = 2,
    /// Team-shared subsystem unit whose provider is attribution, not lifecycle ownership.
    SharedActor = 3,
}

/// Authored response of a link when its owner reaches a lifecycle boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum OwnerLinkPolicy {
    /// Preserve the linked entity and its current state.
    Persist = 0,
    /// Depart/deactivate the linked entity without changing its life state.
    Depart = 1,
    /// Set a linked unit to Defeated and deactivate its actor.
    Defeat = 2,
}

/// Authored response of a link when an encounter wave changes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum WaveLinkPolicy {
    /// Preserve state and Action Gauge across the wave boundary.
    Persist = 0,
    /// Preserve the entity but reset its linked Action Gauge.
    ResetGauge = 1,
    /// Depart/deactivate the entity at wave end.
    Depart = 2,
}

/// Runtime entity addressed by one explicit owner link.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LinkedEntity {
    /// Target-capable unit, optionally with its own timeline actor.
    Unit(UnitId),
    /// Timeline-only actor such as a transformation countdown.
    TimelineActor(TimelineActorId),
}

/// Policy for an active transformation at owner defeat or wave transition.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum TransformEndPolicy {
    /// Keep the transformed definition and state.
    Persist = 0,
    /// Restore the original definition and ability set exactly once.
    End = 1,
}

/// Action-Gauge position assigned by an explicit revival.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ReviveGaugePolicy {
    /// Retain the actor's current gauge.
    Preserve = 0,
    /// Reset the actor to the full base gauge.
    Reset = 1,
    /// Make the actor eligible at zero gauge.
    Immediate = 2,
}

/// Complete battle-domain input for one newly allocated linked unit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedUnitDefinition {
    combatant: ResolvedCombatantSpec,
    source: SourceDefinitionId,
    formation: FormationIndex,
    kind: LinkedEntityKind,
    presence: PresenceState,
    action_ability: Option<AbilityId>,
    initial_gauge: ActionGauge,
    owner_defeat: OwnerLinkPolicy,
    owner_departure: OwnerLinkPolicy,
    wave: WaveLinkPolicy,
}

impl LinkedUnitDefinition {
    /// Creates a linked summon or memosprite with explicit lifecycle policies.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        combatant: ResolvedCombatantSpec,
        source: SourceDefinitionId,
        formation: FormationIndex,
        kind: LinkedEntityKind,
        presence: PresenceState,
        action_ability: Option<AbilityId>,
        initial_gauge: ActionGauge,
        owner_defeat: OwnerLinkPolicy,
        owner_departure: OwnerLinkPolicy,
        wave: WaveLinkPolicy,
    ) -> Option<Self> {
        if kind == LinkedEntityKind::Countdown
            || (kind == LinkedEntityKind::SharedActor && action_ability.is_some())
            || !matches!(
                presence,
                PresenceState::Present | PresenceState::Untargetable | PresenceState::Linked
            )
            || action_ability
                .is_some_and(|ability| combatant.abilities().binary_search(&ability).is_err())
        {
            return None;
        }
        Some(Self {
            combatant,
            source,
            formation,
            kind,
            presence,
            action_ability,
            initial_gauge,
            owner_defeat,
            owner_departure,
            wave,
        })
    }

    pub(crate) const fn combatant(&self) -> &ResolvedCombatantSpec {
        &self.combatant
    }
    pub(crate) const fn source(&self) -> SourceDefinitionId {
        self.source
    }
    pub(crate) const fn formation(&self) -> FormationIndex {
        self.formation
    }
    pub(crate) const fn kind(&self) -> LinkedEntityKind {
        self.kind
    }
    pub(crate) const fn presence(&self) -> PresenceState {
        self.presence
    }
    pub(crate) const fn action_ability(&self) -> Option<AbilityId> {
        self.action_ability
    }
    pub(crate) const fn initial_gauge(&self) -> ActionGauge {
        self.initial_gauge
    }
    pub(crate) const fn owner_defeat(&self) -> OwnerLinkPolicy {
        self.owner_defeat
    }
    pub(crate) const fn owner_departure(&self) -> OwnerLinkPolicy {
        self.owner_departure
    }
    pub(crate) const fn wave(&self) -> WaveLinkPolicy {
        self.wave
    }
}

/// Catalog-owned linked combatant resolved by a typed Rule IR summon.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedUnitCatalogDefinition {
    id: crate::UnitDefinitionId,
    definition: LinkedUnitDefinition,
}

impl LinkedUnitCatalogDefinition {
    #[must_use]
    pub fn new(id: crate::UnitDefinitionId, definition: LinkedUnitDefinition) -> Option<Self> {
        (definition.combatant().form() == id).then_some(Self { id, definition })
    }
    pub(crate) const fn id(&self) -> crate::UnitDefinitionId {
        self.id
    }
    pub(crate) const fn definition(&self) -> &LinkedUnitDefinition {
        &self.definition
    }
}

/// Timeline-only countdown created by an authored transformation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountdownDefinition {
    ability: AbilityId,
    initial_gauge: ActionGauge,
    speed: Speed,
    owner_defeat: OwnerLinkPolicy,
    owner_departure: OwnerLinkPolicy,
    wave: WaveLinkPolicy,
}

impl CountdownDefinition {
    #[must_use]
    pub const fn new(
        ability: AbilityId,
        initial_gauge: ActionGauge,
        speed: Speed,
        owner_defeat: OwnerLinkPolicy,
        owner_departure: OwnerLinkPolicy,
        wave: WaveLinkPolicy,
    ) -> Self {
        Self {
            ability,
            initial_gauge,
            speed,
            owner_defeat,
            owner_departure,
            wave,
        }
    }
    pub(crate) const fn ability(self) -> AbilityId {
        self.ability
    }
    pub(crate) const fn initial_gauge(self) -> ActionGauge {
        self.initial_gauge
    }
    pub(crate) const fn speed(self) -> Speed {
        self.speed
    }
    pub(crate) const fn owner_defeat(self) -> OwnerLinkPolicy {
        self.owner_defeat
    }
    pub(crate) const fn owner_departure(self) -> OwnerLinkPolicy {
        self.owner_departure
    }
    pub(crate) const fn wave(self) -> WaveLinkPolicy {
        self.wave
    }
}

/// Catalog-owned timeline-only definition resolved by a Rule IR countdown code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountdownCatalogDefinition {
    code: u32,
    definition: CountdownDefinition,
}

impl CountdownCatalogDefinition {
    #[must_use]
    pub const fn new(code: u32, definition: CountdownDefinition) -> Option<Self> {
        if code == 0 {
            None
        } else {
            Some(Self { code, definition })
        }
    }
    pub(crate) const fn code(self) -> u32 {
        self.code
    }
    pub(crate) const fn definition(self) -> CountdownDefinition {
        self.definition
    }
}

/// Atomic ability/form replacement with an optional state-ending countdown.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransformationDefinition {
    replacement_form: UnitDefinitionId,
    replacement_abilities: Box<[AbilityId]>,
    countdown: Option<CountdownDefinition>,
    defeat: TransformEndPolicy,
    wave: TransformEndPolicy,
}

impl TransformationDefinition {
    #[must_use]
    pub fn new(
        replacement_form: UnitDefinitionId,
        replacement_abilities: Vec<AbilityId>,
        countdown: Option<CountdownDefinition>,
        defeat: TransformEndPolicy,
        wave: TransformEndPolicy,
    ) -> Option<Self> {
        if replacement_abilities.is_empty()
            || replacement_abilities
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return None;
        }
        Some(Self {
            replacement_form,
            replacement_abilities: replacement_abilities.into_boxed_slice(),
            countdown,
            defeat,
            wave,
        })
    }
    pub(crate) const fn replacement_form(&self) -> UnitDefinitionId {
        self.replacement_form
    }
    pub(crate) fn replacement_abilities(&self) -> &[AbilityId] {
        &self.replacement_abilities
    }
    pub(crate) const fn countdown(&self) -> Option<CountdownDefinition> {
        self.countdown
    }
    pub(crate) const fn defeat(&self) -> TransformEndPolicy {
        self.defeat
    }
    pub(crate) const fn wave(&self) -> TransformEndPolicy {
        self.wave
    }
}

/// Explicit revival state; cleanup is authored through adjacent effect operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviveDefinition {
    restored_hp: Hp,
    presence: PresenceState,
    gauge: ReviveGaugePolicy,
}

impl ReviveDefinition {
    #[must_use]
    pub const fn new(
        restored_hp: Hp,
        presence: PresenceState,
        gauge: ReviveGaugePolicy,
    ) -> Option<Self> {
        if restored_hp.get() == 0 || !presence.is_active() {
            None
        } else {
            Some(Self {
                restored_hp,
                presence,
                gauge,
            })
        }
    }
    pub(crate) const fn restored_hp(self) -> Hp {
        self.restored_hp
    }
    pub(crate) const fn presence(self) -> PresenceState {
        self.presence
    }
    pub(crate) const fn gauge(self) -> ReviveGaugePolicy {
        self.gauge
    }
}
