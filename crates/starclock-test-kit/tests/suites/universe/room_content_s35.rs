use std::sync::{Arc, OnceLock};

use sha2::{Digest, Sha256};
use starclock_mode_universe::{catalog::UniverseCatalog, encounter::RoomContentKind};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] =
    include_bytes!("../../../../../config/universe-generated/config.sora");
const STABLE_KEYS: [&str; 3] = [
    "universe.room.400611",
    "universe.room.400612",
    "universe.room.501",
];

fn catalog() -> &'static UniverseCatalog {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    CATALOG
        .get_or_init(|| {
            let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
            UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog")
        })
        .as_ref()
}

fn text(hasher: &mut Sha256, value: &str) {
    hasher.update(u64::try_from(value.len()).unwrap().to_le_bytes());
    hasher.update(value.as_bytes());
}

#[test]
fn goal07_p5_m15_s35_materializes_exact_rooms_and_content_bindings() {
    let selected = catalog()
        .rooms()
        .iter()
        .filter(|room| (161..=163).contains(&room.id().get()))
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 3);
    assert!(
        selected
            .iter()
            .map(|room| room.id().get())
            .eq(161_u32..=163)
    );
    assert!(
        selected
            .iter()
            .map(|room| room.stable_key())
            .eq(STABLE_KEYS)
    );

    let room_content = catalog()
        .room_content()
        .iter()
        .filter(|binding| (161..=163).contains(&binding.room().get()))
        .collect::<Vec<_>>();
    assert_eq!(room_content.len(), 7);
    assert_eq!(
        room_content
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::EncounterGroup)
            .count(),
        3
    );
    assert_eq!(
        room_content
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::FixedContent)
            .count(),
        4
    );
    assert_eq!(
        room_content
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::ExternalDecision)
            .count(),
        0
    );

    let mut hasher = Sha256::new();
    text(&mut hasher, "starclock-goal07-p5-m15-s35-room-content-v1");
    for room in selected {
        hasher.update(room.id().get().to_le_bytes());
        text(&mut hasher, room.stable_key());
        hasher.update(room.domain().get().to_le_bytes());
        text(&mut hasher, room.source_room_id());
        text(&mut hasher, room.map_entrance());
        text(&mut hasher, room.source_group_id());
        hasher.update(
            u64::try_from(room.section_ids().len())
                .unwrap()
                .to_le_bytes(),
        );
        for section in room.section_ids() {
            hasher.update(section.to_le_bytes());
        }
        let bindings = room_content
            .iter()
            .filter(|binding| binding.room() == room.id())
            .collect::<Vec<_>>();
        hasher.update(u64::try_from(bindings.len()).unwrap().to_le_bytes());
        for binding in bindings {
            text(&mut hasher, binding.condition_key());
            text(&mut hasher, binding.source_content_id());
            hasher.update([binding.kind() as u8]);
            match binding.encounter_group() {
                Some(group) => {
                    hasher.update([1]);
                    hasher.update(group.get().to_le_bytes());
                }
                None => hasher.update([0]),
            }
        }
    }
    let digest: [u8; 32] = hasher.finalize().into();
    assert_eq!(
        digest,
        [
            15, 177, 129, 158, 254, 159, 78, 46, 132, 239, 47, 54, 113, 73, 97, 181, 140, 90, 243,
            52, 19, 138, 76, 71, 0, 13, 178, 49, 103, 254, 84, 181,
        ]
    );
}

#[test]
fn goal07_p5_m15_s35_closes_all_room_content_and_primary_group_contracts() {
    assert_eq!(catalog().rooms().len(), 163);
    assert_eq!(catalog().room_content().len(), 380);
    assert_eq!(
        catalog()
            .room_content()
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::EncounterGroup)
            .count(),
        174
    );
    assert_eq!(
        catalog()
            .room_content()
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::FixedContent)
            .count(),
        78
    );
    assert_eq!(
        catalog()
            .room_content()
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::ExternalDecision)
            .count(),
        128
    );

    for room in catalog().rooms() {
        assert!(catalog().domain(room.domain()).is_some());
        assert!(!room.section_ids().is_empty());
        let bindings = catalog()
            .room_content()
            .iter()
            .filter(|binding| binding.room() == room.id())
            .collect::<Vec<_>>();
        assert!(!bindings.is_empty());
        assert_eq!(
            bindings
                .iter()
                .filter(|binding| binding.condition_key() == room.source_group_id())
                .count(),
            1
        );
        for binding in bindings {
            match binding.kind() {
                RoomContentKind::EncounterGroup => {
                    let group = catalog()
                        .encounter_group(binding.encounter_group().expect("encounter group"))
                        .expect("resolved encounter group");
                    assert_eq!(group.source_group_id(), binding.source_content_id());
                }
                RoomContentKind::FixedContent | RoomContentKind::ExternalDecision => {
                    assert_eq!(binding.encounter_group(), None);
                }
            }
        }
    }
}
