use std::sync::{Arc, OnceLock};

use sha2::{Digest, Sha256};
use starclock_mode_universe::{catalog::UniverseCatalog, encounter::RoomContentKind};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

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
fn goal07_p5_m15_s30_materializes_exact_rooms_and_content_bindings() {
    let selected = catalog()
        .rooms()
        .iter()
        .filter(|room| (1..=32).contains(&room.id().get()))
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 32);
    assert!(selected.iter().map(|room| room.id().get()).eq(1_u32..=32));
    assert!(
        selected
            .iter()
            .map(|room| room.stable_key())
            .eq(std::iter::once("universe.room.100").chain(
                (1..=31)
                    .map(|value| format!("universe.room.10000{value:02}"))
                    .collect::<Vec<_>>()
                    .iter()
                    .map(String::as_str),
            ))
    );

    let room_content = catalog()
        .room_content()
        .iter()
        .filter(|binding| (1..=32).contains(&binding.room().get()))
        .collect::<Vec<_>>();
    assert_eq!(room_content.len(), 114);
    assert_eq!(
        room_content
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::EncounterGroup)
            .count(),
        31
    );
    assert_eq!(
        room_content
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::FixedContent)
            .count(),
        39
    );
    assert_eq!(
        room_content
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::ExternalDecision)
            .count(),
        44
    );

    let mut hasher = Sha256::new();
    text(&mut hasher, "starclock-goal07-p5-m15-s30-room-content-v1");
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
            52, 122, 255, 13, 240, 245, 152, 222, 205, 251, 109, 150, 12, 244, 178, 195, 203, 217,
            84, 221, 208, 209, 98, 142, 170, 79, 59, 198, 211, 6, 43, 191,
        ]
    );
}

#[test]
fn goal07_p5_m15_s30_resolves_one_primary_binding_and_exact_group_contracts() {
    for room in catalog()
        .rooms()
        .iter()
        .filter(|room| (1..=32).contains(&room.id().get()))
    {
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
