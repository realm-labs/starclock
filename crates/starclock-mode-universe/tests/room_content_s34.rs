use std::sync::{Arc, OnceLock};

use sha2::{Digest, Sha256};
use starclock_mode_universe::{catalog::UniverseCatalog, encounter::RoomContentKind};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");
const STABLE_KEYS: [&str; 32] = [
    "universe.room.300833",
    "universe.room.301",
    "universe.room.302",
    "universe.room.303",
    "universe.room.304",
    "universe.room.305",
    "universe.room.306",
    "universe.room.307",
    "universe.room.400111",
    "universe.room.400112",
    "universe.room.400121",
    "universe.room.400122",
    "universe.room.400131",
    "universe.room.400132",
    "universe.room.400142",
    "universe.room.400211",
    "universe.room.400212",
    "universe.room.400221",
    "universe.room.400222",
    "universe.room.400231",
    "universe.room.400232",
    "universe.room.400311",
    "universe.room.400312",
    "universe.room.400321",
    "universe.room.400322",
    "universe.room.400331",
    "universe.room.400332",
    "universe.room.400412",
    "universe.room.400422",
    "universe.room.400432",
    "universe.room.400511",
    "universe.room.400512",
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
fn goal07_p5_m15_s34_materializes_exact_rooms_and_content_bindings() {
    let selected = catalog()
        .rooms()
        .iter()
        .filter(|room| (129..=160).contains(&room.id().get()))
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 32);
    assert!(
        selected
            .iter()
            .map(|room| room.id().get())
            .eq(129_u32..=160)
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
        .filter(|binding| (129..=160).contains(&binding.room().get()))
        .collect::<Vec<_>>();
    assert_eq!(room_content.len(), 101);
    assert_eq!(
        room_content
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::EncounterGroup)
            .count(),
        37
    );
    assert_eq!(
        room_content
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::FixedContent)
            .count(),
        27
    );
    assert_eq!(
        room_content
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::ExternalDecision)
            .count(),
        37
    );

    let mut hasher = Sha256::new();
    text(&mut hasher, "starclock-goal07-p5-m15-s34-room-content-v1");
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
            163, 196, 149, 225, 162, 79, 104, 228, 193, 142, 171, 166, 77, 101, 41, 97, 41, 208,
            244, 229, 112, 61, 92, 19, 254, 33, 112, 140, 125, 7, 69, 96,
        ]
    );
}

#[test]
fn goal07_p5_m15_s34_resolves_one_primary_binding_and_exact_group_contracts() {
    for room in catalog()
        .rooms()
        .iter()
        .filter(|room| (129..=160).contains(&room.id().get()))
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
