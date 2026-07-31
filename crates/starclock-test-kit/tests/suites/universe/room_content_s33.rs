use std::sync::{Arc, OnceLock};

use sha2::{Digest, Sha256};
use starclock_mode_universe::{catalog::UniverseCatalog, encounter::RoomContentKind};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] =
    include_bytes!("../../../../../config/universe-generated/config.sora");
const STABLE_KEYS: [&str; 32] = [
    "universe.room.300221",
    "universe.room.300222",
    "universe.room.300223",
    "universe.room.300231",
    "universe.room.300232",
    "universe.room.300233",
    "universe.room.300241",
    "universe.room.300242",
    "universe.room.300243",
    "universe.room.300311",
    "universe.room.300312",
    "universe.room.300313",
    "universe.room.300321",
    "universe.room.300322",
    "universe.room.300323",
    "universe.room.300331",
    "universe.room.300332",
    "universe.room.300333",
    "universe.room.300412",
    "universe.room.300413",
    "universe.room.300422",
    "universe.room.300423",
    "universe.room.300432",
    "universe.room.300433",
    "universe.room.300511",
    "universe.room.300512",
    "universe.room.300513",
    "universe.room.300611",
    "universe.room.300612",
    "universe.room.300713",
    "universe.room.300813",
    "universe.room.300823",
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
fn goal07_p5_m15_s33_materializes_exact_rooms_and_content_bindings() {
    let selected = catalog()
        .rooms()
        .iter()
        .filter(|room| (97..=128).contains(&room.id().get()))
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 32);
    assert!(selected.iter().map(|room| room.id().get()).eq(97_u32..=128));
    assert!(
        selected
            .iter()
            .map(|room| room.stable_key())
            .eq(STABLE_KEYS)
    );

    let room_content = catalog()
        .room_content()
        .iter()
        .filter(|binding| (97..=128).contains(&binding.room().get()))
        .collect::<Vec<_>>();
    assert_eq!(room_content.len(), 38);
    assert_eq!(
        room_content
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::EncounterGroup)
            .count(),
        18
    );
    assert_eq!(
        room_content
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::FixedContent)
            .count(),
        0
    );
    assert_eq!(
        room_content
            .iter()
            .filter(|binding| binding.kind() == RoomContentKind::ExternalDecision)
            .count(),
        20
    );

    let mut hasher = Sha256::new();
    text(&mut hasher, "starclock-goal07-p5-m15-s33-room-content-v1");
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
            225, 184, 96, 200, 180, 233, 101, 154, 114, 51, 252, 178, 211, 109, 117, 203, 69, 93,
            122, 110, 226, 221, 151, 43, 222, 47, 68, 121, 114, 86, 243, 121,
        ]
    );
}

#[test]
fn goal07_p5_m15_s33_resolves_one_primary_binding_and_exact_group_contracts() {
    for room in catalog()
        .rooms()
        .iter()
        .filter(|room| (97..=128).contains(&room.id().get()))
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
