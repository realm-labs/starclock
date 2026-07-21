use std::sync::{Arc, OnceLock};

use proptest::{
    prelude::*,
    test_runner::{Config as ProptestConfig, FileFailurePersistence, RngAlgorithm, RngSeed},
};
use starclock_combat::{
    AbilityId, Battle, BattleSeed, BattleSpec, BattleSpecDigest, CombatantSpecDigest, Command,
    ConcedePolicy, DecisionKind, EncounterId, EnemyDefinitionId, FormationIndex, Hp,
    ParticipantSource, ParticipantSpec, ProgramId, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, SelectorId, Speed, TeamResourceSpec, TeamSide, UnitDefinitionId,
    UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
    },
};
use starclock_replay::{
    battle::{BattleTraceEntry, battle_record_count, encode_battle_trace, verify_battle_replay},
    digest::{ConfigBundleDigest, ControllerDigest, EntrySpecDigest},
    format::{ControllerIdentity, ReplayEntry, ReplayHeader, ReplayIdentity, decode_replay},
    record::RecordKind,
};

const BATTLE_REPLAY_CORRUPTION_SEED: u64 = 0x6261_7474_6c65_2d31;
const COMMANDS: usize = 512;
const CATALOG_DIGEST: [u8; 32] = [0xd1; 32];
const SPEC_DIGEST: [u8; 32] = [0xd2; 32];
const BATTLE_SEED: [u8; 32] = [0xd3; 32];

fn property_config() -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        max_shrink_iters: 4_096,
        failure_persistence: Some(Box::new(FileFailurePersistence::SourceParallel(
            "proptest-regressions",
        ))),
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(BATTLE_REPLAY_CORRUPTION_SEED),
        ..ProptestConfig::default()
    }
}

fn definition<I>(raw: u32) -> I
where
    I: TryFrom<u32>,
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).expect("fixture definition ID is non-zero")
}

fn catalog() -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new("battle-property-catalog-v1", CATALOG_DIGEST);
    let selector: SelectorId = definition(1);
    let program: ProgramId = definition(1);
    builder.add_selector(SelectorDefinition::new(selector).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_program(ProgramDefinition::new(
        program,
        vec![],
        vec![selector],
        vec![],
        vec![],
    ));
    for raw in 1..=2 {
        let ability: AbilityId = definition(raw);
        let unit: UnitDefinitionId = definition(raw);
        builder.add_ability(
            AbilityDefinition::new(ability, program, selector, vec![]).with_action(
                AbilityActionDefinition::new(
                    AbilityKind::Basic,
                    1,
                    TargetInvalidationPolicy::CancelRemainingForTarget,
                    ActionResourcePolicy::new(
                        0,
                        0,
                        starclock_combat::Energy::ZERO,
                        starclock_combat::Energy::ZERO,
                    ),
                )
                .unwrap()
                .with_hits(vec![ActionHitDefinition::new(Vec::new())])
                .unwrap(),
            ),
        );
        builder.add_unit(UnitDefinition::new(unit, vec![ability], vec![]));
    }
    let enemy: EnemyDefinitionId = definition(1);
    builder.add_enemy(EnemyDefinition::new(
        enemy,
        definition(2),
        vec![definition(2)],
    ));
    builder.add_encounter(EncounterDefinition::new(definition(1), vec![enemy], vec![]));
    builder.build().unwrap()
}

fn combatant(form: u32, digest: u8) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        definition(form),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000_000).unwrap(),
        Speed::from_scaled(100_000_000).unwrap(),
        ResolvedDefinitionBindings::new(vec![definition(form)], vec![], vec![]).unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
}

fn battle() -> Battle {
    let spec = BattleSpec::new(
        "battle-property-rules-v1",
        BattleSpecDigest::new(SPEC_DIGEST).unwrap(),
        definition::<EncounterId>(1),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 0xe1),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(4).unwrap(),
                ParticipantSource::EncounterEnemy(definition(1)),
                combatant(2, 0xe2),
            ),
        ],
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog(), spec, BattleSeed::new(BATTLE_SEED)).unwrap()
}

fn header() -> ReplayHeader {
    ReplayHeader::new(
        ReplayIdentity::new(
            "battle-property-v1",
            "battle-property-rules-v1",
            "battle-property-catalog-v1",
            ConfigBundleDigest::new(CATALOG_DIGEST),
            starclock_combat::NUMERIC_POLICY_REVISION,
            starclock_combat::rng::RNG_ALGORITHM_REVISION,
            starclock_combat::STATE_HASH_REVISION,
        )
        .unwrap(),
        ControllerIdentity::new(
            "battle-property-controller-v1",
            ControllerDigest::new([0xd4; 32]),
        )
        .unwrap(),
        7,
        ReplayEntry::Battle {
            definition_id: 1,
            spec_digest: EntrySpecDigest::new(SPEC_DIGEST),
        },
        battle_record_count(COMMANDS).unwrap(),
    )
    .unwrap()
}

fn supported_command(battle: &Battle) -> Command {
    let decision = battle.decision().unwrap();
    let selected = match decision.kind() {
        DecisionKind::BattleStart => decision.legal_commands().first(),
        DecisionKind::InterruptWindow => decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::PassInterruptWindow { .. })),
        DecisionKind::NormalAction => decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::UseAbility { .. })),
        DecisionKind::BattleChoice => None,
    };
    selected.cloned().unwrap()
}

fn replay() -> &'static [u8] {
    static REPLAY: OnceLock<Vec<u8>> = OnceLock::new();
    REPLAY.get_or_init(build_replay)
}

fn build_replay() -> Vec<u8> {
    let mut battle = battle();
    let mut trace = Vec::with_capacity(COMMANDS);
    for _ in 0..COMMANDS {
        let command = supported_command(&battle);
        let resolution = battle.apply(command.clone()).unwrap();
        trace.push(BattleTraceEntry::new(command, resolution.state_hash()));
    }
    encode_battle_trace(&header(), &trace).unwrap()
}

fn unique_offset(bytes: &[u8], needle: &[u8]) -> usize {
    let offsets = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(offset, candidate)| (candidate == needle).then_some(offset))
        .collect::<Vec<_>>();
    assert_eq!(offsets.len(), 1, "fixture payload must occur exactly once");
    offsets[0]
}

proptest! {
    #![proptest_config(property_config())]

    #[test]
    fn every_generated_envelope_or_domain_corruption_is_rejected(
        corruption in 0_u8..8,
        selector in any::<usize>(),
        mask in 1_u8..=u8::MAX,
    ) {
        let original = replay().to_vec();
        let original_report = verify_battle_replay(&original, battle()).unwrap();
        prop_assert_eq!(original_report.command_count(), COMMANDS as u32);
        let decoded = decode_replay(&original).unwrap();
        let mut corrupted = original.clone();

        match corruption {
            0 => {
                let keep = selector % corrupted.len();
                corrupted.truncate(keep);
            }
            1 => corrupted.push(mask),
            2 => {
                let offset = unique_offset(&corrupted, &CATALOG_DIGEST);
                corrupted[offset + selector % CATALOG_DIGEST.len()] ^= mask;
            }
            3 | 4 => {
                let parity = usize::from(corruption - 3);
                let records = decoded.records().iter().skip(parity).step_by(2).collect::<Vec<_>>();
                let record = records[selector % records.len()];
                let offset = unique_offset(&corrupted, record.payload());
                corrupted[offset + selector % record.payload().len()] ^= mask;
            }
            5 => {
                let record = decoded.records().iter().find(|record| {
                    record.kind() == RecordKind::AcceptedBattleCommand
                }).unwrap();
                let payload = record.payload();
                let offset = unique_offset(&corrupted, payload);
                corrupted[offset..offset + 2].copy_from_slice(&u16::MAX.to_le_bytes());
            }
            6 | 7 => {
                let record = &decoded.records()[selector % decoded.records().len()];
                let payload_offset = unique_offset(&corrupted, record.payload());
                let record_offset = payload_offset - 13;
                if corruption == 6 {
                    corrupted[record_offset] = 0xff;
                } else {
                    corrupted[record_offset + 1..record_offset + 9]
                        .copy_from_slice(&u64::MAX.to_le_bytes());
                }
            }
            _ => unreachable!(),
        }
        prop_assert!(verify_battle_replay(&corrupted, battle()).is_err());
    }
}
