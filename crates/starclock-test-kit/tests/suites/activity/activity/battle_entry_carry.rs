//! The public start boundary rebinds prior carry without rewriting its ledger.

use std::sync::Arc;

use starclock_activity::{
    ActivityBattleHandoff, ActivityBattleResultContract, ActivityBattleResultSubmission,
    ActivityBattleSettlementError, ActivityBattleStartRequest, ActivityOptionId,
    ActivityParticipantCarryDefinition, ActivityTransactionState, BattleBinding, BattleOutcome,
    EncounterInitiativePolicy, EncounterPreparationDefinition, EnergyCarryPolicy, HpCarryPolicy,
    LifeCarryPolicy, ParticipantBattleState, PreparedBattleVariant, PresenceCarryPolicy,
    TechniqueContributionDigest,
};
use starclock_combat::{
    AbilityId, Battle, BattleSpec, CombatantSpecDigest, EncounterId, EnemyDefinitionId, LifeState,
    ParticipantInitialState, ParticipantSpec, PresenceState, ProgramId, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, SelectorId, TeamSide, UnitDefinitionId,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, ActionResourcePolicy, TargetInvalidationPolicy,
            TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
    },
};

use super::{
    Setup, battle_spec, energy, hp, metric_binding, node, participant, participant_state,
    projection, result,
};

fn second_prepared(
    hp_policy: HpCarryPolicy,
    energy_policy: EnergyCarryPolicy,
) -> (Setup, ActivityTransactionState) {
    let mut setup = Setup::new(true);
    let mut state = setup.state();
    setup.prepare(&mut state, node(20), 1, 1);
    let first = setup.start(&mut state);
    let awaiting = state.state_hash(setup.identity, &setup.graph, setup.instance, &setup.rng);
    state
        .submit_pending_battle_result(
            setup.identity,
            &setup.graph,
            setup.instance,
            &setup.rng,
            ActivityBattleResultSubmission::new(
                awaiting,
                result(
                    first.identity(),
                    BattleOutcome::Won,
                    participant_state(700, 60, LifeState::Alive, PresenceState::Present),
                    0,
                ),
            ),
        )
        .unwrap();

    let mut spec = battle_spec();
    let original = &spec.participants()[0];
    let base = original.combatant();
    let lowered = ResolvedCombatantSpec::new(
        base.form(),
        base.level(),
        hp(500),
        base.speed(),
        ResolvedDefinitionBindings::new(
            base.abilities().to_vec(),
            base.rule_bundles().to_vec(),
            base.modifiers().to_vec(),
        )
        .unwrap(),
        CombatantSpecDigest::new([0xa1; 32]).unwrap(),
    )
    .unwrap()
    .with_energy(energy(0), energy(40))
    .unwrap();
    let changed = ParticipantSpec::new(
        original.side(),
        original.formation(),
        original.source(),
        lowered,
    )
    .with_locked_combatant_digest(original.locked_combatant_digest());
    spec = BattleSpec::new(
        spec.assembly_digest(),
        spec.encounter(),
        vec![changed, spec.participants()[1].clone()],
        spec.resources(TeamSide::Player).clone(),
        spec.resources(TeamSide::Enemy).clone(),
        spec.concede_policy(),
    )
    .unwrap();
    setup.preparation = Arc::new(
        EncounterPreparationDefinition::new(
            ActivityOptionId::new(10).unwrap(),
            EncounterInitiativePolicy::PlayerControlled,
            setup.roster.digest(),
            0,
            vec![],
            vec![PreparedBattleVariant::new(
                vec![],
                TechniqueContributionDigest::new([0x35; 32]).unwrap(),
                BattleBinding::new(spec, "battle", setup.roster.digest()).unwrap(),
            )],
        )
        .unwrap(),
    );
    setup.contract = Arc::new(
        ActivityBattleResultContract::new(
            Arc::new(projection()),
            vec![ActivityParticipantCarryDefinition::new(
                participant(1),
                hp_policy,
                energy_policy,
                LifeCarryPolicy::CarryExact,
                PresenceCarryPolicy::CarryExact,
            )],
            vec![metric_binding()],
        )
        .unwrap(),
    );
    setup.prepare(&mut state, node(30), 2, 2);
    (setup, state)
}

fn assert_destination_initial_state(handoff: &ActivityBattleHandoff) {
    let carry = handoff.participant_carry()[0];
    let spec = &handoff.battle_spec().participants()[0];
    let initial = ParticipantInitialState::new(
        carry.current_hp(),
        spec.combatant().maximum_hp(),
        carry.current_energy(),
        spec.combatant().maximum_energy(),
        carry.life(),
        carry.presence(),
    )
    .unwrap();
    let entered = spec.clone().with_initial_state(initial).unwrap();
    let input = handoff.battle_spec();
    let spec = BattleSpec::new(
        input.assembly_digest(),
        input.encounter(),
        vec![entered, input.participants()[1].clone()],
        input.resources(TeamSide::Player).clone(),
        input.resources(TeamSide::Enemy).clone(),
        input.concede_policy(),
    )
    .unwrap();
    let mut builder = CombatCatalogBuilder::new([0xb1; 32]);
    for raw in [101, 201] {
        let selector = SelectorId::new(raw).unwrap();
        let program = ProgramId::new(raw).unwrap();
        let ability = AbilityId::new(raw).unwrap();
        builder.add_selector(SelectorDefinition::new(selector).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single).unwrap(),
        ));
        builder.add_program(ProgramDefinition::new(
            program,
            vec![],
            vec![selector],
            vec![],
            vec![],
        ));
        builder.add_ability(
            AbilityDefinition::new(ability, program, selector, vec![]).with_action(
                AbilityActionDefinition::new(
                    AbilityKind::Basic,
                    1,
                    TargetInvalidationPolicy::CancelRemainingForTarget,
                    ActionResourcePolicy::new(0, 0, energy(0), energy(0)),
                )
                .unwrap(),
            ),
        );
        builder.add_unit(UnitDefinition::new(
            UnitDefinitionId::new(raw).unwrap(),
            vec![ability],
            vec![],
        ));
    }
    builder.add_enemy(EnemyDefinition::new(
        EnemyDefinitionId::new(201).unwrap(),
        UnitDefinitionId::new(201).unwrap(),
        vec![AbilityId::new(201).unwrap()],
    ));
    builder.add_encounter(EncounterDefinition::new(
        EncounterId::new(1).unwrap(),
        vec![EnemyDefinitionId::new(201).unwrap()],
        vec![],
    ));
    Battle::create(builder.build().unwrap(), spec, handoff.identity().seed()).unwrap();
}

#[test]
fn entry_without_prior_ledger_keeps_explicit_initial_state_not_restore_policy() {
    let mut setup = Setup::new(false);
    let input = battle_spec();
    let player = input.participants()[0]
        .clone()
        .with_initial_state(
            ParticipantInitialState::new(
                hp(123),
                hp(1_000),
                energy(17),
                energy(100),
                LifeState::Alive,
                PresenceState::Departed,
            )
            .unwrap(),
        )
        .unwrap();
    let spec = BattleSpec::new(
        input.assembly_digest(),
        input.encounter(),
        vec![player, input.participants()[1].clone()],
        input.resources(TeamSide::Player).clone(),
        input.resources(TeamSide::Enemy).clone(),
        input.concede_policy(),
    )
    .unwrap();
    setup.preparation = Arc::new(
        EncounterPreparationDefinition::new(
            ActivityOptionId::new(10).unwrap(),
            EncounterInitiativePolicy::PlayerControlled,
            setup.roster.digest(),
            0,
            vec![],
            vec![PreparedBattleVariant::new(
                vec![],
                TechniqueContributionDigest::new([0x36; 32]).unwrap(),
                BattleBinding::new(spec, "battle", setup.roster.digest()).unwrap(),
            )],
        )
        .unwrap(),
    );
    setup.contract = Arc::new(
        ActivityBattleResultContract::new(
            Arc::new(projection()),
            vec![ActivityParticipantCarryDefinition::new(
                participant(1),
                HpCarryPolicy::RestoreFull,
                EnergyCarryPolicy::ResetZero,
                LifeCarryPolicy::RestoreAlive,
                PresenceCarryPolicy::RestorePresent,
            )],
            vec![metric_binding()],
        )
        .unwrap(),
    );
    let mut state = setup.state();
    setup.prepare(&mut state, node(20), 1, 1);
    let handoff = setup.start(&mut state);
    let carry = handoff.participant_carry()[0];
    assert_eq!(carry.current_hp(), hp(123));
    assert_eq!(carry.current_energy(), energy(17));
    assert_eq!(carry.presence(), PresenceState::Departed);
    assert_destination_initial_state(&handoff);
}

#[test]
fn entry_clamped_capacity_handoff_is_valid_and_ledger_remains_previous_result() {
    let (setup, mut state) =
        second_prepared(HpCarryPolicy::CarryClamped, EnergyCarryPolicy::CarryClamped);
    let before = state
        .player_view(setup.identity, &setup.graph, setup.instance, &setup.rng)
        .participant_carry()[0];
    let rng = setup.rng.snapshots();
    let handoff = setup.start(&mut state);
    let carry = handoff.participant_carry()[0];
    assert_eq!(carry.current_hp(), hp(500));
    assert_eq!(carry.maximum_hp(), hp(500));
    assert_eq!(carry.current_energy(), energy(40));
    assert_eq!(carry.maximum_energy(), energy(40));
    assert_destination_initial_state(&handoff);
    assert_eq!(
        state
            .player_view(setup.identity, &setup.graph, setup.instance, &setup.rng)
            .participant_carry()[0],
        before
    );
    assert_eq!(setup.rng.snapshots(), rng);

    let hash = state.state_hash(setup.identity, &setup.graph, setup.instance, &setup.rng);
    let projected = ParticipantBattleState::new(
        participant(1),
        hp(450),
        hp(500),
        energy(30),
        energy(40),
        carry.life(),
        carry.presence(),
    )
    .unwrap();
    state
        .submit_pending_battle_result(
            setup.identity,
            &setup.graph,
            setup.instance,
            &setup.rng,
            ActivityBattleResultSubmission::new(
                hash,
                result(handoff.identity(), BattleOutcome::Won, projected, 0),
            ),
        )
        .unwrap();
    let settled = state
        .player_view(setup.identity, &setup.graph, setup.instance, &setup.rng)
        .participant_carry()[0];
    assert_eq!(settled.maximum_hp(), hp(500));
    assert_eq!(settled.current_hp(), hp(450));
}

#[test]
fn entry_exact_capacity_rejection_preserves_bytes_rng_and_pending_boundary() {
    for (hp_policy, energy_policy) in [
        (HpCarryPolicy::CarryExact, EnergyCarryPolicy::CarryClamped),
        (HpCarryPolicy::CarryClamped, EnergyCarryPolicy::CarryExact),
    ] {
        let (mut setup, mut state) = second_prepared(hp_policy, energy_policy);
        let bytes =
            state.canonical_state_bytes(setup.identity, &setup.graph, setup.instance, &setup.rng);
        let hash = state.state_hash(setup.identity, &setup.graph, setup.instance, &setup.rng);
        let rng = setup.rng.snapshots();
        for _ in 0..2 {
            assert_eq!(
                state.start_pending_battle(
                    &setup.graph,
                    &setup.rng,
                    ActivityBattleStartRequest::new(
                        hash,
                        setup.identity,
                        setup.instance,
                        Arc::clone(&setup.contract)
                    )
                ),
                Err(ActivityBattleSettlementError::CarryInvariant)
            );
            assert_eq!(
                state.canonical_state_bytes(
                    setup.identity,
                    &setup.graph,
                    setup.instance,
                    &setup.rng
                ),
                bytes
            );
            assert_eq!(setup.rng.snapshots(), rng);
        }
        setup.contract = Arc::new(
            ActivityBattleResultContract::new(
                Arc::new(projection()),
                vec![ActivityParticipantCarryDefinition::new(
                    participant(1),
                    HpCarryPolicy::CarryClamped,
                    EnergyCarryPolicy::CarryClamped,
                    LifeCarryPolicy::CarryExact,
                    PresenceCarryPolicy::CarryExact,
                )],
                vec![metric_binding()],
            )
            .unwrap(),
        );
        assert_destination_initial_state(&setup.start(&mut state));
    }
}

#[test]
fn entry_capacity_fresh_reconstruction_matches_handoff_and_state_bytes() {
    let (setup, mut state) =
        second_prepared(HpCarryPolicy::CarryClamped, EnergyCarryPolicy::CarryClamped);
    let (fresh_setup, mut fresh) =
        second_prepared(HpCarryPolicy::CarryClamped, EnergyCarryPolicy::CarryClamped);
    assert_eq!(setup.start(&mut state), fresh_setup.start(&mut fresh));
    assert_eq!(
        state.canonical_state_bytes(setup.identity, &setup.graph, setup.instance, &setup.rng),
        fresh.canonical_state_bytes(
            fresh_setup.identity,
            &fresh_setup.graph,
            fresh_setup.instance,
            &fresh_setup.rng
        )
    );
}
