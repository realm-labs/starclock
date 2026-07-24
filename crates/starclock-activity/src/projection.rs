use starclock_combat::{
    AssemblyDigest, BattleFault, BattleSeed, BattleSpecDigest, BattleStateHash, CombatInputDigest,
    Energy, Hp, LifeState, PresenceState,
};

use crate::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityInstanceId, BattleProjectionDigest,
    BattleResultDigest, BattleSequence, EventDigest, ParticipantId, ParticipantLockDigest,
    ProjectionId, ScopeIdentity, TerminalOutcome, codec::CanonicalWriter,
};

/// Terminal battle outcome returned across the orchestration seam.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BattleOutcome {
    Won = 0,
    Lost = 1,
    Faulted = 2,
}

impl From<BattleOutcome> for TerminalOutcome {
    fn from(value: BattleOutcome) -> Self {
        match value {
            BattleOutcome::Won => Self::Complete,
            BattleOutcome::Lost => Self::Failed,
            BattleOutcome::Faulted => Self::Faulted,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum MetricValueKind {
    BoundedInteger = 0,
    FixedScalar = 1,
    Ratio = 2,
    Probability = 3,
    ActionValue = 4,
}

/// Exact typed metric payload. Fixed-point domains use signed millionths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetricValue {
    BoundedInteger(i64),
    FixedScalar(i64),
    Ratio(i64),
    Probability(i64),
    ActionValue(i64),
}

impl MetricValue {
    #[must_use]
    pub const fn kind(self) -> MetricValueKind {
        match self {
            Self::BoundedInteger(_) => MetricValueKind::BoundedInteger,
            Self::FixedScalar(_) => MetricValueKind::FixedScalar,
            Self::Ratio(_) => MetricValueKind::Ratio,
            Self::Probability(_) => MetricValueKind::Probability,
            Self::ActionValue(_) => MetricValueKind::ActionValue,
        }
    }

    const fn raw(self) -> i64 {
        match self {
            Self::BoundedInteger(value)
            | Self::FixedScalar(value)
            | Self::Ratio(value)
            | Self::Probability(value)
            | Self::ActionValue(value) => value,
        }
    }
}

/// One field declared before a battle starts.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProjectionField {
    Outcome,
    FinalStateHash,
    EventDigest,
    TerminalFault,
    ParticipantState(ParticipantId),
    Metric {
        key: Box<str>,
        kind: MetricValueKind,
    },
}

/// Closed declared projection used to validate the returned payload exactly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleResultProjection {
    id: ProjectionId,
    fields: Box<[ProjectionField]>,
}

impl BattleResultProjection {
    pub fn new(
        id: ProjectionId,
        fields: Vec<ProjectionField>,
    ) -> Result<Self, BattleResultProjectionError> {
        if fields.len() < 4 || fields.len() > 100 {
            return Err(BattleResultProjectionError::InvalidFieldCount);
        }
        for required in [
            ProjectionField::Outcome,
            ProjectionField::FinalStateHash,
            ProjectionField::EventDigest,
            ProjectionField::TerminalFault,
        ] {
            if fields.iter().filter(|field| **field == required).count() != 1 {
                return Err(BattleResultProjectionError::MissingOrDuplicateCoreField);
            }
        }
        for (index, field) in fields.iter().enumerate() {
            if let ProjectionField::Metric { key, .. } = field {
                if !valid_key(key) {
                    return Err(BattleResultProjectionError::InvalidMetricKey);
                }
                if fields[..index].iter().any(|prior| matches!(prior, ProjectionField::Metric { key: prior, .. } if prior == key)) {
                    return Err(BattleResultProjectionError::DuplicateMetricKey);
                }
            }
            if let ProjectionField::ParticipantState(participant) = field
                && fields[..index]
                    .iter()
                    .any(|prior| prior == &ProjectionField::ParticipantState(*participant))
            {
                return Err(BattleResultProjectionError::DuplicateParticipant);
            }
        }
        Ok(Self {
            id,
            fields: fields.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn id(&self) -> ProjectionId {
        self.id
    }
    #[must_use]
    pub fn fields(&self) -> &[ProjectionField] {
        &self.fields
    }

    #[must_use]
    pub fn digest(&self) -> BattleProjectionDigest {
        let mut writer = CanonicalWriter::new(b"starclock-battle-projection-v1");
        self.encode(&mut writer);
        BattleProjectionDigest::new(writer.finish()).expect("SHA-256 output is non-zero")
    }

    pub(crate) fn matches(&self, values: &[ProjectedValue]) -> bool {
        self.fields.len() == values.len()
            && self
                .fields
                .iter()
                .zip(values)
                .all(|(field, value)| value.matches(field))
    }

    pub(crate) fn encode(&self, writer: &mut CanonicalWriter) {
        writer.u32(self.id.get());
        writer.u64(self.fields.len() as u64);
        for field in &self.fields {
            encode_field(field, writer);
        }
    }
}

/// Immutable configuration digests carried by every returned result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BattleResultConfiguration {
    definition_digest: ActivityDefinitionDigest,
    config_digest: ActivityConfigDigest,
    participant_lock_digest: ParticipantLockDigest,
}

impl BattleResultConfiguration {
    #[must_use]
    pub const fn new(
        definition_digest: ActivityDefinitionDigest,
        config_digest: ActivityConfigDigest,
        participant_lock_digest: ParticipantLockDigest,
    ) -> Self {
        Self {
            definition_digest,
            config_digest,
            participant_lock_digest,
        }
    }
}

/// Full identity returned with a battle result; every field is checked by Activity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BattleResultIdentity {
    scope: ScopeIdentity,
    battle_sequence: BattleSequence,
    configuration: BattleResultConfiguration,
    combat_input_digest: CombatInputDigest,
    assembly_digest: AssemblyDigest,
    seed: BattleSeed,
}

impl BattleResultIdentity {
    #[must_use]
    pub const fn new(
        scope: ScopeIdentity,
        battle_sequence: BattleSequence,
        configuration: BattleResultConfiguration,
        combat_input_digest: CombatInputDigest,
        assembly_digest: AssemblyDigest,
        seed: BattleSeed,
    ) -> Self {
        Self {
            scope,
            battle_sequence,
            configuration,
            combat_input_digest,
            assembly_digest,
            seed,
        }
    }

    /// Reconstructs the historical single-digest identity used by replay v2.
    #[must_use]
    pub fn new_legacy(
        scope: ScopeIdentity,
        battle_sequence: BattleSequence,
        configuration: BattleResultConfiguration,
        spec_digest: BattleSpecDigest,
        seed: BattleSeed,
    ) -> Self {
        Self::new(
            scope,
            battle_sequence,
            configuration,
            CombatInputDigest::new(spec_digest.bytes())
                .expect("legacy BattleSpecDigest is non-zero"),
            AssemblyDigest::new(spec_digest.bytes()).expect("legacy BattleSpecDigest is non-zero"),
            seed,
        )
    }

    #[must_use]
    pub const fn activity(self) -> ActivityInstanceId {
        self.scope.activity()
    }
    #[must_use]
    pub const fn scope(self) -> ScopeIdentity {
        self.scope
    }
    #[must_use]
    pub const fn battle_sequence(self) -> BattleSequence {
        self.battle_sequence
    }
    #[must_use]
    pub const fn definition_digest(self) -> ActivityDefinitionDigest {
        self.configuration.definition_digest
    }
    #[must_use]
    pub const fn config_digest(self) -> ActivityConfigDigest {
        self.configuration.config_digest
    }
    #[must_use]
    pub const fn participant_lock_digest(self) -> ParticipantLockDigest {
        self.configuration.participant_lock_digest
    }
    #[must_use]
    pub const fn combat_input_digest(self) -> CombatInputDigest {
        self.combat_input_digest
    }
    #[must_use]
    pub const fn assembly_digest(self) -> AssemblyDigest {
        self.assembly_digest
    }
    /// Returns historical single-digest assembly identity for replay v2 only.
    #[must_use]
    pub fn spec_digest(self) -> BattleSpecDigest {
        BattleSpecDigest::new(self.assembly_digest.bytes())
            .expect("assembly identities are non-zero")
    }
    #[must_use]
    pub const fn seed(self) -> BattleSeed {
        self.seed
    }

    pub(crate) fn encode(self, writer: &mut CanonicalWriter) {
        writer.u64(self.scope.activity().get());
        writer.u32(self.scope.section().get());
        writer.u32(self.scope.node().get());
        writer.u32(self.scope.attempt().get());
        writer.u32(self.battle_sequence.get());
        writer.digest(self.configuration.definition_digest.bytes());
        writer.digest(self.configuration.config_digest.bytes());
        writer.digest(self.configuration.participant_lock_digest.bytes());
        writer.text(starclock_combat::COMBAT_INPUT_CODEC_REVISION);
        writer.digest(self.combat_input_digest.bytes());
        writer.digest(self.assembly_digest.bytes());
        writer.digest(self.seed.bytes());
    }
}

/// One returned value corresponding positionally to a declared projection field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectedValue {
    Outcome(BattleOutcome),
    FinalStateHash(BattleStateHash),
    EventDigest(EventDigest),
    TerminalFault(Option<BattleFault>),
    ParticipantState(ParticipantBattleState),
    Metric { key: Box<str>, value: MetricValue },
}

impl ProjectedValue {
    fn matches(&self, field: &ProjectionField) -> bool {
        match (field, self) {
            (ProjectionField::Outcome, Self::Outcome(_))
            | (ProjectionField::FinalStateHash, Self::FinalStateHash(_))
            | (ProjectionField::EventDigest, Self::EventDigest(_))
            | (ProjectionField::TerminalFault, Self::TerminalFault(_)) => true,
            (ProjectionField::ParticipantState(participant), Self::ParticipantState(state)) => {
                *participant == state.participant
            }
            (
                ProjectionField::Metric { key, kind },
                Self::Metric {
                    key: value_key,
                    value,
                },
            ) => key == value_key && *kind == value.kind(),
            _ => false,
        }
    }

    fn encode(&self, writer: &mut CanonicalWriter) {
        match self {
            Self::Outcome(value) => {
                writer.byte(0);
                writer.byte(*value as u8);
            }
            Self::FinalStateHash(value) => {
                writer.byte(1);
                writer.digest(value.bytes());
            }
            Self::EventDigest(value) => {
                writer.byte(2);
                writer.digest(value.bytes());
            }
            Self::TerminalFault(value) => {
                writer.byte(3);
                writer.bool(value.is_some());
                if let Some(value) = value {
                    writer.byte(value.kind() as u8);
                    writer.byte(value.boundary() as u8);
                    writer.byte(value.policy() as u8);
                    writer.u32(value.context_code());
                    writer.bool(value.numeric_context().is_some());
                    if let Some(context) = value.numeric_context() {
                        writer.i64(context);
                    }
                }
            }
            Self::Metric { key, value } => {
                writer.byte(4);
                writer.text(key);
                writer.byte(value.kind() as u8);
                writer.i64(value.raw());
            }
            Self::ParticipantState(value) => {
                writer.byte(5);
                value.encode(writer);
            }
        }
    }
}

/// Untrusted battle-result envelope. Activity verifies identity, digest and projection atomically.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleResult {
    identity: BattleResultIdentity,
    values: Box<[ProjectedValue]>,
    claimed_digest: BattleResultDigest,
}

impl BattleResult {
    #[must_use]
    pub fn new(
        identity: BattleResultIdentity,
        values: Vec<ProjectedValue>,
        claimed_digest: BattleResultDigest,
    ) -> Self {
        Self {
            identity,
            values: values.into_boxed_slice(),
            claimed_digest,
        }
    }

    #[must_use]
    pub fn seal(identity: BattleResultIdentity, values: Vec<ProjectedValue>) -> Self {
        let claimed_digest = Self::digest_for(identity, &values);
        Self::new(identity, values, claimed_digest)
    }

    #[must_use]
    pub fn digest_for(
        identity: BattleResultIdentity,
        values: &[ProjectedValue],
    ) -> BattleResultDigest {
        let mut writer = CanonicalWriter::new(b"starclock-battle-result-v1");
        identity.encode(&mut writer);
        writer.u64(values.len() as u64);
        for value in values {
            value.encode(&mut writer);
        }
        BattleResultDigest::new(writer.finish()).expect("SHA-256 output is non-zero")
    }

    #[must_use]
    pub const fn identity(&self) -> BattleResultIdentity {
        self.identity
    }
    #[must_use]
    pub fn values(&self) -> &[ProjectedValue] {
        &self.values
    }
    #[must_use]
    pub const fn claimed_digest(&self) -> BattleResultDigest {
        self.claimed_digest
    }
    #[must_use]
    pub fn actual_digest(&self) -> BattleResultDigest {
        Self::digest_for(self.identity, &self.values)
    }

    pub(crate) fn outcome(&self) -> Option<BattleOutcome> {
        self.values.iter().find_map(|value| match value {
            ProjectedValue::Outcome(outcome) => Some(*outcome),
            _ => None,
        })
    }

    pub(crate) fn terminal_fault(&self) -> Option<Option<BattleFault>> {
        self.values.iter().find_map(|value| match value {
            ProjectedValue::TerminalFault(fault) => Some(*fault),
            _ => None,
        })
    }

    pub(crate) fn participant_states(&self) -> impl Iterator<Item = ParticipantBattleState> + '_ {
        self.values.iter().filter_map(|value| match value {
            ProjectedValue::ParticipantState(state) => Some(*state),
            _ => None,
        })
    }

    pub(crate) fn metrics(&self) -> impl Iterator<Item = (&str, MetricValue)> + '_ {
        self.values.iter().filter_map(|value| match value {
            ProjectedValue::Metric { key, value } => Some((key.as_ref(), *value)),
            _ => None,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParticipantBattleState {
    participant: ParticipantId,
    current_hp: Hp,
    maximum_hp: Hp,
    current_energy: Energy,
    maximum_energy: Energy,
    life: LifeState,
    presence: PresenceState,
}

impl ParticipantBattleState {
    #[must_use]
    pub fn new(
        participant: ParticipantId,
        current_hp: Hp,
        maximum_hp: Hp,
        current_energy: Energy,
        maximum_energy: Energy,
        life: LifeState,
        presence: PresenceState,
    ) -> Option<Self> {
        if current_hp.get() > maximum_hp.get()
            || current_energy.scaled() > maximum_energy.scaled()
            || (matches!(life, LifeState::Alive) && current_hp.get() == 0)
            || (matches!(life, LifeState::Downed | LifeState::Defeated) && current_hp.get() != 0)
        {
            None
        } else {
            Some(Self {
                participant,
                current_hp,
                maximum_hp,
                current_energy,
                maximum_energy,
                life,
                presence,
            })
        }
    }
    #[must_use]
    pub const fn participant(self) -> ParticipantId {
        self.participant
    }
    #[must_use]
    pub const fn current_hp(self) -> Hp {
        self.current_hp
    }
    #[must_use]
    pub const fn maximum_hp(self) -> Hp {
        self.maximum_hp
    }
    #[must_use]
    pub const fn current_energy(self) -> Energy {
        self.current_energy
    }
    #[must_use]
    pub const fn maximum_energy(self) -> Energy {
        self.maximum_energy
    }
    #[must_use]
    pub const fn life(self) -> LifeState {
        self.life
    }
    #[must_use]
    pub const fn presence(self) -> PresenceState {
        self.presence
    }

    fn encode(self, writer: &mut CanonicalWriter) {
        writer.u32(self.participant.get());
        writer.i64(self.current_hp.get());
        writer.i64(self.maximum_hp.get());
        writer.i64(self.current_energy.scaled());
        writer.i64(self.maximum_energy.scaled());
        writer.byte(self.life as u8);
        writer.byte(self.presence as u8);
    }
}

fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 120
        && key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn encode_field(field: &ProjectionField, writer: &mut CanonicalWriter) {
    match field {
        ProjectionField::Outcome => writer.byte(0),
        ProjectionField::FinalStateHash => writer.byte(1),
        ProjectionField::EventDigest => writer.byte(2),
        ProjectionField::TerminalFault => writer.byte(3),
        ProjectionField::ParticipantState(participant) => {
            writer.byte(5);
            writer.u32(participant.get());
        }
        ProjectionField::Metric { key, kind } => {
            writer.byte(4);
            writer.text(key);
            writer.byte(*kind as u8);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleResultProjectionError {
    InvalidFieldCount,
    MissingOrDuplicateCoreField,
    InvalidMetricKey,
    DuplicateMetricKey,
    DuplicateParticipant,
}
