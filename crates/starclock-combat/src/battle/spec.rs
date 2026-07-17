use core::fmt;

use crate::{
    id::{
        AbilityId, EncounterId, EnemyDefinitionId, ModifierDefinitionId, RuleBundleId,
        UnitDefinitionId,
    },
    numeric::domain::{Hp, Speed},
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
    speed: Speed,
    abilities: Box<[AbilityId]>,
    rule_bundles: Box<[RuleBundleId]>,
    modifiers: Box<[ModifierDefinitionId]>,
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
            speed,
            abilities: bindings.abilities,
            rule_bundles: bindings.rule_bundles,
            modifiers: bindings.modifiers,
            digest,
        })
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
    /// Returns base entry Speed.
    #[must_use]
    pub const fn speed(&self) -> Speed {
        self.speed
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
    source: ParticipantSource,
    combatant: ResolvedCombatantSpec,
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
        Self {
            side,
            formation,
            source,
            combatant,
        }
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
}

/// Initial team-scoped Skill Point state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TeamResourceSpec {
    skill_points: u16,
    maximum_skill_points: u16,
}

impl TeamResourceSpec {
    /// Creates a bounded team resource state.
    #[must_use]
    pub const fn new(skill_points: u16, maximum_skill_points: u16) -> Option<Self> {
        if skill_points <= maximum_skill_points {
            Some(Self {
                skill_points,
                maximum_skill_points,
            })
        } else {
            None
        }
    }

    /// Returns current Skill Points.
    #[must_use]
    pub const fn skill_points(self) -> u16 {
        self.skill_points
    }
    /// Returns the authored cap.
    #[must_use]
    pub const fn maximum_skill_points(self) -> u16 {
        self.maximum_skill_points
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
        participants.sort_by_key(|entry| (entry.side, entry.formation));
        if participants
            .windows(2)
            .any(|pair| pair[0].side == pair[1].side && pair[0].formation == pair[1].formation)
        {
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
    pub const fn resources(&self, side: TeamSide) -> TeamResourceSpec {
        match side {
            TeamSide::Player => self.player_resources,
            TeamSide::Enemy => self.enemy_resources,
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
