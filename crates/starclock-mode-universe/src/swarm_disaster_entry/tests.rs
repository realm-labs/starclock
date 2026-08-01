use starclock_activity::{
    ActivityValue, BuildDigest, LoadoutLockScope, OpaqueParticipantBuild, ParticipantId,
    ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};

use crate::error::UniverseCatalogLoadErrorKind;

use super::*;

const BUNDLE: &[u8] = include_bytes!("../../../../config/swarm-disaster-generated/config.sora");

#[test]
fn compiles_entry_into_exact_sixteen_slot_activity_profile() {
    let factory = SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let entry = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        participants(policy()),
    )
    .with_progression(
        vec![("swarm-disaster.communing-dimension.1".into(), 3)],
        vec!["swarm-disaster.communing-trail.101".into()],
        Some("swarm-disaster.trailblaze-bonus.101".into()),
    );
    let instance = factory.compile_entry(entry).unwrap();
    assert_eq!(instance.area(), "swarm-disaster.area.201");
    assert_eq!(instance.difficulty(), 1);
    assert_eq!(instance.path(), "universe.path.preservation");
    assert_eq!(instance.audience_die(), "swarm-disaster.audience-die.1");
    assert_eq!(instance.state_definition().slots().len(), 16);
    assert_eq!(
        instance.participants().digest(),
        participants(policy()).digest()
    );
    assert_eq!(
        slot_value(&instance, 0x5344_0008),
        &ActivityValue::BoundedInteger(20)
    );
    assert_eq!(
        slot_value(&instance, 0x5344_0005),
        &ActivityValue::BoundedCounterMap(vec![(1, 50)].into_boxed_slice())
    );
}

#[test]
fn compiles_all_five_difficulties_and_eight_path_die_pairs() {
    let factory = SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let pairs = [
        (
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
        ),
        ("universe.path.remembrance", "swarm-disaster.audience-die.2"),
        ("universe.path.nihility", "swarm-disaster.audience-die.3"),
        ("universe.path.abundance", "swarm-disaster.audience-die.4"),
        ("universe.path.hunt", "swarm-disaster.audience-die.5"),
        ("universe.path.destruction", "swarm-disaster.audience-die.6"),
        ("universe.path.elation", "swarm-disaster.audience-die.7"),
        ("universe.path.propagation", "swarm-disaster.audience-die.8"),
    ];
    for difficulty in 1_u8..=5 {
        for (path, die) in pairs {
            let instance = factory
                .compile_entry(SwarmDisasterEntry::new(
                    format!("swarm-disaster.area.20{difficulty}"),
                    path,
                    die,
                    participants(policy()),
                ))
                .unwrap();
            assert_eq!(instance.difficulty(), difficulty);
            assert_eq!(instance.path(), path);
            assert_eq!(instance.audience_die(), die);
        }
    }
}

#[test]
fn rejects_mismatched_die_duplicate_progression_and_participant_policy() {
    let factory = SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let mismatch = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.2",
        participants(policy()),
    );
    assert_eq!(
        factory.compile_entry(mismatch).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidReference
    );
    let duplicate = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        participants(policy()),
    )
    .with_progression(
        vec![
            ("swarm-disaster.communing-dimension.1".into(), 1),
            ("swarm-disaster.communing-dimension.1".into(), 2),
        ],
        vec![],
        None,
    );
    assert_eq!(
        factory.compile_entry(duplicate).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidDefinition
    );
    let wrong_policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Node,
        LoadoutLockScope::Activity,
    )
    .unwrap();
    let invalid = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        participants(wrong_policy),
    );
    assert_eq!(
        factory.compile_entry(invalid).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidDefinition
    );
    let guide_area = SwarmDisasterEntry::new(
        "swarm-disaster.area.101",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        participants(policy()),
    );
    assert_eq!(
        factory.compile_entry(guide_area).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidReference
    );
    let overflow = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        participants(policy()),
    )
    .with_progression(
        vec![("swarm-disaster.communing-dimension.1".into(), u16::MAX)],
        vec![],
        None,
    );
    assert_eq!(
        factory.compile_entry(overflow).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidDefinition
    );
    let unknown_bonus = SwarmDisasterEntry::new(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        participants(policy()),
    )
    .with_progression(
        vec![],
        vec![],
        Some("swarm-disaster.trailblaze-bonus.999".into()),
    );
    assert_eq!(
        factory.compile_entry(unknown_bonus).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidReference
    );
}

fn policy() -> ParticipantPolicy {
    ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .unwrap()
}

fn participants(policy: ParticipantPolicy) -> ParticipantLock {
    let entries = (0_u8..4)
        .map(|index| {
            let byte = index + 1;
            let build = OpaqueParticipantBuild::new(
                CombatantSpecDigest::new([byte; 32]).unwrap(),
                BuildDigest::new([byte + 32; 32]).unwrap(),
                "swarm-entry-test-build-v1",
                ParticipantSourceKind::CompiledBuild,
            )
            .unwrap();
            ParticipantLockEntry::new(
                ParticipantId::new(u32::from(index) + 1).unwrap(),
                0,
                index,
                UnitDefinitionId::new(20_001 + u32::from(index)).unwrap(),
                build,
            )
            .unwrap()
        })
        .collect();
    ParticipantLock::seal(policy, entries).unwrap()
}

fn slot_value(instance: &SwarmDisasterRuntimeInstance, id: u32) -> &ActivityValue {
    instance
        .state_definition()
        .slots()
        .iter()
        .find(|slot| slot.id().get() == id)
        .unwrap()
        .initial()
}
