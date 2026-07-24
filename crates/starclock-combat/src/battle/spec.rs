use core::fmt;

use crate::{
    formula::{model::CombatElement, toughness::EnemyRank},
    id::{
        AbilityId, EncounterId, EnemyDefinitionId, ModifierDefinitionId, RuleBundleId,
        UnitDefinitionId,
    },
    numeric::domain::{Energy, Hp, Speed, StatValue},
    rule::model::RuleSource,
    toughness::model::ToughnessLayerSpec,
};

/// Maximum occupied formation index accepted by the core model.
pub(crate) const MAX_FORMATION_INDEX: u8 = 31;
/// Maximum player-side formation index in the shared activity contract.
pub(crate) const MAX_PLAYER_FORMATION_INDEX: u8 = 7;
/// Conservative battle-construction bound before linked actors are introduced.
pub(crate) const MAX_INITIAL_PARTICIPANTS: usize = 40;

macro_rules! digest_type {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name([u8; 32]);

        impl $name {
            /// Creates a non-zero SHA-256 identity.
            #[must_use]
            pub fn new(bytes: [u8; 32]) -> Option<Self> {
                (!bytes.iter().all(|byte| *byte == 0)).then_some(Self(bytes))
            }

            /// Returns the canonical digest bytes.
            #[must_use]
            pub const fn bytes(self) -> [u8; 32] {
                self.0
            }
        }
    };
}

digest_type!(
    CombatantSpecDigest,
    "Digest of one generic resolved combatant assembly."
);
digest_type!(
    BattleSpecDigest,
    "Digest of one complete immutable battle request."
);

/// Exact seed of the isolated battle RNG stream.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BattleSeed([u8; 32]);

impl BattleSeed {
    /// Wraps the 32 canonical seed bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the exact seed bytes.
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

/// Checked combatant level accepted by the core; 81-95 support special sources.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnitLevel(u8);

impl UnitLevel {
    /// Creates a level in the documented core range 1-95.
    #[must_use]
    pub const fn new(raw: u8) -> Option<Self> {
        if raw >= 1 && raw <= 95 {
            Some(Self(raw))
        } else {
            None
        }
    }

    /// Returns the integral level.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Canonical formation slot, independent from a unit's stable runtime ID.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FormationIndex(u8);

impl FormationIndex {
    /// Creates a slot in the shared 0-31 battle range.
    #[must_use]
    pub const fn new(raw: u8) -> Option<Self> {
        if raw <= MAX_FORMATION_INDEX {
            Some(Self(raw))
        } else {
            None
        }
    }

    /// Returns the zero-based authored slot.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Stable formation side and controller ownership axis.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum TeamSide {
    /// Player-controlled allied side.
    Player = 0,
    /// Encounter-controlled hostile side.
    Enemy = 1,
}

impl TeamSide {
    pub(crate) const fn canonical_index(self) -> usize {
        self as usize
    }
}

/// Source binding used to validate a participant against the encounter.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ParticipantSource {
    /// Upstream-compiled player or trial combatant.
    Player,
    /// One enemy definition listed by the selected encounter.
    EncounterEnemy(EnemyDefinitionId),
    /// Battle-created linked entity attributed to a stable source definition.
    Linked(crate::SourceDefinitionId),
}

/// Whether this battle profile offers explicit concession.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ConcedePolicy {
    /// Offer concession at player decision points. The no-concession policy is
    /// added only when the normal action pipeline can provide another command.
    Allowed,
}

/// Canonical combat-definition selections produced by an upstream compiler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedDefinitionBindings {
    abilities: Box<[AbilityId]>,
    rule_bundles: Box<[RuleBundleId]>,
    modifiers: Box<[ModifierDefinitionId]>,
}

/// Source-attributed build modifier selected for one resolved combatant.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResolvedModifierBinding {
    definition: ModifierDefinitionId,
    source: crate::SourceDefinitionId,
}

impl ResolvedModifierBinding {
    /// Binds one modifier definition to the generic source that selected it.
    #[must_use]
    pub const fn new(definition: ModifierDefinitionId, source: crate::SourceDefinitionId) -> Self {
        Self { definition, source }
    }

    #[must_use]
    pub const fn definition(self) -> ModifierDefinitionId {
        self.definition
    }

    #[must_use]
    pub const fn source(self) -> crate::SourceDefinitionId {
        self.source
    }
}

impl ResolvedDefinitionBindings {
    /// Validates set-like combat definition references.
    pub fn new(
        abilities: Vec<AbilityId>,
        rule_bundles: Vec<RuleBundleId>,
        modifiers: Vec<ModifierDefinitionId>,
    ) -> Result<Self, CombatantSpecError> {
        if abilities.is_empty() {
            return Err(CombatantSpecError::EmptyAbilitySet);
        }
        if !strictly_ordered(&abilities)
            || !strictly_ordered(&rule_bundles)
            || !strictly_ordered(&modifiers)
        {
            return Err(CombatantSpecError::NonCanonicalReferences);
        }
        Ok(Self {
            abilities: abilities.into_boxed_slice(),
            rule_bundles: rule_bundles.into_boxed_slice(),
            modifiers: modifiers.into_boxed_slice(),
        })
    }

    /// Returns abilities in canonical definition-ID order.
    #[must_use]
    pub fn abilities(&self) -> &[AbilityId] {
        &self.abilities
    }
    /// Returns rule bundles in canonical definition-ID order.
    #[must_use]
    pub fn rule_bundles(&self) -> &[RuleBundleId] {
        &self.rule_bundles
    }
    /// Returns modifiers in canonical definition-ID order.
    #[must_use]
    pub fn modifiers(&self) -> &[ModifierDefinitionId] {
        &self.modifiers
    }
}

/// Generic, build-system-free combatant input accepted by one battle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedCombatantSpec {
    form: UnitDefinitionId,
    level: UnitLevel,
    maximum_hp: Hp,
    base_attack: StatValue,
    base_defense: StatValue,
    speed: Speed,
    current_energy: Energy,
    maximum_energy: Energy,
    rank: EnemyRank,
    weaknesses: Box<[CombatElement]>,
    toughness_layers: Box<[ToughnessLayerSpec]>,
    abilities: Box<[AbilityId]>,
    rule_bundles: Box<[RuleBundleId]>,
    modifiers: Box<[ModifierDefinitionId]>,
    modifier_bindings: Box<[ResolvedModifierBinding]>,
    sources: Box<[RuleSource]>,
    digest: CombatantSpecDigest,
}

impl ResolvedCombatantSpec {
    /// Constructs a canonically ordered generic combatant assembly.
    pub fn new(
        form: UnitDefinitionId,
        level: UnitLevel,
        maximum_hp: Hp,
        speed: Speed,
        bindings: ResolvedDefinitionBindings,
        digest: CombatantSpecDigest,
    ) -> Result<Self, CombatantSpecError> {
        if maximum_hp.get() == 0 {
            return Err(CombatantSpecError::ZeroMaximumHp);
        }
        Ok(Self {
            form,
            level,
            maximum_hp,
            base_attack: StatValue::from_scaled(0).expect("zero is a valid stat"),
            base_defense: StatValue::from_scaled(0).expect("zero is a valid stat"),
            speed,
            current_energy: Energy::ZERO,
            maximum_energy: Energy::ZERO,
            rank: EnemyRank::Normal,
            weaknesses: Box::new([]),
            toughness_layers: Box::new([]),
            abilities: bindings.abilities,
            rule_bundles: bindings.rule_bundles,
            modifiers: bindings.modifiers,
            modifier_bindings: Box::new([]),
            sources: Box::new([]),
            digest,
        })
    }

    /// Attaches generic base ATK and DEF contributions resolved upstream.
    #[must_use]
    pub const fn with_base_attack_defense(mut self, attack: StatValue, defense: StatValue) -> Self {
        self.base_attack = attack;
        self.base_defense = defense;
        self
    }
    /// Attaches canonical generic source bindings resolved upstream.
    pub fn with_sources(mut self, sources: Vec<RuleSource>) -> Result<Self, CombatantSpecError> {
        if sources
            .windows(2)
            .any(|pair| pair[0].definition() >= pair[1].definition())
            || sources.iter().any(|source| {
                source.tags().windows(2).any(|pair| pair[0] >= pair[1])
                    || source.digest().iter().all(|byte| *byte == 0)
            })
            || self.modifier_bindings.iter().any(|binding| {
                sources
                    .binary_search_by_key(&binding.source, RuleSource::definition)
                    .is_err()
            })
        {
            return Err(CombatantSpecError::InvalidSourceBindings);
        }
        self.sources = sources.into_boxed_slice();
        Ok(self)
    }
    /// Attaches one exact source binding for every selected modifier.
    pub fn with_modifier_bindings(
        mut self,
        bindings: Vec<ResolvedModifierBinding>,
    ) -> Result<Self, CombatantSpecError> {
        if bindings
            .windows(2)
            .any(|pair| pair[0].definition >= pair[1].definition)
            || bindings.len() != self.modifiers.len()
            || bindings
                .iter()
                .zip(self.modifiers.iter())
                .any(|(binding, definition)| {
                    binding.definition != *definition
                        || self
                            .sources
                            .binary_search_by_key(&binding.source, RuleSource::definition)
                            .is_err()
                })
        {
            return Err(CombatantSpecError::InvalidModifierBindings);
        }
        self.modifier_bindings = bindings.into_boxed_slice();
        Ok(self)
    }
    /// Sets checked entry and maximum Energy for this resolved combatant.
    pub fn with_energy(
        mut self,
        current: Energy,
        maximum: Energy,
    ) -> Result<Self, CombatantSpecError> {
        if current > maximum {
            return Err(CombatantSpecError::EnergyAboveMaximum);
        }
        self.current_energy = current;
        self.maximum_energy = maximum;
        Ok(self)
    }

    /// Adds canonical elemental weaknesses and ordered Toughness layers.
    pub fn with_toughness(
        mut self,
        rank: EnemyRank,
        weaknesses: Vec<CombatElement>,
        layers: Vec<ToughnessLayerSpec>,
    ) -> Result<Self, CombatantSpecError> {
        if !strictly_ordered(&weaknesses)
            || layers.windows(2).any(|pair| pair[0].key() >= pair[1].key())
        {
            return Err(CombatantSpecError::NonCanonicalToughness);
        }
        self.rank = rank;
        self.weaknesses = weaknesses.into_boxed_slice();
        self.toughness_layers = layers.into_boxed_slice();
        Ok(self)
    }

    /// Returns the selected combat form.
    #[must_use]
    pub const fn form(&self) -> UnitDefinitionId {
        self.form
    }
    /// Returns the combatant level.
    #[must_use]
    pub const fn level(&self) -> UnitLevel {
        self.level
    }
    /// Returns full starting/maximum HP for this baseline input.
    #[must_use]
    pub const fn maximum_hp(&self) -> Hp {
        self.maximum_hp
    }
    /// Returns the resolved generic base ATK contribution.
    #[must_use]
    pub const fn base_attack(&self) -> StatValue {
        self.base_attack
    }
    /// Returns the resolved generic base DEF contribution.
    #[must_use]
    pub const fn base_defense(&self) -> StatValue {
        self.base_defense
    }
    /// Returns base entry Speed.
    #[must_use]
    pub const fn speed(&self) -> Speed {
        self.speed
    }
    /// Returns entry Energy.
    #[must_use]
    pub const fn current_energy(&self) -> Energy {
        self.current_energy
    }
    /// Returns the authored Energy cap.
    #[must_use]
    pub const fn maximum_energy(&self) -> Energy {
        self.maximum_energy
    }
    #[must_use]
    pub const fn rank(&self) -> EnemyRank {
        self.rank
    }
    #[must_use]
    pub fn weaknesses(&self) -> &[CombatElement] {
        &self.weaknesses
    }
    #[must_use]
    pub fn toughness_layers(&self) -> &[ToughnessLayerSpec] {
        &self.toughness_layers
    }
    /// Returns abilities in canonical definition-ID order.
    #[must_use]
    pub fn abilities(&self) -> &[AbilityId] {
        &self.abilities
    }
    /// Returns selected rule bundles in canonical definition-ID order.
    #[must_use]
    pub fn rule_bundles(&self) -> &[RuleBundleId] {
        &self.rule_bundles
    }
    /// Returns selected modifiers in canonical definition-ID order.
    #[must_use]
    pub fn modifiers(&self) -> &[ModifierDefinitionId] {
        &self.modifiers
    }
    /// Returns source-attributed selected modifiers in definition-ID order.
    #[must_use]
    pub fn modifier_bindings(&self) -> &[ResolvedModifierBinding] {
        &self.modifier_bindings
    }
    /// Returns canonical generic source bindings.
    #[must_use]
    pub fn sources(&self) -> &[RuleSource] {
        &self.sources
    }
    /// Returns the exact resolved assembly digest.
    #[must_use]
    pub const fn digest(&self) -> CombatantSpecDigest {
        self.digest
    }
}

/// Validation failure independent from a particular catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombatantSpecError {
    /// A combatant cannot enter with zero maximum HP.
    ZeroMaximumHp,
    /// A targetable combatant requires at least one ability definition.
    EmptyAbilitySet,
    /// Set-like definition references must be strictly increasing and unique.
    NonCanonicalReferences,
    /// Generic source IDs/tags must be canonical and digests must be nonzero.
    InvalidSourceBindings,
    /// Every selected modifier must have one canonical selected-source binding.
    InvalidModifierBindings,
    /// Entry Energy cannot exceed its authored maximum.
    EnergyAboveMaximum,
    /// Weaknesses and layer keys must be strictly ordered and unique.
    NonCanonicalToughness,
}

impl fmt::Display for CombatantSpecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid resolved combatant: {self:?}")
    }
}

impl std::error::Error for CombatantSpecError {}

/// One occupied initial formation slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantSpec {
    side: TeamSide,
    formation: FormationIndex,
    wave: u16,
    source: ParticipantSource,
    combatant: ResolvedCombatantSpec,
    locked_combatant_digest: CombatantSpecDigest,
    initial_state: Option<ParticipantInitialState>,
}

/// Optional runtime state supplied by a cross-battle activity handoff.
///
/// Ordinary standalone battles omit this value and retain the authored
/// full-HP/current-Energy defaults from [`ResolvedCombatantSpec`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParticipantInitialState {
    current_hp: Hp,
    current_energy: Energy,
    life: crate::LifeState,
    presence: crate::PresenceState,
}

impl ParticipantInitialState {
    #[must_use]
    pub fn new(
        current_hp: Hp,
        maximum_hp: Hp,
        current_energy: Energy,
        maximum_energy: Energy,
        life: crate::LifeState,
        presence: crate::PresenceState,
    ) -> Option<Self> {
        if current_hp.get() > maximum_hp.get()
            || current_energy.scaled() > maximum_energy.scaled()
            || (life == crate::LifeState::Alive && current_hp.get() == 0)
            || (matches!(life, crate::LifeState::Downed | crate::LifeState::Defeated)
                && current_hp.get() != 0)
        {
            return None;
        }
        Some(Self {
            current_hp,
            current_energy,
            life,
            presence,
        })
    }

    #[must_use]
    pub const fn current_hp(self) -> Hp {
        self.current_hp
    }
    #[must_use]
    pub const fn current_energy(self) -> Energy {
        self.current_energy
    }
    #[must_use]
    pub const fn life(self) -> crate::LifeState {
        self.life
    }
    #[must_use]
    pub const fn presence(self) -> crate::PresenceState {
        self.presence
    }
}

impl ParticipantSpec {
    /// Binds one resolved combatant to a side, formation slot and source.
    #[must_use]
    pub const fn new(
        side: TeamSide,
        formation: FormationIndex,
        source: ParticipantSource,
        combatant: ResolvedCombatantSpec,
    ) -> Self {
        let locked_combatant_digest = combatant.digest();
        Self {
            side,
            formation,
            wave: 1,
            source,
            combatant,
            locked_combatant_digest,
            initial_state: None,
        }
    }

    /// Assigns an encounter enemy to a one-based wave; players remain wave one.
    #[must_use]
    pub fn with_wave(mut self, wave: u16) -> Option<Self> {
        if wave == 0 || (matches!(self.side, TeamSide::Player) && wave != 1) {
            None
        } else {
            self.wave = wave;
            Some(self)
        }
    }

    /// Supplies validated activity carry state for this exact participant.
    #[must_use]
    pub fn with_initial_state(mut self, state: ParticipantInitialState) -> Option<Self> {
        if state.current_hp.get() > self.combatant.maximum_hp().get()
            || state.current_energy.scaled() > self.combatant.maximum_energy().scaled()
        {
            None
        } else {
            self.initial_state = Some(state);
            Some(self)
        }
    }

    /// Retains the pre-mode resolved build identity checked by an Activity
    /// participant lock when runtime mode contributions alter the combatant.
    #[must_use]
    pub fn with_locked_combatant_digest(
        mut self,
        locked_combatant_digest: CombatantSpecDigest,
    ) -> Self {
        self.locked_combatant_digest = locked_combatant_digest;
        self
    }

    /// Returns the formation side.
    #[must_use]
    pub const fn side(&self) -> TeamSide {
        self.side
    }
    /// Returns the zero-based formation slot.
    #[must_use]
    pub const fn formation(&self) -> FormationIndex {
        self.formation
    }
    /// Returns the one-based encounter entry wave.
    #[must_use]
    pub const fn wave(&self) -> u16 {
        self.wave
    }
    /// Returns the encounter/player source binding.
    #[must_use]
    pub const fn source(&self) -> ParticipantSource {
        self.source
    }
    /// Returns the generic combatant assembly.
    #[must_use]
    pub const fn combatant(&self) -> &ResolvedCombatantSpec {
        &self.combatant
    }
    /// Returns the pre-mode combatant identity bound by an outer Activity.
    #[must_use]
    pub const fn locked_combatant_digest(&self) -> CombatantSpecDigest {
        self.locked_combatant_digest
    }
    /// Returns explicit cross-battle carry state when supplied.
    #[must_use]
    pub const fn initial_state(&self) -> Option<ParticipantInitialState> {
        self.initial_state
    }
}

/// Authored wave-boundary behavior for one keyed team resource.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum TeamResourceWavePolicy {
    Persist = 0,
    ResetToInitial = 1,
    Clear = 2,
}

/// One generic team-owned resource definition such as a shared subsystem tally.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyedTeamResourceSpec {
    id: crate::SourceDefinitionId,
    stable_key: Option<Box<str>>,
    initial: u16,
    maximum: u16,
    wave: TeamResourceWavePolicy,
}

impl KeyedTeamResourceSpec {
    #[must_use]
    pub const fn new(
        id: crate::SourceDefinitionId,
        initial: u16,
        maximum: u16,
        wave: TeamResourceWavePolicy,
    ) -> Option<Self> {
        if initial <= maximum {
            Some(Self {
                id,
                stable_key: None,
                initial,
                maximum,
                wave,
            })
        } else {
            None
        }
    }
    /// Binds the numeric resource to the exact Rule IR semantic key.
    #[must_use]
    pub fn with_stable_key(mut self, stable_key: impl Into<Box<str>>) -> Option<Self> {
        let stable_key = stable_key.into();
        if stable_key.trim().is_empty() {
            return None;
        }
        self.stable_key = Some(stable_key);
        Some(self)
    }
    #[must_use]
    pub const fn id(&self) -> crate::SourceDefinitionId {
        self.id
    }
    /// Returns the Rule IR semantic key when this resource is named.
    #[must_use]
    pub fn stable_key(&self) -> Option<&str> {
        self.stable_key.as_deref()
    }
    #[must_use]
    pub const fn initial(&self) -> u16 {
        self.initial
    }
    #[must_use]
    pub const fn maximum(&self) -> u16 {
        self.maximum
    }
    #[must_use]
    pub const fn wave(&self) -> TeamResourceWavePolicy {
        self.wave
    }
}

/// Initial team-scoped Skill Point and generic shared-resource state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TeamResourceSpec {
    skill_points: u16,
    maximum_skill_points: u16,
    keyed: Box<[KeyedTeamResourceSpec]>,
}

impl TeamResourceSpec {
    /// Creates a bounded team resource state.
    #[must_use]
    pub fn new(skill_points: u16, maximum_skill_points: u16) -> Option<Self> {
        if skill_points <= maximum_skill_points {
            Some(Self {
                skill_points,
                maximum_skill_points,
                keyed: Box::new([]),
            })
        } else {
            None
        }
    }

    /// Returns current Skill Points.
    #[must_use]
    pub const fn skill_points(&self) -> u16 {
        self.skill_points
    }
    /// Returns the authored cap.
    #[must_use]
    pub const fn maximum_skill_points(&self) -> u16 {
        self.maximum_skill_points
    }
    /// Adds a canonical unique set of generic team-owned resources.
    #[must_use]
    pub fn with_keyed(mut self, mut keyed: Vec<KeyedTeamResourceSpec>) -> Option<Self> {
        keyed.sort_by_key(|entry| entry.id);
        if keyed.windows(2).any(|pair| pair[0].id == pair[1].id)
            || keyed
                .iter()
                .filter_map(|entry| entry.stable_key())
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != keyed
                    .iter()
                    .filter(|entry| entry.stable_key.is_some())
                    .count()
        {
            return None;
        }
        self.keyed = keyed.into_boxed_slice();
        Some(self)
    }
    #[must_use]
    pub fn keyed(&self) -> &[KeyedTeamResourceSpec] {
        &self.keyed
    }
}

/// Complete immutable request for constructing one isolated battle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleSpec {
    rules_revision: Box<str>,
    digest: BattleSpecDigest,
    encounter: EncounterId,
    participants: Box<[ParticipantSpec]>,
    player_resources: TeamResourceSpec,
    enemy_resources: TeamResourceSpec,
    concede: ConcedePolicy,
}

impl BattleSpec {
    /// Validates local shape and canonicalizes participants by side/formation.
    pub fn new(
        rules_revision: impl Into<Box<str>>,
        digest: BattleSpecDigest,
        encounter: EncounterId,
        mut participants: Vec<ParticipantSpec>,
        player_resources: TeamResourceSpec,
        enemy_resources: TeamResourceSpec,
        concede: ConcedePolicy,
    ) -> Result<Self, BattleSpecError> {
        let rules_revision = rules_revision.into();
        if rules_revision.is_empty()
            || rules_revision.len() > 128
            || !rules_revision.bytes().all(|byte| byte.is_ascii_graphic())
        {
            return Err(BattleSpecError::InvalidRulesRevision);
        }
        if participants.is_empty() {
            return Err(BattleSpecError::EmptyParticipants);
        }
        if participants.len() > MAX_INITIAL_PARTICIPANTS {
            return Err(BattleSpecError::TooManyParticipants);
        }
        participants.sort_by_key(|entry| (entry.wave, entry.side, entry.formation));
        if participants.windows(2).any(|pair| {
            pair[0].wave == pair[1].wave
                && pair[0].side == pair[1].side
                && pair[0].formation == pair[1].formation
        }) {
            return Err(BattleSpecError::DuplicateFormation);
        }
        if participants.iter().any(|entry| {
            entry.side == TeamSide::Player && entry.formation.get() > MAX_PLAYER_FORMATION_INDEX
        }) {
            return Err(BattleSpecError::PlayerFormationOutOfRange);
        }
        if !participants
            .iter()
            .any(|entry| entry.side == TeamSide::Player)
            || !participants
                .iter()
                .any(|entry| entry.side == TeamSide::Enemy)
        {
            return Err(BattleSpecError::MissingSide);
        }
        Ok(Self {
            rules_revision,
            digest,
            encounter,
            participants: participants.into_boxed_slice(),
            player_resources,
            enemy_resources,
            concede,
        })
    }

    /// Returns the rules compatibility revision.
    #[must_use]
    pub fn rules_revision(&self) -> &str {
        &self.rules_revision
    }
    /// Returns the exact battle request digest.
    #[must_use]
    pub const fn digest(&self) -> BattleSpecDigest {
        self.digest
    }
    /// Returns the selected encounter definition.
    #[must_use]
    pub const fn encounter(&self) -> EncounterId {
        self.encounter
    }
    /// Returns participants in canonical side/formation order.
    #[must_use]
    pub fn participants(&self) -> &[ParticipantSpec] {
        &self.participants
    }
    /// Returns one side's initial team resource state.
    #[must_use]
    pub const fn resources(&self, side: TeamSide) -> &TeamResourceSpec {
        match side {
            TeamSide::Player => &self.player_resources,
            TeamSide::Enemy => &self.enemy_resources,
        }
    }
    /// Returns whether concession is part of this profile's command surface.
    #[must_use]
    pub const fn concede_policy(&self) -> ConcedePolicy {
        self.concede
    }
}

/// Local `BattleSpec` construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleSpecError {
    /// Rules revisions are non-empty printable ASCII up to 128 bytes.
    InvalidRulesRevision,
    /// At least one participant is required.
    EmptyParticipants,
    /// Initial construction exceeded its reviewed hard bound.
    TooManyParticipants,
    /// Two participants occupy the same side/formation slot.
    DuplicateFormation,
    /// Player-side initial slots are limited to 0-7.
    PlayerFormationOutOfRange,
    /// Initial formation must contain both player and enemy sides.
    MissingSide,
}

impl fmt::Display for BattleSpecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid battle specification: {self:?}")
    }
}

impl std::error::Error for BattleSpecError {}

fn strictly_ordered<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}
