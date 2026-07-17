use starclock_combat::{
    AbilityId, BattleSeed, BattleSpec, BattleSpecDigest, CombatantSpecDigest, ConcedePolicy,
    EncounterId, EnemyDefinitionId, FormationIndex, Hp, ParticipantSource, ParticipantSpec,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed, TeamResourceSpec, TeamSide,
    UnitDefinitionId, UnitLevel, rng::derive::StreamPath,
};

use super::{SYNTHETIC_STANDARD_RULES_REVISION, SYNTHETIC_STANDARD_SPEC_DIGEST};

pub(super) fn battle_spec() -> BattleSpec {
    let player = combatant(1, 1, 1_000, 200_000_000, 0xc1);
    let enemy = combatant(2, 2, 600, 50_000_000, 0xc2);
    BattleSpec::new(
        SYNTHETIC_STANDARD_RULES_REVISION,
        BattleSpecDigest::new(SYNTHETIC_STANDARD_SPEC_DIGEST).expect("static digest is non-zero"),
        EncounterId::new(1).expect("static ID"),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).expect("static formation"),
                ParticipantSource::Player,
                player,
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).expect("static formation"),
                ParticipantSource::EncounterEnemy(
                    EnemyDefinitionId::new(1).expect("static enemy ID"),
                ),
                enemy,
            ),
        ],
        TeamResourceSpec::new(0, 5).expect("static resource bounds"),
        TeamResourceSpec::new(0, 0).expect("static resource bounds"),
        ConcedePolicy::Allowed,
    )
    .expect("committed synthetic Standard spec must validate")
}

fn combatant(form: u32, ability: u32, hp: i64, speed: i64, digest: u8) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        UnitDefinitionId::new(form).expect("static form ID"),
        UnitLevel::new(80).expect("static level"),
        Hp::new(hp).expect("static HP"),
        Speed::from_scaled(speed).expect("static Speed"),
        ResolvedDefinitionBindings::new(
            vec![AbilityId::new(ability).expect("static ability ID")],
            vec![],
            vec![],
        )
        .expect("static bindings are canonical"),
        CombatantSpecDigest::new([digest; 32]).expect("static digest is non-zero"),
    )
    .expect("static combatant is valid")
}

pub(super) fn battle_seed(master_seed: u64) -> BattleSeed {
    let path = StreamPath::new("standard-v1", 1, 0, 0, 0, 0, "battle")
        .expect("static stream path is valid");
    BattleSeed::new(path.derive_seed(master_seed).bytes())
}
