use std::sync::Arc;

use starclock_activity::{
    ActivityInstanceId, ActivityMasterSeed, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivityScope, ActivityScopePath, ActivityStateHash,
    ActivityTerminalOutcome, ActivityTransactionEventKind, ActivityValue, AttemptId, BuildDigest,
    GraphActivityCommandError, LoadoutLockScope, NodeId, OpaqueParticipantBuild, ParticipantId,
    ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope, SectionId, SlotResetPoint,
};
use starclock_build::{
    ability::{AbilityInvestment, AbilityLevel},
    compiler::LoadoutCompiler,
    spec::{CombatantBuildSpec, EidolonLevel, PromotionStage},
    substitution::{BuildFieldSource, BuildSubstitutionKind, OwnedBuildMinimumFacts},
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId, UnitLevel};
use starclock_data::divergent_universe_catalog::{
    DivergentUniverseAreaId, DivergentUniverseCyclicalChallengeId, DivergentUniverseDifficultyId,
    DivergentUniverseModuleId, DivergentUniverseRunFamily,
};
use starclock_data::divergent_universe_mapping_catalog::{
    DivergentUniverseAvatarLocator, DivergentUniversePublicIdentityResolution,
};
use starclock_data::divergent_universe_progression_catalog::{
    DivergentUniverseDivisionId, DivergentUniverseProtocolId,
};

use super::economy::{
    DivergentUniverseCurrencyCommand, DivergentUniverseCurrencyCommandError,
    DivergentUniverseCurrencyGainRule, DivergentUniverseCurrencyKind,
    DivergentUniverseCurrencyResetRule, DivergentUniverseCurrencyScope,
    DivergentUniverseCurrencySpendRule, DivergentUniverseEconomyError,
    DivergentUniverseRuntimeConstantValue,
};
use super::entry_flow::{
    DivergentUniverseEntry, DivergentUniverseEntryFlowError, DivergentUniverseRoomPolicy,
    DivergentUniverseRuntimeFactory,
};
use super::mapping::{
    DivergentUniverseAvatarBindingAccuracy, DivergentUniverseMappingInput,
    DivergentUniverseMappingRuntimeError, DivergentUniverseTemporaryMinimumAccuracy,
};
use super::progression::{
    DivergentUniverseAstronomicalEntry, DivergentUniverseAstronomicalMode,
    DivergentUniverseCyclicalRefresh, DivergentUniverseProgressionRuntimeError,
    DivergentUniverseProtocolRule,
};
use super::scope::DivergentUniverseLogicalScopeKind;
use super::snapshot::{
    DivergentUniverseAccountSnapshotDigest, DivergentUniverseInputSnapshot,
    DivergentUniverseLoadoutSnapshotDigest, DivergentUniverseSnapshotError,
};
use super::state::{
    ACCOUNT_LOADOUT_SNAPSHOT_SLOT, ASTRONOMICAL_MODE_SLOT, COGNOCULI_SLOT, CURRENCIES_SLOT,
    CYCLICAL_EPOCH_SLOT, DIFFICULTY_LEVELS_SLOT, DIVISION_SLOT, LAYER_SEQUENCE_SLOT,
    MAPPING_STATE_SLOT, PARTY_SNAPSHOT_SLOT, PERMANENT_UNLOCKS_SLOT, PLANE_FLAGS_SLOT,
    PROTOCOL_SLOT, ROOM_SLOT, RUN_FLAGS_SLOT,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../../config/generated/config.sora");

#[test]
fn ordinary_flow_uses_exact_layers_preserves_carry_and_rejects_stale_route() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let compiled = factory
        .compile(entry("401", "3011"))
        .expect("formal Ordinary entry");
    assert_eq!(compiled.run_family(), DivergentUniverseRunFamily::Ordinary);
    assert_eq!(compiled.layers().len(), 3);
    assert_eq!(compiled.finish_conditions().len(), 13);
    assert!(compiled.cyclical_challenge().is_none());
    assert!(compiled.weekly_modifier_ids().is_empty());
    assert_eq!(
        compiled.room_policy(),
        DivergentUniverseRoomPolicy::LogicalLayerCheckpointNoCandidatePromotion,
    );

    let started = compiled
        .start(instance(1), ActivityMasterSeed::from_u64(22))
        .expect("start flow");
    let mut activity = started.into_activity();
    let before = activity.canonical_state_bytes();
    let view = activity.player_view();
    let decision = view.decision().expect("layer route");
    let option = decision.options()[0].id();
    let stale = ActivityStateHash::new([7; 32]).expect("non-zero stale hash");
    assert!(
        activity
            .choose_option(stale, decision.id(), option)
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);

    activity
        .choose_option(view.state_hash(), decision.id(), option)
        .expect("advance first layer");
    let second = activity.player_view();
    assert_eq!(
        slot_value(&second, LAYER_SEQUENCE_SLOT),
        &ActivityValue::BoundedInteger(2)
    );
    assert_eq!(
        slot_value(&second, ROOM_SLOT),
        &ActivityValue::OptionalId(None)
    );
    assert_eq!(
        slot_value(&second, CURRENCIES_SLOT),
        &ActivityValue::BoundedCounterMap(Box::new([])),
    );
    assert_eq!(
        slot_value(&second, PERMANENT_UNLOCKS_SLOT),
        &ActivityValue::OrderedIdSet(vec![7, 9].into_boxed_slice()),
    );
    assert_eq!(
        slot_value(&second, RUN_FLAGS_SLOT),
        &ActivityValue::OrderedIdSet(vec![1].into_boxed_slice()),
    );

    complete(&mut activity);
    let terminal = activity.player_view();
    assert_eq!(
        terminal.terminal(),
        Some(ActivityTerminalOutcome::Completed)
    );
    assert_eq!(
        slot_value(&terminal, RUN_FLAGS_SLOT),
        &ActivityValue::OrderedIdSet(Box::new([])),
    );
    assert_eq!(
        slot_value(&terminal, PERMANENT_UNLOCKS_SLOT),
        &ActivityValue::OrderedIdSet(vec![7, 9].into_boxed_slice()),
    );
}

#[test]
fn cyclical_flow_binds_exact_weekly_identity_without_promoting_room_candidates() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let compiled = factory
        .compile(entry("20401", "3011"))
        .expect("Cyclical entry");
    assert_eq!(compiled.run_family(), DivergentUniverseRunFamily::Cyclical);
    assert_eq!(
        compiled
            .cyclical_challenge()
            .expect("weekly challenge")
            .as_str(),
        "divergent-universe.cyclical-area.20401",
    );
    assert!(compiled.weekly_modifier_ids().is_empty());
    assert_eq!(compiled.definition().graph().nodes().len(), 5);
    let mut activity = compiled
        .start(instance(2), ActivityMasterSeed::from_u64(22))
        .expect("start flow")
        .into_activity();
    complete(&mut activity);
    assert_eq!(
        activity.player_view().terminal(),
        Some(ActivityTerminalOutcome::Completed),
    );
}

#[test]
fn mismatched_difficulty_and_historical_module_fail_before_activity_exists() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    assert!(matches!(
        factory.compile(entry("103", "3011")),
        Err(DivergentUniverseEntryFlowError::DifficultyAreaMismatch),
    ));
    let historical = DivergentUniverseModuleId::new("divergent-universe.module.6002101")
        .expect("well-formed historical identity");
    assert!(matches!(
        factory.compile(entry("401", "3011").with_module(historical)),
        Err(DivergentUniverseEntryFlowError::HistoricalModuleRejected),
    ));
}

#[test]
fn difficulty_protocol_and_star_pioneer_progress_execute_in_activity_state() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let compiled = factory
        .compile(entry("401", "3011").with_astronomical(star_pioneer(1, 1, 0, true)))
        .expect("Star-Pioneer entry");
    let progression = compiled.progression();
    assert_eq!(progression.difficulty_levels(), [46, 47, 48]);
    assert_eq!(
        progression.mode(),
        Some(DivergentUniverseAstronomicalMode::StarPioneer)
    );
    assert_eq!(
        progression.selected_division().expect("division").as_str(),
        "divergent-universe.astronomical-division.1"
    );
    assert_eq!(
        progression
            .settled_division()
            .expect("settled division")
            .as_str(),
        "divergent-universe.astronomical-division.2"
    );
    let contribution = progression.protocol_contribution().expect("protocol");
    assert_eq!(contribution.attack_increase().scaled(), 320_000);
    assert_eq!(contribution.maximum_hp_increase().scaled(), 2_000_000);
    assert_eq!(contribution.speed_increase().scaled(), 160_000);
    assert_eq!(contribution.maximum_toughness_increase(), None);
    assert_eq!(contribution.difficulty_changes().len(), 3);

    let mut activity = compiled
        .start(instance(3), ActivityMasterSeed::from_u64(22))
        .expect("start")
        .into_activity();
    let initial = activity.player_view();
    assert_eq!(
        slot_value(&initial, DIFFICULTY_LEVELS_SLOT),
        &ActivityValue::OrderedIdSet(vec![46, 47, 48].into_boxed_slice())
    );
    assert_eq!(
        slot_value(&initial, ASTRONOMICAL_MODE_SLOT),
        &ActivityValue::OptionalId(Some(2))
    );
    assert_eq!(
        slot_value(&initial, DIVISION_SLOT),
        &ActivityValue::OptionalId(Some(1))
    );
    assert_eq!(
        slot_value(&initial, PROTOCOL_SLOT),
        &ActivityValue::OptionalId(Some(1))
    );
    complete(&mut activity);
    let terminal = activity.player_view();
    assert_eq!(
        slot_value(&terminal, DIVISION_SLOT),
        &ActivityValue::OptionalId(Some(2))
    );
    assert_eq!(
        slot_value(&terminal, COGNOCULI_SLOT),
        &ActivityValue::BoundedInteger(0)
    );
}

#[test]
fn all_eight_protocols_lower_closed_rules_and_exact_fixed_point_scaling() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let mut difficulty_rules = 0;
    let mut entry_rules = 0;
    let mut berserk_rules = 0;
    for level in 1..=8 {
        let compiled = factory
            .compile(entry("401", "3011").with_astronomical(star_pioneer(level, level, 0, true)))
            .expect("protocol entry");
        let contribution = compiled
            .progression()
            .protocol_contribution()
            .expect("protocol contribution");
        assert_eq!(
            contribution.attack_increase().scaled(),
            i64::from(level) * 320_000
        );
        assert_eq!(
            contribution.speed_increase().scaled(),
            i64::from(level) * 160_000
        );
        difficulty_rules += contribution.difficulty_changes().len();
        entry_rules += contribution.entry_rules().len();
        berserk_rules += contribution.berserk_changes().len();
        if level == 7 {
            assert!(contribution.difficulty_changes().contains(
                &DivergentUniverseProtocolRule::IncreaseStorePrice {
                    ratio: starclock_combat::Ratio::from_scaled(250_000),
                }
            ));
            assert!(
                contribution.difficulty_changes().contains(
                    &DivergentUniverseProtocolRule::GrantRandomLevelOneDomains { count: 2 }
                )
            );
        }
    }
    assert_eq!((difficulty_rules, entry_rules, berserk_rules), (19, 3, 3));
}

#[test]
fn all_twenty_two_difficulties_bind_their_exact_released_level_schedules() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let cases: [(&str, &str, &[u16]); 22] = [
        ("103", "1001", &[]),
        ("401", "3011", &[46, 47, 48]),
        ("401", "3012", &[49, 50, 51, 52]),
        ("401", "3013", &[53, 54, 55]),
        ("402", "3021", &[56, 57, 58]),
        ("402", "3022", &[59, 60, 61, 62]),
        ("402", "3023", &[63, 64, 65]),
        ("403", "3031", &[66, 67, 68]),
        ("403", "3032", &[69, 70, 71, 72]),
        ("403", "3033", &[73, 74, 75]),
        ("404", "3041", &[76, 77, 78]),
        ("404", "3042", &[79, 80, 81, 82]),
        ("404", "3043", &[83, 84, 85]),
        ("405", "3051", &[86, 87, 88]),
        ("405", "3052", &[89, 90, 91, 92]),
        ("405", "3053", &[93, 94, 95]),
        ("406", "3061", &[86, 87, 88]),
        ("406", "3062", &[89, 90, 91, 92]),
        ("406", "3063", &[93, 94, 95]),
        ("409", "3071", &[86, 87, 88]),
        ("409", "3072", &[89, 90, 91, 92]),
        ("409", "3073", &[93, 94, 95]),
    ];
    for (area, difficulty, levels) in cases {
        let compiled = factory
            .compile(entry(area, difficulty))
            .expect("released difficulty");
        assert_eq!(compiled.progression().difficulty_levels(), levels);
    }
}

#[test]
fn practice_preserves_progress_and_rejects_locked_or_above_cap_protocols() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let compiled = factory
        .compile(entry("405", "3051").with_astronomical(practice(5, 3, 2, true)))
        .expect("Practice entry");
    assert_eq!(
        compiled.progression().mode(),
        Some(DivergentUniverseAstronomicalMode::Practice)
    );
    let mut activity = compiled
        .start(instance(4), ActivityMasterSeed::from_u64(22))
        .expect("start")
        .into_activity();
    complete(&mut activity);
    let terminal = activity.player_view();
    assert_eq!(
        slot_value(&terminal, DIVISION_SLOT),
        &ActivityValue::OptionalId(Some(5))
    );
    assert_eq!(
        slot_value(&terminal, COGNOCULI_SLOT),
        &ActivityValue::BoundedInteger(2)
    );

    assert!(matches!(
        factory.compile(entry("402", "3021").with_astronomical(practice(2, 3, 0, true))),
        Err(DivergentUniverseEntryFlowError::Progression(
            DivergentUniverseProgressionRuntimeError::ProtocolAboveDivisionCap
        )),
    ));
    assert!(matches!(
        factory.compile(entry("401", "3011").with_astronomical(star_pioneer(1, 1, 0, false))),
        Err(DivergentUniverseEntryFlowError::Progression(
            DivergentUniverseProgressionRuntimeError::AstronomicalModeLocked
        )),
    ));
}

#[test]
fn cognoculi_boundary_advances_only_after_the_exact_division_threshold() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let pending = factory
        .compile(entry("403", "3031").with_astronomical(star_pioneer(3, 3, 0, true)))
        .expect("first Cognoculus");
    assert_eq!(pending.progression().settled_cognoculi(), 1);
    assert_eq!(
        pending
            .progression()
            .settled_division()
            .expect("division")
            .as_str(),
        "divergent-universe.astronomical-division.3"
    );
    let advanced = factory
        .compile(entry("403", "3031").with_astronomical(star_pioneer(3, 3, 1, true)))
        .expect("second Cognoculus");
    assert_eq!(advanced.progression().settled_cognoculi(), 0);
    assert_eq!(
        advanced
            .progression()
            .settled_division()
            .expect("division")
            .as_str(),
        "divergent-universe.astronomical-division.4"
    );
    let terminal = factory
        .compile(entry("409", "3071").with_astronomical(star_pioneer(8, 8, 4, true)))
        .expect("terminal Division transition");
    assert_eq!(
        terminal
            .progression()
            .settled_division()
            .expect("terminal Division")
            .as_str(),
        "divergent-universe.astronomical-division.9"
    );
    assert!(matches!(
        factory.compile(entry("409", "3071").with_astronomical(star_pioneer(9, 8, 0, true))),
        Err(DivergentUniverseEntryFlowError::Progression(
            DivergentUniverseProgressionRuntimeError::TerminalDivisionEntry
        )),
    ));
}

#[test]
fn unsuccessful_cognoculi_policy_respects_exact_retention_boundaries() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let never = factory
        .compile(entry("403", "3031").with_astronomical(star_pioneer(3, 3, 1, true)))
        .expect("never extinguish");
    assert_eq!(never.progression().unsuccessful_cognoculi(0), 1);

    let first = factory
        .compile(entry("405", "3051").with_astronomical(star_pioneer(5, 5, 2, true)))
        .expect("first-layer retention");
    assert_eq!(first.progression().unsuccessful_cognoculi(0), 0);
    assert_eq!(first.progression().unsuccessful_cognoculi(1), 2);

    let second = factory
        .compile(entry("406", "3061").with_astronomical(star_pioneer(7, 7, 3, true)))
        .expect("second-layer retention");
    assert_eq!(second.progression().unsuccessful_cognoculi(1), 0);
    assert_eq!(second.progression().unsuccessful_cognoculi(2), 3);

    let practice = factory
        .compile(entry("405", "3051").with_astronomical(practice(5, 3, 2, true)))
        .expect("Practice retention");
    assert_eq!(practice.progression().unsuccessful_cognoculi(0), 2);
}

#[test]
fn cyclical_refresh_is_exact_required_and_changes_fresh_activity_state() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let first = factory
        .compile(cyclical_entry("20401", "3011", 7, "20401"))
        .expect("cycle 7");
    let second = factory
        .compile(cyclical_entry("20401", "3011", 8, "20401"))
        .expect("cycle 8");
    assert_eq!(first.progression().cyclical_epoch(), Some(7));
    let first_state = first
        .start(instance(5), ActivityMasterSeed::from_u64(22))
        .expect("first start")
        .into_activity();
    let second_state = second
        .start(instance(5), ActivityMasterSeed::from_u64(22))
        .expect("second start")
        .into_activity();
    assert_ne!(
        first_state.canonical_state_bytes(),
        second_state.canonical_state_bytes()
    );
    assert_eq!(
        slot_value(&first_state.player_view(), CYCLICAL_EPOCH_SLOT),
        &ActivityValue::OptionalId(Some(7))
    );
    assert!(matches!(
        factory.compile(raw_entry("20401", "3011")),
        Err(DivergentUniverseEntryFlowError::Progression(
            DivergentUniverseProgressionRuntimeError::MissingCyclicalRefresh
        )),
    ));
    assert!(matches!(
        factory.compile(cyclical_entry("20401", "3011", 7, "20402")),
        Err(DivergentUniverseEntryFlowError::Progression(
            DivergentUniverseProgressionRuntimeError::CyclicalChallengeMismatch
        )),
    ));
    assert!(matches!(
        factory.compile(
            cyclical_entry("20401", "3011", 7, "20401")
                .with_astronomical(star_pioneer(1, 1, 0, true))
        ),
        Err(DivergentUniverseEntryFlowError::Progression(
            DivergentUniverseProgressionRuntimeError::AstronomicalModeCyclicalMismatch
        )),
    ));
}

#[test]
fn currencies_and_constants_lower_to_closed_typed_runtime_values() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let compiled = factory.compile(entry("401", "3011")).expect("entry");
    let economy = compiled.economy();
    assert_eq!(economy.currencies().len(), 2);

    let fragments = economy.currency(DivergentUniverseCurrencyKind::CosmicFragment);
    assert_eq!(fragments.scope(), DivergentUniverseCurrencyScope::Run);
    assert_eq!(
        fragments.reset_rule(),
        DivergentUniverseCurrencyResetRule::RunEnd
    );
    assert_eq!(
        fragments.gain_rules(),
        [
            DivergentUniverseCurrencyGainRule::CurseChest,
            DivergentUniverseCurrencyGainRule::GambleCoinUnit,
        ]
    );
    assert_eq!(
        fragments.spend_rules(),
        [
            DivergentUniverseCurrencySpendRule::CurseChest,
            DivergentUniverseCurrencySpendRule::OtherServicesDeferredToP2B4,
        ]
    );
    assert!(fragments.credit_operations(1).is_ok());
    assert!(fragments.spend_operation(1).is_ok());
    assert_eq!(
        fragments.credit_operations(0),
        Err(DivergentUniverseEconomyError::ZeroAmount)
    );
    assert_eq!(
        fragments.credit_operations(u64::MAX),
        Err(DivergentUniverseEconomyError::AmountOutOfRange)
    );
    assert_eq!(
        fragments.workbench_reset_operation(),
        Err(DivergentUniverseEconomyError::WrongResetBoundary)
    );

    let heat = economy.currency(DivergentUniverseCurrencyKind::WorkbenchHeat);
    assert_eq!(heat.scope(), DivergentUniverseCurrencyScope::Workbench);
    assert_eq!(
        heat.reset_rule(),
        DivergentUniverseCurrencyResetRule::EachWorkbench
    );
    assert!(heat.workbench_reset_operation().is_ok());

    assert_eq!(economy.constants().len(), 18);
    assert_eq!(economy.excluded_constant_count(), 16);
    assert!(economy.constants().iter().any(|constant| {
        constant.id().as_str()
            == "divergent-universe.constant.roguetourn-fundamental-formulacategories"
            && constant.value()
                == &DivergentUniverseRuntimeConstantValue::IntegerArray(
                    vec![1, 5].into_boxed_slice(),
                )
    }));
    assert!(economy.constants().iter().any(|constant| {
        constant.id().as_str()
            == "divergent-universe.constant.roguetourntitan-titan-bless-weight-modify-json-path"
            && matches!(constant.value(), DivergentUniverseRuntimeConstantValue::OpaqueText(value)
                if value.as_ref() == "Config/Level/Rogue/RogueTitanWeight/RogueTitanWeightModify.json")
    }));
}

#[test]
fn input_and_save_snapshots_bind_participants_and_capture_without_mutation() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let participants = participants();
    let snapshot = input_snapshot(&participants);
    let wrong_participants = participants_with_build([8; 32]);
    let invalid = DivergentUniverseEntry::new(
        area_id("401"),
        difficulty_id("3011"),
        wrong_participants,
        snapshot,
        vec![],
    );
    assert!(matches!(
        invalid,
        Err(DivergentUniverseEntryFlowError::Snapshot(
            DivergentUniverseSnapshotError::ParticipantSnapshotMismatch
        ))
    ));

    let compiled = factory.compile(entry("401", "3011")).expect("entry");
    let activity = compiled
        .start(instance(6), ActivityMasterSeed::from_u64(22))
        .expect("start")
        .into_activity();
    let before = activity.canonical_state_bytes();
    let first = compiled
        .capture_save_snapshot(&activity)
        .expect("capture save");
    let second = compiled
        .capture_save_snapshot(&activity)
        .expect("repeat save");
    assert_eq!(first, second);
    assert_eq!(activity.canonical_state_bytes(), before);
    compiled
        .validate_save_snapshot(&first)
        .expect("fresh current inputs validate");
    let mut corrupted = first;
    corrupted.corrupt_state_for_test();
    assert_eq!(
        compiled.validate_save_snapshot(&corrupted),
        Err(DivergentUniverseSnapshotError::StateHashMismatch)
    );

    let player = activity.player_view();
    assert!(player.slots().iter().all(|slot| !matches!(
        slot.id(),
        ACCOUNT_LOADOUT_SNAPSHOT_SLOT | PARTY_SNAPSHOT_SLOT
    )));
    let debug = activity.debug_view();
    assert!(
        debug
            .all_slots()
            .iter()
            .any(|slot| slot.id() == ACCOUNT_LOADOUT_SNAPSHOT_SLOT)
    );
    assert!(
        debug
            .all_slots()
            .iter()
            .any(|slot| slot.id() == PARTY_SNAPSHOT_SLOT)
    );
}

#[test]
fn terminal_settlement_is_immutable_and_run_currency_reset_is_explicit() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let compiled = factory
        .compile(entry("401", "3011").with_astronomical(star_pioneer(1, 1, 0, true)))
        .expect("entry");
    let mut activity = compiled
        .start(instance(7), ActivityMasterSeed::from_u64(22))
        .expect("start")
        .into_activity();
    assert_eq!(
        compiled.progression_settlement(&activity),
        Err(DivergentUniverseSnapshotError::ActivityNotCompleted)
    );
    let input_before = compiled.input_snapshot().clone();
    let credit = DivergentUniverseCurrencyCommand::Credit {
        currency: DivergentUniverseCurrencyKind::CosmicFragment,
        rule: DivergentUniverseCurrencyGainRule::CurseChest,
        amount: 10,
    };
    let current = activity.state_hash();
    compiled
        .apply_currency_command(&mut activity, current, credit)
        .expect("credit fragments");
    let fragments = compiled
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment);
    assert_eq!(
        slot_value(&activity.player_view(), CURRENCIES_SLOT),
        &ActivityValue::BoundedCounterMap(vec![(fragments.key(), 10)].into_boxed_slice())
    );

    let credited = activity.canonical_state_bytes();
    let stale = ActivityStateHash::new([7; 32]).expect("stale hash");
    assert_eq!(
        compiled.apply_currency_command(
            &mut activity,
            stale,
            DivergentUniverseCurrencyCommand::Spend {
                currency: DivergentUniverseCurrencyKind::CosmicFragment,
                rule: DivergentUniverseCurrencySpendRule::CurseChest,
                amount: 1,
            },
        ),
        Err(DivergentUniverseCurrencyCommandError::Activity(
            GraphActivityCommandError::StaleStateHash
        ))
    );
    assert_eq!(activity.canonical_state_bytes(), credited);

    let current = activity.state_hash();
    let overdraft = compiled.apply_currency_command(
        &mut activity,
        current,
        DivergentUniverseCurrencyCommand::Spend {
            currency: DivergentUniverseCurrencyKind::CosmicFragment,
            rule: DivergentUniverseCurrencySpendRule::CurseChest,
            amount: 11,
        },
    );
    assert!(
        matches!(
            overdraft,
            Err(DivergentUniverseCurrencyCommandError::InsufficientBalance)
        ),
        "unexpected overdraft result: {overdraft:?}"
    );
    assert_eq!(activity.canonical_state_bytes(), credited);

    let current = activity.state_hash();
    compiled
        .apply_currency_command(
            &mut activity,
            current,
            DivergentUniverseCurrencyCommand::Credit {
                currency: DivergentUniverseCurrencyKind::WorkbenchHeat,
                rule: DivergentUniverseCurrencyGainRule::Unspecified,
                amount: 3,
            },
        )
        .expect("credit Workbench Heat");
    let current = activity.state_hash();
    compiled
        .apply_currency_command(
            &mut activity,
            current,
            DivergentUniverseCurrencyCommand::ResetWorkbenchHeat,
        )
        .expect("reset Workbench Heat");
    complete(&mut activity);
    assert_eq!(
        slot_value(&activity.player_view(), CURRENCIES_SLOT),
        &ActivityValue::BoundedCounterMap(Box::new([]))
    );
    let settlement = compiled
        .progression_settlement(&activity)
        .expect("terminal settlement");
    assert_eq!(settlement.input(), input_before.digest());
    assert_eq!(settlement.permanent_unlocks(), [7, 9]);
    assert_eq!(settlement.finish_conditions().len(), 13);
    assert_eq!(compiled.input_snapshot(), &input_before);
    let current = activity.state_hash();
    assert_eq!(
        compiled.apply_currency_command(
            &mut activity,
            current,
            DivergentUniverseCurrencyCommand::Credit {
                currency: DivergentUniverseCurrencyKind::CosmicFragment,
                rule: DivergentUniverseCurrencyGainRule::CurseChest,
                amount: 1,
            },
        ),
        Err(DivergentUniverseCurrencyCommandError::ActivityCompleted)
    );
}
