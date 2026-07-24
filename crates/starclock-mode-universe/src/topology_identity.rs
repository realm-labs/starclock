//! Stable option identities for the Standard Universe topology compiler.

use starclock_activity::ActivityOptionId;

use crate::id::{BlessingId, EncounterMemberId, ResonanceId, RoomId};

const PATH_OPTION_OFFSET: u64 = 1_000_000;
const TOPOLOGY_OPTION_OFFSET: u64 = 2_000_000;
const ROOM_OPTION_OFFSET: u64 = 1_000_000_000_000;
const CONTENT_OPTION_OFFSET: u64 = 2_000_000_000_000;
const MEMBER_OPTION_OFFSET: u64 = 3_000_000_000_000;
const ENGAGE_OPTION_OFFSET: u64 = 4_000_000_000_000;
const INTERACTION_OPTION_OFFSET: u64 = 4_500_000_000_000;
const SERVICE_INTERACTION_OPTION_OFFSET: u64 = 4_600_000_000_000;
const REWARD_OPTION_OFFSET: u64 = 5_000_000_000_000;
const FORMATION_OPTION_OFFSET: u64 = 5_500_000_000_000;
const FORMATION_SKIP_OPTION_OFFSET: u64 = 5_900_000_000_000;
const ROUTE_OPTION_OFFSET: u64 = 6_000_000_000_000;
const EXIT_OPTION_OFFSET: u64 = 7_000_000_000_000;

pub(super) fn path_option(path: u32) -> ActivityOptionId {
    option(PATH_OPTION_OFFSET + u64::from(path))
}

pub(super) fn topology_option(topology: u32) -> ActivityOptionId {
    option(TOPOLOGY_OPTION_OFFSET + u64::from(topology))
}

pub(super) fn room_option(source: u64, room: RoomId) -> ActivityOptionId {
    option(ROOM_OPTION_OFFSET + source * 1_000 + u64::from(room.get()))
}

pub(super) fn content_option(source: u64, room: RoomId) -> ActivityOptionId {
    option(CONTENT_OPTION_OFFSET + source * 1_000 + u64::from(room.get()))
}

pub(super) fn member_option(
    source: u64,
    room: RoomId,
    member: EncounterMemberId,
) -> ActivityOptionId {
    option(
        MEMBER_OPTION_OFFSET
            + source * 1_000_000
            + u64::from(room.get()) * 1_000
            + u64::from(member.get()),
    )
}

pub(super) fn engage_option(
    source: u64,
    room: RoomId,
    member: EncounterMemberId,
) -> ActivityOptionId {
    option(
        ENGAGE_OPTION_OFFSET
            + source * 1_000_000
            + u64::from(room.get()) * 1_000
            + u64::from(member.get()),
    )
}

pub(super) fn interaction_option(source: u64, room: RoomId) -> ActivityOptionId {
    option(INTERACTION_OPTION_OFFSET + source * 10_000_000 + u64::from(room.get()))
}

pub(super) fn service_interaction_option(
    source: u64,
    room: RoomId,
    selection: u32,
) -> ActivityOptionId {
    option(
        SERVICE_INTERACTION_OPTION_OFFSET
            + source * 10_000_000
            + u64::from(room.get()) * 100
            + u64::from(selection),
    )
}

pub(super) fn occurrence_choice_option(source: u64, room: RoomId, choice: u32) -> ActivityOptionId {
    option(
        INTERACTION_OPTION_OFFSET
            + source * 10_000_000
            + u64::from(room.get()) * 1_000
            + u64::from(choice),
    )
}

pub(super) fn blessing_option(source: u64, blessing: BlessingId) -> ActivityOptionId {
    option(REWARD_OPTION_OFFSET + source * 1_000_000 + u64::from(blessing.get()))
}

pub(super) fn formation_option(source: u64, formation: ResonanceId) -> ActivityOptionId {
    option(FORMATION_OPTION_OFFSET + source * 1_000_000 + u64::from(formation.get()))
}

pub(super) fn formation_skip_option(source: u64) -> ActivityOptionId {
    option(FORMATION_SKIP_OPTION_OFFSET + source)
}

pub(super) fn route_option(edge: u32) -> ActivityOptionId {
    option(ROUTE_OPTION_OFFSET + u64::from(edge))
}

pub(super) fn exit_option(source: u32) -> ActivityOptionId {
    option(EXIT_OPTION_OFFSET + u64::from(source))
}

fn option(raw: u64) -> ActivityOptionId {
    ActivityOptionId::new(raw).expect("derived option ID is non-zero")
}
