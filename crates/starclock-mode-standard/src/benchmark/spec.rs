use starclock_combat::{
    AbilityId, BattleSeed, BattleSpec, BattleSpecDigest, CombatantSpecDigest, ConcedePolicy,
    EncounterId, EnemyDefinitionId, FormationIndex, Hp, KeyedTeamResourceSpec, ParticipantSource,
    ParticipantSpec, ResolvedCombatantSpec, ResolvedDefinitionBindings, SourceDefinitionId, Speed,
    TeamResourceSpec, TeamResourceWavePolicy, TeamSide, UnitDefinitionId, UnitLevel,
    rng::derive::StreamPath,
};

use super::{BENCHMARK_RULES_REVISION, BenchmarkScenario};

pub(super) fn battle_spec(scenario: BenchmarkScenario) -> BattleSpec {
    let (players, enemies, form, ability, encounter) = match scenario {
        BenchmarkScenario::Ordinary | BenchmarkScenario::HashSmall => (1, 1, 1, 1, 1),
        BenchmarkScenario::TriggerHeavyProxy => (1, 1, 2, 2, 1),
        BenchmarkScenario::FullKernel => (1, 1, 4, 4, 1),
        BenchmarkScenario::HashMedium => (2, 2, 1, 1, 2),
        BenchmarkScenario::HashLarge => (4, 4, 1, 1, 3),
    };
    let mut participants = Vec::with_capacity(players + enemies);
    for index in 0..players {
        participants.push(participant(
            TeamSide::Player,
            index,
            ParticipantSource::Player,
            combatant(form, ability, index as u8 + 1),
        ));
    }
    for index in 0..enemies {
        participants.push(participant(
            TeamSide::Enemy,
            index + 4,
            ParticipantSource::EncounterEnemy(EnemyDefinitionId::new(1).expect("static ID")),
            combatant(3, 3, index as u8 + 0x21),
        ));
    }
    let player_resources = TeamResourceSpec::new(0, 5)
        .expect("static resource bounds")
        .with_keyed(vec![
            KeyedTeamResourceSpec::new(
                SourceDefinitionId::new(1).expect("static ID"),
                0,
                65_535,
                TeamResourceWavePolicy::Persist,
            )
            .expect("static keyed resource bounds"),
        ])
        .expect("static keyed resource IDs are unique");
    BattleSpec::new(
        BENCHMARK_RULES_REVISION,
        BattleSpecDigest::new([scenario.code(); 32]).expect("scenario code is non-zero"),
        EncounterId::new(encounter).expect("static ID"),
        participants,
        player_resources,
        TeamResourceSpec::new(0, 0).expect("static resource bounds"),
        ConcedePolicy::Allowed,
    )
    .expect("benchmark battle spec must validate")
}

fn participant(
    side: TeamSide,
    formation: usize,
    source: ParticipantSource,
    combatant: ResolvedCombatantSpec,
) -> ParticipantSpec {
    ParticipantSpec::new(
        side,
        FormationIndex::new(formation as u8).expect("benchmark formation is in range"),
        source,
        combatant,
    )
}

fn combatant(form: u32, ability: u32, digest: u8) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        UnitDefinitionId::new(form).expect("static form ID"),
        UnitLevel::new(80).expect("static level"),
        Hp::new(1_000_000).expect("static HP"),
        Speed::from_scaled(100_000_000).expect("static Speed"),
        ResolvedDefinitionBindings::new(
            vec![AbilityId::new(ability).expect("static ability ID")],
            vec![],
            vec![],
        )
        .expect("static bindings are canonical"),
        CombatantSpecDigest::new([digest; 32]).expect("digest byte is non-zero"),
    )
    .expect("static combatant is valid")
}

pub(super) fn battle_seed(scenario: BenchmarkScenario, master_seed: u64) -> BattleSeed {
    let path = StreamPath::new(
        "standard-benchmark-v1",
        1,
        u32::from(scenario.code()),
        0,
        0,
        0,
        "battle",
    )
    .expect("static stream path is valid");
    BattleSeed::new(path.derive_seed(master_seed).bytes())
}
