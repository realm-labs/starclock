use starclock_replay::{
    component::{
        ComponentIdentityError, ConfigurationComponentIdentity, ConfigurationComponentKind,
        ConfigurationComponentSet,
    },
    digest::{ComponentDigest, EntrySpecDigest},
    entry::ReplayEntry,
    envelope::{ReplayEnvironment, ReplayError, ReplayHeader, decode_replay, encode_replay},
};

fn component(
    kind: ConfigurationComponentKind,
    id: &str,
    byte: u8,
) -> ConfigurationComponentIdentity {
    ConfigurationComponentIdentity::new(kind, id, ComponentDigest::new([byte; 32])).unwrap()
}

#[test]
fn replay_envelope_round_trips_and_rejects_unknown_records() {
    let header = ReplayHeader::new(
        ReplayEnvironment::new("4.4").unwrap(),
        component_set(0x44),
        42,
        ReplayEntry::Battle {
            definition_id: 7,
            spec_digest: EntrySpecDigest::new([0x77; 32]),
        },
        0,
    )
    .unwrap();
    let bytes = encode_replay(&header, &[], Vec::new()).unwrap();
    assert_eq!(decode_replay(&bytes).unwrap().header(), &header);

    let mut unknown = encode_replay(
        &ReplayHeader::new(
            header.environment().clone(),
            header.components().clone(),
            header.master_seed(),
            header.entry().clone(),
            1,
        )
        .unwrap(),
        &[starclock_replay::record::RecordRef::new(
            starclock_replay::record::RecordKind::ControllerDiagnostic,
            0,
            &[],
        )
        .unwrap()],
        Vec::new(),
    )
    .unwrap();
    let record_kind_offset = unknown.len() - 13;
    unknown[record_kind_offset] = 0xff;
    assert!(matches!(
        decode_replay(&unknown),
        Err(ReplayError::Format(
            starclock_replay::record::ReplayFormatError::UnknownRecordKind(0xff)
        ))
    ));
}

fn component_set(controller_byte: u8) -> ConfigurationComponentSet {
    ConfigurationComponentSet::new(vec![
        component(
            ConfigurationComponentKind::CombatCatalog,
            "combat-v4.4",
            0x11,
        ),
        component(
            ConfigurationComponentKind::ActivityCore,
            "activity-core",
            0x22,
        ),
        component(
            ConfigurationComponentKind::ModeContent,
            "standard-universe",
            0x33,
        ),
        component(
            ConfigurationComponentKind::Controller,
            "baseline",
            controller_byte,
        ),
    ])
    .unwrap()
}

#[test]
fn component_root_is_canonical_and_reports_the_first_mismatch() {
    let expected = component_set(0x44);
    assert_eq!(
        expected.root().bytes(),
        [
            126, 217, 36, 235, 26, 87, 203, 124, 213, 193, 199, 14, 33, 210, 84, 204, 204, 158,
            210, 10, 105, 84, 252, 225, 132, 183, 91, 191, 174, 140, 28, 168,
        ]
    );
    let actual = component_set(0x45);
    let divergence = expected.verify_exact(&actual).unwrap_err();
    assert_eq!(divergence.index, 3);
    assert_eq!(divergence.expected.unwrap().digest().bytes(), [0x44; 32]);
    assert_eq!(divergence.actual.unwrap().digest().bytes(), [0x45; 32]);
}

#[test]
fn component_set_rejects_duplicate_or_unsorted_keys() {
    let duplicate = component(ConfigurationComponentKind::CombatCatalog, "combat-v4.4", 1);
    assert_eq!(
        ConfigurationComponentSet::new(vec![duplicate.clone(), duplicate]).unwrap_err(),
        ComponentIdentityError::NonCanonicalOrder
    );
    assert_eq!(
        ConfigurationComponentSet::new(vec![
            component(ConfigurationComponentKind::Controller, "controller", 1),
            component(ConfigurationComponentKind::CombatCatalog, "catalog", 2),
        ])
        .unwrap_err(),
        ComponentIdentityError::NonCanonicalOrder
    );
}
