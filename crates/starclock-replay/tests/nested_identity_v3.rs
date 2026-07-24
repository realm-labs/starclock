use starclock_activity::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityInstanceId, AttemptId,
    BattleResultConfiguration, BattleResultDigest, BattleResultIdentity, BattleSequence, NodeId,
    ParticipantLockDigest, ScopeIdentity, SectionId,
};
use starclock_combat::{AssemblyDigest, BattleSeed, CombatInputDigest};
use starclock_replay::{
    activity::v3::{
        NestedBattleEndV3, NestedBattleIdentityDivergence, NestedBattleStartV3,
        compare_nested_end_v3, compare_nested_start_v3, decode_nested_battle_end_v3,
        decode_nested_battle_start_v3, encode_nested_battle_end_v3, encode_nested_battle_start_v3,
    },
    digest::ComponentRootDigest,
};

fn identity(combat: u8, assembly: u8, scope_node: u32) -> BattleResultIdentity {
    BattleResultIdentity::new(
        ScopeIdentity::new(
            ActivityInstanceId::new(1).unwrap(),
            SectionId::new(2).unwrap(),
            NodeId::new(scope_node).unwrap(),
            AttemptId::new(4).unwrap(),
        ),
        BattleSequence::new(5).unwrap(),
        BattleResultConfiguration::new(
            ActivityDefinitionDigest::new([0x11; 32]).unwrap(),
            ActivityConfigDigest::new([0x12; 32]).unwrap(),
            ParticipantLockDigest::new([0x13; 32]).unwrap(),
        ),
        CombatInputDigest::new([combat; 32]).unwrap(),
        AssemblyDigest::new([assembly; 32]).unwrap(),
        BattleSeed::new([0x16; 32]),
    )
}

#[test]
fn v3_nested_identity_round_trips_all_six_contract_fields() {
    let identity = identity(0x14, 0x15, 3);
    let start = NestedBattleStartV3::new(
        ComponentRootDigest::new([0x21; 32]),
        starclock_combat::COMBAT_INPUT_CODEC_REVISION,
        identity,
    )
    .unwrap();
    let start_bytes = encode_nested_battle_start_v3(&start).unwrap();
    assert_eq!(decode_nested_battle_start_v3(&start_bytes).unwrap(), start);

    let end = NestedBattleEndV3::new(identity, BattleResultDigest::new([0x22; 32]).unwrap());
    let end_bytes = encode_nested_battle_end_v3(end);
    assert_eq!(decode_nested_battle_end_v3(&end_bytes).unwrap(), end);
}

#[test]
fn identity_divergence_is_reported_in_frozen_order() {
    let recorded = NestedBattleStartV3::new(
        ComponentRootDigest::new([1; 32]),
        starclock_combat::COMBAT_INPUT_CODEC_REVISION,
        identity(2, 3, 4),
    )
    .unwrap();
    assert_eq!(
        compare_nested_start_v3(
            &recorded,
            ComponentRootDigest::new([9; 32]),
            "wrong",
            identity(8, 7, 6),
        ),
        Err(NestedBattleIdentityDivergence::Component)
    );
    assert_eq!(
        compare_nested_start_v3(
            &recorded,
            recorded.component_root(),
            "wrong",
            identity(8, 7, 4),
        ),
        Err(NestedBattleIdentityDivergence::Assembly)
    );
    assert_eq!(
        compare_nested_start_v3(
            &recorded,
            recorded.component_root(),
            "wrong",
            identity(8, 3, 4),
        ),
        Err(NestedBattleIdentityDivergence::CombatInput)
    );
    assert_eq!(
        compare_nested_start_v3(
            &recorded,
            recorded.component_root(),
            starclock_combat::COMBAT_INPUT_CODEC_REVISION,
            identity(2, 3, 6),
        ),
        Err(NestedBattleIdentityDivergence::Handoff)
    );

    let digest = BattleResultDigest::new([5; 32]).unwrap();
    assert_eq!(
        compare_nested_end_v3(
            NestedBattleEndV3::new(identity(2, 3, 4), digest),
            identity(2, 3, 4),
            BattleResultDigest::new([6; 32]).unwrap(),
        ),
        Err(NestedBattleIdentityDivergence::Result)
    );
}
