use crate::divergent_universe::equation_progress::main_progress_key;

#[derive(Debug, Eq, PartialEq)]
struct VerticalSliceReplayReceipt {
    contributed_input: starclock_combat::CombatInputDigest,
    control_input: starclock_combat::CombatInputDigest,
    battle_result: starclock_activity::BattleResultDigest,
    battle_final_state: starclock_combat::BattleStateHash,
    battle_outcome: starclock_activity::BattleOutcome,
    post_settlement_state: ActivityStateHash,
    terminal_state: ActivityStateHash,
}

#[test]
fn first_ordinary_vertical_slice_executes_real_battle_and_replays_from_production_inputs() {
    let first = execute_first_ordinary_vertical_slice(22, false);
    let replay = execute_first_ordinary_vertical_slice(22, false);
    let control = execute_first_ordinary_vertical_slice(22, true);
    assert_eq!(first, replay);
    assert_ne!(first.contributed_input, first.control_input);
    assert_eq!(first.battle_outcome, starclock_activity::BattleOutcome::Won);
    assert_eq!(control.battle_outcome, starclock_activity::BattleOutcome::Won);
    assert_ne!(first.battle_final_state, control.battle_final_state);
}

#[test]
fn first_ordinary_vertical_slice_rejects_invalid_order_stale_state_and_wrong_entry_without_mutation()
{
    let (factory, core, participants, mapping) = vertical_slice_inputs();
    let wrong = DivergentUniverseEntry::new(
        area_id("402"),
        difficulty_id("3012"),
        Arc::clone(&participants),
        input_snapshot(&participants),
        Vec::new(),
    )
    .expect("well-formed entry")
    .with_mapping_snapshot(Arc::clone(&mapping))
    .with_first_ordinary_vertical_slice();
    assert!(matches!(
        factory.compile(wrong),
        Err(DivergentUniverseEntryFlowError::VerticalSliceSelectionMismatch)
    ));

    let flow = vertical_slice_flow(&factory, Arc::clone(&participants), Arc::clone(&mapping));
    let mut activity = flow
        .start(instance(23), ActivityMasterSeed::from_u64(22))
        .expect("vertical-slice start")
        .into_activity();
    let before = activity.canonical_state_bytes();
    let current = activity.state_hash();
    let out_of_order = flow.apply_first_ordinary_vertical_slice_transition(
        &mut activity,
        current,
        super::vertical_slice::DivergentUniverseVerticalSliceTransition::AcquireBlessing,
    );
    assert!(matches!(
        out_of_order,
        Err(super::vertical_slice::DivergentUniverseVerticalSliceError::Activity(_))
    ));
    assert_eq!(activity.canonical_state_bytes(), before);

    flow.apply_first_ordinary_vertical_slice_transition(
        &mut activity,
        current,
        super::vertical_slice::DivergentUniverseVerticalSliceTransition::AcquireEquation,
    )
    .expect("equation acquisition");
    let after_equation = activity.canonical_state_bytes();
    let stale = flow.apply_first_ordinary_vertical_slice_transition(
        &mut activity,
        current,
        super::vertical_slice::DivergentUniverseVerticalSliceTransition::AcquireBlessing,
    );
    assert!(matches!(
        stale,
        Err(super::vertical_slice::DivergentUniverseVerticalSliceError::Activity(
            GraphActivityCommandError::StaleStateHash
        ))
    ));
    assert_eq!(activity.canonical_state_bytes(), after_equation);

    let materialization = factory
        .materialize_first_ordinary_vertical_slice_battle(&flow, &core)
        .expect("battle materialization");
    let before_battle_rejection = activity.canonical_state_bytes();
    assert!(matches!(
        flow.start_first_ordinary_vertical_slice_battle(
            &mut activity,
            ActivityStateHash::new([9; 32]).expect("stale state"),
            AttemptId::new(1).expect("attempt"),
            starclock_activity::BattleSequence::new(1).expect("battle sequence"),
            &materialization,
        ),
        Err(super::battle::DivergentUniverseBattleError::StaleState)
    ));
    assert_eq!(activity.canonical_state_bytes(), before_battle_rejection);
}

fn execute_first_ordinary_vertical_slice(
    seed: u64,
    control_verification: bool,
) -> VerticalSliceReplayReceipt {
    let (factory, core, participants, mapping) = vertical_slice_inputs();
    let flow = vertical_slice_flow(&factory, participants, mapping);
    let mut activity = flow
        .start(instance(22), ActivityMasterSeed::from_u64(seed))
        .expect("vertical-slice start")
        .into_activity();
    assert_eq!(
        activity
            .player_view()
            .decision()
            .expect("encounter decision")
            .kind(),
        starclock_activity::ActivityDecisionKind::Encounter
    );

    let current = activity.state_hash();
    flow.apply_first_ordinary_vertical_slice_transition(
        &mut activity,
        current,
        super::vertical_slice::DivergentUniverseVerticalSliceTransition::AcquireEquation,
    )
    .expect("acquire Equation");
    let content = flow
        .first_ordinary_vertical_slice_content()
        .expect("frozen content");
    assert!(ordered_set_contains(
        &activity,
        super::state::EQUATIONS_SLOT,
        content.equation_key()
    ));
    assert_eq!(
        counter_value(
            &activity,
            super::state::EQUATION_PROGRESS_SLOT,
            main_progress_key(content.equation_key())
        ),
        0
    );

    let current = activity.state_hash();
    flow.apply_first_ordinary_vertical_slice_transition(
        &mut activity,
        current,
        super::vertical_slice::DivergentUniverseVerticalSliceTransition::AcquireBlessing,
    )
    .expect("acquire owned Blessing");
    assert_eq!(
        counter_value(
            &activity,
            super::state::EQUATION_PROGRESS_SLOT,
            main_progress_key(content.equation_key())
        ),
        1
    );
    assert_eq!(
        counter_value(
            &activity,
            super::state::BLESSINGS_SLOT,
            content.blessing_key()
        ),
        1
    );

    let insufficient = activity.canonical_state_bytes();
    let current = activity.state_hash();
    let insufficient_result = flow.apply_first_ordinary_vertical_slice_transition(
        &mut activity,
        current,
        super::vertical_slice::DivergentUniverseVerticalSliceTransition::EnhanceBlessingAtWorkbench,
    );
    assert!(insufficient_result.is_err());
    assert_eq!(activity.canonical_state_bytes(), insufficient);
    let current = activity.state_hash();
    flow.apply_currency_command(
        &mut activity,
        current,
        DivergentUniverseCurrencyCommand::Credit {
            currency: DivergentUniverseCurrencyKind::WorkbenchHeat,
            rule: DivergentUniverseCurrencyGainRule::Unspecified,
            amount: 1,
        },
    )
    .expect("credit policy Workbench Heat");
    let current = activity.state_hash();
    flow.apply_first_ordinary_vertical_slice_transition(
        &mut activity,
        current,
        super::vertical_slice::DivergentUniverseVerticalSliceTransition::EnhanceBlessingAtWorkbench,
    )
    .expect("enhance same Blessing identity");
    assert_eq!(
        counter_value(
            &activity,
            super::state::EQUATION_PROGRESS_SLOT,
            main_progress_key(content.equation_key())
        ),
        1,
        "enhancement does not double-count the owned Blessing identity"
    );
    assert_eq!(
        counter_value(
            &activity,
            super::state::BLESSINGS_SLOT,
            content.blessing_key()
        ),
        2
    );
    assert_eq!(
        counter_value(
            &activity,
            super::state::SERVICE_RECEIPTS_SLOT,
            content.workbench_receipt_key()
        ),
        1
    );

    let current = activity.state_hash();
    flow.apply_first_ordinary_vertical_slice_transition(
        &mut activity,
        current,
        super::vertical_slice::DivergentUniverseVerticalSliceTransition::AcceptTitanBoon,
    )
    .expect("accept Titan Boon");
    assert!(ordered_set_contains(
        &activity,
        super::state::TITAN_BOONS_SLOT,
        content.titan_boon_key()
    ));

    let materialization = factory
        .materialize_first_ordinary_vertical_slice_battle(&flow, &core)
        .expect("materialize real battle");
    assert_eq!(materialization.stage_id(), "83002081");
    assert_eq!(
        materialization.encounter_group_id(),
        "divergent-universe.encounter-group.300202"
    );
    assert_eq!(
        materialization.accuracy(),
        super::battle::DivergentUniverseEncounterSelectionAccuracy::VersionedProjectPolicyCandidateWithCalibratedSharedMinionProxy
    );
    assert_ne!(
        materialization.battle_spec().combat_input_digest(),
        materialization
            .control_battle_spec()
            .combat_input_digest()
    );
    let execution_materialization = if control_verification {
        materialization.control_verification()
    } else {
        materialization.clone()
    };
    let current = activity.state_hash();
    let execution = flow
        .execute_first_ordinary_vertical_slice_battle(
            &mut activity,
            current,
            AttemptId::new(1).expect("attempt"),
            starclock_activity::BattleSequence::new(1).expect("battle sequence"),
            &execution_materialization,
        )
        .expect("execute and settle real battle");
    assert_eq!(execution.report().outcome(), starclock_activity::BattleOutcome::Won);
    assert_eq!(activity.player_view().completed_battle_count(), 1);
    assert_eq!(activity.player_view().decision().expect("post-battle reward").kind(), starclock_activity::ActivityDecisionKind::Reward);
    let reward = activity.player_view().decision().expect("reward").clone();
    let hash = activity.state_hash();
    flow.choose_battle_blessing(&mut activity, hash, reward.id(), reward.options()[0].id()).expect("accepted reward");
    assert_eq!(
        slot_value(&activity.player_view(), LAYER_SEQUENCE_SLOT),
        &ActivityValue::BoundedInteger(2)
    );
    assert_eq!(
        activity
            .player_view()
            .decision()
            .expect("later-layer route")
            .kind(),
        starclock_activity::ActivityDecisionKind::Route
    );

    let receipt = VerticalSliceReplayReceipt {
        contributed_input: materialization.battle_spec().combat_input_digest(),
        control_input: materialization
            .control_battle_spec()
            .combat_input_digest(),
        battle_result: execution.result().claimed_digest(),
        battle_final_state: execution.report().final_state_hash(),
        battle_outcome: execution.report().outcome(),
        post_settlement_state: execution.post_settlement_state_hash(),
        terminal_state: {
            complete(&mut activity);
            activity.state_hash()
        },
    };
    assert_eq!(
        activity.player_view().terminal(),
        Some(ActivityTerminalOutcome::Completed)
    );
    assert_eq!(
        slot_value(&activity.player_view(), super::state::EQUATIONS_SLOT),
        &ActivityValue::OrderedIdSet(Box::new([]))
    );
    assert_eq!(
        slot_value(&activity.player_view(), super::state::BLESSINGS_SLOT),
        &ActivityValue::BoundedCounterMap(Box::new([]))
    );
    assert_eq!(
        slot_value(&activity.player_view(), super::state::TITAN_BOONS_SLOT),
        &ActivityValue::OrderedIdSet(Box::new([]))
    );
    receipt
}

fn vertical_slice_inputs() -> (
    DivergentUniverseRuntimeFactory,
    Arc<starclock_data::catalog::SimulationCatalog>,
    Arc<ParticipantLock>,
    Arc<super::mapping::DivergentUniverseMappingSnapshot>,
) {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production DU bundle");
    let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
    let selections = [(1308_u32, 70_u8), (1005, 80), (1009, 80), (1105, 80)];
    let mut owned_builds = Vec::new();
    for (avatar, level) in selections {
        let form = core
            .character_form_for_source_avatar(avatar)
            .expect("released party form");
        let mut build = mapping_build(&core, form, level, if level == 70 { 5 } else { 6 }, true);
        if avatar == 1308 {
            let character = core
                .build_catalog()
                .character(form)
                .expect("Acheron build definition");
            let light_cone = core
                .build_catalog()
                .light_cone_ids()
                .find(|id| {
                    core.build_catalog()
                        .light_cone(*id)
                        .is_some_and(|definition| definition.path() == character.path())
                })
                .expect("compatible released Light Cone");
            build = build.with_light_cone(starclock_build::spec::LightConeLoadout::new(
                light_cone,
                starclock_build::light_cone::LightConeLevel::new(80)
                    .expect("Light Cone level"),
                PromotionStage::new(6).expect("Light Cone promotion"),
                starclock_build::light_cone::Superimposition::new(5)
                    .expect("superimposition"),
            ));
        }
        owned_builds.push((avatar, build));
    }

    let policy = ParticipantPolicy::new(
        1,
        4,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .expect("four-person party policy");
    let entries = owned_builds
        .iter()
        .enumerate()
        .map(|(index, (_, build))| {
            let compiled = LoadoutCompiler
                .compile(core.build_catalog(), core.combat_catalog(), build)
                .expect("compile caller build");
            let opaque = OpaqueParticipantBuild::new(
                compiled.combatant().digest(),
                BuildDigest::new(compiled.build_digest().bytes()).expect("build digest"),
                ParticipantSourceKind::CompiledBuild,
            )
            .expect("opaque build");
            ParticipantLockEntry::new(
                ParticipantId::new(u32::try_from(index + 1).expect("participant index"))
                    .expect("participant ID"),
                0,
                u8::try_from(index).expect("formation index"),
                build.form(),
                opaque,
            )
            .expect("participant lock entry")
        })
        .collect();
    let participants =
        Arc::new(ParticipantLock::seal(policy, entries).expect("sealed participant lock"));
    let mapping_inputs = owned_builds
        .iter()
        .enumerate()
        .map(|(index, (source_avatar, owned))| {
            let temporary = mapping_build(&core, owned.form(), 80, 6, true);
            DivergentUniverseMappingInput::new(
                ParticipantId::new(u32::try_from(index + 1).expect("participant index"))
                    .expect("participant ID"),
                avatar(&source_avatar.to_string()),
                (index == 0).then(|| (owned.clone(), OwnedBuildMinimumFacts::new(true))),
                temporary,
            )
        })
        .collect();
    let mapping = Arc::new(
        factory
            .compile_mapping(&participants, &core, mapping_inputs)
            .expect("four-person Arithmetic Mapping"),
    );
    (factory, core, participants, mapping)
}

fn vertical_slice_flow(
    factory: &DivergentUniverseRuntimeFactory,
    participants: Arc<ParticipantLock>,
    mapping: Arc<super::mapping::DivergentUniverseMappingSnapshot>,
) -> super::entry_flow::DivergentUniverseFlowInstance {
    factory
        .compile(
            DivergentUniverseEntry::new(
                area_id("401"),
                difficulty_id("3011"),
                Arc::clone(&participants),
                input_snapshot(&participants),
                Vec::new(),
            )
            .expect("vertical-slice entry")
            .with_mapping_snapshot(mapping)
            .with_first_ordinary_vertical_slice(),
        )
        .expect("compile vertical-slice flow")
}

fn counter_value(
    activity: &starclock_activity::GraphActivity,
    slot: starclock_activity::ActivitySlotId,
    key: u64,
) -> i64 {
    match slot_value(&activity.player_view(), slot) {
        ActivityValue::BoundedCounterMap(values) => values
            .binary_search_by_key(&key, |entry| entry.0)
            .ok()
            .map_or(0, |index| values[index].1),
        other => panic!("expected counter-map slot, got {other:?}"),
    }
}

fn ordered_set_contains(
    activity: &starclock_activity::GraphActivity,
    slot: starclock_activity::ActivitySlotId,
    key: u64,
) -> bool {
    match slot_value(&activity.player_view(), slot) {
        ActivityValue::OrderedIdSet(values) => values.binary_search(&key).is_ok(),
        other => panic!("expected ordered-ID slot, got {other:?}"),
    }
}
