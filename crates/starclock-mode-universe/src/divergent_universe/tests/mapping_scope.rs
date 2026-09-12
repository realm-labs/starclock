#[test]
fn arithmetic_mapping_catalog_closes_eligibility_and_released_identity_denominators() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
    let catalog = factory.bundle.mapping_catalog();
    assert_eq!(catalog.eligibility().len(), 84);
    assert_eq!(catalog.builds().len(), 95);
    assert_eq!(catalog.rules().len(), 7);
    assert_eq!(catalog.source_obligations(), 258);
    assert_eq!(
        catalog
            .builds()
            .iter()
            .filter(|value| value.eligible_catalog_entry)
            .count(),
        84,
    );
    assert_eq!(
        catalog
            .builds()
            .iter()
            .filter(|value| !value.eligible_catalog_entry)
            .count(),
        11,
    );
    assert_eq!(
        catalog
            .builds()
            .iter()
            .filter(|value| {
                value.public_identity
                    != DivergentUniversePublicIdentityResolution::ResolvedAvatarConfig
            })
            .count(),
        4,
    );
    assert!(catalog.builds().iter().all(|value| {
        value.public_identity == DivergentUniversePublicIdentityResolution::ResolvedAvatarConfig
            || !value.eligible_catalog_entry
    }));
    assert!(
        catalog
            .builds()
            .iter()
            .filter(|value| value.eligible_catalog_entry)
            .all(|value| {
                value.public_identity
                    == DivergentUniversePublicIdentityResolution::ResolvedAvatarConfig
            })
    );
    let caller_bound = catalog
        .builds()
        .iter()
        .filter(|value| value.eligible_catalog_entry)
        .find(|value| {
            value
                .avatar
                .as_str()
                .parse::<u32>()
                .ok()
                .and_then(|source| core.character_form_for_source_avatar(source))
                .is_none()
        })
        .expect("eligible caller-bound identity");
    let form = core
        .character_form_for_source_avatar(1001)
        .expect("fixture form");
    let temporary = mapping_build(&core, form, 70, 5, false);
    let participants = compiled_participants(&core, &mapping_build(&core, form, 80, 6, true));
    let snapshot = factory
        .compile_mapping(
            &participants,
            &core,
            vec![DivergentUniverseMappingInput::new(
                participant_id(),
                caller_bound.avatar.clone(),
                None,
                temporary,
            )],
        )
        .expect("explicit caller-bound mapping");
    assert_eq!(
        snapshot.participants()[0].avatar_binding_accuracy(),
        DivergentUniverseAvatarBindingAccuracy::VersionedProjectPolicyCallerProvided
    );
}

#[test]
fn arithmetic_mapping_preserves_stronger_owned_fields_without_mutating_caller_build() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
    let avatar = avatar("1001");
    let form = core
        .character_form_for_source_avatar(1001)
        .expect("released source avatar binding");
    let owned = mapping_build(&core, form, 80, 6, true);
    let temporary = mapping_build(&core, form, 70, 5, false);
    let owned_before = owned.clone();
    let temporary_before = temporary.clone();
    let participants = compiled_participants(&core, &owned);
    let owned_compiled = LoadoutCompiler
        .compile(core.build_catalog(), core.combat_catalog(), &owned)
        .expect("owned build");

    let snapshot = factory
        .compile_mapping(
            &participants,
            &core,
            vec![DivergentUniverseMappingInput::new(
                participant_id(),
                avatar,
                Some((owned.clone(), OwnedBuildMinimumFacts::new(true))),
                temporary.clone(),
            )],
        )
        .expect("mapping snapshot");
    let mapped = &snapshot.participants()[0];
    assert_eq!(mapped.participant(), participant_id());
    assert_eq!(mapped.avatar().as_str(), "1001");
    assert_eq!(
        mapped.avatar_binding_accuracy(),
        DivergentUniverseAvatarBindingAccuracy::CoreCatalogResolved
    );
    assert_eq!(
        mapped.compiled().build_digest(),
        owned_compiled.build_digest()
    );
    assert_eq!(
        mapped.receipt().kind(),
        BuildSubstitutionKind::StrengthenedOwned
    );
    assert_eq!(mapped.receipt().progression(), BuildFieldSource::Owned);
    assert_eq!(mapped.receipt().abilities(), BuildFieldSource::Owned);
    assert_eq!(mapped.receipt().eidolon(), BuildFieldSource::Owned);
    assert_eq!(mapped.receipt().light_cone(), BuildFieldSource::Owned);
    assert_eq!(mapped.receipt().contributions(), BuildFieldSource::Owned);
    assert_eq!(owned, owned_before);
    assert_eq!(temporary, temporary_before);
}

#[test]
fn arithmetic_mapping_rejects_ineligible_inputs_and_refreshes_then_tears_down() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
    let catalog = factory.bundle.mapping_catalog();
    let ineligible = catalog
        .builds()
        .iter()
        .find(|value| !value.eligible_catalog_entry)
        .expect("ineligible mapping row");
    let form = core
        .character_form_for_source_avatar(1001)
        .expect("released source avatar binding");
    let owned = mapping_build(&core, form, 80, 6, true);
    let temporary = mapping_build(&core, form, 70, 5, false);
    let participants = compiled_participants(&core, &owned);
    assert!(matches!(
        factory.compile_mapping(
            &participants,
            &core,
            vec![DivergentUniverseMappingInput::new(
                participant_id(),
                ineligible.avatar.clone(),
                Some((owned.clone(), OwnedBuildMinimumFacts::new(true))),
                temporary.clone(),
            )],
        ),
        Err(DivergentUniverseMappingRuntimeError::IneligibleAvatar)
    ));

    let input = DivergentUniverseMappingInput::new(
        participant_id(),
        avatar("1001"),
        Some((owned.clone(), OwnedBuildMinimumFacts::new(true))),
        temporary.clone(),
    );
    assert_eq!(
        input.accuracy(),
        DivergentUniverseTemporaryMinimumAccuracy::VersionedProjectPolicyCallerProvided
    );
    let first = factory
        .compile_mapping(&participants, &core, vec![input.clone()])
        .expect("initial mapping");
    let first_before = first.clone();
    let repeated = factory
        .refresh_mapping(&first, &participants, &core, vec![input])
        .expect("deterministic refresh");
    assert_eq!(repeated.digest(), first.digest());

    let refreshed = factory
        .refresh_mapping(
            &first,
            &participants,
            &core,
            vec![DivergentUniverseMappingInput::new(
                participant_id(),
                avatar("1001"),
                None,
                temporary.clone(),
            )],
        )
        .expect("accepted party-change refresh");
    assert_ne!(refreshed.digest(), first.digest());
    assert_eq!(
        refreshed.participants()[0].receipt().kind(),
        BuildSubstitutionKind::Trial
    );
    assert_eq!(first, first_before);
    assert_eq!(owned.level().get(), 80);
    assert_eq!(temporary.level().get(), 70);

    let removed_digest = refreshed.digest();
    let teardown = refreshed.teardown();
    assert_eq!(teardown.participant_lock(), participants.digest());
    assert_eq!(teardown.removed_snapshot(), removed_digest);
    assert_eq!(teardown.removed_participants(), 1);
}

#[test]
fn arithmetic_mapping_is_bound_to_entry_state_and_cleared_at_run_finalization() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
    let form = core
        .character_form_for_source_avatar(1001)
        .expect("released source avatar binding");
    let owned = mapping_build(&core, form, 80, 6, true);
    let temporary = mapping_build(&core, form, 70, 5, false);
    let participants = compiled_participants(&core, &owned);
    let mapping = Arc::new(
        factory
            .compile_mapping(
                &participants,
                &core,
                vec![DivergentUniverseMappingInput::new(
                    participant_id(),
                    avatar("1001"),
                    Some((owned.clone(), OwnedBuildMinimumFacts::new(true))),
                    temporary,
                )],
            )
            .expect("mapping snapshot"),
    );
    let account = input_snapshot(&participants);
    let without_mapping = factory
        .compile(
            DivergentUniverseEntry::new(
                area_id("401"),
                difficulty_id("3011"),
                Arc::clone(&participants),
                account.clone(),
                vec![],
            )
            .expect("entry"),
        )
        .expect("entry without mapping");
    let with_mapping = factory
        .compile(
            DivergentUniverseEntry::new(
                area_id("401"),
                difficulty_id("3011"),
                Arc::clone(&participants),
                account.clone(),
                vec![],
            )
            .expect("entry")
            .with_mapping_snapshot(Arc::clone(&mapping)),
        )
        .expect("entry with mapping");
    assert_ne!(
        without_mapping.definition().identity().config_digest(),
        with_mapping.definition().identity().config_digest()
    );
    assert_eq!(
        with_mapping.mapping_snapshot().map(|value| value.digest()),
        Some(mapping.digest())
    );
    let mut activity = with_mapping
        .start(instance(8), ActivityMasterSeed::from_u64(22))
        .expect("start")
        .into_activity();
    assert!(matches!(
        slot_value(&activity.player_view(), MAPPING_STATE_SLOT),
        ActivityValue::BoundedCounterMap(values) if values.len() == 1
    ));
    complete(&mut activity);
    assert_eq!(
        slot_value(&activity.player_view(), MAPPING_STATE_SLOT),
        &ActivityValue::BoundedCounterMap(Box::new([]))
    );
    assert_eq!(account, input_snapshot(&participants));

    let mismatched = participants_with_build([8; 32]);
    assert!(matches!(
        factory.compile(
            DivergentUniverseEntry::new(
                area_id("401"),
                difficulty_id("3011"),
                Arc::clone(&mismatched),
                input_snapshot(&mismatched),
                vec![],
            )
            .expect("entry")
            .with_mapping_snapshot(mapping),
        ),
        Err(DivergentUniverseEntryFlowError::Mapping(
            DivergentUniverseMappingRuntimeError::ParticipantLockMismatch
        ))
    ));
}

#[test]
fn logical_run_plane_node_and_battle_scopes_reset_in_transition_order() {
    assert_eq!(
        DivergentUniverseLogicalScopeKind::ALL
            .map(DivergentUniverseLogicalScopeKind::activity_scope),
        [
            ActivityScope::Activity,
            ActivityScope::Section,
            ActivityScope::Node,
            ActivityScope::Attempt,
        ]
    );
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let compiled = factory.compile(entry("401", "3011")).expect("entry");
    let scopes = compiled.definition().state_definition().logical_scopes();
    assert_eq!(scopes.classes().len(), 4);
    assert_eq!(scopes.bindings().len(), compiled.layers().len() + 1);
    assert_eq!(
        scopes
            .classes()
            .iter()
            .map(|class| (class.id(), class.parent()))
            .collect::<Vec<_>>(),
        vec![
            (DivergentUniverseLogicalScopeKind::Run.class_id(), None),
            (
                DivergentUniverseLogicalScopeKind::Plane.class_id(),
                Some(DivergentUniverseLogicalScopeKind::Run.class_id()),
            ),
            (
                DivergentUniverseLogicalScopeKind::Node.class_id(),
                Some(DivergentUniverseLogicalScopeKind::Plane.class_id()),
            ),
            (
                DivergentUniverseLogicalScopeKind::Battle.class_id(),
                Some(DivergentUniverseLogicalScopeKind::Node.class_id()),
            ),
        ]
    );

    let mut activity = compiled
        .start(instance(9), ActivityMasterSeed::from_u64(22))
        .expect("start")
        .into_activity();
    assert_logical_layer(&activity.player_view(), 1);
    let marker = ActivityProgramDefinition::new(
        ActivityProgramId::new(900).expect("program ID"),
        vec![ActivityOperation::SetOrderedIdSet {
            slot: PLANE_FLAGS_SLOT,
            values: vec![44].into_boxed_slice(),
        }],
    )
    .expect("boundary program");
    let state = activity.state_hash();
    activity
        .apply_boundary_program(state, &marker)
        .expect("set plane marker");
    assert_eq!(
        slot_value(&activity.player_view(), PLANE_FLAGS_SLOT),
        &ActivityValue::OrderedIdSet(vec![44].into_boxed_slice())
    );

    let before_rejection = activity.canonical_state_bytes();
    let view = activity.player_view();
    let decision = view.decision().expect("route");
    let option = decision.options()[0].id();
    assert_eq!(
        activity.choose_option(
            ActivityStateHash::new([7; 32]).expect("stale hash"),
            decision.id(),
            option,
        ),
        Err(GraphActivityCommandError::StaleStateHash)
    );
    assert_eq!(activity.canonical_state_bytes(), before_rejection);

    let events = activity
        .choose_option(view.state_hash(), decision.id(), option)
        .expect("accepted transition");
    let section_reset = event_position(&events, |kind| {
        matches!(kind, ActivityTransactionEventKind::SlotReset { slot, point }
            if *slot == PLANE_FLAGS_SLOT && *point == SlotResetPoint::SectionStart)
    });
    let node_reset = event_position(&events, |kind| {
        matches!(kind, ActivityTransactionEventKind::SlotReset { slot, point }
            if *slot == ROOM_SLOT && *point == SlotResetPoint::NodeStart)
    });
    let traversal = event_position(&events, |kind| {
        matches!(kind, ActivityTransactionEventKind::EdgeTraversed(_))
    });
    assert!(section_reset < traversal && node_reset < traversal);
    assert_eq!(
        slot_value(&activity.player_view(), PLANE_FLAGS_SLOT),
        &ActivityValue::OrderedIdSet(Box::new([]))
    );
    assert_logical_layer(&activity.player_view(), 2);

    let battle = ActivityScopePath::new(instance(9))
        .enter_section(SectionId::new(2).expect("section"))
        .expect("enter Plane")
        .enter_node(NodeId::new(2).expect("node"))
        .expect("enter Node")
        .enter_attempt(AttemptId::new(1).expect("attempt"))
        .expect("enter Battle attempt");
    assert_eq!(battle.active_scope(), ActivityScope::Attempt);
    assert_eq!(
        battle
            .identity(ActivityScope::Attempt)
            .expect("Battle identity")
            .scope(),
        ActivityScope::Attempt
    );
    complete(&mut activity);
    assert!(activity.player_view().logical_scopes().is_empty());
}

#[test]
fn party_change_mapping_refresh_creates_fresh_identity_and_never_mutates_old_activity() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
    let old_form = core
        .character_form_for_source_avatar(1001)
        .expect("old form");
    let old_owned = mapping_build(&core, old_form, 80, 6, true);
    let old_minimum = mapping_build(&core, old_form, 70, 5, false);
    let old_participants = compiled_participants(&core, &old_owned);
    let old_mapping = factory
        .compile_mapping(
            &old_participants,
            &core,
            vec![DivergentUniverseMappingInput::new(
                participant_id(),
                avatar("1001"),
                Some((old_owned.clone(), OwnedBuildMinimumFacts::new(true))),
                old_minimum.clone(),
            )],
        )
        .expect("old mapping");
    let old_compiled = factory
        .compile(
            DivergentUniverseEntry::new(
                area_id("401"),
                difficulty_id("3011"),
                Arc::clone(&old_participants),
                input_snapshot(&old_participants),
                vec![],
            )
            .expect("old entry")
            .with_mapping_snapshot(Arc::new(old_mapping.clone())),
        )
        .expect("old flow");
    let old_activity = old_compiled
        .start(instance(10), ActivityMasterSeed::from_u64(22))
        .expect("old start")
        .into_activity();
    let old_bytes = old_activity.canonical_state_bytes();
    let old_hash = old_activity.state_hash();

    let next = factory
        .bundle
        .mapping_catalog()
        .builds()
        .iter()
        .find_map(|build| {
            let source = build.avatar.as_str().parse::<u32>().ok()?;
            let form = core.character_form_for_source_avatar(source)?;
            (build.eligible_catalog_entry && form != old_form).then(|| (build.avatar.clone(), form))
        })
        .expect("second released eligible form");
    let new_owned = mapping_build(&core, next.1, 80, 6, true);
    let new_minimum = mapping_build(&core, next.1, 70, 5, false);
    let new_participants = compiled_participants(&core, &new_owned);
    assert!(matches!(
        factory.refresh_mapping(
            &old_mapping,
            &new_participants,
            &core,
            vec![DivergentUniverseMappingInput::new(
                participant_id(),
                avatar("1001"),
                None,
                old_minimum,
            )],
        ),
        Err(DivergentUniverseMappingRuntimeError::ParticipantAvatarMismatch)
    ));
    assert_eq!(old_activity.canonical_state_bytes(), old_bytes);
    assert_eq!(old_activity.state_hash(), old_hash);

    let refreshed = factory
        .refresh_mapping(
            &old_mapping,
            &new_participants,
            &core,
            vec![DivergentUniverseMappingInput::new(
                participant_id(),
                next.0,
                Some((new_owned, OwnedBuildMinimumFacts::new(true))),
                new_minimum,
            )],
        )
        .expect("accepted party refresh");
    assert_ne!(refreshed.participant_lock(), old_mapping.participant_lock());
    let refreshed_compiled = factory
        .compile(
            DivergentUniverseEntry::new(
                area_id("401"),
                difficulty_id("3011"),
                Arc::clone(&new_participants),
                input_snapshot(&new_participants),
                vec![],
            )
            .expect("refreshed entry")
            .with_mapping_snapshot(Arc::new(refreshed)),
        )
        .expect("refreshed flow");
    assert_ne!(
        refreshed_compiled.definition().identity().config_digest(),
        old_compiled.definition().identity().config_digest()
    );
    let refreshed_activity = refreshed_compiled
        .start(instance(10), ActivityMasterSeed::from_u64(22))
        .expect("refreshed start")
        .into_activity();
    assert_ne!(refreshed_activity.state_hash(), old_hash);
    assert_eq!(old_activity.canonical_state_bytes(), old_bytes);
}

fn complete(activity: &mut starclock_activity::GraphActivity) {
    while activity.player_view().terminal().is_none() {
        let view = activity.player_view();
        let decision = view.decision().expect("route decision");
        assert_eq!(decision.options().len(), 1);
        activity
            .choose_option(view.state_hash(), decision.id(), decision.options()[0].id())
            .expect("offered route");
    }
}

fn assert_logical_layer(view: &starclock_activity::ActivityPlayerView, ordinal: u64) {
    let scopes = view.logical_scopes();
    assert_eq!(scopes.len(), 3);
    assert_eq!(
        scopes
            .iter()
            .map(|scope| (scope.address().class(), scope.address().key()))
            .collect::<Vec<_>>(),
        vec![
            (DivergentUniverseLogicalScopeKind::Run.class_id(), 1),
            (DivergentUniverseLogicalScopeKind::Plane.class_id(), ordinal,),
            (DivergentUniverseLogicalScopeKind::Node.class_id(), ordinal,),
        ]
    );
}

fn event_position(
    events: &[starclock_activity::ActivityTransactionEvent],
    predicate: impl Fn(&ActivityTransactionEventKind) -> bool,
) -> usize {
    events
        .iter()
        .position(|event| predicate(event.kind()))
        .expect("ordered transition event")
}

fn slot_value(
    view: &starclock_activity::ActivityPlayerView,
    id: starclock_activity::ActivitySlotId,
) -> &ActivityValue {
    view.slots()
        .iter()
        .find(|slot| slot.id() == id)
        .expect("visible slot")
        .value()
}

fn entry(area: &str, difficulty: &str) -> DivergentUniverseEntry {
    let entry = raw_entry(area, difficulty);
    if area.starts_with("204") {
        entry.with_cyclical_refresh(refresh(1, area))
    } else {
        entry
    }
}

fn raw_entry(area: &str, difficulty: &str) -> DivergentUniverseEntry {
    let participants = participants();
    let input_snapshot = input_snapshot(&participants);
    DivergentUniverseEntry::new(
        area_id(area),
        difficulty_id(difficulty),
        participants,
        input_snapshot,
        vec![9, 7],
    )
    .expect("entry")
}

fn area_id(raw: &str) -> DivergentUniverseAreaId {
    DivergentUniverseAreaId::new(format!("divergent-universe.area.{raw}")).expect("area ID")
}

fn difficulty_id(raw: &str) -> DivergentUniverseDifficultyId {
    DivergentUniverseDifficultyId::new(format!("divergent-universe.difficulty.{raw}"))
        .expect("difficulty ID")
}

fn cyclical_entry(
    area: &str,
    difficulty: &str,
    epoch: u64,
    challenge: &str,
) -> DivergentUniverseEntry {
    raw_entry(area, difficulty).with_cyclical_refresh(refresh(epoch, challenge))
}

fn refresh(epoch: u64, challenge: &str) -> DivergentUniverseCyclicalRefresh {
    DivergentUniverseCyclicalRefresh::new(
        epoch,
        DivergentUniverseCyclicalChallengeId::new(format!(
            "divergent-universe.cyclical-area.{challenge}"
        ))
        .expect("challenge ID"),
    )
    .expect("cycle")
}

fn star_pioneer(
    division: u16,
    protocol: u16,
    cognoculi: u16,
    unlocked: bool,
) -> DivergentUniverseAstronomicalEntry {
    DivergentUniverseAstronomicalEntry::star_pioneer(
        division_id(division),
        protocol_id(protocol),
        cognoculi,
        unlocked,
    )
}

fn practice(
    division: u16,
    protocol: u16,
    cognoculi: u16,
    unlocked: bool,
) -> DivergentUniverseAstronomicalEntry {
    DivergentUniverseAstronomicalEntry::practice(
        division_id(division),
        protocol_id(protocol),
        cognoculi,
        unlocked,
    )
}

fn division_id(raw: u16) -> DivergentUniverseDivisionId {
    DivergentUniverseDivisionId::new(format!("divergent-universe.astronomical-division.{raw}"))
        .expect("division ID")
}

fn protocol_id(raw: u16) -> DivergentUniverseProtocolId {
    DivergentUniverseProtocolId::new(format!("divergent-universe.protocol.{raw}"))
        .expect("protocol ID")
}

fn participants() -> Arc<ParticipantLock> {
    participants_with_build([2; 32])
}

fn participants_with_build(build_digest: [u8; 32]) -> Arc<ParticipantLock> {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .expect("participant policy");
    let build = OpaqueParticipantBuild::new(
        CombatantSpecDigest::new([1; 32]).expect("spec digest"),
        BuildDigest::new(build_digest).expect("build digest"),
        ParticipantSourceKind::Synthetic,
    )
    .expect("opaque build");
    let participant = ParticipantLockEntry::new(
        ParticipantId::new(1).expect("participant ID"),
        0,
        0,
        UnitDefinitionId::new(1).expect("unit ID"),
        build,
    )
    .expect("participant");
    Arc::new(ParticipantLock::seal(policy, vec![participant]).expect("participant lock"))
}

fn compiled_participants(
    core: &starclock_data::catalog::SimulationCatalog,
    build: &CombatantBuildSpec,
) -> Arc<ParticipantLock> {
    let compiled = LoadoutCompiler
        .compile(core.build_catalog(), core.combat_catalog(), build)
        .expect("compiled participant build");
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .expect("participant policy");
    let opaque = OpaqueParticipantBuild::new(
        compiled.combatant().digest(),
        BuildDigest::new(compiled.build_digest().bytes()).expect("build digest"),
        ParticipantSourceKind::CompiledBuild,
    )
    .expect("opaque build");
    let participant = ParticipantLockEntry::new(participant_id(), 0, 0, build.form(), opaque)
        .expect("participant");
    Arc::new(ParticipantLock::seal(policy, vec![participant]).expect("participant lock"))
}

fn mapping_build(
    core: &starclock_data::catalog::SimulationCatalog,
    form: UnitDefinitionId,
    level: u8,
    promotion: u8,
    maximum_abilities: bool,
) -> CombatantBuildSpec {
    let character = core
        .build_catalog()
        .character(form)
        .expect("character build definition");
    let abilities = character
        .ability_levels()
        .iter()
        .map(|table| {
            AbilityInvestment::new(
                table.family(),
                if maximum_abilities {
                    table.invested_cap()
                } else {
                    AbilityLevel::new(1).expect("minimum ability level")
                },
            )
        })
        .collect::<Vec<_>>();
    CombatantBuildSpec::new(
        form,
        UnitLevel::new(level).expect("unit level"),
        PromotionStage::new(promotion).expect("promotion"),
    )
    .with_ability_levels(abilities)
    .expect("ability investments")
    .with_eidolon(EidolonLevel::E0)
}

fn avatar(raw: &str) -> DivergentUniverseAvatarLocator {
    DivergentUniverseAvatarLocator::new(raw).expect("avatar locator")
}

fn participant_id() -> ParticipantId {
    ParticipantId::new(1).expect("participant ID")
}

fn input_snapshot(participants: &ParticipantLock) -> DivergentUniverseInputSnapshot {
    DivergentUniverseInputSnapshot::seal(
        DivergentUniverseAccountSnapshotDigest::new([3; 32]).expect("account snapshot"),
        DivergentUniverseLoadoutSnapshotDigest::new([4; 32]).expect("loadout snapshot"),
        participants,
    )
}

fn instance(raw: u64) -> ActivityInstanceId {
    ActivityInstanceId::new(raw).expect("instance ID")
}
