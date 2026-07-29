use std::sync::{Arc, OnceLock};

use sha2::{Digest, Sha256};
use starclock_mode_universe::{catalog::UniverseCatalog, encounter::RoomContentKind};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");
const STABLE_KEYS: [&str; 32] = [
    "universe.room.1000032",
    "universe.room.1000033",
    "universe.room.1000034",
    "universe.room.1000035",
    "universe.room.200111",
    "universe.room.200112",
    "universe.room.200121",
    "universe.room.200122",
    "universe.room.200131",
    "universe.room.200132",
    "universe.room.200141",
    "universe.room.200142",
    "universe.room.200152",
    "universe.room.200211",
    "universe.room.200212",
    "universe.room.200213",
    "universe.room.200221",
    "universe.room.200222",
    "universe.room.200223",
    "universe.room.200231",
    "universe.room.200232",
    "universe.room.200233",
    "universe.room.200241",
    "universe.room.200242",
    "universe.room.200243",
    "universe.room.200311",
    "universe.room.200312",
    "universe.room.200313",
    "universe.room.200321",
    "universe.room.200322",
    "universe.room.200323",
    "universe.room.200331",
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
fn goal07_p5_m15_s31_materializes_exact_rooms_and_content_bindings() {
    let selected = catalog()
        .rooms()
        .iter()
        .filter(|room| (33..=64).contains(&room.id().get()))
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 32);
    assert!(selected.iter().map(|room| room.id().get()).eq(33_u32..=64));
    assert!(
        selected
            .iter()
            .map(|room| room.stable_key())
            .eq(STABLE_KEYS)
    );

    let room_content = catalog()
        .room_content()
        .iter()
        .filter(|binding| (33..=64).contains(&binding.room().get()))
        .collect::<Vec<_>>();
    assert_eq!(room_content.len(), 65);
    assert_eq!(
        room_content
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::EncounterGroup)
            .count(),
        46
    );
    assert_eq!(
        room_content
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::FixedContent)
            .count(),
        6
    );
    assert_eq!(
        room_content
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::ExternalDecision)
            .count(),
        13
    );

    let mut hasher = Sha256::new();
    text(&mut hasher, "starclock-goal07-p5-m15-s31-room-content-v1");
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
            53, 120, 49, 110, 243, 198, 89, 57, 65, 183, 200, 122, 85, 94, 101, 73, 211, 36, 239,
            237, 157, 200, 77, 239, 27, 94, 193, 187, 187, 243, 68, 169,
        ]
    );
}

#[test]
fn goal07_p5_m15_s31_resolves_one_primary_binding_and_exact_group_contracts() {
    for room in catalog()
        .rooms()
        .iter()
        .filter(|room| (33..=64).contains(&room.id().get()))
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
