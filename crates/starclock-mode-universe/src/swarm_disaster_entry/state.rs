use starclock_activity::{
    ActivityScope, ActivitySlotDefinition, ActivitySlotId, ActivityStateDefinition,
    ActivityStateSource, ActivityStateVisibility, ActivityValue, SlotCarryPolicy, SlotResetPoint,
};

use crate::{
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_structural::SwarmDisasterEntryArea,
    swarm_disaster_unique::entry_access::SwarmDisasterEntrySelection,
};

const ENTRY: u32 = 0x5344_0001;
pub(super) const AUDIENCE_DIE: u32 = 0x5344_0002;
const COMMUNING: u32 = 0x5344_0003;
const PROGRESSION: u32 = 0x5344_0004;
const RESOURCES: u32 = 0x5344_0005;
pub(super) const CONTENT: u32 = 0x5344_0006;
const DEFERRED: u32 = 0x5344_0007;
pub(super) const COUNTDOWN: u32 = 0x5344_0008;
pub(super) const DISARRAY: u32 = 0x5344_0009;
pub(super) const PLANE: u32 = 0x5344_000A;
pub(super) const NODE_STATE: u32 = 0x5344_000B;
pub(super) const NODE_DOMAIN: u32 = 0x5344_000C;
pub(super) const NODE_BEACON: u32 = 0x5344_000D;
const NODE_VISIT: u32 = 0x5344_000E;
const DICE_RESOLUTION: u32 = 0x5344_000F;
const COMMUNING_CHOICE: u32 = 0x5344_0010;

pub(super) struct SwarmStateCompileInput<'a> {
    pub(super) area: SwarmDisasterEntryArea,
    pub(super) selection: SwarmDisasterEntrySelection,
    pub(super) communing: &'a [(u32, u16)],
    pub(super) progression: &'a [u64],
    pub(super) bonus: Option<u32>,
    pub(super) countdown: i64,
    pub(super) currency: i64,
    pub(super) audience_state: &'a [(u64, i64)],
}

pub(super) fn compile(
    input: SwarmStateCompileInput<'_>,
) -> Result<ActivityStateDefinition, UniverseCatalogLoadError> {
    let SwarmStateCompileInput {
        area,
        selection,
        communing,
        progression,
        bonus,
        countdown,
        currency,
        audience_state,
    } = input;
    let entry = vec![
        (1, i64::from(area.id)),
        (2, i64::from(area.difficulty)),
        (3, i64::from(selection.path_id)),
        (4, i64::from(selection.audience_die_id)),
    ];
    let communing = communing
        .iter()
        .map(|(id, value)| (u64::from(*id), i64::from(*value)))
        .collect();
    let mut progression = progression.iter().map(|key| (*key, 1)).collect::<Vec<_>>();
    if let Some(bonus) = bonus {
        progression.push((1, i64::from(bonus)));
        progression.sort_unstable_by_key(|(key, _)| *key);
    }
    let slots = vec![
        map(
            ENTRY,
            ActivityScope::Activity,
            entry,
            4,
            0,
            i64::from(u32::MAX),
            false,
        )?,
        map(
            AUDIENCE_DIE,
            ActivityScope::Activity,
            audience_state.to_vec(),
            64,
            0,
            i64::from(u32::MAX),
            false,
        )?,
        map(
            COMMUNING,
            ActivityScope::Activity,
            communing,
            7,
            0,
            20,
            false,
        )?,
        map(
            PROGRESSION,
            ActivityScope::Activity,
            progression,
            256,
            0,
            i64::from(u32::MAX),
            false,
        )?,
        map(
            RESOURCES,
            ActivityScope::Activity,
            vec![(1, currency)],
            16,
            0,
            1_000_000_000,
            false,
        )?,
        map(
            CONTENT,
            ActivityScope::Activity,
            vec![],
            256,
            0,
            i64::MAX,
            false,
        )?,
        map(
            DEFERRED,
            ActivityScope::Activity,
            vec![],
            4_096,
            0,
            i64::MAX,
            true,
        )?,
        integer(COUNTDOWN, countdown, -1_000_000, 1_000_000)?,
        map(
            DISARRAY,
            ActivityScope::Activity,
            vec![],
            64,
            0,
            i64::MAX,
            false,
        )?,
        map(
            PLANE,
            ActivityScope::Section,
            vec![],
            16,
            0,
            i64::MAX,
            false,
        )?,
        map(
            NODE_STATE,
            ActivityScope::Section,
            vec![],
            1_991,
            0,
            i64::MAX,
            false,
        )?,
        map(
            NODE_DOMAIN,
            ActivityScope::Section,
            vec![],
            1_991,
            0,
            i64::MAX,
            false,
        )?,
        map(
            NODE_BEACON,
            ActivityScope::Section,
            vec![],
            1_991,
            0,
            i64::MAX,
            false,
        )?,
        map(
            NODE_VISIT,
            ActivityScope::Node,
            vec![],
            32,
            0,
            i64::MAX,
            true,
        )?,
        map(
            DICE_RESOLUTION,
            ActivityScope::Attempt,
            vec![],
            128,
            0,
            i64::MAX,
            false,
        )?,
        map(
            COMMUNING_CHOICE,
            ActivityScope::Attempt,
            vec![],
            64,
            0,
            i64::MAX,
            false,
        )?,
    ];
    ActivityStateDefinition::new(slots, vec![], vec![]).map_err(|_| invalid_state())
}

#[allow(clippy::too_many_arguments)]
fn map(
    id: u32,
    owner: ActivityScope,
    initial: Vec<(u64, i64)>,
    maximum_entries: u32,
    minimum: i64,
    maximum: i64,
    debug: bool,
) -> Result<ActivitySlotDefinition, UniverseCatalogLoadError> {
    slot(
        id,
        owner,
        ActivityValue::BoundedCounterMap(initial.into_boxed_slice()),
        Some((minimum, maximum)),
        Some(maximum_entries),
        debug,
    )
}

fn integer(
    id: u32,
    initial: i64,
    minimum: i64,
    maximum: i64,
) -> Result<ActivitySlotDefinition, UniverseCatalogLoadError> {
    slot(
        id,
        ActivityScope::Activity,
        ActivityValue::BoundedInteger(initial),
        Some((minimum, maximum)),
        None,
        false,
    )
}

fn slot(
    id: u32,
    owner: ActivityScope,
    initial: ActivityValue,
    bounds: Option<(i64, i64)>,
    maximum_entries: Option<u32>,
    debug: bool,
) -> Result<ActivitySlotDefinition, UniverseCatalogLoadError> {
    let (reset, carry) = match owner {
        ActivityScope::Activity => (SlotResetPoint::ActivityStart, SlotCarryPolicy::CarryExact),
        ActivityScope::Section => (SlotResetPoint::SectionStart, SlotCarryPolicy::Reset),
        ActivityScope::Node => (SlotResetPoint::NodeStart, SlotCarryPolicy::Reset),
        ActivityScope::Attempt => (SlotResetPoint::AttemptStart, SlotCarryPolicy::Reset),
    };
    ActivitySlotDefinition::new_with_policy(
        ActivitySlotId::new(id).expect("static Swarm slot ID is non-zero"),
        owner,
        initial,
        bounds,
        maximum_entries,
        vec![reset],
        carry,
        if debug {
            ActivityStateVisibility::DebugOnly
        } else {
            ActivityStateVisibility::Player
        },
        ActivityStateSource::new(0x5344_1400_0000_0000 + u64::from(id))
            .expect("static Swarm source is non-zero"),
    )
    .map_err(|_| invalid_state())
}

fn invalid_state() -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(
        UniverseCatalogLoadErrorKind::InvalidDefinition,
        "invalid Swarm Activity state profile",
    )
}
