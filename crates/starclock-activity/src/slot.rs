use crate::{
    ActivityScope, ActivitySlotId, ActivityStateDefinitionError, ActivityStateSource,
    ActivityStateVisibility, SlotCarryPolicy, codec::CanonicalWriter,
    state_definition::validate_policy,
};

pub const MAX_SLOT_COLLECTION_ENTRIES: u32 = 4_096;

/// Closed value domains accepted by Goal 01 activity slots.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum SlotValueKind {
    BoundedInteger = 0,
    FixedScalar = 1,
    Boolean = 2,
    StableId = 3,
    OptionalId = 4,
    OrderedIdSet = 5,
    BoundedCounterMap = 6,
}

/// Typed activity-owned value. Fixed scalars use signed millionths.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivityValue {
    BoundedInteger(i64),
    FixedScalar(i64),
    Boolean(bool),
    StableId(u64),
    OptionalId(Option<u64>),
    OrderedIdSet(Box<[u64]>),
    /// Canonically sorted stable-ID keys with definition-bounded integer values.
    BoundedCounterMap(Box<[(u64, i64)]>),
}

impl ActivityValue {
    #[must_use]
    pub const fn kind(&self) -> SlotValueKind {
        match self {
            Self::BoundedInteger(_) => SlotValueKind::BoundedInteger,
            Self::FixedScalar(_) => SlotValueKind::FixedScalar,
            Self::Boolean(_) => SlotValueKind::Boolean,
            Self::StableId(_) => SlotValueKind::StableId,
            Self::OptionalId(_) => SlotValueKind::OptionalId,
            Self::OrderedIdSet(_) => SlotValueKind::OrderedIdSet,
            Self::BoundedCounterMap(_) => SlotValueKind::BoundedCounterMap,
        }
    }

    fn structurally_valid(&self) -> bool {
        match self {
            Self::StableId(value) => *value != 0,
            Self::OptionalId(Some(value)) => *value != 0,
            Self::OrderedIdSet(values) => {
                values.iter().all(|value| *value != 0)
                    && values.windows(2).all(|pair| pair[0] < pair[1])
            }
            Self::BoundedCounterMap(values) => {
                values.iter().all(|(key, _)| *key != 0)
                    && values.windows(2).all(|pair| pair[0].0 < pair[1].0)
            }
            _ => true,
        }
    }

    pub(crate) fn encode(&self, writer: &mut CanonicalWriter) {
        writer.byte(self.kind() as u8);
        match self {
            Self::BoundedInteger(value) | Self::FixedScalar(value) => writer.i64(*value),
            Self::Boolean(value) => writer.bool(*value),
            Self::StableId(value) => writer.u64(*value),
            Self::OptionalId(value) => {
                writer.bool(value.is_some());
                if let Some(value) = value {
                    writer.u64(*value);
                }
            }
            Self::OrderedIdSet(values) => {
                writer.u64(values.len() as u64);
                for value in values {
                    writer.u64(*value);
                }
            }
            Self::BoundedCounterMap(values) => {
                writer.u64(values.len() as u64);
                for (key, value) in values {
                    writer.u64(*key);
                    writer.i64(*value);
                }
            }
        }
    }
}

/// Reset boundaries visible to the cross-battle aggregate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum SlotResetPoint {
    ActivityStart = 0,
    SectionStart = 1,
    NodeStart = 2,
    AttemptStart = 3,
    BattleStart = 4,
    BattleEnd = 5,
}

impl SlotResetPoint {
    const fn scope(self) -> ActivityScope {
        match self {
            Self::ActivityStart => ActivityScope::Activity,
            Self::SectionStart => ActivityScope::Section,
            Self::NodeStart => ActivityScope::Node,
            Self::AttemptStart => ActivityScope::Attempt,
            Self::BattleStart | Self::BattleEnd => ActivityScope::Attempt,
        }
    }
}

/// Immutable type/bounds/reset contract for one activity state slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivitySlotDefinition {
    id: ActivitySlotId,
    owner: ActivityScope,
    initial: ActivityValue,
    minimum: Option<i64>,
    maximum: Option<i64>,
    resets: Box<[SlotResetPoint]>,
    maximum_entries: Option<u32>,
    carry: SlotCarryPolicy,
    visibility: ActivityStateVisibility,
    source: Option<ActivityStateSource>,
}

impl ActivitySlotDefinition {
    pub fn new(
        id: ActivitySlotId,
        owner: ActivityScope,
        initial: ActivityValue,
        bounds: Option<(i64, i64)>,
        resets: Vec<SlotResetPoint>,
    ) -> Result<Self, SlotDefinitionError> {
        let maximum_entries = match &initial {
            ActivityValue::OrderedIdSet(_) | ActivityValue::BoundedCounterMap(_) => {
                Some(MAX_SLOT_COLLECTION_ENTRIES)
            }
            _ => None,
        };
        Self::new_internal(
            id,
            owner,
            initial,
            bounds,
            maximum_entries,
            resets,
            SlotCarryPolicy::Reset,
            ActivityStateVisibility::Private,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_policy(
        id: ActivitySlotId,
        owner: ActivityScope,
        initial: ActivityValue,
        bounds: Option<(i64, i64)>,
        maximum_entries: Option<u32>,
        resets: Vec<SlotResetPoint>,
        carry: SlotCarryPolicy,
        visibility: ActivityStateVisibility,
        source: ActivityStateSource,
    ) -> Result<Self, SlotDefinitionError> {
        Self::new_internal(
            id,
            owner,
            initial,
            bounds,
            maximum_entries,
            resets,
            carry,
            visibility,
            Some(source),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_internal(
        id: ActivitySlotId,
        owner: ActivityScope,
        initial: ActivityValue,
        bounds: Option<(i64, i64)>,
        maximum_entries: Option<u32>,
        resets: Vec<SlotResetPoint>,
        carry: SlotCarryPolicy,
        visibility: ActivityStateVisibility,
        source: Option<ActivityStateSource>,
    ) -> Result<Self, SlotDefinitionError> {
        if !initial.structurally_valid() {
            return Err(SlotDefinitionError::InvalidInitialValue);
        }
        let (minimum, maximum) = match bounds {
            Some((minimum, maximum)) => {
                if !matches!(
                    &initial,
                    ActivityValue::BoundedInteger(_)
                        | ActivityValue::FixedScalar(_)
                        | ActivityValue::BoundedCounterMap(_)
                ) {
                    return Err(SlotDefinitionError::BoundsForUnboundedKind);
                }
                if minimum > maximum {
                    return Err(SlotDefinitionError::InvalidBounds);
                }
                let outside = match &initial {
                    ActivityValue::BoundedInteger(value) | ActivityValue::FixedScalar(value) => {
                        *value < minimum || *value > maximum
                    }
                    ActivityValue::BoundedCounterMap(values) => values
                        .iter()
                        .any(|(_, value)| *value < minimum || *value > maximum),
                    _ => unreachable!("kind checked above"),
                };
                if outside {
                    return Err(SlotDefinitionError::InitialOutsideBounds);
                }
                (Some(minimum), Some(maximum))
            }
            None => {
                if matches!(
                    &initial,
                    ActivityValue::BoundedInteger(_) | ActivityValue::BoundedCounterMap(_)
                ) {
                    return Err(SlotDefinitionError::MissingIntegerBounds);
                }
                (None, None)
            }
        };
        if resets.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(SlotDefinitionError::NonCanonicalResets);
        }
        if resets.iter().any(|point| point.scope() < owner) {
            return Err(SlotDefinitionError::ResetBeforeOwnerLifetime);
        }
        let collection_length = match &initial {
            ActivityValue::OrderedIdSet(values) => Some(values.len()),
            ActivityValue::BoundedCounterMap(values) => Some(values.len()),
            _ => None,
        };
        match (collection_length, maximum_entries) {
            (Some(length), Some(limit))
                if limit > 0
                    && limit <= MAX_SLOT_COLLECTION_ENTRIES
                    && length <= limit as usize => {}
            (Some(_), _) => return Err(SlotDefinitionError::InvalidCollectionLimit),
            (None, Some(_)) => return Err(SlotDefinitionError::CollectionLimitForScalar),
            (None, None) => {}
        }
        validate_policy(owner, carry).map_err(|error| match error {
            ActivityStateDefinitionError::SnapshotBeforeOwnerExit => {
                SlotDefinitionError::SnapshotBeforeOwnerExit
            }
            _ => unreachable!("slot policy validation returns one local error"),
        })?;
        Ok(Self {
            id,
            owner,
            initial,
            minimum,
            maximum,
            resets: resets.into_boxed_slice(),
            maximum_entries,
            carry,
            visibility,
            source,
        })
    }

    #[must_use]
    pub const fn id(&self) -> ActivitySlotId {
        self.id
    }
    #[must_use]
    pub const fn owner(&self) -> ActivityScope {
        self.owner
    }
    #[must_use]
    pub const fn kind(&self) -> SlotValueKind {
        self.initial.kind()
    }
    #[must_use]
    pub const fn initial(&self) -> &ActivityValue {
        &self.initial
    }
    #[must_use]
    pub fn resets(&self) -> &[SlotResetPoint] {
        &self.resets
    }
    #[must_use]
    pub const fn maximum_entries(&self) -> Option<u32> {
        self.maximum_entries
    }
    #[must_use]
    pub const fn carry(&self) -> SlotCarryPolicy {
        self.carry
    }
    #[must_use]
    pub const fn visibility(&self) -> ActivityStateVisibility {
        self.visibility
    }
    #[must_use]
    pub const fn source(&self) -> Option<ActivityStateSource> {
        self.source
    }

    pub(crate) fn encode(&self, writer: &mut CanonicalWriter) {
        writer.u32(self.id.get());
        writer.byte(self.owner as u8);
        self.initial.encode(writer);
        writer.bool(self.minimum.is_some());
        if let Some(value) = self.minimum {
            writer.i64(value);
        }
        if let Some(value) = self.maximum {
            writer.i64(value);
        }
        writer.u64(self.resets.len() as u64);
        for reset in &self.resets {
            writer.byte(*reset as u8);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotDefinitionError {
    InvalidInitialValue,
    BoundsForUnboundedKind,
    MissingIntegerBounds,
    InvalidBounds,
    InitialOutsideBounds,
    NonCanonicalResets,
    ResetBeforeOwnerLifetime,
    InvalidCollectionLimit,
    CollectionLimitForScalar,
    SnapshotBeforeOwnerExit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScopedSlots {
    definitions: Box<[ActivitySlotDefinition]>,
    values: Box<[ActivityValue]>,
}

impl ScopedSlots {
    pub(crate) fn new(
        mut definitions: Vec<ActivitySlotDefinition>,
    ) -> Result<Self, ActivitySlotId> {
        definitions.sort_by_key(ActivitySlotDefinition::id);
        if let Some(pair) = definitions.windows(2).find(|pair| pair[0].id == pair[1].id) {
            return Err(pair[0].id);
        }
        let values = definitions
            .iter()
            .map(|definition| definition.initial.clone())
            .collect();
        Ok(Self {
            definitions: definitions.into_boxed_slice(),
            values,
        })
    }

    pub(crate) fn value(&self, id: ActivitySlotId) -> Option<&ActivityValue> {
        self.definitions
            .binary_search_by_key(&id, ActivitySlotDefinition::id)
            .ok()
            .map(|index| &self.values[index])
    }

    pub(crate) fn reset(&mut self, point: SlotResetPoint) -> Vec<ActivitySlotId> {
        let mut reset = Vec::new();
        for (definition, value) in self.definitions.iter().zip(self.values.iter_mut()) {
            if definition.resets.contains(&point) {
                *value = definition.initial.clone();
                reset.push(definition.id);
            }
        }
        reset
    }

    pub(crate) fn encode(&self, writer: &mut CanonicalWriter) {
        writer.u64(self.definitions.len() as u64);
        for (definition, value) in self.definitions.iter().zip(&self.values) {
            definition.encode(writer);
            value.encode(writer);
        }
    }
}
