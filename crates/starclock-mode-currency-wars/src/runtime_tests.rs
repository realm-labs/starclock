use std::sync::Arc;

use starclock_activity::{
    ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, BuildDigest,
    LoadoutLockScope, OpaqueParticipantBuild, ParticipantId, ParticipantLock, ParticipantLockEntry,
    ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};

use crate::{
    CurrencyWarsDeployment, CurrencyWarsGambit, CurrencyWarsPosition, CurrencyWarsPositionKind,
    CurrencyWarsRoleId, CurrencyWarsRoleState, CurrencyWarsRoster, CurrencyWarsRun,
    CurrencyWarsRunDefinition, CurrencyWarsRunSetup, catalog::tests_support,
};

#[test]
fn run_starts_at_the_first_route_encounter() {
    let run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(1).unwrap(),
        ActivityMasterSeed::from_u64(7),
    )
    .unwrap();

    assert_eq!(run.gold(), 10);
    assert_eq!(run.team_level(), 1);
    assert_eq!(run.squad_hp(), 100);
    assert_eq!(
        run.player_view().decision().unwrap().kind(),
        ActivityDecisionKind::Encounter
    );
}

#[test]
fn refresh_and_purchase_are_atomic_activity_boundaries() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(1).unwrap(),
        ActivityMasterSeed::from_u64(7),
    )
    .unwrap();
    let offered = run.refresh_shop().unwrap();

    assert_eq!(offered.as_ref(), &[CurrencyWarsRoleId::new(1001).unwrap()]);
    assert_eq!(run.gold(), 8);
    run.buy_role(offered[0]).unwrap();
    assert_eq!(run.gold(), 7);
    assert_eq!(
        run.roster()
            .unwrap()
            .count(CurrencyWarsRoleState::new(offered[0], 1).unwrap()),
        2,
    );
}

fn definition(initial_gold: u32) -> Arc<CurrencyWarsRunDefinition> {
    let catalog = Arc::new(tests_support::catalog());
    let role = CurrencyWarsRoleId::new(1001).unwrap();
    let state = CurrencyWarsRoleState::new(role, 1).unwrap();
    let roster = CurrencyWarsRoster::new(&catalog, [(state, 1)]).unwrap();
    let deployment = CurrencyWarsDeployment::new(
        &catalog,
        &roster,
        1,
        [(
            CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, 1).unwrap(),
            state,
        )],
    )
    .unwrap();
    Arc::new(
        CurrencyWarsRunDefinition::new(
            identity(),
            Arc::clone(&catalog),
            catalog.routes()[0].id,
            catalog.difficulties()[0].source_id,
            CurrencyWarsGambit::Standard,
            participants(),
            CurrencyWarsRunSetup {
                initial_gold,
                initial_team_level: 1,
                initial_experience: 0,
                roster,
                deployment,
            },
        )
        .unwrap(),
    )
}

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(1).unwrap(),
        ActivityDefinitionDigest::new([1; 32]).unwrap(),
        ActivityConfigDigest::new([2; 32]).unwrap(),
    )
}

fn participants() -> ParticipantLock {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Team,
        LoadoutLockScope::Activity,
    )
    .unwrap();
    let entry = ParticipantLockEntry::new(
        ParticipantId::new(1).unwrap(),
        0,
        0,
        UnitDefinitionId::new(1).unwrap(),
        OpaqueParticipantBuild::new(
            CombatantSpecDigest::new([3; 32]).unwrap(),
            BuildDigest::new([4; 32]).unwrap(),
            ParticipantSourceKind::FixedResolved,
        )
        .unwrap(),
    )
    .unwrap();
    ParticipantLock::seal(policy, vec![entry]).unwrap()
}
